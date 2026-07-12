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
    let method_req = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);

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
                response_message: status.canonical_reason().unwrap_or("").to_string(),
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
                .expect("Failed to clone request builder")
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
                                        mime,
                                        item.key
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
                *req = req.try_clone().unwrap().multipart(form);
                total_len
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
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    // Built-in functions
    result = result.replace(
        "${__threadNum}",
        &variables.get("__threadNum").cloned().unwrap_or_default(),
    );
    result = result.replace(
        "${__time()}",
        &chrono::Utc::now().timestamp_millis().to_string(),
    );
    result
}

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

fn urlencoding(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}
