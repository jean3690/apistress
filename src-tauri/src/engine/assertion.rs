use std::collections::HashMap;

use super::plan::ChildElement;
use super::result::{AssertionResult, SampleResult};

pub fn evaluate_assertion(
    child: &ChildElement,
    result: &SampleResult,
    _variables: &HashMap<String, String>,
) -> AssertionResult {
    match child {
        ChildElement::ResponseAssertion(a) => evaluate_response_assertion(a, result),
        ChildElement::JsonAssertion(a) => evaluate_json_assertion(a, result),
        ChildElement::DurationAssertion(a) => evaluate_duration_assertion(a, result),
        _ => AssertionResult {
            name: String::new(),
            failure: false,
            failure_message: String::new(),
        },
    }
}

fn evaluate_response_assertion(
    a: &super::plan::ResponseAssertion,
    result: &SampleResult,
) -> AssertionResult {
    if !a.enabled || a.patterns.is_empty() {
        return AssertionResult {
            name: a.name.clone(),
            failure: false,
            failure_message: String::new(),
        };
    }

    let test_value = match a.test_field.as_str() {
        "responseCode" => &result.response_code,
        "responseMessage" => &result.response_message,
        "responseBody" => &result.response_body,
        "responseHeaders" => "", // simplified
        "requestHeaders" => "",
        "url" => &result.url,
        _ => "",
    };

    let negate = a.pattern_matching.starts_with("not");
    let match_type = if negate { &a.pattern_matching[3..] } else { &a.pattern_matching };
    // Normalize: "notContains" -> "Contains"
    let match_type = if match_type.is_empty() { "contains" } else { match_type };

    let mut failures: Vec<String> = Vec::new();
    for pattern in &a.patterns {
        if pattern.is_empty() {
            continue;
        }
        let matched = match match_type {
            "contains" | "substring" => test_value.contains(pattern),
            "matches" | "equals" => test_value == pattern,
            _ => test_value.contains(pattern),
        };

        let should_pass = if negate { !matched } else { matched };
        if !should_pass {
            failures.push(format!(
                "Expected '{}' {} '{}' in {}",
                test_value,
                a.pattern_matching,
                pattern,
                a.test_field
            ));
        }
    }

    AssertionResult {
        name: a.name.clone(),
        failure: !failures.is_empty(),
        failure_message: failures.join("; "),
    }
}

fn evaluate_json_assertion(
    a: &super::plan::JsonAssertion,
    result: &SampleResult,
) -> AssertionResult {
    if !a.enabled {
        return AssertionResult {
            name: a.name.clone(),
            failure: false,
            failure_message: String::new(),
        };
    }

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result.response_body);
    let exists = parsed.is_ok();

    let failure = match a.comparison_mode.as_str() {
        "exists" => !exists,
        "notExists" => exists,
        "equals" => {
            if let Ok(val) = &parsed {
                let expected: Result<serde_json::Value, _> = serde_json::from_str(&a.expected_value);
                match expected {
                    Ok(exp) => val != &exp,
                    Err(_) => {
                        val.as_str().map_or(true, |s| s != a.expected_value)
                    }
                }
            } else {
                true
            }
        }
        _ => false,
    };

    AssertionResult {
        name: a.name.clone(),
        failure,
        failure_message: if failure {
            format!(
                "JSON assertion failed: mode={}, path={}",
                a.comparison_mode, a.json_path
            )
        } else {
            String::new()
        },
    }
}

fn evaluate_duration_assertion(
    a: &super::plan::DurationAssertion,
    result: &SampleResult,
) -> AssertionResult {
    if !a.enabled {
        return AssertionResult {
            name: a.name.clone(),
            failure: false,
            failure_message: String::new(),
        };
    }

    let failure = result.elapsed > a.max_duration;
    AssertionResult {
        name: a.name.clone(),
        failure,
        failure_message: if failure {
            format!(
                "Duration {}ms exceeded max {}ms",
                result.elapsed, a.max_duration
            )
        } else {
            String::new()
        },
    }
}
