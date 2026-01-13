//! Labels view for managing and viewing label usage

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use crate::models::{normalize_label_key, split_label_dimension};
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
    /// Alternate names for the label
    pub aliases: Vec<String>,
    /// Optional dimension extracted from "dimension:value"
    pub dimension: Option<String>,
    /// Optional dimension value extracted from "dimension:value"
    pub value: Option<String>,
}

/// Labels view state for tracking selection and interaction
#[derive(Debug)]
pub struct LabelsViewState {
    list_state: ListState,
    search_query: String,
    search_cursor: usize,
    is_searching: bool,
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
            search_cursor: 0,
            is_searching: false,
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

    /// Check if search mode is active
    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
        self.search_cursor = 0;
        self.list_state.select(Some(0));
    }

    /// Stop search mode
    pub fn stop_search(&mut self) {
        self.is_searching = false;
        self.search_cursor = 0;
    }

    /// Set search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_cursor = query.len();
        self.search_query = query;
        self.list_state.select(Some(0));
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    /// Insert character into search query
    pub fn insert_search_char(&mut self, c: char) {
        if c == '\n' {
            return;
        }
        self.search_query.insert(self.search_cursor, c);
        self.search_cursor += 1;
        self.list_state.select(Some(0));
    }

    /// Delete character from search query
    pub fn delete_search_char(&mut self) {
        if self.search_cursor > 0 {
            self.search_query.remove(self.search_cursor - 1);
            self.search_cursor -= 1;
            self.list_state.select(Some(0));
        }
    }

    /// Get labels filtered by the current search query.
    pub fn filtered_labels<'a>(&self, labels: &'a [LabelStats]) -> Vec<&'a LabelStats> {
        filter_label_stats(labels, &self.search_query)
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
                Span::styled(format!("{total_labels}"), Style::default().fg(Color::Cyan)),
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

    fn render_search_bar(&self, area: Rect, buf: &mut Buffer, state: &LabelsViewState) {
        let title = if state.is_searching() {
            "Search (active)"
        } else {
            "Search"
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(self.block_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let text = if state.search_query().is_empty() {
            Span::styled(
                "Type to filter labels...",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
        } else {
            Span::raw(state.search_query())
        };

        Paragraph::new(Line::from(text)).render(inner, buf);
    }

    fn render_labels_list(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut ListState,
        labels: &[&LabelStats],
        total_count: usize,
        has_query: bool,
    ) {
        let label_items: Vec<ListItem> = if labels.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                if has_query {
                    "No labels match search"
                } else {
                    "No labels found"
                },
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            labels
                .iter()
                .map(|label_stat| {
                    let label_stat = *label_stat;
                    let color = label_stat.color.unwrap_or(Color::White);
                    let mut spans = vec![
                        Span::styled(
                            format!("[{}]", label_stat.name),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("({})", label_stat.count),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ];

                    if !label_stat.aliases.is_empty() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            format!(
                                "+{} alias{}",
                                label_stat.aliases.len(),
                                if label_stat.aliases.len() == 1 { "" } else { "es" }
                            ),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect()
        };

        let title = if total_count == labels.len() {
            format!("Labels ({})", total_count)
        } else {
            format!("Labels ({}/{})", labels.len(), total_count)
        };

        let labels_list = List::new(label_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
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
            "a: Add | d: Delete | e: Edit | s: Stats | /: Search | Esc: Clear";
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
        let show_search = state.is_searching() || !state.search_query().is_empty();
        let mut constraints = vec![
            Constraint::Length(7), // Summary
        ];
        if show_search {
            constraints.push(Constraint::Length(3)); // Search bar
        }
        constraints.push(Constraint::Min(5)); // Labels list
        constraints.push(Constraint::Length(1)); // Help

        // Create layout: summary + optional search + labels list + help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let filtered_labels = state.filtered_labels(&self.labels);
        if let Some(selected) = state.list_state.selected() {
            if selected >= filtered_labels.len() {
                state.list_state.select(if filtered_labels.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        } else if !filtered_labels.is_empty() {
            state.list_state.select(Some(0));
        }

        // Render components
        let mut chunk_index = 0;
        self.render_summary(chunks[chunk_index], buf);
        chunk_index += 1;

        if show_search {
            self.render_search_bar(chunks[chunk_index], buf, state);
            chunk_index += 1;
        }

        let has_query = !state.search_query().is_empty();
        self.render_labels_list(
            chunks[chunk_index],
            buf,
            &mut state.list_state,
            &filtered_labels,
            self.labels.len(),
            has_query,
        );
        chunk_index += 1;

        self.render_help(chunks[chunk_index], buf);
    }
}

fn color_for_label(name: &str) -> Color {
    const PALETTE: &[Color] = &[
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::Blue,
        Color::Red,
        Color::LightBlue,
        Color::LightMagenta,
    ];

    let mut hash: u64 = 0;
    for byte in name.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
    }
    let idx = (hash % PALETTE.len() as u64) as usize;
    PALETTE[idx]
}

fn dimension_parts(label: &str) -> (Option<String>, Option<String>) {
    split_label_dimension(label)
        .map(|(dimension, value)| (Some(dimension), Some(value)))
        .unwrap_or((None, None))
}

fn label_matches_query(label: &LabelStats, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_lowercase();
    let query_normalized = normalize_label_key(&query_lower);

    let mut candidates: Vec<String> = Vec::new();
    candidates.push(label.name.to_lowercase());
    if let Some(ref dimension) = label.dimension {
        candidates.push(dimension.to_lowercase());
    }
    if let Some(ref value) = label.value {
        candidates.push(value.to_lowercase());
    }
    if let (Some(dimension), Some(value)) = (label.dimension.as_ref(), label.value.as_ref())
    {
        candidates.push(format!("{}:{}", dimension.to_lowercase(), value.to_lowercase()));
    }
    for alias in &label.aliases {
        candidates.push(alias.to_lowercase());
    }

    candidates.into_iter().any(|candidate| {
        candidate.contains(&query_lower)
            || (!query_normalized.is_empty()
                && normalize_label_key(&candidate).contains(&query_normalized))
    })
}

fn filter_label_stats<'a>(labels: &'a [LabelStats], query: &str) -> Vec<&'a LabelStats> {
    labels
        .iter()
        .filter(|label| label_matches_query(label, query))
        .collect()
}

/// Helper function to compute label statistics from issues
pub fn compute_label_stats<'a, I>(issues: I) -> Vec<LabelStats>
where
    I: IntoIterator<Item = &'a crate::beads::models::Issue>,
{
    struct LabelAggregate {
        total: usize,
        counts: HashMap<String, usize>,
    }

    let mut aggregates: HashMap<String, LabelAggregate> = HashMap::new();

    for issue in issues {
        for label in &issue.labels {
            let normalized = normalize_label_key(label);
            let key = if normalized.is_empty() {
                label.to_lowercase()
            } else {
                normalized
            };
            let entry = aggregates.entry(key).or_insert_with(|| LabelAggregate {
                total: 0,
                counts: HashMap::new(),
            });
            entry.total += 1;
            *entry.counts.entry(label.clone()).or_insert(0) += 1;
        }
    }

    let mut stats: Vec<LabelStats> = aggregates.into_values().map(|aggregate| {
            let (canonical, _) = aggregate
                .counts
                .iter()
                .max_by(|(name_a, count_a), (name_b, count_b)| {
                    count_a.cmp(count_b).then_with(|| name_a.cmp(name_b))
                })
                .map(|(name, count)| (name.clone(), *count))
                .unwrap_or_else(|| ("".to_string(), 0));

            let mut aliases: Vec<String> = aggregate
                .counts
                .keys()
                .filter(|name| *name != &canonical)
                .cloned()
                .collect();
            aliases.sort();

            let (dimension, value) = dimension_parts(&canonical);

            LabelStats {
                name: canonical.clone(),
                count: aggregate.total,
                color: Some(color_for_label(&canonical)),
                aliases,
                dimension,
                value,
            }
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
                aliases: vec![],
                dimension: None,
                value: None,
            },
            LabelStats {
                name: "feature".to_string(),
                count: 3,
                color: Some(Color::Green),
                aliases: vec![],
                dimension: None,
                value: None,
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

    #[test]
    fn test_labels_view_state_new() {
        let state = LabelsViewState::new();
        assert_eq!(state.selected(), Some(0));
        assert!(state.search_query().is_empty());
    }

    #[test]
    fn test_labels_view_state_default() {
        let state = LabelsViewState::default();
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_labels_view_state_list_state() {
        let state = LabelsViewState::new();
        let list_state = state.list_state();
        assert_eq!(list_state.selected(), Some(0));
    }

    #[test]
    fn test_labels_view_state_list_state_mut() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(5));
        assert_eq!(state.selected(), Some(5));
    }

    #[test]
    fn test_select_next_wraparound() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(2));

        state.select_next(3); // len=3, current=2, should wrap to 0
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_select_next_middle() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(1));

        state.select_next(5); // len=5, current=1, should go to 2
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_select_next_empty_list() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(0));

        state.select_next(0); // Empty list
        assert_eq!(state.selected(), Some(0)); // Should remain unchanged
    }

    #[test]
    fn test_select_next_no_selection() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(None);

        state.select_next(5);
        assert_eq!(state.selected(), Some(0)); // Should select first
    }

    #[test]
    fn test_select_previous_wraparound() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(0));

        state.select_previous(3); // len=3, current=0, should wrap to 2
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_select_previous_middle() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(3));

        state.select_previous(5); // len=5, current=3, should go to 2
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_select_previous_empty_list() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(0));

        state.select_previous(0); // Empty list
        assert_eq!(state.selected(), Some(0)); // Should remain unchanged
    }

    #[test]
    fn test_select_previous_no_selection() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(None);

        state.select_previous(5);
        assert_eq!(state.selected(), Some(0)); // Should select first
    }

    #[test]
    fn test_search_query_getter() {
        let mut state = LabelsViewState::new();
        state.set_search_query("test".to_string());

        assert_eq!(state.search_query(), "test");
    }

    #[test]
    fn test_set_search_query() {
        let mut state = LabelsViewState::new();
        state.set_search_query("bug".to_string());

        assert_eq!(state.search_query, "bug");
    }

    #[test]
    fn test_clear_search() {
        let mut state = LabelsViewState::new();
        state.set_search_query("test".to_string());
        state.clear_search();

        assert!(state.search_query().is_empty());
    }

    #[test]
    fn test_labels_view_default() {
        let view = LabelsView::default();
        assert_eq!(view.labels.len(), 0);
    }

    #[test]
    fn test_labels_view_new_default_style() {
        let view = LabelsView::new();
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
    }

    #[test]
    fn test_labels_view_builder_chain() {
        let labels = vec![LabelStats {
            name: "test".to_string(),
            count: 1,
            color: None,
            aliases: vec![],
            dimension: None,
            value: None,
        }];
        let style = Style::default().fg(Color::Yellow);

        let view = LabelsView::new().labels(labels.clone()).block_style(style);

        assert_eq!(view.labels.len(), 1);
        assert_eq!(view.block_style, style);
    }

    #[test]
    fn test_label_stats_fields() {
        let stats = LabelStats {
            name: "bug".to_string(),
            count: 5,
            color: Some(Color::Red),
            aliases: vec![],
            dimension: None,
            value: None,
        };

        assert_eq!(stats.name, "bug");
        assert_eq!(stats.count, 5);
        assert_eq!(stats.color, Some(Color::Red));
    }

    #[test]
    fn test_label_stats_no_color() {
        let stats = LabelStats {
            name: "feature".to_string(),
            count: 3,
            color: None,
            aliases: vec![],
            dimension: None,
            value: None,
        };

        assert!(stats.color.is_none());
    }

    #[test]
    fn test_compute_label_stats_single_issue_multiple_labels() {
        let issues = vec![create_test_issue_with_labels("1", vec!["a", "b", "c"])];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 3);

        // All should have count=1
        for stat in &stats {
            assert_eq!(stat.count, 1);
        }
    }

    #[test]
    fn test_compute_label_stats_multiple_issues_same_labels() {
        let issues = vec![
            create_test_issue_with_labels("1", vec!["bug"]),
            create_test_issue_with_labels("2", vec!["bug"]),
            create_test_issue_with_labels("3", vec!["bug"]),
        ];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "bug");
        assert_eq!(stats[0].count, 3);
    }

    #[test]
    fn test_compute_label_stats_alias_grouping() {
        let issues = vec![
            create_test_issue_with_labels("1", vec!["bug-fix"]),
            create_test_issue_with_labels("2", vec!["bugfix"]),
        ];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].count, 2);
        // When counts are equal, alphabetically later label becomes canonical
        // "bugfix" > "bug-fix" alphabetically, so "bugfix" is canonical and "bug-fix" is alias
        assert_eq!(stats[0].name, "bugfix");
        assert!(stats[0].aliases.contains(&"bug-fix".to_string()));
    }

    #[test]
    fn test_compute_label_stats_issues_with_no_labels() {
        let issues = vec![
            create_test_issue_with_labels("1", vec![]),
            create_test_issue_with_labels("2", vec!["bug"]),
        ];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "bug");
    }

    #[test]
    fn test_compute_label_stats_alphabetical_secondary_sort() {
        let issues = vec![
            create_test_issue_with_labels("1", vec!["zebra"]),
            create_test_issue_with_labels("2", vec!["apple"]),
        ];

        let stats = compute_label_stats(&issues);
        // Both have count=1, should be sorted alphabetically
        assert_eq!(stats[0].name, "apple");
        assert_eq!(stats[1].name, "zebra");
    }

    #[test]
    fn test_select_next_single_item() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(0));

        state.select_next(1); // len=1, should stay at 0
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_select_previous_single_item() {
        let mut state = LabelsViewState::new();
        state.list_state_mut().select(Some(0));

        state.select_previous(1); // len=1, should stay at 0
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_labels_view_empty_labels() {
        let view = LabelsView::new().labels(vec![]);
        assert_eq!(view.labels.len(), 0);
    }

    #[test]
    fn test_compute_label_stats_color_set() {
        let issues = vec![create_test_issue_with_labels("1", vec!["test"])];

        let stats = compute_label_stats(&issues);
        assert_eq!(stats.len(), 1);
        assert!(stats[0].color.is_some());
    }

    #[test]
    fn test_filtered_labels_matches_alias_and_dimension() {
        let labels = vec![LabelStats {
            name: "bug-fix".to_string(),
            count: 2,
            color: Some(Color::Red),
            aliases: vec!["bugfix".to_string()],
            dimension: Some("state".to_string()),
            value: Some("patrol".to_string()),
        }];

        let mut state = LabelsViewState::new();
        state.set_search_query("bugfix".to_string());
        let filtered = state.filtered_labels(&labels);
        assert_eq!(filtered.len(), 1);

        state.set_search_query("state".to_string());
        let filtered = state.filtered_labels(&labels);
        assert_eq!(filtered.len(), 1);

        state.set_search_query("patrol".to_string());
        let filtered = state.filtered_labels(&labels);
        assert_eq!(filtered.len(), 1);
    }
}
