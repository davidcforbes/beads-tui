//! Layout engine for beads-tui
//!
//! Provides a flexible layout system for organizing UI components

mod pane;
mod responsive;

pub use pane::*;
pub use responsive::*;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main application layout areas
#[derive(Debug, Clone)]
pub struct AppLayout {
    pub title_bar: Rect,
    pub tab_bar: Rect,
    pub content: Rect,
    pub status_bar: Rect,
}

impl AppLayout {
    /// Create a new application layout from the given area
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title bar
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Content area
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        Self {
            title_bar: chunks[0],
            tab_bar: chunks[1],
            content: chunks[2],
            status_bar: chunks[3],
        }
    }

    /// Create a layout with custom title and status bar heights
    pub fn with_custom_heights(
        area: Rect,
        title_height: u16,
        tab_height: u16,
        status_height: u16,
    ) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(title_height),
                Constraint::Length(tab_height),
                Constraint::Min(0),
                Constraint::Length(status_height),
            ])
            .split(area);

        Self {
            title_bar: chunks[0],
            tab_bar: chunks[1],
            content: chunks[2],
            status_bar: chunks[3],
        }
    }
}

/// Split a rectangle into horizontal sections with given constraints
pub fn split_horizontal(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

/// Split a rectangle into vertical sections with given constraints
pub fn split_vertical(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

/// Create a centered rectangle with given width and height percentages
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_layout() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = AppLayout::new(area);

        assert_eq!(layout.title_bar.height, 3);
        assert_eq!(layout.tab_bar.height, 3);
        assert_eq!(layout.status_bar.height, 3);
        assert!(layout.content.height > 0);
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 40);
        let centered = centered_rect(50, 50, area);

        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 20);
    }
}
