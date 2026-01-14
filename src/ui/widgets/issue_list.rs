//! Issue list widget with sorting and filtering

use crate::beads::models::{Issue, IssueStatus, IssueType};
use crate::models::table_config::TableConfig;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState, Widget},
};

/// Sort column for issue list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Id,
    Title,
    Status,
    Priority,
    Type,
    Created,
    Updated,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Label matching mode for filtering
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LabelMatchMode {
    /// Match issues that have ANY of the specified labels (OR logic)
    #[default]
    Any,
    /// Match issues that have ALL of the specified labels (AND logic)
    All,
}

/// Column filter with cached lowercase versions for performance
#[derive(Debug, Clone)]
pub struct ColumnFilters {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub type_filter: String,
    pub no_assignee: bool,
    pub no_labels: bool,
    pub labels: Vec<String>,
    pub label_match_mode: LabelMatchMode,
    // Cached lowercase versions to avoid repeated allocations
    cached_id_lower: String,
    cached_title_lower: String,
    cached_status_lower: String,
    cached_type_lower: String,
    cached_labels_lower: Vec<String>,
}

impl Default for ColumnFilters {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            status: String::new(),
            priority: String::new(),
            type_filter: String::new(),
            no_assignee: false,
            no_labels: false,
            labels: Vec::new(),
            label_match_mode: LabelMatchMode::Any,
            cached_id_lower: String::new(),
            cached_title_lower: String::new(),
            cached_status_lower: String::new(),
            cached_type_lower: String::new(),
            cached_labels_lower: Vec::new(),
        }
    }
}

impl ColumnFilters {
    /// Update ID filter and refresh cache
    pub fn set_id(&mut self, id: String) {
        self.cached_id_lower = id.to_lowercase();
        self.id = id;
    }

    /// Update title filter and refresh cache
    pub fn set_title(&mut self, title: String) {
        self.cached_title_lower = title.to_lowercase();
        self.title = title;
    }

    /// Update status filter and refresh cache
    pub fn set_status(&mut self, status: String) {
        self.cached_status_lower = status.to_lowercase();
        self.status = status;
    }

    /// Update type filter and refresh cache
    pub fn set_type_filter(&mut self, type_filter: String) {
        self.cached_type_lower = type_filter.to_lowercase();
        self.type_filter = type_filter;
    }

    /// Update labels filter and refresh cache
    pub fn set_labels(&mut self, labels: Vec<String>) {
        self.cached_labels_lower = labels.iter().map(|l| l.to_lowercase()).collect();
        self.labels = labels;
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.title.clear();
        self.status.clear();
        self.priority.clear();
        self.type_filter.clear();
        self.no_assignee = false;
        self.no_labels = false;
        self.labels.clear();
        self.label_match_mode = LabelMatchMode::Any;
        // Clear caches
        self.cached_id_lower.clear();
        self.cached_title_lower.clear();
        self.cached_status_lower.clear();
        self.cached_type_lower.clear();
        self.cached_labels_lower.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
            && self.title.is_empty()
            && self.status.is_empty()
            && self.priority.is_empty()
            && self.type_filter.is_empty()
            && !self.no_assignee
            && !self.no_labels
            && self.labels.is_empty()
    }

    /// Check if an issue matches the current filters (optimized with cached lowercase strings)
    pub fn matches(&self, issue: &Issue) -> bool {
        // If all filters are empty, match everything
        if self.is_empty() {
            return true;
        }

        // Pre-compute lowercase versions once for all filter checks
        // This avoids repeated allocations for the same issue
        let id_lower = if !self.id.is_empty() {
            Some(issue.id.to_lowercase())
        } else {
            None
        };
        let title_lower = if !self.title.is_empty() {
            Some(issue.title.to_lowercase())
        } else {
            None
        };
        let status_lower = if !self.status.is_empty() {
            Some(issue.status.to_string().to_lowercase())
        } else {
            None
        };

        // Check ID filter (substring match, case-insensitive)
        if let Some(id_lower) = id_lower {
            let temp_lower;
            let filter_lower = if !self.cached_id_lower.is_empty() {
                &self.cached_id_lower
            } else {
                temp_lower = self.id.to_lowercase();
                &temp_lower
            };
            if !id_lower.contains(filter_lower) {
                return false;
            }
        }

        // Check title filter (substring match, case-insensitive)
        if let Some(title_lower) = title_lower {
            let temp_lower;
            let filter_lower = if !self.cached_title_lower.is_empty() {
                &self.cached_title_lower
            } else {
                temp_lower = self.title.to_lowercase();
                &temp_lower
            };
            if !title_lower.contains(filter_lower) {
                return false;
            }
        }

        // Check status filter (exact match, case-insensitive)
        if let Some(status_str) = status_lower {
            let temp_lower;
            let filter_lower = if !self.cached_status_lower.is_empty() {
                &self.cached_status_lower
            } else {
                temp_lower = self.status.to_lowercase();
                &temp_lower
            };
            if status_str != *filter_lower {
                return false;
            }
        }

        // Check priority filter (exact match)
        if !self.priority.is_empty() {
            let priority_str = issue.priority.to_string();
            if priority_str != self.priority {
                return false;
            }
        }

        // Check type filter (exact match, case-insensitive)
        if !self.type_filter.is_empty() {
            let type_str = issue.issue_type.to_string().to_lowercase();
            let temp_lower;
            let filter_lower = if !self.cached_type_lower.is_empty() {
                &self.cached_type_lower
            } else {
                temp_lower = self.type_filter.to_lowercase();
                &temp_lower
            };
            if type_str != *filter_lower {
                return false;
            }
        }

        // Check no-assignee filter
        if self.no_assignee && issue.assignee.is_some() {
            return false;
        }

        // Check no-labels filter
        if self.no_labels && !issue.labels.is_empty() {
            return false;
        }

        // Check specific labels filter (with AND/OR logic)
        if !self.labels.is_empty() {
            let temp_labels_lower;
            let labels_lower = if !self.cached_labels_lower.is_empty() {
                &self.cached_labels_lower
            } else {
                temp_labels_lower = self
                    .labels
                    .iter()
                    .map(|l| l.to_lowercase())
                    .collect::<Vec<_>>();
                &temp_labels_lower
            };

            // Pre-compute lowercase versions of issue labels to avoid O(n²) repeated conversions
            let issue_labels_lower: Vec<String> = issue.labels.iter()
                .map(|l| l.to_lowercase())
                .collect();

            let matches = match self.label_match_mode {
                LabelMatchMode::Any => {
                    // OR logic: issue must have at least one of the specified labels
                    labels_lower.iter().any(|filter_label_lower| {
                        issue_labels_lower.iter().any(|issue_label_lower| {
                            issue_label_lower.contains(filter_label_lower)
                        })
                    })
                }
                LabelMatchMode::All => {
                    // AND logic: issue must have all of the specified labels
                    labels_lower.iter().all(|filter_label_lower| {
                        issue_labels_lower.iter().any(|issue_label_lower| {
                            issue_label_lower.contains(filter_label_lower)
                        })
                    })
                }
            };

            if !matches {
                return false;
            }
        }

        true
    }
}

/// Hierarchy information for an issue in the tree
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HierarchyInfo {
    /// Depth level in the hierarchy (0 = root)
    depth: usize,
    /// Tree prefix to display (e.g., "├── ", "└── ", "│   ")
    prefix: String,
    /// Whether this is the last child at its level
    is_last: bool,
}

