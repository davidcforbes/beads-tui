//! Multi-line text editor widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Text editor state
#[derive(Debug, Clone)]
pub struct TextEditorState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    is_focused: bool,
    max_lines: Option<usize>,
}

impl Default for TextEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditorState {
    /// Create a new text editor state
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            is_focused: false,
            max_lines: None,
        }
    }

    /// Get the text content as a single string
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set the text content
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        let text = text.into();
        if text.is_empty() {
            self.lines = vec![String::new()];
        } else {
            self.lines = text.lines().map(|l| l.to_string()).collect();
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// Get all lines
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Get cursor position (line, column)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_line, self.cursor_col)
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Set maximum number of lines
    pub fn set_max_lines(&mut self, max: Option<usize>) {
        self.max_lines = max;
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
        } else {
            let line = &mut self.lines[self.cursor_line];
            line.insert(self.cursor_col, c);
            self.cursor_col += 1;
        }
    }

    /// Insert newline at cursor
    fn insert_newline(&mut self) {
        // Check max lines limit
        if let Some(max) = self.max_lines {
            if self.lines.len() >= max {
                return;
            }
        }

        let current_line = &self.lines[self.cursor_line];
        let after = current_line[self.cursor_col..].to_string();
        let before = current_line[..self.cursor_col].to_string();

        self.lines[self.cursor_line] = before;
        self.lines.insert(self.cursor_line + 1, after);

        self.cursor_line += 1;
        self.cursor_col = 0;
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            // Delete character in current line
            self.lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        let line = &self.lines[self.cursor_line];
        if self.cursor_col < line.len() {
            // Delete character in current line
            self.lines[self.cursor_line].remove(self.cursor_col);
        } else if self.cursor_line < self.lines.len() - 1 {
            // Merge with next line
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next_line);
        }
    }

    /// Move cursor up
    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }

    /// Move cursor down
    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_line].len();
    }

    /// Move cursor to start of text
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
    }

    /// Move cursor to end of text
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_line = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_line].len();
    }

    /// Clear all text
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if editor is empty
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Update scroll to ensure cursor is visible
    pub(crate) fn update_scroll(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }

        // Scroll down if cursor is below visible area
        if self.cursor_line >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_line - visible_lines + 1;
        }

        // Scroll up if cursor is above visible area
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        }
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
}

/// Text editor widget
pub struct TextEditor<'a> {
    placeholder: Option<&'a str>,
    style: Style,
    focused_style: Style,
    block: Option<Block<'a>>,
    show_line_numbers: bool,
    wrap: bool,
}

