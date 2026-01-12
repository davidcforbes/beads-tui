//! Multi-field form widget for beads-tui

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Validation rules for form fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationRule {
    /// Field must not be empty
    Required,
    /// Value must be one of the allowed enum values
    Enum(Vec<String>),
    /// Value must be a valid integer >= 0
    PositiveInteger,
    /// Value must match beads ID format (beads-xxx-xxxx)
    BeadsIdFormat,
    /// Value must not contain spaces
    NoSpaces,
    /// Value must be a valid date (RFC3339 or relative)
    Date,
    /// Value must be a date >= now (for due dates)
    FutureDate,
}

/// Form field type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Single-line text input
    Text,
    /// Password input (masked)
    Password,
    /// Multi-line text area
    TextArea,
    /// Dropdown selector
    Selector,
    /// Read-only display field
    ReadOnly,
}

/// Form field definition
#[derive(Debug, Clone)]
pub struct FormField {
    /// Field identifier
    pub id: String,
    /// Field label
    pub label: String,
    /// Field type
    pub field_type: FieldType,
    /// Field value
    pub value: String,
    /// Is field required
    pub required: bool,
    /// Validation error message
    pub error: Option<String>,
    /// Field placeholder text
    pub placeholder: Option<String>,
    /// Available options for selector fields
    pub options: Vec<String>,
    /// Validation rules for this field
    pub validation_rules: Vec<ValidationRule>,
    /// File path if loaded from file
    pub loaded_from_file: Option<String>,
}

