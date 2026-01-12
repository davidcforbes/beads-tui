//! Filter builder widget for creating and editing filter criteria

use crate::beads::models::{IssueStatus, IssueType, Priority};
use crate::ui::widgets::filter_panel::FilterCriteria;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

/// Filter builder state
#[derive(Debug, Clone)]
pub struct FilterBuilderState {
    criteria: FilterCriteria,
    list_state: ListState,
    section: FilterSection,
}

/// Section in the filter builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSection {
    Status,
    Priority,
    Type,
    Labels,
}

impl FilterBuilderState {
    /// Create a new filter builder state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            criteria: FilterCriteria::new(),
            list_state,
            section: FilterSection::Status,
        }
    }

    /// Get the filter criteria
    pub fn criteria(&self) -> &FilterCriteria {
        &self.criteria
    }

    /// Get mutable reference to filter criteria
    pub fn criteria_mut(&mut self) -> &mut FilterCriteria {
        &mut self.criteria
    }

    /// Set the filter criteria
    pub fn set_criteria(&mut self, criteria: FilterCriteria) {
        self.criteria = criteria;
    }

    /// Get the current section
    pub fn section(&self) -> FilterSection {
        self.section
    }

    /// Set the section
    pub fn set_section(&mut self, section: FilterSection) {
        self.section = section;
        self.list_state.select(Some(0));
    }

    /// Get the selected index
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Select next item in current section
    pub fn select_next(&mut self) {
        let count = self.section_item_count();
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

    /// Select previous item in current section
    pub fn select_previous(&mut self) {
        let count = self.section_item_count();
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

    /// Toggle the currently selected item
    pub fn toggle_selected(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };

        match self.section {
            FilterSection::Status => {
                let statuses = [
                    IssueStatus::Open,
                    IssueStatus::InProgress,
                    IssueStatus::Blocked,
                    IssueStatus::Closed,
                ];
                if index < statuses.len() {
                    self.criteria.toggle_status(statuses[index]);
                }
            }
            FilterSection::Priority => {
                let priorities = [
                    Priority::P0,
                    Priority::P1,
                    Priority::P2,
                    Priority::P3,
                    Priority::P4,
                ];
                if index < priorities.len() {
                    self.criteria.toggle_priority(priorities[index]);
                }
            }
            FilterSection::Type => {
                let types = [
                    IssueType::Epic,
                    IssueType::Feature,
                    IssueType::Task,
                    IssueType::Bug,
                    IssueType::Chore,
                ];
                if index < types.len() {
                    self.criteria.toggle_type(types[index]);
                }
            }
            FilterSection::Labels => {
                // Labels are handled differently - they're added via input
            }
        }
    }

    fn section_item_count(&self) -> usize {
        match self.section {
            FilterSection::Status => 4,
            FilterSection::Priority => 5,
            FilterSection::Type => 5,
            FilterSection::Labels => self.criteria.labels.len(),
        }
    }

    /// Clear all filters
    pub fn clear_all(&mut self) {
        self.criteria.clear();
    }

    /// Clear current section filters
    pub fn clear_section(&mut self) {
        match self.section {
            FilterSection::Status => self.criteria.statuses.clear(),
            FilterSection::Priority => self.criteria.priorities.clear(),
            FilterSection::Type => self.criteria.types.clear(),
            FilterSection::Labels => self.criteria.labels.clear(),
        }
    }
}

impl Default for FilterBuilderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter builder widget
pub struct FilterBuilder<'a> {
    title: &'a str,
    show_help: bool,
    style: Style,
    selected_style: Style,
    active_style: Style,
}

impl<'a> FilterBuilder<'a> {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self {
            title: "Filter Builder",
            show_help: true,
            style: Style::default(),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            active_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Show or hide help text
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
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

    /// Set active filter style
    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    fn status_color(status: &IssueStatus) -> Color {
        match status {
            IssueStatus::Open => Color::Green,
            IssueStatus::InProgress => Color::Cyan,
            IssueStatus::Blocked => Color::Red,
            IssueStatus::Closed => Color::Gray,
        }
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

    fn type_symbol(issue_type: &IssueType) -> &'static str {
        match issue_type {
            IssueType::Bug => "🐛",
            IssueType::Feature => "✨",
            IssueType::Task => "📋",
            IssueType::Epic => "🎯",
            IssueType::Chore => "🔧",
        }
    }

