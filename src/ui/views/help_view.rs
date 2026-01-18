//! Help view displaying keyboard shortcuts and documentation

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Help section category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpSection {
    /// Global keyboard shortcuts
    Global,
    /// UI Layout overview
    UILayout,
    /// Issues view shortcuts
    Issues,
    /// Dependencies view shortcuts
    Dependencies,
    /// Labels view shortcuts
    Labels,
    /// Database view shortcuts
    Database,
    /// Kanban view shortcuts
    Kanban,
    /// Gantt view shortcuts
    Gantt,
    /// PERT view shortcuts
    Pert,
    /// Search interface shortcuts
    Search,
    /// About information
    About,
}

impl HelpSection {
    /// Get all help sections
    pub fn all() -> Vec<HelpSection> {
        vec![
            Self::Global,
            Self::UILayout,
            Self::Issues,
            Self::Dependencies,
            Self::Labels,
            Self::Database,
            Self::Kanban,
            Self::Gantt,
            Self::Pert,
            Self::Search,
            Self::About,
        ]
    }

    /// Get display name for the section
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::UILayout => "UI Layout",
            Self::Issues => "Issues",
            Self::Dependencies => "Dependencies",
            Self::Labels => "Labels",
            Self::Database => "Database",
            Self::Kanban => "Kanban",
            Self::Gantt => "Gantt",
            Self::Pert => "PERT",
            Self::Search => "Search",
            Self::About => "About",
        }
    }
}

/// Help view state
#[derive(Debug, Clone)]
pub struct HelpViewState {
    /// Current scroll offset
    pub scroll_offset: u16,
}

impl HelpViewState {
    /// Create a new help view state
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: u16) {
        self.scroll_offset = offset;
    }
}

impl Default for HelpViewState {
    fn default() -> Self {
        Self::new()
    }
}

