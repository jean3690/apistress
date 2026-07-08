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
