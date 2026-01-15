//! Create issue form view with section navigator

use crate::beads::models::{IssueType, Priority};
use crate::models::{split_labels, validate_labels};
use crate::ui::views::issue_form_builder::{build_issue_form_with_sections, IssueFormMode};
use crate::ui::widgets::{Form, FormField, FormState, ValidationRule};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Form sections for the create issue form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormSection {
    Summary,
    Scheduling,
    Relationships,
    Labels,
    Text,
    Metadata,
}

impl FormSection {
    /// Get all sections in order
    pub fn all() -> Vec<Self> {
        vec![
            Self::Summary,
            Self::Scheduling,
            Self::Relationships,
            Self::Labels,
            Self::Text,
            Self::Metadata,
        ]
    }

    /// Get the display name for the section
    pub fn display_name(&self) -> &str {
        match self {
            Self::Summary => "Summary",
            Self::Scheduling => "Scheduling",
            Self::Relationships => "Relationships",
            Self::Labels => "Labels",
            Self::Text => "Text",
            Self::Metadata => "Metadata",
        }
    }

    /// Get the description for the section
    pub fn description(&self) -> &str {
        match self {
            Self::Summary => "Title, type, priority, status",
            Self::Scheduling => "Due date, defer date, time estimate",
            Self::Relationships => "Parent issue, dependencies",
            Self::Labels => "Tags and categories",
            Self::Text => "Description, design, acceptance criteria, notes",
            Self::Metadata => "Read-only system information",
        }
    }

    /// Check if section has required fields
    pub fn has_required_fields(&self) -> bool {
        matches!(self, Self::Summary)
    }
}

/// Create issue form state
#[derive(Debug)]
pub struct CreateIssueFormState {
    form_state: FormState,
    current_section: FormSection,
    show_preview: bool,
}

