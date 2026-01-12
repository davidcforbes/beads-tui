//! Date picker and date range selector widgets

use chrono::{Datelike, Duration, Local, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

/// Preset date range options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRangePreset {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    Custom,
}

impl DateRangePreset {
    /// Get all preset options
    pub fn all() -> Vec<Self> {
        vec![
            Self::Today,
            Self::Yesterday,
            Self::Last7Days,
            Self::Last30Days,
            Self::ThisWeek,
            Self::LastWeek,
            Self::ThisMonth,
            Self::LastMonth,
            Self::Custom,
        ]
    }

    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Today => "Today",
            Self::Yesterday => "Yesterday",
            Self::Last7Days => "Last 7 days",
            Self::Last30Days => "Last 30 days",
            Self::ThisWeek => "This week",
            Self::LastWeek => "Last week",
            Self::ThisMonth => "This month",
            Self::LastMonth => "Last month",
            Self::Custom => "Custom range...",
        }
    }

    /// Calculate date range for preset
    pub fn date_range(&self) -> Option<(NaiveDate, NaiveDate)> {
        let now = Local::now().naive_local().date();

        match self {
            Self::Today => Some((now, now)),
            Self::Yesterday => {
                let yesterday = now - Duration::days(1);
                Some((yesterday, yesterday))
            }
            Self::Last7Days => {
                let start = now - Duration::days(6);
                Some((start, now))
            }
            Self::Last30Days => {
                let start = now - Duration::days(29);
                Some((start, now))
            }
            Self::ThisWeek => {
                let weekday = now.weekday().num_days_from_monday();
                let start = now - Duration::days(weekday as i64);
                Some((start, now))
            }
            Self::LastWeek => {
                let weekday = now.weekday().num_days_from_monday();
                let this_week_start = now - Duration::days(weekday as i64);
                let last_week_end = this_week_start - Duration::days(1);
                let last_week_start = last_week_end - Duration::days(6);
                Some((last_week_start, last_week_end))
            }
            Self::ThisMonth => {
                let start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)?;
                Some((start, now))
            }
            Self::LastMonth => {
                let last_month_date = if now.month() == 1 {
                    NaiveDate::from_ymd_opt(now.year() - 1, 12, 1)?
                } else {
                    NaiveDate::from_ymd_opt(now.year(), now.month() - 1, 1)?
                };

                let this_month_start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)?;
                let last_month_end = this_month_start - Duration::days(1);

                Some((last_month_date, last_month_end))
            }
            Self::Custom => None,
        }
    }
}

/// Date range state
#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
}

impl DateRange {
    /// Create a new empty date range
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    /// Create a date range from start and end dates
    pub fn from_dates(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// Clear the date range
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }

    /// Check if a date is within the range
    pub fn contains(&self, date: &NaiveDate) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => *date >= start && *date <= end,
            (Some(start), None) => *date >= start,
            (None, Some(end)) => *date <= end,
            (None, None) => true,
        }
    }

    /// Get a formatted string representation
    pub fn format(&self) -> String {
        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                if start == end {
                    format!("{start}")
                } else {
                    format!("{start} to {end}")
                }
            }
            (Some(start), None) => format!("From {start}"),
            (None, Some(end)) => format!("Until {end}"),
            (None, None) => "Any date".to_string(),
        }
    }
}

impl Default for DateRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Date range picker state
#[derive(Debug, Clone)]
pub struct DateRangePickerState {
    preset: DateRangePreset,
    range: DateRange,
    list_state: ListState,
    is_custom_mode: bool,
}

