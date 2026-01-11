//! Dialog widgets for confirmations and alerts

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

/// Dialog button
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogButton {
    pub label: String,
    pub action: String,
}

impl DialogButton {
    /// Create a new dialog button
    pub fn new<S: Into<String>>(label: S, action: S) -> Self {
        Self {
            label: label.into(),
            action: action.into(),
        }
    }
}

/// Dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Information dialog (blue)
    Info,
    /// Warning dialog (yellow)
    Warning,
    /// Error dialog (red)
    Error,
    /// Success dialog (green)
    Success,
    /// Confirmation dialog (default)
    Confirm,
}

impl DialogType {
    fn color(&self) -> Color {
        match self {
            DialogType::Info => Color::Blue,
            DialogType::Warning => Color::Yellow,
            DialogType::Error => Color::Red,
            DialogType::Success => Color::Green,
            DialogType::Confirm => Color::Cyan,
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            DialogType::Info => "ℹ",
            DialogType::Warning => "⚠",
            DialogType::Error => "✖",
            DialogType::Success => "✓",
            DialogType::Confirm => "?",
        }
    }
}

/// Dialog state for tracking selected button
#[derive(Debug, Clone)]
pub struct DialogState {
    selected_button: usize,
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogState {
    /// Create a new dialog state
    pub fn new() -> Self {
        Self { selected_button: 0 }
    }

    /// Get the selected button index
    pub fn selected_button(&self) -> usize {
        self.selected_button
    }

    /// Select the next button
    pub fn select_next(&mut self, button_count: usize) {
        if button_count > 0 {
            self.selected_button = (self.selected_button + 1) % button_count;
        }
    }

    /// Select the previous button
    pub fn select_previous(&mut self, button_count: usize) {
        if button_count > 0 {
            self.selected_button = if self.selected_button == 0 {
                button_count - 1
            } else {
                self.selected_button - 1
            };
        }
    }

    /// Reset to first button
    pub fn reset(&mut self) {
        self.selected_button = 0;
    }
}

/// Dialog widget for confirmations, alerts, and messages
pub struct Dialog<'a> {
    title: &'a str,
    message: &'a str,
    dialog_type: DialogType,
    buttons: Vec<DialogButton>,
    width: u16,
    height: u16,
}

impl<'a> Dialog<'a> {
    /// Create a new dialog
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Confirm,
            buttons: vec![DialogButton::new("OK", "ok")],
            width: 50,
            height: 10,
        }
    }

    /// Create a confirmation dialog with Yes/No buttons
    pub fn confirm(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Confirm,
            buttons: vec![
                DialogButton::new("Yes", "yes"),
                DialogButton::new("No", "no"),
            ],
            width: 50,
            height: 10,
        }
    }

    /// Create a save/cancel dialog
    pub fn save_cancel(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Confirm,
            buttons: vec![
                DialogButton::new("Save", "save"),
                DialogButton::new("Cancel", "cancel"),
            ],
            width: 50,
            height: 10,
        }
    }

    /// Create an error dialog
    pub fn error(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Error,
            buttons: vec![DialogButton::new("OK", "ok")],
            width: 50,
            height: 10,
        }
    }

    /// Create a warning dialog
    pub fn warning(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Warning,
            buttons: vec![DialogButton::new("OK", "ok")],
            width: 50,
            height: 10,
        }
    }

    /// Create an info dialog
    pub fn info(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Info,
            buttons: vec![DialogButton::new("OK", "ok")],
            width: 50,
            height: 10,
        }
    }

    /// Create a success dialog
    pub fn success(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            dialog_type: DialogType::Success,
            buttons: vec![DialogButton::new("OK", "ok")],
            width: 50,
            height: 10,
        }
    }

    /// Set the dialog type
    pub fn dialog_type(mut self, dialog_type: DialogType) -> Self {
        self.dialog_type = dialog_type;
        self
    }

    /// Set custom buttons
    pub fn buttons(mut self, buttons: Vec<DialogButton>) -> Self {
        self.buttons = buttons;
        self
    }

    /// Set dialog width
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set dialog height
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Render the dialog with state
    pub fn render_with_state(self, area: Rect, buf: &mut Buffer, state: &DialogState) {
        // Calculate centered dialog position
        let dialog_rect = Self::centered_rect(self.width, self.height, area);

        // Clear the dialog area
        Clear.render(dialog_rect, buf);

        // Create outer block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} {} ", self.dialog_type.symbol(), self.title))
            .border_style(Style::default().fg(self.dialog_type.color()));

        let inner = block.inner(dialog_rect);
        block.render(dialog_rect, buf);

        // Create inner layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message area
                Constraint::Length(3), // Buttons area
            ])
            .split(inner);

        // Render message
        let message = Paragraph::new(self.message)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center);
        message.render(chunks[0], buf);

        // Render buttons
        self.render_buttons(chunks[1], buf, state);
    }

    fn render_buttons(&self, area: Rect, buf: &mut Buffer, state: &DialogState) {
        if self.buttons.is_empty() {
            return;
        }

        // Calculate button layout
        let button_width = 12;
        let spacing = 2;
        let total_width =
            self.buttons.len() as u16 * button_width + (self.buttons.len() as u16 - 1) * spacing;
        let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

        for (i, button) in self.buttons.iter().enumerate() {
            let button_x = start_x + i as u16 * (button_width + spacing);
            let button_rect = Rect {
                x: button_x,
                y: area.y + 1,
                width: button_width,
                height: 1,
            };

            let is_selected = i == state.selected_button;

            let button_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(self.dialog_type.color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.dialog_type.color())
            };

            let button_text = if is_selected {
                format!("[ {} ]", button.label)
            } else {
                format!("  {}  ", button.label)
            };

            let button_line = Line::from(Span::styled(button_text, button_style));
            button_line.render(button_rect, buf);
        }
    }

    fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        Rect {
            x,
            y,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    }
}

