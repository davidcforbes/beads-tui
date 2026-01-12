/// Application state management
use crate::beads::BeadsClient;
use crate::ui::views::{
    compute_label_stats, DatabaseStats, DatabaseStatus, DependenciesViewState, HelpSection,
    IssuesViewState, LabelStats, LabelsViewState,
};
use crate::ui::widgets::DialogState;

use super::PerfStats;

/// Notification message type for user feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Error,
    Success,
    Info,
    Warning,
}

/// Notification message with type and content
#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub message: String,
    pub notification_type: NotificationType,
}

#[derive(Debug)]
pub struct AppState {
    pub should_quit: bool,
    pub selected_tab: usize,
    pub tabs: Vec<&'static str>,
    pub beads_client: BeadsClient,
    pub issues_view_state: IssuesViewState,
    pub dependencies_view_state: DependenciesViewState,
    pub labels_view_state: LabelsViewState,
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
    /// Notification message to display to user
    pub notification: Option<NotificationMessage>,
    /// Whether beads daemon is currently running
    pub daemon_running: bool,
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

        // Check daemon status
        let daemon_running = Self::check_daemon_status_sync(&beads_client);

        Self {
            should_quit: false,
            selected_tab: 0,
            tabs: vec!["Issues", "Dependencies", "Labels", "Database", "Help"],
            beads_client,
            issues_view_state: IssuesViewState::new(issues),
            dependencies_view_state: DependenciesViewState::new(),
            labels_view_state: LabelsViewState::new(),
            label_stats,
            database_stats,
            database_status: DatabaseStatus::Ready,
            dirty: true, // Initial render required
            perf_stats: PerfStats::new(),
            show_perf_stats: false,
            help_section: HelpSection::Global,
            dialog_state: None,
            pending_action: None,
            notification: None,
            daemon_running,
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

    /// Check daemon status synchronously using tokio runtime
    fn check_daemon_status_sync(client: &BeadsClient) -> bool {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(client.check_daemon_status())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to check daemon status: {:?}", e);
                false
            })
    }

    /// Reload issues from beads database
    pub fn reload_issues(&mut self) {
        let issues = Self::load_issues_sync(&self.beads_client);

        // Update label statistics
        self.label_stats = compute_label_stats(&issues);

        // Update database stats
        self.database_stats.total_issues = issues.len();

        // Update daemon status
        self.daemon_running = Self::check_daemon_status_sync(&self.beads_client);

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

    /// Set a notification message to display to the user
    pub fn set_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notification = Some(NotificationMessage {
            message,
            notification_type,
        });
        self.mark_dirty();
    }

    /// Set an error notification
    pub fn set_error(&mut self, message: String) {
        self.set_notification(message, NotificationType::Error);
    }

    /// Set a success notification
    pub fn set_success(&mut self, message: String) {
        self.set_notification(message, NotificationType::Success);
    }

    /// Set an info notification
    pub fn set_info(&mut self, message: String) {
        self.set_notification(message, NotificationType::Info);
    }

    /// Set a warning notification
    pub fn set_warning(&mut self, message: String) {
        self.set_notification(message, NotificationType::Warning);
    }

    /// Clear the current notification
    pub fn clear_notification(&mut self) {
        self.notification = None;
        self.mark_dirty();
    }

    /// Clear error (alias for clear_notification for backward compatibility)
    pub fn clear_error(&mut self) {
        self.clear_notification();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
