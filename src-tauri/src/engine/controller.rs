use std::collections::HashMap;

use super::plan::ChildElement;

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
pub fn evaluate_controller(
    child: &ChildElement,
    loop_counts: &mut HashMap<String, u32>,
    variables: &HashMap<String, String>,
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
            let condition = interpolate_vars(&ic.condition, variables);
            let is_true = !condition.is_empty()
                && condition != "false"
                && condition != "0";
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
            let condition = interpolate_vars(&wc.condition, variables);
            let is_true = !condition.is_empty()
                && condition != "false"
                && condition != "0";
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
            let count = loop_counts.entry(tc.id.clone()).or_insert(0);
            *count += 1;
            let max_per_min = tc.throughput.max(1) as u64;
            // Simple throttling: execute once every N calls
            let interval = (60u64).div_ceil(max_per_min).max(1);
            if *count as u64 % interval == 0 {
                ControllerAction::Execute(tc.children.clone())
            } else {
                ControllerAction::Skip
            }
        }
        _ => ControllerAction::Skip,
    }
}

fn interpolate_vars(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}
