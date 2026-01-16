/// Tests for filter state desync bug (beads-tui-95e0)
///
/// Verifies that filter state and selection remain consistent across
/// external issue updates (from CLI or another TUI instance).
use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Priority};
use beads_tui::ui::views::SearchInterfaceState;
use chrono::Utc;

/// Helper to create a test issue
fn create_test_issue(id: &str, title: &str, status: IssueStatus) -> Issue {
    Issue {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        status,
        priority: Priority::P2,
        issue_type: IssueType::Task,
        assignee: None,
        labels: vec![],
        created: Utc::now(),
        updated: Utc::now(),
        closed: None,
        dependencies: vec![],
        blocks: vec![],
        notes: vec![],
        ..Default::default()
    }
}

#[test]
fn test_selection_preserved_by_id_when_issues_reordered() {
    // Create initial issues in order: A, B, C
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Select issue B (index 1)
    state.list_state_mut().select(Some(1));
    assert_eq!(state.selected_issue().unwrap().id, "beads-bbb");

    // External update: issues reordered to C, A, B
    let reordered_issues = vec![
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
    ];

    state.set_issues(reordered_issues);

    // Selection should still be on issue B, now at index 2
    assert_eq!(state.list_state().selected(), Some(2));
    assert_eq!(state.selected_issue().unwrap().id, "beads-bbb");
}

#[test]
fn test_selection_cleared_when_selected_issue_removed() {
    // Create initial issues: A, B, C
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Select issue B (index 1)
    state.list_state_mut().select(Some(1));
    assert_eq!(state.selected_issue().unwrap().id, "beads-bbb");

    // External update: issue B removed, only A and C remain
    let updated_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    state.set_issues(updated_issues);

    // Selection should be cleared since issue B no longer exists
    assert_eq!(state.list_state().selected(), None);
}

#[test]
fn test_filter_state_preserved_across_reload() {
    // Create initial issues
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Closed),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Apply search query filter
    state.search_state_mut().set_query("Task A");
    state.update_filtered_issues();

    // Verify filter works: only Task A visible
    assert_eq!(state.filtered_issues().len(), 1);
    assert_eq!(state.filtered_issues()[0].id, "beads-aaa");

    // External update: add new issue D
    let updated_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Closed),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-ddd", "Task A Clone", IssueStatus::Open),
    ];

    state.set_issues(updated_issues);

    // Filter should still be active: shows Task A and Task A Clone
    assert_eq!(state.filtered_issues().len(), 2);
    assert_eq!(state.search_state().query(), "Task A");
}

#[test]
fn test_selection_preserved_with_filter_active() {
    // Create initial issues
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Bug A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Bug B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Apply search filter for "Bug"
    state.search_state_mut().set_query("Bug");
    state.update_filtered_issues();

    // Select Bug B (index 1 in filtered list)
    state.list_state_mut().select(Some(1));
    assert_eq!(state.selected_issue().unwrap().id, "beads-bbb");

    // External update: reorder all issues
    let reordered_issues = vec![
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-bbb", "Bug B", IssueStatus::Open),
        create_test_issue("beads-aaa", "Bug A", IssueStatus::Open),
    ];

    state.set_issues(reordered_issues);

    // Filter still shows Bug A and Bug B (in new order)
    assert_eq!(state.filtered_issues().len(), 2);

    // Selection should still be on Bug B, now at index 0 in filtered list
    assert_eq!(state.list_state().selected(), Some(0));
    assert_eq!(state.selected_issue().unwrap().id, "beads-bbb");
}

#[test]
fn test_selection_preserved_when_unfiltered_issues_change() {
    // Create initial issues
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Bug A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Bug B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Apply search filter for "Bug"
    state.search_state_mut().set_query("Bug");
    state.update_filtered_issues();

    // Select Bug A (index 0 in filtered list)
    state.list_state_mut().select(Some(0));
    assert_eq!(state.selected_issue().unwrap().id, "beads-aaa");

    // External update: add new task (won't appear in filter) and modify Task C
    let updated_issues = vec![
        create_test_issue("beads-aaa", "Bug A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Bug B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C Modified", IssueStatus::Open),
        create_test_issue("beads-ddd", "Task D", IssueStatus::Open),
    ];

    state.set_issues(updated_issues);

    // Filtered list unchanged (still just Bug A and Bug B)
    assert_eq!(state.filtered_issues().len(), 2);

    // Selection should still be on Bug A at index 0
    assert_eq!(state.list_state().selected(), Some(0));
    assert_eq!(state.selected_issue().unwrap().id, "beads-aaa");
}

#[test]
fn test_no_explicit_selection_change_preserved_across_reload() {
    // Create initial issues
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Default selection is Some(0) - selecting first item
    assert_eq!(state.list_state().selected(), Some(0));
    assert_eq!(state.selected_issue().unwrap().id, "beads-aaa");

    // External update: add new issues, reorder so A is still first
    let updated_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
    ];

    state.set_issues(updated_issues);

    // Selection should still be on Task A at index 0
    assert_eq!(state.list_state().selected(), Some(0));
    assert_eq!(state.selected_issue().unwrap().id, "beads-aaa");
}

#[test]
fn test_selection_stays_at_same_issue_when_earlier_issues_removed() {
    // Create initial issues: A, B, C, D, E
    let initial_issues = vec![
        create_test_issue("beads-aaa", "Task A", IssueStatus::Open),
        create_test_issue("beads-bbb", "Task B", IssueStatus::Open),
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-ddd", "Task D", IssueStatus::Open),
        create_test_issue("beads-eee", "Task E", IssueStatus::Open),
    ];

    let mut state = SearchInterfaceState::new(initial_issues);

    // Select Task D (index 3)
    state.list_state_mut().select(Some(3));
    assert_eq!(state.selected_issue().unwrap().id, "beads-ddd");

    // External update: remove A and B, leaving C, D, E
    let updated_issues = vec![
        create_test_issue("beads-ccc", "Task C", IssueStatus::Open),
        create_test_issue("beads-ddd", "Task D", IssueStatus::Open),
        create_test_issue("beads-eee", "Task E", IssueStatus::Open),
    ];

    state.set_issues(updated_issues);

    // Task D should still be selected, now at index 1
    assert_eq!(state.list_state().selected(), Some(1));
    assert_eq!(state.selected_issue().unwrap().id, "beads-ddd");
}
