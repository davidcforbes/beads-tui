//! Kanban board view for visual issue management

use crate::beads::models::{Issue, IssueStatus, Priority};
use crate::models::kanban_config::{
    CardSort, ColumnDefinition, ColumnId, GroupingMode, KanbanConfig,
};
use crate::ui::widgets::kanban_card::{render_kanban_card, CardMode, KanbanCardConfig};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use std::collections::HashSet;

const COLLAPSED_COLUMN_WIDTH: u16 = 6;
const STATUS_COLLAPSE_HINT: &str = " [v^<]";

/// Column manager overlay state
#[derive(Debug, Clone)]
pub struct ColumnManagerState {
    /// Working copy of columns for editing
    pub columns: Vec<ColumnDefinition>,
    /// Selected column index in the list
    pub selected_index: usize,
    /// Whether the overlay is visible
    pub visible: bool,
    /// Edit mode (None, WipLimit)
    pub edit_mode: Option<ColumnEditMode>,
    /// Input buffer for editing fields
    pub input_buffer: String,
}

/// Column edit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnEditMode {
    /// Editing WIP limit
    WipLimit,
}

impl Default for ColumnManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnManagerState {
    /// Create a new column manager state
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            selected_index: 0,
            visible: false,
            edit_mode: None,
            input_buffer: String::new(),
        }
    }

    /// Open the overlay with columns from config
    pub fn open(&mut self, config_columns: Vec<ColumnDefinition>) {
        self.columns = config_columns;
        self.selected_index = 0;
        self.visible = true;
        self.edit_mode = None;
        self.input_buffer.clear();
    }

    /// Close the overlay
    pub fn close(&mut self) {
        self.visible = false;
        self.edit_mode = None;
        self.input_buffer.clear();
    }

    /// Move selected column up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.columns
                .swap(self.selected_index, self.selected_index - 1);
            self.selected_index -= 1;
        }
    }

    /// Move selected column down
    pub fn move_down(&mut self) {
        if self.selected_index < self.columns.len().saturating_sub(1) {
            self.columns
                .swap(self.selected_index, self.selected_index + 1);
            self.selected_index += 1;
        }
    }

    /// Navigate to next column in list
    pub fn next_column(&mut self) {
        if !self.columns.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.columns.len();
        }
    }

    /// Navigate to previous column in list
    pub fn previous_column(&mut self) {
        if !self.columns.is_empty() {
            self.selected_index =
                (self.selected_index + self.columns.len() - 1) % self.columns.len();
        }
    }

    /// Toggle visibility of selected column
    pub fn toggle_visibility(&mut self, mode: GroupingMode) {
        if let Some(col) = self.columns.get_mut(self.selected_index) {
            if !col.id.is_mandatory(mode) {
                col.visible = !col.visible;
            }
        }
    }

    /// Start editing WIP limit for selected column
    pub fn edit_wip_limit(&mut self) {
        if let Some(col) = self.columns.get(self.selected_index) {
            self.edit_mode = Some(ColumnEditMode::WipLimit);
            self.input_buffer = col.wip_limit.map_or(String::new(), |l| l.to_string());
        }
    }

    /// Apply WIP limit edit
    pub fn apply_wip_limit_edit(&mut self) {
        if let Some(col) = self.columns.get_mut(self.selected_index) {
            col.wip_limit = if self.input_buffer.is_empty() {
                None
            } else {
                self.input_buffer.parse::<usize>().ok()
            };
        }
        self.edit_mode = None;
        self.input_buffer.clear();
    }

    /// Cancel current edit
    pub fn cancel_edit(&mut self) {
        self.edit_mode = None;
        self.input_buffer.clear();
    }
}

/// State for Kanban board view
#[derive(Debug)]
pub struct KanbanViewState {
    /// Kanban board configuration
    config: KanbanConfig,
    /// All issues to display
    issues: Vec<Issue>,
    /// Selected column index
    selected_column: usize,
    /// Selected card index within column
    selected_card: usize,
    /// Card display mode
    card_mode: CardMode,
    /// Column manager overlay state
    column_manager: ColumnManagerState,
    /// Horizontal scroll offset in columns
    horizontal_scroll: usize,
    /// Columns that are collapsed (status grouping only)
    collapsed_columns: HashSet<ColumnId>,
}

