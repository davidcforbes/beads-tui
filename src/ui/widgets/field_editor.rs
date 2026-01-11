//! Field editor widget for text input and editing

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Field editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Single-line text input
    SingleLine,
    /// Multi-line text area
    MultiLine,
}

/// Field editor state
pub struct FieldEditorState {
    content: String,
    cursor_position: usize,
    mode: EditorMode,
    is_focused: bool,
}

impl Default for FieldEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldEditorState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            mode: EditorMode::SingleLine,
            is_focused: false,
        }
    }

    pub fn with_content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = content.into();
        self.cursor_position = self.content.len();
        self
    }

    pub fn with_mode(mut self, mode: EditorMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get the current content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the content
    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        self.content = content.into();
        self.cursor_position = self.content.len().min(self.cursor_position);
    }

    /// Clear the content
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if self.mode == EditorMode::SingleLine && c == '\n' {
            return; // Ignore newlines in single-line mode
        }
        self.content.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.content.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_to_start(&mut self) {
        if self.mode == EditorMode::SingleLine {
            self.cursor_position = 0;
        } else {
            // Find start of current line
            let before_cursor = &self.content[..self.cursor_position];
            if let Some(last_newline) = before_cursor.rfind('\n') {
                self.cursor_position = last_newline + 1;
            } else {
                self.cursor_position = 0;
            }
        }
    }

    /// Move cursor to end of line
    pub fn move_cursor_to_end(&mut self) {
        if self.mode == EditorMode::SingleLine {
            self.cursor_position = self.content.len();
        } else {
            // Find end of current line
            let after_cursor = &self.content[self.cursor_position..];
            if let Some(next_newline) = after_cursor.find('\n') {
                self.cursor_position += next_newline;
            } else {
                self.cursor_position = self.content.len();
            }
        }
    }

    /// Get line and column of cursor (for multi-line mode)
    pub fn cursor_line_col(&self) -> (usize, usize) {
        let before_cursor = &self.content[..self.cursor_position];
        let line = before_cursor.matches('\n').count();
        let col = if let Some(last_newline_pos) = before_cursor.rfind('\n') {
            self.cursor_position - last_newline_pos - 1
        } else {
            self.cursor_position
        };
        (line, col)
    }
}

/// Field editor widget
pub struct FieldEditor<'a> {
    label: Option<&'a str>,
    placeholder: Option<&'a str>,
    style: Style,
    focused_style: Style,
    block: Option<Block<'a>>,
}

impl<'a> FieldEditor<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            placeholder: None,
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            block: Some(Block::default().borders(Borders::ALL)),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Default for FieldEditor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for FieldEditor<'a> {
    type State = FieldEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build title with label
        let title = if let Some(label) = self.label {
            if state.is_focused {
                format!("{} [editing]", label)
            } else {
                label.to_string()
            }
        } else if state.is_focused {
            "[editing]".to_string()
        } else {
            String::new()
        };

        // Build block
        let block = if let Some(mut block) = self.block {
            if !title.is_empty() {
                block = block.title(title);
            }
            let style = if state.is_focused {
                self.focused_style
            } else {
                self.style
            };
            block.style(style)
        } else {
            Block::default()
        };

        // Build content
        let text = if state.content.is_empty() {
            if let Some(placeholder) = self.placeholder {
                vec![Line::from(Span::styled(
                    placeholder,
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ))]
            } else {
                vec![Line::from("")]
            }
        } else {
            // Split content by lines
            state
                .content
                .lines()
                .map(|line| Line::from(line.to_string()))
                .collect()
        };

        // Create paragraph
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);

        // Render cursor if focused
        if state.is_focused && area.width > 2 && area.height > 2 {
            let (line, col) = state.cursor_line_col();
            let cursor_x = area.x + 1 + col as u16; // +1 for border
            let cursor_y = area.y + 1 + line as u16; // +1 for border

            // Only render cursor if it's within bounds
            if cursor_x < area.x + area.width - 1 && cursor_y < area.y + area.height - 1 {
                buf.get_mut(cursor_x, cursor_y)
                    .set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_editor_state_creation() {
        let state = FieldEditorState::new();
        assert_eq!(state.content(), "");
        assert_eq!(state.cursor_position(), 0);
        assert!(!state.is_focused());
    }

    #[test]
    fn test_field_editor_with_content() {
        let state = FieldEditorState::new().with_content("Hello");
        assert_eq!(state.content(), "Hello");
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_insert_char() {
        let mut state = FieldEditorState::new();
        state.insert_char('H');
        state.insert_char('i');
        assert_eq!(state.content(), "Hi");
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_delete_char() {
        let mut state = FieldEditorState::new().with_content("Hello");
        state.delete_char();
        assert_eq!(state.content(), "Hell");
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = FieldEditorState::new().with_content("Hello");

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 4);

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 3);

        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 4);

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), 0);

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_single_line_mode_ignores_newline() {
        let mut state = FieldEditorState::new();
        state.insert_char('H');
        state.insert_char('\n');
        state.insert_char('i');
        assert_eq!(state.content(), "Hi");
    }

    #[test]
    fn test_multi_line_mode_accepts_newline() {
        let mut state = FieldEditorState::new().with_mode(EditorMode::MultiLine);
        state.insert_char('H');
        state.insert_char('\n');
        state.insert_char('i');
        assert_eq!(state.content(), "H\ni");
    }

    #[test]
    fn test_cursor_line_col() {
        let mut state = FieldEditorState::new().with_mode(EditorMode::MultiLine);
        state.insert_char('H');
        state.insert_char('i');
        state.insert_char('\n');
        state.insert_char('B');
        state.insert_char('y');
        state.insert_char('e');

        assert_eq!(state.cursor_line_col(), (1, 3)); // Line 1, col 3 (0-indexed)
    }

    #[test]
    fn test_clear() {
        let mut state = FieldEditorState::new().with_content("Hello");
        state.clear();
        assert_eq!(state.content(), "");
        assert_eq!(state.cursor_position(), 0);
    }
}
