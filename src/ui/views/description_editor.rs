//! Full-screen description editor view

use crate::ui::widgets::{TextEditor, TextEditorState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Description editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Normal editing mode
    Normal,
    /// Insert mode (typing)
    Insert,
    /// Command mode (for save/cancel)
    Command,
}

/// Description editor state
#[derive(Debug, Clone)]
pub struct DescriptionEditorState {
    editor_state: TextEditorState,
    mode: EditorMode,
    title: String,
    help_visible: bool,
    modified: bool,
    saved: bool,
    cancelled: bool,
}

impl DescriptionEditorState {
    /// Create a new description editor state
    pub fn new(title: String, initial_text: String) -> Self {
        let mut editor_state = TextEditorState::new();
        if !initial_text.is_empty() {
            editor_state.set_text(initial_text);
        }
        editor_state.set_focused(true);

        Self {
            editor_state,
            mode: EditorMode::Insert,
            title,
            help_visible: true,
            modified: false,
            saved: false,
            cancelled: false,
        }
    }

    /// Get the editor state
    pub fn editor_state(&self) -> &TextEditorState {
        &self.editor_state
    }

    /// Get mutable editor state
    pub fn editor_state_mut(&mut self) -> &mut TextEditorState {
        &mut self.editor_state
    }

    /// Get current mode
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// Get title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Check if help is visible
    pub fn is_help_visible(&self) -> bool {
        self.help_visible
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// Check if content has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark as modified
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// Check if saved
    pub fn is_saved(&self) -> bool {
        self.saved
    }

    /// Mark as saved
    pub fn save(&mut self) {
        self.saved = true;
        self.modified = false;
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Get the edited text
    pub fn text(&self) -> String {
        self.editor_state.text()
    }

    /// Insert character and mark as modified
    pub fn insert_char(&mut self, c: char) {
        self.editor_state.insert_char(c);
        self.modified = true;
    }

    /// Delete character and mark as modified
    pub fn delete_char(&mut self) {
        self.editor_state.delete_char();
        self.modified = true;
    }

    /// Insert newline and mark as modified
    pub fn insert_newline(&mut self) {
        self.editor_state.insert_char('\n');
        self.modified = true;
    }

    /// Delete current line (move to start, delete chars to end)
    pub fn delete_line(&mut self) {
        self.editor_state.move_cursor_to_line_start();
        let line_len = self.editor_state.lines()[self.editor_state.cursor_position().0].len();
        for _ in 0..line_len {
            self.editor_state.delete_char_forward();
        }
        self.modified = true;
    }

    /// Move cursor up
    pub fn move_cursor_up(&mut self) {
        self.editor_state.move_cursor_up();
    }

    /// Move cursor down
    pub fn move_cursor_down(&mut self) {
        self.editor_state.move_cursor_down();
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        self.editor_state.move_cursor_left();
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        self.editor_state.move_cursor_right();
    }

    /// Move to start of line
    pub fn move_to_line_start(&mut self) {
        self.editor_state.move_cursor_to_line_start();
    }

    /// Move to end of line
    pub fn move_to_line_end(&mut self) {
        self.editor_state.move_cursor_to_line_end();
    }
}

/// Full-screen description editor view
pub struct DescriptionEditorView<'a> {
    show_help: bool,
    show_stats: bool,
    block_style: Style,
    help_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> DescriptionEditorView<'a> {
    /// Create a new description editor view
    pub fn new() -> Self {
        Self {
            show_help: true,
            show_stats: true,
            block_style: Style::default().fg(Color::Cyan),
            help_style: Style::default().fg(Color::DarkGray),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Show or hide help
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    /// Show or hide statistics
    pub fn show_stats(mut self, show: bool) -> Self {
        self.show_stats = show;
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Set help style
    pub fn help_style(mut self, style: Style) -> Self {
        self.help_style = style;
        self
    }

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer, state: &DescriptionEditorState) {
        let title_text = if state.is_modified() {
            format!("{} [modified]", state.title())
        } else {
            state.title().to_string()
        };

        let mode_text = match state.mode() {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
        };

        let mode_color = match state.mode() {
            EditorMode::Normal => Color::Cyan,
            EditorMode::Insert => Color::Green,
            EditorMode::Command => Color::Yellow,
        };

        let line = Line::from(vec![
            Span::styled(
                &title_text,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                mode_text,
                Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
            ),
        ]);

        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }

    fn render_help_bar(&self, area: Rect, buf: &mut Buffer, state: &DescriptionEditorState) {
        if !self.show_help || !state.is_help_visible() {
            return;
        }

        let help_text = match state.mode() {
            EditorMode::Normal | EditorMode::Insert => {
                "Ctrl+S: Save | Ctrl+Q: Cancel | Ctrl+H: Toggle Help | Arrow Keys: Navigate"
            }
            EditorMode::Command => "Enter: Confirm | Esc: Cancel",
        };

        let line = Line::from(Span::styled(help_text, self.help_style));
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }

    fn render_stats_bar(&self, area: Rect, buf: &mut Buffer, state: &DescriptionEditorState) {
        if !self.show_stats {
            return;
        }

        let line_count = state.editor_state().line_count();
        let (cursor_line, cursor_col) = state.editor_state().cursor_position();
        let char_count = state.editor_state().text().len();

        let stats_text = format!(
            "Lines: {} | Pos: {}:{} | Chars: {}",
            line_count,
            cursor_line + 1,
            cursor_col + 1,
            char_count
        );

        let line = Line::from(Span::styled(
            stats_text,
            Style::default().fg(Color::DarkGray),
        ));
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

impl<'a> Default for DescriptionEditorView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for DescriptionEditorView<'a> {
    type State = DescriptionEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout: title bar (1) + editor (fill) + help bar (1) + stats bar (1)
        let mut constraints = vec![Constraint::Length(1)]; // Title bar

        // Editor area
        constraints.push(Constraint::Min(5));

        // Help bar (if visible)
        if self.show_help && state.is_help_visible() {
            constraints.push(Constraint::Length(1));
        }

        // Stats bar (if visible)
        if self.show_stats {
            constraints.push(Constraint::Length(1));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_idx = 0;

        // Render title bar
        self.render_title_bar(chunks[chunk_idx], buf, state);
        chunk_idx += 1;

        // Render editor
        let editor_block = Block::default()
            .borders(Borders::ALL)
            .title("Description")
            .style(self.block_style);

        let editor = TextEditor::new().block(editor_block);

        StatefulWidget::render(editor, chunks[chunk_idx], buf, &mut state.editor_state);
        chunk_idx += 1;

        // Render help bar if visible
        if self.show_help && state.is_help_visible() {
            self.render_help_bar(chunks[chunk_idx], buf, state);
            chunk_idx += 1;
        }

        // Render stats bar if visible
        if self.show_stats {
            self.render_stats_bar(chunks[chunk_idx], buf, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_editor_state_creation() {
        let state = DescriptionEditorState::new("Test Issue".to_string(), String::new());
        assert_eq!(state.title(), "Test Issue");
        assert_eq!(state.mode(), EditorMode::Insert);
        assert!(!state.is_modified());
        assert!(!state.is_saved());
        assert!(!state.is_cancelled());
    }

    #[test]
    fn test_description_editor_state_with_initial_text() {
        let initial_text = "This is a test description.";
        let state = DescriptionEditorState::new("Test Issue".to_string(), initial_text.to_string());
        assert_eq!(state.text(), initial_text);
    }

    #[test]
    fn test_mode_switching() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert_eq!(state.mode(), EditorMode::Insert);

        state.set_mode(EditorMode::Normal);
        assert_eq!(state.mode(), EditorMode::Normal);

        state.set_mode(EditorMode::Command);
        assert_eq!(state.mode(), EditorMode::Command);
    }

    #[test]
    fn test_modification_tracking() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(!state.is_modified());

        state.insert_char('a');
        assert!(state.is_modified());

        state.save();
        assert!(!state.is_modified());
        assert!(state.is_saved());
    }

    #[test]
    fn test_help_toggle() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(state.is_help_visible());

        state.toggle_help();
        assert!(!state.is_help_visible());

        state.toggle_help();
        assert!(state.is_help_visible());
    }

    #[test]
    fn test_save() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_char('a');
        assert!(state.is_modified());
        assert!(!state.is_saved());

        state.save();
        assert!(!state.is_modified());
        assert!(state.is_saved());
    }

    #[test]
    fn test_cancel() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(!state.is_cancelled());

        state.cancel();
        assert!(state.is_cancelled());
    }

    #[test]
    fn test_text_editing_marks_modified() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        state.insert_char('h');
        assert!(state.is_modified());

        state.set_modified(false);
        state.delete_char();
        assert!(state.is_modified());

        state.set_modified(false);
        state.insert_newline();
        assert!(state.is_modified());

        state.set_modified(false);
        state.delete_line();
        assert!(state.is_modified());
    }

    #[test]
    fn test_cursor_movement_does_not_mark_modified() {
        let mut state =
            DescriptionEditorState::new("Test".to_string(), "Line 1\nLine 2".to_string());
        state.set_modified(false);

        state.move_cursor_up();
        assert!(!state.is_modified());

        state.move_cursor_down();
        assert!(!state.is_modified());

        state.move_cursor_left();
        assert!(!state.is_modified());

        state.move_cursor_right();
        assert!(!state.is_modified());

        state.move_to_line_start();
        assert!(!state.is_modified());

        state.move_to_line_end();
        assert!(!state.is_modified());
    }

    // EditorMode tests
    #[test]
    fn test_editor_mode_equality() {
        assert_eq!(EditorMode::Normal, EditorMode::Normal);
        assert_ne!(EditorMode::Normal, EditorMode::Insert);
        assert_ne!(EditorMode::Insert, EditorMode::Command);
    }

    #[test]
    fn test_editor_mode_clone() {
        let mode = EditorMode::Insert;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    // DescriptionEditorState additional tests
    #[test]
    fn test_description_editor_state_clone() {
        let state = DescriptionEditorState::new("Test".to_string(), "content".to_string());
        let cloned = state.clone();
        assert_eq!(cloned.title(), state.title());
        assert_eq!(cloned.text(), state.text());
        assert_eq!(cloned.mode(), state.mode());
    }

    #[test]
    fn test_empty_initial_text() {
        let state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert_eq!(state.text(), "");
    }

    #[test]
    fn test_multiline_initial_text() {
        let text = "Line 1\nLine 2\nLine 3";
        let state = DescriptionEditorState::new("Test".to_string(), text.to_string());
        assert_eq!(state.text(), text);
    }

    #[test]
    fn test_set_modified_explicitly() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(!state.is_modified());

        state.set_modified(true);
        assert!(state.is_modified());

        state.set_modified(false);
        assert!(!state.is_modified());
    }

    #[test]
    fn test_save_clears_modified_flag() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_char('x');
        assert!(state.is_modified());

        state.save();
        assert!(!state.is_modified());
        assert!(state.is_saved());
    }

    #[test]
    fn test_cancel_after_save() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.save();
        state.cancel();
        assert!(state.is_saved());
        assert!(state.is_cancelled());
    }

