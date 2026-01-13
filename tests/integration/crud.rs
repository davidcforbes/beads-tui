//! Integration tests for beads-tui

use crate::common::TestHarness;
use beads_tui::beads::BeadsClient;
use std::process::Command;

#[tokio::test]
async fn test_beads_client_creation() {
    let _client = BeadsClient::new();
    assert!(BeadsClient::check_available().is_ok());
}

#[test]
fn test_config_load() {
    // Config should load even if file doesn't exist (returns defaults)
    let result = beads_tui::config::Config::load();
    assert!(result.is_ok());
}

// ========== CRUD Integration Tests ==========

#[tokio::test]
async fn test_crud_create_issue() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create a new issue
    let output = Command::new("bd")
        .args([
            "create",
            "--title",
            "Test Issue",
            "--type",
            "task",
            "--priority",
            "P2",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to create issue");

    assert!(output.status.success(), "Create command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created"), "Output should confirm creation");
    
    // Extract issue ID
    let issue_id = stdout
        .split_whitespace()
        .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
        .expect("Should find issue ID in output")
        .trim_end_matches(':');

    // Verify issue was created by listing
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");
    
    assert_eq!(issues.len(), 1, "Should have exactly 1 issue");
    assert_eq!(issues[0].id, issue_id, "Issue ID should match");
    assert_eq!(issues[0].title, "Test Issue", "Title should match");
}

#[tokio::test]
async fn test_crud_read_issue() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create an issue
    let output = Command::new("bd")
        .args([
            "create",
            "--title",
            "Read Test Issue",
            "--type",
            "bug",
            "--priority",
            "P1",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to create issue");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_id = stdout
        .split_whitespace()
        .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
        .unwrap()
        .trim_end_matches(':');

    // Read the issue via show command
    let show_output = Command::new("bd")
        .args(["show", issue_id])
        .current_dir(root)
        .output()
        .expect("Failed to show issue");

    assert!(show_output.status.success(), "Show command should succeed");
    
    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_stdout.contains("Read Test Issue"), "Should display issue title");
    assert!(show_stdout.contains("bug") || show_stdout.contains("Bug"), "Should display issue type");

    // Read via client API
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "Read Test Issue");
}

#[tokio::test]
async fn test_crud_update_issue() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create an issue
    let output = Command::new("bd")
        .args([
            "create",
            "--title",
            "Original Title",
            "--type",
            "task",
            "--priority",
            "P3",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to create issue");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_id = stdout
        .split_whitespace()
        .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
        .unwrap()
        .trim_end_matches(':');

    // Update the issue title
    let update_output = Command::new("bd")
        .args(["update", issue_id, "--title", "Updated Title"])
        .current_dir(root)
        .output()
        .expect("Failed to update issue");

    assert!(update_output.status.success(), "Update command should succeed");

    // Verify update via client
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "Updated Title", "Title should be updated");

    // Update status
    let status_output = Command::new("bd")
        .args(["update", issue_id, "--status", "in_progress"])
        .current_dir(root)
        .output()
        .expect("Failed to update status");

    assert!(status_output.status.success(), "Status update should succeed");

    // Verify status update
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert_eq!(issues[0].status.to_string(), "in_progress", "Status should be updated");

    // Update priority
    let priority_output = Command::new("bd")
        .args(["update", issue_id, "--priority", "P1"])
        .current_dir(root)
        .output()
        .expect("Failed to update priority");

    assert!(priority_output.status.success(), "Priority update should succeed");

    // Verify priority update
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert_eq!(issues[0].priority.to_string(), "P1", "Priority should be updated");
}

