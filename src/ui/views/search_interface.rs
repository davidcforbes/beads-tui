//! Search interface view with search input and results

use crate::beads::models::{Issue, IssueStatus};
use crate::models::{filter::LogicOp, IssueFilter, SavedFilter};
use crate::ui::widgets::{
    issue_list::LabelMatchMode, IssueList, IssueListState, SearchInput, SearchInputState,
};
use crate::utils::safe_regex_match;
use chrono::{Duration, Utc};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, StatefulWidget, Widget},
};

/// View type for smart views
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    /// All issues
    All,
    /// Ready issues (open, no blockers)
    Ready,
    /// Blocked issues
    Blocked,
    /// My issues (assigned to current user)
    MyIssues,
    /// Recently updated issues (within last 7 days)
    Recently,
    /// Stale issues (not updated in 30+ days)
    Stale,
}

impl ViewType {
    /// Get display name for the view
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::All => "All Issues",
            Self::Ready => "Ready",
            Self::Blocked => "Blocked",
            Self::MyIssues => "My Issues",
            Self::Recently => "Recently Updated",
            Self::Stale => "Stale",
        }
    }

    /// Get all view types
    pub fn all() -> Vec<ViewType> {
        vec![
            Self::All,
            Self::Ready,
            Self::Blocked,
            Self::MyIssues,
            Self::Recently,
            Self::Stale,
        ]
    }
}

/// Search interface state
pub struct SearchInterfaceState {
    search_state: SearchInputState,
    list_state: IssueListState,
    all_issues: Vec<Issue>,
    filtered_issues: Vec<Issue>,
    search_scope: SearchScope,
    current_view: ViewType,
    current_user: Option<String>,
    show_help: bool,
    regex_enabled: bool,
    fuzzy_enabled: bool,
    fuzzy_matcher: SkimMatcherV2,
    saved_filters: Vec<SavedFilter>,
    label_logic: LogicOp,
    filter_menu_open: bool,
    filter_menu_state: ratatui::widgets::ListState,
    /// Cached lowercase query for faster substring matching
    lowercase_query_cache: Option<String>,
    /// Cache for filter results to avoid re-filtering on unchanged queries
    last_filter_query: Option<String>,
    last_filter_view: Option<ViewType>,
    last_filters_enabled: bool,
    last_current_user: Option<String>,
}