/// Help view widget
pub struct HelpView<'a> {
    selected_section: HelpSection,
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> HelpView<'a> {
    /// Create a new help view
    pub fn new() -> Self {
        Self {
            selected_section: HelpSection::Global,
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the selected section
    pub fn selected_section(mut self, section: HelpSection) -> Self {
        self.selected_section = section;
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Render a shortcut line with consistent alignment
    fn render_shortcut(&self, key: &str, desc: &str) -> Line<'static> {
        // Fixed width for key column
        const KEY_COL_WIDTH: usize = 35;
        
        let key_text = key.to_string();
        let padding = if key_text.len() < KEY_COL_WIDTH {
            " ".repeat(KEY_COL_WIDTH - key_text.len())
        } else {
            " ".to_string()
        };

        Line::from(vec![
            Span::styled(key_text, Style::default().fg(Color::Green)),
            Span::raw(padding),
            Span::raw("- "),
            Span::raw(desc.to_string()),
        ])
    }

    fn render_global_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Global & Navigation Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // Global Actions
        lines.push(Line::from(Span::styled("Global Actions", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("q, Ctrl+Q, Ctrl+C", "Exit the application"));
        lines.push(self.render_shortcut("?", "Toggle keyboard shortcuts overlay"));
        lines.push(self.render_shortcut("F1", "Toggle context-sensitive help"));
        lines.push(self.render_shortcut("Ctrl+P or F12", "Toggle performance statistics"));
        lines.push(self.render_shortcut("Ctrl+Z", "Undo last action"));
        lines.push(self.render_shortcut("Ctrl+Y", "Redo last undone action"));
        lines.push(self.render_shortcut("Esc", "Dismiss notification / close overlays"));
        lines.push(self.render_shortcut("Ctrl+H", "Show notification history")); // Removed N as per previous fix
        lines.push(Line::from(""));

        // Navigation
        lines.push(Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("Tab", "Switch to the next top-level tab"));
        lines.push(self.render_shortcut("Shift+Tab", "Switch to the previous tab"));
        lines.push(self.render_shortcut("1-9", "Jump to tab by number"));
        lines.push(self.render_shortcut("Up/Down or j/k", "Move selection up/down"));
        lines.push(self.render_shortcut("Left/Right or h/l", "Move selection left/right or collapse/expand"));
        lines.push(self.render_shortcut("PageUp/PageDown", "Page up/down in lists"));
        lines.push(self.render_shortcut("Ctrl+U/Ctrl+D", "Page up/down in lists"));
        lines.push(self.render_shortcut("Home/End or g/G", "Jump to top/bottom"));
        lines.push(Line::from(""));

        // General Operations
        lines.push(Line::from(Span::styled("General Operations", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("Enter", "Open details, confirm, or toggle expand"));
        lines.push(self.render_shortcut("Esc", "Close dialogs, clear search, go back"));
        lines.push(self.render_shortcut("r or F5", "Refresh data"));

        lines
    }

    fn render_ui_layout_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "UI Layout Overview",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(Line::from(Span::styled("Screen Layout", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(Line::from("The screen is organized into distinct sections:"));
        lines.push(Line::from(""));
        
        lines.push(Line::from(vec![
            Span::styled("1. TITLE Bar", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" (Top)"),
        ]));
        lines.push(Line::from("   - Application title and version"));
        lines.push(Line::from("   - Issue counts (Open, In Progress, Blocked, Closed)"));
        lines.push(Line::from("   - Global search bar"));
        lines.push(Line::from("   - Daemon status indicator"));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("2. VIEWS Container", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from("   - Tab selection: Issues | Split | Kanban | Dependencies | etc."));
        lines.push(Line::from("   - Use 1-9 or Tab/Shift+Tab to switch views"));
        lines.push(Line::from("   - Each tab shows count of items in that view"));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("3. FILTERS Container", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" (Issues View Only)"),
        ]));
        lines.push(Line::from("   - Quick filter dropdowns for Status, Type, Labels, Priority"));
        lines.push(Line::from("   - Created/Updated date filters"));
        lines.push(Line::from("   - Press hotkeys (S, T, L, I, C, U) to open dropdowns"));
        lines.push(Line::from("   - Shows filtered count vs total: (filtered/total)"));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("4. Content Area", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from("   - Main workspace displaying current view"));
        lines.push(Line::from("   - Issues list, Kanban board, Gantt chart, etc."));
        lines.push(Line::from("   - Scrollable with Up/Down or PgUp/PgDn"));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("5. ACTIONS Bar", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" (Bottom)"),
        ]));
        lines.push(Line::from("   - Context-sensitive action hints"));
        lines.push(Line::from("   - Navigation: ↓:Up ↑:Down →:Scroll-Right ←:Scroll-Left"));
        lines.push(Line::from("   - Quick actions: R:Read | N:New | E:Edit | D:Delete | F:Find | O:Open | X:Close"));
        lines.push(Line::from("   - Actions change based on current view/mode"));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("Navigation Tips", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(Line::from("• Use arrow keys or hjkl (Vim-style) for navigation"));
        lines.push(Line::from("• PgUp/PgDn or Ctrl+U/Ctrl+D for page scrolling"));
        lines.push(Line::from("• Home/End or g/G to jump to top/bottom"));
        lines.push(Line::from("• Tab to cycle through UI elements"));
        lines.push(Line::from("• ? to open this help at any time"));

        lines
    }

    fn render_issues_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Issues View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // List View
        lines.push(Line::from(Span::styled("Issues List", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("n", "Create a new issue"));
        lines.push(self.render_shortcut("e", "Edit selected issue"));
        lines.push(self.render_shortcut("d", "Delete selected issue (with confirmation)"));
        lines.push(self.render_shortcut("x", "Close selected issue"));
        lines.push(self.render_shortcut("o", "Reopen selected issue"));
        lines.push(self.render_shortcut("F2", "Quick edit issue title"));
        lines.push(self.render_shortcut("p", "Change priority of selected issue"));
        lines.push(self.render_shortcut("s", "Change status of selected issue"));
        lines.push(self.render_shortcut("l", "Edit labels for selected issue"));
        lines.push(self.render_shortcut("a", "Edit assignee for selected issue"));
        lines.push(self.render_shortcut("+", "Add dependency to selected issue"));
        lines.push(self.render_shortcut("-", "Remove dependency from selected issue"));
        lines.push(self.render_shortcut(">", "Indent issue (make child of previous)"));
        lines.push(self.render_shortcut("<", "Outdent issue (promote to parent level)"));
        lines.push(self.render_shortcut("Space", "Toggle Select"));
        lines.push(self.render_shortcut("Ctrl+A", "Select All"));
        lines.push(self.render_shortcut("Ctrl+N", "Clear selection"));
        lines.push(self.render_shortcut("c", "Open column manager"));
        lines.push(self.render_shortcut("v", "Cycle issue scope"));
        lines.push(Line::from(""));

        // Detail / Split
        lines.push(Line::from(Span::styled("Detail / Split View", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("Enter", "Open full detail view"));
        lines.push(self.render_shortcut("e", "Edit selected issue"));
        lines.push(self.render_shortcut("d", "Delete selected issue"));
        lines.push(self.render_shortcut("Esc or q", "Return to list view"));
        lines.push(self.render_shortcut("Alt+H", "Toggle issue history panel"));
        lines.push(Line::from(""));

        // Forms
        lines.push(Line::from(Span::styled("Forms (Create/Edit)", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("Tab", "Move focus to next form field"));
        lines.push(self.render_shortcut("Shift+Tab", "Move focus to previous form field"));
        lines.push(self.render_shortcut("Enter", "Save and close the form"));
        lines.push(self.render_shortcut("Esc", "Close form without saving"));
        lines.push(self.render_shortcut("Ctrl+L", "Load description content from file path"));
        lines.push(Line::from(""));

        // Record Detail Form Actions
        lines.push(Line::from(Span::styled("Record Detail Form", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("r/R", "Open selected issue in Read Mode"));
        lines.push(self.render_shortcut("e/E", "Open selected issue in Edit Mode"));
        lines.push(self.render_shortcut("Tab", "Switch focus (list/detail in split view)"));
        lines.push(self.render_shortcut("Ctrl+S", "Save changes"));
        lines.push(self.render_shortcut("Ctrl+X", "Cancel editing and revert changes"));
        lines.push(self.render_shortcut("Ctrl+Del", "Soft delete issue"));
        lines.push(self.render_shortcut("Ctrl+J", "Copy issue as JSON to clipboard"));
        lines.push(self.render_shortcut("Ctrl+P", "Export issue to Markdown file"));
        lines.push(self.render_shortcut("Up/Down", "Scroll detail view"));
        lines.push(self.render_shortcut("PgUp/PgDn", "Page up/down in detail view"));

        lines
    }

    fn render_dependencies_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Dependencies View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("Up/Down or j/k", "Move selection"));
        lines.push(self.render_shortcut("Tab", "Switch focus between Dependencies and Blocks"));
        lines.push(self.render_shortcut("a", "Add dependency (opens dialog)"));
        lines.push(self.render_shortcut("d", "Remove dependency (with confirmation)"));
        lines.push(self.render_shortcut("g", "Show dependency graph (stub)"));
        lines.push(self.render_shortcut("c", "Check circular dependencies (stub)"));
        lines.push(self.render_shortcut("Enter", "View selected issue details"));
        lines.push(self.render_shortcut("Esc", "Return to Issues view"));

        lines
    }

    fn render_labels_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Labels View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("Up/Down or j/k", "Move selection"));
        lines.push(self.render_shortcut("/", "Search labels"));
        lines.push(self.render_shortcut("a", "Add label"));
        lines.push(self.render_shortcut("e", "Edit label"));
        lines.push(self.render_shortcut("d", "Delete label"));
        lines.push(self.render_shortcut("s", "Show label stats info"));
        lines.push(self.render_shortcut("Esc", "Return to Issues view"));

        lines
    }

    fn render_database_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Database View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("r", "Refresh database status"));
        lines.push(self.render_shortcut("s", "Sync database with remote"));
        lines.push(self.render_shortcut("x", "Export issues to JSONL"));
        lines.push(self.render_shortcut("i", "Import issues from JSONL"));
        lines.push(self.render_shortcut("v", "Verify database integrity"));
        lines.push(self.render_shortcut("c", "Compact database"));
        lines.push(self.render_shortcut("t", "Start/stop daemon"));
        lines.push(self.render_shortcut("Esc", "Return to Issues view"));

        lines
    }

    fn render_kanban_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Kanban Board View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("Up/Down/Left/Right or h/j/k/l", "Move between cards/columns"));
        lines.push(self.render_shortcut("Space", "Move card to next column"));
        lines.push(self.render_shortcut("c", "Configure board columns"));
        lines.push(self.render_shortcut("Ctrl+F1", "Toggle Open column collapse"));
        lines.push(self.render_shortcut("Ctrl+F2", "Toggle In Progress column collapse"));
        lines.push(self.render_shortcut("Ctrl+F3", "Toggle Blocked column collapse"));
        lines.push(self.render_shortcut("Ctrl+F4", "Toggle Closed column collapse"));

        lines
    }

    fn render_gantt_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Gantt Chart View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("Up/Down", "Move selection"));
        lines.push(self.render_shortcut("+ / -", "Zoom in/out"));
        lines.push(self.render_shortcut("g", "Change grouping mode"));
        lines.push(self.render_shortcut("c", "Configure chart settings"));

        lines
    }

    fn render_pert_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "PERT Chart View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Project Evaluation and Review Technique"),
            Line::from(""),
        ];

        lines.push(self.render_shortcut("Up/Down", "Move selection"));
        lines.push(self.render_shortcut("+ / -", "Zoom in/out"));
        lines.push(self.render_shortcut("c", "Configure chart settings"));

        lines
    }

    fn render_search_help(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Search & Filter Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.push(Line::from(Span::styled("Search Bar", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("/", "Focus search bar"));
        lines.push(self.render_shortcut("Esc", "Clear search input and return focus"));
        lines.push(self.render_shortcut("Shift+N", "Jump to next search result"));
        lines.push(self.render_shortcut("Alt+N", "Jump to previous search result"));
        lines.push(self.render_shortcut("Alt+Z", "Toggle fuzzy search"));
        lines.push(self.render_shortcut("Alt+R", "Toggle regex search"));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("Filter Bar", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(Line::from("The FILTERS container appears below the VIEWS tabs in Issues view."));
        lines.push(Line::from("Use hotkeys to quickly filter issues:"));
        lines.push(Line::from(""));
        lines.push(self.render_shortcut("S", "Open Status filter dropdown (All, Open, In Progress, Blocked, Closed)"));
        lines.push(self.render_shortcut("T", "Open Type filter dropdown (All, Bug, Feature, Task, Epic, Chore)"));
        lines.push(self.render_shortcut("L", "Open Labels filter dropdown"));
        lines.push(self.render_shortcut("I", "Open Priority filter dropdown (P0-P4)"));
        lines.push(self.render_shortcut("C", "Open Created date filter dropdown"));
        lines.push(self.render_shortcut("U", "Open Updated date filter dropdown"));
        lines.push(Line::from(""));
        lines.push(self.render_shortcut("f", "Toggle quick filters on/off"));
        lines.push(self.render_shortcut("Shift+F", "Clear current filters"));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("Saved Filters", Style::default().add_modifier(Modifier::UNDERLINED))));
        lines.push(self.render_shortcut("Alt+S", "Save current filter configuration"));
        lines.push(self.render_shortcut("Alt+F", "Open saved filters menu"));
        lines.push(self.render_shortcut("F3-F11", "Apply saved filter hotkeys"));

        lines
    }

    fn render_about(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "About Beads-TUI",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::Cyan)),
                Span::raw("0.1.0"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description:", Style::default().fg(Color::Cyan)),
                Span::raw(""),
            ]),
            Line::from("Interactive terminal UI for the Beads issue tracking system."),
            Line::from("Built with Rust and Ratatui."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Repository:", Style::default().fg(Color::Cyan)),
                Span::raw(" https://github.com/davidcforbes/beads-tui"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("License:", Style::default().fg(Color::Cyan)),
                Span::raw(" MIT"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Author:", Style::default().fg(Color::Cyan)),
                Span::raw(" David Forbes"),
            ]),
        ]
    }

    fn get_section_content(&self, section: HelpSection) -> Vec<Line<'static>> {
        match section {
            HelpSection::Global => self.render_global_help(),
            HelpSection::UILayout => self.render_ui_layout_help(),
            HelpSection::Issues => self.render_issues_help(),
            HelpSection::Dependencies => self.render_dependencies_help(),
            HelpSection::Labels => self.render_labels_help(),
            HelpSection::Database => self.render_database_help(),
            HelpSection::Kanban => self.render_kanban_help(),
            HelpSection::Gantt => self.render_gantt_help(),
            HelpSection::Pert => self.render_pert_help(),
            HelpSection::Search => self.render_search_help(),
            HelpSection::About => self.render_about(),
        }
    }
}

