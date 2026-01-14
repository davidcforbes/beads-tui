/// Application state management
use crate::beads::BeadsClient;
use crate::config::Config;
use crate::models::SavedFilter;
use std::collections::VecDeque;
use crate::ui::views::{
    compute_label_stats, BondingInterfaceState, DatabaseStats, DatabaseStatus, DatabaseViewState,
    DependenciesViewState, Formula, FormulaBrowserState, GanttViewState, HelpSection,
    HistoryOpsState, IssuesViewState, KanbanViewState, LabelStats, LabelsViewState, PertViewState,
    PourWizardState, WispManagerState,
};
use crate::ui::widgets::{
    DependencyDialogState, DialogState, FilterQuickSelectState, FilterSaveDialogState,
};

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
    pub dependency_dialog_state: DependencyDialogState,
    pub labels_view_state: LabelsViewState,
    pub pert_view_state: PertViewState,
    pub gantt_view_state: GanttViewState,
    pub kanban_view_state: KanbanViewState,
    pub database_stats: DatabaseStats,
    pub database_status: DatabaseStatus,
    pub database_view_state: DatabaseViewState,
    pub formulas: Vec<Formula>,
    pub formula_browser_state: FormulaBrowserState,
    pub pour_wizard_state: Option<PourWizardState>,
    pub wisp_manager_state: WispManagerState,
    pub bonding_interface_state: BondingInterfaceState,
    pub history_ops_state: HistoryOpsState,
    pub molecular_tabs: Vec<&'static str>,
    pub selected_molecular_tab: usize,
    pub label_stats: Vec<LabelStats>,
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
    /// Notification queue (FIFO - oldest notifications rendered first)
    pub notifications: Vec<NotificationMessage>,
    /// Notification history (all past notifications, max 100)
    pub notification_history: VecDeque<NotificationMessage>,
    /// Whether notification history panel is visible
    pub show_notification_history: bool,
    /// Notification history panel state
    pub notification_history_state: crate::ui::widgets::NotificationHistoryState,
    /// Whether to show issue history panel
    pub show_issue_history: bool,
    /// Issue history panel state
    pub issue_history_state: crate::ui::widgets::IssueHistoryState,
    /// Whether to show label picker
    pub show_label_picker: bool,
    /// Priority selector state for batch priority updates
    pub priority_selector_state: crate::ui::widgets::SelectorState,
    /// Status selector state for batch status changes
    pub status_selector_state: crate::ui::widgets::SelectorState,
    /// Label picker state for batch label operations
    pub label_picker_state: crate::ui::widgets::LabelPickerState,
    /// Column manager state (None when closed)
    pub column_manager_state: Option<crate::ui::widgets::ColumnManagerState>,
    /// Whether beads daemon is currently running
    pub daemon_running: bool,
    /// Application configuration
    pub config: Config,
    /// Filter save dialog state
    pub filter_save_dialog_state: Option<FilterSaveDialogState>,
    /// Filter quick-select menu state
    pub filter_quick_select_state: Option<FilterQuickSelectState>,
    /// Name of filter being edited (if any)
    pub editing_filter_name: Option<String>,
    /// Name of filter pending deletion confirmation
    pub delete_confirmation_filter: Option<String>,
    /// Delete confirmation dialog state
    pub delete_dialog_state: Option<DialogState>,
    /// Pending dependency removal (issue_id, depends_on_id)
    pub pending_dependency_removal: Option<(String, String)>,
    /// Dependency removal confirmation dialog state
    pub dependency_removal_dialog_state: Option<DialogState>,
    /// Whether keyboard shortcut help overlay is visible
    pub show_shortcut_help: bool,
    /// Whether context-sensitive help overlay is visible
    pub show_context_help: bool,
    /// Loading spinner widget (None if not loading)
    pub loading_spinner: Option<crate::ui::widgets::Spinner>,
    /// Loading operation message
    pub loading_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let beads_client = BeadsClient::new();

        // Load issues on startup
        let issues = Self::load_issues_sync(&beads_client);

        // Compute label statistics
        let label_stats = compute_label_stats(&issues);
        let label_picker_labels: Vec<String> = label_stats
            .iter()
            .map(|stat| stat.name.clone())
            .collect();

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

        // Load configuration
        let config = Config::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load config: {:?}, using defaults", e);
            Config::default()
        });

        let formulas = vec![
            Formula {
                name: "Feature".to_string(),
                description: "Standard feature template with estimate and labels".to_string(),
                variables: vec![
                    "title".to_string(),
                    "description".to_string(),
                    "estimate".to_string(),
                ],
            },
            Formula {
                name: "Bug".to_string(),
                description: "Bug report template with steps to reproduce and priority".to_string(),
                variables: vec![
                    "title".to_string(),
                    "repro_steps".to_string(),
                    "priority".to_string(),
                ],
            },
            Formula {
                name: "Chore".to_string(),
                description: "Maintenance task or internal improvement".to_string(),
                variables: vec!["title".to_string(), "details".to_string()],
            },
            Formula {
                name: "Release".to_string(),
                description: "Release checklist and deployment steps".to_string(),
                variables: vec!["version".to_string(), "date".to_string()],
            },
        ];

        let mut issues_view_state = IssuesViewState::new(issues.clone());
        issues_view_state.set_saved_filters(config.filters.clone());
        // Load table configuration from config
        issues_view_state
            .search_state_mut()
            .list_state_mut()
            .set_table_config(config.table.clone());

        Self {
            should_quit: false,
            selected_tab: 0,
            tabs: vec![
                "Issues",
                "Dependencies",
                "Labels",
                "PERT",
                "Gantt",
                "Kanban",
                "Molecular",
                "Database",
                "Help",
            ],
            beads_client,
            issues_view_state,
            dependencies_view_state: DependenciesViewState::new(),
            dependency_dialog_state: DependencyDialogState::new(),
            labels_view_state: LabelsViewState::new(),
            pert_view_state: PertViewState::new(issues.clone()),
            gantt_view_state: GanttViewState::new(issues.clone()),
            kanban_view_state: KanbanViewState::new(issues),
            label_stats,
            database_stats,
            database_status: DatabaseStatus::Ready,
            database_view_state: DatabaseViewState::new(),
            formulas,
            formula_browser_state: FormulaBrowserState::new(),
            pour_wizard_state: None,
            wisp_manager_state: WispManagerState::new(),
            bonding_interface_state: BondingInterfaceState::new(),
            history_ops_state: HistoryOpsState::new(),
            molecular_tabs: vec!["Formulas", "Wisps", "Bonds", "Squash/Burn"],
            selected_molecular_tab: 0,
            dirty: true, // Initial render required
            perf_stats: PerfStats::new(),
            show_perf_stats: false,
            help_section: HelpSection::Global,
            dialog_state: None,
            pending_action: None,
            notifications: Vec::new(),
            notification_history: VecDeque::new(),
            show_notification_history: false,
            notification_history_state: crate::ui::widgets::NotificationHistoryState::new(),
            show_issue_history: false,
            issue_history_state: crate::ui::widgets::IssueHistoryState::new(),
            show_label_picker: false,
            priority_selector_state: crate::ui::widgets::SelectorState::new(),
            status_selector_state: crate::ui::widgets::SelectorState::new(),
            label_picker_state: crate::ui::widgets::LabelPickerState::new(label_picker_labels),
            column_manager_state: None,
            daemon_running,
            config,
            filter_save_dialog_state: None,
            filter_quick_select_state: None,
            editing_filter_name: None,
            delete_confirmation_filter: None,
            delete_dialog_state: None,
            pending_dependency_removal: None,
            dependency_removal_dialog_state: None,
            show_shortcut_help: false,
            show_context_help: false,
            loading_spinner: None,
            loading_message: None,
        }
    }

    /// Load issues synchronously using tokio runtime
    fn load_issues_sync(client: &BeadsClient) -> Vec<crate::beads::models::Issue> {
        crate::runtime::RUNTIME.block_on(client.list_issues(None, None))
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load issues: {:?}", e);
                vec![]
            })
    }

    /// Check daemon status synchronously using tokio runtime
    fn check_daemon_status_sync(client: &BeadsClient) -> bool {
        crate::runtime::RUNTIME.block_on(client.check_daemon_status())
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
        self.label_picker_state.set_available_labels(
            self.label_stats
                .iter()
                .map(|stat| stat.name.clone())
                .collect(),
        );

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

    /// Toggle notification history panel visibility
    pub fn toggle_notification_history(&mut self) {
        self.show_notification_history = !self.show_notification_history;
        self.mark_dirty();
    }

    /// Navigate to next help section
    pub fn next_help_section(&mut self) {
        let sections = HelpSection::all();
        let current_idx = sections
            .iter()
            .position(|&s| s == self.help_section)
            .unwrap_or(0);
        self.help_section = sections[(current_idx + 1) % sections.len()];
        self.mark_dirty();
    }

    /// Navigate to previous help section
    pub fn previous_help_section(&mut self) {
        let sections = HelpSection::all();
        let current_idx = sections
            .iter()
            .position(|&s| s == self.help_section)
            .unwrap_or(0);
        self.help_section = if current_idx == 0 {
            sections[sections.len() - 1]
        } else {
            sections[current_idx - 1]
        };
        self.mark_dirty();
    }

    /// Add a notification message to the queue
    pub fn set_notification(&mut self, message: String, notification_type: NotificationType) {
        let notification = NotificationMessage {
            message,
            notification_type,
            created_at: std::time::Instant::now(),
        };

        // Add to current notification queue
        self.notifications.push(notification.clone());

        // Add to history (limit to 100 most recent)
        self.notification_history.push_back(notification);
        if self.notification_history.len() > 100 {
            self.notification_history.pop_front();
        }

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

    /// Clear the oldest notification from the queue
    pub fn clear_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.notifications.remove(0); // Remove oldest (FIFO)
        }
        self.mark_dirty();
    }

    /// Clear all notifications from the queue
    pub fn clear_all_notifications(&mut self) {
        self.notifications.clear();
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
        const AUTO_DISMISS_DURATION: std::time::Duration =
            std::time::Duration::from_secs(3);

        // Remove expired Info and Success notifications
        self.notifications.retain(|notification| {
            let should_auto_dismiss = matches!(
                notification.notification_type,
                NotificationType::Info | NotificationType::Success
            );

            if should_auto_dismiss {
                notification.created_at.elapsed() < AUTO_DISMISS_DURATION
            } else {
                // Error and Warning notifications are kept until manually dismissed
                true
            }
        });

        if !self.notifications.is_empty() {
            self.mark_dirty();
        }
    }

    /// Show the filter save dialog
    pub fn show_filter_save_dialog(&mut self) {
        self.filter_save_dialog_state = Some(FilterSaveDialogState::new());
        self.mark_dirty();
    }

    /// Hide the filter save dialog
    pub fn hide_filter_save_dialog(&mut self) {
        self.filter_save_dialog_state = None;
        self.mark_dirty();
    }

    /// Check if filter save dialog is visible
    pub fn is_filter_save_dialog_visible(&self) -> bool {
        self.filter_save_dialog_state.is_some()
    }

    /// Apply a saved filter to the issues view search state
    pub fn apply_saved_filter(&mut self, saved_filter: &SavedFilter) {
        self.issues_view_state
            .search_state_mut()
            .apply_filter(&saved_filter.filter);
        self.mark_dirty();
    }

    /// Start editing a filter - opens save dialog with existing filter data
    pub fn start_edit_filter(&mut self, filter_name: &str) {
        if let Some(filter) = self.config.get_filter(filter_name).cloned() {
            // Store the name of the filter being edited
            self.editing_filter_name = Some(filter_name.to_string());

            // Create filter save dialog state and populate with existing data
            let mut dialog_state = FilterSaveDialogState::new();
            dialog_state.set_name(&filter.name);
            // description field is not used in SavedFilter, so leave empty
            if let Some(hotkey) = filter.hotkey {
                dialog_state.set_hotkey(Some(hotkey.to_string()));
            }

            self.filter_save_dialog_state = Some(dialog_state);
            self.mark_dirty();
        }
    }

    /// Save edited filter - updates existing filter instead of creating new one
    pub fn save_edited_filter(&mut self) -> Result<(), String> {
        // Get the name of the filter being edited
        let editing_name = self
            .editing_filter_name
            .as_ref()
            .ok_or_else(|| "No filter is being edited".to_string())?
            .clone();

        // Get dialog state
        let dialog_state = self
            .filter_save_dialog_state
            .as_ref()
            .ok_or_else(|| "Filter save dialog is not open".to_string())?;

        // Validate dialog input
        dialog_state.validate()?;

        // Get current filter state from search interface using delegation
        let filter = self.issues_view_state.search_state().get_current_filter();

        // Parse hotkey (convert from string to char)
        let hotkey = dialog_state.hotkey().and_then(|h| h.chars().next());

        // Create SavedFilter
        let saved_filter = SavedFilter {
            name: dialog_state.name().to_string(),
            filter,
            hotkey,
        };

        // Update the filter in config
        if !self.config.update_filter(&editing_name, saved_filter.clone()) {
            return Err(format!("Filter '{}' not found", editing_name));
        }

        // Save config to disk
        self.config.save().map_err(|e| {
            tracing::error!("Failed to save config: {:?}", e);
            "Failed to save configuration. Check logs for details.".to_string()
        })?;

        // Synchronize saved filters with search state
        self.issues_view_state.set_saved_filters(self.config.filters.clone());

        // Show success notification
        self.set_success(format!("Filter '{}' updated successfully", saved_filter.name));

        // Clear edit state
        self.editing_filter_name = None;

        // Hide dialog
        self.hide_filter_save_dialog();

        Ok(())
    }

    /// Save current filter as a new saved filter
    pub fn save_current_filter(&mut self) -> Result<(), String> {
        let dialog_state = self
            .filter_save_dialog_state
            .as_ref()
            .ok_or_else(|| "Filter save dialog is not open".to_string())?;

        dialog_state.validate()?;

        let filter = self.issues_view_state.search_state().get_current_filter();
        let hotkey = dialog_state.hotkey().and_then(|h| h.chars().next());

        let saved_filter = SavedFilter {
            name: dialog_state.name().to_string(),
            filter,
            hotkey,
        };

        self.config.add_filter(saved_filter.clone());
        self.config.save().map_err(|e| {
            tracing::error!("Failed to save config after adding filter: {:?}", e);
            "Failed to save configuration. Check logs for details.".to_string()
        })?;

        self.issues_view_state
            .set_saved_filters(self.config.filters.clone());
        self.set_success(format!("Filter '{}' saved successfully", saved_filter.name));
        self.hide_filter_save_dialog();

        Ok(())
    }

    /// Show delete confirmation dialog for a filter
    pub fn show_delete_filter_confirmation(&mut self, filter_name: &str) {
        self.delete_confirmation_filter = Some(filter_name.to_string());
        self.delete_dialog_state = Some(DialogState::new());
        self.mark_dirty();
    }

    /// Confirm and execute filter deletion
    pub fn confirm_delete_filter(&mut self) -> Result<(), String> {
        let filter_name = self
            .delete_confirmation_filter
            .as_ref()
            .ok_or_else(|| "No filter pending deletion".to_string())?
            .clone();

        // Remove filter from config
        if !self.config.remove_filter(&filter_name) {
            return Err(format!("Filter '{}' not found", filter_name));
        }

        // Save config to disk
        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save config after deleting filter: {:?}", e);
            return Err("Failed to save configuration. Check logs for details.".to_string());
        }

        // Update the saved filters in issues view state
        self.issues_view_state
            .set_saved_filters(self.config.filters.clone());

        // Clear delete confirmation state
        self.delete_confirmation_filter = None;
        self.delete_dialog_state = None;

        self.mark_dirty();

        Ok(())
    }

    /// Cancel filter deletion
    pub fn cancel_delete_filter(&mut self) {
        self.delete_confirmation_filter = None;
        self.delete_dialog_state = None;
        self.mark_dirty();
    }

    /// Check if currently editing a filter
    pub fn is_editing_filter(&self) -> bool {
        self.editing_filter_name.is_some()
    }

    /// Check if delete confirmation dialog is visible
    pub fn is_delete_confirmation_visible(&self) -> bool {
        self.delete_confirmation_filter.is_some()
    }

    /// Show dependency removal confirmation dialog
    pub fn show_dependency_removal_confirmation(&mut self, issue_id: &str, depends_on_id: &str) {
        self.pending_dependency_removal = Some((issue_id.to_string(), depends_on_id.to_string()));
        self.dependency_removal_dialog_state = Some(DialogState::new());
        self.mark_dirty();
    }

    /// Confirm and execute dependency removal
    pub fn confirm_remove_dependency(&mut self) -> Result<(), String> {
        let (issue_id, depends_on_id) = self
            .pending_dependency_removal
            .as_ref()
            .ok_or_else(|| "No dependency pending removal".to_string())?
            .clone();

        // Use the beads client to remove the dependency
        use crate::runtime;
        match runtime::RUNTIME.block_on(
            self.beads_client.remove_dependency(&issue_id, &depends_on_id),
        ) {
            Ok(()) => {
                tracing::info!(
                    "Removed dependency: {} no longer depends on {}",
                    issue_id,
                    depends_on_id
                );

                // Clear confirmation state
                self.pending_dependency_removal = None;
                self.dependency_removal_dialog_state = None;

                // Reload issues to reflect the change
                self.reload_issues();

                self.mark_dirty();

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to remove dependency: {}", e);
                Err(format!(
                    "Failed to remove dependency: {}\n\nCommon causes:\n• Dependency does not exist\n• Invalid issue ID format\n• Network connectivity issues\n\nVerify with 'bd show <issue-id>'",
                    e
                ))
            }
        }
    }

    /// Cancel dependency removal
    pub fn cancel_remove_dependency(&mut self) {
        self.pending_dependency_removal = None;
        self.dependency_removal_dialog_state = None;
        self.mark_dirty();
    }

    /// Check if dependency removal confirmation dialog is visible
    pub fn is_dependency_removal_confirmation_visible(&self) -> bool {
        self.pending_dependency_removal.is_some()
    }

    /// Show the keyboard shortcut help overlay
    pub fn show_shortcut_help(&mut self) {
        self.show_shortcut_help = true;
        self.mark_dirty();
    }

    /// Hide the keyboard shortcut help overlay
    pub fn hide_shortcut_help(&mut self) {
        self.show_shortcut_help = false;
        self.mark_dirty();
    }

    /// Check if keyboard shortcut help overlay is visible
    pub fn is_shortcut_help_visible(&self) -> bool {
        self.show_shortcut_help
    }

    /// Show the context-sensitive help overlay
    pub fn show_context_help(&mut self) {
        self.show_context_help = true;
        self.mark_dirty();
    }

    /// Hide the context-sensitive help overlay
    pub fn hide_context_help(&mut self) {
        self.show_context_help = false;
        self.mark_dirty();
    }

    /// Check if context-sensitive help overlay is visible
    pub fn is_context_help_visible(&self) -> bool {
        self.show_context_help
    }

    /// Get context-sensitive help content based on current view and focus
    pub fn get_context_help_content(&self) -> (String, Vec<crate::ui::widgets::KeyBinding>) {
        use crate::ui::widgets::KeyBinding;

        match self.selected_tab {
            0 => {
                // Issues view
                let title = "Issues View Help".to_string();
                let bindings = vec![
                    KeyBinding::new("↑/↓ or j/k", "Navigate issues"),
                    KeyBinding::new("←/→ or h/l", "Navigate columns"),
                    KeyBinding::new("Enter", "View/edit issue details"),
                    KeyBinding::new("n", "Create new issue"),
                    KeyBinding::new("e", "Edit selected issue"),
                    KeyBinding::new("d", "Delete selected issue"),
                    KeyBinding::new("c", "Close selected issue"),
                    KeyBinding::new("/", "Search issues"),
                    KeyBinding::new("f", "Quick filters"),
                    KeyBinding::new("Ctrl+S", "Save current filter"),
                    KeyBinding::new("F1-F11", "Apply saved filter"),
                    KeyBinding::new("Space", "Toggle column sort"),
                    KeyBinding::new("Tab", "Cycle view modes"),
                    KeyBinding::new("Esc", "Cancel/return to list"),
                ];
                (title, bindings)
            }
            1 => {
                // Dependencies view
                let title = "Dependencies View Help".to_string();
                let bindings = vec![
                    KeyBinding::new("↑/↓ or j/k", "Navigate dependencies"),
                    KeyBinding::new("Tab", "Switch between Dependencies/Blocks"),
                    KeyBinding::new("a", "Add dependency"),
                    KeyBinding::new("d", "Remove dependency"),
                    KeyBinding::new("Enter", "View issue details"),
                    KeyBinding::new("g", "Show dependency graph"),
                    KeyBinding::new("Esc", "Return to issues"),
                ];
                (title, bindings)
            }
            2 => {
                // Labels view
                let title = "Labels View Help".to_string();
                let bindings = vec![
                    KeyBinding::new("↑/↓ or j/k", "Navigate labels"),
                    KeyBinding::new("Enter", "Select/apply label"),
                    KeyBinding::new("a", "Add new label"),
                    KeyBinding::new("d", "Delete label"),
                    KeyBinding::new("/", "Search labels"),
                    KeyBinding::new("Esc", "Return to issues"),
                ];
                (title, bindings)
            }
            3 => {
                // PERT view
                let title = "PERT Chart View Help".to_string();
                let bindings = vec![
                    KeyBinding::new("↑/↓ or j/k", "Navigate nodes"),
                    KeyBinding::new("+/-", "Zoom in/out"),
                    KeyBinding::new("c", "Configure chart settings"),
                    KeyBinding::new("Enter", "View node details"),
                    KeyBinding::new("Esc", "Return to issues"),
                ];
                (title, bindings)
            }
            4 => {
                // Gantt view
                let title = "Gantt Chart View Help".to_string();
                let bindings = vec![
                    KeyBinding::new("↑/↓ or j/k", "Navigate tasks"),
                    KeyBinding::new("+/-", "Zoom timeline in/out"),
                    KeyBinding::new("g", "Change grouping mode"),
                    KeyBinding::new("c", "Configure chart settings"),
                    KeyBinding::new("Enter", "View task details"),
                    KeyBinding::new("Esc", "Return to issues"),
                ];
                (title, bindings)
            }
            _ => {
                // Default/unknown view
                let title = "Help".to_string();
                let bindings = vec![
                    KeyBinding::new("1-5", "Switch tabs"),
                    KeyBinding::new("F1", "Context help (this overlay)"),
                    KeyBinding::new("q", "Quit application"),
                    KeyBinding::new("N", "Notification history"),
                ];
                (title, bindings)
            }
        }
    }

    /// Start showing a loading indicator with the given message
    pub fn start_loading<S: Into<String>>(&mut self, message: S) {
        use crate::ui::widgets::Spinner;
        use ratatui::style::{Color, Style};

        let msg = message.into();
        self.loading_message = Some(msg.clone());
        self.loading_spinner = Some(
            Spinner::new()
                .label(msg)
                .style(Style::default().fg(Color::Cyan)),
        );
        self.mark_dirty();
    }

    /// Stop showing the loading indicator
    pub fn stop_loading(&mut self) {
        self.loading_spinner = None;
        self.loading_message = None;
        self.mark_dirty();
    }

    /// Check if a loading operation is in progress
    pub fn is_loading(&self) -> bool {
        self.loading_spinner.is_some()
    }

    /// Save table configuration from issues view to config and persist to disk
    pub fn save_table_config(&mut self) -> Result<(), String> {
        // Copy table config from issues view state to app config
        let table_config = self
            .issues_view_state
            .search_state()
            .list_state()
            .table_config()
            .clone();
        
        self.config.table = table_config;

        // Save config to disk
        self.config.save().map_err(|e| {
            tracing::error!("Failed to save table config: {:?}", e);
            "Failed to save configuration. Check logs for details.".to_string()
        })
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
            tabs: vec![
                "Issues",
                "Dependencies",
                "Labels",
                "PERT",
                "Gantt",
                "Kanban",
                "Molecular",
                "Database",
                "Help",
            ],
            beads_client: BeadsClient::new(),
            issues_view_state: IssuesViewState::new(vec![]),
            dependencies_view_state: DependenciesViewState::new(),
            dependency_dialog_state: DependencyDialogState::new(),
            labels_view_state: LabelsViewState::new(),
            pert_view_state: PertViewState::new(vec![]),
            gantt_view_state: GanttViewState::new(vec![]),
            kanban_view_state: KanbanViewState::new(vec![]),
            database_view_state: DatabaseViewState::new(),
            formulas: vec![],
            formula_browser_state: FormulaBrowserState::new(),
            pour_wizard_state: None,
            wisp_manager_state: WispManagerState::new(),
            bonding_interface_state: BondingInterfaceState::new(),
            history_ops_state: HistoryOpsState::new(),
            molecular_tabs: vec![],
            selected_molecular_tab: 0,
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
            notifications: Vec::new(),
            daemon_running: false,
            config: Config::default(),
            filter_save_dialog_state: None,
            filter_quick_select_state: None,
            editing_filter_name: None,
            delete_confirmation_filter: None,
            delete_dialog_state: None,
            pending_dependency_removal: None,
            dependency_removal_dialog_state: None,
            show_shortcut_help: false,
            show_context_help: false,
            loading_spinner: None,
            loading_message: None,
            notification_history: VecDeque::new(),
            show_notification_history: false,
            notification_history_state: crate::ui::widgets::NotificationHistoryState::new(),
            show_issue_history: false,
            issue_history_state: crate::ui::widgets::IssueHistoryState::new(),
            show_label_picker: false,
            priority_selector_state: crate::ui::widgets::SelectorState::new(),
            status_selector_state: crate::ui::widgets::SelectorState::new(),
            label_picker_state: crate::ui::widgets::LabelPickerState::new(vec![]),
            column_manager_state: None,
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
        state.selected_tab = 8; // Last tab (Help)

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
        assert_eq!(state.selected_tab, 8); // Wraps to last tab
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

        assert!(!state.notifications.is_empty());
        let notification = state.notifications.last().unwrap();
        assert_eq!(notification.message, "Test message");
        assert_eq!(notification.notification_type, NotificationType::Info);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_set_error() {
        let mut state = create_test_app_state();

        state.set_error("Error message".to_string());

        assert!(!state.notifications.is_empty());
        let notification = state.notifications.last().unwrap();
        assert_eq!(notification.message, "Error message");
        assert_eq!(notification.notification_type, NotificationType::Error);
    }

    #[test]
    fn test_set_success() {
        let mut state = create_test_app_state();

        state.set_success("Success message".to_string());

        assert!(!state.notifications.is_empty());
        let notification = state.notifications.last().unwrap();
        assert_eq!(notification.message, "Success message");
        assert_eq!(notification.notification_type, NotificationType::Success);
    }

    #[test]
    fn test_set_info() {
        let mut state = create_test_app_state();

        state.set_info("Info message".to_string());

        assert!(!state.notifications.is_empty());
        let notification = state.notifications.last().unwrap();
        assert_eq!(notification.message, "Info message");
        assert_eq!(notification.notification_type, NotificationType::Info);
    }

    #[test]
    fn test_set_warning() {
        let mut state = create_test_app_state();

        state.set_warning("Warning message".to_string());

        assert!(!state.notifications.is_empty());
        let notification = state.notifications.last().unwrap();
        assert_eq!(notification.message, "Warning message");
        assert_eq!(notification.notification_type, NotificationType::Warning);
    }

    #[test]
    fn test_clear_notification() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());
        assert!(!state.notifications.is_empty());

        state.clear_dirty();
        state.clear_notification();

        assert!(state.notifications.is_empty());
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_error_alias() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());
        assert!(!state.notifications.is_empty());

        state.clear_error();
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_check_notification_timeout_error_not_auto_dismissed() {
        let mut state = create_test_app_state();
        state.set_error("Error".to_string());

        // Even after time passes, error should not auto-dismiss
        state.check_notification_timeout();
        assert!(!state.notifications.is_empty());
    }

    #[test]
    fn test_check_notification_timeout_warning_not_auto_dismissed() {
        let mut state = create_test_app_state();
        state.set_warning("Warning".to_string());

        // Even after time passes, warning should not auto-dismiss
        state.check_notification_timeout();
        assert!(!state.notifications.is_empty());
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

    #[test]
    fn test_notification_type_clone() {
        let nt = NotificationType::Error;
        let cloned = nt;
        assert_eq!(nt, cloned);
    }

    #[test]
    fn test_notification_type_copy() {
        let nt = NotificationType::Success;
        let copied = nt;
        assert_eq!(nt, copied);
    }

    #[test]
    fn test_notification_message_clone() {
        let notification = NotificationMessage {
            message: "Test".to_string(),
            notification_type: NotificationType::Info,
            created_at: std::time::Instant::now(),
        };

        let cloned = notification.clone();
        assert_eq!(notification.message, cloned.message);
        assert_eq!(notification.notification_type, cloned.notification_type);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.should_quit);
        assert_eq!(state.selected_tab, 0);
        assert_eq!(state.tabs.len(), 9);
    }

    #[test]
    fn test_next_tab_multiple_times() {
        let mut state = create_test_app_state();

        // Navigate through all tabs
        for i in 1..9 {
            state.next_tab();
            assert_eq!(state.selected_tab, i);
        }

        // Next should wrap to 0
        state.next_tab();
        assert_eq!(state.selected_tab, 0);
    }

    #[test]
    fn test_previous_tab_multiple_times() {
        let mut state = create_test_app_state();
        state.selected_tab = 8; // Start at last tab

        // Navigate backward through all tabs
        for i in (0..8).rev() {
            state.previous_tab();
            assert_eq!(state.selected_tab, i);
        }

        // Previous should wrap to last
        state.previous_tab();
        assert_eq!(state.selected_tab, 8);
    }

    #[test]
    fn test_dirty_flag_on_tab_navigation() {
        let mut state = create_test_app_state();
        state.dirty = false;

        state.next_tab();
        assert!(state.is_dirty());

        state.clear_dirty();
        assert!(!state.is_dirty());

        state.previous_tab();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_toggle_perf_stats_enables_profiling() {
        let mut state = create_test_app_state();
        assert!(!state.show_perf_stats);
        assert!(!state.perf_stats.is_enabled());

        state.toggle_perf_stats();
        assert!(state.show_perf_stats);
        assert!(state.perf_stats.is_enabled());
    }

    #[test]
    fn test_help_section_wraps_around() {
        let mut state = create_test_app_state();
        let sections = HelpSection::all();

        // Set to last section
        state.help_section = sections[sections.len() - 1];

        // Next should wrap to first
        state.next_help_section();
        assert_eq!(state.help_section, sections[0]);
    }

    #[test]
    fn test_previous_help_section_wraps_around() {
        let mut state = create_test_app_state();
        let sections = HelpSection::all();

        // Set to first section
        state.help_section = sections[0];

        // Previous should wrap to last
        state.previous_help_section();
        assert_eq!(state.help_section, sections[sections.len() - 1]);
    }

    #[test]
    fn test_notification_replacement() {
        let mut state = create_test_app_state();

        state.set_error("First error".to_string());
        assert_eq!(state.notifications.last().unwrap().message, "First error");

        // Setting a new notification should replace the old one
        state.set_success("Success!".to_string());
        assert_eq!(state.notifications.last().unwrap().message, "Success!");
        assert_eq!(
            state.notifications.last().unwrap().notification_type,
            NotificationType::Success
        );
    }

    #[test]
    fn test_set_notification_marks_dirty() {
        let mut state = create_test_app_state();
        state.dirty = false;

        state.set_notification("Test".to_string(), NotificationType::Info);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_notification_marks_dirty() {
        let mut state = create_test_app_state();
        state.set_info("Info".to_string());
        state.dirty = false;

        state.clear_notification();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_notification_types_all_different() {
        assert_ne!(NotificationType::Error, NotificationType::Warning);
        assert_ne!(NotificationType::Success, NotificationType::Info);
        assert_ne!(NotificationType::Error, NotificationType::Info);
        assert_ne!(NotificationType::Warning, NotificationType::Success);
    }

    #[test]
    fn test_help_section_navigation_marks_dirty() {
        let mut state = create_test_app_state();
        state.dirty = false;

        state.next_help_section();
        assert!(state.is_dirty());

        state.dirty = false;
        state.previous_help_section();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_notification_when_none() {
        let mut state = create_test_app_state();
        assert!(state.notifications.is_empty());

        // Should not panic
        state.clear_notification();
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_check_notification_timeout_when_none() {
        let mut state = create_test_app_state();
        assert!(state.notifications.is_empty());

        // Should not panic
        state.check_notification_timeout();
        assert!(state.notifications.is_empty());
    }

    // === New comprehensive tests ===

    #[test]
    fn test_notification_type_debug() {
        let nt = NotificationType::Error;
        let debug = format!("{:?}", nt);
        assert!(debug.contains("Error"));

        let nt2 = NotificationType::Success;
        let debug2 = format!("{:?}", nt2);
        assert!(debug2.contains("Success"));
    }

    #[test]
    fn test_notification_message_debug() {
        let msg = NotificationMessage {
            message: "Test".to_string(),
            notification_type: NotificationType::Info,
            created_at: std::time::Instant::now(),
        };
        let debug = format!("{:?}", msg);
        assert!(debug.contains("NotificationMessage"));
        assert!(debug.contains("Test"));
    }

    #[test]
    fn test_app_state_debug() {
        let state = create_test_app_state();
        let debug = format!("{:?}", state);
        assert!(debug.contains("AppState"));
    }

    #[test]
    fn test_notification_created_at_timestamp() {
        let mut state = create_test_app_state();
        let before = std::time::Instant::now();

        state.set_info("Test".to_string());

        let after = std::time::Instant::now();
        let notification = state.notifications.last().unwrap();
        assert!(notification.created_at >= before);
        assert!(notification.created_at <= after);
    }

    #[test]
    fn test_check_notification_timeout_info_auto_dismiss() {
        let mut state = create_test_app_state();

        // Create an info notification with a timestamp in the past
        state.notifications.push(NotificationMessage {
            message: "Old info".to_string(),
            notification_type: NotificationType::Info,
            created_at: std::time::Instant::now() - std::time::Duration::from_secs(4),
        });

        state.check_notification_timeout();
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_check_notification_timeout_success_auto_dismiss() {
        let mut state = create_test_app_state();

        // Create a success notification with a timestamp in the past
        state.notifications.push(NotificationMessage {
            message: "Old success".to_string(),
            notification_type: NotificationType::Success,
            created_at: std::time::Instant::now() - std::time::Duration::from_secs(4),
        });

        state.check_notification_timeout();
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_check_notification_timeout_error_no_auto_dismiss() {
        let mut state = create_test_app_state();

        // Create an error notification with a timestamp in the past
        state.notifications.push(NotificationMessage {
            message: "Old error".to_string(),
            notification_type: NotificationType::Error,
            created_at: std::time::Instant::now() - std::time::Duration::from_secs(10),
        });

        state.check_notification_timeout();
        assert!(!state.notifications.is_empty()); // Should not be auto-dismissed
    }

    #[test]
    fn test_check_notification_timeout_warning_no_auto_dismiss() {
        let mut state = create_test_app_state();

        // Create a warning notification with a timestamp in the past
        state.notifications.push(NotificationMessage {
            message: "Old warning".to_string(),
            notification_type: NotificationType::Warning,
            created_at: std::time::Instant::now() - std::time::Duration::from_secs(10),
        });

        state.check_notification_timeout();
        assert!(!state.notifications.is_empty()); // Should not be auto-dismissed
    }

    #[test]
    fn test_check_notification_timeout_info_recent_not_dismissed() {
        let mut state = create_test_app_state();

        // Create a recent info notification
        state.notifications.push(NotificationMessage {
            message: "Recent info".to_string(),
            notification_type: NotificationType::Info,
            created_at: std::time::Instant::now(),
        });

        state.check_notification_timeout();
        assert!(!state.notifications.is_empty()); // Should not be dismissed yet
    }

    #[test]
    fn test_toggle_perf_stats_twice() {
        let mut state = create_test_app_state();

        state.toggle_perf_stats();
        assert!(state.show_perf_stats);

        state.toggle_perf_stats();
        assert!(!state.show_perf_stats);
    }

    #[test]
    fn test_tabs_count() {
        let state = create_test_app_state();
        assert_eq!(state.tabs.len(), 9);
        assert_eq!(state.tabs[0], "Issues");
        assert_eq!(state.tabs[1], "Dependencies");
        assert_eq!(state.tabs[2], "Labels");
        assert_eq!(state.tabs[3], "PERT");
        assert_eq!(state.tabs[4], "Gantt");
        assert_eq!(state.tabs[5], "Kanban");
        assert_eq!(state.tabs[6], "Molecular");
        assert_eq!(state.tabs[7], "Database");
        assert_eq!(state.tabs[8], "Help");
    }

    #[test]
    fn test_initial_state_defaults() {
        let state = create_test_app_state();
        assert!(!state.should_quit);
        assert_eq!(state.selected_tab, 0);
        assert!(!state.is_dirty());
        assert!(!state.show_perf_stats);
        assert_eq!(state.help_section, HelpSection::Global);
        assert!(state.dialog_state.is_none());
        assert!(state.pending_action.is_none());
        assert!(state.notifications.is_empty());
        assert!(!state.daemon_running);
    }

    #[test]
    fn test_mark_dirty_idempotent() {
        let mut state = create_test_app_state();
        state.dirty = false;

        state.mark_dirty();
        assert!(state.is_dirty());

        state.mark_dirty();
        assert!(state.is_dirty());
    }

    #[test]
    fn test_clear_dirty_idempotent() {
        let mut state = create_test_app_state();
        state.dirty = true;

        state.clear_dirty();
        assert!(!state.is_dirty());

        state.clear_dirty();
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_notification_type_all_variants() {
        let variants = vec![
            NotificationType::Error,
            NotificationType::Success,
            NotificationType::Info,
            NotificationType::Warning,
        ];

        for variant in &variants {
            let debug = format!("{:?}", variant);
            assert!(!debug.is_empty());
        }
    }
}

