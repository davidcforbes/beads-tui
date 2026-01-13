//! Notification history panel widget for viewing past notifications

use crate::models::app_state::{NotificationMessage, NotificationType};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Notification history panel state
#[derive(Debug)]
pub struct NotificationHistoryState {
    list_state: ListState,
}

impl NotificationHistoryState {
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

impl Default for NotificationHistoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification history panel widget
pub struct NotificationHistoryPanel<'a> {
    notifications: &'a [NotificationMessage],
}

impl<'a> NotificationHistoryPanel<'a> {
    pub fn new(notifications: &'a [NotificationMessage]) -> Self {
        Self { notifications }
    }

    /// Format elapsed time since notification was created
    fn format_elapsed(instant: std::time::Instant) -> String {
        let elapsed = instant.elapsed();
        let total_seconds = elapsed.as_secs();

        if total_seconds < 60 {
            format!("{}s ago", total_seconds)
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            format!("{}m ago", minutes)
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            format!("{}h ago", hours)
        } else {
            let days = total_seconds / 86400;
            format!("{}d ago", days)
        }
    }

    /// Get notification type indicator and color
    fn get_type_indicator(notification_type: &NotificationType) -> (&'static str, Color) {
        match notification_type {
            NotificationType::Info => ("ℹ", Color::Cyan),
            NotificationType::Success => ("✓", Color::Green),
            NotificationType::Warning => ("⚠", Color::Yellow),
            NotificationType::Error => ("✗", Color::Red),
        }
    }
}

impl<'a> StatefulWidget for NotificationHistoryPanel<'a> {
    type State = NotificationHistoryState;

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
        let block = Block::default()
            .title(" Notification History ")
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

        // Render notification list (most recent first)
        let items: Vec<ListItem> = self
            .notifications
            .iter()
            .rev() // Most recent first
            .map(|notification| {
                let (indicator, color) = Self::get_type_indicator(&notification.notification_type);
                let elapsed = Self::format_elapsed(notification.created_at);

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", indicator),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&notification.message),
                    Span::styled(
                        format!(" ({})", elapsed),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("» ");

        StatefulWidget::render(list, chunks[0], buf, &mut state.list_state);

        // Render help text
        let help_text = if self.notifications.is_empty() {
            Line::from(vec![
                Span::styled("No notifications yet", Style::default().fg(Color::DarkGray)),
                Span::raw("  •  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" to close"),
            ])
        } else {
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate  •  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Close"),
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

    #[test]
    fn test_notification_history_state_new() {
        let state = NotificationHistoryState::new();
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_notification_history_state_select_next() {
        let mut state = NotificationHistoryState::new();

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
    fn test_notification_history_state_select_previous() {
        let mut state = NotificationHistoryState::new();
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
    fn test_notification_history_state_empty_list() {
        let mut state = NotificationHistoryState::new();
        state.select_next(0);
        // Should not crash with empty list
    }

    #[test]
    fn test_format_elapsed_seconds() {
        let now = std::time::Instant::now();
        let formatted = NotificationHistoryPanel::<'_>::format_elapsed(now);
        assert!(formatted.ends_with("s ago"));
    }

    #[test]
    fn test_get_type_indicator() {
        let (indicator, color) = NotificationHistoryPanel::<'_>::get_type_indicator(&NotificationType::Info);
        assert_eq!(indicator, "ℹ");
        assert_eq!(color, Color::Cyan);

        let (indicator, color) = NotificationHistoryPanel::<'_>::get_type_indicator(&NotificationType::Success);
        assert_eq!(indicator, "✓");
        assert_eq!(color, Color::Green);

        let (indicator, color) = NotificationHistoryPanel::<'_>::get_type_indicator(&NotificationType::Warning);
        assert_eq!(indicator, "⚠");
        assert_eq!(color, Color::Yellow);

        let (indicator, color) = NotificationHistoryPanel::<'_>::get_type_indicator(&NotificationType::Error);
        assert_eq!(indicator, "✗");
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_notification_history_panel_new() {
        let notifications = vec![];
        let panel = NotificationHistoryPanel::new(&notifications);
        assert_eq!(panel.notifications.len(), 0);
    }

    #[test]
    fn test_notification_history_state_default() {
        let state = NotificationHistoryState::default();
        assert_eq!(state.selected(), Some(0));
    }
}