impl FormField {
    /// Create a new text field
    pub fn text<S: Into<String>>(id: S, label: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Text,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
        }
    }

    /// Create a new password field (input is masked)
    pub fn password<S: Into<String>>(id: S, label: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Password,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
        }
    }

    /// Create a new text area field
    pub fn text_area<S: Into<String>>(id: S, label: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::TextArea,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
        }
    }

    /// Create a new selector field
    pub fn selector<S: Into<String>>(id: S, label: S, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Selector,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            options,
            validation_rules: Vec::new(),
            loaded_from_file: None,
        }
    }

    /// Create a new read-only field
    pub fn read_only<S: Into<String>>(id: S, label: S, value: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::ReadOnly,
            value: value.into(),
            required: false,
            error: None,
            placeholder: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
        }
    }

    /// Set field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set placeholder text
    pub fn placeholder<S: Into<String>>(mut self, placeholder: S) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set field value
    pub fn value<S: Into<String>>(mut self, value: S) -> Self {
        self.value = value.into();
        self
    }

    /// Add a validation rule
    pub fn with_validation(mut self, rule: ValidationRule) -> Self {
        self.validation_rules.push(rule);
        self
    }

    /// Validate the field
    pub fn validate(&mut self) -> bool {
        // Legacy required check
        if self.required && self.value.trim().is_empty() {
            self.error = Some(format!("{} is required", self.label));
            return false;
        }

        // Check all validation rules
        for rule in &self.validation_rules {
            if let Some(error) = Self::validate_rule(&self.value, rule, &self.label) {
                self.error = Some(error);
                return false;
            }
        }

        self.error = None;
        true
    }

    /// Validate a single rule
    fn validate_rule(value: &str, rule: &ValidationRule, label: &str) -> Option<String> {
        match rule {
            ValidationRule::Required => {
                if value.trim().is_empty() {
                    Some(format!("{} is required", label))
                } else {
                    None
                }
            }
            ValidationRule::Enum(allowed_values) => {
                if !value.is_empty() && !allowed_values.iter().any(|v| v == value) {
                    Some(format!(
                        "{} must be one of: {}",
                        label,
                        allowed_values.join(", ")
                    ))
                } else {
                    None
                }
            }
            ValidationRule::PositiveInteger => {
                if !value.is_empty() {
                    match value.parse::<i64>() {
                        Ok(n) if n >= 0 => None,
                        Ok(_) => Some(format!("{} must be >= 0", label)),
                        Err(_) => Some(format!("{} must be a valid number", label)),
                    }
                } else {
                    None
                }
            }
            ValidationRule::BeadsIdFormat => {
                if !value.is_empty() {
                    let parts: Vec<&str> = value.split('-').collect();
                    if parts.len() == 3
                        && parts[0] == "beads"
                        && parts[1].len() == 4
                        && parts[2].len() == 4
                        && parts[1].chars().all(|c| c.is_alphanumeric())
                        && parts[2].chars().all(|c| c.is_alphanumeric())
                    {
                        None
                    } else {
                        Some(format!(
                            "{} must match format: beads-xxxx-xxxx",
                            label
                        ))
                    }
                } else {
                    None
                }
            }
            ValidationRule::NoSpaces => {
                if !value.is_empty() && value.contains(' ') {
                    Some(format!("{} must not contain spaces", label))
                } else {
                    None
                }
            }
            ValidationRule::Date => {
                if !value.is_empty() {
                    // Try to parse as RFC3339 or relative date
                    if chrono::DateTime::parse_from_rfc3339(value).is_ok() {
                        None
                    } else if Self::is_relative_date(value) {
                        None
                    } else {
                        Some(format!(
                            "{} must be a valid date (RFC3339 or relative like '1d', '2w')",
                            label
                        ))
                    }
                } else {
                    None
                }
            }
            ValidationRule::FutureDate => {
                if !value.is_empty() {
                    // Parse the date and check if it's in the future
                    if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(value) {
                        if parsed_date.timestamp() >= chrono::Utc::now().timestamp() {
                            None
                        } else {
                            Some(format!("{} must be in the future", label))
                        }
                    } else if Self::is_relative_date(value) {
                        // Relative dates are always future by definition
                        None
                    } else {
                        Some(format!(
                            "{} must be a valid future date",
                            label
                        ))
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Check if a value is a relative date format (e.g., "1d", "2w", "3m")
    fn is_relative_date(value: &str) -> bool {
        let value = value.trim();
        if value.len() < 2 {
            return false;
        }

        let (num_part, unit_part) = value.split_at(value.len() - 1);
        num_part.parse::<u32>().is_ok()
            && matches!(unit_part, "d" | "w" | "m" | "y" | "h")
    }
}

/// Form state
#[derive(Debug)]
pub struct FormState {
    fields: Vec<FormField>,
    focused_index: usize,
    cursor_position: usize,
}

impl FormState {
    /// Create a new form state
    pub fn new(fields: Vec<FormField>) -> Self {
        Self {
            fields,
            focused_index: 0,
            cursor_position: 0,
        }
    }

    /// Get the currently focused field
    pub fn focused_field(&self) -> Option<&FormField> {
        self.fields.get(self.focused_index)
    }

    /// Get mutable reference to currently focused field
    pub fn focused_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.focused_index)
    }

    /// Get all fields
    pub fn fields(&self) -> &[FormField] {
        &self.fields
    }

    /// Get mutable reference to all fields
    pub fn fields_mut(&mut self) -> &mut [FormField] {
        &mut self.fields
    }

    /// Get field by ID
    pub fn get_field(&self, id: &str) -> Option<&FormField> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Get mutable field by ID
    pub fn get_field_mut(&mut self, id: &str) -> Option<&mut FormField> {
        self.fields.iter_mut().find(|f| f.id == id)
    }

    /// Set field value by ID
    pub fn set_value(&mut self, id: &str, value: String) {
        if let Some(field) = self.get_field_mut(id) {
            field.value = value;
        }
    }

    /// Get field value by ID
    pub fn get_value(&self, id: &str) -> Option<&str> {
        self.get_field(id).map(|f| f.value.as_str())
    }

    /// Move focus to next field
    pub fn focus_next(&mut self) {
        if self.focused_index < self.fields.len().saturating_sub(1) {
            self.focused_index += 1;
            self.cursor_position = 0;
        }
    }

    /// Move focus to previous field
    pub fn focus_previous(&mut self) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
            self.cursor_position = 0;
        }
    }

    /// Get focused field index
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Set focused field index
    pub fn set_focused_index(&mut self, index: usize) {
        if index < self.fields.len() {
            self.focused_index = index;
            self.cursor_position = 0;
        }
    }

    /// Get cursor position in current field
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Insert character at cursor in focused field
    pub fn insert_char(&mut self, c: char) {
        let cursor_pos = self.cursor_position;
        if let Some(field) = self.focused_field_mut() {
            if field.field_type == FieldType::ReadOnly {
                return;
            }
            if field.field_type == FieldType::Text && c == '\n' {
                return;
            }
            field.value.insert(cursor_pos, c);
        }
        self.cursor_position += 1;
    }

    /// Delete character before cursor in focused field
    pub fn delete_char(&mut self) {
        let cursor_pos = self.cursor_position;
        if let Some(field) = self.focused_field_mut() {
            if field.field_type == FieldType::ReadOnly {
                return;
            }
            if cursor_pos > 0 && cursor_pos <= field.value.len() {
                field.value.remove(cursor_pos - 1);
            }
        }
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if let Some(field) = self.focused_field() {
            if self.cursor_position < field.value.len() {
                self.cursor_position += 1;
            }
        }
    }

    /// Move cursor to start
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_to_end(&mut self) {
        if let Some(field) = self.focused_field() {
            self.cursor_position = field.value.len();
        }
    }

    /// Validate all fields
    pub fn validate(&mut self) -> bool {
        let mut all_valid = true;
        for field in &mut self.fields {
            if !field.validate() {
                all_valid = false;
            }
        }
        all_valid
    }

    /// Check if form has any errors
    pub fn has_errors(&self) -> bool {
        self.fields.iter().any(|f| f.error.is_some())
    }

    /// Clear all field errors
    pub fn clear_errors(&mut self) {
        for field in &mut self.fields {
            field.error = None;
        }
    }

    /// Load content from a file into the currently focused field
    /// Returns Ok(()) on success, or Err with an error message
    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), String> {
        use std::fs;
        use std::path::Path;

        // Validate focused field is a text area
        let focused_idx = self.focused_index;
        if focused_idx >= self.fields.len() {
            return Err("No field focused".to_string());
        }

        let field = &self.fields[focused_idx];
        if field.field_type != FieldType::TextArea {
            return Err(format!("Cannot load file into {:?} field", field.field_type));
        }

        // Validate file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        if !path.is_file() {
            return Err(format!("Path is not a file: {}", file_path));
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Validate UTF-8 (already validated by read_to_string, but check for null bytes)
        if content.contains('\0') {
            return Err("File contains invalid UTF-8 characters".to_string());
        }

        // Update field value and track file path
        let field = &mut self.fields[focused_idx];
        field.value = content;
        field.loaded_from_file = Some(file_path.to_string());
        field.error = None;

        // Reset cursor to end
        self.cursor_position = field.value.len();

        Ok(())
    }

    /// Clear the loaded file path for the focused field
    pub fn clear_loaded_file(&mut self) {
        if let Some(field) = self.focused_field_mut() {
            field.loaded_from_file = None;
        }
    }

    /// Get the file path if the focused field was loaded from a file
    pub fn get_loaded_file_path(&self) -> Option<&String> {
        self.focused_field()?.loaded_from_file.as_ref()
    }
}