impl KanbanViewState {
    /// Create a new Kanban view state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            config: KanbanConfig::default(),
            issues,
            selected_column: 0,
            selected_card: 0,
            card_mode: CardMode::FourLine,
            column_manager: ColumnManagerState::new(),
            horizontal_scroll: 0,
            collapsed_columns: HashSet::new(),
        }
    }

    /// Get the issues for display
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Set the issues
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.issues = issues;
    }

    /// Get the selected column index
    pub fn selected_column(&self) -> usize {
        self.selected_column
    }

    /// Get the selected card index
    pub fn selected_card(&self) -> usize {
        self.selected_card
    }

    /// Move selection to next column
    pub fn next_column(&mut self) {
        let visible_columns = self.visible_columns().len();
        if visible_columns > 0 {
            self.selected_column = (self.selected_column + 1) % visible_columns;
            self.selected_card = 0;
        }
    }

    /// Move selection to previous column
    pub fn previous_column(&mut self) {
        let visible_columns = self.visible_columns().len();
        if visible_columns > 0 {
            self.selected_column = if self.selected_column == 0 {
                visible_columns - 1
            } else {
                self.selected_column - 1
            };
            self.selected_card = 0;
        }
    }

    /// Move selection to next card
    pub fn next_card(&mut self) {
        let column_issues = self.get_column_issues(self.selected_column);
        if !column_issues.is_empty() {
            self.selected_card = (self.selected_card + 1) % column_issues.len();
        }
    }

    /// Move selection to previous card
    pub fn previous_card(&mut self) {
        let column_issues = self.get_column_issues(self.selected_column);
        if !column_issues.is_empty() {
            self.selected_card = if self.selected_card == 0 {
                column_issues.len() - 1
            } else {
                self.selected_card - 1
            };
        }
    }

    /// Get the currently selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        let column_issues = self.get_column_issues(self.selected_column);
        column_issues.get(self.selected_card).copied()
    }

    /// Toggle card display mode
    pub fn toggle_card_mode(&mut self) {
        self.card_mode = match self.card_mode {
            CardMode::SingleLine => CardMode::TwoLine,
            CardMode::TwoLine => CardMode::FourLine,
            CardMode::FourLine => CardMode::SingleLine,
        };
    }

    /// Toggle collapsed state for a status column (status grouping only)
    pub fn toggle_column_collapse(&mut self, id: ColumnId) {
        if self.config.grouping_mode != GroupingMode::Status {
            return;
        }
        if !Self::is_status_column_id(&id) {
            return;
        }
        if self.collapsed_columns.contains(&id) {
            self.collapsed_columns.remove(&id);
        } else {
            self.collapsed_columns.insert(id);
        }
    }

    fn is_column_collapsed(&self, id: &ColumnId) -> bool {
        self.collapsed_columns.contains(id)
    }

    fn is_status_grouping(&self) -> bool {
        self.config.grouping_mode == GroupingMode::Status
    }

    fn is_status_column_id(id: &ColumnId) -> bool {
        matches!(
            id,
            ColumnId::StatusOpen
                | ColumnId::StatusInProgress
                | ColumnId::StatusBlocked
                | ColumnId::StatusClosed
        )
    }

    /// Set search query filter
    pub fn set_search_query(&mut self, query: Option<String>) {
        self.config.filters.search_query = query;
    }

    /// Add label filter
    pub fn add_label_filter(&mut self, label: String) {
        if !self.config.filters.labels.contains(&label) {
            self.config.filters.labels.push(label);
        }
    }

    /// Remove label filter
    pub fn remove_label_filter(&mut self, label: &str) {
        self.config.filters.labels.retain(|l| l != label);
    }

    /// Add assignee filter
    pub fn add_assignee_filter(&mut self, assignee: String) {
        if !self.config.filters.assignees.contains(&assignee) {
            self.config.filters.assignees.push(assignee);
        }
    }

    /// Remove assignee filter
    pub fn remove_assignee_filter(&mut self, assignee: &str) {
        self.config.filters.assignees.retain(|a| a != assignee);
    }

    /// Add status filter
    pub fn add_status_filter(&mut self, status: String) {
        if !self.config.filters.statuses.contains(&status) {
            self.config.filters.statuses.push(status);
        }
    }

    /// Remove status filter
    pub fn remove_status_filter(&mut self, status: &str) {
        self.config.filters.statuses.retain(|s| s != status);
    }

    /// Clear all filters
    pub fn clear_all_filters(&mut self) {
        self.config.filters.clear();
    }

    /// Check if any filters are active
    pub fn has_active_filters(&self) -> bool {
        self.config.filters.is_active()
    }

    /// Set sort order for a specific column
    pub fn set_column_sort(&mut self, column_index: usize, sort: CardSort) {
        let visible_columns_count = self.config.visible_columns().len();
        if column_index < visible_columns_count {
            // Find the actual column in the config (accounting for hidden columns)
            let mut visible_idx = 0;
            for col in &mut self.config.columns {
                if col.visible {
                    if visible_idx == column_index {
                        col.card_sort = sort;
                        return;
                    }
                    visible_idx += 1;
                }
            }
        }
    }

    /// Get sort order for a specific column
    pub fn get_column_sort(&self, column_index: usize) -> Option<CardSort> {
        let visible_columns = self.visible_columns();
        visible_columns.get(column_index).map(|col| col.card_sort)
    }

    /// Open the column manager overlay
    pub fn open_column_manager(&mut self) {
        self.column_manager.open(self.config.columns.clone());
    }

    /// Close the column manager overlay without applying changes
    pub fn close_column_manager(&mut self) {
        self.column_manager.close();
    }

    /// Apply column manager changes to config
    pub fn apply_column_manager_changes(&mut self) {
        self.config.columns = self.column_manager.columns.clone();
        self.column_manager.close();

        // Ensure selected_column is still valid after changes
        let visible_count = self.visible_columns().len();
        if self.selected_column >= visible_count && visible_count > 0 {
            self.selected_column = visible_count - 1;
        }
        self.selected_card = 0;
    }

    /// Reset columns to defaults
    pub fn reset_columns_to_default(&mut self) {
        let mode = self.config.grouping_mode;
        self.config.columns = match mode {
            GroupingMode::Status => vec![
                ColumnDefinition::new(ColumnId::StatusOpen),
                ColumnDefinition::new(ColumnId::StatusInProgress),
                ColumnDefinition::new(ColumnId::StatusBlocked),
                ColumnDefinition::new(ColumnId::StatusClosed),
            ],
            GroupingMode::Priority => vec![
                ColumnDefinition::new(ColumnId::PriorityP0),
                ColumnDefinition::new(ColumnId::PriorityP1),
                ColumnDefinition::new(ColumnId::PriorityP2),
                ColumnDefinition::new(ColumnId::PriorityP3),
                ColumnDefinition::new(ColumnId::PriorityP4),
            ],
            GroupingMode::Assignee | GroupingMode::Label => {
                vec![ColumnDefinition::new(ColumnId::Unassigned)]
            }
        };

        // Update overlay if it's open
        if self.column_manager.visible {
            self.column_manager.open(self.config.columns.clone());
        }
    }

    /// Check if column manager is visible
    pub fn is_column_manager_visible(&self) -> bool {
        self.column_manager.visible
    }

    /// Get mutable reference to column manager (for input handling)
    pub fn column_manager_mut(&mut self) -> &mut ColumnManagerState {
        &mut self.column_manager
    }

    /// Increase width of selected column
    pub fn increase_column_width(&mut self) {
        let visible_columns = self.visible_columns();
        if let Some(column) = visible_columns.get(self.selected_column) {
            let column_id = column.id.clone();
            if let Some(col) = self.config.get_column_mut(&column_id) {
                let new_width = col.width + 5;
                col.set_width(new_width);
            }
        }
    }

    /// Decrease width of selected column
    pub fn decrease_column_width(&mut self) {
        let visible_columns = self.visible_columns();
        if let Some(column) = visible_columns.get(self.selected_column) {
            let column_id = column.id.clone();
            if let Some(col) = self.config.get_column_mut(&column_id) {
                let new_width = col.width.saturating_sub(5);
                col.set_width(new_width);
            }
        }
    }

    /// Scroll horizontally left
    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
    }

    /// Scroll horizontally right
    pub fn scroll_right(&mut self) {
        let visible_columns = self.visible_columns();
        if self.horizontal_scroll < visible_columns.len().saturating_sub(1) {
            self.horizontal_scroll += 1;
        }
    }

    /// Ensure the selected column is visible within the current scroll view
    pub fn ensure_selected_column_visible(&mut self, viewport_width: u16) {
        let visible_columns = self.visible_columns();

        // Calculate which columns fit in viewport starting from horizontal_scroll
        let mut accumulated_width = 0u16;
        let mut visible_range_end = self.horizontal_scroll;

        for idx in self.horizontal_scroll..visible_columns.len() {
            if let Some(col) = visible_columns.get(idx) {
                accumulated_width += col.width;
                if accumulated_width <= viewport_width {
                    visible_range_end = idx;
                } else {
                    break;
                }
            }
        }

        // If selected column is before scroll position, scroll left to it
        if self.selected_column < self.horizontal_scroll {
            self.horizontal_scroll = self.selected_column;
        }

        // If selected column is after visible range, scroll right to show it
        if self.selected_column > visible_range_end {
            self.horizontal_scroll = self.selected_column;
        }
    }

    /// Get horizontal scroll offset
    pub fn horizontal_scroll(&self) -> usize {
        self.horizontal_scroll
    }

    /// Move selected card to next column (right)
    pub fn move_card_to_next_column(&mut self) -> Result<(), String> {
        let visible_columns = self.visible_columns();
        if self.selected_column >= visible_columns.len().saturating_sub(1) {
            return Err("Already in last column".to_string());
        }

        let target_column_idx = self.selected_column + 1;
        self.move_card_to_column(target_column_idx)
    }

    /// Move selected card to previous column (left)
    pub fn move_card_to_previous_column(&mut self) -> Result<(), String> {
        if self.selected_column == 0 {
            return Err("Already in first column".to_string());
        }

        let target_column_idx = self.selected_column - 1;
        self.move_card_to_column(target_column_idx)
    }

    /// Move selected card to a specific column by index
    pub fn move_card_to_column(&mut self, target_column_idx: usize) -> Result<(), String> {
        // Extract column IDs before we start mutating
        let (source_column_id, target_column_id, issue_id) = {
            let visible_columns = self.visible_columns();

            // Get source and target columns
            let source_column = visible_columns
                .get(self.selected_column)
                .ok_or("Invalid source column")?;
            let target_column = visible_columns
                .get(target_column_idx)
                .ok_or("Invalid target column")?;

            // Get the selected issue
            let source_issues = self.get_column_issues(self.selected_column);
            let issue = source_issues
                .get(self.selected_card)
                .ok_or("No card selected")?;

            (
                source_column.id.clone(),
                target_column.id.clone(),
                issue.id.clone(),
            )
        }; // visible_columns borrow ends here

        // Find the issue in our issues vector and update it
        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found in state")?;

        // Update issue based on grouping mode and target column
        match self.config.grouping_mode {
            GroupingMode::Status => {
                let new_status = match &target_column_id {
                    ColumnId::StatusOpen => IssueStatus::Open,
                    ColumnId::StatusInProgress => IssueStatus::InProgress,
                    ColumnId::StatusBlocked => IssueStatus::Blocked,
                    ColumnId::StatusClosed => IssueStatus::Closed,
                    _ => return Err("Invalid status column".to_string()),
                };
                issue_mut.status = new_status;
            }
            GroupingMode::Priority => {
                let new_priority = match &target_column_id {
                    ColumnId::PriorityP0 => Priority::P0,
                    ColumnId::PriorityP1 => Priority::P1,
                    ColumnId::PriorityP2 => Priority::P2,
                    ColumnId::PriorityP3 => Priority::P3,
                    ColumnId::PriorityP4 => Priority::P4,
                    _ => return Err("Invalid priority column".to_string()),
                };
                issue_mut.priority = new_priority;
            }
            GroupingMode::Assignee => match &target_column_id {
                ColumnId::Assignee(name) => {
                    issue_mut.assignee = Some(name.clone());
                }
                ColumnId::Unassigned => {
                    issue_mut.assignee = None;
                }
                _ => return Err("Invalid assignee column".to_string()),
            },
            GroupingMode::Label => {
                // For label mode, we need to handle adding/removing labels
                match (&source_column_id, &target_column_id) {
                    (ColumnId::Label(old_label), ColumnId::Label(new_label)) => {
                        // Remove old label, add new label
                        issue_mut.labels.retain(|l| l != old_label);
                        if !issue_mut.labels.contains(new_label) {
                            issue_mut.labels.push(new_label.clone());
                        }
                    }
                    (ColumnId::Label(old_label), ColumnId::Unassigned) => {
                        // Remove label
                        issue_mut.labels.retain(|l| l != old_label);
                    }
                    (ColumnId::Unassigned, ColumnId::Label(new_label)) => {
                        // Add label
                        if !issue_mut.labels.contains(new_label) {
                            issue_mut.labels.push(new_label.clone());
                        }
                    }
                    _ => return Err("Invalid label column".to_string()),
                }
            }
        }

        // Update the selected column to target and reset card selection
        self.selected_column = target_column_idx;
        self.selected_card = 0;

        Ok(())
    }

    /// Quick action: Update selected issue status
    pub fn update_selected_issue_status(&mut self, status: IssueStatus) -> Result<(), String> {
        let column_issues = self.get_column_issues(self.selected_column);
        let issue = column_issues
            .get(self.selected_card)
            .ok_or("No card selected")?;
        let issue_id = issue.id.clone();

        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found")?;

        issue_mut.status = status;
        Ok(())
    }

    /// Quick action: Update selected issue priority
    pub fn update_selected_issue_priority(&mut self, priority: Priority) -> Result<(), String> {
        let column_issues = self.get_column_issues(self.selected_column);
        let issue = column_issues
            .get(self.selected_card)
            .ok_or("No card selected")?;
        let issue_id = issue.id.clone();

        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found")?;

        issue_mut.priority = priority;
        Ok(())
    }

    /// Quick action: Update selected issue assignee
    pub fn update_selected_issue_assignee(
        &mut self,
        assignee: Option<String>,
    ) -> Result<(), String> {
        let column_issues = self.get_column_issues(self.selected_column);
        let issue = column_issues
            .get(self.selected_card)
            .ok_or("No card selected")?;
        let issue_id = issue.id.clone();

        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found")?;

        issue_mut.assignee = assignee;
        Ok(())
    }

    /// Quick action: Add label to selected issue
    pub fn add_label_to_selected_issue(&mut self, label: String) -> Result<(), String> {
        let column_issues = self.get_column_issues(self.selected_column);
        let issue = column_issues
            .get(self.selected_card)
            .ok_or("No card selected")?;
        let issue_id = issue.id.clone();

        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found")?;

        if !issue_mut.labels.contains(&label) {
            issue_mut.labels.push(label);
        }
        Ok(())
    }

    /// Quick action: Remove label from selected issue
    pub fn remove_label_from_selected_issue(&mut self, label: &str) -> Result<(), String> {
        let column_issues = self.get_column_issues(self.selected_column);
        let issue = column_issues
            .get(self.selected_card)
            .ok_or("No card selected")?;
        let issue_id = issue.id.clone();

        let issue_mut = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or("Issue not found")?;

        issue_mut.labels.retain(|l| l != label);
        Ok(())
    }

    /// Refresh issues from external source while preserving view state
    pub fn refresh_issues(&mut self, new_issues: Vec<Issue>) {
        // Preserve current state
        let old_selected_column = self.selected_column;
        let old_selected_card = self.selected_card;
        let old_scroll = self.horizontal_scroll;

        // Get currently selected issue ID if any
        let selected_issue_id = self.selected_issue().map(|i| i.id.clone());

        // Update issues
        self.issues = new_issues;

        // Restore scroll position
        self.horizontal_scroll = old_scroll;

        // Try to restore selection to same issue
        if let Some(issue_id) = selected_issue_id {
            // Try to find the issue in the new data
            let visible_column_count = self.visible_columns().len();
            let mut found = false;

            for col_idx in 0..visible_column_count {
                let column_issues = self.get_column_issues(col_idx);
                for (card_idx, issue) in column_issues.iter().enumerate() {
                    if issue.id == issue_id {
                        self.selected_column = col_idx;
                        self.selected_card = card_idx;
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }

            // If not found, try to keep same column/card position
            if !found {
                self.selected_column =
                    old_selected_column.min(visible_column_count.saturating_sub(1));
                let column_issue_count = self.get_column_issues(self.selected_column).len();
                self.selected_card = old_selected_card.min(column_issue_count.saturating_sub(1));
            }
        }
    }

    /// Get visible columns
    fn visible_columns(&self) -> Vec<&ColumnDefinition> {
        self.config
            .columns
            .iter()
            .filter(|col| col.visible)
            .collect()
    }

    /// Get issues for a specific column by index
    fn get_column_issues(&self, column_index: usize) -> Vec<&Issue> {
        let visible_columns = self.visible_columns();
        if let Some(column) = visible_columns.get(column_index) {
            let mut issues = self.filter_issues_for_column(column);

            // Apply global filters
            issues = self.apply_global_filters(issues);

            // Apply column-specific sorting
            self.sort_issues(issues, column.card_sort)
        } else {
            Vec::new()
        }
    }

    /// Apply global filters to a list of issues
    fn apply_global_filters<'a>(&'a self, issues: Vec<&'a Issue>) -> Vec<&'a Issue> {
        let filters = &self.config.filters;

        issues
            .into_iter()
            .filter(|issue| {
                // Search query filter
                if let Some(query) = &filters.search_query {
                    let query_lower = query.to_lowercase();
                    let title_match = issue.title.to_lowercase().contains(&query_lower);
                    let desc_match = issue
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false);
                    if !title_match && !desc_match {
                        return false;
                    }
                }

                // Label filter
                if !filters.labels.is_empty() {
                    let has_matching_label = filters
                        .labels
                        .iter()
                        .any(|label| issue.labels.contains(label));
                    if !has_matching_label {
                        return false;
                    }
                }

                // Assignee filter
                if !filters.assignees.is_empty() {
                    let matches_assignee = match &issue.assignee {
                        Some(assignee) => filters.assignees.contains(assignee),
                        None => filters.assignees.contains(&"unassigned".to_string()),
                    };
                    if !matches_assignee {
                        return false;
                    }
                }

                // Status filter
                if !filters.statuses.is_empty() {
                    let status_str = format!("{:?}", issue.status).to_lowercase();
                    if !filters
                        .statuses
                        .iter()
                        .any(|s| s.to_lowercase() == status_str)
                    {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Sort issues based on the specified sort order
    fn sort_issues<'a>(&self, mut issues: Vec<&'a Issue>, sort: CardSort) -> Vec<&'a Issue> {
        use crate::beads::models::Priority;

        match sort {
            CardSort::Priority => {
                issues.sort_by(|a, b| {
                    // P0 (0) first, P4 (4) last
                    let a_val = match a.priority {
                        Priority::P0 => 0,
                        Priority::P1 => 1,
                        Priority::P2 => 2,
                        Priority::P3 => 3,
                        Priority::P4 => 4,
                    };
                    let b_val = match b.priority {
                        Priority::P0 => 0,
                        Priority::P1 => 1,
                        Priority::P2 => 2,
                        Priority::P3 => 3,
                        Priority::P4 => 4,
                    };
                    a_val.cmp(&b_val)
                });
            }
            CardSort::Title => {
                issues.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            CardSort::Created => {
                issues.sort_by(|a, b| b.created.cmp(&a.created)); // Newest first
            }
            CardSort::Updated => {
                issues.sort_by(|a, b| b.updated.cmp(&a.updated)); // Most recent first
            }
        }

        issues
    }

    /// Filter issues for a specific column based on its configuration
    fn filter_issues_for_column(&self, column: &ColumnDefinition) -> Vec<&Issue> {
        match self.config.grouping_mode {
            GroupingMode::Status => match &column.id {
                ColumnId::StatusOpen => self
                    .issues
                    .iter()
                    .filter(|issue| issue.status == IssueStatus::Open)
                    .collect(),
                ColumnId::StatusInProgress => self
                    .issues
                    .iter()
                    .filter(|issue| issue.status == IssueStatus::InProgress)
                    .collect(),
                ColumnId::StatusBlocked => self
                    .issues
                    .iter()
                    .filter(|issue| issue.status == IssueStatus::Blocked)
                    .collect(),
                ColumnId::StatusClosed => self
                    .issues
                    .iter()
                    .filter(|issue| issue.status == IssueStatus::Closed)
                    .collect(),
                _ => Vec::new(),
            },
            GroupingMode::Priority => match &column.id {
                ColumnId::PriorityP0 => self
                    .issues
                    .iter()
                    .filter(|issue| issue.priority == Priority::P0)
                    .collect(),
                ColumnId::PriorityP1 => self
                    .issues
                    .iter()
                    .filter(|issue| issue.priority == Priority::P1)
                    .collect(),
                ColumnId::PriorityP2 => self
                    .issues
                    .iter()
                    .filter(|issue| issue.priority == Priority::P2)
                    .collect(),
                ColumnId::PriorityP3 => self
                    .issues
                    .iter()
                    .filter(|issue| issue.priority == Priority::P3)
                    .collect(),
                ColumnId::PriorityP4 => self
                    .issues
                    .iter()
                    .filter(|issue| issue.priority == Priority::P4)
                    .collect(),
                _ => Vec::new(),
            },
            GroupingMode::Assignee => {
                if let ColumnId::Assignee(assignee) = &column.id {
                    self.issues
                        .iter()
                        .filter(|issue| issue.assignee.as_deref() == Some(assignee.as_str()))
                        .collect()
                } else if matches!(column.id, ColumnId::Unassigned) {
                    self.issues
                        .iter()
                        .filter(|issue| issue.assignee.is_none())
                        .collect()
                } else {
                    Vec::new()
                }
            }
            GroupingMode::Label => {
                if let ColumnId::Label(label) = &column.id {
                    self.issues
                        .iter()
                        .filter(|issue| issue.labels.contains(&label.clone()))
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

use ratatui::widgets::StatefulWidget;

/// Kanban board widget
pub struct KanbanView;

impl StatefulWidget for KanbanView {
    type State = KanbanViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Self::render_with_state(area, buf, state);
    }
}

impl Widget for KanbanView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render a placeholder when no state is provided
        // This is a fallback, but in practice we use render_stateful_widget
        let block = Block::default()
            .title("Kanban Board")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        let text = vec![
            Line::from(Span::styled(
                "Kanban Board View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Press Up/Down/Left/Right or h/j/k/l to navigate"),
            Line::from("Press Space to move card"),
            Line::from("Press c to configure columns"),
        ];

        let paragraph = Paragraph::new(text);
        paragraph.render(inner, buf);
    }
}

impl KanbanView {
    /// Create a new Kanban view
    pub fn new() -> Self {
        Self
    }

    /// Render the Kanban board with state
    pub fn render_with_state(area: Rect, buf: &mut Buffer, state: &KanbanViewState) {
        let visible_columns = state.visible_columns();

        if visible_columns.is_empty() {
            // No columns to display
            let block = Block::default()
                .title("Kanban Board")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));
            let inner = block.inner(area);
            block.render(area, buf);

            // Clear the inner area with a solid background
            for y in inner.y..inner.y + inner.height {
                for x in inner.x..inner.x + inner.width {
                    if x < buf.area.right() && y < buf.area.bottom() {
                        buf.get_mut(x, y).set_style(Style::default().bg(Color::Black));
                    }
                }
            }

            let text = Line::from("No columns configured. Press 'c' to configure columns.");
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(Color::Yellow).bg(Color::Black));
            paragraph.render(inner, buf);
            return;
        }

        // Split area into filter row and columns
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if state.has_active_filters() { 2 } else { 0 }),
                Constraint::Min(0),
            ])
            .split(area);

        // Render filter row if filters are active
        if state.has_active_filters() {
            Self::render_filter_row(chunks[0], buf, state);
        }

        let columns_area = if state.has_active_filters() {
            chunks[1]
        } else {
            area
        };

        let viewport_width = columns_area.width;
        let status_widths = Self::compute_status_column_widths(state, &visible_columns, viewport_width);
        let use_status_widths = status_widths.is_some();
        let column_widths: Vec<u16> = status_widths.unwrap_or_else(|| {
            visible_columns.iter().map(|col| col.width).collect()
        });
        let scroll_offset = if use_status_widths {
            0
        } else {
            state.horizontal_scroll()
        };

        // Determine visible columns that fit in viewport
        let mut columns_to_render = Vec::new();
        let mut accumulated_width = 0u16;

        for (idx, (col, width)) in visible_columns
            .iter()
            .zip(column_widths.iter())
            .enumerate()
            .skip(scroll_offset)
        {
            if accumulated_width + *width <= viewport_width {
                columns_to_render.push((idx, col, *width));
                accumulated_width += *width;
            } else {
                break;
            }
        }

        // Create layout constraints for visible columns only
        let constraints: Vec<Constraint> = columns_to_render
            .iter()
            .map(|(_, _, width)| Constraint::Length(*width))
            .collect();

        if !constraints.is_empty() {
            let columns_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(columns_area);

            // Render each visible column
            for ((col_idx, column, _), column_area) in
                columns_to_render.iter().zip(columns_layout.iter())
            {
                Self::render_column(
                    *column_area,
                    buf,
                    column,
                    state,
                    *col_idx,
                    *col_idx == state.selected_column,
                );
            }
        }

        // Show scroll indicators if needed
        if scroll_offset > 0 && !use_status_widths {
            // Left scroll indicator
            let indicator = " < ";
            buf.set_string(
                columns_area.x,
                columns_area.y,
                indicator,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        }

        if scroll_offset + columns_to_render.len() < visible_columns.len() && !use_status_widths {
            // Right scroll indicator
            let indicator = " > ";
            let x = columns_area.x + viewport_width.saturating_sub(indicator.len() as u16);
            buf.set_string(
                x,
                columns_area.y,
                indicator,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        }

        // Render column manager overlay on top if visible
        if state.is_column_manager_visible() {
            Self::render_column_manager(area, buf, state);
        }
    }

    fn compute_status_column_widths(
        state: &KanbanViewState,
        visible_columns: &[&ColumnDefinition],
        available_width: u16,
    ) -> Option<Vec<u16>> {
        if !state.is_status_grouping() {
            return None;
        }

        let collapsed_count = visible_columns
            .iter()
            .filter(|col| state.is_column_collapsed(&col.id))
            .count() as u16;
        let total_collapsed = collapsed_count * COLLAPSED_COLUMN_WIDTH;
        if total_collapsed > available_width {
            return None;
        }

        let expanded_count = visible_columns.len() as u16 - collapsed_count;
        let remaining_width = available_width.saturating_sub(total_collapsed);
        if expanded_count > 0 && remaining_width < expanded_count {
            return None;
        }

        let base_width = if expanded_count > 0 {
            remaining_width / expanded_count
        } else {
            0
        };
        let mut extra = if expanded_count > 0 {
            remaining_width % expanded_count
        } else {
            0
        };

        let mut widths = Vec::with_capacity(visible_columns.len());
        for col in visible_columns {
            if state.is_column_collapsed(&col.id) {
                widths.push(COLLAPSED_COLUMN_WIDTH);
            } else {
                let mut width = base_width;
                if extra > 0 {
                    width += 1;
                    extra -= 1;
                }
                widths.push(width);
            }
        }

        Some(widths)
    }

    /// Render the filter row showing active filters
    fn render_filter_row(area: Rect, buf: &mut Buffer, state: &KanbanViewState) {
        let filters = &state.config.filters;
        let mut spans = vec![Span::styled(
            "Filters: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )];

        let mut filter_parts = Vec::new();

        if let Some(query) = &filters.search_query {
            filter_parts.push(format!("\"{}\"", query));
        }

        if !filters.labels.is_empty() {
            filter_parts.push(format!("labels:{}", filters.labels.join(",")));
        }

        if !filters.assignees.is_empty() {
            filter_parts.push(format!("assignees:{}", filters.assignees.join(",")));
        }

        if !filters.statuses.is_empty() {
            filter_parts.push(format!("status:{}", filters.statuses.join(",")));
        }

        if filter_parts.is_empty() {
            spans.push(Span::raw("(none)"));
        } else {
            for (i, part) in filter_parts.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" | "));
                }
                spans.push(Span::styled(part, Style::default().fg(Color::Cyan)));
            }
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "(Press 'f' to edit, Shift+F to clear)",
            Style::default().fg(Color::DarkGray),
        ));

        // Clear the area with a solid background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if x < buf.area.right() && y < buf.area.bottom() {
                    buf.get_mut(x, y).set_style(Style::default().bg(Color::Black));
                }
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().bg(Color::Black));
        paragraph.render(area, buf);
    }

    /// Render the column manager overlay
    fn render_column_manager(area: Rect, buf: &mut Buffer, state: &KanbanViewState) {
        // Calculate centered overlay area (60% width, 80% height)
        let overlay_width = (area.width as f32 * 0.6).max(50.0).min(area.width as f32) as u16;
        let overlay_height = (area.height as f32 * 0.8).max(20.0).min(area.height as f32) as u16;

        let overlay_x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = area.y + (area.height.saturating_sub(overlay_height)) / 2;

        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Render background block
        let block = Block::default()
            .title(" Column Manager ")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(overlay_area);
        block.render(overlay_area, buf);

        // Clear the inner area with a solid background
        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                if x < buf.area.right() && y < buf.area.bottom() {
                    buf.get_mut(x, y).set_style(Style::default().bg(Color::Black));
                }
            }
        }

        // Split into help line and column list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(inner);

        // Help line
        let help_spans = vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Reorder  "),
            Span::styled("v", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle  "),
            Span::styled("w", Style::default().fg(Color::Yellow)),
            Span::raw(": WIP Limit  "),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw(": Reset  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Cancel"),
        ];
        let help_line = Line::from(help_spans);
        Paragraph::new(help_line).render(chunks[0], buf);

        // Render column list
        let manager = &state.column_manager;
        let list_area = chunks[1];

        let mut y = list_area.y;
        for (idx, col) in manager.columns.iter().enumerate() {
            if y >= list_area.y + list_area.height {
                break;
            }

            let is_selected = idx == manager.selected_index;
            let is_editing = is_selected && manager.edit_mode.is_some();

            // Build line spans
            let mut spans = Vec::new();

            // Selection indicator
            if is_selected {
                spans.push(Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("  "));
            }

            // Visibility toggle
            let visibility_marker = if col.visible { "[x]" } else { "[ ]" };
            let visibility_style = if col.visible {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(visibility_marker, visibility_style));
            spans.push(Span::raw(" "));

            // Column label
            let label_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(&col.label, label_style));
            spans.push(Span::raw(" "));

            // WIP Limit
            let wip_text =
                if is_editing && matches!(manager.edit_mode, Some(ColumnEditMode::WipLimit)) {
                    format!("[WIP: {}█]", manager.input_buffer)
                } else {
                    match col.wip_limit {
                        Some(limit) => format!("[WIP: {}]", limit),
                        None => "[WIP: -]".to_string(),
                    }
                };
            let wip_style = if is_editing {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(wip_text, wip_style));

            let line = Line::from(spans);
            buf.set_line(list_area.x, y, &line, list_area.width);
            y += 1;
        }

        // Status line
        let status_text = format!("{} columns total", manager.columns.len());
        let status_line = Line::from(Span::styled(
            status_text,
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(status_line).render(chunks[2], buf);
    }

    /// Render a single column
    fn render_column(
        area: Rect,
        buf: &mut Buffer,
        column: &ColumnDefinition,
        state: &KanbanViewState,
        column_index: usize,
        is_selected: bool,
    ) {
        let is_collapsed = state.is_column_collapsed(&column.id);
        let border_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Add sort indicator and active marker to title
        let sort_indicator = if is_collapsed {
            ""
        } else {
            match column.card_sort {
                CardSort::Priority => " [vP]",
                CardSort::Title => " [vA]",
                CardSort::Created => " [vC]",
                CardSort::Updated => " [vU]",
            }
        };
        let active_marker = if is_selected { "> " } else { "" };
        let collapse_hint = if state.is_status_grouping()
            && KanbanViewState::is_status_column_id(&column.id)
        {
            STATUS_COLLAPSE_HINT
        } else {
            ""
        };
        let title = format!(
            " {}{}{}{} ",
            active_marker, column.label, collapse_hint, sort_indicator
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(area);
        block.render(area, buf);

        // Clear the inner area with a solid background
        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                if x < buf.area.right() && y < buf.area.bottom() {
                    buf.get_mut(x, y).set_style(Style::default().bg(Color::Black));
                }
            }
        }

        // Get issues for this column
        let column_issues = state.get_column_issues(column_index);

        if column_issues.is_empty() {
            // Empty column
            let text = Line::from("(empty)");
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
            paragraph.render(inner, buf);
            return;
        }

        // Render cards
        let card_outer_width = inner.width.saturating_sub(2);
        if card_outer_width < 4 {
            let count_text = format!("{}", column_issues.len());
            let line = Line::from(Span::styled(
                count_text,
                Style::default().fg(Color::DarkGray),
            ));
            buf.set_line(inner.x, inner.y, &line, inner.width);
            return;
        }
        let card_inner_width = card_outer_width.saturating_sub(2);
        let card_config = KanbanCardConfig::default()
            .mode(state.card_mode)
            .max_width(card_inner_width);

        let mut y = inner.y;
        for (card_idx, issue) in column_issues.iter().enumerate() {
            if y >= inner.y + inner.height {
                break; // Out of space
            }

            let is_card_selected = is_selected && card_idx == state.selected_card;
            let card_lines = render_kanban_card(issue, &card_config, is_card_selected);

            let card_height = card_lines.len() as u16 + 2;
            if y + card_height > inner.y + inner.height {
                break;
            }

            let card_area = Rect {
                x: inner.x + 1,
                y,
                width: card_outer_width,
                height: card_height,
            };
            let card_border = if is_card_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let card_block = Block::default()
                .borders(Borders::ALL)
                .border_style(card_border)
                .border_type(BorderType::Plain)
                .style(Style::default().bg(Color::Black));
            let card_inner = card_block.inner(card_area);
            card_block.render(card_area, buf);

            // Clear the card inner area with a solid background
            for y in card_inner.y..card_inner.y + card_inner.height {
                for x in card_inner.x..card_inner.x + card_inner.width {
                    if x < buf.area.right() && y < buf.area.bottom() {
                        buf.get_mut(x, y).set_style(Style::default().bg(Color::Black));
                    }
                }
            }

            let mut line_y = card_inner.y;
            for line in card_lines {
                if line_y >= card_inner.y + card_inner.height {
                    break;
                }
                buf.set_line(card_inner.x, line_y, &line, card_inner.width);
                line_y += 1;
            }

            y += card_height;
        }
    }
}

