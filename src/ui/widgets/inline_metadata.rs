//! Inline metadata display widget for showing labels, assignee, and age

use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

/// Metadata display configuration
#[derive(Debug, Clone)]
pub struct MetadataDisplayConfig {
    /// Show labels
    pub show_labels: bool,
    /// Show assignee
    pub show_assignee: bool,
    /// Show age (time since creation)
    pub show_age: bool,
    /// Show last updated time
    pub show_updated: bool,
    /// Maximum number of labels to display
    pub max_labels: usize,
    /// Label style
    pub label_style: Style,
    /// Assignee style
    pub assignee_style: Style,
    /// Age style
    pub age_style: Style,
    /// Separator between metadata items
    pub separator: String,
}

impl Default for MetadataDisplayConfig {
    fn default() -> Self {
        Self {
            show_labels: true,
            show_assignee: true,
            show_age: true,
            show_updated: false,
            max_labels: 3,
            label_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
            assignee_style: Style::default().fg(Color::Yellow),
            age_style: Style::default().fg(Color::DarkGray),
            separator: " • ".to_string(),
        }
    }
}

impl MetadataDisplayConfig {
    /// Create a new metadata display config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to show labels
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set whether to show assignee
    pub fn show_assignee(mut self, show: bool) -> Self {
        self.show_assignee = show;
        self
    }

    /// Set whether to show age
    pub fn show_age(mut self, show: bool) -> Self {
        self.show_age = show;
        self
    }

    /// Set whether to show updated time
    pub fn show_updated(mut self, show: bool) -> Self {
        self.show_updated = show;
        self
    }

    /// Set maximum number of labels to display
    pub fn max_labels(mut self, max: usize) -> Self {
        self.max_labels = max;
        self
    }

    /// Set label style
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Set assignee style
    pub fn assignee_style(mut self, style: Style) -> Self {
        self.assignee_style = style;
        self
    }

    /// Set age style
    pub fn age_style(mut self, style: Style) -> Self {
        self.age_style = style;
        self
    }

    /// Set separator
    pub fn separator<S: Into<String>>(mut self, sep: S) -> Self {
        self.separator = sep.into();
        self
    }
}

/// Format a duration as a human-readable age string
pub fn format_age(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(timestamp);

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if seconds < 60 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{minutes}m ago")
    } else if hours < 24 {
        format!("{hours}h ago")
    } else if days < 7 {
        format!("{days}d ago")
    } else if days < 30 {
        let weeks = days / 7;
        format!("{weeks}w ago")
    } else if days < 365 {
        let months = days / 30;
        format!("{months}mo ago")
    } else {
        let years = days / 365;
        format!("{years}y ago")
    }
}

/// Format labels as a compact string
pub fn format_labels(labels: &[String], max_labels: usize) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let visible_labels: Vec<_> = labels.iter().take(max_labels).collect();
    let hidden_count = labels.len().saturating_sub(max_labels);

    let mut result = visible_labels
        .iter()
        .map(|l| format!("#{l}"))
        .collect::<Vec<_>>()
        .join(" ");

    if hidden_count > 0 {
        result.push_str(&format!(" +{hidden_count}"));
    }

    result
}

/// Format assignee as a compact string
pub fn format_assignee(assignee: Option<&str>) -> String {
    assignee.map(|a| format!("@{a}")).unwrap_or_default()
}

