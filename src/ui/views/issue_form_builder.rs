//! Unified form builder for Issue views
//!
//! Provides consistent form layout across Add, Edit, Read, and Split views

use crate::beads::models::Issue;
use crate::ui::widgets::{FormField, ValidationRule};
use ratatui::layout::Constraint;

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
#[must_use]
pub fn build_issue_form(mode: IssueFormMode, issue: Option<&Issue>) -> Vec<FormField> {
    let mut fields = Vec::new();
    let is_readonly = mode == IssueFormMode::Read;

    add_core_fields_simple(&mut fields, issue, is_readonly);
    add_description_field_simple(&mut fields, issue, is_readonly);
    add_date_estimate_fields_simple(&mut fields, issue, is_readonly);
    add_extended_text_fields_simple(&mut fields, issue, is_readonly);
    add_relationships_simple(&mut fields, issue);
    add_notes_simple(&mut fields, issue);

    fields
}

/// Helper function to add core fields (ID, Title, Status, Priority, Type, Assignee, Labels)
fn add_core_fields_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    // Helper to get value or empty string
    let get_value = |issue: Option<&Issue>, getter: fn(&Issue) -> String| -> String {
        issue.map(getter).unwrap_or_default()
    };

    // ========== ID (Read-only) ==========
    if let Some(issue) = issue {
        fields.push(FormField::read_only("id", "ID", &issue.id));
    }

    // ========== Title ==========
    let title_value = get_value(issue, |i| i.title.clone());
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
        .map_or_else(|| "Open".to_string(), |i| format!("{:?}", i.status));

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
        .map_or_else(|| "P2".to_string(), |i| format!("{}", i.priority));

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
        .map_or_else(|| "Task".to_string(), |i| format!("{:?}", i.issue_type));

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
        .map_or("", String::as_str);

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
}

/// Helper function to add description field
fn add_description_field_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    let description_value = issue
        .as_ref()
        .and_then(|i| i.description.as_ref())
        .map_or("", String::as_str);

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
}

/// Helper function to add date and estimate fields
fn add_date_estimate_fields_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
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
        .map(ToString::to_string)
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
        .map_or("", String::as_str);

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
        .map_or("", String::as_str);

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
}

/// Helper function to add extended text fields (Design Notes, Acceptance Criteria)
fn add_extended_text_fields_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    // ========== Design Notes ==========
    let design_notes_value = issue
        .as_ref()
        .and_then(|i| i.design_notes.as_ref())
        .map_or("", String::as_str);

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
        .map_or("", String::as_str);

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
}

/// Helper function to add relationship fields
fn add_relationships_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>) {
    // ========== Parent ID ==========
    let parent_id_value = issue
        .as_ref()
        .and_then(|i| i.parent_id.as_ref())
        .map_or("", String::as_str);

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
}

/// Helper function to add notes field
fn add_notes_simple(fields: &mut Vec<FormField>, issue: Option<&Issue>) {
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
}

/// Helper function to add header section fields
fn add_header_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    fields.push(FormField::section_header("section_header", "(Header Section)"));
    add_header_basic_fields(fields, issue, is_readonly);
    add_header_dates_fields(fields, issue, is_readonly);
}