#[tokio::test]
async fn test_crud_delete_issue() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create multiple issues
    let create = |title: &str| {
        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .expect("Failed to create issue");
        let s = String::from_utf8_lossy(&output.stdout);
        s.split_whitespace()
            .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
            .unwrap()
            .trim_end_matches(':')
            .to_string()
    };

    let id1 = create("Issue 1");
    let id2 = create("Issue 2");
    let id3 = create("Issue 3");

    // Verify all created
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");
    assert_eq!(issues.len(), 3, "Should have 3 issues");

    // Delete one issue (requires --force to actually delete)
    let delete_output = Command::new("bd")
        .args(["delete", &id2, "--force"])
        .current_dir(root)
        .output()
        .expect("Failed to delete issue");

    assert!(delete_output.status.success(), "Delete command should succeed");

    // Verify deletion
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");
    
    assert_eq!(issues.len(), 2, "Should have 2 issues after deletion");
    assert!(
        issues.iter().any(|i| i.id == id1),
        "Issue 1 should still exist"
    );
    assert!(
        !issues.iter().any(|i| i.id == id2),
        "Issue 2 should be deleted"
    );
    assert!(
        issues.iter().any(|i| i.id == id3),
        "Issue 3 should still exist"
    );
}

#[tokio::test]
async fn test_crud_list_with_filters() {
    use beads_tui::beads::models::IssueStatus;
    
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create issues with different statuses
    let create_issue = |title: &str, issue_type: &str, status: &str| {
        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                title,
                "--type",
                issue_type,
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .expect("Failed to create issue");
        
        assert!(output.status.success(), "Create command should succeed for {}", title);
        
        // Extract ID from create command output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let id = stdout
            .split_whitespace()
            .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
            .unwrap_or_else(|| panic!("Failed to find issue ID for {}. Output: {}", title, stdout))
            .trim_end_matches(':')
            .to_string();

        if status != "open" {
            let update_result = Command::new("bd")
                .args(["update", &id, "--status", status])
                .current_dir(root)
                .status()
                .expect("Failed to update status");
            assert!(update_result.success(), "Update command should succeed for {}", title);
        }
    };

    create_issue("Bug Issue", "bug", "open");
    create_issue("Feature Issue", "feature", "in_progress");
    create_issue("Task Issue", "task", "closed");
    create_issue("Another Task", "task", "open");

    // List all issues
    let all_issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list all issues");
    assert_eq!(all_issues.len(), 4, "Should have 4 total issues");

    // List by status - open
    let open_issues = harness
        .client
        .list_issues(Some(IssueStatus::Open), None)
        .await
        .expect("Failed to list open issues");
    assert_eq!(open_issues.len(), 2, "Should have 2 open issues");

    // List by status - closed
    let closed_issues = harness
        .client
        .list_issues(Some(IssueStatus::Closed), None)
        .await
        .expect("Failed to list closed issues");
    assert_eq!(closed_issues.len(), 1, "Should have 1 closed issue");

    // List by status - in_progress
    let in_progress_issues = harness
        .client
        .list_issues(Some(IssueStatus::InProgress), None)
        .await
        .expect("Failed to list in_progress issues");
    assert_eq!(in_progress_issues.len(), 1, "Should have 1 in_progress issue");

    // List with limit
    let limited_issues = harness
        .client
        .list_issues(None, Some(2))
        .await
        .expect("Failed to list with limit");
    assert_eq!(limited_issues.len(), 2, "Should have 2 issues with limit");
}

#[tokio::test]
async fn test_crud_bulk_operations() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    // Create multiple issues in bulk
    let mut ids = Vec::new();
    for i in 1..=5 {
        let title = format!("Bulk Issue {}", i);
        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                &title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .expect("Failed to create issue");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let id = stdout
            .split_whitespace()
            .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
            .unwrap()
            .trim_end_matches(':')
            .to_string();
        ids.push(id);
    }

    // Verify all created
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");
    assert_eq!(issues.len(), 5, "Should have 5 issues");

    // Bulk update status
    for id in &ids[0..3] {
        Command::new("bd")
            .args(["update", id, "--status", "in_progress"])
            .current_dir(root)
            .status()
            .expect("Failed to update status");
    }

    // Verify bulk update
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");
    
    let in_progress_count = issues
        .iter()
        .filter(|i| i.status.to_string() == "in_progress")
        .count();
    assert_eq!(in_progress_count, 3, "Should have 3 in_progress issues");
}