/// Build a map of issue IDs to their hierarchy information
/// based on dependencies (blocks relationships)
fn build_hierarchy_map(issues: &[&Issue]) -> std::collections::HashMap<String, HierarchyInfo> {
    use std::collections::{HashMap, HashSet};

    // Build parent-child relationships
    // If A blocks B, then A is parent of B
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut has_parent: HashSet<String> = HashSet::new();

    for issue in issues {
        // For each issue that this issue blocks, add it as a child
        if !issue.blocks.is_empty() {
            let children = children_map
                .entry(issue.id.clone())
                .or_default();
            for blocked_id in &issue.blocks {
                children.push(blocked_id.clone());
                has_parent.insert(blocked_id.clone());
            }
        }
    }

    // Find root issues (those with no parents)
    let roots: Vec<String> = issues
        .iter()
        .map(|i| i.id.clone())
        .filter(|id| !has_parent.contains(id))
        .collect();

    // Build hierarchy info via DFS
    let mut hierarchy_map: HashMap<String, HierarchyInfo> = HashMap::new();

    fn build_tree(
        issue_id: &str,
        depth: usize,
        parent_prefixes: &[bool],
        is_last: bool,
        children_map: &HashMap<String, Vec<String>>,
        hierarchy_map: &mut HashMap<String, HierarchyInfo>,
    ) {
        // Build prefix for this node
        let mut prefix = String::new();

        // Add parent prefixes
        for (i, &has_sibling_below) in parent_prefixes.iter().enumerate() {
            if i < parent_prefixes.len() {
                prefix.push_str(if has_sibling_below { "│   " } else { "    " });
            }
        }

        // Add this node's connector
        if depth > 0 {
            prefix.push_str(if is_last { "└── " } else { "├── " });
        }

        hierarchy_map.insert(
            issue_id.to_string(),
            HierarchyInfo {
                depth,
                prefix,
                is_last,
            },
        );

        // Process children
        if let Some(children) = children_map.get(issue_id) {
            let child_count = children.len();
            for (idx, child_id) in children.iter().enumerate() {
                let child_is_last = idx == child_count - 1;

                // Build new parent prefixes for children
                // Reuse the same vector to avoid allocation
                let mut new_prefixes = Vec::with_capacity(parent_prefixes.len() + 1);
                new_prefixes.extend_from_slice(parent_prefixes);
                new_prefixes.push(!is_last);

                build_tree(
                    child_id,
                    depth + 1,
                    &new_prefixes,
                    child_is_last,
                    children_map,
                    hierarchy_map,
                );
            }
        }
    }

    // Build tree for each root
    for (idx, root_id) in roots.iter().enumerate() {
        let is_last_root = idx == roots.len() - 1;
        build_tree(
            root_id,
            0,
            &[],
            is_last_root,
            &children_map,
            &mut hierarchy_map,
        );
    }

    hierarchy_map
}

/// Issue list state
#[derive(Debug)]
pub struct IssueListState {
    table_state: TableState,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    /// Editing state: (issue_index, edit_buffer, cursor_position)
    editing: Option<(usize, String, usize)>,
    /// Quick filters enabled
    filters_enabled: bool,
    /// Column filters
    column_filters: ColumnFilters,
    /// Table configuration (column widths, visibility, order)
    table_config: TableConfig,
    /// Focused column for resize/reorder operations (index in visible columns)
    focused_column: Option<usize>,
}

impl Default for IssueListState {
    fn default() -> Self {
        Self::new()
    }
}