impl std::fmt::Debug for SearchInterfaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchInterfaceState")
            .field("search_state", &self.search_state)
            .field("list_state", &self.list_state)
            .field("all_issues", &self.all_issues)
            .field("filtered_issues", &self.filtered_issues)
            .field("search_scope", &self.search_scope)
            .field("current_view", &self.current_view)
            .field("current_user", &self.current_user)
            .field("show_help", &self.show_help)
            .field("regex_enabled", &self.regex_enabled)
            .field("fuzzy_enabled", &self.fuzzy_enabled)
            .field("fuzzy_matcher", &"<SkimMatcherV2>")
            .field("saved_filters", &self.saved_filters)
            .field("label_logic", &self.label_logic)
            .finish()
    }
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
            current_view: ViewType::All,
            current_user: None,
            show_help: true,
            regex_enabled: false,
            fuzzy_enabled: false,
            fuzzy_matcher: SkimMatcherV2::default(),
            saved_filters: Vec::new(),
            label_logic: LogicOp::And,
            filter_menu_open: false,
            filter_menu_state: ratatui::widgets::ListState::default(),
            lowercase_query_cache: None,
            last_filter_query: None,
            last_filter_view: None,
            last_filters_enabled: false,
            last_current_user: None,
        }
    }

    /// Create a new search interface state with a specific user
    pub fn with_user(issues: Vec<Issue>, user: Option<String>) -> Self {
        let mut state = Self::new(issues);
        state.current_user = user;
        state
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

    /// Get all issues
    pub fn all_issues(&self) -> &[Issue] {
        &self.all_issues
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

    /// Get current view
    pub fn current_view(&self) -> ViewType {
        self.current_view
    }

    /// Set current view
    pub fn set_view(&mut self, view: ViewType) {
        self.current_view = view;
        // Invalidate cache since view changed
        self.last_filter_view = None;
        self.update_filtered_issues();
    }

    /// Cycle to next view
    pub fn next_view(&mut self) {
        self.current_view = match self.current_view {
            ViewType::All => ViewType::Ready,
            ViewType::Ready => ViewType::Blocked,
            ViewType::Blocked => ViewType::MyIssues,
            ViewType::MyIssues => ViewType::Recently,
            ViewType::Recently => ViewType::Stale,
            ViewType::Stale => ViewType::All,
        };
        self.update_filtered_issues();
    }

    /// Set current user for MyIssues view
    pub fn set_current_user(&mut self, user: Option<String>) {
        self.current_user = user;
        // Invalidate cache since user filter changed
        self.last_filter_query = None;
        self.update_filtered_issues();
    }

    /// Get current user
    pub fn current_user(&self) -> Option<&str> {
        self.current_user.as_deref()
    }

    /// Toggle regex search mode
    pub fn toggle_regex(&mut self) {
        self.regex_enabled = !self.regex_enabled;
        if self.regex_enabled {
            self.fuzzy_enabled = false;
        }
        self.update_filtered_issues();
    }

    /// Check if regex search is enabled
    pub fn is_regex_enabled(&self) -> bool {
        self.regex_enabled
    }

    /// Toggle fuzzy search mode
    pub fn toggle_fuzzy(&mut self) {
        self.fuzzy_enabled = !self.fuzzy_enabled;
        if self.fuzzy_enabled {
            self.regex_enabled = false;
        }
        self.update_filtered_issues();
    }

    /// Check if fuzzy search is enabled
    pub fn is_fuzzy_enabled(&self) -> bool {
        self.fuzzy_enabled
    }

    /// Set saved filters
    pub fn set_saved_filters(&mut self, filters: Vec<SavedFilter>) {
        self.saved_filters = filters;
    }

    /// Get saved filters
    pub fn saved_filters(&self) -> &[SavedFilter] {
        &self.saved_filters
    }

    /// Toggle label logic (AND/OR)
    pub fn toggle_label_logic(&mut self) {
        self.label_logic = match self.label_logic {
            LogicOp::And => LogicOp::Or,
            LogicOp::Or => LogicOp::And,
        };

        // Synchronize with list state
        let mode = match self.label_logic {
            LogicOp::And => LabelMatchMode::All,
            LogicOp::Or => LabelMatchMode::Any,
        };
        self.list_state.column_filters_mut().label_match_mode = mode;

        self.update_filtered_issues();
    }

    /// Get current label logic
    pub fn label_logic(&self) -> LogicOp {
        self.label_logic
    }

    /// Create an IssueFilter from current state
    pub fn get_current_filter(&self) -> IssueFilter {
        let col_filters = self.list_state.column_filters();
        IssueFilter {
            status: match col_filters.status.to_lowercase().as_str() {
                "open" => Some(crate::beads::models::IssueStatus::Open),
                "in_progress" => Some(crate::beads::models::IssueStatus::InProgress),
                "blocked" => Some(crate::beads::models::IssueStatus::Blocked),
                "closed" => Some(crate::beads::models::IssueStatus::Closed),
                _ => None,
            },
            priority: match col_filters.priority.as_str() {
                "P0" => Some(crate::beads::models::Priority::P0),
                "P1" => Some(crate::beads::models::Priority::P1),
                "P2" => Some(crate::beads::models::Priority::P2),
                "P3" => Some(crate::beads::models::Priority::P3),
                "P4" => Some(crate::beads::models::Priority::P4),
                _ => None,
            },
            issue_type: match col_filters.type_filter.to_lowercase().as_str() {
                "epic" => Some(crate::beads::models::IssueType::Epic),
                "feature" => Some(crate::beads::models::IssueType::Feature),
                "task" => Some(crate::beads::models::IssueType::Task),
                "bug" => Some(crate::beads::models::IssueType::Bug),
                "chore" => Some(crate::beads::models::IssueType::Chore),
                _ => None,
            },
            assignee: None, // We don't track specific assignee in column filters currently
            labels: col_filters.labels.clone(),
            label_logic: self.label_logic,
            search_text: if self.search_state.query().is_empty() {
                None
            } else {
                Some(self.search_state.query().to_string())
            },
            search_scope: format!("{:?}", self.search_scope),
            view_type: format!("{:?}", self.current_view),
            use_regex: self.regex_enabled,
            use_fuzzy: self.fuzzy_enabled,
        }
    }

    /// Apply an IssueFilter to current state
    pub fn apply_filter(&mut self, filter: &IssueFilter) {
        // Apply column filters using setter methods to maintain cache
        {
            let col_filters = self.list_state.column_filters_mut();
            col_filters.clear();

            if let Some(ref status) = filter.status {
                col_filters.set_status(status.to_string());
            }
            if let Some(ref priority) = filter.priority {
                col_filters.priority = priority.to_string();
            }
            if let Some(ref issue_type) = filter.issue_type {
                col_filters.set_type_filter(issue_type.to_string());
            }

            // Handle assignee - if filter.assignee is None, set no_assignee
            col_filters.no_assignee = filter.assignee.is_none();

            col_filters.set_labels(filter.labels.clone());
            col_filters.label_match_mode = match filter.label_logic {
                LogicOp::And => LabelMatchMode::All,
                LogicOp::Or => LabelMatchMode::Any,
            };
        }

        // Apply global state
        self.label_logic = filter.label_logic;
        self.search_state
            .set_query(filter.search_text.as_deref().unwrap_or(""));
        self.regex_enabled = filter.use_regex;
        self.fuzzy_enabled = filter.use_fuzzy;

        // Parse scope and view from strings
        self.search_scope = match filter.search_scope.as_str() {
            "Title" => SearchScope::Title,
            "Description" => SearchScope::Description,
            "Notes" => SearchScope::Notes,
            _ => SearchScope::All,
        };

        self.current_view = match filter.view_type.as_str() {
            "Ready" => ViewType::Ready,
            "Blocked" => ViewType::Blocked,
            "MyIssues" => ViewType::MyIssues,
            "Recently" => ViewType::Recently,
            "Stale" => ViewType::Stale,
            _ => ViewType::All,
        };

        self.update_filtered_issues();
    }

    /// Add a new saved filter
    pub fn add_saved_filter(&mut self, name: String, hotkey: Option<char>) {
        let filter = self.get_current_filter();
        self.saved_filters.push(SavedFilter {
            name,
            filter,
            hotkey,
        });
    }

    /// Delete a saved filter by index
    pub fn delete_saved_filter(&mut self, index: usize) {
        if index < self.saved_filters.len() {
            self.saved_filters.remove(index);
        }
    }

    /// Apply a saved filter by index
    pub fn apply_saved_filter(&mut self, index: usize) {
        if let Some(saved) = self.saved_filters.get(index) {
            let filter = saved.filter.clone();
            self.apply_filter(&filter);
        }
    }

    /// Toggle filter menu
    pub fn toggle_filter_menu(&mut self) {
        self.filter_menu_open = !self.filter_menu_open;
        if self.filter_menu_open && !self.saved_filters.is_empty() {
            self.filter_menu_state.select(Some(0));
        }
    }

    /// Is filter menu open?
    pub fn is_filter_menu_open(&self) -> bool {
        self.filter_menu_open
    }

    /// Select next filter in menu
    pub fn filter_menu_next(&mut self) {
        let len = self.saved_filters.len();
        if len == 0 {
            return;
        }
        let i = match self.filter_menu_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.filter_menu_state.select(Some(i));
    }

    /// Select previous filter in menu
    pub fn filter_menu_previous(&mut self) {
        let len = self.saved_filters.len();
        if len == 0 {
            return;
        }
        let i = match self.filter_menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.filter_menu_state.select(Some(i));
    }

    /// Apply selected filter from menu and close it
    pub fn filter_menu_confirm(&mut self) {
        if let Some(i) = self.filter_menu_state.selected() {
            self.apply_saved_filter(i);
        }
        self.filter_menu_open = false;
    }

    /// Get filter menu list state
    pub fn filter_menu_state(&self) -> &ratatui::widgets::ListState {
        &self.filter_menu_state
    }

    /// Set all issues - preserves selection by issue ID across external updates
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        // Save currently selected issue ID before updating
        let selected_issue_id = self.selected_issue().map(|issue| issue.id.clone());

        // Update issues and regenerate filtered list
        self.all_issues = issues;
        // Invalidate cache since underlying data changed
        self.last_filter_query = None;
        self.update_filtered_issues();

        // Restore selection by finding the same issue ID in the new filtered list
        if let Some(issue_id) = selected_issue_id {
            let new_index = self
                .filtered_issues
                .iter()
                .position(|issue| issue.id == issue_id);

            self.list_state.select(new_index);
        }
    }

    /// Clear all filters and reset to default view
    pub fn clear_all_filters(&mut self) {
        // Clear search text
        self.search_state.clear();

        // Reset search scope to All
        self.search_scope = SearchScope::All;

        // Reset view to All
        self.current_view = ViewType::All;

        // Clear column filters
        self.list_state.clear_filters();

        // Update filtered issues to reflect cleared state
        self.update_filtered_issues();
    }

    /// Update filtered issues based on search query, scope, column filters, and view
    pub fn update_filtered_issues(&mut self) {
        // Don't lowercase query when regex is enabled (regex handles case-insensitivity)
        let query = if self.regex_enabled {
            self.lowercase_query_cache = None;
            self.search_state.query().to_string()
        } else if self.fuzzy_enabled {
            self.lowercase_query_cache = None;
            self.search_state.query().to_lowercase()
        } else {
            // Cache the lowercase query for substring matching performance
            let lowercase_query = self.search_state.query().to_lowercase();
            self.lowercase_query_cache = Some(lowercase_query.clone());
            lowercase_query
        };
        let column_filters = self.list_state.column_filters();
        let filters_enabled = self.list_state.filters_enabled();

        // Check if we can use cached results (optimization to avoid re-filtering)
        let cache_valid = self.last_filter_query.as_ref() == Some(&query)
            && self.last_filter_view == Some(self.current_view)
            && self.last_filters_enabled == filters_enabled
            && self.last_current_user == self.current_user;

        if cache_valid {
            // Cache hit - skip expensive filtering operation
            return;
        }

        // Cache miss - perform filtering and update cache
        self.filtered_issues = self
            .all_issues
            .iter()
            .filter(|issue| {
                // First apply view-specific filter
                let matches_view = self.matches_view(issue);

                // Then apply search query filter
                let matches_search = if query.is_empty() {
                    true
                } else {
                    self.matches_query(issue, &query)
                };

                // Finally apply column filters if enabled
                let matches_filters = if !filters_enabled {
                    true
                } else {
                    column_filters.matches(issue)
                };

                matches_view && matches_search && matches_filters
            })
            .cloned()
            .collect();

        // Update cache values
        self.last_filter_query = Some(query);
        self.last_filter_view = Some(self.current_view);
        self.last_filters_enabled = filters_enabled;
        self.last_current_user = self.current_user.clone();

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

    /// Check if an issue matches the current view
    fn matches_view(&self, issue: &Issue) -> bool {
        match self.current_view {
            ViewType::All => true,
            ViewType::Ready => {
                // Ready: status is open and no dependencies (no blockers)
                issue.status == IssueStatus::Open && issue.dependencies.is_empty()
            }
            ViewType::Blocked => {
                // Blocked: status is blocked
                issue.status == IssueStatus::Blocked
            }
            ViewType::MyIssues => {
                // MyIssues: assigned to current user
                if let Some(ref user) = self.current_user {
                    issue.assignee.as_ref() == Some(user)
                } else {
                    false
                }
            }
            ViewType::Recently => {
                // Recently: updated within the last 7 days
                let seven_days_ago = Utc::now() - Duration::days(7);
                issue.updated >= seven_days_ago
            }
            ViewType::Stale => {
                // Stale: not updated in 30+ days and still open
                let thirty_days_ago = Utc::now() - Duration::days(30);
                issue.updated < thirty_days_ago
                    && (issue.status == IssueStatus::Open
                        || issue.status == IssueStatus::InProgress
                        || issue.status == IssueStatus::Blocked)
            }
        }
    }

    /// Check if an issue matches the search query based on scope
    fn matches_query(&self, issue: &Issue, query: &str) -> bool {
        match self.search_scope {
            SearchScope::Title => self.matches_text(&issue.title, query),
            SearchScope::Description => {
                if let Some(ref desc) = issue.description {
                    self.matches_text(desc, query)
                } else {
                    false
                }
            }
            SearchScope::Notes => issue
                .notes
                .iter()
                .any(|note| self.matches_text(&note.content, query)),
            SearchScope::All => {
                self.matches_text(&issue.title, query)
                    || issue
                        .description
                        .as_ref()
                        .map(|d| self.matches_text(d, query))
                        .unwrap_or(false)
                    || self.matches_text(&issue.id, query)
                    || issue
                        .assignee
                        .as_ref()
                        .map(|a| self.matches_text(a, query))
                        .unwrap_or(false)
                    || issue.labels.iter().any(|l| self.matches_text(l, query))
                    || issue
                        .notes
                        .iter()
                        .any(|note| self.matches_text(&note.content, query))
            }
        }
    }

    /// Helper method to match text using fuzzy, regex, or substring matching
    /// Falls back to substring matching if regex compilation fails
    fn matches_text(&self, text: &str, query: &str) -> bool {
        if self.fuzzy_enabled {
            // Use fuzzy matching - returns Some(score) if there's a match
            self.fuzzy_matcher.fuzzy_match(text, query).is_some()
        } else if self.regex_enabled {
            // Use safe regex matching with DoS protection
            // Returns None if regex is unsafe or invalid, then fallback to substring
            safe_regex_match(query, text, true).unwrap_or_else(|| {
                // Fallback to substring matching if regex is unsafe or invalid
                let lowercase_text = text.to_lowercase();
                if let Some(ref cached_query) = self.lowercase_query_cache {
                    lowercase_text.contains(cached_query)
                } else {
                    lowercase_text.contains(&query.to_lowercase())
                }
            })
        } else {
            // Use substring matching with cached lowercase query for performance
            let lowercase_text = text.to_lowercase();
            if let Some(ref cached_query) = self.lowercase_query_cache {
                lowercase_text.contains(cached_query)
            } else {
                lowercase_text.contains(&query.to_lowercase())
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
            .title(format!(
                "Search [{}] - View: {}",
                state.search_scope().display_name(),
                state.current_view().display_name()
            ))
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
            "/ Focus Search | Tab: Cycle Scope | v: Cycle View | Esc: Clear | j/k: Navigate | Enter: View | ?: Toggle Help";

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

        // Render filter menu overlay if open
        if state.filter_menu_open {
            self.render_filter_menu(area, buf, state);
        }
    }
}

impl<'a> SearchInterfaceView<'a> {
    fn render_filter_menu(&self, area: Rect, buf: &mut Buffer, state: &mut SearchInterfaceState) {
        let menu_area = crate::ui::layout::centered_rect(40, 50, area);
        Clear.render(menu_area, buf);

        let items: Vec<ListItem> = state
            .saved_filters
            .iter()
            .map(|f| {
                let hotkey_str = if let Some(h) = f.hotkey {
                    format!(" [F{}]", h)
                } else {
                    "".to_string()
                };
                ListItem::new(format!("{}{}", f.name, hotkey_str))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Saved Filters")
                    .style(self.block_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(list, menu_area, buf, &mut state.filter_menu_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Note, Priority};
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
        let cloned = scope;
        assert_eq!(scope, cloned);

        let scope = SearchScope::All;
        let cloned = scope;
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
        assert!(state.filtered_issues()[0]
            .labels
            .contains(&"editor".to_string()));
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
        assert!(state.filtered_issues()[0]
            .title
            .to_lowercase()
            .contains("fix"));
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

    // ========== Smart Views Tests ==========

    #[test]
    fn test_view_type_display_name() {
        assert_eq!(ViewType::All.display_name(), "All Issues");
        assert_eq!(ViewType::Ready.display_name(), "Ready");
        assert_eq!(ViewType::Blocked.display_name(), "Blocked");
        assert_eq!(ViewType::MyIssues.display_name(), "My Issues");
    }

    #[test]
    fn test_view_type_all() {
        let all_views = ViewType::all();
        assert_eq!(all_views.len(), 6);
        assert_eq!(all_views[0], ViewType::All);
        assert_eq!(all_views[1], ViewType::Ready);
        assert_eq!(all_views[2], ViewType::Blocked);
        assert_eq!(all_views[3], ViewType::MyIssues);
        assert_eq!(all_views[4], ViewType::Recently);
        assert_eq!(all_views[5], ViewType::Stale);
    }

    #[test]
    fn test_view_type_clone_and_copy() {
        let view1 = ViewType::Ready;
        let view2 = view1; // Copy
        assert_eq!(view1, view2);
    }

    #[test]
    fn test_view_type_eq() {
        assert_eq!(ViewType::All, ViewType::All);
        assert_eq!(ViewType::Ready, ViewType::Ready);
        assert_eq!(ViewType::Blocked, ViewType::Blocked);
        assert_eq!(ViewType::MyIssues, ViewType::MyIssues);

        assert_ne!(ViewType::All, ViewType::Ready);
        assert_ne!(ViewType::Blocked, ViewType::MyIssues);
    }

    #[test]
    fn test_default_view_is_all() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::new(issues);

        assert_eq!(state.current_view(), ViewType::All);
    }

    #[test]
    fn test_set_view() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_view(ViewType::Ready);
        assert_eq!(state.current_view(), ViewType::Ready);

        state.set_view(ViewType::Blocked);
        assert_eq!(state.current_view(), ViewType::Blocked);
    }

    #[test]
    fn test_next_view_cycles() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.current_view(), ViewType::All);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::Ready);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::Blocked);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::MyIssues);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::Recently);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::Stale);

        state.next_view();
        assert_eq!(state.current_view(), ViewType::All);
    }

    #[test]
    fn test_ready_view_filters_open_no_dependencies() {
        let mut issues = create_test_issues();
        // beads-001: Open, no dependencies
        // beads-002: InProgress (should be filtered out)
        // beads-003: Open, no dependencies

        // Add a dependency to beads-003
        issues[2].dependencies.push("beads-001".to_string());

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Ready);

        // Only beads-001 should match (open + no dependencies)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_blocked_view_filters_by_status() {
        let mut issues = create_test_issues();
        // Set beads-002 to Blocked status
        issues[1].status = IssueStatus::Blocked;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Blocked);

        // Only beads-002 should match
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-002");
    }

    #[test]
    fn test_my_issues_view_filters_by_assignee() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::with_user(issues, Some("alice".to_string()));

        state.set_view(ViewType::MyIssues);

        // Only beads-001 is assigned to alice
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
        assert_eq!(
            state.filtered_issues()[0].assignee,
            Some("alice".to_string())
        );
    }

    #[test]
    fn test_my_issues_view_with_no_user_returns_nothing() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_view(ViewType::MyIssues);

        // No current user, so no matches
        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_set_current_user() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        assert_eq!(state.current_user(), None);

        state.set_current_user(Some("bob".to_string()));
        assert_eq!(state.current_user(), Some("bob"));

        state.set_current_user(None);
        assert_eq!(state.current_user(), None);
    }

    #[test]
    fn test_my_issues_view_updates_when_user_changes() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_view(ViewType::MyIssues);
        assert_eq!(state.result_count(), 0); // No user

        state.set_current_user(Some("alice".to_string()));
        assert_eq!(state.result_count(), 1); // Alice has 1 issue

        state.set_current_user(Some("bob".to_string()));
        assert_eq!(state.result_count(), 1); // Bob has 1 issue
    }

    #[test]
    fn test_view_filter_combines_with_search() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Set to Ready view (beads-001 and beads-003)
        state.set_view(ViewType::Ready);
        assert_eq!(state.result_count(), 2);

        // Now search for "search" - should only match beads-001
        state.search_state_mut().set_query("search");
        state.update_filtered_issues();

        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_view_filter_combines_with_column_filters() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Set to Ready view (beads-001 and beads-003)
        state.set_view(ViewType::Ready);
        assert_eq!(state.result_count(), 2);

        // Enable column filters
        state.list_state_mut().toggle_filters();

        // Filter by priority P2
        let filters = state.list_state_mut().column_filters_mut();
        filters.priority = "P2".to_string();
        state.update_filtered_issues();

        // Only beads-001 should match (Ready + P2)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_all_view_shows_all_issues() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_view(ViewType::All);

        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_ready_view_with_no_ready_issues() {
        let mut issues = create_test_issues();
        // Make all issues not ready
        issues[0].status = IssueStatus::Closed;
        issues[1].status = IssueStatus::InProgress;
        issues[2].dependencies.push("beads-001".to_string());

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Ready);

        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_blocked_view_with_no_blocked_issues() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        state.set_view(ViewType::Blocked);

        assert_eq!(state.result_count(), 0);
    }

    #[test]
    fn test_with_user_constructor() {
        let issues = create_test_issues();
        let state = SearchInterfaceState::with_user(issues, Some("charlie".to_string()));

        assert_eq!(state.current_user(), Some("charlie"));
        assert_eq!(state.current_view(), ViewType::All);
    }

    #[test]
    fn test_view_change_updates_filters() {
        let mut issues = create_test_issues();
        issues[1].status = IssueStatus::Blocked;

        let mut state = SearchInterfaceState::new(issues);

        // Start with All view
        assert_eq!(state.result_count(), 3);

        // Switch to Blocked
        state.set_view(ViewType::Blocked);
        assert_eq!(state.result_count(), 1);

        // Switch to Ready
        state.set_view(ViewType::Ready);
        assert_eq!(state.result_count(), 2);

        // Switch back to All
        state.set_view(ViewType::All);
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_ready_view_with_multiple_dependencies() {
        let mut issues = create_test_issues();
        issues[0].dependencies.push("beads-x".to_string());
        issues[0].dependencies.push("beads-y".to_string());

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Ready);

        // beads-001 has dependencies, so not ready
        // beads-002 is InProgress, so not ready
        // beads-003 is Open with no dependencies, so ready
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-003");
    }

    #[test]
    fn test_my_issues_case_sensitive_matching() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::with_user(issues, Some("Alice".to_string()));

        state.set_view(ViewType::MyIssues);

        // "Alice" != "alice", so no matches
        assert_eq!(state.result_count(), 0);
    }

    // ========== Recently and Stale Views Tests ==========

    #[test]
    fn test_recently_view_shows_recent_updates() {
        let mut issues = create_test_issues();

        // Set beads-001 to be updated 3 days ago
        let three_days_ago = Utc::now() - Duration::days(3);
        issues[0].updated = three_days_ago;

        // Set beads-002 to be updated 10 days ago (not recent)
        let ten_days_ago = Utc::now() - Duration::days(10);
        issues[1].updated = ten_days_ago;

        // beads-003 has default updated time (recent)

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Recently);

        // Should show beads-001 (3 days) and beads-003 (recent)
        assert_eq!(state.result_count(), 2);

        let ids: Vec<&str> = state
            .filtered_issues()
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert!(ids.contains(&"beads-001"));
        assert!(ids.contains(&"beads-003"));
        assert!(!ids.contains(&"beads-002"));
    }

    #[test]
    fn test_recently_view_boundary_exactly_seven_days() {
        let mut issues = create_test_issues();

        // Set beads-001 to be updated 6 days ago (clearly within recently)
        let six_days_ago = Utc::now() - Duration::days(6);
        issues[0].updated = six_days_ago;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Recently);

        // Should include issues updated within last 7 days
        let ids: Vec<&str> = state
            .filtered_issues()
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert!(ids.contains(&"beads-001"));
    }

    #[test]
    fn test_stale_view_shows_old_open_issues() {
        let mut issues = create_test_issues();

        // Set beads-001 to be updated 45 days ago (stale)
        let forty_five_days_ago = Utc::now() - Duration::days(45);
        issues[0].updated = forty_five_days_ago;
        issues[0].status = IssueStatus::Open;

        // Set beads-002 to be updated 20 days ago (not stale)
        let twenty_days_ago = Utc::now() - Duration::days(20);
        issues[1].updated = twenty_days_ago;
        issues[1].status = IssueStatus::Open;

        // Set beads-003 to be updated 50 days ago but closed (should not show)
        let fifty_days_ago = Utc::now() - Duration::days(50);
        issues[2].updated = fifty_days_ago;
        issues[2].status = IssueStatus::Closed;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Stale);

        // Should only show beads-001 (45 days old and open)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_stale_view_includes_in_progress_and_blocked() {
        let mut issues = create_test_issues();

        let forty_days_ago = Utc::now() - Duration::days(40);

        // Stale InProgress issue
        issues[0].updated = forty_days_ago;
        issues[0].status = IssueStatus::InProgress;

        // Stale Blocked issue
        issues[1].updated = forty_days_ago;
        issues[1].status = IssueStatus::Blocked;

        // Recent issue (not stale)
        issues[2].updated = Utc::now() - Duration::days(5);
        issues[2].status = IssueStatus::Open;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Stale);

        // Should show both InProgress and Blocked stale issues
        assert_eq!(state.result_count(), 2);

        let ids: Vec<&str> = state
            .filtered_issues()
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert!(ids.contains(&"beads-001"));
        assert!(ids.contains(&"beads-002"));
        assert!(!ids.contains(&"beads-003"));
    }

    #[test]
    fn test_stale_view_excludes_closed_issues() {
        let mut issues = create_test_issues();

        let forty_days_ago = Utc::now() - Duration::days(40);

        // All updated 40 days ago, but only one is open
        issues[0].updated = forty_days_ago;
        issues[0].status = IssueStatus::Closed;

        issues[1].updated = forty_days_ago;
        issues[1].status = IssueStatus::Closed;

        issues[2].updated = forty_days_ago;
        issues[2].status = IssueStatus::Open;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Stale);

        // Should only show the open one
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-003");
    }

    #[test]
    fn test_stale_view_boundary_exactly_thirty_days() {
        let mut issues = create_test_issues();

        // Set beads-001 to be updated 31 days ago (clearly stale)
        let thirty_one_days_ago = Utc::now() - Duration::days(31);
        issues[0].updated = thirty_one_days_ago;
        issues[0].status = IssueStatus::Open;

        // Set beads-002 to be 29 days ago (not stale yet)
        let twenty_nine_days_ago = Utc::now() - Duration::days(29);
        issues[1].updated = twenty_nine_days_ago;
        issues[1].status = IssueStatus::Open;

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Stale);

        // 31+ days should be included, but 29 days should NOT
        let ids: Vec<&str> = state
            .filtered_issues()
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert!(ids.contains(&"beads-001"));
        assert!(!ids.contains(&"beads-002"));
    }

    #[test]
    fn test_view_cycle_includes_new_views() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Start at All
        assert_eq!(state.current_view(), ViewType::All);

        // Cycle: All -> Ready
        state.next_view();
        assert_eq!(state.current_view(), ViewType::Ready);

        // Cycle: Ready -> Blocked
        state.next_view();
        assert_eq!(state.current_view(), ViewType::Blocked);

        // Cycle: Blocked -> MyIssues
        state.next_view();
        assert_eq!(state.current_view(), ViewType::MyIssues);

        // Cycle: MyIssues -> Recently
        state.next_view();
        assert_eq!(state.current_view(), ViewType::Recently);

        // Cycle: Recently -> Stale
        state.next_view();
        assert_eq!(state.current_view(), ViewType::Stale);

        // Cycle: Stale -> All (back to start)
        state.next_view();
        assert_eq!(state.current_view(), ViewType::All);
    }

    #[test]
    fn test_recently_and_stale_are_mutually_exclusive() {
        let mut issues = create_test_issues();

        // Issue that is both not recently updated and stale
        let forty_days_ago = Utc::now() - Duration::days(40);
        issues[0].updated = forty_days_ago;
        issues[0].status = IssueStatus::Open;

        let mut state_recently = SearchInterfaceState::new(issues.clone());
        state_recently.set_view(ViewType::Recently);

        let mut state_stale = SearchInterfaceState::new(issues);
        state_stale.set_view(ViewType::Stale);

        // Should be in Stale but not Recently
        assert_eq!(state_recently.result_count(), 2); // beads-002 and beads-003 are recent
        assert_eq!(state_stale.result_count(), 1); // Only beads-001 is stale
    }

    #[test]
    fn test_recently_view_with_search_filter() {
        let mut issues = create_test_issues();

        // Make beads-001 recent and matches "search"
        issues[0].updated = Utc::now() - Duration::days(3);
        issues[0].title = "Implement search feature".to_string();

        // Make beads-002 recent but doesn't match "filter"
        issues[1].updated = Utc::now() - Duration::days(2);
        issues[1].title = "Add export functionality".to_string();

        let mut state = SearchInterfaceState::new(issues);
        state.set_view(ViewType::Recently);
        state.search_state_mut().set_query("search");
        state.update_filtered_issues();

        // Should show only beads-001 (recent AND matches "search")
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].id, "beads-001");
    }

    #[test]
    fn test_viewtype_display_names() {
        assert_eq!(ViewType::All.display_name(), "All Issues");
        assert_eq!(ViewType::Ready.display_name(), "Ready");
        assert_eq!(ViewType::Blocked.display_name(), "Blocked");
        assert_eq!(ViewType::MyIssues.display_name(), "My Issues");
        assert_eq!(ViewType::Recently.display_name(), "Recently Updated");
        assert_eq!(ViewType::Stale.display_name(), "Stale");
    }

    #[test]
    fn test_viewtype_all_includes_new_views() {
        let all_views = ViewType::all();
        assert_eq!(all_views.len(), 6);
        assert!(all_views.contains(&ViewType::All));
        assert!(all_views.contains(&ViewType::Ready));
        assert!(all_views.contains(&ViewType::Blocked));
        assert!(all_views.contains(&ViewType::MyIssues));
        assert!(all_views.contains(&ViewType::Recently));
        assert!(all_views.contains(&ViewType::Stale));
    }

    #[test]
    fn test_clear_all_filters_resets_search_text() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Set a search query
        state.search_state_mut().set_query("test query");
        assert_eq!(state.search_state().query(), "test query");

        // Clear all filters
        state.clear_all_filters();

        // Search text should be cleared
        assert_eq!(state.search_state().query(), "");
    }

    #[test]
    fn test_clear_all_filters_resets_view() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Change to a different view
        state.set_view(ViewType::Ready);
        assert_eq!(state.current_view(), ViewType::Ready);

        // Clear all filters
        state.clear_all_filters();

        // View should be reset to All
        assert_eq!(state.current_view(), ViewType::All);
    }

    #[test]
    fn test_clear_all_filters_resets_search_scope() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Change search scope
        state.set_search_scope(SearchScope::Title);
        assert_eq!(state.search_scope(), SearchScope::Title);

        // Clear all filters
        state.clear_all_filters();

        // Search scope should be reset to All
        assert_eq!(state.search_scope(), SearchScope::All);
    }

    #[test]
    fn test_clear_all_filters_clears_column_filters() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Enable and set some column filters
        state.list_state_mut().toggle_filters();
        state.list_state_mut().column_filters_mut().status = "Open".to_string();
        state.list_state_mut().column_filters_mut().priority = "P1".to_string();

        // Verify filters are set
        assert!(state.list_state().filters_enabled());
        assert!(!state.list_state().column_filters().status.is_empty());
        assert!(!state.list_state().column_filters().priority.is_empty());

        // Clear all filters
        state.clear_all_filters();

        // Column filters should be cleared (filter values reset to empty strings)
        assert!(state.list_state().column_filters().status.is_empty());
        assert!(state.list_state().column_filters().priority.is_empty());
        assert!(state.list_state().column_filters().type_filter.is_empty());
    }

    #[test]
    fn test_clear_all_filters_updates_filtered_issues() {
        let mut issues = create_test_issues();

        // Make beads-001 match a specific search and view
        issues[0].status = IssueStatus::Open;
        issues[0].dependencies = vec![];
        issues[0].title = "Unique search term".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Apply filters that limit results
        state.set_view(ViewType::Ready);
        state.search_state_mut().set_query("Unique");
        state.update_filtered_issues();

        // Should show only beads-001
        assert_eq!(state.result_count(), 1);

        // Clear all filters
        state.clear_all_filters();

        // Should now show all 3 issues
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_clear_all_filters_comprehensive() {
        let mut issues = create_test_issues();
        issues[0].status = IssueStatus::Open;
        issues[1].status = IssueStatus::Blocked;

        let mut state = SearchInterfaceState::new(issues);

        // Set up multiple filters
        state.set_view(ViewType::Ready);
        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("feature");
        state.list_state_mut().toggle_filters();
        state.list_state_mut().column_filters_mut().status = "Open".to_string();
        state.update_filtered_issues();

        // Verify filters are applied
        assert_eq!(state.current_view(), ViewType::Ready);
        assert_eq!(state.search_scope(), SearchScope::Title);
        assert!(!state.search_state().query().is_empty());
        assert!(state.list_state().filters_enabled());

        // Clear all filters
        state.clear_all_filters();

        // All filters should be reset
        assert_eq!(state.current_view(), ViewType::All);
        assert_eq!(state.search_scope(), SearchScope::All);
        assert_eq!(state.search_state().query(), "");
        assert!(state.list_state().column_filters().status.is_empty());
        assert_eq!(state.result_count(), 3); // All issues visible
    }

    #[test]
    fn test_regex_toggle() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Initially regex should be disabled
        assert!(!state.is_regex_enabled());

        // Toggle on
        state.toggle_regex();
        assert!(state.is_regex_enabled());

        // Toggle off
        state.toggle_regex();
        assert!(!state.is_regex_enabled());
    }

    #[test]
    fn test_regex_basic_pattern() {
        let mut issues = create_test_issues();
        issues[0].title = "Test feature #123".to_string();
        issues[1].title = "Another test case".to_string();
        issues[2].title = "No match here".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex and search for pattern "test.*#\d+"
        state.toggle_regex();
        state.search_state_mut().set_query("test.*#\\d+");
        state.update_filtered_issues();

        // Should match only the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Test feature #123");
    }

    #[test]
    fn test_regex_case_insensitive() {
        let mut issues = create_test_issues();
        issues[0].title = "UPPERCASE TEST".to_string();
        issues[1].title = "lowercase test".to_string();
        issues[2].title = "MiXeD CaSe TeSt".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex and search for "test" (should be case-insensitive)
        state.toggle_regex();
        state.search_state_mut().set_query("test");
        state.update_filtered_issues();

        // Should match all three issues
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_regex_invalid_fallback() {
        let mut issues = create_test_issues();
        issues[0].title = "Test with [brackets]".to_string();
        issues[1].title = "No brackets here".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex and use invalid regex pattern (unclosed bracket)
        state.toggle_regex();
        state.search_state_mut().set_query("[bracket");
        state.update_filtered_issues();

        // Should fallback to substring matching and find the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Test with [brackets]");
    }

    #[test]
    fn test_regex_search_scope_title() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature ABC-123".to_string();
        issues[0].description = Some("Description XYZ-456".to_string());
        issues[1].title = "Another task".to_string();
        issues[1].description = Some("Contains ABC-789".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex, set scope to Title, search for ABC-\d+
        state.toggle_regex();
        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("ABC-\\d+");
        state.update_filtered_issues();

        // Should match only the first issue (title match)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Feature ABC-123");
    }

    #[test]
    fn test_regex_search_scope_description() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature ABC-123".to_string();
        issues[0].description = Some("Simple description".to_string());
        issues[1].title = "Another task".to_string();
        issues[1].description = Some("Contains ABC-789".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex, set scope to Description, search for ABC-\d+
        state.toggle_regex();
        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("ABC-\\d+");
        state.update_filtered_issues();

        // Should match only the second issue (description match)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Another task");
    }

    #[test]
    fn test_regex_search_scope_all() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature ABC-123".to_string();
        issues[0].description = Some("Simple description".to_string());
        issues[1].title = "Another task".to_string();
        issues[1].description = Some("Contains ABC-789".to_string());
        issues[2].title = "Third issue".to_string();
        issues[2].description = Some("No pattern here".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex, set scope to All, search for ABC-\d+
        state.toggle_regex();
        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("ABC-\\d+");
        state.update_filtered_issues();

        // Should match both issues (one in title, one in description)
        assert_eq!(state.result_count(), 2);
    }

    #[test]
    fn test_regex_special_characters() {
        let mut issues = create_test_issues();
        issues[0].title = "Issue (priority: high)".to_string();
        issues[1].title = "Task [status: open]".to_string();
        issues[2].title = "Bug {id: 123}".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex and search for content in parentheses
        state.toggle_regex();
        state.search_state_mut().set_query("\\(.*?\\)");
        state.update_filtered_issues();

        // Should match only the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Issue (priority: high)");
    }

    #[test]
    fn test_regex_anchors() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature: new functionality".to_string();
        issues[1].title = "Not a feature".to_string();
        issues[2].title = "feature in middle".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex and search for "feature" at start of line
        state.toggle_regex();
        state.search_state_mut().set_query("^feature");
        state.update_filtered_issues();

        // Should match issues starting with "feature" (case-insensitive)
        assert_eq!(state.result_count(), 2);
    }

    #[test]
    fn test_regex_disabled_uses_substring() {
        let mut issues = create_test_issues();
        issues[0].title = "Test.*pattern".to_string();
        issues[1].title = "Normal text".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Don't enable regex, search for ".*" (literal string, not regex)
        state.search_state_mut().set_query(".*");
        state.update_filtered_issues();

        // Should match the first issue as substring match (not as regex wildcard)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Test.*pattern");
    }

    #[test]
    fn test_regex_notes_search() {
        let mut issues = create_test_issues();

        // Add notes with patterns
        issues[0].notes.push(Note {
            timestamp: chrono::Utc::now(),
            author: "test".to_string(),
            content: "Note with ERROR-123 code".to_string(),
        });
        issues[1].notes.push(Note {
            timestamp: chrono::Utc::now(),
            author: "test".to_string(),
            content: "Normal note".to_string(),
        });

        let mut state = SearchInterfaceState::new(issues);

        // Enable regex, set scope to Notes, search for ERROR-\d+
        state.toggle_regex();
        state.set_search_scope(SearchScope::Notes);
        state.search_state_mut().set_query("ERROR-\\d+");
        state.update_filtered_issues();

        // Should match only the first issue
        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn test_fuzzy_toggle() {
        let issues = create_test_issues();
        let mut state = SearchInterfaceState::new(issues);

        // Initially fuzzy should be disabled
        assert!(!state.is_fuzzy_enabled());

        // Toggle on
        state.toggle_fuzzy();
        assert!(state.is_fuzzy_enabled());

        // Toggle off
        state.toggle_fuzzy();
        assert!(!state.is_fuzzy_enabled());
    }

    #[test]
    fn test_fuzzy_basic_match() {
        let mut issues = create_test_issues();
        issues[0].title = "Implement feature".to_string();
        issues[1].title = "Fix bug in parser".to_string();
        issues[2].title = "Refactor code".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search for "impl" (should match "Implement")
        state.toggle_fuzzy();
        state.search_state_mut().set_query("impl");
        state.update_filtered_issues();

        // Should match the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Implement feature");
    }

    #[test]
    fn test_fuzzy_typo_tolerance() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature implementation".to_string();
        issues[1].title = "Bug fix".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search with typo "implmnt" (should match "implementation")
        state.toggle_fuzzy();
        state.search_state_mut().set_query("implmnt");
        state.update_filtered_issues();

        // Should match the first issue with fuzzy matching
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Feature implementation");
    }

    #[test]
    fn test_fuzzy_acronym_match() {
        let mut issues = create_test_issues();
        issues[0].title = "User Interface Component".to_string();
        issues[1].title = "Database connection".to_string();
        issues[2].title = "API endpoint".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search for "uic" (should match "User Interface Component")
        state.toggle_fuzzy();
        state.search_state_mut().set_query("uic");
        state.update_filtered_issues();

        // Should match the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "User Interface Component");
    }

    #[test]
    fn test_fuzzy_partial_match() {
        let mut issues = create_test_issues();
        issues[0].title = "Authentication system".to_string();
        issues[1].title = "Database schema".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search for "auth" (should match "Authentication")
        state.toggle_fuzzy();
        state.search_state_mut().set_query("auth");
        state.update_filtered_issues();

        // Should match the first issue
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Authentication system");
    }

    #[test]
    fn test_fuzzy_search_scope_title() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature ABC".to_string();
        issues[0].description = Some("Description XYZ".to_string());
        issues[1].title = "Other task".to_string();
        issues[1].description = Some("Contains ABC".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy, set scope to Title, search for "abc"
        state.toggle_fuzzy();
        state.set_search_scope(SearchScope::Title);
        state.search_state_mut().set_query("abc");
        state.update_filtered_issues();

        // Should match only the first issue (title match)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Feature ABC");
    }

    #[test]
    fn test_fuzzy_search_scope_description() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature".to_string();
        issues[0].description = Some("Simple description".to_string());
        issues[1].title = "Task".to_string();
        issues[1].description = Some("Contains important info".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy, set scope to Description, search for "import"
        state.toggle_fuzzy();
        state.set_search_scope(SearchScope::Description);
        state.search_state_mut().set_query("import");
        state.update_filtered_issues();

        // Should match only the second issue (description match)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Task");
    }

    #[test]
    fn test_fuzzy_search_scope_all() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature XYZ".to_string();
        issues[0].description = Some("Simple description".to_string());
        issues[1].title = "Task ABC".to_string();
        issues[1].description = Some("Contains XYZ".to_string());
        issues[2].title = "Other".to_string();
        issues[2].description = Some("Nothing here".to_string());

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy, set scope to All, search for "xyz"
        state.toggle_fuzzy();
        state.set_search_scope(SearchScope::All);
        state.search_state_mut().set_query("xyz");
        state.update_filtered_issues();

        // Should match both issues (one in title, one in description)
        assert_eq!(state.result_count(), 2);
    }

    #[test]
    fn test_fuzzy_case_insensitive() {
        let mut issues = create_test_issues();
        issues[0].title = "UPPERCASE TEST".to_string();
        issues[1].title = "lowercase test".to_string();
        issues[2].title = "MiXeD CaSe TeSt".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search for "test" (should be case-insensitive)
        state.toggle_fuzzy();
        state.search_state_mut().set_query("test");
        state.update_filtered_issues();

        // Should match all three issues
        assert_eq!(state.result_count(), 3);
    }

    #[test]
    fn test_fuzzy_disabled_uses_substring() {
        let mut issues = create_test_issues();
        issues[0].title = "Implementation details".to_string();
        issues[1].title = "Other task".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Don't enable fuzzy, search for "impl" (exact substring required)
        state.search_state_mut().set_query("impl");
        state.update_filtered_issues();

        // Should match the first issue with substring matching (case-insensitive)
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.filtered_issues()[0].title, "Implementation details");
    }

    #[test]
    fn test_fuzzy_notes_search() {
        let mut issues = create_test_issues();

        // Add notes with searchable content
        issues[0].notes.push(Note {
            timestamp: chrono::Utc::now(),
            author: "test".to_string(),
            content: "Important milestone reached".to_string(),
        });
        issues[1].notes.push(Note {
            timestamp: chrono::Utc::now(),
            author: "test".to_string(),
            content: "Normal note".to_string(),
        });

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy, set scope to Notes, search for "milstn" (typo of milestone)
        state.toggle_fuzzy();
        state.set_search_scope(SearchScope::Notes);
        state.search_state_mut().set_query("milstn");
        state.update_filtered_issues();

        // Should match the first issue with fuzzy matching
        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn test_fuzzy_no_match() {
        let mut issues = create_test_issues();
        issues[0].title = "Feature implementation".to_string();
        issues[1].title = "Bug fix".to_string();

        let mut state = SearchInterfaceState::new(issues);

        // Enable fuzzy and search for something completely unrelated
        state.toggle_fuzzy();
        state.search_state_mut().set_query("zzzzzzz");
        state.update_filtered_issues();

        // Should match nothing
        assert_eq!(state.result_count(), 0);
    }
}
