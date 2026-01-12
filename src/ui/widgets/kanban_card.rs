//! Kanban card widget for rendering issue cards in columns

use crate::beads::models::{Issue, Priority};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::inline_metadata::MetadataDisplayConfig;

/// Card display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardMode {
    /// Single line: ID + truncated title only
    SingleLine,
    /// Two lines: ID + title (line 1), metadata (line 2)
    TwoLine,
}

/// Card renderer configuration
#[derive(Debug, Clone)]
pub struct KanbanCardConfig {
    /// Card display mode
    pub mode: CardMode,
    /// Maximum width for card content (column width minus borders/padding)
    pub max_width: u16,
    /// Style for selected cards
    pub selected_style: Style,
    /// Style for unselected cards
    pub normal_style: Style,
    /// Style for card ID
    pub id_style: Style,
    /// Style for card title
    pub title_style: Style,
    /// Priority color mapping
    pub priority_colors: bool,
    /// Metadata display configuration
    pub metadata_config: MetadataDisplayConfig,
}

impl Default for KanbanCardConfig {
    fn default() -> Self {
        Self {
            mode: CardMode::TwoLine,
            max_width: 30,
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default(),
            id_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            title_style: Style::default(),
            priority_colors: true,
            metadata_config: MetadataDisplayConfig::default()
                .show_age(false)
                .show_updated(false)
                .max_labels(2),
        }
    }
}

impl KanbanCardConfig {
    /// Create a new card config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set card mode
    pub fn mode(mut self, mode: CardMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Set selected style
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }
}

/// Get priority color
fn priority_color(priority: Priority) -> Color {
    match priority {
        Priority::P0 => Color::Red,
        Priority::P1 => Color::LightRed,
        Priority::P2 => Color::Yellow,
        Priority::P3 => Color::Blue,
        Priority::P4 => Color::DarkGray,
    }
}

/// Wrap text to fit within a given width
/// Returns vector of lines, each line is at most max_width characters
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_len = word.len();

        // If adding this word would exceed width, start a new line
        if current_width + word_len + (if current_width > 0 { 1 } else { 0 }) > max_width {
            // If current line is not empty, push it
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }

            // If word itself is too long, truncate it
            if word_len > max_width {
                current_line = format!("{}...", &word[..max_width.saturating_sub(3)]);
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
                continue;
            }
        }

        // Add word to current line
        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }
        current_line.push_str(word);
        current_width += word_len;
    }

    // Push the last line if not empty
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // If no lines were created, return empty line
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Truncate text with ellipsis if it exceeds max_width
fn truncate_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &text[..max_width - 3])
    }
}

