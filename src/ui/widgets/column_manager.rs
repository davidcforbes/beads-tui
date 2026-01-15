//! Column manager widget for hiding/showing/reordering table columns

use crate::models::table_config::ColumnDefinition;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidget, Widget},
};

/// Column manager action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnManagerAction {
    /// Move selected column up
    MoveUp,
    /// Move selected column down
    MoveDown,
    /// Toggle visibility of selected column
    ToggleVisibility,
    /// Reset to default configuration
    Reset,
    /// Apply changes and close
    Apply,
    /// Cancel without applying
    Cancel,
}

/// Column manager state
#[derive(Debug)]
pub struct ColumnManagerState {
    /// Current column definitions (working copy)
    columns: Vec<ColumnDefinition>,
    /// Selected column index
    selected: usize,
    /// Whether changes were made
    modified: bool,
}

impl ColumnManagerState {
    /// Create a new column manager state from current columns
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self {
            columns,
            selected: 0,
            modified: false,
        }
    }

    /// Get the current columns
    pub fn columns(&self) -> &[ColumnDefinition] {
        &self.columns
    }

    /// Get the selected column index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.columns.len().saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.columns.len().max(1);
    }

    /// Move selected column up in the list
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.columns.swap(self.selected, self.selected - 1);
            self.selected -= 1;
            self.modified = true;
        }
    }

    /// Move selected column down in the list
    pub fn move_down(&mut self) {
        if self.selected < self.columns.len() - 1 {
            self.columns.swap(self.selected, self.selected + 1);
            self.selected += 1;
            self.modified = true;
        }
    }

    /// Toggle visibility of selected column
    pub fn toggle_visibility(&mut self) {
        if let Some(col) = self.columns.get_mut(self.selected) {
            // Don't allow hiding mandatory columns
            if !col.id.is_mandatory() {
                col.visible = !col.visible;
                self.modified = true;
            }
        }
    }

    /// Reset to default columns
    pub fn reset(&mut self, default_columns: Vec<ColumnDefinition>) {
        self.columns = default_columns;
        self.selected = 0;
        self.modified = true;
    }

    /// Check if changes were made
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Apply action
    pub fn apply_action(&mut self, action: ColumnManagerAction) -> bool {
        match action {
            ColumnManagerAction::MoveUp => {
                self.move_up();
                false
            }
            ColumnManagerAction::MoveDown => {
                self.move_down();
                false
            }
            ColumnManagerAction::ToggleVisibility => {
                self.toggle_visibility();
                false
            }
            ColumnManagerAction::Reset => {
                // Reset requires caller to provide default columns
                // Return true to signal it needs handling
                true
            }
            ColumnManagerAction::Apply | ColumnManagerAction::Cancel => true,
        }
    }
}

/// Column manager widget
pub struct ColumnManager<'a> {
    title: &'a str,
    show_help: bool,
}

