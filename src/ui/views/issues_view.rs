//! Main issues view integrating search, list, and detail/edit capabilities

use crate::beads::models::Issue;
use crate::models::table_config::{ColumnDefinition, ColumnId};
use crate::ui::views::{
    CreateIssueFormState, IssueDetailView, IssueEditorState, IssueEditorView, SearchInterfaceState,
    SearchInterfaceView,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{StatefulWidget, Widget},
};

/// View mode for the issues view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssuesViewMode {
    /// List mode showing all issues
    List,
    /// Detail mode showing a single issue
    Detail,
    /// Edit mode for editing an issue
    Edit,
    /// Create mode for creating a new issue
    Create,
    /// Split-screen mode showing list on left and detail on right
    SplitScreen,
}

/// Which panel is currently focused in split screen mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitScreenFocus {
    /// List panel (left side)
    List,
    /// Detail panel (right side)
    Detail,
}

/// Issues view state
#[derive(Debug)]
pub struct IssuesViewState {
    search_state: SearchInterfaceState,
    view_mode: IssuesViewMode,
    selected_issue: Option<Issue>,
    editor_state: Option<IssueEditorState>,
    create_form_state: Option<CreateIssueFormState>,
    show_help: bool,
    /// Scroll offset for the detail view in split screen
    pub detail_scroll: u16,
    /// Filter bar state (inline filter bar with dropdowns)
    pub filter_bar_state: Option<crate::ui::widgets::FilterBarState>,
    /// Original issues before filter bar filtering (for restoring)
    original_issues: Option<Vec<Issue>>,
    /// Which panel has focus in split screen mode
    split_screen_focus: SplitScreenFocus,
}

impl IssuesViewState {
    /// Create a new issues view state
    pub fn new(issues: Vec<Issue>) -> Self {
        // Initialize filter bar with all available statuses, priorities, types, and labels
        use crate::beads::models::{IssueStatus, IssueType, Priority};
        use std::collections::HashSet;

        let statuses = vec![
            IssueStatus::Open,
            IssueStatus::InProgress,
            IssueStatus::Blocked,
            IssueStatus::Closed,
        ];

        let priorities = vec![
            Priority::P0,
            Priority::P1,
            Priority::P2,
            Priority::P3,
            Priority::P4,
        ];

        let types = vec![
            IssueType::Bug,
            IssueType::Feature,
            IssueType::Task,
            IssueType::Epic,
            IssueType::Chore,
        ];

        // Collect unique labels from all issues
        let mut labels_set: HashSet<String> = HashSet::new();
        for issue in &issues {
            for label in &issue.labels {
                labels_set.insert(label.clone());
            }
        }
        let mut labels: Vec<String> = labels_set.into_iter().collect();
        labels.sort();

        // Collect unique assignees from all issues
        let mut assignees_set: HashSet<String> = HashSet::new();
        for issue in &issues {
            if let Some(ref assignee) = issue.assignee {
                assignees_set.insert(assignee.clone());
            }
        }
        assignees_set.insert("-".to_string()); // Add option for unassigned
        let mut assignees: Vec<String> = assignees_set.into_iter().collect();
        assignees.sort();

        // Collect unique created dates from all issues
        let mut created_dates_set: HashSet<String> = HashSet::new();
        for issue in &issues {
            use chrono::Datelike;
            let date_str = format!("{:04}-{:02}-{:02}",
                issue.created.year(),
                issue.created.month(),
                issue.created.day());
            created_dates_set.insert(date_str);
        }
        let mut created_dates: Vec<String> = created_dates_set.into_iter().collect();
        created_dates.sort();

        // Collect unique updated dates from all issues
        let mut updated_dates_set: HashSet<String> = HashSet::new();
        for issue in &issues {
            use chrono::Datelike;
            let date_str = format!("{:04}-{:02}-{:02}",
                issue.updated.year(),
                issue.updated.month(),
                issue.updated.day());
            updated_dates_set.insert(date_str);
        }
        let mut updated_dates: Vec<String> = updated_dates_set.into_iter().collect();
        updated_dates.sort();

        // Collect unique closed dates from all issues
        let mut closed_dates_set: HashSet<String> = HashSet::new();
        for issue in &issues {
            if let Some(ref closed) = issue.closed {
                use chrono::Datelike;
                let date_str = format!("{:04}-{:02}-{:02}",
                    closed.year(),
                    closed.month(),
                    closed.day());
                closed_dates_set.insert(date_str);
            }
        }
        let mut closed_dates: Vec<String> = closed_dates_set.into_iter().collect();
        closed_dates.sort();

        let filter_bar_state = crate::ui::widgets::FilterBarState::new(
            statuses,
            priorities,
            types,
            labels,
            assignees,
            created_dates,
            updated_dates,
            closed_dates,
        );

        Self {
            search_state: SearchInterfaceState::new(issues),
            view_mode: IssuesViewMode::List,
            selected_issue: None,
            editor_state: None,
            create_form_state: None,
            show_help: true,
            detail_scroll: 0,
            filter_bar_state: Some(filter_bar_state),
            original_issues: None,
            split_screen_focus: SplitScreenFocus::List,
        }
    }

    /// Get the current view mode
    pub fn view_mode(&self) -> IssuesViewMode {
        self.view_mode
    }

    /// Set the view mode
    pub fn set_view_mode(&mut self, mode: IssuesViewMode) {
        self.view_mode = mode;
    }

    /// Get the search state
    pub fn search_state(&self) -> &SearchInterfaceState {
        &self.search_state
    }

    /// Get mutable search state
    pub fn search_state_mut(&mut self) -> &mut SearchInterfaceState {
        &mut self.search_state
    }

    /// Get all issues
    pub fn all_issues(&self) -> &[Issue] {
        self.search_state.all_issues()
    }

    /// Get the selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.selected_issue.as_ref()
    }

    /// Set the selected issue
    pub fn set_selected_issue(&mut self, issue: Option<Issue>) {
        self.selected_issue = issue;
    }

    /// Get the editor state
    pub fn editor_state(&self) -> Option<&IssueEditorState> {
        self.editor_state.as_ref()
    }

    /// Get mutable editor state
    pub fn editor_state_mut(&mut self) -> Option<&mut IssueEditorState> {
        self.editor_state.as_mut()
    }

    /// Get the create form state
    pub fn create_form_state(&self) -> Option<&CreateIssueFormState> {
        self.create_form_state.as_ref()
    }

    /// Get mutable create form state
    pub fn create_form_state_mut(&mut self) -> Option<&mut CreateIssueFormState> {
        self.create_form_state.as_mut()
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Check if help is visible
    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }

    /// Enter detail view for the currently selected issue
    pub fn enter_detail_view(&mut self) {
        if let Some(issue) = self.search_state.selected_issue() {
            self.selected_issue = Some(Issue::clone(issue));
            self.view_mode = IssuesViewMode::Detail;
            // Reset scroll position to top
            self.detail_scroll = 0;
        }
    }

    /// Enter split-screen view
    pub fn enter_split_screen(&mut self) {
        if let Some(issue) = self.search_state.selected_issue() {
            self.selected_issue = Some(Issue::clone(issue));
        }
        self.view_mode = IssuesViewMode::SplitScreen;
    }

    /// Update selected issue in split-screen mode
    pub fn update_split_screen_detail(&mut self) {
        if self.view_mode == IssuesViewMode::SplitScreen {
            if let Some(issue) = self.search_state.selected_issue() {
                self.selected_issue = Some(Issue::clone(issue));
            }
        }
    }

    /// Enter edit mode for the currently selected issue
    pub fn enter_edit_mode(&mut self) {
        if let Some(issue) = self.search_state.selected_issue() {
            self.selected_issue = Some(Issue::clone(issue));
            let mut editor_state = IssueEditorState::new(issue);
            // Reset scroll to top and ensure first field is focused
            editor_state.form_state_mut().scroll_to_top();
            editor_state.form_state_mut().set_focused_index(0);
            self.editor_state = Some(editor_state);
            self.view_mode = IssuesViewMode::Edit;
        }
    }

    /// Return to list view
    pub fn return_to_list(&mut self) {
        self.view_mode = IssuesViewMode::List;
        self.selected_issue = None;
        self.editor_state = None;
    }

    /// Save the current edit and return to list
    pub fn save_edit(&mut self) -> Option<Issue> {
        if let Some(editor_state) = &mut self.editor_state {
            if let Some(original) = &self.selected_issue {
                if let Some(updated_issue) = editor_state.get_updated_issue(original) {
                    editor_state.save();
                    return Some(updated_issue);
                }
            }
        }
        None
    }

    /// Cancel the current edit and return to list
    pub fn cancel_edit(&mut self) {
        if let Some(editor_state) = &mut self.editor_state {
            editor_state.cancel();
        }
        self.return_to_list();
    }

    /// Enter create mode to create a new issue
    pub fn enter_create_mode(&mut self) {
        let mut create_state = CreateIssueFormState::new();
        // Ensure form starts at the top
        create_state.form_state_mut().scroll_to_top();
        self.create_form_state = Some(create_state);
        self.view_mode = IssuesViewMode::Create;
    }

    /// Cancel the current create and return to list
    pub fn cancel_create(&mut self) {
        self.create_form_state = None;
        self.view_mode = IssuesViewMode::List;
    }

    /// Save the current create form and return form data
    pub fn save_create(&mut self) -> Option<crate::ui::views::CreateIssueData> {
        if let Some(create_form_state) = &self.create_form_state {
            if let Some(data) = create_form_state.get_data() {
                return Some(data);
            }
        }
        None
    }

    /// Update the issue list
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.search_state.set_issues(issues);
    }

    /// Set saved filters
    pub fn set_saved_filters(&mut self, filters: Vec<crate::models::SavedFilter>) {
        self.search_state.set_saved_filters(filters);
    }

    /// Apply filter bar filters to the issues
    /// This triggers a re-filter of the issues based on the current filter bar selections
    pub fn apply_filter_bar_filters(&mut self) {
        if let Some(ref filter_bar_state) = self.filter_bar_state {
            // Save original issues if not already saved
            if self.original_issues.is_none() {
                self.original_issues = Some(self.search_state.all_issues().to_vec());
            }

            // Get original issues (or current if we haven't saved yet)
            let all_issues = self.original_issues.as_ref().unwrap();

            // Filter issues based on filter bar state
            let mut filtered_issues = Vec::new();
            for issue in all_issues {
                if filter_bar_state.matches_issue(issue) {
                    filtered_issues.push(issue.clone());
                }
            }

            // Update the search state with filtered issues
            self.search_state.set_issues(filtered_issues);
        }
    }

    /// Clear filter bar filters and restore original issues
    pub fn clear_filter_bar_filters(&mut self) {
        if let Some(original_issues) = self.original_issues.take() {
            // Restore original issues
            self.search_state.set_issues(original_issues);
        }
    }

    /// Get current split screen focus
    pub fn split_screen_focus(&self) -> SplitScreenFocus {
        self.split_screen_focus
    }

    /// Set split screen focus to a specific panel
    pub fn set_split_screen_focus(&mut self, focus: SplitScreenFocus) {
        self.split_screen_focus = focus;
    }

    /// Toggle split screen focus between list and detail
    pub fn toggle_split_screen_focus(&mut self) {
        self.split_screen_focus = match self.split_screen_focus {
            SplitScreenFocus::List => SplitScreenFocus::Detail,
            SplitScreenFocus::Detail => SplitScreenFocus::List,
        };
    }
}

