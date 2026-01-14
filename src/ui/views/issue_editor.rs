use crate::beads::models::Issue;
use crate::ui::widgets::form::{FormField, FormState, ValidationRule};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use std::collections::HashMap;

#[cfg(test)]
use chrono::Utc;

/// Section grouping for organizing form fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Summary,
    Scheduling,
    Relationships,
    Labels,
    Text,
    Metadata,
}

impl Section {
    pub fn title(&self) -> &'static str {
        match self {
            Section::Summary => "Summary",
            Section::Scheduling => "Scheduling",
            Section::Relationships => "Relationships",
            Section::Labels => "Labels",
            Section::Text => "Text",
            Section::Metadata => "Metadata (Read-Only)",
        }
    }

    pub fn all() -> Vec<Section> {
        vec![
            Section::Summary,
            Section::Scheduling,
            Section::Relationships,
            Section::Labels,
            Section::Text,
            Section::Metadata,
        ]
    }
}

/// Tracks a field's original value for dirty checking
#[derive(Debug, Clone)]
struct FieldOriginal {
    value: String,
    section: Section,
}

/// Change record for displaying what has been modified
#[derive(Debug, Clone)]
pub struct FieldChange {
    pub field_id: String,
    pub label: String,
    pub old_value: String,
    pub new_value: String,
    pub section: Section,
}

/// State for the Issue Editor view
#[derive(Debug)]
pub struct IssueEditorState {
    /// The underlying form state
    form_state: FormState,
    /// Original issue being edited
    original_issue: Issue,
    /// Original values for dirty tracking
    original_values: HashMap<String, FieldOriginal>,
    /// Field to section mapping
    field_sections: HashMap<String, Section>,
    /// Whether to show the change summary panel
    show_change_summary: bool,
    /// Whether the editor has been saved
    is_saved: bool,
}

impl IssueEditorState {
    /// Create a new editor state from an existing issue
    pub fn new(issue: &Issue) -> Self {
        let mut fields = Vec::new();
        let mut original_values = HashMap::new();
        let mut field_sections = HashMap::new();

        // Helper to add a field and track its original value
        let mut add_field = |field: FormField, section: Section| {
            let field_id = field.id.clone();
            let original_value = field.value.clone();
            original_values.insert(
                field_id.clone(),
                FieldOriginal {
                    value: original_value,
                    section,
                },
            );
            field_sections.insert(field_id, section);
            fields.push(field);
        };

        // Section: Summary
        add_field(
            FormField::text("title", "Title")
                .required()
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::MaxLength(256))
                .value(&issue.title),
            Section::Summary,
        );

        add_field(
            FormField::selector(
                "status",
                "Status",
                vec![
                    "open".to_string(),
                    "in_progress".to_string(),
                    "blocked".to_string(),
                    "closed".to_string(),
                ],
            )
            .required()
            .value(issue.status.to_string())
            .with_validation(ValidationRule::Enum(vec![
                "open".to_string(),
                "in_progress".to_string(),
                "blocked".to_string(),
                "closed".to_string(),
            ])),
            Section::Summary,
        );

        add_field(
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
            .required()
            .value(issue.priority.to_string())
            .with_validation(ValidationRule::Enum(vec![
                "P0".to_string(),
                "P1".to_string(),
                "P2".to_string(),
                "P3".to_string(),
                "P4".to_string(),
            ])),
            Section::Summary,
        );

        add_field(
            FormField::selector(
                "type",
                "Type",
                vec![
                    "task".to_string(),
                    "bug".to_string(),
                    "feature".to_string(),
                    "epic".to_string(),
                    "chore".to_string(),
                ],
            )
            .required()
            .value(issue.issue_type.to_string())
            .with_validation(ValidationRule::Enum(vec![
                "task".to_string(),
                "bug".to_string(),
                "feature".to_string(),
                "epic".to_string(),
                "chore".to_string(),
            ])),
            Section::Summary,
        );

        add_field(
            FormField::text("assignee", "Assignee")
                .value(issue.assignee.as_deref().unwrap_or(""))
                .placeholder("username")
                .with_validation(ValidationRule::MaxLength(128)),
            Section::Summary,
        );

        // Section: Relationships
        add_field(
            FormField::text_area("dependencies", "Dependencies")
                .value(issue.dependencies.join("\n"))
                .placeholder("beads-xxxx-xxxx (one per line)")
                .with_validation(ValidationRule::MaxLength(2048)),
            Section::Relationships,
        );

