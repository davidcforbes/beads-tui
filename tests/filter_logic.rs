use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Priority};
use beads_tui::ui::views::{SearchInterfaceState, SearchScope, ViewType};
use chrono::Utc;

fn create_test_issue(
    id: &str,
    title: &str,
    status: IssueStatus,
    assignee: Option<&str>,
    labels: Vec<&str>,
) -> Issue {
    Issue {
        id: id.to_string(),
        title: title.to_string(),
        status,
        priority: Priority::P2,
        issue_type: IssueType::Task,
        description: None,
        assignee: assignee.map(|s| s.to_string()),
        labels: labels.into_iter().map(|s| s.to_string()).collect(),
        dependencies: vec![],
        blocks: vec![],
        created: Utc::now(),
        updated: Utc::now(),
        closed: None,
        notes: vec![],
    }
}

#[test]
fn test_filter_combinations() {
    let issues = vec![
        create_test_issue(
            "1",
            "Bug in UI",
            IssueStatus::Open,
            Some("alice"),
            vec!["bug", "ui"],
        ),
        create_test_issue(
            "2",
            "Feature request",
            IssueStatus::InProgress,
            Some("bob"),
            vec!["feature"],
        ),
        create_test_issue(
            "3",
            "Backend crash",
            IssueStatus::Blocked,
            None,
            vec!["bug", "backend"],
        ),
        create_test_issue(
            "4",
            "Documentation",
            IssueStatus::Closed,
            Some("alice"),
            vec!["docs"],
        ),
    ];

    let mut state = SearchInterfaceState::new(issues);
    state.set_current_user(Some("alice".to_string()));

    // 1. Search by title
    state.search_state_mut().set_query("ui".to_string());
    state.update_filtered_issues();
    assert_eq!(state.filtered_issues().len(), 1);
    assert_eq!(state.filtered_issues()[0].id, "1");

    // 2. Filter by view (MyIssues)
    state.search_state_mut().set_query("".to_string());
    state.set_view(ViewType::MyIssues);
    state.update_filtered_issues();
    assert_eq!(state.filtered_issues().len(), 2); // 1 and 4

    // 3. Combine search and view
    state.search_state_mut().set_query("bug".to_string());
    state.update_filtered_issues();
    assert_eq!(state.filtered_issues().len(), 1); // Only 1

    // 4. Filter by status (Blocked)
    state.search_state_mut().set_query("".to_string());
    state.set_view(ViewType::Blocked);
    state.update_filtered_issues();
    assert_eq!(state.filtered_issues().len(), 1); // Only 3
}
