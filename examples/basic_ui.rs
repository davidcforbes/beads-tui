//! Basic UI example demonstrating beads-tui components
//!
//! This example shows how to:
//! - Initialize the terminal
//! - Create a simple layout
//! - Render widgets
//! - Handle keyboard input
//! - Clean up on exit
//!
//! Run with: cargo run --example basic_ui

use beads_tui::ui::widgets::{Dialog, StatusBar, TabBar};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
    Terminal,
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

struct App {
    selected_tab: usize,
    selected_item: usize,
    show_dialog: bool,
    items: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            selected_tab: 0,
            selected_item: 0,
            show_dialog: false,
            items: vec![
                "Welcome to beads-tui!".to_string(),
                "Use ↑/↓ or j/k to navigate".to_string(),
                "Press Tab to switch tabs".to_string(),
                "Press 'd' to show dialog".to_string(),
                "Press '?' for help".to_string(),
                "Press 'q' to quit".to_string(),
            ],
        }
    }

    fn next_item(&mut self) {
        if self.selected_item < self.items.len() - 1 {
            self.selected_item += 1;
        }
    }

    fn previous_item(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 3;
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if handle_input(&mut app, key) {
                return Ok(());
            }
        }
    }
}

fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    // If dialog is shown, handle dialog input
    if app.show_dialog {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.show_dialog = false;
            }
            _ => {}
        }
        return false;
    }

    // Normal input handling
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('d') => app.show_dialog = true,
        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
        KeyCode::Tab => app.next_tab(),
        _ => {}
    }

    false
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.size();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    // Render tab bar
    let tabs = vec!["Issues", "Dependencies", "Labels"];
    let tab_bar = TabBar::new(tabs.clone()).selected(app.selected_tab);
    f.render_widget(CustomWidget(tab_bar), chunks[0]);

    // Render main content based on selected tab
    match app.selected_tab {
        0 => render_issues_tab(f, chunks[1], app),
        1 => render_dependencies_tab(f, chunks[1]),
        2 => render_labels_tab(f, chunks[1]),
        _ => {}
    }

    // Render status bar
    let status_bar = StatusBar::new().left(vec![Span::raw(format!(
        "Example App | Tab: {} | Item: {}/{}",
        tabs[app.selected_tab],
        app.selected_item + 1,
        app.items.len()
    ))]);
    f.render_widget(CustomWidget(status_bar), chunks[2]);

    // Render dialog if shown
    if app.show_dialog {
        let dialog = Dialog::new(
            "Example Dialog",
            "This is a demonstration dialog.\n\nPress Enter or Esc to close.",
        );
        let area = centered_rect(60, 40, size);
        f.render_widget(dialog, area);
    }
}

// Wrapper to make widgets with custom render() methods work with render_widget
struct CustomWidget<T>(T);

impl<T> Widget for CustomWidget<T>
where
    T: CustomRender,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.0.render(area, buf);
    }
}

trait CustomRender {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

impl<'a> CustomRender for TabBar<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        TabBar::render(self, area, buf);
    }
}

impl<'a> CustomRender for StatusBar<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        StatusBar::render(self, area, buf);
    }
}

fn render_issues_tab(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_item {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(item.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Getting Started"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_widget(list, area);
}

fn render_dependencies_tab(f: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled(
            "Dependencies View",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(""),
        Line::from("This tab would show:"),
        Line::from("• Issue dependency trees"),
        Line::from("• Blocking relationships"),
        Line::from("• Dependency cycles"),
        Line::from(""),
        Line::from("See the main application for full functionality!"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Dependencies"))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn render_labels_tab(f: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled(
            "Labels View",
            Style::default().fg(Color::Green),
        )]),
        Line::from(""),
        Line::from("This tab would show:"),
        Line::from("• All available labels"),
        Line::from("• Label usage statistics"),
        Line::from("• Label management options"),
        Line::from(""),
        Line::from("See the main application for full functionality!"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Labels"))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rect
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
