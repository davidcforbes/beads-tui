//! Toast notification widget for displaying temporary messages

use crate::models::{NotificationMessage, NotificationType};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Position where toast notifications appear
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    /// Top of the screen
    Top,
    /// Bottom of the screen
    Bottom,
}

/// Toast notification widget configuration
#[derive(Clone)]
pub struct ToastConfig {
    /// Position on screen
    pub position: ToastPosition,
    /// Maximum width of toast (percentage of screen width)
    pub max_width_percent: u16,
    /// Show dismiss hint
    pub show_dismiss_hint: bool,
    /// Custom dismiss hint text
    pub dismiss_hint: String,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            position: ToastPosition::Top,
            max_width_percent: 80,
            show_dismiss_hint: true,
            dismiss_hint: "Press Esc to dismiss".to_string(),
        }
    }
}

/// Toast notification widget for displaying temporary messages
pub struct Toast<'a> {
    notification: &'a NotificationMessage,
    config: ToastConfig,
    theme: Option<&'a crate::ui::themes::Theme>,
}

impl<'a> Toast<'a> {
    /// Create a new toast notification widget
    pub fn new(notification: &'a NotificationMessage) -> Self {
        Self {
            notification,
            config: ToastConfig::default(),
            theme: None,
        }
    }

    /// Set the position of the toast
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.config.position = position;
        self
    }

    /// Set the maximum width percentage
    pub fn max_width_percent(mut self, percent: u16) -> Self {
        self.config.max_width_percent = percent.min(100);
        self
    }

    /// Set whether to show the dismiss hint
    pub fn show_dismiss_hint(mut self, show: bool) -> Self {
        self.config.show_dismiss_hint = show;
        self
    }

    /// Set custom dismiss hint text
    pub fn dismiss_hint<S: Into<String>>(mut self, hint: S) -> Self {
        self.config.dismiss_hint = hint.into();
        self
    }

    /// Set the configuration
    pub fn config(mut self, config: ToastConfig) -> Self {
        self.config = config;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: &'a crate::ui::themes::Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Get the notification type styling
    fn get_style(&self) -> (Color, Color, &'static str, &'static str) {
        use crate::ui::themes::Theme;

        let default_theme = Theme::default();
        let theme_ref = self.theme.unwrap_or(&default_theme);

        match self.notification.notification_type {
            NotificationType::Error => (theme_ref.error, Color::White, "✖", "Error"),
            NotificationType::Success => (theme_ref.success, Color::White, "✓", "Success"),
            NotificationType::Info => (theme_ref.info, Color::White, "ℹ", "Info"),
            NotificationType::Warning => (theme_ref.warning, Color::Black, "⚠", "Warning"),
        }
    }

    /// Calculate the area for the toast notification
    fn calculate_area(&self, area: Rect) -> Rect {
        let (width_percent, height) = (self.config.max_width_percent, 3u16);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(match self.config.position {
                ToastPosition::Top => [Constraint::Length(height), Constraint::Min(0)],
                ToastPosition::Bottom => [Constraint::Min(0), Constraint::Length(height)],
            })
            .split(area);

        let toast_vertical_area = match self.config.position {
            ToastPosition::Top => vertical_chunks[0],
            ToastPosition::Bottom => vertical_chunks[1],
        };

        // Center horizontally
        let width = (area.width * width_percent / 100).max(20).min(area.width);
        let margin = (area.width.saturating_sub(width)) / 2;

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(margin),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(toast_vertical_area)[1]
    }
}

impl<'a> Widget for Toast<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let toast_area = self.calculate_area(area);

        // Clear the area first
        Clear.render(toast_area, buf);

        let (bg_color, fg_color, icon, type_label) = self.get_style();

        // Build the notification text
        let mut spans = vec![
            Span::styled(
                icon,
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                type_label,
                Style::default()
                    .fg(fg_color)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::raw(": "),
            Span::styled(&self.notification.message, Style::default().fg(fg_color)),
        ];

        // Add dismiss hint if enabled
        if self.config.show_dismiss_hint {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("({})", self.config.dismiss_hint),
                Style::default().fg(fg_color).add_modifier(Modifier::DIM),
            ));
        }

        let notification_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(bg_color).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(bg_color).fg(fg_color));

        let notification_text = Paragraph::new(Line::from(spans))
            .block(notification_block)
            .alignment(Alignment::Center);

        notification_text.render(toast_area, buf);
    }
}

