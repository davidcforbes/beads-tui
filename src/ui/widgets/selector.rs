//! Selector widget for enums and options

use crate::beads::models::{IssueStatus, IssueType, Priority};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

/// Generic selector state for managing selection
#[derive(Debug)]
pub struct SelectorState {
    list_state: ListState,
    is_open: bool,
}

impl Default for SelectorState {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectorState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            list_state: state,
            is_open: false,
        }
    }

    pub fn select_next(&mut self, len: usize) {
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

    pub fn select_previous(&mut self, len: usize) {
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

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn select(&mut self, index: usize, len: usize) {
        if index < len {
            self.list_state.select(Some(index));
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Priority selector widget
pub struct PrioritySelector {
    current: Priority,
    label: Option<String>,
}

impl PrioritySelector {
    pub fn new(current: Priority) -> Self {
        Self {
            current,
            label: Some("Priority".to_string()),
        }
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn get_options() -> Vec<Priority> {
        vec![
            Priority::P0,
            Priority::P1,
            Priority::P2,
            Priority::P3,
            Priority::P4,
        ]
    }

    fn priority_color(priority: &Priority) -> Color {
        match priority {
            Priority::P0 => Color::Red,
            Priority::P1 => Color::LightRed,
            Priority::P2 => Color::Yellow,
            Priority::P3 => Color::Blue,
            Priority::P4 => Color::Gray,
        }
    }

    fn priority_description(priority: &Priority) -> &'static str {
        match priority {
            Priority::P0 => "Critical",
            Priority::P1 => "High",
            Priority::P2 => "Medium",
            Priority::P3 => "Low",
            Priority::P4 => "Backlog",
        }
    }
}

impl StatefulWidget for PrioritySelector {
    type State = SelectorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let options = Self::get_options();

        if !state.is_open {
            // Show current selection
            let text = format!(
                "{} ({})",
                self.current,
                Self::priority_description(&self.current)
            );
            let span = Span::styled(
                text,
                Style::default().fg(Self::priority_color(&self.current)),
            );
            let line = Line::from(vec![span]);

            let title = self.label.unwrap_or_else(|| "Priority".to_string());
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{title} [v]"));

            let paragraph = ratatui::widgets::Paragraph::new(line).block(block);
            ratatui::widgets::Widget::render(paragraph, area, buf);
        } else {
            // Show dropdown list
            let items: Vec<ListItem> = options
                .iter()
                .map(|p| {
                    let text = format!("{} ({})", p, Self::priority_description(p));
                    ListItem::new(text).style(Style::default().fg(Self::priority_color(p)))
                })
                .collect();

            let title = self.label.unwrap_or_else(|| "Priority".to_string());
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{title} [^]")),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            StatefulWidget::render(list, area, buf, &mut state.list_state);
        }
    }
}

/// Status selector widget
pub struct StatusSelector {
    current: IssueStatus,
    label: Option<String>,
}

impl StatusSelector {
    pub fn new(current: IssueStatus) -> Self {
        Self {
            current,
            label: Some("Status".to_string()),
        }
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn get_options() -> Vec<IssueStatus> {
        vec![
            IssueStatus::Open,
            IssueStatus::InProgress,
            IssueStatus::Blocked,
            IssueStatus::Closed,
        ]
    }

    fn status_color(status: &IssueStatus) -> Color {
        match status {
            IssueStatus::Open => Color::Green,
            IssueStatus::InProgress => Color::Cyan,
            IssueStatus::Blocked => Color::Red,
            IssueStatus::Closed => Color::Gray,
        }
    }
}

impl StatefulWidget for StatusSelector {
    type State = SelectorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let options = Self::get_options();

        if !state.is_open {
            // Show current selection
            let text = format!("{:?}", self.current);
            let span = Span::styled(text, Style::default().fg(Self::status_color(&self.current)));
            let line = Line::from(vec![span]);

            let title = self.label.unwrap_or_else(|| "Status".to_string());
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{title} [v]"));

            let paragraph = ratatui::widgets::Paragraph::new(line).block(block);
            ratatui::widgets::Widget::render(paragraph, area, buf);
        } else {
            // Show dropdown list
            let items: Vec<ListItem> = options
                .iter()
                .map(|s| {
                    let text = format!("{s:?}");
                    ListItem::new(text).style(Style::default().fg(Self::status_color(s)))
                })
                .collect();

            let title = self.label.unwrap_or_else(|| "Status".to_string());
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{title} [^]")),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            StatefulWidget::render(list, area, buf, &mut state.list_state);
        }
    }
}

/// Type selector widget
pub struct TypeSelector {
    current: IssueType,
    label: Option<String>,
}

impl TypeSelector {
    pub fn new(current: IssueType) -> Self {
        Self {
            current,
            label: Some("Type".to_string()),
        }
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn get_options() -> Vec<IssueType> {
        vec![
            IssueType::Epic,
            IssueType::Feature,
            IssueType::Task,
            IssueType::Bug,
            IssueType::Chore,
        ]
    }

    fn type_symbol(issue_type: &IssueType) -> &'static str {
        match issue_type {
            IssueType::Bug => "🐛",
            IssueType::Feature => "✨",
            IssueType::Task => "📋",
            IssueType::Epic => "🎯",
            IssueType::Chore => "🔧",
        }
    }
}

impl StatefulWidget for TypeSelector {
    type State = SelectorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let options = Self::get_options();

        if !state.is_open {
            // Show current selection
            let text = format!("{} {:?}", Self::type_symbol(&self.current), self.current);
            let span = Span::styled(text, Style::default());
            let line = Line::from(vec![span]);

            let title = self.label.unwrap_or_else(|| "Type".to_string());
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{title} [v]"));

            let paragraph = ratatui::widgets::Paragraph::new(line).block(block);
            ratatui::widgets::Widget::render(paragraph, area, buf);
        } else {
            // Show dropdown list
            let items: Vec<ListItem> = options
                .iter()
                .map(|t| {
                    let text = format!("{} {:?}", Self::type_symbol(t), t);
                    ListItem::new(text)
                })
                .collect();

            let title = self.label.unwrap_or_else(|| "Type".to_string());
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{title} [^]")),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            StatefulWidget::render(list, area, buf, &mut state.list_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_state_creation() {
        let state = SelectorState::new();
        assert_eq!(state.selected(), Some(0));
        assert!(!state.is_open());
    }

    #[test]
    fn test_selector_state_navigation() {
        let mut state = SelectorState::new();

        state.select_next(5);
        assert_eq!(state.selected(), Some(1));

        state.select_previous(5);
        assert_eq!(state.selected(), Some(0));

        state.select_previous(5);
        assert_eq!(state.selected(), Some(4)); // Wraps around
    }

    #[test]
    fn test_selector_state_toggle() {
        let mut state = SelectorState::new();
        assert!(!state.is_open());

        state.toggle();
        assert!(state.is_open());

        state.toggle();
        assert!(!state.is_open());
    }

    #[test]
    fn test_priority_selector_options() {
        let options = PrioritySelector::get_options();
        assert_eq!(options.len(), 5);
        assert_eq!(options[0], Priority::P0);
        assert_eq!(options[4], Priority::P4);
    }

    #[test]
    fn test_status_selector_options() {
        let options = StatusSelector::get_options();
        assert_eq!(options.len(), 4);
        assert!(options.contains(&IssueStatus::Open));
        assert!(options.contains(&IssueStatus::Closed));
    }

    #[test]
    fn test_type_selector_options() {
        let options = TypeSelector::get_options();
        assert_eq!(options.len(), 5);
        assert!(options.contains(&IssueType::Bug));
        assert!(options.contains(&IssueType::Feature));
    }

    #[test]
    fn test_selector_state_default() {
        let state = SelectorState::default();
        assert_eq!(state.selected(), Some(0));
        assert!(!state.is_open());
    }

    #[test]
    fn test_selector_state_select() {
        let mut state = SelectorState::new();

        state.select(3, 5);
        assert_eq!(state.selected(), Some(3));

        // Out of bounds should not change selection
        state.select(10, 5);
        assert_eq!(state.selected(), Some(3));
    }

    #[test]
    fn test_selector_state_select_next_wraparound() {
        let mut state = SelectorState::new();
        state.select(4, 5);

        state.select_next(5);
        assert_eq!(state.selected(), Some(0)); // Wraps to beginning
    }

    #[test]
    fn test_selector_state_select_previous_wraparound() {
        let mut state = SelectorState::new();
        // Starts at 0

        state.select_previous(5);
        assert_eq!(state.selected(), Some(4)); // Wraps to end
    }

    #[test]
    fn test_selector_state_navigation_with_empty_list() {
        let mut state = SelectorState::new();

        state.select_next(0);
        assert_eq!(state.selected(), Some(0)); // No change

        state.select_previous(0);
        assert_eq!(state.selected(), Some(0)); // No change
    }

    #[test]
    fn test_selector_state_navigation_with_single_item() {
        let mut state = SelectorState::new();

        state.select_next(1);
        assert_eq!(state.selected(), Some(0)); // Wraps to itself

        state.select_previous(1);
        assert_eq!(state.selected(), Some(0)); // Wraps to itself
    }

    #[test]
    fn test_selector_state_open_close() {
        let mut state = SelectorState::new();

        state.open();
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
    }

    #[test]
    fn test_selector_state_multiple_toggles() {
        let mut state = SelectorState::new();

        for _ in 0..4 {
            let before = state.is_open();
            state.toggle();
            assert_ne!(before, state.is_open());
        }

        // After even number of toggles, should be back to initial closed state
        assert!(!state.is_open());
    }

    #[test]
    fn test_priority_selector_creation() {
        let selector = PrioritySelector::new(Priority::P2);
        assert_eq!(selector.current, Priority::P2);
        assert_eq!(selector.label, Some("Priority".to_string()));
    }

    #[test]
    fn test_priority_selector_label() {
        let selector = PrioritySelector::new(Priority::P1).label("Custom Priority");
        assert_eq!(selector.label, Some("Custom Priority".to_string()));
    }

    #[test]
    fn test_priority_selector_all_options() {
        let options = PrioritySelector::get_options();
        assert_eq!(
            options,
            vec![
                Priority::P0,
                Priority::P1,
                Priority::P2,
                Priority::P3,
                Priority::P4,
            ]
        );
    }

    #[test]
    fn test_priority_selector_colors() {
        assert_eq!(PrioritySelector::priority_color(&Priority::P0), Color::Red);
        assert_eq!(
            PrioritySelector::priority_color(&Priority::P1),
            Color::LightRed
        );
        assert_eq!(
            PrioritySelector::priority_color(&Priority::P2),
            Color::Yellow
        );
        assert_eq!(PrioritySelector::priority_color(&Priority::P3), Color::Blue);
        assert_eq!(PrioritySelector::priority_color(&Priority::P4), Color::Gray);
    }

    #[test]
    fn test_priority_selector_descriptions() {
        assert_eq!(
            PrioritySelector::priority_description(&Priority::P0),
            "Critical"
        );
        assert_eq!(
            PrioritySelector::priority_description(&Priority::P1),
            "High"
        );
        assert_eq!(
            PrioritySelector::priority_description(&Priority::P2),
            "Medium"
        );
        assert_eq!(PrioritySelector::priority_description(&Priority::P3), "Low");
        assert_eq!(
            PrioritySelector::priority_description(&Priority::P4),
            "Backlog"
        );
    }

    #[test]
    fn test_status_selector_creation() {
        let selector = StatusSelector::new(IssueStatus::InProgress);
        assert_eq!(selector.current, IssueStatus::InProgress);
        assert_eq!(selector.label, Some("Status".to_string()));
    }

    #[test]
    fn test_status_selector_label() {
        let selector = StatusSelector::new(IssueStatus::Open).label("Custom Status");
        assert_eq!(selector.label, Some("Custom Status".to_string()));
    }

    #[test]
    fn test_status_selector_all_options() {
        let options = StatusSelector::get_options();
        assert_eq!(
            options,
            vec![
                IssueStatus::Open,
                IssueStatus::InProgress,
                IssueStatus::Blocked,
                IssueStatus::Closed,
            ]
        );
    }

    #[test]
    fn test_status_selector_colors() {
        assert_eq!(
            StatusSelector::status_color(&IssueStatus::Open),
            Color::Green
        );
        assert_eq!(
            StatusSelector::status_color(&IssueStatus::InProgress),
            Color::Cyan
        );
        assert_eq!(
            StatusSelector::status_color(&IssueStatus::Blocked),
            Color::Red
        );
        assert_eq!(
            StatusSelector::status_color(&IssueStatus::Closed),
            Color::Gray
        );
    }

    #[test]
    fn test_type_selector_creation() {
        let selector = TypeSelector::new(IssueType::Bug);
        assert_eq!(selector.current, IssueType::Bug);
        assert_eq!(selector.label, Some("Type".to_string()));
    }

    #[test]
    fn test_type_selector_label() {
        let selector = TypeSelector::new(IssueType::Task).label("Custom Type");
        assert_eq!(selector.label, Some("Custom Type".to_string()));
    }

    #[test]
    fn test_type_selector_all_options() {
        let options = TypeSelector::get_options();
        assert_eq!(
            options,
            vec![
                IssueType::Epic,
                IssueType::Feature,
                IssueType::Task,
                IssueType::Bug,
                IssueType::Chore,
            ]
        );
    }

    #[test]
    fn test_type_selector_symbols() {
        assert_eq!(TypeSelector::type_symbol(&IssueType::Epic), "🎯");
        assert_eq!(TypeSelector::type_symbol(&IssueType::Feature), "✨");
        assert_eq!(TypeSelector::type_symbol(&IssueType::Task), "📋");
        assert_eq!(TypeSelector::type_symbol(&IssueType::Bug), "🐛");
        assert_eq!(TypeSelector::type_symbol(&IssueType::Chore), "🔧");
    }

    #[test]
    fn test_selector_state_select_with_none() {
        let mut state = SelectorState::new();
        state.list_state.select(None);

        state.select_next(5);
        assert_eq!(state.selected(), Some(0)); // Starts from 0 when None

        state.list_state.select(None);
        state.select_previous(5);
        assert_eq!(state.selected(), Some(0)); // Starts from 0 when None
    }

    #[test]
    fn test_priority_selector_builder_chain() {
        let selector = PrioritySelector::new(Priority::P3).label("Test");
        assert_eq!(selector.current, Priority::P3);
        assert_eq!(selector.label, Some("Test".to_string()));
    }

    #[test]
    fn test_status_selector_builder_chain() {
        let selector = StatusSelector::new(IssueStatus::Blocked).label("Test");
        assert_eq!(selector.current, IssueStatus::Blocked);
        assert_eq!(selector.label, Some("Test".to_string()));
    }

    #[test]
    fn test_type_selector_builder_chain() {
        let selector = TypeSelector::new(IssueType::Feature).label("Test");
        assert_eq!(selector.current, IssueType::Feature);
        assert_eq!(selector.label, Some("Test".to_string()));
    }
}
