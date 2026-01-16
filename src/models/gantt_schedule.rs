//! Gantt chart schedule model with date derivation and timeline computation

use crate::beads::models::Issue;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Time estimate for an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeEstimate {
    /// Hours
    Hours(u32),
    /// Days
    Days(u32),
    /// Weeks
    Weeks(u32),
}

impl TimeEstimate {
    /// Convert estimate to duration
    pub fn to_duration(&self) -> Duration {
        match self {
            TimeEstimate::Hours(h) => Duration::hours(*h as i64),
            TimeEstimate::Days(d) => Duration::days(*d as i64),
            TimeEstimate::Weeks(w) => Duration::weeks(*w as i64),
        }
    }

    /// Convert estimate to hours
    pub fn to_hours(&self) -> u32 {
        match self {
            TimeEstimate::Hours(h) => *h,
            TimeEstimate::Days(d) => d * 8, // Assuming 8-hour workday
            TimeEstimate::Weeks(w) => w * 40, // Assuming 40-hour workweek
        }
    }
}

/// Schedule data for an issue (future extension of Issue model)
/// These fields will eventually be added to the Issue model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleData {
    /// Earliest date work can start (deferred until)
    pub defer_until: Option<DateTime<Utc>>,
    /// Target completion date (due date)
    pub due_at: Option<DateTime<Utc>>,
    /// Estimated effort required
    pub estimate: Option<TimeEstimate>,
}

/// Computed schedule for an issue in Gantt chart
#[derive(Debug, Clone)]
pub struct IssueSchedule {
    /// Issue ID
    pub issue_id: String,
    /// Computed start date
    pub start: Option<DateTime<Utc>>,
    /// Computed end date
    pub end: Option<DateTime<Utc>>,
    /// Whether the issue is scheduled (has both start and end)
    pub is_scheduled: bool,
    /// Original schedule data
    pub schedule_data: ScheduleData,
}

impl IssueSchedule {
    /// Create a schedule from an issue and schedule data
    ///
    /// Date derivation precedence:
    /// 1. Start date:
    ///    - Use defer_until if available
    ///    - Otherwise use created date
    /// 2. End date:
    ///    - Use due_at if available
    ///    - Otherwise compute from start + estimate
    ///    - If no estimate, end = start + 1 day (minimum task duration)
    pub fn from_issue(issue: &Issue, schedule_data: ScheduleData) -> Self {
        let (start, end) = Self::derive_dates(issue, &schedule_data);
        let is_scheduled = start.is_some() && end.is_some();

        Self {
            issue_id: issue.id.clone(),
            start,
            end,
            is_scheduled,
            schedule_data,
        }
    }

    /// Derive start and end dates based on precedence rules
    fn derive_dates(
        issue: &Issue,
        schedule_data: &ScheduleData,
    ) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        // Determine start date
        let start = schedule_data.defer_until.or(Some(issue.created));

        // Determine end date
        let end = if let Some(due_at) = schedule_data.due_at {
            // Use explicit due date
            Some(due_at)
        } else if let (Some(start_date), Some(estimate)) = (start, &schedule_data.estimate) {
            // Compute from start + estimate
            Some(start_date + estimate.to_duration())
        } else {
            start.map(|start_date| start_date + Duration::days(1))
        };

        (start, end)
    }

    /// Get the duration of the task in days
    pub fn duration_days(&self) -> Option<i64> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some((end - start).num_days()),
            _ => None,
        }
    }

    /// Get the duration of the task in hours
    pub fn duration_hours(&self) -> Option<i64> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some((end - start).num_hours()),
            _ => None,
        }
    }

    /// Check if the task is overdue
    pub fn is_overdue(&self, now: DateTime<Utc>) -> bool {
        match self.end {
            Some(end) => end < now,
            None => false,
        }
    }

    /// Check if the task is in progress (started but not ended)
    pub fn is_in_progress(&self, now: DateTime<Utc>) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => start <= now && now < end,
            _ => false,
        }
    }

    /// Check if the task is upcoming (not yet started)
    pub fn is_upcoming(&self, now: DateTime<Utc>) -> bool {
        match self.start {
            Some(start) => start > now,
            None => false,
        }
    }
}

/// Zoom level for Gantt chart timeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ZoomLevel {
    /// Show hours (for short-term planning)
    Hours,
    /// Show days (default view)
    #[default]
    Days,
    /// Show weeks (for medium-term planning)
    Weeks,
    /// Show months (for long-term planning)
    Months,
}

impl ZoomLevel {
    /// Get the duration represented by one unit at this zoom level
    pub fn unit_duration(&self) -> Duration {
        match self {
            ZoomLevel::Hours => Duration::hours(1),
            ZoomLevel::Days => Duration::days(1),
            ZoomLevel::Weeks => Duration::weeks(1),
            ZoomLevel::Months => Duration::days(30), // Approximate
        }
    }

