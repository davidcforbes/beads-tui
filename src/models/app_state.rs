/// Application state management
use crate::beads::BeadsClient;
use crate::ui::views::{
    compute_label_stats, DatabaseStats, DatabaseStatus, HelpSection, IssuesViewState, LabelStats,
};
use crate::ui::widgets::DialogState;

use super::PerfStats;

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
    /// Dirty flag to track whether UI needs redrawing
    dirty: bool,
    /// Performance profiling statistics
    pub perf_stats: PerfStats,
    /// Whether to show performance stats in UI
    pub show_perf_stats: bool,
    /// Selected help section
    pub help_section: HelpSection,
    /// Dialog state for confirmations
    pub dialog_state: Option<DialogState>,
    /// Pending action waiting for dialog confirmation
    pub pending_action: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let beads_client = BeadsClient::new();

        // Load issues on startup
        let issues = Self::load_issues_sync(&beads_client);

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
            dirty: true, // Initial render required
            perf_stats: PerfStats::new(),
            show_perf_stats: false,
            help_section: HelpSection::Global,
            dialog_state: None,
            pending_action: None,
        }
    }

    /// Load issues synchronously using tokio runtime
    fn load_issues_sync(client: &BeadsClient) -> Vec<crate::beads::models::Issue> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(client.list_issues(None, None))
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load issues: {:?}", e);
                vec![]
            })
    }

    /// Reload issues from beads database
    pub fn reload_issues(&mut self) {
        let issues = Self::load_issues_sync(&self.beads_client);
        
        // Update label statistics
        self.label_stats = compute_label_stats(&issues);
        
        // Update database stats
        self.database_stats.total_issues = issues.len();
        
        // Update issues view
        self.issues_view_state.set_issues(issues);
        
        // Mark dirty to trigger redraw
        self.mark_dirty();
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
        self.mark_dirty();
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.tabs.len() - 1;
        }
        self.mark_dirty();
    }

    /// Mark the UI as dirty, requiring a redraw
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if UI needs redrawing
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag after rendering
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Toggle performance stats display
    pub fn toggle_perf_stats(&mut self) {
        self.show_perf_stats = !self.show_perf_stats;
        if self.show_perf_stats && !self.perf_stats.is_enabled() {
            self.perf_stats.set_enabled(true);
        }
        self.mark_dirty();
    }

    /// Navigate to next help section
    pub fn next_help_section(&mut self) {
        let sections = HelpSection::all();
        let current_idx = sections.iter().position(|&s| s == self.help_section).unwrap_or(0);
        self.help_section = sections[(current_idx + 1) % sections.len()];
        self.mark_dirty();
    }

    /// Navigate to previous help section
    pub fn previous_help_section(&mut self) {
        let sections = HelpSection::all();
        let current_idx = sections.iter().position(|&s| s == self.help_section).unwrap_or(0);
        self.help_section = if current_idx == 0 {
            sections[sections.len() - 1]
        } else {
            sections[current_idx - 1]
        };
        self.mark_dirty();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
