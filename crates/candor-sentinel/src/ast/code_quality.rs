/// Code quality AST checks: single-use helpers, over-abstraction,
/// unreachable code detection, and if-else chain detection.
use std::collections::HashMap;

use crate::rules::{RuleViolation, ViolationSeverity};

use super::fn_analysis::collect_fn_body;

/// Check if a function is a single-use helper (definition appears once
/// as a fn, and is referenced exactly once elsewhere).
pub fn check_single_use_helpers(
    fn_defs: &HashMap<String, Vec<usize>>,
    fn_calls: &HashMap<String, Vec<usize>>,
    lines: &[&str],
) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    for (name, def_lines) in fn_defs {
        let call_count = fn_calls.get(name).map(|c| c.len()).unwrap_or(0);
        let def_count = def_lines.len();

        // Only flag functions defined within this file that are called once
        // or zero times (private helpers no one calls)
        if def_count == 1 && call_count == 1 {
            let line_no = def_lines[0] + 1; // 1-indexed
            violations.push(RuleViolation {
                rule: "ast:single-use-helper".into(),
                description: format!(
                    "Function `{}` (defined at line {}) is only called once — consider inlining",
                    name, line_no
                ),
                severity: ViolationSeverity::Warning,
            });
        }

        // Zero-call private functions (not pub) are dead code
        if def_count == 1 && call_count == 0 {
            let line_no = def_lines[0] + 1;
            // Check if it's a public function — we can only approximate
            let def_line = lines[def_lines[0]].trim();
            // Skip `main` (called by the runtime) and pub functions
            if name != "main"
                && !def_line.trim_start().starts_with("pub ")
                && !def_line.trim_start().starts_with("pub(")
            {
                violations.push(RuleViolation {
                    rule: "ast:unused-function".into(),
                    description: format!(
                        "Function `{}` (defined at line {}) is defined but never called — dead code",
                        name, line_no
                    ),
                    severity: ViolationSeverity::Warning,
                });
            }
        }
    }
    violations
}