impl<'a> Default for HelpView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for HelpView<'a> {
    type State = HelpViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Update state scroll offset from app state (will be passed in)
        // Note: The actual scroll offset comes from app.help_scroll_offset

        // Create layout: title (1) + content (fill)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render title
        let title = Paragraph::new("Help & Keyboard Shortcuts")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        title.render(chunks[0], buf);

        // Render content with scroll
        let content_lines = self.get_section_content(self.selected_section);

        // Calculate visible area height (subtract 2 for borders)
        let visible_height = chunks[1].height.saturating_sub(2) as usize;

        // Apply scroll offset
        let start_line = state.scroll_offset as usize;
        let visible_lines: Vec<Line> = content_lines
            .into_iter()
            .skip(start_line)
            .take(visible_height)
            .collect();

        let content = Paragraph::new(visible_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} (Use ↑/↓ to scroll)", self.selected_section.display_name())),
            )
            .wrap(Wrap { trim: true });
        content.render(chunks[1], buf);
    }
}

// Keep the old Widget implementation for backward compatibility in tests
impl<'a> Widget for HelpView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = HelpViewState::new();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// Event handling implementation
use super::ViewEventHandler;
use crate::models::AppState;
use crate::config::Action;
use crossterm::event::{KeyEvent, MouseEvent};