        add_field(
            FormField::text_area("blocks", "Blocks")
                .value(issue.blocks.join("\n"))
                .placeholder("beads-xxxx-xxxx (one per line)")
                .with_validation(ValidationRule::MaxLength(2048)),
            Section::Relationships,
        );

        // Section: Labels
        add_field(
            FormField::text_area("labels", "Labels")
                .value(issue.labels.join("\n"))
                .placeholder("label-name (one per line)")
                .with_validation(ValidationRule::MaxLength(2048)),
            Section::Labels,
        );

        // Section: Text
        add_field(
            FormField::text_area("description", "Description")
                .value(issue.description.as_deref().unwrap_or(""))
                .placeholder("Detailed description of the issue")
                .with_validation(ValidationRule::MaxLength(1048576)),
            Section::Text,
        );

        // Section: Metadata (read-only)
        add_field(
            FormField::read_only("id", "Issue ID", &issue.id),
            Section::Metadata,
        );

        add_field(
            FormField::read_only(
                "created",
                "Created",
                &issue.created.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            ),
            Section::Metadata,
        );

        add_field(
            FormField::read_only(
                "updated",
                "Updated",
                &issue.updated.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            ),
            Section::Metadata,
        );

        if let Some(closed) = issue.closed {
            add_field(
                FormField::read_only(
                    "closed",
                    "Closed",
                    &closed.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                ),
                Section::Metadata,
            );
        }

        let form_state = FormState::new(fields);

