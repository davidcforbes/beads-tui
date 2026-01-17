//! Multi-field form widget for beads-tui

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Layout hint for form field positioning
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutHint {
    /// Field takes full width (default for vertical layout)
    FullWidth,
    /// Field is part of a horizontal group
    HorizontalGroup {
        /// Group identifier (fields with same group_id render on same row)
        group_id: String,
        /// Width constraint for this field within the group
        width: Constraint,
    },
}

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
    /// Value must not exceed maximum length (in bytes)
    MaxLength(usize),
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
    /// Section header with horizontal border
    SectionHeader,
    /// Radio button group
    RadioButton,
    /// Label editor with colon-separated values
    LabelEditor,
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
    /// Help text (shown with F1)
    pub help_text: Option<String>,
    /// Available options for selector fields
    pub options: Vec<String>,
    /// Validation rules for this field
    pub validation_rules: Vec<ValidationRule>,
    /// File path if loaded from file
    pub loaded_from_file: Option<String>,
    /// Is field hidden
    pub hidden: bool,
    /// Layout hint for positioning
    pub layout_hint: Option<LayoutHint>,
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
            help_text: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
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
            help_text: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
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
            help_text: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
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
            help_text: None,
            options,
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
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
            help_text: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
        }
    }

    /// Create a new section header field
    pub fn section_header<S: Into<String>>(id: S, label: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::SectionHeader,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            help_text: None,
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
        }
    }

    /// Create a new radio button field
    pub fn radio_button<S: Into<String>>(id: S, label: S, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::RadioButton,
            value: String::new(),
            required: false,
            error: None,
            placeholder: None,
            help_text: None,
            options,
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
        }
    }

    /// Create a new label editor field (colon-separated labels)
    pub fn label_editor<S: Into<String>>(id: S, label: S) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::LabelEditor,
            value: String::new(),
            required: false,
            error: None,
            placeholder: Some("Separate labels with ':' (e.g., bug:urgent:frontend)".to_string()),
            help_text: Some("Enter labels separated by colons. Example: bug:urgent:frontend".to_string()),
            options: Vec::new(),
            validation_rules: Vec::new(),
            loaded_from_file: None,
            hidden: false,
            layout_hint: None,
        }
    }

    /// Set field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set field as hidden
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Set placeholder text
    pub fn placeholder<S: Into<String>>(mut self, placeholder: S) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set help text (shown when user presses F1)
    pub fn help_text<S: Into<String>>(mut self, help_text: S) -> Self {
        self.help_text = Some(help_text.into());
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

    /// Mark field as part of a horizontal group with specified width
    pub fn in_horizontal_group(mut self, group_id: &str, width: Constraint) -> Self {
        self.layout_hint = Some(LayoutHint::HorizontalGroup {
            group_id: group_id.to_string(),
            width,
        });
        self
    }

    /// Mark field as full-width (default)
    pub fn full_width(mut self) -> Self {
        self.layout_hint = Some(LayoutHint::FullWidth);
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
                    Some(format!("{label} is required"))
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
                        Ok(_) => Some(format!("{label} must be >= 0")),
                        Err(_) => Some(format!("{label} must be a valid number")),
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
                        Some(format!("{label} must match format: beads-xxxx-xxxx"))
                    }
                } else {
                    None
                }
            }
            ValidationRule::NoSpaces => {
                if !value.is_empty() && value.contains(' ') {
                    Some(format!("{label} must not contain spaces"))
                } else {
                    None
                }
            }
            ValidationRule::Date => {
                if !value.is_empty() {
                    // Try to parse as RFC3339 or relative date
                    if chrono::DateTime::parse_from_rfc3339(value).is_ok()
                        || Self::is_relative_date(value)
                    {
                        None
                    } else {
                        Some(format!(
                            "{label} must be a valid date (RFC3339 or relative like '1d', '2w')"
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
                            Some(format!("{label} must be in the future"))
                        }
                    } else if Self::is_relative_date(value) {
                        // Relative dates are always future by definition
                        None
                    } else {
                        Some(format!("{label} must be a valid future date"))
                    }
                } else {
                    None
                }
            }
            ValidationRule::MaxLength(max_len) => {
                let byte_len = value.len();
                if byte_len > *max_len {
                    // Format max_len in human-readable form
                    let max_str = if *max_len >= 1_048_576 {
                        format!("{} MB", max_len / 1_048_576)
                    } else if *max_len >= 1024 {
                        format!("{} KB", max_len / 1024)
                    } else {
                        format!("{} bytes", max_len)
                    };
                    Some(format!(
                        "{} exceeds maximum length of {} (current: {} bytes)",
                        label, max_str, byte_len
                    ))
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
        num_part.parse::<u32>().is_ok() && matches!(unit_part, "d" | "w" | "m" | "y" | "h")
    }

    /// Parse labels from colon-separated string
    /// Example: "bug:urgent:frontend" -> ["bug", "urgent", "frontend"]
    pub fn parse_labels(&self) -> Vec<String> {
        if self.field_type != FieldType::LabelEditor {
            return Vec::new();
        }

        self.value
            .split(':')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Format labels as colon-separated string
    /// Example: ["bug", "urgent", "frontend"] -> "bug:urgent:frontend"
    pub fn set_labels(&mut self, labels: &[String]) {
        if self.field_type != FieldType::LabelEditor {
            return;
        }

        self.value = labels.join(":");
    }
}

/// Search state for Ctrl+F functionality
#[derive(Debug, Clone)]
pub struct SearchState {
    /// Search query string
    pub query: String,
    /// Matches found: Vec<(field_index, char_offset)>
    pub matches: Vec<(usize, usize)>,
    /// Currently selected match index
    pub current_match: usize,
}

/// Form state
#[derive(Debug)]
pub struct FormState {
    fields: Vec<FormField>,
    focused_index: usize,
    cursor_position: usize,
    showing_help: bool,
    /// Scroll offset (top visible field index)
    scroll_offset: usize,
    /// Visible height in field rows
    visible_height: usize,
    /// Active search state
    search_state: Option<SearchState>,
}

impl FormState {
    /// Create a new form state
    pub fn new(fields: Vec<FormField>) -> Self {
        Self {
            fields,
            focused_index: 0,
            cursor_position: 0,
            showing_help: false,
            scroll_offset: 0,
            visible_height: 20, // Default, will be updated in render
            search_state: None,
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
        let mut next_index = self.focused_index + 1;
        while next_index < self.fields.len() {
            if !self.fields[next_index].hidden {
                self.focused_index = next_index;
                self.cursor_position = 0;
                self.ensure_focused_visible();
                return;
            }
            next_index += 1;
        }
    }

    /// Move focus to previous field
    pub fn focus_previous(&mut self) {
        if self.focused_index == 0 {
            return;
        }
        let mut prev_index = self.focused_index - 1;
        loop {
            if !self.fields[prev_index].hidden {
                self.focused_index = prev_index;
                self.cursor_position = 0;
                self.ensure_focused_visible();
                return;
            }
            if prev_index == 0 {
                break;
            }
            prev_index -= 1;
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

            // Check MaxLength validation rules before inserting
            let would_exceed_max = field.validation_rules.iter().any(|rule| {
                if let ValidationRule::MaxLength(max_len) = rule {
                    field.value.len() + c.len_utf8() > *max_len
                } else {
                    false
                }
            });

            if would_exceed_max {
                // Set error but don't insert the character
                field.error = Some("Maximum length reached".to_string());
                return;
            }

            field.value.insert(cursor_pos, c);
            // Validate immediately to provide real-time feedback
            field.validate();
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
                // Validate immediately to provide real-time feedback
                field.validate();
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
        let mut first_error_index: Option<usize> = None;

        for (i, field) in self.fields.iter_mut().enumerate() {
            if !field.validate() {
                all_valid = false;
                if first_error_index.is_none() {
                    first_error_index = Some(i);
                }
            }
        }

        // Auto-focus first field with error
        if let Some(error_idx) = first_error_index {
            self.focused_index = error_idx;
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

    /// Toggle help display for the focused field
    pub fn toggle_help(&mut self) {
        self.showing_help = !self.showing_help;
    }

    /// Check if help is currently being shown
    pub fn is_showing_help(&self) -> bool {
        self.showing_help
    }

    /// Hide help display
    pub fn hide_help(&mut self) {
        self.showing_help = false;
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
            return Err(format!(
                "Cannot load file into {:?} field",
                field.field_type
            ));
        }

        // Sanitize and canonicalize path to prevent path traversal attacks
        let path = Path::new(file_path);

        // Canonicalize the path to resolve any ../.. or symlinks
        let canonical_path = path
            .canonicalize()
            .map_err(|_| "File not found or inaccessible".to_string())?;

        // Open file first to get atomic handle, then verify metadata
        // This eliminates TOCTOU race condition between is_file() check and read
        let file = fs::File::open(&canonical_path)
            .map_err(|_| "Failed to open file: permission denied".to_string())?;

        // Verify it's actually a file using the file handle's metadata
        let metadata = file
            .metadata()
            .map_err(|_| "Failed to read file metadata".to_string())?;

        if !metadata.is_file() {
            return Err("Path is not a regular file".to_string());
        }

        // Read file content using the validated file handle
        use std::io::Read;
        let mut content = String::new();
        let mut file = file; // Make mutable for reading
        file.read_to_string(&mut content).map_err(|_| {
            "Failed to read file: permission denied or file is not UTF-8".to_string()
        })?;

        // Validate UTF-8 (already validated by read_to_string, but check for null bytes)
        if content.contains('\0') {
            return Err("File contains invalid UTF-8 characters".to_string());
        }

        // Check MaxLength validation rules before loading
        let field = &self.fields[focused_idx];
        for rule in &field.validation_rules {
            if let ValidationRule::MaxLength(max_len) = rule {
                if content.len() > *max_len {
                    let max_str = if *max_len >= 1_048_576 {
                        format!("{} MB", max_len / 1_048_576)
                    } else if *max_len >= 1024 {
                        format!("{} KB", max_len / 1024)
                    } else {
                        format!("{} bytes", max_len)
                    };
                    return Err(format!(
                        "File exceeds maximum length of {} (file size: {} bytes)",
                        max_str,
                        content.len()
                    ));
                }
            }
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

    /// Calculate the height of a field in rows
    fn calculate_field_height(field: &FormField) -> usize {
        match field.field_type {
            FieldType::SectionHeader => 2,
            FieldType::TextArea => 6,
            _ => 3,
        }
    }

    /// Scroll up by specified number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll down by specified number of lines
    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.fields.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    /// Scroll to top of form
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom of form
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.fields.len().saturating_sub(1);
    }

    /// Ensure the focused field is visible by auto-scrolling if needed
    pub fn ensure_focused_visible(&mut self) {
        if self.focused_index < self.scroll_offset {
            // Focused field is above visible window, scroll up
            self.scroll_offset = self.focused_index;
        } else {
            // Calculate how many fields fit in visible window
            let mut cumulative_height = 0;
            let mut visible_count = 0;
            for i in self.scroll_offset..self.fields.len() {
                let height = Self::calculate_field_height(&self.fields[i]);
                if cumulative_height + height <= self.visible_height {
                    cumulative_height += height;
                    visible_count += 1;
                } else {
                    break;
                }
            }

            let visible_end = self.scroll_offset + visible_count;
            if self.focused_index >= visible_end {
                // Focused field is below visible window, scroll down
                self.scroll_offset = self.focused_index.saturating_sub(visible_count - 1);
            }
        }
    }

    /// Start a new search
    pub fn start_search(&mut self, query: String) {
        let mut matches = Vec::new();
        for (field_idx, field) in self.fields.iter().enumerate() {
            // Search in label
            if let Some(pos) = field.label.to_lowercase().find(&query.to_lowercase()) {
                matches.push((field_idx, pos));
            }
            // Search in value
            if let Some(pos) = field.value.to_lowercase().find(&query.to_lowercase()) {
                matches.push((field_idx, pos));
            }
        }

        if !matches.is_empty() {
            self.search_state = Some(SearchState {
                query,
                matches,
                current_match: 0,
            });
            // Jump to first match
            if let Some(search) = &self.search_state {
                self.focused_index = search.matches[0].0;
                self.ensure_focused_visible();
            }
        } else {
            self.search_state = None;
        }
    }

    /// Jump to next search match
    pub fn next_match(&mut self) {
        if let Some(ref mut search) = self.search_state {
            if !search.matches.is_empty() {
                search.current_match = (search.current_match + 1) % search.matches.len();
                self.focused_index = search.matches[search.current_match].0;
                self.ensure_focused_visible();
            }
        }
    }

    /// Jump to previous search match
    pub fn prev_match(&mut self) {
        if let Some(ref mut search) = self.search_state {
            if !search.matches.is_empty() {
                search.current_match = if search.current_match == 0 {
                    search.matches.len() - 1
                } else {
                    search.current_match - 1
                };
                self.focused_index = search.matches[search.current_match].0;
                self.ensure_focused_visible();
            }
        }
    }

    /// Clear search state
    pub fn clear_search(&mut self) {
        self.search_state = None;
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Find the next field in the same horizontal group (for right arrow navigation)
    /// Returns None if not in a horizontal group or if at the end of the group
    pub fn find_next_in_horizontal_group(&self) -> Option<usize> {
        let current_field = self.focused_field()?;

        if let Some(LayoutHint::HorizontalGroup { group_id, .. }) = &current_field.layout_hint {
            // Find next field with the same group_id
            for i in (self.focused_index + 1)..self.fields.len() {
                if self.fields[i].hidden {
                    continue;
                }

                if let Some(LayoutHint::HorizontalGroup { group_id: next_group_id, .. }) = &self.fields[i].layout_hint {
                    if next_group_id == group_id {
                        return Some(i);
                    }
                }
                // If we hit a different group or full-width field, we're at the end of this group
                break;
            }
        }

        None
    }

    /// Find the previous field in the same horizontal group (for left arrow navigation)
    /// Returns None if not in a horizontal group or if at the start of the group
    pub fn find_prev_in_horizontal_group(&self) -> Option<usize> {
        let current_field = self.focused_field()?;

        if let Some(LayoutHint::HorizontalGroup { group_id, .. }) = &current_field.layout_hint {
            // Find previous field with the same group_id
            for i in (0..self.focused_index).rev() {
                if self.fields[i].hidden {
                    continue;
                }

                if let Some(LayoutHint::HorizontalGroup { group_id: prev_group_id, .. }) = &self.fields[i].layout_hint {
                    if prev_group_id == group_id {
                        return Some(i);
                    }
                }
                // If we hit a different group or full-width field, we're at the start of this group
                break;
            }
        }

        None
    }

    /// Move to the next field within the same horizontal group (right arrow)
    /// Returns true if moved, false if not in a group or at the end
    pub fn move_right_in_group(&mut self) -> bool {
        if let Some(next_idx) = self.find_next_in_horizontal_group() {
            self.focused_index = next_idx;
            self.cursor_position = 0;
            self.ensure_focused_visible();
            true
        } else {
            false
        }
    }

    /// Move to the previous field within the same horizontal group (left arrow)
    /// Returns true if moved, false if not in a group or at the start
    pub fn move_left_in_group(&mut self) -> bool {
        if let Some(prev_idx) = self.find_prev_in_horizontal_group() {
            self.focused_index = prev_idx;
            self.cursor_position = 0;
            self.ensure_focused_visible();
            true
        } else {
            false
        }
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

        // Update visible height in state (in field rows, not pixels)
        state.visible_height = inner.height as usize;

        // Filter visible fields (not hidden)
        let all_visible_indices: Vec<usize> = state
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.hidden)
            .map(|(i, _)| i)
            .collect();

        if all_visible_indices.is_empty() {
            if state.fields.is_empty() {
                let empty_msg = Paragraph::new("No fields defined").style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                );
                empty_msg.render(inner, buf);
            }
            return;
        }

        // Apply scroll windowing: only show fields within scroll window
        let scroll_offset = state.scroll_offset;
        let visible_indices: Vec<usize> = all_visible_indices
            .into_iter()
            .skip(scroll_offset)
            .collect();

        // Determine if we need scroll indicators
        let can_scroll_up = scroll_offset > 0;
        let can_scroll_down = scroll_offset + visible_indices.len() < state.fields.len();

        // Show scroll indicators
        if can_scroll_up {
            buf.set_string(
                inner.x + inner.width / 2,
                inner.y,
                "▲",
                Style::default().fg(Color::Yellow),
            );
        }
        if can_scroll_down {
            buf.set_string(
                inner.x + inner.width / 2,
                inner.y + inner.height - 1,
                "▼",
                Style::default().fg(Color::Yellow),
            );
        }

        // Calculate layout for visible fields
        let constraints: Vec<Constraint> = visible_indices
            .iter()
            .map(|&i| {
                let f = &state.fields[i];
                match f.field_type {
                    FieldType::TextArea => Constraint::Min(5),
                    FieldType::SectionHeader => Constraint::Length(2),
                    _ => Constraint::Length(3),
                }
            })
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Render visible fields
        for (chunk_idx, &field_idx) in visible_indices.iter().enumerate() {
            let field = &state.fields[field_idx];
            let chunk = &chunks[chunk_idx];
            let is_focused = field_idx == state.focused_index;

            // Handle SectionHeader specially (no border, centered text + horizontal line)
            if field.field_type == FieldType::SectionHeader {
                let header_text = Span::styled(
                    &field.label,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
                let header_line = Line::from(vec![Span::raw(" "), header_text, Span::raw(" ")]);

                // Render centered header
                let header_paragraph = Paragraph::new(header_line).centered();
                header_paragraph.render(*chunk, buf);

                // Render horizontal line below
                if chunk.height > 1 {
                    let line_area = Rect {
                        y: chunk.y + 1,
                        height: 1,
                        ..*chunk
                    };
                    let line = "─".repeat(line_area.width as usize);
                    buf.set_string(
                        line_area.x,
                        line_area.y,
                        line,
                        Style::default().fg(Color::DarkGray),
                    );
                }
                continue; // Skip normal field rendering
            }

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
                title = format!("{title} [from: {file_path}]");
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

            let content: Vec<Line> = if field.field_type == FieldType::RadioButton {
                // Render radio buttons horizontally: ( ) Option1  (•) Option2  ( ) Option3
                let mut spans = Vec::new();
                for (idx, option) in field.options.iter().enumerate() {
                    if idx > 0 {
                        spans.push(Span::raw("  "));
                    }
                    let is_selected = field.value == *option;
                    let marker = if is_selected { "(•)" } else { "( )" };
                    spans.push(Span::raw(marker));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        option.clone(),
                        if is_selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        },
                    ));
                }
                vec![Line::from(spans)]
            } else if field.value.is_empty() {
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
                        Line::from(Span::styled(format!("[!] {error}"), self.error_style));
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

        // Show help text for focused field if help is toggled
        if state.showing_help {
            if let Some(focused_field) = state.focused_field() {
                if let Some(ref help_text) = focused_field.help_text {
                    // Render help text at bottom of form area
                    let help_area = Rect {
                        x: inner.x,
                        y: inner.y + inner.height.saturating_sub(2),
                        width: inner.width,
                        height: 2,
                    };

                    let help_paragraph = Paragraph::new(vec![Line::from(Span::styled(
                        format!("Help (F1 to hide): {}", help_text),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::ITALIC),
                    ))])
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow)),
                    );

                    help_paragraph.render(help_area, buf);
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
    fn test_validation_auto_focuses_first_error() {
        let fields = vec![
            FormField::text("title", "Title"),
            FormField::text("description", "Description").required(),
            FormField::text("assignee", "Assignee").required(),
        ];
        let mut state = FormState::new(fields);

        // Start with focus on first field
        assert_eq!(state.focused_index, 0);

        // Validate - should fail and focus second field (first error)
        assert!(!state.validate());
        assert_eq!(state.focused_index, 1);

        // Fill second field, validate again - should focus third field
        state.set_value("description", "Test".to_string());
        assert!(!state.validate());
        assert_eq!(state.focused_index, 2);

        // Fill third field, validate - should succeed and keep focus
        state.set_value("assignee", "User".to_string());
        assert!(state.validate());
        assert_eq!(state.focused_index, 2); // Focus doesn't change on success
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
        let mut field = FormField::text("title", "Title").with_validation(ValidationRule::Required);

        assert!(!field.validate());
        assert!(field.error.is_some());

        field.value = "Test".to_string();
        assert!(field.validate());
        assert!(field.error.is_none());
    }

    #[test]
    fn test_validation_rule_enum() {
        let mut field =
            FormField::text("status", "Status").with_validation(ValidationRule::Enum(vec![
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
        let mut field = FormField::text("label", "Label").with_validation(ValidationRule::NoSpaces);

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
        let mut field =
            FormField::text("due_date", "Due Date").with_validation(ValidationRule::Date);

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
        let mut field =
            FormField::text("due_date", "Due Date").with_validation(ValidationRule::FutureDate);

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

    #[test]
    fn test_validation_rule_clone() {
        let rule1 = ValidationRule::Required;
        let rule2 = rule1.clone();
        assert_eq!(rule1, rule2);

        let rule3 = ValidationRule::Enum(vec!["a".to_string(), "b".to_string()]);
        let rule4 = rule3.clone();
        assert_eq!(rule3, rule4);
    }

    #[test]
    fn test_validation_rule_eq() {
        assert_eq!(ValidationRule::Required, ValidationRule::Required);
        assert_eq!(
            ValidationRule::PositiveInteger,
            ValidationRule::PositiveInteger
        );
        assert_ne!(ValidationRule::Required, ValidationRule::PositiveInteger);

        let enum1 = ValidationRule::Enum(vec!["a".to_string()]);
        let enum2 = ValidationRule::Enum(vec!["a".to_string()]);
        let enum3 = ValidationRule::Enum(vec!["b".to_string()]);
        assert_eq!(enum1, enum2);
        assert_ne!(enum1, enum3);
    }

    #[test]
    fn test_field_type_clone() {
        let ft1 = FieldType::Text;
        let ft2 = ft1.clone();
        assert_eq!(ft1, ft2);
    }

    #[test]
    fn test_field_type_eq() {
        assert_eq!(FieldType::Text, FieldType::Text);
        assert_eq!(FieldType::Password, FieldType::Password);
        assert_eq!(FieldType::TextArea, FieldType::TextArea);
        assert_eq!(FieldType::Selector, FieldType::Selector);
        assert_eq!(FieldType::ReadOnly, FieldType::ReadOnly);
        assert_ne!(FieldType::Text, FieldType::Password);
    }

    #[test]
    fn test_form_field_text_area() {
        let field = FormField::text_area("description", "Description");
        assert_eq!(field.id, "description");
        assert_eq!(field.label, "Description");
        assert_eq!(field.field_type, FieldType::TextArea);
        assert_eq!(field.value, "");
    }

    #[test]
    fn test_form_field_selector() {
        let options = vec!["open".to_string(), "closed".to_string()];
        let field = FormField::selector("status", "Status", options.clone());
        assert_eq!(field.id, "status");
        assert_eq!(field.label, "Status");
        assert_eq!(field.field_type, FieldType::Selector);
        assert_eq!(field.options, options);
    }

    #[test]
    fn test_form_field_read_only() {
        let field = FormField::read_only("id", "ID", "beads-1234-5678");
        assert_eq!(field.id, "id");
        assert_eq!(field.label, "ID");
        assert_eq!(field.field_type, FieldType::ReadOnly);
        assert_eq!(field.value, "beads-1234-5678");
    }

    #[test]
    fn test_form_field_placeholder() {
        let field = FormField::text("title", "Title").placeholder("Enter title...");
        assert_eq!(field.placeholder, Some("Enter title...".to_string()));
    }

    #[test]
    fn test_form_field_value() {
        let field = FormField::text("title", "Title").value("Initial Value");
        assert_eq!(field.value, "Initial Value");
    }

    #[test]
    fn test_form_field_clone() {
        let field = FormField::text("title", "Title")
            .required()
            .placeholder("Enter title...")
            .value("Test");

        let cloned = field.clone();
        assert_eq!(cloned.id, field.id);
        assert_eq!(cloned.label, field.label);
        assert_eq!(cloned.field_type, field.field_type);
        assert_eq!(cloned.value, field.value);
        assert_eq!(cloned.required, field.required);
        assert_eq!(cloned.placeholder, field.placeholder);
    }

    #[test]
    fn test_form_field_builder_chain() {
        let field = FormField::text("estimate", "Estimate")
            .required()
            .placeholder("Enter hours")
            .value("10")
            .with_validation(ValidationRule::PositiveInteger);

        assert!(field.required);
        assert_eq!(field.placeholder, Some("Enter hours".to_string()));
        assert_eq!(field.value, "10");
        assert_eq!(field.validation_rules.len(), 1);
    }

    #[test]
    fn test_form_state_empty_fields() {
        let state = FormState::new(vec![]);
        assert_eq!(state.fields.len(), 0);
        assert_eq!(state.focused_index, 0);
    }

    #[test]
    fn test_form_state_single_field() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);
        assert_eq!(state.focused_index(), 0);

        state.focus_next();
        assert_eq!(state.focused_index(), 0); // Can't go beyond last

        state.focus_previous();
        assert_eq!(state.focused_index(), 0); // Can't go below first
    }

    #[test]
    fn test_form_state_get_value_nonexistent() {
        let fields = vec![FormField::text("title", "Title")];
        let state = FormState::new(fields);
        assert_eq!(state.get_value("nonexistent"), None);
    }

    // === New comprehensive tests ===

    #[test]
    fn test_validation_rule_debug() {
        let rule = ValidationRule::Required;
        let debug = format!("{:?}", rule);
        assert!(debug.contains("Required"));

        let rule2 = ValidationRule::Enum(vec!["a".to_string()]);
        let debug2 = format!("{:?}", rule2);
        assert!(debug2.contains("Enum"));
    }

    #[test]
    fn test_field_type_debug() {
        let ft = FieldType::TextArea;
        let debug = format!("{:?}", ft);
        assert!(debug.contains("TextArea"));

        let ft2 = FieldType::Selector;
        let debug2 = format!("{:?}", ft2);
        assert!(debug2.contains("Selector"));
    }

    #[test]
    fn test_form_field_debug() {
        let field = FormField::text("title", "Title");
        let debug = format!("{:?}", field);
        assert!(debug.contains("FormField"));
        assert!(debug.contains("title"));
    }

    #[test]
    fn test_form_state_debug() {
        let fields = vec![FormField::text("title", "Title")];
        let state = FormState::new(fields);
        let debug = format!("{:?}", state);
        assert!(debug.contains("FormState"));
    }

    #[test]
    fn test_read_only_field_insert_protection() {
        let fields = vec![FormField::read_only("id", "ID", "beads-1234-5678")];
        let mut state = FormState::new(fields);

        state.insert_char('x');
        assert_eq!(state.get_value("id"), Some("beads-1234-5678"));
    }

    #[test]
    fn test_read_only_field_delete_protection() {
        let fields = vec![FormField::read_only("id", "ID", "beads-1234-5678")];
        let mut state = FormState::new(fields);
        state.cursor_position = 5;

        state.delete_char();
        assert_eq!(state.get_value("id"), Some("beads-1234-5678"));
    }

    #[test]
    fn test_text_field_newline_protection() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        state.insert_char('\n');
        assert_eq!(state.get_value("title"), Some(""));
    }

    #[test]
    fn test_text_area_allows_newlines() {
        let fields = vec![FormField::text_area("description", "Description")];
        let mut state = FormState::new(fields);

        state.insert_char('a');
        state.insert_char('\n');
        state.insert_char('b');
        assert_eq!(state.get_value("description"), Some("a\nb"));
    }

    #[test]
    fn test_cursor_at_start_boundary() {
        let fields = vec![FormField::text("title", "Title").value("Hello")];
        let mut state = FormState::new(fields);
        state.cursor_position = 0;

        state.move_cursor_left();
        assert_eq!(state.cursor_position(), 0);

        state.delete_char();
        assert_eq!(state.get_value("title"), Some("Hello"));
    }

    #[test]
    fn test_cursor_at_end_boundary() {
        let fields = vec![FormField::text("title", "Title").value("Hello")];
        let mut state = FormState::new(fields);
        state.cursor_position = 5;

        state.move_cursor_right();
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_insert_char_updates_cursor() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        assert_eq!(state.cursor_position(), 0);
        state.insert_char('a');
        assert_eq!(state.cursor_position(), 1);
        state.insert_char('b');
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_delete_char_updates_cursor() {
        let fields = vec![FormField::text("title", "Title").value("abc")];
        let mut state = FormState::new(fields);
        state.cursor_position = 3;

        state.delete_char();
        assert_eq!(state.cursor_position(), 2);
        assert_eq!(state.get_value("title"), Some("ab"));
    }

    #[test]
    fn test_focus_index_boundary_checks() {
        let fields = vec![
            FormField::text("f1", "Field 1"),
            FormField::text("f2", "Field 2"),
        ];
        let mut state = FormState::new(fields);

        state.set_focused_index(1);
        assert_eq!(state.focused_index(), 1);

        state.set_focused_index(5);
        assert_eq!(state.focused_index(), 1); // Should not change

        state.set_focused_index(0);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_focus_navigation_resets_cursor() {
        let fields = vec![
            FormField::text("f1", "Field 1").value("abc"),
            FormField::text("f2", "Field 2").value("def"),
        ];
        let mut state = FormState::new(fields);
        state.cursor_position = 3;

        state.focus_next();
        assert_eq!(state.cursor_position(), 0);

        state.cursor_position = 2;
        state.focus_previous();
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_set_focused_index_resets_cursor() {
        let fields = vec![
            FormField::text("f1", "Field 1").value("abc"),
            FormField::text("f2", "Field 2").value("def"),
        ];
        let mut state = FormState::new(fields);
        state.cursor_position = 3;

        state.set_focused_index(1);
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_validation_whitespace_vs_empty() {
        let mut field = FormField::text("title", "Title").with_validation(ValidationRule::Required);

        field.value = "   ".to_string();
        assert!(!field.validate());
        assert!(field.error.is_some());

        field.value = "".to_string();
        assert!(!field.validate());
        assert!(field.error.is_some());

        field.value = " a ".to_string();
        assert!(field.validate());
        assert!(field.error.is_none());
    }

    #[test]
    fn test_validation_beads_id_edge_cases() {
        let mut field = FormField::text("id", "ID").with_validation(ValidationRule::BeadsIdFormat);

        // Special characters in parts
        field.value = "beads-12@4-5678".to_string();
        assert!(!field.validate());

        // Too many parts
        field.value = "beads-1234-5678-9999".to_string();
        assert!(!field.validate());

        // Too few parts
        field.value = "beads-1234".to_string();
        assert!(!field.validate());

        // Correct length but wrong prefix
        field.value = "issue-1234-5678".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_positive_integer_edge_cases() {
        let mut field =
            FormField::text("value", "Value").with_validation(ValidationRule::PositiveInteger);

        // Decimal number
        field.value = "1.5".to_string();
        assert!(!field.validate());

        // Very large number
        field.value = "999999999999".to_string();
        assert!(field.validate());

        // Leading zeros
        field.value = "007".to_string();
        assert!(field.validate());

        // Plus sign (i64::parse accepts this)
        field.value = "+100".to_string();
        assert!(field.validate());
    }

    #[test]
    fn test_validation_relative_date_formats() {
        let mut field = FormField::text("date", "Date").with_validation(ValidationRule::Date);

        // Valid relative dates
        field.value = "1h".to_string();
        assert!(field.validate());

        field.value = "5d".to_string();
        assert!(field.validate());

        field.value = "2w".to_string();
        assert!(field.validate());

        field.value = "3m".to_string();
        assert!(field.validate());

        field.value = "1y".to_string();
        assert!(field.validate());

        // Invalid relative dates
        field.value = "d5".to_string();
        assert!(!field.validate());

        field.value = "1x".to_string();
        assert!(!field.validate());

        field.value = "abc".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_validation_enum_case_sensitive() {
        let mut field =
            FormField::text("status", "Status").with_validation(ValidationRule::Enum(vec![
                "open".to_string(),
                "closed".to_string(),
            ]));

        field.value = "Open".to_string();
        assert!(!field.validate());

        field.value = "OPEN".to_string();
        assert!(!field.validate());

        field.value = "open".to_string();
        assert!(field.validate());
    }

    #[test]
    fn test_selector_field_with_empty_options() {
        let field = FormField::selector("status", "Status", vec![]);
        assert_eq!(field.options.len(), 0);
        assert_eq!(field.field_type, FieldType::Selector);
    }

    #[test]
    fn test_form_widget_default() {
        let form = Form::default();
        assert_eq!(form.title, None);
        assert!(form.block.is_some());
    }

    #[test]
    fn test_form_widget_builder_title() {
        let form = Form::new().title("User Settings");
        assert_eq!(form.title, Some("User Settings"));
    }

    #[test]
    fn test_form_widget_builder_style() {
        let style = Style::default().fg(Color::Yellow);
        let form = Form::new().style(style);
        assert_eq!(form.style, style);
    }

    #[test]
    fn test_form_widget_builder_focused_style() {
        let style = Style::default().fg(Color::Green);
        let form = Form::new().focused_style(style);
        assert_eq!(form.focused_style, style);
    }

    #[test]
    fn test_form_widget_builder_error_style() {
        let style = Style::default().fg(Color::Red);
        let form = Form::new().error_style(style);
        assert_eq!(form.error_style, style);
    }

    #[test]
    fn test_form_widget_builder_block() {
        let block = Block::default().title("Custom Block");
        let form = Form::new().block(block.clone());
        assert!(form.block.is_some());
    }

    #[test]
    fn test_form_widget_builder_chain() {
        let form = Form::new()
            .title("Settings")
            .style(Style::default().fg(Color::White))
            .focused_style(Style::default().fg(Color::Cyan))
            .error_style(Style::default().fg(Color::Red));

        assert_eq!(form.title, Some("Settings"));
    }

    #[test]
    fn test_form_field_with_multiple_validation_rules_order() {
        let mut field = FormField::text("label", "Label")
            .with_validation(ValidationRule::Required)
            .with_validation(ValidationRule::NoSpaces);

        // Empty fails on Required first
        field.value = "".to_string();
        assert!(!field.validate());
        assert!(field.error.as_ref().unwrap().contains("required"));

        // With spaces fails on NoSpaces
        field.value = "bug fix".to_string();
        assert!(!field.validate());
        assert!(field.error.as_ref().unwrap().contains("spaces"));

        // Valid passes all
        field.value = "bug-fix".to_string();
        assert!(field.validate());
    }

    #[test]
    fn test_get_field_mut_updates_field() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        if let Some(field) = state.get_field_mut("title") {
            field.value = "Updated".to_string();
        }

        assert_eq!(state.get_value("title"), Some("Updated"));
    }

    #[test]
    fn test_focused_field_mut_updates_field() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        if let Some(field) = state.focused_field_mut() {
            field.value = "Modified".to_string();
        }

        assert_eq!(state.get_value("title"), Some("Modified"));
    }

    #[test]
    fn test_fields_accessor() {
        let fields = vec![
            FormField::text("f1", "Field 1"),
            FormField::text("f2", "Field 2"),
        ];
        let state = FormState::new(fields);

        let all_fields = state.fields();
        assert_eq!(all_fields.len(), 2);
        assert_eq!(all_fields[0].id, "f1");
        assert_eq!(all_fields[1].id, "f2");
    }

    #[test]
    fn test_fields_mut_accessor() {
        let fields = vec![FormField::text("f1", "Field 1")];
        let mut state = FormState::new(fields);

        let all_fields = state.fields_mut();
        all_fields[0].value = "Changed".to_string();

        assert_eq!(state.get_value("f1"), Some("Changed"));
    }

    #[test]
    fn test_clear_errors_removes_all_errors() {
        let fields = vec![
            FormField::text("f1", "Field 1").with_validation(ValidationRule::Required),
            FormField::text("f2", "Field 2").with_validation(ValidationRule::Required),
        ];
        let mut state = FormState::new(fields);

        state.validate();
        assert!(state.has_errors());

        state.clear_errors();
        assert!(!state.has_errors());
    }

    #[test]
    fn test_password_field_builder() {
        let field = FormField::password("pwd", "Password")
            .required()
            .placeholder("Enter password");

        assert_eq!(field.field_type, FieldType::Password);
        assert!(field.required);
        assert_eq!(field.placeholder, Some("Enter password".to_string()));
    }

    #[test]
    fn test_insert_char_at_middle_position() {
        let fields = vec![FormField::text("title", "Title").value("ac")];
        let mut state = FormState::new(fields);
        state.cursor_position = 1;

        state.insert_char('b');
        assert_eq!(state.get_value("title"), Some("abc"));
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_delete_char_at_middle_position() {
        let fields = vec![FormField::text("title", "Title").value("abc")];
        let mut state = FormState::new(fields);
        state.cursor_position = 2;

        state.delete_char();
        assert_eq!(state.get_value("title"), Some("ac"));
        assert_eq!(state.cursor_position(), 1);
    }

    #[test]
    fn test_validation_no_spaces_allows_underscores_and_dashes() {
        let mut field = FormField::text("label", "Label").with_validation(ValidationRule::NoSpaces);

        field.value = "bug_fix".to_string();
        assert!(field.validate());

        field.value = "bug-fix".to_string();
        assert!(field.validate());

        field.value = "bug fix".to_string();
        assert!(!field.validate());
    }

    #[test]
    fn test_insert_char_triggers_validation() {
        // Test that insert_char() calls validation and shows errors in real-time
        let fields = vec![FormField::text("title", "Title").required()];
        let mut state = FormState::new(fields);

        // Initially empty, should have no error
        assert_eq!(state.focused_field().unwrap().error, None);

        // Insert a character - validation should run and clear any error
        state.insert_char('H');
        assert_eq!(state.focused_field().unwrap().value, "H");
        assert_eq!(state.focused_field().unwrap().error, None);

        // Delete the character - should trigger validation and show error
        state.delete_char();
        assert_eq!(state.focused_field().unwrap().value, "");
        assert!(state.focused_field().unwrap().error.is_some());
        assert!(state
            .focused_field()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("required"));
    }

    #[test]
    fn test_delete_char_triggers_validation() {
        // Test that delete_char() calls validation
        let fields = vec![FormField::text("title", "Title").required()];
        let mut state = FormState::new(fields);

        // Add some valid content
        state.insert_char('H');
        state.insert_char('i');
        assert_eq!(state.focused_field().unwrap().value, "Hi");
        assert_eq!(state.focused_field().unwrap().error, None);

        // Delete one char - still valid
        state.delete_char();
        assert_eq!(state.focused_field().unwrap().value, "H");
        assert_eq!(state.focused_field().unwrap().error, None);

        // Delete last char - should show error
        state.delete_char();
        assert_eq!(state.focused_field().unwrap().value, "");
        assert!(state.focused_field().unwrap().error.is_some());
    }

    #[test]
    fn test_validation_clears_errors_when_input_becomes_valid() {
        // Test that validation clears errors when the input becomes valid
        let fields = vec![FormField::text("estimate", "Estimate")
            .with_validation(ValidationRule::PositiveInteger)];
        let mut state = FormState::new(fields);

        // Enter invalid input
        state.insert_char('-');
        state.insert_char('5');
        assert_eq!(state.focused_field().unwrap().value, "-5");
        assert!(state.focused_field().unwrap().error.is_some());

        // Delete to make it valid
        state.move_cursor_to_start();
        state.delete_char(); // This won't delete because cursor at start
        assert_eq!(state.cursor_position(), 0);

        // Move cursor and delete the '-'
        state.move_cursor_right();
        state.delete_char();
        assert_eq!(state.focused_field().unwrap().value, "5");
        assert_eq!(state.focused_field().unwrap().error, None);
    }

    #[test]
    fn test_validation_on_rapid_input() {
        // Test that rapid typing triggers validation on each character
        let fields =
            vec![FormField::text("label", "Label").with_validation(ValidationRule::NoSpaces)];
        let mut state = FormState::new(fields);

        // Type valid characters rapidly
        for c in "bug-fix".chars() {
            state.insert_char(c);
        }
        assert_eq!(state.focused_field().unwrap().value, "bug-fix");
        assert_eq!(state.focused_field().unwrap().error, None);

        // Add a space (invalid)
        state.insert_char(' ');
        assert_eq!(state.focused_field().unwrap().value, "bug-fix ");
        assert!(state.focused_field().unwrap().error.is_some());
        assert!(state
            .focused_field()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("spaces"));
    }

    #[test]
    fn test_validation_preserves_errors_across_fields() {
        // Test that validation errors are preserved when switching fields
        let fields = vec![
            FormField::text("title", "Title").required(),
            FormField::text("assignee", "Assignee"),
        ];
        let mut state = FormState::new(fields);

        // Leave first field empty (error)
        state.focus_next();

        // Manually validate to set the error
        state.fields[0].validate();

        // Check the first field has an error
        assert!(state.fields[0].error.is_some());

        // Type in second field
        state.insert_char('A');
        assert_eq!(state.focused_field().unwrap().value, "A");

        // First field should still have its error
        assert!(state.fields[0].error.is_some());
        assert!(state.fields[0].error.as_ref().unwrap().contains("required"));
    }

    #[test]
    fn test_form_field_help_text() {
        let field = FormField::text("date", "Date").help_text("Format: YYYY-MM-DD");
        assert_eq!(field.help_text, Some("Format: YYYY-MM-DD".to_string()));
    }

    #[test]
    fn test_form_state_help_toggle() {
        let fields = vec![FormField::text("title", "Title").help_text("Enter the issue title")];
        let mut state = FormState::new(fields);

        assert!(!state.is_showing_help());
        state.toggle_help();
        assert!(state.is_showing_help());
        state.toggle_help();
        assert!(!state.is_showing_help());
    }

    #[test]
    fn test_form_state_hide_help() {
        let fields = vec![FormField::text("title", "Title")];
        let mut state = FormState::new(fields);

        state.toggle_help();
        assert!(state.is_showing_help());
        state.hide_help();
        assert!(!state.is_showing_help());
    }
}
