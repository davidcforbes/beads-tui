//! Gantt chart view for timeline and dependency visualization

use crate::beads::models::Issue;
use crate::models::gantt_schedule::{IssueSchedule, ScheduleData, ZoomLevel};
use crate::ui::widgets::gantt_chart::{GanttChart, GanttChartConfig, GroupingMode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// State for Gantt chart view
#[derive(Debug)]
pub struct GanttViewState {
    /// All issues to display
    issues: Vec<Issue>,
    /// Selected issue index
    selected_issue: usize,
    /// Gantt chart configuration
    config: GanttChartConfig,
    /// Timeline zoom level
    zoom: ZoomLevel,
}

impl GanttViewState {
    /// Create a new Gantt view state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            issues,
            selected_issue: 0,
            config: GanttChartConfig::default(),
            zoom: ZoomLevel::Weeks,
        }
    }

    /// Get the issues for display
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Set the issues
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.issues = issues;
        if self.selected_issue >= self.issues.len() && !self.issues.is_empty() {
            self.selected_issue = self.issues.len() - 1;
        }
    }

    /// Get the selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.issues.get(self.selected_issue)
    }

    /// Move selection to next issue
    pub fn next_issue(&mut self) {
        if !self.issues.is_empty() {
            self.selected_issue = (self.selected_issue + 1) % self.issues.len();
        }
    }

    /// Move selection to previous issue
    pub fn previous_issue(&mut self) {
        if !self.issues.is_empty() {
            self.selected_issue = if self.selected_issue == 0 {
                self.issues.len() - 1
            } else {
                self.selected_issue - 1
            };
        }
    }

    /// Zoom in (increase detail)
    pub fn zoom_in(&mut self) {
        self.zoom = match self.zoom {
            ZoomLevel::Months => ZoomLevel::Weeks,
            ZoomLevel::Weeks => ZoomLevel::Days,
            ZoomLevel::Days => ZoomLevel::Hours,
            ZoomLevel::Hours => ZoomLevel::Hours,
        };
        self.config.timeline.zoom_level = self.zoom;
    }

    /// Zoom out (decrease detail)
    pub fn zoom_out(&mut self) {
        self.zoom = match self.zoom {
            ZoomLevel::Hours => ZoomLevel::Days,
            ZoomLevel::Days => ZoomLevel::Weeks,
            ZoomLevel::Weeks => ZoomLevel::Months,
            ZoomLevel::Months => ZoomLevel::Months,
        };
        self.config.timeline.zoom_level = self.zoom;
    }

    /// Toggle grouping mode
    pub fn cycle_grouping(&mut self) {
        self.config.grouping = match self.config.grouping {
            GroupingMode::None => GroupingMode::Status,
            GroupingMode::Status => GroupingMode::Priority,
            GroupingMode::Priority => GroupingMode::Assignee,
            GroupingMode::Assignee => GroupingMode::Type,
            GroupingMode::Type => GroupingMode::None,
        };
    }

    /// Get the Gantt configuration
    pub fn config(&self) -> &GanttChartConfig {
        &self.config
    }
}

/// Gantt chart view widget
pub struct GanttView;

impl Widget for GanttView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Gantt Chart")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        let text = vec![
            Line::from(Span::styled(
                "Gantt Chart View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Press j/k to navigate issues"),
            Line::from("Press +/- to zoom in/out"),
            Line::from("Press g to cycle grouping mode"),
        ];

        let paragraph = Paragraph::new(text);
        paragraph.render(inner, buf);
    }
}

impl GanttView {
    /// Create a new Gantt view
    pub fn new() -> Self {
        Self
    }

    /// Render the Gantt chart with state
    pub fn render_with_state(area: Rect, buf: &mut Buffer, state: &GanttViewState) {
        if state.issues.is_empty() {
            let block = Block::default()
                .title("Gantt Chart")
                .borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);

            let text = Line::from("No issues to display");
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
            paragraph.render(inner, buf);
            return;
        }

        // Compute schedules for all issues
        let issue_refs: Vec<&Issue> = state.issues.iter().collect();
        let schedules: Vec<IssueSchedule> = issue_refs
            .iter()
            .map(|issue| IssueSchedule::from_issue(issue, ScheduleData::default()))
            .collect();

        // Get selected issue ID
        let selected_id = state.selected_issue().map(|issue| issue.id.clone());

        // Create and render Gantt chart
        let gantt = GanttChart::new(schedules, issue_refs)
            .config(state.config.clone())
            .selected(selected_id);

        gantt.render(area, buf);
    }
}

impl Default for GanttView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: None,
            labels: vec![],
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
        }
    }

    #[test]
    fn test_gantt_view_state_new() {
        let issues = vec![create_test_issue("TEST-1", "Test Issue")];
        let state = GanttViewState::new(issues.clone());

        assert_eq!(state.issues().len(), 1);
        assert_eq!(state.selected_issue, 0);
    }

    #[test]
    fn test_navigation() {
        let issues = vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
        ];
        let mut state = GanttViewState::new(issues);

        // Next issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);

        // Next issue (wraps)
        state.next_issue();
        assert_eq!(state.selected_issue, 0);

        // Previous issue (wraps)
        state.previous_issue();
        assert_eq!(state.selected_issue, 1);
    }

    #[test]
    fn test_zoom() {
        let mut state = GanttViewState::new(vec![]);

        assert_eq!(state.zoom, ZoomLevel::Weeks);

        // Zoom in
        state.zoom_in();
        assert_eq!(state.zoom, ZoomLevel::Days);

        // Zoom out
        state.zoom_out();
        assert_eq!(state.zoom, ZoomLevel::Weeks);

        state.zoom_out();
        assert_eq!(state.zoom, ZoomLevel::Months);
    }

    #[test]
    fn test_grouping() {
        let mut state = GanttViewState::new(vec![]);

        assert_eq!(state.config.grouping, GroupingMode::Status);

        state.cycle_grouping();
        assert_eq!(state.config.grouping, GroupingMode::Priority);

        state.cycle_grouping();
        assert_eq!(state.config.grouping, GroupingMode::Assignee);
    }

    #[test]
    fn test_set_issues_updates_selection() {
        let mut state = GanttViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
            create_test_issue("TEST-3", "Issue 3"),
        ]);

        state.selected_issue = 2;

        // Reduce issues - selection should adjust
        state.set_issues(vec![create_test_issue("TEST-1", "Issue 1")]);
        assert_eq!(state.selected_issue, 0);
    }
}
