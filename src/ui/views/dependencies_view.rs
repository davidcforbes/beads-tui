//! Dependencies view showing issue dependencies and blocks relationships

use crate::beads::models::{Issue, IssueStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Get color for issue status
fn status_color(status: &IssueStatus) -> Color {
    match status {
        IssueStatus::Open => Color::Green,
        IssueStatus::InProgress => Color::Cyan,
        IssueStatus::Blocked => Color::Red,
        IssueStatus::Closed => Color::Gray,
    }
}

/// Which list is currently focused in the dependencies view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyFocus {
    Dependencies,
    Blocks,
}

/// Dependencies view state for tracking selection and focus
#[derive(Debug)]
pub struct DependenciesViewState {
    focus: DependencyFocus,
    dependencies_list_state: ListState,
    blocks_list_state: ListState,
}

impl Default for DependenciesViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl DependenciesViewState {
    /// Create a new dependencies view state
    pub fn new() -> Self {
        let mut dep_state = ListState::default();
        dep_state.select(Some(0));
        let mut blocks_state = ListState::default();
        blocks_state.select(Some(0));
        Self {
            focus: DependencyFocus::Dependencies,
            dependencies_list_state: dep_state,
            blocks_list_state: blocks_state,
        }
    }

    /// Get current focus
    pub fn focus(&self) -> DependencyFocus {
        self.focus
    }

    /// Toggle focus between dependencies and blocks
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            DependencyFocus::Dependencies => DependencyFocus::Blocks,
            DependencyFocus::Blocks => DependencyFocus::Dependencies,
        };
    }

    /// Get the selected dependency index
    pub fn selected_dependency(&self) -> Option<usize> {
        self.dependencies_list_state.selected()
    }

    /// Get the selected block index
    pub fn selected_block(&self) -> Option<usize> {
        self.blocks_list_state.selected()
    }

    /// Select next item in the focused list
    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let list_state = match self.focus {
            DependencyFocus::Dependencies => &mut self.dependencies_list_state,
            DependencyFocus::Blocks => &mut self.blocks_list_state,
        };

        let i = match list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    /// Select previous item in the focused list
    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let list_state = match self.focus {
            DependencyFocus::Dependencies => &mut self.dependencies_list_state,
            DependencyFocus::Blocks => &mut self.blocks_list_state,
        };

        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }
}

/// Dependencies view widget
pub struct DependenciesView<'a> {
    issue: Option<&'a Issue>,
    all_issues: Vec<&'a Issue>,
    block_style: Style,
}

impl<'a> DependenciesView<'a> {
    /// Create a new dependencies view
    pub fn new(all_issues: Vec<&'a Issue>) -> Self {
        Self {
            issue: None,
            all_issues,
            block_style: Style::default().fg(Color::Cyan),
        }
    }

    /// Set the selected issue to view dependencies for
    pub fn issue(mut self, issue: &'a Issue) -> Self {
        self.issue = Some(issue);
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    fn render_no_selection(&self, area: Rect, buf: &mut Buffer) {
        let message = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No issue selected",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Select an issue from the Issues view to view its dependencies",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dependencies")
                .style(self.block_style),
        );

        message.render(area, buf);
    }

    fn render_dependencies(
        &self,
        area: Rect,
        buf: &mut Buffer,
        issue: &Issue,
        state: &mut DependenciesViewState,
    ) {
        // Create layout: issue info (3) + dependencies (fill) + blocks (fill) + help (1)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Issue info
                Constraint::Min(5),    // Dependencies
                Constraint::Min(5),    // Blocks
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render issue info
        let info_text = format!("{} - {}", issue.id, issue.title);
        let info = Paragraph::new(info_text)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Selected Issue")
                    .style(self.block_style),
            );
        info.render(chunks[0], buf);

