//! Search interface view with search input and results

use crate::beads::models::Issue;
use crate::ui::widgets::{IssueList, IssueListState, SearchInput, SearchInputState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Search interface state
#[derive(Debug)]
pub struct SearchInterfaceState {
    search_state: SearchInputState,
    list_state: IssueListState,
    all_issues: Vec<Issue>,
    filtered_issues: Vec<Issue>,
    search_scope: SearchScope,
    show_help: bool,
}

/// Search scope for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    /// Search in title only
    Title,
    /// Search in description only
    Description,
    /// Search in notes only
    Notes,
    /// Search in all fields
    All,
}

impl SearchScope {
    /// Get display name for the scope
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Description => "Description",
            Self::Notes => "Notes",
            Self::All => "All",
        }
    }

    /// Get all scopes
    pub fn all() -> Vec<SearchScope> {
        vec![Self::Title, Self::Description, Self::Notes, Self::All]
    }
}

impl SearchInterfaceState {
    /// Create a new search interface state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            search_state: SearchInputState::new(),
            list_state: IssueListState::new(),
            all_issues: issues.clone(),
            filtered_issues: issues,
            search_scope: SearchScope::All,
            show_help: true,
        }
    }

    /// Get search input state
    pub fn search_state(&self) -> &SearchInputState {
        &self.search_state
    }

    /// Get mutable search input state
    pub fn search_state_mut(&mut self) -> &mut SearchInputState {
        &mut self.search_state
    }

    /// Get issue list state
    pub fn list_state(&self) -> &IssueListState {
        &self.list_state
    }

    /// Get mutable issue list state
    pub fn list_state_mut(&mut self) -> &mut IssueListState {
        &mut self.list_state
    }

    /// Get filtered issues
    pub fn filtered_issues(&self) -> &[Issue] {
        &self.filtered_issues
    }

    /// Get search scope
    pub fn search_scope(&self) -> SearchScope {
        self.search_scope
    }

    /// Set search scope
    pub fn set_search_scope(&mut self, scope: SearchScope) {
        self.search_scope = scope;
        self.update_filtered_issues();
    }

    /// Cycle to next search scope
    pub fn next_search_scope(&mut self) {
        self.search_scope = match self.search_scope {
            SearchScope::Title => SearchScope::Description,
            SearchScope::Description => SearchScope::Notes,
            SearchScope::Notes => SearchScope::All,
            SearchScope::All => SearchScope::Title,
        };
        self.update_filtered_issues();
    }

    /// Set all issues
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.all_issues = issues;
        self.update_filtered_issues();
    }

    /// Update filtered issues based on search query, scope, and column filters
    pub fn update_filtered_issues(&mut self) {
        let query = self.search_state.query().to_lowercase();
        let column_filters = self.list_state.column_filters();
        let filters_enabled = self.list_state.filters_enabled();

        self.filtered_issues = self
            .all_issues
            .iter()
            .filter(|issue| {
                // First apply search query filter
                let matches_search = if query.is_empty() {
                    true
                } else {
                    self.matches_query(issue, &query)
                };

                // Then apply column filters if enabled
                let matches_filters = if !filters_enabled {
                    true
                } else {
                    column_filters.matches(issue)
                };

                matches_search && matches_filters
            })
            .cloned()
            .collect();

        // Reset selection if out of bounds
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_issues.len() {
                self.list_state.select(if self.filtered_issues.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        }
    }

    /// Check if an issue matches the search query based on scope
    fn matches_query(&self, issue: &Issue, query: &str) -> bool {
        match self.search_scope {
            SearchScope::Title => issue.title.to_lowercase().contains(query),
            SearchScope::Description => {
                if let Some(ref desc) = issue.description {
                    desc.to_lowercase().contains(query)
                } else {
                    false
                }
            }
            SearchScope::Notes => issue
                .notes
                .iter()
                .any(|note| note.content.to_lowercase().contains(query)),
            SearchScope::All => {
                issue.title.to_lowercase().contains(query)
                    || issue
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(query))
                        .unwrap_or(false)
                    || issue.id.to_lowercase().contains(query)
                    || issue
                        .assignee
                        .as_ref()
                        .map(|a| a.to_lowercase().contains(query))
                        .unwrap_or(false)
                    || issue
                        .labels
                        .iter()
                        .any(|l| l.to_lowercase().contains(query))
                    || issue
                        .notes
                        .iter()
                        .any(|note| note.content.to_lowercase().contains(query))
            }
        }
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Check if help is visible
    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }

    /// Get selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        let index = self.list_state.selected()?;
        self.filtered_issues.get(index)
    }

    /// Get number of results
    pub fn result_count(&self) -> usize {
        self.filtered_issues.len()
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_state.clear();
        self.update_filtered_issues();
    }
}

