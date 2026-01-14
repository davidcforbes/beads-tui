//! Help view displaying keyboard shortcuts and documentation

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Help section category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpSection {
    /// Global keyboard shortcuts
    Global,
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

    fn render_global_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Global Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Green)),
                Span::raw("         - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Green)),
                Span::raw("       - Next tab"),
            ]),
            Line::from(vec![
                Span::styled("Shift+Tab", Style::default().fg(Color::Green)),
                Span::raw(" - Previous tab"),
            ]),
            Line::from(vec![
                Span::styled("1-5", Style::default().fg(Color::Green)),
                Span::raw("       - Jump to tab directly"),
            ]),
            Line::from(vec![
                Span::styled("?", Style::default().fg(Color::Green)),
                Span::raw("         - Toggle help"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+C", Style::default().fg(Color::Green)),
                Span::raw("    - Force quit"),
            ]),
        ]
    }

    fn render_issues_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Issues View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Green)),
                Span::raw("         - Focus search input"),
            ]),
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Green)),
                Span::raw("       - Cycle search scope"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Clear search"),
            ]),
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate list (or ↓/↑)"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - View issue details"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::raw("         - Edit selected issue"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Create new issue"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Delete selected issue"),
            ]),
            Line::from(vec![
                Span::styled("x", Style::default().fg(Color::Green)),
                Span::raw("         - Close selected issue"),
            ]),
            Line::from(vec![
                Span::styled("o", Style::default().fg(Color::Green)),
                Span::raw("         - Reopen closed issue"),
            ]),
            Line::from(vec![
                Span::styled("r", Style::default().fg(Color::Green)),
                Span::raw("         - Rename issue title (in-place edit)"),
            ]),
            Line::from(vec![
                Span::styled(">", Style::default().fg(Color::Green)),
                Span::raw("         - Indent issue (make child of previous)"),
            ]),
            Line::from(vec![
                Span::styled("<", Style::default().fg(Color::Green)),
                Span::raw("         - Outdent issue (promote to parent level)"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+Up/Down", Style::default().fg(Color::Green)),
                Span::raw(" - Reorder child within parent"),
            ]),
        ]
    }

    fn render_dependencies_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Dependencies View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate tree (or ↓/↑)"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - Expand/collapse node"),
            ]),
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Green)),
                Span::raw("         - Add dependency"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Remove dependency"),
            ]),
            Line::from(vec![
                Span::styled("g", Style::default().fg(Color::Green)),
                Span::raw("         - Show dependency graph"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Check for cycles"),
            ]),
        ]
    }

    fn render_labels_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Labels View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate labels (or ↓/↑)"),
            ]),
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Green)),
                Span::raw("         - Add new label"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Delete label"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::raw("         - Edit label"),
            ]),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::raw("         - Show label statistics"),
            ]),
            Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Green)),
                Span::raw("         - Search labels"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - Apply search filter"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Clear search filter"),
            ]),
        ]
    }

    fn render_database_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Database View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::raw("         - Sync database"),
            ]),
            Line::from(vec![
                Span::styled("i", Style::default().fg(Color::Green)),
                Span::raw("         - Import data"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::raw("         - Export data"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Start/stop daemon"),
            ]),
            Line::from(vec![
                Span::styled("r", Style::default().fg(Color::Green)),
                Span::raw("         - Refresh status"),
            ]),
        ]
    }

    fn render_kanban_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Kanban Board View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("↑/↓/←/→", Style::default().fg(Color::Green)),
                Span::raw("   - Navigate between cards"),
            ]),
            Line::from(vec![
                Span::styled("j/k/h/l", Style::default().fg(Color::Green)),
                Span::raw("   - Navigate (vim-style)"),
            ]),
            Line::from(vec![
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw("     - Move card to different column"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Configure board settings"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - View/edit card details"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Go back"),
            ]),
        ]
    }

    fn render_gantt_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Gantt Chart View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate through tasks"),
            ]),
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate (vim-style)"),
            ]),
            Line::from(vec![
                Span::styled("+/-", Style::default().fg(Color::Green)),
                Span::raw("       - Zoom timeline in/out"),
            ]),
            Line::from(vec![
                Span::styled("g", Style::default().fg(Color::Green)),
                Span::raw("         - Change grouping mode"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Configure chart settings"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - View task details"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Go back"),
            ]),
        ]
    }

    fn render_pert_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "PERT Chart View Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Project Evaluation and Review Technique"),
            Line::from(""),
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate through nodes"),
            ]),
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Green)),
                Span::raw("       - Navigate (vim-style)"),
            ]),
            Line::from(vec![
                Span::styled("+/-", Style::default().fg(Color::Green)),
                Span::raw("       - Zoom in/out"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Configure chart settings"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("     - View node details"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Go back"),
            ]),
        ]
    }

    fn render_search_help(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Search Interface Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Green)),
                Span::raw("         - Focus search input"),
            ]),
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Green)),
                Span::raw("       - Cycle search scope (All/Title/Description/etc)"),
            ]),
            Line::from(vec![
                Span::styled("f", Style::default().fg(Color::Green)),
                Span::raw("         - Open quick filter menu"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
                Span::raw("    - Save current filter"),
            ]),
            Line::from(vec![
                Span::styled("F1-F11", Style::default().fg(Color::Green)),
                Span::raw("    - Apply saved filter (1-11)"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+R", Style::default().fg(Color::Green)),
                Span::raw("    - Toggle regex mode"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+F", Style::default().fg(Color::Green)),
                Span::raw("    - Toggle fuzzy search"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw("       - Clear search or go back"),
            ]),
        ]
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

impl<'a> Widget for HelpView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        // Render content
        let content_lines = self.get_section_content(self.selected_section);
        let content = Paragraph::new(content_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.selected_section.display_name()),
            )
            .wrap(Wrap { trim: true });
        content.render(chunks[1], buf);
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
        assert_eq!(sections.len(), 10);
        assert_eq!(sections[0], HelpSection::Global);
        assert_eq!(sections[9], HelpSection::About);
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
        assert_eq!(sections[1], HelpSection::Issues);
        assert_eq!(sections[2], HelpSection::Dependencies);
        assert_eq!(sections[3], HelpSection::Labels);
        assert_eq!(sections[4], HelpSection::Database);
        assert_eq!(sections[5], HelpSection::Kanban);
        assert_eq!(sections[6], HelpSection::Gantt);
        assert_eq!(sections[7], HelpSection::Pert);
        assert_eq!(sections[8], HelpSection::Search);
        assert_eq!(sections[9], HelpSection::About);
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

        view.render(area, &mut buffer);

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

            view.render(area, &mut buffer);

            let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
            assert!(has_content, "Section {:?} should render content", section);
        }
    }

    #[test]
    fn test_widget_rendering_with_custom_style() {
        let view = HelpView::new().block_style(Style::default().fg(Color::Cyan));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);

        view.render(area, &mut buffer);

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
        view.render(area, &mut buffer);

        // Should not panic
    }

    #[test]
    fn test_help_section_all_matches_variant_count() {
        let all_sections = HelpSection::all();
        // Should have exactly 10 variants
        assert_eq!(all_sections.len(), 10);
    }

    #[test]
    fn test_widget_rendering_zero_area() {
        let view = HelpView::new();
        let area = Rect::new(0, 0, 0, 0);
        let mut buffer = Buffer::empty(area);

        // Should handle zero-sized areas gracefully
        view.render(area, &mut buffer);

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