impl ViewEventHandler for HelpViewState {
    fn handle_key_event(app: &mut AppState, key: KeyEvent) -> bool {
        let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

        // Handle notification dismissal with Esc
        if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
            app.clear_notification();
            return true;
        }

        match action {
            Some(Action::MoveUp) => {
                app.scroll_help_up();
                true
            }
            Some(Action::MoveDown) => {
                app.scroll_help_down();
                true
            }
            Some(Action::MoveRight) | Some(Action::NextTab) => {
                app.next_help_section();
                true
            }
            Some(Action::MoveLeft) | Some(Action::PrevTab) => {
                app.previous_help_section();
                true
            }
            Some(Action::CancelDialog) => {
                app.selected_tab = 0; // Go back to issues
                true
            }
            _ => false,
        }
    }

    fn view_name() -> &'static str {
        "HelpView"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_section_display_name() {
        assert_eq!(HelpSection::Global.display_name(), "Global");
        assert_eq!(HelpSection::Issues.display_name(), "Issues");
        assert_eq!(HelpSection::Dependencies.display_name(), "Dependencies");
        assert_eq!(HelpSection::Labels.display_name(), "Labels");
        assert_eq!(HelpSection::Database.display_name(), "Database");
        assert_eq!(HelpSection::Kanban.display_name(), "Kanban");
        assert_eq!(HelpSection::Gantt.display_name(), "Gantt");
        assert_eq!(HelpSection::Pert.display_name(), "PERT");
        assert_eq!(HelpSection::Search.display_name(), "Search");
        assert_eq!(HelpSection::About.display_name(), "About");
    }

    #[test]
    fn test_help_section_all() {
        let sections = HelpSection::all();
        assert_eq!(sections.len(), 11);
        assert_eq!(sections[0], HelpSection::Global);
        assert_eq!(sections[10], HelpSection::About);
    }

    #[test]
    fn test_help_view_creation() {
        let view = HelpView::new();
        assert_eq!(view.selected_section, HelpSection::Global);
    }

    #[test]
    fn test_help_view_selected_section() {
        let view = HelpView::new().selected_section(HelpSection::About);
        assert_eq!(view.selected_section, HelpSection::About);
    }

    #[test]
    fn test_help_section_equality() {
        assert_eq!(HelpSection::Global, HelpSection::Global);
        assert_eq!(HelpSection::Issues, HelpSection::Issues);
        assert_eq!(HelpSection::Dependencies, HelpSection::Dependencies);
        assert_eq!(HelpSection::Labels, HelpSection::Labels);
        assert_eq!(HelpSection::Database, HelpSection::Database);
        assert_eq!(HelpSection::About, HelpSection::About);

        assert_ne!(HelpSection::Global, HelpSection::Issues);
        assert_ne!(HelpSection::Dependencies, HelpSection::About);
    }

    #[test]
    fn test_help_section_all_order() {
        let sections = HelpSection::all();
        assert_eq!(sections[0], HelpSection::Global);
        assert_eq!(sections[1], HelpSection::UILayout);
        assert_eq!(sections[2], HelpSection::Issues);
        assert_eq!(sections[3], HelpSection::Dependencies);
        assert_eq!(sections[4], HelpSection::Labels);
        assert_eq!(sections[5], HelpSection::Database);
        assert_eq!(sections[6], HelpSection::Kanban);
        assert_eq!(sections[7], HelpSection::Gantt);
        assert_eq!(sections[8], HelpSection::Pert);
        assert_eq!(sections[9], HelpSection::Search);
        assert_eq!(sections[10], HelpSection::About);
    }

    #[test]
    fn test_help_section_all_contains_all() {
        let sections = HelpSection::all();
        assert!(sections.contains(&HelpSection::Global));
        assert!(sections.contains(&HelpSection::Issues));
        assert!(sections.contains(&HelpSection::Dependencies));
        assert!(sections.contains(&HelpSection::Labels));
        assert!(sections.contains(&HelpSection::Database));
        assert!(sections.contains(&HelpSection::Kanban));
        assert!(sections.contains(&HelpSection::Gantt));
        assert!(sections.contains(&HelpSection::Pert));
        assert!(sections.contains(&HelpSection::Search));
        assert!(sections.contains(&HelpSection::About));
    }

    #[test]
    fn test_help_view_default() {
        let view = HelpView::default();
        assert_eq!(view.selected_section, HelpSection::Global);
    }

    #[test]
    fn test_help_view_default_same_as_new() {
        let default_view = HelpView::default();
        let new_view = HelpView::new();
        assert_eq!(default_view.selected_section, new_view.selected_section);
    }

    #[test]
    fn test_help_view_block_style() {
        let style = Style::default().fg(Color::Red);
        let view = HelpView::new().block_style(style);
        assert_eq!(view.block_style, style);
    }

    #[test]
    fn test_help_view_builder_chain() {
        let style = Style::default().fg(Color::Yellow);
        let view = HelpView::new()
            .selected_section(HelpSection::Labels)
            .block_style(style);

        assert_eq!(view.selected_section, HelpSection::Labels);
        assert_eq!(view.block_style, style);
    }

    #[test]
    fn test_help_view_all_sections() {
        for section in HelpSection::all() {
            let view = HelpView::new().selected_section(section);
            assert_eq!(view.selected_section, section);
        }
    }

    #[test]
    fn test_render_global_help_non_empty() {
        let view = HelpView::new();
        let lines = view.render_global_help();
        assert!(!lines.is_empty());
        assert!(lines.len() > 5); // Should have title + shortcuts
    }

    #[test]
    fn test_render_issues_help_non_empty() {
        let view = HelpView::new();
        let lines = view.render_issues_help();
        assert!(!lines.is_empty());
        assert!(lines.len() > 5);
    }

    #[test]
    fn test_render_dependencies_help_non_empty() {
        let view = HelpView::new();
        let lines = view.render_dependencies_help();
        assert!(!lines.is_empty());
        assert!(lines.len() > 5);
    }

    #[test]
    fn test_render_labels_help_non_empty() {
        let view = HelpView::new();
        let lines = view.render_labels_help();
        assert!(!lines.is_empty());
        assert!(lines.len() > 5);
    }

    #[test]
    fn test_render_database_help_non_empty() {
        let view = HelpView::new();
        let lines = view.render_database_help();
        assert!(!lines.is_empty());
        assert!(lines.len() > 3);
    }

    #[test]
    fn test_render_about_non_empty() {
        let view = HelpView::new();
        let lines = view.render_about();
        assert!(!lines.is_empty());
        assert!(lines.len() > 5);
    }

    #[test]
    fn test_get_section_content_global() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::Global);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_issues() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::Issues);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_dependencies() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::Dependencies);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_labels() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::Labels);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_database() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::Database);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_about() {
        let view = HelpView::new();
        let content = view.get_section_content(HelpSection::About);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_get_section_content_matches_render_methods() {
        let view = HelpView::new();

        assert_eq!(
            view.get_section_content(HelpSection::Global).len(),
            view.render_global_help().len()
        );
        assert_eq!(
            view.get_section_content(HelpSection::Issues).len(),
            view.render_issues_help().len()
        );
        assert_eq!(
            view.get_section_content(HelpSection::Dependencies).len(),
            view.render_dependencies_help().len()
        );
        assert_eq!(
            view.get_section_content(HelpSection::Labels).len(),
            view.render_labels_help().len()
        );
        assert_eq!(
            view.get_section_content(HelpSection::Database).len(),
            view.render_database_help().len()
        );
        assert_eq!(
            view.get_section_content(HelpSection::About).len(),
            view.render_about().len()
        );
    }

    #[test]
    fn test_help_section_clone() {
        let section = HelpSection::Issues;
        let cloned = section;
        assert_eq!(section, cloned);
    }

    #[test]
    fn test_help_section_copy() {
        let section = HelpSection::Labels;
        let copied = section;
        assert_eq!(section, copied);
    }

    #[test]
    fn test_help_section_all_clone() {
        for section in HelpSection::all() {
            let cloned = section;
            assert_eq!(section, cloned);
        }
    }

    #[test]
    fn test_help_section_display_name_all_sections() {
        for section in HelpSection::all() {
            let name = section.display_name();
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_render_global_help_has_title() {
        let view = HelpView::new();
        let lines = view.render_global_help();
        assert!(!lines.is_empty());
        // First line should be the title
        let title_line = &lines[0];
        assert!(title_line
            .spans
            .iter()
            .any(|s| s.content.contains("Global")));
    }

    #[test]
    fn test_render_issues_help_has_title() {
        let view = HelpView::new();
        let lines = view.render_issues_help();
        assert!(!lines.is_empty());
        let title_line = &lines[0];
        assert!(title_line
            .spans
            .iter()
            .any(|s| s.content.contains("Issues")));
    }

    #[test]
    fn test_render_dependencies_help_has_title() {
        let view = HelpView::new();
        let lines = view.render_dependencies_help();
        assert!(!lines.is_empty());
        let title_line = &lines[0];
        assert!(title_line
            .spans
            .iter()
            .any(|s| s.content.contains("Dependencies")));
    }

    #[test]
    fn test_render_labels_help_has_title() {
        let view = HelpView::new();
        let lines = view.render_labels_help();
        assert!(!lines.is_empty());
        let title_line = &lines[0];
        assert!(title_line
            .spans
            .iter()
            .any(|s| s.content.contains("Labels")));
    }

    #[test]
    fn test_render_database_help_has_title() {
        let view = HelpView::new();
        let lines = view.render_database_help();
        assert!(!lines.is_empty());
        let title_line = &lines[0];
        assert!(title_line
            .spans
            .iter()
            .any(|s| s.content.contains("Database")));
    }

    #[test]
    fn test_render_about_has_version() {
        let view = HelpView::new();
        let lines = view.render_about();
        let content_str = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert!(content_str.contains("Version"));
    }

    #[test]
    fn test_render_about_has_description() {
        let view = HelpView::new();
        let lines = view.render_about();
        let content_str = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert!(content_str.contains("Description"));
    }

    #[test]
    fn test_render_about_has_repository() {
        let view = HelpView::new();
        let lines = view.render_about();
        let content_str = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert!(content_str.contains("Repository"));
    }

    #[test]
    fn test_render_about_has_license() {
        let view = HelpView::new();
        let lines = view.render_about();
        let content_str = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert!(content_str.contains("License"));
    }

    #[test]
    fn test_render_about_has_author() {
        let view = HelpView::new();
        let lines = view.render_about();
        let content_str = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert!(content_str.contains("Author"));
    }

    #[test]
    fn test_help_view_builder_multiple_chains() {
        let style1 = Style::default().fg(Color::Red);
        let style2 = Style::default().fg(Color::Blue);

        let view1 = HelpView::new()
            .selected_section(HelpSection::Issues)
            .block_style(style1);

        let view2 = HelpView::new()
            .block_style(style2)
            .selected_section(HelpSection::About);

        assert_eq!(view1.selected_section, HelpSection::Issues);
        assert_eq!(view1.block_style, style1);

        assert_eq!(view2.selected_section, HelpSection::About);
        assert_eq!(view2.block_style, style2);
    }

    #[test]
    fn test_help_view_default_block_style() {
        let view = HelpView::new();
        assert_eq!(view.block_style.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_help_section_all_no_duplicates() {
        let sections = HelpSection::all();
        for (i, section1) in sections.iter().enumerate() {
            for (j, section2) in sections.iter().enumerate() {
                if i != j {
                    assert_ne!(section1, section2);
                }
            }
        }
    }

    #[test]
    fn test_get_section_content_all_sections_non_empty() {
        let view = HelpView::new();
        for section in HelpSection::all() {
            let content = view.get_section_content(section);
            assert!(
                !content.is_empty(),
                "Section {:?} should have content",
                section
            );
        }
    }

    #[test]
    fn test_help_section_copy_trait() {
        fn takes_copy<T: Copy>(_: T) {}
        takes_copy(HelpSection::Global);
    }

    #[test]
    fn test_help_view_selected_section_changes() {
        let view1 = HelpView::new();
        assert_eq!(view1.selected_section, HelpSection::Global);

        let view2 = view1.selected_section(HelpSection::Dependencies);
        assert_eq!(view2.selected_section, HelpSection::Dependencies);
    }

    #[test]
    fn test_help_section_debug_formatting() {
        let section = HelpSection::Global;
        let debug_str = format!("{:?}", section);
        assert_eq!(debug_str, "Global");

        let section = HelpSection::Issues;
        let debug_str = format!("{:?}", section);
        assert_eq!(debug_str, "Issues");

        let section = HelpSection::Dependencies;
        let debug_str = format!("{:?}", section);
        assert_eq!(debug_str, "Dependencies");
    }

    #[test]
    fn test_render_global_help_contains_quit_shortcut() {
        let view = HelpView::new();
        let lines = view.render_global_help();
        let text: String = lines.iter().map(|line| line.to_string()).collect();
        assert!(text.contains("q") || text.contains("Quit") || text.contains("Exit"));
    }

    #[test]
    fn test_render_issues_help_contains_create_shortcut() {
        let view = HelpView::new();
        let lines = view.render_issues_help();
        let text: String = lines.iter().map(|line| line.to_string()).collect();
        assert!(text.contains("c") || text.contains("Create") || text.contains("New"));
    }

    #[test]
    fn test_render_dependencies_help_contains_add_shortcut() {
        let view = HelpView::new();
        let lines = view.render_dependencies_help();
        let text: String = lines.iter().map(|line| line.to_string()).collect();
        assert!(text.contains("a") || text.contains("Add") || text.contains("dependency"));
    }

    #[test]
    fn test_render_labels_help_contains_add_shortcut() {
        let view = HelpView::new();
        let lines = view.render_labels_help();
        let text: String = lines.iter().map(|line| line.to_string()).collect();
        assert!(text.contains("a") || text.contains("Add") || text.contains("label"));
    }

    #[test]
    fn test_render_database_help_contains_export_shortcut() {
        let view = HelpView::new();
        let lines = view.render_database_help();
        let text: String = lines.iter().map(|line| line.to_string()).collect();
        assert!(text.contains("e") || text.contains("Export") || text.contains("export"));
    }

    #[test]
    fn test_widget_trait_rendering() {
        let view = HelpView::new().selected_section(HelpSection::Global);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        Widget::render(view, area, &mut buffer);

        // Buffer should be modified
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Widget should render content to buffer");
    }

    #[test]
    fn test_widget_rendering_different_sections() {
        let sections = HelpSection::all();
        let area = Rect::new(0, 0, 100, 30);

        for section in sections {
            let view = HelpView::new().selected_section(section);
            let mut buffer = Buffer::empty(area);

            Widget::render(view, area, &mut buffer);

            let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
            assert!(has_content, "Section {:?} should render content", section);
        }
    }

    #[test]
    fn test_widget_rendering_with_custom_style() {
        let view = HelpView::new().block_style(Style::default().fg(Color::Cyan));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        Widget::render(view, area, &mut buffer);

        // Should render without panic
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content);
    }

    #[test]
    fn test_widget_rendering_small_area() {
        let view = HelpView::new();
        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::empty(area);

        // Should handle small areas gracefully
        Widget::render(view, area, &mut buffer);

        // Should not panic
    }

    #[test]
    fn test_help_section_all_matches_variant_count() {
        let all_sections = HelpSection::all();
        // Should have exactly 11 variants
        assert_eq!(all_sections.len(), 11);
    }

    #[test]
    fn test_widget_rendering_zero_area() {
        let view = HelpView::new();
        let area = Rect::new(0, 0, 0, 0);
        let mut buffer = Buffer::empty(area);

        // Should handle zero-sized areas gracefully
        Widget::render(view, area, &mut buffer);

        // Should not panic
    }

    #[test]
    fn test_render_about_has_multiple_lines() {
        let view = HelpView::new();
        let lines = view.render_about();

        // About section should have substantial content
        assert!(
            lines.len() > 5,
            "About section should have multiple information lines"
        );
    }

    #[test]
    fn test_all_render_methods_produce_distinct_content() {
        let view = HelpView::new();

        let global = view.render_global_help();
        let issues = view.render_issues_help();
        let deps = view.render_dependencies_help();

        let global_text: String = global.iter().map(|l| l.to_string()).collect();
        let issues_text: String = issues.iter().map(|l| l.to_string()).collect();
        let deps_text: String = deps.iter().map(|l| l.to_string()).collect();

        // Each section should have unique content
        assert_ne!(global_text, issues_text);
        assert_ne!(global_text, deps_text);
        assert_ne!(issues_text, deps_text);
    }
}
