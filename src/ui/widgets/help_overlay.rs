//! Help overlay widget for displaying context-sensitive help popups
//!
//! Provides a popup overlay that shows keyboard shortcuts and help information
//! on top of the current view, without taking over the full screen like HelpView.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

/// Help overlay position on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpOverlayPosition {
    /// Center of the screen
    Center,
    /// Top-right corner
    TopRight,
    /// Bottom-right corner
    BottomRight,
    /// Top-left corner
    TopLeft,
    /// Bottom-left corner
    BottomLeft,
}

/// Keyboard shortcut entry for the help overlay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    /// The key or key combination (e.g., "j/k", "Ctrl+C", "Enter")
    pub key: String,
    /// Description of what the key does
    pub description: String,
}

impl KeyBinding {
    /// Create a new key binding
    pub fn new<K: Into<String>, D: Into<String>>(key: K, description: D) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

/// Help overlay widget for contextual help popups
pub struct HelpOverlay<'a> {
    /// Title of the help overlay
    title: &'a str,
    /// Optional subtitle or context
    subtitle: Option<&'a str>,
    /// List of keyboard shortcuts to display
    key_bindings: Vec<KeyBinding>,
    /// Position on screen
    position: HelpOverlayPosition,
    /// Width percentage (10-100)
    width_percent: u16,
    /// Height percentage (10-100)
    height_percent: u16,
    /// Optional custom content lines (instead of key bindings)
    custom_content: Option<Vec<Line<'a>>>,
    /// Border style
    border_style: Style,
    /// Title style
    title_style: Style,
    /// Key style (for keyboard shortcuts)
    key_style: Style,
    /// Description style
    description_style: Style,
    /// Dismiss hint text
    dismiss_hint: Option<&'a str>,
}

impl<'a> HelpOverlay<'a> {
    /// Create a new help overlay with a title
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            key_bindings: Vec::new(),
            position: HelpOverlayPosition::Center,
            width_percent: 50,
            height_percent: 60,
            custom_content: None,
            border_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            title_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            key_style: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            description_style: Style::default().fg(Color::White),
            dismiss_hint: Some("Press ? or Esc to close"),
        }
    }

    /// Set the subtitle
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Add a single key binding
    pub fn key_binding<K: Into<String>, D: Into<String>>(mut self, key: K, description: D) -> Self {
        self.key_bindings.push(KeyBinding::new(key, description));
        self
    }

    /// Set all key bindings at once
    pub fn key_bindings(mut self, bindings: Vec<KeyBinding>) -> Self {
        self.key_bindings = bindings;
        self
    }

    /// Set custom content (overrides key bindings)
    pub fn custom_content(mut self, lines: Vec<Line<'a>>) -> Self {
        self.custom_content = Some(lines);
        self
    }

    /// Set the position
    pub fn position(mut self, position: HelpOverlayPosition) -> Self {
        self.position = position;
        self
    }

    /// Set width as percentage (10-100)
    pub fn width_percent(mut self, percent: u16) -> Self {
        self.width_percent = percent.clamp(10, 100);
        self
    }

    /// Set height as percentage (10-100)
    pub fn height_percent(mut self, percent: u16) -> Self {
        self.height_percent = percent.clamp(10, 100);
        self
    }

    /// Set the border style
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set the title style
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set the key style
    pub fn key_style(mut self, style: Style) -> Self {
        self.key_style = style;
        self
    }

    /// Set the description style
    pub fn description_style(mut self, style: Style) -> Self {
        self.description_style = style;
        self
    }

    /// Set the dismiss hint
    pub fn dismiss_hint(mut self, hint: &'a str) -> Self {
        self.dismiss_hint = Some(hint);
        self
    }

    /// Hide the dismiss hint
    pub fn hide_dismiss_hint(mut self) -> Self {
        self.dismiss_hint = None;
        self
    }

    /// Calculate the overlay area based on position and size
    fn calculate_area(&self, screen: Rect) -> Rect {
        let width = (screen.width as u32 * self.width_percent as u32 / 100) as u16;
        let height = (screen.height as u32 * self.height_percent as u32 / 100) as u16;

        // Ensure minimum size
        let width = width.max(30).min(screen.width);
        let height = height.max(10).min(screen.height);

        match self.position {
            HelpOverlayPosition::Center => {
                let x = (screen.width.saturating_sub(width)) / 2;
                let y = (screen.height.saturating_sub(height)) / 2;
                Rect::new(x, y, width, height)
            }
            HelpOverlayPosition::TopRight => {
                let x = screen.width.saturating_sub(width);
                Rect::new(x, 0, width, height)
            }
            HelpOverlayPosition::BottomRight => {
                let x = screen.width.saturating_sub(width);
                let y = screen.height.saturating_sub(height);
                Rect::new(x, y, width, height)
            }
            HelpOverlayPosition::TopLeft => Rect::new(0, 0, width, height),
            HelpOverlayPosition::BottomLeft => {
                let y = screen.height.saturating_sub(height);
                Rect::new(0, y, width, height)
            }
        }
    }

    /// Build the content lines from key bindings
    fn build_key_binding_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for binding in &self.key_bindings {
            let key_width = 15; // Fixed width for alignment
            let key_text = format!("{:<width$}", binding.key, width = key_width);

            lines.push(Line::from(vec![
                Span::styled(key_text, self.key_style),
                Span::styled(binding.description.clone(), self.description_style),
            ]));
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "No keyboard shortcuts defined",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }
}

