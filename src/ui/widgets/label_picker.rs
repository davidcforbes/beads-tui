//! Label picker widget with autocomplete

use crate::models::normalize_label_key;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

/// Label picker state
#[derive(Debug, Clone)]
pub struct LabelPickerState {
    available_labels: Vec<String>,
    selected_labels: Vec<String>,
    filter_query: String,
    filter_cursor: usize,
    list_state: ListState,
    is_filtering: bool,
    label_weights: HashMap<String, usize>,
}

impl LabelPickerState {
    /// Create a new label picker state
    pub fn new(available_labels: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            available_labels,
            selected_labels: Vec::new(),
            filter_query: String::new(),
            filter_cursor: 0,
            list_state,
            is_filtering: false,
            label_weights: HashMap::new(),
        }
    }

    /// Get selected labels
    pub fn selected_labels(&self) -> &[String] {
        &self.selected_labels
    }

    /// Set selected labels
    pub fn set_selected_labels(&mut self, labels: Vec<String>) {
        self.selected_labels = labels;
    }

    /// Get available labels
    pub fn available_labels(&self) -> &[String] {
        &self.available_labels
    }

    /// Set available labels
    pub fn set_available_labels(&mut self, labels: Vec<String>) {
        self.available_labels = labels;
        self.list_state.select(Some(0));
    }

    /// Set label usage weights for ranking suggestions
    pub fn set_label_weights(&mut self, weights: HashMap<String, usize>) {
        self.label_weights = weights;
    }

    /// Get filter query
    pub fn filter_query(&self) -> &str {
        &self.filter_query
    }

    /// Check if filtering is active
    pub fn is_filtering(&self) -> bool {
        self.is_filtering
    }

    /// Start filtering mode
    pub fn start_filtering(&mut self) {
        self.is_filtering = true;
        self.filter_query.clear();
        self.filter_cursor = 0;
    }

    /// Stop filtering mode
    pub fn stop_filtering(&mut self) {
        self.is_filtering = false;
        self.filter_query.clear();
        self.filter_cursor = 0;
    }

    /// Insert character in filter query
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            return;
        }
        self.filter_query.insert(self.filter_cursor, c);
        self.filter_cursor += 1;
        self.list_state.select(Some(0));
    }

    /// Delete character from filter query
    pub fn delete_char(&mut self) {
        if self.filter_cursor > 0 {
            self.filter_query.remove(self.filter_cursor - 1);
            self.filter_cursor -= 1;
        }
    }

    /// Get filtered labels based on current query
    pub fn filtered_labels(&self) -> Vec<&str> {
        if self.filter_query.is_empty() {
            let mut labels: Vec<&String> = self.available_labels.iter().collect();
            labels.sort_by(|a, b| {
                let weight_a = self.label_weights.get(*a).cloned().unwrap_or(0);
                let weight_b = self.label_weights.get(*b).cloned().unwrap_or(0);
                weight_b.cmp(&weight_a).then_with(|| a.cmp(b))
            });
            labels.into_iter().map(|s| s.as_str()).collect()
        } else {
            let query_lower = self.filter_query.to_lowercase();
            let normalized_query = normalize_label_key(&query_lower);
            let mut matches: Vec<(usize, &String)> = self
                .available_labels
                .iter()
                .filter_map(|label| {
                    let label_lower = label.to_lowercase();
                    let normalized_label = normalize_label_key(&label_lower);
                    let base_score = fuzzy_score(&label_lower, &query_lower)
                        .or_else(|| fuzzy_score(&normalized_label, &normalized_query))?;
                    let weight = self.label_weights.get(label).cloned().unwrap_or(0);
                    Some((base_score.saturating_add(weight.saturating_mul(10)), label))
                })
                .collect();

            matches.sort_by(|(score_a, label_a), (score_b, label_b)| {
                score_b.cmp(score_a).then_with(|| label_a.cmp(label_b))
            });

            matches
                .into_iter()
                .map(|(_, label)| label.as_str())
                .collect()
        }
    }

    /// Select next label in filtered list
    pub fn select_next(&mut self) {
        let filtered = self.filtered_labels();
        let count = filtered.len();

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

    /// Select previous label in filtered list
    pub fn select_previous(&mut self) {
        let filtered = self.filtered_labels();
        let count = filtered.len();

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

    /// Toggle selection of currently highlighted label
    pub fn toggle_selected(&mut self) {
        let filtered = self.filtered_labels();
        let Some(index) = self.list_state.selected() else {
            return;
        };

        if index >= filtered.len() {
            return;
        }

        let label = filtered[index].to_string();

        if let Some(pos) = self.selected_labels.iter().position(|l| l == &label) {
            self.selected_labels.remove(pos);
        } else {
            self.selected_labels.push(label);
        }
    }

    /// Add a label to selected labels
    pub fn add_label<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !self.selected_labels.contains(&label) {
            self.selected_labels.push(label);
        }
    }

    /// Remove a label from selected labels
    pub fn remove_label(&mut self, label: &str) {
        self.selected_labels.retain(|l| l != label);
    }

    /// Clear all selected labels
    pub fn clear_selected(&mut self) {
        self.selected_labels.clear();
    }

    /// Check if a label is selected
    pub fn is_selected(&self, label: &str) -> bool {
        self.selected_labels.contains(&label.to_string())
    }
}

