//! Compact form widget with horizontal field grouping for beads-tui

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use super::form::{FieldType, FormState, LayoutHint};

/// Compact form widget that renders fields inline with horizontal grouping
pub struct CompactForm<'a> {
    block: Option<Block<'a>>,
    focused_style: Style,
    unfocused_style: Style,
    readonly_style: Style,
    error_style: Style,
}

impl<'a> CompactForm<'a> {
    /// Create a new compact form
    pub fn new() -> Self {
        Self {
            block: None,
            focused_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
            unfocused_style: Style::default().fg(Color::Gray),
            readonly_style: Style::default().fg(Color::DarkGray),
            error_style: Style::default().fg(Color::Red),
        }
    }

    /// Set the outer block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set focused field style
    pub fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    /// Set unfocused field style
    pub fn unfocused_style(mut self, style: Style) -> Self {
        self.unfocused_style = style;
        self
    }

    /// Set read-only field style
    pub fn readonly_style(mut self, style: Style) -> Self {
        self.readonly_style = style;
        self
    }

    /// Set error style
    pub fn error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }
}

impl<'a> Default for CompactForm<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a row of fields in the compact form
#[derive(Debug)]
struct FieldRow {
    /// Field indices in this row
    field_indices: Vec<usize>,
    /// Whether this is a horizontal group or single full-width field
    is_horizontal: bool,
    /// Group ID if horizontal group
    group_id: Option<String>,
}