impl<'a> Widget for Dialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = DialogState::new();
        self.render_with_state(area, buf, &state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_button_creation() {
        let button = DialogButton::new("OK", "ok");
        assert_eq!(button.label, "OK");
        assert_eq!(button.action, "ok");
    }

    #[test]
    fn test_dialog_state_creation() {
        let state = DialogState::new();
        assert_eq!(state.selected_button(), 0);
    }

    #[test]
    fn test_dialog_state_navigation() {
        let mut state = DialogState::new();

        state.select_next(3);
        assert_eq!(state.selected_button(), 1);

        state.select_next(3);
        assert_eq!(state.selected_button(), 2);

        state.select_next(3);
        assert_eq!(state.selected_button(), 0); // Wraps around

        state.select_previous(3);
        assert_eq!(state.selected_button(), 2);

        state.select_previous(3);
        assert_eq!(state.selected_button(), 1);
    }

    #[test]
    fn test_dialog_state_reset() {
        let mut state = DialogState::new();

        state.select_next(3);
        state.select_next(3);
        assert_eq!(state.selected_button(), 2);

        state.reset();
        assert_eq!(state.selected_button(), 0);
    }

    #[test]
    fn test_dialog_types() {
        assert_eq!(DialogType::Info.color(), Color::Blue);
        assert_eq!(DialogType::Warning.color(), Color::Yellow);
        assert_eq!(DialogType::Error.color(), Color::Red);
        assert_eq!(DialogType::Success.color(), Color::Green);
        assert_eq!(DialogType::Confirm.color(), Color::Cyan);

        assert_eq!(DialogType::Info.symbol(), "ℹ");
        assert_eq!(DialogType::Warning.symbol(), "⚠");
        assert_eq!(DialogType::Error.symbol(), "✖");
        assert_eq!(DialogType::Success.symbol(), "✓");
        assert_eq!(DialogType::Confirm.symbol(), "?");
    }

    #[test]
    fn test_dialog_creation() {
        let dialog = Dialog::new("Test", "This is a test");
        assert_eq!(dialog.title, "Test");
        assert_eq!(dialog.message, "This is a test");
        assert_eq!(dialog.buttons.len(), 1);
        assert_eq!(dialog.buttons[0].label, "OK");
    }

    #[test]
    fn test_confirm_dialog() {
        let dialog = Dialog::confirm("Confirm", "Are you sure?");
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.buttons[0].label, "Yes");
        assert_eq!(dialog.buttons[1].label, "No");
    }

    #[test]
    fn test_save_cancel_dialog() {
        let dialog = Dialog::save_cancel("Save Changes", "Save your changes?");
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.buttons[0].label, "Save");
        assert_eq!(dialog.buttons[1].label, "Cancel");
    }

    #[test]
    fn test_error_dialog() {
        let dialog = Dialog::error("Error", "An error occurred");
        assert_eq!(dialog.dialog_type, DialogType::Error);
        assert_eq!(dialog.buttons.len(), 1);
    }

    #[test]
    fn test_custom_buttons() {
        let dialog = Dialog::new("Test", "Message").buttons(vec![
            DialogButton::new("One", "1"),
            DialogButton::new("Two", "2"),
            DialogButton::new("Three", "3"),
        ]);
        assert_eq!(dialog.buttons.len(), 3);
    }
}
