//! Navigation system for beads-tui
//!
//! Manages view navigation, breadcrumbs, and history

use std::collections::VecDeque;

/// View identifier for navigation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum View {
    Issues,
    IssueDetail(String),
    Dependencies,
    DependencyGraph(String),
    Labels,
    Database,
    Help,
    Settings,
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            View::Issues => write!(f, "Issues"),
            View::IssueDetail(id) => write!(f, "Issue: {id}"),
            View::Dependencies => write!(f, "Dependencies"),
            View::DependencyGraph(id) => write!(f, "Dependency Graph: {id}"),
            View::Labels => write!(f, "Labels"),
            View::Database => write!(f, "Database"),
            View::Help => write!(f, "Help"),
            View::Settings => write!(f, "Settings"),
        }
    }
}

/// Navigation history manager
pub struct NavigationHistory {
    history: VecDeque<View>,
    current_index: usize,
    max_history: usize,
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self::new(50)
    }
}

impl NavigationHistory {
    /// Create a new navigation history with maximum size
    pub fn new(max_history: usize) -> Self {
        let mut history = VecDeque::with_capacity(max_history + 1);
        history.push_back(View::Issues);
        Self {
            history,
            current_index: 0,
            max_history,
        }
    }

    /// Get the current view
    pub fn current(&self) -> &View {
        &self.history[self.current_index]
    }

    /// Navigate to a new view
    pub fn push(&mut self, view: View) {
        // Remove any forward history when navigating to a new view
        self.history.truncate(self.current_index + 1);

        // Add new view
        self.history.push_back(view);

        // Maintain max history size
        if self.history.len() > self.max_history {
            self.history.pop_front();
        } else {
            self.current_index += 1;
        }
    }

