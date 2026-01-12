//! Gantt chart widget for visualizing issue schedules and dependencies

use crate::beads::models::Issue;
use crate::models::gantt_schedule::{IssueSchedule, TimelineConfig, ZoomLevel};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use std::collections::HashMap;

/// Grouping mode for swim lanes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum GroupingMode {
    /// No grouping - all issues in a single lane
    None,
    /// Group by issue status
    #[default]
    Status,
    /// Group by issue priority
    Priority,
    /// Group by assignee
    Assignee,
    /// Group by issue type
    Type,
}


/// Configuration for Gantt chart rendering
#[derive(Debug, Clone)]
pub struct GanttChartConfig {
    /// Timeline configuration (zoom, date range)
    pub timeline: TimelineConfig,
    /// Grouping mode for swim lanes
    pub grouping: GroupingMode,
    /// Lane height in rows
    pub lane_height: u16,
    /// Show grid lines
    pub show_grid: bool,
    /// Show today marker
    pub show_today: bool,
    /// Style for the selected issue
    pub selected_style: Style,
    /// Style for normal issues
    pub normal_style: Style,
    /// Style for overdue issues
    pub overdue_style: Style,
}

impl Default for GanttChartConfig {
    fn default() -> Self {
        Self {
            timeline: TimelineConfig::default(),
            grouping: GroupingMode::default(),
            lane_height: 3,
            show_grid: true,
            show_today: true,
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(Color::Cyan),
            overdue_style: Style::default().fg(Color::Red),
        }
    }
}

impl GanttChartConfig {
    /// Create a new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set grouping mode
    pub fn grouping(mut self, mode: GroupingMode) -> Self {
        self.grouping = mode;
        self
    }

    /// Set lane height
    pub fn lane_height(mut self, height: u16) -> Self {
        self.lane_height = height.max(2);
        self
    }
}

/// Swim lane group with issues
#[derive(Debug, Clone)]
struct SwimLane {
    /// Lane name/label
    name: String,
    /// Issues in this lane
    schedules: Vec<IssueSchedule>,
}

/// Gantt chart widget
pub struct GanttChart<'a> {
    /// Issue schedules to render
    schedules: Vec<IssueSchedule>,
    /// Issues map for lookup
    issues: HashMap<String, &'a Issue>,
    /// Configuration
    config: GanttChartConfig,
    /// Selected issue ID
    selected_id: Option<String>,
}