impl Default for CreateIssueFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateIssueFormState {
    /// Get git user.name from git config as default assignee
    fn get_git_user_name() -> Result<String, String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["config", "--get", "user.name"])
            .output()
            .map_err(|e| format!("Failed to run git config: {}", e))?;

        if output.status.success() {
            let user_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if user_name.is_empty() {
                return Err("git user.name is empty".to_string());
            }

            // Validate username contains only safe characters
            if !user_name
                .chars()
                .all(|c| c.is_alphanumeric() || " .-_@".contains(c))
            {
                return Err(format!(
                    "git user.name contains invalid characters: {}",
                    user_name
                ));
            }

            Ok(user_name)
        } else {
            Err("git config failed".to_string())
        }
    }

    /// Update field visibility based on current section
    fn update_field_visibility(&mut self) {
        let visible_fields: Vec<String> = self
            .current_section_fields()
            .iter()
            .map(|s| s.to_string())
            .collect();
        for field in self.form_state.fields_mut() {
            field.hidden = !visible_fields.contains(&field.id);
        }

        // Reset focus to first visible field of the section
        if let Some(idx) = self.form_state.fields().iter().position(|f| !f.hidden) {
            self.form_state.set_focused_index(idx);
        }
    }

    /// Create a new create issue form state
    pub fn new() -> Self {
        // Start with sectioned form fields for core issue fields (Create mode)
        let mut fields = build_issue_form_with_sections(IssueFormMode::Create, None);

        // Remove the read-only fields that don't make sense in create mode
        fields.retain(|f| !matches!(f.id.as_str(), "id" | "created" | "updated" | "closed"));

        // Add create-specific fields that aren't in the base Issue model
        // Scheduling section
        fields.push(FormField::text("due_date", "Due Date")
            .placeholder("YYYY-MM-DD (optional)")
            .help_text("Press F1: Enter a date in YYYY-MM-DD format (e.g., 2024-12-31). This is when the issue should be completed.")
            .with_validation(ValidationRule::MaxLength(32)));

        fields.push(FormField::text("defer_date", "Defer Date")
            .placeholder("YYYY-MM-DD (optional)")
            .help_text("Press F1: Enter a date in YYYY-MM-DD format. Issue will be hidden from ready list until this date.")
            .with_validation(ValidationRule::MaxLength(32)));

        fields.push(FormField::text("time_estimate", "Time Estimate")
            .placeholder("e.g., 2h, 3d, 1w (optional)")
            .help_text("Press F1: Estimate format: 2h (2 hours), 3d (3 days), 1w (1 week). Used for planning and tracking.")
            .with_validation(ValidationRule::MaxLength(32)));

        // Relationships section (parent is create-specific)
        fields.push(FormField::text("parent", "Parent Issue")
            .placeholder("beads-xxx (optional)")
            .help_text("Press F1: Enter parent issue ID (e.g., beads-abc123). This issue will be part of the parent epic/feature.")
            .with_validation(ValidationRule::BeadsIdFormat)
            .with_validation(ValidationRule::MaxLength(64)));

        // Text section (design, acceptance, notes are create-specific)
        fields.push(FormField::text_area("design", "Design")
            .placeholder("Design notes and approach (optional)")
            .with_validation(ValidationRule::MaxLength(1048576)));

        fields.push(FormField::text_area("acceptance", "Acceptance Criteria")
            .placeholder("How to verify this is done (optional)")
            .with_validation(ValidationRule::MaxLength(1048576)));

        fields.push(FormField::text_area("notes", "Notes")
            .placeholder("Additional notes (optional)")
            .with_validation(ValidationRule::MaxLength(1048576)));

        // Try to get default assignee from git config and update the assignee field
        if let Ok(git_user) = Self::get_git_user_name() {
            if let Some(assignee_field) = fields.iter_mut().find(|f| f.id == "assignee") {
                assignee_field.value = git_user;
            }
        }

        let mut state = Self {
            form_state: FormState::new(fields),
            current_section: FormSection::Summary,
            show_preview: false,
        };
        state.update_field_visibility();
        state
    }

    /// Get the current section
    pub fn current_section(&self) -> FormSection {
        self.current_section
    }

    /// Set the current section
    pub fn set_section(&mut self, section: FormSection) {
        self.current_section = section;
        self.update_field_visibility();
    }

    /// Navigate to the next section
    pub fn next_section(&mut self) {
        let sections = FormSection::all();
        let current_idx = sections
            .iter()
            .position(|s| *s == self.current_section)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % sections.len();
        self.current_section = sections[next_idx];
        self.update_field_visibility();
    }

    /// Navigate to the previous section
    pub fn prev_section(&mut self) {
        let sections = FormSection::all();
        let current_idx = sections
            .iter()
            .position(|s| *s == self.current_section)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            sections.len() - 1
        } else {
            current_idx - 1
        };
        self.current_section = sections[prev_idx];
        self.update_field_visibility();
    }

    /// Toggle preview mode
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    /// Check if in preview mode
    pub fn is_preview_mode(&self) -> bool {
        self.show_preview
    }

    /// Get fields for the current section
    pub fn current_section_fields(&self) -> Vec<&'static str> {
        match self.current_section {
            FormSection::Summary => vec!["title", "type", "priority", "status"],
            FormSection::Scheduling => vec!["due_date", "defer_date", "time_estimate"],
            FormSection::Relationships => vec!["parent", "dependencies"],
            FormSection::Labels => vec!["assignee", "labels"],
            FormSection::Text => vec!["description", "design", "acceptance", "notes"],
            FormSection::Metadata => vec![], // Read-only, shown in preview
        }
    }

    /// Check if current section is complete (all required fields filled)
    pub fn is_section_complete(&self, section: FormSection) -> bool {
        match section {
            FormSection::Summary => {
                // Title is required
                self.form_state
                    .get_value("title")
                    .filter(|s| !s.is_empty())
                    .is_some()
            }
            _ => true, // Other sections have no required fields
        }
    }

    /// Get the underlying form state
    pub fn form_state(&self) -> &FormState {
        &self.form_state
    }

    /// Get mutable reference to the underlying form state
    pub fn form_state_mut(&mut self) -> &mut FormState {
        &mut self.form_state
    }

    /// Validate the form
    pub fn validate(&mut self) -> bool {
        let mut is_valid = self.form_state.validate();

        if let Some(labels_field) = self.form_state.get_field_mut("labels") {
            let labels = split_labels(&labels_field.value);
            match validate_labels(&labels) {
                Ok(()) => labels_field.error = None,
                Err(error) => {
                    labels_field.error = Some(error);
                    is_valid = false;
                }
            }
        }

        is_valid
    }

    /// Get the form data as a tuple
    pub fn get_data(&self) -> Option<CreateIssueData> {
        if self.form_state.has_errors() {
            return None;
        }

        let title = self.form_state.get_value("title")?.to_string();
        let issue_type = self.parse_issue_type(self.form_state.get_value("type")?)?;
        let priority = self.parse_priority(self.form_state.get_value("priority")?)?;
        let status_str = self.form_state.get_value("status")?;
        let assignee = self
            .form_state
            .get_value("assignee")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let labels_str = self
            .form_state
            .get_value("labels")
            .filter(|s| !s.is_empty())
            .unwrap_or("");
        let labels = split_labels(labels_str);
        let description = self
            .form_state
            .get_value("description")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // New fields
        let due_date = self
            .form_state
            .get_value("due_date")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let defer_date = self
            .form_state
            .get_value("defer_date")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let time_estimate = self
            .form_state
            .get_value("time_estimate")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let parent = self
            .form_state
            .get_value("parent")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let dependencies_str = self
            .form_state
            .get_value("dependencies")
            .filter(|s| !s.is_empty())
            .unwrap_or("");
        let dependencies: Vec<String> = dependencies_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let design = self
            .form_state
            .get_value("design")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let acceptance = self
            .form_state
            .get_value("acceptance")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let notes = self
            .form_state
            .get_value("notes")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Some(CreateIssueData {
            title,
            issue_type,
            priority,
            status: status_str.to_string(),
            assignee,
            labels,
            description,
            due_date,
            defer_date,
            time_estimate,
            parent,
            dependencies,
            design,
            acceptance,
            notes,
        })
    }

    fn parse_issue_type(&self, type_str: &str) -> Option<IssueType> {
        match type_str {
            "Epic" => Some(IssueType::Epic),
            "Feature" => Some(IssueType::Feature),
            "Task" => Some(IssueType::Task),
            "Bug" => Some(IssueType::Bug),
            "Chore" => Some(IssueType::Chore),
            _ => None,
        }
    }

    fn parse_priority(&self, priority_str: &str) -> Option<Priority> {
        if priority_str.starts_with("P0") {
            Some(Priority::P0)
        } else if priority_str.starts_with("P1") {
            Some(Priority::P1)
        } else if priority_str.starts_with("P2") {
            Some(Priority::P2)
        } else if priority_str.starts_with("P3") {
            Some(Priority::P3)
        } else if priority_str.starts_with("P4") {
            Some(Priority::P4)
        } else {
            None
        }
    }

    /// Clear the form
    pub fn clear(&mut self) {
        self.form_state = FormState::new(vec![
            // Summary section
            FormField::text("title", "Title")
                .required()
                .placeholder("Brief description of the issue")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::MaxLength(256)),
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
            .value("Task")
            .required(),
            FormField::selector(
                "priority",
                "Priority",
                vec![
                    "P0 (Critical)".to_string(),
                    "P1 (High)".to_string(),
                    "P2 (Medium)".to_string(),
                    "P3 (Low)".to_string(),
                    "P4 (Backlog)".to_string(),
                ],
            )
            .value("P2 (Medium)")
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
            .value("Open")
            .required(),
            // Scheduling section
            FormField::text("due_date", "Due Date")
                .placeholder("YYYY-MM-DD (optional)")
                .help_text("Press F1: Enter a date in YYYY-MM-DD format (e.g., 2024-12-31). This is when the issue should be completed.")
                .with_validation(ValidationRule::MaxLength(32)),
            FormField::text("defer_date", "Defer Date")
                .placeholder("YYYY-MM-DD (optional)")
                .help_text("Press F1: Enter a date in YYYY-MM-DD format. Issue will be hidden from ready list until this date.")
                .with_validation(ValidationRule::MaxLength(32)),
            FormField::text("time_estimate", "Time Estimate")
                .placeholder("e.g., 2h, 3d, 1w (optional)")
                .help_text("Press F1: Estimate format: 2h (2 hours), 3d (3 days), 1w (1 week). Used for planning and tracking.")
                .with_validation(ValidationRule::MaxLength(32)),
            // Relationships section
            FormField::text("parent", "Parent Issue")
                .placeholder("beads-xxx (optional)")
                .help_text("Press F1: Enter parent issue ID (e.g., beads-abc123). This issue will be part of the parent epic/feature.")
                .with_validation(ValidationRule::BeadsIdFormat)
                .with_validation(ValidationRule::MaxLength(64)),
            FormField::text("dependencies", "Dependencies")
                .placeholder("comma-separated beads-xxx (optional)")
                .help_text("Press F1: Enter comma-separated issue IDs that this issue depends on. This issue will be blocked until dependencies are resolved.")
                .with_validation(ValidationRule::MaxLength(2048)),
            // Labels section
            {
                let mut assignee_field = FormField::text("assignee", "Assignee")
                    .placeholder("username (optional)")
                    .with_validation(ValidationRule::MaxLength(128));

                // Try to get default assignee from git config
                if let Ok(git_user) = Self::get_git_user_name() {
                    assignee_field = assignee_field.value(&git_user);
                }
                assignee_field
            },
            FormField::text("labels", "Labels")
                .placeholder("comma-separated labels (optional)")
                .with_validation(ValidationRule::MaxLength(2048)),
            // Text section
            FormField::text_area("description", "Description")
                .placeholder("Detailed description of the issue (optional)")
                .with_validation(ValidationRule::MaxLength(1048576)),
            FormField::text_area("design", "Design")
                .placeholder("Design notes and approach (optional)")
                .with_validation(ValidationRule::MaxLength(1048576)),
            FormField::text_area("acceptance", "Acceptance Criteria")
                .placeholder("How to verify this is done (optional)")
                .with_validation(ValidationRule::MaxLength(1048576)),
            FormField::text_area("notes", "Notes")
                .placeholder("Additional notes (optional)")
                .with_validation(ValidationRule::MaxLength(1048576)),
        ]);
        self.current_section = FormSection::Summary;
        self.show_preview = false;
        self.update_field_visibility();
    }
}

