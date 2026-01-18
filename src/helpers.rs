//! Helper functions for issues view operations

use crate::beads;
use crate::models;
use crate::runtime;
use crate::ui;

/// Reorder child issue within parent's blocks list
pub fn reorder_child_issue(app: &mut models::AppState, direction: i32) {
    use crate::beads::models::Issue;
    let selected_issue = match app.issues_view_state.search_state().selected_issue() {
        Some(issue) => Issue::clone(issue),
        None => {
            app.set_info("No issue selected".to_string());
            return;
        }
    };

    let all_issues = app.issues_view_state.all_issues();
    let parent = all_issues
        .iter()
        .find(|issue| issue.blocks.contains(&selected_issue.id));

    let parent = match parent {
        Some(issue) => issue,
        None => {
            app.set_info("Selected issue has no parent".to_string());
            return;
        }
    };

    let mut new_order = parent.blocks.clone();
    let current_idx = match new_order.iter().position(|id| id == &selected_issue.id) {
        Some(idx) => idx,
        None => {
            app.set_error("Selected issue not found in parent blocks".to_string());
            return;
        }
    };

    if new_order.len() < 2 {
        app.set_info("Parent has only one child".to_string());
        return;
    }

    let target_idx = if direction < 0 {
        if current_idx == 0 {
            app.set_info("Already at the top".to_string());
            return;
        }
        current_idx - 1
    } else {
        if current_idx + 1 >= new_order.len() {
            app.set_info("Already at the bottom".to_string());
            return;
        }
        current_idx + 1
    };

    new_order.swap(current_idx, target_idx);

    let parent_id = parent.id.clone();
    let current_children = parent.blocks.clone();
    // Using global runtime instead of creating new runtime
    let client = &app.beads_client;

    let mut removed: Vec<String> = Vec::new();
    for child_id in &current_children {
        if let Err(e) = runtime::RUNTIME.block_on(client.remove_dependency(child_id, &parent_id)) {
            for restored_id in &removed {
                let _ = runtime::RUNTIME.block_on(client.add_dependency(restored_id, &parent_id));
            }
            app.set_error(format!("Failed to reorder children: {e}"));
            return;
        }
        removed.push(child_id.clone());
    }

    let mut added: Vec<String> = Vec::new();
    for child_id in &new_order {
        if let Err(e) = runtime::RUNTIME.block_on(client.add_dependency(child_id, &parent_id)) {
            for added_id in &added {
                let _ = runtime::RUNTIME.block_on(client.remove_dependency(added_id, &parent_id));
            }
            for restored_id in &current_children {
                let _ = runtime::RUNTIME.block_on(client.add_dependency(restored_id, &parent_id));
            }
            app.set_error(format!("Failed to reorder children: {e}"));
            return;
        }
        added.push(child_id.clone());
    }

    app.set_success(format!("Reordered children under {}", parent_id));
    app.reload_issues();

    let search_state = app.issues_view_state.search_state_mut();
    if let Some(idx) = search_state
        .filtered_issues()
        .iter()
        .position(|issue| issue.id == selected_issue.id)
    {
        search_state.list_state_mut().select(Some(idx));
    }
}

/// Collect unique statuses from issues
pub fn collect_unique_statuses(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::IssueStatus> {
    use std::collections::HashSet;
    let mut statuses: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.status)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    statuses.sort();
    statuses
}

/// Collect unique priorities from issues
pub fn collect_unique_priorities(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::Priority> {
    use std::collections::HashSet;
    let mut priorities: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.priority)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    priorities.sort();
    priorities
}

/// Collect unique types from issues
pub fn collect_unique_types(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::IssueType> {
    use std::collections::HashSet;
    let mut types: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.issue_type)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    types.sort();
    types
}

/// Collect unique labels from issues
pub fn collect_unique_labels(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    let mut labels: Vec<_> = issues_state
        .all_issues()
        .iter()
        .flat_map(|i| i.labels.iter().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    labels.sort();
    labels
}

/// Collect unique assignees from issues
pub fn collect_unique_assignees(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    let mut assignees: Vec<_> = issues_state
        .all_issues()
        .iter()
        .filter_map(|i| i.assignee.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    assignees.push("-".to_string()); // Add option for unassigned
    assignees.sort();
    assignees
}

/// Collect unique created dates from issues
pub fn collect_unique_created_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| format!("{:04}-{:02}-{:02}",
            i.created.year(),
            i.created.month(),
            i.created.day()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}

/// Collect unique updated dates from issues
pub fn collect_unique_updated_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| format!("{:04}-{:02}-{:02}",
            i.updated.year(),
            i.updated.month(),
            i.updated.day()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}

/// Collect unique closed dates from issues
pub fn collect_unique_closed_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .filter_map(|i| i.closed.as_ref().map(|c| format!("{:04}-{:02}-{:02}",
            c.year(),
            c.month(),
            c.day())))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}