    /// Get the label format for dates at this zoom level
    pub fn date_format(&self) -> &'static str {
        match self {
            ZoomLevel::Hours => "%H:%M",
            ZoomLevel::Days => "%b %d",
            ZoomLevel::Weeks => "W%V %Y",
            ZoomLevel::Months => "%b %Y",
        }
    }

    /// Compute the span in units for a date range at this zoom level
    pub fn compute_span(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
        let duration = end - start;
        match self {
            ZoomLevel::Hours => duration.num_hours(),
            ZoomLevel::Days => duration.num_days(),
            ZoomLevel::Weeks => duration.num_weeks(),
            ZoomLevel::Months => duration.num_days() / 30, // Approximate
        }
    }

    /// Format a date according to the zoom level
    pub fn format_date(&self, date: DateTime<Utc>) -> String {
        date.format(self.date_format()).to_string()
    }
}

/// Timeline configuration for Gantt chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineConfig {
    /// Current zoom level
    pub zoom_level: ZoomLevel,
    /// Start date of visible timeline
    pub viewport_start: DateTime<Utc>,
    /// End date of visible timeline
    pub viewport_end: DateTime<Utc>,
}

impl TimelineConfig {
    /// Create a new timeline configuration
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            zoom_level: ZoomLevel::default(),
            viewport_start: start,
            viewport_end: end,
        }
    }

    /// Get the number of units visible in the current viewport
    pub fn visible_units(&self) -> i64 {
        self.zoom_level
            .compute_span(self.viewport_start, self.viewport_end)
    }

    /// Zoom in (show more detail)
    pub fn zoom_in(&mut self) {
        self.zoom_level = match self.zoom_level {
            ZoomLevel::Months => ZoomLevel::Weeks,
            ZoomLevel::Weeks => ZoomLevel::Days,
            ZoomLevel::Days => ZoomLevel::Hours,
            ZoomLevel::Hours => ZoomLevel::Hours, // Already at max zoom
        };
    }

    /// Zoom out (show less detail)
    pub fn zoom_out(&mut self) {
        self.zoom_level = match self.zoom_level {
            ZoomLevel::Hours => ZoomLevel::Days,
            ZoomLevel::Days => ZoomLevel::Weeks,
            ZoomLevel::Weeks => ZoomLevel::Months,
            ZoomLevel::Months => ZoomLevel::Months, // Already at min zoom
        };
    }

    /// Pan the viewport forward in time
    pub fn pan_forward(&mut self) {
        let delta = self.zoom_level.unit_duration() * 5; // Pan by 5 units
        self.viewport_start += delta;
        self.viewport_end += delta;
    }

    /// Pan the viewport backward in time
    pub fn pan_backward(&mut self) {
        let delta = self.zoom_level.unit_duration() * 5; // Pan by 5 units
        self.viewport_start -= delta;
        self.viewport_end -= delta;
    }

    /// Fit the timeline to show all scheduled issues
    pub fn fit_to_schedules(&mut self, schedules: &[IssueSchedule]) {
        let scheduled: Vec<&IssueSchedule> = schedules.iter().filter(|s| s.is_scheduled).collect();

        if scheduled.is_empty() {
            return;
        }

        let min_start = scheduled
            .iter()
            .filter_map(|s| s.start)
            .min()
            .unwrap_or_else(Utc::now);

        let max_end = scheduled
            .iter()
            .filter_map(|s| s.end)
            .max()
            .unwrap_or_else(|| Utc::now() + Duration::days(30));

        // Add some padding
        let padding = (max_end - min_start) / 10;
        self.viewport_start = min_start - padding;
        self.viewport_end = max_end + padding;
    }
}

impl Default for TimelineConfig {
    fn default() -> Self {
        let now = Utc::now();
        Self::new(now - Duration::days(30), now + Duration::days(90))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};

