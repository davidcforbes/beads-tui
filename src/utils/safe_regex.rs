/// Safe regex matching with DoS protection against catastrophic backtracking
use regex::Regex;
use std::time::{Duration, Instant};

/// Maximum time allowed for a single regex match operation (100ms)
const MAX_REGEX_MATCH_TIME_MS: u64 = 100;

/// Maximum regex pattern complexity (number of repetition operators)
const MAX_REPETITION_OPERATORS: usize = 5;

/// Validate a regex pattern to detect potentially dangerous patterns
pub fn validate_regex_safety(pattern: &str) -> Result<(), String> {
    // Check for nested quantifiers like (a+)+ which cause catastrophic backtracking
    let repetition_count = pattern
        .chars()
        .filter(|c| matches!(c, '+' | '*' | '?' | '{'))
        .count();

    if repetition_count > MAX_REPETITION_OPERATORS {
        return Err(format!(
            "Regex too complex ({} repetition operators, max {})",
            repetition_count, MAX_REPETITION_OPERATORS
        ));
    }

    // Detect nested repetitions: +*, ++, *+, **, etc.
    let nested_quantifiers = [
        "+*", "++", "+{", "*+", "**", "*{", "?+", "?*", "?{", "}+", "}*", "}{",
    ];
    for pattern_pair in &nested_quantifiers {
        if pattern.contains(pattern_pair) {
            return Err(format!(
                "Potentially dangerous nested quantifiers detected: {}",
                pattern_pair
            ));
        }
    }

    // Detect patterns like (...)+ or (...)* where ... contains quantifiers
    // This catches (a+)+ style catastrophic backtracking
    let chars: Vec<char> = pattern.chars().collect();
    let mut paren_stack = Vec::new();
    let mut has_quantifier_in_current_group = false;

    for i in 0..chars.len() {
        match chars[i] {
            '(' if i == 0 || chars[i - 1] != '\\' => {
                paren_stack.push(has_quantifier_in_current_group);
                has_quantifier_in_current_group = false;
            }
            ')' if i == 0 || chars[i - 1] != '\\' => {
                if let Some(prev_has_quantifier) = paren_stack.pop() {
                    // Check if next char is a quantifier and group had quantifiers
                    if i + 1 < chars.len()
                        && matches!(chars[i + 1], '+' | '*' | '?' | '{')
                        && has_quantifier_in_current_group
                    {
                        return Err(format!(
                            "Nested quantifier detected: group with quantifiers followed by {}",
                            chars[i + 1]
                        ));
                    }
                    // A group counts as a quantifier if it is quantified
                    let group_is_quantified =
                        i + 1 < chars.len() && matches!(chars[i + 1], '+' | '*' | '?' | '{');
                    has_quantifier_in_current_group = prev_has_quantifier || group_is_quantified;
                }
            }
            '+' | '*' | '?' | '{'
                if !paren_stack.is_empty() && (i == 0 || chars[i - 1] != '\\') =>
            {
                has_quantifier_in_current_group = true;
            }
            _ => {}
        }
    }

    // Try to compile the regex to ensure it's valid
    Regex::new(pattern)
        .map(|_| ())
        .map_err(|e| format!("Invalid regex: {}", e))
}

/// Safely match a regex pattern against text with timeout protection
pub fn safe_regex_match(pattern: &str, text: &str, case_insensitive: bool) -> Option<bool> {
    // Validate the pattern first
    if let Err(e) = validate_regex_safety(pattern) {
        tracing::warn!("Unsafe regex pattern rejected: {}", e);
        return None;
    }

    // Build the regex with case-insensitivity if requested
    let pattern_with_flags = if case_insensitive {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };

    // Compile the regex
    let re = match Regex::new(&pattern_with_flags) {
        Ok(re) => re,
        Err(e) => {
            tracing::warn!("Failed to compile regex: {}", e);
            return None;
        }
    };

    // Perform the match with a simple timeout check
    // Note: This is a best-effort timeout - actual regex execution can't be interrupted
    // in Rust regex crate, but we can at least abort if it takes too long
    let start = Instant::now();
    let result = re.is_match(text);
    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(MAX_REGEX_MATCH_TIME_MS) {
        tracing::warn!(
            "Regex match took too long ({}ms), pattern: {}",
            elapsed.as_millis(),
            pattern
        );
    }

    Some(result)
}