/// Helper function to add basic header fields (ID, Title, Status, Type, Priority, Assignee)
fn add_header_basic_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    // Helper to get value or empty string
    let get_value = |issue: Option<&Issue>, getter: fn(&Issue) -> String| -> String {
        issue.map(getter).unwrap_or_default()
    };

    // ID (Read-only, only in Read/Edit modes)
    if let Some(issue) = issue {
        fields.push(FormField::read_only("id", "ID#", &issue.id));
    }

    // Title
    let title_value = get_value(issue, |i| i.title.clone());
    if is_readonly {
        fields.push(FormField::read_only("title", "TITLE", &title_value));
    } else {
        fields.push(
            FormField::text("title", "TITLE")
                .value(&title_value)
                .placeholder("Brief description of the issue")
                .required()
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::MaxLength(256)),
        );
    }

    // Status (Selector) - Row 1
    let status_value = issue
        .as_ref()
        .map_or_else(|| "Open".to_string(), |i| format!("{:?}", i.status));

    if is_readonly {
        fields.push(
            FormField::read_only("status", "STATUS", &status_value)
                .in_horizontal_group("header_row1", Constraint::Percentage(25))
        );
    } else {
        fields.push(
            FormField::selector(
                "status",
                "STATUS",
                vec![
                    "Open".to_string(),
                    "InProgress".to_string(),
                    "Blocked".to_string(),
                    "Closed".to_string(),
                ],
            )
            .value(&status_value)
            .required()
            .in_horizontal_group("header_row1", Constraint::Percentage(25)),
        );
    }

    // Type (Selector) - Row 1
    let type_value = issue
        .as_ref()
        .map_or_else(|| "Task".to_string(), |i| format!("{:?}", i.issue_type));

    if is_readonly {
        fields.push(
            FormField::read_only("type", "TYPE", &type_value)
                .in_horizontal_group("header_row1", Constraint::Percentage(25))
        );
    } else {
        fields.push(
            FormField::selector(
                "type",
                "TYPE",
                vec![
                    "Epic".to_string(),
                    "Feature".to_string(),
                    "Task".to_string(),
                    "Bug".to_string(),
                    "Chore".to_string(),
                ],
            )
            .value(&type_value)
            .required()
            .in_horizontal_group("header_row1", Constraint::Percentage(25)),
        );
    }

    // Priority (Selector) - Row 1
    let priority_value = issue
        .as_ref()
        .map_or_else(|| "P2".to_string(), |i| format!("{}", i.priority));

    if is_readonly {
        fields.push(
            FormField::read_only("priority", "PRIORITY", &priority_value)
                .in_horizontal_group("header_row1", Constraint::Percentage(25))
        );
    } else {
        fields.push(
            FormField::selector(
                "priority",
                "PRIORITY",
                vec![
                    "P0".to_string(),
                    "P1".to_string(),
                    "P2".to_string(),
                    "P3".to_string(),
                    "P4".to_string(),
                ],
            )
            .value(&priority_value)
            .required()
            .in_horizontal_group("header_row1", Constraint::Percentage(25)),
        );
    }

    // Assignee - Row 1
    let assignee_value = issue
        .as_ref()
        .and_then(|i| i.assignee.as_ref())
        .map_or("", String::as_str);

    if is_readonly {
        let display_value = if assignee_value.is_empty() {
            "Unassigned"
        } else {
            assignee_value
        };
        fields.push(
            FormField::read_only("assignee", "ASSIGNEE", display_value)
                .in_horizontal_group("header_row1", Constraint::Percentage(25))
        );
    } else {
        fields.push(
            FormField::text("assignee", "ASSIGNEE")
                .value(assignee_value)
                .placeholder("Unassigned")
                .in_horizontal_group("header_row1", Constraint::Percentage(25)),
        );
    }
}

