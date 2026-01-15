//! Issue history panel widget for viewing audit log and timeline

use crate::beads::models::Issue;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Issue history panel state
#[derive(Debug)]
pub struct IssueHistoryState {
    list_state: ListState,
}

impl IssueHistoryState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    len - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }
}

impl Default for IssueHistoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Issue history panel widget
pub struct IssueHistoryPanel<'a> {
    issue: Option<&'a Issue>,
}

impl<'a> IssueHistoryPanel<'a> {
    pub fn new(issue: Option<&'a Issue>) -> Self {
        Self { issue }
    }
}

impl<'a> StatefulWidget for IssueHistoryPanel<'a> {
    type State = IssueHistoryState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create a centered modal
        let modal_width = area.width.saturating_sub(10).min(80);
        let modal_height = area.height.saturating_sub(6).min(30);
        let modal_x = (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        // Clear the modal area with a background
        for y in modal_area.y..modal_area.y + modal_area.height {
            for x in modal_area.x..modal_area.x + modal_area.width {
                buf.get_mut(x, y).set_bg(Color::Black);
            }
        }

        // Create the block
        let title = if let Some(issue) = self.issue {
            format!(" Issue History - {} ", issue.id)
        } else {
            " Issue History ".to_string()
        };

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        // Split into list and help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(inner);

        // Build history timeline
        let items: Vec<ListItem> = if let Some(issue) = self.issue {
            let mut events = Vec::new();

            // Created event
            events.push(ListItem::new(vec![
                Line::from(vec![Span::styled(
                    issue.created.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(vec![
                    Span::styled(
                        "  [+] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Issue created"),
                ]),
                Line::from(""),
            ]));

            // Notes events (chronologically)
            for note in &issue.notes {
                events.push(ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        note.timestamp.format("%Y-%m-%d %H:%M").to_string(),
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![
                        Span::styled("  [Note] ", Style::default().fg(Color::Cyan)),
                        Span::styled(&note.author, Style::default().fg(Color::Yellow)),
                        Span::raw(" added note"),
                    ]),
                    Line::from(format!("     {}", note.content)),
                    Line::from(""),
                ]));
            }

            // Updated event (if different from created)
            if issue.updated != issue.created {
                events.push(ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        issue.updated.format("%Y-%m-%d %H:%M").to_string(),
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![
                        Span::styled("  ↻ ", Style::default().fg(Color::Yellow)),
                        Span::raw("Issue updated"),
                    ]),
                    Line::from(""),
                ]));
            }

            // Closed event
            if let Some(closed) = issue.closed {
                events.push(ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        closed.format("%Y-%m-%d %H:%M").to_string(),
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![
                        Span::styled(
                            "  ✗ ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Issue closed"),
                    ]),
                    Line::from(""),
                ]));
            }

            events
        } else {
            vec![ListItem::new(Line::from(Span::styled(
                "No issue selected",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        };

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("» ");

        StatefulWidget::render(list, chunks[0], buf, &mut state.list_state);

        // Render help text
        let help_text = if self.issue.is_some() && !self.issue.unwrap().notes.is_empty() {
            Line::from(vec![
                Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate | "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Close"),
            ])
        } else {
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" to close"),
            ])
        };

        let help = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        help.render(chunks[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Note, Priority};
    use chrono::Utc;

    #[test]
    fn test_issue_history_state_new() {
        let state = IssueHistoryState::new();
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_issue_history_state_select_next() {
        let mut state = IssueHistoryState::new();

        state.select_next(5);
        assert_eq!(state.selected(), Some(1));

        state.select_next(5);
        assert_eq!(state.selected(), Some(2));

        // At end, should stay at end
        state.list_state.select(Some(4));
        state.select_next(5);
        assert_eq!(state.selected(), Some(4));
    }

    #[test]
    fn test_issue_history_state_select_previous() {
        let mut state = IssueHistoryState::new();
        state.list_state.select(Some(3));

        state.select_previous();
        assert_eq!(state.selected(), Some(2));

        state.select_previous();
        assert_eq!(state.selected(), Some(1));

        state.select_previous();
        assert_eq!(state.selected(), Some(0));

        // At start, should stay at start
        state.select_previous();
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_issue_history_state_empty_list() {
        let mut state = IssueHistoryState::new();
        state.select_next(0);
        // Should not crash with empty list
    }

    #[test]
    fn test_issue_history_panel_new() {
        let panel = IssueHistoryPanel::new(None);
        assert!(panel.issue.is_none());
    }

    #[test]
    fn test_issue_history_state_default() {
        let state = IssueHistoryState::default();
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_issue_history_panel_with_issue() {
        let issue = Issue {
            id: "test-123".to_string(),
            title: "Test Issue".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: Some("Test description".to_string()),
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![Note {
                timestamp: Utc::now(),
                author: "test-user".to_string(),
                content: "Test note".to_string(),
            }],
        };

        let panel = IssueHistoryPanel::new(Some(&issue));
        assert!(panel.issue.is_some());
        assert_eq!(panel.issue.unwrap().id, "test-123");
    }
}
