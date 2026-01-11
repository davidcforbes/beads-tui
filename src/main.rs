use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run the app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

struct App {
    should_quit: bool,
    selected_tab: usize,
    tabs: Vec<&'static str>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            selected_tab: 0,
            tabs: vec!["Issues", "Dependencies", "Labels", "Database", "Help"],
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
    }

    fn previous_tab(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.tabs.len() - 1;
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.previous_tab(),
                    KeyCode::Char('1') => app.selected_tab = 0,
                    KeyCode::Char('2') => app.selected_tab = 1,
                    KeyCode::Char('3') => app.selected_tab = 2,
                    KeyCode::Char('4') => app.selected_tab = 3,
                    KeyCode::Char('5') => app.selected_tab = 4,
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("Beads-TUI v0.1.0")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Tabs and content
    let tabs_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    // Tab bar
    let tabs: Vec<ListItem> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let style = if i == app.selected_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {} {} ", i + 1, name)).style(style)
        })
        .collect();

    let tabs_widget = List::new(tabs)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(tabs_widget, tabs_chunks[0]);

    // Content area based on selected tab
    let content = match app.selected_tab {
        0 => render_issues_view(),
        1 => render_dependencies_view(),
        2 => render_labels_view(),
        3 => render_database_view(),
        4 => render_help_view(),
        _ => render_help_view(),
    };
    f.render_widget(content, tabs_chunks[1]);

    // Status bar
    let status_bar = Paragraph::new("Press 'q' to quit | Tab/Shift+Tab to switch tabs | 1-5 for direct tab access")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, chunks[2]);
}

fn render_issues_view() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Issues View",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This will show the list of issues from your beads database."),
        Line::from(""),
        Line::from("Features to implement:"),
        Line::from("  • Issue list with filtering"),
        Line::from("  • Create new issue"),
        Line::from("  • Edit existing issues"),
        Line::from("  • Close/reopen issues"),
        Line::from("  • Bulk operations"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Issues"))
}

fn render_dependencies_view() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Dependencies View",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This will show dependency trees and relationships."),
        Line::from(""),
        Line::from("Features to implement:"),
        Line::from("  • Dependency tree visualization"),
        Line::from("  • Add/remove dependencies"),
        Line::from("  • Cycle detection"),
        Line::from("  • Dependency graph"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Dependencies"))
}

fn render_labels_view() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Labels View",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This will show label management interface."),
        Line::from(""),
        Line::from("Features to implement:"),
        Line::from("  • Label browser"),
        Line::from("  • Add/remove labels"),
        Line::from("  • Label autocomplete"),
        Line::from("  • State management"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Labels"))
}

fn render_database_view() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Database View",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This will show database status and operations."),
        Line::from(""),
        Line::from("Features to implement:"),
        Line::from("  • Database dashboard"),
        Line::from("  • Import/export"),
        Line::from("  • Daemon management"),
        Line::from("  • Sync operations"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Database"))
}

fn render_help_view() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Help & Keyboard Shortcuts",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("Global:", Style::default().fg(Color::Cyan))),
        Line::from("  q         - Quit application"),
        Line::from("  Tab       - Next tab"),
        Line::from("  Shift+Tab - Previous tab"),
        Line::from("  1-5       - Jump to tab directly"),
        Line::from(""),
        Line::from(Span::styled("Coming Soon:", Style::default().fg(Color::Cyan))),
        Line::from("  ?         - Show help"),
        Line::from("  /         - Search"),
        Line::from("  :         - Command palette"),
        Line::from("  n         - New issue"),
        Line::from("  f         - Filter builder"),
        Line::from(""),
        Line::from(Span::styled("About:", Style::default().fg(Color::Cyan))),
        Line::from("  Beads-TUI v0.1.0"),
        Line::from("  Interactive terminal UI for Beads"),
        Line::from("  https://github.com/YOUR_USERNAME/beads-tui"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"))
}
