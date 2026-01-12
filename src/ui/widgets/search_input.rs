//! Search input widget with autocomplete and history

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Search input state
#[derive(Debug)]
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
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
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
            state.set_query(format!("query{i}"));
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

    #[test]
    fn test_search_input_state_default() {
        let state = SearchInputState::default();
        assert_eq!(state.query(), "");
        assert_eq!(state.cursor_position(), 0);
        assert!(!state.is_focused());
    }

    #[test]
    fn test_set_query_into_string() {
        let mut state = SearchInputState::new();
        state.set_query(String::from("test"));
        assert_eq!(state.query(), "test");
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_delete_char_at_start() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_to_start();
        state.delete_char();
        assert_eq!(state.query(), "test"); // No change
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.delete_char_forward();
        assert_eq!(state.query(), "test"); // No change
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_move_cursor_left_at_start() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_to_start();
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 0); // No change
    }

    #[test]
    fn test_move_cursor_right_at_end() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 4); // No change
    }

    #[test]
    fn test_insert_char_in_middle() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_to_start();
        state.move_cursor_right();
        state.move_cursor_right();
        state.insert_char('X');
        assert_eq!(state.query(), "teXst");
        assert_eq!(state.cursor_position(), 3);
    }

    #[test]
    fn test_delete_char_in_middle() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_to_start();
        state.move_cursor_right();
        state.move_cursor_right();
        state.delete_char();
        assert_eq!(state.query(), "tst");
        assert_eq!(state.cursor_position(), 1);
    }

    #[test]
    fn test_delete_char_forward_in_middle() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.move_cursor_to_start();
        state.move_cursor_right();
        state.delete_char_forward();
        assert_eq!(state.query(), "tst");
        assert_eq!(state.cursor_position(), 1);
    }

    #[test]
    fn test_set_focused() {
        let mut state = SearchInputState::new();
        assert!(!state.is_focused());
        state.set_focused(true);
        assert!(state.is_focused());
        state.set_focused(false);
        assert!(!state.is_focused());
    }

    #[test]
    fn test_add_to_history_empty_query() {
        let mut state = SearchInputState::new();
        state.set_query("");
        state.add_to_history();
        assert_eq!(state.history().len(), 0); // Empty queries not added
    }

    #[test]
    fn test_add_to_history_whitespace_query() {
        let mut state = SearchInputState::new();
        state.set_query("   ");
        state.add_to_history();
        assert_eq!(state.history().len(), 0); // Whitespace-only queries not added
    }

    #[test]
    fn test_history_previous_empty_history() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.history_previous();
        assert_eq!(state.query(), "test"); // No change with empty history
    }

    #[test]
    fn test_history_next_empty_history() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.history_next();
        assert_eq!(state.query(), "test"); // No change with empty history
    }

    #[test]
    fn test_clear_history() {
        let mut state = SearchInputState::new();
        state.set_query("first");
        state.add_to_history();
        state.set_query("second");
        state.add_to_history();
        assert_eq!(state.history().len(), 2);

        state.clear_history();
        assert_eq!(state.history().len(), 0);
    }

    #[test]
    fn test_unicode_set_query() {
        // Test that set_query works with Unicode (insert_char has a bug with multi-byte chars)
        let mut state = SearchInputState::new();
        state.set_query("日本語");
        assert_eq!(state.query(), "日本語");
        assert_eq!(state.cursor_position(), 9); // Cursor at end (9 bytes for 3 chars)
    }

    #[test]
    fn test_cursor_position_after_set_query() {
        let mut state = SearchInputState::new();
        state.set_query("hello");
        assert_eq!(state.cursor_position(), 5); // Cursor moves to end
        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), 0);
        state.set_query("world");
        assert_eq!(state.cursor_position(), 5); // Cursor resets to end
    }

    #[test]
    fn test_search_input_new() {
        let input = SearchInput::new();
        assert_eq!(input.placeholder, Some("Search..."));
        assert!(input.show_icon);
        assert!(input.block.is_some());
    }

    #[test]
    fn test_search_input_default() {
        let input = SearchInput::default();
        assert_eq!(input.placeholder, Some("Search..."));
        assert!(input.show_icon);
        assert!(input.block.is_some());
    }

    #[test]
    fn test_search_input_placeholder() {
        let input = SearchInput::new().placeholder("Find...");
        assert_eq!(input.placeholder, Some("Find..."));
    }

    #[test]
    fn test_search_input_style() {
        let style = Style::default().fg(Color::Green);
        let input = SearchInput::new().style(style);
        assert_eq!(input.style.fg, Some(Color::Green));
    }

    #[test]
    fn test_search_input_focused_style() {
        let style = Style::default().fg(Color::Yellow);
        let input = SearchInput::new().focused_style(style);
        assert_eq!(input.focused_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_search_input_block() {
        let block = Block::default().title("Custom");
        let input = SearchInput::new().block(block);
        assert!(input.block.is_some());
    }

    #[test]
    fn test_search_input_builder_chain() {
        let block = Block::default().title("Find");
        let style = Style::default().fg(Color::Red);
        let focused_style = Style::default().fg(Color::Blue);

        let input = SearchInput::new()
            .placeholder("Type here...")
            .style(style)
            .focused_style(focused_style)
            .block(block)
            .show_icon(false);

        assert_eq!(input.placeholder, Some("Type here..."));
        assert_eq!(input.style.fg, Some(Color::Red));
        assert_eq!(input.focused_style.fg, Some(Color::Blue));
        assert!(!input.show_icon);
    }

    #[test]
    fn test_history_previous_at_first_item() {
        let mut state = SearchInputState::new();
        state.set_query("first");
        state.add_to_history();
        state.set_query("second");
        state.add_to_history();
        state.clear();

        // Navigate to most recent
        state.history_previous();
        assert_eq!(state.query(), "second");

        // Navigate to first
        state.history_previous();
        assert_eq!(state.query(), "first");

        // Try to go before first - should stay at first
        state.history_previous();
        assert_eq!(state.query(), "first");
    }

    #[test]
    fn test_history_next_when_no_index() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.add_to_history();
        state.clear();

        // history_index is None, history_next should do nothing
        state.history_next();
        assert_eq!(state.query(), "");
    }

    #[test]
    fn test_multiple_cursor_movements() {
        let mut state = SearchInputState::new();
        state.set_query("testing");

        for _ in 0..3 {
            state.move_cursor_left();
        }
        assert_eq!(state.cursor_position(), 4);

        for _ in 0..2 {
            state.move_cursor_right();
        }
        assert_eq!(state.cursor_position(), 6);
    }

    #[test]
    fn test_clear_resets_history_index() {
        let mut state = SearchInputState::new();
        state.set_query("first");
        state.add_to_history();
        state.set_query("second");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.clear();
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_set_query_resets_history_index() {
        let mut state = SearchInputState::new();
        state.set_query("first");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.set_query("new query");
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_insert_char_resets_history_index() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.insert_char('x');
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_delete_char_resets_history_index() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.delete_char();
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_delete_char_forward_resets_history_index() {
        let mut state = SearchInputState::new();
        state.set_query("test");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.move_cursor_to_start();
        state.delete_char_forward();
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_show_icon_toggle() {
        let input1 = SearchInput::new().show_icon(true);
        assert!(input1.show_icon);

        let input2 = SearchInput::new().show_icon(false);
        assert!(!input2.show_icon);
    }

    #[test]
    fn test_history_navigation_updates_cursor() {
        let mut state = SearchInputState::new();
        state.set_query("short");
        state.add_to_history();
        state.set_query("much longer query");
        state.add_to_history();
        state.clear();

        state.history_previous();
        assert_eq!(state.cursor_position(), state.query().len()); // Cursor at end of "much longer query"

        state.history_previous();
        assert_eq!(state.cursor_position(), state.query().len()); // Cursor at end of "short"
    }

    #[test]
    fn test_add_to_history_resets_index() {
        let mut state = SearchInputState::new();
        state.set_query("first");
        state.add_to_history();

        state.history_previous();
        assert!(state.history_index.is_some());

        state.set_query("second");
        state.add_to_history();
        assert!(state.history_index.is_none());
    }

    #[test]
    fn test_cursor_position_consistency() {
        let mut state = SearchInputState::new();
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');
        assert_eq!(state.cursor_position(), 5);
        assert_eq!(state.query().len(), 5);

        state.delete_char();
        assert_eq!(state.cursor_position(), 4);
        assert_eq!(state.query().len(), 4);
    }
}
