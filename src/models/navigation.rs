/// Navigation state and history management

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    IssueList,
    IssueDetail(String), // Issue ID
    Dependencies,
    Labels,
    Database,
    Help,
}

#[derive(Debug)]
pub struct NavigationStack {
    history: Vec<View>,
    current: usize,
}

impl NavigationStack {
    pub fn new() -> Self {
        Self {
            history: vec![View::IssueList],
            current: 0,
        }
    }

    pub fn push(&mut self, view: View) {
        // Remove any forward history if we're not at the end
        self.history.truncate(self.current + 1);
        self.history.push(view);
        self.current += 1;
    }

    pub fn back(&mut self) -> Option<&View> {
        if self.current > 0 {
            self.current -= 1;
            Some(&self.history[self.current])
        } else {
            None
        }
    }

    pub fn forward(&mut self) -> Option<&View> {
        if self.current + 1 < self.history.len() {
            self.current += 1;
            Some(&self.history[self.current])
        } else {
            None
        }
    }

    pub fn current(&self) -> &View {
        &self.history[self.current]
    }

    pub fn can_go_back(&self) -> bool {
        self.current > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.current + 1 < self.history.len()
    }
}

impl Default for NavigationStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_stack_new() {
        let nav = NavigationStack::new();
        assert_eq!(nav.current(), &View::IssueList);
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_navigation_stack_default() {
        let nav = NavigationStack::default();
        assert_eq!(nav.current(), &View::IssueList);
    }

    #[test]
    fn test_push_single_view() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        assert_eq!(nav.current(), &View::Dependencies);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_push_multiple_views() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        nav.push(View::Labels);
        nav.push(View::Database);

        assert_eq!(nav.current(), &View::Database);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_back_navigation() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        nav.push(View::Labels);

        let view = nav.back();
        assert_eq!(view, Some(&View::Dependencies));
        assert_eq!(nav.current(), &View::Dependencies);
        assert!(nav.can_go_back());
        assert!(nav.can_go_forward());
    }

    #[test]
    fn test_back_to_start() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);

        nav.back();
        assert_eq!(nav.current(), &View::IssueList);
        assert!(!nav.can_go_back());
        assert!(nav.can_go_forward());
    }

    #[test]
    fn test_back_at_start_returns_none() {
        let mut nav = NavigationStack::new();
        let view = nav.back();
        assert_eq!(view, None);
        assert_eq!(nav.current(), &View::IssueList);
    }

    #[test]
    fn test_forward_navigation() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        nav.push(View::Labels);
        nav.back();

        let view = nav.forward();
        assert_eq!(view, Some(&View::Labels));
        assert_eq!(nav.current(), &View::Labels);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_forward_at_end_returns_none() {
        let mut nav = NavigationStack::new();
        let view = nav.forward();
        assert_eq!(view, None);
    }

    #[test]
    fn test_push_clears_forward_history() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        nav.push(View::Labels);
        nav.push(View::Database);

        // Go back twice
        nav.back();
        nav.back();

        assert_eq!(nav.current(), &View::Dependencies);
        assert!(nav.can_go_forward());

        // Push a new view - should clear forward history
        nav.push(View::Help);

        assert_eq!(nav.current(), &View::Help);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());

        // Forward should not work anymore
        let view = nav.forward();
        assert_eq!(view, None);
    }

    #[test]
    fn test_back_and_forward_sequence() {
        let mut nav = NavigationStack::new();
        nav.push(View::Dependencies);
        nav.push(View::Labels);
        nav.push(View::Database);

        // Current: Database
        assert_eq!(nav.current(), &View::Database);

        // Back to Labels
        nav.back();
        assert_eq!(nav.current(), &View::Labels);

        // Back to Dependencies
        nav.back();
        assert_eq!(nav.current(), &View::Dependencies);

        // Forward to Labels
        nav.forward();
        assert_eq!(nav.current(), &View::Labels);

        // Forward to Database
        nav.forward();
        assert_eq!(nav.current(), &View::Database);
    }

    #[test]
    fn test_issue_detail_view() {
        let mut nav = NavigationStack::new();
        nav.push(View::IssueDetail("beads-123".to_string()));

        assert_eq!(
            nav.current(),
            &View::IssueDetail("beads-123".to_string())
        );
    }

    #[test]
    fn test_multiple_issue_detail_views() {
        let mut nav = NavigationStack::new();
        nav.push(View::IssueDetail("beads-001".to_string()));
        nav.push(View::IssueDetail("beads-002".to_string()));
        nav.push(View::IssueDetail("beads-003".to_string()));

        assert_eq!(
            nav.current(),
            &View::IssueDetail("beads-003".to_string())
        );

        nav.back();
        assert_eq!(
            nav.current(),
            &View::IssueDetail("beads-002".to_string())
        );

        nav.back();
        assert_eq!(
            nav.current(),
            &View::IssueDetail("beads-001".to_string())
        );
    }

    #[test]
    fn test_view_equality() {
        assert_eq!(View::IssueList, View::IssueList);
        assert_eq!(View::Dependencies, View::Dependencies);
        assert_eq!(
            View::IssueDetail("test".to_string()),
            View::IssueDetail("test".to_string())
        );
        assert_ne!(
            View::IssueDetail("test1".to_string()),
            View::IssueDetail("test2".to_string())
        );
        assert_ne!(View::IssueList, View::Dependencies);
    }
}