/// Build inline metadata spans
pub fn build_metadata_spans(
    labels: &[String],
    assignee: Option<&str>,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
    config: &MetadataDisplayConfig,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut added_any = false;

    // Add labels
    if config.show_labels && !labels.is_empty() {
        let labels_text = format_labels(labels, config.max_labels);
        if !labels_text.is_empty() {
            if added_any {
                spans.push(Span::raw(config.separator.clone()));
            }
            spans.push(Span::styled(labels_text, config.label_style));
            added_any = true;
        }
    }

    // Add assignee
    if config.show_assignee {
        let assignee_text = format_assignee(assignee);
        if !assignee_text.is_empty() {
            if added_any {
                spans.push(Span::raw(config.separator.clone()));
            }
            spans.push(Span::styled(assignee_text, config.assignee_style));
            added_any = true;
        }
    }

    // Add age (created time)
    if config.show_age {
        let age_text = format_age(created);
        if !age_text.is_empty() {
            if added_any {
                spans.push(Span::raw(config.separator.clone()));
            }
            spans.push(Span::styled(age_text, config.age_style));
            added_any = true;
        }
    }

    // Add updated time
    if config.show_updated {
        if let Some(updated_time) = updated {
            let updated_text = format!("updated {}", format_age(updated_time));
            if added_any {
                spans.push(Span::raw(config.separator.clone()));
            }
            spans.push(Span::styled(updated_text, config.age_style));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_age_just_now() {
        let now = Utc::now();
        assert_eq!(format_age(now), "just now");
    }

    #[test]
    fn test_format_age_minutes() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::minutes(5);
        assert_eq!(format_age(timestamp), "5m ago");
    }

    #[test]
    fn test_format_age_hours() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::hours(3);
        assert_eq!(format_age(timestamp), "3h ago");
    }

    #[test]
    fn test_format_age_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(2);
        assert_eq!(format_age(timestamp), "2d ago");
    }

    #[test]
    fn test_format_age_weeks() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(14);
        assert_eq!(format_age(timestamp), "2w ago");
    }

    #[test]
    fn test_format_age_months() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(60);
        assert_eq!(format_age(timestamp), "2mo ago");
    }

    #[test]
    fn test_format_age_years() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(730);
        assert_eq!(format_age(timestamp), "2y ago");
    }

    #[test]
    fn test_format_labels_empty() {
        let labels: Vec<String> = Vec::new();
        assert_eq!(format_labels(&labels, 3), "");
    }

    #[test]
    fn test_format_labels_single() {
        let labels = vec!["bug".to_string()];
        assert_eq!(format_labels(&labels, 3), "#bug");
    }

    #[test]
    fn test_format_labels_multiple() {
        let labels = vec![
            "bug".to_string(),
            "urgent".to_string(),
            "backend".to_string(),
        ];
        assert_eq!(format_labels(&labels, 3), "#bug #urgent #backend");
    }

    #[test]
    fn test_format_labels_truncated() {
        let labels = vec![
            "bug".to_string(),
            "urgent".to_string(),
            "backend".to_string(),
            "frontend".to_string(),
            "testing".to_string(),
        ];
        assert_eq!(format_labels(&labels, 3), "#bug #urgent #backend +2");
    }

    #[test]
    fn test_format_labels_max_zero() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        assert_eq!(format_labels(&labels, 0), " +2");
    }

    #[test]
    fn test_format_assignee_some() {
        assert_eq!(format_assignee(Some("alice")), "@alice");
    }

    #[test]
    fn test_format_assignee_none() {
        assert_eq!(format_assignee(None), "");
    }

    #[test]
    fn test_build_metadata_spans_all() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(2);
        let updated = Some(Utc::now() - chrono::Duration::minutes(30));
        let config = MetadataDisplayConfig::default();

        let spans = build_metadata_spans(&labels, assignee, created, updated, &config);

        // Should have: labels + sep + assignee + sep + age = 5 spans
        assert!(spans.len() >= 5);
    }

    #[test]
    fn test_build_metadata_spans_no_labels() {
        let labels: Vec<String> = Vec::new();
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default();

        let spans = build_metadata_spans(&labels, assignee, created, None, &config);

        // Should have: assignee + sep + age = 3 spans
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn test_build_metadata_spans_no_assignee() {
        let labels = vec!["bug".to_string()];
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default();

        let spans = build_metadata_spans(&labels, None, created, None, &config);

        // Should have: labels + sep + age = 3 spans
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn test_build_metadata_spans_only_age() {
        let labels: Vec<String> = Vec::new();
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default()
            .show_labels(false)
            .show_assignee(false);

        let spans = build_metadata_spans(&labels, None, created, None, &config);

        // Should have only age
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_build_metadata_spans_custom_separator() {
        let labels = vec!["bug".to_string()];
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default().separator(" | ");

        let spans = build_metadata_spans(&labels, assignee, created, None, &config);

        // Check that separator is used
        assert!(spans.iter().any(|s| s.content == " | "));
    }

    #[test]
    fn test_metadata_display_config_builder() {
        let config = MetadataDisplayConfig::new()
            .show_labels(true)
            .show_assignee(false)
            .show_age(true)
            .max_labels(5)
            .separator(" / ");

        assert!(config.show_labels);
        assert!(!config.show_assignee);
        assert!(config.show_age);
        assert_eq!(config.max_labels, 5);
        assert_eq!(config.separator, " / ");
    }

    #[test]
    fn test_metadata_display_config_default() {
        let config = MetadataDisplayConfig::default();

        assert!(config.show_labels);
        assert!(config.show_assignee);
        assert!(config.show_age);
        assert!(!config.show_updated);
        assert_eq!(config.max_labels, 3);
        assert_eq!(config.separator, " • ");
    }

    #[test]
    fn test_metadata_display_config_clone() {
        let config = MetadataDisplayConfig::default();
        let cloned = config.clone();
        assert_eq!(config.show_labels, cloned.show_labels);
        assert_eq!(config.show_assignee, cloned.show_assignee);
        assert_eq!(config.show_age, cloned.show_age);
        assert_eq!(config.max_labels, cloned.max_labels);
        assert_eq!(config.separator, cloned.separator);
    }

    #[test]
    fn test_metadata_display_config_new_equals_default() {
        let new_config = MetadataDisplayConfig::new();
        let default_config = MetadataDisplayConfig::default();
        assert_eq!(new_config.show_labels, default_config.show_labels);
        assert_eq!(new_config.show_assignee, default_config.show_assignee);
        assert_eq!(new_config.max_labels, default_config.max_labels);
    }

    #[test]
    fn test_metadata_display_config_builder_chain() {
        let style = Style::default().fg(Color::Red);
        let config = MetadataDisplayConfig::new()
            .show_labels(false)
            .show_assignee(true)
            .show_age(false)
            .show_updated(true)
            .max_labels(10)
            .separator(" | ")
            .label_style(style)
            .assignee_style(style)
            .age_style(style);

        assert!(!config.show_labels);
        assert!(config.show_assignee);
        assert!(!config.show_age);
        assert!(config.show_updated);
        assert_eq!(config.max_labels, 10);
        assert_eq!(config.separator, " | ");
        assert_eq!(config.label_style.fg, Some(Color::Red));
        assert_eq!(config.assignee_style.fg, Some(Color::Red));
        assert_eq!(config.age_style.fg, Some(Color::Red));
    }

    #[test]
    fn test_format_age_boundary_59_seconds() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::seconds(59);
        assert_eq!(format_age(timestamp), "just now");
    }

    #[test]
    fn test_format_age_boundary_60_seconds() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::seconds(60);
        assert_eq!(format_age(timestamp), "1m ago");
    }

    #[test]
    fn test_format_age_boundary_59_minutes() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::minutes(59);
        assert_eq!(format_age(timestamp), "59m ago");
    }

    #[test]
    fn test_format_age_boundary_60_minutes() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::minutes(60);
        assert_eq!(format_age(timestamp), "1h ago");
    }

    #[test]
    fn test_format_age_boundary_23_hours() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::hours(23);
        assert_eq!(format_age(timestamp), "23h ago");
    }

    #[test]
    fn test_format_age_boundary_24_hours() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::hours(24);
        assert_eq!(format_age(timestamp), "1d ago");
    }

    #[test]
    fn test_format_age_boundary_6_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(6);
        assert_eq!(format_age(timestamp), "6d ago");
    }

    #[test]
    fn test_format_age_boundary_7_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(7);
        assert_eq!(format_age(timestamp), "1w ago");
    }

    #[test]
    fn test_format_age_boundary_29_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(29);
        assert_eq!(format_age(timestamp), "4w ago");
    }

    #[test]
    fn test_format_age_boundary_30_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(30);
        assert_eq!(format_age(timestamp), "1mo ago");
    }

    #[test]
    fn test_format_age_boundary_364_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(364);
        assert_eq!(format_age(timestamp), "12mo ago");
    }

    #[test]
    fn test_format_age_boundary_365_days() {
        let now = Utc::now();
        let timestamp = now - chrono::Duration::days(365);
        assert_eq!(format_age(timestamp), "1y ago");
    }

    #[test]
    fn test_format_labels_max_one() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        assert_eq!(format_labels(&labels, 1), "#bug +1");
    }

    #[test]
    fn test_format_labels_max_exact_count() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        assert_eq!(format_labels(&labels, 2), "#bug #urgent");
    }

    #[test]
    fn test_format_assignee_with_special_chars() {
        assert_eq!(format_assignee(Some("alice-bob")), "@alice-bob");
        assert_eq!(format_assignee(Some("alice_bob")), "@alice_bob");
    }

    #[test]
    fn test_build_metadata_spans_show_updated_enabled() {
        let labels: Vec<String> = Vec::new();
        let created = Utc::now() - chrono::Duration::hours(5);
        let updated = Some(Utc::now() - chrono::Duration::hours(1));
        let config = MetadataDisplayConfig::default()
            .show_labels(false)
            .show_assignee(false)
            .show_updated(true);

        let spans = build_metadata_spans(&labels, None, created, updated, &config);

        // Should have: age + sep + updated = 3 spans
        assert_eq!(spans.len(), 3);
        assert!(spans.iter().any(|s| s.content.contains("updated")));
    }

    #[test]
    fn test_build_metadata_spans_show_updated_no_value() {
        let labels: Vec<String> = Vec::new();
        let created = Utc::now() - chrono::Duration::hours(5);
        let config = MetadataDisplayConfig::default()
            .show_labels(false)
            .show_assignee(false)
            .show_updated(true);

        let spans = build_metadata_spans(&labels, None, created, None, &config);

        // Should have only age (no updated available)
        assert_eq!(spans.len(), 1);
        assert!(!spans.iter().any(|s| s.content.contains("updated")));
    }

    #[test]
    fn test_build_metadata_spans_all_disabled() {
        let labels = vec!["bug".to_string()];
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default()
            .show_labels(false)
            .show_assignee(false)
            .show_age(false);

        let spans = build_metadata_spans(&labels, assignee, created, None, &config);

        // Should have no spans
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_build_metadata_spans_with_all_features() {
        let labels = vec!["bug".to_string(), "urgent".to_string(), "backend".to_string()];
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(5);
        let updated = Some(Utc::now() - chrono::Duration::hours(1));
        let config = MetadataDisplayConfig::default()
            .show_updated(true)
            .max_labels(2);

        let spans = build_metadata_spans(&labels, assignee, created, updated, &config);

        // Should have: labels + sep + assignee + sep + age + sep + updated = 7 spans
        assert_eq!(spans.len(), 7);
    }

    #[test]
    fn test_build_metadata_spans_empty_labels_not_shown() {
        let labels: Vec<String> = Vec::new();
        let assignee = Some("alice");
        let created = Utc::now() - chrono::Duration::hours(2);
        let config = MetadataDisplayConfig::default().show_labels(true);

        let spans = build_metadata_spans(&labels, assignee, created, None, &config);

        // Should not include labels section (empty)
        assert!(!spans.iter().any(|s| s.content.contains('#')));
    }
}