    fn create_test_issue(id: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: "Test Issue".to_string(),
            description: None,
            issue_type: IssueType::Task,
            status: IssueStatus::Open,
            priority: Priority::P2,
            labels: vec![],
            assignee: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_time_estimate_to_duration() {
        assert_eq!(TimeEstimate::Hours(8).to_duration(), Duration::hours(8));
        assert_eq!(TimeEstimate::Days(2).to_duration(), Duration::days(2));
        assert_eq!(TimeEstimate::Weeks(1).to_duration(), Duration::weeks(1));
    }

    #[test]
    fn test_time_estimate_to_hours() {
        assert_eq!(TimeEstimate::Hours(8).to_hours(), 8);
        assert_eq!(TimeEstimate::Days(2).to_hours(), 16);
        assert_eq!(TimeEstimate::Weeks(1).to_hours(), 40);
    }

    #[test]
    fn test_issue_schedule_with_explicit_dates() {
        let issue = create_test_issue("test-001");
        let start = Utc::now();
        let end = start + Duration::days(5);

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(end),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert!(schedule.is_scheduled);
        assert_eq!(schedule.start, Some(start));
        assert_eq!(schedule.end, Some(end));
        assert_eq!(schedule.duration_days(), Some(5));
    }

    #[test]
    fn test_issue_schedule_with_estimate() {
        let issue = create_test_issue("test-002");
        let start = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: None,
            estimate: Some(TimeEstimate::Days(3)),
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert!(schedule.is_scheduled);
        assert_eq!(schedule.start, Some(start));
        assert_eq!(schedule.end, Some(start + Duration::days(3)));
        assert_eq!(schedule.duration_days(), Some(3));
    }

    #[test]
    fn test_issue_schedule_fallback_to_created() {
        let issue = create_test_issue("test-003");
        let schedule_data = ScheduleData::default();

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // Should use created date as start, and created + 1 day as end
        assert!(schedule.is_scheduled);
        assert_eq!(schedule.start, Some(issue.created));
        assert_eq!(schedule.end, Some(issue.created + Duration::days(1)));
    }

    #[test]
    fn test_issue_schedule_unscheduled() {
        let issue = create_test_issue("test-004");
        let schedule_data = ScheduleData {
            defer_until: None,
            due_at: None,
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data.clone());

        // Should still have dates (fallback to created)
        assert!(schedule.is_scheduled);
        assert_eq!(schedule.start, Some(issue.created));
    }

    #[test]
    fn test_issue_schedule_is_overdue() {
        let issue = create_test_issue("test-005");
        let past = Utc::now() - Duration::days(10);

        let schedule_data = ScheduleData {
            defer_until: Some(past),
            due_at: Some(past + Duration::days(5)),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert!(schedule.is_overdue(Utc::now()));
        assert!(!schedule.is_upcoming(Utc::now()));
    }

    #[test]
    fn test_issue_schedule_is_upcoming() {
        let issue = create_test_issue("test-006");
        let future = Utc::now() + Duration::days(10);

        let schedule_data = ScheduleData {
            defer_until: Some(future),
            due_at: Some(future + Duration::days(5)),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert!(!schedule.is_overdue(Utc::now()));
        assert!(schedule.is_upcoming(Utc::now()));
    }

    #[test]
    fn test_issue_schedule_is_in_progress() {
        let issue = create_test_issue("test-007");
        let past = Utc::now() - Duration::days(2);
        let future = Utc::now() + Duration::days(3);

        let schedule_data = ScheduleData {
            defer_until: Some(past),
            due_at: Some(future),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert!(schedule.is_in_progress(Utc::now()));
        assert!(!schedule.is_overdue(Utc::now()));
        assert!(!schedule.is_upcoming(Utc::now()));
    }

    #[test]
    fn test_zoom_level_unit_duration() {
        assert_eq!(ZoomLevel::Hours.unit_duration(), Duration::hours(1));
        assert_eq!(ZoomLevel::Days.unit_duration(), Duration::days(1));
        assert_eq!(ZoomLevel::Weeks.unit_duration(), Duration::weeks(1));
        assert_eq!(ZoomLevel::Months.unit_duration(), Duration::days(30));
    }

    #[test]
    fn test_zoom_level_compute_span() {
        let start = Utc::now();
        let end = start + Duration::days(10);

        assert_eq!(ZoomLevel::Hours.compute_span(start, end), 240); // 10 days * 24 hours
        assert_eq!(ZoomLevel::Days.compute_span(start, end), 10);
        assert_eq!(ZoomLevel::Weeks.compute_span(start, end), 1); // 10 days / 7
    }

    #[test]
    fn test_timeline_config_zoom() {
        let mut config = TimelineConfig::default();

        assert_eq!(config.zoom_level, ZoomLevel::Days);

        config.zoom_in();
        assert_eq!(config.zoom_level, ZoomLevel::Hours);

        config.zoom_in(); // Should stay at Hours
        assert_eq!(config.zoom_level, ZoomLevel::Hours);

        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Days);

        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Weeks);

        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Months);

        config.zoom_out(); // Should stay at Months
        assert_eq!(config.zoom_level, ZoomLevel::Months);
    }

    #[test]
    fn test_timeline_config_pan() {
        let start = Utc::now();
        let end = start + Duration::days(30);
        let mut config = TimelineConfig::new(start, end);

        let original_start = config.viewport_start;

        config.pan_forward();
        assert!(config.viewport_start > original_start);

        config.pan_backward();
        assert_eq!(config.viewport_start, original_start);
    }

    #[test]
    fn test_timeline_config_fit_to_schedules() {
        let issue1 = create_test_issue("test-001");
        let issue2 = create_test_issue("test-002");

        let start1 = Utc::now();
        let end1 = start1 + Duration::days(10);

        let start2 = start1 + Duration::days(5);
        let end2 = start2 + Duration::days(20);

        let schedule1 = IssueSchedule::from_issue(
            &issue1,
            ScheduleData {
                defer_until: Some(start1),
                due_at: Some(end1),
                estimate: None,
            },
        );

        let schedule2 = IssueSchedule::from_issue(
            &issue2,
            ScheduleData {
                defer_until: Some(start2),
                due_at: Some(end2),
                estimate: None,
            },
        );

        let mut config = TimelineConfig::default();
        config.fit_to_schedules(&[schedule1, schedule2]);

        // Should include both schedules with padding
        assert!(config.viewport_start < start1);
        assert!(config.viewport_end > end2);
    }

    #[test]
    fn test_time_estimate_clone() {
        let estimate = TimeEstimate::Days(5);
        let cloned = estimate;
        assert_eq!(estimate, cloned);
    }

    #[test]
    fn test_time_estimate_equality() {
        assert_eq!(TimeEstimate::Hours(24), TimeEstimate::Hours(24));
        assert_ne!(TimeEstimate::Hours(24), TimeEstimate::Days(1));
        assert_eq!(TimeEstimate::Weeks(2), TimeEstimate::Weeks(2));
    }

    #[test]
    fn test_time_estimate_zero_values() {
        assert_eq!(TimeEstimate::Hours(0).to_hours(), 0);
        assert_eq!(TimeEstimate::Days(0).to_hours(), 0);
        assert_eq!(TimeEstimate::Weeks(0).to_hours(), 0);
    }

    #[test]
    fn test_time_estimate_large_values() {
        assert_eq!(TimeEstimate::Hours(1000).to_hours(), 1000);
        assert_eq!(TimeEstimate::Days(100).to_hours(), 800); // 100 * 8
        assert_eq!(TimeEstimate::Weeks(10).to_hours(), 400); // 10 * 40
    }

    #[test]
    fn test_schedule_data_default() {
        let data = ScheduleData::default();
        assert!(data.defer_until.is_none());
        assert!(data.due_at.is_none());
        assert!(data.estimate.is_none());
    }

    #[test]
    fn test_schedule_data_clone() {
        let now = Utc::now();
        let data = ScheduleData {
            defer_until: Some(now),
            due_at: Some(now + Duration::days(5)),
            estimate: Some(TimeEstimate::Days(3)),
        };

        let cloned = data.clone();
        assert_eq!(cloned.defer_until, data.defer_until);
        assert_eq!(cloned.due_at, data.due_at);
        assert_eq!(cloned.estimate, data.estimate);
    }

    #[test]
    fn test_schedule_data_partial_fields() {
        let now = Utc::now();

        // Only defer_until
        let data1 = ScheduleData {
            defer_until: Some(now),
            due_at: None,
            estimate: None,
        };
        assert!(data1.defer_until.is_some());
        assert!(data1.due_at.is_none());

        // Only due_at
        let data2 = ScheduleData {
            defer_until: None,
            due_at: Some(now),
            estimate: None,
        };
        assert!(data2.defer_until.is_none());
        assert!(data2.due_at.is_some());

        // Only estimate
        let data3 = ScheduleData {
            defer_until: None,
            due_at: None,
            estimate: Some(TimeEstimate::Hours(8)),
        };
        assert!(data3.estimate.is_some());
    }

    #[test]
    fn test_issue_schedule_clone() {
        let issue = create_test_issue("test-clone");
        let schedule = IssueSchedule::from_issue(&issue, ScheduleData::default());

        let cloned = schedule.clone();
        assert_eq!(cloned.issue_id, schedule.issue_id);
        assert_eq!(cloned.start, schedule.start);
        assert_eq!(cloned.end, schedule.end);
        assert_eq!(cloned.is_scheduled, schedule.is_scheduled);
    }

    #[test]
    fn test_issue_schedule_duration_hours() {
        let issue = create_test_issue("test-hours");
        let start = Utc::now();
        let end = start + Duration::hours(12);

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(end),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);
        assert_eq!(schedule.duration_hours(), Some(12));
    }