/// Check for over-abstraction: function whose body is just a single
/// call to another function (thin wrapper with no added logic).
pub fn check_over_abstraction(fn_defs: &HashMap<String, Vec<usize>>, lines: &[&str]) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    for (name, def_lines) in fn_defs {
        for &def_line in def_lines {
            // Collect the body lines of this function
            let body = collect_fn_body(lines, def_line);
            if body.is_empty() {
                continue;
            }

            // A thin wrapper: body has exactly one statement that's a function call
            // Count actual statements (non-empty, non-brace, non-comment, non-fn-def lines)
            let stmts: Vec<&str> = body
                .iter()
                .map(|l| l.trim())
                .filter(|l| {
                    !l.is_empty()
                        && *l != "{"
                        && *l != "}"
                        && !l.starts_with("//")
                        && !l.starts_with("/*")
                        && !l.starts_with('*')
                        && !l.starts_with("fn ")
                })
                .collect();

            if stmts.len() == 1 {
                let stmt = stmts[0];
                // Check if it's just calling another function (possibly with semicolon)
                let clean_stmt = stmt.trim_end_matches(';').trim();
                if clean_stmt.contains('(') && clean_stmt.ends_with(')') {
                    // Extract the called function name
                    if let Some(called_name) = clean_stmt.split('(').next() {
                        let called_name = called_name.trim();
                        // Skip if calling self recursively
                        if called_name != name
                            && !called_name.is_empty()
                            && !called_name.starts_with('&')
                            && !called_name.starts_with('*')
                        {
                            violations.push(RuleViolation {
                                rule: "ast:over-abstraction".into(),
                                description: format!(
                                    "Function `{}` (line {}) is a thin wrapper around `{}` — inline it",
                                    name,
                                    def_line + 1,
                                    called_name
                                ),
                                severity: ViolationSeverity::Warning,
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Check if a line is unreachable (comes after return/panic!/todo!/unreachable!)
pub fn has_unreachable_after(lines: &[&str], idx: usize) -> Option<String> {
    if idx == 0 || idx >= lines.len() {
        return None;
    }
    let prev_line = lines[idx - 1].trim();

    // Check if previous line ends a block: `return expr;`, `panic!(...);`, etc.
    let terminating_patterns = [
        "return ",
        "return;",
        "panic!(",
        "unreachable!(",
        "todo!(",
        "unimplemented!(",
        "std::process::exit(",
        "process::exit(",
        "std::mem::forget(",
        "mem::forget(",
        "loop { break",
    ];

    let is_terminator = terminating_patterns.iter().any(|p| {
        if p.ends_with('(') {
            prev_line.contains(p) && prev_line.ends_with(';')
        } else if p.ends_with(';') || p.ends_with("break") {
            prev_line.starts_with(p)
        } else {
            prev_line.starts_with(p) && prev_line.ends_with(';')
        }
    });

    if !is_terminator {
        return None;
    }

    // Don't flag empty lines, closing braces, or comments after a return
    let current = lines[idx].trim();
    if current.is_empty()
        || current == "}"
        || current.starts_with("//")
        || current.starts_with("/*")
        || current.starts_with('*')
    {
        return None;
    }

    Some(format!(
        "Code after `{}` at line {} may be unreachable",
        prev_line,
        idx + 1 // 1-indexed for user display
    ))
}

fn count_opening_braces(s: &str) -> i32 {
    s.chars().filter(|&c| c == '{').count() as i32
}

fn count_closing_braces(s: &str) -> i32 {
    s.chars().filter(|&c| c == '}').count() as i32
}

/// Detect long if-else chains that should be match statements.
pub fn check_if_else_chains(lines: &[&str]) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Check for `if` not preceded by `else` (start of a chain)
        // Must handle `if ... {` and not `} else if ... {` or `else if ... {`
        if (trimmed.starts_with("if ") || trimmed.starts_with("if("))
            && !trimmed.starts_with("else if")
            && !lines[i].contains("else if")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
        {
            let mut chain_len = 1;
            let start = i;

            // Look ahead for else if branches
            let mut j = i + 1;
            let mut brace_depth = count_opening_braces(trimmed) - count_closing_braces(trimmed);
            while j < lines.len() {
                let l = lines[j].trim();

                // Track brace depth
                brace_depth += count_opening_braces(l) - count_closing_braces(l);
                let has_else_if = l.contains("else if");
                let has_else_block = (l.contains("else {") || l.contains("else{")) && !l.contains("else if");

                if has_else_if && !has_else_block {
                    // We should only count a new branch if the else-if starts at the
                    // same or lower indentation as the original if (i.e., same scope level)
                    if brace_depth <= 1 {
                        chain_len += 1;
                    }
                } else if has_else_block && brace_depth <= 1 {
                    // Final `else { }` is not counted in chain length — it's the
                    // catch-all, not a condition. We just note the chain ends here.
                    break;
                }

                // If brace depth drops to 0 (or below), we've exited the entire
                // if-else construct
                if brace_depth <= 0 && (l == "}" || l.starts_with("}")) && !has_else_if {
                    break;
                }

                j += 1;
            }

            if chain_len >= 3 {
                violations.push(RuleViolation {
                    rule: "ast:if-else-chain".into(),
                    description: format!(
                        "Long if-else chain with {} branches starting at line {} — consider `match`",
                        chain_len,
                        start + 1
                    ),
                    severity: ViolationSeverity::Warning,
                });
            }
        }
        i += 1;
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_use_helper_detected() {
        let source = r#"
fn helper_parse_token(s: &str) -> &str {
    s.trim()
}

fn main() {
    let data = " hello ";
    let cleaned = helper_parse_token(data);
    println!("{}", cleaned);
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let mut fn_defs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut fn_calls: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if super::super::fn_analysis::is_fn_def(line)
                && let Some(name) = super::super::fn_analysis::extract_fn_name(line)
            {
                fn_defs.entry(name).or_default().push(i);
            }
        }
        let defined_names: Vec<String> = fn_defs.keys().cloned().collect();
        for (i, line) in lines.iter().enumerate() {
            let calls = super::super::fn_analysis::extract_fn_calls(line, &defined_names);
            for call in calls {
                fn_calls.entry(call).or_default().push(i);
            }
        }

        let violations = check_single_use_helpers(&fn_defs, &fn_calls, &lines);
        let single_use: Vec<_> = violations
            .iter()
            .filter(|v| v.rule == "ast:single-use-helper")
            .collect();
        assert_eq!(single_use.len(), 1, "Should detect single-use helper function");
        assert!(single_use[0].description.contains("helper_parse_token"));
    }

    #[test]
    fn test_over_abstraction_detected() {
        let source = r#"
fn do_real_work(x: i32) -> i32 {
    x * 2 + 1
}

fn thin_wrapper(value: i32) -> i32 {
    do_real_work(value)
}

fn another_thin_wrapper(a: i32) -> i32 {
    do_real_work(a)
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let mut fn_defs: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, line) in lines.iter().enumerate() {
            if super::super::fn_analysis::is_fn_def(line)
                && let Some(name) = super::super::fn_analysis::extract_fn_name(line)
            {
                fn_defs.entry(name).or_default().push(i);
            }
        }

        let violations = check_over_abstraction(&fn_defs, &lines);
        let wrappers: Vec<_> = violations.iter().filter(|v| v.rule == "ast:over-abstraction").collect();
        assert_eq!(wrappers.len(), 2, "Should detect both thin wrappers");
    }

    #[test]
    fn test_unreachable_code_detected() {
        let source = r#"
fn example() {
    let x = 42;
    return;
    let y = x + 1;
    println!("{}", y);
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let mut violations = Vec::new();
        for i in 0..lines.len() {
            if let Some(desc) = has_unreachable_after(&lines, i) {
                violations.push(RuleViolation {
                    rule: "ast:unreachable-code".into(),
                    description: desc,
                    severity: ViolationSeverity::Warning,
                });
            }
        }
        assert!(!violations.is_empty(), "Should detect unreachable code after return");
    }

    #[test]
    fn test_if_else_chain_detected() {
        let source = r#"
fn classify(value: i32) -> &'static str {
    if value == 1 {
        "one"
    } else if value == 2 {
        "two"
    } else if value == 3 {
        "three"
    } else if value == 4 {
        "four"
    } else {
        "other"
    }
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let violations = check_if_else_chains(&lines);
        let chains: Vec<_> = violations.iter().filter(|v| v.rule == "ast:if-else-chain").collect();
        assert!(
            !chains.is_empty(),
            "Should detect long if-else chain, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_short_if_else_not_flagged() {
        let source = r#"
fn classify(value: i32) -> &'static str {
    if value == 1 {
        "one"
    } else if value == 2 {
        "two"
    } else {
        "other"
    }
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let violations = check_if_else_chains(&lines);
        let chains: Vec<_> = violations.iter().filter(|v| v.rule == "ast:if-else-chain").collect();
        assert!(
            chains.is_empty(),
            "Short if-else chain (2 branches) should not be flagged"
        );
    }

    #[test]
    fn test_unused_function_detected() {
        let source = r#"
fn helper_internal(x: i32) -> i32 {
    x * 2
}

pub fn called_fn() -> i32 {
    42
}

fn main() {
    let _ = called_fn();
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let mut fn_defs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut fn_calls: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if super::super::fn_analysis::is_fn_def(line)
                && let Some(name) = super::super::fn_analysis::extract_fn_name(line)
            {
                fn_defs.entry(name).or_default().push(i);
            }
        }
        let defined_names: Vec<String> = fn_defs.keys().cloned().collect();
        for (i, line) in lines.iter().enumerate() {
            let calls = super::super::fn_analysis::extract_fn_calls(line, &defined_names);
            for call in calls {
                fn_calls.entry(call).or_default().push(i);
            }
        }

        let violations = check_single_use_helpers(&fn_defs, &fn_calls, &lines);
        let unused: Vec<_> = violations.iter().filter(|v| v.rule == "ast:unused-function").collect();
        assert_eq!(unused.len(), 1, "Should detect one unused function");
        assert!(unused[0].description.contains("helper_internal"));
    }

    #[test]
    fn test_unreachable_after_return() {
        let lines: Vec<&str> = "\
return;
let x = 1;"
            .lines()
            .collect();
        let result = has_unreachable_after(&lines, 1);
        assert!(result.is_some(), "Should detect code after return");
    }

    #[test]
    fn test_unreachable_not_on_empty() {
        let lines: Vec<&str> = "\
return;
"
        .lines()
        .collect();
        let result = has_unreachable_after(&lines, 1);
        assert!(result.is_none(), "Empty line after return should not be flagged");
    }
}
