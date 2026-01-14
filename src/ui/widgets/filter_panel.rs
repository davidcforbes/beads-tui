//! Filter panel widget for displaying and managing active filters

use crate::beads::models::{IssueStatus, IssueType, Priority};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

/// Filter criteria for issues
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub statuses: Vec<IssueStatus>,
    pub priorities: Vec<Priority>,
    pub types: Vec<IssueType>,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub search_query: Option<String>,
    pub has_dependencies: Option<bool>,
    pub is_blocked: Option<bool>,
}

impl FilterCriteria {
    /// Create new empty filter criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any filters are active
    pub fn is_active(&self) -> bool {
        !self.statuses.is_empty()
            || !self.priorities.is_empty()
            || !self.types.is_empty()
            || !self.labels.is_empty()
            || self.assignee.is_some()
            || self.search_query.is_some()
            || self.has_dependencies.is_some()
            || self.is_blocked.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.statuses.clear();
        self.priorities.clear();
        self.types.clear();
        self.labels.clear();
        self.assignee = None;
        self.search_query = None;
        self.has_dependencies = None;
        self.is_blocked = None;
    }

    /// Add a status filter
    pub fn add_status(&mut self, status: IssueStatus) {
        if !self.statuses.contains(&status) {
            self.statuses.push(status);
        }
    }

    /// Remove a status filter
    pub fn remove_status(&mut self, status: &IssueStatus) {
        self.statuses.retain(|s| s != status);
    }

    /// Toggle a status filter
    pub fn toggle_status(&mut self, status: IssueStatus) {
        if self.statuses.contains(&status) {
            self.remove_status(&status);
        } else {
            self.add_status(status);
        }
    }

    /// Add a priority filter
    pub fn add_priority(&mut self, priority: Priority) {
        if !self.priorities.contains(&priority) {
            self.priorities.push(priority);
        }
    }

    /// Remove a priority filter
    pub fn remove_priority(&mut self, priority: &Priority) {
        self.priorities.retain(|p| p != priority);
    }

    /// Toggle a priority filter
    pub fn toggle_priority(&mut self, priority: Priority) {
        if self.priorities.contains(&priority) {
            self.remove_priority(&priority);
        } else {
            self.add_priority(priority);
        }
    }

    /// Add a type filter
    pub fn add_type(&mut self, issue_type: IssueType) {
        if !self.types.contains(&issue_type) {
            self.types.push(issue_type);
        }
    }

    /// Remove a type filter
    pub fn remove_type(&mut self, issue_type: &IssueType) {
        self.types.retain(|t| t != issue_type);
    }

    /// Toggle a type filter
    pub fn toggle_type(&mut self, issue_type: IssueType) {
        if self.types.contains(&issue_type) {
            self.remove_type(&issue_type);
        } else {
            self.add_type(issue_type);
        }
    }

    /// Add a label filter
    pub fn add_label<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !self.labels.contains(&label) {
            self.labels.push(label);
        }
    }

    /// Remove a label filter
    pub fn remove_label(&mut self, label: &str) {
        self.labels.retain(|l| l != label);
    }

    /// Toggle a label filter
    pub fn toggle_label<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if self.labels.contains(&label) {
            self.remove_label(&label);
        } else {
            self.labels.push(label);
        }
    }

    /// Set assignee filter
    pub fn set_assignee<S: Into<String>>(&mut self, assignee: Option<S>) {
        self.assignee = assignee.map(|s| s.into());
    }

    /// Set search query
    pub fn set_search_query<S: Into<String>>(&mut self, query: Option<S>) {
        self.search_query = query.map(|s| s.into());
    }
}

/// Filter panel widget
pub struct FilterPanel<'a> {
    criteria: &'a FilterCriteria,
    result_count: Option<usize>,
    show_empty_message: bool,
    style: Style,
    active_style: Style,
}