    /// Go back in history
    pub fn back(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    /// Go forward in history
    pub fn forward(&mut self) -> bool {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// Check if we can go forward
    pub fn can_go_forward(&self) -> bool {
        self.current_index + 1 < self.history.len()
    }

    /// Get breadcrumb trail for current view
    pub fn breadcrumbs(&self) -> Vec<String> {
        let mut crumbs = vec!["Home".to_string()];

        match self.current() {
            View::Issues => crumbs.push("Issues".to_string()),
            View::IssueDetail(id) => {
                crumbs.push("Issues".to_string());
                crumbs.push(id.clone());
            }
            View::Dependencies => crumbs.push("Dependencies".to_string()),
            View::DependencyGraph(id) => {
                crumbs.push("Dependencies".to_string());
                crumbs.push(format!("Graph: {id}"));
            }
            View::Labels => crumbs.push("Labels".to_string()),
            View::Database => crumbs.push("Database".to_string()),
            View::Help => crumbs.push("Help".to_string()),
            View::Settings => crumbs.push("Settings".to_string()),
        }

        crumbs
    }

    /// Get history for display (limited to last N items)
    pub fn get_recent_history(&self, count: usize) -> Vec<(usize, &View)> {
        let start = if self.history.len() > count {
            self.history.len() - count
        } else {
            0
        };

        self.history.iter().enumerate().skip(start).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history() {
        let mut nav = NavigationHistory::new(10);

        assert_eq!(nav.current(), &View::Issues);
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());

        nav.push(View::IssueDetail("beads-001".to_string()));
        assert_eq!(nav.current(), &View::IssueDetail("beads-001".to_string()));
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());

        nav.back();
        assert_eq!(nav.current(), &View::Issues);
        assert!(!nav.can_go_back());
        assert!(nav.can_go_forward());

        nav.forward();
        assert_eq!(nav.current(), &View::IssueDetail("beads-001".to_string()));
    }

    #[test]
    fn test_breadcrumbs() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::IssueDetail("beads-001".to_string()));

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Issues", "beads-001"]);
    }

    #[test]
    fn test_view_display_issues() {
        let view = View::Issues;
        assert_eq!(view.to_string(), "Issues");
    }

    #[test]
    fn test_view_display_issue_detail() {
        let view = View::IssueDetail("beads-123".to_string());
        assert_eq!(view.to_string(), "Issue: beads-123");
    }

    #[test]
    fn test_view_display_dependencies() {
        let view = View::Dependencies;
        assert_eq!(view.to_string(), "Dependencies");
    }

    #[test]
    fn test_view_display_dependency_graph() {
        let view = View::DependencyGraph("beads-456".to_string());
        assert_eq!(view.to_string(), "Dependency Graph: beads-456");
    }

    #[test]
    fn test_view_display_labels() {
        let view = View::Labels;
        assert_eq!(view.to_string(), "Labels");
    }

    #[test]
    fn test_view_display_database() {
        let view = View::Database;
        assert_eq!(view.to_string(), "Database");
    }

    #[test]
    fn test_view_display_help() {
        let view = View::Help;
        assert_eq!(view.to_string(), "Help");
    }

    #[test]
    fn test_view_display_settings() {
        let view = View::Settings;
        assert_eq!(view.to_string(), "Settings");
    }

    #[test]
    fn test_navigation_history_new() {
        let nav = NavigationHistory::new(25);
        assert_eq!(nav.current(), &View::Issues);
        assert_eq!(nav.max_history, 25);
        assert_eq!(nav.current_index, 0);
    }

    #[test]
    fn test_navigation_history_default() {
        let nav = NavigationHistory::default();
        assert_eq!(nav.current(), &View::Issues);
        assert_eq!(nav.max_history, 50);
    }

    #[test]
    fn test_push_truncates_forward_history() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);
        nav.push(View::Help);
        nav.back();
        nav.back();

        // At View::Issues, forward history has Labels and Help
        assert_eq!(nav.current(), &View::Issues);
        assert!(nav.can_go_forward());

        // Push new view, should truncate forward history
        nav.push(View::Dependencies);
        assert_eq!(nav.current(), &View::Dependencies);
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_push_maintains_max_history() {
        let mut nav = NavigationHistory::new(3);

        nav.push(View::Labels);
        nav.push(View::Help);
        nav.push(View::Settings);

        // History should be [Issues, Labels, Help, Settings] but max is 3
        // So it should drop Issues from the front
        assert_eq!(nav.history.len(), 3);

        // Go back twice to get to first item
        nav.back();
        nav.back();

        // First item should be Labels (Issues was dropped)
        assert_eq!(nav.current(), &View::Labels);
    }

    #[test]
    fn test_back_at_start() {
        let mut nav = NavigationHistory::new(10);
        assert!(!nav.back());
        assert_eq!(nav.current(), &View::Issues);
    }

    #[test]
    fn test_forward_at_end() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);

        assert!(!nav.forward());
        assert_eq!(nav.current(), &View::Labels);
    }

    #[test]
    fn test_multiple_back_forward() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);
        nav.push(View::Help);
        nav.push(View::Settings);

        nav.back();
        nav.back();
        assert_eq!(nav.current(), &View::Labels);

        nav.forward();
        assert_eq!(nav.current(), &View::Help);
    }

    #[test]
    fn test_breadcrumbs_dependencies() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Dependencies);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Dependencies"]);
    }

    #[test]
    fn test_breadcrumbs_dependency_graph() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::DependencyGraph("beads-789".to_string()));

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Dependencies", "Graph: beads-789"]);
    }

    #[test]
    fn test_breadcrumbs_labels() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Labels"]);
    }

    #[test]
    fn test_breadcrumbs_database() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Database);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Database"]);
    }

    #[test]
    fn test_breadcrumbs_help() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Help);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Help"]);
    }

    #[test]
    fn test_breadcrumbs_settings() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Settings);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Settings"]);
    }

    #[test]
    fn test_breadcrumbs_issues() {
        let nav = NavigationHistory::new(10);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs, vec!["Home", "Issues"]);
    }

    #[test]
    fn test_get_recent_history_with_small_count() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);
        nav.push(View::Help);
        nav.push(View::Settings);

        let recent = nav.get_recent_history(2);
        assert_eq!(recent.len(), 2);

        // Should get last 2 items
        assert_eq!(recent[0].1, &View::Help);
        assert_eq!(recent[1].1, &View::Settings);
    }

    #[test]
    fn test_get_recent_history_with_large_count() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);
        nav.push(View::Help);

        let recent = nav.get_recent_history(10);
        assert_eq!(recent.len(), 3); // Issues, Labels, Help

        assert_eq!(recent[0].1, &View::Issues);
        assert_eq!(recent[1].1, &View::Labels);
        assert_eq!(recent[2].1, &View::Help);
    }

    #[test]
    fn test_get_recent_history_with_zero_count() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);

        let recent = nav.get_recent_history(0);
        assert_eq!(recent.len(), 0);
    }

    #[test]
    fn test_get_recent_history_includes_indices() {
        let mut nav = NavigationHistory::new(10);
        nav.push(View::Labels);
        nav.push(View::Help);

        let recent = nav.get_recent_history(3);

        // Check that indices are correct
        assert_eq!(recent[0].0, 0);
        assert_eq!(recent[1].0, 1);
        assert_eq!(recent[2].0, 2);
    }

    #[test]
    fn test_view_equality() {
        let view1 = View::IssueDetail("beads-123".to_string());
        let view2 = View::IssueDetail("beads-123".to_string());
        let view3 = View::IssueDetail("beads-456".to_string());

        assert_eq!(view1, view2);
        assert_ne!(view1, view3);
    }

    #[test]
    fn test_navigation_history_with_single_item() {
        let nav = NavigationHistory::new(10);

        assert_eq!(nav.history.len(), 1);
        assert_eq!(nav.current(), &View::Issues);
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());
    }
}
