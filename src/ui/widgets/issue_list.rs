//! Issue list widget with sorting and filtering

use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState},
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

/// Issue list state
#[derive(Debug)]
pub struct IssueListState {
    table_state: TableState,
    sort_column: SortColumn,
    sort_direction: SortDirection,
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
        }
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
}

/// Issue list widget
pub struct IssueList<'a> {
    issues: Vec<&'a Issue>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    show_details: bool,
    search_query: Option<String>,
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
}

impl<'a> StatefulWidget for IssueList<'a> {
    type State = IssueListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
                    format!("Type {}", sort_indicator)
                } else {
                    "Type".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Id {
                    format!("ID {}", sort_indicator)
                } else {
                    "ID".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Title {
                    format!("Title {}", sort_indicator)
                } else {
                    "Title".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Status {
                    format!("Status {}", sort_indicator)
                } else {
                    "Status".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                if state.sort_column == SortColumn::Priority {
                    format!("Priority {}", sort_indicator)
                } else {
                    "Priority".to_string()
                },
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        let header = Row::new(header_cells)
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        // Build rows
        let rows: Vec<Row> = issues
            .iter()
            .map(|issue| {
                let type_cell = Cell::from(Self::type_symbol(&issue.issue_type));
                
                // Apply highlighting if search query is present
                let id_cell = if let Some(ref query) = self.search_query {
                    Cell::from(Line::from(Self::highlight_text(&issue.id, query)))
                } else {
                    Cell::from(issue.id.clone())
                };

                let title_cell = if let Some(ref query) = self.search_query {
                    Cell::from(Line::from(Self::highlight_text(&issue.title, query)))
                } else {
                    Cell::from(issue.title.clone())
                };

                let status_cell = Cell::from(Span::styled(
                    format!("{:?}", issue.status),
                    Style::default().fg(Self::status_color(&issue.status)),
                ));
                let priority_cell = Cell::from(Span::styled(
                    format!("{:?}", issue.priority),
                    Style::default().fg(Self::priority_color(&issue.priority)),
                ));

                Row::new(vec![type_cell, id_cell, title_cell, status_cell, priority_cell])
                    .height(1)
            })
            .collect();

        // Build table
        let widths = [
            Constraint::Length(6),   // Type
            Constraint::Length(15),  // ID
            Constraint::Min(30),     // Title
            Constraint::Length(12),  // Status
            Constraint::Length(10),  // Priority
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Issues"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(table, area, buf, &mut state.table_state);
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

        IssueList::sort_issues(&mut sorted_issues, SortColumn::Priority, SortDirection::Ascending);
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
}