fn fuzzy_score(label: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return None;
    }

    let haystack = label.as_bytes();
    let needle = query.as_bytes();
    let mut score = 0usize;
    let mut index = 0usize;
    let mut last_match: Option<usize> = None;

    for &needle_byte in needle {
        let mut found = None;
        for (i, &hay_byte) in haystack.iter().enumerate().skip(index) {
            if hay_byte == needle_byte {
                found = Some(i);
                break;
            }
        }

        let pos = found?;
        if let Some(prev) = last_match {
            if pos == prev + 1 {
                score += 3;
            } else {
                score += 1;
            }
        } else {
            score += 2;
            if pos == 0 {
                score += 3;
            }
        }

        last_match = Some(pos);
        index = pos + 1;
    }

    if label.starts_with(query) {
        score += 15;
    } else if label.contains(query) {
        score += 8;
    }

    Some(score)
}

impl Default for LabelPickerState {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Label picker widget
pub struct LabelPicker<'a> {
    title: &'a str,
    show_filter_hint: bool,
    style: Style,
    selected_style: Style,
    active_style: Style,
}

impl<'a> LabelPicker<'a> {
    /// Create a new label picker
    pub fn new() -> Self {
        Self {
            title: "Labels",
            show_filter_hint: true,
            style: Style::default(),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            active_style: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Show or hide filter hint
    pub fn show_filter_hint(mut self, show: bool) -> Self {
        self.show_filter_hint = show;
        self
    }

    /// Set style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set selected item style
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set active label style
    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }
}

impl<'a> Default for LabelPicker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for LabelPicker<'a> {
    type State = LabelPickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create main layout
        let mut constraints = vec![
            Constraint::Min(8), // Label list
        ];

        if !state.selected_labels.is_empty() {
            constraints.insert(0, Constraint::Length(3)); // Selected labels
        }

        if state.is_filtering {
            constraints.push(Constraint::Length(3)); // Filter input
        }

