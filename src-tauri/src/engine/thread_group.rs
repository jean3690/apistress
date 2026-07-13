use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use reqwest::Client;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use super::assertion;
use super::controller::{self, ControllerAction};
use super::plan::{ChildElement, ThreadGroup};
use super::result::{ExecutionContext, SampleResult};
use super::sampler;
use super::timer::{self, TimerAction};

/// Bundled context for a single virtual user execution.
pub struct VuContext {
    pub client: Arc<Client>,
    pub client_no_redirect: Arc<Client>,
    pub tg: Arc<ThreadGroup>,
    pub result_tx: mpsc::UnboundedSender<Vec<SampleResult>>,
    pub cancel: Arc<AtomicBool>,
    pub thread_index: u32,
    pub total_threads: u32,
    pub total_samples: Arc<AtomicU32>,
    pub error_count: Arc<AtomicU32>,
    pub threads_active: Arc<AtomicU32>,
    pub csv_rows: Arc<Vec<Vec<(String, String)>>>,
}

pub async fn execute_virtual_user(ctx: VuContext) {
    let exec_ctx = ExecutionContext {
        thread_name: format!("{}-{}", ctx.tg.name, ctx.thread_index + 1),
        tg_name: ctx.tg.name.clone(),
        group_threads: ctx.tg.num_threads,
        all_threads: ctx.total_threads,
    };

    // Ramp-up delay
    if ctx.tg.ramp_up > 0 && ctx.tg.num_threads > 0 {
        let delay_ms =
            (ctx.thread_index as f64 * ctx.tg.ramp_up as f64 / ctx.tg.num_threads as f64) * 1000.0;
        sleep(Duration::from_millis(delay_ms as u64)).await;
    }

    // Delay before start
    if ctx.tg.delay > 0 {
        sleep(Duration::from_secs(ctx.tg.delay as u64)).await;
    }

    if ctx.cancel.load(Ordering::Relaxed) {
        return;
    }

    ctx.threads_active.fetch_add(1, Ordering::Relaxed);

    // Warm-up: results collected during this period are not sent
    let warmup_end = if ctx.tg.warm_up > 0 {
        Some(std::time::Instant::now() + Duration::from_secs(ctx.tg.warm_up as u64))
    } else {
        None
    };

    let iterations = if ctx.tg.loops < 0 {
        u32::MAX
    } else {
        ctx.tg.loops as u32
    };
    let duration_limit = if ctx.tg.duration > 0 {
        Some(std::time::Instant::now() + Duration::from_secs(ctx.tg.duration as u64))
    } else {
        None
    };
    let csv_idx = Arc::new(AtomicUsize::new(0));

    for _iteration in 0..iterations {
        if ctx.cancel.load(Ordering::Relaxed) {
            break;
        }

        if let Some(limit) = duration_limit {
            if std::time::Instant::now() >= limit {
                break;
            }
        }

        let mut variables: HashMap<String, String> = HashMap::new();
        variables.insert("__threadNum".to_string(), (ctx.thread_index + 1).to_string());
        if !ctx.csv_rows.is_empty() {
            let idx = csv_idx.fetch_add(1, Ordering::Relaxed) % ctx.csv_rows.len();
            for (key, value) in &ctx.csv_rows[idx] {
                variables.insert(key.clone(), value.clone());
            }
        }

        let results = execute_children(
            &ctx.client,
            &ctx.client_no_redirect,
            &ctx.tg.children,
            &mut variables,
            &exec_ctx,
            &ctx.cancel,
        )
        .await;

        let in_warmup = warmup_end
            .map(|end| std::time::Instant::now() < end)
            .unwrap_or(false);

        if !in_warmup {
            let _ = ctx.result_tx.send(results.clone());
        }

        for result in &results {
            if result.id.is_empty() {
                continue;
            }
            super::runner::push_elapsed(result.elapsed);
            ctx.total_samples.fetch_add(1, Ordering::Relaxed);
            if !result.success {
                ctx.error_count.fetch_add(1, Ordering::Relaxed);
            }

            if !result.success {
                match ctx.tg.on_error_action.as_str() {
                    "stopThread" => {
                        ctx.threads_active.fetch_sub(1, Ordering::Relaxed);
                        return;
                    }
                    "stopTest" => {
                        ctx.cancel.store(true, Ordering::Relaxed);
                        ctx.threads_active.fetch_sub(1, Ordering::Relaxed);
                        return;
                    }
                    "startNextLoop" => break,
                    _ => {}
                }
            }
        }
    }

    ctx.threads_active.fetch_sub(1, Ordering::Relaxed);
}

