//! Filter save dialog widget

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Filter save dialog state
#[derive(Debug, Clone)]
pub struct FilterSaveDialogState {
    name: String,
    description: String,
    hotkey: Option<String>,
    focused_field: FilterSaveField,
    name_cursor: usize,
    description_cursor: usize,
}

/// Field in the filter save dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSaveField {
    Name,
    Description,
    Hotkey,
}

impl Default for FilterSaveDialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterSaveDialogState {
    /// Create a new filter save dialog state
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            hotkey: None,
            focused_field: FilterSaveField::Name,
            name_cursor: 0,
            description_cursor: 0,
        }
    }

    /// Get the filter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the filter name
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
        self.name_cursor = self.name.len();
    }

    /// Get the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Set the description
    pub fn set_description<S: Into<String>>(&mut self, description: S) {
        self.description = description.into();
        self.description_cursor = self.description.len();
    }

    /// Get the hotkey
    pub fn hotkey(&self) -> Option<&str> {
        self.hotkey.as_deref()
    }

    /// Set the hotkey
    pub fn set_hotkey<S: Into<String>>(&mut self, hotkey: Option<S>) {
        self.hotkey = hotkey.map(|s| s.into());
    }

    /// Get the currently focused field
    pub fn focused_field(&self) -> FilterSaveField {
        self.focused_field
    }

    /// Focus the next field
    pub fn focus_next(&mut self) {
        self.focused_field = match self.focused_field {
            FilterSaveField::Name => FilterSaveField::Description,
            FilterSaveField::Description => FilterSaveField::Hotkey,
            FilterSaveField::Hotkey => FilterSaveField::Name,
        };
    }

    /// Focus the previous field
    pub fn focus_previous(&mut self) {
        self.focused_field = match self.focused_field {
            FilterSaveField::Name => FilterSaveField::Hotkey,
            FilterSaveField::Description => FilterSaveField::Name,
            FilterSaveField::Hotkey => FilterSaveField::Description,
        };
    }

    /// Insert character at cursor in focused field
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            return;
        }

        match self.focused_field {
            FilterSaveField::Name => {
                self.name.insert(self.name_cursor, c);
                self.name_cursor += 1;
            }
            FilterSaveField::Description => {
                self.description.insert(self.description_cursor, c);
                self.description_cursor += 1;
            }
            FilterSaveField::Hotkey => {
                // Hotkey is selected from a list, not typed
            }
        }
    }

    /// Delete character before cursor in focused field
    pub fn delete_char(&mut self) {
        match self.focused_field {
            FilterSaveField::Name => {
                if self.name_cursor > 0 {
                    self.name.remove(self.name_cursor - 1);
                    self.name_cursor -= 1;
                }
            }
            FilterSaveField::Description => {
                if self.description_cursor > 0 {
                    self.description.remove(self.description_cursor - 1);
                    self.description_cursor -= 1;
                }
            }
            FilterSaveField::Hotkey => {
                self.hotkey = None;
            }
        }
    }

    /// Move cursor left in focused field
    pub fn move_cursor_left(&mut self) {
        match self.focused_field {
            FilterSaveField::Name => {
                if self.name_cursor > 0 {
                    self.name_cursor -= 1;
                }
            }
            FilterSaveField::Description => {
                if self.description_cursor > 0 {
                    self.description_cursor -= 1;
                }
            }
            FilterSaveField::Hotkey => {}
        }
    }

    /// Move cursor right in focused field
    pub fn move_cursor_right(&mut self) {
        match self.focused_field {
            FilterSaveField::Name => {
                if self.name_cursor < self.name.len() {
                    self.name_cursor += 1;
                }
            }
            FilterSaveField::Description => {
                if self.description_cursor < self.description.len() {
                    self.description_cursor += 1;
                }
            }
            FilterSaveField::Hotkey => {}
        }
    }

    /// Validate the dialog data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Filter name is required".to_string());
        }

        if self.name.len() > 50 {
            return Err("Filter name must be 50 characters or less".to_string());
        }

        if self.description.len() > 200 {
            return Err("Description must be 200 characters or less".to_string());
        }

        Ok(())
    }

    /// Clear the dialog
    pub fn clear(&mut self) {
        self.name.clear();
        self.description.clear();
        self.hotkey = None;
        self.focused_field = FilterSaveField::Name;
        self.name_cursor = 0;
        self.description_cursor = 0;
    }

    /// Check if the dialog has data
    pub fn has_data(&self) -> bool {
        !self.name.is_empty() || !self.description.is_empty() || self.hotkey.is_some()
    }
}