impl<'a> FilterPanel<'a> {
    /// Create a new filter panel
    pub fn new(criteria: &'a FilterCriteria) -> Self {
        Self {
            criteria,
            result_count: None,
            show_empty_message: true,
            style: Style::default(),
            active_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set result count
    pub fn result_count(mut self, count: Option<usize>) -> Self {
        self.result_count = count;
        self
    }

    /// Show or hide empty message
    pub fn show_empty_message(mut self, show: bool) -> Self {
        self.show_empty_message = show;
        self
    }

    /// Set style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
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

    fn build_filter_items(&self) -> Vec<ListItem<'_>> {
        let mut items = Vec::new();

        // Status filters
        if !self.criteria.statuses.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Status:",
                self.active_style,
            ))));
            for status in &self.criteria.statuses {
                let symbol = crate::ui::themes::Theme::status_symbol(status);
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(
                        format!("{} {status:?}", symbol),
                        Style::default().fg(Self::status_color(status)),
                    ),
                ])));
            }
        }

        // Priority filters
        if !self.criteria.priorities.is_empty() {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Priority:",
                self.active_style,
            ))));
            for priority in &self.criteria.priorities {
                let symbol = crate::ui::themes::Theme::priority_symbol(priority);
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(
                        format!("{} {}", symbol, priority),
                        Style::default().fg(Self::priority_color(priority)),
                    ),
                ])));
            }
        }

        // Type filters
        if !self.criteria.types.is_empty() {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Type:",
                self.active_style,
            ))));
            for issue_type in &self.criteria.types {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(Self::type_symbol(issue_type), Style::default()),
                    Span::raw(" "),
                    Span::raw(format!("{issue_type:?}")),
                ])));
            }
        }

        // Label filters
        if !self.criteria.labels.is_empty() {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Labels:",
                self.active_style,
            ))));
            for label in &self.criteria.labels {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  🏷  "),
                    Span::styled(label, Style::default().fg(Color::Magenta)),
                ])));
            }
        }

        // Assignee filter
        if let Some(ref assignee) = self.criteria.assignee {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Assignee:",
                self.active_style,
            ))));
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  👤 "),
                Span::styled(assignee, Style::default().fg(Color::Cyan)),
            ])));
        }

        // Search query
        if let Some(ref query) = self.criteria.search_query {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Search:",
                self.active_style,
            ))));
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  🔍 \""),
                Span::styled(query, Style::default().fg(Color::Yellow)),
                Span::raw("\""),
            ])));
        }

        // Boolean filters
        if self.criteria.has_dependencies.is_some() || self.criteria.is_blocked.is_some() {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Conditions:",
                self.active_style,
            ))));

            if let Some(has_deps) = self.criteria.has_dependencies {
                let text = if has_deps {
                    "Has dependencies"
                } else {
                    "No dependencies"
                };
                items.push(ListItem::new(format!("  • {text}")));
            }

            if let Some(is_blocked) = self.criteria.is_blocked {
                let text = if is_blocked { "Blocked" } else { "Not blocked" };
                items.push(ListItem::new(format!("  • {text}")));
            }
        }

        items
    }
}

