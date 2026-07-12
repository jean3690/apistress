use std::collections::HashMap;

use super::plan::ChildElement;
use super::sampler;

/// What to do after evaluating a controller
pub enum ControllerAction {
    /// Execute the controller's children
    Execute(Vec<ChildElement>),
    /// Skip this controller entirely
    Skip,
    /// Break out of the current loop level
    Break,
}

/// Evaluate a controller and decide what action to take.
/// `loop_counts` tracks per-controller iteration counts.
/// `throughput_times` tracks last execution instant for ThroughputControllers.
#[must_use]
pub fn evaluate_controller(
    child: &ChildElement,
    loop_counts: &mut HashMap<String, u32>,
    variables: &HashMap<String, String>,
    throughput_times: &mut HashMap<String, std::time::Instant>,
) -> ControllerAction {
    match child {
        ChildElement::LoopController(lc) => {
            if !lc.enabled {
                return ControllerAction::Skip;
            }
            let count = loop_counts.entry(lc.id.clone()).or_insert(0);
            if lc.loops >= 0 && *count >= lc.loops as u32 {
                return ControllerAction::Break;
            }
            *count += 1;
            ControllerAction::Execute(lc.children.clone())
        }
        ChildElement::IfController(ic) => {
            if !ic.enabled {
                return ControllerAction::Skip;
            }
            let condition = sampler::interpolate(&ic.condition, variables);
            let is_true = !condition.is_empty() && condition != "false" && condition != "0";
            if is_true {
                ControllerAction::Execute(ic.children.clone())
            } else {
                ControllerAction::Skip
            }
        }
        ChildElement::WhileController(wc) => {
            if !wc.enabled {
                return ControllerAction::Skip;
            }
            let condition = sampler::interpolate(&wc.condition, variables);
            let is_true = !condition.is_empty() && condition != "false" && condition != "0";
            if is_true {
                ControllerAction::Execute(wc.children.clone())
            } else {
                ControllerAction::Break
            }
        }
        ChildElement::TransactionController(tc) => {
            if !tc.enabled {
                return ControllerAction::Skip;
            }
            ControllerAction::Execute(tc.children.clone())
        }
        ChildElement::ThroughputController(tc) => {
            if !tc.enabled {
                return ControllerAction::Skip;
            }
            if tc.throughput == 0 {
                return ControllerAction::Skip;
            }
            // Time-based throttling
            let interval_ms = (60_000u64).div_ceil(tc.throughput.max(1) as u64);
            let now = std::time::Instant::now();
            let last = throughput_times.entry(tc.id.clone()).or_insert_with(|| {
                // First time: allow immediate execution but record start
                now
            });
            if now.duration_since(*last).as_millis() as u64 >= interval_ms {
                *last = now;
                ControllerAction::Execute(tc.children.clone())
            } else {
                ControllerAction::Skip
            }
        }
        _ => ControllerAction::Skip,
    }
}
