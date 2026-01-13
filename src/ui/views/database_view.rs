//! Database view for status, sync, and management operations

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Database view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseViewMode {
    Dashboard,
    Sync,
    Maintenance,
    Daemon,
}

impl DatabaseViewMode {
    pub fn all() -> Vec<Self> {
        vec![Self::Dashboard, Self::Sync, Self::Maintenance, Self::Daemon]
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Sync => "Sync",
            Self::Maintenance => "Maintenance",
            Self::Daemon => "Daemon",
        }
    }
}

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
#[derive(Debug, Clone, Default)]
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

use crate::ui::widgets::LoadingIndicator;

/// Database view state
#[derive(Debug)]
pub struct DatabaseViewState {
    pub mode: DatabaseViewMode,
    pub sync_logs: Vec<String>,
    pub daemon_logs: Vec<String>,
    pub integrity_report: Option<String>,
    pub active_operation: Option<String>,
    pub operation_progress: Option<f64>,
    pub is_input_focused: bool,
    pub input_value: String,
    pub input_prompt: String,
}

impl Default for DatabaseViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseViewState {
    pub fn new() -> Self {
        Self {
            mode: DatabaseViewMode::Dashboard,
            sync_logs: Vec::new(),
            daemon_logs: Vec::new(),
            integrity_report: None,
            active_operation: None,
            operation_progress: None,
            is_input_focused: false,
            input_value: String::new(),
            input_prompt: String::new(),
        }
    }

    pub fn set_mode(&mut self, mode: DatabaseViewMode) {
        self.mode = mode;
    }

    pub fn add_sync_log(&mut self, log: String) {
        self.sync_logs.push(log);
    }

    pub fn add_daemon_log(&mut self, log: String) {
        self.daemon_logs.push(log);
    }

    pub fn set_integrity_report(&mut self, report: String) {
        self.integrity_report = Some(report);
    }

    pub fn start_operation(&mut self, name: String) {
        self.active_operation = Some(name);
        self.operation_progress = None;
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.operation_progress = Some(progress);
    }

    pub fn finish_operation(&mut self) {
        self.active_operation = None;
        self.operation_progress = None;
    }

    pub fn start_input(&mut self, prompt: String, default: String) {
        self.is_input_focused = true;
        self.input_prompt = prompt;
        self.input_value = default;
    }

    pub fn finish_input(&mut self) -> String {
        self.is_input_focused = false;
        let val = self.input_value.clone();
        self.input_value.clear();
        val
    }

