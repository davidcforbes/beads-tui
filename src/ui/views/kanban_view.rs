//! Kanban board view for visual issue management

use crate::beads::models::{Issue, IssueStatus, Priority};
use crate::models::kanban_config::{ColumnDefinition, ColumnId, GroupingMode, KanbanConfig};
use crate::ui::widgets::kanban_card::{render_kanban_card, CardMode, KanbanCardConfig};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::HashMap;

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
    /// Scroll offset for each column (column_index -> scroll_offset)
    column_scrolls: HashMap<usize, usize>,
    /// Card display mode
    card_mode: CardMode,
}

impl KanbanViewState {
    /// Create a new Kanban view state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            config: KanbanConfig::default(),
            issues,
            selected_column: 0,
            selected_card: 0,
            column_scrolls: HashMap::new(),
            card_mode: CardMode::TwoLine,
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
            CardMode::TwoLine => CardMode::SingleLine,
        };
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
            self.filter_issues_for_column(column)
        } else {
            Vec::new()
        }
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

/// Kanban board widget
pub struct KanbanView;

impl Widget for KanbanView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render a placeholder for now
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
            Line::from("Press Tab/Shift+Tab to navigate columns"),
            Line::from("Press j/k to navigate cards"),
            Line::from("Press Enter to view issue details"),
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
                .borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);

            let text = Line::from("No columns configured. Press 'c' to configure columns.");
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
            paragraph.render(inner, buf);
            return;
        }

        // Calculate column widths
        let column_count = visible_columns.len();
        let constraints: Vec<Constraint> = visible_columns
            .iter()
            .map(|col| Constraint::Length(col.width))
            .collect();

        let columns_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render each column
        for (col_idx, (column, column_area)) in visible_columns
            .iter()
            .zip(columns_layout.iter())
            .enumerate()
        {
            Self::render_column(
                *column_area,
                buf,
                column,
                state,
                col_idx,
                col_idx == state.selected_column,
            );
        }
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
        let border_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = format!(" {} ", column.label);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

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
        let card_config = KanbanCardConfig::default()
            .mode(state.card_mode)
            .max_width(inner.width.saturating_sub(2));

        let mut y = inner.y;
        for (card_idx, issue) in column_issues.iter().enumerate() {
            if y >= inner.y + inner.height {
                break; // Out of space
            }

            let is_card_selected = is_selected && card_idx == state.selected_card;
            let card_lines = render_kanban_card(issue, &card_config, is_card_selected);

            for line in card_lines {
                if y >= inner.y + inner.height {
                    break;
                }
                buf.set_line(inner.x + 1, y, &line, inner.width.saturating_sub(2));
                y += 1;
            }

            // Add spacing between cards
            y += 1;
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

        assert_eq!(state.card_mode, CardMode::TwoLine);
        state.toggle_card_mode();
        assert_eq!(state.card_mode, CardMode::SingleLine);
        state.toggle_card_mode();
        assert_eq!(state.card_mode, CardMode::TwoLine);
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
}
