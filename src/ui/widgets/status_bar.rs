//! Status bar widget with context, mode, and stats display
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct StatusBar<'a> {
    left: Vec<Span<'a>>,
    center: Vec<Span<'a>>,
    right: Vec<Span<'a>>,
}

impl<'a> StatusBar<'a> {
    pub fn new() -> Self {
        Self {
            left: vec![],
            center: vec![],
            right: vec![],
        }
    }

    pub fn left(mut self, spans: Vec<Span<'a>>) -> Self {
        self.left = spans;
        self
    }

    pub fn center(mut self, spans: Vec<Span<'a>>) -> Self {
        self.center = spans;
        self
    }

    pub fn right(mut self, spans: Vec<Span<'a>>) -> Self {
        self.right = spans;
        self
    }

    pub fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(area);

        // Left section - context
        let left_para = Paragraph::new(Line::from(self.left.clone()))
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        Widget::render(left_para, chunks[0], buf);

        // Center section - mode
        let center_para = Paragraph::new(Line::from(self.center.clone()))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        Widget::render(center_para, chunks[1], buf);

        // Right section - stats
        let right_para = Paragraph::new(Line::from(self.right.clone()))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::ALL));
        Widget::render(right_para, chunks[2], buf);
    }
}

impl<'a> Default for StatusBar<'a> {
    fn default() -> Self {
        Self::new()
    }
}
