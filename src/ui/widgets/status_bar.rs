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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_new() {
        let status_bar = StatusBar::new();
        assert!(status_bar.left.is_empty());
        assert!(status_bar.center.is_empty());
        assert!(status_bar.right.is_empty());
    }

    #[test]
    fn test_status_bar_default() {
        let status_bar = StatusBar::default();
        assert!(status_bar.left.is_empty());
        assert!(status_bar.center.is_empty());
        assert!(status_bar.right.is_empty());
    }

    #[test]
    fn test_status_bar_left() {
        let spans = vec![Span::raw("left")];
        let status_bar = StatusBar::new().left(spans.clone());
        assert_eq!(status_bar.left.len(), 1);
        assert_eq!(status_bar.left[0].content, "left");
    }

    #[test]
    fn test_status_bar_center() {
        let spans = vec![Span::raw("center")];
        let status_bar = StatusBar::new().center(spans.clone());
        assert_eq!(status_bar.center.len(), 1);
        assert_eq!(status_bar.center[0].content, "center");
    }

    #[test]
    fn test_status_bar_right() {
        let spans = vec![Span::raw("right")];
        let status_bar = StatusBar::new().right(spans.clone());
        assert_eq!(status_bar.right.len(), 1);
        assert_eq!(status_bar.right[0].content, "right");
    }

    #[test]
    fn test_status_bar_builder_chain() {
        let status_bar = StatusBar::new()
            .left(vec![Span::raw("left")])
            .center(vec![Span::raw("center")])
            .right(vec![Span::raw("right")]);

        assert_eq!(status_bar.left.len(), 1);
        assert_eq!(status_bar.center.len(), 1);
        assert_eq!(status_bar.right.len(), 1);
        assert_eq!(status_bar.left[0].content, "left");
        assert_eq!(status_bar.center[0].content, "center");
        assert_eq!(status_bar.right[0].content, "right");
    }

    #[test]
    fn test_status_bar_multiple_spans() {
        let spans = vec![Span::raw("first"), Span::raw("second"), Span::raw("third")];
        let status_bar = StatusBar::new().left(spans.clone());
        assert_eq!(status_bar.left.len(), 3);
        assert_eq!(status_bar.left[0].content, "first");
        assert_eq!(status_bar.left[1].content, "second");
        assert_eq!(status_bar.left[2].content, "third");
    }

    #[test]
    fn test_status_bar_empty_spans() {
        let status_bar = StatusBar::new().left(vec![]);
        assert!(status_bar.left.is_empty());
    }

    #[test]
    fn test_status_bar_styled_spans() {
        let span = Span::styled("styled", Style::default().fg(Color::Red));
        let status_bar = StatusBar::new().left(vec![span.clone()]);
        assert_eq!(status_bar.left.len(), 1);
        assert_eq!(status_bar.left[0].content, "styled");
        assert_eq!(status_bar.left[0].style.fg, Some(Color::Red));
    }
}