/// Helper function to add header date fields
fn add_header_dates_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    // Est Minutes - Row 2
    let est_minutes_value = issue
        .as_ref()
        .and_then(|i| i.est_minutes.as_ref())
        .map(ToString::to_string)
        .unwrap_or_default();

    if is_readonly {
        let display_value = if est_minutes_value.is_empty() {
            "Not estimated"
        } else {
            &est_minutes_value
        };
        fields.push(
            FormField::read_only("est_minutes", "EST-MINUTES", display_value)
                .in_horizontal_group("header_row2", Constraint::Length(15))
        );
    } else {
        fields.push(
            FormField::text("est_minutes", "EST-MINUTES")
                .value(&est_minutes_value)
                .placeholder("Minutes")
                .in_horizontal_group("header_row2", Constraint::Length(15)),
        );
    }

    // Dates - Row 2
    if let Some(issue) = issue {
        let created_str = issue.created.format("%Y-%m-%d %H:%M").to_string();
        fields.push(
            FormField::read_only("created", "CREATED", &created_str)
                .in_horizontal_group("header_row2", Constraint::Length(20))
        );

        let updated_str = issue.updated.format("%Y-%m-%d %H:%M").to_string();
        fields.push(
            FormField::read_only("updated", "UPDATED", &updated_str)
                .in_horizontal_group("header_row2", Constraint::Length(20))
        );
    }

    // Due Date - Row 2
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
        fields.push(
            FormField::read_only("due_date", "DUE", display_value)
                .in_horizontal_group("header_row2", Constraint::Length(20))
        );
    } else {
        fields.push(
            FormField::text("due_date", "DUE")
                .value(&due_date_value)
                .placeholder("YYYY-MM-DD HH:MM")
                .in_horizontal_group("header_row2", Constraint::Length(20)),
        );
    }

    // Defer Date - Row 2
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
        fields.push(
            FormField::read_only("defer_date", "DEFER", display_value)
                .in_horizontal_group("header_row2", Constraint::Length(20))
        );
    } else {
        fields.push(
            FormField::text("defer_date", "DEFER")
                .value(&defer_date_value)
                .placeholder("YYYY-MM-DD HH:MM")
                .in_horizontal_group("header_row2", Constraint::Length(20)),
        );
    }

    // Closed Date - Row 2
    if let Some(issue) = issue {
        if let Some(closed) = issue.closed {
            let closed_str = closed.format("%Y-%m-%d %H:%M").to_string();
            fields.push(
                FormField::read_only("closed", "CLOSED", &closed_str)
                    .in_horizontal_group("header_row2", Constraint::Length(20))
            );
        }
    }

    // Close Reason - Full Width
    let close_reason_value = issue
        .as_ref()
        .and_then(|i| i.close_reason.as_ref())
        .map_or("", String::as_str);

    if is_readonly {
        let display_value = if close_reason_value.is_empty() {
            "N/A"
        } else {
            close_reason_value
        };
        fields.push(
            FormField::read_only("close_reason", "CLOSE-REASON", display_value)
                .full_width()
        );
    } else {
        fields.push(
            FormField::text("close_reason", "CLOSE-REASON")
                .value(close_reason_value)
                .placeholder("Reason for closing")
                .full_width(),
        );
    }

    // External Reference - Full Width
    let external_ref_value = issue
        .as_ref()
        .and_then(|i| i.external_reference.as_ref())
        .map_or("", String::as_str);

    if is_readonly {
        let display_value = if external_ref_value.is_empty() {
            "No reference"
        } else {
            external_ref_value
        };
        fields.push(
            FormField::read_only("external_reference", "EXTERNAL-REFERENCE", display_value)
                .full_width()
        );
    } else {
        fields.push(
            FormField::text("external_reference", "EXTERNAL-REFERENCE")
                .value(external_ref_value)
                .placeholder("External link or reference")
                .full_width(),
        );
    }
}

/// Helper function to add label and flags section fields
fn add_label_flags_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    fields.push(FormField::section_header("section_labels_flags", "(Label & Flags Section)"));

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
        fields.push(FormField::read_only("labels", "ADD-LABEL", display_value));
    } else {
        // Use LabelEditor with colon-separated format for Edit/Create modes
        let labels_colon = issue
            .as_ref()
            .map(|i| i.labels.join(":"))
            .unwrap_or_default();
        let mut label_field = FormField::label_editor("labels", "ADD-LABEL");
        label_field.value = labels_colon;
        fields.push(label_field);
    }

    // Flags
    let flags_value = issue
        .as_ref()
        .map_or_else(|| "None".to_string(), |i| {
            let mut flags = Vec::new();
            if i.flags.pinned { flags.push("Pinned"); }
            if i.flags.template { flags.push("Template"); }
            if i.flags.ephemeral { flags.push("Ephemeral"); }
            if flags.is_empty() {
                "None".to_string()
            } else {
                flags.join(", ")
            }
        });
    fields.push(FormField::read_only("flags", "FLAGS", &flags_value));
}

