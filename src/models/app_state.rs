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
    pub created_at: std::time::Instant,
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
            created_at: std::time::Instant::now(),
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

    /// Check and auto-dismiss old notifications
    /// Info and Success notifications are auto-dismissed after 3 seconds
    /// Error and Warning notifications require manual dismissal
    pub fn check_notification_timeout(&mut self) {
        if let Some(ref notification) = self.notification {
            let should_auto_dismiss = matches!(
                notification.notification_type,
                NotificationType::Info | NotificationType::Success
            );

            if should_auto_dismiss {
                const AUTO_DISMISS_DURATION: std::time::Duration =
                    std::time::Duration::from_secs(3);
                if notification.created_at.elapsed() >= AUTO_DISMISS_DURATION {
                    self.clear_notification();
                }
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a minimal AppState for testing
    fn create_test_app_state() -> AppState {
        AppState {
            should_quit: false,
            selected_tab: 0,
            tabs: vec!["Issues", "Dependencies", "Labels", "Database", "Help"],
            beads_client: BeadsClient::new(),
            issues_view_state: IssuesViewState::new(vec![]),
            dependencies_view_state: DependenciesViewState::new(),
            labels_view_state: LabelsViewState::new(),
            label_stats: vec![],
            database_stats: DatabaseStats {
                total_issues: 0,
                open_issues: 0,
                closed_issues: 0,
                blocked_issues: 0,
                database_size: 0,
                last_sync: None,
            },
            database_status: DatabaseStatus::Ready,
            dirty: false,
            perf_stats: PerfStats::new(),
            show_perf_stats: false,
            help_section: HelpSection::Global,
            dialog_state: None,
            pending_action: None,
            notification: None,
            daemon_running: false,
        }
    }

    // NotificationType tests
    #[test]
    fn test_notification_type_equality() {
        assert_eq!(NotificationType::Error, NotificationType::Error);
        assert_eq!(NotificationType::Success, NotificationType::Success);
        assert_eq!(NotificationType::Info, NotificationType::Info);
        assert_eq!(NotificationType::Warning, NotificationType::Warning);
        assert_ne!(NotificationType::Error, NotificationType::Success);
    }

    // Tab navigation tests
    #[test]
    fn test_next_tab() {
        let mut state = create_test_app_state();
        assert_eq!(state.selected_tab, 0);

        state.next_tab();
        assert_eq!(state.selected_tab, 1);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_next_tab_wraps_around() {
        let mut state = create_test_app_state();
        state.selected_tab = 4; // Last tab (Help)

        state.next_tab();
        assert_eq!(state.selected_tab, 0); // Wraps to first tab
    }

    #[test]
    fn test_previous_tab() {
        let mut state = create_test_app_state();
        state.selected_tab = 2;

        state.previous_tab();
        assert_eq!(state.selected_tab, 1);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_previous_tab_wraps_around() {
        let mut state = create_test_app_state();
        state.selected_tab = 0; // First tab

        state.previous_tab();
        assert_eq!(state.selected_tab, 4); // Wraps to last tab
    }

    // Dirty flag tests
    #[test]
    fn test_mark_dirty() {
        let mut state = create_test_app_state();
        state.dirty = false;

        state.mark_dirty();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_dirty() {
        let mut state = create_test_app_state();
        state.dirty = true;

        state.clear_dirty();
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_is_dirty() {
        let mut state = create_test_app_state();
        state.dirty = true;
        assert!(state.is_dirty());

        state.dirty = false;
        assert!(!state.is_dirty());
    }

    // Performance stats tests
    #[test]
    fn test_toggle_perf_stats() {
        let mut state = create_test_app_state();
        assert!(!state.show_perf_stats);

        state.toggle_perf_stats();
        assert!(state.show_perf_stats);
        assert!(state.is_dirty());

        state.clear_dirty();
        state.toggle_perf_stats();
        assert!(!state.show_perf_stats);
        assert!(state.is_dirty());
    }

    // Help section navigation tests
    #[test]
    fn test_next_help_section() {
        let mut state = create_test_app_state();
        state.help_section = HelpSection::Global;

        state.next_help_section();
        assert_ne!(state.help_section, HelpSection::Global);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_previous_help_section() {
        let mut state = create_test_app_state();
        state.help_section = HelpSection::Global;

        state.previous_help_section();
        assert_ne!(state.help_section, HelpSection::Global);
        assert!(state.is_dirty());
    }

    // Notification tests
    #[test]
    fn test_set_notification() {
        let mut state = create_test_app_state();

        state.set_notification("Test message".to_string(), NotificationType::Info);

        assert!(state.notification.is_some());
        let notification = state.notification.as_ref().unwrap();
        assert_eq!(notification.message, "Test message");
        assert_eq!(notification.notification_type, NotificationType::Info);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_set_error() {
        let mut state = create_test_app_state();

        state.set_error("Error message".to_string());

        assert!(state.notification.is_some());
        let notification = state.notification.as_ref().unwrap();
        assert_eq!(notification.message, "Error message");
        assert_eq!(notification.notification_type, NotificationType::Error);
    }

    #[test]
    fn test_set_success() {
        let mut state = create_test_app_state();

        state.set_success("Success message".to_string());

        assert!(state.notification.is_some());
        let notification = state.notification.as_ref().unwrap();
        assert_eq!(notification.message, "Success message");
        assert_eq!(notification.notification_type, NotificationType::Success);
    }

    #[test]
    fn test_set_info() {
        let mut state = create_test_app_state();

        state.set_info("Info message".to_string());

        assert!(state.notification.is_some());
        let notification = state.notification.as_ref().unwrap();
        assert_eq!(notification.message, "Info message");
        assert_eq!(notification.notification_type, NotificationType::Info);
    }

    #[test]
    fn test_set_warning() {
        let mut state = create_test_app_state();

        state.set_warning("Warning message".to_string());

        assert!(state.notification.is_some());
        let notification = state.notification.as_ref().unwrap();
        assert_eq!(notification.message, "Warning message");
        assert_eq!(notification.notification_type, NotificationType::Warning);
    }

    #[test]
    fn test_clear_notification() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());
        assert!(state.notification.is_some());

        state.clear_dirty();
        state.clear_notification();

        assert!(state.notification.is_none());
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_error_alias() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());
        assert!(state.notification.is_some());

        state.clear_error();
        assert!(state.notification.is_none());
    }

    #[test]
    fn test_check_notification_timeout_error_not_auto_dismissed() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());

        // Even after time passes, error should not auto-dismiss
        state.check_notification_timeout();
        assert!(state.notification.is_some());
    }

    #[test]
    fn test_check_notification_timeout_warning_not_auto_dismissed() {
        let mut state = create_test_app_state();
        state.set_warning("Warning".to_string());

        // Even after time passes, warning should not auto-dismiss
        state.check_notification_timeout();
        assert!(state.notification.is_some());
    }

    #[test]
    fn test_notification_message_creation() {
        let notification = NotificationMessage {
            message: "Test".to_string(),
            notification_type: NotificationType::Success,
            created_at: std::time::Instant::now(),
        };

        assert_eq!(notification.message, "Test");
        assert_eq!(notification.notification_type, NotificationType::Success);
    }
}