impl Default for DateRangePickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl DateRangePickerState {
    /// Create a new date range picker state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            preset: DateRangePreset::Today,
            range: DateRange::new(),
            list_state,
            is_custom_mode: false,
        }
    }

    /// Get the current date range
    pub fn date_range(&self) -> &DateRange {
        &self.range
    }

    /// Set a custom date range
    pub fn set_date_range(&mut self, start: Option<NaiveDate>, end: Option<NaiveDate>) {
        self.range.start = start;
        self.range.end = end;
        self.preset = DateRangePreset::Custom;
        self.is_custom_mode = true;
    }

    /// Get the current preset
    pub fn preset(&self) -> DateRangePreset {
        self.preset
    }

    /// Select next preset
    pub fn select_next(&mut self) {
        let presets = DateRangePreset::all();
        let count = presets.len();

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

    /// Select previous preset
    pub fn select_previous(&mut self) {
        let presets = DateRangePreset::all();
        let count = presets.len();

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

    /// Apply the currently selected preset
    pub fn apply_selected(&mut self) {
        let presets = DateRangePreset::all();
        let Some(index) = self.list_state.selected() else {
            return;
        };

        if index >= presets.len() {
            return;
        }

        let preset = presets[index];
        self.preset = preset;

        if preset == DateRangePreset::Custom {
            self.is_custom_mode = true;
        } else {
            self.is_custom_mode = false;
            if let Some((start, end)) = preset.date_range() {
                self.range = DateRange::from_dates(start, end);
            }
        }
    }

    /// Clear the date range
    pub fn clear(&mut self) {
        self.range.clear();
        self.preset = DateRangePreset::Today;
        self.is_custom_mode = false;
        self.list_state.select(Some(0));
    }

    /// Check if in custom mode
    pub fn is_custom_mode(&self) -> bool {
        self.is_custom_mode
    }

    /// Set custom mode
    pub fn set_custom_mode(&mut self, enabled: bool) {
        self.is_custom_mode = enabled;
        if enabled {
            self.preset = DateRangePreset::Custom;
        }
    }
}

/// Date range picker widget
pub struct DateRangePicker<'a> {
    title: &'a str,
    style: Style,
    selected_style: Style,
}

impl<'a> DateRangePicker<'a> {
    /// Create a new date range picker
    pub fn new() -> Self {
        Self {
            title: "Date Range",
            style: Style::default(),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
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
}

impl<'a> Default for DateRangePicker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for DateRangePicker<'a> {
    type State = DateRangePickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Preset list
                Constraint::Length(3), // Current range display
            ])
            .split(area);

        // Build preset list
        let presets = DateRangePreset::all();
        let items: Vec<ListItem> = presets
            .iter()
            .map(|preset| {
                let mut content = preset.name().to_string();

                // Add date range preview for non-custom presets
                if *preset != DateRangePreset::Custom {
                    if let Some((start, end)) = preset.date_range() {
                        if start == end {
                            content.push_str(&format!(" ({start})"));
                        } else {
                            content.push_str(&format!(" ({start} to {end})"));
                        }
                    }
                }

                ListItem::new(Line::from(content))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .highlight_style(self.selected_style)
            .highlight_symbol("> ");

        StatefulWidget::render(list, chunks[0], buf, &mut state.list_state);

        // Render current range
        let range_text = if state.range.is_empty() {
            "No date range selected".to_string()
        } else {
            format!("Selected: {}", state.range.format())
        };

        let range_block = Block::default()
            .borders(Borders::ALL)
            .title("Current Range");

        let range_inner = range_block.inner(chunks[1]);
        range_block.render(chunks[1], buf);

        let range_para = Line::from(Span::styled(
            range_text,
            if state.range.is_empty() {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC)
            } else {
                Style::default().fg(Color::Green)
            },
        ));

        range_para.render(range_inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_creation() {
        let range = DateRange::new();
        assert!(range.is_empty());
        assert_eq!(range.format(), "Any date");
    }

    #[test]
    fn test_date_range_from_dates() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let range = DateRange::from_dates(start, end);

        assert!(!range.is_empty());
        assert_eq!(range.start, Some(start));
        assert_eq!(range.end, Some(end));
    }

    #[test]
    fn test_date_range_contains() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let range = DateRange::from_dates(start, end);

        let mid = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(range.contains(&mid));

        let before = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        assert!(!range.contains(&before));

        let after = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        assert!(!range.contains(&after));
    }

    #[test]
    fn test_date_range_format() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let range = DateRange::from_dates(start, end);
        assert!(range.format().contains("2024-01-01"));
        assert!(range.format().contains("2024-01-31"));