/// Helper function to add text section fields (description, design notes, acceptance criteria, notes)
fn add_text_section_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>, is_readonly: bool) {
    // ========================================
    // SECTION: Description Section
    // ========================================
    fields.push(FormField::section_header("section_description", "(Description Section)"));

    let description_value = issue
        .as_ref()
        .and_then(|i| i.description.as_ref())
        .map_or("", String::as_str);

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

    // ========================================
    // SECTION: Design Notes Section
    // ========================================
    fields.push(FormField::section_header("section_design_notes", "(Design Notes Section)"));

    let design_notes_value = issue
        .as_ref()
        .and_then(|i| i.design_notes.as_ref())
        .map_or("", String::as_str);

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

    // ========================================
    // SECTION: Acceptance Criteria Section
    // ========================================
    fields.push(FormField::section_header("section_acceptance_criteria", "(Acceptance Criteria Section)"));

    let acceptance_criteria_value = issue
        .as_ref()
        .and_then(|i| i.acceptance_criteria.as_ref())
        .map_or("", String::as_str);

    if is_readonly {
        let display_value = if acceptance_criteria_value.is_empty() {
            "No acceptance criteria"
        } else {
            acceptance_criteria_value
        };
        fields.push(FormField::read_only("acceptance_criteria", "Acceptance Criteria", display_value));
    } else {
        fields.push(
            FormField::text_area("acceptance_criteria", "Acceptance Criteria")
                .value(acceptance_criteria_value)
                .placeholder("Acceptance criteria"),
        );
    }

    // ========================================
    // SECTION: Notes Section
    // ========================================
    fields.push(FormField::section_header("section_notes", "(Notes Section)"));

    if let Some(issue) = issue {
        if issue.notes.is_empty() {
            fields.push(FormField::read_only("notes", "Notes", "No notes"));
        } else {
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
    // SECTION: Comments Section
    // ========================================
    fields.push(FormField::section_header("section_comments", "(Comments Section)"));

    // Comments are the same as notes in the beads model
    if let Some(issue) = issue {
        if issue.notes.is_empty() {
            fields.push(FormField::read_only("comments", "ADD-COMMENT", "No comments"));
        } else {
            for (idx, note) in issue.notes.iter().enumerate() {
                let comment_header = format!("M#{}-ID: {}, M#{}-AUTHOR: {}, M#{}-DATE: {}",
                    idx + 1, note.id,
                    idx + 1, note.author,
                    idx + 1, note.timestamp.format("%Y-%m-%d %H:%M"));
                fields.push(FormField::read_only(
                    &format!("comment_{idx}_header"),
                    &format!("COMMENT#{}", idx + 1),
                    &comment_header
                ));
                fields.push(FormField::read_only(
                    &format!("comment_{idx}"),
                    &format!("COMMENT#{}", idx + 1),
                    &note.content
                ));
            }
        }
    }
}

/// Helper function to add relationship section fields (parent, children, blocks, events, discovered)
fn add_relationship_fields(fields: &mut Vec<FormField>, issue: Option<&Issue>) {
    // ========================================
    // SECTION: Parent Section
    // ========================================
    fields.push(FormField::section_header("section_parent", "(Parent Section)"));

    let parent_id_value = issue
        .as_ref()
        .and_then(|i| i.parent_id.as_ref())
        .map_or("", String::as_str);

    let display_value = if parent_id_value.is_empty() {
        "No parent"
    } else {
        parent_id_value
    };
    fields.push(FormField::read_only("parent_id", "PARENT-ID#", display_value));

    // ========================================
    // SECTION: Children Section
    // ========================================
    fields.push(FormField::section_header("section_children", "(Children Section)"));

    if let Some(issue) = issue {
        if issue.children_ids.is_empty() {
            fields.push(FormField::read_only("children", "ADD-CHILD", "No children"));
        } else {
            for (idx, child_id) in issue.children_ids.iter().enumerate() {
                fields.push(FormField::read_only(
                    &format!("child_{idx}"),
                    &format!("C#{}-ID", idx + 1),
                    child_id
                ));
            }
        }
    }

    // ========================================
    // SECTION: Block Section
    // ========================================
    fields.push(FormField::section_header("section_blocks", "(Block Section)"));

    if let Some(issue) = issue {
        if issue.blocks.is_empty() {
            fields.push(FormField::read_only("blocks", "ADD-BLOCK", "No blocks"));
        } else {
            for (idx, block_id) in issue.blocks.iter().enumerate() {
                fields.push(FormField::read_only(
                    &format!("block_{idx}"),
                    &format!("B#{}-ID", idx + 1),
                    block_id
                ));
            }
        }
    }

    // ========================================
    // SECTION: Event Section
    // ========================================
    fields.push(FormField::section_header("section_events", "(Event Section)"));

    if let Some(issue) = issue {
        if issue.event_ids.is_empty() {
            fields.push(FormField::read_only("events", "ADD-EVENT", "No events"));
        } else {
            for (idx, event_id) in issue.event_ids.iter().enumerate() {
                fields.push(FormField::read_only(
                    &format!("event_{idx}"),
                    &format!("E#{}-ID", idx + 1),
                    event_id
                ));
            }
        }
    }

    // ========================================
    // SECTION: Dependencies Section
    // ========================================
    fields.push(FormField::section_header("section_dependencies", "(Dependencies Section)"));

    if let Some(issue) = issue {
        if issue.dependencies.is_empty() {
            fields.push(FormField::read_only("dependencies", "DEPENDS-ON", "No dependencies"));
        } else {
            for (idx, dep_id) in issue.dependencies.iter().enumerate() {
                fields.push(FormField::read_only(
                    &format!("dependency_{idx}"),
                    &format!("DEP#{}-ID", idx + 1),
                    dep_id
                ));
            }
        }
    }

    // ========================================
    // SECTION: Discovered Section
    // ========================================
    fields.push(FormField::section_header("section_discovered", "(Discovered Section)"));

    if let Some(issue) = issue {
        if issue.discovered_ids.is_empty() {
            fields.push(FormField::read_only("discovered", "ADD-DISCOVERY", "No discovered items"));
        } else {
            for (idx, discovered_id) in issue.discovered_ids.iter().enumerate() {
                fields.push(FormField::read_only(
                    &format!("discovered_{idx}"),
                    &format!("D#{}-ID", idx + 1),
                    discovered_id
                ));
            }
        }
    }
}

/// Build an issue form with organized sections matching the Record Detail Form layout
///
/// Organizes fields into the following sections:
/// - Header Section: ID, TITLE, STATUS, TYPE, PRIORITY, ASSIGNEE, dates, etc.
/// - Label & Flags Section: Labels and boolean flags
/// - Description Section
/// - Design Notes Section
/// - Acceptance Criteria Section
/// - Notes/Comments Section
/// - Parent Section
/// - Children Section
/// - Block Section
/// - Dependencies Section
/// - Event Section
/// - Discovered Section
#[must_use]
pub fn build_issue_form_with_sections(mode: IssueFormMode, issue: Option<&Issue>) -> Vec<FormField> {
    let mut fields = Vec::new();
    let is_readonly = mode == IssueFormMode::Read;

    // Add all sections using helper functions
    add_header_fields(&mut fields, issue, is_readonly);
    add_label_flags_fields(&mut fields, issue, is_readonly);
    add_text_section_fields(&mut fields, issue, is_readonly);
    add_relationship_fields(&mut fields, issue);

    // ========================================
    // SECTION: Record Actions Section
    // ========================================
    fields.push(FormField::section_header("section_actions", "(Record Actions Section)"));
    fields.push(FormField::read_only(
        "actions",
        "Actions",
        "Ctrl+S:Save | Ctrl+X:Cancel | Ctrl+Del:Soft Delete | Ctrl+J:Copy JSON to Clipboard | CTRL+P:Export to Markdown"
    ));

    fields
}
