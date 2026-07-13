use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::plan::{ChildElement, TestPlan};
use super::result::{AssertionEvalResult, SampleResult, StatusPayload};
use super::thread_group;

static CANCEL: OnceLock<Arc<AtomicBool>> = OnceLock::new();
static RESPONSE_BODIES: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();
static ELAPSED_BUFFER: OnceLock<Arc<Mutex<Vec<u64>>>> = OnceLock::new();

/// Called by each sampler execution to feed live metrics
pub fn push_elapsed(elapsed: u64) {
    if let Some(buf) = ELAPSED_BUFFER.get() {
        let mut guard = buf.lock().expect("mutex poisoned");
        if guard.len() > 5000 {
            guard.remove(0);
        }
        guard.push(elapsed);
    }
}

fn compute_percentiles() -> (u64, u64, u64, f64, u64) {
    let guard = match ELAPSED_BUFFER.get() {
        Some(buf) => buf.lock().expect("mutex poisoned"),
        None => return (0, 0, 0, 0.0, 0),
    };
    if guard.is_empty() {
        return (0, 0, 0, 0.0, 0);
    }
    let mut sorted = guard.clone();
    sorted.sort_unstable();
    let n = sorted.len();
    let avg = sorted.iter().sum::<u64>() / n as u64;
    let p50 = sorted[n * 50 / 100];
    let p90 = sorted[n * 90 / 100];
    let p99 = sorted[n * 99 / 100];
    let throughput = n as f64 / 0.25; // samples per 250ms window → req/s estimate
    (p50, p90, p99, throughput, avg)
}

fn reset_elapsed_buffer() {
    if let Some(buf) = ELAPSED_BUFFER.get() {
        buf.lock().expect("mutex poisoned").clear();
    }
}

fn reset_bodies() {
    if let Some(store) = RESPONSE_BODIES.get() {
        store.lock().expect("mutex poisoned").clear();
    }
}

/// Max response body size to store (bytes). Larger bodies are truncated.
const MAX_BODY_SIZE: usize = 50_000;