        Self {
            form_state,
            original_issue: issue.clone(),
            original_values,
            field_sections,
            show_change_summary: true,
            is_saved: false,
        }
    }

    /// Get the underlying form state
    pub fn form_state(&self) -> &FormState {
        &self.form_state
    }

    /// Get mutable access to the form state
    pub fn form_state_mut(&mut self) -> &mut FormState {
        &mut self.form_state
    }

    /// Get the original issue
    pub fn original_issue(&self) -> &Issue {
        &self.original_issue
    }

    /// Check if any field has been modified
    pub fn is_dirty(&self) -> bool {
        !self.get_changes().is_empty()
    }

    /// Check if a specific field has been modified
    pub fn is_field_dirty(&self, field_id: &str) -> bool {
        if let Some(original) = self.original_values.get(field_id) {
            if let Some(current) = self.form_state.get_value(field_id) {
                return original.value != current;
            }
        }
        false
    }

    /// Get all field changes
    pub fn get_changes(&self) -> Vec<FieldChange> {
        let mut changes = Vec::new();

        for (field_id, original) in &self.original_values {
            if let Some(current_value) = self.form_state.get_value(field_id) {
                if original.value != current_value {
                    if let Some(field) = self.form_state.get_field(field_id) {
                        changes.push(FieldChange {
                            field_id: field_id.clone(),
                            label: field.label.clone(),
                            old_value: original.value.clone(),
                            new_value: current_value.to_string(),
                            section: original.section,
                        });
                    }
                }
            }
        }

        changes
    }

    /// Reset a specific field to its original value
    pub fn reset_field(&mut self, field_id: &str) {
        if let Some(original) = self.original_values.get(field_id) {
            self.form_state.set_value(field_id, original.value.clone());
        }
    }

    /// Reset all fields in a section to their original values
    pub fn reset_section(&mut self, section: Section) {
        for (field_id, original) in &self.original_values {
            if original.section == section {
                self.form_state.set_value(field_id, original.value.clone());
            }
        }
    }

    /// Reset all fields to their original values
    pub fn reset_all(&mut self) {
        for (field_id, original) in &self.original_values {
            self.form_state.set_value(field_id, original.value.clone());
        }
    }

    /// Reload from the original issue (discards all changes)
    pub fn reload_from_source(&mut self) {
        let issue = self.original_issue.clone();
        *self = Self::new(&issue);
    }

    /// Toggle the change summary panel visibility
    pub fn toggle_change_summary(&mut self) {
        self.show_change_summary = !self.show_change_summary;
    }

    /// Check if the change summary is visible
    pub fn is_change_summary_visible(&self) -> bool {
        self.show_change_summary
    }

    /// Validate all fields
    pub fn validate(&mut self) -> bool {
        self.form_state.validate()
    }

    /// Check if there are any validation errors
    pub fn has_errors(&self) -> bool {
        self.form_state.has_errors()
    }

    /// Get the issue ID
    pub fn issue_id(&self) -> &str {
        &self.original_issue.id
    }

    /// Convert form values back into an updated Issue
    pub fn get_updated_issue(&self, _original: &Issue) -> Option<Issue> {
        if self.has_errors() {
            return None;
        }

        let mut updated = self.original_issue.clone();

        // Apply form values to issue
        use crate::beads::models::{IssueStatus, IssueType, Priority};

        if let Some(title) = self.form_state.get_value("title") {
            updated.title = title.to_string();
        }
        if let Some(status_str) = self.form_state.get_value("status") {
            updated.status = match status_str {
                "open" => IssueStatus::Open,
                "in_progress" => IssueStatus::InProgress,
                "blocked" => IssueStatus::Blocked,
                "closed" => IssueStatus::Closed,
                _ => IssueStatus::Open,
            };
        }
        if let Some(priority_str) = self.form_state.get_value("priority") {
            updated.priority = match priority_str {
                "P0" => Priority::P0,
                "P1" => Priority::P1,
                "P2" => Priority::P2,
                "P3" => Priority::P3,
                "P4" => Priority::P4,
                _ => Priority::P2,
            };
        }
        if let Some(type_str) = self.form_state.get_value("type") {
            updated.issue_type = match type_str {
                "epic" => IssueType::Epic,
                "feature" => IssueType::Feature,
                "task" => IssueType::Task,
                "bug" => IssueType::Bug,
                "chore" => IssueType::Chore,
                _ => IssueType::Task,
            };
        }
        if let Some(assignee) = self.form_state.get_value("assignee") {
            updated.assignee = if assignee.is_empty() {
                None
            } else {
                Some(assignee.to_string())
            };
        }
        if let Some(description) = self.form_state.get_value("description") {
            updated.description = if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            };
        }

        // Parse list fields (dependencies, blocked_by, labels)
        if let Some(deps) = self.form_state.get_value("dependencies") {
            updated.dependencies = deps
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Some(blocks) = self.form_state.get_value("blocks") {
            updated.blocks = blocks
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Some(labels) = self.form_state.get_value("labels") {
            updated.labels = labels
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Some(updated)
    }

    /// Get an IssueUpdate with only changed fields
    pub fn get_issue_update(&self) -> Option<crate::beads::client::IssueUpdate> {
        use crate::beads::client::IssueUpdate;
        use crate::beads::models::{IssueStatus, IssueType, Priority};

        if self.has_errors() {
            return None;
        }

        let changes = self.get_changes();
        if changes.is_empty() {
            return None;
        }

        let mut update = IssueUpdate::new();

        for change in changes {
            match change.field_id.as_str() {
                "title" => {
                    update = update.title(change.new_value);
                }
                "status" => {
                    let status = match change.new_value.as_str() {
                        "open" => IssueStatus::Open,
                        "in_progress" => IssueStatus::InProgress,
                        "blocked" => IssueStatus::Blocked,
                        "closed" => IssueStatus::Closed,
                        _ => continue,
                    };
                    update = update.status(status);
                }
                "priority" => {
                    let priority = match change.new_value.as_str() {
                        "P0" => Priority::P0,
                        "P1" => Priority::P1,
                        "P2" => Priority::P2,
                        "P3" => Priority::P3,
                        "P4" => Priority::P4,
                        _ => continue,
                    };
                    update = update.priority(priority);
                }
                "type" => {
                    let issue_type = match change.new_value.as_str() {
                        "epic" => IssueType::Epic,
                        "feature" => IssueType::Feature,
                        "task" => IssueType::Task,
                        "bug" => IssueType::Bug,
                        "chore" => IssueType::Chore,
                        _ => continue,
                    };
                    update = update.issue_type(issue_type);
                }
                "assignee" => {
                    update = update.assignee(change.new_value);
                }
                "description" => {
                    update = update.description(change.new_value);
                }
                "labels" => {
                    let labels: Vec<String> = change
                        .new_value
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    update = update.labels(labels);
                }
                _ => {
                    // dependencies and blocks are not supported in IssueUpdate yet
                }
            }
        }

        Some(update)
    }

    /// Mark the editor as saved
    pub fn save(&mut self) {
        self.is_saved = true;
    }

    /// Mark the editor as cancelled
    pub fn cancel(&mut self) {
        // Could add cancelled state tracking in the future
    }

    /// Check if the editor has been saved
    pub fn is_saved(&self) -> bool {
        self.is_saved
    }

    /// Check if the editor has been cancelled
    pub fn is_cancelled(&self) -> bool {
        // Placeholder for future implementation
        false
    }

    /// Get fields grouped by section
    pub fn fields_by_section(&self) -> HashMap<Section, Vec<String>> {
        let mut grouped: HashMap<Section, Vec<String>> = HashMap::new();

        for (field_id, section) in &self.field_sections {
            grouped.entry(*section).or_default().push(field_id.clone());
        }

        grouped
    }
}

/// Widget for rendering the change summary panel
pub struct ChangeSummaryPanel<'a> {
    changes: &'a [FieldChange],
}

