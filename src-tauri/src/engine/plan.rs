// Fields on these structs are deserialized from JSON and may not be
// directly read in Rust code (used only through serde).
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpAuth {
    #[serde(rename = "type", default)]
    pub auth_type: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormDataItem {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpBody {
    pub mode: String,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub form_data: Option<Vec<FormDataItem>>,
    #[serde(default)]
    pub url_encoded: Option<Vec<KeyValuePair>>,
}

// ---- Samplers ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default)]
    pub query_params: Vec<KeyValuePair>,
    #[serde(default)]
    pub body: HttpBody,
    #[serde(default)]
    pub auth: HttpAuth,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_true")]
    pub use_keep_alive: bool,
}

// ---- GraphQL Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQlSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub variables: String,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

// ---- SSE Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SseSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default)]
    pub max_events: u32,
}

// ---- MQTT Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MqttSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub broker_url: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default = "default_qos")]
    pub qos: u8,
    #[serde(default)]
    pub message: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default)]
    pub mode: String,
}

// ---- WebSocket Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default)]
    pub message: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default)]
    pub mode: String,
}

// ---- gRPC Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub service_name: String,
    #[serde(default)]
    pub method_name: String,
    #[serde(default)]
    pub request_json: String,
    #[serde(default)]
    pub metadata: Vec<KeyValuePair>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_true")]
    pub use_tls: bool,
}

// ---- TCP Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub payload: String,
    #[serde(default)]
    pub payload_type: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_eol")]
    pub eol_byte: u8,
}

// ---- Redis Sampler ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisSampler {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

// ---- Controllers ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopController {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub loops: i32,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfController {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub condition: String,
    #[serde(default = "default_true")]
    pub use_expression: bool,
    #[serde(default)]
    pub evaluate_all: bool,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhileController {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionController {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub include_duration: bool,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThroughputController {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub throughput: u32,
    #[serde(default)]
    pub per_thread: bool,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

// ---- Assertions ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseAssertion {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub test_field: String,
    #[serde(default)]
    pub pattern_matching: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub assume_success: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonAssertion {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub json_path: String,
    #[serde(default)]
    pub expected_value: String,
    #[serde(default)]
    pub comparison_mode: String,
    #[serde(default)]
    pub expect_null: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurationAssertion {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_duration")]
    pub max_duration: u64,
}

// ---- Timers ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstantTimer {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_timer_delay")]
    pub delay: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniformRandomTimer {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub min_delay: u64,
    #[serde(default = "default_timer_delay")]
    pub max_delay: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaussianRandomTimer {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_timer_delay")]
    pub delay: u64,
    #[serde(default)]
    pub deviation: u64,
}

// ---- Processors (Extractors) ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegexExtractor {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub reference_name: String,
    #[serde(default)]
    pub regex: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub match_no: u32,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub use_headers: bool,
    #[serde(default = "default_true")]
    pub use_body: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonExtractor {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub reference_name: String,
    #[serde(default)]
    pub json_path: String,
    #[serde(default)]
    pub default_value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryExtractor {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub reference_name: String,
    #[serde(default)]
    pub left_boundary: String,
    #[serde(default)]
    pub right_boundary: String,
    #[serde(default)]
    pub match_no: u32,
    #[serde(default)]
    pub default_value: String,
}

// ---- Config elements ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpDefaults {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvDataSet {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub variable_names: String,
    #[serde(default)]
    pub delimiter: String,
    #[serde(default = "default_true")]
    pub recycle_on_eof: bool,
    #[serde(default)]
    pub stop_thread_on_eof: bool,
    #[serde(default = "default_true")]
    pub ignore_first_line: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVariables {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub variables: Vec<KeyValuePair>,
}

// ---- UserParameters (preprocessor) ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserParameters {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub parameters: Vec<KeyValuePair>,
}

// ---- Listener (ignored during execution but present in JSON) ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenerConfig {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "type")]
    pub listener_type: String,
}

// ---- Tagged union for all child elements ----

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ChildElement {
    HttpSampler(Box<HttpSampler>),
    GraphQlSampler(Box<GraphQlSampler>),
    SseSampler(Box<SseSampler>),
    MqttSampler(Box<MqttSampler>),
    WebSocketSampler(Box<WebSocketSampler>),
    GrpcSampler(Box<GrpcSampler>),
    TcpSampler(Box<TcpSampler>),
    RedisSampler(Box<RedisSampler>),
    LoopController(LoopController),
    IfController(IfController),
    WhileController(WhileController),
    TransactionController(TransactionController),
    ThroughputController(ThroughputController),
    ConstantTimer(ConstantTimer),
    UniformRandomTimer(UniformRandomTimer),
    GaussianRandomTimer(GaussianRandomTimer),
    ResponseAssertion(ResponseAssertion),
    JsonAssertion(JsonAssertion),
    DurationAssertion(DurationAssertion),
    RegexExtractor(RegexExtractor),
    JsonExtractor(JsonExtractor),
    BoundaryExtractor(BoundaryExtractor),
    HttpDefaults(HttpDefaults),
    CsvDataSet(CsvDataSet),
    UserVariables(UserVariables),
    UserParameters(UserParameters),
    ViewResultsTree(ListenerConfig),
    SummaryReport(ListenerConfig),
    AggregateReport(ListenerConfig),
    AggregateGraph(ListenerConfig),
    ResponseTimeGraph(ListenerConfig),
    GraphResults(ListenerConfig),
}

// ---- ThreadGroup ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadGroup {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_num_threads")]
    pub num_threads: u32,
    #[serde(default)]
    pub ramp_up: u32,
    #[serde(default)]
    pub warm_up: u32,
    #[serde(default)]
    pub loops: i32,
    #[serde(default)]
    pub duration: u32,
    #[serde(default)]
    pub delay: u32,
    #[serde(default)]
    pub scheduler: bool,
    #[serde(default)]
    pub on_error_action: String,
    #[serde(default = "default_true")]
    pub same_user_on_each_iteration: bool,
    #[serde(default)]
    pub children: Vec<ChildElement>,
}

// ---- TestPlan ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPlanAssertion {
    pub metric: String,
    pub operator: String,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPlan {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub comments: String,
    #[serde(default)]
    pub thread_groups: Vec<ThreadGroup>,
    #[serde(default)]
    pub variables: Vec<KeyValuePair>,
    #[serde(default)]
    pub listeners: Vec<ListenerConfig>,
    #[serde(default)]
    pub assertions: Vec<TestPlanAssertion>,
}

// ---- Default value helpers ----

fn default_true() -> bool {
    true
}
fn default_protocol() -> String {
    "https".into()
}
fn default_port() -> u16 {
    443
}
fn default_path() -> String {
    "/".into()
}
fn default_method() -> String {
    "GET".into()
}
fn default_timeout() -> u64 {
    30000
}
fn default_max_duration() -> u64 {
    3000
}
fn default_timer_delay() -> u64 {
    300
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_qos() -> u8 {
    0
}
fn default_eol() -> u8 {
    10
}
fn default_redis_port() -> u16 {
    6379
}
fn default_num_threads() -> u32 {
    10
}
