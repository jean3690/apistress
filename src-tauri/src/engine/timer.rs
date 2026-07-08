use std::collections::HashMap;

use rand::Rng;

use super::plan::ChildElement;

pub enum TimerAction {
    Delay(std::time::Duration),
    None,
}

pub fn evaluate_timer(child: &ChildElement, _variables: &HashMap<String, String>) -> TimerAction {
    match child {
        ChildElement::ConstantTimer(t) => {
            if !t.enabled {
                return TimerAction::None;
            }
            TimerAction::Delay(std::time::Duration::from_millis(t.delay))
        }
        ChildElement::UniformRandomTimer(t) => {
            if !t.enabled {
                return TimerAction::None;
            }
            let mut rng = rand::thread_rng();
            let delay = rng.gen_range(t.min_delay..=t.max_delay);
            TimerAction::Delay(std::time::Duration::from_millis(delay))
        }
        ChildElement::GaussianRandomTimer(t) => {
            if !t.enabled {
                return TimerAction::None;
            }
            // Approximate Gaussian using Box-Muller or central limit
            let mut rng = rand::thread_rng();
            let u1: f64 = rng.gen();
            let u2: f64 = rng.gen();
            let z = (-2.0_f64 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let delay = (t.delay as f64 + z * t.deviation as f64).max(0.0) as u64;
            TimerAction::Delay(std::time::Duration::from_millis(delay))
        }
        _ => TimerAction::None,
    }
}
