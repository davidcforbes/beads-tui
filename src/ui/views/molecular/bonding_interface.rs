//! Bonding Interface for managing issue dependencies

use crate::beads::models::Issue;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Types of Bonds (dependencies)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondType {
    Sequential,
    Parallel,
    Conditional,
}

impl BondType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Sequential => "Sequential (Blocks)",
            Self::Parallel => "Parallel (Related)",
            Self::Conditional => "Conditional (Depends)",
        }
    }
}

/// State for the Bonding Interface
#[derive(Debug)]
pub struct BondingInterfaceState {
    pub left_list_state: ListState,
    pub right_list_state: ListState,
    pub focus_right: bool,
    pub bond_type: BondType,
}

impl Default for BondingInterfaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl BondingInterfaceState {
    pub fn new() -> Self {
        let mut left = ListState::default();
        left.select(Some(0));
        let mut right = ListState::default();
        right.select(Some(0));
        Self {
            left_list_state: left,
            right_list_state: right,
            focus_right: false,
            bond_type: BondType::Sequential,
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus_right = !self.focus_right;
    }

    pub fn select_next(&mut self, len: usize) {
        let state = if self.focus_right {
            &mut self.right_list_state
        } else {
            &mut self.left_list_state
        };
        if len == 0 {
            return;
        }
        let i = match state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    pub fn select_previous(&mut self, len: usize) {
        let state = if self.focus_right {
            &mut self.right_list_state
        } else {
            &mut self.left_list_state
        };
        if len == 0 {
            return;
        }
        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }
}

/// Bonding Interface Widget
pub struct BondingInterface<'a> {
    issues: Vec<&'a Issue>,
    block_style: Style,
}

impl<'a> Default for BondingInterface<'a> {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<'a> BondingInterface<'a> {
    pub fn new(issues: Vec<&'a Issue>) -> Self {
        Self {
            issues,
            block_style: Style::default().fg(Color::Cyan),
        }
    }
}

impl<'a> StatefulWidget for BondingInterface<'a> {
    type State = BondingInterfaceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Bond Type Selector
                Constraint::Min(0),    // Dual Panes
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render Bond Type
        let type_text = format!("Bond Type: {}", state.bond_type.display_name());
        let type_par = Paragraph::new(type_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("1. Select Bond Type")
                .style(self.block_style),
        );
        type_par.render(chunks[0], buf);

        // Render Panes
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let items: Vec<ListItem> = self
            .issues
            .iter()
            .map(|i| ListItem::new(format!("{} {}", i.id, i.title)))
            .collect();

        let left_block = Block::default()
            .borders(Borders::ALL)
            .title("Source (Reactant A)")
            .style(if !state.focus_right {
                Style::default().fg(Color::Yellow)
            } else {
                self.block_style
            });
        let left_list = List::new(items.clone())
            .block(left_block)
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
        StatefulWidget::render(left_list, panes[0], buf, &mut state.left_list_state);

        let right_block = Block::default()
            .borders(Borders::ALL)
            .title("Target (Reactant B)")
            .style(if state.focus_right {
                Style::default().fg(Color::Yellow)
            } else {
                self.block_style
            });
        let right_list = List::new(items)
            .block(right_block)
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
        StatefulWidget::render(right_list, panes[1], buf, &mut state.right_list_state);

        // Render Help
        let help = Paragraph::new(
            "Tab: Switch Panes | j/k: Navigate | Enter: Create Bond | t: Cycle Type",
        )
        .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[2], buf);
    }
}
