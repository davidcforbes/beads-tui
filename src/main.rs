pub mod beads;
pub mod config;
pub mod models;
pub mod ui;

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
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use ui::views::{DatabaseView, DependenciesView, HelpView, IssuesView, LabelsView};
use std::io;

fn main() -> Result<()> {
    // Setup panic hook to restore terminal on panic
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_panic(info);
    }));

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
    let mut app = models::AppState::new();

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
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

// App struct moved to models::AppState

/// Handle keyboard events for the Issues view
fn handle_issues_view_event(key_code: KeyCode, app: &mut models::AppState) {
    use ui::views::IssuesViewMode;

    let issues_state = &mut app.issues_view_state;
    let view_mode = issues_state.view_mode();

    match view_mode {
        IssuesViewMode::List => {
            // List mode: navigation, search, and quick actions
            match key_code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let len = issues_state.search_state().filtered_issues().len();
                    issues_state.search_state_mut().list_state_mut().select_next(len);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let len = issues_state.search_state().filtered_issues().len();
                    issues_state
                        .search_state_mut()
                        .list_state_mut()
                        .select_previous(len);
                }
                KeyCode::Enter => {
                    issues_state.enter_detail_view();
                }
                KeyCode::Char('e') => {
                    issues_state.enter_edit_mode();
                }
                KeyCode::Char('/') => {
                    issues_state.search_state_mut().search_state_mut().set_focused(true);
                }
                KeyCode::Esc => {
                    issues_state.search_state_mut().clear_search();
                    issues_state.search_state_mut().search_state_mut().set_focused(false);
                }
                _ => {}
            }
        }
        IssuesViewMode::Detail => {
            // Detail mode: view navigation
            match key_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    issues_state.return_to_list();
                }
                KeyCode::Char('e') => {
                    issues_state.return_to_list();
                    issues_state.enter_edit_mode();
                }
                _ => {}
            }
        }
        IssuesViewMode::Edit => {
            // Edit mode: form controls
            match key_code {
                KeyCode::Esc => {
                    issues_state.cancel_edit();
                }
                // TODO: Add more editor controls (Tab for field navigation, etc.)
                _ => {}
            }
        }
    }
}

/// Handle keyboard events for the Dependencies view
fn handle_dependencies_view_event(_key_code: KeyCode, _app: &mut models::AppState) {
    // TODO: Implement dependency view controls
}

/// Handle keyboard events for the Labels view
fn handle_labels_view_event(_key_code: KeyCode, _app: &mut models::AppState) {
    // TODO: Implement labels view controls
}

/// Handle keyboard events for the Database view
fn handle_database_view_event(_key_code: KeyCode, _app: &mut models::AppState) {
    // TODO: Implement database view controls
}

/// Handle keyboard events for the Help view
fn handle_help_view_event(_key_code: KeyCode, _app: &mut models::AppState) {
    // TODO: Implement help view controls
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut models::AppState,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Global key bindings
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                        continue;
                    }
                    KeyCode::Char('1') => {
                        app.selected_tab = 0;
                        continue;
                    }
                    KeyCode::Char('2') => {
                        app.selected_tab = 1;
                        continue;
                    }
                    KeyCode::Char('3') => {
                        app.selected_tab = 2;
                        continue;
                    }
                    KeyCode::Char('4') => {
                        app.selected_tab = 3;
                        continue;
                    }
                    KeyCode::Char('5') => {
                        app.selected_tab = 4;
                        continue;
                    }
                    _ => {}
                }

                // Tab-specific key bindings
                match app.selected_tab {
                    0 => handle_issues_view_event(key.code, app),
                    1 => handle_dependencies_view_event(key.code, app),
                    2 => handle_labels_view_event(key.code, app),
                    3 => handle_database_view_event(key.code, app),
                    4 => handle_help_view_event(key.code, app),
                    _ => {}
                }

                // Handle global tab navigation after view-specific handling
                match key.code {
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.previous_tab(),
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

fn ui(f: &mut Frame, app: &mut models::AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("Beads-TUI v0.1.0")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
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
    match app.selected_tab {
        0 => {
            // Issues view (stateful)
            let issues_view = IssuesView::new();
            f.render_stateful_widget(issues_view, tabs_chunks[1], &mut app.issues_view_state);
        }
        1 => {
            // Dependencies view
            let all_issues: Vec<_> = app.issues_view_state.search_state().filtered_issues().iter().collect();
            let selected_issue = app.issues_view_state.selected_issue();
            let mut dependencies_view = DependenciesView::new(all_issues);
            if let Some(issue) = selected_issue {
                dependencies_view = dependencies_view.issue(issue);
            }
            f.render_widget(dependencies_view, tabs_chunks[1]);
        }
        2 => {
            // Labels view
            let labels_view = LabelsView::new().labels(app.label_stats.clone());
            f.render_widget(labels_view, tabs_chunks[1]);
        }
        3 => {
            // Database view
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(false); // TODO: Check actual daemon status
            f.render_widget(database_view, tabs_chunks[1]);
        }
        4 | _ => {
            // Help view
            let help_view = HelpView::new();
            f.render_widget(help_view, tabs_chunks[1]);
        }
    }

    // Status bar
    let status_bar = Paragraph::new(
        "Press 'q' to quit | Tab/Shift+Tab to switch tabs | 1-5 for direct tab access",
    )
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, chunks[2]);
}