async fn execute_children(
    client: &Client,
    client_no_redirect: &Client,
    children: &[ChildElement],
    variables: &mut HashMap<String, String>,
    ctx: &ExecutionContext,
    cancel: &Arc<AtomicBool>,
) -> Vec<SampleResult> {
    let mut results = Vec::new();
    let mut last_sampler_result: Option<SampleResult> = None;

    // Per-child-instance loop counters for controllers
    let mut loop_counts: HashMap<String, u32> = HashMap::new();
    let mut throughput_times: HashMap<String, std::time::Instant> = HashMap::new();

    // Collect HttpDefaults from this level (including inside nested controllers)
    let defaults = sampler::collect_defaults(children);

    // We use an index-based loop so we can handle nested controller iterations
    execute_level(
        client,
        client_no_redirect,
        children,
        variables,
        ctx,
        cancel,
        &mut loop_counts,
        &mut last_sampler_result,
        &mut results,
        &defaults,
        &mut throughput_times,
    )
    .await;

    results
}

#[allow(clippy::too_many_arguments)]
async fn execute_level(
    client: &Client,
    client_no_redirect: &Client,
    children: &[ChildElement],
    variables: &mut HashMap<String, String>,
    ctx: &ExecutionContext,
    cancel: &Arc<AtomicBool>,
    loop_counts: &mut HashMap<String, u32>,
    last_sampler_result: &mut Option<SampleResult>,
    results: &mut Vec<SampleResult>,
    defaults: &[super::plan::HttpDefaults],
    throughput_times: &mut HashMap<String, std::time::Instant>,
) {
    let mut i = 0isize;
    while i < children.len() as isize {
        if cancel.load(Ordering::Relaxed) {
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
                let mut sampler = s.clone();
                sampler::apply_defaults(&mut sampler, defaults);
                let active_client = if sampler.follow_redirects {
                    client
                } else {
                    client_no_redirect
                };
                let mut result =
                    sampler::execute_sampler(active_client, &sampler, variables, ctx)
                        .await;
                // Retry on failure
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_sampler(active_client, &sampler, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::GraphQlSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_graphql_sampler(client, s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_graphql_sampler(client, s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::SseSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_sse_sampler(client, s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_sse_sampler(client, s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::MqttSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_mqtt_sampler(s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_mqtt_sampler(s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::WebSocketSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_websocket_sampler(s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_websocket_sampler(s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::GrpcSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_grpc_sampler(s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_grpc_sampler(s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::TcpSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_tcp_sampler(s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_tcp_sampler(s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
                if !result.id.is_empty() {
                    results.push(result);
                }
            }

            ChildElement::RedisSampler(s) => {
                if !s.enabled {
                    continue;
                }
                let mut result =
                    sampler::execute_redis_sampler(s, variables, ctx).await;
                for _ in 0..s.retry_count {
                    if result.success { break; }
                    sleep(Duration::from_millis(s.retry_delay)).await;
                    result = sampler::execute_redis_sampler(s, variables, ctx).await;
                }
                *last_sampler_result = Some(result.clone());
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

                let action = controller::evaluate_controller(
                    child,
                    loop_counts,
                    variables,
                    throughput_times,
                );
                match action {
                    ControllerAction::Execute(ctrl_children) => {
                        Box::pin(execute_level(
                            client,
                            client_no_redirect,
                            &ctrl_children,
                            variables,
                            ctx,
                            cancel,
                            loop_counts,
                            last_sampler_result,
                            results,
                            defaults,
                            throughput_times,
                        ))
                        .await;

                        // For WhileController, re-evaluate by going back
                        if let ChildElement::WhileController(_) = child {
                            let recheck = controller::evaluate_controller(
                                child,
                                loop_counts,
                                variables,
                                throughput_times,
                            );
                            if matches!(recheck, ControllerAction::Execute(_)) {
                                i -= 1; // re-enter this controller on next iteration
                            }
                        }
                        // For LoopController, also re-enter until break
                        if let ChildElement::LoopController(_) = child {
                            let recheck = controller::evaluate_controller(
                                child,
                                loop_counts,
                                variables,
                                throughput_times,
                            );
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
                    let search_in = if re.use_body { &last.response_body } else { "" };
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

            // HttpDefaults applied above, CsvDataSet handled in runner
            ChildElement::HttpDefaults(_) | ChildElement::CsvDataSet(_) => {}

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
            .filter_map(|c| {
                c.get(1)
                    .or_else(|| c.get(0))
                    .map(|m| m.as_str().to_string())
            })
            .collect();
        let idx = (re.match_no as usize)
            .saturating_sub(1)
            .min(caps.len().saturating_sub(1));
        let value = caps
            .get(idx)
            .cloned()
            .unwrap_or_else(|| re.default_value.clone());
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

fn resolve_json_path<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
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
