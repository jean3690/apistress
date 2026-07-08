use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use reqwest::Client;
use tauri::AppHandle;
use tokio::time::{sleep, Duration};

use super::assertion;
use super::controller::{self, ControllerAction};
use super::plan::{ChildElement, ThreadGroup};
use super::result::{ExecutionContext, SampleResult};
use super::sampler;
use super::timer::{self, TimerAction};

pub async fn execute_virtual_user(
    client: Arc<Client>,
    tg: Arc<ThreadGroup>,
    app_handle: AppHandle,
    cancel: Arc<AtomicBool>,
    thread_index: u32,
    total_threads: u32,
    total_samples: Arc<AtomicU32>,
    error_count: Arc<AtomicU32>,
    threads_active: Arc<AtomicU32>,
    csv_rows: Arc<Vec<Vec<(String, String)>>>,
) {
    let ctx = ExecutionContext {
        thread_name: format!("{}-{}", tg.name, thread_index + 1),
        tg_name: tg.name.clone(),
        group_threads: tg.num_threads,
        all_threads: total_threads,
    };

    // Ramp-up delay
    if tg.ramp_up > 0 && tg.num_threads > 0 {
        let delay_ms =
            (thread_index as f64 * tg.ramp_up as f64 / tg.num_threads as f64) * 1000.0;
        sleep(Duration::from_millis(delay_ms as u64)).await;
    }

    // Delay before start
    if tg.delay > 0 {
        sleep(Duration::from_secs(tg.delay as u64)).await;
    }

    if cancel.load(Ordering::SeqCst) {
        return;
    }

    threads_active.fetch_add(1, Ordering::SeqCst);

    // Execution
    let iterations = if tg.loops < 0 { u32::MAX } else { tg.loops as u32 };
    let duration_limit = if tg.duration > 0 {
        Some(std::time::Instant::now() + Duration::from_secs(tg.duration as u64))
    } else {
        None
    };
    let csv_idx = Arc::new(AtomicUsize::new(0));

    for _iteration in 0..iterations {
        if cancel.load(Ordering::SeqCst) {
            break;
        }

        // Duration check
        if let Some(limit) = duration_limit {
            if std::time::Instant::now() >= limit {
                break;
            }
        }

        let mut variables: HashMap<String, String> = HashMap::new();
        variables.insert(
            "__threadNum".to_string(),
            (thread_index + 1).to_string(),
        );
        // Inject CSV row variables for this iteration
        if !csv_rows.is_empty() {
            let idx = csv_idx.fetch_add(1, Ordering::SeqCst) % csv_rows.len();
            for (key, value) in &csv_rows[idx] {
                variables.insert(key.clone(), value.clone());
            }
        }

        let results = execute_children(
            &client,
            &tg.children,
            &mut variables,
            &ctx,
            &app_handle,
            &cancel,
        )
        .await;

        for result in &results {
            if result.id.is_empty() {
                continue; // skipped
            }
            if result.success {
                total_samples.fetch_add(1, Ordering::SeqCst);
            } else {
                total_samples.fetch_add(1, Ordering::SeqCst);
                error_count.fetch_add(1, Ordering::SeqCst);
            }

            if !result.success {
                match tg.on_error_action.as_str() {
                    "stopThread" => {
                        threads_active.fetch_sub(1, Ordering::SeqCst);
                        return;
                    }
                    "stopTest" => {
                        cancel.store(true, Ordering::SeqCst);
                        threads_active.fetch_sub(1, Ordering::SeqCst);
                        return;
                    }
                    "startNextLoop" => break,
                    _ => {} // continue
                }
            }
        }
    }

    threads_active.fetch_sub(1, Ordering::SeqCst);
}

async fn execute_children(
    client: &Client,
    children: &[ChildElement],
    variables: &mut HashMap<String, String>,
    ctx: &ExecutionContext,
    app_handle: &AppHandle,
    cancel: &Arc<AtomicBool>,
) -> Vec<SampleResult> {
    let mut results = Vec::new();
    let mut last_sampler_result: Option<SampleResult> = None;

    // Per-child-instance loop counters for controllers
    let mut loop_counts: HashMap<String, u32> = HashMap::new();

    // We use an index-based loop so we can handle nested controller iterations
    execute_level(
        client,
        children,
        variables,
        ctx,
        app_handle,
        cancel,
        &mut loop_counts,
        &mut last_sampler_result,
        &mut results,
    )
    .await;

    results
}

