//! History Operations (Squash/Burn) for Molecular Chemistry UI

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// State for History Operations
#[derive(Debug)]
pub struct HistoryOpsState {
    pub list_state: ListState,
}

impl Default for HistoryOpsState {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryOpsState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}

/// History Operations Widget
pub struct HistoryOps<'a> {
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Default for HistoryOps<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> HistoryOps<'a> {
    pub fn new() -> Self {
        Self {
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> StatefulWidget for HistoryOps<'a> {
    type State = HistoryOpsState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Description
                Constraint::Min(0),    // Operation Log
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render Description
        let desc =
            Paragraph::new("Consolidate history (Squash) or permanently remove data (Burn).")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("History Management")
                        .style(self.block_style),
                );
        desc.render(chunks[0], buf);

        // Render Log Placeholder
        let items = vec![ListItem::new("No recent operations.")];
        let log = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Operation Log")
                .style(self.block_style),
        );
        Widget::render(log, chunks[1], buf);

        // Render Help
        let help = Paragraph::new("s: Squash Selected | b: Burn Selected | Esc: Back")
            .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[2], buf);
    }
}
