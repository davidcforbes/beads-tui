//! Issue editor view for modifying existing issues

use crate::beads::{client::IssueUpdate, models::{Issue, IssueStatus, IssueType, Priority}};
use crate::ui::widgets::{Form, FormField, FormState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

/// Issue editor state
#[derive(Debug)]
pub struct IssueEditorState {
    form_state: FormState,
    issue_id: String,
    original_values: HashMap<String, String>,
    modified: bool,
    saved: bool,
    cancelled: bool,
}

impl IssueEditorState {
    /// Create a new issue editor state from an existing issue
    pub fn new(issue: &Issue) -> Self {
        let fields = vec![
            FormField::text("title", "Title")
                .required()
                .value(&issue.title)
                .placeholder("Brief description of the issue"),
            FormField::selector(
                "type",
                "Type",
                vec![
                    "Epic".to_string(),
                    "Feature".to_string(),
                    "Task".to_string(),
                    "Bug".to_string(),
                    "Chore".to_string(),
                ],
            )
            .value(format!("{:?}", issue.issue_type))
            .required(),
            FormField::selector(
                "priority",
                "Priority",
                vec![
                    "P0".to_string(),
                    "P1".to_string(),
                    "P2".to_string(),
                    "P3".to_string(),
                    "P4".to_string(),
                ],
            )
            .value(format!("{}", issue.priority))
            .required(),
            FormField::selector(
                "status",
                "Status",
                vec![
                    "Open".to_string(),
                    "InProgress".to_string(),
                    "Blocked".to_string(),
                    "Closed".to_string(),
                ],
            )
            .value(format!("{:?}", issue.status))
            .required(),
            FormField::text("assignee", "Assignee")
                .value(issue.assignee.as_deref().unwrap_or(""))
                .placeholder("username (optional)"),
            FormField::text("labels", "Labels")
                .value(issue.labels.join(", "))
                .placeholder("comma-separated labels (optional)"),
            FormField::text_area("description", "Description")
                .value(issue.description.as_deref().unwrap_or(""))
                .placeholder("Detailed description of the issue (optional)"),
        ];

        let mut original_values = HashMap::new();
        original_values.insert("title".to_string(), issue.title.clone());
        original_values.insert("type".to_string(), format!("{:?}", issue.issue_type));
        original_values.insert("priority".to_string(), format!("{}", issue.priority));
        original_values.insert("status".to_string(), format!("{:?}", issue.status));
        original_values.insert("assignee".to_string(), issue.assignee.clone().unwrap_or_default());
        original_values.insert("labels".to_string(), issue.labels.join(", "));
        original_values.insert("description".to_string(), issue.description.clone().unwrap_or_default());

        Self {
            form_state: FormState::new(fields),
            issue_id: issue.id.clone(),
            original_values,
            modified: false,
            saved: false,
            cancelled: false,
        }
    }

    /// Get the issue ID being edited
    pub fn issue_id(&self) -> &str {
        &self.issue_id
    }

    /// Get the underlying form state
    pub fn form_state(&self) -> &FormState {
        &self.form_state
    }

    /// Get mutable reference to the underlying form state
    pub fn form_state_mut(&mut self) -> &mut FormState {
        self.mark_modified();
        &mut self.form_state
    }

    /// Check if the form has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark as modified
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    /// Check if the form has been saved
    pub fn is_saved(&self) -> bool {
        self.saved
    }

    /// Mark as saved
    pub fn save(&mut self) {
        self.saved = true;
        self.modified = false;
    }

    /// Check if the form has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Validate the form
    pub fn validate(&mut self) -> bool {
        self.form_state.validate()
    }

    /// Get the form data as an updated issue
    pub fn get_updated_issue(&self, original: &Issue) -> Option<Issue> {
        if self.form_state.has_errors() {
            return None;
        }

        let title = self.form_state.get_value("title")?.to_string();
        let type_str = self.form_state.get_value("type")?;
        let priority_str = self.form_state.get_value("priority")?;
        let status_str = self.form_state.get_value("status")?;
        let assignee = self.form_state.get_value("assignee");
        let labels_str = self.form_state.get_value("labels");
        let description = self.form_state.get_value("description");

        let issue_type = match type_str {
            "Epic" => IssueType::Epic,
            "Feature" => IssueType::Feature,
            "Task" => IssueType::Task,
            "Bug" => IssueType::Bug,
            "Chore" => IssueType::Chore,
            _ => return None,
        };

        let priority = match priority_str {
            "P0" => Priority::P0,
            "P1" => Priority::P1,
            "P2" => Priority::P2,
            "P3" => Priority::P3,
            "P4" => Priority::P4,
            _ => return None,
        };

        let status = match status_str {
            "Open" => IssueStatus::Open,
            "InProgress" => IssueStatus::InProgress,
            "Blocked" => IssueStatus::Blocked,
            "Closed" => IssueStatus::Closed,
            _ => return None,
        };

        let assignee_opt = assignee.and_then(|a| {
            if a.trim().is_empty() {
                None
            } else {
                Some(a.to_string())
            }
        });

        let labels = labels_str
            .map(|l| {
                l.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let description_opt = description.and_then(|d| {
            if d.trim().is_empty() {
                None
            } else {
                Some(d.to_string())
            }
        });

        Some(Issue {
            id: original.id.clone(),
            title,
            description: description_opt,
            issue_type,
            status,
            priority,
            labels,
            assignee: assignee_opt,
            created: original.created,
            updated: chrono::Utc::now(),
            closed: if status == IssueStatus::Closed {
                Some(chrono::Utc::now())
            } else {
                None
            },
            dependencies: original.dependencies.clone(),
            blocks: original.blocks.clone(),
            notes: original.notes.clone(),
        })
    }

    /// Get a map of changed fields with their old and new values
    pub fn get_changed_fields(&self) -> HashMap<String, (String, String)> {
        let mut changes = HashMap::new();
        
        for (field_name, original_value) in &self.original_values {
            if let Some(current_value) = self.form_state.get_value(field_name) {
                let current_str = current_value.to_string();
                if &current_str != original_value {
                    changes.insert(
                        field_name.clone(),
                        (original_value.clone(), current_str),
                    );
                }
            }
        }
        
        changes
    }

    /// Check if any fields have been changed
    pub fn has_changes(&self) -> bool {
        !self.get_changed_fields().is_empty()
    }

    /// Get a formatted change summary for display
    pub fn get_change_summary(&self) -> Vec<String> {
        let changes = self.get_changed_fields();
        let mut summary = Vec::new();
        
        // Sort field names for consistent display
        let mut field_names: Vec<_> = changes.keys().collect();
        field_names.sort();
        
        for field_name in field_names {
            if let Some((old_val, new_val)) = changes.get(field_name) {
                let old_display = if old_val.is_empty() { "<empty>" } else { old_val };
                let new_display = if new_val.is_empty() { "<empty>" } else { new_val };
                summary.push(format!("{field_name}: {old_display} → {new_display}"));
            }
        }
        
        summary
    }

    /// Build an IssueUpdate containing only the changed fields
    pub fn get_issue_update(&self) -> Option<IssueUpdate> {
        if !self.has_changes() {
            return None;
        }

        let changes = self.get_changed_fields();
        let mut update = IssueUpdate::new();

        // Only add fields that have changed
        if changes.contains_key("title") {
            if let Some(title) = self.form_state.get_value("title") {
                update = update.title(title.to_string());
            }
        }

        if changes.contains_key("type") {
            if let Some(type_str) = self.form_state.get_value("type") {
                let issue_type = match type_str {
                    "Epic" => IssueType::Epic,
                    "Feature" => IssueType::Feature,
                    "Task" => IssueType::Task,
                    "Bug" => IssueType::Bug,
                    "Chore" => IssueType::Chore,
                    _ => return None,
                };
                update = update.issue_type(issue_type);
            }
        }

        if changes.contains_key("status") {
            if let Some(status_str) = self.form_state.get_value("status") {
                let status = match status_str {
                    "Open" => IssueStatus::Open,
                    "InProgress" => IssueStatus::InProgress,
                    "Blocked" => IssueStatus::Blocked,
                    "Closed" => IssueStatus::Closed,
                    _ => return None,
                };
                update = update.status(status);
            }
        }

        if changes.contains_key("priority") {
            if let Some(priority_str) = self.form_state.get_value("priority") {
                let priority = match priority_str {
                    "P0" => Priority::P0,
                    "P1" => Priority::P1,
                    "P2" => Priority::P2,
                    "P3" => Priority::P3,
                    "P4" => Priority::P4,
                    _ => return None,
                };
                update = update.priority(priority);
            }
        }

        if changes.contains_key("assignee") {
            if let Some(assignee) = self.form_state.get_value("assignee") {
                update = update.assignee(assignee.to_string());
            }
        }

        if changes.contains_key("labels") {
            if let Some(labels_str) = self.form_state.get_value("labels") {
                let labels: Vec<String> = labels_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                update = update.labels(labels);
            }
        }

        if changes.contains_key("description") {
            if let Some(description) = self.form_state.get_value("description") {
                update = update.description(description.to_string());
            }
        }

        Some(update)
    }

    /// Reset form to original issue data
    pub fn reset(&mut self, issue: &Issue) {
        *self = Self::new(issue);
    }
}

/// Issue editor view widget
pub struct IssueEditorView<'a> {
    show_help: bool,
    block_style: Style,
    help_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> IssueEditorView<'a> {
    /// Create a new issue editor view
    pub fn new() -> Self {
        Self {
            show_help: true,
            block_style: Style::default().fg(Color::Cyan),
            help_style: Style::default().fg(Color::DarkGray),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Show or hide help
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
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

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer, state: &IssueEditorState) {
        let title_text = if state.is_modified() {
            format!("Editing: {} [modified]", state.issue_id())
        } else {
            format!("Editing: {}", state.issue_id())
        };

        let line = Line::from(vec![Span::styled(
            &title_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]);

        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }

    fn render_help_bar(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_help {
            return;
        }

        let help_text =
            "Ctrl+S: Save | Ctrl+Q: Cancel | Tab/Shift+Tab: Navigate Fields | Enter: Next Field";

        let line = Line::from(Span::styled(help_text, self.help_style));
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

impl<'a> Default for IssueEditorView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for IssueEditorView<'a> {
    type State = IssueEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout: title bar (1) + form (fill) + help bar (1)
        let mut constraints = vec![Constraint::Length(1)]; // Title bar

        // Form area
        constraints.push(Constraint::Min(10));

        // Help bar (if visible)
        if self.show_help {
            constraints.push(Constraint::Length(1));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_idx = 0;

        // Render title bar
        self.render_title_bar(chunks[chunk_idx], buf, state);
        chunk_idx += 1;

        // Render form
        let form_block = Block::default()
            .borders(Borders::ALL)
            .title("Edit Issue")
            .style(self.block_style);

        let form = Form::new().block(form_block);

        StatefulWidget::render(form, chunks[chunk_idx], buf, &mut state.form_state);
        chunk_idx += 1;

        // Render help bar if visible
        if self.show_help {
            self.render_help_bar(chunks[chunk_idx], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_issue() -> Issue {
        Issue {
            id: "beads-001".to_string(),
            title: "Test Issue".to_string(),
            description: Some("This is a test issue".to_string()),
            issue_type: IssueType::Task,
            status: IssueStatus::Open,
            priority: Priority::P2,
            labels: vec!["test".to_string(), "demo".to_string()],
            assignee: Some("john".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec!["beads-000".to_string()],
            blocks: vec!["beads-002".to_string()],
            notes: vec![],
        }
    }

    #[test]
    fn test_issue_editor_state_creation() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);
        assert_eq!(state.issue_id(), "beads-001");
        assert!(!state.is_modified());
        assert!(!state.is_saved());
        assert!(!state.is_cancelled());
    }

    #[test]
    fn test_issue_editor_state_preloads_values() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        assert_eq!(state.form_state().get_value("title"), Some("Test Issue"));
        assert_eq!(state.form_state().get_value("type"), Some("Task"));
        assert_eq!(state.form_state().get_value("priority"), Some("P2"));
        assert_eq!(state.form_state().get_value("status"), Some("Open"));
        assert_eq!(state.form_state().get_value("assignee"), Some("john"));
        assert_eq!(state.form_state().get_value("labels"), Some("test, demo"));
        assert_eq!(
            state.form_state().get_value("description"),
            Some("This is a test issue")
        );
    }

    #[test]
    fn test_mark_modified() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);
        assert!(!state.is_modified());

        state.mark_modified();
        assert!(state.is_modified());
    }

    #[test]
    fn test_save() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);
        state.mark_modified();
        assert!(state.is_modified());
        assert!(!state.is_saved());

        state.save();
        assert!(!state.is_modified());
        assert!(state.is_saved());
    }

    #[test]
    fn test_cancel() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);
        assert!(!state.is_cancelled());

        state.cancel();
        assert!(state.is_cancelled());
    }

    #[test]
    fn test_reset() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);
        state.mark_modified();
        state.save();

        state.reset(&issue);
        assert!(!state.is_modified());
        assert!(!state.is_saved());
    }

    #[test]
    fn test_form_state_mut_marks_modified() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);
        assert!(!state.is_modified());

        let _form = state.form_state_mut();
        assert!(state.is_modified());
    }

    #[test]
    fn test_get_updated_issue_with_valid_data() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let updated = state.get_updated_issue(&issue);
        assert!(updated.is_some());

        let updated = updated.unwrap();
        assert_eq!(updated.title, "Test Issue");
        assert_eq!(updated.issue_type, IssueType::Task);
        assert_eq!(updated.priority, Priority::P2);
        assert_eq!(updated.status, IssueStatus::Open);
        assert_eq!(updated.assignee, Some("john".to_string()));
        assert_eq!(updated.labels, vec!["test".to_string(), "demo".to_string()]);
        assert_eq!(
            updated.description,
            Some("This is a test issue".to_string())
        );
    }

    #[test]
    fn test_get_updated_issue_with_modified_title() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.title, "Modified Title");
    }

    #[test]
    fn test_get_updated_issue_with_invalid_type() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("type", "InvalidType".to_string());

        let updated = state.get_updated_issue(&issue);
        assert!(updated.is_none());
    }

    #[test]
    fn test_get_updated_issue_with_invalid_priority() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("priority", "P99".to_string());

        let updated = state.get_updated_issue(&issue);
        assert!(updated.is_none());
    }

    #[test]
    fn test_get_updated_issue_with_invalid_status() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("status", "InvalidStatus".to_string());

        let updated = state.get_updated_issue(&issue);
        assert!(updated.is_none());
    }

    #[test]
    fn test_get_updated_issue_with_empty_assignee() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("assignee", "".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.assignee, None);
    }

    #[test]
    fn test_get_updated_issue_with_whitespace_assignee() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("assignee", "   ".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.assignee, None);
    }

    #[test]
    fn test_get_updated_issue_with_empty_description() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("description", "".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.description, None);
    }

    #[test]
    fn test_get_updated_issue_with_multiple_labels() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("labels", "bug, urgent, p0".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(
            updated.labels,
            vec!["bug".to_string(), "urgent".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn test_get_updated_issue_filters_empty_labels() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("labels", "bug, , urgent, , p0".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(
            updated.labels,
            vec!["bug".to_string(), "urgent".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn test_get_updated_issue_sets_closed_time_when_closed() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("status", "Closed".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.status, IssueStatus::Closed);
        assert!(updated.closed.is_some());
    }

    #[test]
    fn test_has_changes_returns_false_for_unmodified() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        assert!(!state.has_changes());
    }

    #[test]
    fn test_has_changes_returns_true_after_modification() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());

        assert!(state.has_changes());
    }

    #[test]
    fn test_get_changed_fields() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());
        state
            .form_state_mut()
            .set_value("priority", "P0".to_string());

        let changes = state.get_changed_fields();
        assert_eq!(changes.len(), 2);
        assert_eq!(
            changes.get("title"),
            Some(&("Test Issue".to_string(), "Modified Title".to_string()))
        );
        assert_eq!(
            changes.get("priority"),
            Some(&("P2".to_string(), "P0".to_string()))
        );
    }

    #[test]
    fn test_get_change_summary() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());
        state.form_state_mut().set_value("assignee", "".to_string());

        let summary = state.get_change_summary();
        assert_eq!(summary.len(), 2);
        assert!(summary.contains(&"assignee: john → <empty>".to_string()));
        assert!(summary.contains(&"title: Test Issue → Modified Title".to_string()));
    }

    #[test]
    fn test_get_issue_update_returns_none_when_no_changes() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let update = state.get_issue_update();
        assert!(update.is_none());
    }

    #[test]
    fn test_get_issue_update_includes_only_changed_fields() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_validate_returns_true_for_valid_data() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        assert!(state.validate());
    }

    #[test]
    fn test_issue_editor_view_new() {
        let view = IssueEditorView::new();
        assert!(view.show_help);
    }

    #[test]
    fn test_issue_editor_view_default() {
        let view = IssueEditorView::default();
        assert!(view.show_help);
    }

    #[test]
    fn test_issue_editor_view_show_help() {
        let view = IssueEditorView::new().show_help(false);
        assert!(!view.show_help);
    }

    #[test]
    fn test_issue_editor_view_block_style() {
        let style = Style::default().fg(Color::Red);
        let view = IssueEditorView::new().block_style(style);
        assert_eq!(view.block_style.fg, Some(Color::Red));
    }

    #[test]
    fn test_issue_editor_view_help_style() {
        let style = Style::default().fg(Color::Yellow);
        let view = IssueEditorView::new().help_style(style);
        assert_eq!(view.help_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_issue_editor_view_builder_chain() {
        let block_style = Style::default().fg(Color::Magenta);
        let help_style = Style::default().fg(Color::Green);

        let view = IssueEditorView::new()
            .show_help(false)
            .block_style(block_style)
            .help_style(help_style);

        assert!(!view.show_help);
        assert_eq!(view.block_style.fg, Some(Color::Magenta));
        assert_eq!(view.help_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_get_updated_issue_with_epic_type() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "Epic".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.issue_type, IssueType::Epic);
    }

    #[test]
    fn test_get_updated_issue_with_feature_type() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "Feature".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.issue_type, IssueType::Feature);
    }

    #[test]
    fn test_get_updated_issue_with_bug_type() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "Bug".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.issue_type, IssueType::Bug);
    }

    #[test]
    fn test_get_updated_issue_with_chore_type() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "Chore".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.issue_type, IssueType::Chore);
    }

    #[test]
    fn test_get_updated_issue_with_p0_priority() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P0".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.priority, Priority::P0);
    }

    #[test]
    fn test_get_updated_issue_with_p1_priority() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P1".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.priority, Priority::P1);
    }

    #[test]
    fn test_get_updated_issue_with_p3_priority() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P3".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.priority, Priority::P3);
    }

    #[test]
    fn test_get_updated_issue_with_p4_priority() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P4".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.priority, Priority::P4);
    }

    #[test]
    fn test_get_updated_issue_with_in_progress_status() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "InProgress".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.status, IssueStatus::InProgress);
    }

    #[test]
    fn test_get_updated_issue_with_blocked_status() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "Blocked".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.status, IssueStatus::Blocked);
    }

    #[test]
    fn test_issue_with_no_assignee_initially() {
        let mut issue = create_test_issue();
        issue.assignee = None;

        let state = IssueEditorState::new(&issue);
        assert_eq!(state.form_state().get_value("assignee"), Some(""));
    }

    #[test]
    fn test_issue_with_no_labels_initially() {
        let mut issue = create_test_issue();
        issue.labels = vec![];

        let state = IssueEditorState::new(&issue);
        assert_eq!(state.form_state().get_value("labels"), Some(""));
    }

    #[test]
    fn test_issue_with_no_description_initially() {
        let mut issue = create_test_issue();
        issue.description = None;

        let state = IssueEditorState::new(&issue);
        assert_eq!(state.form_state().get_value("description"), Some(""));
    }

    #[test]
    fn test_labels_with_leading_trailing_whitespace() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("labels", "  bug  ,  urgent  ,  p0  ".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(
            updated.labels,
            vec!["bug".to_string(), "urgent".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn test_labels_with_empty_string() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("labels", "".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.labels, Vec::<String>::new());
    }

    #[test]
    fn test_get_changed_fields_with_no_changes() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let changes = state.get_changed_fields();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_get_change_summary_with_no_changes() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let summary = state.get_change_summary();
        assert!(summary.is_empty());
    }

    #[test]
    fn test_multiple_modifications_then_reset() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("title", "Modified Title".to_string());
        state.form_state_mut().set_value("priority", "P0".to_string());
        assert!(state.has_changes());

        state.reset(&issue);
        assert!(!state.has_changes());
        assert_eq!(state.form_state().get_value("title"), Some("Test Issue"));
        assert_eq!(state.form_state().get_value("priority"), Some("P2"));
    }

    #[test]
    fn test_changing_status_from_open_to_in_progress() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "InProgress".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.status, IssueStatus::InProgress);
        assert!(updated.closed.is_none());
    }

    #[test]
    fn test_changing_status_from_open_to_blocked() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "Blocked".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.status, IssueStatus::Blocked);
        assert!(updated.closed.is_none());
    }

    #[test]
    fn test_multiple_fields_changed() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("title", "New Title".to_string());
        state.form_state_mut().set_value("type", "Bug".to_string());
        state.form_state_mut().set_value("priority", "P0".to_string());
        state.form_state_mut().set_value("status", "InProgress".to_string());

        let changes = state.get_changed_fields();
        assert_eq!(changes.len(), 4);
        assert!(changes.contains_key("title"));
        assert!(changes.contains_key("type"));
        assert!(changes.contains_key("priority"));
        assert!(changes.contains_key("status"));
    }

    #[test]
    fn test_form_state_const_access_does_not_mark_modified() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let _form = state.form_state();
        assert!(!state.is_modified());
    }

    #[test]
    fn test_get_issue_update_with_title_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("title", "New Title".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_type_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "Bug".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_status_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "InProgress".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_priority_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P0".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_assignee_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("assignee", "alice".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_labels_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("labels", "bug, urgent".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_get_issue_update_with_description_change() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("description", "New description".to_string());

        let update = state.get_issue_update();
        assert!(update.is_some());
    }

    #[test]
    fn test_changing_assignee_from_some_to_none() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("assignee", "".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.assignee, None);
    }

    #[test]
    fn test_changing_description_from_some_to_none() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("description", "".to_string());

        let updated = state.get_updated_issue(&issue).unwrap();
        assert_eq!(updated.description, None);
    }

    #[test]
    fn test_builder_method_chaining_order_independence() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);

        let view1 = IssueEditorView::new()
            .show_help(false)
            .block_style(style1)
            .help_style(style2);

        let view2 = IssueEditorView::new()
            .help_style(style2)
            .block_style(style1)
            .show_help(false);

        assert_eq!(view1.show_help, view2.show_help);
        assert_eq!(view1.block_style.fg, view2.block_style.fg);
        assert_eq!(view1.help_style.fg, view2.help_style.fg);
    }

    #[test]
    fn test_get_issue_update_with_invalid_type_returns_none() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("type", "InvalidType".to_string());

        let update = state.get_issue_update();
        assert!(update.is_none());
    }

    #[test]
    fn test_get_issue_update_with_invalid_status_returns_none() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("status", "InvalidStatus".to_string());

        let update = state.get_issue_update();
        assert!(update.is_none());
    }

    #[test]
    fn test_get_issue_update_with_invalid_priority_returns_none() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("priority", "P99".to_string());

        let update = state.get_issue_update();
        assert!(update.is_none());
    }

    #[test]
    fn test_save_marks_not_modified() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("title", "Modified".to_string());
        assert!(state.is_modified());

        state.save();
        assert!(!state.is_modified());
    }

    #[test]
    fn test_get_change_summary_displays_empty_values() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state.form_state_mut().set_value("description", "".to_string());

        let summary = state.get_change_summary();
        assert_eq!(summary.len(), 1);
        assert!(summary[0].contains("<empty>"));
    }
}
