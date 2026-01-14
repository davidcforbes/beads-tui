/// Label utilities and validation helpers.
use std::collections::HashSet;

const ALLOWED_LABEL_CHARS: &[char] = &['#', '-', '_', '.', ':', '/'];

/// Split a comma-separated label string into normalized label values.
pub fn split_labels(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|label| label.trim())
        .filter(|label| !label.is_empty())
        .map(|label| label.to_string())
        .collect()
}

/// Normalize a label into a key used for collision detection and alias grouping.
pub fn normalize_label_key(label: &str) -> String {
    label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Split a label into a (dimension, value) pair when formatted as "dimension:value".
pub fn split_label_dimension(label: &str) -> Option<(String, String)> {
    let (dimension, value) = label.split_once(':')?;
    if dimension.is_empty() || value.is_empty() {
        return None;
    }
    Some((dimension.to_string(), value.to_string()))
}

/// Validate a single label string.
pub fn validate_label(label: &str) -> Result<(), String> {
    if label.trim().is_empty() {
        return Err("Label must not be empty".to_string());
    }

    if label.chars().any(|c| c.is_whitespace()) {
        return Err("Label must not contain spaces".to_string());
    }

    if label.contains(',') {
        return Err("Label must not contain commas".to_string());
    }

    if !label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ALLOWED_LABEL_CHARS.contains(&c))
    {
        return Err("Label contains unsupported characters".to_string());
    }

    if normalize_label_key(label).is_empty() {
        return Err("Label must include at least one letter or number".to_string());
    }

    Ok(())
}

/// Validate a list of labels, checking for invalid characters and collisions.
pub fn validate_labels(labels: &[String]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for label in labels {
        validate_label(label)?;

        let key = normalize_label_key(label);
        if !seen.insert(key) {
            return Err("Label name collides with an existing label".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_labels_trims_and_discards_empty() {
        let labels = split_labels(" bug, urgent , , high-priority ");
        assert_eq!(labels, vec!["bug", "urgent", "high-priority"]);
    }

    #[test]
    fn test_normalize_label_key_strips_punctuation() {
        assert_eq!(normalize_label_key("Bug-Fix"), "bugfix");
        assert_eq!(normalize_label_key("#bug_fix"), "bugfix");
    }

    #[test]
    fn test_split_label_dimension() {
        assert_eq!(
            split_label_dimension("state:patrol"),
            Some(("state".to_string(), "patrol".to_string()))
        );
        assert_eq!(split_label_dimension("state:"), None);
        assert_eq!(split_label_dimension("state"), None);
    }

    #[test]
    fn test_validate_label_allows_hash_and_dashes() {
        assert!(validate_label("#bug-fix").is_ok());
    }

    #[test]
    fn test_validate_label_rejects_spaces() {
        assert!(validate_label("bug fix").is_err());
    }

    #[test]
    fn test_validate_label_rejects_commas() {
        assert!(validate_label("bug,fix").is_err());
    }

    #[test]
    fn test_validate_label_rejects_invalid_chars() {
        assert!(validate_label("bug!").is_err());
    }

    #[test]
    fn test_validate_labels_detects_collisions() {
        let labels = vec!["bug-fix".to_string(), "bugfix".to_string()];
        assert!(validate_labels(&labels).is_err());
    }
}