/// Multi-field form widget
pub struct Form<'a> {
    title: Option<&'a str>,
    style: Style,
    focused_style: Style,
    error_style: Style,
    block: Option<Block<'a>>,
}

impl<'a> Form<'a> {
    /// Create a new form
    pub fn new() -> Self {
        Self {
            title: None,
            style: Style::default(),
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            error_style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            block: Some(Block::default().borders(Borders::ALL)),
        }
    }

    /// Set form title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set form style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set focused field style
    pub fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    /// Set error style
    pub fn error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }

    /// Set form block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Default for Form<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for Form<'a> {
    type State = FormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build outer block
        let title = self.title.unwrap_or("Form");
        let block = if let Some(mut block) = self.block {
            block = block.title(title);
            block
        } else {
            Block::default().borders(Borders::ALL).title(title)
        };

        let inner = block.inner(area);
        block.render(area, buf);

        if state.fields.is_empty() {
            let empty_msg = Paragraph::new("No fields defined").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            );
            empty_msg.render(inner, buf);
            return;
        }

        // Calculate layout for fields
        let constraints: Vec<Constraint> = state
            .fields
            .iter()
            .map(|f| {
                if f.field_type == FieldType::TextArea {
                    Constraint::Min(5)
                } else {
                    Constraint::Length(3)
                }
            })
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Render each field
        for (i, field) in state.fields.iter().enumerate() {
            let chunk = &chunks[i];
            let is_focused = i == state.focused_index;

            // Determine field style
            let field_style = if field.error.is_some() {
                self.error_style
            } else if is_focused {
                self.focused_style
            } else {
                self.style
            };

            // Build field title
            let mut title = if field.required {
                format!("{} *", field.label)
            } else {
                field.label.clone()
            };

            // Show file path if loaded from file
            if let Some(ref file_path) = field.loaded_from_file {
                title = format!("{} [from: {}]", title, file_path);
            }

            let title = if is_focused {
                format!("{title} [editing]")
            } else {
                title
            };

            // Create field block
            let field_block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(field_style);

            let field_inner = field_block.inner(*chunk);
            field_block.render(*chunk, buf);

            // Render field content
            if field_inner.height < 1 {
                continue;
            }

            let content: Vec<Line> = if field.value.is_empty() {
                if let Some(ref placeholder) = field.placeholder {
                    vec![Line::from(Span::styled(
                        placeholder.clone(),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ))]
                } else {
                    vec![Line::from("")]
                }
            } else if field.field_type == FieldType::Password {
                // Mask password fields with asterisks
                let masked = "*".repeat(field.value.len());
                vec![Line::from(masked)]
            } else {
                field
                    .value
                    .lines()
                    .map(|line: &str| Line::from(line.to_string()))
                    .collect()
            };

            let paragraph = Paragraph::new(content);
            paragraph.render(field_inner, buf);

            // Render cursor if focused
            if is_focused && field_inner.width > 0 && field_inner.height > 0 {
                let cursor_x = field_inner.x
                    + state.cursor_position.min(field_inner.width as usize - 1) as u16;
                let cursor_y = field_inner.y;

                if cursor_x < field_inner.x + field_inner.width {
                    buf.get_mut(cursor_x, cursor_y)
                        .set_style(Style::default().bg(Color::White).fg(Color::Black));
                }
            }

            // Render error message if present
            if let Some(ref error) = field.error {
                if field_inner.height > 1 {
                    let error_line =
                        Line::from(Span::styled(format!("⚠ {error}"), self.error_style));
                    let error_y = field_inner.y + 1;
                    if error_y < field_inner.y + field_inner.height {
                        error_line.render(
                            Rect {
                                x: field_inner.x,
                                y: error_y,
                                width: field_inner.width,
                                height: 1,
                            },
                            buf,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_creation() {
        let field = FormField::text("title", "Title");
        assert_eq!(field.id, "title");
        assert_eq!(field.label, "Title");
        assert_eq!(field.field_type, FieldType::Text);
        assert!(!field.required);
    }

    #[test]
    fn test_password_field_creation() {
        let field = FormField::password("password", "Password");
        assert_eq!(field.id, "password");
        assert_eq!(field.label, "Password");
        assert_eq!(field.field_type, FieldType::Password);
        assert!(!field.required);
        assert_eq!(field.value, "");
    }

    #[test]
    fn test_form_field_required() {
        let field = FormField::text("title", "Title").required();
        assert!(field.required);
    }

    #[test]
    fn test_form_field_validation() {
        let mut field = FormField::text("title", "Title").required();
        assert!(!field.validate());
        assert!(field.error.is_some());

        field.value = "Test".to_string();
        assert!(field.validate());
        assert!(field.error.is_none());
    }

    #[test]
    fn test_form_state_creation() {
        let fields = vec![
            FormField::text("title", "Title"),
            FormField::text_area("description", "Description"),
        ];
        let state = FormState::new(fields);
        assert_eq!(state.fields.len(), 2);
        assert_eq!(state.focused_index, 0);
    }

    #[test]
    fn test_form_state_navigation() {
        let fields = vec![
            FormField::text("title", "Title"),
            FormField::text("assignee", "Assignee"),
            FormField::text_area("description", "Description"),
        ];
        let mut state = FormState::new(fields);

        assert_eq!(state.focused_index(), 0);

        state.focus_next();
        assert_eq!(state.focused_index(), 1);

        state.focus_next();
        assert_eq!(state.focused_index(), 2);

        state.focus_next();
        assert_eq!(state.focused_index(), 2); // Should not go beyond last field

        state.focus_previous();
        assert_eq!(state.focused_index(), 1);

        state.focus_previous();
        assert_eq!(state.focused_index(), 0);

        state.focus_previous();
        assert_eq!(state.focused_index(), 0); // Should not go below 0
    }

    #[test]
    fn test_form_state_field_access() {
        let fields = vec![
            FormField::text("title", "Title"),
            FormField::text("assignee", "Assignee"),
        ];
        let mut state = FormState::new(fields);

        state.set_value("title", "Test Issue".to_string());
        assert_eq!(state.get_value("title"), Some("Test Issue"));

        state.set_value("assignee", "john".to_string());
        assert_eq!(state.get_value("assignee"), Some("john"));
    }

    #[test]
    fn test_form_state_input() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        state.insert_char('H');
        state.insert_char('i');
        assert_eq!(state.get_value("title"), Some("Hi"));
        assert_eq!(state.cursor_position(), 2);

        state.delete_char();
        assert_eq!(state.get_value("title"), Some("H"));
        assert_eq!(state.cursor_position(), 1);
    }

    #[test]
    fn test_form_state_cursor_movement() {
        let fields = vec![FormField::text("title", "Title").value("Hello")];
        let mut state = FormState::new(fields);
        state.cursor_position = 5;

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 4);

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position(), 0);

        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 1);

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_form_validation() {
        let fields = vec![
            FormField::text("title", "Title").required(),
            FormField::text("assignee", "Assignee"),
        ];
        let mut state = FormState::new(fields);

        assert!(!state.validate());
        assert!(state.has_errors());

        state.set_value("title", "Test Issue".to_string());
        assert!(state.validate());
        assert!(!state.has_errors());
    }

    #[test]
    fn test_form_clear_errors() {
        let fields = vec![FormField::text("title", "Title").required()];
        let mut state = FormState::new(fields);

        state.validate();
        assert!(state.has_errors());

        state.clear_errors();
        assert!(!state.has_errors());
    }

    #[test]
    fn test_validation_rule_required() {
        let mut field = FormField::text("title", "Title")
            .with_validation(ValidationRule::Required);

        assert!(!field.validate());
        assert!(field.error.is_some());

        field.value = "Test".to_string();
        assert!(field.validate());
        assert!(field.error.is_none());
    }

    #[test]
    fn test_validation_rule_enum() {
        let mut field = FormField::text("status", "Status")
            .with_validation(ValidationRule::Enum(vec![
                "open".to_string(),
                "closed".to_string(),
            ]));

        // Empty value should pass
        assert!(field.validate());

        // Valid value should pass
        field.value = "open".to_string();
        assert!(field.validate());
        assert!(field.error.is_none());

        // Invalid value should fail
        field.value = "invalid".to_string();
        assert!(!field.validate());
        assert!(field.error.is_some());
    }

    #[test]
    fn test_validation_rule_positive_integer() {
        let mut field = FormField::text("estimate", "Estimate")
            .with_validation(ValidationRule::PositiveInteger);

        // Empty value should pass
        assert!(field.validate());

        // Valid positive integer
        field.value = "100".to_string();
        assert!(field.validate());

        // Zero should pass
        field.value = "0".to_string();
        assert!(field.validate());

        // Negative should fail
        field.value = "-1".to_string();
        assert!(!field.validate());

        // Non-integer should fail
        field.value = "abc".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_rule_beads_id_format() {
        let mut field = FormField::text("dependency", "Dependency")
            .with_validation(ValidationRule::BeadsIdFormat);

        // Empty value should pass
        assert!(field.validate());

        // Valid beads ID
        field.value = "beads-1234-5678".to_string();
        assert!(field.validate());

        field.value = "beads-abcd-efgh".to_string();
        assert!(field.validate());

        // Invalid formats
        field.value = "beads-12-34".to_string();
        assert!(!field.validate());

        field.value = "issue-1234-5678".to_string();
        assert!(!field.validate());

        field.value = "beads-123-5678".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_rule_no_spaces() {
        let mut field = FormField::text("label", "Label")
            .with_validation(ValidationRule::NoSpaces);

        // Empty value should pass
        assert!(field.validate());

        // Value without spaces
        field.value = "bug-fix".to_string();
        assert!(field.validate());

        // Value with spaces should fail
        field.value = "bug fix".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_rule_date() {
        let mut field = FormField::text("due_date", "Due Date")
            .with_validation(ValidationRule::Date);

        // Empty value should pass
        assert!(field.validate());

        // Valid RFC3339 date
        field.value = "2025-01-15T10:00:00Z".to_string();
        assert!(field.validate());

        // Valid relative date
        field.value = "1d".to_string();
        assert!(field.validate());

        field.value = "2w".to_string();
        assert!(field.validate());

        // Invalid date
        field.value = "not-a-date".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_rule_future_date() {
        let mut field = FormField::text("due_date", "Due Date")
            .with_validation(ValidationRule::FutureDate);

        // Empty value should pass
        assert!(field.validate());

        // Future RFC3339 date should pass
        field.value = "2030-01-01T00:00:00Z".to_string();
        assert!(field.validate());

        // Relative dates are always future
        field.value = "1d".to_string();
        assert!(field.validate());

        // Past date should fail
        field.value = "2020-01-01T00:00:00Z".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_multiple_validation_rules() {
        let mut field = FormField::text("estimate", "Estimate")
            .with_validation(ValidationRule::Required)
            .with_validation(ValidationRule::PositiveInteger);

        // Empty should fail (required)
        assert!(!field.validate());

        // Negative should fail (positive integer)
        field.value = "-1".to_string();
        assert!(!field.validate());

        // Valid should pass
        field.value = "100".to_string();
        assert!(field.validate());
    }
}