impl<'a> Widget for FilterPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Filter list
                Constraint::Length(1), // Result count
            ])
            .split(area);

        // Build title with active indicator
        let title = if self.criteria.is_active() {
            " Filters (Active) "
        } else {
            " Filters "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(self.style);

        let inner = block.inner(chunks[0]);
        block.render(chunks[0], buf);

        // Render filter list
        let items = self.build_filter_items();

        if items.is_empty() && self.show_empty_message {
            let empty_msg = Paragraph::new(Line::from(Span::styled(
                "No active filters",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
            empty_msg.render(inner, buf);
        } else {
            let list = List::new(items);
            list.render(inner, buf);
        }

        // Render result count
        if let Some(count) = self.result_count {
            let count_text = if count == 1 {
                "1 result".to_string()
            } else {
                format!("{count} results")
            };
            let count_span = Span::styled(
                count_text,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let count_line = Line::from(count_span);
            count_line.render(chunks[1], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_criteria_creation() {
        let criteria = FilterCriteria::new();
        assert!(!criteria.is_active());
        assert!(criteria.statuses.is_empty());
        assert!(criteria.priorities.is_empty());
        assert!(criteria.types.is_empty());
        assert!(criteria.labels.is_empty());
        assert!(criteria.assignee.is_none());
        assert!(criteria.search_query.is_none());
    }

    #[test]
    fn test_filter_criteria_status() {
        let mut criteria = FilterCriteria::new();

        criteria.add_status(IssueStatus::Open);
        assert!(criteria.is_active());
        assert_eq!(criteria.statuses.len(), 1);
        assert!(criteria.statuses.contains(&IssueStatus::Open));

        criteria.toggle_status(IssueStatus::Open);
        assert!(!criteria.is_active());
        assert!(criteria.statuses.is_empty());

        criteria.toggle_status(IssueStatus::InProgress);
        assert!(criteria.is_active());
        assert!(criteria.statuses.contains(&IssueStatus::InProgress));
    }

    #[test]
    fn test_filter_criteria_priority() {
        let mut criteria = FilterCriteria::new();

        criteria.add_priority(Priority::P0);
        assert!(criteria.priorities.contains(&Priority::P0));

        criteria.remove_priority(&Priority::P0);
        assert!(!criteria.priorities.contains(&Priority::P0));

        criteria.toggle_priority(Priority::P1);
        assert!(criteria.priorities.contains(&Priority::P1));

        criteria.toggle_priority(Priority::P1);
        assert!(!criteria.priorities.contains(&Priority::P1));
    }

    #[test]
    fn test_filter_criteria_type() {
        let mut criteria = FilterCriteria::new();

        criteria.add_type(IssueType::Bug);
        assert!(criteria.types.contains(&IssueType::Bug));

        criteria.remove_type(&IssueType::Bug);
        assert!(!criteria.types.contains(&IssueType::Bug));

        criteria.toggle_type(IssueType::Feature);
        assert!(criteria.types.contains(&IssueType::Feature));
    }

    #[test]
    fn test_filter_criteria_labels() {
        let mut criteria = FilterCriteria::new();

        criteria.add_label("bug");
        assert!(criteria.labels.contains(&"bug".to_string()));

        criteria.remove_label("bug");
        assert!(!criteria.labels.contains(&"bug".to_string()));

        criteria.toggle_label("urgent");
        assert!(criteria.labels.contains(&"urgent".to_string()));

        criteria.toggle_label("urgent");
        assert!(!criteria.labels.contains(&"urgent".to_string()));
    }

    #[test]
    fn test_filter_criteria_assignee() {
        let mut criteria = FilterCriteria::new();

        criteria.set_assignee(Some("john"));
        assert_eq!(criteria.assignee, Some("john".to_string()));
        assert!(criteria.is_active());

        criteria.set_assignee(None::<String>);
        assert!(criteria.assignee.is_none());
    }

    #[test]
    fn test_filter_criteria_search_query() {
        let mut criteria = FilterCriteria::new();

        criteria.set_search_query(Some("test query"));
        assert_eq!(criteria.search_query, Some("test query".to_string()));
        assert!(criteria.is_active());

        criteria.set_search_query(None::<String>);
        assert!(criteria.search_query.is_none());
    }

    #[test]
    fn test_filter_criteria_clear() {
        let mut criteria = FilterCriteria::new();

        criteria.add_status(IssueStatus::Open);
        criteria.add_priority(Priority::P0);
        criteria.add_type(IssueType::Bug);
        criteria.add_label("urgent");
        criteria.set_assignee(Some("john"));
        criteria.set_search_query(Some("test"));

        assert!(criteria.is_active());

        criteria.clear();

        assert!(!criteria.is_active());
        assert!(criteria.statuses.is_empty());
        assert!(criteria.priorities.is_empty());
        assert!(criteria.types.is_empty());
        assert!(criteria.labels.is_empty());
        assert!(criteria.assignee.is_none());
        assert!(criteria.search_query.is_none());
    }

    #[test]
    fn test_filter_criteria_no_duplicates() {
        let mut criteria = FilterCriteria::new();

        criteria.add_status(IssueStatus::Open);
        criteria.add_status(IssueStatus::Open);
        assert_eq!(criteria.statuses.len(), 1);

        criteria.add_priority(Priority::P0);
        criteria.add_priority(Priority::P0);
        assert_eq!(criteria.priorities.len(), 1);

        criteria.add_type(IssueType::Bug);
        criteria.add_type(IssueType::Bug);
        assert_eq!(criteria.types.len(), 1);

        criteria.add_label("test");
        criteria.add_label("test");
        assert_eq!(criteria.labels.len(), 1);
    }

    #[test]
    fn test_filter_criteria_boolean_filters() {
        let mut criteria = FilterCriteria::new();

        criteria.has_dependencies = Some(true);
        assert!(criteria.is_active());

        criteria.is_blocked = Some(false);
        assert!(criteria.is_active());

        criteria.has_dependencies = None;
        criteria.is_blocked = None;
        assert!(!criteria.is_active());
    }

    #[test]
    fn test_filter_criteria_default() {
        let criteria = FilterCriteria::default();
        assert!(!criteria.is_active());
        assert!(criteria.statuses.is_empty());
        assert!(criteria.priorities.is_empty());
        assert!(criteria.types.is_empty());
        assert!(criteria.labels.is_empty());
        assert!(criteria.assignee.is_none());
        assert!(criteria.search_query.is_none());
        assert!(criteria.has_dependencies.is_none());
        assert!(criteria.is_blocked.is_none());
    }

    #[test]
    fn test_filter_criteria_clone() {
        let mut criteria = FilterCriteria::new();
        criteria.add_status(IssueStatus::Open);
        criteria.add_label("test");

        let cloned = criteria.clone();
        assert_eq!(cloned.statuses, criteria.statuses);
        assert_eq!(cloned.labels, criteria.labels);
    }

    #[test]
    fn test_remove_status_from_empty() {
        let mut criteria = FilterCriteria::new();
        criteria.remove_status(&IssueStatus::Open);
        assert!(criteria.statuses.is_empty());
    }

    #[test]
    fn test_remove_priority_from_empty() {
        let mut criteria = FilterCriteria::new();
        criteria.remove_priority(&Priority::P0);
        assert!(criteria.priorities.is_empty());
    }

    #[test]
    fn test_remove_type_from_empty() {
        let mut criteria = FilterCriteria::new();
        criteria.remove_type(&IssueType::Bug);
        assert!(criteria.types.is_empty());
    }

    #[test]
    fn test_remove_label_from_empty() {
        let mut criteria = FilterCriteria::new();
        criteria.remove_label("nonexistent");
        assert!(criteria.labels.is_empty());
    }

    #[test]
    fn test_multiple_statuses() {
        let mut criteria = FilterCriteria::new();
        criteria.add_status(IssueStatus::Open);
        criteria.add_status(IssueStatus::InProgress);
        criteria.add_status(IssueStatus::Blocked);

        assert_eq!(criteria.statuses.len(), 3);
        assert!(criteria.is_active());
    }

    #[test]
    fn test_multiple_priorities() {
        let mut criteria = FilterCriteria::new();
        criteria.add_priority(Priority::P0);
        criteria.add_priority(Priority::P1);
        criteria.add_priority(Priority::P2);

        assert_eq!(criteria.priorities.len(), 3);
        assert!(criteria.is_active());
    }

    #[test]
    fn test_multiple_types() {
        let mut criteria = FilterCriteria::new();
        criteria.add_type(IssueType::Bug);
        criteria.add_type(IssueType::Feature);
        criteria.add_type(IssueType::Task);

        assert_eq!(criteria.types.len(), 3);
        assert!(criteria.is_active());
    }

    #[test]
    fn test_multiple_labels() {
        let mut criteria = FilterCriteria::new();
        criteria.add_label("bug");
        criteria.add_label("urgent");
        criteria.add_label("frontend");

        assert_eq!(criteria.labels.len(), 3);
        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_only_search() {
        let mut criteria = FilterCriteria::new();
        criteria.set_search_query(Some("test"));

        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_only_assignee() {
        let mut criteria = FilterCriteria::new();
        criteria.set_assignee(Some("john"));

        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_has_dependencies_true() {
        let mut criteria = FilterCriteria::new();
        criteria.has_dependencies = Some(true);

        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_has_dependencies_false() {
        let mut criteria = FilterCriteria::new();
        criteria.has_dependencies = Some(false);

        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_is_blocked_true() {
        let mut criteria = FilterCriteria::new();
        criteria.is_blocked = Some(true);

        assert!(criteria.is_active());
    }

    #[test]
    fn test_is_active_with_is_blocked_false() {
        let mut criteria = FilterCriteria::new();
        criteria.is_blocked = Some(false);

        assert!(criteria.is_active());
    }

    #[test]
    fn test_filter_panel_new() {
        let criteria = FilterCriteria::new();
        let panel = FilterPanel::new(&criteria);

        assert!(panel.result_count.is_none());
        assert!(panel.show_empty_message);
    }

    #[test]
    fn test_filter_panel_result_count() {
        let criteria = FilterCriteria::new();
        let panel = FilterPanel::new(&criteria).result_count(Some(42));

        assert_eq!(panel.result_count, Some(42));
    }

    #[test]
    fn test_filter_panel_show_empty_message() {
        let criteria = FilterCriteria::new();
        let panel = FilterPanel::new(&criteria).show_empty_message(false);

        assert!(!panel.show_empty_message);
    }

    #[test]
    fn test_filter_panel_builder_chain() {
        let criteria = FilterCriteria::new();
        let style = Style::default().fg(Color::Red);
        let active_style = Style::default().fg(Color::Green);

        let panel = FilterPanel::new(&criteria)
            .result_count(Some(10))
            .show_empty_message(false)
            .style(style)
            .active_style(active_style);

        assert_eq!(panel.result_count, Some(10));
        assert!(!panel.show_empty_message);
        assert_eq!(panel.style.fg, Some(Color::Red));
        assert_eq!(panel.active_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_status_color_mapping() {
        assert_eq!(FilterPanel::status_color(&IssueStatus::Open), Color::Green);
        assert_eq!(
            FilterPanel::status_color(&IssueStatus::InProgress),
            Color::Cyan
        );
        assert_eq!(FilterPanel::status_color(&IssueStatus::Blocked), Color::Red);
        assert_eq!(FilterPanel::status_color(&IssueStatus::Closed), Color::Gray);
    }

    #[test]
    fn test_priority_color_mapping() {
        assert_eq!(FilterPanel::priority_color(&Priority::P0), Color::Red);
        assert_eq!(FilterPanel::priority_color(&Priority::P1), Color::LightRed);
        assert_eq!(FilterPanel::priority_color(&Priority::P2), Color::Yellow);
        assert_eq!(FilterPanel::priority_color(&Priority::P3), Color::Blue);
        assert_eq!(FilterPanel::priority_color(&Priority::P4), Color::Gray);
    }

    #[test]
    fn test_type_symbol_mapping() {
        assert_eq!(FilterPanel::type_symbol(&IssueType::Bug), "🐛");
        assert_eq!(FilterPanel::type_symbol(&IssueType::Feature), "✨");
        assert_eq!(FilterPanel::type_symbol(&IssueType::Task), "📋");
        assert_eq!(FilterPanel::type_symbol(&IssueType::Epic), "🎯");
        assert_eq!(FilterPanel::type_symbol(&IssueType::Chore), "🔧");
    }

    #[test]
    fn test_clear_with_boolean_filters() {
        let mut criteria = FilterCriteria::new();

        criteria.has_dependencies = Some(true);
        criteria.is_blocked = Some(false);
        assert!(criteria.is_active());

        criteria.clear();

        assert!(criteria.has_dependencies.is_none());
        assert!(criteria.is_blocked.is_none());
        assert!(!criteria.is_active());
    }
}