        let same_day = DateRange::from_dates(start, start);
        assert_eq!(same_day.format(), "2024-01-01");
    }

    #[test]
    fn test_date_range_picker_state_creation() {
        let state = DateRangePickerState::new();
        assert_eq!(state.preset(), DateRangePreset::Today);
        assert!(!state.is_custom_mode());
    }

    #[test]
    fn test_date_range_picker_navigation() {
        let mut state = DateRangePickerState::new();

        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(1));

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(0));

        // Wrap around to end
        state.select_previous();
        assert_eq!(
            state.list_state.selected(),
            Some(DateRangePreset::all().len() - 1)
        );
    }

    #[test]
    fn test_date_range_picker_apply_preset() {
        let mut state = DateRangePickerState::new();

        // Select and apply "Last 7 days"
        state.list_state.select(Some(2)); // Last7Days
        state.apply_selected();

        assert_eq!(state.preset(), DateRangePreset::Last7Days);
        assert!(!state.range.is_empty());
    }

    #[test]
    fn test_date_range_picker_custom_range() {
        let mut state = DateRangePickerState::new();

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        state.set_date_range(Some(start), Some(end));

        assert_eq!(state.preset(), DateRangePreset::Custom);
        assert!(state.is_custom_mode());
        assert_eq!(state.date_range().start, Some(start));
        assert_eq!(state.date_range().end, Some(end));
    }

    #[test]
    fn test_date_range_picker_clear() {
        let mut state = DateRangePickerState::new();

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        state.set_date_range(Some(start), Some(end));

        state.clear();

        assert!(state.date_range().is_empty());
        assert_eq!(state.preset(), DateRangePreset::Today);
        assert!(!state.is_custom_mode());
    }

    #[test]
    fn test_preset_today() {
        let preset = DateRangePreset::Today;
        let range = preset.date_range().unwrap();
        assert_eq!(range.0, range.1); // Same day
    }

    #[test]
    fn test_preset_last_7_days() {
        let preset = DateRangePreset::Last7Days;
        let (start, end) = preset.date_range().unwrap();

        // Should be 7 days including today
        let days_diff = end.signed_duration_since(start).num_days();
        assert_eq!(days_diff, 6); // 7 days = 6 day difference
    }

    #[test]
    fn test_preset_last_30_days() {
        let preset = DateRangePreset::Last30Days;
        let (start, end) = preset.date_range().unwrap();

        // Should be 30 days including today
        let days_diff = end.signed_duration_since(start).num_days();
        assert_eq!(days_diff, 29); // 30 days = 29 day difference
    }

    #[test]
    fn test_preset_this_week() {
        let preset = DateRangePreset::ThisWeek;
        let (start, end) = preset.date_range().unwrap();

        // Start should be a Monday
        assert_eq!(start.weekday().num_days_from_monday(), 0);

        // End should be today or later in the week
        let now = Local::now().naive_local().date();
        assert_eq!(end, now);
    }

    #[test]
    fn test_preset_custom_returns_none() {
        let preset = DateRangePreset::Custom;
        assert!(preset.date_range().is_none());
    }

    #[test]
    fn test_date_range_partial() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut range = DateRange::new();
        range.start = Some(start);

        assert!(!range.is_empty());
        assert!(range.format().contains("From"));
    }

    #[test]
    fn test_date_range_preset_clone() {
        let preset = DateRangePreset::Today;
        let cloned = preset.clone();
        assert_eq!(preset, cloned);
    }

    #[test]
    fn test_date_range_preset_equality() {
        assert_eq!(DateRangePreset::Today, DateRangePreset::Today);
        assert_ne!(DateRangePreset::Today, DateRangePreset::Yesterday);
        assert_eq!(DateRangePreset::Custom, DateRangePreset::Custom);
    }

    #[test]
    fn test_all_date_range_presets_have_names() {
        for preset in DateRangePreset::all() {
            assert!(!preset.name().is_empty());
        }
    }

    #[test]
    fn test_date_range_preset_all_count() {
        let presets = DateRangePreset::all();
        assert_eq!(presets.len(), 9); // Should have exactly 9 presets
    }

    #[test]
    fn test_date_range_preset_all_contains_custom() {
        let presets = DateRangePreset::all();
        assert!(presets.contains(&DateRangePreset::Custom));
    }

    #[test]
    fn test_date_range_clone() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let range = DateRange::from_dates(start, end);

        let cloned = range.clone();
        assert_eq!(cloned.start, range.start);
        assert_eq!(cloned.end, range.end);
    }

    #[test]
    fn test_date_range_default() {
        let range = DateRange::default();
        assert!(range.is_empty());
        assert_eq!(range.format(), "Any date");
    }

    #[test]
    fn test_date_range_clear() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let mut range = DateRange::from_dates(start, end);

        range.clear();
        assert!(range.is_empty());
        assert_eq!(range.start, None);
        assert_eq!(range.end, None);
    }

    #[test]
    fn test_date_range_contains_only_start() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut range = DateRange::new();
        range.start = Some(start);

        let after = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(range.contains(&after));

        let before = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        assert!(!range.contains(&before));
    }

    #[test]
    fn test_date_range_contains_only_end() {
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let mut range = DateRange::new();
        range.end = Some(end);

        let before = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(range.contains(&before));

        let after = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        assert!(!range.contains(&after));
    }

    #[test]
    fn test_date_range_contains_empty_range() {
        let range = DateRange::new();
        let any_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        // Empty range contains all dates
        assert!(range.contains(&any_date));
    }

    #[test]
    fn test_date_range_contains_boundary() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let range = DateRange::from_dates(start, end);

        // Boundaries should be inclusive
        assert!(range.contains(&start));
        assert!(range.contains(&end));
    }

    #[test]
    fn test_date_range_format_only_end() {
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let mut range = DateRange::new();
        range.end = Some(end);

        assert!(range.format().contains("Until"));
        assert!(range.format().contains("2024-01-31"));
    }

    #[test]
    fn test_date_range_picker_state_clone() {
        let mut state = DateRangePickerState::new();
        state.select_next();

        let cloned = state.clone();
        assert_eq!(cloned.preset(), state.preset());
        assert_eq!(cloned.is_custom_mode(), state.is_custom_mode());
    }

    #[test]
    fn test_date_range_picker_state_default() {
        let state = DateRangePickerState::default();
        assert_eq!(state.preset(), DateRangePreset::Today);
        assert!(!state.is_custom_mode());
    }

    #[test]
    fn test_date_range_picker_apply_selected_out_of_bounds() {
        let mut state = DateRangePickerState::new();
        state.list_state.select(Some(100)); // Invalid index

        state.apply_selected();
        // Should not change or crash
        assert_eq!(state.preset(), DateRangePreset::Today);
    }

    #[test]
    fn test_date_range_picker_apply_selected_custom() {
        let mut state = DateRangePickerState::new();
        
        // Select Custom preset (last one)
        state.list_state.select(Some(DateRangePreset::all().len() - 1));
        state.apply_selected();

        assert_eq!(state.preset(), DateRangePreset::Custom);
        assert!(state.is_custom_mode());
    }

    #[test]
    fn test_preset_yesterday() {
        let preset = DateRangePreset::Yesterday;
        let (start, end) = preset.date_range().unwrap();
        
        // Yesterday should be a single day
        assert_eq!(start, end);
        
        let now = Local::now().naive_local().date();
        let yesterday = now - Duration::days(1);
        assert_eq!(start, yesterday);
    }

    #[test]
    fn test_preset_last_week() {
        let preset = DateRangePreset::LastWeek;
        let (_start, end) = preset.date_range().unwrap();

        // End should be Sunday
        assert_eq!(end.weekday().num_days_from_monday(), 6);
    }

    #[test]
    fn test_preset_this_month() {
        let preset = DateRangePreset::ThisMonth;
        let (start, end) = preset.date_range().unwrap();

        let now = Local::now().naive_local().date();
        
        // Start should be first day of current month
        assert_eq!(start.day(), 1);
        assert_eq!(start.month(), now.month());
        assert_eq!(start.year(), now.year());
        
        // End should be today
        assert_eq!(end, now);
    }

    #[test]
    fn test_preset_last_month() {
        let preset = DateRangePreset::LastMonth;
        let (start, end) = preset.date_range().unwrap();

        // Start should be first day of last month
        assert_eq!(start.day(), 1);
        
        // Should be exactly one month before current month
        let now = Local::now().naive_local().date();
        if now.month() == 1 {
            assert_eq!(start.month(), 12);
            assert_eq!(start.year(), now.year() - 1);
        } else {
            assert_eq!(start.month(), now.month() - 1);
            assert_eq!(start.year(), now.year());
        }
    }

    #[test]
    fn test_all_preset_names() {
        assert_eq!(DateRangePreset::Today.name(), "Today");
        assert_eq!(DateRangePreset::Yesterday.name(), "Yesterday");
        assert_eq!(DateRangePreset::Last7Days.name(), "Last 7 days");
        assert_eq!(DateRangePreset::Last30Days.name(), "Last 30 days");
        assert_eq!(DateRangePreset::ThisWeek.name(), "This week");
        assert_eq!(DateRangePreset::LastWeek.name(), "Last week");
        assert_eq!(DateRangePreset::ThisMonth.name(), "This month");
        assert_eq!(DateRangePreset::LastMonth.name(), "Last month");
        assert_eq!(DateRangePreset::Custom.name(), "Custom range...");
    }
}
