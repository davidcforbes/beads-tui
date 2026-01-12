//! Labels view for managing and viewing label usage

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

/// Label statistics
#[derive(Debug, Clone)]
pub struct LabelStats {
    /// Label name
    pub name: String,
    /// Number of issues with this label
    pub count: usize,
    /// Color for the label (optional)
    pub color: Option<Color>,
}

/// Labels view state for tracking selection and interaction
#[derive(Debug)]
pub struct LabelsViewState {
    list_state: ListState,
    search_query: String,
}

impl Default for LabelsViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelsViewState {
    /// Create a new labels view state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            search_query: String::new(),
        }
    }

    /// Get the list state
    pub fn list_state(&self) -> &ListState {
        &self.list_state
    }

    /// Get mutable list state
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    /// Get selected index
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Select next label
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

    /// Select previous label
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

    /// Get search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Set search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }
}

/// Labels view widget
pub struct LabelsView<'a> {
    labels: Vec<LabelStats>,
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> LabelsView<'a> {
    /// Create a new labels view
    pub fn new() -> Self {
        Self {
            labels: vec![],
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set labels with statistics
    pub fn labels(mut self, labels: Vec<LabelStats>) -> Self {
        self.labels = labels;
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    fn render_summary(&self, area: Rect, buf: &mut Buffer) {
        let total_labels = self.labels.len();
        let total_usage: usize = self.labels.iter().map(|l| l.count).sum();

        let summary_lines = vec![
            Line::from(Span::styled(
                "Label Summary",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Labels:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{total_labels}"),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Usage:   ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{total_usage}"), Style::default().fg(Color::Cyan)),
            ]),
        ];

        let summary = Paragraph::new(summary_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Summary")
                    .style(self.block_style),
            );

        summary.render(area, buf);
    }

    fn render_labels_list(&self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        let label_items: Vec<ListItem> = if self.labels.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No labels found",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            self.labels
                .iter()
                .map(|label_stat| {
                    let color = label_stat.color.unwrap_or(Color::White);
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            &label_stat.name,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("({})", label_stat.count),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]))
                })
                .collect()
        };

        let labels_list = List::new(label_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Labels ({})", self.labels.len()))
                    .style(self.block_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        StatefulWidget::render(labels_list, area, buf, state);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text =
            "a: Add Label | d: Delete Label | e: Edit Label | s: Show Statistics | /: Search";
        let help = Paragraph::new(Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )));
        help.render(area, buf);
    }
}

impl<'a> Default for LabelsView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for LabelsView<'a> {
    type State = LabelsViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout: summary (7) + labels list (fill) + help (1)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Summary
                Constraint::Min(5),    // Labels list
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render components
        self.render_summary(chunks[0], buf);
        self.render_labels_list(chunks[1], buf, &mut state.list_state);
        self.render_help(chunks[2], buf);
    }
}

/// Helper function to compute label statistics from issues
pub fn compute_label_stats<'a, I>(issues: I) -> Vec<LabelStats>
where
    I: IntoIterator<Item = &'a crate::beads::models::Issue>,
{
    let mut label_counts: HashMap<String, usize> = HashMap::new();

    for issue in issues {
        for label in &issue.labels {
            *label_counts.entry(label.clone()).or_insert(0) += 1;
        }
    }

    let mut stats: Vec<LabelStats> = label_counts
        .into_iter()
        .map(|(name, count)| LabelStats {
            name,
            count,
            color: None, // Could be enhanced to assign colors
        })
        .collect();

    stats.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue_with_labels(id: &str, labels: Vec<&str>) -> Issue {
        Issue {
            id: id.to_string(),
            title: "Test".to_string(),
            description: None,
            issue_type: IssueType::Task,
            status: IssueStatus::Open,
            priority: Priority::P2,
            labels: labels.into_iter().map(String::from).collect(),
            assignee: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
        }
    }

    #[test]
    fn test_labels_view_creation() {
        let view = LabelsView::new();
        assert_eq!(view.labels.len(), 0);
    }

    #[test]
    fn test_labels_view_with_labels() {
        let labels = vec![
            LabelStats {
                name: "bug".to_string(),
                count: 5,
                color: Some(Color::Red),
            },
            LabelStats {
                name: "feature".to_string(),
                count: 3,
                color: Some(Color::Green),
            },
        ];

        let view = LabelsView::new().labels(labels.clone());
        assert_eq!(view.labels.len(), 2);
        assert_eq!(view.labels[0].name, "bug");
        assert_eq!(view.labels[0].count, 5);
    }

    #[test]
    fn test_labels_view_block_style() {
        let style = Style::default().fg(Color::Red);
        let view = LabelsView::new().block_style(style);
        assert_eq!(view.block_style, style);
    }

    #[test]
    fn test_compute_label_stats_empty() {
        let issues: Vec<Issue> = vec![];
        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 0);
    }

    #[test]
    fn test_compute_label_stats() {
        let issues = vec![
            create_test_issue_with_labels("1", vec!["bug", "ui"]),
            create_test_issue_with_labels("2", vec!["bug", "backend"]),
            create_test_issue_with_labels("3", vec!["feature", "ui"]),
        ];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 4);

        // Should be sorted by count desc, then name asc
        assert_eq!(stats[0].name, "bug");
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[1].name, "ui");
        assert_eq!(stats[1].count, 2);
    }

    #[test]
    fn test_compute_label_stats_sorting() {
        let issues = vec![
            create_test_issue_with_labels("1", vec!["a"]),
            create_test_issue_with_labels("2", vec!["b"]),
            create_test_issue_with_labels("3", vec!["a"]),
            create_test_issue_with_labels("4", vec!["a"]),
        ];

        let stats = compute_label_stats(&issues);
        // "a" should come first (count=3), then "b" (count=1)
        assert_eq!(stats[0].name, "a");
        assert_eq!(stats[0].count, 3);
        assert_eq!(stats[1].name, "b");
        assert_eq!(stats[1].count, 1);
    }
}