    pub fn cancel_input(&mut self) {
        self.is_input_focused = false;
        self.input_value.clear();
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

    fn render_dashboard(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // Status
                Constraint::Length(11), // Statistics
                Constraint::Min(0),     // Hints
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
    }

    fn render_sync(&self, area: Rect, buf: &mut Buffer, state: &DatabaseViewState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if state.active_operation.is_some() { 3 } else { 0 }),
                Constraint::Min(0),
            ])
            .split(area);

        // Render progress if active
        if let Some(ref op) = state.active_operation {
            let indicator = if let Some(p) = state.operation_progress {
                LoadingIndicator::with_progress(op, p)
            } else {
                LoadingIndicator::new(op)
            };
            indicator.render(chunks[0], buf);
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Sync Log")
            .style(self.block_style);
        
        let logs: Vec<Line> = state.sync_logs.iter().rev().map(|l| Line::from(l.as_str())).collect();
        let p = if logs.is_empty() {
            Paragraph::new("No recent activity.").style(Style::default().fg(Color::DarkGray))
        } else {
            Paragraph::new(logs)
        };
        
        p.block(block).render(chunks[1], buf);
    }

    fn render_maintenance(&self, area: Rect, buf: &mut Buffer, state: &DatabaseViewState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Controls
                Constraint::Min(0),    // Integrity Report
            ])
            .split(area);

        let ops_block = Block::default()
            .borders(Borders::ALL)
            .title("Database Maintenance")
            .style(self.block_style);
        
        let ops = vec![
            Line::from(vec![Span::styled("i", Style::default().fg(Color::Green)), Span::raw(" - Import Issues")]),
            Line::from(vec![Span::styled("e", Style::default().fg(Color::Green)), Span::raw(" - Export Issues")]),
            Line::from(vec![Span::styled("v", Style::default().fg(Color::Green)), Span::raw(" - Verify Integrity")]),
            Line::from(vec![Span::styled("c", Style::default().fg(Color::Green)), Span::raw(" - Compact Database (Destructive History)")]),
        ];
        
        Paragraph::new(ops).block(ops_block).render(chunks[0], buf);

        let report_block = Block::default()
            .borders(Borders::ALL)
            .title("Integrity Report")
            .style(self.block_style);
        
        let report = state.integrity_report.as_deref().unwrap_or("No report available. Run 'v' to verify.");
        Paragraph::new(report)
            .block(report_block)
            .wrap(Wrap { trim: true })
            .render(chunks[1], buf);
    }

    fn render_daemon(&self, area: Rect, buf: &mut Buffer, state: &DatabaseViewState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Status & Controls
                Constraint::Min(0),    // Logs
            ])
            .split(area);

        let status_block = Block::default()
            .borders(Borders::ALL)
            .title("Daemon Management")
            .style(self.block_style);
        
        let status = if self.daemon_running {
            Span::styled("Running", Style::default().fg(Color::Green))
        } else {
            Span::styled("Stopped", Style::default().fg(Color::Red))
        };
        
        let lines = vec![
            Line::from(vec![Span::raw("Current Status: "), status]),
            Line::from(""),
            Line::from(vec![Span::styled("d", Style::default().fg(Color::Green)), Span::raw(" - Start/Stop Daemon")]),
            Line::from(vec![Span::styled("k", Style::default().fg(Color::Red)), Span::raw(" - Kill All beads processes (Force)")]),
        ];
        
        Paragraph::new(lines).block(status_block).render(chunks[0], buf);

        let log_block = Block::default()
            .borders(Borders::ALL)
            .title("Daemon Logs")
            .style(self.block_style);
        
        let logs: Vec<Line> = state.daemon_logs.iter().rev().map(|l| Line::from(l.as_str())).collect();
        let p = if logs.is_empty() {
            Paragraph::new("No recent daemon logs.").style(Style::default().fg(Color::DarkGray))
        } else {
            Paragraph::new(logs)
        };
        
        p.block(log_block).render(chunks[1], buf);
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
                    format!("{total_issues}"),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Open:          ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{open_issues}"),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Closed:        ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{closed_issues}"),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(vec![
                Span::styled("Blocked:       ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{blocked_issues}"),
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
}

impl<'a> Default for DatabaseView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for DatabaseView<'a> {
    type State = DatabaseViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create layout: tabs (3) + content (fill) + help (1)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Sub-tabs
                Constraint::Min(0),    // Mode Content
                Constraint::Length(1), // Navigation hints
            ])
            .split(area);

        // Render Sub-tabs
        let tabs: Vec<Line> = DatabaseViewMode::all().iter().map(|m| {
            let style = if state.mode == *m {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!(" {} ", m.display_name()), style))
        }).collect();
        
        let tab_bar = Paragraph::new(tabs)
            .block(Block::default().borders(Borders::ALL).title("Database Operations"));
        // Note: Simple paragraph representation of tabs for now, could use Tabs widget
        tab_bar.render(chunks[0], buf);

        // Render Content based on mode
        match state.mode {
            DatabaseViewMode::Dashboard => self.render_dashboard(chunks[1], buf),
            DatabaseViewMode::Sync => self.render_sync(chunks[1], buf, state),
            DatabaseViewMode::Maintenance => self.render_maintenance(chunks[1], buf, state),
            DatabaseViewMode::Daemon => self.render_daemon(chunks[1], buf, state),
        }

        // Render Help
        let help = Paragraph::new("[/]: Switch Modes | s: Sync | r: Refresh")
            .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[2], buf);

        // Render input overlay if active
        if state.is_input_focused {
            let input_area = centered_rect(60, 20, area);
            let input_block = Block::default()
                .borders(Borders::ALL)
                .title(state.input_prompt.clone())
                .style(Style::default().fg(Color::Yellow));
            
            let input_p = Paragraph::new(state.input_value.clone())
                .block(input_block);
            
            // Clear area before rendering input
            let clear_area = input_area;
            for y in clear_area.top()..clear_area.bottom() {
                for x in clear_area.left()..clear_area.right() {
                    buf.get_mut(x, y).set_symbol(" ");
                }
            }
            input_p.render(input_area, buf);
        }
    }
}

/// Helper to create a centered rect (duplicate of main.rs helper, consider moving to common)
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Format byte size to human-readable format
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{bytes} B")
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
