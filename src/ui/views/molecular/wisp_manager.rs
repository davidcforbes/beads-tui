//! Wisp Manager for ephemeral tasks

use crate::beads::models::Issue;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// State for the Wisp Manager
#[derive(Debug)]
pub struct WispManagerState {
    pub list_state: ListState,
}

impl Default for WispManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl WispManagerState {
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
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

/// Wisp Manager Widget
pub struct WispManager<'a> {
    wisps: Vec<&'a Issue>,
    block_style: Style,
}

impl<'a> Default for WispManager<'a> {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<'a> WispManager<'a> {
    pub fn new(wisps: Vec<&'a Issue>) -> Self {
        Self {
            wisps,
            block_style: Style::default().fg(Color::Cyan),
        }
    }
}

impl<'a> StatefulWidget for WispManager<'a> {
    type State = WispManagerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(0),    // List
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render Summary
        let summary = Paragraph::new(format!("Active Wisps: {}", self.wisps.len())).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Summary")
                .style(self.block_style),
        );
        summary.render(chunks[0], buf);

        // Render List
        let items: Vec<ListItem> = if self.wisps.is_empty() {
            vec![ListItem::new("No wisps found. Create one with 'n'.")]
        } else {
            self.wisps
                .iter()
                .map(|w| {
                    ListItem::new(Line::from(vec![
                        Span::styled(&w.id, Style::default().fg(Color::DarkGray)),
                        Span::raw(" "),
                        Span::styled(&w.title, Style::default().add_modifier(Modifier::BOLD)),
                    ]))
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Wisps")
                    .style(self.block_style),
            )
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .highlight_symbol("w> ");

        StatefulWidget::render(list, area, buf, &mut state.list_state);

        // Render Help
        let help = Paragraph::new("j/k: Navigate | n: New Wisp | d: Dissolve (Delete)")
            .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[2], buf);
    }
}