async fn execute_level(
    client: &Client,
    children: &[ChildElement],
    variables: &mut HashMap<String, String>,
    ctx: &ExecutionContext,
    app_handle: &AppHandle,
    cancel: &Arc<AtomicBool>,
    loop_counts: &mut HashMap<String, u32>,
    last_sampler_result: &mut Option<SampleResult>,
    results: &mut Vec<SampleResult>,
) {
    let mut i = 0isize;
    while i < children.len() as isize {
        if cancel.load(Ordering::SeqCst) {
            break;
        }

        let child = &children[i as usize];
        i += 1;

        match child {
            // Samplers
            ChildElement::HttpSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let result =
                    sampler::execute_sampler(client, s, variables, ctx, app_handle).await;
                *last_sampler_result = Some(result.clone());
                // Don't add skipped results
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            // Controllers
            ChildElement::LoopController(_)
            | ChildElement::IfController(_)
            | ChildElement::WhileController(_)
            | ChildElement::TransactionController(_)
            | ChildElement::ThroughputController(_) => {
                // Save the loop count for this controller before evaluation
                let saved_count = loop_counts.get(&get_child_id(child)).copied();

                let action = controller::evaluate_controller(child, loop_counts, variables);
                match action {
                    ControllerAction::Execute(ctrl_children) => {
                        Box::pin(execute_level(
                            client,
                            &ctrl_children,
                            variables,
                            ctx,
                            app_handle,
                            cancel,
                            loop_counts,
                            last_sampler_result,
                            results,
                        ))
                        .await;

                        // For WhileController, re-evaluate by going back
                        if let ChildElement::WhileController(_) = child {
                            let recheck =
                                controller::evaluate_controller(child, loop_counts, variables);
                            if matches!(recheck, ControllerAction::Execute(_)) {
                                i -= 1; // re-enter this controller on next iteration
                            }
                        }
                        // For LoopController, also re-enter until break
                        if let ChildElement::LoopController(_) = child {
                            let recheck =
                                controller::evaluate_controller(child, loop_counts, variables);
                            if matches!(recheck, ControllerAction::Execute(_)) {
                                i -= 1;
                            } else {
                                // Reset the counter on break/complete
                                if let Some(prev) = saved_count {
                                    loop_counts.insert(get_child_id(child), prev);
                                }
                            }
                        }
                    }
                    ControllerAction::Skip => {}
                    ControllerAction::Break => {
                        // Reset loop counter for this controller
                        if let Some(prev) = saved_count {
                            loop_counts.insert(get_child_id(child), prev);
                        }
                    }
                }
            }

            // Timers
            ChildElement::ConstantTimer(_)
            | ChildElement::UniformRandomTimer(_)
            | ChildElement::GaussianRandomTimer(_) => {
                let action = timer::evaluate_timer(child, variables);
                if let TimerAction::Delay(d) = action {
                    sleep(d).await;
                }
            }

            // Assertions (evaluate against last sampler result)
            ChildElement::ResponseAssertion(_)
            | ChildElement::JsonAssertion(_)
            | ChildElement::DurationAssertion(_) => {
                if let Some(ref mut last) = last_sampler_result {
                    let ar = assertion::evaluate_assertion(child, last, variables);
                    last.assertion_results.push(ar);
                }
            }

            // Extractors
            ChildElement::RegexExtractor(re) => {
                if !re.enabled {
                    continue;
                }
                if let Some(ref last) = last_sampler_result {
                    let search_in = if re.use_body {
                        &last.response_body
                    } else {
                        ""
                    };
                    extract_regex(re, search_in, variables);
                }
            }
            ChildElement::JsonExtractor(je) => {
                if !je.enabled {
                    continue;
                }
                if let Some(ref last) = last_sampler_result {
                    extract_json(je, &last.response_body, variables);
                }
            }
            ChildElement::BoundaryExtractor(be) => {
                if !be.enabled {
                    continue;
                }
                if let Some(ref last) = last_sampler_result {
                    extract_boundary(be, &last.response_body, variables);
                }
            }

            // Config elements - apply to variables
            ChildElement::UserVariables(uv) => {
                if uv.enabled {
                    for kv in &uv.variables {
                        variables.insert(kv.key.clone(), kv.value.clone());
                    }
                }
            }

            // Config elements applied before sampling (handled in sampler via defaults)
            ChildElement::HttpDefaults(_) | ChildElement::CsvDataSet(_) => {
                // Stored for future use in Phase 4
            }

            // Listeners / other - ignored during execution
            _ => {}
        }
    }
}

