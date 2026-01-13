/// JSON parsing for beads CLI output
use super::{error::*, models::*};
use serde_json::Value;

/// Parse a list of issues from JSON output
pub fn parse_issue_list(json: &str) -> Result<Vec<Issue>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| BeadsError::Json(e, json.to_string()))?;

    if let Some(issues_array) = value.as_array() {
        issues_array
            .iter()
            .map(|v| serde_json::from_value(v.clone()).map_err(|e| BeadsError::Json(e, v.to_string())))
            .collect()
    } else if let Ok(issue) = serde_json::from_value::<Issue>(value.clone()) {
        // Single issue returned
        Ok(vec![issue])
    } else {
        Ok(vec![])
    }
}

/// Parse a single issue from JSON output
pub fn parse_issue(json: &str) -> Result<Issue> {
    serde_json::from_str(json).map_err(|e| BeadsError::Json(e, json.to_string()))
}

/// Parse create response to extract issue ID
pub fn parse_create_response(output: &str) -> Result<String> {
    // beads returns "Created beads-tui-xxxx: Title"
    for line in output.lines() {
        if line.contains("Created") || line.contains("✓") {
            if let Some(id_part) = line.split_whitespace().find(|s| s.starts_with("beads-")) {
                let id = id_part.trim_end_matches(':');
                return Ok(id.to_string());
            }
        }
    }

    Err(BeadsError::CommandError(
        "Failed to parse issue ID from create response".to_string(),
    ))
}

/// Parse statistics from JSON output
pub fn parse_stats(json: &str) -> Result<IssueStats> {
    serde_json::from_str(json).map_err(|e| BeadsError::Json(e, json.to_string()))
}

/// Parse labels from JSON output
pub fn parse_labels(json: &str) -> Result<Vec<Label>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| BeadsError::Json(e, json.to_string()))?;

    if let Some(labels_array) = value.as_array() {
        labels_array
            .iter()
            .map(|v| serde_json::from_value(v.clone()).map_err(|e| BeadsError::Json(e, v.to_string())))
            .collect()
    } else {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_response() {
        let output = "✓ Created beads-tui-abc123: Test Issue";
        let id = parse_create_response(output).unwrap();
        assert_eq!(id, "beads-tui-abc123");
    }

    #[test]
    fn test_parse_create_response_alt_format() {
        let output = "Created beads-tui-xyz789: Another Test";
        let id = parse_create_response(output).unwrap();
        assert_eq!(id, "beads-tui-xyz789");
    }
}