impl CompactForm<'_> {
    /// Group fields into rows based on layout hints
    fn group_fields_into_rows(state: &FormState) -> Vec<FieldRow> {
        let mut rows = Vec::new();
        let mut current_group: Option<String> = None;
        let mut current_group_indices = Vec::new();

        for (idx, field) in state.fields().iter().enumerate() {
            if field.hidden {
                continue;
            }

            match &field.layout_hint {
                Some(LayoutHint::HorizontalGroup { group_id, .. }) => {
                    if let Some(ref current) = current_group {
                        if current == group_id {
                            // Continue current group
                            current_group_indices.push(idx);
                        } else {
                            // Start new group, flush current
                            if !current_group_indices.is_empty() {
                                rows.push(FieldRow {
                                    field_indices: current_group_indices.clone(),
                                    is_horizontal: true,
                                    group_id: current_group.clone(),
                                });
                                current_group_indices.clear();
                            }
                            current_group = Some(group_id.clone());
                            current_group_indices.push(idx);
                        }
                    } else {
                        // Start first group
                        current_group = Some(group_id.clone());
                        current_group_indices.push(idx);
                    }
                }
                Some(LayoutHint::FullWidth) | None => {
                    // Flush any pending group
                    if !current_group_indices.is_empty() {
                        rows.push(FieldRow {
                            field_indices: current_group_indices.clone(),
                            is_horizontal: true,
                            group_id: current_group.clone(),
                        });
                        current_group_indices.clear();
                        current_group = None;
                    }
                    // Add full-width field as its own row
                    rows.push(FieldRow {
                        field_indices: vec![idx],
                        is_horizontal: false,
                        group_id: None,
                    });
                }
            }
        }

        // Flush any remaining group
        if !current_group_indices.is_empty() {
            rows.push(FieldRow {
                field_indices: current_group_indices,
                is_horizontal: true,
                group_id: current_group,
            });
        }

        rows
    }

    /// Render an inline field: "LABEL:value"
    fn render_inline_field(
        &self,
        field_idx: usize,
        state: &FormState,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let field = &state.fields()[field_idx];
        let is_focused = field_idx == state.focused_index();

        // Determine style
        let style = if field.error.is_some() {
            self.error_style
        } else if is_focused {
            self.focused_style
        } else if field.field_type == FieldType::ReadOnly {
            self.readonly_style
        } else {
            self.unfocused_style
        };

        // Build the inline text: "LABEL:value"
        let label_part = format!("{}:", field.label.to_uppercase());
        let value_part = if field.value.is_empty() {
            // Show placeholder with blocks
            "░░░░░░░░░░".to_string()
        } else {
            // Truncate if too long to fit
            let available = area.width.saturating_sub(label_part.len() as u16 + 2);
            if field.value.len() > available as usize {
                format!("{}…", &field.value[..available.saturating_sub(1) as usize])
            } else {
                field.value.clone()
            }
        };

        // Add field type indicator
        let indicator = match field.field_type {
            FieldType::Selector => "▼",
            FieldType::ReadOnly if field.value.contains('T') => "‡", // Date indicator
            _ => "",
        };

        let mut spans = vec![
            Span::styled(label_part, style),
            Span::styled(value_part, style),
        ];

        if !indicator.is_empty() {
            spans.push(Span::styled(indicator, style));
        }

        // Render cursor if focused
        if is_focused {
            spans.push(Span::styled(" │", style));
        }

        let line = Line::from(spans);
        line.render(area, buf);
    }

    /// Render a section header as a horizontal divider
    fn render_section_header(&self, field_idx: usize, state: &FormState, area: Rect, buf: &mut Buffer) {
        let field = &state.fields()[field_idx];

        // Calculate centered position
        let text_width = field.label.len() as u16 + 2;
        let padding = (area.width.saturating_sub(text_width)) / 2;

        buf.set_string(
            area.x + padding,
            area.y,
            format!(" {} ", field.label),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        );

        // Render horizontal line below
        if area.height > 1 {
            let line = "─".repeat(area.width as usize);
            buf.set_string(
                area.x,
                area.y + 1,
                line,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

impl<'a> StatefulWidget for CompactForm<'a> {
    type State = FormState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render outer block if present
        let inner = if let Some(block) = self.block.take() {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        // Group fields into rows
        let rows = Self::group_fields_into_rows(state);

        if rows.is_empty() {
            return;
        }

        // Calculate row heights
        let row_constraints: Vec<Constraint> = rows
            .iter()
            .map(|row| {
                let first_field_idx = row.field_indices[0];
                let field = &state.fields()[first_field_idx];
                match field.field_type {
                    FieldType::SectionHeader => Constraint::Length(2),
                    FieldType::TextArea => Constraint::Min(5),
                    _ => Constraint::Length(1),
                }
            })
            .collect();

        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(inner);

        // Render each row
        for (row_idx, row) in rows.iter().enumerate() {
            if row_idx >= row_areas.len() {
                break;
            }

            let row_area = row_areas[row_idx];

            // Check if this is a section header
            let first_field = &state.fields()[row.field_indices[0]];
            if first_field.field_type == FieldType::SectionHeader {
                self.render_section_header(row.field_indices[0], state, row_area, buf);
                continue;
            }

            if row.is_horizontal {
                // Render horizontal group
                let field_constraints: Vec<Constraint> = row
                    .field_indices
                    .iter()
                    .map(|&idx| {
                        let field = &state.fields()[idx];
                        if let Some(LayoutHint::HorizontalGroup { width, .. }) = &field.layout_hint {
                            width.clone()
                        } else {
                            Constraint::Percentage(100 / row.field_indices.len() as u16)
                        }
                    })
                    .collect();

                let field_areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(field_constraints)
                    .split(row_area);

                for (field_area_idx, &field_idx) in row.field_indices.iter().enumerate() {
                    if field_area_idx < field_areas.len() {
                        self.render_inline_field(field_idx, state, field_areas[field_area_idx], buf);
                    }
                }
            } else {
                // Render full-width field
                let field_idx = row.field_indices[0];
                let field = &state.fields()[field_idx];

                if field.field_type == FieldType::TextArea {
                    // Render text area with border
                    let is_focused = field_idx == state.focused_index();
                    let style = if is_focused {
                        self.focused_style
                    } else {
                        self.unfocused_style
                    };

                    let title = if is_focused {
                        format!("{} [editing]", field.label)
                    } else {
                        field.label.clone()
                    };

                    let field_block = Block::default()
                        .borders(ratatui::widgets::Borders::ALL)
                        .title(title)
                        .style(style);

                    let field_inner = field_block.inner(row_area);
                    field_block.render(row_area, buf);

                    // Render content
                    if field_inner.height > 0 {
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
                            field.value.lines().map(|l| Line::from(l.to_string())).collect()
                        };

                        let paragraph = ratatui::widgets::Paragraph::new(content);
                        paragraph.render(field_inner, buf);
                    }
                } else {
                    // Render inline full-width field
                    self.render_inline_field(field_idx, state, row_area, buf);
                }
            }
        }
    }
}