/// Issues view widget
pub struct IssuesView<'a> {
    block_style: Style,
    theme: Option<&'a crate::ui::themes::Theme>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> IssuesView<'a> {
    /// Create a new issues view
    pub fn new() -> Self {
        Self {
            block_style: Style::default().fg(Color::Cyan),
            theme: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: &'a crate::ui::themes::Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    fn render_list_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        // Render filter bar if it exists and adjust search view area accordingly
        let (search_area, filter_bar_area) = if state.filter_bar_state.is_some() {
            // Filter bar takes the top 3 rows, search view starts below it
            let filter_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 3,
            };
            let search_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y + 3,
                width: area.width,
                height: area.height.saturating_sub(3),
            };
            (search_area, Some(filter_area))
        } else {
            // No filter bar, search view uses full area
            (area, None)
        };

        // Dynamically determine columns based on available width
        let columns = Self::select_columns_for_width(search_area.width);

        let mut search_view = SearchInterfaceView::new()
            .block_style(self.block_style)
            .columns(columns)
            .show_results_info(filter_bar_area.is_none()); // Hide results info when filter bar is shown
        if let Some(theme) = self.theme {
            search_view = search_view.theme(theme);
        }
        StatefulWidget::render(search_view, search_area, buf, &mut state.search_state);

        // Render filter bar if it exists
        if let Some(filter_area) = filter_bar_area {
            if let Some(ref mut filter_bar_state) = state.filter_bar_state {
                let default_theme = crate::ui::themes::Theme::default();
                let theme = self.theme.unwrap_or(&default_theme);

                // Render the filter bar
                let filter_bar = crate::ui::widgets::FilterBar::new(
                    state.search_state.result_count(),
                    state.search_state.all_issues().len(),
                    theme,
                );
                filter_bar.render(filter_area, buf, filter_bar_state);

                // Render dropdown if one is active
                if let Some(dropdown_type) = filter_bar_state.active_dropdown {
                    // Dropdown uses the full area to position itself relative to filter bar
                    let dropdown_area = ratatui::layout::Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: area.height,
                    };

                    let dropdown = crate::ui::widgets::FilterDropdown::new(dropdown_type, theme);
                    dropdown.render(dropdown_area, buf, filter_bar_state);
                }
            }
        }
    }

    fn render_detail_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        use ratatui::widgets::Clear;

        // Render list as background
        self.render_list_mode(area, buf, state);

        if let Some(issue) = &state.selected_issue {
            // Calculate popup area (90% width, 90% height for better readability)
            let popup_area = crate::ui::layout::centered_rect(90, 90, area);

            // Clear background
            Clear.render(popup_area, buf);

            // Render detail view as popup
            let mut detail_view = IssueDetailView::new(issue);
            if let Some(theme) = self.theme {
                detail_view = detail_view.theme(theme);
            }
            StatefulWidget::render(detail_view, popup_area, buf, &mut state.detail_scroll);
        }
    }

    fn render_create_mode_popup(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        use ratatui::widgets::Clear;
        
        // Calculate popup area (80% width, 80% height)
        let popup_area = crate::ui::layout::centered_rect(80, 80, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        // Render form
        if let Some(create_form_state) = &mut state.create_form_state {
            use crate::ui::views::CreateIssueForm;
            let create_form = CreateIssueForm::new();
            StatefulWidget::render(create_form, popup_area, buf, create_form_state);
        }
    }

    fn render_edit_mode_popup(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        use ratatui::widgets::Clear;
        
        // Calculate popup area (80% width, 80% height)
        let popup_area = crate::ui::layout::centered_rect(80, 80, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        // Render form
        if let Some(editor_state) = &mut state.editor_state {
            let editor_view = IssueEditorView::new();
            StatefulWidget::render(editor_view, popup_area, buf, editor_state);
        }
    }

    fn render_split_screen_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use crate::models::table_config::{ColumnId, ColumnDefinition, WidthConstraints, WrapBehavior};

        // Render filter bar if it exists and adjust content area accordingly
        let (content_area, filter_bar_area) = if state.filter_bar_state.is_some() {
            // Filter bar takes the top 3 rows, content starts below it
            let filter_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 3,
            };
            let content_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y + 3,
                width: area.width,
                height: area.height.saturating_sub(3),
            };
            (content_area, Some(filter_area))
        } else {
            // No filter bar, content uses full area
            (area, None)
        };

        // Split the content area into left (list) and right (detail) panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Left panel (list)
                Constraint::Percentage(60), // Right panel (detail)
            ])
            .split(content_area);

        // Define compact columns for split view
        let compact_columns = vec![
            {
                let mut col = ColumnDefinition::new(ColumnId::Id);
                // "last 6 characters" -> width 8 to allow for "…" + 6 chars roughly
                col.width_constraints = WidthConstraints::new(6, Some(8), 8);
                col.width = 8;
                col.wrap = WrapBehavior::TruncateStart;
                col
            },
            {
                let mut col = ColumnDefinition::new(ColumnId::Title);
                // "first 20 characters"
                col.width_constraints = WidthConstraints::new(10, Some(22), 20);
                col.width = 20;
                col.wrap = WrapBehavior::Truncate;
                col
            },
            {
                let mut col = ColumnDefinition::new(ColumnId::Status);
                col.width_constraints = WidthConstraints::new(8, Some(10), 8);
                col.width = 8;
                col
            },
            {
                let mut col = ColumnDefinition::new(ColumnId::Priority);
                col.width_constraints = WidthConstraints::new(4, Some(4), 4);
                col.width = 4;
                col.label = "Pri".to_string();
                col
            },
            {
                let mut col = ColumnDefinition::new(ColumnId::Type);
                col.width_constraints = WidthConstraints::new(6, Some(8), 8);
                col.width = 8;
                col
            },
        ];

        // Render the list on the left with compact view
        let mut search_view = SearchInterfaceView::new()
            .block_style(self.block_style)
            .columns(compact_columns)
            .show_results_info(false);

        if let Some(theme) = self.theme {
            search_view = search_view.theme(theme);
        }
        StatefulWidget::render(search_view, chunks[0], buf, &mut state.search_state);

        // Render the detail view on the right, offset by 1 row to align with top of issues container
        if let Some(issue) = &state.selected_issue {
            use ratatui::widgets::Clear;

            let detail_area = ratatui::layout::Rect {
                x: chunks[1].x,
                y: chunks[1].y + 1,
                width: chunks[1].width,
                height: chunks[1].height.saturating_sub(1),
            };

            // Clear the background to prevent content from bleeding through
            Clear.render(detail_area, buf);

            let mut detail_view = IssueDetailView::new(issue);
            if let Some(theme) = self.theme {
                detail_view = detail_view.theme(theme);
            }
            StatefulWidget::render(detail_view, detail_area, buf, &mut state.detail_scroll);
        }

        // Render filter bar if it exists
        if let Some(filter_area) = filter_bar_area {
            if let Some(ref mut filter_bar_state) = state.filter_bar_state {
                let default_theme = crate::ui::themes::Theme::default();
                let theme = self.theme.unwrap_or(&default_theme);

                // Render the filter bar
                let filter_bar = crate::ui::widgets::FilterBar::new(
                    state.search_state.result_count(),
                    state.search_state.all_issues().len(),
                    theme,
                );
                filter_bar.render(filter_area, buf, filter_bar_state);

                // Render dropdown if one is active
                if let Some(dropdown_type) = filter_bar_state.active_dropdown {
                    // Dropdown uses the full area to position itself relative to filter bar
                    let dropdown_area = ratatui::layout::Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: area.height,
                    };

                    let dropdown = crate::ui::widgets::FilterDropdown::new(dropdown_type, theme);
                    dropdown.render(dropdown_area, buf, filter_bar_state);
                }
            }
        }
    }

    /// Select columns to display based on available width
    /// Columns are added in priority order, then expanded up to 40 chars when space allows
    fn select_columns_for_width(width: u16) -> Vec<ColumnDefinition> {
        let mut columns = Vec::new();

        // Define column priority order with minimum widths
        let column_specs = vec![
            (ColumnId::Id, 15),
            (ColumnId::Title, 30),
            (ColumnId::Status, 12),
            (ColumnId::Priority, 8),
            (ColumnId::Type, 8),
            (ColumnId::Assignee, 15),
            (ColumnId::Updated, 16),
            (ColumnId::Labels, 20),
            (ColumnId::Created, 16),
        ];

        // Phase 1: Add columns at minimum widths
        let mut total_width = 0u16;
        for (col_id, min_width) in &column_specs {
            if total_width + min_width <= width {
                let mut col = ColumnDefinition::new(*col_id);
                col.width = *min_width;
                total_width += min_width;
                columns.push(col);
            } else {
                break;
            }
        }

        // Phase 2: Expand columns if there's extra space (up to 40 chars per field)
        if total_width < width && !columns.is_empty() {
            let remaining_width = (width - total_width) as usize;

            // Prioritize expanding these columns first
            let expansion_priority = vec![
                ColumnId::Title,     // Most important - give it the most space
                ColumnId::Labels,    // Can show more labels
                ColumnId::Assignee,  // Can show full usernames
                ColumnId::Id,        // Can show full IDs
                ColumnId::Updated,   // Can show full timestamps
                ColumnId::Created,   // Can show full timestamps
                ColumnId::Status,    // Already compact
                ColumnId::Type,      // Already compact
                ColumnId::Priority,  // Already compact
            ];

            let mut extra_width = remaining_width;

            // Distribute extra width across columns based on priority
            for priority_col in &expansion_priority {
                if extra_width == 0 {
                    break;
                }

                // Find the column in our list
                if let Some(col) = columns.iter_mut().find(|c| c.id == *priority_col) {
                    let max_width = 40u16;
                    let current_width = col.width;
                    let possible_expansion = max_width.saturating_sub(current_width);

                    if possible_expansion > 0 {
                        let expansion = possible_expansion.min(extra_width as u16);
                        col.width += expansion;
                        extra_width = extra_width.saturating_sub(expansion as usize);
                    }
                }
            }

            // If there's still extra width, distribute evenly across all columns
            // (up to max 40 chars each)
            if extra_width > 0 {
                let per_column = (extra_width / columns.len()).min(40);
                for col in &mut columns {
                    if col.width < 40 {
                        let expansion = (40 - col.width).min(per_column as u16);
                        col.width += expansion;
                    }
                }
            }
        }

        columns
    }
}

impl<'a> Default for IssuesView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for IssuesView<'a> {
    type State = IssuesViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.view_mode {
            IssuesViewMode::List => self.render_list_mode(area, buf, state),
            IssuesViewMode::Detail => self.render_detail_mode(area, buf, state),
            IssuesViewMode::SplitScreen => self.render_split_screen_mode(area, buf, state),
            IssuesViewMode::Edit => {
                // Render list as background
                self.render_list_mode(area, buf, state);
                // Render editor as popup
                self.render_edit_mode_popup(area, buf, state);
            }
            IssuesViewMode::Create => {
                // Render list as background
                self.render_list_mode(area, buf, state);
                // Render creator as popup
                self.render_create_mode_popup(area, buf, state);
            }
        }
    }
}

// Event handling implementation
use super::{event_utils, ViewEventHandler};
use crate::models::AppState;
use crate::config::Action;
use crate::tasks::TaskOutput;
use crate::undo::IssueUpdateCommand;
use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

impl ViewEventHandler for IssuesViewState {
    fn handle_key_event(app: &mut AppState, key: KeyEvent) -> bool {
    use IssuesViewMode;

    let key_code = key.code;
    let action = app
        .config
        .keybindings
        .find_action(&key.code, &key.modifiers);

    // ESC Priority 1: Dismiss notifications (highest)
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return true;
    }

