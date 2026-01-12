//! Issue list widget with sorting and filtering

use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
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

/// Column filter
#[derive(Debug, Clone, Default)]
pub struct ColumnFilters {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub type_filter: String,
}

impl ColumnFilters {
    pub fn clear(&mut self) {
        self.id.clear();
        self.title.clear();
        self.status.clear();
        self.priority.clear();
        self.type_filter.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
            && self.title.is_empty()
            && self.status.is_empty()
            && self.priority.is_empty()
            && self.type_filter.is_empty()
    }

    /// Check if an issue matches the current filters
    pub fn matches(&self, issue: &Issue) -> bool {
        // If all filters are empty, match everything
        if self.is_empty() {
            return true;
        }

        // Check ID filter (substring match, case-insensitive)
        if !self.id.is_empty() {
            let id_lower = issue.id.to_lowercase();
            let filter_lower = self.id.to_lowercase();
            if !id_lower.contains(&filter_lower) {
                return false;
            }
        }

        // Check title filter (substring match, case-insensitive)
        if !self.title.is_empty() {
            let title_lower = issue.title.to_lowercase();
            let filter_lower = self.title.to_lowercase();
            if !title_lower.contains(&filter_lower) {
                return false;
            }
        }

        // Check status filter (exact match, case-insensitive)
        if !self.status.is_empty() {
            let status_str = issue.status.to_string().to_lowercase();
            let filter_lower = self.status.to_lowercase();
            if status_str != filter_lower {
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
            let filter_lower = self.type_filter.to_lowercase();
            if type_str != filter_lower {
                return false;
            }
        }

        true
    }
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
}

/// Issue list widget
pub struct IssueList<'a> {
    issues: Vec<&'a Issue>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    show_details: bool,
    search_query: Option<String>,
    row_height: u16,
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
        }
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

    fn priority_color(priority: &Priority) -> Color {
        match priority {
            Priority::P0 => Color::Red,
            Priority::P1 => Color::LightRed,
            Priority::P2 => Color::Yellow,
            Priority::P3 => Color::Blue,
            Priority::P4 => Color::Gray,
        }
    }

    fn status_color(status: &IssueStatus) -> Color {
        match status {
            IssueStatus::Open => Color::Green,
            IssueStatus::InProgress => Color::Cyan,
            IssueStatus::Blocked => Color::Red,
            IssueStatus::Closed => Color::Gray,
        }
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
            format!("Quick Filters: {} | Press 'f' to toggle", filter_parts.join(" | "))
        };

        let filter_style = if filter_parts.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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
                    Constraint::Min(5),     // Table area
                    Constraint::Length(3),  // Filter row area
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

        // Build header
        let sort_indicator = match state.sort_direction {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        };

        let header_cells = vec![
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Type {
                    format!("Type {sort_indicator}")
                } else {
                    "Type".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Id {
                    format!("ID {sort_indicator}")
                } else {
                    "ID".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Title {
                    format!("Title {sort_indicator}")
                } else {
                    "Title".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Status {
                    format!("Status {sort_indicator}")
                } else {
                    "Status".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Priority {
                    format!("Priority {sort_indicator}")
                } else {
                    "Priority".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        let header = Row::new(header_cells)
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        // Get editing state for rendering
        let editing_state = state.editing_state();

        // Build rows
        let rows: Vec<Row> = issues
            .iter()
            .enumerate()
            .map(|(row_idx, issue)| {
                let type_cell = Cell::from(Self::type_symbol(&issue.issue_type));

                // Apply highlighting if search query is present
                let id_cell = if let Some(ref query) = self.search_query {
                    Cell::from(Line::from(Self::highlight_text(&issue.id, query)))
                } else {
                    Cell::from(issue.id.clone())
                };

                // Check if this row is being edited
                let title_text = if let Some((edit_idx, edit_buffer, cursor_pos)) = editing_state {
                    if row_idx == edit_idx {
                        // Show edit buffer with cursor
                        let before_cursor = &edit_buffer[..cursor_pos];
                        let after_cursor = &edit_buffer[cursor_pos..];
                        format!("{}|{}", before_cursor, after_cursor)
                    } else {
                        issue.title.clone()
                    }
                } else {
                    issue.title.clone()
                };

                // Handle title cell with wrapping support
                let title_cell = if self.row_height > 1 {
                    // Multi-row mode: wrap text
                    let wrapped_lines = Self::wrap_text(&title_text, 30); // Use min constraint as width
                    let lines: Vec<Line> = wrapped_lines
                        .iter()
                        .take(self.row_height as usize)
                        .map(|line_text| {
                            if let Some(ref query) = self.search_query {
                                Line::from(Self::highlight_text(line_text, query))
                            } else {
                                Line::from(line_text.clone())
                            }
                        })
                        .collect();

                    // If wrapped text has fewer lines than row_height, pad with empty lines
                    let mut padded_lines = lines;
                    while padded_lines.len() < self.row_height as usize {
                        padded_lines.push(Line::from(""));
                    }

                    Cell::from(padded_lines)
                } else {
                    // Single-row mode: truncate
                    let truncated = Self::truncate_text(&title_text, 30);
                    if let Some(ref query) = self.search_query {
                        Cell::from(Line::from(Self::highlight_text(&truncated, query)))
                    } else {
                        Cell::from(truncated)
                    }
                };

                let status_cell = Cell::from(Span::styled(
                    format!("{:?}", issue.status),
                    Style::default().fg(Self::status_color(&issue.status)),
                ));
                let priority_cell = Cell::from(Span::styled(
                    format!("{:?}", issue.priority),
                    Style::default().fg(Self::priority_color(&issue.priority)),
                ));

                Row::new(vec![
                    type_cell,
                    id_cell,
                    title_cell,
                    status_cell,
                    priority_cell,
                ])
                .height(self.row_height)
            })
            .collect();

        // Build table
        let widths = [
            Constraint::Length(6),  // Type
            Constraint::Length(15), // ID
            Constraint::Min(30),    // Title
            Constraint::Length(12), // Status
            Constraint::Length(10), // Priority
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Issues"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(table, table_area, buf, &mut state.table_state);

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
}