        // Render dependencies (what this issue depends on)
        let dep_items: Vec<ListItem> = if issue.dependencies.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No dependencies",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            issue
                .dependencies
                .iter()
                .filter_map(|dep_id| {
                    self.all_issues
                        .iter()
                        .find(|i| &i.id == dep_id)
                        .map(|dep_issue| {
                            ListItem::new(Line::from(vec![
                                Span::styled(&dep_issue.id, Style::default().fg(Color::Cyan)),
                                Span::raw(" - "),
                                Span::styled(&dep_issue.title, Style::default().fg(Color::White)),
                                Span::raw(" "),
                                Span::styled(
                                    format!("[{}]", dep_issue.status),
                                    Style::default().fg(status_color(&dep_issue.status)),
                                ),
                            ]))
                        })
                })
                .collect()
        };

        // Add focus indicator to title
        let dep_title = if state.focus == DependencyFocus::Dependencies {
            format!("> Depends On ({})", issue.dependencies.len())
        } else {
            format!("  Depends On ({})", issue.dependencies.len())
        };

        let dependencies = List::new(dep_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(dep_title)
                    .style(self.block_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(
            dependencies,
            chunks[1],
            buf,
            &mut state.dependencies_list_state,
        );

        // Render blocks (what this issue blocks)
        let block_items: Vec<ListItem> = if issue.blocks.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "Does not block any issues",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            issue
                .blocks
                .iter()
                .filter_map(|blocked_id| {
                    self.all_issues
                        .iter()
                        .find(|i| &i.id == blocked_id)
                        .map(|blocked_issue| {
                            ListItem::new(Line::from(vec![
                                Span::styled(&blocked_issue.id, Style::default().fg(Color::Yellow)),
                                Span::raw(" - "),
                                Span::styled(
                                    &blocked_issue.title,
                                    Style::default().fg(Color::White),
                                ),
                                Span::raw(" "),
                                Span::styled(
                                    format!("[{}]", blocked_issue.status),
                                    Style::default().fg(status_color(&blocked_issue.status)),
                                ),
                            ]))
                        })
                })
                .collect()
        };

        // Add focus indicator to title
        let blocks_title = if state.focus == DependencyFocus::Blocks {
            format!("> Blocks ({})", issue.blocks.len())
        } else {
            format!("  Blocks ({})", issue.blocks.len())
        };

        let blocks = List::new(block_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(blocks_title)
                    .style(self.block_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(blocks, chunks[2], buf, &mut state.blocks_list_state);

        // Render help
        let help_text = "Up/Down/j/k: Navigate | Tab: Focus | a: Add | d: Remove | g: Graph | c: Cycle check | Enter: View | Esc: Back";
        let help = Paragraph::new(Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )));
        help.render(chunks[3], buf);
    }
}