impl<'a> ColumnManager<'a> {
    /// Create a new column manager
    pub fn new() -> Self {
        Self {
            title: "Column Manager",
            show_help: true,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Show or hide help
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }
}

impl<'a> Default for ColumnManager<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for ColumnManager<'a> {
    type State = ColumnManagerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create main layout
        let chunks = if self.show_help {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),   // Column list
                    Constraint::Length(4), // Help text
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10)])
                .split(area)
        };

        // Create list items
        let items: Vec<ListItem> = state
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let is_selected = idx == state.selected;
                let visibility = if col.visible { "x" } else { " " };
                let mandatory = if col.id.is_mandatory() { " *" } else { "" };
                let width_info = format!("({})", col.width);

                let content = format!(
                    "[{}] {:12} {:>6} {}",
                    visibility, col.label, width_info, mandatory
                );

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if !col.visible {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        // Create the list widget
        let list_title = if state.modified {
            format!("{} (modified)", self.title)
        } else {
            self.title.to_string()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title.as_str())
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        // Render the list
        Widget::render(list, chunks[0], buf);

        // Render help text
        if self.show_help && chunks.len() > 1 {
            let help_lines = vec![
                Line::from(vec![
                    Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
                    Span::raw(" Select  "),
                    Span::styled("Alt+Left/Right", Style::default().fg(Color::Yellow)),
                    Span::raw(" Move  "),
                    Span::styled("Space", Style::default().fg(Color::Green)),
                    Span::raw(" Toggle  "),
                ]),
                Line::from(vec![
                    Span::styled("r", Style::default().fg(Color::Cyan)),
                    Span::raw(" Reset  "),
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::raw(" Apply  "),
                    Span::styled("Esc", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
                Line::from(vec![Span::styled(
                    "* = Mandatory column (cannot be hidden)",
                    Style::default().fg(Color::Gray),
                )]),
            ];

            let help = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .alignment(Alignment::Left);

            help.render(chunks[1], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::table_config::ColumnId;

    fn create_test_columns() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition::new(ColumnId::Id),
            ColumnDefinition::new(ColumnId::Title),
            ColumnDefinition::new(ColumnId::Status),
            ColumnDefinition::new(ColumnId::Priority),
        ]
    }

    #[test]
    fn test_column_manager_state_creation() {
        let cols = create_test_columns();
        let state = ColumnManagerState::new(cols);
        assert_eq!(state.columns().len(), 4);
        assert_eq!(state.selected(), 0);
        assert!(!state.is_modified());
    }

    #[test]
    fn test_select_next_previous() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        assert_eq!(state.selected(), 0);

        state.select_next();
        assert_eq!(state.selected(), 1);

        state.select_next();
        assert_eq!(state.selected(), 2);

        state.select_previous();
        assert_eq!(state.selected(), 1);

        state.select_previous();
        assert_eq!(state.selected(), 0);

        // Wrap around
        state.select_previous();
        assert_eq!(state.selected(), 3);

        state.select_next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_move_up_down() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 2; // Select Status

        // Move up
        state.move_up();
        assert_eq!(state.selected(), 1);
        assert_eq!(state.columns()[1].id, ColumnId::Status);
        assert_eq!(state.columns()[2].id, ColumnId::Title);
        assert!(state.is_modified());

        // Move down
        state.move_down();
        assert_eq!(state.selected(), 2);
        assert_eq!(state.columns()[2].id, ColumnId::Status);
    }

    #[test]
    fn test_toggle_visibility() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 2; // Select Status (non-mandatory)

        assert!(state.columns()[2].visible);

        state.toggle_visibility();
        assert!(!state.columns()[2].visible);
        assert!(state.is_modified());

        state.toggle_visibility();
        assert!(state.columns()[2].visible);
    }

    #[test]
    fn test_cannot_hide_mandatory_columns() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 0; // Select Id (mandatory)

        assert!(state.columns()[0].visible);

        state.toggle_visibility();
        // Should still be visible
        assert!(state.columns()[0].visible);
        assert!(!state.is_modified()); // No change
    }

    #[test]
    fn test_reset() {
        let mut cols = create_test_columns();
        cols[2].visible = false; // Hide Status

        let mut state = ColumnManagerState::new(cols);
        assert!(!state.columns()[2].visible);

        // Reset to defaults
        let defaults = create_test_columns();
        state.reset(defaults);

        assert!(state.columns()[2].visible);
        assert_eq!(state.selected(), 0);
        assert!(state.is_modified());
    }

    #[test]
    fn test_column_manager_action_equality() {
        assert_eq!(ColumnManagerAction::MoveUp, ColumnManagerAction::MoveUp);
        assert_eq!(ColumnManagerAction::MoveDown, ColumnManagerAction::MoveDown);
        assert_eq!(
            ColumnManagerAction::ToggleVisibility,
            ColumnManagerAction::ToggleVisibility
        );
        assert_eq!(ColumnManagerAction::Reset, ColumnManagerAction::Reset);
        assert_eq!(ColumnManagerAction::Apply, ColumnManagerAction::Apply);
        assert_eq!(ColumnManagerAction::Cancel, ColumnManagerAction::Cancel);

        assert_ne!(ColumnManagerAction::MoveUp, ColumnManagerAction::MoveDown);
        assert_ne!(ColumnManagerAction::Apply, ColumnManagerAction::Cancel);
    }

    #[test]
    fn test_apply_action_move_up() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        state.selected = 1;

        let should_close = state.apply_action(ColumnManagerAction::MoveUp);
        assert!(!should_close); // Move actions don't close
        assert_eq!(state.selected(), 0);
        assert!(state.is_modified());
    }

    #[test]
    fn test_apply_action_move_down() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        state.selected = 1;

        let should_close = state.apply_action(ColumnManagerAction::MoveDown);
        assert!(!should_close);
        assert_eq!(state.selected(), 2);
        assert!(state.is_modified());
    }

    #[test]
    fn test_apply_action_toggle_visibility() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        state.selected = 2;

        let should_close = state.apply_action(ColumnManagerAction::ToggleVisibility);
        assert!(!should_close);
        assert!(!state.columns()[2].visible);
        assert!(state.is_modified());
    }

    #[test]
    fn test_apply_action_reset() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        let should_close = state.apply_action(ColumnManagerAction::Reset);
        assert!(should_close); // Reset needs caller handling
    }

    #[test]
    fn test_apply_action_apply() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        let should_close = state.apply_action(ColumnManagerAction::Apply);
        assert!(should_close); // Apply closes the dialog
    }

    #[test]
    fn test_apply_action_cancel() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        let should_close = state.apply_action(ColumnManagerAction::Cancel);
        assert!(should_close); // Cancel closes the dialog
    }

    #[test]
    fn test_move_up_at_top() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 0;
        state.move_up();

        // Should not move when at the top
        assert_eq!(state.selected(), 0);
        assert!(!state.is_modified());
    }

    #[test]
    fn test_move_down_at_bottom() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 3; // Last column
        state.move_down();

        // Should not move when at the bottom
        assert_eq!(state.selected(), 3);
        assert!(!state.is_modified());
    }

    #[test]
    fn test_select_next_wraparound() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 3; // Last column
        state.select_next();

        // Should wrap to first column
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_select_previous_wraparound() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 0;
        state.select_previous();

        // Should wrap to last column
        assert_eq!(state.selected(), 3);
    }

    #[test]
    fn test_toggle_visibility_out_of_bounds() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 999; // Out of bounds
        state.toggle_visibility();

        // Should not panic or modify
        assert!(!state.is_modified());
    }

    #[test]
    fn test_columns_accessor() {
        let cols = create_test_columns();
        let state = ColumnManagerState::new(cols.clone());

        let columns = state.columns();
        assert_eq!(columns.len(), 4);
        assert_eq!(columns[0].id, ColumnId::Id);
        assert_eq!(columns[1].id, ColumnId::Title);
    }

    #[test]
    fn test_selected_accessor() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        assert_eq!(state.selected(), 0);

        state.selected = 2;
        assert_eq!(state.selected(), 2);
    }

    #[test]
    fn test_column_manager_new() {
        let manager = ColumnManager::new();
        assert_eq!(manager.title, "Column Manager");
        assert!(manager.show_help);
    }

    #[test]
    fn test_column_manager_default() {
        let manager = ColumnManager::default();
        assert_eq!(manager.title, "Column Manager");
        assert!(manager.show_help);
    }

    #[test]
    fn test_column_manager_title() {
        let manager = ColumnManager::new().title("Custom Title");
        assert_eq!(manager.title, "Custom Title");
    }

    #[test]
    fn test_column_manager_show_help() {
        let manager = ColumnManager::new().show_help(false);
        assert!(!manager.show_help);
    }

    #[test]
    fn test_column_manager_builder_chain() {
        let manager = ColumnManager::new().title("Test Manager").show_help(false);

        assert_eq!(manager.title, "Test Manager");
        assert!(!manager.show_help);
    }

    #[test]
    fn test_multiple_toggles() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        state.selected = 2;

        for _ in 0..4 {
            let before = state.columns()[2].visible;
            state.toggle_visibility();
            assert_ne!(before, state.columns()[2].visible);
        }

        // After even number of toggles, should be back to visible
        assert!(state.columns()[2].visible);
    }

    #[test]
    fn test_select_next_with_empty_list() {
        let state = ColumnManagerState::new(vec![]);
        let mut state_copy = state;

        state_copy.select_next();
        // Should not panic with empty list
        assert_eq!(state_copy.selected(), 0);
    }

    #[test]
    fn test_reset_clears_selection() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 3;
        state.reset(create_test_columns());

        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_modified_flag_preserved_across_operations() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        assert!(!state.is_modified());

        state.move_down();
        assert!(state.is_modified());

        state.move_down();
        assert!(state.is_modified()); // Still modified

        state.move_up();
        assert!(state.is_modified()); // Still modified after reverting
    }

    #[test]
    fn test_column_manager_action_debug() {
        let action = ColumnManagerAction::MoveUp;
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "MoveUp");

        let action = ColumnManagerAction::ToggleVisibility;
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "ToggleVisibility");
    }

    #[test]
    fn test_column_manager_action_clone() {
        let action = ColumnManagerAction::Apply;
        let cloned = action;
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_column_manager_action_copy() {
        let action = ColumnManagerAction::Cancel;
        let copied = action;
        assert_eq!(action, copied);
    }

    #[test]
    fn test_column_manager_state_debug() {
        let cols = create_test_columns();
        let state = ColumnManagerState::new(cols);
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("ColumnManagerState"));
    }

    #[test]
    fn test_widget_rendering_with_help() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        let manager = ColumnManager::new().show_help(true);

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content with help");
    }

    #[test]
    fn test_widget_rendering_without_help() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        let manager = ColumnManager::new().show_help(false);

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content without help");
    }

    #[test]
    fn test_widget_rendering_modified_state() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        state.move_down(); // Modify state

        let manager = ColumnManager::new();
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render modified state");
        assert!(state.is_modified());
    }

    #[test]
    fn test_widget_rendering_small_area() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        let manager = ColumnManager::new();

        let area = Rect::new(0, 0, 20, 10);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render in small area");
    }

    #[test]
    fn test_widget_rendering_custom_title() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);
        let manager = ColumnManager::new().title("Custom Columns");

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render with custom title");
    }

    #[test]
    fn test_select_previous_with_empty_list() {
        let state = ColumnManagerState::new(vec![]);
        let mut state_copy = state;

        state_copy.select_previous();
        // Should not panic with empty list and should wrap to 0 (saturating_sub)
        assert_eq!(state_copy.selected(), 0);
    }

    #[test]
    fn test_multiple_move_up_operations() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 3; // Last column

        state.move_up();
        assert_eq!(state.selected(), 2);
        assert_eq!(state.columns()[2].id, ColumnId::Priority);

        state.move_up();
        assert_eq!(state.selected(), 1);

        state.move_up();
        assert_eq!(state.selected(), 0);

        // Should not move further
        state.move_up();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_multiple_move_down_operations() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        state.selected = 0; // First column

        state.move_down();
        assert_eq!(state.selected(), 1);

        state.move_down();
        assert_eq!(state.selected(), 2);

        state.move_down();
        assert_eq!(state.selected(), 3);

        // Should not move further
        state.move_down();
        assert_eq!(state.selected(), 3);
    }

    #[test]
    fn test_all_action_variants_debug() {
        let actions = [
            ColumnManagerAction::MoveUp,
            ColumnManagerAction::MoveDown,
            ColumnManagerAction::ToggleVisibility,
            ColumnManagerAction::Reset,
            ColumnManagerAction::Apply,
            ColumnManagerAction::Cancel,
        ];

        for action in &actions {
            let debug_str = format!("{:?}", action);
            assert!(!debug_str.is_empty(), "Debug format should not be empty");
        }
    }

    #[test]
    fn test_widget_rendering_empty_columns() {
        let mut state = ColumnManagerState::new(vec![]);
        let manager = ColumnManager::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        manager.render(area, &mut buffer, &mut state);

        // Should not panic with empty columns
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render with empty columns");
    }

    #[test]
    fn test_reset_with_different_columns() {
        let mut cols = create_test_columns();
        cols[1].visible = false; // Hide Title
        cols[2].visible = false; // Hide Status

        let mut state = ColumnManagerState::new(cols);
        assert!(!state.columns()[1].visible);
        assert!(!state.columns()[2].visible);

        state.selected = 3;
        state.move_down(); // Try to trigger modified

        // Reset with different defaults (all visible)
        let defaults = create_test_columns();
        state.reset(defaults);

        assert!(state.columns()[1].visible);
        assert!(state.columns()[2].visible);
        assert_eq!(state.selected(), 0);
        assert!(state.is_modified());
    }

    #[test]
    fn test_toggle_visibility_multiple_columns() {
        let cols = create_test_columns();
        let mut state = ColumnManagerState::new(cols);

        // Toggle Status (non-mandatory)
        state.selected = 2;
        state.toggle_visibility();
        assert!(!state.columns()[2].visible);

        // Toggle Priority (non-mandatory)
        state.selected = 3;
        state.toggle_visibility();
        assert!(!state.columns()[3].visible);

        // Both should be hidden
        assert!(!state.columns()[2].visible);
        assert!(!state.columns()[3].visible);
        assert!(state.is_modified());
    }
}
