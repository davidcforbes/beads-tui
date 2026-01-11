//! Autocomplete widget for text input with suggestions

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Autocomplete state
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    options: Vec<String>,
    input: String,
    cursor_position: usize,
    list_state: ListState,
    is_focused: bool,
    show_suggestions: bool,
    selected_value: Option<String>,
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutocompleteState {
    /// Create a new autocomplete state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            options: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            list_state,
            is_focused: false,
            show_suggestions: false,
            selected_value: None,
        }
    }

    /// Set available options
    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        self.list_state.select(Some(0));
    }

    /// Get current input
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Get selected value (confirmed selection)
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_value.as_deref()
    }

    /// Set selected value
    pub fn set_selected_value<S: Into<String>>(&mut self, value: Option<S>) {
        self.selected_value = value.map(|s| s.into());
        if let Some(ref val) = self.selected_value {
            self.input = val.clone();
            self.cursor_position = self.input.len();
        }
    }

    /// Clear selected value
    pub fn clear_selected(&mut self) {
        self.selected_value = None;
        self.input.clear();
        self.cursor_position = 0;
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        if focused {
            self.show_suggestions = !self.input.is_empty();
        } else {
            self.show_suggestions = false;
        }
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Show or hide suggestions
    pub fn set_show_suggestions(&mut self, show: bool) {
        self.show_suggestions = show;
    }

    /// Check if suggestions are shown
    pub fn is_showing_suggestions(&self) -> bool {
        self.show_suggestions
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            return;
        }
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.show_suggestions = true;
        self.list_state.select(Some(0));
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.show_suggestions = !self.input.is_empty();
            self.list_state.select(Some(0));
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
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Get filtered suggestions based on input
    pub fn filtered_suggestions(&self) -> Vec<&str> {
        if self.input.is_empty() {
            self.options.iter().map(|s| s.as_str()).collect()
        } else {
            let input_lower = self.input.to_lowercase();
            self.options
                .iter()
                .filter(|opt| opt.to_lowercase().contains(&input_lower))
                .map(|s| s.as_str())
                .collect()
        }
    }

    /// Select next suggestion
    pub fn select_next(&mut self) {
        let suggestions = self.filtered_suggestions();
        let count = suggestions.len();

        if count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select previous suggestion
    pub fn select_previous(&mut self) {
        let suggestions = self.filtered_suggestions();
        let count = suggestions.len();

        if count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Confirm current selection
    pub fn confirm_selection(&mut self) {
        let suggestions = self.filtered_suggestions();
        if let Some(index) = self.list_state.selected() {
            if index < suggestions.len() {
                let value = suggestions[index].to_string();
                self.selected_value = Some(value.clone());
                self.input = value;
                self.cursor_position = self.input.len();
                self.show_suggestions = false;
            }
        }
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }
}

/// Autocomplete widget
pub struct Autocomplete<'a> {
    placeholder: Option<&'a str>,
    style: Style,
    focused_style: Style,
    selected_style: Style,
    block: Option<Block<'a>>,
    max_suggestions: usize,
}

impl<'a> Autocomplete<'a> {
    /// Create a new autocomplete widget
    pub fn new() -> Self {
        Self {
            placeholder: Some("Type to search..."),
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            block: None,
            max_suggestions: 5,
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

    /// Set selected suggestion style
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set maximum number of suggestions to display
    pub fn max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }
}

impl<'a> Default for Autocomplete<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for Autocomplete<'a> {
    type State = AutocompleteState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout
        let mut constraints = vec![Constraint::Length(3)]; // Input field

        if state.show_suggestions {
            let suggestion_height = state
                .filtered_suggestions()
                .len()
                .min(self.max_suggestions) as u16
                + 2; // +2 for borders
            constraints.push(Constraint::Length(suggestion_height));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render input field
        let input_block = if let Some(mut block) = self.block.clone() {
            let title = if state.selected_value.is_some() {
                "Assignee [selected]"
            } else if state.is_focused {
                "Assignee [typing]"
            } else {
                "Assignee"
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
                .borders(Borders::ALL)
                .title("Assignee")
                .style(if state.is_focused {
                    self.focused_style
                } else {
                    self.style
                })
        };

        let input_inner = input_block.inner(chunks[0]);
        input_block.render(chunks[0], buf);

        // Render input text
        let text = if state.input.is_empty() {
            if let Some(placeholder) = self.placeholder {
                Line::from(Span::styled(
                    placeholder,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ))
            } else {
                Line::from("")
            }
        } else {
            Line::from(state.input.as_str())
        };

        let paragraph = Paragraph::new(text);
        paragraph.render(input_inner, buf);

        // Render cursor
        if state.is_focused && input_inner.width > 0 && input_inner.height > 0 {
            let cursor_x = input_inner.x + state.cursor_position as u16;
            let cursor_y = input_inner.y;

            if cursor_x < input_inner.x + input_inner.width {
                buf.get_mut(cursor_x, cursor_y)
                    .set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }

        // Render suggestions if shown
        if state.show_suggestions && chunks.len() > 1 {
            let suggestions = state.filtered_suggestions();
            let items: Vec<ListItem> = suggestions
                .iter()
                .take(self.max_suggestions)
                .map(|suggestion| ListItem::new(Line::from(suggestion.to_string())))
                .collect();

            let list = if items.is_empty() {
                let empty_items = vec![ListItem::new(Line::from(Span::styled(
                    "No matches",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )))];
                List::new(empty_items)
                    .block(Block::default().borders(Borders::ALL).title("Suggestions"))
            } else {
                List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Suggestions"))
                    .highlight_style(self.selected_style)
                    .highlight_symbol("> ")
            };

            StatefulWidget::render(list, chunks[1], buf, &mut state.list_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_state_creation() {
        let state = AutocompleteState::new();
        assert_eq!(state.input(), "");
        assert_eq!(state.selected_value(), None);
        assert!(!state.is_focused());
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_insert_char() {
        let mut state = AutocompleteState::new();
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');

        assert_eq!(state.input(), "hello");
        assert_eq!(state.cursor_position(), 5);
        assert!(state.is_showing_suggestions());
    }

    #[test]
    fn test_delete_char() {
        let mut state = AutocompleteState::new();
        state.insert_char('h');
        state.insert_char('i');

        state.delete_char();
        assert_eq!(state.input(), "h");
        assert_eq!(state.cursor_position(), 1);
    }

    #[test]
    fn test_filtered_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
            "alice2".to_string(),
        ]);

        // Empty input shows all
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 4);

        // Filter by input
        state.insert_char('a');
        state.insert_char('l');
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"alice"));
        assert!(suggestions.contains(&"alice2"));
    }

    #[test]
    fn test_case_insensitive_filtering() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["Alice".to_string(), "BOB".to_string()]);

        state.insert_char('a');
        state.insert_char('l');
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "Alice");
    }

    #[test]
    fn test_select_next_previous() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);

        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(1));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(2));

        // Wrap around
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0));

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(2));
    }

    #[test]
    fn test_confirm_selection() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string(), "bob".to_string()]);

        state.insert_char('a');
        state.list_state.select(Some(0));
        state.confirm_selection();

        assert_eq!(state.selected_value(), Some("alice"));
        assert_eq!(state.input(), "alice");
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_clear_selected() {
        let mut state = AutocompleteState::new();
        state.set_selected_value(Some("alice"));

        assert_eq!(state.selected_value(), Some("alice"));
        assert_eq!(state.input(), "alice");

        state.clear_selected();
        assert_eq!(state.selected_value(), None);
        assert_eq!(state.input(), "");
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = AutocompleteState::new();
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');

        assert_eq!(state.cursor_position(), 5);

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 4);

        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 5);

        // Can't move past end
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_set_focused() {
        let mut state = AutocompleteState::new();
        state.insert_char('a');

        state.set_focused(true);
        assert!(state.is_focused());
        assert!(state.is_showing_suggestions());

        state.set_focused(false);
        assert!(!state.is_focused());
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_ignore_newlines() {
        let mut state = AutocompleteState::new();
        state.insert_char('h');
        state.insert_char('\n');
        state.insert_char('i');

        assert_eq!(state.input(), "hi");
    }
}
