//! Dialog for adding dependencies between issues

use crate::ui::widgets::{Autocomplete, AutocompleteState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

/// Dependency type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    /// Current issue depends on selected issue (selected blocks current)
    DependsOn,
    /// Current issue blocks selected issue (current blocks selected)
    Blocks,
    /// Bidirectional "see also" relationship between issues
    RelatesTo,
}

impl DependencyType {
    /// Get the display name
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::DependsOn => "Depends On",
            DependencyType::Blocks => "Blocks",
            DependencyType::RelatesTo => "Relates To",
        }
    }

    /// Get the help text
    pub fn help_text(&self) -> &'static str {
        match self {
            DependencyType::DependsOn => "This issue depends on (is blocked by) the selected issue",
            DependencyType::Blocks => "This issue blocks the selected issue",
            DependencyType::RelatesTo => {
                "Bidirectional 'see also' relationship (both issues reference each other)"
            }
        }
    }

    /// Toggle between types
    pub fn toggle(&self) -> Self {
        match self {
            DependencyType::DependsOn => DependencyType::Blocks,
            DependencyType::Blocks => DependencyType::RelatesTo,
            DependencyType::RelatesTo => DependencyType::DependsOn,
        }
    }
}

/// Focus in the dependency dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyDialogFocus {
    /// Focus on issue ID input
    IssueId,
    /// Focus on dependency type selector
    Type,
    /// Focus on buttons
    Buttons,
}

/// State for the add dependency dialog
#[derive(Debug, Clone)]
pub struct DependencyDialogState {
    /// Autocomplete state for issue ID
    pub autocomplete_state: AutocompleteState,
    /// Selected dependency type
    pub dependency_type: DependencyType,
    /// Current focus
    pub focus: DependencyDialogFocus,
    /// Selected button (0 = OK, 1 = Cancel)
    pub selected_button: usize,
    /// Whether dialog is open
    pub is_open: bool,
}

impl Default for DependencyDialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyDialogState {
    /// Create a new dependency dialog state
    pub fn new() -> Self {
        Self {
            autocomplete_state: AutocompleteState::new(),
            dependency_type: DependencyType::DependsOn,
            focus: DependencyDialogFocus::IssueId,
            selected_button: 0,
            is_open: false,
        }
    }

    /// Open the dialog with available issue IDs
    pub fn open(&mut self, issue_ids: Vec<String>) {
        self.is_open = true;
        self.focus = DependencyDialogFocus::IssueId;
        self.selected_button = 0;
        self.autocomplete_state.set_options(issue_ids);
        self.autocomplete_state.set_focused(true);
        self.autocomplete_state.clear_selected();
    }

    /// Close the dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.autocomplete_state.set_focused(false);
        self.autocomplete_state.clear_selected();
    }

    /// Check if dialog is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the selected issue ID
    pub fn selected_issue_id(&self) -> Option<String> {
        self.autocomplete_state
            .selected_value()
            .map(|s| s.to_string())
    }

    /// Get the current input (even if not confirmed)
    pub fn current_input(&self) -> &str {
        self.autocomplete_state.input()
    }

    /// Get the selected dependency type
    pub fn dependency_type(&self) -> DependencyType {
        self.dependency_type
    }

    /// Toggle dependency type
    pub fn toggle_type(&mut self) {
        self.dependency_type = self.dependency_type.toggle();
    }

    /// Move focus to next field
    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            DependencyDialogFocus::IssueId => {
                self.autocomplete_state.set_focused(false);
                DependencyDialogFocus::Type
            }
            DependencyDialogFocus::Type => DependencyDialogFocus::Buttons,
            DependencyDialogFocus::Buttons => {
                self.autocomplete_state.set_focused(true);
                DependencyDialogFocus::IssueId
            }
        };
    }

    /// Move focus to previous field
    pub fn focus_previous(&mut self) {
        self.focus = match self.focus {
            DependencyDialogFocus::IssueId => DependencyDialogFocus::Buttons,
            DependencyDialogFocus::Type => {
                self.autocomplete_state.set_focused(true);
                DependencyDialogFocus::IssueId
            }
            DependencyDialogFocus::Buttons => {
                self.autocomplete_state.set_focused(false);
                DependencyDialogFocus::Type
            }
        };
    }

    /// Get current focus
    pub fn focus(&self) -> DependencyDialogFocus {
        self.focus
    }

    /// Select next button
    pub fn select_next_button(&mut self) {
        self.selected_button = (self.selected_button + 1) % 2;
    }

    /// Select previous button
    pub fn select_previous_button(&mut self) {
        self.selected_button = if self.selected_button == 0 { 1 } else { 0 };
    }

    /// Get selected button (0 = OK, 1 = Cancel)
    pub fn selected_button(&self) -> usize {
        self.selected_button
    }

    /// Check if OK button is selected
    pub fn is_ok_selected(&self) -> bool {
        self.selected_button == 0
    }

    /// Check if Cancel button is selected
    pub fn is_cancel_selected(&self) -> bool {
        self.selected_button == 1
    }
}

