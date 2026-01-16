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

    // ========== Est Minutes ==========
    let est_minutes_value = issue
        .as_ref()
        .and_then(|i| i.est_minutes.as_ref())
        .map(|m| m.to_string())
        .unwrap_or_default();

    if is_readonly {
        let display_value = if est_minutes_value.is_empty() {
            "Not estimated"
        } else {
            &est_minutes_value
        };
        fields.push(FormField::read_only("est_minutes", "Est Minutes", display_value));
    } else {
        fields.push(
            FormField::text("est_minutes", "Est Minutes")
                .value(&est_minutes_value)
                .placeholder("Estimated time in minutes"),
        );
    }

    // ========== Due Date ==========
    let due_date_value = issue
        .as_ref()
        .and_then(|i| i.due_date.as_ref())
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default();

    if is_readonly {
        let display_value = if due_date_value.is_empty() {
            "No due date"
        } else {
            &due_date_value
        };
        fields.push(FormField::read_only("due_date", "Due Date", display_value));
    } else {
        fields.push(
            FormField::text("due_date", "Due Date")
                .value(&due_date_value)
                .placeholder("YYYY-MM-DD HH:MM"),
        );
    }

    // ========== Defer Date ==========
    let defer_date_value = issue
        .as_ref()
        .and_then(|i| i.defer_date.as_ref())
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default();

    if is_readonly {
        let display_value = if defer_date_value.is_empty() {
            "No defer date"
        } else {
            &defer_date_value
        };
        fields.push(FormField::read_only("defer_date", "Defer Date", display_value));
    } else {
        fields.push(
            FormField::text("defer_date", "Defer Date")
                .value(&defer_date_value)
                .placeholder("YYYY-MM-DD HH:MM"),
        );
    }

    // ========== Close Reason ==========
    let close_reason_value = issue
        .as_ref()
        .and_then(|i| i.close_reason.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if close_reason_value.is_empty() {
            "N/A"
        } else {
            close_reason_value
        };
        fields.push(FormField::read_only("close_reason", "Close Reason", display_value));
    } else {
        fields.push(
            FormField::text("close_reason", "Close Reason")
                .value(close_reason_value)
                .placeholder("Reason for closing"),
        );
    }

    // ========== External Reference ==========
    let external_ref_value = issue
        .as_ref()
        .and_then(|i| i.external_reference.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if external_ref_value.is_empty() {
            "No reference"
        } else {
            external_ref_value
        };
        fields.push(FormField::read_only("external_reference", "External Ref", display_value));
    } else {
        fields.push(
            FormField::text("external_reference", "External Ref")
                .value(external_ref_value)
                .placeholder("External link or reference"),
        );
    }

    // ========== Flags ==========
    let flags_value = issue
        .as_ref()
        .map(|i| {
            let mut flags = Vec::new();
            if i.flags.pinned { flags.push("Pinned"); }
            if i.flags.template { flags.push("Template"); }
            if i.flags.ephemeral { flags.push("Ephemeral"); }
            flags.join(", ")
        })
        .unwrap_or_default();

    let display_value = if flags_value.is_empty() {
        "No flags"
    } else {
        &flags_value
    };
    fields.push(FormField::read_only("flags", "Flags", display_value));

    // ========== Design Notes ==========
    let design_notes_value = issue
        .as_ref()
        .and_then(|i| i.design_notes.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if design_notes_value.is_empty() {
            "No design notes"
        } else {
            design_notes_value
        };
        fields.push(FormField::read_only("design_notes", "Design Notes", display_value));
    } else {
        fields.push(
            FormField::text_area("design_notes", "Design Notes")
                .value(design_notes_value)
                .placeholder("Technical design notes"),
        );
    }

    // ========== Acceptance Criteria ==========
    let acceptance_criteria_value = issue
        .as_ref()
        .and_then(|i| i.acceptance_criteria.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if is_readonly {
        let display_value = if acceptance_criteria_value.is_empty() {
            "No acceptance criteria"
        } else {
            acceptance_criteria_value
        };
        fields.push(FormField::read_only("acceptance_criteria", "Accept Criteria", display_value));
    } else {
        fields.push(
            FormField::text_area("acceptance_criteria", "Accept Criteria")
                .value(acceptance_criteria_value)
                .placeholder("Acceptance criteria"),
        );
    }

    // ========== Parent ID ==========
    let parent_id_value = issue
        .as_ref()
        .and_then(|i| i.parent_id.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    let display_value = if parent_id_value.is_empty() {
        "No parent"
    } else {
        parent_id_value
    };
    fields.push(FormField::read_only("parent_id", "Parent ID", display_value));

    // ========== Children IDs ==========
    if let Some(issue) = issue {
        if !issue.children_ids.is_empty() {
            let children_str = issue.children_ids.join(", ");
            fields.push(FormField::read_only("children_ids", "Children", &children_str));
        }
    }

    // ========== Event IDs ==========
    if let Some(issue) = issue {
        if !issue.event_ids.is_empty() {
            let events_str = issue.event_ids.join(", ");
            fields.push(FormField::read_only("event_ids", "Events", &events_str));
        }
    }

    // ========== Discovered IDs ==========
    if let Some(issue) = issue {
        if !issue.discovered_ids.is_empty() {
            let discovered_str = issue.discovered_ids.join(", ");
            fields.push(FormField::read_only("discovered_ids", "Discovered", &discovered_str));
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

/// Build an issue form with organized sections
///
/// Organizes fields into logical sections:
/// - [KEY ATTRIBUTES]: Core issue metadata
/// - [DETAILS]: Description and notes
/// - [RELATIONSHIPS]: Dependencies and blocks
/// - [METADATA][]: Timestamps (read mode only)
pub fn build_issue_form_with_sections(mode: IssueFormMode, issue: Option<&Issue>) -> Vec<FormField> {
    let mut fields = Vec::new();
    let is_readonly = mode == IssueFormMode::Read;

    // Helper to get value or empty string
    let get_value = |issue: &Option<&Issue>, getter: fn(&Issue) -> String| -> String {
        issue.as_ref().map(|i| getter(i)).unwrap_or_default()
    };

    // ========================================
    // SECTION 1: KEY ATTRIBUTES
    // ========================================
    fields.push(FormField::section_header("section_key_attributes", "[KEY ATTRIBUTES]"));

    // ID (Read-only, only in Read/Edit modes)
    if let Some(issue) = issue {
        fields.push(FormField::read_only("id", "ID", &issue.id));
    }

    // Title
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

    // Type (Selector for better UX)
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

    // Status (Selector)
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

    // Priority (Selector)
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

    // Assignee
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

    // Labels (Using LabelEditor for Edit/Create modes)
    if is_readonly {
        let labels_value = issue
            .as_ref()
            .map(|i| i.labels.join(", "))
            .unwrap_or_default();
        let display_value = if labels_value.is_empty() {
            "No labels"
        } else {
            &labels_value
        };
        fields.push(FormField::read_only("labels", "Labels", display_value));
    } else {
        // Use LabelEditor with colon-separated format for Edit/Create modes
        let labels_colon = issue
            .as_ref()
            .map(|i| i.labels.join(":"))
            .unwrap_or_default();
        let mut label_field = FormField::label_editor("labels", "Labels");
        label_field.value = labels_colon;
        fields.push(label_field);
    }

    // ========================================
    // SECTION 2: DETAILS
    // ========================================
    fields.push(FormField::section_header("section_details", "[DETAILS]"));

    // Description
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

    // Notes (Read-only)
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

    // ========================================
    // SECTION 3: RELATIONSHIPS
    // ========================================
    fields.push(FormField::section_header("section_relationships", "[RELATIONSHIPS]"));

    // Dependencies
    if let Some(issue) = issue {
        let deps_str = if issue.dependencies.is_empty() {
            "(none)".to_string()
        } else {
            issue.dependencies.join(", ")
        };
        fields.push(FormField::read_only("dependencies", "Depends On", &deps_str));
    }

    // Blocks
    if let Some(issue) = issue {
        let blocks_str = if issue.blocks.is_empty() {
            "(none)".to_string()
        } else {
            issue.blocks.join(", ")
        };
        fields.push(FormField::read_only("blocks", "Blocks", &blocks_str));
    }

    // ========================================
    // SECTION 4: METADATA
    // ========================================
    if let Some(issue) = issue {
        fields.push(FormField::section_header("section_metadata", "[METADATA]"));

        let created_str = issue.created.format("%Y-%m-%d %H:%M").to_string();
        fields.push(FormField::read_only("created", "Created", &created_str));

        let updated_str = issue.updated.format("%Y-%m-%d %H:%M").to_string();
        fields.push(FormField::read_only("updated", "Updated", &updated_str));

        if let Some(closed) = issue.closed {
            let closed_str = closed.format("%Y-%m-%d %H:%M").to_string();
            fields.push(FormField::read_only("closed", "Closed", &closed_str));
        }
    }

    fields
}