    fn build_items(&self, state: &FilterBuilderState) -> Vec<ListItem<'static>> {
        match state.section {
            FilterSection::Status => {
                let statuses = [
                    IssueStatus::Open,
                    IssueStatus::InProgress,
                    IssueStatus::Blocked,
                    IssueStatus::Closed,
                ];
                statuses
                    .iter()
                    .map(|status| {
                        let is_active = state.criteria.statuses.contains(status);
                        let checkbox = if is_active { "[✓]" } else { "[ ]" };
                        let style = if is_active {
                            Style::default().fg(Self::status_color(status))
                        } else {
                            Style::default()
                        };

                        ListItem::new(Line::from(vec![
                            Span::raw(checkbox),
                            Span::raw(" "),
                            Span::styled(format!("{status:?}"), style),
                        ]))
                    })
                    .collect()
            }
            FilterSection::Priority => {
                let priorities = [
                    Priority::P0,
                    Priority::P1,
                    Priority::P2,
                    Priority::P3,
                    Priority::P4,
                ];
                priorities
                    .iter()
                    .map(|priority| {
                        let is_active = state.criteria.priorities.contains(priority);
                        let checkbox = if is_active { "[✓]" } else { "[ ]" };
                        let style = if is_active {
                            Style::default().fg(Self::priority_color(priority))
                        } else {
                            Style::default()
                        };

                        let desc = match priority {
                            Priority::P0 => "Critical",
                            Priority::P1 => "High",
                            Priority::P2 => "Medium",
                            Priority::P3 => "Low",
                            Priority::P4 => "Backlog",
                        };

                        ListItem::new(Line::from(vec![
                            Span::raw(checkbox),
                            Span::raw(" "),
                            Span::styled(format!("{priority} ({desc})"), style),
                        ]))
                    })
                    .collect()
            }
            FilterSection::Type => {
                let types = [
                    IssueType::Epic,
                    IssueType::Feature,
                    IssueType::Task,
                    IssueType::Bug,
                    IssueType::Chore,
                ];
                types
                    .iter()
                    .map(|issue_type| {
                        let is_active = state.criteria.types.contains(issue_type);
                        let checkbox = if is_active { "[✓]" } else { "[ ]" };
                        let style = if is_active {
                            self.active_style
                        } else {
                            Style::default()
                        };

                        ListItem::new(Line::from(vec![
                            Span::raw(checkbox),
                            Span::raw(" "),
                            Span::styled(Self::type_symbol(issue_type), Style::default()),
                            Span::raw(" "),
                            Span::styled(format!("{issue_type:?}"), style),
                        ]))
                    })
                    .collect()
            }
            FilterSection::Labels => {
                if state.criteria.labels.is_empty() {
                    vec![ListItem::new(Line::from(Span::styled(
                        "No labels selected. Press + to add.",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    )))]
                } else {
                    state
                        .criteria
                        .labels
                        .iter()
                        .map(|label| {
                            ListItem::new(Line::from(vec![
                                Span::raw("[✓] 🏷  "),
                                Span::styled(label.clone(), Style::default().fg(Color::Magenta)),
                            ]))
                        })
                        .collect()
                }
            }
        }
    }
}

