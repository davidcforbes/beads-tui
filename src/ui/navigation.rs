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
}
