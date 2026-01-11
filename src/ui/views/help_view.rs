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
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::raw("         - Create new issue"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Delete selected issue"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Close selected issue"),
            ]),
            Line::from(vec![
                Span::styled("r", Style::default().fg(Color::Green)),
                Span::raw("         - Reopen closed issue"),
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
        assert_eq!(HelpSection::About.display_name(), "About");
    }

    #[test]
    fn test_help_section_all() {
        let sections = HelpSection::all();
        assert_eq!(sections.len(), 6);
        assert_eq!(sections[0], HelpSection::Global);
        assert_eq!(sections[5], HelpSection::About);
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
}