impl IssueListState {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            table_state: state,
            sort_column: SortColumn::Updated,
            sort_direction: SortDirection::Descending,
            editing: None,
            filters_enabled: false,
            column_filters: ColumnFilters::default(),
            table_config: TableConfig::default(),
            focused_column: None,
        }
    }

    /// Toggle quick filters on/off
    pub fn toggle_filters(&mut self) {
        self.filters_enabled = !self.filters_enabled;
    }

    /// Check if filters are enabled
    pub fn filters_enabled(&self) -> bool {
        self.filters_enabled
    }

    /// Get column filters
    pub fn column_filters(&self) -> &ColumnFilters {
        &self.column_filters
    }

    /// Get mutable column filters
    pub fn column_filters_mut(&mut self) -> &mut ColumnFilters {
        &mut self.column_filters
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.column_filters.clear();
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn selected(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.table_state.select(index);
    }

    pub fn sort_by(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_direction = self.sort_direction.toggle();
        } else {
            self.sort_column = column;
            self.sort_direction = SortDirection::Ascending;
        }
    }

    pub fn sort_column(&self) -> SortColumn {
        self.sort_column
    }

    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    /// Start editing the title of the issue at the given index
    pub fn start_editing(&mut self, index: usize, initial_title: String) {
        let cursor_pos = initial_title.len();
        self.editing = Some((index, initial_title, cursor_pos));
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    /// Get the current editing state (index, buffer, cursor)
    pub fn editing_state(&self) -> Option<(usize, &String, usize)> {
        self.editing
            .as_ref()
            .map(|(idx, buf, cursor)| (*idx, buf, *cursor))
    }

    /// Update the edit buffer
    pub fn update_edit_buffer(&mut self, new_text: String) {
        if let Some((idx, _, _)) = self.editing {
            let cursor_pos = new_text.len();
            self.editing = Some((idx, new_text, cursor_pos));
        }
    }

    /// Insert a character at the cursor position
    pub fn insert_char_at_cursor(&mut self, ch: char) {
        if let Some((_idx, ref mut buffer, ref mut cursor)) = self.editing {
            buffer.insert(*cursor, ch);
            *cursor += 1;
        }
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before_cursor(&mut self) {
        if let Some((_idx, ref mut buffer, ref mut cursor)) = self.editing {
            if *cursor > 0 {
                buffer.remove(*cursor - 1);
                *cursor -= 1;
            }
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if let Some((_, _, ref mut cursor)) = self.editing {
            if *cursor > 0 {
                *cursor -= 1;
            }
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if let Some((_, ref buffer, ref mut cursor)) = self.editing {
            if *cursor < buffer.len() {
                *cursor += 1;
            }
        }
    }

    /// Cancel editing and discard changes
    pub fn cancel_editing(&mut self) {
        self.editing = None;
    }

    /// Finish editing and return the edited title
    pub fn finish_editing(&mut self) -> Option<String> {
        self.editing.take().map(|(_, buffer, _)| buffer)
    }

    /// Get the table configuration
    pub fn table_config(&self) -> &TableConfig {
        &self.table_config
    }

    /// Get mutable table configuration
    pub fn table_config_mut(&mut self) -> &mut TableConfig {
        &mut self.table_config
    }

    /// Set the table configuration (for loading from config)
    pub fn set_table_config(&mut self, config: TableConfig) {
        self.table_config = config;
    }

    /// Get the focused column index
    pub fn focused_column(&self) -> Option<usize> {
        self.focused_column
    }

    /// Set the focused column index
    pub fn set_focused_column(&mut self, index: Option<usize>) {
        self.focused_column = index;
    }

    /// Focus next visible column
    pub fn focus_next_column(&mut self) {
        let visible_count = self.table_config.visible_columns().len();
        if visible_count == 0 {
            self.focused_column = None;
            return;
        }

        self.focused_column = Some(match self.focused_column {
            Some(idx) if idx + 1 < visible_count => idx + 1,
            Some(_) => 0,
            None => 0,
        });
    }

    /// Focus previous visible column
    pub fn focus_previous_column(&mut self) {
        let visible_count = self.table_config.visible_columns().len();
        if visible_count == 0 {
            self.focused_column = None;
            return;
        }

        self.focused_column = Some(match self.focused_column {
            Some(idx) if idx > 0 => idx - 1,
            Some(_) => visible_count - 1,
            None => 0,
        });
    }

    /// Move focused column left (reorder)
    pub fn move_focused_column_left(&mut self) {
        if let Some(focused_idx) = self.focused_column {
            let visible_cols = self.table_config.visible_columns();
            if focused_idx > 0 && focused_idx < visible_cols.len() {
                // Get the actual column IDs
                let current_col_id = visible_cols[focused_idx].id;
                let prev_col_id = visible_cols[focused_idx - 1].id;

                // Find their positions in the full column list
                let current_pos = self
                    .table_config
                    .columns
                    .iter()
                    .position(|c| c.id == current_col_id);
                let prev_pos = self
                    .table_config
                    .columns
                    .iter()
                    .position(|c| c.id == prev_col_id);

                if let (Some(curr), Some(prev)) = (current_pos, prev_pos) {
                    self.table_config.reorder_column(curr, prev);
                    self.focused_column = Some(focused_idx - 1);
                }
            }
        }
    }

    /// Move focused column right (reorder)
    pub fn move_focused_column_right(&mut self) {
        if let Some(focused_idx) = self.focused_column {
            let visible_cols = self.table_config.visible_columns();
            if focused_idx < visible_cols.len() - 1 {
                // Get the actual column IDs
                let current_col_id = visible_cols[focused_idx].id;
                let next_col_id = visible_cols[focused_idx + 1].id;

                // Find their positions in the full column list
                let current_pos = self
                    .table_config
                    .columns
                    .iter()
                    .position(|c| c.id == current_col_id);
                let next_pos = self
                    .table_config
                    .columns
                    .iter()
                    .position(|c| c.id == next_col_id);

                if let (Some(curr), Some(next)) = (current_pos, next_pos) {
                    self.table_config.reorder_column(curr, next);
                    self.focused_column = Some(focused_idx + 1);
                }
            }
        }
    }

    /// Decrease width of focused column
    pub fn shrink_focused_column(&mut self) {
        if let Some(focused_idx) = self.focused_column {
            let visible_cols = self.table_config.visible_columns();
            if focused_idx < visible_cols.len() {
                let col_id = visible_cols[focused_idx].id;
                if let Some(col) = self.table_config.get_column(col_id) {
                    let new_width = col.width.saturating_sub(1);
                    self.table_config.set_column_width(col_id, new_width);
                }
            }
        }
    }

    /// Increase width of focused column
    pub fn grow_focused_column(&mut self) {
        if let Some(focused_idx) = self.focused_column {
            let visible_cols = self.table_config.visible_columns();
            if focused_idx < visible_cols.len() {
                let col_id = visible_cols[focused_idx].id;
                if let Some(col) = self.table_config.get_column(col_id) {
                    let new_width = col.width.saturating_add(1);
                    self.table_config.set_column_width(col_id, new_width);
                }
            }
        }
    }
}

/// Issue list widget
pub struct IssueList<'a> {
    issues: Vec<&'a Issue>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    show_details: bool,
    search_query: Option<String>,
    row_height: u16,
    theme: Option<&'a crate::ui::themes::Theme>,
}

impl<'a> IssueList<'a> {
    pub fn new(mut issues: Vec<&'a Issue>) -> Self {
        let sort_column = SortColumn::Updated;
        let sort_direction = SortDirection::Descending;

        // Sort issues by default (updated descending)
        Self::sort_issues(&mut issues, sort_column, sort_direction);

        Self {
            issues,
            sort_column,
            sort_direction,
            show_details: true,
            search_query: None,
            row_height: 1,
            theme: None,
        }
    }

    pub fn theme(mut self, theme: &'a crate::ui::themes::Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn with_sort(mut self, column: SortColumn, direction: SortDirection) -> Self {
        self.sort_column = column;
        self.sort_direction = direction;
        Self::sort_issues(&mut self.issues, column, direction);
        self
    }

    pub fn show_details(mut self, show: bool) -> Self {
        self.show_details = show;
        self
    }

    pub fn search_query(mut self, query: Option<String>) -> Self {
        self.search_query = query;
        self
    }

    pub fn row_height(mut self, height: u16) -> Self {
        self.row_height = height.max(1);
        self
    }

    /// Wrap text to fit within a given width
    /// Returns vector of lines, each line is at most max_width characters
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        if max_width == 0 {
            return vec![String::new()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in text.split_whitespace() {
            let word_len = word.len();

            // If adding this word would exceed width, start a new line
            if current_width + word_len + (if current_width > 0 { 1 } else { 0 }) > max_width {
                // If current line is not empty, push it
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }

                // If word itself is too long, truncate it
                if word_len > max_width {
                    current_line = format!("{}...", &word[..max_width.saturating_sub(3)]);
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                    continue;
                }
            }

            // Add word to current line
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 1;
            }
            current_line.push_str(word);
            current_width += word_len;
        }

        // Push the last line if not empty
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // If no lines were created, return empty line
        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Truncate text with ellipsis if it exceeds max_width
    fn truncate_text(text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            text.to_string()
        } else if max_width <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &text[..max_width - 3])
        }
    }

    /// Highlight matching text in a string
    fn highlight_text(text: &str, query: &str) -> Vec<Span<'static>> {
        if query.is_empty() {
            return vec![Span::raw(text.to_string())];
        }

        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        let mut spans = Vec::new();
        let mut last_end = 0;

        // Find all occurrences of the query in the text
        for (idx, _) in text_lower.match_indices(&query_lower) {
            // Add text before the match
            if idx > last_end {
                spans.push(Span::raw(text[last_end..idx].to_string()));
            }

            // Add the matched text with highlighting
            spans.push(Span::styled(
                text[idx..idx + query.len()].to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = idx + query.len();
        }

        // Add remaining text after the last match
        if last_end < text.len() {
            spans.push(Span::raw(text[last_end..].to_string()));
        }

        spans
    }

    fn sort_issues(issues: &mut Vec<&'a Issue>, column: SortColumn, direction: SortDirection) {
        issues.sort_by(|a, b| {
            let ordering = match column {
                SortColumn::Id => a.id.cmp(&b.id),
                SortColumn::Title => a.title.cmp(&b.title),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Priority => a.priority.cmp(&b.priority),
                SortColumn::Type => a.issue_type.cmp(&b.issue_type),
                SortColumn::Created => a.created.cmp(&b.created),
                SortColumn::Updated => a.updated.cmp(&b.updated),
            };

            match direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
    }

    fn type_symbol(issue_type: &IssueType) -> &'static str {
        match issue_type {
            IssueType::Bug => "🐛",
            IssueType::Feature => "✨",
            IssueType::Task => "📋",
            IssueType::Epic => "🎯",
            IssueType::Chore => "🔧",
        }
    }

    fn status_symbol(status: &IssueStatus) -> &'static str {
        match status {
            IssueStatus::Open => "○",
            IssueStatus::InProgress => "◐",
            IssueStatus::Blocked => "◩",
            IssueStatus::Closed => "✓",
        }
    }

    /// Format a date for display in table cells
    fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
        use chrono::Local;
        let local = date.with_timezone(&Local);
        local.format("%Y-%m-%d %H:%M").to_string()
    }

    /// Get cell content for a given issue and column
    #[allow(clippy::too_many_arguments)]
    fn get_cell_content<'b>(
        issue: &'b Issue,
        column_id: crate::models::table_config::ColumnId,
        search_query: &Option<String>,
        edit_state: Option<(usize, String, usize)>,
        row_idx: usize,
        wrap_width: usize,
        row_height: u16,
        hierarchy_map: &std::collections::HashMap<String, HierarchyInfo>,
        theme: Option<&'b crate::ui::themes::Theme>,
    ) -> Cell<'b> {
        use crate::models::table_config::ColumnId;

        match column_id {
            ColumnId::Type => Cell::from(Self::type_symbol(&issue.issue_type)),

            ColumnId::Id => {
                if let Some(ref query) = search_query {
                    Cell::from(Line::from(Self::highlight_text(&issue.id, query)))
                } else {
                    Cell::from(issue.id.clone())
                }
            }

            ColumnId::Title => {
                // Check if this row is being edited
                let title_text = if let Some((edit_idx, ref edit_buffer, cursor_pos)) = edit_state {
                    if row_idx == edit_idx {
                        let before_cursor = &edit_buffer[..cursor_pos];
                        let after_cursor = &edit_buffer[cursor_pos..];
                        format!("{before_cursor}|{after_cursor}")
                    } else {
                        issue.title.clone()
                    }
                } else {
                    issue.title.clone()
                };

                // Add tree prefix and status symbol if this issue is in the hierarchy
                let title_with_tree = if let Some(hierarchy_info) = hierarchy_map.get(&issue.id) {
                    format!(
                        "{}{} {}",
                        hierarchy_info.prefix,
                        Self::status_symbol(&issue.status),
                        title_text
                    )
                } else {
                    title_text
                };

                // Handle title cell with wrapping support
                if row_height > 1 {
                    // Multi-row mode: wrap text
                    let wrapped_lines = Self::wrap_text(&title_with_tree, wrap_width);
                    let lines: Vec<Line> = wrapped_lines
                        .iter()
                        .take(row_height as usize)
                        .map(|line_text| {
                            if let Some(ref query) = search_query {
                                Line::from(Self::highlight_text(line_text, query))
                            } else {
                                Line::from(line_text.clone())
                            }
                        })
                        .collect();

                    // If wrapped text has fewer lines than row_height, pad with empty lines
                    let mut padded_lines = lines;
                    while padded_lines.len() < row_height as usize {
                        padded_lines.push(Line::from(""));
                    }

                    Cell::from(padded_lines)
                } else {
                    // Single-row mode: truncate
                    let truncated = Self::truncate_text(&title_with_tree, wrap_width);
                    if let Some(ref query) = search_query {
                        Cell::from(Line::from(Self::highlight_text(&truncated, query)))
                    } else {
                        Cell::from(truncated)
                    }
                }
            }

            ColumnId::Status => {
                use crate::ui::themes::Theme;
                let default_theme = Theme::default();
                let theme_ref = theme.unwrap_or(&default_theme);
                let symbol = Theme::status_symbol(&issue.status);
                let color = theme_ref.status_color(&issue.status);
                Cell::from(Span::styled(
                    format!("{} {:?}", symbol, issue.status),
                    Style::default().fg(color),
                ))
            }

            ColumnId::Priority => {
                use crate::ui::themes::Theme;
                let default_theme = Theme::default();
                let theme_ref = theme.unwrap_or(&default_theme);
                let symbol = Theme::priority_symbol(&issue.priority);
                let color = theme_ref.priority_color(&issue.priority);
                Cell::from(Span::styled(
                    format!("{} {:?}", symbol, issue.priority),
                    Style::default().fg(color),
                ))
            }

            ColumnId::Assignee => {
                let text = issue.assignee.as_deref().unwrap_or("-");
                if let Some(ref query) = search_query {
                    Cell::from(Line::from(Self::highlight_text(text, query)))
                } else {
                    Cell::from(text.to_string())
                }
            }

            ColumnId::Labels => {
                let text = if issue.labels.is_empty() {
                    "-".to_string()
                } else {
                    issue.labels.join(", ")
                };

                if row_height > 1 && text.len() > wrap_width {
                    let wrapped_lines = Self::wrap_text(&text, wrap_width);
                    let lines: Vec<Line> = wrapped_lines
                        .iter()
                        .take(row_height as usize)
                        .map(|line_text| {
                            if let Some(ref query) = search_query {
                                Line::from(Self::highlight_text(line_text, query))
                            } else {
                                Line::from(line_text.clone())
                            }
                        })
                        .collect();

                    let mut padded_lines = lines;
                    while padded_lines.len() < row_height as usize {
                        padded_lines.push(Line::from(""));
                    }
                    Cell::from(padded_lines)
                } else {
                    let truncated = Self::truncate_text(&text, wrap_width);
                    if let Some(ref query) = search_query {
                        Cell::from(Line::from(Self::highlight_text(&truncated, query)))
                    } else {
                        Cell::from(truncated)
                    }
                }
            }

            ColumnId::Updated => Cell::from(Self::format_date(&issue.updated)),

            ColumnId::Created => Cell::from(Self::format_date(&issue.created)),
        }
    }

    /// Render the filter row below the table
    fn render_filter_row(area: Rect, buf: &mut Buffer, state: &IssueListState) {
        use ratatui::widgets::Paragraph;

        let filters = state.column_filters();
        let mut filter_parts = Vec::new();

        // Show active filters
        if !filters.id.is_empty() {
            filter_parts.push(format!("ID: {}", filters.id));
        }
        if !filters.title.is_empty() {
            filter_parts.push(format!("Title: {}", filters.title));
        }
        if !filters.status.is_empty() {
            filter_parts.push(format!("Status: {}", filters.status));
        }
        if !filters.priority.is_empty() {
            filter_parts.push(format!("Priority: {}", filters.priority));
        }
        if !filters.type_filter.is_empty() {
            filter_parts.push(format!("Type: {}", filters.type_filter));
        }

        let filter_text = if filter_parts.is_empty() {
            "Quick Filters: [No filters active] Press 'f' to toggle filters".to_string()
        } else {
            format!(
                "Quick Filters: {} | Press 'f' to toggle",
                filter_parts.join(" | ")
            )
        };

        let filter_style = if filter_parts.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };

        let paragraph = Paragraph::new(filter_text)
            .style(filter_style)
            .block(Block::default().borders(Borders::ALL).title("Filters"));

        paragraph.render(area, buf);
    }
}

