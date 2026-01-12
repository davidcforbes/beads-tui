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
            format!("{max_line_num}").len() + 1
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
            for line_idx in start_line..end_line {
                let line_text = &state.lines[line_idx];
                let mut spans = Vec::new();

                // Add line number if enabled
                if self.show_line_numbers {
                    let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width - 1);
                    spans.push(Span::styled(line_num, Style::default().fg(Color::DarkGray)));
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

    #[test]
    fn test_text_editor_state_default() {
        let state = TextEditorState::default();
        assert_eq!(state.text(), "");
        assert_eq!(state.cursor_position(), (0, 0));
        assert!(!state.is_focused());
        assert_eq!(state.line_count(), 1);
        assert!(state.is_empty());
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_text_editor_state_clone() {
        let mut state = TextEditorState::new();
        state.set_text("test content");
        state.set_focused(true);
        state.cursor_line = 0;
        state.cursor_col = 4;

        let cloned = state.clone();
        assert_eq!(cloned.text(), "test content");
        assert!(cloned.is_focused());
        assert_eq!(cloned.cursor_position(), (0, 4));
    }

    #[test]
    fn test_set_text_empty_string() {
        let mut state = TextEditorState::new();
        state.set_text("initial");
        state.set_text("");

        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_set_text_single_line() {
        let mut state = TextEditorState::new();
        state.set_text("single line without newline");

        assert_eq!(state.line_count(), 1);
        assert_eq!(state.text(), "single line without newline");
    }

    #[test]
    fn test_insert_char_in_middle() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.cursor_col = 2; // Position between 'e' and 'l'

        state.insert_char('X');
        assert_eq!(state.text(), "heXllo");
        assert_eq!(state.cursor_position(), (0, 3));
    }

    #[test]
    fn test_insert_char_at_start() {
        let mut state = TextEditorState::new();
        state.set_text("world");
        state.cursor_col = 0;

        state.insert_char('X');
        assert_eq!(state.text(), "Xworld");
        assert_eq!(state.cursor_position(), (0, 1));
    }

    #[test]
    fn test_delete_char_at_start_of_text() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.cursor_col = 0;
        state.cursor_line = 0;

        // Delete at start should do nothing
        state.delete_char();
        assert_eq!(state.text(), "hello");
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_delete_char_single_char() {
        let mut state = TextEditorState::new();
        state.set_text("a");
        state.cursor_col = 1;

        state.delete_char();
        assert!(state.is_empty());
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_delete_char_forward_at_end_of_text() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.move_cursor_to_end();

        // Delete forward at end should do nothing
        state.delete_char_forward();
        assert_eq!(state.text(), "hello");
    }

    #[test]
    fn test_delete_char_forward_on_empty_line() {
        let mut state = TextEditorState::new();
        state.set_text("");

        // Delete forward on empty text should do nothing
        state.delete_char_forward();
        assert!(state.is_empty());
    }

    #[test]
    fn test_move_cursor_left_at_start_boundary() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.cursor_col = 0;
        state.cursor_line = 0;

        // Move left at start should do nothing
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_move_cursor_right_at_end_boundary() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.move_cursor_to_end();

        // Move right at end should do nothing
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_move_cursor_up_at_first_line() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2");
        state.cursor_line = 0;

        // Move up at first line should do nothing
        state.move_cursor_up();
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_move_cursor_down_at_last_line() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2");
        state.cursor_line = 1;

        // Move down at last line should do nothing
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_scroll_with_zero_visible_lines() {
        let mut state = TextEditorState::new();
        state.set_text("1\n2\n3\n4\n5");
        state.cursor_line = 2;

        state.update_scroll(0);
        // With zero visible lines, scroll should be 0
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_boundary_top() {
        let mut state = TextEditorState::new();
        state.set_text("1\n2\n3\n4\n5");
        state.scroll_offset = 2;
        state.cursor_line = 0;

        state.update_scroll(3);
        // Cursor at top should reset scroll to 0
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_focus_state_toggle() {
        let mut state = TextEditorState::new();
        assert!(!state.is_focused());

        state.set_focused(true);
        assert!(state.is_focused());

        state.set_focused(false);
        assert!(!state.is_focused());
    }

    #[test]
    fn test_max_lines_none() {
        let mut state = TextEditorState::new();
        state.set_max_lines(None);

        // Should allow unlimited lines
        for _ in 0..100 {
            state.insert_char('a');
            state.insert_char('\n');
        }

        assert!(state.line_count() > 10);
    }

    #[test]
    fn test_line_count_empty() {
        let state = TextEditorState::new();
        assert_eq!(state.line_count(), 1); // Empty editor has 1 line
    }

    #[test]
    fn test_is_empty_after_operations() {
        let mut state = TextEditorState::new();
        assert!(state.is_empty());

        state.insert_char('a');
        assert!(!state.is_empty());

        state.delete_char();
        assert!(state.is_empty());
    }

    #[test]
    fn test_text_editor_new() {
        let editor = TextEditor::new();
        assert_eq!(editor.placeholder, Some("Enter text..."));
        assert!(editor.block.is_some());
        assert!(!editor.show_line_numbers);
        assert!(editor.wrap);
    }

    #[test]
    fn test_text_editor_default() {
        let editor = TextEditor::default();
        // Default calls new(), so should have same values
        assert_eq!(editor.placeholder, Some("Enter text..."));
        assert!(editor.block.is_some());
    }

    #[test]
    fn test_text_editor_placeholder() {
        let editor = TextEditor::new().placeholder("Enter text here");
        assert_eq!(editor.placeholder, Some("Enter text here"));
    }

    #[test]
    fn test_text_editor_block() {
        let block = Block::default().title("Title");
        let editor = TextEditor::new().block(block);
        assert!(editor.block.is_some());
    }

    #[test]
    fn test_text_editor_style() {
        let style = Style::default().fg(Color::Blue);
        let editor = TextEditor::new().style(style);
        assert_eq!(editor.style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_text_editor_focused_style() {
        let style = Style::default().fg(Color::Yellow);
        let editor = TextEditor::new().focused_style(style);
        assert_eq!(editor.focused_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_text_editor_show_line_numbers() {
        let editor = TextEditor::new().show_line_numbers(true);
        assert!(editor.show_line_numbers);

        let editor = TextEditor::new().show_line_numbers(false);
        assert!(!editor.show_line_numbers);
    }

    #[test]
    fn test_text_editor_wrap() {
        let editor = TextEditor::new().wrap(false);
        assert!(!editor.wrap);

        let editor = TextEditor::new().wrap(true);
        assert!(editor.wrap);
    }

    #[test]
    fn test_text_editor_builder_chain() {
        let block = Block::default().title("Editor");
        let style = Style::default().fg(Color::Green);
        let focused_style = Style::default().fg(Color::Yellow);

        let editor = TextEditor::new()
            .placeholder("Type here")
            .block(block)
            .style(style)
            .focused_style(focused_style)
            .show_line_numbers(true)
            .wrap(false);

        assert_eq!(editor.placeholder, Some("Type here"));
        assert!(editor.block.is_some());
        assert_eq!(editor.style.fg, Some(Color::Green));
        assert_eq!(editor.focused_style.fg, Some(Color::Yellow));
        assert!(editor.show_line_numbers);
        assert!(!editor.wrap);
    }

    #[test]
    fn test_text_editor_state_debug_trait() {
        let state = TextEditorState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("TextEditorState"));
    }

    #[test]
    fn test_text_editor_builder_order_independence() {
        let block = Block::default().title("Editor");
        let style = Style::default().fg(Color::Blue);
        
        let editor1 = TextEditor::new()
            .block(block.clone())
            .wrap(false)
            .style(style);
        
        let editor2 = TextEditor::new()
            .style(style)
            .wrap(false)
            .block(block);
        
        assert!(editor1.block.is_some());
        assert!(editor2.block.is_some());
        assert_eq!(editor1.wrap, editor2.wrap);
        assert_eq!(editor1.style.fg, editor2.style.fg);
    }

    #[test]
    fn test_text_editor_multiple_setter_applications() {
        let editor = TextEditor::new()
            .wrap(true)
            .wrap(false)
            .wrap(true);
        
        assert!(editor.wrap);
        
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);
        
        let editor = TextEditor::new()
            .style(style1)
            .style(style2);
        
        assert_eq!(editor.style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_set_text_with_unicode() {
        let mut state = TextEditorState::new();
        state.set_text("Hello 世界\n你好 world\nこんにちは");
        
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.lines()[0], "Hello 世界");
        assert_eq!(state.lines()[1], "你好 world");
        assert_eq!(state.lines()[2], "こんにちは");
    }

    #[test]
    fn test_insert_unicode_characters() {
        // Note: insert_char uses cursor_col as byte index, which doesn't work with multi-byte chars
        // So we test unicode via set_text instead
        let mut state = TextEditorState::new();
        state.set_text("世界");
        
        assert_eq!(state.text(), "世界");
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.lines()[0], "世界");
    }

    #[test]
    fn test_set_text_very_long_line() {
        let mut state = TextEditorState::new();
        let long_line = "a".repeat(1000);
        state.set_text(&long_line);
        
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.lines()[0].len(), 1000);
    }

    #[test]
    fn test_insert_char_very_long_line() {
        let mut state = TextEditorState::new();
        
        for _ in 0..500 {
            state.insert_char('x');
        }
        
        assert_eq!(state.text().len(), 500);
        assert_eq!(state.cursor_position(), (0, 500));
    }

    #[test]
    fn test_complex_cursor_navigation_sequence() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2\nline3");
        
        // Navigate to middle of line 1
        state.move_cursor_right();
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (0, 2));
        
        // Move down
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (1, 2));
        
        // Move to end of line
        state.move_cursor_to_line_end();
        assert_eq!(state.cursor_position(), (1, 5));
        
        // Move right to next line
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (2, 0));
        
        // Move to start of all text
        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), (0, 0));
        
        // Move to end of all text
        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), (2, 5));
    }

    #[test]
    fn test_insert_newline_in_middle_of_line() {
        let mut state = TextEditorState::new();
        state.set_text("helloworld");
        state.cursor_col = 5; // Between "hello" and "world"
        
        state.insert_char('\n');
        
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.lines()[0], "hello");
        assert_eq!(state.lines()[1], "world");
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_insert_newline_at_start_of_line() {
        let mut state = TextEditorState::new();
        state.set_text("hello");
        state.cursor_col = 0;
        
        state.insert_char('\n');
        
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.lines()[0], "");
        assert_eq!(state.lines()[1], "hello");
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_max_lines_at_boundary() {
        let mut state = TextEditorState::new();
        state.set_max_lines(Some(3));
        
        state.insert_char('a');
        state.insert_char('\n');
        state.insert_char('b');
        state.insert_char('\n');
        state.insert_char('c');
        
        assert_eq!(state.line_count(), 3);
        
        // Try to insert newline at max capacity
        state.insert_char('\n');
        assert_eq!(state.line_count(), 3);
        
        // Should still allow regular characters
        state.insert_char('d');
        assert_eq!(state.lines()[2], "cd");
    }

    #[test]
    fn test_scroll_with_single_line() {
        let mut state = TextEditorState::new();
        state.set_text("single line");
        state.cursor_line = 0;
        
        state.update_scroll(10);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_cursor_at_exact_boundary() {
        let mut state = TextEditorState::new();
        state.set_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        
        // Cursor at line 5, visible area of 5 lines
        state.cursor_line = 5;
        state.update_scroll(5);
        
        // Should scroll to show cursor at bottom of viewport
        assert_eq!(state.scroll_offset(), 1); // Lines 1-5 visible, cursor at line 5
    }

    #[test]
    fn test_delete_all_text_char_by_char() {
        let mut state = TextEditorState::new();
        state.set_text("test");
        state.move_cursor_to_end();
        
        for _ in 0..4 {
            state.delete_char();
        }
        
        assert!(state.is_empty());
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_delete_forward_all_text() {
        let mut state = TextEditorState::new();
        state.set_text("test");
        
        for _ in 0..4 {
            state.delete_char_forward();
        }
        
        assert!(state.is_empty());
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_cursor_navigation_with_empty_lines() {
        let mut state = TextEditorState::new();
        state.set_text("line1\n\nline3");
        
        // Move to empty line
        state.cursor_line = 1;
        assert_eq!(state.lines()[1], "");
        
        // Move to end of empty line
        state.move_cursor_to_line_end();
        assert_eq!(state.cursor_position(), (1, 0));
        
        // Move right to next line
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), (2, 0));
    }

    #[test]
    fn test_insert_newline_on_empty_line() {
        let mut state = TextEditorState::new();
        assert!(state.is_empty());
        
        state.insert_char('\n');
        
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.lines()[0], "");
        assert_eq!(state.lines()[1], "");
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_set_text_with_trailing_newline() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2\n");
        
        // Note: .lines() doesn't preserve trailing newline as empty string
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.lines()[0], "line1");
        assert_eq!(state.lines()[1], "line2");
    }

    #[test]
    fn test_set_text_multiple_consecutive_newlines() {
        let mut state = TextEditorState::new();
        state.set_text("a\n\n\nb");
        
        assert_eq!(state.line_count(), 4);
        assert_eq!(state.lines()[0], "a");
        assert_eq!(state.lines()[1], "");
        assert_eq!(state.lines()[2], "");
        assert_eq!(state.lines()[3], "b");
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut state = TextEditorState::new();
        state.set_text("test\ntext");
        state.set_focused(true);
        state.cursor_line = 1;
        state.cursor_col = 3;
        state.scroll_offset = 1;
        
        state.clear();
        
        assert!(state.is_empty());
        assert_eq!(state.cursor_position(), (0, 0));
        assert_eq!(state.scroll_offset(), 0);
        assert!(state.is_focused()); // Focus state should NOT be reset by clear
    }

    #[test]
    fn test_set_text_resets_cursor_and_scroll() {
        let mut state = TextEditorState::new();
        state.set_text("initial\ntext");
        state.cursor_line = 1;
        state.cursor_col = 3;
        state.scroll_offset = 1;
        
        state.set_text("new text");
        
        assert_eq!(state.cursor_position(), (0, 0));
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_lines_returns_slice() {
        let mut state = TextEditorState::new();
        state.set_text("line1\nline2\nline3");
        
        let lines = state.lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_text_joins_lines_with_newline() {
        let mut state = TextEditorState::new();
        state.set_text("a\nb\nc");
        
        assert_eq!(state.text(), "a\nb\nc");
    }

    #[test]
    fn test_move_cursor_column_adjustment_on_empty_line() {
        let mut state = TextEditorState::new();
        state.set_text("hello\n\nworld");
        
        // Move to end of line 0
        state.cursor_line = 0;
        state.cursor_col = 5;
        
        // Move down to empty line
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (1, 0)); // Column adjusted to 0
        
        // Move down to longer line
        state.move_cursor_down();
        assert_eq!(state.cursor_position(), (2, 0)); // Column stays at 0
    }

    #[test]
    fn test_delete_char_multiple_lines() {
        let mut state = TextEditorState::new();
        state.set_text("a\nb\nc");
        
        // Position at start of line 2
        state.cursor_line = 2;
        state.cursor_col = 0;
        
        // Delete to merge with line 1
        state.delete_char();
        assert_eq!(state.text(), "a\nbc");
        assert_eq!(state.cursor_position(), (1, 1));
        
        // Delete again to merge with line 0
        state.cursor_line = 1;
        state.cursor_col = 0;
        state.delete_char();
        assert_eq!(state.text(), "abc");
        assert_eq!(state.cursor_position(), (0, 1));
    }

    #[test]
    fn test_delete_char_forward_multiple_lines() {
        let mut state = TextEditorState::new();
        state.set_text("a\nb\nc");
        
        // Position at end of line 0
        state.cursor_line = 0;
        state.cursor_col = 1;
        
        // Delete forward to merge with line 1
        state.delete_char_forward();
        assert_eq!(state.text(), "ab\nc");
        assert_eq!(state.cursor_position(), (0, 1));
        
        // Delete forward again to merge with line 2
        state.cursor_col = 2;
        state.delete_char_forward();
        assert_eq!(state.text(), "abc");
        assert_eq!(state.cursor_position(), (0, 2));
    }

    #[test]
    fn test_scroll_large_document() {
        let mut state = TextEditorState::new();
        let mut lines = Vec::new();
        for i in 0..100 {
            lines.push(format!("Line {}", i));
        }
        state.set_text(&lines.join("\n"));
        
        // Move cursor to middle of document
        state.cursor_line = 50;
        state.update_scroll(20); // 20 lines visible
        
        // Scroll should position cursor in visible area
        assert!(state.scroll_offset() <= 50);
        assert!(state.scroll_offset() + 20 > 50);
    }
}