impl<'a> TextEditor<'a> {
    /// Create a new text editor
    pub fn new() -> Self {
        Self {
            placeholder: Some("Enter text..."),
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            block: Some(Block::default().borders(Borders::ALL).title("Text")),
            show_line_numbers: false,
            wrap: true,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
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

    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Show or hide line numbers
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Enable or disable text wrapping
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
}

impl<'a> Default for TextEditor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for TextEditor<'a> {
    type State = TextEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build block
        let block = if let Some(mut block) = self.block {
            let title = if state.is_focused {
                format!("Text [{}L]", state.line_count())
            } else {
                "Text".to_string()
            };
            block = block.title(title);

            let style = if state.is_focused {
                self.focused_style
            } else {
                self.style
            };
            block.style(style)
        } else {
            Block::default()
        };

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Update scroll to ensure cursor is visible
        state.update_scroll(inner.height as usize);

        // Calculate line number width if needed
        let line_num_width = if self.show_line_numbers {
            let max_line_num = (state.scroll_offset + inner.height as usize).min(state.lines.len());
            format!("{}", max_line_num).len() + 1
        } else {
            0
        };

        // Build content lines
        let mut lines = Vec::new();
        let visible_lines = inner.height as usize;
        let start_line = state.scroll_offset;
        let end_line = (start_line + visible_lines).min(state.lines.len());

        if state.is_empty() && self.placeholder.is_some() {
            // Show placeholder
            lines.push(Line::from(Span::styled(
                self.placeholder.unwrap(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            for (_idx, line_idx) in (start_line..end_line).enumerate() {
                let line_text = &state.lines[line_idx];
                let mut spans = Vec::new();

                // Add line number if enabled
                if self.show_line_numbers {
                    let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width - 1);
                    spans.push(Span::styled(
                        line_num,
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                // Add line content
                spans.push(Span::raw(line_text.clone()));

                lines.push(Line::from(spans));
            }
        }

        // Render text
        let paragraph = if self.wrap {
            Paragraph::new(lines).wrap(Wrap { trim: false })
        } else {
            Paragraph::new(lines)
        };
        paragraph.render(inner, buf);

        // Render cursor if focused and not on placeholder
        if state.is_focused && !state.is_empty() && inner.width > 0 && inner.height > 0 {
            let cursor_screen_line = state.cursor_line.saturating_sub(state.scroll_offset);

            if cursor_screen_line < inner.height as usize {
                let cursor_x = inner.x + line_num_width as u16 + state.cursor_col as u16;
                let cursor_y = inner.y + cursor_screen_line as u16;

                if cursor_x < inner.x + inner.width {
                    buf.get_mut(cursor_x, cursor_y)
                        .set_style(Style::default().bg(Color::White).fg(Color::Black));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_state_creation() {
        let state = TextEditorState::new();
        assert_eq!(state.text(), "");
        assert_eq!(state.cursor_position(), (0, 0));
        assert!(!state.is_focused());
        assert_eq!(state.line_count(), 1);
        assert!(state.is_empty());
    }

    #[test]
    fn test_set_text() {
        let mut state = TextEditorState::new();
        state.set_text("Line 1\nLine 2\nLine 3");
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.lines()[0], "Line 1");
        assert_eq!(state.lines()[1], "Line 2");
        assert_eq!(state.lines()[2], "Line 3");
    }

    #[test]
    fn test_insert_char() {
        let mut state = TextEditorState::new();
        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.text(), "hi");
        assert_eq!(state.cursor_position(), (0, 2));
    }

    #[test]
    fn test_insert_newline() {
        let mut state = TextEditorState::new();
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');
        state.insert_char('\n');
        state.insert_char('w');
        state.insert_char('o');
        state.insert_char('r');
        state.insert_char('l');
        state.insert_char('d');

        assert_eq!(state.line_count(), 2);
        assert_eq!(state.lines()[0], "hello");
        assert_eq!(state.lines()[1], "world");
        assert_eq!(state.cursor_position(), (1, 5));
    }

    #[test]
    fn test_delete_char() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.move_cursor_to_end();

        state.delete_char();
        assert_eq!(state.text(), "hell");

        state.delete_char();
        assert_eq!(state.text(), "hel");
    }

    #[test]
    fn test_delete_char_merge_lines() {
        let mut state = TextEditorState::new();
        state.set_text("hello\nworld");

        // Move to start of line 2
        state.cursor_line = 1;
        state.cursor_col = 0;

        state.delete_char();
        assert_eq!(state.text(), "helloworld");
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_cursor_navigation() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2\nline3");

        // Start at (0, 0)
        assert_eq!(state.cursor_position(), (0, 0));

        // Move right
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (0, 1));

        // Move down
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (1, 1));

        // Move to end of line
        state.move_cursor_to_line_end();
        assert_eq!(state.cursor_position(), (1, 5));

        // Move right (should go to next line)
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (2, 0));

        // Move left (should go to previous line end)
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), (1, 5));

        // Move to start of line
        state.move_cursor_to_line_start();
        assert_eq!(state.cursor_position(), (1, 0));

        // Move up
        state.move_cursor_up();
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_move_cursor_to_start_end() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2\nline3");

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), (2, 5));

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_clear() {
        let mut state = TextEditorState::new();
        state.set_text("hello\nworld");
        state.clear();

        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_max_lines() {
        let mut state = TextEditorState::new();
        state.set_max_lines(Some(2));

        state.insert_char('a');
        state.insert_char('\n');
        state.insert_char('b');

        assert_eq!(state.line_count(), 2);

        // Try to insert another newline (should be ignored)
        state.insert_char('\n');
        assert_eq!(state.line_count(), 2);
    }

    #[test]
    fn test_scroll_update() {
        let mut state = TextEditorState::new();
        state.set_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

        // Visible area of 5 lines
        state.cursor_line = 7;
        state.update_scroll(5);

        // Cursor at line 7, should scroll to show it
        assert_eq!(state.scroll_offset, 3);

        // Move cursor up
        state.cursor_line = 2;
        state.update_scroll(5);

        // Should scroll up to show cursor
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn test_delete_forward() {
        let mut state = TextEditorState::new();
        state.set_text("hello");

        // Delete at position 0
        state.delete_char_forward();
        assert_eq!(state.text(), "ello");

        // Delete at position 0 again
        state.delete_char_forward();
        assert_eq!(state.text(), "llo");
    }

    #[test]
    fn test_delete_forward_merge_lines() {
        let mut state = TextEditorState::new();
        state.set_text("hello\nworld");

        // Move to end of line 1
        state.cursor_line = 0;
        state.cursor_col = 5;

        state.delete_char_forward();
        assert_eq!(state.text(), "helloworld");
        assert_eq!(state.line_count(), 1);
    }

    #[test]
    fn test_cursor_column_adjustment() {
        let mut state = TextEditorState::new();
        state.set_text("hello\nhi\nworld");

        // Move to end of line 0
        state.cursor_line = 0;
        state.cursor_col = 5;

        // Move down to shorter line
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (1, 2)); // Adjusted to line length

        // Move down to longer line
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (2, 2)); // Keeps column position
    }
}