impl<'a> StatefulWidget for DependenciesView<'a> {
    type State = DependenciesViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match self.issue {
            Some(issue) => self.render_dependencies(area, buf, issue, state),
            None => self.render_no_selection(area, buf),
        }
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
            description: Some("Test description".to_string()),
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
        }
    }

    #[test]
    fn test_dependencies_view_creation() {
        let issues = vec![];
        let view = DependenciesView::new(issues);
        assert!(view.issue.is_none());
    }

    #[test]
    fn test_dependencies_view_with_issue() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];

        let view = DependenciesView::new(issues).issue(&issue1);
        assert!(view.issue.is_some());
    }

    #[test]
    fn test_dependencies_view_block_style() {
        let issues = vec![];
        let style = Style::default().fg(Color::Red);
        let view = DependenciesView::new(issues).block_style(style);
        assert_eq!(view.block_style, style);
    }

    // status_color tests
    #[test]
    fn test_status_color_open() {
        assert_eq!(status_color(&IssueStatus::Open), Color::Green);
    }

    #[test]
    fn test_status_color_in_progress() {
        assert_eq!(status_color(&IssueStatus::InProgress), Color::Cyan);
    }

    #[test]
    fn test_status_color_blocked() {
        assert_eq!(status_color(&IssueStatus::Blocked), Color::Red);
    }

    #[test]
    fn test_status_color_closed() {
        assert_eq!(status_color(&IssueStatus::Closed), Color::Gray);
    }

    // DependencyFocus tests
    #[test]
    fn test_dependency_focus_equality() {
        assert_eq!(DependencyFocus::Dependencies, DependencyFocus::Dependencies);
        assert_eq!(DependencyFocus::Blocks, DependencyFocus::Blocks);
        assert_ne!(DependencyFocus::Dependencies, DependencyFocus::Blocks);
    }

    // DependenciesViewState tests
    #[test]
    fn test_dependencies_view_state_new() {
        let state = DependenciesViewState::new();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
        assert_eq!(state.selected_dependency(), Some(0));
        assert_eq!(state.selected_block(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_default() {
        let state = DependenciesViewState::default();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
        assert_eq!(state.selected_dependency(), Some(0));
        assert_eq!(state.selected_block(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_toggle_focus() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Blocks);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
    }

    #[test]
    fn test_dependencies_view_state_select_next_empty_list() {
        let mut state = DependenciesViewState::new();
        state.select_next(0); // Empty list
                              // Should not panic or change state
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_previous_empty_list() {
        let mut state = DependenciesViewState::new();
        state.select_previous(0); // Empty list
                                  // Should not panic or change state
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_next_dependencies() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
        assert_eq!(state.selected_dependency(), Some(0));

        state.select_next(3);
        assert_eq!(state.selected_dependency(), Some(1));

        state.select_next(3);
        assert_eq!(state.selected_dependency(), Some(2));

        // Wraparound
        state.select_next(3);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_previous_dependencies() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
        assert_eq!(state.selected_dependency(), Some(0));

        // Wraparound to end
        state.select_previous(3);
        assert_eq!(state.selected_dependency(), Some(2));

        state.select_previous(3);
        assert_eq!(state.selected_dependency(), Some(1));

        state.select_previous(3);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_next_blocks() {
        let mut state = DependenciesViewState::new();
        state.toggle_focus(); // Switch to Blocks
        assert_eq!(state.focus(), DependencyFocus::Blocks);
        assert_eq!(state.selected_block(), Some(0));

        state.select_next(3);
        assert_eq!(state.selected_block(), Some(1));

        state.select_next(3);
        assert_eq!(state.selected_block(), Some(2));

        // Wraparound
        state.select_next(3);
        assert_eq!(state.selected_block(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_previous_blocks() {
        let mut state = DependenciesViewState::new();
        state.toggle_focus(); // Switch to Blocks
        assert_eq!(state.focus(), DependencyFocus::Blocks);
        assert_eq!(state.selected_block(), Some(0));

        // Wraparound to end
        state.select_previous(3);
        assert_eq!(state.selected_block(), Some(2));

        state.select_previous(3);
        assert_eq!(state.selected_block(), Some(1));

        state.select_previous(3);
        assert_eq!(state.selected_block(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_independent_list_selections() {
        let mut state = DependenciesViewState::new();

        // Navigate in dependencies list
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
        state.select_next(5);
        state.select_next(5);
        assert_eq!(state.selected_dependency(), Some(2));

        // Toggle to blocks and navigate
        state.toggle_focus();
        state.select_next(5);
        assert_eq!(state.selected_block(), Some(1));

        // Toggle back - dependencies selection should be preserved
        state.toggle_focus();
        assert_eq!(state.selected_dependency(), Some(2));
    }

    #[test]
    fn test_dependencies_view_state_select_next_single_item() {
        let mut state = DependenciesViewState::new();
        state.select_next(1);
        // With single item, should wrap from 0 to 0
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_dependencies_view_state_select_previous_single_item() {
        let mut state = DependenciesViewState::new();
        state.select_previous(1);
        // With single item, should wrap from 0 to 0
        assert_eq!(state.selected_dependency(), Some(0));
    }

    // DependenciesView widget tests
    #[test]
    fn test_dependencies_view_new_empty() {
        let issues: Vec<&Issue> = vec![];
        let view = DependenciesView::new(issues);
        assert!(view.issue.is_none());
        assert_eq!(view.all_issues.len(), 0);
    }

    #[test]
    fn test_dependencies_view_new_with_issues() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];

        let view = DependenciesView::new(issues);
        assert!(view.issue.is_none());
        assert_eq!(view.all_issues.len(), 2);
    }

    #[test]
    fn test_dependencies_view_builder_chain() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];
        let style = Style::default().fg(Color::Magenta);

        let view = DependenciesView::new(issues)
            .issue(&issue1)
            .block_style(style);

        assert!(view.issue.is_some());
        assert_eq!(view.block_style, style);
    }

    #[test]
    fn test_dependencies_view_issue_with_dependencies() {
        let mut issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");

        // Issue 1 depends on Issue 2
        issue1.dependencies = vec!["beads-002".to_string()];

        let issues = vec![&issue1, &issue2, &issue3];
        let view = DependenciesView::new(issues).issue(&issue1);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().id, "beads-001");
        assert_eq!(view.issue.unwrap().dependencies.len(), 1);
    }

    #[test]
    fn test_dependencies_view_issue_with_blocks() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");

        // Issue 2 blocks Issue 1 and Issue 3
        issue2.blocks = vec!["beads-001".to_string(), "beads-003".to_string()];

        let issues = vec![&issue1, &issue2, &issue3];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().id, "beads-002");
        assert_eq!(view.issue.unwrap().blocks.len(), 2);
    }

    #[test]
    fn test_dependencies_view_issue_with_no_relationships() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");

        let issues = vec![&issue1, &issue2];
        let view = DependenciesView::new(issues).issue(&issue1);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 0);
        assert_eq!(view.issue.unwrap().blocks.len(), 0);
    }

    #[test]
    fn test_dependencies_view_default_block_style() {
        let issues: Vec<&Issue> = vec![];
        let view = DependenciesView::new(issues);
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
    }

    // Copy/Clone trait tests
    #[test]
    fn test_dependency_focus_copy_trait() {
        let focus1 = DependencyFocus::Dependencies;
        let focus2 = focus1;
        assert_eq!(focus1, focus2);
        // Both should still be usable after copy
        assert_eq!(focus1, DependencyFocus::Dependencies);
        assert_eq!(focus2, DependencyFocus::Dependencies);
    }

    #[test]
    fn test_dependency_focus_clone_trait() {
        let focus1 = DependencyFocus::Blocks;
        let focus2 = focus1;
        assert_eq!(focus1, focus2);
        assert_eq!(focus1, DependencyFocus::Blocks);
        assert_eq!(focus2, DependencyFocus::Blocks);
    }

    // Multiple toggle operations
    #[test]
    fn test_toggle_focus_multiple_times() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Blocks);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Blocks);

        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Dependencies);
    }

    // Edge case: Selection wraparound at exact boundary
    #[test]
    fn test_select_next_wraparound_at_boundary() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.selected_dependency(), Some(0));

        // Navigate to last item (index 4 in list of 5)
        for _ in 0..4 {
            state.select_next(5);
        }
        assert_eq!(state.selected_dependency(), Some(4));

        // Next should wrap to 0
        state.select_next(5);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_select_previous_wraparound_at_boundary() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.selected_dependency(), Some(0));

        // Previous from 0 should wrap to last item
        state.select_previous(5);
        assert_eq!(state.selected_dependency(), Some(4));

        // Previous should go to 3
        state.select_previous(5);
        assert_eq!(state.selected_dependency(), Some(3));
    }

    // State consistency across multiple focus changes with navigation
    #[test]
    fn test_state_consistency_complex_navigation() {
        let mut state = DependenciesViewState::new();

        // Navigate in dependencies
        state.select_next(10);
        state.select_next(10);
        state.select_next(10);
        assert_eq!(state.selected_dependency(), Some(3));

        // Switch to blocks and navigate
        state.toggle_focus();
        state.select_next(10);
        state.select_next(10);
        assert_eq!(state.selected_block(), Some(2));

        // Switch back and verify dependencies preserved
        state.toggle_focus();
        assert_eq!(state.selected_dependency(), Some(3));
        assert_eq!(state.selected_block(), Some(2));

        // Continue navigating dependencies
        state.select_previous(10);
        assert_eq!(state.selected_dependency(), Some(2));

        // Switch to blocks and verify preserved
        state.toggle_focus();
        assert_eq!(state.selected_block(), Some(2));
    }

    // Builder pattern variations
    #[test]
    fn test_builder_pattern_chain_order_independence() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];
        let style = Style::default().fg(Color::Yellow);

        // Chain in one order
        let view1 = DependenciesView::new(issues.clone())
            .block_style(style)
            .issue(&issue1);

        // Chain in different order
        let view2 = DependenciesView::new(issues)
            .issue(&issue1)
            .block_style(style);

        assert_eq!(view1.block_style, view2.block_style);
        assert!(view1.issue.is_some());
        assert!(view2.issue.is_some());
    }

    #[test]
    fn test_builder_multiple_style_applications() {
        let issues: Vec<&Issue> = vec![];
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);
        let style3 = Style::default().fg(Color::Green);

        let view = DependenciesView::new(issues)
            .block_style(style1)
            .block_style(style2)
            .block_style(style3);

        // Last style should win
        assert_eq!(view.block_style, style3);
    }

    #[test]
    fn test_builder_multiple_issue_applications() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");
        let issues = vec![&issue1, &issue2, &issue3];

        let view = DependenciesView::new(issues)
            .issue(&issue1)
            .issue(&issue2)
            .issue(&issue3);

        // Last issue should win
        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().id, "beads-003");
    }

    // Missing dependency/block IDs
    #[test]
    fn test_issue_with_missing_dependency_ids() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");

        // Issue 2 depends on non-existent issues
        issue2.dependencies = vec!["beads-999".to_string(), "beads-888".to_string()];

        let issues = vec![&issue1, &issue2];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        // filter_map in render should handle missing IDs gracefully
        assert_eq!(view.issue.unwrap().dependencies.len(), 2);
    }

    #[test]
    fn test_issue_with_missing_block_ids() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");

        // Issue 2 blocks non-existent issues
        issue2.blocks = vec!["beads-777".to_string(), "beads-666".to_string()];

        let issues = vec![&issue1, &issue2];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        // filter_map in render should handle missing IDs gracefully
        assert_eq!(view.issue.unwrap().blocks.len(), 2);
    }

    #[test]
    fn test_issue_with_partial_missing_dependencies() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let mut issue3 = create_test_issue("beads-003", "Issue 3");

        // Issue 3 depends on Issue 1 (exists) and Issue 999 (missing)
        issue3.dependencies = vec!["beads-001".to_string(), "beads-999".to_string()];

        let issues = vec![&issue1, &issue2, &issue3];
        let view = DependenciesView::new(issues).issue(&issue3);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 2);
    }

    // Different issue status combinations
    #[test]
    fn test_dependencies_with_different_statuses() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let mut issue3 = create_test_issue("beads-003", "Issue 3");
        let mut issue4 = create_test_issue("beads-004", "Issue 4");
        let mut issue5 = create_test_issue("beads-005", "Issue 5");

        issue2.status = IssueStatus::InProgress;
        issue3.status = IssueStatus::Blocked;
        issue4.status = IssueStatus::Closed;
        issue5.dependencies = vec![
            "beads-001".to_string(),
            "beads-002".to_string(),
            "beads-003".to_string(),
            "beads-004".to_string(),
        ];

        let issues = vec![&issue1, &issue2, &issue3, &issue4, &issue5];
        let view = DependenciesView::new(issues).issue(&issue5);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 4);
    }

    #[test]
    fn test_blocks_with_different_statuses() {
        let mut issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let mut issue3 = create_test_issue("beads-003", "Issue 3");
        let mut issue4 = create_test_issue("beads-004", "Issue 4");
        let mut issue5 = create_test_issue("beads-005", "Issue 5");

        issue1.status = IssueStatus::InProgress;
        issue2.status = IssueStatus::Blocked;
        issue3.status = IssueStatus::Closed;
        issue4.status = IssueStatus::Open;

        issue5.blocks = vec![
            "beads-001".to_string(),
            "beads-002".to_string(),
            "beads-003".to_string(),
            "beads-004".to_string(),
        ];

        let issues = vec![&issue1, &issue2, &issue3, &issue4, &issue5];
        let view = DependenciesView::new(issues).issue(&issue5);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().blocks.len(), 4);
    }

    // Default vs new equivalence
    #[test]
    fn test_default_equals_new() {
        let state1 = DependenciesViewState::default();
        let state2 = DependenciesViewState::new();

        assert_eq!(state1.focus(), state2.focus());
        assert_eq!(state1.selected_dependency(), state2.selected_dependency());
        assert_eq!(state1.selected_block(), state2.selected_block());
    }

    // Empty vs populated lists
    #[test]
    fn test_both_lists_empty() {
        let issue = create_test_issue("beads-001", "Issue 1");
        let issues = vec![&issue];
        let view = DependenciesView::new(issues).issue(&issue);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 0);
        assert_eq!(view.issue.unwrap().blocks.len(), 0);
    }

    #[test]
    fn test_dependencies_populated_blocks_empty() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");

        issue2.dependencies = vec!["beads-001".to_string(), "beads-003".to_string()];

        let issues = vec![&issue1, &issue2, &issue3];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 2);
        assert_eq!(view.issue.unwrap().blocks.len(), 0);
    }

    #[test]
    fn test_dependencies_empty_blocks_populated() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");

        issue2.blocks = vec!["beads-001".to_string(), "beads-003".to_string()];

        let issues = vec![&issue1, &issue2, &issue3];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 0);
        assert_eq!(view.issue.unwrap().blocks.len(), 2);
    }

    #[test]
    fn test_both_lists_populated() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let mut issue2 = create_test_issue("beads-002", "Issue 2");
        let issue3 = create_test_issue("beads-003", "Issue 3");
        let issue4 = create_test_issue("beads-004", "Issue 4");

        issue2.dependencies = vec!["beads-001".to_string()];
        issue2.blocks = vec!["beads-003".to_string(), "beads-004".to_string()];

        let issues = vec![&issue1, &issue2, &issue3, &issue4];
        let view = DependenciesView::new(issues).issue(&issue2);

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 1);
        assert_eq!(view.issue.unwrap().blocks.len(), 2);
    }

    // All DependencyFocus inequality variants
    #[test]
    fn test_all_dependency_focus_inequalities() {
        assert_ne!(DependencyFocus::Dependencies, DependencyFocus::Blocks);
        assert_ne!(DependencyFocus::Blocks, DependencyFocus::Dependencies);
    }

    // Large number of dependencies/blocks
    #[test]
    fn test_large_number_of_dependencies() {
        let mut issues_vec: Vec<Issue> = vec![];
        for i in 0..100 {
            issues_vec.push(create_test_issue(
                &format!("beads-{:03}", i),
                &format!("Issue {}", i),
            ));
        }

        let mut main_issue = create_test_issue("beads-main", "Main Issue");
        main_issue.dependencies = (0..100).map(|i| format!("beads-{:03}", i)).collect();

        issues_vec.push(main_issue);

        let issue_refs: Vec<&Issue> = issues_vec.iter().collect();
        let view = DependenciesView::new(issue_refs).issue(issues_vec.last().unwrap());

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().dependencies.len(), 100);
    }

    #[test]
    fn test_large_number_of_blocks() {
        let mut issues_vec: Vec<Issue> = vec![];
        for i in 0..100 {
            issues_vec.push(create_test_issue(
                &format!("beads-{:03}", i),
                &format!("Issue {}", i),
            ));
        }

        let mut main_issue = create_test_issue("beads-main", "Main Issue");
        main_issue.blocks = (0..100).map(|i| format!("beads-{:03}", i)).collect();

        issues_vec.push(main_issue);

        let issue_refs: Vec<&Issue> = issues_vec.iter().collect();
        let view = DependenciesView::new(issue_refs).issue(issues_vec.last().unwrap());

        assert!(view.issue.is_some());
        assert_eq!(view.issue.unwrap().blocks.len(), 100);
    }

    // Select operations with exact boundary values
    #[test]
    fn test_select_next_at_exact_last_index() {
        let mut state = DependenciesViewState::new();

        // Navigate to exact last index
        for _ in 0..9 {
            state.select_next(10);
        }
        assert_eq!(state.selected_dependency(), Some(9));

        // Next should wrap to 0
        state.select_next(10);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_select_previous_from_first_index() {
        let mut state = DependenciesViewState::new();
        assert_eq!(state.selected_dependency(), Some(0));

        // Previous from first should wrap to last
        state.select_previous(10);
        assert_eq!(state.selected_dependency(), Some(9));
    }

    // Focus-specific navigation doesn't affect other list
    #[test]
    fn test_navigation_only_affects_focused_list() {
        let mut state = DependenciesViewState::new();

        // Navigate dependencies
        state.select_next(5);
        state.select_next(5);
        assert_eq!(state.selected_dependency(), Some(2));
        assert_eq!(state.selected_block(), Some(0)); // Blocks unchanged

        // Switch to blocks
        state.toggle_focus();

        // Navigate blocks
        state.select_next(5);
        state.select_next(5);
        state.select_next(5);
        assert_eq!(state.selected_dependency(), Some(2)); // Dependencies unchanged
        assert_eq!(state.selected_block(), Some(3));
    }

    // Debug trait tests
    #[test]
    fn test_dependency_focus_debug() {
        let focus = DependencyFocus::Dependencies;
        let debug_str = format!("{:?}", focus);
        assert_eq!(debug_str, "Dependencies");

        let focus = DependencyFocus::Blocks;
        let debug_str = format!("{:?}", focus);
        assert_eq!(debug_str, "Blocks");
    }

    #[test]
    fn test_dependencies_view_state_debug() {
        let state = DependenciesViewState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("DependenciesViewState"));
    }

    // Clone and Copy trait tests
    #[test]
    fn test_dependency_focus_clone() {
        let focus = DependencyFocus::Dependencies;
        let cloned = focus;
        assert_eq!(focus, cloned);
    }

    #[test]
    fn test_dependency_focus_copy() {
        let focus = DependencyFocus::Blocks;
        let copied = focus;
        assert_eq!(focus, copied);
    }

    // Widget rendering tests
    #[test]
    fn test_render_no_selection_widget() {
        let issues = vec![];
        let view = DependenciesView::new(issues);
        let mut state = DependenciesViewState::new();
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        view.render(area, &mut buffer, &mut state);

        // Buffer should be modified
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content for no selection");
    }

    #[test]
    fn test_render_with_issue() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];

        let view = DependenciesView::new(issues).issue(&issue1);
        let mut state = DependenciesViewState::new();
        let area = Rect::new(0, 0, 100, 30);
        let mut buffer = Buffer::empty(area);

        view.render(area, &mut buffer, &mut state);

        // Buffer should be modified
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content with issue");
    }

    #[test]
    fn test_render_small_area() {
        let issue = create_test_issue("beads-001", "Issue 1");
        let issues = vec![&issue];
        let view = DependenciesView::new(issues).issue(&issue);
        let mut state = DependenciesViewState::new();
        let area = Rect::new(0, 0, 20, 10);
        let mut buffer = Buffer::empty(area);

        // Should handle small areas gracefully
        view.render(area, &mut buffer, &mut state);

        // Should not panic
    }

    #[test]
    fn test_render_with_custom_style() {
        let issue = create_test_issue("beads-001", "Issue 1");
        let issues = vec![&issue];
        let style = Style::default().fg(Color::Yellow);
        let view = DependenciesView::new(issues)
            .issue(&issue)
            .block_style(style);
        let mut state = DependenciesViewState::new();
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        view.render(area, &mut buffer, &mut state);

        // Should render without panic
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content);
    }

    // Builder pattern tests
    #[test]
    fn test_builder_chaining() {
        let issue = create_test_issue("beads-001", "Issue 1");
        let issues = vec![&issue];
        let style = Style::default().fg(Color::Magenta);

        let view = DependenciesView::new(issues)
            .issue(&issue)
            .block_style(style);

        assert!(view.issue.is_some());
        assert_eq!(view.block_style, style);
    }

    // All status colors test
    #[test]
    fn test_all_status_colors() {
        let statuses = vec![
            (IssueStatus::Open, Color::Green),
            (IssueStatus::InProgress, Color::Cyan),
            (IssueStatus::Blocked, Color::Red),
            (IssueStatus::Closed, Color::Gray),
        ];

        for (status, expected_color) in statuses {
            assert_eq!(status_color(&status), expected_color);
        }
    }

    // State initialization tests
    #[test]
    fn test_new_state_has_both_lists_initialized() {
        let state = DependenciesViewState::new();

        // Both lists should have selection initialized
        assert!(state.selected_dependency().is_some());
        assert!(state.selected_block().is_some());
    }

    // Edge case: navigation with length 1
    #[test]
    fn test_select_next_with_single_item() {
        let mut state = DependenciesViewState::new();

        // With only one item, next should stay at 0
        state.select_next(1);
        assert_eq!(state.selected_dependency(), Some(0));

        state.select_next(1);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    #[test]
    fn test_select_previous_with_single_item() {
        let mut state = DependenciesViewState::new();

        // With only one item, previous should stay at 0
        state.select_previous(1);
        assert_eq!(state.selected_dependency(), Some(0));

        state.select_previous(1);
        assert_eq!(state.selected_dependency(), Some(0));
    }

    // Navigation in blocks list
    #[test]
    fn test_blocks_navigation() {
        let mut state = DependenciesViewState::new();

        // Switch to blocks
        state.toggle_focus();
        assert_eq!(state.focus(), DependencyFocus::Blocks);

        // Navigate in blocks
        state.select_next(5);
        assert_eq!(state.selected_block(), Some(1));

        state.select_next(5);
        assert_eq!(state.selected_block(), Some(2));

        state.select_previous(5);
        assert_eq!(state.selected_block(), Some(1));

        // Dependencies should be unchanged
        assert_eq!(state.selected_dependency(), Some(0));
    }

    // View with empty issues list
    #[test]
    fn test_view_with_empty_issues_list() {
        let issues = vec![];
        let view = DependenciesView::new(issues);

        assert!(view.issue.is_none());
        assert!(view.all_issues.is_empty());
    }

    // Multiple calls to block_style
    #[test]
    fn test_block_style_can_be_changed() {
        let issues = vec![];
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);

        let view = DependenciesView::new(issues)
            .block_style(style1)
            .block_style(style2);

        assert_eq!(view.block_style, style2);
    }
}