fn body_store() -> Arc<Mutex<HashMap<String, String>>> {
    RESPONSE_BODIES
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

/// Store a response body for later retrieval by sample ID.
/// Returns the (possibly truncated) body to include in the result for display.
pub fn store_response_body(id: &str, body: String) -> String {
    if body.is_empty() {
        return body;
    }
    let store = body_store();
    let mut map = store.lock().expect("mutex poisoned");
    if map.len() > 10_000 {
        map.clear(); // prevent unbounded memory growth, keep last 10K entries
    }
    let stored = if body.len() > MAX_BODY_SIZE {
        let mut truncated = body[..MAX_BODY_SIZE].to_string();
        truncated.push_str("\n...[truncated]");
        truncated
    } else {
        body
    };
    map.insert(id.to_string(), stored);
    // Return empty — caller sets body to empty in IPC payload
    String::new()
}

/// Retrieve a stored response body by sample ID.
pub fn get_response_body(id: &str) -> String {
    let store = body_store();
    let mut guard = store.lock().expect("mutex poisoned");
    guard.remove(id).unwrap_or_default()
}

fn cancel_token() -> Arc<AtomicBool> {
    CANCEL
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

fn reset_cancel() {
    if let Some(token) = CANCEL.get() {
        token.store(false, Ordering::Relaxed);
    }
}

/// Collect CsvDataSet configs from children, load CSV files, return rows of variable maps
fn load_csv_datasets(children: &[ChildElement]) -> Vec<Vec<(String, String)>> {
    let mut all_rows: Vec<Vec<(String, String)>> = Vec::new();
    for child in children {
        if let ChildElement::CsvDataSet(csv) = child {
            if !csv.enabled || csv.filename.is_empty() {
                continue;
            }
            let var_names: Vec<String> = csv
                .variable_names
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if var_names.is_empty() {
                continue;
            }
            if let Ok(mut rdr) = csv::ReaderBuilder::new()
                .delimiter(csv.delimiter.chars().next().unwrap_or(',') as u8)
                .has_headers(csv.ignore_first_line)
                .from_path(&csv.filename)
            {
                for record in rdr.records().flatten() {
                    let row: Vec<(String, String)> = var_names
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (name.clone(), record.get(i).unwrap_or("").to_string()))
                        .collect();
                    all_rows.push(row);
                }
            }
        }
        // Recurse into controller children
        match child {
            ChildElement::LoopController(c) => all_rows.extend(load_csv_datasets(&c.children)),
            ChildElement::IfController(c) => all_rows.extend(load_csv_datasets(&c.children)),
            ChildElement::WhileController(c) => all_rows.extend(load_csv_datasets(&c.children)),
            ChildElement::TransactionController(c) => {
                all_rows.extend(load_csv_datasets(&c.children))
            }
            ChildElement::ThroughputController(c) => {
                all_rows.extend(load_csv_datasets(&c.children))
            }
            _ => {}
        }
    }
    all_rows
}

/// Start the full test plan execution.
pub async fn start_test_plan(plan: TestPlan, app_handle: AppHandle) -> Result<(), String> {
    reset_cancel();
    reset_bodies();
    reset_elapsed_buffer();

    let client = Arc::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?,
    );

    let client_no_redirect = Arc::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(false)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?,
    );

    let total_samples = Arc::new(AtomicU32::new(0));
    let error_count = Arc::new(AtomicU32::new(0));
    let threads_active = Arc::new(AtomicU32::new(0));
    let total_threads: u32 = plan.thread_groups.iter().map(|tg| tg.num_threads).sum();
    let _elapsed_buffer: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::with_capacity(5000)));

    // Clone app_handle for the status task and for virtual users
    let status_handle = app_handle.clone();

    // Spawn status emitter task
    let status_cancel = cancel_token().clone();
    let status_samples = total_samples.clone();
    let status_errors = error_count.clone();
    let status_active = threads_active.clone();

    let status_task = tokio::spawn(async move {
        loop {
            if status_cancel.load(Ordering::Relaxed) {
                let payload = StatusPayload {
                    status: "completed".to_string(),
                    threads_active: 0,
                    total_samples: status_samples.load(Ordering::Relaxed),
                    error_count: status_errors.load(Ordering::Relaxed),
                    p50: None,
                    p90: None,
                    p99: None,
                    throughput: None,
                    avg_response_time: None,
                };
                let _ = status_handle.emit("test://status", &payload);
                break;
            }

            let (p50, p90, p99, throughput, avg) = compute_percentiles();
            let payload = StatusPayload {
                status: "running".to_string(),
                threads_active: status_active.load(Ordering::Relaxed),
                total_samples: status_samples.load(Ordering::Relaxed),
                error_count: status_errors.load(Ordering::Relaxed),
                p50: Some(p50),
                p90: Some(p90),
                p99: Some(p99),
                throughput: Some(throughput),
                avg_response_time: Some(avg),
            };
            let _ = status_handle.emit("test://status", &payload);

            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    // Create batch channel for result aggregation (100ms flush, ~200 results max)
    let (batch_tx, mut batch_rx) = mpsc::unbounded_channel::<Vec<SampleResult>>();

    // Spawn batch collector — coalesces results across VUs before emitting to frontend
    let batch_app = app_handle.clone();
    let batch_task = tokio::spawn(async move {
        let mut buffer: Vec<SampleResult> = Vec::with_capacity(500);
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_millis(100),
                batch_rx.recv(),
            )
            .await
            {
                Ok(Some(batch)) => {
                    buffer.extend(batch);
                    while let Ok(more) = batch_rx.try_recv() {
                        buffer.extend(more);
                    }
                }
                Ok(None) => break, // All senders dropped — VUs finished
                Err(_) => {}       // timeout — flush below
            }
            if !buffer.is_empty() {
                let _ = batch_app.emit(
                    "test://batch-result",
                    &serde_json::json!({ "results": &buffer }),
                );
                buffer.clear();
            }
        }
        // Flush final batch
        if !buffer.is_empty() {
            let _ = batch_app.emit(
                "test://batch-result",
                &serde_json::json!({ "results": &buffer }),
            );
        }
    });

    // Spawn virtual users
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for tg in &plan.thread_groups {
        if !tg.enabled {
            continue;
        }
        // Load CSV datasets for this thread group
        let csv_rows = Arc::new(load_csv_datasets(&tg.children));

        let tg = Arc::new(tg.clone());
        let client_clone = client.clone();
        let client_no_redirect_clone = client_no_redirect.clone();
        let cancel = cancel_token().clone();
        let samples = total_samples.clone();
        let errors = error_count.clone();
        let active = threads_active.clone();
        let csv_data = csv_rows.clone();

        for i in 0..tg.num_threads {
            let vu_ctx = thread_group::VuContext {
                client: client_clone.clone(),
                client_no_redirect: client_no_redirect_clone.clone(),
                tg: tg.clone(),
                result_tx: batch_tx.clone(),
                cancel: cancel.clone(),
                thread_index: i,
                total_threads,
                total_samples: samples.clone(),
                error_count: errors.clone(),
                threads_active: active.clone(),
                csv_rows: csv_data.clone(),
            };
            let handle = tokio::spawn(thread_group::execute_virtual_user(vu_ctx));
            handles.push(handle);
        }
    }

    // Wait for all virtual users to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Drop the last sender so the batch task exits
    drop(batch_tx);
    let _ = batch_task.await;

    // Signal cancellation and wait for status task to finish
    cancel_token().store(true, Ordering::Relaxed);
    let _ = status_task.await;

    // Evaluate test-level assertions
    let total = total_samples.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    let error_rate = if total > 0 { errors as f64 / total as f64 * 100.0 } else { 0.0 };
    let (_p50, _p90, p99, _throughput, avg) = compute_percentiles();

    let mut assertion_results: Vec<AssertionEvalResult> = Vec::new();
    for a in &plan.assertions {
        let actual = match a.metric.as_str() {
            "errorRate" => error_rate,
            "avgResponseTime" => avg as f64,
            "throughput" => total as f64, // total samples over test duration
            "p99" => p99 as f64,
            _ => 0.0,
        };
        let passed = match a.operator.as_str() {
            "lt" => actual < a.value,
            "gt" => actual > a.value,
            _ => false,
        };
        assertion_results.push(AssertionEvalResult {
            metric: a.metric.clone(),
            operator: a.operator.clone(),
            expected: a.value,
            actual,
            passed,
        });
    }
    let _ = app_handle.emit(
        "test://assertion-result",
        &serde_json::json!({ "assertions": &assertion_results }),
    );

    Ok(())
}

/// Stop the test execution.
pub fn stop_test_plan() {
    cancel_token().store(true, Ordering::Relaxed);
}