/// Data extracted from the create issue form
#[derive(Debug, Clone)]
pub struct CreateIssueData {
    pub title: String,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub status: String,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub defer_date: Option<String>,
    pub time_estimate: Option<String>,
    pub parent: Option<String>,
    pub dependencies: Vec<String>,
    pub design: Option<String>,
    pub acceptance: Option<String>,
    pub notes: Option<String>,
}

/// Create issue form view
pub struct CreateIssueForm<'a> {
    title: &'a str,
    show_help: bool,
}

impl<'a> CreateIssueForm<'a> {
    /// Create a new create issue form view
    pub fn new() -> Self {
        Self {
            title: "Create New Issue",
            show_help: true,
        }
    }

    /// Set the form title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Show or hide help text
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    /// Render the preview panel showing formatted issue data
    fn render_preview(&self, area: Rect, buf: &mut Buffer, state: &CreateIssueFormState) {
        // Create main layout with help text at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20),   // Preview area
                Constraint::Length(3), // Help text
            ])
            .split(area);

        // Build preview content
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Issue Preview",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        // Get form values
        let title = state.form_state().get_value("title").unwrap_or("");
        let issue_type = state.form_state().get_value("type").unwrap_or("");
        let priority = state.form_state().get_value("priority").unwrap_or("");
        let status = state.form_state().get_value("status").unwrap_or("");
        let assignee = state.form_state().get_value("assignee").unwrap_or("");
        let labels = state.form_state().get_value("labels").unwrap_or("");
        let description = state.form_state().get_value("description").unwrap_or("");
        let due_date = state.form_state().get_value("due_date").unwrap_or("");
        let defer_date = state.form_state().get_value("defer_date").unwrap_or("");
        let time_estimate = state.form_state().get_value("time_estimate").unwrap_or("");
        let parent = state.form_state().get_value("parent").unwrap_or("");
        let dependencies = state.form_state().get_value("dependencies").unwrap_or("");
        let design = state.form_state().get_value("design").unwrap_or("");
        let acceptance = state.form_state().get_value("acceptance").unwrap_or("");
        let notes = state.form_state().get_value("notes").unwrap_or("");

        // Summary section
        lines.push(Line::from(vec![Span::styled(
            "Summary",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        if !title.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Title: ", Style::default().fg(Color::Gray)),
                Span::raw(title),
            ]));
        }
        if !issue_type.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Type: ", Style::default().fg(Color::Gray)),
                Span::styled(issue_type, Style::default().fg(Color::Cyan)),
            ]));
        }
        if !priority.is_empty() {
            let priority_color = if priority.contains("P0") || priority.contains("P1") {
                Color::Red
            } else if priority.contains("P2") {
                Color::Yellow
            } else {
                Color::Gray
            };
            lines.push(Line::from(vec![
                Span::styled("  Priority: ", Style::default().fg(Color::Gray)),
                Span::styled(priority, Style::default().fg(priority_color)),
            ]));
        }
        if !status.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::Gray)),
                Span::styled(status, Style::default().fg(Color::Green)),
            ]));
        }
        lines.push(Line::from(""));

        // Scheduling section
        if !due_date.is_empty() || !defer_date.is_empty() || !time_estimate.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Scheduling",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            if !due_date.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Due Date: ", Style::default().fg(Color::Gray)),
                    Span::raw(due_date),
                ]));
            }
            if !defer_date.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Defer Date: ", Style::default().fg(Color::Gray)),
                    Span::raw(defer_date),
                ]));
            }
            if !time_estimate.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Time Estimate: ", Style::default().fg(Color::Gray)),
                    Span::raw(time_estimate),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Relationships section
        if !parent.is_empty() || !dependencies.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Relationships",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            if !parent.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Parent: ", Style::default().fg(Color::Gray)),
                    Span::styled(parent, Style::default().fg(Color::Magenta)),
                ]));
            }
            if !dependencies.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Dependencies: ", Style::default().fg(Color::Gray)),
                    Span::styled(dependencies, Style::default().fg(Color::Magenta)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Labels section
        if !assignee.is_empty() || !labels.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Labels",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            if !assignee.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Assignee: ", Style::default().fg(Color::Gray)),
                    Span::raw(assignee),
                ]));
            }
            if !labels.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Labels: ", Style::default().fg(Color::Gray)),
                    Span::styled(labels, Style::default().fg(Color::Blue)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Text section
        if !description.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Description",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            for line in description.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
            lines.push(Line::from(""));
        }
        if !design.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Design",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            for line in design.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
            lines.push(Line::from(""));
        }
        if !acceptance.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Acceptance Criteria",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            for line in acceptance.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
            lines.push(Line::from(""));
        }
        if !notes.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Notes",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            for line in notes.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
        }

        // Render preview panel
        let preview = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Issue Preview "),
            )
            .alignment(Alignment::Left);
        preview.render(chunks[0], buf);

        // Render help text
        let help_lines = vec![Line::from(vec![
            Span::styled("Ctrl+P", Style::default().fg(Color::Magenta)),
            Span::raw(" Back to Form  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Submit  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ])];

        let help = Paragraph::new(help_lines)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .alignment(Alignment::Center);

        help.render(chunks[1], buf);
    }
}

impl<'a> Default for CreateIssueForm<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for CreateIssueForm<'a> {
    type State = CreateIssueFormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Check if in preview mode
        if state.is_preview_mode() {
            self.render_preview(area, buf, state);
            return;
        }

        // Create main layout
        let chunks = if self.show_help {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(20),   // Form area
                    Constraint::Length(3), // Help text
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(20)])
                .split(area)
        };

        // Render form
        let form = Form::new()
            .title(self.title)
            .focused_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .error_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        StatefulWidget::render(form, chunks[0], buf, &mut state.form_state);

        // Render help text
        if self.show_help && chunks.len() > 1 {
            let help_lines = vec![Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Next field  "),
                Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Previous field  "),
                Span::styled("Ctrl+L", Style::default().fg(Color::Cyan)),
                Span::raw(" Load  "),
                Span::styled("Ctrl+P", Style::default().fg(Color::Magenta)),
                Span::raw(" Preview  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(" Submit  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ])];

            let help = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .alignment(Alignment::Center);

            help.render(chunks[1], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue_form_state_creation() {
        let state = CreateIssueFormState::new();
        assert_eq!(state.form_state().fields().len(), 15); // Updated for all fields
        assert_eq!(state.current_section(), FormSection::Summary);
        assert!(!state.is_preview_mode());
    }

    #[test]
    fn test_create_issue_form_validation() {
        let mut state = CreateIssueFormState::new();

        // Should fail validation with empty title
        assert!(!state.validate());

        // Add title and validate again
        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());
        assert!(state.validate());
    }

    #[test]
    fn test_create_issue_data_extraction() {
        let mut state = CreateIssueFormState::new();

        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());
        state
            .form_state_mut()
            .set_value("assignee", "john".to_string());
        state
            .form_state_mut()
            .set_value("labels", "bug, urgent".to_string());
        state
            .form_state_mut()
            .set_value("description", "This is a test".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.title, "Test Issue");
        assert_eq!(data.issue_type, IssueType::Task);
        assert_eq!(data.priority, Priority::P2);
        assert_eq!(data.assignee, Some("john".to_string()));
        assert_eq!(data.labels.len(), 2);
        assert!(data.labels.contains(&"bug".to_string()));
        assert!(data.labels.contains(&"urgent".to_string()));
        assert_eq!(data.description, Some("This is a test".to_string()));
    }

    #[test]
    fn test_parse_issue_type() {
        let state = CreateIssueFormState::new();

        assert_eq!(state.parse_issue_type("Epic"), Some(IssueType::Epic));
        assert_eq!(state.parse_issue_type("Feature"), Some(IssueType::Feature));
        assert_eq!(state.parse_issue_type("Task"), Some(IssueType::Task));
        assert_eq!(state.parse_issue_type("Bug"), Some(IssueType::Bug));
        assert_eq!(state.parse_issue_type("Chore"), Some(IssueType::Chore));
        assert_eq!(state.parse_issue_type("Invalid"), None);
    }

    #[test]
    fn test_parse_priority() {
        let state = CreateIssueFormState::new();

        assert_eq!(state.parse_priority("P0 (Critical)"), Some(Priority::P0));
        assert_eq!(state.parse_priority("P1 (High)"), Some(Priority::P1));
        assert_eq!(state.parse_priority("P2 (Medium)"), Some(Priority::P2));
        assert_eq!(state.parse_priority("P3 (Low)"), Some(Priority::P3));
        assert_eq!(state.parse_priority("P4 (Backlog)"), Some(Priority::P4));
        assert_eq!(state.parse_priority("Invalid"), None);
    }

    #[test]
    fn test_clear_form() {
        let mut state = CreateIssueFormState::new();

        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());
        state
            .form_state_mut()
            .set_value("assignee", "john".to_string());

        assert_eq!(state.form_state().get_value("title"), Some("Test Issue"));

        state.clear();

        assert_eq!(state.form_state().get_value("title"), Some(""));
        // Assignee may have git config default, so just check it's not "john" anymore
        assert_ne!(state.form_state().get_value("assignee"), Some("john"));
    }

    #[test]
    fn test_optional_fields() {
        let mut state = CreateIssueFormState::new();

        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        // Assignee may be populated from git config, so it could be Some or None
        // Just verify labels and description are empty as expected
        assert_eq!(data.labels.len(), 0);
        assert_eq!(data.description, None);
    }

    // FormSection tests
    #[test]
    fn test_form_section_all() {
        let sections = FormSection::all();
        assert_eq!(sections.len(), 6);
        assert_eq!(sections[0], FormSection::Summary);
        assert_eq!(sections[1], FormSection::Scheduling);
        assert_eq!(sections[2], FormSection::Relationships);
        assert_eq!(sections[3], FormSection::Labels);
        assert_eq!(sections[4], FormSection::Text);
        assert_eq!(sections[5], FormSection::Metadata);
    }

    #[test]
    fn test_form_section_display_name() {
        assert_eq!(FormSection::Summary.display_name(), "Summary");
        assert_eq!(FormSection::Scheduling.display_name(), "Scheduling");
        assert_eq!(FormSection::Relationships.display_name(), "Relationships");
        assert_eq!(FormSection::Labels.display_name(), "Labels");
        assert_eq!(FormSection::Text.display_name(), "Text");
        assert_eq!(FormSection::Metadata.display_name(), "Metadata");
    }

    #[test]
    fn test_form_section_description() {
        assert_eq!(
            FormSection::Summary.description(),
            "Title, type, priority, status"
        );
        assert_eq!(
            FormSection::Scheduling.description(),
            "Due date, defer date, time estimate"
        );
        assert_eq!(
            FormSection::Relationships.description(),
            "Parent issue, dependencies"
        );
        assert_eq!(FormSection::Labels.description(), "Tags and categories");
        assert_eq!(
            FormSection::Text.description(),
            "Description, design, acceptance criteria, notes"
        );
        assert_eq!(
            FormSection::Metadata.description(),
            "Read-only system information"
        );
    }

    #[test]
    fn test_form_section_has_required_fields() {
        assert!(FormSection::Summary.has_required_fields());
        assert!(!FormSection::Scheduling.has_required_fields());
        assert!(!FormSection::Relationships.has_required_fields());
        assert!(!FormSection::Labels.has_required_fields());
        assert!(!FormSection::Text.has_required_fields());
        assert!(!FormSection::Metadata.has_required_fields());
    }

    #[test]
    fn test_form_section_equality() {
        assert_eq!(FormSection::Summary, FormSection::Summary);
        assert_ne!(FormSection::Summary, FormSection::Scheduling);
        assert_eq!(FormSection::Labels, FormSection::Labels);
    }

    // CreateIssueFormState tests
    #[test]
    fn test_create_issue_form_state_default() {
        let state = CreateIssueFormState::default();
        assert_eq!(state.current_section(), FormSection::Summary);
        assert!(!state.is_preview_mode());
    }

    #[test]
    fn test_set_section() {
        let mut state = CreateIssueFormState::new();
        assert_eq!(state.current_section(), FormSection::Summary);

        state.set_section(FormSection::Labels);
        assert_eq!(state.current_section(), FormSection::Labels);

        state.set_section(FormSection::Metadata);
        assert_eq!(state.current_section(), FormSection::Metadata);
    }

    #[test]
    fn test_next_section() {
        let mut state = CreateIssueFormState::new();
        assert_eq!(state.current_section(), FormSection::Summary);

        state.next_section();
        assert_eq!(state.current_section(), FormSection::Scheduling);

        state.next_section();
        assert_eq!(state.current_section(), FormSection::Relationships);

        state.next_section();
        assert_eq!(state.current_section(), FormSection::Labels);

        state.next_section();
        assert_eq!(state.current_section(), FormSection::Text);

        state.next_section();
        assert_eq!(state.current_section(), FormSection::Metadata);

        // Wraparound
        state.next_section();
        assert_eq!(state.current_section(), FormSection::Summary);
    }

    #[test]
    fn test_prev_section() {
        let mut state = CreateIssueFormState::new();
        assert_eq!(state.current_section(), FormSection::Summary);

        // Wraparound to end
        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Metadata);

        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Text);

        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Labels);

        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Relationships);

        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Scheduling);

        state.prev_section();
        assert_eq!(state.current_section(), FormSection::Summary);
    }

    #[test]
    fn test_toggle_preview() {
        let mut state = CreateIssueFormState::new();
        assert!(!state.is_preview_mode());

        state.toggle_preview();
        assert!(state.is_preview_mode());

        state.toggle_preview();
        assert!(!state.is_preview_mode());
    }

    #[test]
    fn test_current_section_fields_summary() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Summary);
        let fields = state.current_section_fields();
        assert_eq!(fields, vec!["title", "type", "priority", "status"]);
    }

    #[test]
    fn test_current_section_fields_scheduling() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Scheduling);
        let fields = state.current_section_fields();
        assert_eq!(fields, vec!["due_date", "defer_date", "time_estimate"]);
    }

    #[test]
    fn test_current_section_fields_relationships() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Relationships);
        let fields = state.current_section_fields();
        assert_eq!(fields, vec!["parent", "dependencies"]);
    }

    #[test]
    fn test_current_section_fields_labels() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Labels);
        let fields = state.current_section_fields();
        assert_eq!(fields, vec!["assignee", "labels"]);
    }

    #[test]
    fn test_current_section_fields_text() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Text);
        let fields = state.current_section_fields();
        assert_eq!(fields, vec!["description", "design", "acceptance", "notes"]);
    }

    #[test]
    fn test_current_section_fields_metadata() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Metadata);
        let fields = state.current_section_fields();
        assert_eq!(fields.len(), 0); // Metadata is read-only
    }

    #[test]
    fn test_is_section_complete_summary_empty() {
        let state = CreateIssueFormState::new();
        // Title is empty by default
        assert!(!state.is_section_complete(FormSection::Summary));
    }

    #[test]
    fn test_is_section_complete_summary_filled() {
        let mut state = CreateIssueFormState::new();
        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());
        assert!(state.is_section_complete(FormSection::Summary));
    }

    #[test]
    fn test_is_section_complete_other_sections() {
        let state = CreateIssueFormState::new();
        // Other sections have no required fields
        assert!(state.is_section_complete(FormSection::Scheduling));
        assert!(state.is_section_complete(FormSection::Relationships));
        assert!(state.is_section_complete(FormSection::Labels));
        assert!(state.is_section_complete(FormSection::Text));
        assert!(state.is_section_complete(FormSection::Metadata));
    }

    #[test]
    fn test_get_data_with_all_fields() {
        let mut state = CreateIssueFormState::new();

        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());
        state
            .form_state_mut()
            .set_value("type", "Feature".to_string());
        state
            .form_state_mut()
            .set_value("priority", "P1 (High)".to_string());
        state
            .form_state_mut()
            .set_value("status", "InProgress".to_string());
        state
            .form_state_mut()
            .set_value("assignee", "alice".to_string());
        state
            .form_state_mut()
            .set_value("labels", "frontend, ui".to_string());
        state
            .form_state_mut()
            .set_value("description", "Test description".to_string());
        state
            .form_state_mut()
            .set_value("due_date", "2026-01-31".to_string());
        state
            .form_state_mut()
            .set_value("defer_date", "2026-01-20".to_string());
        state
            .form_state_mut()
            .set_value("time_estimate", "3d".to_string());
        state
            .form_state_mut()
            .set_value("parent", "beads-abcd-0001".to_string());
        state.form_state_mut().set_value(
            "dependencies",
            "beads-efgh-0002, beads-ijkl-0003".to_string(),
        );
        state
            .form_state_mut()
            .set_value("design", "Design notes".to_string());
        state
            .form_state_mut()
            .set_value("acceptance", "Acceptance criteria".to_string());
        state
            .form_state_mut()
            .set_value("notes", "Additional notes".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.title, "Test Issue");
        assert_eq!(data.issue_type, IssueType::Feature);
        assert_eq!(data.priority, Priority::P1);
        assert_eq!(data.status, "InProgress");
        assert_eq!(data.assignee, Some("alice".to_string()));
        assert_eq!(data.labels.len(), 2);
        assert!(data.labels.contains(&"frontend".to_string()));
        assert!(data.labels.contains(&"ui".to_string()));
        assert_eq!(data.description, Some("Test description".to_string()));
        assert_eq!(data.due_date, Some("2026-01-31".to_string()));
        assert_eq!(data.defer_date, Some("2026-01-20".to_string()));
        assert_eq!(data.time_estimate, Some("3d".to_string()));
        assert_eq!(data.parent, Some("beads-abcd-0001".to_string()));
        assert_eq!(data.dependencies.len(), 2);
        assert!(data.dependencies.contains(&"beads-efgh-0002".to_string()));
        assert!(data.dependencies.contains(&"beads-ijkl-0003".to_string()));
        assert_eq!(data.design, Some("Design notes".to_string()));
        assert_eq!(data.acceptance, Some("Acceptance criteria".to_string()));
        assert_eq!(data.notes, Some("Additional notes".to_string()));
    }

    #[test]
    fn test_get_data_returns_none_with_errors() {
        let mut state = CreateIssueFormState::new();
        // Don't set title (required field)
        assert!(!state.validate());
        assert!(state.get_data().is_none());
    }

    #[test]
    fn test_get_data_empty_strings_become_none() {
        let mut state = CreateIssueFormState::new();
        state
            .form_state_mut()
            .set_value("title", "Test".to_string());
        state.form_state_mut().set_value("assignee", "".to_string());
        state
            .form_state_mut()
            .set_value("description", "".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.assignee, None);
        assert_eq!(data.description, None);
    }

    #[test]
    fn test_get_data_labels_splitting() {
        let mut state = CreateIssueFormState::new();
        state
            .form_state_mut()
            .set_value("title", "Test".to_string());
        state
            .form_state_mut()
            .set_value("labels", "  bug ,  urgent , high-priority  ".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.labels.len(), 3);
        assert_eq!(data.labels[0], "bug");
        assert_eq!(data.labels[1], "urgent");
        assert_eq!(data.labels[2], "high-priority");
    }

    #[test]
    fn test_validate_rejects_invalid_labels() {
        let mut state = CreateIssueFormState::new();
        state
            .form_state_mut()
            .set_value("title", "Test".to_string());
        state
            .form_state_mut()
            .set_value("labels", "bug fix, ok".to_string());

        assert!(!state.validate());
        assert!(state.get_data().is_none());

        let error = state
            .form_state()
            .get_field("labels")
            .and_then(|field| field.error.clone())
            .unwrap_or_default();
        assert!(error.contains("spaces"));
    }

    #[test]
    fn test_get_data_dependencies_splitting() {
        let mut state = CreateIssueFormState::new();
        state
            .form_state_mut()
            .set_value("title", "Test".to_string());
        state.form_state_mut().set_value(
            "dependencies",
            "  beads-abcd-0001 ,  beads-efgh-0002  ".to_string(),
        );

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.dependencies.len(), 2);
        assert_eq!(data.dependencies[0], "beads-abcd-0001");
        assert_eq!(data.dependencies[1], "beads-efgh-0002");
    }

    #[test]
    fn test_clear_resets_section_and_preview() {
        let mut state = CreateIssueFormState::new();
        state.set_section(FormSection::Labels);
        state.toggle_preview();
        state
            .form_state_mut()
            .set_value("title", "Test".to_string());

        assert_eq!(state.current_section(), FormSection::Labels);
        assert!(state.is_preview_mode());

        state.clear();

        assert_eq!(state.current_section(), FormSection::Summary);
        assert!(!state.is_preview_mode());
        assert_eq!(state.form_state().get_value("title"), Some(""));
    }

    // CreateIssueForm widget tests
    #[test]
    fn test_create_issue_form_new() {
        let form = CreateIssueForm::new();
        assert_eq!(form.title, "Create New Issue");
        assert!(form.show_help);
    }

    #[test]
    fn test_create_issue_form_default() {
        let form = CreateIssueForm::default();
        assert_eq!(form.title, "Create New Issue");
        assert!(form.show_help);
    }

    #[test]
    fn test_create_issue_form_title() {
        let form = CreateIssueForm::new().title("Custom Title");
        assert_eq!(form.title, "Custom Title");
    }

    #[test]
    fn test_create_issue_form_show_help() {
        let form = CreateIssueForm::new().show_help(false);
        assert!(!form.show_help);

        let form2 = CreateIssueForm::new().show_help(true);
        assert!(form2.show_help);
    }

    #[test]
    fn test_create_issue_form_builder_chain() {
        let form = CreateIssueForm::new().title("Edit Issue").show_help(false);

        assert_eq!(form.title, "Edit Issue");
        assert!(!form.show_help);
    }

    // CreateIssueData tests
    #[test]
    fn test_create_issue_data_creation() {
        let data = CreateIssueData {
            title: "Test".to_string(),
            issue_type: IssueType::Bug,
            priority: Priority::P0,
            status: "Open".to_string(),
            assignee: Some("bob".to_string()),
            labels: vec!["urgent".to_string()],
            description: Some("desc".to_string()),
            due_date: Some("2026-12-31".to_string()),
            defer_date: Some("2026-01-01".to_string()),
            time_estimate: Some("1w".to_string()),
            parent: Some("beads-prnt-0001".to_string()),
            dependencies: vec!["beads-deps-0001".to_string()],
            design: Some("design".to_string()),
            acceptance: Some("acceptance".to_string()),
            notes: Some("notes".to_string()),
        };

        assert_eq!(data.title, "Test");
        assert_eq!(data.issue_type, IssueType::Bug);
        assert_eq!(data.priority, Priority::P0);
        assert_eq!(data.assignee, Some("bob".to_string()));
    }
}
