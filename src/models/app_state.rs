/// Application state management
use crate::beads::BeadsClient;
use crate::ui::views::{compute_label_stats, DatabaseStats, DatabaseStatus, IssuesViewState, LabelStats};

#[derive(Debug)]
pub struct AppState {
    pub should_quit: bool,
    pub selected_tab: usize,
    pub tabs: Vec<&'static str>,
    pub beads_client: BeadsClient,
    pub issues_view_state: IssuesViewState,
    pub label_stats: Vec<LabelStats>,
    pub database_stats: DatabaseStats,
    pub database_status: DatabaseStatus,
}

impl AppState {
    pub fn new() -> Self {
        let beads_client = BeadsClient::new();
        
        // TODO: Load issues asynchronously
        // For now, start with empty issues
        let issues = vec![];
        
        // Compute label statistics
        let label_stats = compute_label_stats(&issues);
        
        // Create database stats
        let database_stats = DatabaseStats {
            total_issues: issues.len(),
            open_issues: 0,
            closed_issues: 0,
            blocked_issues: 0,
            database_size: 0,
            last_sync: None,
        };
        
        Self {
            should_quit: false,
            selected_tab: 0,
            tabs: vec!["Issues", "Dependencies", "Labels", "Database", "Help"],
            beads_client,
            issues_view_state: IssuesViewState::new(issues),
            label_stats,
            database_stats,
            database_status: DatabaseStatus::Ready,
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.tabs.len() - 1;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