/// Filter save dialog widget
pub struct FilterSaveDialog<'a> {
    title: &'a str,
    show_hotkey: bool,
    width: u16,
    height: u16,
    style: Style,
    focused_style: Style,
}

impl<'a> FilterSaveDialog<'a> {
    /// Create a new filter save dialog
    pub fn new() -> Self {
        Self {
            title: "Save Filter",
            show_hotkey: true,
            width: 60,
            height: 14,
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set the dialog title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Show or hide hotkey field
    pub fn show_hotkey(mut self, show: bool) -> Self {
        self.show_hotkey = show;
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

    /// Set style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set focused style
    pub fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    /// Render the dialog with state
    pub fn render_with_state(self, area: Rect, buf: &mut Buffer, state: &FilterSaveDialogState) {
        // Calculate centered dialog position
        let dialog_rect = Self::centered_rect(self.width, self.height, area);

        // Clear the dialog area
        Clear.render(dialog_rect, buf);

        // Create outer block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_rect);
        block.render(dialog_rect, buf);

        // Create field layout
        let mut constraints = vec![
            Constraint::Length(3), // Name field
            Constraint::Length(3), // Description field
        ];

        if self.show_hotkey {
            constraints.push(Constraint::Length(3)); // Hotkey field
        }

        constraints.push(Constraint::Min(1)); // Spacer
        constraints.push(Constraint::Length(1)); // Help text

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Render name field
        self.render_field(
            chunks[0],
            buf,
            "Name",
            &state.name,
            state.name_cursor,
            state.focused_field == FilterSaveField::Name,
            "Enter a name for this filter",
        );

        // Render description field
        self.render_field(
            chunks[1],
            buf,
            "Description (optional)",
            &state.description,
            state.description_cursor,
            state.focused_field == FilterSaveField::Description,
            "Brief description",
        );

        // Render hotkey field
        let mut chunk_index = 2;
        if self.show_hotkey {
            let hotkey_text = state.hotkey.as_deref().unwrap_or("None");
            self.render_field(
                chunks[2],
                buf,
                "Hotkey (optional)",
                hotkey_text,
                0,
                state.focused_field == FilterSaveField::Hotkey,
                "Press 1-9 or F1-F12",
            );
            chunk_index = 3;
        }

        // Render help text
        let help_text = vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" Next  "),
            Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ];
        let help_line = Line::from(help_text);
        let help = Paragraph::new(help_line).alignment(Alignment::Center);
        help.render(chunks[chunk_index + 1], buf);
    }

    fn render_field(
        &self,
        area: Rect,
        buf: &mut Buffer,
        label: &str,
        value: &str,
        cursor_pos: usize,
        is_focused: bool,
        placeholder: &str,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(if is_focused {
                self.focused_style
            } else {
                self.style
            });

        let inner = block.inner(area);
        block.render(area, buf);

        // Render value or placeholder
        let text = if value.is_empty() {
            Line::from(Span::styled(
                placeholder,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))
        } else {
            Line::from(value)
        };

        let paragraph = Paragraph::new(text);
        paragraph.render(inner, buf);

        // Render cursor if focused
        if is_focused && !value.is_empty() && inner.width > 0 && inner.height > 0 {
            let cursor_x = inner.x + cursor_pos.min(value.len()) as u16;
            let cursor_y = inner.y;

            if cursor_x < inner.x + inner.width {
                buf.get_mut(cursor_x, cursor_y)
                    .set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
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

impl<'a> Default for FilterSaveDialog<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Widget for FilterSaveDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = FilterSaveDialogState::new();
        self.render_with_state(area, buf, &state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_save_dialog_state_creation() {
        let state = FilterSaveDialogState::new();
        assert_eq!(state.name(), "");
        assert_eq!(state.description(), "");
        assert_eq!(state.hotkey(), None);
        assert_eq!(state.focused_field(), FilterSaveField::Name);
    }

    #[test]
    fn test_filter_save_dialog_state_setters() {
        let mut state = FilterSaveDialogState::new();

        state.set_name("My Filter");
        assert_eq!(state.name(), "My Filter");

        state.set_description("A test filter");
        assert_eq!(state.description(), "A test filter");

        state.set_hotkey(Some("F1"));
        assert_eq!(state.hotkey(), Some("F1"));

        state.set_hotkey(None::<String>);
        assert_eq!(state.hotkey(), None);
    }

    #[test]
    fn test_filter_save_dialog_field_navigation() {
        let mut state = FilterSaveDialogState::new();

        assert_eq!(state.focused_field(), FilterSaveField::Name);

        state.focus_next();
        assert_eq!(state.focused_field(), FilterSaveField::Description);

        state.focus_next();
        assert_eq!(state.focused_field(), FilterSaveField::Hotkey);

        state.focus_next();
        assert_eq!(state.focused_field(), FilterSaveField::Name);

        state.focus_previous();
        assert_eq!(state.focused_field(), FilterSaveField::Hotkey);

        state.focus_previous();
        assert_eq!(state.focused_field(), FilterSaveField::Description);
    }

    #[test]
    fn test_filter_save_dialog_insert_char() {
        let mut state = FilterSaveDialogState::new();

        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');
        assert_eq!(state.name(), "test");

        state.focus_next();
        state.insert_char('d');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('c');
        assert_eq!(state.description(), "desc");
    }

    #[test]
    fn test_filter_save_dialog_delete_char() {
        let mut state = FilterSaveDialogState::new();

        state.set_name("hello");
        state.delete_char();
        assert_eq!(state.name(), "hell");

        state.focus_next();
        state.set_description("world");
        state.delete_char();
        assert_eq!(state.description(), "worl");
    }

    #[test]
    fn test_filter_save_dialog_cursor_movement() {
        let mut state = FilterSaveDialogState::new();

        state.set_name("hello");

        state.move_cursor_left();
        assert_eq!(state.name_cursor, 4);

        state.move_cursor_left();
        assert_eq!(state.name_cursor, 3);

        state.move_cursor_right();
        assert_eq!(state.name_cursor, 4);

        state.move_cursor_right();
        assert_eq!(state.name_cursor, 5);

        // Can't move past end
        state.move_cursor_right();
        assert_eq!(state.name_cursor, 5);
    }

    #[test]
    fn test_filter_save_dialog_validation() {
        let mut state = FilterSaveDialogState::new();

        // Empty name should fail
        assert!(state.validate().is_err());

        state.set_name("Valid Name");
        assert!(state.validate().is_ok());

        // Too long name should fail
        state.set_name("a".repeat(51));
        assert!(state.validate().is_err());

        state.set_name("Valid Name");
        assert!(state.validate().is_ok());

        // Too long description should fail
        state.set_description("a".repeat(201));
        assert!(state.validate().is_err());

        state.set_description("Valid description");
        assert!(state.validate().is_ok());
    }

    #[test]
    fn test_filter_save_dialog_clear() {
        let mut state = FilterSaveDialogState::new();

        state.set_name("Test");
        state.set_description("Description");
        state.set_hotkey(Some("F1"));

        assert!(state.has_data());

        state.clear();

        assert!(!state.has_data());
        assert_eq!(state.name(), "");
        assert_eq!(state.description(), "");
        assert_eq!(state.hotkey(), None);
        assert_eq!(state.focused_field(), FilterSaveField::Name);
    }

    #[test]
    fn test_filter_save_dialog_has_data() {
        let mut state = FilterSaveDialogState::new();

        assert!(!state.has_data());

        state.set_name("Test");
        assert!(state.has_data());

        state.clear();
        state.set_description("Desc");
        assert!(state.has_data());

        state.clear();
        state.set_hotkey(Some("F1"));
        assert!(state.has_data());
    }

    #[test]
    fn test_filter_save_dialog_ignore_newlines() {
        let mut state = FilterSaveDialogState::new();

        state.insert_char('h');
        state.insert_char('\n');
        state.insert_char('i');
        assert_eq!(state.name(), "hi");
    }
}
