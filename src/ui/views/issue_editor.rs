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
}