/// Widget for adding dependencies
pub struct DependencyDialog<'a> {
    /// Title of the current issue
    current_issue_title: &'a str,
    /// Block style
    block_style: Style,
}

impl<'a> DependencyDialog<'a> {
    /// Create a new dependency dialog
    pub fn new(current_issue_title: &'a str) -> Self {
        Self {
            current_issue_title,
            block_style: Style::default().fg(Color::Cyan),
        }
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Calculate centered rect
    fn centered_rect(&self, width: u16, height: u16, area: Rect) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(height)) / 2),
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(width)) / 2),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(vertical[1])[1]
    }
}

impl<'a> StatefulWidget for DependencyDialog<'a> {
    type State = DependencyDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.is_open {
            return;
        }

        // Clear the background
        Clear.render(area, buf);

        // Calculate dialog size and position
        let dialog_width = 70.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));
        let dialog_area = self.centered_rect(dialog_width, dialog_height, area);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Current issue info
                Constraint::Length(3), // Type selector
                Constraint::Length(3), // Help text
                Constraint::Min(5),    // Autocomplete
                Constraint::Length(3), // Buttons
            ])
            .margin(1)
            .split(dialog_area);

        // Render outer block
        Block::default()
            .title(" Add Dependency ")
            .borders(Borders::ALL)
            .style(self.block_style)
            .render(dialog_area, buf);

        // Render current issue info
        let info_text = format!("Current Issue: {}", self.current_issue_title);
        let info = Paragraph::new(info_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("From"));
        info.render(chunks[0], buf);

        // Render dependency type selector
        let type_focused = state.focus == DependencyDialogFocus::Type;
        let type_style = if type_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let type_selector = Paragraph::new(Line::from(vec![
            Span::styled(
                if state.dependency_type == DependencyType::DependsOn {
                    "> Depends On"
                } else {
                    "  Depends On"
                },
                if state.dependency_type == DependencyType::DependsOn && type_focused {
                    type_style
                } else if state.dependency_type == DependencyType::DependsOn {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::raw("    "),
            Span::styled(
                if state.dependency_type == DependencyType::Blocks {
                    "> Blocks"
                } else {
                    "  Blocks"
                },
                if state.dependency_type == DependencyType::Blocks && type_focused {
                    type_style
                } else if state.dependency_type == DependencyType::Blocks {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::raw("    "),
            Span::styled(
                if state.dependency_type == DependencyType::RelatesTo {
                    "> Relates To"
                } else {
                    "  Relates To"
                },
                if state.dependency_type == DependencyType::RelatesTo && type_focused {
                    type_style
                } else if state.dependency_type == DependencyType::RelatesTo {
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Relationship Type")
                .border_style(if type_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                }),
        );
        type_selector.render(chunks[1], buf);

        // Render help text
        let help_style = Style::default().fg(Color::DarkGray);
        let help = Paragraph::new(state.dependency_type.help_text())
            .style(help_style)
            .block(Block::default().borders(Borders::ALL).title("Explanation"));
        help.render(chunks[2], buf);

        // Render autocomplete for issue ID
        let autocomplete_focused = state.focus == DependencyDialogFocus::IssueId;
        let autocomplete = Autocomplete::new()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Select Issue ")
                    .border_style(if autocomplete_focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .placeholder("Type to search for an issue ID...")
            .max_suggestions(8);

        autocomplete.render(chunks[3], buf, &mut state.autocomplete_state);

        // Render buttons
        let buttons_focused = state.focus == DependencyDialogFocus::Buttons;
        let ok_style = if buttons_focused && state.selected_button == 0 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let cancel_style = if buttons_focused && state.selected_button == 1 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let buttons = Paragraph::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(" OK ", ok_style),
            Span::raw("        "),
            Span::styled(" Cancel ", cancel_style),
        ]))
        .alignment(Alignment::Center);

        buttons.render(chunks[4], buf);

        // Render keyboard hints at the bottom
        let hints = if state.focus == DependencyDialogFocus::IssueId {
            "Tab: Next | Shift+Tab: Previous | Up/Down: Navigate | Enter: Select | Esc: Cancel"
        } else if state.focus == DependencyDialogFocus::Type {
            "Tab: Next | Shift+Tab: Previous | Space: Toggle | Enter: Confirm | Esc: Cancel"
        } else {
            "Tab: Next | Shift+Tab: Previous | Left/Right: Navigate | Enter: Confirm | Esc: Cancel"
        };

        let hint_line = Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)));
        let hint_para = Paragraph::new(hint_line).alignment(Alignment::Center);

        // Render hints below the dialog
        if dialog_area.bottom() < area.bottom() {
            let hint_area = Rect {
                x: dialog_area.x,
                y: dialog_area.bottom(),
                width: dialog_area.width,
                height: 1,
            };
            hint_para.render(hint_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_type_as_str() {
        assert_eq!(DependencyType::DependsOn.as_str(), "Depends On");
        assert_eq!(DependencyType::Blocks.as_str(), "Blocks");
        assert_eq!(DependencyType::RelatesTo.as_str(), "Relates To");
    }

    #[test]
    fn test_dependency_type_toggle() {
        assert_eq!(DependencyType::DependsOn.toggle(), DependencyType::Blocks);
        assert_eq!(DependencyType::Blocks.toggle(), DependencyType::RelatesTo);
        assert_eq!(
            DependencyType::RelatesTo.toggle(),
            DependencyType::DependsOn
        );
    }

    #[test]
    fn test_dependency_dialog_state_new() {
        let state = DependencyDialogState::new();
        assert!(!state.is_open());
        assert_eq!(state.focus(), DependencyDialogFocus::IssueId);
        assert_eq!(state.selected_button(), 0);
    }

    #[test]
    fn test_dependency_dialog_state_open_close() {
        let mut state = DependencyDialogState::new();
        assert!(!state.is_open());

        state.open(vec!["issue-1".to_string(), "issue-2".to_string()]);
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
    }

    #[test]
    fn test_dependency_dialog_state_toggle_type() {
        let mut state = DependencyDialogState::new();
        assert_eq!(state.dependency_type(), DependencyType::DependsOn);

        state.toggle_type();
        assert_eq!(state.dependency_type(), DependencyType::Blocks);

        state.toggle_type();
        assert_eq!(state.dependency_type(), DependencyType::RelatesTo);

        state.toggle_type();
        assert_eq!(state.dependency_type(), DependencyType::DependsOn);
    }

    #[test]
    fn test_dependency_dialog_state_focus_navigation() {
        let mut state = DependencyDialogState::new();
        assert_eq!(state.focus(), DependencyDialogFocus::IssueId);

        state.focus_next();
        assert_eq!(state.focus(), DependencyDialogFocus::Type);

        state.focus_next();
        assert_eq!(state.focus(), DependencyDialogFocus::Buttons);

        state.focus_next();
        assert_eq!(state.focus(), DependencyDialogFocus::IssueId);

        state.focus_previous();
        assert_eq!(state.focus(), DependencyDialogFocus::Buttons);
    }

    #[test]
    fn test_dependency_dialog_state_button_selection() {
        let mut state = DependencyDialogState::new();
        assert_eq!(state.selected_button(), 0);
        assert!(state.is_ok_selected());
        assert!(!state.is_cancel_selected());

        state.select_next_button();
        assert_eq!(state.selected_button(), 1);
        assert!(!state.is_ok_selected());
        assert!(state.is_cancel_selected());

        state.select_previous_button();
        assert_eq!(state.selected_button(), 0);
    }
}