/// Multi-toast widget for displaying multiple notifications
pub struct ToastStack<'a> {
    notifications: &'a [NotificationMessage],
    config: ToastConfig,
}

impl<'a> ToastStack<'a> {
    /// Create a new toast stack
    pub fn new(notifications: &'a [NotificationMessage]) -> Self {
        Self {
            notifications,
            config: ToastConfig::default(),
        }
    }

    /// Set the position of the toasts
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.config.position = position;
        self
    }

    /// Set the maximum width percentage
    pub fn max_width_percent(mut self, percent: u16) -> Self {
        self.config.max_width_percent = percent.min(100);
        self
    }

    /// Set whether to show the dismiss hint
    pub fn show_dismiss_hint(mut self, show: bool) -> Self {
        self.config.show_dismiss_hint = show;
        self
    }

    /// Set custom dismiss hint text
    pub fn dismiss_hint<S: Into<String>>(mut self, hint: S) -> Self {
        self.config.dismiss_hint = hint.into();
        self
    }

    /// Set the configuration
    pub fn config(mut self, config: ToastConfig) -> Self {
        self.config = config;
        self
    }
}

impl<'a> Widget for ToastStack<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.notifications.is_empty() {
            return;
        }

        // Render up to 5 most recent notifications, stacked vertically
        const MAX_VISIBLE_NOTIFICATIONS: usize = 5;
        const NOTIFICATION_HEIGHT: u16 = 3; // Height of each notification
        const NOTIFICATION_SPACING: u16 = 1; // Gap between notifications

        let visible_count = self.notifications.len().min(MAX_VISIBLE_NOTIFICATIONS);

        // Start from the oldest visible notification
        let start_idx = self.notifications.len().saturating_sub(visible_count);

        for (i, notification) in self.notifications[start_idx..].iter().enumerate() {
            // Calculate vertical offset for this notification
            let y_offset = match self.config.position {
                ToastPosition::Top => {
                    // Stack downward from the top
                    i as u16 * (NOTIFICATION_HEIGHT + NOTIFICATION_SPACING)
                }
                ToastPosition::Bottom => {
                    // Stack upward from the bottom
                    let total_stack_height =
                        visible_count as u16 * (NOTIFICATION_HEIGHT + NOTIFICATION_SPACING);
                    area.height.saturating_sub(total_stack_height)
                        + (i as u16 * (NOTIFICATION_HEIGHT + NOTIFICATION_SPACING))
                }
            };

            // Create a custom area for this notification
            let notification_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: area.height.saturating_sub(y_offset),
            };

            // Only render if there's enough space
            if notification_area.height >= NOTIFICATION_HEIGHT {
                let toast = Toast::new(notification).config(self.config.clone());
                toast.render(notification_area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_notification(
        message: &str,
        notification_type: NotificationType,
    ) -> NotificationMessage {
        NotificationMessage {
            message: message.to_string(),
            notification_type,
            created_at: std::time::Instant::now(),
        }
    }

    #[test]
    fn test_toast_config_default() {
        let config = ToastConfig::default();
        assert_eq!(config.position, ToastPosition::Top);
        assert_eq!(config.max_width_percent, 80);
        assert!(config.show_dismiss_hint);
        assert_eq!(config.dismiss_hint, "Press Esc to dismiss");
    }

    #[test]
    fn test_toast_position_variants() {
        assert_eq!(ToastPosition::Top, ToastPosition::Top);
        assert_eq!(ToastPosition::Bottom, ToastPosition::Bottom);
        assert_ne!(ToastPosition::Top, ToastPosition::Bottom);
    }

    #[test]
    fn test_toast_builder_pattern() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification)
            .position(ToastPosition::Bottom)
            .max_width_percent(60)
            .show_dismiss_hint(false);

        assert_eq!(toast.config.position, ToastPosition::Bottom);
        assert_eq!(toast.config.max_width_percent, 60);
        assert!(!toast.config.show_dismiss_hint);
    }

    #[test]
    fn test_toast_max_width_clamping() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification).max_width_percent(150);

        // Should clamp to 100
        assert_eq!(toast.config.max_width_percent, 100);
    }

    #[test]
    fn test_toast_custom_dismiss_hint() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification).dismiss_hint("Press Q to close");

        assert_eq!(toast.config.dismiss_hint, "Press Q to close");
    }

    #[test]
    fn test_toast_get_style() {
        let error = create_test_notification("Error", NotificationType::Error);
        let toast = Toast::new(&error);
        let (bg_color, fg_color, icon, label) = toast.get_style();
        assert_eq!(bg_color, Color::Red);
        assert_eq!(fg_color, Color::White);
        assert_eq!(icon, "✖");
        assert_eq!(label, "Error");

        let success = create_test_notification("Success", NotificationType::Success);
        let toast = Toast::new(&success);
        let (bg_color, fg_color, icon, label) = toast.get_style();
        assert_eq!(bg_color, Color::Green);
        assert_eq!(fg_color, Color::White);
        assert_eq!(icon, "✓");
        assert_eq!(label, "Success");

        let info = create_test_notification("Info", NotificationType::Info);
        let toast = Toast::new(&info);
        let (bg_color, fg_color, icon, label) = toast.get_style();
        assert_eq!(bg_color, Color::Blue);
        assert_eq!(fg_color, Color::White);
        assert_eq!(icon, "ℹ");
        assert_eq!(label, "Info");

        let warning = create_test_notification("Warning", NotificationType::Warning);
        let toast = Toast::new(&warning);
        let (bg_color, fg_color, icon, label) = toast.get_style();
        assert_eq!(bg_color, Color::Yellow);
        assert_eq!(fg_color, Color::Black);
        assert_eq!(icon, "⚠");
        assert_eq!(label, "Warning");
    }

    #[test]
    fn test_toast_calculate_area_top() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification).position(ToastPosition::Top);

        let area = Rect::new(0, 0, 100, 50);
        let toast_area = toast.calculate_area(area);

        assert_eq!(toast_area.y, 0); // Should be at top
        assert_eq!(toast_area.height, 3); // Fixed height
        assert!(toast_area.width <= area.width * 80 / 100); // Within max width
    }

    #[test]
    fn test_toast_calculate_area_bottom() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification).position(ToastPosition::Bottom);

        let area = Rect::new(0, 0, 100, 50);
        let toast_area = toast.calculate_area(area);

        assert_eq!(toast_area.y, 47); // Should be near bottom (50 - 3)
        assert_eq!(toast_area.height, 3); // Fixed height
    }

    #[test]
    fn test_toast_calculate_area_centered() {
        let notification = create_test_notification("Test", NotificationType::Info);
        let toast = Toast::new(&notification).max_width_percent(50);

        let area = Rect::new(0, 0, 100, 50);
        let toast_area = toast.calculate_area(area);

        // Should be centered horizontally
        let expected_width = 50; // 50% of 100
        let expected_margin = (100 - expected_width) / 2;
        assert_eq!(toast_area.x, expected_margin);
        assert_eq!(toast_area.width, expected_width);
    }

    #[test]
    fn test_toast_stack_empty() {
        let notifications: Vec<NotificationMessage> = vec![];
        let stack = ToastStack::new(&notifications);

        // Should not panic with empty notifications
        let area = Rect::new(0, 0, 100, 50);
        let mut buf = Buffer::empty(area);
        stack.render(area, &mut buf);
    }

    #[test]
    fn test_toast_stack_single() {
        let notifications = vec![create_test_notification("Test", NotificationType::Info)];
        let stack = ToastStack::new(&notifications);

        assert_eq!(stack.notifications.len(), 1);
    }

    #[test]
    fn test_toast_stack_multiple() {
        let notifications = vec![
            create_test_notification("First", NotificationType::Info),
            create_test_notification("Second", NotificationType::Success),
            create_test_notification("Third", NotificationType::Error),
        ];
        let stack = ToastStack::new(&notifications);

        assert_eq!(stack.notifications.len(), 3);
        // Currently renders only the last one
        assert_eq!(stack.notifications.last().unwrap().message, "Third");
    }

    #[test]
    fn test_toast_stack_builder_pattern() {
        let notifications = vec![create_test_notification("Test", NotificationType::Info)];
        let stack = ToastStack::new(&notifications)
            .position(ToastPosition::Bottom)
            .max_width_percent(70)
            .show_dismiss_hint(false);

        assert_eq!(stack.config.position, ToastPosition::Bottom);
        assert_eq!(stack.config.max_width_percent, 70);
        assert!(!stack.config.show_dismiss_hint);
    }
}