impl<'a> GanttChart<'a> {
    /// Create a new Gantt chart
    pub fn new(schedules: Vec<IssueSchedule>, issues: Vec<&'a Issue>) -> Self {
        let issues_map: HashMap<String, &'a Issue> = issues
            .into_iter()
            .map(|issue| (issue.id.clone(), issue))
            .collect();

        Self {
            schedules,
            issues: issues_map,
            config: GanttChartConfig::default(),
            selected_id: None,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: GanttChartConfig) -> Self {
        self.config = config;
        self
    }

    /// Set selected issue ID
    pub fn selected(mut self, id: Option<String>) -> Self {
        self.selected_id = id;
        self
    }

    /// Group schedules into swim lanes based on grouping mode
    fn group_into_lanes(&self) -> Vec<SwimLane> {
        use crate::beads::models::{IssueStatus, IssueType, Priority};

        let mut lanes: HashMap<String, Vec<IssueSchedule>> = HashMap::new();

        for schedule in &self.schedules {
            let lane_key = if let Some(issue) = self.issues.get(&schedule.issue_id) {
                match self.config.grouping {
                    GroupingMode::None => "All Issues".to_string(),
                    GroupingMode::Status => match issue.status {
                        IssueStatus::Open => "Open",
                        IssueStatus::InProgress => "In Progress",
                        IssueStatus::Blocked => "Blocked",
                        IssueStatus::Closed => "Closed",
                    }
                    .to_string(),
                    GroupingMode::Priority => match issue.priority {
                        Priority::P0 => "P0 - Critical",
                        Priority::P1 => "P1 - High",
                        Priority::P2 => "P2 - Medium",
                        Priority::P3 => "P3 - Low",
                        Priority::P4 => "P4 - Backlog",
                    }
                    .to_string(),
                    GroupingMode::Assignee => {
                        issue.assignee.clone().unwrap_or_else(|| "Unassigned".to_string())
                    }
                    GroupingMode::Type => match issue.issue_type {
                        IssueType::Bug => "Bug",
                        IssueType::Feature => "Feature",
                        IssueType::Task => "Task",
                        IssueType::Epic => "Epic",
                        IssueType::Chore => "Chore",
                    }
                    .to_string(),
                }
            } else {
                "Unknown".to_string()
            };

            lanes
                .entry(lane_key)
                .or_default()
                .push(schedule.clone());
        }

        // Convert to sorted vector of swim lanes
        let mut result: Vec<SwimLane> = lanes
            .into_iter()
            .map(|(name, schedules)| SwimLane { name, schedules })
            .collect();

        // Sort lanes by name
        result.sort_by(|a, b| a.name.cmp(&b.name));

        result
    }

    /// Calculate x position for a date on the timeline
    fn date_to_x(&self, date: DateTime<Utc>, area: Rect) -> Option<u16> {
        let timeline_start = self.config.timeline.viewport_start;
        let timeline_end = self.config.timeline.viewport_end;

        if date < timeline_start || date > timeline_end {
            return None;
        }

        let total_duration = (timeline_end - timeline_start).num_days() as f64;
        let date_offset = (date - timeline_start).num_days() as f64;
        let ratio = date_offset / total_duration;

        let chart_width = area.width.saturating_sub(20) as f64; // Reserve space for labels
        let x = (ratio * chart_width) as u16 + 20; // Offset for labels

        Some(x.min(area.width.saturating_sub(1)))
    }

    /// Render the time axis
    fn render_time_axis(&self, area: Rect, buf: &mut Buffer) {
        let axis_y = area.y;

        // Render axis line
        for x in area.x + 20..area.right() {
            buf.get_mut(x, axis_y + 1).set_symbol("─");
        }

        // Calculate tick interval based on zoom level
        let tick_interval = match self.config.timeline.zoom_level {
            ZoomLevel::Hours => Duration::hours(6), // Show every 6 hours
            ZoomLevel::Days => Duration::days(1),
            ZoomLevel::Weeks => Duration::weeks(1),
            ZoomLevel::Months => Duration::days(30),
        };

        let mut current = self.config.timeline.viewport_start;
        while current <= self.config.timeline.viewport_end {
            if let Some(x) = self.date_to_x(current, area) {
                if x >= area.x + 20 && x < area.right() {
                    // Render tick mark
                    buf.get_mut(x, axis_y + 1).set_symbol("┬");

                    // Render date label
                    let label = match self.config.timeline.zoom_level {
                        ZoomLevel::Hours => format!("{}h", current.hour()),
                        ZoomLevel::Days => format!("{}/{}", current.month(), current.day()),
                        ZoomLevel::Weeks => format!("W{}", current.iso_week().week()),
                        ZoomLevel::Months => format!("{}/{}", current.year(), current.month()),
                    };

                    let label_x = x.saturating_sub(label.len() as u16 / 2);
                    if label_x >= area.x + 20 && label_x + label.len() as u16 <= area.right() {
                        for (i, ch) in label.chars().enumerate() {
                            buf.get_mut(label_x + i as u16, axis_y)
                                .set_char(ch)
                                .set_style(Style::default().fg(Color::Gray));
                        }
                    }
                }
            }

            current += tick_interval;
        }

        // Render today marker if enabled
        if self.config.show_today {
            let today = Utc::now();
            if let Some(x) = self.date_to_x(today, area) {
                if x >= area.x + 20 && x < area.right() {
                    for y in area.y..area.bottom() {
                        buf.get_mut(x, y)
                            .set_symbol("│")
                            .set_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                    }
                }
            }
        }
    }

    /// Truncate text to fit width
    fn truncate_text(text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            text.to_string()
        } else if max_width <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &text[..max_width - 3])
        }
    }

    /// Render swim lanes with issue bars
    fn render_lanes(&self, area: Rect, buf: &mut Buffer, lanes: &[SwimLane]) {
        let mut current_y = area.y + 2; // Start after time axis

        for lane in lanes {
            if current_y + self.config.lane_height > area.bottom() {
                break; // Out of space
            }

            // Render lane label
            let lane_label = format!("{} ({})", lane.name, lane.schedules.len());
            let truncated_label = Self::truncate_text(&lane_label, 18);
            for (i, ch) in truncated_label.chars().enumerate() {
                let x_pos = area.x + (i as u16);
                let max_x = area.x + 19;
                if x_pos < max_x {
                    buf.get_mut(x_pos, current_y)
                        .set_char(ch)
                        .set_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
                }
            }

            // Render lane separator
            if self.config.show_grid {
                for x in area.x..area.right() {
                    buf.get_mut(x, current_y + self.config.lane_height - 1)
                        .set_symbol("─")
                        .set_style(Style::default().fg(Color::DarkGray));
                }
            }

            // Render issue bars in this lane
            for schedule in &lane.schedules {
                if let (Some(start), Some(end)) = (schedule.start, schedule.end) {
                    if let (Some(start_x), Some(end_x)) =
                        (self.date_to_x(start, area), self.date_to_x(end, area))
                    {
                        let bar_y = current_y + 1;
                        let bar_width = end_x.saturating_sub(start_x).max(1);

                        // Determine style based on selection and overdue status
                        let is_selected = self.selected_id.as_ref() == Some(&schedule.issue_id);
                        let is_overdue = end < Utc::now()
                            && self
                                .issues
                                .get(&schedule.issue_id)
                                .map(|i| {
                                    i.status != crate::beads::models::IssueStatus::Closed
                                })
                                .unwrap_or(false);

                        let bar_style = if is_selected {
                            self.config.selected_style
                        } else if is_overdue {
                            self.config.overdue_style
                        } else {
                            self.config.normal_style
                        };

                        // Render bar background
                        for x in start_x..start_x + bar_width {
                            if x >= area.x + 20 && x < area.right() && bar_y < area.bottom() {
                                buf.get_mut(x, bar_y)
                                    .set_symbol("█")
                                    .set_style(bar_style);
                            }
                        }

                        // Render issue ID and title on the bar
                        if let Some(issue) = self.issues.get(&schedule.issue_id) {
                            let bar_text = format!("{} {}", issue.id, issue.title);
                            let max_text_width = bar_width.saturating_sub(2) as usize;
                            let truncated = Self::truncate_text(&bar_text, max_text_width);

                            for (i, ch) in truncated.chars().enumerate() {
                                let text_x = start_x + 1 + i as u16;
                                if text_x >= area.x + 20 && text_x < area.right() && bar_y < area.bottom() {
                                    buf.get_mut(text_x, bar_y)
                                        .set_char(ch)
                                        .set_style(
                                            bar_style
                                                .fg(Color::Black)
                                                .add_modifier(Modifier::BOLD),
                                        );
                                }
                            }
                        }
                    }
                }
            }

            current_y += self.config.lane_height;
        }
    }
}

