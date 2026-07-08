use std::collections::HashMap;
use std::time::Instant;

use reqwest::Client;
use tauri::AppHandle;
use tauri::Emitter;

use super::plan::{HttpBody, HttpSampler};
use super::result::{ExecutionContext, SampleResult};

pub async fn execute_sampler(
    client: &Client,
    sampler: &HttpSampler,
    variables: &HashMap<String, String>,
    ctx: &ExecutionContext,
    app_handle: &AppHandle,
) -> SampleResult {
    if !sampler.enabled {
        return skipped_result(sampler, ctx);
    }

    let url = build_url(sampler, variables);
    let method = sampler.method.to_uppercase();
    let method_req = reqwest::Method::from_bytes(method.as_bytes())
        .unwrap_or(reqwest::Method::GET);

    let mut req = client.request(method_req.clone(), &url);

    // Headers
    for h in &sampler.headers {
        if !h.key.is_empty() {
            req = req.header(&h.key, interpolate(&h.value, variables));
        }
    }

    // Auth
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

    // Body
    let sent_bytes = apply_body(&mut req, &sampler.body, variables);

    // Timeout
    req = req.timeout(std::time::Duration::from_millis(sampler.timeout));

    let start = Instant::now();
    let resp_result = req.send().await;
    let elapsed = start.elapsed().as_millis() as u64;

    match resp_result {
        Ok(resp) => {
            let status = resp.status();
            let status_code = status.as_u16();
            let resp_headers: Vec<super::plan::KeyValuePair> = resp
                .headers()
                .iter()
                .map(|(k, v)| super::plan::KeyValuePair {
                    key: k.as_str().to_string(),
                    value: v.to_str().unwrap_or("").to_string(),
                })
                .collect();

            let body_bytes = resp.bytes().await.unwrap_or_default();
            let body_len = body_bytes.len() as u64;
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();

            let result = SampleResult {
                id: super::result::new_sample_id(),
                timestamp: super::result::now_ms(),
                thread_name: ctx.thread_name.clone(),
                sampler_name: sampler.name.clone(),
                label: sampler.name.clone(),
                elapsed,
                connect_time: 0,
                latency: elapsed,
                bytes: body_len,
                sent_bytes,
                response_code: status_code.to_string(),
                response_message: status
                    .canonical_reason()
                    .unwrap_or("")
                    .to_string(),
                success: !status.is_server_error() && !status.is_client_error(),
                url: url.clone(),
                method: method.clone(),
                request_headers: sampler.headers.clone(),
                response_headers: resp_headers,
                response_body: body_str,
                assertion_results: vec![],
                error_message: None,
                thread_group: ctx.tg_name.clone(),
                group_threads: ctx.group_threads,
                all_threads: ctx.all_threads,
            };

            let _ = app_handle.emit("test://result", &result);
            result
        }
        Err(e) => {
            let is_timeout = e.is_timeout();
            let msg = if is_timeout {
                format!("Request timeout after {}ms", sampler.timeout)
            } else {
                e.to_string()
            };

            let result = SampleResult {
                id: super::result::new_sample_id(),
                timestamp: super::result::now_ms(),
                thread_name: ctx.thread_name.clone(),
                sampler_name: sampler.name.clone(),
                label: sampler.name.clone(),
                elapsed,
                connect_time: 0,
                latency: 0,
                bytes: 0,
                sent_bytes,
                response_code: "0".to_string(),
                response_message: msg.clone(),
                success: false,
                url: url.clone(),
                method: method.clone(),
                request_headers: sampler.headers.clone(),
                response_headers: vec![],
                response_body: String::new(),
                assertion_results: vec![],
                error_message: Some(msg),
                thread_group: ctx.tg_name.clone(),
                group_threads: ctx.group_threads,
                all_threads: ctx.all_threads,
            };

            let _ = app_handle.emit("test://result", &result);
            result
        }
    }
}

fn skipped_result(sampler: &HttpSampler, ctx: &ExecutionContext) -> SampleResult {
    SampleResult {
        id: String::new(),
        timestamp: super::result::now_ms(),
        thread_name: ctx.thread_name.clone(),
        sampler_name: sampler.name.clone(),
        label: sampler.name.clone(),
        elapsed: 0,
        connect_time: 0,
        latency: 0,
        bytes: 0,
        sent_bytes: 0,
        response_code: String::new(),
        response_message: String::new(),
        success: true,
        url: String::new(),
        method: String::new(),
        request_headers: vec![],
        response_headers: vec![],
        response_body: String::new(),
        assertion_results: vec![],
        error_message: None,
        thread_group: ctx.tg_name.clone(),
        group_threads: ctx.group_threads,
        all_threads: ctx.all_threads,
    }
}

fn apply_body(
    req: &mut reqwest::RequestBuilder,
    body: &HttpBody,
    variables: &HashMap<String, String>,
) -> u64 {
    match body.mode.as_str() {
        "raw" => {
            let raw = body.raw.as_deref().unwrap_or("");
            let content_type = body.content_type.as_deref().unwrap_or("text/plain");
            let interpolated = interpolate(raw, variables);
            let len = interpolated.len() as u64;
            *req = req
                .try_clone()
                .unwrap()
                .header("Content-Type", content_type)
                .body(interpolated);
            len
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
                *req = req
                    .try_clone()
                    .unwrap()
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(body_str);
                len
            } else {
                0
            }
        }
        _ => 0,
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

    format!("{}://{}{}{}{}", protocol, domain, port_str, path, query_string)
}

pub fn interpolate(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    // Built-in functions
    result = result.replace("${__threadNum}", &variables.get("__threadNum").cloned().unwrap_or_default());
    result = result.replace(
        "${__time()}",
        &chrono::Utc::now().timestamp_millis().to_string(),
    );
    result
}

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((triple >> 6) & 0x3F) as usize] } else { b'=' } as char);
        result.push(if chunk.len() > 2 { CHARS[(triple & 0x3F) as usize] } else { b'=' } as char);
    }
    result
}

fn urlencoding(s: &str) -> String {
    // Simple percent-encoding for common characters
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace('+', "%2B")
}
