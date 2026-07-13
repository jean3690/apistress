use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use super::plan::KeyValuePair;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleResult {
    pub id: String,
    pub timestamp: i64,
    pub thread_name: String,
    pub sampler_name: String,
    pub label: String,
    pub elapsed: u64,
    pub connect_time: u64,
    pub latency: u64,
    pub bytes: u64,
    pub sent_bytes: u64,
    pub response_code: String,
    pub response_message: String,
    pub success: bool,
    pub url: String,
    pub method: String,
    pub request_headers: Vec<KeyValuePair>,
    pub response_headers: Vec<KeyValuePair>,
    pub response_body: String,
    pub assertion_results: Vec<AssertionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub thread_group: String,
    pub group_threads: u32,
    pub all_threads: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertionResult {
    pub name: String,
    pub failure: bool,
    pub failure_message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub status: String,
    pub threads_active: u32,
    pub total_samples: u32,
    pub error_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_response_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertionEvalResult {
    pub metric: String,
    pub operator: String,
    pub expected: f64,
    pub actual: f64,
    pub passed: bool,
}

impl SampleResult {
    /// Create a base result pre-filled with context fields.
    #[must_use]
    pub fn new(name: &str, ctx: &ExecutionContext) -> Self {
        SampleResult {
            id: new_sample_id(),
            timestamp: now_ms(),
            thread_name: ctx.thread_name.clone(),
            sampler_name: name.to_string(),
            label: name.to_string(),
            thread_group: ctx.tg_name.clone(),
            group_threads: ctx.group_threads,
            all_threads: ctx.all_threads,
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
        }
    }

    /// Create a result for a skipped (disabled) sampler.
    #[must_use]
    pub fn skipped(name: &str, ctx: &ExecutionContext) -> Self {
        SampleResult {
            id: String::new(),
            ..SampleResult::new(name, ctx)
        }
    }
}

pub fn new_sample_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub struct ExecutionContext {
    pub thread_name: String,
    pub tg_name: String,
    pub group_threads: u32,
    pub all_threads: u32,
}
