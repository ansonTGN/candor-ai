/// AI narration comment pattern definitions and detection.
///
/// These are comments written in a "tutorial" voice that narrate
/// what the code is doing. They add noise without value in production code.
use crate::rules::{RuleViolation, ViolationSeverity};

/// Patterns that indicate AI-generated narration comments.
pub const NARRATION_PATTERNS: &[&str] = &[
    "now we",
    "here we",
    "let us",
    "let's",
    "first we",
    "next we",
    "this function",
    "this method",
    "this helper",
    "this utility",
    "we create",
    "we define",
    "we implement",
    "we set up",
    "we build",
    "we add",
    "we write",
    "we need",
    "we want",
    "we can",
    "we will",
    "we use",
    "we call",
    "we check",
    "we handle",
    "we return",
    "we parse",
    "we convert",
    "we transform",
    "we generate",
    "we process",
    "we take",
    "we start",
    "we begin",
    "essentially",
    "basically just",
    "simply a",
    "in other words",
    "that is to say",
    "as mentioned",
    "as noted",
    "as we saw",
    "the idea is",
    "the concept is",
    "basically,",
    "simply put",
    "in essence",
    "// now ",
    "// next, ",
];

/// Check for AI narration comments across the file.
pub fn check_narration_comments(lines: &[&str]) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Only check comments
        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with("*") {
            continue;
        }

        // Remove comment markers for pattern matching
        let comment_text = if let Some(s) = trimmed.strip_prefix("//") {
            s
        } else if let Some(s) = trimmed.strip_prefix("/*") {
            s
        } else if let Some(s) = trimmed.strip_prefix('*') {
            s
        } else {
            continue;
        };
        let comment_text = comment_text.to_lowercase();

        // Check each pattern
        for pattern in NARRATION_PATTERNS {
            if comment_text.contains(pattern) {
                violations.push(RuleViolation {
                    rule: "ast:narration-comment".into(),
                    description: format!("AI narration comment at line {}: matches pattern '{}'", i + 1, pattern),
                    severity: ViolationSeverity::Warning,
                });
                break; // One violation per comment line
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_narration_comments_detected() {
        let source = r#"
// Now we create the config parser
fn parse_config(input: &str) -> Config {
    // Here we implement the parsing logic
    Config { data: input.to_string() }
}

// This function handles the validation
fn validate(cfg: &Config) -> bool {
    // First we check the input
    cfg.data.len() > 0
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let violations = check_narration_comments(&lines);
        assert!(
            violations.len() >= 3,
            "Expected at least 3 narration comment violations, got {}",
            violations.len()
        );
    }
}
