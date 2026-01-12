//! Create issue form view with section navigator

use crate::beads::models::{IssueType, Priority};
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
    /// Create a new create issue form state
    pub fn new() -> Self {
        let fields = vec![
            // Summary section
            FormField::text("title", "Title")
                .required()
                .placeholder("Brief description of the issue")
                .with_validation(ValidationRule::Required),
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
            .required()
            .with_validation(ValidationRule::Enum(vec![
                "Epic".to_string(),
                "Feature".to_string(),
                "Task".to_string(),
                "Bug".to_string(),
                "Chore".to_string(),
            ])),
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
            .required()
            .with_validation(ValidationRule::Enum(vec![
                "P0 (Critical)".to_string(),
                "P1 (High)".to_string(),
                "P2 (Medium)".to_string(),
                "P3 (Low)".to_string(),
                "P4 (Backlog)".to_string(),
            ])),
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
            .required()
            .with_validation(ValidationRule::Enum(vec![
                "Open".to_string(),
                "InProgress".to_string(),
                "Blocked".to_string(),
                "Closed".to_string(),
            ])),
            // Scheduling section
            FormField::text("due_date", "Due Date").placeholder("YYYY-MM-DD (optional)"),
            FormField::text("defer_date", "Defer Date").placeholder("YYYY-MM-DD (optional)"),
            FormField::text("time_estimate", "Time Estimate")
                .placeholder("e.g., 2h, 3d, 1w (optional)"),
            // Relationships section
            FormField::text("parent", "Parent Issue")
                .placeholder("beads-xxx (optional)")
                .with_validation(ValidationRule::BeadsIdFormat),
            FormField::text("dependencies", "Dependencies")
                .placeholder("comma-separated beads-xxx (optional)"),
            // Labels section
            FormField::text("assignee", "Assignee").placeholder("username (optional)"),
            FormField::text("labels", "Labels").placeholder("comma-separated labels (optional)"),
            // Text section
            FormField::text_area("description", "Description")
                .placeholder("Detailed description of the issue (optional)"),
            FormField::text_area("design", "Design")
                .placeholder("Design notes and approach (optional)"),
            FormField::text_area("acceptance", "Acceptance Criteria")
                .placeholder("How to verify this is done (optional)"),
            FormField::text_area("notes", "Notes").placeholder("Additional notes (optional)"),
        ];

        Self {
            form_state: FormState::new(fields),
            current_section: FormSection::Summary,
            show_preview: false,
        }
    }

    /// Get the current section
    pub fn current_section(&self) -> FormSection {
        self.current_section
    }

    /// Set the current section
    pub fn set_section(&mut self, section: FormSection) {
        self.current_section = section;
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
    pub fn current_section_fields(&self) -> Vec<&str> {
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
        self.form_state.validate()
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
        let labels: Vec<String> = labels_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
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
            FormField::text("due_date", "Due Date").placeholder("YYYY-MM-DD (optional)"),
            FormField::text("defer_date", "Defer Date").placeholder("YYYY-MM-DD (optional)"),
            FormField::text("time_estimate", "Time Estimate")
                .placeholder("e.g., 2h, 3d, 1w (optional)"),
            // Relationships section
            FormField::text("parent", "Parent Issue")
                .placeholder("beads-xxx (optional)"),
            FormField::text("dependencies", "Dependencies")
                .placeholder("comma-separated beads-xxx (optional)"),
            // Labels section
            FormField::text("assignee", "Assignee").placeholder("username (optional)"),
            FormField::text("labels", "Labels").placeholder("comma-separated labels (optional)"),
            // Text section
            FormField::text_area("description", "Description")
                .placeholder("Detailed description of the issue (optional)"),
            FormField::text_area("design", "Design")
                .placeholder("Design notes and approach (optional)"),
            FormField::text_area("acceptance", "Acceptance Criteria")
                .placeholder("How to verify this is done (optional)"),
            FormField::text_area("notes", "Notes").placeholder("Additional notes (optional)"),
        ]);
        self.current_section = FormSection::Summary;
        self.show_preview = false;
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
}

impl<'a> Default for CreateIssueForm<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for CreateIssueForm<'a> {
    type State = CreateIssueFormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
                Span::styled("Tab/↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Next field  "),
                Span::styled("Shift+Tab/↑", Style::default().fg(Color::Yellow)),
                Span::raw(" Previous field  "),
                Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
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
        assert_eq!(state.form_state().get_value("assignee"), Some(""));
    }

    #[test]
    fn test_optional_fields() {
        let mut state = CreateIssueFormState::new();

        state
            .form_state_mut()
            .set_value("title", "Test Issue".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.assignee, None);
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
        assert_eq!(FormSection::Summary.description(), "Title, type, priority, status");
        assert_eq!(FormSection::Scheduling.description(), "Due date, defer date, time estimate");
        assert_eq!(FormSection::Relationships.description(), "Parent issue, dependencies");
        assert_eq!(FormSection::Labels.description(), "Tags and categories");
        assert_eq!(FormSection::Text.description(), "Description, design, acceptance criteria, notes");
        assert_eq!(FormSection::Metadata.description(), "Read-only system information");
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
        state.form_state_mut().set_value("title", "Test Issue".to_string());
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

        state.form_state_mut().set_value("title", "Test Issue".to_string());
        state.form_state_mut().set_value("type", "Feature".to_string());
        state.form_state_mut().set_value("priority", "P1 (High)".to_string());
        state.form_state_mut().set_value("status", "InProgress".to_string());
        state.form_state_mut().set_value("assignee", "alice".to_string());
        state.form_state_mut().set_value("labels", "frontend, ui".to_string());
        state.form_state_mut().set_value("description", "Test description".to_string());
        state.form_state_mut().set_value("due_date", "2026-01-31".to_string());
        state.form_state_mut().set_value("defer_date", "2026-01-20".to_string());
        state.form_state_mut().set_value("time_estimate", "3d".to_string());
        state.form_state_mut().set_value("parent", "beads-abcd-0001".to_string());
        state.form_state_mut().set_value("dependencies", "beads-efgh-0002, beads-ijkl-0003".to_string());
        state.form_state_mut().set_value("design", "Design notes".to_string());
        state.form_state_mut().set_value("acceptance", "Acceptance criteria".to_string());
        state.form_state_mut().set_value("notes", "Additional notes".to_string());

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
        state.form_state_mut().set_value("title", "Test".to_string());
        state.form_state_mut().set_value("assignee", "".to_string());
        state.form_state_mut().set_value("description", "".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.assignee, None);
        assert_eq!(data.description, None);
    }

    #[test]
    fn test_get_data_labels_splitting() {
        let mut state = CreateIssueFormState::new();
        state.form_state_mut().set_value("title", "Test".to_string());
        state.form_state_mut().set_value("labels", "  bug ,  urgent , high-priority  ".to_string());

        assert!(state.validate());

        let data = state.get_data().unwrap();
        assert_eq!(data.labels.len(), 3);
        assert_eq!(data.labels[0], "bug");
        assert_eq!(data.labels[1], "urgent");
        assert_eq!(data.labels[2], "high-priority");
    }

    #[test]
    fn test_get_data_dependencies_splitting() {
        let mut state = CreateIssueFormState::new();
        state.form_state_mut().set_value("title", "Test".to_string());
        state.form_state_mut().set_value("dependencies", "  beads-abcd-0001 ,  beads-efgh-0002  ".to_string());

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
        state.form_state_mut().set_value("title", "Test".to_string());

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
        let form = CreateIssueForm::new()
            .title("Edit Issue")
            .show_help(false);

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
