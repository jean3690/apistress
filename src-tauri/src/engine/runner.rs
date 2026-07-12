use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use tauri::AppHandle;
use tauri::Emitter;
use tokio::task::JoinHandle;

use super::plan::{ChildElement, TestPlan};
use super::result::StatusPayload;
use super::thread_group;

static CANCEL: OnceLock<Arc<AtomicBool>> = OnceLock::new();

fn cancel_token() -> Arc<AtomicBool> {
    CANCEL
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

fn reset_cancel() {
    if let Some(token) = CANCEL.get() {
        token.store(false, Ordering::SeqCst);
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

    // Clone app_handle for the status task and for virtual users
    let status_handle = app_handle.clone();

    // Spawn status emitter task
    let status_cancel = cancel_token().clone();
    let status_samples = total_samples.clone();
    let status_errors = error_count.clone();
    let status_active = threads_active.clone();

    let status_task = tokio::spawn(async move {
        loop {
            if status_cancel.load(Ordering::SeqCst) {
                let payload = StatusPayload {
                    status: "completed".to_string(),
                    threads_active: 0,
                    total_samples: status_samples.load(Ordering::SeqCst),
                    error_count: status_errors.load(Ordering::SeqCst),
                };
                let _ = status_handle.emit("test://status", &payload);
                break;
            }

            let payload = StatusPayload {
                status: "running".to_string(),
                threads_active: status_active.load(Ordering::SeqCst),
                total_samples: status_samples.load(Ordering::SeqCst),
                error_count: status_errors.load(Ordering::SeqCst),
            };
            let _ = status_handle.emit("test://status", &payload);

            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
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
        let app_clone = app_handle.clone();
        let cancel = cancel_token().clone();
        let samples = total_samples.clone();
        let errors = error_count.clone();
        let active = threads_active.clone();
        let csv_data = csv_rows.clone();

        for i in 0..tg.num_threads {
            let handle = tokio::spawn(thread_group::execute_virtual_user(
                client_clone.clone(),
                client_no_redirect_clone.clone(),
                tg.clone(),
                app_clone.clone(),
                cancel.clone(),
                i,
                total_threads,
                samples.clone(),
                errors.clone(),
                active.clone(),
                csv_data.clone(),
            ));
            handles.push(handle);
        }
    }

    // Wait for all virtual users to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Signal cancellation and wait for status task to finish
    cancel_token().store(true, Ordering::SeqCst);
    let _ = status_task.await;

    Ok(())
}

/// Stop the test execution.
pub fn stop_test_plan() {
    cancel_token().store(true, Ordering::SeqCst);
}