        if self.show_filter_hint && !state.is_filtering {
            constraints.push(Constraint::Length(1)); // Hint text
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_index = 0;

        // Render selected labels
        if !state.selected_labels.is_empty() {
            let selected_text = state
                .selected_labels
                .iter()
                .map(|l| format!("🏷 {l}"))
                .collect::<Vec<_>>()
                .join(", ");

            let selected_block = Block::default().borders(Borders::ALL).title("Selected");

            let selected_inner = selected_block.inner(chunks[chunk_index]);
            selected_block.render(chunks[chunk_index], buf);

            let selected_para =
                Paragraph::new(Line::from(Span::styled(selected_text, self.active_style)));
            selected_para.render(selected_inner, buf);

            chunk_index += 1;
        }

        // Render label list
        let filtered = state.filtered_labels();
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|label| {
                let is_selected = state.is_selected(label);
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let style = if is_selected {
                    self.active_style
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(checkbox),
                    Span::raw(" 🏷  "),
                    Span::styled(label.to_string(), style),
                ]))
            })
            .collect();

        let list_title = if state.is_filtering {
            format!("{} (filtering)", self.title)
        } else {
            self.title.to_string()
        };

        let list = if items.is_empty() {
            let empty_items = vec![ListItem::new(Line::from(Span::styled(
                if state.is_filtering {
                    "No labels match filter"
                } else {
                    "No labels available"
                },
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))];
            List::new(empty_items).block(Block::default().borders(Borders::ALL).title(list_title))
        } else {
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title(list_title))
                .highlight_style(self.selected_style)
                .highlight_symbol("> ")
        };

        StatefulWidget::render(list, chunks[chunk_index], buf, &mut state.list_state);
        chunk_index += 1;

        // Render filter input
        if state.is_filtering && chunk_index < chunks.len() {
            let filter_block = Block::default().borders(Borders::ALL).title("Filter");

            let filter_inner = filter_block.inner(chunks[chunk_index]);
            filter_block.render(chunks[chunk_index], buf);

            let filter_text = if state.filter_query.is_empty() {
                Line::from(Span::styled(
                    "Type to filter...",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ))
            } else {
                Line::from(state.filter_query.as_str())
            };

            let filter_para = Paragraph::new(filter_text);
            filter_para.render(filter_inner, buf);

            // Render cursor
            if !state.filter_query.is_empty() && filter_inner.width > 0 && filter_inner.height > 0 {
                let cursor_x = filter_inner.x + state.filter_cursor as u16;
                let cursor_y = filter_inner.y;

                if cursor_x < filter_inner.x + filter_inner.width {
                    buf.get_mut(cursor_x, cursor_y)
                        .set_style(Style::default().bg(Color::White).fg(Color::Black));
                }
            }

            chunk_index += 1;
        }

        // Render hint text
        if self.show_filter_hint && !state.is_filtering && chunk_index < chunks.len() {
            let hint = Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(" to filter  "),
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw(" to toggle"),
            ]);
            hint.render(chunks[chunk_index], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_picker_state_creation() {
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let state = LabelPickerState::new(labels.clone());

        assert_eq!(state.available_labels(), &labels);
        assert!(state.selected_labels().is_empty());
        assert_eq!(state.filter_query(), "");
        assert!(!state.is_filtering());
    }

    #[test]
    fn test_label_picker_filtering() {
        let labels = vec![
            "bug".to_string(),
            "feature".to_string(),
            "urgent".to_string(),
            "enhancement".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        assert!(state.is_filtering());

        state.insert_char('u');
        state.insert_char('r');
        state.insert_char('g');

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "urgent");

        state.stop_filtering();
        assert!(!state.is_filtering());
        assert_eq!(state.filter_query(), "");
    }

    #[test]
    fn test_label_picker_selection() {
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.add_label("bug");
        assert!(state.is_selected("bug"));
        assert!(!state.is_selected("feature"));

        state.remove_label("bug");
        assert!(!state.is_selected("bug"));

        state.add_label("bug");
        state.add_label("feature");
        assert_eq!(state.selected_labels().len(), 2);

        state.clear_selected();
        assert!(state.selected_labels().is_empty());
    }

    #[test]
    fn test_label_picker_toggle() {
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.list_state.select(Some(0));
        state.toggle_selected();
        assert!(state.is_selected("bug"));

        state.toggle_selected();
        assert!(!state.is_selected("bug"));
    }

    #[test]
    fn test_label_picker_navigation() {
        let labels = vec![
            "bug".to_string(),
            "feature".to_string(),
            "urgent".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(1));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(2));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0)); // Wrap around

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(2));
    }

    #[test]
    fn test_label_picker_filter_deletion() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');

        assert_eq!(state.filter_query(), "test");

        state.delete_char();
        assert_eq!(state.filter_query(), "tes");

        state.delete_char();
        state.delete_char();
        state.delete_char();
        assert_eq!(state.filter_query(), "");
    }

    #[test]
    fn test_label_picker_case_insensitive_filter() {
        let labels = vec![
            "BUG".to_string(),
            "Feature".to_string(),
            "URGENT".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('u');
        state.insert_char('r');

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 2); // "Feature" and "URGENT" contain "ur"
    }

    #[test]
    fn test_label_picker_no_duplicates() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.add_label("bug");
        state.add_label("bug");
        state.add_label("bug");

        assert_eq!(state.selected_labels().len(), 1);
    }

    #[test]
    fn test_label_picker_ignore_newlines() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('t');
        state.insert_char('\n');
        state.insert_char('e');

        assert_eq!(state.filter_query(), "te");
    }

    #[test]
    fn test_label_picker_state_default() {
        let state = LabelPickerState::default();
        assert!(state.available_labels().is_empty());
        assert!(state.selected_labels().is_empty());
        assert!(!state.is_filtering());
    }

    #[test]
    fn test_set_selected_labels() {
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let mut state = LabelPickerState::new(labels);

        let selected = vec!["bug".to_string(), "urgent".to_string()];
        state.set_selected_labels(selected.clone());

        assert_eq!(state.selected_labels(), &selected);
    }

    #[test]
    fn test_set_available_labels() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        let new_labels = vec!["feature".to_string(), "urgent".to_string()];
        state.set_available_labels(new_labels.clone());

        assert_eq!(state.available_labels(), &new_labels);
        assert_eq!(state.list_state.selected(), Some(0)); // Resets selection
    }

    #[test]
    fn test_filtered_labels_with_empty_query() {
        let labels = vec![
            "bug".to_string(),
            "feature".to_string(),
            "urgent".to_string(),
        ];
        let state = LabelPickerState::new(labels.clone());

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered, vec!["bug", "feature", "urgent"]);
    }

    #[test]
    fn test_filtered_labels_partial_match() {
        let labels = vec![
            "bug-fix".to_string(),
            "feature-request".to_string(),
            "bug-report".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('b');
        state.insert_char('u');
        state.insert_char('g');

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"bug-fix"));
        assert!(filtered.contains(&"bug-report"));
    }

    #[test]
    fn test_filtered_labels_fuzzy_normalized_match() {
        let labels = vec!["bug-fix".to_string(), "feature".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        for c in "bugfix".chars() {
            state.insert_char(c);
        }

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "bug-fix");
    }

    #[test]
    fn test_filtered_labels_ranked_by_weight() {
        let labels = vec![
            "bug".to_string(),
            "feature".to_string(),
            "urgent".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        let mut weights = HashMap::new();
        weights.insert("urgent".to_string(), 5);
        weights.insert("bug".to_string(), 2);
        state.set_label_weights(weights);

        let filtered = state.filtered_labels();
        assert_eq!(filtered[0], "urgent");
        assert_eq!(filtered[1], "bug");
    }

    #[test]
    fn test_select_next_with_empty_filtered_list() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('x'); // No match

        let before = state.list_state.selected();
        state.select_next();
        let after = state.list_state.selected();

        assert_eq!(before, after); // Should not change
    }

    #[test]
    fn test_select_previous_with_empty_filtered_list() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('x'); // No match

        let before = state.list_state.selected();
        state.select_previous();
        let after = state.list_state.selected();

        assert_eq!(before, after); // Should not change
    }

    #[test]
    fn test_toggle_selected_with_no_selection() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.list_state.select(None);
        state.toggle_selected();

        assert!(state.selected_labels().is_empty()); // Should not crash
    }

    #[test]
    fn test_toggle_selected_with_invalid_index() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.list_state.select(Some(10)); // Out of bounds
        state.toggle_selected();

        assert!(state.selected_labels().is_empty()); // Should not crash
    }

    #[test]
    fn test_delete_char_at_beginning() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('t');

        // Move cursor to beginning by deleting
        state.delete_char();

        // Try to delete at position 0
        state.delete_char();

        assert_eq!(state.filter_query(), "");
    }

    #[test]
    fn test_insert_char_updates_cursor() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();

        state.insert_char('a');
        assert_eq!(state.filter_cursor, 1);

        state.insert_char('b');
        assert_eq!(state.filter_cursor, 2);

        state.insert_char('c');
        assert_eq!(state.filter_cursor, 3);
    }

    #[test]
    fn test_delete_char_updates_cursor() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('a');
        state.insert_char('b');
        assert_eq!(state.filter_cursor, 2);

        state.delete_char();
        assert_eq!(state.filter_cursor, 1);

        state.delete_char();
        assert_eq!(state.filter_cursor, 0);
    }

    #[test]
    fn test_label_picker_new() {
        let picker = LabelPicker::new();
        assert_eq!(picker.title, "Labels");
        assert!(picker.show_filter_hint);
    }

    #[test]
    fn test_label_picker_default() {
        let picker = LabelPicker::default();
        assert_eq!(picker.title, "Labels");
        assert!(picker.show_filter_hint);
    }

    #[test]
    fn test_label_picker_title() {
        let picker = LabelPicker::new().title("Custom Labels");
        assert_eq!(picker.title, "Custom Labels");
    }

    #[test]
    fn test_label_picker_show_filter_hint() {
        let picker = LabelPicker::new().show_filter_hint(false);
        assert!(!picker.show_filter_hint);
    }

    #[test]
    fn test_label_picker_style() {
        let style = Style::default().fg(Color::Red);
        let picker = LabelPicker::new().style(style);
        assert_eq!(picker.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_label_picker_selected_style() {
        let style = Style::default().bg(Color::Blue);
        let picker = LabelPicker::new().selected_style(style);
        assert_eq!(picker.selected_style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_label_picker_active_style() {
        let style = Style::default().fg(Color::Yellow);
        let picker = LabelPicker::new().active_style(style);
        assert_eq!(picker.active_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_label_picker_builder_chain() {
        let style = Style::default().fg(Color::Green);
        let selected = Style::default().bg(Color::Cyan);
        let active = Style::default().fg(Color::White);

        let picker = LabelPicker::new()
            .title("My Labels")
            .show_filter_hint(false)
            .style(style)
            .selected_style(selected)
            .active_style(active);

        assert_eq!(picker.title, "My Labels");
        assert!(!picker.show_filter_hint);
        assert_eq!(picker.style.fg, Some(Color::Green));
        assert_eq!(picker.selected_style.bg, Some(Color::Cyan));
        assert_eq!(picker.active_style.fg, Some(Color::White));
    }

    #[test]
    fn test_start_filtering_clears_previous_query() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('t');
        state.insert_char('e');
        state.stop_filtering();

        state.start_filtering();
        assert_eq!(state.filter_query(), "");
        assert_eq!(state.filter_cursor, 0);
    }

    #[test]
    fn test_insert_char_resets_list_selection() {
        let labels = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.list_state.select(Some(2));

        state.insert_char('a');
        assert_eq!(state.list_state.selected(), Some(0)); // Resets to 0
    }

    #[test]
    fn test_remove_label_nonexistent() {
        let labels = vec!["bug".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.add_label("bug");
        state.remove_label("feature"); // Not in selected

        assert_eq!(state.selected_labels().len(), 1);
        assert!(state.is_selected("bug"));
    }

    #[test]
    fn test_filtered_labels_no_matches() {
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('x');
        state.insert_char('y');
        state.insert_char('z');

        let filtered = state.filtered_labels();
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_navigation_wraparound_with_filtered_list() {
        let labels = vec![
            "bug-1".to_string(),
            "bug-2".to_string(),
            "feature".to_string(),
        ];
        let mut state = LabelPickerState::new(labels);

        state.start_filtering();
        state.insert_char('b');
        state.insert_char('u');
        state.insert_char('g');

        // Only 2 items match: bug-1, bug-2
        state.list_state.select(Some(1));
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0)); // Wraps to beginning

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(1)); // Wraps to end
    }
}
