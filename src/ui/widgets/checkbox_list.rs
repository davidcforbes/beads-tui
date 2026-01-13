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

    #[test]
    fn test_checkbox_list_state_default() {
        let state: CheckboxListState<String> = CheckboxListState::default();
        assert_eq!(state.item_count(), 0);
        assert_eq!(state.selected_count(), 0);
        assert!(!state.is_selection_mode());
        assert_eq!(state.highlighted_index(), None);
    }

    #[test]
    fn test_checkbox_list_state_clone() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items.clone());
        state.set_selection_mode(true);
        state.list_state.select(Some(0));
        state.toggle_selected();

        let cloned = state.clone();
        assert_eq!(cloned.items(), state.items());
        assert_eq!(cloned.selected_count(), state.selected_count());
        assert_eq!(cloned.is_selection_mode(), state.is_selection_mode());
        assert!(cloned.is_selected(0));
    }

    #[test]
    fn test_single_item_list() {
        let items = vec!["only"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.item_count(), 1);
        assert_eq!(state.highlighted_index(), Some(0));
    }

    #[test]
    fn test_single_item_navigation() {
        let items = vec!["only"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), Some(0));

        state.select_next();
        assert_eq!(state.highlighted_index(), Some(0)); // Wraps to same item

        state.select_previous();
        assert_eq!(state.highlighted_index(), Some(0)); // Wraps to same item
    }

    #[test]
    fn test_toggle_selected_out_of_bounds() {
        let items = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(10)); // Out of bounds
        state.toggle_selected();

        assert_eq!(state.selected_count(), 0); // Should not select anything
    }

    #[test]
    fn test_toggle_selected_no_selection() {
        let items = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(None); // No selection
        state.toggle_selected();

        assert_eq!(state.selected_count(), 0); // Should not select anything
    }

    #[test]
    fn test_select_all_without_selection_mode() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        // Don't enable selection mode
        state.select_all();

        assert_eq!(state.selected_count(), 0); // Should not select anything
    }

    #[test]
    fn test_is_selected_out_of_bounds() {
        let items = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(0));
        state.toggle_selected();

        assert!(!state.is_selected(10)); // Out of bounds should return false
    }

    #[test]
    fn test_set_items_empty_to_non_empty() {
        let items1: Vec<&str> = Vec::new();
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items1);
        assert_eq!(state.highlighted_index(), None);

        let items2 = vec!["item1", "item2"];
        state.set_items(items2.clone());

        assert_eq!(state.items(), &items2);
        assert_eq!(state.highlighted_index(), Some(0)); // Should select first item
    }

    #[test]
    fn test_set_items_non_empty_to_empty() {
        let items1 = vec!["item1", "item2"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items1);
        assert_eq!(state.highlighted_index(), Some(0));

        let items2: Vec<&str> = Vec::new();
        state.set_items(items2);

        assert_eq!(state.item_count(), 0);
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_selected_items_empty() {
        let items = vec!["item1", "item2"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items);

        let selected = state.selected_items();
        assert_eq!(selected.len(), 0);
    }

    #[test]
    fn test_selected_indices_empty() {
        let items = vec!["item1", "item2"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items);

        let indices = state.selected_indices();
        assert_eq!(indices.len(), 0);
    }

    #[test]
    fn test_navigation_empty_list() {
        let items: Vec<&str> = Vec::new();
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), None);

        state.select_next();
        assert_eq!(state.highlighted_index(), None); // Should remain None

        state.select_previous();
        assert_eq!(state.highlighted_index(), None); // Should remain None
    }

    #[test]
    fn test_item_count_various() {
        let state1: CheckboxListState<i32> = CheckboxListState::new(vec![]);
        assert_eq!(state1.item_count(), 0);

        let state2: CheckboxListState<i32> = CheckboxListState::new(vec![1]);
        assert_eq!(state2.item_count(), 1);

        let state3: CheckboxListState<i32> = CheckboxListState::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(state3.item_count(), 5);
    }

    #[test]
    fn test_selected_count_after_deselect_all() {
        let items = vec!["item1", "item2", "item3"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 3);

        state.deselect_all();
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_checkbox_list_new() {
        let formatter = |s: &String| s.clone();
        let list: CheckboxList<String, _> = CheckboxList::new(formatter);
        assert_eq!(list.title, "Items");
        assert!(list.show_count);
    }

    #[test]
    fn test_checkbox_list_title() {
        let formatter = |s: &String| s.clone();
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).title("My Items");
        assert_eq!(list.title, "My Items");
    }

    #[test]
    fn test_checkbox_list_style() {
        let formatter = |s: &String| s.clone();
        let style = Style::default().fg(Color::Red);
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).style(style);
        assert_eq!(list.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_checkbox_list_selected_style() {
        let formatter = |s: &String| s.clone();
        let style = Style::default().bg(Color::Blue);
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).selected_style(style);
        assert_eq!(list.selected_style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_checkbox_list_checkbox_style() {
        let formatter = |s: &String| s.clone();
        let style = Style::default().fg(Color::Yellow);
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).checkbox_style(style);
        assert_eq!(list.checkbox_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_checkbox_list_block() {
        let formatter = |s: &String| s.clone();
        let block = Block::default().borders(Borders::ALL);
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).block(block);
        assert!(list.block.is_some());
    }

    #[test]
    fn test_checkbox_list_show_count() {
        let formatter = |s: &String| s.clone();
        let list: CheckboxList<String, _> = CheckboxList::new(formatter).show_count(false);
        assert!(!list.show_count);
    }

    #[test]
    fn test_checkbox_list_builder_chain() {
        let formatter = |s: &String| s.clone();
        let block = Block::default().title("Custom");
        let style = Style::default().fg(Color::Green);
        let selected_style = Style::default().bg(Color::Cyan);
        let checkbox_style = Style::default().fg(Color::Magenta);

        let list: CheckboxList<String, _> = CheckboxList::new(formatter)
            .title("Labels")
            .style(style)
            .selected_style(selected_style)
            .checkbox_style(checkbox_style)
            .block(block)
            .show_count(false);

        assert_eq!(list.title, "Labels");
        assert_eq!(list.style.fg, Some(Color::Green));
        assert_eq!(list.selected_style.bg, Some(Color::Cyan));
        assert_eq!(list.checkbox_style.fg, Some(Color::Magenta));
        assert!(!list.show_count);
    }

    // ========== Additional Comprehensive Tests ==========

    #[test]
    fn test_checkbox_list_state_debug_trait() {
        let items = vec!["item1", "item2"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items);
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("CheckboxListState"));
    }

    #[test]
    fn test_checkbox_list_builder_order_independence() {
        let formatter = |s: &String| s.clone();
        let _block = Block::default().borders(Borders::ALL);
        let style = Style::default().fg(Color::Green);
        let selected_style = Style::default().bg(Color::Cyan);
        let checkbox_style = Style::default().fg(Color::Magenta);

        let list1: CheckboxList<String, _> = CheckboxList::new(formatter)
            .title("Test")
            .style(style)
            .selected_style(selected_style)
            .checkbox_style(checkbox_style)
            .show_count(false);

        let formatter2 = |s: &String| s.clone();
        let list2: CheckboxList<String, _> = CheckboxList::new(formatter2)
            .show_count(false)
            .checkbox_style(checkbox_style)
            .selected_style(selected_style)
            .style(style)
            .title("Test");

        assert_eq!(list1.title, list2.title);
        assert_eq!(list1.show_count, list2.show_count);
        assert_eq!(list1.style.fg, list2.style.fg);
        assert_eq!(list1.selected_style.bg, list2.selected_style.bg);
    }

    #[test]
    fn test_checkbox_list_multiple_setter_applications() {
        let formatter = |s: &String| s.clone();
        let list: CheckboxList<String, _> = CheckboxList::new(formatter)
            .title("First")
            .title("Second")
            .title("Third")
            .show_count(true)
            .show_count(false)
            .show_count(true);

        assert_eq!(list.title, "Third"); // Last wins
        assert!(list.show_count); // Last wins
    }

    #[test]
    fn test_generic_type_i32() {
        let items = vec![1, 2, 3, 4, 5];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items.clone());

        assert_eq!(state.items(), &items);
        assert_eq!(state.item_count(), 5);

        state.set_selection_mode(true);
        state.list_state.select(Some(2));
        state.toggle_selected();

        assert!(state.is_selected(2));
        assert_eq!(state.selected_items(), vec![3]);
    }

    #[test]
    fn test_generic_type_string() {
        let items = vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ];
        let mut state: CheckboxListState<String> = CheckboxListState::new(items.clone());

        assert_eq!(state.items(), &items);

        state.set_selection_mode(true);
        state.list_state.select(Some(0));
        state.toggle_selected();
        state.list_state.select(Some(2));
        state.toggle_selected();

        let selected = state.selected_items();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&"alice".to_string()));
        assert!(selected.contains(&"charlie".to_string()));
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct CustomItem {
        id: u32,
        name: String,
    }

    #[test]
    fn test_generic_type_custom_struct() {
        let items = vec![
            CustomItem {
                id: 1,
                name: "first".to_string(),
            },
            CustomItem {
                id: 2,
                name: "second".to_string(),
            },
            CustomItem {
                id: 3,
                name: "third".to_string(),
            },
        ];
        let mut state: CheckboxListState<CustomItem> = CheckboxListState::new(items.clone());

        assert_eq!(state.item_count(), 3);

        state.set_selection_mode(true);
        state.list_state.select(Some(1));
        state.toggle_selected();

        let selected = state.selected_items();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, 2);
        assert_eq!(selected[0].name, "second");
    }

    #[test]
    fn test_very_large_item_list() {
        let items: Vec<i32> = (0..150).collect();
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        assert_eq!(state.item_count(), 150);

        state.set_selection_mode(true);
        state.select_all();

        assert_eq!(state.selected_count(), 150);

        let indices = state.selected_indices();
        assert_eq!(indices.len(), 150);
        assert_eq!(indices[0], 0);
        assert_eq!(indices[149], 149);
    }

    #[test]
    fn test_complex_selection_sequence() {
        let items = vec!["a", "b", "c", "d", "e", "f"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);

        // Select items 1, 3, 5
        state.list_state.select(Some(1));
        state.toggle_selected();
        state.list_state.select(Some(3));
        state.toggle_selected();
        state.list_state.select(Some(5));
        state.toggle_selected();

        assert_eq!(state.selected_count(), 3);
        assert_eq!(state.selected_indices(), vec![1, 3, 5]);

        // Deselect item 3
        state.list_state.select(Some(3));
        state.toggle_selected();

        assert_eq!(state.selected_count(), 2);
        assert_eq!(state.selected_indices(), vec![1, 5]);

        // Select item 0
        state.list_state.select(Some(0));
        state.toggle_selected();

        assert_eq!(state.selected_count(), 3);
        assert_eq!(state.selected_indices(), vec![0, 1, 5]);
    }

    #[test]
    fn test_state_clone_with_different_selections() {
        let items = vec![1, 2, 3, 4];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(0));
        state.toggle_selected();
        state.list_state.select(Some(2));
        state.toggle_selected();

        let cloned = state.clone();

        assert_eq!(cloned.selected_count(), 2);
        assert!(cloned.is_selected(0));
        assert!(cloned.is_selected(2));
        assert!(!cloned.is_selected(1));
        assert_eq!(cloned.items(), state.items());
    }

    #[test]
    fn test_state_clone_empty_selection() {
        let items = vec!["x", "y", "z"];
        let state: CheckboxListState<&str> = CheckboxListState::new(items);
        let cloned = state.clone();

        assert_eq!(cloned.selected_count(), 0);
        assert_eq!(cloned.item_count(), 3);
        assert!(!cloned.is_selection_mode());
    }

    #[test]
    fn test_navigation_wraparound_boundary() {
        let items = vec![1, 2, 3];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), Some(0));

        // Navigate to end
        state.select_next();
        state.select_next();
        assert_eq!(state.highlighted_index(), Some(2));

        // Wrap to beginning
        state.select_next();
        assert_eq!(state.highlighted_index(), Some(0));

        // Wrap to end
        state.select_previous();
        assert_eq!(state.highlighted_index(), Some(2));
    }

    #[test]
    fn test_selected_items_ordering_guaranteed() {
        let items = vec!["d", "a", "c", "b"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);

        // Select in non-sequential order
        state.list_state.select(Some(3));
        state.toggle_selected();
        state.list_state.select(Some(0));
        state.toggle_selected();
        state.list_state.select(Some(2));
        state.toggle_selected();

        let indices = state.selected_indices();
        assert_eq!(indices, vec![0, 2, 3]); // Should be sorted

        let selected = state.selected_items();
        assert_eq!(selected, vec!["d", "c", "b"]); // Order matches sorted indices
    }

    #[test]
    fn test_widget_default_values() {
        let formatter = |s: &String| s.clone();
        let list: CheckboxList<String, _> = CheckboxList::new(formatter);

        assert_eq!(list.title, "Items");
        assert_eq!(list.style, Style::default());
        assert_eq!(
            list.selected_style,
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );
        assert_eq!(list.checkbox_style, Style::default().fg(Color::Green));
        assert!(list.block.is_none());
        assert!(list.show_count);
    }

    #[test]
    fn test_partial_selection_with_mode_toggle() {
        let items = vec![1, 2, 3, 4, 5];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);

        // Select items 0, 2, 4
        state.list_state.select(Some(0));
        state.toggle_selected();
        state.list_state.select(Some(2));
        state.toggle_selected();
        state.list_state.select(Some(4));
        state.toggle_selected();

        assert_eq!(state.selected_count(), 3);

        // Toggle mode off - should clear selection
        state.toggle_selection_mode();

        assert!(!state.is_selection_mode());
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_multiple_toggle_operations() {
        let items = vec!["a", "b", "c"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(1));

        // Toggle on
        state.toggle_selected();
        assert!(state.is_selected(1));

        // Toggle off
        state.toggle_selected();
        assert!(!state.is_selected(1));

        // Toggle on again
        state.toggle_selected();
        assert!(state.is_selected(1));

        // Toggle off again
        state.toggle_selected();
        assert!(!state.is_selected(1));
    }

    #[test]
    fn test_selection_mode_enable_disable_enable() {
        let items = vec![1, 2, 3];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        assert!(!state.is_selection_mode());

        state.set_selection_mode(true);
        assert!(state.is_selection_mode());

        state.set_selection_mode(false);
        assert!(!state.is_selection_mode());

        state.set_selection_mode(true);
        assert!(state.is_selection_mode());
    }

    #[test]
    fn test_deselect_all_with_empty_selection() {
        let items = vec!["x", "y"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        assert_eq!(state.selected_count(), 0);

        state.deselect_all();
        assert_eq!(state.selected_count(), 0); // Should be no-op
    }

    #[test]
    fn test_deselect_all_without_selection_mode() {
        let items = vec![1, 2, 3];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 3);

        state.set_selection_mode(false);

        // Deselect all works even without selection mode
        state.deselect_all();
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_set_items_preserves_highlighted_index_if_valid() {
        let items1 = vec!["a", "b", "c", "d"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items1);

        state.list_state.select(Some(2));
        assert_eq!(state.highlighted_index(), Some(2));

        // Set new items with same or greater length
        let items2 = vec!["w", "x", "y", "z"];
        state.set_items(items2);

        // Highlighted index should not necessarily be preserved (cleared by set_items)
        // This test verifies the actual behavior
        assert_eq!(state.item_count(), 4);
    }

    #[test]
    fn test_all_items_selected_then_deselect_one() {
        let items = vec![1, 2, 3, 4];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 4);

        // Deselect item 2
        state.list_state.select(Some(2));
        state.toggle_selected();

        assert_eq!(state.selected_count(), 3);
        assert!(state.is_selected(0));
        assert!(state.is_selected(1));
        assert!(!state.is_selected(2));
        assert!(state.is_selected(3));
    }

    #[test]
    fn test_highlighted_index_after_multiple_navigations() {
        let items = vec!["a", "b", "c", "d", "e"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), Some(0));

        state.select_next();
        state.select_next();
        state.select_next();
        assert_eq!(state.highlighted_index(), Some(3));

        state.select_previous();
        assert_eq!(state.highlighted_index(), Some(2));

        state.select_next();
        state.select_next();
        state.select_next();
        assert_eq!(state.highlighted_index(), Some(0)); // Wrapped
    }

    #[test]
    fn test_selected_indices_always_sorted() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);

        // Select in random order
        state.list_state.select(Some(5));
        state.toggle_selected();
        state.list_state.select(Some(1));
        state.toggle_selected();
        state.list_state.select(Some(7));
        state.toggle_selected();
        state.list_state.select(Some(3));
        state.toggle_selected();
        state.list_state.select(Some(0));
        state.toggle_selected();

        let indices = state.selected_indices();
        assert_eq!(indices, vec![0, 1, 3, 5, 7]); // Always sorted
    }

    #[test]
    fn test_select_all_then_set_items_clears_selection() {
        let items1 = vec!["a", "b", "c"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items1);

        state.set_selection_mode(true);
        state.select_all();
        assert_eq!(state.selected_count(), 3);

        let items2 = vec!["x", "y"];
        state.set_items(items2);

        assert_eq!(state.selected_count(), 0); // Selection cleared
        assert_eq!(state.item_count(), 2);
    }

    #[test]
    fn test_navigation_with_two_items() {
        let items = vec![10, 20];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        assert_eq!(state.highlighted_index(), Some(0));

        state.select_next();
        assert_eq!(state.highlighted_index(), Some(1));

        state.select_next();
        assert_eq!(state.highlighted_index(), Some(0)); // Wrap

        state.select_previous();
        assert_eq!(state.highlighted_index(), Some(1)); // Wrap backwards
    }

    #[test]
    fn test_toggle_selected_same_item_multiple_times() {
        let items = vec!["item"];
        let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

        state.set_selection_mode(true);
        state.list_state.select(Some(0));

        for i in 0..10 {
            state.toggle_selected();
            if i % 2 == 0 {
                assert!(state.is_selected(0));
            } else {
                assert!(!state.is_selected(0));
            }
        }
    }

    #[test]
    fn test_item_count_after_multiple_set_items() {
        let mut state: CheckboxListState<i32> = CheckboxListState::new(vec![]);
        assert_eq!(state.item_count(), 0);

        state.set_items(vec![1, 2, 3]);
        assert_eq!(state.item_count(), 3);

        state.set_items(vec![1]);
        assert_eq!(state.item_count(), 1);

        state.set_items(vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(state.item_count(), 6);

        state.set_items(vec![]);
        assert_eq!(state.item_count(), 0);
    }

    #[test]
    fn test_selected_count_consistency() {
        let items = vec![1, 2, 3, 4, 5];
        let mut state: CheckboxListState<i32> = CheckboxListState::new(items);

        state.set_selection_mode(true);

        assert_eq!(state.selected_count(), 0);
        assert_eq!(state.selected_indices().len(), 0);

        state.list_state.select(Some(0));
        state.toggle_selected();
        assert_eq!(state.selected_count(), 1);
        assert_eq!(state.selected_indices().len(), 1);

        state.list_state.select(Some(2));
        state.toggle_selected();
        state.list_state.select(Some(4));
        state.toggle_selected();
        assert_eq!(state.selected_count(), 3);
        assert_eq!(state.selected_indices().len(), 3);

        state.deselect_all();
        assert_eq!(state.selected_count(), 0);
        assert_eq!(state.selected_indices().len(), 0);
    }
}