/// Search interface view widget
pub struct SearchInterfaceView<'a> {
    block_style: Style,
    help_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> SearchInterfaceView<'a> {
    /// Create a new search interface view
    pub fn new() -> Self {
        Self {
            block_style: Style::default().fg(Color::Cyan),
            help_style: Style::default().fg(Color::DarkGray),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Set help style
    pub fn help_style(mut self, style: Style) -> Self {
        self.help_style = style;
        self
    }

    fn render_search_bar(&self, area: Rect, buf: &mut Buffer, state: &mut SearchInterfaceState) {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Search [{}]", state.search_scope().display_name()))
            .style(self.block_style);

        let search_input = SearchInput::new().block(search_block);

        StatefulWidget::render(search_input, area, buf, &mut state.search_state);
    }

    fn render_results_info(&self, area: Rect, buf: &mut Buffer, state: &SearchInterfaceState) {
        let total = state.all_issues.len();
        let filtered = state.result_count();

        let info_text = if state.search_state().query().is_empty() {
            format!("Showing all {total} issues")
        } else {
            format!("Found {filtered} of {total} issues")
        };

        let line = Line::from(vec![
            Span::styled(info_text, Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("[Scope: {}]", state.search_scope().display_name()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }

    fn render_help_bar(&self, area: Rect, buf: &mut Buffer) {
        let help_text =
            "/ Focus Search | Tab: Cycle Scope | Esc: Clear | j/k: Navigate | Enter: View | ?:Toggle Help";

        let line = Line::from(Span::styled(help_text, self.help_style));
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

impl<'a> Default for SearchInterfaceView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for SearchInterfaceView<'a> {
    type State = SearchInterfaceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout: search bar (3) + info (1) + results (fill) + help (1 if visible)
        let mut constraints = vec![
            Constraint::Length(3), // Search bar
            Constraint::Length(1), // Results info
            Constraint::Min(5),    // Results list
        ];

        // Help bar (if visible)
        if state.is_help_visible() {
            constraints.push(Constraint::Length(1));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_idx = 0;

        // Render search bar
        self.render_search_bar(chunks[chunk_idx], buf, state);
        chunk_idx += 1;

        // Render results info
        self.render_results_info(chunks[chunk_idx], buf, state);
        chunk_idx += 1;

        // Render results list
        let issue_refs: Vec<&Issue> = state.filtered_issues.iter().collect();
        let search_query = if !state.search_state.query().is_empty() {
            Some(state.search_state.query().to_string())
        } else {
            None
        };
        let issue_list = IssueList::new(issue_refs).search_query(search_query);

        StatefulWidget::render(issue_list, chunks[chunk_idx], buf, &mut state.list_state);
        chunk_idx += 1;

        // Render help bar if visible
        if state.is_help_visible() {
            self.render_help_bar(chunks[chunk_idx], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issues() -> Vec<Issue> {
        vec![
            Issue {
                id: "beads-001".to_string(),
                title: "Implement search feature".to_string(),
                description: Some("Add search functionality to the app".to_string()),
                issue_type: IssueType::Feature,
                status: IssueStatus::Open,
                priority: Priority::P2,
                labels: vec!["search".to_string(), "ui".to_string()],
                assignee: Some("alice".to_string()),
                created: Utc::now(),
                updated: Utc::now(),
                closed: None,
                dependencies: vec![],
                blocks: vec![],
                notes: vec![],
            },
            Issue {
                id: "beads-002".to_string(),
                title: "Fix bug in editor".to_string(),
                description: Some("Editor crashes on long text".to_string()),
                issue_type: IssueType::Bug,
                status: IssueStatus::InProgress,
                priority: Priority::P1,
                labels: vec!["bug".to_string(), "editor".to_string()],
                assignee: Some("bob".to_string()),
                created: Utc::now(),
                updated: Utc::now(),
                closed: None,
                dependencies: vec![],
                blocks: vec![],
                notes: vec![],
            },
            Issue {
                id: "beads-003".to_string(),
                title: "Update documentation".to_string(),
                description: None,
                issue_type: IssueType::Task,
                status: IssueStatus::Open,
                priority: Priority::P3,
                labels: vec!["docs".to_string()],
                assignee: None,
                created: Utc::now(),
                updated: Utc::now(),
                closed: None,
                dependencies: vec![],
                blocks: vec![],
                notes: vec![],
            },
        ]
    }

    #[test]
    fn test_search_interface_state_creation() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues.clone());

        assert_eq!(state.result_count(), 3);
        assert_eq!(state.search_scope(), SearchScope::All);
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_search_by_title() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("search");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_search_by_description() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("crashes");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_search_by_notes() {
        use crate::beads::models::Note;

        let mut issues = create_test_issues();
        // Add notes to the first issue
        issues[0].notes.push(Note {
            timestamp: Utc::now(),
            author: "alice".to_string(),
            content: "Investigation complete, ready to implement".to_string(),
        });

        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Notes);
        state.search_state_mut().set_query("investigation");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_search_all_fields() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("alice");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(
            state.filtered_issues()[0].assignee,
            Some("alice".to_string())
        );
    }

    #[test]
    fn test_search_case_insensitive() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("BUG");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_search_no_matches() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("nonexistent");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_cycle_search_scope() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.search_scope(), SearchScope::All);

        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Title);

        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Description);

        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Notes);

        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::All);
    }

    #[test]
    fn test_clear_search() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("search");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        state.clear_search();
        assert_eq!(state.search_state().query(), "");
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_toggle_help() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert!(state.is_help_visible());

        state.toggle_help();
        assert!(!state.is_help_visible());

        state.toggle_help();
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_search_scope_display_name() {
        assert_eq!(SearchScope::Title.display_name(), "Title");
        assert_eq!(SearchScope::Description.display_name(), "Description");
        assert_eq!(SearchScope::Notes.display_name(), "Notes");
        assert_eq!(SearchScope::All.display_name(), "All");
    }

    #[test]
    fn test_set_issues_updates_filters() {
        let initial_issues = create_test_issues();
        let mut state = SearchInterfaceState::new(initial_issues);

        state.search_state_mut().set_query("search");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        // Set new issues
        let new_issues = vec![Issue {
            id: "beads-004".to_string(),
            title: "New search task".to_string(),
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
        }];

        state.set_issues(new_issues);
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-004");
    }

    #[test]
    fn test_search_scope_clone() {
        let scope = SearchScope::Title;
        let cloned = scope.clone();
        assert_eq!(scope, cloned);

        let scope = SearchScope::All;
        let cloned = scope.clone();
        assert_eq!(scope, cloned);
    }

    #[test]
    fn test_search_scope_eq() {
        assert_eq!(SearchScope::Title, SearchScope::Title);
        assert_eq!(SearchScope::Description, SearchScope::Description);
        assert_eq!(SearchScope::Notes, SearchScope::Notes);
        assert_eq!(SearchScope::All, SearchScope::All);

        assert_ne!(SearchScope::Title, SearchScope::Description);
        assert_ne!(SearchScope::Notes, SearchScope::All);
    }

    #[test]
    fn test_search_scope_all() {
        let all_scopes = SearchScope::all();
        assert_eq!(all_scopes.len(), 4);
        assert_eq!(all_scopes[0], SearchScope::Title);
        assert_eq!(all_scopes[1], SearchScope::Description);
        assert_eq!(all_scopes[2], SearchScope::Notes);
        assert_eq!(all_scopes[3], SearchScope::All);
    }

    #[test]
    fn test_search_interface_state_with_empty_issues() {
        let issues: Vec<Issue> = vec![];
        let state = SearchInterfaceState::new(issues);

        assert_eq!(state.result_count(), 0);
        assert_eq!(state.filtered_issues().len(), 0);
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_search_state_getter() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("test");
        assert_eq!(state.search_state().query(), "test");
    }

    #[test]
    fn test_list_state_getter() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(1));
        assert_eq!(state.list_state().selected(), Some(1));
    }

    #[test]
    fn test_filtered_issues_initially_all() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues.clone());

        assert_eq!(state.filtered_issues().len(), issues.len());
        assert_eq!(state.filtered_issues()[0].id, issues[0].id);
    }

    #[test]
    fn test_selected_issue_none_when_no_selection() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Explicitly deselect
        state.list_state_mut().select(None);
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_selected_issue_some_when_selected() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(1));
        let selected = state.selected_issue();

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "beads-002");
    }

    #[test]
    fn test_selected_issue_out_of_bounds() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(100));
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_result_count_with_filters() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.result_count(), 3);

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn test_set_search_scope_updates_filters() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("editor");
        state.set_search_scope(SearchScope::Title);

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_update_filtered_issues_resets_selection_when_out_of_bounds() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(2));
        assert_eq!(state.list_state().selected(), Some(2));

        state.search_state_mut().set_query("search");
        state.update_filtered_issues();

        // Should reset to Some(0) since only 1 result and previous selection (2) is out of bounds
        assert_eq!(state.list_state().selected(), Some(0));
    }

    #[test]
    fn test_search_in_labels() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("ui");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_search_in_id() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("beads-002");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_search_with_no_description() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("anything");
        state.update_filtered_issues();

        // beads-003 has no description, so should not match
        // beads-001 and beads-002 have descriptions but "anything" doesn't match
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_with_no_assignee() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("charlie");
        state.update_filtered_issues();

        // No issues assigned to charlie
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_with_empty_notes() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Notes);
        state.search_state_mut().set_query("anything");
        state.update_filtered_issues();

        // All test issues have empty notes
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_with_multiple_labels() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("editor");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
        assert!(state.filtered_issues()[0].labels.contains(&"editor".to_string()));
    }

    #[test]
    fn test_search_interface_view_new() {
        let view = SearchInterfaceView::new();
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
        assert_eq!(view.help_style, Style::default().fg(Color::DarkGray));
    }

    #[test]
    fn test_search_interface_view_default() {
        let view = SearchInterfaceView::default();
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
        assert_eq!(view.help_style, Style::default().fg(Color::DarkGray));
    }

    #[test]
    fn test_search_interface_view_block_style() {
        let custom_style = Style::default().fg(Color::Green);
        let view = SearchInterfaceView::new().block_style(custom_style);
        assert_eq!(view.block_style, custom_style);
    }

    #[test]
    fn test_search_interface_view_help_style() {
        let custom_style = Style::default().fg(Color::Magenta);
        let view = SearchInterfaceView::new().help_style(custom_style);
        assert_eq!(view.help_style, custom_style);
    }

    #[test]
    fn test_search_interface_view_builder_chain() {
        let block_style = Style::default().fg(Color::Red);
        let help_style = Style::default().fg(Color::Blue);

        let view = SearchInterfaceView::new()
            .block_style(block_style)
            .help_style(help_style);

        assert_eq!(view.block_style, block_style);
        assert_eq!(view.help_style, help_style);
    }

    // ========== Additional Comprehensive Tests ==========

    #[test]
    fn test_search_interface_state_debug_trait() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues);
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("SearchInterfaceState"));
    }

    #[test]
    fn test_search_scope_debug_trait() {
        let scope = SearchScope::All;
        let debug_str = format!("{:?}", scope);
        assert!(debug_str.contains("All"));
    }

    #[test]
    fn test_search_scope_copy_trait() {
        let scope1 = SearchScope::Title;
        let scope2 = scope1; // Copy
        assert_eq!(scope1, scope2);

        let scope3 = SearchScope::All;
        let scope4 = scope3; // Copy
        assert_eq!(scope3, scope4);
    }

    #[test]
    fn test_all_search_scope_variants() {
        let scopes = vec![
            SearchScope::Title,
            SearchScope::Description,
            SearchScope::Notes,
            SearchScope::All,
        ];

        for scope in &scopes {
            assert!(!scope.display_name().is_empty());
        }

        // Verify all are distinct
        assert_ne!(scopes[0], scopes[1]);
        assert_ne!(scopes[1], scopes[2]);
        assert_ne!(scopes[2], scopes[3]);
    }

    #[test]
    fn test_search_interface_view_builder_order_independence() {
        let block_style = Style::default().fg(Color::Green);
        let help_style = Style::default().fg(Color::Yellow);

        let view1 = SearchInterfaceView::new()
            .block_style(block_style)
            .help_style(help_style);

        let view2 = SearchInterfaceView::new()
            .help_style(help_style)
            .block_style(block_style);

        assert_eq!(view1.block_style, view2.block_style);
        assert_eq!(view1.help_style, view2.help_style);
    }

    #[test]
    fn test_search_interface_view_multiple_setter_applications() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);
        let style3 = Style::default().fg(Color::Green);

        let view = SearchInterfaceView::new()
            .block_style(style1)
            .block_style(style2)
            .block_style(style3);

        assert_eq!(view.block_style, style3); // Last wins
    }

    #[test]
    fn test_search_with_very_long_query() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        let long_query = "x".repeat(1000);
        state.search_state_mut().set_query(&long_query);
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 0); // Should not match anything
    }

    #[test]
    fn test_search_with_special_characters() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("@#$%");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_with_unicode() {
        let mut issues = create_test_issues();
        issues[0].title = "测试 Unicode search".to_string();

        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("测试");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_multiple_search_updates() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("search");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        state.search_state_mut().set_query("editor");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        state.clear_search();
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_search_scope_cycling_complete_loop() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        let initial = state.search_scope();

        state.next_search_scope();
        state.next_search_scope();
        state.next_search_scope();
        state.next_search_scope();

        assert_eq!(state.search_scope(), initial); // Full cycle
    }

    #[test]
    fn test_set_search_scope_directly_all_variants() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Title);
        assert_eq!(state.search_scope(), SearchScope::Title);

        state.set_search_scope(SearchScope::Description);
        assert_eq!(state.search_scope(), SearchScope::Description);

        state.set_search_scope(SearchScope::Notes);
        assert_eq!(state.search_scope(), SearchScope::Notes);

        state.set_search_scope(SearchScope::All);
        assert_eq!(state.search_scope(), SearchScope::All);
    }

    #[test]
    fn test_filtered_issues_preserves_order() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues);

        // Should preserve original order
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
        assert_eq!(state.filtered_issues()[1].id, "beads-002");
        assert_eq!(state.filtered_issues()[2].id, "beads-003");
    }

    #[test]
    fn test_search_with_partial_match_in_title() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("fix");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert!(state.filtered_issues()[0].title.to_lowercase().contains("fix"));
    }

    #[test]
    fn test_search_with_partial_match_in_description() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("text");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_result_count_consistency() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.result_count(), state.filtered_issues().len());

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), state.filtered_issues().len());
    }

    #[test]
    fn test_selected_issue_after_filter_change() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(0));
        let selected_before = state.selected_issue().map(|i| i.id.clone());

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();

        let selected_after = state.selected_issue().map(|i| i.id.clone());

        // Selection should reset to first match
        assert_ne!(selected_before, selected_after);
    }

    #[test]
    fn test_help_toggle_multiple_times() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        for i in 0..10 {
            state.toggle_help();
            if i % 2 == 0 {
                assert!(!state.is_help_visible());
            } else {
                assert!(state.is_help_visible());
            }
        }
    }

    #[test]
    fn test_set_issues_clears_previous_results() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.result_count(), 3);

        let new_issues = vec![];
        state.set_issues(new_issues);

        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_all_with_multiple_matches() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("beads");
        state.update_filtered_issues();

        // All issues have "beads" in their ID
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_search_title_with_no_match_in_description() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("crashes");
        state.update_filtered_issues();

        // "crashes" is in description, not title
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_search_description_with_no_match_in_title() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("implement");
        state.update_filtered_issues();

        // "implement" is in title, not description
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_clear_search_resets_to_all_issues() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        state.clear_search();
        assert_eq!(state.result_count(), 3);
        assert_eq!(state.search_state().query(), "");
    }

    #[test]
    fn test_search_state_mut_allows_modifications() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("test");
        assert_eq!(state.search_state().query(), "test");

        state.search_state_mut().clear();
        assert_eq!(state.search_state().query(), "");
    }

    #[test]
    fn test_list_state_mut_allows_modifications() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(2));
        assert_eq!(state.list_state().selected(), Some(2));

        state.list_state_mut().select(None);
        assert_eq!(state.list_state().selected(), None);
    }

    #[test]
    fn test_filtered_issues_empty_after_no_match_search() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("zzzznonexistent");
        state.update_filtered_issues();

        assert!(state.filtered_issues().is_empty());
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_selected_issue_with_single_result() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("bug");
        state.update_filtered_issues();

        state.list_state_mut().select(Some(0));
        let selected = state.selected_issue();

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "beads-002");
    }

    #[test]
    fn test_update_filtered_issues_with_empty_filter_to_none_selection() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.list_state_mut().select(Some(1));
        state.search_state_mut().set_query("nonexistent");
        state.update_filtered_issues();

        // Selection should become None when no results
        assert_eq!(state.list_state().selected(), None);
    }

    #[test]
    fn test_search_scope_all_includes_all_scopes() {
        let mut issues = create_test_issues();

        // Add content to all searchable fields
        issues[0].title = "title_match".to_string();
        issues[1].description = Some("description_match".to_string());
        
        use crate::beads::models::Note;
        issues[2].notes.push(Note {
            timestamp: Utc::now(),
            author: "test".to_string(),
            content: "notes_match".to_string(),
        });

        let mut state = SearchInterfaceState::new(issues);

        state.set_search_scope(SearchScope::All);

        // Should match title
        state.search_state_mut().set_query("title_match");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        // Should match description
        state.search_state_mut().set_query("description_match");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        // Should match notes
        state.search_state_mut().set_query("notes_match");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn test_search_with_whitespace() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("  search  ");
        state.update_filtered_issues();

        // Whitespace is not trimmed, so won't match
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_cycle_search_scope_from_each_start() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Start from Title
        state.set_search_scope(SearchScope::Title);
        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Description);

        // Start from Description
        state.set_search_scope(SearchScope::Description);
        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Notes);

        // Start from Notes
        state.set_search_scope(SearchScope::Notes);
        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::All);

        // Start from All
        state.set_search_scope(SearchScope::All);
        state.next_search_scope();
        assert_eq!(state.search_scope(), SearchScope::Title);
    }

    #[test]
    fn test_filtered_issues_slice_vs_vec() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues.clone());

        let filtered_slice = state.filtered_issues();
        assert_eq!(filtered_slice.len(), issues.len());

        // Verify it returns a slice
        for (i, issue) in filtered_slice.iter().enumerate() {
            assert_eq!(issue.id, issues[i].id);
        }
    }

    #[test]
    fn test_search_case_insensitive_all_scopes() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Title scope
        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("SEARCH");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        // Description scope
        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("CRASHES");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);

        // All scope
        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("ALICE");
        state.update_filtered_issues();
        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn test_search_interface_state_initial_help_visible() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues);

        assert!(state.is_help_visible());
    }

    #[test]
    fn test_search_with_numbers() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.search_state_mut().set_query("001");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_result_count_zero_with_empty_issues() {
        let issues = vec![];
        let state = SearchInterfaceState::new(issues);

        assert_eq!(state.result_count(), 0);
    }
}