    #[test]
    fn test_issue_schedule_duration_none_when_no_dates() {
        let issue = create_test_issue("test-no-dates");
        // Create schedule with no dates at all (should still fallback to created)
        let schedule_data = ScheduleData::default();
        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // Should have dates from fallback
        assert!(schedule.duration_days().is_some());
        assert!(schedule.duration_hours().is_some());
    }

    #[test]
    fn test_issue_schedule_is_overdue_with_no_end() {
        let issue = create_test_issue("test-no-end");
        // Create a schedule with start but impossible to have no end due to fallback
        // So test with schedule having end in the future
        let future = Utc::now() + Duration::days(10);
        let schedule_data = ScheduleData {
            defer_until: Some(future),
            due_at: Some(future + Duration::days(5)),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);
        assert!(!schedule.is_overdue(Utc::now()));
    }

    #[test]
    fn test_issue_schedule_is_upcoming_with_no_start() {
        let issue = create_test_issue("test-no-start");
        // With fallback, start will always be Some(created), test can't have None
        let schedule_data = ScheduleData::default();
        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // Should not be upcoming (start is now/past)
        assert!(!schedule.is_upcoming(Utc::now()));
    }

    #[test]
    fn test_issue_schedule_is_in_progress_boundary() {
        let issue = create_test_issue("test-boundary");
        let now = Utc::now();
        let start = now;
        let end = now + Duration::hours(1);

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(end),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);
        // At exact start time, should be in progress
        assert!(schedule.is_in_progress(now));
    }

    #[test]
    fn test_zoom_level_clone() {
        let level = ZoomLevel::Days;
        let cloned = level;
        assert_eq!(level, cloned);
    }

    #[test]
    fn test_zoom_level_default() {
        let level = ZoomLevel::default();
        assert_eq!(level, ZoomLevel::Days);
    }

    #[test]
    fn test_zoom_level_equality() {
        assert_eq!(ZoomLevel::Hours, ZoomLevel::Hours);
        assert_ne!(ZoomLevel::Hours, ZoomLevel::Days);
        assert_eq!(ZoomLevel::Weeks, ZoomLevel::Weeks);
    }

    #[test]
    fn test_zoom_level_date_format() {
        assert_eq!(ZoomLevel::Hours.date_format(), "%H:%M");
        assert_eq!(ZoomLevel::Days.date_format(), "%b %d");
        assert_eq!(ZoomLevel::Weeks.date_format(), "W%V %Y");
        assert_eq!(ZoomLevel::Months.date_format(), "%b %Y");
    }

    #[test]
    fn test_zoom_level_format_date() {
        let date = Utc::now();

        // Just verify they don't panic and return strings
        let hours_str = ZoomLevel::Hours.format_date(date);
        let days_str = ZoomLevel::Days.format_date(date);
        let weeks_str = ZoomLevel::Weeks.format_date(date);
        let months_str = ZoomLevel::Months.format_date(date);

        assert!(!hours_str.is_empty());
        assert!(!days_str.is_empty());
        assert!(!weeks_str.is_empty());
        assert!(!months_str.is_empty());
    }

    #[test]
    fn test_zoom_level_compute_span_zero_duration() {
        let now = Utc::now();

        assert_eq!(ZoomLevel::Hours.compute_span(now, now), 0);
        assert_eq!(ZoomLevel::Days.compute_span(now, now), 0);
        assert_eq!(ZoomLevel::Weeks.compute_span(now, now), 0);
        assert_eq!(ZoomLevel::Months.compute_span(now, now), 0);
    }

    #[test]
    fn test_timeline_config_new() {
        let start = Utc::now();
        let end = start + Duration::days(30);
        let config = TimelineConfig::new(start, end);

        assert_eq!(config.zoom_level, ZoomLevel::Days);
        assert_eq!(config.viewport_start, start);
        assert_eq!(config.viewport_end, end);
    }

    #[test]
    fn test_timeline_config_default() {
        let config = TimelineConfig::default();

        assert_eq!(config.zoom_level, ZoomLevel::Days);
        assert!(config.viewport_end > config.viewport_start);
    }

    #[test]
    fn test_timeline_config_clone() {
        let start = Utc::now();
        let end = start + Duration::days(30);
        let config = TimelineConfig::new(start, end);

        let cloned = config.clone();
        assert_eq!(cloned.zoom_level, config.zoom_level);
        assert_eq!(cloned.viewport_start, config.viewport_start);
        assert_eq!(cloned.viewport_end, config.viewport_end);
    }

    #[test]
    fn test_timeline_config_visible_units() {
        let start = Utc::now();
        let end = start + Duration::days(10);
        let mut config = TimelineConfig::new(start, end);

        config.zoom_level = ZoomLevel::Days;
        assert_eq!(config.visible_units(), 10);

        config.zoom_level = ZoomLevel::Hours;
        assert_eq!(config.visible_units(), 240); // 10 * 24
    }

    #[test]
    fn test_timeline_config_fit_to_schedules_empty() {
        let mut config = TimelineConfig::default();
        let original_start = config.viewport_start;
        let original_end = config.viewport_end;

        config.fit_to_schedules(&[]);

        // Should not change when empty
        assert_eq!(config.viewport_start, original_start);
        assert_eq!(config.viewport_end, original_end);
    }

    #[test]
    fn test_timeline_config_fit_to_schedules_single() {
        let issue = create_test_issue("test-single");
        let start = Utc::now();
        let end = start + Duration::days(10);

        let schedule = IssueSchedule::from_issue(
            &issue,
            ScheduleData {
                defer_until: Some(start),
                due_at: Some(end),
                estimate: None,
            },
        );

        let mut config = TimelineConfig::default();
        config.fit_to_schedules(&[schedule]);

        // Should include the schedule with padding
        assert!(config.viewport_start < start);
        assert!(config.viewport_end > end);
    }

    #[test]
    fn test_timeline_config_pan_multiple_times() {
        let start = Utc::now();
        let end = start + Duration::days(30);
        let mut config = TimelineConfig::new(start, end);

        let original_start = config.viewport_start;

        config.pan_forward();
        let after_first = config.viewport_start;
        assert!(after_first > original_start);

        config.pan_forward();
        let after_second = config.viewport_start;
        assert!(after_second > after_first);

        config.pan_backward();
        config.pan_backward();
        assert_eq!(config.viewport_start, original_start);
    }

    #[test]
    fn test_time_estimate_debug_trait() {
        let hours = TimeEstimate::Hours(8);
        let debug_str = format!("{:?}", hours);
        assert!(debug_str.contains("Hours"));

        let days = TimeEstimate::Days(5);
        let debug_str = format!("{:?}", days);
        assert!(debug_str.contains("Days"));

        let weeks = TimeEstimate::Weeks(2);
        let debug_str = format!("{:?}", weeks);
        assert!(debug_str.contains("Weeks"));
    }

    #[test]
    fn test_schedule_data_debug_trait() {
        let data = ScheduleData::default();
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("ScheduleData"));
    }

    #[test]
    fn test_issue_schedule_debug_trait() {
        let issue = create_test_issue("test-debug");
        let schedule = IssueSchedule::from_issue(&issue, ScheduleData::default());
        let debug_str = format!("{:?}", schedule);
        assert!(debug_str.contains("IssueSchedule"));
    }

    #[test]
    fn test_zoom_level_debug_trait() {
        let level = ZoomLevel::Days;
        let debug_str = format!("{:?}", level);
        assert!(debug_str.contains("Days"));
    }

    #[test]
    fn test_timeline_config_debug_trait() {
        let config = TimelineConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TimelineConfig"));
    }

    #[test]
    fn test_time_estimate_all_variants() {
        let hours = TimeEstimate::Hours(1);
        let days = TimeEstimate::Days(1);
        let weeks = TimeEstimate::Weeks(1);

        assert_eq!(hours.to_hours(), 1);
        assert_eq!(days.to_hours(), 8);
        assert_eq!(weeks.to_hours(), 40);
    }

    #[test]
    fn test_time_estimate_copy_trait() {
        let estimate = TimeEstimate::Days(5);
        let copied = estimate;
        assert_eq!(estimate, copied);

        // Can still use original after copy
        assert_eq!(estimate.to_hours(), 40);
    }

    #[test]
    fn test_schedule_data_all_combinations() {
        let now = Utc::now();

        // All fields present
        let data1 = ScheduleData {
            defer_until: Some(now),
            due_at: Some(now + Duration::days(5)),
            estimate: Some(TimeEstimate::Days(3)),
        };
        assert!(data1.defer_until.is_some());
        assert!(data1.due_at.is_some());
        assert!(data1.estimate.is_some());

        // Two fields
        let data2 = ScheduleData {
            defer_until: Some(now),
            due_at: Some(now + Duration::days(5)),
            estimate: None,
        };
        assert!(data2.defer_until.is_some());
        assert!(data2.due_at.is_some());

        // All fields None
        let data3 = ScheduleData::default();
        assert!(data3.defer_until.is_none());
        assert!(data3.due_at.is_none());
        assert!(data3.estimate.is_none());
    }

    #[test]
    fn test_issue_schedule_estimate_precedence_over_default() {
        let issue = create_test_issue("test-precedence");
        let start = Utc::now();

        // Estimate should be used to compute end
        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: None,
            estimate: Some(TimeEstimate::Weeks(2)),
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert_eq!(schedule.start, Some(start));
        assert_eq!(schedule.end, Some(start + Duration::weeks(2)));
        assert_eq!(schedule.duration_days(), Some(14));
    }

    #[test]
    fn test_issue_schedule_due_at_precedence_over_estimate() {
        let issue = create_test_issue("test-due-precedence");
        let start = Utc::now();
        let due = start + Duration::days(7);

        // due_at should take precedence over estimate
        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(due),
            estimate: Some(TimeEstimate::Days(10)), // Different from actual due
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert_eq!(schedule.start, Some(start));
        assert_eq!(schedule.end, Some(due)); // Uses due_at, not start + estimate
        assert_eq!(schedule.duration_days(), Some(7));
    }

    #[test]
    fn test_issue_schedule_is_overdue_at_exact_time() {
        let issue = create_test_issue("test-exact-overdue");
        let now = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(now - Duration::days(5)),
            due_at: Some(now), // Exactly now
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // At exact end time, should NOT be overdue (end < now is false)
        assert!(!schedule.is_overdue(now));

        // One second later, should be overdue
        let one_second_later = now + Duration::seconds(1);
        assert!(schedule.is_overdue(one_second_later));
    }

    #[test]
    fn test_issue_schedule_is_in_progress_at_end_boundary() {
        let issue = create_test_issue("test-end-boundary");
        let start = Utc::now() - Duration::days(5);
        let end = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(end),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // At exact end time, should NOT be in progress (now < end is false)
        assert!(!schedule.is_in_progress(end));

        // Just before end, should be in progress
        let just_before = end - Duration::seconds(1);
        assert!(schedule.is_in_progress(just_before));
    }

    #[test]
    fn test_issue_schedule_is_upcoming_at_exact_start() {
        let issue = create_test_issue("test-start-upcoming");
        let start = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(start + Duration::days(5)),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // At exact start time, should NOT be upcoming (start > now is false)
        assert!(!schedule.is_upcoming(start));

        // Just before start, should be upcoming
        let just_before = start - Duration::seconds(1);
        assert!(schedule.is_upcoming(just_before));
    }

    #[test]
    fn test_zoom_level_all_variants_coverage() {
        let variants = vec![
            ZoomLevel::Hours,
            ZoomLevel::Days,
            ZoomLevel::Weeks,
            ZoomLevel::Months,
        ];

        for variant in variants {
            // Ensure all variants can be created and used
            let _ = variant.unit_duration();
            let _ = variant.date_format();
            let _ = variant;
        }
    }

    #[test]
    fn test_zoom_level_copy_trait() {
        let level = ZoomLevel::Weeks;
        let copied = level;
        assert_eq!(level, copied);

        // Can still use original after copy
        assert_eq!(level.unit_duration(), Duration::weeks(1));
    }

    #[test]
    fn test_timeline_config_pan_with_different_zoom_levels() {
        let start = Utc::now();
        let end = start + Duration::days(30);

        // Test pan at Hours zoom
        let mut config = TimelineConfig::new(start, end);
        config.zoom_level = ZoomLevel::Hours;
        let original = config.viewport_start;
        config.pan_forward();
        let delta_hours = (config.viewport_start - original).num_hours();
        assert_eq!(delta_hours, 5); // 5 hours

        // Test pan at Weeks zoom
        let mut config = TimelineConfig::new(start, end);
        config.zoom_level = ZoomLevel::Weeks;
        let original = config.viewport_start;
        config.pan_forward();
        let delta_weeks = (config.viewport_start - original).num_weeks();
        assert_eq!(delta_weeks, 5); // 5 weeks
    }

    #[test]
    fn test_timeline_config_zoom_from_each_level() {
        // Start from Hours
        let mut config = TimelineConfig {
            zoom_level: ZoomLevel::Hours,
            ..Default::default()
        };
        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Days);

        // Start from Days
        config.zoom_level = ZoomLevel::Days;
        config.zoom_in();
        assert_eq!(config.zoom_level, ZoomLevel::Hours);
        config.zoom_level = ZoomLevel::Days;
        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Weeks);

        // Start from Weeks
        config.zoom_level = ZoomLevel::Weeks;
        config.zoom_in();
        assert_eq!(config.zoom_level, ZoomLevel::Days);
        config.zoom_level = ZoomLevel::Weeks;
        config.zoom_out();
        assert_eq!(config.zoom_level, ZoomLevel::Months);

        // Start from Months
        config.zoom_level = ZoomLevel::Months;
        config.zoom_in();
        assert_eq!(config.zoom_level, ZoomLevel::Weeks);
    }

    #[test]
    fn test_timeline_config_fit_to_schedules_with_unscheduled() {
        let issue1 = create_test_issue("test-scheduled");
        let issue2 = create_test_issue("test-unscheduled");

        let start = Utc::now();
        let end = start + Duration::days(10);

        let schedule1 = IssueSchedule::from_issue(
            &issue1,
            ScheduleData {
                defer_until: Some(start),
                due_at: Some(end),
                estimate: None,
            },
        );

        // Create an unscheduled issue (though with fallback it will have dates)
        let schedule2 = IssueSchedule::from_issue(&issue2, ScheduleData::default());

        let mut config = TimelineConfig::default();
        config.fit_to_schedules(&[schedule1, schedule2]);

        // Should fit to include both schedules
        assert!(config.viewport_end > config.viewport_start);
    }

    #[test]
    fn test_zoom_level_compute_span_large_duration() {
        let start = Utc::now();
        let end = start + Duration::days(365); // One year

        assert_eq!(ZoomLevel::Days.compute_span(start, end), 365);
        assert_eq!(ZoomLevel::Weeks.compute_span(start, end), 52); // 365 / 7
        assert_eq!(ZoomLevel::Months.compute_span(start, end), 12); // 365 / 30
    }

    #[test]
    fn test_issue_schedule_negative_duration() {
        let issue = create_test_issue("test-negative");
        let start = Utc::now();
        let end = start - Duration::days(5); // End before start

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: Some(end),
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        // Should have negative duration
        assert_eq!(schedule.duration_days(), Some(-5));
        assert!(schedule.duration_hours().unwrap() < 0);
    }

    #[test]
    fn test_time_estimate_conversion_consistency() {
        // 1 week = 40 hours = 5 days (8-hour workday)
        let weeks = TimeEstimate::Weeks(1);
        let days = TimeEstimate::Days(5);
        let hours = TimeEstimate::Hours(40);

        assert_eq!(weeks.to_hours(), 40);
        assert_eq!(days.to_hours(), 40);
        assert_eq!(hours.to_hours(), 40);

        // Duration conversions
        assert_eq!(weeks.to_duration(), Duration::weeks(1));
        assert_eq!(days.to_duration(), Duration::days(5));
        assert_eq!(hours.to_duration(), Duration::hours(40));
    }

    #[test]
    fn test_timeline_config_viewport_span() {
        let start = Utc::now();
        let end = start + Duration::days(60);
        let config = TimelineConfig::new(start, end);

        // Verify viewport span
        let span_days = (config.viewport_end - config.viewport_start).num_days();
        assert_eq!(span_days, 60);
    }

    #[test]
    fn test_issue_schedule_with_hours_estimate() {
        let issue = create_test_issue("test-hours-estimate");
        let start = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(start),
            due_at: None,
            estimate: Some(TimeEstimate::Hours(16)),
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert_eq!(schedule.start, Some(start));
        assert_eq!(schedule.end, Some(start + Duration::hours(16)));
        assert_eq!(schedule.duration_hours(), Some(16));
    }

    #[test]
    fn test_zoom_level_format_date_consistency() {
        let date = Utc::now();

        // Verify all zoom levels can format dates without panicking
        for level in [
            ZoomLevel::Hours,
            ZoomLevel::Days,
            ZoomLevel::Weeks,
            ZoomLevel::Months,
        ] {
            let formatted = level.format_date(date);
            assert!(!formatted.is_empty());

            // Verify it matches the date_format
            let expected = date.format(level.date_format()).to_string();
            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn test_timeline_config_visible_units_all_zoom_levels() {
        let start = Utc::now();
        let end = start + Duration::days(7);

        let mut config = TimelineConfig::new(start, end);

        config.zoom_level = ZoomLevel::Hours;
        assert_eq!(config.visible_units(), 168); // 7 * 24

        config.zoom_level = ZoomLevel::Days;
        assert_eq!(config.visible_units(), 7);

        config.zoom_level = ZoomLevel::Weeks;
        assert_eq!(config.visible_units(), 1);

        config.zoom_level = ZoomLevel::Months;
        assert_eq!(config.visible_units(), 0); // 7 days / 30
    }

    #[test]
    fn test_issue_schedule_same_start_and_end() {
        let issue = create_test_issue("test-same-time");
        let now = Utc::now();

        let schedule_data = ScheduleData {
            defer_until: Some(now),
            due_at: Some(now), // Same as start
            estimate: None,
        };

        let schedule = IssueSchedule::from_issue(&issue, schedule_data);

        assert_eq!(schedule.duration_days(), Some(0));
        assert_eq!(schedule.duration_hours(), Some(0));
        assert!(!schedule.is_in_progress(now)); // Not in progress if duration is 0
    }
}
