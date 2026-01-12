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
            let suggestion_height =
                state.filtered_suggestions().len().min(self.max_suggestions) as u16 + 2; // +2 for borders
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

    #[test]
    fn test_autocomplete_state_default() {
        let state = AutocompleteState::default();
        assert_eq!(state.input(), "");
        assert_eq!(state.selected_value(), None);
        assert!(!state.is_focused());
        assert!(!state.is_showing_suggestions());
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_autocomplete_state_clone() {
        let mut state = AutocompleteState::new();
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');
        state.set_focused(true);

        let cloned = state.clone();
        assert_eq!(cloned.input(), "test");
        assert!(cloned.is_focused());
        assert!(cloned.is_showing_suggestions());
        assert_eq!(cloned.cursor_position(), 4);
    }

    #[test]
    fn test_set_options_empty() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![]);
        assert_eq!(state.filtered_suggestions().len(), 0);
    }

    #[test]
    fn test_set_options_single() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["single".to_string()]);
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "single");
    }

    #[test]
    fn test_set_options_resets_selection() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        state.select_next();
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(2));

        state.set_options(vec!["x".to_string(), "y".to_string()]);
        assert_eq!(state.list_state.selected(), Some(0)); // Reset to 0
    }

    #[test]
    fn test_delete_char_at_start() {
        let mut state = AutocompleteState::new();
        state.insert_char('a');
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 0);

        state.delete_char();
        assert_eq!(state.input(), "a"); // No change
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_delete_char_empty_input() {
        let mut state = AutocompleteState::new();
        assert_eq!(state.input(), "");

        state.delete_char();
        assert_eq!(state.input(), ""); // Still empty
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_delete_char_hides_suggestions_when_empty() {
        let mut state = AutocompleteState::new();
        state.insert_char('a');
        assert!(state.is_showing_suggestions());

        state.delete_char();
        assert_eq!(state.input(), "");
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_move_cursor_left_at_start() {
        let mut state = AutocompleteState::new();
        state.insert_char('a');
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 0);

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 0); // Can't go below 0
    }

    #[test]
    fn test_insert_char_in_middle() {
        let mut state = AutocompleteState::new();
        state.insert_char('a');
        state.insert_char('c');
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 1);

        state.insert_char('b');
        assert_eq!(state.input(), "abc");
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_select_next_empty_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![]);
        state.select_next();
        // Should not panic with empty suggestions
    }

    #[test]
    fn test_select_previous_empty_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![]);
        state.select_previous();
        // Should not panic with empty suggestions
    }

    #[test]
    fn test_select_next_single_suggestion() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["only".to_string()]);
        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0)); // Wraps back to 0
    }

    #[test]
    fn test_select_previous_single_suggestion() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["only".to_string()]);
        assert_eq!(state.list_state.selected(), Some(0));

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(0)); // Wraps back to 0
    }

    #[test]
    fn test_confirm_selection_no_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![]);
        state.confirm_selection();
        assert_eq!(state.selected_value(), None);
    }

    #[test]
    fn test_confirm_selection_out_of_bounds() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string()]);
        state.list_state.select(Some(10)); // Out of bounds
        state.confirm_selection();
        assert_eq!(state.selected_value(), None); // Should not set value
    }

    #[test]
    fn test_set_selected_value_some() {
        let mut state = AutocompleteState::new();
        state.set_selected_value(Some("test_value"));
        assert_eq!(state.selected_value(), Some("test_value"));
        assert_eq!(state.input(), "test_value");
        assert_eq!(state.cursor_position(), 10);
    }

    #[test]
    fn test_set_selected_value_none() {
        let mut state = AutocompleteState::new();
        state.set_selected_value(Some("initial"));
        assert_eq!(state.selected_value(), Some("initial"));

        state.set_selected_value(None::<String>);
        assert_eq!(state.selected_value(), None);
    }

    #[test]
    fn test_set_show_suggestions() {
        let mut state = AutocompleteState::new();
        assert!(!state.is_showing_suggestions());

        state.set_show_suggestions(true);
        assert!(state.is_showing_suggestions());

        state.set_show_suggestions(false);
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_set_focused_with_empty_input() {
        let mut state = AutocompleteState::new();
        assert_eq!(state.input(), "");

        state.set_focused(true);
        assert!(state.is_focused());
        assert!(!state.is_showing_suggestions()); // Empty input doesn't show suggestions
    }

    #[test]
    fn test_autocomplete_new() {
        let widget = Autocomplete::new();
        assert_eq!(widget.placeholder, Some("Type to search..."));
        assert_eq!(widget.max_suggestions, 5);
    }

    #[test]
    fn test_autocomplete_default() {
        let widget = Autocomplete::default();
        assert_eq!(widget.placeholder, Some("Type to search..."));
        assert_eq!(widget.max_suggestions, 5);
    }

    #[test]
    fn test_autocomplete_placeholder() {
        let widget = Autocomplete::new().placeholder("Enter name...");
        assert_eq!(widget.placeholder, Some("Enter name..."));
    }

    #[test]
    fn test_autocomplete_style() {
        let style = Style::default().fg(Color::Red);
        let widget = Autocomplete::new().style(style);
        assert_eq!(widget.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_autocomplete_focused_style() {
        let style = Style::default().fg(Color::Blue);
        let widget = Autocomplete::new().focused_style(style);
        assert_eq!(widget.focused_style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_autocomplete_selected_style() {
        let style = Style::default().bg(Color::Yellow);
        let widget = Autocomplete::new().selected_style(style);
        assert_eq!(widget.selected_style.bg, Some(Color::Yellow));
    }

    #[test]
    fn test_autocomplete_block() {
        let block = Block::default().title("Custom");
        let widget = Autocomplete::new().block(block);
        assert!(widget.block.is_some());
    }

    #[test]
    fn test_autocomplete_max_suggestions() {
        let widget = Autocomplete::new().max_suggestions(10);
        assert_eq!(widget.max_suggestions, 10);
    }

    #[test]
    fn test_autocomplete_builder_chain() {
        let block = Block::default().title("Test");
        let style = Style::default().fg(Color::Green);
        let focused_style = Style::default().fg(Color::Cyan);
        let selected_style = Style::default().bg(Color::Blue);

        let widget = Autocomplete::new()
            .placeholder("Search...")
            .style(style)
            .focused_style(focused_style)
            .selected_style(selected_style)
            .block(block)
            .max_suggestions(8);

        assert_eq!(widget.placeholder, Some("Search..."));
        assert_eq!(widget.style.fg, Some(Color::Green));
        assert_eq!(widget.focused_style.fg, Some(Color::Cyan));
        assert_eq!(widget.selected_style.bg, Some(Color::Blue));
        assert_eq!(widget.max_suggestions, 8);
    }

    #[test]
    fn test_autocomplete_state_debug_trait() {
        let state = AutocompleteState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AutocompleteState"));
    }

    #[test]
    fn test_autocomplete_builder_order_independence() {
        let widget1 = Autocomplete::new()
            .placeholder("Search")
            .max_suggestions(10)
            .style(Style::default().fg(Color::Red))
            .focused_style(Style::default().fg(Color::Blue));

        let widget2 = Autocomplete::new()
            .focused_style(Style::default().fg(Color::Blue))
            .style(Style::default().fg(Color::Red))
            .max_suggestions(10)
            .placeholder("Search");

        assert_eq!(widget1.placeholder, widget2.placeholder);
        assert_eq!(widget1.max_suggestions, widget2.max_suggestions);
        assert_eq!(widget1.style, widget2.style);
        assert_eq!(widget1.focused_style, widget2.focused_style);
    }

    #[test]
    fn test_autocomplete_multiple_applications() {
        let widget = Autocomplete::new()
            .placeholder("First")
            .placeholder("Second")
            .placeholder("Third");

        assert_eq!(widget.placeholder, Some("Third"));

        let widget2 = Autocomplete::new()
            .max_suggestions(5)
            .max_suggestions(10)
            .max_suggestions(15);

        assert_eq!(widget2.max_suggestions, 15);
    }

    #[test]
    fn test_very_large_option_list() {
        let mut state = AutocompleteState::new();
        let mut options = Vec::new();
        
        // Create 100 options
        for i in 0..100 {
            options.push(format!("option{:03}", i));
        }
        
        state.set_options(options);
        
        // All options should be available
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 100);
        
        // Filter should work
        state.input.push_str("option050");
        state.cursor_position = state.input.len();
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "option050");
    }

    #[test]
    fn test_max_suggestions_zero() {
        let widget = Autocomplete::new().max_suggestions(0);
        assert_eq!(widget.max_suggestions, 0);
    }

    #[test]
    fn test_max_suggestions_one() {
        let widget = Autocomplete::new().max_suggestions(1);
        assert_eq!(widget.max_suggestions, 1);
    }

    #[test]
    fn test_max_suggestions_very_large() {
        let widget = Autocomplete::new().max_suggestions(1000);
        assert_eq!(widget.max_suggestions, 1000);
    }

    #[test]
    fn test_complex_selection_sequence() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
            "david".to_string(),
        ]);
        
        // Type 'al'
        state.insert_char('a');
        state.insert_char('l');
        assert_eq!(state.input(), "al");
        assert!(state.is_showing_suggestions());
        assert_eq!(state.list_state.selected(), Some(0));
        
        // Only "alice" matches "al"
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 1);
        
        // Confirm selection
        state.confirm_selection();
        assert_eq!(state.selected_value(), Some("alice"));
        assert_eq!(state.input(), "alice");
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_unicode_options() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "Hello 世界".to_string(),
            "Привет мир".to_string(),
            "مرحبا العالم".to_string(),
        ]);
        
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 3);
        
        // Filter with unicode - use input directly to avoid byte boundary issues
        state.input = "世界".to_string();
        state.cursor_position = state.input.len();
        state.show_suggestions = true;
        
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "Hello 世界");
    }

    #[test]
    fn test_special_characters_in_options() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "user@example.com".to_string(),
            "file_name.txt".to_string(),
            "path/to/file".to_string(),
            "!@#$%^&*()".to_string(),
        ]);
        
        state.insert_char('@');
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 2); // user@example.com and !@#$%^&*()
    }

    #[test]
    fn test_all_options_match_filter() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ]);
        
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');
        
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 3); // All match "test"
    }

    #[test]
    fn test_no_options_match_filter() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);
        
        state.insert_char('x');
        state.insert_char('y');
        state.insert_char('z');
        
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 0); // None match "xyz"
    }

    #[test]
    fn test_clear_selected_after_typing() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string(), "bob".to_string()]);
        
        state.insert_char('a');
        state.confirm_selection();
        assert_eq!(state.selected_value(), Some("alice"));
        
        state.clear_selected();
        assert_eq!(state.selected_value(), None);
        assert_eq!(state.input(), "");
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_cursor_position_after_operations() {
        let mut state = AutocompleteState::new();
        
        state.insert_char('h');
        assert_eq!(state.cursor_position(), 1);
        
        state.insert_char('e');
        assert_eq!(state.cursor_position(), 2);
        
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');
        assert_eq!(state.cursor_position(), 5);
        
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 4);
        
        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 5);
        
        state.delete_char();
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_selected_value_persistence() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string(), "bob".to_string()]);
        
        state.insert_char('a');
        state.confirm_selection();
        assert_eq!(state.selected_value(), Some("alice"));
        
        // Selected value should persist until cleared or changed
        state.move_cursor_left();
        assert_eq!(state.selected_value(), Some("alice"));
        
        state.move_cursor_right();
        assert_eq!(state.selected_value(), Some("alice"));
    }

    #[test]
    fn test_focus_state_shows_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string()]);
        
        state.insert_char('a');
        assert!(state.is_showing_suggestions());
        
        state.set_focused(false);
        assert!(!state.is_showing_suggestions());
        assert!(!state.is_focused());
        
        state.set_focused(true);
        assert!(state.is_showing_suggestions()); // Should show since input is not empty
        assert!(state.is_focused());
    }

    #[test]
    fn test_suggestions_hide_after_confirm() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string(), "bob".to_string()]);
        
        state.insert_char('a');
        assert!(state.is_showing_suggestions());
        
        state.confirm_selection();
        assert!(!state.is_showing_suggestions());
    }

    #[test]
    fn test_suggestions_show_after_typing() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string()]);
        
        assert!(!state.is_showing_suggestions());
        
        state.insert_char('a');
        assert!(state.is_showing_suggestions());
        
        state.insert_char('l');
        assert!(state.is_showing_suggestions());
    }

    #[test]
    fn test_delete_all_chars_hides_suggestions() {
        let mut state = AutocompleteState::new();
        state.set_options(vec!["alice".to_string()]);
        
        state.insert_char('a');
        state.insert_char('l');
        assert!(state.is_showing_suggestions());
        
        state.delete_char();
        assert!(state.is_showing_suggestions()); // Still has "a"
        
        state.delete_char();
        assert!(!state.is_showing_suggestions()); // Empty input
    }

    #[test]
    fn test_selection_wraps_around() {
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
        
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0)); // Wraps to start
        
        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(2)); // Wraps to end
    }

    #[test]
    fn test_filtered_suggestions_empty_input() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);
        
        let suggestions = state.filtered_suggestions();
        assert_eq!(suggestions.len(), 3); // All options when input is empty
    }

    #[test]
    fn test_insert_char_resets_selection() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);
        
        state.select_next();
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(2));
        
        state.insert_char('a');
        assert_eq!(state.list_state.selected(), Some(0)); // Reset to first
    }

    #[test]
    fn test_delete_char_resets_selection() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);
        
        state.insert_char('a');
        state.select_next();
        state.select_next();
        
        state.delete_char();
        assert_eq!(state.list_state.selected(), Some(0)); // Reset to first
    }

    #[test]
    fn test_set_options_preserves_input() {
        let mut state = AutocompleteState::new();
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');
        
        assert_eq!(state.input(), "test");
        
        state.set_options(vec!["new1".to_string(), "new2".to_string()]);
        
        assert_eq!(state.input(), "test"); // Input should be preserved
    }

    #[test]
    fn test_case_insensitive_unicode() {
        let mut state = AutocompleteState::new();
        state.set_options(vec![
            "HELLO".to_string(),
            "World".to_string(),
            "TeSt".to_string(),
        ]);
        
        state.insert_char('h');
        state.insert_char('e');
        state.insert_char('l');
        
        let filtered = state.filtered_suggestions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "HELLO");
    }

    #[test]
    fn test_empty_placeholder() {
        let widget = Autocomplete::new().placeholder("");
        assert_eq!(widget.placeholder, Some(""));
    }

    #[test]
    fn test_widget_default_values() {
        let widget = Autocomplete::new();
        
        assert_eq!(widget.placeholder, Some("Type to search..."));
        assert_eq!(widget.style, Style::default());
        assert_eq!(widget.focused_style, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        assert_eq!(widget.selected_style, Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
        assert_eq!(widget.block, None);
        assert_eq!(widget.max_suggestions, 5);
    }

    #[test]
    fn test_cursor_position_getter() {
        let mut state = AutocompleteState::new();
        assert_eq!(state.cursor_position(), 0);
        
        state.insert_char('a');
        assert_eq!(state.cursor_position(), 1);
        
        state.insert_char('b');
        assert_eq!(state.cursor_position(), 2);
        
        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 1);
    }
}
