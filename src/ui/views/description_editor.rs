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
        let line_len = self.editor_state.lines()
            [self.editor_state.cursor_position().0]
            .len();
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
                Style::default()
                    .fg(mode_color)
                    .add_modifier(Modifier::BOLD),
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

        StatefulWidget::render(
            editor,
            chunks[chunk_idx],
            buf,
            &mut state.editor_state,
        );
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
        let state =
            DescriptionEditorState::new("Test Issue".to_string(), initial_text.to_string());
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
}
