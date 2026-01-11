//! Checkbox list widget for multi-selection

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};
use std::collections::HashSet;

/// Checkbox list state
#[derive(Debug, Clone)]
pub struct CheckboxListState<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    items: Vec<T>,
    selected: HashSet<usize>,
    list_state: ListState,
    selection_mode: bool,
}

impl<T> CheckboxListState<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    /// Create a new checkbox list state
    pub fn new(items: Vec<T>) -> Self {
        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            items,
            selected: HashSet::new(),
            list_state,
            selection_mode: false,
        }
    }

    /// Get all items
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Set items
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected.clear();
        if !self.items.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }

    /// Get selected item indices
    pub fn selected_indices(&self) -> Vec<usize> {
        let mut indices: Vec<_> = self.selected.iter().copied().collect();
        indices.sort_unstable();
        indices
    }

    /// Get selected items
    pub fn selected_items(&self) -> Vec<T> {
        self.selected_indices()
            .iter()
            .filter_map(|&idx| self.items.get(idx).cloned())
            .collect()
    }

    /// Check if an item is selected
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    /// Check if selection mode is enabled
    pub fn is_selection_mode(&self) -> bool {
        self.selection_mode
    }

    /// Enable or disable selection mode
    pub fn set_selection_mode(&mut self, enabled: bool) {
        self.selection_mode = enabled;
        if !enabled {
            self.selected.clear();
        }
    }

    /// Toggle selection mode
    pub fn toggle_selection_mode(&mut self) {
        self.set_selection_mode(!self.selection_mode);
    }

    /// Toggle selection of currently highlighted item
    pub fn toggle_selected(&mut self) {
        if !self.selection_mode {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };

        if index >= self.items.len() {
            return;
        }

        if self.selected.contains(&index) {
            self.selected.remove(&index);
        } else {
            self.selected.insert(index);
        }
    }

    /// Select all items
    pub fn select_all(&mut self) {
        if !self.selection_mode {
            return;
        }

        for i in 0..self.items.len() {
            self.selected.insert(i);
        }
    }

    /// Deselect all items
    pub fn deselect_all(&mut self) {
        self.selected.clear();
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let count = self.items.len();
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

    /// Select previous item
    pub fn select_previous(&mut self) {
        let count = self.items.len();
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

    /// Get the number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get the number of selected items
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Get currently highlighted index
    pub fn highlighted_index(&self) -> Option<usize> {
        self.list_state.selected()
    }
}

impl<T> Default for CheckboxListState<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Checkbox list widget
pub struct CheckboxList<'a, T, F>
where
    T: Clone + Eq + std::hash::Hash,
    F: Fn(&T) -> String,
{
    title: &'a str,
    style: Style,
    selected_style: Style,
    checkbox_style: Style,
    block: Option<Block<'a>>,
    show_count: bool,
    item_formatter: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T, F> CheckboxList<'a, T, F>
where
    T: Clone + Eq + std::hash::Hash,
    F: Fn(&T) -> String,
{
    /// Create a new checkbox list
    pub fn new(item_formatter: F) -> Self {
        Self {
            title: "Items",
            style: Style::default(),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            checkbox_style: Style::default().fg(Color::Green),
            block: None,
            show_count: true,
            item_formatter,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
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

    /// Set checkbox style
    pub fn checkbox_style(mut self, style: Style) -> Self {
        self.checkbox_style = style;
        self
    }

    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Show or hide selection count
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }
}

impl<'a, T, F> StatefulWidget for CheckboxList<'a, T, F>
where
    T: Clone + Eq + std::hash::Hash,
    F: Fn(&T) -> String,
{
    type State = CheckboxListState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build title
        let title = if state.selection_mode && self.show_count {
            format!(
                "{} [{}/{}]",
                self.title,
                state.selected_count(),
                state.item_count()
            )
        } else {
            self.title.to_string()
        };

        // Build block
        let block = if let Some(mut block) = self.block {
            block = block.title(title);
            block
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(self.style)
        };

        let inner = block.inner(area);
        block.render(area, buf);

        // Build list items
        let items: Vec<ListItem> = state
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = state.is_selected(idx);
                let item_text = (self.item_formatter)(item);

                let mut spans = Vec::new();

                // Add checkbox if in selection mode
                if state.selection_mode {
                    let checkbox = if is_selected { "[✓] " } else { "[ ] " };
                    spans.push(Span::styled(checkbox, self.checkbox_style));
                }

                // Add item text
                spans.push(Span::raw(item_text));

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Render list
        let list = if items.is_empty() {
            let empty_items = vec![ListItem::new(Line::from(Span::styled(
                "No items",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))];
            List::new(empty_items)
        } else {
            List::new(items)
                .highlight_style(self.selected_style)
                .highlight_symbol("> ")
        };

        StatefulWidget::render(list, inner, buf, &mut state.list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_list_state_creation() {
        let items = vec!["item1", "item2", "item3"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items.clone());

        assert_eq!(state.items(), &items);
        assert_eq!(state.selected_count(), 0);
        assert!(!state.is_selection_mode());
    }

    #[test]
    fn test_toggle_selection_mode() {
        let items = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.toggle_selection_mode();
        assert!(state.is_selection_mode());

        state.toggle_selection_mode();
        assert!(!state.is_selection_mode());
    }

    #[test]
    fn test_toggle_selected() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        // Enable selection mode
        state.set_selection_mode(true);

        // Toggle first item
        state.list_state.select(Some(0));
        state.toggle_selected();
        assert!(state.is_selected(0));
        assert_eq!(state.selected_count(), 1);

        // Toggle it again to deselect
        state.toggle_selected();
        assert!(!state.is_selected(0));
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_toggle_selected_without_selection_mode() {
        let items = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        // Don't enable selection mode
        state.list_state.select(Some(0));
        state.toggle_selected();

        // Should not select anything
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_select_all() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();

        assert_eq!(state.selected_count(), 3);
        assert!(state.is_selected(0));
        assert!(state.is_selected(1));
        assert!(state.is_selected(2));
    }

    #[test]
    fn test_deselect_all() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 3);

        state.deselect_all();
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_selected_items() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(0));
        state.toggle_selected();
        state.list_state.select(Some(2));
        state.toggle_selected();

        let selected = state.selected_items();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&"item1"));
        assert!(selected.contains(&"item3"));
    }

    #[test]
    fn test_navigation() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), Some(0));

        state.select_next();
        assert_eq!(state.highlighted_index(), Some(1));

        state.select_next();
        assert_eq!(state.highlighted_index(), Some(2));

        // Wrap around
        state.select_next();
        assert_eq!(state.highlighted_index(), Some(0));

        state.select_previous();
        assert_eq!(state.highlighted_index(), Some(2));
    }

    #[test]
    fn test_set_items() {
        let items1 = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items1);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 2);

        // Set new items should clear selection
        let items2 = vec!["item3", "item4", "item5"];
        state.set_items(items2.clone());

        assert_eq!(state.items(), &items2);
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_disable_selection_mode_clears_selection() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 3);

        // Disable selection mode should clear selection
        state.set_selection_mode(false);
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_selected_indices() {
        let items = vec!["item1", "item2", "item3", "item4"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(3));
        state.toggle_selected();
        state.list_state.select(Some(1));
        state.toggle_selected();
        state.list_state.select(Some(0));
        state.toggle_selected();

        let indices = state.selected_indices();
        assert_eq!(indices, vec![0, 1, 3]); // Should be sorted
    }

    #[test]
    fn test_empty_list() {
        let items: Vec<&str> = Vec::new();
        let state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.item_count(), 0);
        assert_eq!(state.selected_count(), 0);
        assert_eq!(state.highlighted_index(), None);
    }
}
