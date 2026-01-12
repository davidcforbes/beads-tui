//! Tab bar widget for navigation between views

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Widget},
};

pub struct TabBar<'a> {
    tabs: Vec<&'a str>,
    selected: usize,
    block: Option<Block<'a>>,
}

impl<'a> TabBar<'a> {
    pub fn new(tabs: Vec<&'a str>) -> Self {
        Self {
            tabs,
            selected: 0,
            block: None,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let items: Vec<ListItem> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, &name)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!(" {} {} ", i + 1, name)).style(style)
            })
            .collect();

        let list = if let Some(block) = self.block.clone() {
            List::new(items).block(block)
        } else {
            List::new(items).block(Block::default().borders(Borders::ALL).title("Tabs"))
        };

        Widget::render(list, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_new() {
        let tabs = vec!["Tab1", "Tab2", "Tab3"];
        let tab_bar = TabBar::new(tabs.clone());
        assert_eq!(tab_bar.tabs, tabs);
        assert_eq!(tab_bar.selected, 0);
        assert!(tab_bar.block.is_none());
    }

    #[test]
    fn test_tab_bar_empty_tabs() {
        let tabs: Vec<&str> = vec![];
        let tab_bar = TabBar::new(tabs.clone());
        assert_eq!(tab_bar.tabs.len(), 0);
        assert_eq!(tab_bar.selected, 0);
    }

    #[test]
    fn test_tab_bar_single_tab() {
        let tabs = vec!["Only Tab"];
        let tab_bar = TabBar::new(tabs.clone());
        assert_eq!(tab_bar.tabs.len(), 1);
        assert_eq!(tab_bar.tabs[0], "Only Tab");
    }

    #[test]
    fn test_tab_bar_selected() {
        let tabs = vec!["Tab1", "Tab2", "Tab3"];
        let tab_bar = TabBar::new(tabs).selected(2);
        assert_eq!(tab_bar.selected, 2);
    }

    #[test]
    fn test_tab_bar_selected_zero() {
        let tabs = vec!["Tab1", "Tab2"];
        let tab_bar = TabBar::new(tabs).selected(0);
        assert_eq!(tab_bar.selected, 0);
    }

    #[test]
    fn test_tab_bar_selected_beyond_range() {
        let tabs = vec!["Tab1", "Tab2"];
        let tab_bar = TabBar::new(tabs).selected(10);
        assert_eq!(tab_bar.selected, 10); // Doesn't validate bounds
    }

    #[test]
    fn test_tab_bar_block() {
        let tabs = vec!["Tab1", "Tab2"];
        let block = Block::default().title("Custom Title");
        let tab_bar = TabBar::new(tabs).block(block);
        assert!(tab_bar.block.is_some());
    }

    #[test]
    fn test_tab_bar_block_none_by_default() {
        let tabs = vec!["Tab1"];
        let tab_bar = TabBar::new(tabs);
        assert!(tab_bar.block.is_none());
    }

    #[test]
    fn test_tab_bar_builder_chain() {
        let tabs = vec!["Issues", "Dependencies", "Labels"];
        let block = Block::default().title("Navigation");
        let tab_bar = TabBar::new(tabs.clone()).selected(1).block(block);

        assert_eq!(tab_bar.tabs, tabs);
        assert_eq!(tab_bar.selected, 1);
        assert!(tab_bar.block.is_some());
    }

    #[test]
    fn test_tab_bar_multiple_tabs() {
        let tabs = vec!["A", "B", "C", "D", "E"];
        let tab_bar = TabBar::new(tabs.clone());
        assert_eq!(tab_bar.tabs.len(), 5);
        assert_eq!(tab_bar.tabs[0], "A");
        assert_eq!(tab_bar.tabs[4], "E");
    }

    #[test]
    fn test_tab_bar_selected_chaining() {
        let tabs = vec!["Tab1", "Tab2", "Tab3"];
        let tab_bar = TabBar::new(tabs).selected(1).selected(2);
        assert_eq!(tab_bar.selected, 2); // Last selected wins
    }

    #[test]
    fn test_tab_bar_with_long_names() {
        let tabs = vec![
            "Very Long Tab Name One",
            "Another Long Tab Name",
            "Short",
        ];
        let tab_bar = TabBar::new(tabs.clone());
        assert_eq!(tab_bar.tabs.len(), 3);
        assert_eq!(tab_bar.tabs[0], "Very Long Tab Name One");
    }
}