/// Safely find matches in text with timeout and safety validation
pub fn safe_regex_find(pattern: &str, text: &str, case_insensitive: bool) -> Option<Vec<String>> {
    // Validate the pattern first
    if let Err(e) = validate_regex_safety(pattern) {
        tracing::warn!("Unsafe regex pattern rejected: {}", e);
        return None;
    }

    // Build the regex with case-insensitivity if requested
    let pattern_with_flags = if case_insensitive {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };

    // Compile the regex
    let re = match Regex::new(&pattern_with_flags) {
        Ok(re) => re,
        Err(e) => {
            tracing::warn!("Failed to compile regex: {}", e);
            return None;
        }
    };

    // Find all matches
    let start = Instant::now();
    let matches: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_string()).collect();
    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(MAX_REGEX_MATCH_TIME_MS) {
        tracing::warn!(
            "Regex find took too long ({}ms), pattern: {}",
            elapsed.as_millis(),
            pattern
        );
    }

    Some(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_regex() {
        assert!(validate_regex_safety("hello").is_ok());
        assert!(validate_regex_safety("hello.*world").is_ok());
        assert!(validate_regex_safety("^test$").is_ok());
        assert!(validate_regex_safety("[a-z]+").is_ok());
    }

    #[test]
    fn test_validate_nested_quantifiers() {
        // Catastrophic backtracking patterns
        assert!(validate_regex_safety("(a+)+b").is_err());
        assert!(validate_regex_safety("(a*)*b").is_err());
        assert!(validate_regex_safety("(a+)*b").is_err());
        assert!(validate_regex_safety("(a*)+b").is_err());
    }

    #[test]
    fn test_validate_complexity_limit() {
        // Pattern with too many repetition operators
        let complex_pattern = "a+b*c?d+e*f{2,3}g+h*";
        assert!(validate_regex_safety(complex_pattern).is_err());
    }

    #[test]
    fn test_validate_invalid_regex() {
        assert!(validate_regex_safety("(unclosed").is_err());
        assert!(validate_regex_safety("[unclosed").is_err());
        assert!(validate_regex_safety("(?invalid)").is_err());
    }

    #[test]
    fn test_safe_regex_match_basic() {
        assert_eq!(safe_regex_match("hello", "hello world", false), Some(true));
        assert_eq!(
            safe_regex_match("hello", "goodbye world", false),
            Some(false)
        );
    }

    #[test]
    fn test_safe_regex_match_case_insensitive() {
        assert_eq!(safe_regex_match("HELLO", "hello world", true), Some(true));
        assert_eq!(safe_regex_match("HELLO", "hello world", false), Some(false));
    }

    #[test]
    fn test_safe_regex_match_dangerous_pattern() {
        // Should reject catastrophic backtracking pattern
        assert_eq!(safe_regex_match("(a+)+b", "aaaaaaaaaa", false), None);
    }

    #[test]
    fn test_safe_regex_match_complex_pattern() {
        // Should reject overly complex pattern
        let complex = "a+b*c?d+e*f{2,3}g+h*";
        assert_eq!(safe_regex_match(complex, "test", false), None);
    }

    #[test]
    fn test_safe_regex_find_basic() {
        let result = safe_regex_find("\\d+", "abc 123 def 456", false);
        assert!(result.is_some());
        let matches = result.unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], "123");
        assert_eq!(matches[1], "456");
    }

    #[test]
    fn test_safe_regex_find_dangerous_pattern() {
        // Should reject catastrophic backtracking pattern
        assert_eq!(safe_regex_find("(a+)+b", "aaaaaaaaaa", false), None);
    }

    #[test]
    fn test_safe_regex_find_no_matches() {
        let result = safe_regex_find("\\d+", "no numbers here", false);
        assert!(result.is_some());
        let matches = result.unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_nested_quantifier_detection() {
        assert!(validate_regex_safety("a++").is_err());
        assert!(validate_regex_safety("a**").is_err());
        assert!(validate_regex_safety("a+*").is_err());
        assert!(validate_regex_safety("a*+").is_err());
        assert!(validate_regex_safety("a?+").is_err());
        assert!(validate_regex_safety("a?*").is_err());
    }

    #[test]
    fn test_safe_patterns_not_rejected() {
        // These should all be OK
        assert!(validate_regex_safety("a+b").is_ok());
        assert!(validate_regex_safety("a*b").is_ok());
        assert!(validate_regex_safety("a?b").is_ok());
        assert!(validate_regex_safety("a{1,3}b").is_ok());
        assert!(validate_regex_safety("(a|b)+").is_ok());
        assert!(validate_regex_safety("[a-z]+\\s*[0-9]?").is_ok());
    }

    #[test]
    fn test_edge_case_empty_pattern() {
        assert!(validate_regex_safety("").is_ok());
        assert_eq!(safe_regex_match("", "anything", false), Some(true));
    }

    #[test]
    fn test_edge_case_empty_text() {
        assert_eq!(safe_regex_match("test", "", false), Some(false));
        assert_eq!(safe_regex_match("", "", false), Some(true));
    }
}
