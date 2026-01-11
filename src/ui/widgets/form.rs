//! Multi-field form widget for beads-tui

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Form field type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Single-line text input
    Text,
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

    /// Validate the field
    pub fn validate(&mut self) -> bool {
        if self.required && self.value.trim().is_empty() {
            self.error = Some(format!("{} is required", self.label));
            false
        } else {
            self.error = None;
            true
        }
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
            let title = if field.required {
                format!("{} *", field.label)
            } else {
                field.label.clone()
            };

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
}
