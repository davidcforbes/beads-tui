/// JSON parsing for beads CLI output
use super::{error::*, models::*};
use serde_json::Value;

/// Parse a list of issues from JSON output (optimized to avoid clones)
pub fn parse_issue_list(json: &str) -> Result<Vec<Issue>> {
    // Try to deserialize directly as Vec<Issue> first (most common case)
    match serde_json::from_str::<Vec<Issue>>(json) {
        Ok(issues) => return Ok(issues),
        Err(vec_err) => {
            // Fall back to single issue
            match serde_json::from_str::<Issue>(json) {
                Ok(issue) => return Ok(vec![issue]),
                Err(issue_err) => {
                    // Check if it's valid JSON at all (empty array/object is ok)
                    if let Ok(value) = serde_json::from_str::<Value>(json) {
                        // Valid JSON but not issues - return empty
                        if value.is_array() || value.is_object() {
                            return Ok(vec![]);
                        }
                    }
                    // Invalid JSON - return error from Vec attempt
                    return Err(BeadsError::Json(vec_err, json.to_string()));
                }
            }
        }
    }
}

/// Parse a single issue from JSON output
pub fn parse_issue(json: &str) -> Result<Issue> {
    serde_json::from_str(json).map_err(|e| BeadsError::Json(e, json.to_string()))
}

/// Parse create response to extract issue ID
pub fn parse_create_response(output: &str) -> Result<String> {
    // beads returns "Created beads-tui-xxxx: Title" or "✓ Created issue: beads-tui-xxxx"
    // In test environments, it may use temporary IDs like ".tmpXXXXXX-xxx"
    for line in output.lines() {
        if line.contains("Created") || line.contains("✓") {
            // Look for an ID: either starts with "beads-" or ".tmp" (test environment)
            if let Some(id_part) = line
                .split_whitespace()
                .find(|s| s.starts_with("beads-") || s.starts_with(".tmp"))
            {
                let id = id_part.trim_end_matches(':');
                return Ok(id.to_string());
            }
        }
    }

    // Log the full output for debugging but don't expose it to users
    tracing::error!(
        "Failed to parse issue ID from create response. Output:\n{}",
        output
    );
    Err(BeadsError::CommandError(
        "Failed to parse issue ID from create response. Check logs for details.".to_string(),
    ))
}

/// Parse statistics from JSON output
pub fn parse_stats(json: &str) -> Result<IssueStats> {
    serde_json::from_str(json).map_err(|e| BeadsError::Json(e, json.to_string()))
}

/// Parse labels from JSON output
pub fn parse_labels(json: &str) -> Result<Vec<Label>> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| BeadsError::Json(e, json.to_string()))?;

    if let Some(labels_array) = value.as_array() {
        labels_array
            .iter()
            .map(|v| {
                serde_json::from_value(v.clone()).map_err(|e| BeadsError::Json(e, v.to_string()))
            })
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

    #[test]
    fn test_parse_create_response_tmp_id() {
        let output = "✓ Created issue: .tmpXYZ123-abc";
        let id = parse_create_response(output).unwrap();
        assert_eq!(id, ".tmpXYZ123-abc");
    }

    #[test]
    fn test_parse_create_response_with_colon() {
        let output = "✓ Created issue: beads-tui-test123:";
        let id = parse_create_response(output).unwrap();
        assert_eq!(id, "beads-tui-test123");
    }

    #[test]
    fn test_parse_create_response_failure() {
        let output = "Error: Something went wrong";
        let result = parse_create_response(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_issue_list_array() {
        let json = r#"[
            {
                "id": "beads-1",
                "title": "Test 1",
                "status": "open",
                "priority": "P1",
                "issue_type": "bug",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z"
            },
            {
                "id": "beads-2",
                "title": "Test 2",
                "status": "closed",
                "priority": "P2",
                "issue_type": "feature",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z"
            }
        ]"#;

        let issues = parse_issue_list(json).unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, "beads-1");
        assert_eq!(issues[1].id, "beads-2");
    }

    #[test]
    fn test_parse_issue_list_single_object() {
        let json = r#"{
            "id": "beads-1",
            "title": "Test",
            "status": "open",
            "priority": "P1",
            "issue_type": "bug",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;

        let issues = parse_issue_list(json).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "beads-1");
    }

    #[test]
    fn test_parse_issue_list_empty() {
        let json = "[]";
        let issues = parse_issue_list(json).unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_parse_issue_list_invalid_json() {
        let json = "not valid json";
        let result = parse_issue_list(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_issue() {
        let json = r#"{
            "id": "beads-1",
            "title": "Test",
            "status": "open",
            "priority": "P1",
            "issue_type": "bug",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;

        let issue = parse_issue(json).unwrap();
        assert_eq!(issue.id, "beads-1");
        assert_eq!(issue.title, "Test");
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.priority, Priority::P1);
        assert_eq!(issue.issue_type, IssueType::Bug);
    }

    #[test]
    fn test_parse_issue_invalid() {
        let json = "{}";
        let result = parse_issue(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_stats() {
        let json = r#"{
            "total_issues": 100,
            "open": 20,
            "closed": 70,
            "blocked": 5,
            "in_progress": 5,
            "ready_to_work": 10,
            "avg_lead_time_hours": 24.5
        }"#;

        let stats = parse_stats(json).unwrap();
        assert_eq!(stats.total_issues, 100);
        assert_eq!(stats.open, 20);
        assert_eq!(stats.closed, 70);
        assert_eq!(stats.blocked, 5);
        assert_eq!(stats.in_progress, 5);
    }

    #[test]
    fn test_parse_stats_invalid() {
        let json = "{}";
        let result = parse_stats(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_labels() {
        let json = r#"[
            {"name": "bug", "count": 10},
            {"name": "feature", "count": 5}
        ]"#;

        let labels = parse_labels(json).unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[1].name, "feature");
    }

    #[test]
    fn test_parse_labels_empty() {
        let json = "[]";
        let labels = parse_labels(json).unwrap();
        assert_eq!(labels.len(), 0);
    }

    #[test]
    fn test_parse_labels_invalid_json() {
        let json = "not json";
        let result = parse_labels(json);
        assert!(result.is_err());
    }
}
