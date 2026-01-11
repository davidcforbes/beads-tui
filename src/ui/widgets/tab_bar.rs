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
