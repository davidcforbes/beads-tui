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

    /// Update filtered issues based on search query and scope
    pub fn update_filtered_issues(&mut self) {
        let query = self.search_state.query().to_lowercase();

        if query.is_empty() {
            self.filtered_issues = self.all_issues.clone();
        } else {
            self.filtered_issues = self
                .all_issues
                .iter()
                .filter(|issue| self.matches_query(issue, &query))
                .cloned()
                .collect();
        }

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
}
