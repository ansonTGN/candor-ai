/// AST-level function analysis for slop detection.
///
/// Extracts function definitions, call references, and body scopes
/// using line-based source analysis (no syn dependency).
/// Check if a line looks like a function definition header.
pub fn is_fn_def(line: &str) -> bool {
    let trimmed = line.trim();
    // Skip lines that are comments or contain "#["
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#[") {
        return false;
    }
    // Match `fn name(...` at start (possibly with pub/async/unsafe/extern modifiers)
    let stripped = strip_visibility_and_modifiers(trimmed);
    stripped.starts_with("fn ")
        && stripped[3..]
            .trim_start()
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
}

pub fn strip_visibility_and_modifiers(s: &str) -> &str {
    let mut s = s.trim_start();
    loop {
        let before = s;
        for prefix in &[
            "pub ",
            "pub(crate) ",
            "pub(super) ",
            "pub(self) ",
            "async ",
            "unsafe ",
            "extern ",
            "const ",
            "default ",
            "override ",
        ] {
            if let Some(rest) = s.strip_prefix(prefix) {
                s = rest;
                break;
            }
        }
        // Also handle pub(in path) etc — simplified: just skip "pub(" ... ")"
        if s.starts_with("pub(")
            && let Some(close) = s.find(')')
        {
            s = s[close + 1..].trim_start();
        }
        if s == before {
            break;
        }
    }
    s
}

/// Extract function name from a `fn name(...` line.
pub fn extract_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let stripped = strip_visibility_and_modifiers(trimmed);
    if let Some(name_start) = stripped.strip_prefix("fn ") {
        let after_fn = name_start.trim_start();
        // name is up to '(' or '<' (generics) or ' ' (whitespace before parens in some edge cases)
        if let Some(paren) = after_fn.find('(') {
            let name_candidate = after_fn[..paren].trim();
            // Strip generics like 'name<T>'
            let name = name_candidate
                .split('<')
                .next()
                .unwrap_or(name_candidate)
                .trim();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract function call references from a line.
/// Ignores the definition site itself.
pub fn extract_fn_calls(line: &str, defined_fns: &[String]) -> Vec<String> {
    let mut calls = Vec::new();
    for fn_name in defined_fns {
        // Look for `fn_name(` pattern, but not `fn fn_name(` (definition)
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("fn {}", fn_name)) {
            continue;
        }
        // Simple detection: function_name followed by '('
        let search = format!("{}(", fn_name);
        if line.contains(&search) {
            calls.push(fn_name.clone());
        }
    }
    calls
}

/// Collect the lines comprising a function body, given the definition line.
pub fn collect_fn_body<'a>(lines: &'a [&str], def_line: usize) -> Vec<&'a str> {
    let mut body = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_body = false;

    for (i, line) in lines.iter().copied().enumerate().skip(def_line) {
        for ch in line.chars() {
            if ch == '{' {
                brace_depth += 1;
                in_body = true;
            } else if ch == '}' {
                brace_depth -= 1;
            }
        }
        if in_body {
            body.push(line);
            if brace_depth <= 0 && i > def_line {
                break;
            }
        }
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_def_extraction() {
        assert_eq!(extract_fn_name("fn foo() {}").as_deref(), Some("foo"));
        assert_eq!(
            extract_fn_name("pub fn bar(x: i32) -> i32").as_deref(),
            Some("bar")
        );
        assert_eq!(extract_fn_name("async fn baz()").as_deref(), Some("baz"));
        assert_eq!(
            extract_fn_name("pub async fn qux()").as_deref(),
            Some("qux")
        );
        assert_eq!(
            extract_fn_name("fn with_generics<T: Clone>(x: T)").as_deref(),
            Some("with_generics")
        );
        assert!(extract_fn_name("// just a comment").is_none());
        assert!(extract_fn_name("#[derive(Debug)]").is_none());
    }

    #[test]
    fn test_strip_modifiers() {
        assert_eq!(strip_visibility_and_modifiers("pub fn foo"), "fn foo");
        assert_eq!(
            strip_visibility_and_modifiers("pub(crate) fn bar"),
            "fn bar"
        );
        assert_eq!(strip_visibility_and_modifiers("async fn baz"), "fn baz");
        assert_eq!(
            strip_visibility_and_modifiers("pub unsafe fn qux"),
            "fn qux"
        );
    }

    #[test]
    fn test_fn_body_collection() {
        let lines: Vec<&str> = r#"
fn foo() {
    let x = 1;
    let y = 2;
    x + y
}
fn bar() {
    42
}
"#
        .lines()
        .collect();

        let body = collect_fn_body(&lines, 1);
        assert!(!body.is_empty(), "Should collect function body");
        assert!(body.iter().any(|l| l.contains("let x = 1")));
        assert!(body.iter().any(|l| l.contains("x + y")));

        let body2 = collect_fn_body(&lines, 6);
        assert!(
            body2.iter().any(|l| l.contains("42")),
            "Should collect bar's body"
        );
    }
}
