//! Database view for status, sync, and management operations

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Database operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseStatus {
    /// Database is ready
    Ready,
    /// Database is syncing
    Syncing,
    /// Database sync failed
    Error,
    /// Database is offline
    Offline,
}

impl DatabaseStatus {
    /// Get display name for the status
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Syncing => "Syncing",
            Self::Error => "Error",
            Self::Offline => "Offline",
        }
    }

    /// Get color for the status
    pub fn color(&self) -> Color {
        match self {
            Self::Ready => Color::Green,
            Self::Syncing => Color::Yellow,
            Self::Error => Color::Red,
            Self::Offline => Color::Gray,
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of issues
    pub total_issues: usize,
    /// Number of open issues
    pub open_issues: usize,
    /// Number of closed issues
    pub closed_issues: usize,
    /// Number of blocked issues
    pub blocked_issues: usize,
    /// Database size in bytes
    pub database_size: u64,
    /// Last sync timestamp
    pub last_sync: Option<String>,
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self {
            total_issues: 0,
            open_issues: 0,
            closed_issues: 0,
            blocked_issues: 0,
            database_size: 0,
            last_sync: None,
        }
    }
}

/// Database view widget
pub struct DatabaseView<'a> {
    status: DatabaseStatus,
    stats: DatabaseStats,
    daemon_running: bool,
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> DatabaseView<'a> {
    /// Create a new database view
    pub fn new() -> Self {
        Self {
            status: DatabaseStatus::Ready,
            stats: DatabaseStats::default(),
            daemon_running: false,
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set database status
    pub fn status(mut self, status: DatabaseStatus) -> Self {
        self.status = status;
        self
    }

    /// Set database statistics
    pub fn stats(mut self, stats: DatabaseStats) -> Self {
        self.stats = stats;
        self
    }

    /// Set daemon running state
    pub fn daemon_running(mut self, running: bool) -> Self {
        self.daemon_running = running;
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    fn render_status(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Database Status",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status:        ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.status.display_name().to_string(),
                    Style::default()
                        .fg(self.status.color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Daemon:        ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if self.daemon_running {
                        "Running".to_string()
                    } else {
                        "Stopped".to_string()
                    },
                    Style::default().fg(if self.daemon_running {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Sync:     ", Style::default().fg(Color::Gray)),
                Span::raw(
                    self.stats
                        .last_sync
                        .as_deref()
                        .unwrap_or("Never")
                        .to_string(),
                ),
            ]),
        ]
    }

    fn render_statistics(&self) -> Vec<Line<'static>> {
        let total_issues = self.stats.total_issues;
        let open_issues = self.stats.open_issues;
        let closed_issues = self.stats.closed_issues;
        let blocked_issues = self.stats.blocked_issues;
        let database_size = self.stats.database_size;
        
        vec![
            Line::from(Span::styled(
                "Statistics",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Issues:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", total_issues),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Open:          ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", open_issues),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Closed:        ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", closed_issues),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(vec![
                Span::styled("Blocked:       ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", blocked_issues),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Database Size: ", Style::default().fg(Color::Gray)),
                Span::raw(format_size(database_size)),
            ]),
        ]
    }

    fn render_operations(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Available Operations",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::raw("         - Sync database with remote"),
            ]),
            Line::from(vec![
                Span::styled("i", Style::default().fg(Color::Green)),
                Span::raw("         - Import issues from file"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::raw("         - Export issues to file"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw("         - Start/stop daemon"),
            ]),
            Line::from(vec![
                Span::styled("r", Style::default().fg(Color::Green)),
                Span::raw("         - Refresh status"),
            ]),
            Line::from(vec![
                Span::styled("v", Style::default().fg(Color::Green)),
                Span::raw("         - Verify database integrity"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(Color::Green)),
                Span::raw("         - Compact database"),
            ]),
        ]
    }
}

impl<'a> Default for DatabaseView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Widget for DatabaseView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create layout: status + statistics + operations
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // Status
                Constraint::Length(11), // Statistics
                Constraint::Min(12),    // Operations
            ])
            .split(area);

        // Render status
        let status_lines = self.render_status();
        let status = Paragraph::new(status_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status")
                    .style(self.block_style),
            );
        status.render(chunks[0], buf);

        // Render statistics
        let stats_lines = self.render_statistics();
        let stats = Paragraph::new(stats_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Statistics")
                    .style(self.block_style),
            );
        stats.render(chunks[1], buf);

        // Render operations
        let ops_lines = self.render_operations();
        let ops = Paragraph::new(ops_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Operations")
                    .style(self.block_style),
            )
            .wrap(Wrap { trim: true });
        ops.render(chunks[2], buf);
    }
}

/// Format byte size to human-readable format
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_status_display_name() {
        assert_eq!(DatabaseStatus::Ready.display_name(), "Ready");
        assert_eq!(DatabaseStatus::Syncing.display_name(), "Syncing");
        assert_eq!(DatabaseStatus::Error.display_name(), "Error");
        assert_eq!(DatabaseStatus::Offline.display_name(), "Offline");
    }

    #[test]
    fn test_database_status_color() {
        assert_eq!(DatabaseStatus::Ready.color(), Color::Green);
        assert_eq!(DatabaseStatus::Syncing.color(), Color::Yellow);
        assert_eq!(DatabaseStatus::Error.color(), Color::Red);
        assert_eq!(DatabaseStatus::Offline.color(), Color::Gray);
    }

    #[test]
    fn test_database_view_creation() {
        let view = DatabaseView::new();
        assert_eq!(view.status, DatabaseStatus::Ready);
        assert!(!view.daemon_running);
    }

    #[test]
    fn test_database_view_status() {
        let view = DatabaseView::new().status(DatabaseStatus::Syncing);
        assert_eq!(view.status, DatabaseStatus::Syncing);
    }

    #[test]
    fn test_database_view_daemon_running() {
        let view = DatabaseView::new().daemon_running(true);
        assert!(view.daemon_running);
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(2048), "2.00 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(5_242_880), "5.00 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(2_147_483_648), "2.00 GB");
    }

    #[test]
    fn test_database_stats_default() {
        let stats = DatabaseStats::default();
        assert_eq!(stats.total_issues, 0);
        assert_eq!(stats.open_issues, 0);
        assert_eq!(stats.closed_issues, 0);
        assert_eq!(stats.blocked_issues, 0);
        assert_eq!(stats.database_size, 0);
        assert!(stats.last_sync.is_none());
    }
}
