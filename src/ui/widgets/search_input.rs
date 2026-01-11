//! Search input widget with autocomplete and history

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Search input state
pub struct SearchInputState {
    query: String,
    cursor_position: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    max_history: usize,
    is_focused: bool,
}

impl Default for SearchInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchInputState {
    /// Create a new search input state
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
            max_history: 50,
            is_focused: false,
        }
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the query
    pub fn set_query<S: Into<String>>(&mut self, query: S) {
        self.query = query.into();
        self.cursor_position = self.query.len();
        self.history_index = None;
    }

    /// Clear the query
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_position = 0;
        self.history_index = None;
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
        if c == '\n' {
            return; // Ignore newlines
        }
        self.query.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.history_index = None;
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.query.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.history_index = None;
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.query.len() {
            self.query.remove(self.cursor_position);
            self.history_index = None;
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
        if self.cursor_position < self.query.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.query.len();
    }

    /// Add query to history
    pub fn add_to_history(&mut self) {
        if !self.query.trim().is_empty() {
            // Remove duplicate if exists
            self.history.retain(|q| q != &self.query);

            // Add to history
            self.history.push(self.query.clone());

            // Trim history to max size
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
        self.history_index = None;
    }

    /// Navigate to previous history item
    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // Start from the most recent
                self.history_index = Some(self.history.len() - 1);
                self.query = self.history[self.history.len() - 1].clone();
            }
            Some(idx) => {
                if idx > 0 {
                    self.history_index = Some(idx - 1);
                    self.query = self.history[idx - 1].clone();
                }
            }
        }
        self.cursor_position = self.query.len();
    }

    /// Navigate to next history item
    pub fn history_next(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // Do nothing
            }
            Some(idx) => {
                if idx < self.history.len() - 1 {
                    self.history_index = Some(idx + 1);
                    self.query = self.history[idx + 1].clone();
                } else {
                    // Clear query when going past the end
                    self.history_index = None;
                    self.query.clear();
                }
            }
        }
        self.cursor_position = self.query.len();
    }

    /// Get search history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Clear search history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = None;
    }
}

/// Search input widget
pub struct SearchInput<'a> {
    placeholder: Option<&'a str>,
    style: Style,
    focused_style: Style,
    block: Option<Block<'a>>,
    show_icon: bool,
}

impl<'a> SearchInput<'a> {
    /// Create a new search input
    pub fn new() -> Self {
        Self {
            placeholder: Some("Search..."),
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            block: Some(Block::default().borders(Borders::ALL).title("Search")),
            show_icon: true,
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

    /// Show or hide search icon
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }
}

impl<'a> Default for SearchInput<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for SearchInput<'a> {
    type State = SearchInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build block
        let block = if let Some(mut block) = self.block {
            let title = if state.is_focused {
                "Search [searching]"
            } else {
                "Search"
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

        // Build content
        let mut spans = Vec::new();

        if self.show_icon {
            spans.push(Span::styled("🔍 ", Style::default().fg(Color::DarkGray)));
        }

        if state.query.is_empty() {
            if let Some(placeholder) = self.placeholder {
                spans.push(Span::styled(
                    placeholder,
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ));
            }
        } else {
            spans.push(Span::raw(&state.query));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);

        // Render cursor if focused
        if state.is_focused && inner.width > 0 && inner.height > 0 {
            let icon_offset = if self.show_icon { 2 } else { 0 };
            let cursor_x = inner.x + icon_offset + state.cursor_position as u16;
            let cursor_y = inner.y;

            if cursor_x < inner.x + inner.width {
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
    fn test_search_input_state_creation() {
        let state = SearchInputState::new();
        assert_eq!(state.query(), "");
        assert_eq!(state.cursor_position(), 0);
        assert!(!state.is_focused());
    }

    #[test]
    fn test_insert_char() {
        let mut state = SearchInputState::new();
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');
        assert_eq!(state.query(), "test");
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_delete_char() {
        let mut state = SearchInputState::new();
        state.set_query("hello");
        state.delete_char();
        assert_eq!(state.query(), "hell");
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = SearchInputState::new();
        state.set_query("hello");

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 4);

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), 0);

        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 1);

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_clear() {
        let mut state = SearchInputState::new();
        state.set_query("hello");
        state.clear();
        assert_eq!(state.query(), "");
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_history() {
        let mut state = SearchInputState::new();

        state.set_query("first");
        state.add_to_history();

        state.set_query("second");
        state.add_to_history();

        state.set_query("third");
        state.add_to_history();

        assert_eq!(state.history().len(), 3);
        assert_eq!(state.history()[0], "first");
        assert_eq!(state.history()[1], "second");
        assert_eq!(state.history()[2], "third");
    }

    #[test]
    fn test_history_navigation() {
        let mut state = SearchInputState::new();

        state.set_query("first");
        state.add_to_history();

        state.set_query("second");
        state.add_to_history();

        state.clear();

        // Navigate to previous (most recent)
        state.history_previous();
        assert_eq!(state.query(), "second");

        // Navigate to previous again
        state.history_previous();
        assert_eq!(state.query(), "first");

        // Navigate forward
        state.history_next();
        assert_eq!(state.query(), "second");

        // Navigate past the end
        state.history_next();
        assert_eq!(state.query(), "");
    }

    #[test]
    fn test_history_deduplication() {
        let mut state = SearchInputState::new();

        state.set_query("test");
        state.add_to_history();

        state.set_query("other");
        state.add_to_history();

        state.set_query("test");
        state.add_to_history();

        // "test" should only appear once (the most recent)
        assert_eq!(state.history().len(), 2);
        assert_eq!(state.history()[0], "other");
        assert_eq!(state.history()[1], "test");
    }

    #[test]
    fn test_history_max_size() {
        let mut state = SearchInputState::new();
        state.max_history = 3;

        for i in 0..5 {
            state.set_query(format!("query{}", i));
            state.add_to_history();
        }

        assert_eq!(state.history().len(), 3);
        assert_eq!(state.history()[0], "query2");
        assert_eq!(state.history()[1], "query3");
        assert_eq!(state.history()[2], "query4");
    }

    #[test]
    fn test_ignore_newlines() {
        let mut state = SearchInputState::new();
        state.insert_char('h');
        state.insert_char('\n');
        state.insert_char('i');
        assert_eq!(state.query(), "hi");
    }
}