/// Render a Kanban card for an issue
/// Returns vector of Lines, one per line of the card
pub fn render_kanban_card(
    issue: &Issue,
    config: &KanbanCardConfig,
    is_selected: bool,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let base_style = if is_selected {
        config.selected_style
    } else {
        config.normal_style
    };

    // Calculate available width for content
    let max_width = config.max_width.max(10) as usize; // Minimum 10 chars

    match config.mode {
        CardMode::SingleLine => {
            // Single line: ID + truncated title
            let id_str = format!("{} ", issue.id);
            let title_width = max_width.saturating_sub(id_str.len());
            let title_truncated = truncate_text(&issue.title, title_width);

            let mut spans = vec![
                Span::styled(id_str, config.id_style.patch(base_style)),
                Span::styled(title_truncated, config.title_style.patch(base_style)),
            ];

            // Pad the line to full width if selected (for consistent highlighting)
            if is_selected {
                let content_len = issue.id.len() + 1 + title_width.min(issue.title.len());
                if content_len < max_width {
                    spans.push(Span::styled(
                        " ".repeat(max_width - content_len),
                        base_style,
                    ));
                }
            }

            lines.push(Line::from(spans));
        }
        CardMode::TwoLine => {
            // Line 1: ID + title (wrapped)
            let id_str = format!("{} ", issue.id);
            let id_len = id_str.len();
            let title_width = max_width.saturating_sub(id_len);
            let wrapped_title = wrap_text(&issue.title, title_width);

            // First line of title (with ID)
            if let Some(first_line) = wrapped_title.first() {
                let mut spans = vec![
                    Span::styled(id_str, config.id_style.patch(base_style)),
                    Span::styled(first_line.clone(), config.title_style.patch(base_style)),
                ];

                // Pad to full width if selected
                if is_selected {
                    let content_len = id_len + first_line.len();
                    if content_len < max_width {
                        spans.push(Span::styled(
                            " ".repeat(max_width - content_len),
                            base_style,
                        ));
                    }
                }

                lines.push(Line::from(spans));
            }

            // Subsequent lines of title (indented to align with text after ID)
            for title_line in wrapped_title.iter().skip(1) {
                let mut spans = vec![
                    Span::styled(" ".repeat(id_len), base_style),
                    Span::styled(title_line.clone(), config.title_style.patch(base_style)),
                ];

                // Pad to full width if selected
                if is_selected {
                    let content_len = id_len + title_line.len();
                    if content_len < max_width {
                        spans.push(Span::styled(
                            " ".repeat(max_width - content_len),
                            base_style,
                        ));
                    }
                }

                lines.push(Line::from(spans));
            }

            // Line 2: Compact metadata (priority, status, assignee, labels)
            let mut metadata_spans = Vec::new();

            // Priority indicator
            if config.priority_colors {
                let priority_symbol = match issue.priority {
                    Priority::P0 => "◆",
                    Priority::P1 => "●",
                    Priority::P2 => "○",
                    Priority::P3 => "◇",
                    Priority::P4 => "·",
                };
                metadata_spans.push(Span::styled(
                    format!("{priority_symbol} "),
                    Style::default()
                        .fg(priority_color(issue.priority))
                        .patch(base_style),
                ));
            }

            // Status badge
            let status_str = match issue.status {
                crate::beads::models::IssueStatus::Open => "OPN",
                crate::beads::models::IssueStatus::InProgress => "WIP",
                crate::beads::models::IssueStatus::Blocked => "BLK",
                crate::beads::models::IssueStatus::Closed => "CLS",
            };
            metadata_spans.push(Span::styled(
                format!("{status_str} "),
                Style::default().fg(Color::White).patch(base_style),
            ));

            // Assignee and labels using inline_metadata helpers
            let assignee_str = if let Some(assignee) = &issue.assignee {
                format!("@{assignee} ")
            } else {
                String::new()
            };
            if !assignee_str.is_empty() {
                metadata_spans.push(Span::styled(
                    assignee_str,
                    config.metadata_config.assignee_style.patch(base_style),
                ));
            }

            // Labels (truncated to fit)
            if !issue.labels.is_empty() {
                let max_labels = config.metadata_config.max_labels;
                let visible_labels: Vec<_> = issue.labels.iter().take(max_labels).collect();
                let hidden_count = issue.labels.len().saturating_sub(max_labels);

                for (i, label) in visible_labels.iter().enumerate() {
                    metadata_spans.push(Span::styled(
                        format!("#{label}"),
                        config.metadata_config.label_style.patch(base_style),
                    ));
                    if i < visible_labels.len() - 1 || hidden_count > 0 {
                        metadata_spans.push(Span::raw(" "));
                    }
                }

                if hidden_count > 0 {
                    metadata_spans.push(Span::styled(
                        format!("+{hidden_count}"),
                        config.metadata_config.label_style.patch(base_style),
                    ));
                }
            }

            // Pad metadata line to full width if selected
            if is_selected {
                let metadata_len: usize = metadata_spans.iter().map(|s| s.content.len()).sum();
                if metadata_len < max_width {
                    metadata_spans.push(Span::styled(
                        " ".repeat(max_width - metadata_len),
                        base_style,
                    ));
                }
            }

            lines.push(Line::from(metadata_spans));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType};
    use chrono::Utc;

    fn create_test_issue() -> Issue {
        Issue {
            id: "TEST-123".to_string(),
            title: "Test issue title".to_string(),
            description: None,
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: Some("john".to_string()),
            labels: vec!["bug".to_string(), "urgent".to_string()],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_wrap_text_simple() {
        let text = "This is a test";
        let wrapped = wrap_text(text, 10);
        assert_eq!(wrapped, vec!["This is a", "test"]);
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "ThisIsAVeryLongWordThatShouldBeTruncated";
        let wrapped = wrap_text(text, 10);
        assert_eq!(wrapped[0], "ThisIsA...");
    }

    #[test]
    fn test_wrap_text_fits_exactly() {
        let text = "Exact";
        let wrapped = wrap_text(text, 5);
        assert_eq!(wrapped, vec!["Exact"]);
    }

    #[test]
    fn test_truncate_text_short() {
        assert_eq!(truncate_text("short", 10), "short");
    }

    #[test]
    fn test_truncate_text_long() {
        assert_eq!(truncate_text("This is a very long text", 10), "This is...");
    }

    #[test]
    fn test_truncate_text_very_short_width() {
        assert_eq!(truncate_text("test", 2), "...");
    }

    #[test]
    fn test_render_single_line_card() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default().mode(CardMode::SingleLine);
        let lines = render_kanban_card(&issue, &config, false);

        assert_eq!(lines.len(), 1);
        // Check that ID is present
        let line_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line_text.starts_with("TEST-123 "));
        assert!(line_text.contains("Test issue"));
    }

    #[test]
    fn test_render_two_line_card() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default().mode(CardMode::TwoLine);
        let lines = render_kanban_card(&issue, &config, false);

        assert!(lines.len() >= 2);

        // Line 1 should have ID and title
        let line1_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line1_text.starts_with("TEST-123 "));
        assert!(line1_text.contains("Test issue"));

        // Line 2 should have metadata
        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_text.contains("OPN")); // Status
        assert!(line2_text.contains("@john")); // Assignee
    }

    #[test]
    fn test_render_selected_card() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, true);

        // Check that spans have the selected style
        assert!(lines.iter().any(|line| line
            .spans
            .iter()
            .any(|span| span.style == config.selected_style)));
    }

    #[test]
    fn test_render_card_with_long_title() {
        let mut issue = create_test_issue();
        issue.title = "This is a very long title that should wrap across multiple lines in the card when rendered".to_string();

        let config = KanbanCardConfig::default()
            .mode(CardMode::TwoLine)
            .max_width(30);
        let lines = render_kanban_card(&issue, &config, false);

        // Should have multiple lines for wrapped title + metadata line
        assert!(lines.len() > 2);
    }

    #[test]
    fn test_priority_color_mapping() {
        assert_eq!(priority_color(Priority::P0), Color::Red);
        assert_eq!(priority_color(Priority::P1), Color::LightRed);
        assert_eq!(priority_color(Priority::P2), Color::Yellow);
        assert_eq!(priority_color(Priority::P3), Color::Blue);
        assert_eq!(priority_color(Priority::P4), Color::DarkGray);
    }

    #[test]
    fn test_wrap_text_empty() {
        let wrapped = wrap_text("", 10);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn test_wrap_text_single_word() {
        let wrapped = wrap_text("Word", 10);
        assert_eq!(wrapped, vec!["Word"]);
    }

    #[test]
    fn test_render_card_no_assignee() {
        let mut issue = create_test_issue();
        issue.assignee = None;

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!line2_text.contains('@'));
    }

    #[test]
    fn test_render_card_no_labels() {
        let mut issue = create_test_issue();
        issue.labels.clear();

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!line2_text.contains('#'));
    }

    #[test]
    fn test_render_card_truncate_labels() {
        let mut issue = create_test_issue();
        issue.labels = vec![
            "label1".to_string(),
            "label2".to_string(),
            "label3".to_string(),
            "label4".to_string(),
        ];

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        // Should show "+2" for hidden labels (config.max_labels defaults to 2)
        assert!(line2_text.contains("+2"));
    }

    #[test]
    fn test_card_mode_clone() {
        let mode = CardMode::SingleLine;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_card_mode_equality() {
        assert_eq!(CardMode::SingleLine, CardMode::SingleLine);
        assert_eq!(CardMode::TwoLine, CardMode::TwoLine);
        assert_ne!(CardMode::SingleLine, CardMode::TwoLine);
    }

    #[test]
    fn test_kanban_card_config_default() {
        let config = KanbanCardConfig::default();
        assert_eq!(config.mode, CardMode::TwoLine);
        assert_eq!(config.max_width, 30);
        assert!(config.priority_colors);
    }

    #[test]
    fn test_kanban_card_config_new() {
        let config = KanbanCardConfig::new();
        assert_eq!(config.mode, CardMode::TwoLine);
        assert_eq!(config.max_width, 30);
    }

    #[test]
    fn test_kanban_card_config_clone() {
        let config = KanbanCardConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.mode, config.mode);
        assert_eq!(cloned.max_width, config.max_width);
        assert_eq!(cloned.priority_colors, config.priority_colors);
    }

    #[test]
    fn test_kanban_card_config_builder_mode() {
        let config = KanbanCardConfig::new().mode(CardMode::SingleLine);
        assert_eq!(config.mode, CardMode::SingleLine);
    }

    #[test]
    fn test_kanban_card_config_builder_max_width() {
        let config = KanbanCardConfig::new().max_width(50);
        assert_eq!(config.max_width, 50);
    }

    #[test]
    fn test_kanban_card_config_builder_selected_style() {
        let style = Style::default().fg(Color::Red);
        let config = KanbanCardConfig::new().selected_style(style);
        assert_eq!(config.selected_style.fg, Some(Color::Red));
    }

    #[test]
    fn test_kanban_card_config_builder_chain() {
        let style = Style::default().bg(Color::Blue);
        let config = KanbanCardConfig::new()
            .mode(CardMode::SingleLine)
            .max_width(40)
            .selected_style(style);

        assert_eq!(config.mode, CardMode::SingleLine);
        assert_eq!(config.max_width, 40);
        assert_eq!(config.selected_style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_wrap_text_zero_width() {
        let wrapped = wrap_text("test", 0);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn test_wrap_text_multiple_words_fit() {
        let wrapped = wrap_text("one two three", 15);
        assert_eq!(wrapped, vec!["one two three"]);
    }

    #[test]
    fn test_wrap_text_multiple_lines() {
        let text = "This is a longer text that needs multiple lines";
        let wrapped = wrap_text(text, 15);
        assert!(wrapped.len() > 2);
        assert!(wrapped[0].len() <= 15);
        assert!(wrapped[1].len() <= 15);
    }

    #[test]
    fn test_truncate_text_exact_width() {
        assert_eq!(truncate_text("12345", 5), "12345");
    }

    #[test]
    fn test_truncate_text_width_three() {
        assert_eq!(truncate_text("test", 3), "...");
    }

    #[test]
    fn test_truncate_text_empty() {
        assert_eq!(truncate_text("", 10), "");
    }

    #[test]
    fn test_render_card_in_progress_status() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::InProgress;

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_text.contains("WIP"));
    }

    #[test]
    fn test_render_card_blocked_status() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::Blocked;

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_text.contains("BLK"));
    }

    #[test]
    fn test_render_card_closed_status() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::Closed;

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_text.contains("CLS"));
    }

    #[test]
    fn test_render_card_minimum_width() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default().max_width(5);
        let lines = render_kanban_card(&issue, &config, false);

        // Should enforce minimum width of 10
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_card_different_priorities() {
        for priority in [Priority::P0, Priority::P1, Priority::P2, Priority::P3, Priority::P4] {
            let mut issue = create_test_issue();
            issue.priority = priority;

            let config = KanbanCardConfig::default();
            let lines = render_kanban_card(&issue, &config, false);

            // All priorities should render
            assert!(lines.len() >= 2);
        }
    }

    #[test]
    fn test_render_card_priority_colors_disabled() {
        let issue = create_test_issue();
        let mut config = KanbanCardConfig::default();
        config.priority_colors = false;

        let lines = render_kanban_card(&issue, &config, false);

        // Should still render without priority symbols
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_render_single_line_unselected() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default().mode(CardMode::SingleLine);
        let lines = render_kanban_card(&issue, &config, false);

        assert_eq!(lines.len(), 1);
        // Verify it doesn't have excessive padding when not selected
        let line_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line_text.contains("TEST-123"));
    }

    #[test]
    fn test_render_card_with_single_label() {
        let mut issue = create_test_issue();
        issue.labels = vec!["single".to_string()];

        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);

        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_text.contains("#single"));
        assert!(!line2_text.contains('+')); // No "+N" indicator
    }

    #[test]
    fn test_wrap_text_word_at_exact_boundary() {
        let wrapped = wrap_text("12345 678", 5);
        assert_eq!(wrapped, vec!["12345", "678"]);
    }

    #[test]
    fn test_card_mode_debug() {
        let mode = CardMode::SingleLine;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("SingleLine"));
    }

    #[test]
    fn test_card_mode_copy() {
        let mode1 = CardMode::TwoLine;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }

    #[test]
    fn test_card_mode_partial_eq() {
        assert_eq!(CardMode::SingleLine, CardMode::SingleLine);
        assert_eq!(CardMode::TwoLine, CardMode::TwoLine);
        assert_ne!(CardMode::SingleLine, CardMode::TwoLine);
    }

    #[test]
    fn test_kanban_card_config_debug() {
        let config = KanbanCardConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("KanbanCardConfig"));
    }

    #[test]
    fn test_kanban_card_config_clone_independence() {
        let mut config1 = KanbanCardConfig::default();
        let mut config2 = config1.clone();
        
        config1.max_width = 50;
        config2.max_width = 100;
        
        assert_eq!(config1.max_width, 50);
        assert_eq!(config2.max_width, 100);
    }

    #[test]
    fn test_kanban_card_config_builder_order_independence() {
        let style1 = Style::default().fg(Color::Red);
        
        let config1 = KanbanCardConfig::default()
            .mode(CardMode::TwoLine)
            .max_width(80)
            .selected_style(style1);
        
        let config2 = KanbanCardConfig::default()
            .max_width(80)
            .selected_style(style1)
            .mode(CardMode::TwoLine);
        
        assert_eq!(config1.mode, config2.mode);
        assert_eq!(config1.max_width, config2.max_width);
        assert_eq!(config1.selected_style.fg, config2.selected_style.fg);
    }

    #[test]
    fn test_kanban_card_config_builder_chaining() {
        let selected = Style::default().bg(Color::Blue);
        
        let config = KanbanCardConfig::default()
            .mode(CardMode::SingleLine)
            .max_width(60)
            .selected_style(selected);
        
        assert_eq!(config.mode, CardMode::SingleLine);
        assert_eq!(config.max_width, 60);
        assert_eq!(config.selected_style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_kanban_card_config_default_values() {
        let config = KanbanCardConfig::default();
        
        assert_eq!(config.mode, CardMode::TwoLine);
        assert!(config.priority_colors);
    }

    #[test]
    fn test_priority_color_all_priorities() {
        let p0_color = priority_color(Priority::P0);
        let p1_color = priority_color(Priority::P1);
        let p2_color = priority_color(Priority::P2);
        let p3_color = priority_color(Priority::P3);
        let p4_color = priority_color(Priority::P4);
        
        // All priorities should have colors
        assert_eq!(p0_color, Color::Red);
        assert_eq!(p1_color, Color::LightRed);
        assert_eq!(p2_color, Color::Yellow);
        assert_eq!(p3_color, Color::Blue);
        assert_eq!(p4_color, Color::DarkGray);
    }

    #[test]
    fn test_wrap_text_width_one() {
        let wrapped = wrap_text("abc", 1);
        // With width=1, long words get truncated to "..." (3 chars)
        assert!(!wrapped.is_empty());
        assert_eq!(wrapped[0], "...");
    }

    #[test]
    fn test_wrap_text_preserves_spaces() {
        let wrapped = wrap_text("a b c d", 5);
        // Should preserve spaces and wrap appropriately
        assert!(!wrapped.is_empty());
        for line in wrapped {
            assert!(line.len() <= 5);
        }
    }

    #[test]
    fn test_truncate_text_width_zero() {
        let result = truncate_text("test", 0);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_text_width_one() {
        let result = truncate_text("test", 1);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_text_unicode() {
        let result = truncate_text("你好世界", 6);
        // Should handle unicode correctly
        assert!(result.len() <= 10); // Each char is 3 bytes, so 6 chars max would be "你好..." (9 bytes)
    }

    #[test]
    fn test_render_card_very_long_id() {
        let mut issue = create_test_issue();
        issue.id = "VERY-LONG-PROJECT-IDENTIFIER-12345".to_string();
        
        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);
        
        assert!(!lines.is_empty());
        let line1_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line1_text.contains("VERY-LONG"));
    }

    #[test]
    fn test_render_card_empty_title() {
        let mut issue = create_test_issue();
        issue.title = String::new();
        
        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);
        
        // Should still render with empty title
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_card_many_labels() {
        let mut issue = create_test_issue();
        issue.labels = vec![
            "label1".to_string(),
            "label2".to_string(),
            "label3".to_string(),
            "label4".to_string(),
            "label5".to_string(),
        ];
        
        let config = KanbanCardConfig::default();
        let lines = render_kanban_card(&issue, &config, false);
        
        let line2_text: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        // Should show some labels and possibly a +N indicator
        assert!(line2_text.contains('#'));
    }

    #[test]
    fn test_render_card_max_width_variations() {
        let issue = create_test_issue();
        
        for width in [10, 20, 30, 50, 100] {
            let config = KanbanCardConfig::default().max_width(width);
            let lines = render_kanban_card(&issue, &config, false);
            
            // All widths should render successfully
            assert!(!lines.is_empty());
        }
    }

    #[test]
    fn test_render_card_priority_colors_comparison() {
        let issue = create_test_issue();
        
        let config_with = KanbanCardConfig::default();
        let mut config_without = KanbanCardConfig::default();
        config_without.priority_colors = false;
        
        let lines_with = render_kanban_card(&issue, &config_with, false);
        let lines_without = render_kanban_card(&issue, &config_without, false);
        
        // Both should render
        assert!(!lines_with.is_empty());
        assert!(!lines_without.is_empty());
        // Line counts should be the same
        assert_eq!(lines_with.len(), lines_without.len());
    }

    #[test]
    fn test_render_single_line_selected() {
        let issue = create_test_issue();
        let config = KanbanCardConfig::default().mode(CardMode::SingleLine);
        let lines = render_kanban_card(&issue, &config, true);
        
        assert_eq!(lines.len(), 1);
        // Should have padding for selected state
        let line_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line_text.len() >= config.max_width as usize);
    }

    #[test]
    fn test_render_two_line_very_long_title() {
        let mut issue = create_test_issue();
        issue.title = "This is a very long title that should wrap across multiple lines in two-line mode and test the wrapping functionality".to_string();
        
        let config = KanbanCardConfig::default()
            .mode(CardMode::TwoLine)
            .max_width(30);
        let lines = render_kanban_card(&issue, &config, false);
        
        // Should have multiple lines for wrapped title plus metadata
        assert!(lines.len() > 2);
    }

    #[test]
    fn test_render_card_all_metadata_combinations() {
        let base_issue = create_test_issue();
        
        // Test with assignee and labels
        let mut issue1 = base_issue.clone();
        issue1.assignee = Some("alice".to_string());
        issue1.labels = vec!["bug".to_string()];
        let config = KanbanCardConfig::default();
        let lines1 = render_kanban_card(&issue1, &config, false);
        let line2_1: String = lines1[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_1.contains("@alice"));
        assert!(line2_1.contains("#bug"));
        
        // Test with no assignee, with labels
        let mut issue2 = base_issue.clone();
        issue2.assignee = None;
        issue2.labels = vec!["feature".to_string()];
        let lines2 = render_kanban_card(&issue2, &config, false);
        let line2_2: String = lines2[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!line2_2.contains('@'));
        assert!(line2_2.contains("#feature"));
        
        // Test with assignee, no labels
        let mut issue3 = base_issue.clone();
        issue3.assignee = Some("bob".to_string());
        issue3.labels = vec![];
        let lines3 = render_kanban_card(&issue3, &config, false);
        let line2_3: String = lines3[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(line2_3.contains("@bob"));
        assert!(!line2_3.contains('#'));
        
        // Test with no assignee, no labels
        let mut issue4 = base_issue.clone();
        issue4.assignee = None;
        issue4.labels = vec![];
        let lines4 = render_kanban_card(&issue4, &config, false);
        let line2_4: String = lines4[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!line2_4.contains('@'));
        assert!(!line2_4.contains('#'));
    }

    #[test]
    fn test_render_card_custom_styles() {
        let issue = create_test_issue();
        
        let selected = Style::default().bg(Color::Magenta).fg(Color::White);
        let mut config = KanbanCardConfig::default();
        config.selected_style = selected;
        config.normal_style = Style::default().bg(Color::DarkGray);
        config.id_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        config.title_style = Style::default().fg(Color::Green);
        
        let lines = render_kanban_card(&issue, &config, false);
        
        // Should render with custom styles
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_wrap_text_trailing_spaces() {
        let wrapped = wrap_text("test  ", 10);
        // Should handle trailing spaces
        assert!(!wrapped.is_empty());
        assert!(wrapped[0].len() <= 10);
    }

    #[test]
    fn test_wrap_text_leading_spaces() {
        let wrapped = wrap_text("  test", 10);
        // Should handle leading spaces
        assert!(!wrapped.is_empty());
        assert!(wrapped[0].len() <= 10);
    }
}
