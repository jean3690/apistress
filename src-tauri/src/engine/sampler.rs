use std::collections::HashMap;
use std::time::Instant;

use reqwest::Client;

use super::plan::{GraphQlSampler, GrpcSampler, HttpBody, HttpSampler, KeyValuePair, MqttSampler, RedisSampler, SseSampler, TcpSampler, WebSocketSampler};
use super::result::{new_sample_id, ExecutionContext, SampleResult};
use super::runner;

pub async fn execute_sampler(
    client: &Client,
    sampler: &HttpSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let url = build_url(sampler, variables);
    let method = sampler.method.to_uppercase();
    let method_req = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);

    let mut req = client.request(method_req.clone(), &url);

    for h in &sampler.headers {
        if !h.key.is_empty() {
            req = req.header(&h.key, interpolate(&h.value, variables));
        }
    }

    match sampler.auth.auth_type.as_str() {
        "basic" => {
            let user = sampler.auth.username.as_deref().unwrap_or("");
            let pass = sampler.auth.password.as_deref().unwrap_or("");
            let creds = base64_encode(&format!("{}:{}", user, pass));
            req = req.header("Authorization", format!("Basic {}", creds));
        }
        "bearer" => {
            let token = sampler.auth.token.as_deref().unwrap_or("");
            if !token.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }
        _ => {}
    }

    let (req2, sent_bytes) = apply_body(req, &sampler.body, variables);
    req = req2;

    req = req.timeout(std::time::Duration::from_millis(sampler.timeout));

    let start = Instant::now();
    let resp_result = req.send().await;
    let elapsed = start.elapsed().as_millis() as u64;

    match resp_result {
        Ok(resp) => {
            let status = resp.status();
            let status_code = status.as_u16();
            let resp_headers: Vec<KeyValuePair> = resp
                .headers()
                .iter()
                .map(|(k, v)| KeyValuePair {
                    key: k.as_str().to_string(),
                    value: v.to_str().unwrap_or("").to_string(),
                })
                .collect();

            let body_bytes = resp.bytes().await.unwrap_or_default();
            let body_len = body_bytes.len() as u64;
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();

            let sample_id = new_sample_id();
            runner::store_response_body(&sample_id, body_str);

            SampleResult {
                id: sample_id,
                elapsed,
                connect_time: 0,
                latency: elapsed,
                bytes: body_len,
                sent_bytes,
                response_code: status_code.to_string(),
                response_message: status.canonical_reason().unwrap_or("").to_string(),
                success: !status.is_server_error() && !status.is_client_error(),
                url,
                method,
                request_headers: sampler.headers.clone(),
                response_headers: resp_headers,
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Err(e) => {
            let is_timeout = e.is_timeout();
            let msg = if is_timeout {
                format!("Request timeout after {}ms", sampler.timeout)
            } else {
                e.to_string()
            };
            SampleResult {
                elapsed,
                sent_bytes,
                response_code: "0".into(),
                response_message: msg.clone(),
                success: false,
                url,
                method,
                request_headers: sampler.headers.clone(),
                error_message: Some(msg),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
    }
}

fn apply_body(
    req: reqwest::RequestBuilder,
    body: &HttpBody,
    variables: &HashMap<String, String>,
) -> (reqwest::RequestBuilder, u64) {
    match body.mode.as_str() {
        "raw" => {
            let raw = body.raw.as_deref().unwrap_or("");
            let content_type = body.content_type.as_deref().unwrap_or("text/plain");
            let interpolated = interpolate(raw, variables);
            let len = interpolated.len() as u64;
            (req.header("Content-Type", content_type).body(interpolated), len)
        }
        "x-www-form-urlencoded" => {
            if let Some(params) = &body.url_encoded {
                let kv: Vec<String> = params
                    .iter()
                    .filter(|p| !p.key.is_empty())
                    .map(|p| {
                        format!(
                            "{}={}",
                            urlencoding(&interpolate(&p.key, variables)),
                            urlencoding(&interpolate(&p.value, variables))
                        )
                    })
                    .collect();
                let body_str = kv.join("&");
                let len = body_str.len() as u64;
                (
                    req.header("Content-Type", "application/x-www-form-urlencoded")
                        .body(body_str),
                    len,
                )
            } else {
                (req, 0)
            }
        }
        "form-data" => {
            if let Some(form_data) = &body.form_data {
                let mut form = reqwest::multipart::Form::new();
                let mut total_len = 0u64;
                for item in form_data {
                    match item.item_type.as_str() {
                        "file" => {
                            let file_bytes = std::fs::read(&item.value).unwrap_or_default();
                            total_len += file_bytes.len() as u64;
                            let filename = item.filename.as_deref().unwrap_or("file");
                            let mime = item
                                .mime_type
                                .as_deref()
                                .unwrap_or("application/octet-stream");
                            let part = reqwest::multipart::Part::bytes(file_bytes)
                                .file_name(filename.to_string())
                                .mime_str(mime)
                                .unwrap_or_else(|_| {
                                    eprintln!(
                                        "Warning: invalid MIME type '{}' for field '{}', using default",
                                        mime, item.key
                                    );
                                    reqwest::multipart::Part::bytes(vec![])
                                        .file_name(filename.to_string())
                                });
                            form = form.part(item.key.clone(), part);
                        }
                        _ => {
                            let value = interpolate(&item.value, variables);
                            total_len += value.len() as u64;
                            form = form.text(item.key.clone(), value);
                        }
                    }
                }
                (req.multipart(form), total_len)
            } else {
                (req, 0)
            }
        }
        _ => (req, 0),
    }
}

pub fn build_url(sampler: &HttpSampler, variables: &HashMap<String, String>) -> String {
    let protocol = if sampler.protocol.is_empty() {
        "https"
    } else {
        &sampler.protocol
    };
    let domain = interpolate(&sampler.domain, variables);
    let path = if sampler.path.starts_with('/') {
        sampler.path.clone()
    } else {
        format!("/{}", sampler.path)
    };
    let path = interpolate(&path, variables);

    let port_str = if (protocol == "https" && sampler.port == 443)
        || (protocol == "http" && sampler.port == 80)
        || sampler.port == 0
    {
        String::new()
    } else {
        format!(":{}", sampler.port)
    };

    // Query params
    let query_string = if sampler.query_params.is_empty() {
        String::new()
    } else {
        let params: Vec<String> = sampler
            .query_params
            .iter()
            .filter(|p| !p.key.is_empty())
            .map(|p| {
                format!(
                    "{}={}",
                    urlencoding(&interpolate(&p.key, variables)),
                    urlencoding(&interpolate(&p.value, variables))
                )
            })
            .collect();
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    };

    format!(
        "{}://{}{}{}{}",
        protocol, domain, port_str, path, query_string
    )
}

/// Collect HttpDefaults from a list of children (non-recursive at this level;
/// callers should recurse into controller children as needed).
#[must_use]
pub fn collect_defaults(children: &[super::plan::ChildElement]) -> Vec<super::plan::HttpDefaults> {
    let mut defaults = Vec::new();
    for child in children {
        if let super::plan::ChildElement::HttpDefaults(d) = child {
            if d.enabled {
                defaults.push(d.clone());
            }
        }
    }
    defaults
}

/// Apply HttpDefaults to a sampler, filling only fields that are empty/default.
pub fn apply_defaults(
    sampler: &mut super::plan::HttpSampler,
    defaults: &[super::plan::HttpDefaults],
) {
    for d in defaults {
        if !d.enabled {
            continue;
        }
        if (sampler.protocol.is_empty() || sampler.protocol == "https") && !d.protocol.is_empty() {
            sampler.protocol = d.protocol.clone();
        }
        if sampler.domain.is_empty() && !d.domain.is_empty() {
            sampler.domain = d.domain.clone();
        }
        if sampler.port == 443 && d.port != 443 && d.port != 0 {
            sampler.port = d.port;
        }
        if (sampler.path.is_empty() || sampler.path == "/") && !d.path.is_empty() && d.path != "/" {
            sampler.path = d.path.clone();
        }
        // Merge headers: defaults first, then sampler-specific headers
        for h in &d.headers {
            if !h.key.is_empty()
                && !sampler
                    .headers
                    .iter()
                    .any(|sh| sh.key.eq_ignore_ascii_case(&h.key))
            {
                sampler.headers.push(h.clone());
            }
        }
    }
}

#[must_use]
pub fn interpolate(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let s = template; // alias
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if let Some(after_dollar) = rest.strip_prefix("${") {
            if let Some(end) = after_dollar.find('}') {
                let inner = &after_dollar[..end];
                match inner {
                    "__threadNum" => {
                        if let Some(v) = variables.get("__threadNum") {
                            result.push_str(v);
                        }
                    }
                    "__time()" => {
                        use std::fmt::Write;
                        let _ = write!(result, "{}", chrono::Utc::now().timestamp_millis());
                    }
                    _ => {}
                }
                i += 3 + end; // skip ${...}
                continue;
            }
        } else if let Some(after_braces) = rest.strip_prefix("{{") {
            if let Some(end) = after_braces.find("}}") {
                let key = &after_braces[..end];
                if let Some(value) = variables.get(key) {
                    result.push_str(value);
                }
                i += 4 + end; // skip {{...}}
                continue;
            }
        }
        // Push current char
        if let Some(ch) = s[i..].chars().next() {
            result.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    result
}

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

fn urlencoding(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

// ---- GraphQL Sampler Execution ----

pub async fn execute_graphql_sampler(
    client: &Client,
    sampler: &GraphQlSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let url = interpolate(&sampler.url, variables);
    let query = interpolate(&sampler.query, variables);
    let vars_str = interpolate(&sampler.variables, variables);

    let mut body_map = serde_json::Map::new();
    body_map.insert("query".into(), serde_json::Value::String(query));
    if !vars_str.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&vars_str) {
            body_map.insert("variables".into(), parsed);
        }
    }
    let request_body = serde_json::json!(&body_map).to_string();

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(request_body.clone());
    for h in &sampler.headers {
        if !h.key.is_empty() {
            req = req.header(&h.key, interpolate(&h.value, variables));
        }
    }
    req = req.timeout(std::time::Duration::from_millis(sampler.timeout));

    let start = Instant::now();
    let resp_result = req.send().await;
    let elapsed = start.elapsed().as_millis() as u64;

    match resp_result {
        Ok(resp) => {
            let status = resp.status();
            let status_code = status.as_u16();
            let resp_headers: Vec<KeyValuePair> = resp
                .headers()
                .iter()
                .map(|(k, v)| KeyValuePair {
                    key: k.as_str().to_string(),
                    value: v.to_str().unwrap_or("").to_string(),
                })
                .collect();
            let body_bytes = resp.bytes().await.unwrap_or_default();
            let body_len = body_bytes.len() as u64;
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();

            let sample_id = new_sample_id();
            runner::store_response_body(&sample_id, body_str);

            SampleResult {
                id: sample_id,
                elapsed,
                connect_time: 0,
                latency: elapsed,
                bytes: body_len,
                sent_bytes: request_body.len() as u64,
                response_code: status_code.to_string(),
                response_message: status.canonical_reason().unwrap_or("").to_string(),
                success: !status.is_server_error() && !status.is_client_error(),
                url,
                method: "POST".into(),
                request_headers: sampler.headers.clone(),
                response_headers: resp_headers,
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Err(e) => {
            let is_timeout = e.is_timeout();
            let msg = if is_timeout {
                format!("Request timeout after {}ms", sampler.timeout)
            } else {
                e.to_string()
            };
            SampleResult {
                elapsed,
                sent_bytes: request_body.len() as u64,
                response_code: "0".into(),
                response_message: msg.clone(),
                success: false,
                url,
                method: "POST".into(),
                request_headers: sampler.headers.clone(),
                error_message: Some(msg),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
    }
}

// ---- SSE Sampler Execution ----

pub async fn execute_sse_sampler(
    client: &Client,
    sampler: &SseSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let url = interpolate(&sampler.url, variables);
    let mut req = client.get(&url);
    for h in &sampler.headers {
        if !h.key.is_empty() {
            req = req.header(&h.key, interpolate(&h.value, variables));
        }
    }
    req = req.timeout(std::time::Duration::from_millis(sampler.timeout));

    let overall_start = Instant::now();
    let resp_result = req.send().await;

    match resp_result {
        Ok(mut resp) => {
            let status = resp.status();
            if !status.is_success() {
                return SampleResult {
                    elapsed: overall_start.elapsed().as_millis() as u64,
                    response_code: status.as_u16().to_string(),
                    response_message: status.canonical_reason().unwrap_or("").to_string(),
                    success: false,
                    url,
                    method: "GET".into(),
                    request_headers: sampler.headers.clone(),
                    error_message: Some(format!("SSE endpoint returned {}", status.as_u16())),
                    ..SampleResult::new(&sampler.name, ctx)
                };
            }

            let mut event_count = 0u32;
            let mut last_event_data = String::new();
            let mut first_event_time: Option<u64> = None;
            let max_events = if sampler.max_events == 0 {
                u32::MAX
            } else {
                sampler.max_events
            };
            let mut buffer = String::new();

            loop {
                match resp.chunk().await {
                    Ok(Some(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if let Some(after_prefix) = line.strip_prefix("data:") {
                                let data = after_prefix.trim().to_string();
                                if first_event_time.is_none() {
                                    first_event_time =
                                        Some(overall_start.elapsed().as_millis() as u64);
                                }
                                last_event_data = data;
                                event_count += 1;
                                if event_count >= max_events {
                                    break;
                                }
                            }
                        }
                        if event_count >= max_events {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }

            let elapsed = overall_start.elapsed().as_millis() as u64;
            let first_event_ms = first_event_time.unwrap_or(0);

            let sample_id = new_sample_id();
            if !last_event_data.is_empty() {
                runner::store_response_body(&sample_id, last_event_data.clone());
            }

            SampleResult {
                id: sample_id,
                elapsed,
                latency: first_event_ms,
                bytes: last_event_data.len() as u64,
                response_code: event_count.to_string(),
                response_message: if event_count > 0 {
                    "SSE stream completed".into()
                } else {
                    "No events received".into()
                },
                success: event_count > 0,
                url,
                method: "GET".into(),
                request_headers: sampler.headers.clone(),
                error_message: if event_count == 0 {
                    Some("No SSE events received".into())
                } else {
                    None
                },
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Err(e) => {
            let is_timeout = e.is_timeout();
            let msg = if is_timeout {
                format!("Request timeout after {}ms", sampler.timeout)
            } else {
                e.to_string()
            };
            SampleResult {
                elapsed: overall_start.elapsed().as_millis() as u64,
                response_code: "0".into(),
                response_message: msg.clone(),
                success: false,
                url,
                method: "GET".into(),
                request_headers: sampler.headers.clone(),
                error_message: Some(msg),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
    }
}

// ---- MQTT Sampler Execution ----

pub async fn execute_mqtt_sampler(
    sampler: &MqttSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let broker_url = interpolate(&sampler.broker_url, variables);
    let client_id = interpolate(&sampler.client_id, variables);
    let topic = interpolate(&sampler.topic, variables);
    let payload = interpolate(&sampler.message, variables);

    let (host, port) = parse_broker_url(&broker_url);

    use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
    use tokio::time::Duration;

    let mut mqttoptions = MqttOptions::new(&client_id, &host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_clean_session(true);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);
    let overall_start = Instant::now();

    // Initial poll to establish connection
    let conn_result =
        tokio::time::timeout(Duration::from_millis(sampler.timeout), eventloop.poll()).await;
    let connect_time = overall_start.elapsed().as_millis() as u64;

    if conn_result.is_err()
        || conn_result
            .as_ref()
            .ok()
            .map(|r| r.is_err())
            .unwrap_or(true)
    {
        return SampleResult {
            id: new_sample_id(),
            elapsed: connect_time,
            connect_time,
            sent_bytes: payload.len() as u64,
            response_code: "1".into(),
            response_message: "MQTT connection failed".into(),
            success: false,
            url: broker_url,
            error_message: Some("MQTT connection failed or timed out".into()),
            ..SampleResult::new(&sampler.name, ctx)
        };
    }

    let qos = match sampler.qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
    };

    // Subscribe if pubsub mode
    if sampler.mode == "pubsub" {
        let _ = client.subscribe(&topic, qos).await;
        let _ =
            tokio::time::timeout(Duration::from_millis(5000), eventloop.poll()).await;
    }

    // Publish
    let pub_start = Instant::now();
    let _ = client
        .publish(&topic, qos, false, payload.as_bytes())
        .await;
    let _ =
        tokio::time::timeout(Duration::from_millis(5000), eventloop.poll()).await;
    let publish_elapsed = pub_start.elapsed().as_millis() as u64;

    // If pubsub mode, wait for incoming publish
    let mut received = String::new();
    if sampler.mode == "pubsub" {
        let _recv_start = Instant::now();
        loop {
            let poll_result = tokio::time::timeout(
                Duration::from_millis(sampler.timeout),
                eventloop.poll(),
            )
            .await;
            match poll_result {
                Ok(Ok(Event::Incoming(Packet::Publish(p)))) => {
                    received = String::from_utf8_lossy(&p.payload).to_string();
                    break;
                }
                Ok(Ok(_)) => continue,
                Ok(Err(_)) => break,
                Err(_) => break,
            }
        }
    }

    // Disconnect
    let _ = client.disconnect().await;

    let elapsed = overall_start.elapsed().as_millis() as u64;

    SampleResult {
        id: new_sample_id(),
        elapsed,
        connect_time,
        latency: publish_elapsed,
        bytes: received.len() as u64,
        sent_bytes: payload.len() as u64,
        response_code: "0".into(),
        response_message: format!("MQTT published to {}", topic),
        success: true,
        url: broker_url,
        response_body: received,
        ..SampleResult::new(&sampler.name, ctx)
    }
}

fn parse_broker_url(url: &str) -> (String, u16) {
    let without_scheme = url
        .trim_start_matches("tcp://")
        .trim_start_matches("ssl://")
        .trim_start_matches("mqtt://");
    if let Some(colon_pos) = without_scheme.rfind(':') {
        let host = without_scheme[..colon_pos].to_string();
        let port: u16 = without_scheme[colon_pos + 1..].parse().unwrap_or(1883);
        (host, port)
    } else {
        (without_scheme.to_string(), 1883)
    }
}

// ---- WebSocket Sampler Execution ----

pub async fn execute_websocket_sampler(
    sampler: &WebSocketSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let url = interpolate(&sampler.url, variables);
    let message = interpolate(&sampler.message, variables);

    use tokio::time::Duration;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};

    let start = Instant::now();
    let connect_result =
        tokio::time::timeout(Duration::from_millis(sampler.timeout), connect_async(&url)).await;
    let connect_time = start.elapsed().as_millis() as u64;

    match connect_result {
        Ok(Ok((mut ws_stream, _resp))) => {
            let mut received = String::new();
            let mut send_time = 0u64;
            let mut recv_time = 0u64;

            if sampler.mode != "connect" && !message.is_empty() {
                let send_start = Instant::now();
                let _ = ws_stream.send(Message::Text(message.clone())).await;
                send_time = send_start.elapsed().as_millis() as u64;

                // Wait for response via stream
                let recv_start = Instant::now();
                match tokio::time::timeout(
                    Duration::from_millis(sampler.timeout),
                    ws_stream.next(),
                )
                .await
                {
                    Ok(Some(Ok(Message::Text(t)))) => {
                        received = t;
                        recv_time = recv_start.elapsed().as_millis() as u64;
                    }
                    Ok(Some(Ok(Message::Binary(b)))) => {
                        received = String::from_utf8_lossy(&b).to_string();
                        recv_time = recv_start.elapsed().as_millis() as u64;
                    }
                    Ok(Some(Ok(Message::Close(_)))) => {
                        received = "Connection closed".into();
                    }
                    _ => {
                        received = String::new();
                    }
                }
            }

            let _ = ws_stream.close(None).await;
            let elapsed = start.elapsed().as_millis() as u64;

            let sample_id = new_sample_id();
            if !received.is_empty() {
                runner::store_response_body(&sample_id, received.clone());
            }

            SampleResult {
                id: sample_id,
                elapsed,
                connect_time,
                latency: send_time + recv_time,
                bytes: received.len() as u64,
                sent_bytes: message.len() as u64,
                response_code: "0".into(),
                response_message: format!("WS {} received {} bytes", url, received.len()),
                success: true,
                url,
                method: "WS".into(),
                request_headers: sampler.headers.clone(),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Ok(Err(e)) => SampleResult {
            elapsed: connect_time,
            connect_time,
            response_code: "1".into(),
            response_message: e.to_string(),
            success: false,
            url,
            method: "WS".into(),
            request_headers: sampler.headers.clone(),
            error_message: Some(e.to_string()),
            ..SampleResult::new(&sampler.name, ctx)
        },
        Err(_) => SampleResult {
            elapsed: connect_time,
            connect_time,
            response_code: "1".into(),
            response_message: "WebSocket connection timeout".into(),
            success: false,
            url,
            method: "WS".into(),
            request_headers: sampler.headers.clone(),
            error_message: Some("Connection timeout".into()),
            ..SampleResult::new(&sampler.name, ctx)
        },
    }
}

// ---- gRPC Sampler Execution ----
// Uses reqwest HTTP/2 with manual gRPC wire framing.
// Payload is sent as raw bytes (JSON or protobuf binary — user's choice).
// Response is parsed as gRPC frame and returned as text.

pub async fn execute_grpc_sampler(
    sampler: &GrpcSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let endpoint = interpolate(&sampler.endpoint, variables);
    let request_json = interpolate(&sampler.request_json, variables);
    let service_name = interpolate(&sampler.service_name, variables);
    let method_name = interpolate(&sampler.method_name, variables);

    use tokio::time::Duration;

    // Build gRPC URL: {endpoint}/{ServiceName}/{MethodName}
    let url = format!(
        "{}/{}/{}",
        endpoint.trim_end_matches('/'),
        service_name.trim_start_matches('/'),
        method_name
    );

    // Build gRPC frame: 1 byte flag (0) + 4 bytes BE length + payload
    let pay_bytes = request_json.as_bytes();
    let mut frame = Vec::with_capacity(5 + pay_bytes.len());
    frame.push(0u8); // no compression
    frame.extend_from_slice(&(pay_bytes.len() as u32).to_be_bytes());
    frame.extend_from_slice(pay_bytes);

    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut req = client
        .post(&url)
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(frame)
        .timeout(Duration::from_millis(sampler.timeout));

    for h in &sampler.metadata {
        if !h.key.is_empty() {
            req = req.header(&h.key, interpolate(&h.value, variables));
        }
    }

    let start = Instant::now();
    let resp_result = req.send().await;
    let elapsed = start.elapsed().as_millis() as u64;

    match resp_result {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let resp_bytes = resp.bytes().await.unwrap_or_default();

            // Parse gRPC response frame: skip 5-byte header
            let resp_body = if resp_bytes.len() > 5 {
                let msg_len =
                    u32::from_be_bytes([resp_bytes[1], resp_bytes[2], resp_bytes[3], resp_bytes[4]])
                        as usize;
                if msg_len > 0 && resp_bytes.len() >= 5 + msg_len {
                    String::from_utf8_lossy(&resp_bytes[5..5 + msg_len]).to_string()
                } else {
                    String::from_utf8_lossy(&resp_bytes).to_string()
                }
            } else {
                String::new()
            };

            let sample_id = new_sample_id();
            if !resp_body.is_empty() {
                runner::store_response_body(&sample_id, resp_body.clone());
            }

            SampleResult {
                id: sample_id,
                elapsed,
                latency: elapsed,
                bytes: resp_bytes.len() as u64,
                sent_bytes: pay_bytes.len() as u64,
                response_code: status_code.to_string(),
                response_message: if status_code == 200 {
                    "gRPC OK".into()
                } else {
                    format!("gRPC status {}", status_code)
                },
                success: status_code == 200,
                url,
                method: "POST".into(),
                request_headers: sampler.metadata.clone(),
                error_message: if status_code != 200 {
                    Some(format!("gRPC returned status {}", status_code))
                } else {
                    None
                },
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Err(e) => {
            let is_timeout = e.is_timeout();
            let msg = if is_timeout {
                format!("gRPC timeout after {}ms", sampler.timeout)
            } else {
                e.to_string()
            };
            SampleResult {
                elapsed,
                sent_bytes: pay_bytes.len() as u64,
                response_code: "1".into(),
                response_message: msg.clone(),
                success: false,
                url,
                method: "POST".into(),
                request_headers: sampler.metadata.clone(),
                error_message: Some(msg),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
    }
}

// ---- TCP Sampler Execution ----

pub async fn execute_tcp_sampler(
    sampler: &TcpSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let host = interpolate(&sampler.host, variables);
    let payload_str = interpolate(&sampler.payload, variables);

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::Duration;

    let start = Instant::now();

    let stream_result = tokio::time::timeout(
        Duration::from_millis(sampler.timeout),
        TcpStream::connect(format!("{}:{}", host, sampler.port)),
    )
    .await;

    let connect_time = start.elapsed().as_millis() as u64;

    match stream_result {
        Ok(Ok(mut stream)) => {
            // Send payload
            let payload_bytes: Vec<u8> = if sampler.payload_type == "hex" {
                hex_decode(&payload_str)
            } else {
                payload_str.as_bytes().to_vec()
            };

            let send_start = Instant::now();
            if let Err(e) = stream.write_all(&payload_bytes).await {
                return tcp_error_result(sampler, ctx, connect_time, &format!("Send error: {}", e), start);
            }
            let _ = stream.flush().await;
            let send_time = send_start.elapsed().as_millis() as u64;

            // Read response until timeout or EOL byte
            let mut buf = vec![0u8; 4096];
            let mut received = Vec::new();
            let eol = sampler.eol_byte;

            loop {
                match tokio::time::timeout(
                    Duration::from_millis(std::cmp::max(sampler.timeout.saturating_sub(send_time), 100)),
                    stream.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break, // EOF
                    Ok(Ok(n)) => {
                        received.extend_from_slice(&buf[..n]);
                        if eol > 0 && buf[..n].contains(&eol) {
                            break;
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => break, // timeout
                }
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let response_str = String::from_utf8_lossy(&received).to_string();

            let sample_id = new_sample_id();
            if !response_str.is_empty() {
                runner::store_response_body(&sample_id, response_str.clone());
            }

            SampleResult {
                id: sample_id,
                elapsed,
                connect_time,
                latency: send_time,
                bytes: received.len() as u64,
                sent_bytes: payload_bytes.len() as u64,
                response_code: "0".into(),
                response_message: format!("TCP {} bytes sent, {} bytes received", payload_bytes.len(), received.len()),
                success: true,
                url: format!("{}:{}", host, sampler.port),
                method: "TCP".into(),
                ..SampleResult::new(&sampler.name, ctx)
            }
        }
        Ok(Err(e)) => tcp_error_result(sampler, ctx, connect_time, &format!("Connect error: {}", e), start),
        Err(_) => tcp_error_result(sampler, ctx, connect_time, "Connection timeout", start),
    }
}

fn tcp_error_result(
    sampler: &TcpSampler,
    ctx: &ExecutionContext,
    connect_time: u64,
    msg: &str,
    start: Instant,
) -> SampleResult {
    SampleResult {
        id: new_sample_id(),
        elapsed: start.elapsed().as_millis() as u64,
        connect_time,
        sent_bytes: sampler.payload.len() as u64,
        response_code: "1".into(),
        response_message: msg.to_string(),
        success: false,
        url: format!("{}:{}", sampler.host, sampler.port),
        method: "TCP".into(),
        error_message: Some(msg.to_string()),
        ..SampleResult::new(&sampler.name, ctx)
    }
}

fn hex_decode(hex: &str) -> Vec<u8> {
    let hex = hex.replace(|c: char| c.is_whitespace(), "");
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..std::cmp::min(i + 2, hex.len())], 16).ok())
        .collect()
}

// ---- Redis Sampler Execution ----

pub async fn execute_redis_sampler(
    sampler: &RedisSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
) -> SampleResult {
    if !sampler.enabled {
        return SampleResult::skipped(&sampler.name, ctx);
    }

    let host = interpolate(&sampler.host, variables);
    let password = interpolate(&sampler.password, variables);
    let command = interpolate(&sampler.command, variables);

    use tokio::time::Duration;

    let client_result = redis::Client::open(format!("redis://{}:{}/", host, sampler.port));
    let start = Instant::now();

    match client_result {
        Ok(client) => {
            let conn_result = tokio::time::timeout(
                Duration::from_millis(sampler.timeout),
                client.get_multiplexed_async_connection(),
            )
            .await;

            let connect_time = start.elapsed().as_millis() as u64;

            match conn_result {
                Ok(Ok(mut conn)) => {
                    // AUTH if password provided
                    if !password.is_empty() {
                        let auth_result = redis::cmd("AUTH")
                            .arg(&password)
                            .query_async::<String>(&mut conn)
                            .await;
                        if let Err(e) = auth_result {
                            return redis_error_result(sampler, ctx, connect_time, &format!("AUTH failed: {}", e), start);
                        }
                    }

                    // Execute command (parse first word as command, rest as args)
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if parts.is_empty() {
                        return redis_error_result(sampler, ctx, connect_time, "Empty command", start);
                    }

                    let cmd_start = Instant::now();
                    let mut redis_cmd = redis::cmd(parts[0]);
                    for arg in &parts[1..] {
                        redis_cmd.arg(*arg);
                    }

                    let result: redis::RedisResult<redis::Value> =
                        tokio::time::timeout(
                            Duration::from_millis(sampler.timeout.saturating_sub(connect_time)),
                            redis_cmd.query_async(&mut conn),
                        )
                        .await
                        .unwrap_or(Err(redis::RedisError::from((
                            redis::ErrorKind::IoError,
                            "Timeout",
                        ))));

                    let cmd_time = cmd_start.elapsed().as_millis() as u64;
                    let elapsed = start.elapsed().as_millis() as u64;

                    match result {
                        Ok(val) => {
                            let response_str = format_redis_value(&val);
                            let sample_id = new_sample_id();
                            if !response_str.is_empty() {
                                runner::store_response_body(&sample_id, response_str.clone());
                            }

                            SampleResult {
                                id: sample_id,
                                elapsed,
                                connect_time,
                                latency: cmd_time,
                                bytes: response_str.len() as u64,
                                sent_bytes: command.len() as u64,
                                response_code: "0".into(),
                                response_message: format!("Redis {} OK", parts[0].to_uppercase()),
                                success: true,
                                url: format!("redis://{}:{}/", host, sampler.port),
                                method: parts[0].to_uppercase(),
                                ..SampleResult::new(&sampler.name, ctx)
                            }
                        }
                        Err(e) => redis_error_result(sampler, ctx, connect_time, &e.to_string(), start),
                    }
                }
                Ok(Err(e)) => redis_error_result(sampler, ctx, connect_time, &e.to_string(), start),
                Err(_) => redis_error_result(sampler, ctx, connect_time, "Connection timeout", start),
            }
        }
        Err(e) => redis_error_result(sampler, ctx, 0, &format!("Invalid URL: {}", e), start),
    }
}

fn redis_error_result(
    sampler: &RedisSampler,
    ctx: &ExecutionContext,
    connect_time: u64,
    msg: &str,
    start: Instant,
) -> SampleResult {
    SampleResult {
        id: new_sample_id(),
        elapsed: start.elapsed().as_millis() as u64,
        connect_time,
        sent_bytes: sampler.command.len() as u64,
        response_code: "1".into(),
        response_message: msg.to_string(),
        success: false,
        url: format!("redis://{}:{}/", sampler.host, sampler.port),
        error_message: Some(msg.to_string()),
        ..SampleResult::new(&sampler.name, ctx)
    }
}

fn format_redis_value(val: &redis::Value) -> String {
    format!("{val:?}")
}