impl<'a> StatefulWidget for IssueList<'a> {
    type State = IssueListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Split area if filters are enabled
        let (table_area, filter_area) = if state.filters_enabled() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),    // Table area
                    Constraint::Length(3), // Filter row area
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        // Update sorting if state has changed
        let mut issues = self.issues;
        if state.sort_column != self.sort_column || state.sort_direction != self.sort_direction {
            Self::sort_issues(&mut issues, state.sort_column, state.sort_direction);
        }

        // Build hierarchy map for tree rendering
        let hierarchy_map = build_hierarchy_map(&issues);

        // Build header from TableConfig
        let sort_indicator = match state.sort_direction {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        };

        // Get visible columns from table config
        let visible_columns = state.table_config().visible_columns();
        let focused_col_idx = state.focused_column();

        let header_cells: Vec<Cell> = visible_columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                // Map ColumnId to SortColumn to check if this column is sorted
                let is_sorted = match col.id {
                    crate::models::table_config::ColumnId::Type => {
                        state.sort_column == SortColumn::Type
                    }
                    crate::models::table_config::ColumnId::Id => {
                        state.sort_column == SortColumn::Id
                    }
                    crate::models::table_config::ColumnId::Title => {
                        state.sort_column == SortColumn::Title
                    }
                    crate::models::table_config::ColumnId::Status => {
                        state.sort_column == SortColumn::Status
                    }
                    crate::models::table_config::ColumnId::Priority => {
                        state.sort_column == SortColumn::Priority
                    }
                    crate::models::table_config::ColumnId::Updated => {
                        state.sort_column == SortColumn::Updated
                    }
                    crate::models::table_config::ColumnId::Created => {
                        state.sort_column == SortColumn::Created
                    }
                    _ => false,
                };

                let label = if is_sorted {
                    format!("{} {}", col.label, sort_indicator)
                } else {
                    col.label.clone()
                };

                // Highlight focused column
                let style = if Some(idx) == focused_col_idx {
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                };

                Cell::from(Span::styled(label, style))
            })
            .collect();

        let header = Row::new(header_cells)
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        // Get editing state for rendering (clone to avoid borrow conflicts)
        let editing_state = state
            .editing_state()
            .map(|(idx, buf, cursor)| (idx, buf.clone(), cursor));

        // Virtual scrolling: Calculate visible range to optimize rendering
        let total_issues = issues.len();
        let viewport_height = table_area
            .height
            .saturating_sub(3) // Subtract borders (2) and header (1)
            as usize;

        // Calculate visible window with buffer
        let buffer_size = viewport_height.saturating_mul(2); // Render 2x viewport above/below
        let selected_idx = state.table_state.selected().unwrap_or(0);

        // Calculate start and end of visible window
        let start_idx = selected_idx.saturating_sub(buffer_size).min(total_issues);
        let end_idx = (selected_idx + viewport_height + buffer_size)
            .min(total_issues)
            .max(start_idx);

        // Only create rows for visible range
        let visible_issues = if start_idx < end_idx {
            &issues[start_idx..end_idx]
        } else {
            &[]
        };

        // Build rows only for visible range, using TableConfig columns
        let row_height_to_use = state.table_config().row_height;
        let rows: Vec<Row> = visible_issues
            .iter()
            .enumerate()
            .map(|(visible_idx, issue)| {
                let row_idx = start_idx + visible_idx; // Actual index in full list

                // Generate cells for all visible columns
                let cells: Vec<Cell> = visible_columns
                    .iter()
                    .map(|col| {
                        let wrap_width = col.width as usize;
                        Self::get_cell_content(
                            issue,
                            col.id,
                            &self.search_query,
                            editing_state.clone(),
                            row_idx,
                            wrap_width,
                            row_height_to_use,
                            &hierarchy_map,
                            self.theme,
                        )
                    })
                    .collect();

                Row::new(cells).height(row_height_to_use)
            })
            .collect();

        // Build table widths from TableConfig
        let widths: Vec<Constraint> = visible_columns
            .iter()
            .map(|col| {
                // Use column width as preferred, but allow Title to be flexible
                if col.id == crate::models::table_config::ColumnId::Title {
                    Constraint::Min(col.width_constraints.min)
                } else {
                    Constraint::Length(col.width)
                }
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Issues ({}/{})", total_issues, total_issues)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // Adjust TableState selection to be relative to the windowed view
        let original_selected = state.table_state.selected();
        if let Some(selected) = original_selected {
            if selected >= start_idx && selected < end_idx {
                // Selection is within visible window, adjust to relative index
                state.table_state.select(Some(selected - start_idx));
            } else {
                // Selection is outside window, don't show highlight
                state.table_state.select(None);
            }
        }

        StatefulWidget::render(table, table_area, buf, &mut state.table_state);

        // Restore original selection
        state.table_state.select(original_selected);

        // Render filter row if enabled
        if let Some(filter_area) = filter_area {
            Self::render_filter_row(filter_area, buf, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str, priority: Priority, status: IssueStatus) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            issue_type: IssueType::Task,
            status,
            priority,
            labels: vec![],
            assignee: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
        }
    }

    #[test]
    fn test_issue_list_state_creation() {
        let state = IssueListState::new();
        assert_eq!(state.selected(), Some(0));
        assert_eq!(state.sort_column(), SortColumn::Updated);
        assert_eq!(state.sort_direction(), SortDirection::Descending);
    }

    #[test]
    fn test_issue_list_state_navigation() {
        let mut state = IssueListState::new();

        state.select_next(5);
        assert_eq!(state.selected(), Some(1));

        state.select_previous(5);
        assert_eq!(state.selected(), Some(0));

        state.select_previous(5);
        assert_eq!(state.selected(), Some(4)); // Wraps around
    }

    #[test]
    fn test_sort_direction_toggle() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);

        let dir = SortDirection::Descending;
        assert_eq!(dir.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_issue_list_state_sorting() {
        let mut state = IssueListState::new();

        state.sort_by(SortColumn::Priority);
        assert_eq!(state.sort_column(), SortColumn::Priority);
        assert_eq!(state.sort_direction(), SortDirection::Ascending);

        state.sort_by(SortColumn::Priority);
        assert_eq!(state.sort_direction(), SortDirection::Descending);

        state.sort_by(SortColumn::Title);
        assert_eq!(state.sort_column(), SortColumn::Title);
        assert_eq!(state.sort_direction(), SortDirection::Ascending);
    }

    #[test]
    fn test_issue_list_sorting() {
        let issue1 = create_test_issue("beads-001", "Task A", Priority::P2, IssueStatus::Open);
        let issue2 = create_test_issue("beads-002", "Task B", Priority::P1, IssueStatus::Open);
        let issue3 = create_test_issue("beads-003", "Task C", Priority::P3, IssueStatus::Open);

        let issues = vec![&issue1, &issue2, &issue3];
        let mut sorted_issues = issues.clone();

        IssueList::sort_issues(
            &mut sorted_issues,
            SortColumn::Priority,
            SortDirection::Ascending,
        );
        assert_eq!(sorted_issues[0].id, "beads-002"); // P1
        assert_eq!(sorted_issues[1].id, "beads-001"); // P2
        assert_eq!(sorted_issues[2].id, "beads-003"); // P3
    }

    #[test]
    fn test_highlight_text() {
        // Test empty query
        let spans = IssueList::highlight_text("hello world", "");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "hello world");

        // Test single match
        let spans = IssueList::highlight_text("hello world", "world");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "hello ");
        assert_eq!(spans[1].content, "world");
        assert_eq!(spans[1].style.fg, Some(Color::Black));
        assert_eq!(spans[1].style.bg, Some(Color::Yellow));

        // Test case-insensitive match
        let spans = IssueList::highlight_text("Hello World", "world");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "Hello ");
        assert_eq!(spans[1].content, "World");

        // Test multiple matches
        let spans = IssueList::highlight_text("test test test", "test");
        assert_eq!(spans.len(), 5); // test, " ", test, " ", test
        assert_eq!(spans[0].content, "test");
        assert_eq!(spans[1].content, " ");
        assert_eq!(spans[2].content, "test");
        assert_eq!(spans[3].content, " ");
        assert_eq!(spans[4].content, "test");

        // Test match at the beginning
        let spans = IssueList::highlight_text("world hello", "world");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "world");
        assert_eq!(spans[1].content, " hello");

        // Test match at the end
        let spans = IssueList::highlight_text("hello world", "world");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "hello ");
        assert_eq!(spans[1].content, "world");

        // Test no match
        let spans = IssueList::highlight_text("hello world", "xyz");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "hello world");
    }

    #[test]
    fn test_wrap_text_simple() {
        let text = "This is a test";
        let wrapped = IssueList::wrap_text(text, 10);
        assert_eq!(wrapped, vec!["This is a", "test"]);
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "ThisIsAVeryLongWordThatShouldBeTruncated";
        let wrapped = IssueList::wrap_text(text, 10);
        assert_eq!(wrapped[0], "ThisIsA...");
    }

    #[test]
    fn test_wrap_text_fits_exactly() {
        let text = "Exact";
        let wrapped = IssueList::wrap_text(text, 5);
        assert_eq!(wrapped, vec!["Exact"]);
    }

    #[test]
    fn test_wrap_text_empty() {
        let wrapped = IssueList::wrap_text("", 10);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn test_wrap_text_single_word() {
        let wrapped = IssueList::wrap_text("Word", 10);
        assert_eq!(wrapped, vec!["Word"]);
    }

    #[test]
    fn test_wrap_text_zero_width() {
        let wrapped = IssueList::wrap_text("test", 0);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn test_wrap_text_multiple_lines() {
        let text = "This is a longer text that will wrap across multiple lines when rendered";
        let wrapped = IssueList::wrap_text(text, 20);
        assert!(wrapped.len() >= 3);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_truncate_text_short() {
        assert_eq!(IssueList::truncate_text("short", 10), "short");
    }

    #[test]
    fn test_truncate_text_long() {
        assert_eq!(
            IssueList::truncate_text("This is a very long text", 10),
            "This is..."
        );
    }

    #[test]
    fn test_truncate_text_very_short_width() {
        assert_eq!(IssueList::truncate_text("test", 2), "...");
    }

    #[test]
    fn test_truncate_text_exact_width() {
        assert_eq!(IssueList::truncate_text("exactly10c", 10), "exactly10c");
    }

    #[test]
    fn test_truncate_text_one_over() {
        assert_eq!(IssueList::truncate_text("exactly11ch", 10), "exactly...");
    }

    // ColumnFilters tests
    #[test]
    fn test_column_filters_empty() {
        let filters = ColumnFilters::default();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_column_filters_matches_id_substring() {
        let filters = ColumnFilters {
            id: "tui".to_string(),
            ..Default::default()
        };

        let issue = create_test_issue("beads-tui-123", "Test", Priority::P2, IssueStatus::Open);
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_column_filters_matches_case_insensitive() {
        let filters = ColumnFilters {
            id: "TUI".to_string(),
            ..Default::default()
        };

        let issue = create_test_issue("beads-tui-123", "Test", Priority::P2, IssueStatus::Open);
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_column_filters_no_match() {
        let filters = ColumnFilters {
            id: "xyz".to_string(),
            ..Default::default()
        };

        let issue = create_test_issue("beads-tui-123", "Test", Priority::P2, IssueStatus::Open);
        assert!(!filters.matches(&issue));
    }

    #[test]
    fn test_column_filters_clear() {
        let mut filters = ColumnFilters {
            id: "test".to_string(),
            title: "example".to_string(),
            ..Default::default()
        };
        filters.clear();
        assert!(filters.is_empty());
    }

    // IssueListState editing tests
    #[test]
    fn test_issue_list_state_start_editing() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test Title".to_string());
        assert!(state.is_editing());
        assert_eq!(
            state.editing_state(),
            Some((0, &"Test Title".to_string(), 10))
        );
    }

    #[test]
    fn test_issue_list_state_insert_char() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test".to_string());
        state.insert_char_at_cursor('X');
        assert_eq!(state.editing_state(), Some((0, &"TestX".to_string(), 5)));
    }

    #[test]
    fn test_issue_list_state_delete_char() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test".to_string());
        state.delete_char_before_cursor();
        assert_eq!(state.editing_state(), Some((0, &"Tes".to_string(), 3)));
    }

    #[test]
    fn test_issue_list_state_move_cursor() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test".to_string());
        state.move_cursor_left();
        state.move_cursor_left();
        assert_eq!(state.editing_state(), Some((0, &"Test".to_string(), 2)));
        state.move_cursor_right();
        assert_eq!(state.editing_state(), Some((0, &"Test".to_string(), 3)));
    }

    #[test]
    fn test_issue_list_state_cancel_editing() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test".to_string());
        state.cancel_editing();
        assert!(!state.is_editing());
    }

    #[test]
    fn test_issue_list_state_finish_editing() {
        let mut state = IssueListState::new();
        state.start_editing(0, "Test".to_string());
        let result = state.finish_editing();
        assert_eq!(result, Some("Test".to_string()));
        assert!(!state.is_editing());
    }

    // Column focus tests
    #[test]
    fn test_focus_next_column_wraparound() {
        let mut state = IssueListState::new();
        state.set_focused_column(Some(0));
        let visible_count = state.table_config().visible_columns().len();

        // Advance through all columns and wrap back to start
        for _ in 0..visible_count {
            state.focus_next_column();
        }
        assert_eq!(state.focused_column(), Some(0)); // Wraps around
    }

    #[test]
    fn test_focus_previous_column_wraparound() {
        let mut state = IssueListState::new();
        state.set_focused_column(Some(0));
        state.focus_previous_column();
        let visible_count = state.table_config().visible_columns().len();
        assert_eq!(state.focused_column(), Some(visible_count - 1)); // Wraps to end
    }

    // IssueList widget builder tests
    #[test]
    fn test_issue_list_with_sort() {
        let issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        let list =
            IssueList::new(vec![&issue]).with_sort(SortColumn::Priority, SortDirection::Descending);
        assert_eq!(list.sort_column, SortColumn::Priority);
        assert_eq!(list.sort_direction, SortDirection::Descending);
    }

    #[test]
    fn test_issue_list_show_details() {
        let issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        let list = IssueList::new(vec![&issue]).show_details(false);
        assert!(!list.show_details);
    }

    #[test]
    fn test_issue_list_row_height_clamped() {
        let issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        let list = IssueList::new(vec![&issue]).row_height(0);
        assert_eq!(list.row_height, 1); // Clamped to minimum 1
    }

    // Empty/null filter tests
    #[test]
    fn test_column_filters_no_assignee() {
        let mut issue_with_assignee =
            create_test_issue("beads-001", "Test 1", Priority::P2, IssueStatus::Open);
        issue_with_assignee.assignee = Some("user1".to_string());

        let issue_no_assignee =
            create_test_issue("beads-002", "Test 2", Priority::P2, IssueStatus::Open);

        let filters = ColumnFilters {
            no_assignee: true,
            ..Default::default()
        };

        // Should match issue with no assignee
        assert!(filters.matches(&issue_no_assignee));

        // Should not match issue with assignee
        assert!(!filters.matches(&issue_with_assignee));
    }

    #[test]
    fn test_column_filters_no_labels() {
        let mut issue_with_labels =
            create_test_issue("beads-001", "Test 1", Priority::P2, IssueStatus::Open);
        issue_with_labels.labels = vec!["bug".to_string(), "frontend".to_string()];

        let issue_no_labels =
            create_test_issue("beads-002", "Test 2", Priority::P2, IssueStatus::Open);

        let filters = ColumnFilters {
            no_labels: true,
            ..Default::default()
        };

        // Should match issue with no labels
        assert!(filters.matches(&issue_no_labels));

        // Should not match issue with labels
        assert!(!filters.matches(&issue_with_labels));
    }

    #[test]
    fn test_column_filters_no_assignee_and_no_labels() {
        let mut issue_has_both =
            create_test_issue("beads-001", "Test 1", Priority::P2, IssueStatus::Open);
        issue_has_both.assignee = Some("user1".to_string());
        issue_has_both.labels = vec!["bug".to_string()];

        let mut issue_no_assignee =
            create_test_issue("beads-002", "Test 2", Priority::P2, IssueStatus::Open);
        issue_no_assignee.labels = vec!["bug".to_string()];

        let mut issue_no_labels =
            create_test_issue("beads-003", "Test 3", Priority::P2, IssueStatus::Open);
        issue_no_labels.assignee = Some("user1".to_string());

        let issue_has_neither =
            create_test_issue("beads-004", "Test 4", Priority::P2, IssueStatus::Open);

        let filters = ColumnFilters {
            no_assignee: true,
            no_labels: true,
            ..Default::default()
        };

        // Should only match issue with neither assignee nor labels
        assert!(filters.matches(&issue_has_neither));

        // Should not match issues with assignee or labels
        assert!(!filters.matches(&issue_has_both));
        assert!(!filters.matches(&issue_no_assignee));
        assert!(!filters.matches(&issue_no_labels));
    }

    #[test]
    fn test_column_filters_clear_resets_empty_filters() {
        let mut filters = ColumnFilters {
            no_assignee: true,
            no_labels: true,
            status: "Open".to_string(),
            ..Default::default()
        };

        filters.clear();

        assert!(!filters.no_assignee);
        assert!(!filters.no_labels);
        assert!(filters.status.is_empty());
    }

    #[test]
    fn test_column_filters_is_empty_with_empty_filters() {
        let mut filters = ColumnFilters::default();
        assert!(filters.is_empty());

        // Setting no_assignee should make it non-empty
        filters.no_assignee = true;
        assert!(!filters.is_empty());

        filters.no_assignee = false;
        assert!(filters.is_empty());

        // Setting no_labels should make it non-empty
        filters.no_labels = true;
        assert!(!filters.is_empty());
    }

    #[test]
    fn test_column_filters_combined_with_other_filters() {
        let issue1 = create_test_issue(
            "beads-001",
            "Feature request",
            Priority::P2,
            IssueStatus::Open,
        );

        let mut issue2 = create_test_issue("beads-002", "Bug fix", Priority::P1, IssueStatus::Open);
        issue2.assignee = Some("user1".to_string());

        let issue3 = create_test_issue(
            "beads-003",
            "Feature request",
            Priority::P2,
            IssueStatus::Closed,
        );

        let filters = ColumnFilters {
            no_assignee: true,
            status: "Open".to_string(),
            ..Default::default()
        };

        // Should match issue1 (no assignee AND status is Open)
        assert!(filters.matches(&issue1));

        // Should not match issue2 (has assignee)
        assert!(!filters.matches(&issue2));

        // Should not match issue3 (status is not Open)
        assert!(!filters.matches(&issue3));
    }

    #[test]
    fn test_label_filter_any_mode_single_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec![
            "bug".to_string(),
            "frontend".to_string(),
            "urgent".to_string(),
        ];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string()],
            label_match_mode: LabelMatchMode::Any,
            ..Default::default()
        };

        // Should match because issue has the "bug" label
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_any_mode_multiple_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["bug".to_string(), "frontend".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "backend".to_string()],
            label_match_mode: LabelMatchMode::Any,
            ..Default::default()
        };

        // Should match because issue has at least one of the specified labels ("bug")
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_any_mode_no_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["documentation".to_string(), "ui".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "backend".to_string()],
            label_match_mode: LabelMatchMode::Any,
            ..Default::default()
        };

        // Should not match because issue doesn't have any of the specified labels
        assert!(!filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_all_mode_all_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec![
            "bug".to_string(),
            "frontend".to_string(),
            "urgent".to_string(),
        ];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "frontend".to_string()],
            label_match_mode: LabelMatchMode::All,
            ..Default::default()
        };

        // Should match because issue has all of the specified labels
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_all_mode_partial_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["bug".to_string(), "ui".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "frontend".to_string()],
            label_match_mode: LabelMatchMode::All,
            ..Default::default()
        };

        // Should not match because issue doesn't have all of the specified labels (missing "frontend")
        assert!(!filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_all_mode_no_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["documentation".to_string(), "ui".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "backend".to_string()],
            label_match_mode: LabelMatchMode::All,
            ..Default::default()
        };

        // Should not match because issue doesn't have any of the specified labels
        assert!(!filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_case_insensitive() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["BUG".to_string(), "Frontend".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string(), "frontend".to_string()],
            label_match_mode: LabelMatchMode::All,
            ..Default::default()
        };

        // Should match because label matching is case-insensitive
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_substring_match() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["frontend-bug".to_string(), "ui-component".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string()],
            label_match_mode: LabelMatchMode::Any,
            ..Default::default()
        };

        // Should match because "bug" is a substring of "frontend-bug"
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_filter_combined_with_status() {
        let mut issue1 = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue1.labels = vec!["bug".to_string(), "frontend".to_string()];

        let mut issue2 = create_test_issue("beads-002", "Test", Priority::P2, IssueStatus::Closed);
        issue2.labels = vec!["bug".to_string(), "frontend".to_string()];

        let filters = ColumnFilters {
            labels: vec!["bug".to_string()],
            label_match_mode: LabelMatchMode::Any,
            status: "Open".to_string(),
            ..Default::default()
        };

        // Should match issue1 (has label AND status is Open)
        assert!(filters.matches(&issue1));

        // Should not match issue2 (has label BUT status is not Open)
        assert!(!filters.matches(&issue2));
    }

    #[test]
    fn test_label_filter_empty_matches_all() {
        let mut issue = create_test_issue("beads-001", "Test", Priority::P2, IssueStatus::Open);
        issue.labels = vec!["bug".to_string()];

        let filters = ColumnFilters::default();

        // Should match because labels filter is empty
        assert!(filters.matches(&issue));
    }

    #[test]
    fn test_label_match_mode_default() {
        let filters = ColumnFilters::default();

        // Default label_match_mode should be Any
        assert_eq!(filters.label_match_mode, LabelMatchMode::Any);
    }

    #[test]
    fn test_label_filter_clear_resets_labels_and_mode() {
        let mut filters = ColumnFilters {
            labels: vec!["bug".to_string(), "frontend".to_string()],
            label_match_mode: LabelMatchMode::All,
            ..Default::default()
        };

        filters.clear();

        assert!(filters.labels.is_empty());
        assert_eq!(filters.label_match_mode, LabelMatchMode::Any);
    }

    #[test]
    fn test_label_filter_is_empty_with_labels() {
        let mut filters = ColumnFilters::default();
        assert!(filters.is_empty());

        // Setting labels should make it non-empty
        filters.labels = vec!["bug".to_string()];
        assert!(!filters.is_empty());

        filters.labels.clear();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_virtual_scrolling_large_list() {
        // Create a large list of issues to test virtual scrolling
        let mut issues = Vec::new();
        for i in 0..5000 {
            issues.push(create_test_issue(
                &format!("beads-{:04}", i),
                &format!("Issue {}", i),
                Priority::P2,
                IssueStatus::Open,
            ));
        }

        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let widget = IssueList::new(issue_refs);

        // Test that widget can be created with large list
        assert_eq!(widget.issues.len(), 5000);

        // Test state with large list
        let mut state = IssueListState::new();
        state.select(Some(2500)); // Select middle item

        // Navigation should work
        state.select_next(5000);
        assert_eq!(state.selected(), Some(2501));

        state.select_previous(5000);
        assert_eq!(state.selected(), Some(2500));

        // Jump to end
        state.select(Some(4999));
        assert_eq!(state.selected(), Some(4999));

        // Wrap around
        state.select_next(5000);
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_virtual_scrolling_performance() {
        // Create a very large list to verify performance optimization
        let mut issues = Vec::new();
        for i in 0..10000 {
            issues.push(create_test_issue(
                &format!("beads-{:05}", i),
                &format!("Performance test issue number {}", i),
                if i % 5 == 0 {
                    Priority::P0
                } else if i % 5 == 1 {
                    Priority::P1
                } else if i % 5 == 2 {
                    Priority::P2
                } else if i % 5 == 3 {
                    Priority::P3
                } else {
                    Priority::P4
                },
                if i % 4 == 0 {
                    IssueStatus::Open
                } else if i % 4 == 1 {
                    IssueStatus::InProgress
                } else if i % 4 == 2 {
                    IssueStatus::Blocked
                } else {
                    IssueStatus::Closed
                },
            ));
        }

        let issue_refs: Vec<&Issue> = issues.iter().collect();

        // Creating the widget should be fast even with 10k issues
        let start = std::time::Instant::now();
        let _widget = IssueList::new(issue_refs);
        let duration = start.elapsed();

        // Should complete in under 100ms even with 10k issues
        assert!(
            duration.as_millis() < 100,
            "Widget creation took {}ms, expected < 100ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_build_hierarchy_map_simple() {
        use chrono::Utc;

        // Create parent issue that blocks child
        let parent = Issue {
            id: "parent".to_string(),
            title: "Parent Issue".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P1,
            issue_type: IssueType::Epic,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec!["child".to_string()],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let child = Issue {
            id: "child".to_string(),
            title: "Child Issue".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["parent".to_string()],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let issues = vec![&parent, &child];
        let hierarchy_map = build_hierarchy_map(&issues);

        // Check that both issues are in the map
        assert!(hierarchy_map.contains_key("parent"));
        assert!(hierarchy_map.contains_key("child"));

        // Parent should be at depth 0 (root)
        let parent_info = hierarchy_map.get("parent").unwrap();
        assert_eq!(parent_info.depth, 0);
        assert_eq!(parent_info.prefix, ""); // Root has no prefix

        // Child should be at depth 1 with tree prefix
        let child_info = hierarchy_map.get("child").unwrap();
        assert_eq!(child_info.depth, 1);
        assert!(child_info.prefix.contains("└──") || child_info.prefix.contains("├──"));
    }

    #[test]
    fn test_build_hierarchy_map_multiple_children() {
        use chrono::Utc;

        let parent = Issue {
            id: "parent".to_string(),
            title: "Parent".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P1,
            issue_type: IssueType::Epic,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![
                "child1".to_string(),
                "child2".to_string(),
                "child3".to_string(),
            ],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let child1 = Issue {
            id: "child1".to_string(),
            title: "Child 1".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["parent".to_string()],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let child2 = Issue {
            id: "child2".to_string(),
            title: "Child 2".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["parent".to_string()],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let child3 = Issue {
            id: "child3".to_string(),
            title: "Child 3".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["parent".to_string()],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let issues = vec![&parent, &child1, &child2, &child3];
        let hierarchy_map = build_hierarchy_map(&issues);

        // All issues should be in the map
        assert_eq!(hierarchy_map.len(), 4);

        // All children should be at depth 1
        assert_eq!(hierarchy_map.get("child1").unwrap().depth, 1);
        assert_eq!(hierarchy_map.get("child2").unwrap().depth, 1);
        assert_eq!(hierarchy_map.get("child3").unwrap().depth, 1);

        // Last child should have └── prefix
        let child3_prefix = &hierarchy_map.get("child3").unwrap().prefix;
        assert!(child3_prefix.contains("└──"));

        // Other children should have ├── prefix
        let child1_prefix = &hierarchy_map.get("child1").unwrap().prefix;
        assert!(child1_prefix.contains("├──"));
    }

    #[test]
    fn test_build_hierarchy_map_deep_tree() {
        use chrono::Utc;

        // Create a 3-level deep tree: root -> level1 -> level2
        let root = Issue {
            id: "root".to_string(),
            title: "Root".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P1,
            issue_type: IssueType::Epic,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec!["level1".to_string()],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let level1 = Issue {
            id: "level1".to_string(),
            title: "Level 1".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Feature,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["root".to_string()],
            blocks: vec!["level2".to_string()],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let level2 = Issue {
            id: "level2".to_string(),
            title: "Level 2".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec!["level1".to_string()],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let issues = vec![&root, &level1, &level2];
        let hierarchy_map = build_hierarchy_map(&issues);

        // Check depths
        assert_eq!(hierarchy_map.get("root").unwrap().depth, 0);
        assert_eq!(hierarchy_map.get("level1").unwrap().depth, 1);
        assert_eq!(hierarchy_map.get("level2").unwrap().depth, 2);

        // Check prefixes contain tree characters
        let level2_prefix = &hierarchy_map.get("level2").unwrap().prefix;
        assert!(level2_prefix.contains("└──") || level2_prefix.contains("├──"));
        // Should have indentation from parent levels
        assert!(level2_prefix.len() > 4);
    }

    #[test]
    fn test_build_hierarchy_map_no_hierarchy() {
        use chrono::Utc;

        // Create issues with no blocking relationships
        let issue1 = Issue {
            id: "issue1".to_string(),
            title: "Issue 1".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let issue2 = Issue {
            id: "issue2".to_string(),
            title: "Issue 2".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let issues = vec![&issue1, &issue2];
        let hierarchy_map = build_hierarchy_map(&issues);

        // All issues should be roots (depth 0, no prefix)
        assert_eq!(hierarchy_map.get("issue1").unwrap().depth, 0);
        assert_eq!(hierarchy_map.get("issue1").unwrap().prefix, "");
        assert_eq!(hierarchy_map.get("issue2").unwrap().depth, 0);
        assert_eq!(hierarchy_map.get("issue2").unwrap().prefix, "");
    }
}