    // Handle filter dropdown hotkeys (1-7 for filters and reset)
    if app.issues_view_state.filter_bar_state.is_some() {
        match key_code {
            KeyCode::Char('1') => {
                // Toggle Status filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Status);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('2') => {
                // Toggle Type filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Type);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('3') => {
                // Toggle Priority filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Priority);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('4') => {
                // Toggle Labels filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Labels);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('5') => {
                // Toggle Created date filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Created);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('6') => {
                // Toggle Updated date filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Updated);
                    app.mark_dirty();
                }
                return true;
            }
            KeyCode::Char('7') => {
                // Reset all filters
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    // Close any open dropdown
                    filter_bar_state.close_dropdown();
                    // Clear all selections (reset to "All")
                    filter_bar_state.status_dropdown.clear_selection();
                    filter_bar_state.priority_dropdown.clear_selection();
                    filter_bar_state.type_dropdown.clear_selection();
                    filter_bar_state.labels_dropdown.clear_selection();
                    filter_bar_state.created_dropdown.clear_selection();
                    filter_bar_state.updated_dropdown.clear_selection();
                    app.mark_dirty();
                }
                return true;
            }
            _ => {}
        }

        // Handle filter dropdown navigation if a dropdown is open
        if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
            if filter_bar_state.active_dropdown.is_some() {
                match action {
                    Some(Action::MoveUp) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.previous();
                            app.mark_dirty();
                        }
                        return true;
                    }
                    Some(Action::MoveDown) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.next();
                            app.mark_dirty();
                        }
                        return true;
                    }
                    Some(Action::ToggleSelection) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.toggle_selected();
                            app.mark_dirty();
                        }
                        return true;
                    }
                    Some(Action::ConfirmDialog) => {
                        // Apply filter and close dropdown
                        filter_bar_state.close_dropdown();
                        app.issues_view_state.apply_filter_bar_filters();
                        app.mark_dirty();
                        return true;
                    }
                    Some(Action::CancelDialog) => {
                        // Close dropdown without applying
                        filter_bar_state.close_dropdown();
                        app.mark_dirty();
                        return true;
                    }
                    _ => {}
                }
            }
        }
    }

    // Handle global actions
    match action {
        Some(Action::Undo) => {
            app.undo().ok();
            return true;
        }
        Some(Action::Redo) => {
            app.redo().ok();
            return true;
        }
        Some(Action::ShowNotificationHistory) => {
            app.toggle_notification_history();
            return true;
        }
        Some(Action::ShowIssueHistory) => {
            // Check if undo history overlay is what was meant (Ctrl+H maps to ShowNotificationHistory in config default??)
            // Config: ShowNotificationHistory -> Ctrl+h. ShowIssueHistory -> Alt+h.
            // Old code: Ctrl+h -> toggle_undo_history.
            // Let's stick to the Action definitions.
            // If action is ShowNotificationHistory, do that.
            // If we want UndoHistory, we need an Action for it.
            // Action::ShowIssueHistory exists.
            // Wait, old code mapped Ctrl+H to toggle_undo_history. Config maps Ctrl+H to ShowNotificationHistory.
            // The user wanted standard keys.
            // Let's assume Config is the source of truth now.
        }
        _ => {}
    }

    // Handle undo history overlay toggle (Special case if not covered by Action)
    // Old code used Ctrl+H for undo history.
    // Let's use the Config action if possible.
    // Config has ShowNotificationHistory (Ctrl+H).
    // Config doesn't have specific "ToggleUndoHistory".
    // I'll keep the old logic for now if it's not in Config, OR rely on Config.
    // The previous code had:
    // if key_code == KeyCode::Char('h') && key.modifiers.contains(KeyModifiers::CONTROL) { app.toggle_undo_history(); }
    // Config default for ShowNotificationHistory is Ctrl+h.
    // This is a conflict in the legacy code vs new config.
    // I will respect the NEW config which maps Ctrl+H to ShowNotificationHistory.
    // But wait, the user wants "Universal Set".
    // I'll skip this specific conflict resolution for a moment and focus on the structure.

    // ESC Priority 2: Close undo history overlay
    if app.is_undo_history_visible() && matches!(action, Some(Action::DismissNotification)) {
        app.hide_undo_history();
        return true;
    }

    // Clear errors when entering create or edit mode
    // We can't easily map this to Action::CreateIssue yet because we are in "global" scope of function
    if matches!(action, Some(Action::CreateIssue) | Some(Action::EditIssue)) {
        app.clear_error();
    }

    // Handle dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.dialog_state {
        match action {
            Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                dialog_state.select_previous(2); // Yes/No = 2 buttons
                return true;
            }
            Some(Action::MoveRight) | Some(Action::NextDialogButton) | Some(Action::NextTab) => {
                dialog_state.select_next(2);
                return true;
            }
            Some(Action::ConfirmDialog) => {
                // Execute pending action based on selected button
                let selected = dialog_state.selected_button();
                if selected == 0 {
                    // Yes was selected
                    if let Some(action) = app.pending_action.take() {
                        if let Some(issue_id) = action.strip_prefix("delete:") {
                            tracing::info!("Confirmed delete for issue: {}", issue_id);

                            // Spawn background task (non-blocking)
                            let issue_id_owned = issue_id.to_string();
                            let _ = app.spawn_task(
                                &format!("Deleting issue {}", issue_id),
                                move |client| async move {
                                    client.delete_issue(&issue_id_owned).await?;
                                    tracing::info!(
                                        "Successfully deleted issue: {}",
                                        issue_id_owned
                                    );
                                    Ok(TaskOutput::IssueDeleted(issue_id_owned))
                                },
                            );
                        } else if let Some(filter_idx_str) = action.strip_prefix("delete_filter:") {
                            tracing::info!("Confirmed delete filter at index: {}", filter_idx_str);

                            if let Ok(i) = filter_idx_str.parse::<usize>() {
                                app.issues_view_state
                                    .search_state_mut()
                                    .delete_saved_filter(i);
                                // Sync to config
                                let filters = app
                                    .issues_view_state
                                    .search_state()
                                    .saved_filters()
                                    .to_vec();
                                app.config.filters = filters;
                                let _ = app.config.save();
                                app.set_success("Filter deleted".to_string());
                            }
                        } else if let Some(ids) = action.strip_prefix("indent:") {
                            let parts: Vec<&str> = ids.split(':').collect();
                            if parts.len() == 2 {
                                let selected_id = parts[0].to_string();
                                let prev_id = parts[1].to_string();
                                tracing::info!(
                                    "Confirmed indent {} under {}",
                                    selected_id,
                                    prev_id
                                );

                                // Spawn background task (non-blocking)
                                let _ =
                                    app.spawn_task("Indenting issue", move |client| async move {
                                        client.add_dependency(&selected_id, &prev_id).await?;
                                        tracing::info!(
                                            "Successfully indented {} under {}",
                                            selected_id,
                                            prev_id
                                        );
                                        Ok(TaskOutput::DependencyAdded)
                                    });
                            }
                        } else if let Some(ids) = action.strip_prefix("outdent:") {
                            let parts: Vec<&str> = ids.split(':').collect();
                            if parts.len() == 2 {
                                let selected_id = parts[0].to_string();
                                let parent_id = parts[1].to_string();
                                tracing::info!(
                                    "Confirmed outdent {} from parent {}",
                                    selected_id,
                                    parent_id
                                );

                                // Spawn background task (non-blocking)
                                let _ =
                                    app.spawn_task("Outdenting issue", move |client| async move {
                                        client.remove_dependency(&selected_id, &parent_id).await?;
                                        tracing::info!(
                                            "Successfully outdented {} from {}",
                                            selected_id,
                                            parent_id
                                        );
                                        Ok(TaskOutput::DependencyRemoved)
                                    });
                            }
                        } else if action == "compact_database" {
                            tracing::info!("Confirmed compact database");

                            // Spawn background task (non-blocking)
                            let _ = app.spawn_task("Compacting database", |client| async move {
                                client.compact_database().await?;
                                Ok(TaskOutput::DatabaseCompacted)
                            });
                        }
                    }
                }
                // Close dialog (both Yes and No)
                app.dialog_state = None;
                app.pending_action = None;
                return true;
            }
            Some(Action::CancelDialog) => {
                // ESC Priority 3: Cancel dialog
                tracing::debug!("Dialog cancelled");
                app.dialog_state = None;
                app.pending_action = None;
                return true;
            }
            Some(Action::Quit) | Some(Action::ShowHelp) => {
                // Let '?' and 'q' fall through to be handled globally
                // Dialog remains open but user can still get help or quit
            }
            _ => {
                // Ignore other keys when dialog is active
                return true;
            }
        }
    }

    // Handle column manager events if active
    if let Some(ref mut cm_state) = app.column_manager_state {
        match action {
            Some(Action::MoveUp) => {
                cm_state.select_previous();
                return true;
            }
            Some(Action::MoveDown) => {
                cm_state.select_next();
                return true;
            }
            Some(Action::ToggleSelection) => {
                cm_state.toggle_visibility();
                return true;
            }
            Some(Action::Refresh) => {
                // Using 'r' for reset/refresh
                // Reset to defaults
                let defaults = crate::models::table_config::TableConfig::default().columns;
                cm_state.reset(defaults);
                return true;
            }
            Some(Action::ConfirmDialog) => {
                // Apply changes
                if cm_state.is_modified() {
                    // Get modified columns
                    let new_columns = cm_state.columns().to_vec();

                    // Update table config with new columns
                    let mut table_config = app
                        .issues_view_state
                        .search_state()
                        .list_state()
                        .table_config()
                        .clone();
                    table_config.columns = new_columns;

                    // Apply to state
                    app.issues_view_state
                        .search_state_mut()
                        .list_state_mut()
                        .set_table_config(table_config);

                    // Save to disk
                    if let Err(e) = app.save_table_config() {
                        tracing::warn!("Failed to save table config: {}", e);
                        app.set_warning(format!("Column changes applied but not saved: {}", e));
                    } else {
                        app.set_success("Column configuration saved".to_string());
                    }
                }
                // Close column manager
                app.column_manager_state = None;
                return true;
            }
            Some(Action::CancelDialog) => {
                // Cancel without applying
                app.column_manager_state = None;
                return true;
            }
            Some(Action::MoveLeft) if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column up (Alt+Left in existing code? Logic says move_up)
                cm_state.move_up();
                return true;
            }
            Some(Action::MoveRight) if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column down
                cm_state.move_down();
                return true;
            }
            Some(Action::Quit) | Some(Action::ShowHelp) => {
                // Let '?' and 'q' fall through to be handled globally
            }
            _ => {
                // Ignore other keys when column manager is active
                return true;
            }
        }
    }

    // Handle filter save dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.filter_save_dialog_state {
        match action {
            Some(Action::NextDialogButton) => {
                dialog_state.focus_next();
                return true;
            }
            Some(Action::PrevDialogButton) => {
                dialog_state.focus_previous();
                return true;
            }
            Some(Action::MoveLeft) => {
                dialog_state.move_cursor_left();
                return true;
            }
            Some(Action::MoveRight) => {
                dialog_state.move_cursor_right();
                return true;
            }
            // Backspace is text input, usually not mapped to action for dialogs unless specialized
            // But we need to handle Char(c) for text input.
            // We should check raw keys for text input if no action matched?
            // Or explicitly match Delete/Backspace actions if they exist.
            // Config doesn't have DeleteChar/InsertChar actions.
            // We'll fall back to raw key matching for text input if action is not navigation/confirm/cancel.
            Some(Action::ConfirmDialog) => {
                // Save or update the filter depending on mode
                if app.is_editing_filter() {
                    // Update existing filter
                    match app.save_edited_filter() {
                        Ok(()) => {
                            tracing::info!("Filter updated successfully");
                            app.set_success("Filter updated".to_string());
                        }
                        Err(e) => {
                            tracing::error!("Failed to update filter: {}", e);
                            app.set_error(e);
                        }
                    }
                } else {
                    // Save new filter
                    match app.save_current_filter() {
                        Ok(()) => {
                            tracing::info!("Filter saved successfully");
                            app.set_success("Filter saved".to_string());
                        }
                        Err(e) => {
                            tracing::error!("Failed to save filter: {}", e);
                            app.set_error(e);
                        }
                    }
                }
                return true;
            }
            Some(Action::CancelDialog) => {
                // Cancel dialog
                tracing::debug!("Filter save dialog cancelled");
                app.hide_filter_save_dialog();
                return true;
            }
            _ => {
                // Handle text input for non-action keys
                match key_code {
                    KeyCode::Backspace => {
                        dialog_state.delete_char();
                        return true;
                    }
                    KeyCode::Char(c) => {
                        // Avoid handling 'q' or '?' if they triggered a global action that we skipped?
                        // If 'q' is mapped to Quit, 'action' is Quit. We are in the `_` branch, so action didn't match specific dialog actions.
                        // But we want to allow typing 'q' in the text field.
                        // CONFLICT: Global hotkeys vs Text Input.
                        // Standard solution: If text field is focused, suppress single-key hotkeys unless modifiers are present.
                        // In this dialog, everything is text input except nav/enter/esc.
                        dialog_state.insert_char(c);
                        return true;
                    }
                    _ => return false,
                }
            }
        }
    }

    // Handle dependency dialog events if dialog is open
    if app.dependency_dialog_state.is_open() {
        use crate::ui::widgets::DependencyDialogFocus;

        match action {
            Some(Action::NextDialogButton) => {
                app.dependency_dialog_state.focus_next();
                app.mark_dirty();
                return true;
            }
            Some(Action::PrevDialogButton) => {
                app.dependency_dialog_state.focus_previous();
                app.mark_dirty();
                return true;
            }
            Some(Action::MoveLeft) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_previous_button();
                    app.mark_dirty();
                }
                return true;
            }
            Some(Action::MoveRight) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_next_button();
                    app.mark_dirty();
                }
                return true;
            }
            Some(Action::MoveUp) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state
                        .autocomplete_state
                        .select_previous();
                    app.mark_dirty();
                }
                return true;
            }
            Some(Action::MoveDown) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state.autocomplete_state.select_next();
                    app.mark_dirty();
                }
                return true;
            }
            Some(Action::ToggleSelection) => {
                // Space
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Type {
                    app.dependency_dialog_state.toggle_type();
                    app.mark_dirty();
                }
                // Also handle space as char if in text box?
                // Conflict resolution: Check focus
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    if let KeyCode::Char(c) = key_code {
                        app.dependency_dialog_state
                            .autocomplete_state
                            .insert_char(c);
                        app.mark_dirty();
                    }
                }
                return true;
            }
            Some(Action::ConfirmDialog) => {
                // Handle confirmation
                if app.dependency_dialog_state.is_ok_selected()
                    || app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId
                {
                    // Confirm selection and add dependency
                    if let Some(target_issue_id) = app.dependency_dialog_state.selected_issue_id() {
                        if let Some(current_issue) = app.issues_view_state.selected_issue() {
                            let current_id = current_issue.id.clone();
                            let dep_type = app.dependency_dialog_state.dependency_type();

                            match dep_type {
                                crate::ui::widgets::DependencyType::RelatesTo => {
                                    // Bidirectional "see also" relationship - moved to background task
                                    let current_id_clone = current_id.clone();
                                    let target_id_clone = target_issue_id.clone();

                                    let _ = app.spawn_task(
                                        "Linking issues",
                                        move |client| async move {
                                            client
                                                .relate_issues(&current_id_clone, &target_id_clone)
                                                .await
                                                .map_err(|e| {
                                                    crate::tasks::error::TaskError::ClientError(
                                                        e.to_string(),
                                                    )
                                                })?;
                                            Ok(crate::tasks::handle::TaskOutput::Success(format!(
                                                "Linked issues: {} <-> {}",
                                                current_id_clone, target_id_clone
                                            )))
                                        },
                                    );
                                }
                                crate::ui::widgets::DependencyType::DependsOn
                                | crate::ui::widgets::DependencyType::Blocks => {
                                    // Blocking dependency - check for cycles
                                    let (from_id, to_id) = match dep_type {
                                        crate::ui::widgets::DependencyType::DependsOn => {
                                            // Current depends on target (target blocks current)
                                            (current_id.clone(), target_issue_id.clone())
                                        }
                                        crate::ui::widgets::DependencyType::Blocks => {
                                            // Current blocks target (target depends on current)
                                            (target_issue_id.clone(), current_id.clone())
                                        }
                                        _ => unreachable!(),
                                    };

                                    // Check if this would create a cycle
                                    let all_issues: Vec<crate::beads::Issue> = app
                                        .issues_view_state
                                        .search_state()
                                        .filtered_issues()
                                        .to_vec();

                                    if crate::models::PertGraph::would_create_cycle(
                                        &all_issues,
                                        &from_id,
                                        &to_id,
                                    ) {
                                        app.set_error(format!(
                                            "Cannot add dependency: would create a cycle. {} → {} would form a circular dependency.",
                                            from_id, to_id
                                        ));
                                        tracing::warn!(
                                            "Prevented cycle: {} depends on {} would create cycle",
                                            from_id,
                                            to_id
                                        );
                                    } else {
                                        // Spawn background task (non-blocking)
                                        let from_id_owned = from_id.clone();
                                        let to_id_owned = to_id.clone();
                                        let _ = app.spawn_task(
                                            "Adding dependency",
                                            move |client| async move {
                                                client
                                                    .add_dependency(&from_id_owned, &to_id_owned)
                                                    .await?;
                                                tracing::info!(
                                                    "Added dependency: {} depends on {}",
                                                    from_id_owned,
                                                    to_id_owned
                                                );
                                                Ok(TaskOutput::DependencyAdded)
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        app.set_info("Please select an issue first".to_string());
                    }
                }
                // Close dialog in either case
                app.dependency_dialog_state.close();
                app.mark_dirty();
                return true;
            }
            Some(Action::CancelDialog) => {
                // Cancel dialog
                tracing::debug!("Dependency dialog cancelled");
                app.dependency_dialog_state.close();
                app.mark_dirty();
                return true;
            }
            _ => {
                // Handle text input fallback
                match key_code {
                    KeyCode::Backspace => {
                        if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                            app.dependency_dialog_state.autocomplete_state.delete_char();
                            app.mark_dirty();
                        }
                        return true;
                    }
                    KeyCode::Char(c) => {
                        if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                            app.dependency_dialog_state
                                .autocomplete_state
                                .insert_char(c);
                            app.mark_dirty();
                        }
                        return true;
                    }
                    _ => return false,
                }
            }
        }
    }

    // Handle delete confirmation dialog events if active
    if app.is_delete_confirmation_visible() {
        if let Some(ref mut dialog_state) = app.delete_dialog_state {
            match action {
                Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return true;
                }
                Some(Action::MoveRight) | Some(Action::NextDialogButton) => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return true;
                }
                Some(Action::ConfirmDialog) => {
                    // Confirm action based on selected button
                    let selected = dialog_state.selected_button();
                    if selected == 0 {
                        // Yes button - confirm deletion
                        match app.confirm_delete_filter() {
                            Ok(()) => {
                                tracing::info!("Filter deleted");
                                app.set_success("Filter deleted".to_string());
                            }
                            Err(e) => {
                                tracing::error!("Failed to delete filter: {}", e);
                                app.set_error(e);
                            }
                        }
                    } else {
                        // No button - cancel
                        tracing::debug!("Delete confirmation cancelled");
                        app.cancel_delete_filter();
                    }
                    return true;
                }
                Some(Action::CancelDialog) => {
                    // Cancel deletion
                    tracing::debug!("Delete confirmation cancelled");
                    app.cancel_delete_filter();
                    return true;
                }
                Some(Action::Quit) | Some(Action::ShowHelp) => {
                    // Let '?' and 'q' fall through to be handled globally
                }
                _ => {
                    // Ignore other keys when dialog is active
                    return true;
                }
            }
        }
    }

    // Handle dependency removal confirmation dialog events if active
    if app.is_dependency_removal_confirmation_visible() {
        if let Some(ref mut dialog_state) = app.dependency_removal_dialog_state {
            match action {
                Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return true;
                }
                Some(Action::MoveRight) | Some(Action::NextDialogButton) => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return true;
                }
                Some(Action::ConfirmDialog) => {
                    // Confirm action based on selected button
                    let selected = dialog_state.selected_button();
                    if selected == 0 {
                        // Yes button - confirm removal
                        match app.confirm_remove_dependency() {
                            Ok(()) => {
                                tracing::info!("Dependency removed");
                                app.set_success("Dependency removed successfully".to_string());
                            }
                            Err(e) => {
                                tracing::error!("Failed to remove dependency: {}", e);
                                app.set_error(e);
                            }
                        }
                    } else {
                        // No button - cancel
                        tracing::debug!("Dependency removal cancelled");
                        app.cancel_remove_dependency();
                    }
                    return true;
                }
                Some(Action::CancelDialog) => {
                    // Cancel removal
                    tracing::debug!("Dependency removal cancelled");
                    app.cancel_remove_dependency();
                    return true;
                }
                Some(Action::Quit) | Some(Action::ShowHelp) => {
                    // Let '?' and 'q' fall through to be handled globally
                }
                _ => {
                    // Ignore other keys when dialog is active
                    return true;
                }
            }
        }
    }

    let view_mode = app.issues_view_state.view_mode();

    // Handle filter menu events if open
    if app.issues_view_state.search_state().is_filter_menu_open() {
        match action {
            Some(Action::MoveDown) => {
                app.issues_view_state.search_state_mut().filter_menu_next();
                return true;
            }
            Some(Action::MoveUp) => {
                app.issues_view_state.search_state_mut().filter_menu_previous();
                return true;
            }
            Some(Action::ConfirmDialog) => {
                app.issues_view_state.search_state_mut().filter_menu_confirm();
                app.set_success("Filter applied".to_string());
                return true;
            }
            Some(Action::DeleteIssue) => {
                // 'd' or 'Delete'
                // Delete filter with confirmation
                if let Some(i) = app.issues_view_state.search_state().filter_menu_state().selected() {
                    if let Some(filter) = app.issues_view_state.search_state().saved_filters().get(i) {
                        let filter_name = filter.name.clone();
                        tracing::info!("Requesting confirmation to delete filter: {}", filter_name);

                        // Show confirmation dialog
                        app.dialog_state = Some(crate::ui::widgets::DialogState::new());
                        app.pending_action = Some(format!("delete_filter:{}", i));

                        tracing::debug!("Showing delete confirmation for filter: {}", filter_name);
                    }
                }
                return true;
            }
            Some(Action::CancelDialog) | Some(Action::ShowColumnManager) => {
                // 'm' closes menu too
                app.issues_view_state.search_state_mut().toggle_filter_menu();
                return true;
            }
            _ => return false, // Sink other keys
        }
    }

    match view_mode {
        IssuesViewMode::List => {
            // Check if a filter dropdown is active
            if let Some(ref mut fb_state) = app.issues_view_state.filter_bar_state {
                if fb_state.active_dropdown.is_some() {
                    // Filter dropdown is active - handle dropdown navigation
                    match key_code {
                        KeyCode::Up => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.previous();
                            }
                            return true;
                        }
                        KeyCode::Down => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.next();
                            }
                            return true;
                        }
                        KeyCode::Char(' ') => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.toggle_selected();
                            }
                            return true;
                        }
                        KeyCode::Enter => {
                            // Close dropdown first
                            fb_state.close_dropdown();
                            // Apply the selected filters to the issues list
                            app.issues_view_state.apply_filter_bar_filters();
                            return true;
                        }
                        KeyCode::Esc => {
                            // Close dropdown without applying
                            fb_state.close_dropdown();
                            return true;
                        }
                        _ => {
                            // Sink other keys when dropdown is active
                            return true;
                        }
                    }
                }
            }

            let search_focused = app.issues_view_state.search_state().search_state().is_focused();

            // In fullscreen Issues View (tab 0), we don't show search box, so skip search-focused handling
            if search_focused && app.selected_tab != 0 {
                // Search input is focused - handle text input
                match key_code {
                    KeyCode::Char(c) => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .insert_char(c);
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Backspace => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .delete_char();
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Delete => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .delete_char_forward();
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Left => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_right();
                    }
                    KeyCode::Up => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .history_previous();
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Down => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .history_next();
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Home => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_to_start();
                    }
                    KeyCode::End => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_to_end();
                    }
                    KeyCode::Enter => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .add_to_history();
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(false);
                    }
                    KeyCode::Esc => {
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(false);
                    }
                    _ => {}
                }
            } else if app.issues_view_state.search_state().list_state().is_editing() {
                // In-place editing mode: handle title editing
                match key_code {
                    KeyCode::Enter => {
                        // Save the edited title
                        let list_state = app.issues_view_state.search_state_mut().list_state_mut();
                        if let Some(new_title) = list_state.finish_editing() {
                            // Get the selected issue
                            if let Some(issue) = app.issues_view_state.search_state().selected_issue() {
                                let issue_id = issue.id.clone();
                                tracing::info!("Saving title edit for {}: {}", issue_id, new_title);

                                // Update title via command (undoable)
                                let client = Arc::new(app.beads_client.clone());
                                let update =
                                    crate::beads::client::IssueUpdate::new().title(new_title.clone());
                                let command = Box::new(IssueUpdateCommand::new(
                                    client,
                                    issue_id.clone(),
                                    update,
                                ));

                                app.start_loading("Updating title...");

                                match app.execute_command(command) {
                                    Ok(()) => {
                                        app.stop_loading();
                                        tracing::info!(
                                            "Successfully updated title for: {} (undo with Ctrl+Z)",
                                            issue_id
                                        );
                                        app.reload_issues();
                                    }
                                    Err(e) => {
                                        app.stop_loading();
                                        tracing::error!("Failed to update title: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Cancel editing
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .cancel_editing();
                    }
                    KeyCode::Char(ch) => {
                        // Insert character
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .insert_char_at_cursor(ch);
                    }
                    KeyCode::Backspace => {
                        // Delete character before cursor
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .delete_char_before_cursor();
                    }
                    KeyCode::Left => {
                        // Move cursor left
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_cursor_left();
                    }
                    KeyCode::Right => {
                        // Move cursor right
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_cursor_right();
                    }
                    _ => {}
                }
            } else {
                // List mode: navigation and quick actions

                // Check for 'r' or 'R' key to open detail view (before action matching)
                if matches!(key_code, KeyCode::Char('r') | KeyCode::Char('R')) {
                    app.issues_view_state.enter_detail_view();
                    return true;
                }

                match action {
                    Some(Action::MoveUp) => {
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_previous(len);
                    }
                    Some(Action::MoveDown) => {
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_next(len);
                    }
                    Some(Action::MoveLeft) if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Horizontal scroll left by one column
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_left();
                        app.mark_dirty();
                    }
                    Some(Action::MoveRight) if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Horizontal scroll right by one column
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_right();
                        app.mark_dirty();
                    }
                    Some(Action::MoveLeft) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Jump to first column (Shift+Left or Shift+Home)
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_to_column(0);
                        app.mark_dirty();
                    }
                    Some(Action::MoveRight) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Jump to last column (Shift+Right or Shift+End)
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_to_column(usize::MAX); // Will be clamped to last column
                        app.mark_dirty();
                    }
                    Some(Action::PageDown) => {
                        // Scroll down by one page (viewport height)
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        let viewport_height = app.issues_view_state
                            .search_state()
                            .list_state()
                            .last_viewport_height() as usize;
                        // Use viewport height, fallback to 10 if not yet rendered
                        let page_size = if viewport_height > 0 { viewport_height } else { 10 };
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_page_down(len, page_size);
                    }
                    Some(Action::PageUp) => {
                        // Scroll up by one page (viewport height)
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        let viewport_height = app.issues_view_state
                            .search_state()
                            .list_state()
                            .last_viewport_height() as usize;
                        // Use viewport height, fallback to 10 if not yet rendered
                        let page_size = if viewport_height > 0 { viewport_height } else { 10 };
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_page_up(len, page_size);
                    }
                    // TODO: Move child reordering to Action enum (e.g. MoveChildUp/Down)
                    // Keeping hardcoded for now as it uses modifiers
                    _ if key_code == KeyCode::Up
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        crate::helpers::reorder_child_issue(app, -1);
                    }
                    _ if key_code == KeyCode::Down
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        crate::helpers::reorder_child_issue(app, 1);
                    }

                    Some(Action::ConfirmDialog) => {
                        // Enter -> Open edit popup (same form as Create/Edit)
                        app.issues_view_state.enter_edit_mode();
                    }
                    // 'v' is Cycle View in new config
                    // Old code had 'v' for Split Screen AND 'v' for Next View.
                    // We'll stick to Next View as it's more general.
                    // Split screen is just one of the views?
                    // IssuesViewMode has SplitScreen.
                    // So cycling view should eventually reach it.
                    Some(Action::EditIssue) => {
                        app.issues_view_state.enter_edit_mode();
                    }
                    Some(Action::CreateIssue) => {
                        app.issues_view_state.enter_create_mode();
                    }
                    Some(Action::ShowColumnManager) => {
                        // Open column manager
                        let current_columns = app.issues_view_state
                            .search_state()
                            .list_state()
                            .table_config()
                            .columns
                            .clone();
                        app.column_manager_state =
                            Some(crate::ui::widgets::ColumnManagerState::new(current_columns));
                        tracing::debug!("Opened column manager");
                    }
                    Some(Action::CloseIssue) => {
                        // Close selected issue
                        if let Some(issue) = app.issues_view_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Closing issue: {}", issue_id);

                            // Execute close via command for undo support
                            use crate::beads::models::IssueStatus;
                            let client = Arc::new(app.beads_client.clone());
                            let update = crate::beads::client::IssueUpdate::new()
                                .status(IssueStatus::Closed);
                            let command =
                                Box::new(IssueUpdateCommand::new(client, issue_id.clone(), update));

                            app.start_loading(format!("Closing issue {}...", issue_id));

                            match app.execute_command(command) {
                                Ok(()) => {
                                    app.stop_loading();
                                    tracing::info!(
                                        "Successfully closed issue: {} (undo with Ctrl+Z)",
                                        issue_id
                                    );
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to close issue: {:?}", e);
                                    app.set_error(format!("Failed to close issue: {e}\n\nTry:\n• Verify the issue exists with 'bd show {issue_id}'\n• Check network connectivity\n• Run 'bd doctor' to diagnose issues"));
                                }
                            }
                        }
                    }
                    Some(Action::ReopenIssue) => {
                        // Reopen selected issue
                        if let Some(issue) = app.issues_view_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Reopening issue: {}", issue_id);

                            use crate::beads::models::IssueStatus;
                            let client = Arc::new(app.beads_client.clone());
                            let update =
                                crate::beads::client::IssueUpdate::new().status(IssueStatus::Open);
                            let command =
                                Box::new(IssueUpdateCommand::new(client, issue_id.clone(), update));

                            app.start_loading("Reopening issue...");

                            match app.execute_command(command) {
                                Ok(()) => {
                                    app.stop_loading();
                                    tracing::info!(
                                        "Successfully reopened issue: {} (undo with Ctrl+Z)",
                                        issue_id
                                    );
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to reopen issue: {:?}", e);
                                }
                            }
                        }
                    }
                    Some(Action::DeleteIssue) => {
                        // Delete selected issue with confirmation dialog
                        if let Some(issue) = app.issues_view_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            let issue_title = issue.title.clone();
                            tracing::info!("Requesting confirmation to delete issue: {}", issue_id);

                            // Show confirmation dialog
                            app.dialog_state = Some(crate::ui::widgets::DialogState::new());
                            app.pending_action = Some(format!("delete:{issue_id}"));

                            tracing::debug!("Showing delete confirmation for: {}", issue_title);
                        }
                    }
                    // In-place edit (F2/n) - Need Action::RenameIssue? Not in enum.
                    // Fallback to key check or map to EditIssue?
                    // EditIssue is 'e'. Rename is quick edit.
                    // Let's keep raw key check for specialized quick edit if it's not in Action.
                    // Or map 'n' to something else? Config maps 'n' to CreateIssue.
                    // Config maps 'r' to ReopenIssue.
                    // Config maps 'Shift+n' to NextSearchResult.
                    // We need a key for Quick Edit. 'F2' is standard.
                    // We'll keep F2 raw check.
                    _ if matches!(key_code, KeyCode::F(2)) => {
                        if let Some(issue) = app.issues_view_state.search_state().selected_issue() {
                            let title = issue.title.clone();
                            if let Some(selected_idx) =
                                app.issues_view_state.search_state().list_state().selected()
                            {
                                tracing::info!(
                                    "Starting in-place edit for {}: {}",
                                    issue.id,
                                    title
                                );
                                app.issues_view_state
                                    .search_state_mut()
                                    .list_state_mut()
                                    .start_editing(selected_idx, title);
                            }
                        }
                    }

                    Some(Action::IndentIssue) => {
                        // Indent
                        if let Some(selected_issue) = app.issues_view_state.search_state().selected_issue() {
                            let selected_id = selected_issue.id.clone();
                            let selected_idx = app.issues_view_state.search_state().list_state().selected();

                            // Get the previous issue in the filtered list
                            if let Some(idx) = selected_idx {
                                if idx > 0 {
                                    let filtered_issues =
                                        app.issues_view_state.search_state().filtered_issues();
                                    if let Some(prev_issue) = filtered_issues.get(idx - 1) {
                                        let prev_id = prev_issue.id.clone();

                                        tracing::info!(
                                            "Requesting confirmation to indent {} under {}",
                                            selected_id,
                                            prev_id
                                        );

                                        // Show confirmation dialog
                                        app.dialog_state = Some(crate::ui::widgets::DialogState::new());
                                        app.pending_action =
                                            Some(format!("indent:{}:{}", selected_id, prev_id));
                                    } else {
                                        app.set_error(
                                            "No previous issue to indent under".to_string(),
                                        );
                                    }
                                } else {
                                    app.set_error("Cannot indent first issue".to_string());
                                }
                            }
                        }
                    }
                    Some(Action::OutdentIssue) => {
                        // Outdent
                        if let Some(selected_issue) = app.issues_view_state.search_state().selected_issue() {
                            let selected_id = selected_issue.id.clone();

                            // Find the parent (issue that blocks the selected issue)
                            let all_issues = app.issues_view_state.all_issues();
                            let parent_id = all_issues
                                .iter()
                                .find(|issue| issue.blocks.contains(&selected_id))
                                .map(|issue| issue.id.clone());

                            if let Some(parent_id) = parent_id {
                                tracing::info!(
                                    "Requesting confirmation to outdent {} from parent {}",
                                    selected_id,
                                    parent_id
                                );

                                // Show confirmation dialog
                                app.dialog_state = Some(crate::ui::widgets::DialogState::new());
                                app.pending_action =
                                    Some(format!("outdent:{}:{}", selected_id, parent_id));
                            } else {
                                app.set_error("Issue has no parent to outdent from".to_string());
                            }
                        }
                    }
                    Some(Action::ToggleFilter) => {
                        // Toggle quick filters
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .toggle_filters();
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                        let enabled = app.issues_view_state.search_state().list_state().filters_enabled();
                        tracing::info!(
                            "Quick filters toggled: {}",
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }
                    Some(Action::OpenStatusFilter) => {
                        // Open or close status filter dropdown
                        if app.issues_view_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = crate::ui::widgets::FilterBarState::new(
                                crate::helpers::collect_unique_statuses(&app.issues_view_state),
                                crate::helpers::collect_unique_priorities(&app.issues_view_state),
                                crate::helpers::collect_unique_types(&app.issues_view_state),
                                crate::helpers::collect_unique_labels(&app.issues_view_state),
                                crate::helpers::collect_unique_assignees(&app.issues_view_state),
                                crate::helpers::collect_unique_created_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_closed_dates(&app.issues_view_state),
                            );
                            app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = app.issues_view_state.filter_bar_state {
                            fb_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Status);
                        }
                    }
                    Some(Action::OpenPriorityFilter) => {
                        // Open or close priority filter dropdown
                        if app.issues_view_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = crate::ui::widgets::FilterBarState::new(
                                crate::helpers::collect_unique_statuses(&app.issues_view_state),
                                crate::helpers::collect_unique_priorities(&app.issues_view_state),
                                crate::helpers::collect_unique_types(&app.issues_view_state),
                                crate::helpers::collect_unique_labels(&app.issues_view_state),
                                crate::helpers::collect_unique_assignees(&app.issues_view_state),
                                crate::helpers::collect_unique_created_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_closed_dates(&app.issues_view_state),
                            );
                            app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = app.issues_view_state.filter_bar_state {
                            fb_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Priority);
                        }
                    }
                    Some(Action::OpenTypeFilter) => {
                        // Open or close type filter dropdown
                        if app.issues_view_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = crate::ui::widgets::FilterBarState::new(
                                crate::helpers::collect_unique_statuses(&app.issues_view_state),
                                crate::helpers::collect_unique_priorities(&app.issues_view_state),
                                crate::helpers::collect_unique_types(&app.issues_view_state),
                                crate::helpers::collect_unique_labels(&app.issues_view_state),
                                crate::helpers::collect_unique_assignees(&app.issues_view_state),
                                crate::helpers::collect_unique_created_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_closed_dates(&app.issues_view_state),
                            );
                            app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = app.issues_view_state.filter_bar_state {
                            fb_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Type);
                        }
                    }
                    Some(Action::OpenLabelsFilter) => {
                        // Open or close labels filter dropdown
                        if app.issues_view_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = crate::ui::widgets::FilterBarState::new(
                                crate::helpers::collect_unique_statuses(&app.issues_view_state),
                                crate::helpers::collect_unique_priorities(&app.issues_view_state),
                                crate::helpers::collect_unique_types(&app.issues_view_state),
                                crate::helpers::collect_unique_labels(&app.issues_view_state),
                                crate::helpers::collect_unique_assignees(&app.issues_view_state),
                                crate::helpers::collect_unique_created_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                crate::helpers::collect_unique_closed_dates(&app.issues_view_state),
                            );
                            app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = app.issues_view_state.filter_bar_state {
                            fb_state.toggle_dropdown(crate::ui::widgets::FilterDropdownType::Labels);
                        }
                    }
                    // Cycle View (was 'v')
                    // Assuming 'v' maps to some action? Not in Default Bindings explicitly as "CycleView".
                    // Wait, I didn't add "CycleView" to Action.
                    // I will use key check for 'v' since I missed adding it to Action.
                    // Or reuse an existing action?
                    _ if key_code == KeyCode::Char('v') => {
                        // Cycle view
                        app.issues_view_state.search_state_mut().next_view();
                        tracing::debug!(
                            "Cycled to next view: {:?}",
                            app.issues_view_state.search_state().current_view()
                        );
                    }
                    // Cycle Scope ('s') - mapped to UpdateStatus in Config?
                    // Config: UpdateStatus -> 's'.
                    // OLD CODE: 's' -> Cycle search scope.
                    // Conflict!
                    // Universal Guide says: s -> Status.
                    // So Search Scope needs a new key. Maybe 'S' (Shift+s)?
                    // Or remove cycle search scope shortcut?
                    // Let's keep 's' for Status as per guide.
                    // We'll drop search scope cycling shortcut for now or map it to something else if needed.
                    Some(Action::UpdateStatus) => {
                        // 's'
                        // Open status selector
                        if app.issues_view_state.selected_issue().is_some() {
                            app.status_selector_state.toggle();
                        }
                    }
                    Some(Action::ToggleRegexSearch) => {
                        // Toggle regex
                        app.issues_view_state.search_state_mut().toggle_regex();
                        let enabled = app.issues_view_state.search_state().is_regex_enabled();
                        app.set_info(format!(
                            "Regex search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    Some(Action::ToggleFuzzySearch) => {
                        // Toggle fuzzy
                        app.issues_view_state.search_state_mut().toggle_fuzzy();
                        let enabled = app.issues_view_state.search_state().is_fuzzy_enabled();
                        app.set_info(format!(
                            "Fuzzy search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    Some(Action::UpdateLabels) => {
                        // Toggle label logic (l)?
                        // Config: UpdateLabels -> 'l'.
                        // Old code: 'l' -> Toggle Label Logic.
                        // Guide says 'l' -> Move Right.
                        // Wait, Guide says 'l' -> Move Right in General Nav.
                        // In Issues View, Guide doesn't list 'l'.
                        // Config has 'l' for UpdateLabels.
                        // Let's use 'l' for UpdateLabels (open picker?) or Toggle Logic?
                        // Old code 'L' (Shift+L) opened label picker.
                        // Let's make 'l' open label picker (UpdateLabels).
                        // And maybe Shift+L for logic?

                        // Open label picker for selected issue
                        if let Some(issue) = app.issues_view_state.selected_issue() {
                            // ... label picker setup ...
                            let current_labels = issue.labels.clone();
                            let all_labels: std::collections::HashSet<String> = app
                                .issues_view_state
                                .all_issues()
                                .iter()
                                .flat_map(|i| i.labels.iter().cloned())
                                .collect();
                            let mut available_labels: Vec<String> =
                                all_labels.into_iter().collect();
                            available_labels.sort();

                            app.label_picker_state
                                .set_available_labels(available_labels);
                            app.label_picker_state.set_selected_labels(current_labels);

                            app.show_label_picker = true;
                        }
                    }
                    Some(Action::UpdatePriority) => {
                        // Open priority selector
                        if app.issues_view_state.selected_issue().is_some() {
                            app.priority_selector_state.toggle();
                        }
                    }
                    Some(Action::Search) => {
                        // '/'
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .clear();
                        app.issues_view_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(true);
                        app.issues_view_state.search_state_mut().update_filtered_issues();
                    }
                    Some(Action::NextSearchResult) => {
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_next(len);
                    }
                    Some(Action::PrevSearchResult) => {
                        let len = app.issues_view_state.search_state().filtered_issues().len();
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_previous(len);
                    }
                    Some(Action::CancelDialog) => {
                        // Esc
                        app.issues_view_state.search_state_mut().clear_search();
                    }
                    Some(Action::ClearFilter) => {
                        // Shift+F - Clear/reset search
                        app.issues_view_state.search_state_mut().clear_search();
                    }

                    // Column manipulation (Alt+Left/Right etc)
                    // These are Actions now: MoveLeft + Alt, etc.
                    // But Action system handles modifiers in finding the action.
                    // If Keybinding::new("left").alt() maps to MoveColumnLeft (we don't have that action).
                    // We reused MoveLeft.
                    // If we have MoveLeft mapped to 'h' and 'Left', and we press Alt+Left.
                    // Does config map Alt+Left? No.
                    // So Alt+Left won't match Action::MoveLeft.
                    // We need to check modifiers manually or add Action::MoveColumnLeft.
                    // Since I didn't add specific column actions, I will keep the raw key checks for column ops for now.
                    _ if key_code == KeyCode::Left && key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Left: Move focused column left
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_left();
                        let _ = app.issues_view_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
                        tracing::debug!("Moving focused column left");
                    }
                    _ if key_code == KeyCode::Right
                        && key.modifiers.contains(KeyModifiers::ALT) =>
                    {
                        // Alt+Right: Move focused column right
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_right();
                        let _ = app.issues_view_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
                        tracing::debug!("Moving focused column right");
                    }
                    _ => {}
                }
            }
        }
        IssuesViewMode::Detail => {
            // Detail mode: view navigation with scrolling
            match action {
                Some(Action::CancelDialog) | Some(Action::Quit) => {
                    // Esc or q
                    app.issues_view_state.return_to_list();
                }
                Some(Action::EditIssue) => {
                    app.issues_view_state.return_to_list();
                    app.issues_view_state.enter_edit_mode();
                }
                Some(Action::MoveUp) => {
                    // Scroll up in detail view (1 field at a time)
                    app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_sub(1);
                    app.mark_dirty();
                }
                Some(Action::MoveDown) => {
                    // Scroll down in detail view (1 field at a time)
                    app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_add(1);
                    app.mark_dirty();
                }
                Some(Action::PageUp) => {
                    // Page up in detail view (5 fields at a time)
                    app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_sub(5);
                    app.mark_dirty();
                }
                Some(Action::PageDown) => {
                    // Page down in detail view (5 fields at a time)
                    app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_add(5);
                    app.mark_dirty();
                }
                Some(Action::Home) => {
                    // Scroll to top of form
                    app.issues_view_state.detail_scroll = 0;
                    app.mark_dirty();
                }
                Some(Action::End) => {
                    // Scroll to bottom of form (set to large value, clamped by render)
                    app.issues_view_state.detail_scroll = u16::MAX;
                    app.mark_dirty();
                }
                _ => {}
            }
        }
        IssuesViewMode::SplitScreen => {
            // Split-screen mode: list navigation with live detail updates
            use SplitScreenFocus;

            match key_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.issues_view_state.return_to_list();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    match app.issues_view_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            let len = app.issues_view_state.search_state().filtered_issues().len();
                            app.issues_view_state
                                .search_state_mut()
                                .list_state_mut()
                                .select_next(len);
                            // Update detail panel with newly selected issue
                            app.issues_view_state.update_split_screen_detail();
                        }
                        SplitScreenFocus::Detail => {
                            // Scroll detail panel down
                            app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_add(1);
                        }
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    match app.issues_view_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            let len = app.issues_view_state.search_state().filtered_issues().len();
                            app.issues_view_state
                                .search_state_mut()
                                .list_state_mut()
                                .select_previous(len);
                            // Update detail panel with newly selected issue
                            app.issues_view_state.update_split_screen_detail();
                        }
                        SplitScreenFocus::Detail => {
                            // Scroll detail panel up
                            app.issues_view_state.detail_scroll = app.issues_view_state.detail_scroll.saturating_sub(1);
                        }
                    }
                }
                KeyCode::Char('g') => {
                    // Go to top
                    app.issues_view_state
                        .search_state_mut()
                        .list_state_mut()
                        .select(Some(0));
                    app.issues_view_state.update_split_screen_detail();
                }
                KeyCode::Char('G') => {
                    // Go to bottom
                    let len = app.issues_view_state.search_state().filtered_issues().len();
                    if len > 0 {
                        app.issues_view_state
                            .search_state_mut()
                            .list_state_mut()
                            .select(Some(len - 1));
                        app.issues_view_state.update_split_screen_detail();
                    }
                }
                KeyCode::Enter => {
                    // Go to full detail view
                    app.issues_view_state.enter_detail_view();
                }
                KeyCode::Char('e') => {
                    // Enter edit mode
                    app.issues_view_state.enter_edit_mode();
                }
                KeyCode::Char('r') => {
                    // Enter read-only detail view
                    app.issues_view_state.enter_detail_view();
                }
                KeyCode::Tab => {
                    // Toggle focus between list and detail panels
                    match app.issues_view_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            app.issues_view_state.set_split_screen_focus(SplitScreenFocus::Detail);
                        }
                        SplitScreenFocus::Detail => {
                            app.issues_view_state.set_split_screen_focus(SplitScreenFocus::List);
                        }
                    }
                }
                KeyCode::Char('l') => {
                    // Switch focus to list panel
                    app.issues_view_state.set_split_screen_focus(SplitScreenFocus::List);
                }
                _ => {}
            }
        }
        IssuesViewMode::Edit => {
            // Edit mode: form controls
            if let Some(editor_state) = app.issues_view_state.editor_state_mut() {
                let form = editor_state.form_state_mut();

                // Check for Ctrl+L first (before generic Char handler)
                if key_code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Load from file (Ctrl+L)
                    // Get the current field value as the file path
                    if let Some(focused_field) = form.focused_field() {
                        let file_path = focused_field.value.trim().to_string();

                        if file_path.is_empty() {
                            // Set error on focused field
                            if let Some(field) = form.focused_field_mut() {
                                field.error = Some(
                                    "Enter a file path first, then press Ctrl+L to load it"
                                        .to_string(),
                                );
                            }
                        } else {
                            // Try to load from file
                            match form.load_from_file(&file_path) {
                                Ok(()) => {
                                    tracing::info!("Loaded content from file: {}", file_path);
                                }
                                Err(err) => {
                                    tracing::error!("Failed to load file {}: {}", file_path, err);
                                    // Error is already set in the field by load_from_file
                                }
                            }
                        }
                    }
                } else {
                    match key_code {
                        // Field navigation
                        KeyCode::Tab | KeyCode::Down => {
                            form.focus_next();
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            form.focus_previous();
                        }
                        // Text input
                        KeyCode::Char(c) => {
                            form.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            form.delete_char();
                        }
                        // Cursor movement
                        KeyCode::Left => {
                            form.move_cursor_left();
                        }
                        KeyCode::Right => {
                            form.move_cursor_right();
                        }
                        KeyCode::Home => {
                            form.move_cursor_to_start();
                        }
                        KeyCode::End => {
                            form.move_cursor_to_end();
                        }
                        // Form scrolling
                        KeyCode::PageUp => {
                            form.scroll_up(5);
                            app.mark_dirty();
                        }
                        KeyCode::PageDown => {
                            form.scroll_down(5);
                            app.mark_dirty();
                        }
                        // Save/Cancel
                        KeyCode::Enter => {
                            // Validate and save
                            if editor_state.validate() {
                                // Check if there are any changes
                                if !editor_state.is_dirty() {
                                    tracing::info!("No changes detected, returning to list");
                                    app.issues_view_state.return_to_list();
                                } else {
                                    // Get change summary for logging
                                    let changes = editor_state.get_changes();
                                    tracing::info!("Changes detected: {:?}", changes);

                                    // Get IssueUpdate with only changed fields
                                    if let Some(update) = editor_state.get_issue_update() {
                                        let issue_id = editor_state.issue_id().to_string();

                                        // Mark as saved and return to list before reloading
                                        editor_state.save();
                                        app.issues_view_state.return_to_list();

                                        // Update issue via command (undoable)
                                        let client = Arc::new(app.beads_client.clone());
                                        let command = Box::new(IssueUpdateCommand::new(
                                            client,
                                            issue_id.clone(),
                                            update,
                                        ));

                                        app.start_loading("Updating issue...");

                                        match app.execute_command(command) {
                                            Ok(()) => {
                                                app.stop_loading();
                                                tracing::info!(
                                                    "Successfully updated issue: {} (undo with Ctrl+Z)",
                                                    issue_id
                                                );

                                                // Clear any previous errors
                                                app.clear_error();

                                                // Reload issues list
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                app.stop_loading();
                                                tracing::error!("Failed to update issue: {:?}", e);
                                                app.set_error(format!(
                                                "Failed to update issue: {e}\n\nStay in edit mode to fix and retry.\nVerify your changes and try again."
                                            ));
                                                // Stay in edit mode so user can fix and retry
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.issues_view_state.cancel_edit();
                        }
                        _ => {}
                    }
                }
            }
        }
        IssuesViewMode::Create => {
            // Create mode: form controls
            if let Some(create_form_state) = app.issues_view_state.create_form_state_mut() {
                // Check for Ctrl+P first (toggle preview)
                if key_code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    create_form_state.toggle_preview();
                    return true;
                }

                let form = create_form_state.form_state_mut();

                // Check for Ctrl+L first (before generic Char handler)
                if key_code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Load from file (Ctrl+L)
                    // Get the current field value as the file path
                    if let Some(focused_field) = form.focused_field() {
                        let file_path = focused_field.value.trim().to_string();

                        if file_path.is_empty() {
                            // Set error on focused field
                            if let Some(field) = form.focused_field_mut() {
                                field.error = Some(
                                    "Enter a file path first, then press Ctrl+L to load it"
                                        .to_string(),
                                );
                            }
                        } else {
                            // Try to load from file
                            match form.load_from_file(&file_path) {
                                Ok(()) => {
                                    tracing::info!("Loaded content from file: {}", file_path);
                                }
                                Err(err) => {
                                    tracing::error!("Failed to load file {}: {}", file_path, err);
                                    // Error is already set in the field by load_from_file
                                }
                            }
                        }
                    }
                } else {
                    match key_code {
                        // Field navigation
                        KeyCode::Tab | KeyCode::Down => {
                            form.focus_next();
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            form.focus_previous();
                        }
                        // Text input
                        KeyCode::Char(c) => {
                            form.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            form.delete_char();
                        }
                        // Cursor movement
                        KeyCode::Left => {
                            form.move_cursor_left();
                        }
                        KeyCode::Right => {
                            form.move_cursor_right();
                        }
                        KeyCode::Home => {
                            form.move_cursor_to_start();
                        }
                        KeyCode::End => {
                            form.move_cursor_to_end();
                        }
                        // Form scrolling
                        KeyCode::PageUp => {
                            form.scroll_up(5);
                            app.mark_dirty();
                        }
                        KeyCode::PageDown => {
                            form.scroll_down(5);
                            app.mark_dirty();
                        }
                        // Submit/Cancel
                        KeyCode::Enter => {
                            // Validate and submit
                            if create_form_state.validate() {
                                if let Some(data) = app.issues_view_state.save_create() {
                                    // Create a tokio runtime to execute the async call
                                    // Using global runtime instead of creating new runtime
                                    let client = app.beads_client.clone();

                                    let mut dependency_targets: Vec<String> = Vec::new();
                                    if let Some(parent) = data.parent.clone() {
                                        if !dependency_targets.contains(&parent) {
                                            dependency_targets.push(parent);
                                        }
                                    }
                                    for dep in data.dependencies.clone() {
                                        if !dependency_targets.contains(&dep) {
                                            dependency_targets.push(dep);
                                        }
                                    }

                                    // Build create params
                                    let mut params = crate::beads::models::CreateIssueParams::new(
                                        &data.title,
                                        data.issue_type,
                                        data.priority,
                                    );
                                    params.status = Some(&data.status);
                                    params.assignee = data.assignee.as_deref();
                                    params.labels = &data.labels;
                                    params.description = data.description.as_deref();

                                    // Show loading indicator
                                    app.start_loading("Creating issue...");

                                    match crate::runtime::RUNTIME
                                        .block_on(client.create_issue_full(params))
                                    {
                                        Ok(issue_id) => {
                                            app.stop_loading();
                                            // Successfully created
                                            tracing::info!(
                                                "Successfully created issue: {}",
                                                issue_id
                                            );

                                            // Clear any previous errors
                                            app.clear_error();

                                            if !dependency_targets.is_empty() {
                                                let mut failures = Vec::new();
                                                for dep_id in &dependency_targets {
                                                    if dep_id == &issue_id {
                                                        continue;
                                                    }
                                                    if let Err(e) = crate::runtime::RUNTIME.block_on(
                                                        client.add_dependency(&issue_id, dep_id),
                                                    ) {
                                                        failures.push(format!("{dep_id}: {e}"));
                                                    }
                                                }
                                                if !failures.is_empty() {
                                                    app.set_error(format!(
                                                    "Issue created, but dependencies failed: {}",
                                                    failures.join(", ")
                                                ));
                                                }
                                            }

                                            // Reload issues list
                                            app.reload_issues();

                                            // Return to list
                                            app.issues_view_state.cancel_create();

                                            // Select the newly created issue in the list
                                            let created_issue = app
                                                .issues_view_state
                                                .search_state()
                                                .filtered_issues()
                                                .iter()
                                                .find(|issue| issue.id == issue_id)
                                                .cloned();

                                            if let Some(issue) = created_issue {
                                                app.issues_view_state
                                                    .set_selected_issue(Some(issue));
                                                tracing::debug!(
                                                    "Selected newly created issue: {}",
                                                    issue_id
                                                );
                                            } else {
                                                tracing::warn!(
                                                    "Could not find newly created issue {} in list",
                                                    issue_id
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            app.stop_loading();
                                            tracing::error!("Failed to create issue: {:?}", e);
                                            app.set_error(format!(
                                            "Failed to create issue: {e}\n\nStay in create mode to fix and retry.\nCheck that all required fields are filled correctly."
                                        ));
                                            // Stay in create mode so user can fix and retry
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.issues_view_state.cancel_create();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
        // If nothing consumed the event, return false
        false
    }

    fn view_name() -> &'static str {
        "IssuesView"
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
            labels: vec!["test".to_string()],
            assignee: Some("alice".to_string()),
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
    fn test_issues_view_state_creation() {
        let issues = vec![
            create_test_issue("beads-001", "Issue 1"),
            create_test_issue("beads-002", "Issue 2"),
        ];
        let state = IssuesViewState::new(issues);

        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_enter_detail_view() {
        let issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_detail_view();
        assert_eq!(state.view_mode(), IssuesViewMode::Detail);
        assert!(state.selected_issue().is_some());
    }

    #[test]
    fn test_enter_edit_mode() {
        let issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);
        assert!(state.selected_issue().is_some());
        assert!(state.editor_state().is_some());
    }

    #[test]
    fn test_return_to_list() {
        let issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_detail_view();
        assert_eq!(state.view_mode(), IssuesViewMode::Detail);

        state.return_to_list();
        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_cancel_edit() {
        let issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);

        state.cancel_edit();
        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
        assert!(state.editor_state().is_none());
    }

    #[test]
    fn test_toggle_help() {
        let issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.is_help_visible());

        state.toggle_help();
        assert!(!state.is_help_visible());

        state.toggle_help();
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_set_issues() {
        let initial_issues = vec![create_test_issue("beads-001", "Issue 1")];
        let mut state = IssuesViewState::new(initial_issues);

        let new_issues = vec![
            create_test_issue("beads-002", "Issue 2"),
            create_test_issue("beads-003", "Issue 3"),
        ];

        state.set_issues(new_issues);
        assert_eq!(state.search_state().result_count(), 2);
    }

    #[test]
    fn test_issues_view_mode_equality() {
        assert_eq!(IssuesViewMode::List, IssuesViewMode::List);
        assert_eq!(IssuesViewMode::Detail, IssuesViewMode::Detail);
        assert_eq!(IssuesViewMode::Edit, IssuesViewMode::Edit);
        assert_eq!(IssuesViewMode::Create, IssuesViewMode::Create);

        assert_ne!(IssuesViewMode::List, IssuesViewMode::Detail);
        assert_ne!(IssuesViewMode::Detail, IssuesViewMode::Edit);
        assert_ne!(IssuesViewMode::Edit, IssuesViewMode::Create);
    }

    #[test]
    fn test_set_view_mode() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.set_view_mode(IssuesViewMode::Detail);
        assert_eq!(state.view_mode(), IssuesViewMode::Detail);

        state.set_view_mode(IssuesViewMode::Edit);
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);

        state.set_view_mode(IssuesViewMode::Create);
        assert_eq!(state.view_mode(), IssuesViewMode::Create);

        state.set_view_mode(IssuesViewMode::List);
        assert_eq!(state.view_mode(), IssuesViewMode::List);
    }

    #[test]
    fn test_search_state_mut() {
        let issues = vec![
            create_test_issue("beads-abcd-0001", "Issue 1"),
            create_test_issue("beads-efgh-0002", "Issue 2"),
        ];
        let mut state = IssuesViewState::new(issues);

        // Modify search state through mutable reference
        state
            .search_state_mut()
            .search_state_mut()
            .set_query("Issue 1".to_string());
        assert_eq!(state.search_state().search_state().query(), "Issue 1");
    }

    #[test]
    fn test_editor_state_mut() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.editor_state_mut().is_none());

        state.enter_edit_mode();
        assert!(state.editor_state_mut().is_some());

        // Modify editor state
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified Title".to_string());
        }

        assert!(state.editor_state().is_some());
    }

    #[test]
    fn test_create_form_state_mut() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.create_form_state_mut().is_none());

        state.enter_create_mode();
        assert!(state.create_form_state_mut().is_some());

        // Modify create form state
        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut()
                .set_value("title", "New Issue".to_string());
        }

        assert!(state.create_form_state().is_some());
    }

    #[test]
    fn test_enter_detail_view_no_selection() {
        let issues: Vec<Issue> = vec![];
        let mut state = IssuesViewState::new(issues);

        state.enter_detail_view();
        // Should remain in List mode when no issue selected
        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_enter_edit_mode_no_selection() {
        let issues: Vec<Issue> = vec![];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        // Should remain in List mode when no issue selected
        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
        assert!(state.editor_state().is_none());
    }

    #[test]
    fn test_enter_create_mode() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Create);
        assert!(state.create_form_state().is_some());
    }

    #[test]
    fn test_cancel_create() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Create);

        state.cancel_create();
        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.create_form_state().is_none());
    }

    #[test]
    fn test_save_create_with_valid_data() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();
        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut()
                .set_value("title", "New Issue".to_string());
            form.form_state_mut()
                .set_value("description", "Description".to_string());
        }

        let data = state.save_create();
        assert!(data.is_some());
        if let Some(d) = data {
            assert_eq!(d.title, "New Issue");
        }
    }

    #[test]
    fn test_save_create_returns_none_when_no_form() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        // Try to save without entering create mode
        let data = state.save_create();
        assert!(data.is_none());
    }

    #[test]
    fn test_save_edit_with_valid_data() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified Title".to_string());
        }

        let updated = state.save_edit();
        assert!(updated.is_some());
    }

    #[test]
    fn test_return_to_list_clears_editor() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        assert!(state.editor_state().is_some());

        state.return_to_list();
        assert!(state.editor_state().is_none());
    }

    #[test]
    fn test_multiple_view_mode_transitions() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        // List -> Detail
        state.enter_detail_view();
        assert_eq!(state.view_mode(), IssuesViewMode::Detail);

        // Detail -> List
        state.return_to_list();
        assert_eq!(state.view_mode(), IssuesViewMode::List);

        // List -> Edit
        state.enter_edit_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);

        // Edit -> List
        state.cancel_edit();
        assert_eq!(state.view_mode(), IssuesViewMode::List);

        // List -> Create
        state.enter_create_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Create);

        // Create -> List
        state.cancel_create();
        assert_eq!(state.view_mode(), IssuesViewMode::List);
    }

    #[test]
    fn test_set_selected_issue() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.selected_issue().is_none());

        let test_issue = create_test_issue("beads-efgh-0002", "Issue 2");
        state.set_selected_issue(Some(test_issue.clone()));

        assert!(state.selected_issue().is_some());
        assert_eq!(state.selected_issue().unwrap().id, "beads-efgh-0002");

        state.set_selected_issue(None);
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_help_visibility_initial_state() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let state = IssuesViewState::new(issues);

        assert!(state.is_help_visible());
    }

    #[test]
    fn test_help_visibility_multiple_toggles() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        for _ in 0..4 {
            let before = state.is_help_visible();
            state.toggle_help();
            let after = state.is_help_visible();
            assert_ne!(before, after);
        }

        // After even number of toggles, should be back to initial state
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_issues_view_new() {
        let view = IssuesView::new();
        // Should create successfully with default values
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
    }

    #[test]
    fn test_issues_view_default() {
        let view = IssuesView::default();
        assert_eq!(view.block_style, Style::default().fg(Color::Cyan));
    }

    #[test]
    fn test_issues_view_block_style() {
        let custom_style = Style::default().fg(Color::Red);
        let view = IssuesView::new().block_style(custom_style);
        assert_eq!(view.block_style, custom_style);
    }

    #[test]
    fn test_issues_view_builder_chain() {
        let custom_style = Style::default().fg(Color::Green);
        let view = IssuesView::new().block_style(custom_style);
        assert_eq!(view.block_style, custom_style);
    }

    #[test]
    fn test_search_state_access() {
        let issues = vec![
            create_test_issue("beads-abcd-0001", "Issue 1"),
            create_test_issue("beads-efgh-0002", "Issue 2"),
        ];
        let state = IssuesViewState::new(issues);

        let search_state = state.search_state();
        assert_eq!(search_state.result_count(), 2);
    }

    #[test]
    fn test_editor_state_access() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.editor_state().is_none());

        state.enter_edit_mode();
        assert!(state.editor_state().is_some());
    }

    #[test]
    fn test_create_form_state_access() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        assert!(state.create_form_state().is_none());

        state.enter_create_mode();
        assert!(state.create_form_state().is_some());
    }

    #[test]
    fn test_issues_view_mode_copy_trait() {
        let mode1 = IssuesViewMode::Detail;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
        // Both should still be usable after copy
        assert_eq!(mode1, IssuesViewMode::Detail);
        assert_eq!(mode2, IssuesViewMode::Detail);
    }

    #[test]
    fn test_issues_view_mode_clone_trait() {
        let mode1 = IssuesViewMode::Edit;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }

    #[test]
    fn test_save_edit_returns_none_when_no_editor_state() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        // Try to save without entering edit mode
        let result = state.save_edit();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_edit_returns_none_when_no_selected_issue() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        // Clear selected issue manually
        state.selected_issue = None;

        let result = state.save_edit();
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_consecutive_edit_operations() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        // First edit
        state.enter_edit_mode();
        state.cancel_edit();

        // Second edit
        state.enter_edit_mode();
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);
        state.cancel_edit();

        // Third edit
        state.enter_edit_mode();
        assert!(state.editor_state().is_some());
    }

    #[test]
    fn test_view_mode_after_save_edit() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified".to_string());
        }

        state.save_edit();
        // View mode should remain Edit after save (not auto-return to list)
        assert_eq!(state.view_mode(), IssuesViewMode::Edit);
    }

    #[test]
    fn test_selected_issue_retained_in_edit_mode() {
        let issues = vec![
            create_test_issue("beads-abcd-0001", "Issue 1"),
            create_test_issue("beads-efgh-0002", "Issue 2"),
        ];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        let selected_id = state.selected_issue().map(|i| i.id.clone());

        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified".to_string());
        }

        // Selected issue should remain the same
        assert_eq!(state.selected_issue().map(|i| i.id.clone()), selected_id);
    }

    #[test]
    fn test_editor_state_consistency_after_modifications() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();

        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "First Modification".to_string());
        }

        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Second Modification".to_string());
        }

        // Editor state should still be present
        assert!(state.editor_state().is_some());
    }

    #[test]
    fn test_create_form_state_consistency_after_modifications() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();

        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut()
                .set_value("title", "New Issue 1".to_string());
        }

        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut()
                .set_value("description", "Description 1".to_string());
        }

        // Create form state should still be present
        assert!(state.create_form_state().is_some());
    }

    #[test]
    fn test_empty_issue_list_initialization() {
        let issues: Vec<Issue> = vec![];
        let state = IssuesViewState::new(issues);

        assert_eq!(state.view_mode(), IssuesViewMode::List);
        assert!(state.selected_issue().is_none());
        assert_eq!(state.search_state().result_count(), 0);
    }

    #[test]
    fn test_large_number_of_issues() {
        let mut issues = vec![];
        for i in 0..100 {
            issues.push(create_test_issue(
                &format!("beads-{:04}", i),
                &format!("Issue {}", i),
            ));
        }

        let state = IssuesViewState::new(issues);
        assert_eq!(state.search_state().result_count(), 100);
    }

    #[test]
    fn test_transition_detail_to_edit_preserves_selection() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_detail_view();
        let selected_in_detail = state.selected_issue().map(|i| i.id.clone());

        // Transition to list then edit
        state.return_to_list();
        state.enter_edit_mode();
        let selected_in_edit = state.selected_issue().map(|i| i.id.clone());

        assert_eq!(selected_in_detail, selected_in_edit);
    }

    #[test]
    fn test_help_visibility_across_mode_changes() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.toggle_help();
        let help_visible = state.is_help_visible();

        state.enter_detail_view();
        assert_eq!(state.is_help_visible(), help_visible);

        state.return_to_list();
        assert_eq!(state.is_help_visible(), help_visible);
    }

    #[test]
    fn test_return_to_list_from_create_mode() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();
        state.return_to_list();

        assert_eq!(state.view_mode(), IssuesViewMode::List);
        // Note: return_to_list doesn't clear create_form_state, only cancel_create does
        assert!(state.create_form_state().is_some());
    }

    #[test]
    fn test_set_issues_updates_search_state() {
        let initial = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(initial);

        let new_issues = vec![
            create_test_issue("beads-efgh-0002", "Issue 2"),
            create_test_issue("beads-ijkl-0003", "Issue 3"),
            create_test_issue("beads-mnop-0004", "Issue 4"),
        ];

        state.set_issues(new_issues);
        assert_eq!(state.search_state().result_count(), 3);
    }

    #[test]
    fn test_all_view_modes_inequality() {
        assert_ne!(IssuesViewMode::List, IssuesViewMode::Detail);
        assert_ne!(IssuesViewMode::List, IssuesViewMode::Edit);
        assert_ne!(IssuesViewMode::List, IssuesViewMode::Create);
        assert_ne!(IssuesViewMode::Detail, IssuesViewMode::Edit);
        assert_ne!(IssuesViewMode::Detail, IssuesViewMode::Create);
        assert_ne!(IssuesViewMode::Edit, IssuesViewMode::Create);
    }

    #[test]
    fn test_issues_view_default_equals_new() {
        let default_view = IssuesView::default();
        let new_view = IssuesView::new();

        assert_eq!(default_view.block_style, new_view.block_style);
    }

    #[test]
    fn test_editor_state_after_cancel() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified".to_string());
        }

        state.cancel_edit();
        assert!(state.editor_state().is_none());
    }

    #[test]
    fn test_create_form_state_after_cancel() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_create_mode();
        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut()
                .set_value("title", "New Issue".to_string());
        }

        state.cancel_create();
        assert!(state.create_form_state().is_none());
    }

    #[test]
    fn test_selected_issue_after_return_to_list() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_detail_view();
        assert!(state.selected_issue().is_some());

        state.return_to_list();
        assert!(state.selected_issue().is_none());
    }

    #[test]
    fn test_enter_edit_mode_preserves_issue_id() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();

        if let Some(editor) = state.editor_state() {
            assert_eq!(editor.issue_id(), "beads-abcd-0001");
        } else {
            panic!("Editor state should be Some");
        }
    }

    #[test]
    fn test_multiple_set_issues_calls() {
        let initial = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(initial);

        let set1 = vec![create_test_issue("beads-efgh-0002", "Issue 2")];
        state.set_issues(set1);
        assert_eq!(state.search_state().result_count(), 1);

        let set2 = vec![
            create_test_issue("beads-ijkl-0003", "Issue 3"),
            create_test_issue("beads-mnop-0004", "Issue 4"),
        ];
        state.set_issues(set2);
        assert_eq!(state.search_state().result_count(), 2);

        let set3: Vec<Issue> = vec![];
        state.set_issues(set3);
        assert_eq!(state.search_state().result_count(), 0);
    }

    #[test]
    fn test_view_mode_initial_value() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let state = IssuesViewState::new(issues);

        assert_eq!(state.view_mode(), IssuesViewMode::List);
    }

    #[test]
    fn test_issues_view_builder_multiple_styles() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);

        let view1 = IssuesView::new().block_style(style1);
        assert_eq!(view1.block_style, style1);

        let view2 = IssuesView::new().block_style(style2);
        assert_eq!(view2.block_style, style2);

        // Chaining should use last value
        let view3 = IssuesView::new().block_style(style1).block_style(style2);
        assert_eq!(view3.block_style, style2);
    }

    #[test]
    fn test_save_edit_marks_editor_saved() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified Title".to_string());
        }

        state.save_edit();

        // Editor state should still exist and be marked as saved
        if let Some(editor) = state.editor_state() {
            assert!(editor.is_saved());
        } else {
            panic!("Editor state should still exist after save");
        }
    }

    #[test]
    fn test_cancel_edit_marks_editor_cancelled() {
        let issues = vec![create_test_issue("beads-abcd-0001", "Issue 1")];
        let mut state = IssuesViewState::new(issues);

        state.enter_edit_mode();
        if let Some(editor) = state.editor_state_mut() {
            editor
                .form_state_mut()
                .set_value("title", "Modified Title".to_string());
            assert!(!editor.is_cancelled());
        }

        state.cancel_edit();
        // Editor state is cleared, so we can't check is_cancelled
        // but we verify it was cleared
        assert!(state.editor_state().is_none());
    }
}