impl<'a> ChangeSummaryPanel<'a> {
    pub fn new(changes: &'a [FieldChange]) -> Self {
        Self { changes }
    }
}

impl<'a> Widget for ChangeSummaryPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Changes ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.changes.is_empty() {
            let text = Paragraph::new("No changes").style(Style::default().fg(Color::DarkGray));
            text.render(inner, buf);
            return;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("{} field(s) modified:", self.changes.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for change in self.changes {
            // Field label
            lines.push(Line::from(Span::styled(
                format!("• {}", change.label),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));

            // Old value
            let old_display = if change.old_value.is_empty() {
                "(empty)".to_string()
            } else if change.old_value.len() > 50 {
                format!("{}...", &change.old_value[..47])
            } else {
                change.old_value.clone()
            };
            lines.push(Line::from(vec![
                Span::raw("  Old: "),
                Span::styled(old_display, Style::default().fg(Color::Red)),
            ]));

            // New value
            let new_display = if change.new_value.is_empty() {
                "(empty)".to_string()
            } else if change.new_value.len() > 50 {
                format!("{}...", &change.new_value[..47])
            } else {
                change.new_value.clone()
            };
            lines.push(Line::from(vec![
                Span::raw("  New: "),
                Span::styled(new_display, Style::default().fg(Color::Green)),
            ]));

            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}

/// Widget for rendering the Issue Editor view
pub struct IssueEditorView {}

impl IssueEditorView {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IssueEditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for IssueEditorView {
    type State = IssueEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use crate::ui::widgets::form::Form;

        // For now, delegate to the Form widget
        let form = Form::default();
        StatefulWidget::render(form, area, buf, state.form_state_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_issue() -> Issue {
        use crate::beads::models::{IssueStatus, IssueType, Priority};
        Issue {
            id: "beads-test".to_string(),
            title: "Test Issue".to_string(),
            description: Some("Test description".to_string()),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: Some("user1".to_string()),
            dependencies: vec!["beads-dep1-0001".to_string()],
            blocks: vec![],
            labels: vec!["label1".to_string(), "label2".to_string()],
            created: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
            closed: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_new_editor_state() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        assert_eq!(state.original_issue().id, "beads-test");
        assert!(!state.is_dirty());
        assert!(state.get_changes().is_empty());
    }

    #[test]
    fn test_field_sections() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        // Check that fields are in correct sections
        assert_eq!(state.field_sections.get("title"), Some(&Section::Summary));

        assert_eq!(
            state.field_sections.get("dependencies"),
            Some(&Section::Relationships)
        );
        assert_eq!(
            state.field_sections.get("blocks"),
            Some(&Section::Relationships)
        );
        assert_eq!(state.field_sections.get("labels"), Some(&Section::Labels));
        assert_eq!(
            state.field_sections.get("description"),
            Some(&Section::Text)
        );
        assert_eq!(state.field_sections.get("id"), Some(&Section::Metadata));
    }

    #[test]
    fn test_is_dirty_after_modification() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        assert!(!state.is_dirty());

        state
            .form_state_mut()
            .set_value("title", "New Title".to_string());

        assert!(state.is_dirty());
        assert!(state.is_field_dirty("title"));
        assert!(!state.is_field_dirty("description"));
    }

    #[test]
    fn test_get_changes() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "New Title".to_string());
        state
            .form_state_mut()
            .set_value("description", "New description".to_string());

        let changes = state.get_changes();
        assert_eq!(changes.len(), 2);

        let title_change = changes.iter().find(|c| c.field_id == "title").unwrap();
        assert_eq!(title_change.old_value, "Test Issue");
        assert_eq!(title_change.new_value, "New Title");

        let desc_change = changes
            .iter()
            .find(|c| c.field_id == "description")
            .unwrap();
        assert_eq!(desc_change.old_value, "Test description");
        assert_eq!(desc_change.new_value, "New description");
    }

    #[test]
    fn test_reset_field() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified".to_string());
        assert!(state.is_field_dirty("title"));

        state.reset_field("title");
        assert!(!state.is_field_dirty("title"));
        assert_eq!(state.form_state().get_value("title").unwrap(), "Test Issue");
    }

    #[test]
    fn test_reset_section() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        // Modify multiple fields in Summary section
        state
            .form_state_mut()
            .set_value("title", "Modified Title".to_string());
        state
            .form_state_mut()
            .set_value("status", "in_progress".to_string());

        // Modify field in different section
        state
            .form_state_mut()
            .set_value("description", "Modified Desc".to_string());

        assert!(state.is_field_dirty("title"));
        assert!(state.is_field_dirty("status"));
        assert!(state.is_field_dirty("description"));

        // Reset Summary section
        state.reset_section(Section::Summary);

        assert!(!state.is_field_dirty("title"));
        assert!(!state.is_field_dirty("status"));
        assert!(state.is_field_dirty("description")); // Should remain dirty
    }

    #[test]
    fn test_reset_all() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified".to_string());
        state
            .form_state_mut()
            .set_value("description", "Modified Desc".to_string());
        assert!(state.is_dirty());

        state.reset_all();

        assert!(!state.is_dirty());
        assert!(state.get_changes().is_empty());
    }

    #[test]
    fn test_reload_from_source() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        state
            .form_state_mut()
            .set_value("title", "Modified".to_string());
        assert!(state.is_dirty());

        state.reload_from_source();

        assert!(!state.is_dirty());
        assert_eq!(state.form_state().get_value("title").unwrap(), "Test Issue");
    }

    #[test]
    fn test_toggle_change_summary() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        assert!(state.is_change_summary_visible());

        state.toggle_change_summary();
        assert!(!state.is_change_summary_visible());

        state.toggle_change_summary();
        assert!(state.is_change_summary_visible());
    }

    #[test]
    fn test_fields_by_section() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        let grouped = state.fields_by_section();

        assert!(grouped.contains_key(&Section::Summary));
        assert!(grouped.contains_key(&Section::Relationships));
        assert!(grouped.contains_key(&Section::Labels));
        assert!(grouped.contains_key(&Section::Text));
        assert!(grouped.contains_key(&Section::Metadata));

        let summary_fields = &grouped[&Section::Summary];
        assert!(summary_fields.contains(&"title".to_string()));
        assert!(summary_fields.contains(&"status".to_string()));
        assert!(summary_fields.contains(&"priority".to_string()));
    }

    #[test]
    fn test_validation() {
        let issue = create_test_issue();
        let mut state = IssueEditorState::new(&issue);

        // Valid state - validate should succeed for properly initialized state
        let validation_result = state.validate();
        if !validation_result {
            // Debug: print which fields have errors
            for field in state.form_state().fields() {
                if let Some(error) = &field.error {
                    eprintln!("Field '{}' has error: {}", field.id, error);
                }
            }
        }
        assert!(validation_result);
        assert!(!state.has_errors());

        // Invalid state - empty required field
        state.form_state_mut().set_value("title", String::new());
        assert!(!state.validate());
        assert!(state.has_errors());
    }

    #[test]
    fn test_metadata_fields_present() {
        let issue = create_test_issue();
        let state = IssueEditorState::new(&issue);

        assert!(state.form_state().get_value("id").is_some());
        assert!(state.form_state().get_value("created").is_some());
        assert!(state.form_state().get_value("updated").is_some());
    }

    #[test]
    fn test_change_summary_panel_empty() {
        let changes = vec![];
        let panel = ChangeSummaryPanel::new(&changes);

        // Just test that it can be created with empty changes
        assert_eq!(panel.changes.len(), 0);
    }

    #[test]
    fn test_change_summary_panel_with_changes() {
        let changes = vec![FieldChange {
            field_id: "title".to_string(),
            label: "Title".to_string(),
            old_value: "Old".to_string(),
            new_value: "New".to_string(),
            section: Section::Summary,
        }];
        let panel = ChangeSummaryPanel::new(&changes);

        assert_eq!(panel.changes.len(), 1);
    }
}
