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

/// Edit mode for Gantt view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// Editing start date
    StartDate,
    /// Editing due date
    DueDate,
    /// Editing time estimate
    Estimate,
}

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
    /// Vertical scroll offset (swim lanes)
    vertical_scroll: usize,
    /// Edit mode for selected issue
    edit_mode: Option<EditMode>,
}

impl GanttViewState {
    /// Create a new Gantt view state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            issues,
            selected_issue: 0,
            config: GanttChartConfig::default(),
            zoom: ZoomLevel::Weeks,
            vertical_scroll: 0,
            edit_mode: None,
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
        // Preserve selected issue ID before regrouping
        let selected_id = self.selected_issue().map(|i| i.id.clone());

        self.config.grouping = match self.config.grouping {
            GroupingMode::None => GroupingMode::Status,
            GroupingMode::Status => GroupingMode::Priority,
            GroupingMode::Priority => GroupingMode::Assignee,
            GroupingMode::Assignee => GroupingMode::Type,
            GroupingMode::Type => GroupingMode::None,
        };

        // Try to restore selection to same issue after regrouping
        if let Some(id) = selected_id {
            if let Some(new_idx) = self.issues.iter().position(|i| i.id == id) {
                self.selected_issue = new_idx;
            }
        }
    }

    /// Pan timeline forward (to the right)
    pub fn pan_forward(&mut self) {
        self.config.timeline.pan_forward();
    }

    /// Pan timeline backward (to the left)
    pub fn pan_backward(&mut self) {
        self.config.timeline.pan_backward();
    }

    /// Scroll down through swim lanes
    pub fn scroll_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
    }

    /// Scroll up through swim lanes
    pub fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
    }

    /// Get current vertical scroll offset
    pub fn vertical_scroll(&self) -> usize {
        self.vertical_scroll
    }

    /// Start editing the start date of selected issue
    pub fn start_edit_start_date(&mut self) {
        if self.selected_issue().is_some() {
            self.edit_mode = Some(EditMode::StartDate);
        }
    }

    /// Start editing the due date of selected issue
    pub fn start_edit_due_date(&mut self) {
        if self.selected_issue().is_some() {
            self.edit_mode = Some(EditMode::DueDate);
        }
    }

    /// Start editing the estimate of selected issue
    pub fn start_edit_estimate(&mut self) {
        if self.selected_issue().is_some() {
            self.edit_mode = Some(EditMode::Estimate);
        }
    }

    /// Cancel edit mode
    pub fn cancel_edit(&mut self) {
        self.edit_mode = None;
    }

    /// Get current edit mode
    pub fn edit_mode(&self) -> Option<EditMode> {
        self.edit_mode
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.edit_mode.is_some()
    }

    /// Update the selected issue with new schedule data
    /// Returns Ok(issue_id) if successful, or Err with error message
    /// The caller is responsible for calling `bd update` with the returned ID and refreshing the issue list
    pub fn apply_edit(&mut self, _new_start: Option<chrono::DateTime<chrono::Utc>>,
                       _new_due: Option<chrono::DateTime<chrono::Utc>>,
                       _new_estimate: Option<String>) -> Result<String, String> {
        let issue_id = self.selected_issue()
            .map(|i| i.id.clone())
            .ok_or("No issue selected")?;

        self.edit_mode = None;

        // Return the issue ID so caller can update via bd CLI
        Ok(issue_id)
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
        // Build title with edit mode indicator
        let title = if let Some(edit_mode) = state.edit_mode() {
            let mode_str = match edit_mode {
                EditMode::StartDate => "EDIT: Start Date",
                EditMode::DueDate => "EDIT: Due Date",
                EditMode::Estimate => "EDIT: Estimate",
            };
            format!("Gantt Chart - {} (Enter to apply, Esc to cancel)", mode_str)
        } else {
            "Gantt Chart".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(if state.is_editing() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            });

        let inner = block.inner(area);
        block.render(area, buf);

        if state.issues.is_empty() {
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

        gantt.render(inner, buf);
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

    #[test]
    fn test_pan_timeline() {
        let mut state = GanttViewState::new(vec![]);
        let initial_start = state.config.timeline.viewport_start;
        let initial_end = state.config.timeline.viewport_end;

        // Pan forward
        state.pan_forward();
        assert!(state.config.timeline.viewport_start > initial_start);
        assert!(state.config.timeline.viewport_end > initial_end);

        // Pan backward
        state.pan_backward();
        assert_eq!(state.config.timeline.viewport_start, initial_start);
        assert_eq!(state.config.timeline.viewport_end, initial_end);
    }

    #[test]
    fn test_vertical_scroll() {
        let mut state = GanttViewState::new(vec![]);
        assert_eq!(state.vertical_scroll(), 0);

        // Scroll down
        state.scroll_down();
        assert_eq!(state.vertical_scroll(), 1);

        state.scroll_down();
        assert_eq!(state.vertical_scroll(), 2);

        // Scroll up
        state.scroll_up();
        assert_eq!(state.vertical_scroll(), 1);

        // Scroll up past zero (should saturate at 0)
        state.scroll_up();
        state.scroll_up();
        assert_eq!(state.vertical_scroll(), 0);
    }

    #[test]
    fn test_edit_mode() {
        let mut state = GanttViewState::new(vec![create_test_issue("TEST-1", "Issue 1")]);
        assert!(!state.is_editing());
        assert_eq!(state.edit_mode(), None);

        // Start editing start date
        state.start_edit_start_date();
        assert!(state.is_editing());
        assert_eq!(state.edit_mode(), Some(EditMode::StartDate));

        // Cancel edit
        state.cancel_edit();
        assert!(!state.is_editing());
        assert_eq!(state.edit_mode(), None);

        // Start editing due date
        state.start_edit_due_date();
        assert_eq!(state.edit_mode(), Some(EditMode::DueDate));

        // Apply edit clears mode and returns issue ID
        let result = state.apply_edit(None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TEST-1");
        assert!(!state.is_editing());
    }

    #[test]
    fn test_edit_mode_requires_selection() {
        let mut state = GanttViewState::new(vec![]);
        assert!(!state.is_editing());

        // Can't edit without selection
        state.start_edit_start_date();
        assert!(!state.is_editing());

        state.start_edit_due_date();
        assert!(!state.is_editing());

        state.start_edit_estimate();
        assert!(!state.is_editing());
    }

    #[test]
    fn test_grouping_preserves_selection() {
        let mut state = GanttViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
            create_test_issue("TEST-3", "Issue 3"),
        ]);

        // Select second issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);
        let selected_id = state.selected_issue().unwrap().id.clone();

        // Change grouping mode
        state.cycle_grouping();

        // Selection should be preserved
        assert_eq!(state.selected_issue().unwrap().id, selected_id);
    }

    #[test]
    fn test_selection_stability_during_pan() {
        let mut state = GanttViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
        ]);

        // Select second issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);

        // Pan timeline
        state.pan_forward();
        state.pan_backward();

        // Selection should remain stable
        assert_eq!(state.selected_issue, 1);
    }

    #[test]
    fn test_selection_stability_during_zoom() {
        let mut state = GanttViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
        ]);

        // Select second issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);

        // Zoom in and out
        state.zoom_in();
        state.zoom_out();

        // Selection should remain stable
        assert_eq!(state.selected_issue, 1);
    }

    #[test]
    fn test_selection_stability_during_scroll() {
        let mut state = GanttViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1"),
            create_test_issue("TEST-2", "Issue 2"),
        ]);

        // Select second issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);

        // Scroll vertically
        state.scroll_down();
        state.scroll_up();

        // Selection should remain stable
        assert_eq!(state.selected_issue, 1);
    }
}
