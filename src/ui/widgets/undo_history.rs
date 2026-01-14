//! Undo/redo history view widget
//!
//! Displays a timeline of all commands in the undo/redo history with timestamps
//! and highlights the current position.

use chrono::{DateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Widget},
};

/// Entry in the undo history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub is_current: bool,
    pub can_undo: bool,
}

/// Widget for displaying undo/redo history
pub struct UndoHistoryView<'a> {
    entries: Vec<HistoryEntry>,
    block: Option<Block<'a>>,
}

impl<'a> UndoHistoryView<'a> {
    /// Create a new undo history view
    pub fn new(entries: Vec<HistoryEntry>) -> Self {
        Self {
            entries,
            block: None,
        }
    }

    /// Set the block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Format a timestamp for display
    fn format_timestamp(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%H:%M:%S").to_string()
    }
}

impl<'a> Widget for UndoHistoryView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render block if provided
        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if self.entries.is_empty() {
            // Show empty state
            let empty_text = vec![ListItem::new(Line::from(Span::styled(
                "No history available",
                Style::default().fg(Color::DarkGray),
            )))];
            let list = List::new(empty_text);
            list.render(inner_area, buf);
            return;
        }

        // Build list items from history entries
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let time_str = Self::format_timestamp(entry.timestamp);
                let marker = if entry.is_current {
                    "→"
                } else if entry.can_undo {
                    "✓"
                } else {
                    "○"
                };

                let style = if entry.is_current {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if entry.can_undo {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{:3} ", idx + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{} ", marker), style),
                    Span::styled(
                        format!("[{}] ", time_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(entry.description.clone(), style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        list.render(inner_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ratatui::widgets::Borders;

    fn create_test_entry(desc: &str, is_current: bool, can_undo: bool) -> HistoryEntry {
        HistoryEntry {
            description: desc.to_string(),
            timestamp: Utc::now(),
            is_current,
            can_undo,
        }
    }

    #[test]
    fn test_history_view_empty() {
        let view = UndoHistoryView::new(vec![]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_history_view_single_entry() {
        let entries = vec![create_test_entry("Test command", true, true)];
        let view = UndoHistoryView::new(entries);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_history_view_multiple_entries() {
        let entries = vec![
            create_test_entry("Command 1", false, true),
            create_test_entry("Command 2", true, true),
            create_test_entry("Command 3", false, false),
        ];
        let view = UndoHistoryView::new(entries);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_history_view_with_block() {
        let entries = vec![create_test_entry("Test", true, true)];
        let block = Block::default().borders(Borders::ALL).title("History");
        let view = UndoHistoryView::new(entries).block(block);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = Utc::now();
        let formatted = UndoHistoryView::format_timestamp(timestamp);

        // Should be in HH:MM:SS format
        assert_eq!(formatted.len(), 8);
        assert_eq!(formatted.chars().nth(2), Some(':'));
        assert_eq!(formatted.chars().nth(5), Some(':'));
    }
}
