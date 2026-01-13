use beads_tui::models::*;
use beads_tui::beads::models::{IssueStatus};

#[test]
fn test_issue_filter_logic() {
    let mut filter = IssueFilter::new();
    assert!(filter.is_empty());
    
    filter.status = Some(IssueStatus::Open);
    assert!(!filter.is_empty());
    
    filter.clear();
    assert!(filter.is_empty());
}

#[test]
fn test_app_state_tabs() {
    let mut state = AppState::new();
    assert_eq!(state.selected_tab, 0);
    
    state.next_tab();
    assert_eq!(state.selected_tab, 1);
    
    state.previous_tab();
    assert_eq!(state.selected_tab, 0);
}

#[test]
fn test_navigation_history() {
    let mut stack = NavigationStack::new();
    // Default constructor adds IssueList as first element
    assert_eq!(stack.current(), &View::IssueList);
    
    stack.push(View::Labels);
    assert_eq!(stack.current(), &View::Labels);
    
    stack.push(View::Database);
    assert_eq!(stack.current(), &View::Database);
    
    assert_eq!(stack.back(), Some(&View::Labels));
    assert_eq!(stack.forward(), Some(&View::Database));
}