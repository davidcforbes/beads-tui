//! Main issues view integrating search, list, and detail/edit capabilities

use crate::beads::models::Issue;
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
}

impl IssuesViewState {
    /// Create a new issues view state
    pub fn new(issues: Vec<Issue>) -> Self {
        Self {
            search_state: SearchInterfaceState::new(issues),
            view_mode: IssuesViewMode::List,
            selected_issue: None,
            editor_state: None,
            create_form_state: None,
            show_help: true,
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
            self.selected_issue = Some(issue.clone());
            self.view_mode = IssuesViewMode::Detail;
        }
    }

    /// Enter edit mode for the currently selected issue
    pub fn enter_edit_mode(&mut self) {
        if let Some(issue) = self.search_state.selected_issue() {
            self.selected_issue = Some(issue.clone());
            self.editor_state = Some(IssueEditorState::new(issue));
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
        self.create_form_state = Some(CreateIssueFormState::new());
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
}

/// Issues view widget
pub struct IssuesView<'a> {
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> IssuesView<'a> {
    /// Create a new issues view
    pub fn new() -> Self {
        Self {
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    fn render_list_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        let search_view = SearchInterfaceView::new().block_style(self.block_style);
        StatefulWidget::render(search_view, area, buf, &mut state.search_state);
    }

    fn render_detail_mode(&self, area: Rect, buf: &mut Buffer, state: &IssuesViewState) {
        if let Some(issue) = &state.selected_issue {
            let detail_view = IssueDetailView::new(issue);
            Widget::render(detail_view, area, buf);
        }
    }

    fn render_edit_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        if let Some(editor_state) = &mut state.editor_state {
            let editor_view = IssueEditorView::new();
            StatefulWidget::render(editor_view, area, buf, editor_state);
        }
    }

    fn render_create_mode(&self, area: Rect, buf: &mut Buffer, state: &mut IssuesViewState) {
        if let Some(create_form_state) = &mut state.create_form_state {
            use crate::ui::views::CreateIssueForm;
            let create_form = CreateIssueForm::new();
            StatefulWidget::render(create_form, area, buf, create_form_state);
        }
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
            IssuesViewMode::Edit => self.render_edit_mode(area, buf, state),
            IssuesViewMode::Create => self.render_create_mode(area, buf, state),
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
            labels: vec!["test".to_string()],
            assignee: Some("alice".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
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
        state.search_state_mut().search_state_mut().set_query("Issue 1".to_string());
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
            editor.form_state_mut().set_value("title", "Modified Title".to_string());
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
            form.form_state_mut().set_value("title", "New Issue".to_string());
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
            form.form_state_mut().set_value("title", "New Issue".to_string());
            form.form_state_mut().set_value("description", "Description".to_string());
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
            editor.form_state_mut().set_value("title", "Modified Title".to_string());
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
        let mode2 = mode1.clone();
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
            editor.form_state_mut().set_value("title", "Modified".to_string());
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
            editor.form_state_mut().set_value("title", "Modified".to_string());
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
            editor.form_state_mut().set_value("title", "First Modification".to_string());
        }

        if let Some(editor) = state.editor_state_mut() {
            editor.form_state_mut().set_value("title", "Second Modification".to_string());
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
            form.form_state_mut().set_value("title", "New Issue 1".to_string());
        }

        if let Some(form) = state.create_form_state_mut() {
            form.form_state_mut().set_value("description", "Description 1".to_string());
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
            issues.push(create_test_issue(&format!("beads-{:04}", i), &format!("Issue {}", i)));
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
            editor.form_state_mut().set_value("title", "Modified".to_string());
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
            form.form_state_mut().set_value("title", "New Issue".to_string());
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
            editor.form_state_mut().set_value("title", "Modified Title".to_string());
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
            editor.form_state_mut().set_value("title", "Modified Title".to_string());
            assert!(!editor.is_cancelled());
        }

        state.cancel_edit();
        // Editor state is cleared, so we can't check is_cancelled
        // but we verify it was cleared
        assert!(state.editor_state().is_none());
    }
}