    #[test]
    fn test_multiple_saves() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.save();
        assert!(state.is_saved());

        state.insert_char('a');
        assert!(state.is_modified());

        state.save();
        assert!(!state.is_modified());
        assert!(state.is_saved());
    }

    #[test]
    fn test_editor_state_access() {
        let mut state = DescriptionEditorState::new("Test".to_string(), "content".to_string());
        
        // Test immutable access
        let editor_state = state.editor_state();
        assert!(!editor_state.text().is_empty());

        // Test mutable access
        let editor_state_mut = state.editor_state_mut();
        editor_state_mut.insert_char('x');
    }

    #[test]
    fn test_all_editor_modes() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        
        state.set_mode(EditorMode::Normal);
        assert_eq!(state.mode(), EditorMode::Normal);

        state.set_mode(EditorMode::Insert);
        assert_eq!(state.mode(), EditorMode::Insert);

        state.set_mode(EditorMode::Command);
        assert_eq!(state.mode(), EditorMode::Command);
    }

    #[test]
    fn test_title_getter() {
        let state = DescriptionEditorState::new("My Test Title".to_string(), String::new());
        assert_eq!(state.title(), "My Test Title");
    }

    #[test]
    fn test_long_title() {
        let long_title = "A".repeat(200);
        let state = DescriptionEditorState::new(long_title.clone(), String::new());
        assert_eq!(state.title(), &long_title);
    }

    #[test]
    fn test_unicode_title() {
        let title = "Fix 🐛 in 日本語 feature";
        let state = DescriptionEditorState::new(title.to_string(), String::new());
        assert_eq!(state.title(), title);
    }

    // DescriptionEditorView tests
    #[test]
    fn test_description_editor_view_new() {
        let view = DescriptionEditorView::new();
        assert!(view.show_help);
        assert!(view.show_stats);
    }

    #[test]
    fn test_description_editor_view_default() {
        let view = DescriptionEditorView::default();
        assert!(view.show_help);
        assert!(view.show_stats);
    }

    #[test]
    fn test_description_editor_view_show_help() {
        let view = DescriptionEditorView::new().show_help(false);
        assert!(!view.show_help);

        let view = DescriptionEditorView::new().show_help(true);
        assert!(view.show_help);
    }

    #[test]
    fn test_description_editor_view_show_stats() {
        let view = DescriptionEditorView::new().show_stats(false);
        assert!(!view.show_stats);

        let view = DescriptionEditorView::new().show_stats(true);
        assert!(view.show_stats);
    }

    #[test]
    fn test_description_editor_view_block_style() {
        let style = Style::default().fg(Color::Red);
        let view = DescriptionEditorView::new().block_style(style);
        assert_eq!(view.block_style.fg, Some(Color::Red));
    }

    #[test]
    fn test_description_editor_view_help_style() {
        let style = Style::default().fg(Color::Yellow);
        let view = DescriptionEditorView::new().help_style(style);
        assert_eq!(view.help_style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_description_editor_view_builder_chain() {
        let view = DescriptionEditorView::new()
            .show_help(false)
            .show_stats(false)
            .block_style(Style::default().fg(Color::Blue))
            .help_style(Style::default().fg(Color::Green));

        assert!(!view.show_help);
        assert!(!view.show_stats);
        assert_eq!(view.block_style.fg, Some(Color::Blue));
        assert_eq!(view.help_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_insert_multiple_chars() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_char('h');
        state.insert_char('i');
        assert!(state.text().contains('h'));
        assert!(state.text().contains('i'));
        assert!(state.is_modified());
    }

    #[test]
    fn test_delete_from_empty() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.delete_char(); // Should not panic on empty text
        assert_eq!(state.text(), "");
    }

    #[test]
    fn test_insert_newline_marks_modified() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_newline();
        assert!(state.is_modified());
        assert!(state.text().contains('\n'));
    }

    #[test]
    fn test_initial_mode_is_insert() {
        let state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert_eq!(state.mode(), EditorMode::Insert);
    }

    // Copy trait tests
    #[test]
    fn test_editor_mode_copy_trait() {
        let mode1 = EditorMode::Normal;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
        // Both should still be usable after copy
        assert_eq!(mode1, EditorMode::Normal);
        assert_eq!(mode2, EditorMode::Normal);
    }

    // Complex text editing scenarios
    #[test]
    fn test_complex_text_editing_sequence() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        state.insert_char('H');
        state.insert_char('e');
        state.insert_char('l');
        state.insert_char('l');
        state.insert_char('o');
        assert!(state.is_modified());

        state.save();
        assert!(!state.is_modified());

        state.insert_newline();
        state.insert_char('W');
        state.insert_char('o');
        state.insert_char('r');
        state.insert_char('l');
        state.insert_char('d');

        assert!(state.is_modified());
        assert!(state.text().contains("Hello"));
        assert!(state.text().contains("World"));
    }

    #[test]
    fn test_delete_line_on_empty_content() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.delete_line(); // Should not panic on empty content
        assert_eq!(state.text(), "");
        assert!(state.is_modified()); // delete_line marks as modified
    }

    #[test]
    fn test_delete_line_with_content() {
        let mut state = DescriptionEditorState::new("Test".to_string(), "Line to delete".to_string());
        state.set_modified(false);
        state.delete_line();
        assert!(state.is_modified());
    }

    #[test]
    fn test_multiple_mode_switches() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        state.set_mode(EditorMode::Normal);
        assert_eq!(state.mode(), EditorMode::Normal);

        state.set_mode(EditorMode::Command);
        assert_eq!(state.mode(), EditorMode::Command);

        state.set_mode(EditorMode::Insert);
        assert_eq!(state.mode(), EditorMode::Insert);

        state.set_mode(EditorMode::Normal);
        assert_eq!(state.mode(), EditorMode::Normal);
    }

    // View builder pattern variations
    #[test]
    fn test_view_builder_chain_order_independence() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);

        let view1 = DescriptionEditorView::new()
            .show_help(false)
            .block_style(style1)
            .help_style(style2);

        let view2 = DescriptionEditorView::new()
            .block_style(style1)
            .help_style(style2)
            .show_help(false);

        assert_eq!(view1.show_help, view2.show_help);
        assert_eq!(view1.block_style.fg, view2.block_style.fg);
        assert_eq!(view1.help_style.fg, view2.help_style.fg);
    }

    #[test]
    fn test_view_multiple_style_applications() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Green);
        let style3 = Style::default().fg(Color::Blue);

        let view = DescriptionEditorView::new()
            .block_style(style1)
            .block_style(style2)
            .block_style(style3);

        // Last style should win
        assert_eq!(view.block_style.fg, Some(Color::Blue));
    }

    // Empty title edge case
    #[test]
    fn test_empty_title() {
        let state = DescriptionEditorState::new("".to_string(), String::new());
        assert_eq!(state.title(), "");
    }

    #[test]
    fn test_title_with_special_characters() {
        let title = "Fix: Handle edge case with 'quotes' & \"symbols\"";
        let state = DescriptionEditorState::new(title.to_string(), String::new());
        assert_eq!(state.title(), title);
    }

    #[test]
    fn test_title_with_newlines() {
        let title = "Title\nwith\nnewlines";
        let state = DescriptionEditorState::new(title.to_string(), String::new());
        assert_eq!(state.title(), title);
    }

    // Multiple help toggles
    #[test]
    fn test_multiple_help_toggles() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(state.is_help_visible());

        for _ in 0..10 {
            state.toggle_help();
            state.toggle_help();
        }

        assert!(state.is_help_visible()); // Should be back to true after even number of toggles
    }

    // State after multiple operations
    #[test]
    fn test_state_after_multiple_edits_and_saves() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        state.insert_char('a');
        assert!(state.is_modified());

        state.save();
        assert!(state.is_saved());
        assert!(!state.is_modified());

        state.insert_char('b');
        assert!(state.is_modified());

        state.save();
        assert!(state.is_saved());
        assert!(!state.is_modified());
    }

    // Modified flag persistence across operations
    #[test]
    fn test_modified_flag_persistence() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        state.insert_char('x');
        assert!(state.is_modified());

        state.move_cursor_left();
        assert!(state.is_modified()); // Cursor movement should not clear modified flag

        state.move_cursor_right();
        assert!(state.is_modified());
    }

    // All EditorMode inequalities
    #[test]
    fn test_all_editor_mode_inequalities() {
        assert_ne!(EditorMode::Normal, EditorMode::Insert);
        assert_ne!(EditorMode::Normal, EditorMode::Command);
        assert_ne!(EditorMode::Insert, EditorMode::Normal);
        assert_ne!(EditorMode::Insert, EditorMode::Command);
        assert_ne!(EditorMode::Command, EditorMode::Normal);
        assert_ne!(EditorMode::Command, EditorMode::Insert);
    }

    // Cancel without save
    #[test]
    fn test_cancel_without_save() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_char('x');
        state.cancel();
        assert!(state.is_cancelled());
        assert!(state.is_modified()); // Cancel doesn't clear modified flag
        assert!(!state.is_saved());
    }

    // Save without modifications
    #[test]
    fn test_save_without_modifications() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        assert!(!state.is_modified());

        state.save();
        assert!(state.is_saved());
        assert!(!state.is_modified());
    }

    // Multiple cancels
    #[test]
    fn test_multiple_cancels() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.cancel();
        assert!(state.is_cancelled());

        state.cancel();
        assert!(state.is_cancelled()); // Should remain cancelled
    }

    // Text with special characters
    #[test]
    fn test_initial_text_with_special_chars() {
        let text = "Text with\ttabs\nand\nnewlines\rand special: 🎉";
        let state = DescriptionEditorState::new("Test".to_string(), text.to_string());
        assert_eq!(state.text(), text);
    }

    // Delete character marks modified
    #[test]
    fn test_delete_char_marks_modified() {
        let mut state = DescriptionEditorState::new("Test".to_string(), "abc".to_string());
        state.set_modified(false);
        state.delete_char();
        assert!(state.is_modified());
    }

    // Insert newline on empty text
    #[test]
    fn test_insert_newline_on_empty() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.insert_newline();
        assert!(state.is_modified());
        assert_eq!(state.text(), "\n");
    }

    // View default equals new
    #[test]
    fn test_view_default_equals_new() {
        let view1 = DescriptionEditorView::default();
        let view2 = DescriptionEditorView::new();

        assert_eq!(view1.show_help, view2.show_help);
        assert_eq!(view1.show_stats, view2.show_stats);
        assert_eq!(view1.block_style.fg, view2.block_style.fg);
        assert_eq!(view1.help_style.fg, view2.help_style.fg);
    }

    // View with all options disabled
    #[test]
    fn test_view_all_options_disabled() {
        let view = DescriptionEditorView::new()
            .show_help(false)
            .show_stats(false);

        assert!(!view.show_help);
        assert!(!view.show_stats);
    }

    // View with custom styles
    #[test]
    fn test_view_custom_styles() {
        let block_style = Style::default().fg(Color::Magenta);
        let help_style = Style::default().fg(Color::Cyan);

        let view = DescriptionEditorView::new()
            .block_style(block_style)
            .help_style(help_style);

        assert_eq!(view.block_style.fg, Some(Color::Magenta));
        assert_eq!(view.help_style.fg, Some(Color::Cyan));
    }

    // Multiple text operations
    #[test]
    fn test_multiple_text_operations() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());

        for c in "Hello".chars() {
            state.insert_char(c);
        }

        assert!(state.text().contains("Hello"));
        assert!(state.is_modified());
    }

    // Cursor movement on multiline text
    #[test]
    fn test_cursor_movement_multiline() {
        let mut state = DescriptionEditorState::new("Test".to_string(), "Line 1\nLine 2\nLine 3".to_string());

        state.move_cursor_down();
        state.move_cursor_down();
        state.move_to_line_end();
        state.move_to_line_start();

        // Cursor movement should not mark as modified
        assert!(!state.is_modified());
    }

    // Saved flag persistence
    #[test]
    fn test_saved_flag_persistence() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.save();
        assert!(state.is_saved());

        // Saved flag should persist even after other operations
        state.cancel();
        assert!(state.is_saved());
    }

    // Cancelled flag persistence
    #[test]
    fn test_cancelled_flag_persistence() {
        let mut state = DescriptionEditorState::new("Test".to_string(), String::new());
        state.cancel();
        assert!(state.is_cancelled());

        // Cancelled flag should persist
        state.save();
        assert!(state.is_cancelled());
    }

    // Long initial text
    #[test]
    fn test_long_initial_text() {
        let long_text = "Line\n".repeat(100);
        let state = DescriptionEditorState::new("Test".to_string(), long_text.clone());
        // Check that the text contains the expected content
        let result_text = state.text();
        assert!(result_text.contains("Line"));
        assert_eq!(result_text.matches("Line").count(), 100);
    }

    // View toggle both help and stats
    #[test]
    fn test_view_toggle_help_and_stats() {
        let view1 = DescriptionEditorView::new()
            .show_help(false)
            .show_stats(false);

        let view2 = DescriptionEditorView::new()
            .show_help(true)
            .show_stats(true);

        assert!(!view1.show_help && !view1.show_stats);
        assert!(view2.show_help && view2.show_stats);
    }

    // Multiple help style applications
    #[test]
    fn test_view_multiple_help_style_applications() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Green);

        let view = DescriptionEditorView::new()
            .help_style(style1)
            .help_style(style2);

        // Last style should win
        assert_eq!(view.help_style.fg, Some(Color::Green));
    }

    // State with all flags set
    #[test]
    fn test_state_with_all_flags_set() {
        let mut state = DescriptionEditorState::new("Test".to_string(), "content".to_string());
        state.set_modified(true);
        state.save();
        state.cancel();

        assert!(!state.is_modified()); // save() clears modified
        assert!(state.is_saved());
        assert!(state.is_cancelled());
    }

    // Editor mode debug trait
    #[test]
    fn test_editor_mode_debug_trait() {
        let mode = EditorMode::Insert;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Insert"));
    }

    // State debug trait
    #[test]
    fn test_state_debug_trait() {
        let state = DescriptionEditorState::new("Test".to_string(), String::new());
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("DescriptionEditorState"));
    }

    // Clone preserves all fields
    #[test]
    fn test_clone_preserves_all_fields() {
        let mut state = DescriptionEditorState::new("Test Title".to_string(), "content".to_string());
        state.set_mode(EditorMode::Command);
        state.toggle_help();
        state.set_modified(true);
        state.save();
        state.cancel();

        let cloned = state.clone();

        assert_eq!(cloned.title(), state.title());
        assert_eq!(cloned.mode(), state.mode());
        assert_eq!(cloned.is_help_visible(), state.is_help_visible());
        assert_eq!(cloned.is_modified(), state.is_modified());
        assert_eq!(cloned.is_saved(), state.is_saved());
        assert_eq!(cloned.is_cancelled(), state.is_cancelled());
    }

    // Whitespace in title
    #[test]
    fn test_title_with_leading_trailing_whitespace() {
        let title = "  Title with spaces  ";
        let state = DescriptionEditorState::new(title.to_string(), String::new());
        assert_eq!(state.title(), title);
    }

    // View builder all combinations
    #[test]
    fn test_view_builder_all_combinations() {
        let view = DescriptionEditorView::new()
            .show_help(true)
            .show_stats(false)
            .block_style(Style::default().fg(Color::Red))
            .help_style(Style::default().fg(Color::Blue));

        assert!(view.show_help);
        assert!(!view.show_stats);
        assert_eq!(view.block_style.fg, Some(Color::Red));
        assert_eq!(view.help_style.fg, Some(Color::Blue));
    }

    // Initial state defaults
    #[test]
    fn test_initial_state_defaults() {
        let state = DescriptionEditorState::new("Test".to_string(), String::new());

        assert_eq!(state.mode(), EditorMode::Insert);
        assert!(state.is_help_visible());
        assert!(!state.is_modified());
        assert!(!state.is_saved());
        assert!(!state.is_cancelled());
    }
}