fn get_child_id(child: &ChildElement) -> String {
    match child {
        ChildElement::LoopController(c) => c.id.clone(),
        ChildElement::IfController(c) => c.id.clone(),
        ChildElement::WhileController(c) => c.id.clone(),
        ChildElement::TransactionController(c) => c.id.clone(),
        ChildElement::ThroughputController(c) => c.id.clone(),
        _ => String::new(),
    }
}

fn extract_regex(
    re: &super::plan::RegexExtractor,
    search_in: &str,
    variables: &mut HashMap<String, String>,
) {
    let re_result = regex::Regex::new(&re.regex);
    if let Ok(rx) = re_result {
        let caps: Vec<String> = rx
            .captures_iter(search_in)
            .filter_map(|c| c.get(1).or_else(|| c.get(0)).map(|m| m.as_str().to_string()))
            .collect();
        let idx = (re.match_no as usize).saturating_sub(1).min(caps.len().saturating_sub(1));
        let value = caps.get(idx).cloned().unwrap_or_else(|| re.default_value.clone());
        let interpolated = value
            .replace(&format!("${}", re.reference_name), &value)
            .replace("$0", &caps.first().cloned().unwrap_or_default());
        variables.insert(re.reference_name.clone(), interpolated);
    } else {
        variables.insert(re.reference_name.clone(), re.default_value.clone());
    }
}

fn extract_json(
    je: &super::plan::JsonExtractor,
    body: &str,
    variables: &mut HashMap<String, String>,
) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        // Simple JSON path: $.foo.bar or $[0].key
        let path = je.json_path.trim_start_matches('$').trim_start_matches('.');
        let found = resolve_json_path(&val, path);
        if let Some(v) = found {
            variables.insert(
                je.reference_name.clone(),
                v.as_str().unwrap_or(&v.to_string()).to_string(),
            );
        } else {
            variables.insert(je.reference_name.clone(), je.default_value.clone());
        }
    } else {
        variables.insert(je.reference_name.clone(), je.default_value.clone());
    }
}

fn resolve_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let segments: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    let mut current = value;
    for seg in segments {
        if seg.starts_with('[') && seg.ends_with(']') {
            let idx: usize = seg[1..seg.len() - 1].parse().ok()?;
            current = current.get(idx)?;
        } else {
            // Try bracket notation: key[0]
            if let Some(bracket_pos) = seg.find('[') {
                let key = &seg[..bracket_pos];
                current = current.get(key)?;
                let rest = &seg[bracket_pos..];
                for sub_seg in rest.split("][") {
                    let clean = sub_seg.trim_matches(|c| c == '[' || c == ']');
                    if let Ok(idx) = clean.parse::<usize>() {
                        current = current.get(idx)?;
                    }
                }
            } else {
                current = current.get(seg)?;
            }
        }
    }
    Some(current)
}

fn extract_boundary(
    be: &super::plan::BoundaryExtractor,
    body: &str,
    variables: &mut HashMap<String, String>,
) {
    let mut extracted = String::new();
    let mut search_start = 0;
    let mut match_num = 0;

    while search_start < body.len() {
        if let Some(left_pos) = body[search_start..].find(&be.left_boundary) {
            let content_start = search_start + left_pos + be.left_boundary.len();
            if let Some(right_pos) = body[content_start..].find(&be.right_boundary) {
                match_num += 1;
                if match_num == be.match_no as usize {
                    extracted = body[content_start..content_start + right_pos].to_string();
                    break;
                }
                search_start = content_start + right_pos + be.right_boundary.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if extracted.is_empty() {
        extracted = be.default_value.clone();
    }
    variables.insert(be.reference_name.clone(), extracted);
}