impl<'a> Default for FilterBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for FilterBuilder<'a> {
    type State = FilterBuilderState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create main layout
        let chunks = if self.show_help {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Section tabs
                    Constraint::Min(8),    // Filter list
                    Constraint::Length(3), // Help text
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Section tabs
                    Constraint::Min(8),    // Filter list
                ])
                .split(area)
        };

        // Render section tabs
        let tab_titles = [("Status", FilterSection::Status),
            ("Priority", FilterSection::Priority),
            ("Type", FilterSection::Type),
            ("Labels", FilterSection::Labels)];

        let tab_spans: Vec<Span> = tab_titles
            .iter()
            .enumerate()
            .flat_map(|(i, (title, section))| {
                let is_active = state.section == *section;
                let style = if is_active {
                    self.active_style
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let mut spans = vec![Span::styled(*title, style)];
                if i < tab_titles.len() - 1 {
                    spans.push(Span::raw(" | "));
                }
                spans
            })
            .collect();

        let tabs_block = Block::default().borders(Borders::ALL).title(self.title);
        let tabs_inner = tabs_block.inner(chunks[0]);
        tabs_block.render(chunks[0], buf);

        let tabs_line = Line::from(tab_spans);
        tabs_line.render(tabs_inner, buf);

        // Render filter list
        let items = self.build_items(state);
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{:?} Filters", state.section)),
            )
            .highlight_style(self.selected_style)
            .highlight_symbol("> ");

        StatefulWidget::render(list, chunks[1], buf, &mut state.list_state);

        // Render help text
        if self.show_help && chunks.len() > 2 {
            let help_spans = vec![
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate  "),
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw(" Toggle  "),
                Span::styled("Tab", Style::default().fg(Color::Cyan)),
                Span::raw(" Switch Section  "),
                Span::styled("C", Style::default().fg(Color::Red)),
                Span::raw(" Clear"),
            ];

            let help_block = Block::default().borders(Borders::ALL).title("Help");
            let help_inner = help_block.inner(chunks[2]);
            help_block.render(chunks[2], buf);

            let help_line = Line::from(help_spans);
            help_line.render(help_inner, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder_state_creation() {
        let state = FilterBuilderState::new();
        assert_eq!(state.section(), FilterSection::Status);
        assert_eq!(state.selected(), Some(0));
        assert!(!state.criteria().is_active());
    }

    #[test]
    fn test_filter_builder_section_switching() {
        let mut state = FilterBuilderState::new();

        state.set_section(FilterSection::Priority);
        assert_eq!(state.section(), FilterSection::Priority);
        assert_eq!(state.selected(), Some(0));

        state.set_section(FilterSection::Type);
        assert_eq!(state.section(), FilterSection::Type);
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_filter_builder_navigation() {
        let mut state = FilterBuilderState::new();

        assert_eq!(state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.selected(), Some(1));

        state.select_next();
        assert_eq!(state.selected(), Some(2));

        state.select_previous();
        assert_eq!(state.selected(), Some(1));

        state.select_previous();
        assert_eq!(state.selected(), Some(0));

        // Wrap around
        state.select_previous();
        assert_eq!(state.selected(), Some(3)); // 4 statuses, 0-indexed
    }

    #[test]
    fn test_filter_builder_toggle_status() {
        let mut state = FilterBuilderState::new();

        state.set_section(FilterSection::Status);
        state.list_state.select(Some(0)); // Open

        state.toggle_selected();
        assert!(state.criteria().statuses.contains(&IssueStatus::Open));

        state.toggle_selected();
        assert!(!state.criteria().statuses.contains(&IssueStatus::Open));
    }

    #[test]
    fn test_filter_builder_toggle_priority() {
        let mut state = FilterBuilderState::new();

        state.set_section(FilterSection::Priority);
        state.list_state.select(Some(0)); // P0

        state.toggle_selected();
        assert!(state.criteria().priorities.contains(&Priority::P0));

        state.toggle_selected();
        assert!(!state.criteria().priorities.contains(&Priority::P0));
    }

    #[test]
    fn test_filter_builder_toggle_type() {
        let mut state = FilterBuilderState::new();

        state.set_section(FilterSection::Type);
        state.list_state.select(Some(0)); // Epic

        state.toggle_selected();
        assert!(state.criteria().types.contains(&IssueType::Epic));

        state.toggle_selected();
        assert!(!state.criteria().types.contains(&IssueType::Epic));
    }

    #[test]
    fn test_filter_builder_clear_section() {
        let mut state = FilterBuilderState::new();

        state.criteria_mut().add_status(IssueStatus::Open);
        state.criteria_mut().add_status(IssueStatus::InProgress);
        state.criteria_mut().add_priority(Priority::P0);

        state.set_section(FilterSection::Status);
        state.clear_section();

        assert!(state.criteria().statuses.is_empty());
        assert!(!state.criteria().priorities.is_empty());
    }

    #[test]
    fn test_filter_builder_clear_all() {
        let mut state = FilterBuilderState::new();

        state.criteria_mut().add_status(IssueStatus::Open);
        state.criteria_mut().add_priority(Priority::P0);
        state.criteria_mut().add_type(IssueType::Bug);

        state.clear_all();

        assert!(!state.criteria().is_active());
        assert!(state.criteria().statuses.is_empty());
        assert!(state.criteria().priorities.is_empty());
        assert!(state.criteria().types.is_empty());
    }

    #[test]
    fn test_filter_builder_set_criteria() {
        let mut state = FilterBuilderState::new();
        let mut criteria = FilterCriteria::new();

        criteria.add_status(IssueStatus::Open);
        criteria.add_priority(Priority::P1);

        state.set_criteria(criteria);

        assert!(state.criteria().statuses.contains(&IssueStatus::Open));
        assert!(state.criteria().priorities.contains(&Priority::P1));
    }

    #[test]
    fn test_filter_builder_state_default() {
        let state = FilterBuilderState::default();
        assert_eq!(state.section(), FilterSection::Status);
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_filter_builder_state_criteria_mut() {
        let mut state = FilterBuilderState::new();
        
        state.criteria_mut().add_status(IssueStatus::Open);
        assert!(state.criteria().statuses.contains(&IssueStatus::Open));
    }

    #[test]
    fn test_select_next_wraparound_priority_section() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Priority);
        
        // Priority has 5 items (P0-P4)
        state.list_state.select(Some(4));
        state.select_next();
        assert_eq!(state.selected(), Some(0)); // Wraps to beginning
    }

    #[test]
    fn test_select_next_wraparound_type_section() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Type);
        
        // Type has 5 items
        state.list_state.select(Some(4));
        state.select_next();
        assert_eq!(state.selected(), Some(0)); // Wraps to beginning
    }

    #[test]
    fn test_select_previous_wraparound_priority_section() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Priority);
        
        state.list_state.select(Some(0));
        state.select_previous();
        assert_eq!(state.selected(), Some(4)); // Wraps to end
    }

    #[test]
    fn test_section_item_count_status() {
        let state = FilterBuilderState::new();
        assert_eq!(state.section_item_count(), 4);
    }

    #[test]
    fn test_section_item_count_priority() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Priority);
        assert_eq!(state.section_item_count(), 5);
    }

    #[test]
    fn test_section_item_count_type() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Type);
        assert_eq!(state.section_item_count(), 5);
    }

    #[test]
    fn test_section_item_count_labels_empty() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Labels);
        assert_eq!(state.section_item_count(), 0);
    }

    #[test]
    fn test_section_item_count_labels_with_items() {
        let mut state = FilterBuilderState::new();
        state.criteria_mut().add_label("bug".to_string());
        state.criteria_mut().add_label("urgent".to_string());
        state.set_section(FilterSection::Labels);
        assert_eq!(state.section_item_count(), 2);
    }

    #[test]
    fn test_toggle_selected_labels_section_does_nothing() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Labels);
        state.list_state.select(Some(0));
        
        let before_count = state.criteria().labels.len();
        state.toggle_selected();
        let after_count = state.criteria().labels.len();
        
        assert_eq!(before_count, after_count);
    }

    #[test]
    fn test_toggle_selected_with_no_selection() {
        let mut state = FilterBuilderState::new();
        state.list_state.select(None);
        
        state.toggle_selected();
        // Should not panic, just do nothing
        assert!(state.criteria().statuses.is_empty());
    }

    #[test]
    fn test_clear_section_priority() {
        let mut state = FilterBuilderState::new();
        state.criteria_mut().add_priority(Priority::P0);
        state.criteria_mut().add_priority(Priority::P1);
        state.criteria_mut().add_status(IssueStatus::Open);
        
        state.set_section(FilterSection::Priority);
        state.clear_section();
        
        assert!(state.criteria().priorities.is_empty());
        assert!(!state.criteria().statuses.is_empty()); // Other sections unchanged
    }

    #[test]
    fn test_clear_section_type() {
        let mut state = FilterBuilderState::new();
        state.criteria_mut().add_type(IssueType::Bug);
        state.criteria_mut().add_type(IssueType::Feature);
        
        state.set_section(FilterSection::Type);
        state.clear_section();
        
        assert!(state.criteria().types.is_empty());
    }

    #[test]
    fn test_clear_section_labels() {
        let mut state = FilterBuilderState::new();
        state.criteria_mut().add_label("bug".to_string());
        state.criteria_mut().add_label("urgent".to_string());
        
        state.set_section(FilterSection::Labels);
        state.clear_section();
        
        assert!(state.criteria().labels.is_empty());
    }

    #[test]
    fn test_filter_builder_new() {
        let builder = FilterBuilder::new();
        assert_eq!(builder.title, "Filter Builder");
        assert!(builder.show_help);
    }

    #[test]
    fn test_filter_builder_default() {
        let builder = FilterBuilder::default();
        assert_eq!(builder.title, "Filter Builder");
        assert!(builder.show_help);
    }

    #[test]
    fn test_filter_builder_title() {
        let builder = FilterBuilder::new().title("Custom Title");
        assert_eq!(builder.title, "Custom Title");
    }

    #[test]
    fn test_filter_builder_show_help() {
        let builder = FilterBuilder::new().show_help(false);
        assert!(!builder.show_help);
    }

    #[test]
    fn test_filter_builder_style() {
        let style = Style::default().fg(Color::Red);
        let builder = FilterBuilder::new().style(style);
        assert_eq!(builder.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_filter_builder_selected_style() {
        let style = Style::default().bg(Color::Blue);
        let builder = FilterBuilder::new().selected_style(style);
        assert_eq!(builder.selected_style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_filter_builder_active_style() {
        let style = Style::default().fg(Color::Yellow);
        let builder = FilterBuilder::new().active_style(style);
        assert_eq!(builder.active_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_filter_builder_builder_chain() {
        let style = Style::default().fg(Color::Magenta);
        let selected = Style::default().bg(Color::Green);
        let active = Style::default().fg(Color::White);
        
        let builder = FilterBuilder::new()
            .title("My Filters")
            .show_help(false)
            .style(style)
            .selected_style(selected)
            .active_style(active);
        
        assert_eq!(builder.title, "My Filters");
        assert!(!builder.show_help);
        assert_eq!(builder.style.fg, Some(Color::Magenta));
        assert_eq!(builder.selected_style.bg, Some(Color::Green));
        assert_eq!(builder.active_style.fg, Some(Color::White));
    }

    #[test]
    fn test_status_color_all_statuses() {
        assert_eq!(FilterBuilder::status_color(&IssueStatus::Open), Color::Green);
        assert_eq!(FilterBuilder::status_color(&IssueStatus::InProgress), Color::Cyan);
        assert_eq!(FilterBuilder::status_color(&IssueStatus::Blocked), Color::Red);
        assert_eq!(FilterBuilder::status_color(&IssueStatus::Closed), Color::Gray);
    }

    #[test]
    fn test_priority_color_all_priorities() {
        assert_eq!(FilterBuilder::priority_color(&Priority::P0), Color::Red);
        assert_eq!(FilterBuilder::priority_color(&Priority::P1), Color::LightRed);
        assert_eq!(FilterBuilder::priority_color(&Priority::P2), Color::Yellow);
        assert_eq!(FilterBuilder::priority_color(&Priority::P3), Color::Blue);
        assert_eq!(FilterBuilder::priority_color(&Priority::P4), Color::Gray);
    }

    #[test]
    fn test_type_symbol_all_types() {
        assert_eq!(FilterBuilder::type_symbol(&IssueType::Bug), "🐛");
        assert_eq!(FilterBuilder::type_symbol(&IssueType::Feature), "✨");
        assert_eq!(FilterBuilder::type_symbol(&IssueType::Task), "📋");
        assert_eq!(FilterBuilder::type_symbol(&IssueType::Epic), "🎯");
        assert_eq!(FilterBuilder::type_symbol(&IssueType::Chore), "🔧");
    }

    #[test]
    fn test_toggle_multiple_statuses() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Status);
        
        // Toggle Open (index 0)
        state.list_state.select(Some(0));
        state.toggle_selected();
        assert!(state.criteria().statuses.contains(&IssueStatus::Open));
        
        // Toggle InProgress (index 1)
        state.list_state.select(Some(1));
        state.toggle_selected();
        assert!(state.criteria().statuses.contains(&IssueStatus::InProgress));
        
        // Both should be active
        assert_eq!(state.criteria().statuses.len(), 2);
    }

    #[test]
    fn test_toggle_multiple_priorities() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Priority);
        
        state.list_state.select(Some(0)); // P0
        state.toggle_selected();
        
        state.list_state.select(Some(2)); // P2
        state.toggle_selected();
        
        assert_eq!(state.criteria().priorities.len(), 2);
        assert!(state.criteria().priorities.contains(&Priority::P0));
        assert!(state.criteria().priorities.contains(&Priority::P2));
    }

    #[test]
    fn test_toggle_multiple_types() {
        let mut state = FilterBuilderState::new();
        state.set_section(FilterSection::Type);
        
        state.list_state.select(Some(0)); // Epic
        state.toggle_selected();
        
        state.list_state.select(Some(3)); // Bug
        state.toggle_selected();
        
        assert_eq!(state.criteria().types.len(), 2);
        assert!(state.criteria().types.contains(&IssueType::Epic));
        assert!(state.criteria().types.contains(&IssueType::Bug));
    }
}