impl<'a> Widget for HelpOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay_area = self.calculate_area(area);

        // Clear the overlay area first
        Clear.render(overlay_area, buf);

        // Build title text
        let mut title_text = self.title.to_string();
        if let Some(subtitle) = self.subtitle {
            title_text.push_str(&format!(" - {}", subtitle));
        }

        // Create layout: header + content + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),                                  // Title
                Constraint::Min(5),                                     // Content
                Constraint::Length(if self.dismiss_hint.is_some() { 2 } else { 0 }), // Footer
            ])
            .split(overlay_area);

        // Render title
        let title_paragraph = Paragraph::new(title_text)
            .style(self.title_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_style(self.border_style),
            );
        title_paragraph.render(chunks[0], buf);

        // Render content
        let content_area = chunks[1];
        let content_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(self.border_style);

        // Use custom content if provided, otherwise build from key bindings
        if let Some(custom_lines) = self.custom_content {
            let paragraph = Paragraph::new(custom_lines)
                .block(content_block)
                .wrap(Wrap { trim: true })
                .scroll((0, 0));
            paragraph.render(content_area, buf);
        } else {
            // Build key binding list
            let binding_lines = self.build_key_binding_lines();
            let items: Vec<ListItem> = binding_lines
                .into_iter()
                .map(ListItem::new)
                .collect();

            let list = List::new(items).block(content_block);
            list.render(content_area, buf);
        }

        // Render footer with dismiss hint
        if let Some(hint) = self.dismiss_hint {
            let footer = Paragraph::new(hint)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                        .border_style(self.border_style),
                );
            footer.render(chunks[2], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_creation() {
        let binding = KeyBinding::new("j/k", "Navigate");
        assert_eq!(binding.key, "j/k");
        assert_eq!(binding.description, "Navigate");
    }

    #[test]
    fn test_key_binding_clone() {
        let binding = KeyBinding::new("Esc", "Cancel");
        let cloned = binding.clone();
        assert_eq!(binding, cloned);
    }

    #[test]
    fn test_help_overlay_creation() {
        let overlay = HelpOverlay::new("Test Help");
        assert_eq!(overlay.title, "Test Help");
        assert!(overlay.key_bindings.is_empty());
        assert_eq!(overlay.position, HelpOverlayPosition::Center);
    }

    #[test]
    fn test_help_overlay_subtitle() {
        let overlay = HelpOverlay::new("Help").subtitle("Context");
        assert_eq!(overlay.subtitle, Some("Context"));
    }

    #[test]
    fn test_help_overlay_key_binding() {
        let overlay = HelpOverlay::new("Help")
            .key_binding("j", "Down")
            .key_binding("k", "Up");
        assert_eq!(overlay.key_bindings.len(), 2);
        assert_eq!(overlay.key_bindings[0].key, "j");
        assert_eq!(overlay.key_bindings[1].key, "k");
    }

    #[test]
    fn test_help_overlay_key_bindings_bulk() {
        let bindings = vec![
            KeyBinding::new("a", "Action A"),
            KeyBinding::new("b", "Action B"),
        ];
        let overlay = HelpOverlay::new("Help").key_bindings(bindings.clone());
        assert_eq!(overlay.key_bindings, bindings);
    }

    #[test]
    fn test_help_overlay_position() {
        let overlay = HelpOverlay::new("Help").position(HelpOverlayPosition::TopRight);
        assert_eq!(overlay.position, HelpOverlayPosition::TopRight);
    }

    #[test]
    fn test_help_overlay_width_percent() {
        let overlay = HelpOverlay::new("Help").width_percent(70);
        assert_eq!(overlay.width_percent, 70);
    }

    #[test]
    fn test_help_overlay_width_percent_clamping() {
        let overlay_low = HelpOverlay::new("Help").width_percent(5);
        assert_eq!(overlay_low.width_percent, 10); // Clamped to minimum

        let overlay_high = HelpOverlay::new("Help").width_percent(150);
        assert_eq!(overlay_high.width_percent, 100); // Clamped to maximum
    }

    #[test]
    fn test_help_overlay_height_percent() {
        let overlay = HelpOverlay::new("Help").height_percent(80);
        assert_eq!(overlay.height_percent, 80);
    }

    #[test]
    fn test_help_overlay_height_percent_clamping() {
        let overlay_low = HelpOverlay::new("Help").height_percent(5);
        assert_eq!(overlay_low.height_percent, 10); // Clamped to minimum

        let overlay_high = HelpOverlay::new("Help").height_percent(200);
        assert_eq!(overlay_high.height_percent, 100); // Clamped to maximum
    }

    #[test]
    fn test_help_overlay_dismiss_hint() {
        let overlay = HelpOverlay::new("Help").dismiss_hint("Press Q");
        assert_eq!(overlay.dismiss_hint, Some("Press Q"));
    }

    #[test]
    fn test_help_overlay_hide_dismiss_hint() {
        let overlay = HelpOverlay::new("Help").hide_dismiss_hint();
        assert_eq!(overlay.dismiss_hint, None);
    }

    #[test]
    fn test_help_overlay_border_style() {
        let style = Style::default().fg(Color::Red);
        let overlay = HelpOverlay::new("Help").border_style(style);
        assert_eq!(overlay.border_style, style);
    }

    #[test]
    fn test_help_overlay_title_style() {
        let style = Style::default().fg(Color::Yellow);
        let overlay = HelpOverlay::new("Help").title_style(style);
        assert_eq!(overlay.title_style, style);
    }

    #[test]
    fn test_help_overlay_key_style() {
        let style = Style::default().fg(Color::Magenta);
        let overlay = HelpOverlay::new("Help").key_style(style);
        assert_eq!(overlay.key_style, style);
    }

    #[test]
    fn test_help_overlay_description_style() {
        let style = Style::default().fg(Color::Gray);
        let overlay = HelpOverlay::new("Help").description_style(style);
        assert_eq!(overlay.description_style, style);
    }

    #[test]
    fn test_help_overlay_custom_content() {
        let lines = vec![
            Line::from("Line 1"),
            Line::from("Line 2"),
        ];
        let overlay = HelpOverlay::new("Help").custom_content(lines.clone());
        assert_eq!(overlay.custom_content, Some(lines));
    }

    #[test]
    fn test_help_overlay_calculate_area_center() {
        let overlay = HelpOverlay::new("Help")
            .width_percent(50)
            .height_percent(50)
            .position(HelpOverlayPosition::Center);

        let screen = Rect::new(0, 0, 100, 50);
        let area = overlay.calculate_area(screen);

        assert_eq!(area.width, 50); // 50% of 100
        assert_eq!(area.height, 25); // 50% of 50
        assert_eq!(area.x, 25); // Centered: (100 - 50) / 2
        assert_eq!(area.y, 12); // Centered: (50 - 25) / 2
    }

    #[test]
    fn test_help_overlay_calculate_area_top_right() {
        let overlay = HelpOverlay::new("Help")
            .width_percent(40)
            .height_percent(30)
            .position(HelpOverlayPosition::TopRight);

        let screen = Rect::new(0, 0, 100, 50);
        let area = overlay.calculate_area(screen);

        assert_eq!(area.width, 40); // 40% of 100
        assert_eq!(area.height, 15); // 30% of 50
        assert_eq!(area.x, 60); // Right aligned: 100 - 40
        assert_eq!(area.y, 0); // Top
    }

    #[test]
    fn test_help_overlay_calculate_area_bottom_left() {
        let overlay = HelpOverlay::new("Help")
            .width_percent(30)
            .height_percent(40)
            .position(HelpOverlayPosition::BottomLeft);

        let screen = Rect::new(0, 0, 100, 50);
        let area = overlay.calculate_area(screen);

        assert_eq!(area.width, 30); // 30% of 100
        assert_eq!(area.height, 20); // 40% of 50
        assert_eq!(area.x, 0); // Left
        assert_eq!(area.y, 30); // Bottom: 50 - 20
    }

    #[test]
    fn test_help_overlay_calculate_area_minimum_size() {
        let overlay = HelpOverlay::new("Help")
            .width_percent(10)
            .height_percent(10);

        let screen = Rect::new(0, 0, 100, 50);
        let area = overlay.calculate_area(screen);

        // Should respect minimum size (30x10)
        assert!(area.width >= 30);
        assert!(area.height >= 10);
    }

    #[test]
    fn test_help_overlay_calculate_area_maximum_size() {
        let overlay = HelpOverlay::new("Help")
            .width_percent(100)
            .height_percent(100);

        let screen = Rect::new(0, 0, 80, 24);
        let area = overlay.calculate_area(screen);

        // Should not exceed screen size
        assert!(area.width <= screen.width);
        assert!(area.height <= screen.height);
    }

    #[test]
    fn test_help_overlay_builder_pattern() {
        let overlay = HelpOverlay::new("Test")
            .subtitle("Sub")
            .key_binding("a", "Action")
            .position(HelpOverlayPosition::TopLeft)
            .width_percent(60)
            .height_percent(70)
            .dismiss_hint("Press X");

        assert_eq!(overlay.title, "Test");
        assert_eq!(overlay.subtitle, Some("Sub"));
        assert_eq!(overlay.key_bindings.len(), 1);
        assert_eq!(overlay.position, HelpOverlayPosition::TopLeft);
        assert_eq!(overlay.width_percent, 60);
        assert_eq!(overlay.height_percent, 70);
        assert_eq!(overlay.dismiss_hint, Some("Press X"));
    }

    #[test]
    fn test_help_overlay_position_variants() {
        assert_eq!(HelpOverlayPosition::Center, HelpOverlayPosition::Center);
        assert_ne!(HelpOverlayPosition::Center, HelpOverlayPosition::TopRight);
        assert_ne!(HelpOverlayPosition::TopLeft, HelpOverlayPosition::BottomLeft);
    }

    #[test]
    fn test_key_binding_equality() {
        let binding1 = KeyBinding::new("a", "Action");
        let binding2 = KeyBinding::new("a", "Action");
        let binding3 = KeyBinding::new("b", "Action");

        assert_eq!(binding1, binding2);
        assert_ne!(binding1, binding3);
    }

    #[test]
    fn test_help_overlay_render_empty_key_bindings() {
        let overlay = HelpOverlay::new("Empty Help");
        let lines = overlay.build_key_binding_lines();

        // Should have placeholder text when no bindings
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_help_overlay_render_with_key_bindings() {
        let overlay = HelpOverlay::new("Help")
            .key_binding("j", "Down")
            .key_binding("k", "Up");

        let lines = overlay.build_key_binding_lines();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_help_overlay_widget_rendering() {
        let overlay = HelpOverlay::new("Test Help")
            .key_binding("q", "Quit")
            .key_binding("?", "Help");

        let area = Rect::new(0, 0, 100, 50);
        let mut buffer = Buffer::empty(area);

        overlay.render(area, &mut buffer);

        // Should render without panic
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content");
    }

    #[test]
    fn test_help_overlay_widget_rendering_custom_content() {
        let lines = vec![
            Line::from("Custom line 1"),
            Line::from("Custom line 2"),
        ];

        let overlay = HelpOverlay::new("Custom Help")
            .custom_content(lines);

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        overlay.render(area, &mut buffer);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content);
    }

    #[test]
    fn test_help_overlay_all_positions() {
        let positions = vec![
            HelpOverlayPosition::Center,
            HelpOverlayPosition::TopRight,
            HelpOverlayPosition::BottomRight,
            HelpOverlayPosition::TopLeft,
            HelpOverlayPosition::BottomLeft,
        ];

        let screen = Rect::new(0, 0, 100, 50);

        for position in positions {
            let overlay = HelpOverlay::new("Test").position(position);
            let area = overlay.calculate_area(screen);

            // All positions should produce valid areas
            assert!(area.width > 0);
            assert!(area.height > 0);
            assert!(area.x < screen.width);
            assert!(area.y < screen.height);
        }
    }
}