impl<'a> Widget for GanttChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Group schedules into swim lanes
        let lanes = self.group_into_lanes();

        // Render time axis
        self.render_time_axis(area, buf);

        // Render swim lanes
        self.render_lanes(area, buf, &lanes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use crate::models::gantt_schedule::ScheduleData;

    fn create_test_issue(id: &str, title: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        }
    }

    fn create_test_schedule(id: &str, days_from_now: i64) -> IssueSchedule {
        let start = Utc::now() + Duration::days(days_from_now);
        let end = start + Duration::days(5);

        IssueSchedule {
            issue_id: id.to_string(),
            start: Some(start),
            end: Some(end),
            is_scheduled: true,
            schedule_data: ScheduleData::default(),
        }
    }

    #[test]
    fn test_gantt_chart_creation() {
        let issue = create_test_issue("TEST-1", "Test Issue");
        let schedule = create_test_schedule("TEST-1", 0);

        let chart = GanttChart::new(vec![schedule], vec![&issue]);
        assert_eq!(chart.schedules.len(), 1);
        assert_eq!(chart.issues.len(), 1);
    }

    #[test]
    fn test_grouping_mode_default() {
        assert_eq!(GroupingMode::default(), GroupingMode::Status);
    }

    #[test]
    fn test_config_builder() {
        let config = GanttChartConfig::new()
            .grouping(GroupingMode::Priority)
            .lane_height(5);

        assert_eq!(config.grouping, GroupingMode::Priority);
        assert_eq!(config.lane_height, 5);
    }

    #[test]
    fn test_lane_height_minimum() {
        let config = GanttChartConfig::new().lane_height(1);
        assert_eq!(config.lane_height, 2); // Minimum is 2
    }

    #[test]
    fn test_truncate_text_short() {
        assert_eq!(GanttChart::truncate_text("short", 10), "short");
    }

    #[test]
    fn test_truncate_text_long() {
        assert_eq!(
            GanttChart::truncate_text("This is a very long text", 10),
            "This is..."
        );
    }

    #[test]
    fn test_truncate_text_very_short_width() {
        assert_eq!(GanttChart::truncate_text("test", 2), "...");
    }

    #[test]
    fn test_group_into_lanes_by_status() {
        let issue1 = create_test_issue("TEST-1", "Open Issue");
        let mut issue2 = create_test_issue("TEST-2", "In Progress Issue");
        issue2.status = IssueStatus::InProgress;

        let schedule1 = create_test_schedule("TEST-1", 0);
        let schedule2 = create_test_schedule("TEST-2", 5);

        let chart = GanttChart::new(vec![schedule1, schedule2], vec![&issue1, &issue2])
            .config(GanttChartConfig::new().grouping(GroupingMode::Status));

        let lanes = chart.group_into_lanes();
        assert_eq!(lanes.len(), 2); // "Open" and "In Progress"
    }

    #[test]
    fn test_group_into_lanes_by_priority() {
        let issue1 = create_test_issue("TEST-1", "High Priority");
        let mut issue2 = create_test_issue("TEST-2", "Low Priority");
        issue2.priority = Priority::P3;

        let schedule1 = create_test_schedule("TEST-1", 0);
        let schedule2 = create_test_schedule("TEST-2", 5);

        let chart = GanttChart::new(vec![schedule1, schedule2], vec![&issue1, &issue2])
            .config(GanttChartConfig::new().grouping(GroupingMode::Priority));

        let lanes = chart.group_into_lanes();
        assert_eq!(lanes.len(), 2); // "P2 - Medium" and "P3 - Low"
    }

    #[test]
    fn test_group_into_lanes_none() {
        let issue1 = create_test_issue("TEST-1", "Issue 1");
        let issue2 = create_test_issue("TEST-2", "Issue 2");

        let schedule1 = create_test_schedule("TEST-1", 0);
        let schedule2 = create_test_schedule("TEST-2", 5);

        let chart = GanttChart::new(vec![schedule1, schedule2], vec![&issue1, &issue2])
            .config(GanttChartConfig::new().grouping(GroupingMode::None));

        let lanes = chart.group_into_lanes();
        assert_eq!(lanes.len(), 1); // "All Issues"
        assert_eq!(lanes[0].schedules.len(), 2);
    }
}