impl Default for KanbanView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::IssueType;

    fn create_test_issue(id: &str, title: &str, status: IssueStatus) -> Issue {
        use chrono::Utc;
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: None,
            labels: vec![],
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
        }
    }

    #[test]
    fn test_kanban_view_state_new() {
        let issues = vec![create_test_issue("TEST-1", "Test Issue", IssueStatus::Open)];
        let state = KanbanViewState::new(issues.clone());

        assert_eq!(state.issues().len(), 1);
        assert_eq!(state.selected_column(), 0);
        assert_eq!(state.selected_card(), 0);
    }

    #[test]
    fn test_navigation() {
        let issues = vec![
            create_test_issue("TEST-1", "Issue 1", IssueStatus::Open),
            create_test_issue("TEST-2", "Issue 2", IssueStatus::Open),
        ];
        let mut state = KanbanViewState::new(issues);

        // Next card
        state.next_card();
        assert_eq!(state.selected_card(), 1);

        // Next card (wraps)
        state.next_card();
        assert_eq!(state.selected_card(), 0);

        // Previous card (wraps)
        state.previous_card();
        assert_eq!(state.selected_card(), 1);
    }

    #[test]
    fn test_column_navigation() {
        let issues = vec![create_test_issue("TEST-1", "Issue 1", IssueStatus::Open)];
        let mut state = KanbanViewState::new(issues);

        let initial_column = state.selected_column();

        // Next column
        state.next_column();
        assert_ne!(state.selected_column(), initial_column);

        // Card should reset
        assert_eq!(state.selected_card(), 0);
    }

    #[test]
    fn test_selected_issue() {
        let issues = vec![
            create_test_issue("TEST-1", "Issue 1", IssueStatus::Open),
            create_test_issue("TEST-2", "Issue 2", IssueStatus::Open),
        ];
        let mut state = KanbanViewState::new(issues);

        let selected = state.selected_issue();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "TEST-1");

        state.next_card();
        let selected = state.selected_issue();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "TEST-2");
    }

    #[test]
    fn test_toggle_card_mode() {
        let state = &mut KanbanViewState::new(vec![]);

        assert_eq!(state.card_mode, CardMode::FourLine);
        state.toggle_card_mode();
        assert_eq!(state.card_mode, CardMode::SingleLine);
        state.toggle_card_mode();
        assert_eq!(state.card_mode, CardMode::TwoLine);
        state.toggle_card_mode();
        assert_eq!(state.card_mode, CardMode::FourLine);
    }

    #[test]
    fn test_filter_issues_by_status() {
        let issues = vec![
            create_test_issue("TEST-1", "Open Issue", IssueStatus::Open),
            create_test_issue("TEST-2", "In Progress Issue", IssueStatus::InProgress),
            create_test_issue("TEST-3", "Closed Issue", IssueStatus::Closed),
        ];
        let state = KanbanViewState::new(issues);

        // Test filtering by status
        let open_column = ColumnDefinition::new(ColumnId::StatusOpen);

        let open_issues = state.filter_issues_for_column(&open_column);
        assert_eq!(open_issues.len(), 1);
        assert_eq!(open_issues[0].id, "TEST-1");
    }

    #[test]
    fn test_empty_column() {
        let issues = vec![create_test_issue("TEST-1", "Open Issue", IssueStatus::Open)];
        let state = KanbanViewState::new(issues);

        let closed_column = ColumnDefinition::new(ColumnId::StatusClosed);

        let closed_issues = state.filter_issues_for_column(&closed_column);
        assert_eq!(closed_issues.len(), 0);
    }

    #[test]
    fn test_set_search_query() {
        let mut state = KanbanViewState::new(vec![]);
        state.set_search_query(Some("test".to_string()));
        assert_eq!(state.config.filters.search_query, Some("test".to_string()));

        state.set_search_query(None);
        assert_eq!(state.config.filters.search_query, None);
    }

    #[test]
    fn test_add_remove_label_filter() {
        let mut state = KanbanViewState::new(vec![]);

        state.add_label_filter("bug".to_string());
        assert_eq!(state.config.filters.labels.len(), 1);
        assert!(state.config.filters.labels.contains(&"bug".to_string()));

        // Adding duplicate should not increase count
        state.add_label_filter("bug".to_string());
        assert_eq!(state.config.filters.labels.len(), 1);

        state.add_label_filter("feature".to_string());
        assert_eq!(state.config.filters.labels.len(), 2);

        state.remove_label_filter("bug");
        assert_eq!(state.config.filters.labels.len(), 1);
        assert!(!state.config.filters.labels.contains(&"bug".to_string()));
    }

    #[test]
    fn test_add_remove_assignee_filter() {
        let mut state = KanbanViewState::new(vec![]);

        state.add_assignee_filter("alice".to_string());
        assert_eq!(state.config.filters.assignees.len(), 1);

        state.remove_assignee_filter("alice");
        assert_eq!(state.config.filters.assignees.len(), 0);
    }

    #[test]
    fn test_add_remove_status_filter() {
        let mut state = KanbanViewState::new(vec![]);

        state.add_status_filter("open".to_string());
        assert_eq!(state.config.filters.statuses.len(), 1);

        state.remove_status_filter("open");
        assert_eq!(state.config.filters.statuses.len(), 0);
    }

    #[test]
    fn test_clear_all_filters() {
        let mut state = KanbanViewState::new(vec![]);

        state.set_search_query(Some("test".to_string()));
        state.add_label_filter("bug".to_string());
        state.add_assignee_filter("alice".to_string());
        state.add_status_filter("open".to_string());

        assert!(state.has_active_filters());

        state.clear_all_filters();

        assert!(!state.has_active_filters());
        assert_eq!(state.config.filters.search_query, None);
        assert!(state.config.filters.labels.is_empty());
        assert!(state.config.filters.assignees.is_empty());
        assert!(state.config.filters.statuses.is_empty());
    }

    #[test]
    fn test_set_column_sort() {
        let mut state = KanbanViewState::new(vec![]);

        // Set sort for first column
        state.set_column_sort(0, CardSort::Title);

        let sort = state.get_column_sort(0);
        assert_eq!(sort, Some(CardSort::Title));
    }

    #[test]
    fn test_apply_global_filters_search() {
        let issues = vec![
            create_test_issue("TEST-1", "Fix login bug", IssueStatus::Open),
            create_test_issue("TEST-2", "Add dashboard", IssueStatus::Open),
        ];
        let mut state = KanbanViewState::new(issues);

        state.set_search_query(Some("login".to_string()));

        let all_issues: Vec<&Issue> = state.issues.iter().collect();
        let filtered = state.apply_global_filters(all_issues);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "TEST-1");
    }

    #[test]
    fn test_sort_issues_by_priority() {
        use crate::beads::models::{IssueType, Priority};
        use chrono::Utc;

        let issues = vec![
            Issue {
                id: "TEST-1".to_string(),
                title: "P2 Task".to_string(),
                status: IssueStatus::Open,
                priority: Priority::P2,
                issue_type: IssueType::Task,
                assignee: None,
                labels: vec![],
                description: None,
                created: Utc::now(),
                updated: Utc::now(),
                closed: None,
                dependencies: vec![],
                blocks: vec![],
                notes: vec![],
            },
            Issue {
                id: "TEST-2".to_string(),
                title: "P0 Task".to_string(),
                status: IssueStatus::Open,
                priority: Priority::P0,
                issue_type: IssueType::Task,
                assignee: None,
                labels: vec![],
                description: None,
                created: Utc::now(),
                updated: Utc::now(),
                closed: None,
                dependencies: vec![],
                blocks: vec![],
                notes: vec![],
            },
        ];

        let state = KanbanViewState::new(issues.clone());
        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let sorted = state.sort_issues(issue_refs, CardSort::Priority);

        // P0 should come first
        assert_eq!(sorted[0].id, "TEST-2");
        assert_eq!(sorted[1].id, "TEST-1");
    }

    #[test]
    fn test_sort_issues_by_title() {
        let issues = vec![
            create_test_issue("TEST-1", "Zebra", IssueStatus::Open),
            create_test_issue("TEST-2", "Alpha", IssueStatus::Open),
        ];

        let state = KanbanViewState::new(issues.clone());
        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let sorted = state.sort_issues(issue_refs, CardSort::Title);

        assert_eq!(sorted[0].title, "Alpha");
        assert_eq!(sorted[1].title, "Zebra");
    }
}
