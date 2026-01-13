//! Filter quick-select menu widget for applying saved filters

use crate::models::SavedFilter;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// State for the filter quick-select menu
#[derive(Debug, Clone)]
pub struct FilterQuickSelectState {
    list_state: ListState,
    filters: Vec<SavedFilter>,
}

impl FilterQuickSelectState {
    /// Create a new filter quick-select state
    pub fn new(filters: Vec<SavedFilter>) -> Self {
        let mut list_state = ListState::default();
        if !filters.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            list_state,
            filters,
        }
    }

    /// Get the number of filters
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if there are no filters
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Get the currently selected filter
    pub fn selected_filter(&self) -> Option<&SavedFilter> {
        self.list_state.selected().and_then(|i| self.filters.get(i))
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Select the next filter
    pub fn select_next(&mut self) {
        let len = self.filters.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select the previous filter
    pub fn select_previous(&mut self) {
        let len = self.filters.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select a filter by index (used for number key shortcuts)
    pub fn select_by_index(&mut self, index: usize) {
        if index < self.filters.len() {
            self.list_state.select(Some(index));
        }
    }

    /// Select a filter by hotkey
    pub fn select_by_hotkey(&mut self, key: char) -> bool {
        if let Some(index) = self.filters.iter().position(|f| f.hotkey == Some(key)) {
            self.list_state.select(Some(index));
            true
        } else {
            false
        }
    }
}

/// Filter quick-select menu widget
pub struct FilterQuickSelectMenu;

impl FilterQuickSelectMenu {
    pub fn new() -> Self {
        Self
    }

    /// Render the filter quick-select menu with state
    pub fn render_with_state(self, area: Rect, buf: &mut Buffer, state: &mut FilterQuickSelectState) {
        StatefulWidget::render(self, area, buf, state);
    }
}

impl Default for FilterQuickSelectMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for FilterQuickSelectMenu {
    type State = FilterQuickSelectState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create block for the menu
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Quick Select Filter ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if state.is_empty() {
            // Show empty message
            let message = Paragraph::new("No saved filters. Press Ctrl+S to save the current filter.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            message.render(inner_area, buf);
            return;
        }

        // Split area into filter list and help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(4)])
            .split(inner_area);

        // Create list items
        let items: Vec<ListItem> = state
            .filters
            .iter()
            .enumerate()
            .map(|(idx, filter)| {
                let hotkey_str = filter
                    .hotkey
                    .map(|h| format!("[{}]", h))
                    .unwrap_or_else(|| "   ".to_string());

                let num_str = format!("{}.", idx + 1);

                // Build filter description
                let mut filter_desc = Vec::new();
                if let Some(ref status) = filter.filter.status {
                    filter_desc.push(format!("status:{}", status));
                }
                if let Some(ref priority) = filter.filter.priority {
                    filter_desc.push(format!("priority:{}", priority));
                }
                if let Some(ref issue_type) = filter.filter.issue_type {
                    filter_desc.push(format!("type:{}", issue_type));
                }
                if !filter.filter.labels.is_empty() {
                    filter_desc.push(format!("labels:{}", filter.filter.labels.join(",")));
                }
                if let Some(ref search) = filter.filter.search_text {
                    let truncated = if search.len() > 20 {
                        format!("{}...", &search[..17])
                    } else {
                        search.clone()
                    };
                    filter_desc.push(format!("search:'{}'", truncated));
                }

                let desc_str = if filter_desc.is_empty() {
                    "all issues".to_string()
                } else {
                    filter_desc.join(", ")
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", num_str),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{:4} ", hotkey_str),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        format!("{:<20} ", filter.name),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(desc_str, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(line)
            })
            .collect();

        // Render the list
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        StatefulWidget::render(list, chunks[0], buf, &mut state.list_state);

        // Render help text
        let help_text = vec![
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate | "),
                Span::styled("1-9", Style::default().fg(Color::Yellow)),
                Span::raw(": Quick select | "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": Apply"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Cyan)),
                Span::raw(": Edit | "),
                Span::styled("d", Style::default().fg(Color::Magenta)),
                Span::raw(": Delete | "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(": Close"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

        help.render(chunks[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IssueFilter;

    fn create_test_filter(name: &str, hotkey: Option<char>) -> SavedFilter {
        SavedFilter {
            name: name.to_string(),
            filter: IssueFilter::new(),
            hotkey,
        }
    }

    #[test]
    fn test_filter_quick_select_state_creation() {
        let filters = vec![
            create_test_filter("Filter 1", Some('1')),
            create_test_filter("Filter 2", Some('2')),
        ];

        let state = FilterQuickSelectState::new(filters);
        assert_eq!(state.len(), 2);
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_filter_quick_select_state_empty() {
        let state = FilterQuickSelectState::new(vec![]);
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(state.selected_filter().is_none());
    }

    #[test]
    fn test_select_next() {
        let filters = vec![
            create_test_filter("Filter 1", Some('1')),
            create_test_filter("Filter 2", Some('2')),
            create_test_filter("Filter 3", Some('3')),
        ];

        let mut state = FilterQuickSelectState::new(filters);
        assert_eq!(state.selected_index(), Some(0));

        state.select_next();
        assert_eq!(state.selected_index(), Some(1));

        state.select_next();
        assert_eq!(state.selected_index(), Some(2));

        // Should wrap around
        state.select_next();
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_select_previous() {
        let filters = vec![
            create_test_filter("Filter 1", Some('1')),
            create_test_filter("Filter 2", Some('2')),
            create_test_filter("Filter 3", Some('3')),
        ];

        let mut state = FilterQuickSelectState::new(filters);
        assert_eq!(state.selected_index(), Some(0));

        // Should wrap around
        state.select_previous();
        assert_eq!(state.selected_index(), Some(2));

        state.select_previous();
        assert_eq!(state.selected_index(), Some(1));

        state.select_previous();
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_select_by_index() {
        let filters = vec![
            create_test_filter("Filter 1", Some('1')),
            create_test_filter("Filter 2", Some('2')),
            create_test_filter("Filter 3", Some('3')),
        ];

        let mut state = FilterQuickSelectState::new(filters);

        state.select_by_index(1);
        assert_eq!(state.selected_index(), Some(1));

        state.select_by_index(2);
        assert_eq!(state.selected_index(), Some(2));

        // Out of bounds should not change selection
        state.select_by_index(10);
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn test_select_by_hotkey() {
        let filters = vec![
            create_test_filter("Filter 1", Some('a')),
            create_test_filter("Filter 2", Some('b')),
            create_test_filter("Filter 3", None),
        ];

        let mut state = FilterQuickSelectState::new(filters);

        assert!(state.select_by_hotkey('b'));
        assert_eq!(state.selected_index(), Some(1));

        assert!(state.select_by_hotkey('a'));
        assert_eq!(state.selected_index(), Some(0));

        // Non-existent hotkey
        assert!(!state.select_by_hotkey('z'));
        assert_eq!(state.selected_index(), Some(0)); // Should not change
    }

    #[test]
    fn test_selected_filter() {
        let filters = vec![
            create_test_filter("Filter 1", Some('1')),
            create_test_filter("Filter 2", Some('2')),
        ];

        let mut state = FilterQuickSelectState::new(filters);

        let selected = state.selected_filter();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Filter 1");

        state.select_next();
        let selected = state.selected_filter();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Filter 2");
    }

    #[test]
    fn test_navigate_empty_list() {
        let mut state = FilterQuickSelectState::new(vec![]);

        state.select_next();
        assert!(state.selected_filter().is_none());

        state.select_previous();
        assert!(state.selected_filter().is_none());
    }

    #[test]
    fn test_select_by_index_empty_list() {
        let mut state = FilterQuickSelectState::new(vec![]);
        state.select_by_index(0);
        assert!(state.selected_filter().is_none());
    }

    #[test]
    fn test_select_by_hotkey_empty_list() {
        let mut state = FilterQuickSelectState::new(vec![]);
        assert!(!state.select_by_hotkey('a'));
    }

    #[test]
    fn test_filter_quick_select_menu_creation() {
        let _menu = FilterQuickSelectMenu::new();
        // Just verify it can be constructed
        let _menu2 = FilterQuickSelectMenu::default();
    }
}
