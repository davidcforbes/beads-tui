//! Unified form builder for Issue views
//!
//! Provides consistent form layout across Add, Edit, Read, and Split views

use crate::beads::models::Issue;
use crate::ui::widgets::{FormField, ValidationRule};

/// Form mode determines which fields are editable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueFormMode {
    /// Read-only mode (detail view)
    Read,
    /// Edit mode (editing existing issue)
    Edit,
    /// Create mode (creating new issue)
    Create,
}

/// Build a standard issue form with consistent field layout
///
/// All views (Read, Edit, Create, Split) use this same builder to ensure
/// consistent field ordering and labels across the application.
pub fn build_issue_form(mode: IssueFormMode, issue: Option<&Issue>) -> Vec<FormField> {
    let mut fields = Vec::new();
    let is_readonly = mode == IssueFormMode::Read;

    // Helper to get value or empty string
    let get_value = |issue: &Option<&Issue>, getter: fn(&Issue) -> String| -> String {
        issue.as_ref().map(|i| getter(i)).unwrap_or_default()
    };

    // ========== ID (Read-only) ==========
    if let Some(issue) = issue {
        fields.push(FormField::read_only("id", "ID", &issue.id));
    }

    // ========== Title ==========
    let title_value = get_value(&issue, |i| i.title.clone());
    if is_readonly {
        fields.push(FormField::read_only("title", "Title", &title_value));
    } else {
        fields.push(
            FormField::text("title", "Title")
                .value(&title_value)
                .placeholder("Brief description of the issue")
                .required()
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::MaxLength(256)),
        );
    }

    // ========== Status ==========
    let status_value = issue
        .as_ref()
        .map(|i| format!("{:?}", i.status))
        .unwrap_or_else(|| "Open".to_string());

    if is_readonly {
        fields.push(FormField::read_only("status", "Status", &status_value));
    } else {
        fields.push(
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
            .value(&status_value)
            .required(),
        );
    }

    // ========== Priority ==========
    let priority_value = issue
        .as_ref()
        .map(|i| format!("{}", i.priority))
        .unwrap_or_else(|| "P2".to_string());

    if is_readonly {
        fields.push(FormField::read_only("priority", "Priority", &priority_value));
    } else {
        fields.push(
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
            .value(&priority_value)
            .required(),
        );
    }

    // ========== Type ==========
    let type_value = issue
        .as_ref()
        .map(|i| format!("{:?}", i.issue_type))
        .unwrap_or_else(|| "Task".to_string());

    if is_readonly {
        fields.push(FormField::read_only("type", "Type", &type_value));
    } else {
        fields.push(
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
            .value(&type_value)
            .required(),
        );
    }

    // ========== Assignee ==========
    let assignee_value = issue
        .as_ref()
        .and_then(|i| i.assignee.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if assignee_value.is_empty() {
            "Unassigned"
        } else {
            assignee_value
        };
        fields.push(FormField::read_only("assignee", "Assignee", display_value));
    } else {
        fields.push(
            FormField::text("assignee", "Assignee")
                .value(assignee_value)
                .placeholder("Unassigned"),
        );
    }

    // ========== Labels ==========
    let labels_value = issue
        .as_ref()
        .map(|i| i.labels.join(", "))
        .unwrap_or_default();

    if is_readonly {
        let display_value = if labels_value.is_empty() {
            "No labels"
        } else {
            &labels_value
        };
        fields.push(FormField::read_only("labels", "Labels", display_value));
    } else {
        fields.push(
            FormField::text("labels", "Labels")
                .value(&labels_value)
                .placeholder("Comma-separated labels"),
        );
    }

    // ========== Description ==========
    let description_value = issue
        .as_ref()
        .and_then(|i| i.description.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if description_value.is_empty() {
            "No description"
        } else {
            description_value
        };
        fields.push(FormField::read_only("description", "Description", display_value));
    } else {
        fields.push(
            FormField::text_area("description", "Description")
                .value(description_value)
                .placeholder("Detailed description of the issue"),
        );
    }

    // ========== Dates (Read-only) ==========
    if let Some(issue) = issue {
        let created_str = issue.created.format("%Y-%m-%d %H:%M").to_string();
        fields.push(FormField::read_only("created", "Created", &created_str));

        let updated_str = issue.updated.format("%Y-%m-%d %H:%M").to_string();
        fields.push(FormField::read_only("updated", "Updated", &updated_str));

        if let Some(closed) = issue.closed {
            let closed_str = closed.format("%Y-%m-%d %H:%M").to_string();
            fields.push(FormField::read_only("closed", "Closed", &closed_str));
        }
    }

    // ========== Dependencies (Read-only) ==========
    if let Some(issue) = issue {
        if !issue.dependencies.is_empty() {
            let deps_str = issue.dependencies.join(", ");
            fields.push(FormField::read_only("dependencies", "Depends On", &deps_str));
        }

        if !issue.blocks.is_empty() {
            let blocks_str = issue.blocks.join(", ");
            fields.push(FormField::read_only("blocks", "Blocks", &blocks_str));
        }
    }

    // ========== Notes (Read-only) ==========
    if let Some(issue) = issue {
        if !issue.notes.is_empty() {
            let notes_text = issue
                .notes
                .iter()
                .map(|note| {
                    format!(
                        "{} - {}: {}",
                        note.timestamp.format("%Y-%m-%d %H:%M"),
                        note.author,
                        note.content
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            fields.push(FormField::read_only("notes", "Notes", &notes_text));
        }
    }

    fields
}
