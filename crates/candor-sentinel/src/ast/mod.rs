/// AST-level slop detection using line-based source analysis (no syn dependency).
///
/// Organized into submodules for maintainability:
/// - `fn_analysis`: function definition/call/body extraction
/// - `narration`: AI narration comment pattern detection
/// - `code_quality`: single-use helpers, over-abstraction, unreachable code, if-else chains
pub mod code_quality;
pub mod fn_analysis;
pub mod narration;

use std::collections::HashMap;

use crate::rules::{RuleViolation, RulesCheck, ViolationSeverity};

/// Run all AST-level checks against a Rust source file.
///
/// This performs structural analysis that goes beyond what simple regex
/// can detect: function reference counting, block scoping for unreachable
/// code detection, if-else chain length measurement, and multi-line
/// comment pattern analysis anchored to specific line numbers.
pub fn check_ast(source: &str) -> RulesCheck {
    let lines: Vec<&str> = source.lines().collect();
    let mut violations: Vec<RuleViolation> = Vec::new();

    // ── Phase 1: Scan for function definitions and calls ──
    let mut fn_defs: HashMap<String, Vec<usize>> = HashMap::new();
    let mut fn_calls: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        if fn_analysis::is_fn_def(line)
            && let Some(name) = fn_analysis::extract_fn_name(line)
        {
            fn_defs.entry(name).or_default().push(i);
        }
    }

    // Phase 1b: Collect defined function names for call detection
    let defined_names: Vec<String> = fn_defs.keys().cloned().collect();

    for (i, line) in lines.iter().enumerate() {
        let calls = fn_analysis::extract_fn_calls(line, &defined_names);
        for call in calls {
            fn_calls.entry(call).or_default().push(i);
        }
    }

    // ── Phase 2: Single-use helper detection ──
    violations.extend(code_quality::check_single_use_helpers(
        &fn_defs, &fn_calls, &lines,
    ));

    // ── Phase 3: Over-abstraction detection ──
    violations.extend(code_quality::check_over_abstraction(&fn_defs, &lines));

    // ── Phase 4: Unreachable code detection ──
    for i in 0..lines.len() {
        if let Some(desc) = code_quality::has_unreachable_after(&lines, i) {
            violations.push(RuleViolation {
                rule: "ast:unreachable-code".into(),
                description: desc,
                severity: ViolationSeverity::Warning,
            });
        }
    }

    // ── Phase 5: AI narration comments ──
    violations.extend(narration::check_narration_comments(&lines));

    // ── Phase 6: if-else chain detection ──
    violations.extend(code_quality::check_if_else_chains(&lines));

    // ── Decision ──
    let passed = violations
        .iter()
        .all(|v| v.severity != ViolationSeverity::Fatal);
    RulesCheck { passed, violations }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_code_passes() {
        let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(x: i32, y: i32) -> i32 {
    x * y
}

pub fn main() {
    let result = add(1, 2);
    let product = multiply(3, 4);
    println!("{}", result + product);
}
"#;
        let check = check_ast(source);
        assert!(
            check.passed,
            "Clean code should pass, got violations: {:?}",
            check.violations
        );
    }

    #[test]
    fn test_multiple_violation_types() {
        let source = r#"
// Now we set up the logger
fn setup_logger() -> Logger {
    // Here we create a new logger instance
    Logger::new()
}

fn main() {
    let logger = setup_logger();
    logger.log("hello");
    return;
    logger.log("unreachable");
}
"#;
        let check = check_ast(source);
        let rule_types: std::collections::HashSet<&str> =
            check.violations.iter().map(|v| v.rule.as_str()).collect();

        assert!(
            rule_types.contains("ast:narration-comment"),
            "Should have narration comment violations"
        );
        assert!(
            rule_types.contains("ast:unreachable-code"),
            "Should have unreachable code violations"
        );
    }
}
