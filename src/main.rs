pub mod beads;
pub mod config;
pub mod models;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Instant;
use ui::views::{DatabaseView, DependenciesView, HelpView, IssuesView, LabelsView};

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
            let search_focused = issues_state.search_state().search_state().is_focused();

            if search_focused {
                // Search input is focused - handle text input
                match key_code {
                    KeyCode::Char(c) => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .insert_char(c);
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Backspace => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .delete_char();
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Delete => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .delete_char_forward();
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Left => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_left();
                    }
                    KeyCode::Right => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_right();
                    }
                    KeyCode::Up => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .history_previous();
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Down => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .history_next();
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    KeyCode::Home => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_to_start();
                    }
                    KeyCode::End => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .move_cursor_to_end();
                    }
                    KeyCode::Enter => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .add_to_history();
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(false);
                    }
                    KeyCode::Esc => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(false);
                    }
                    _ => {}
                }
            } else {
                // List mode: navigation and quick actions
                match key_code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        let len = issues_state.search_state().filtered_issues().len();
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_next(len);
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
                    KeyCode::Char('c') => {
                        issues_state.enter_create_mode();
                    }
                    KeyCode::Char('x') => {
                        // Close selected issue
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Closing issue: {}", issue_id);
                            
                            // Create a tokio runtime to execute the async call
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let client = &app.beads_client;
                            
                            match rt.block_on(client.close_issue(&issue_id, None)) {
                                Ok(()) => {
                                    tracing::info!("Successfully closed issue: {}", issue_id);
                                    
                                    // Reload issues list
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to close issue: {:?}", e);
                                }
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        // Reopen selected issue
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Reopening issue: {}", issue_id);
                            
                            // Create a tokio runtime to execute the async call
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let client = &app.beads_client;
                            
                            match rt.block_on(client.reopen_issue(&issue_id)) {
                                Ok(()) => {
                                    tracing::info!("Successfully reopened issue: {}", issue_id);
                                    
                                    // Reload issues list
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reopen issue: {:?}", e);
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        // Delete selected issue (TODO: add confirmation dialog)
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::warn!("Deleting issue: {} (no confirmation yet)", issue_id);
                            
                            // Create a tokio runtime to execute the async call
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let client = &app.beads_client;
                            
                            match rt.block_on(client.delete_issue(&issue_id)) {
                                Ok(()) => {
                                    tracing::info!("Successfully deleted issue: {}", issue_id);
                                    
                                    // Reload issues list
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to delete issue: {:?}", e);
                                }
                            }
                        }
                    }
                    KeyCode::Char('/') => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(true);
                    }
                    KeyCode::Esc => {
                        issues_state.search_state_mut().clear_search();
                    }
                    _ => {}
                }
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
            if let Some(editor_state) = issues_state.editor_state_mut() {
                let form = editor_state.form_state_mut();

                match key_code {
                    // Field navigation
                    KeyCode::Tab | KeyCode::Down => {
                        form.focus_next();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.focus_previous();
                    }
                    // Text input
                    KeyCode::Char(c) => {
                        form.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        form.delete_char();
                    }
                    // Cursor movement
                    KeyCode::Left => {
                        form.move_cursor_left();
                    }
                    KeyCode::Right => {
                        form.move_cursor_right();
                    }
                    KeyCode::Home => {
                        form.move_cursor_to_start();
                    }
                    KeyCode::End => {
                        form.move_cursor_to_end();
                    }
                    // Save/Cancel
                    KeyCode::Enter => {
                        // Validate and save
                        if editor_state.validate() {
                            // Check if there are any changes
                            if !editor_state.has_changes() {
                                tracing::info!("No changes detected, returning to list");
                                issues_state.return_to_list();
                            } else {
                                // Get change summary for logging
                                let change_summary = editor_state.get_change_summary();
                                tracing::info!("Changes detected: {:?}", change_summary);
                                
                                // Get IssueUpdate with only changed fields
                                if let Some(update) = editor_state.get_issue_update() {
                                    let issue_id = editor_state.issue_id().to_string();
                                    
                                    // Mark as saved and return to list before reloading
                                    editor_state.save();
                                    issues_state.return_to_list();
                                    
                                    // Create a tokio runtime to execute the async call
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    let client = &app.beads_client;
                                    
                                    match rt.block_on(client.update_issue(&issue_id, update)) {
                                        Ok(()) => {
                                            tracing::info!("Successfully updated issue: {}", issue_id);
                                            
                                            // Reload issues list
                                            app.reload_issues();
                                        }
                                        Err(e) => {
                                            // TODO: Show error message to user in UI
                                            tracing::error!("Failed to update issue: {:?}", e);
                                            // TODO: Re-enter edit mode on error
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        issues_state.cancel_edit();
                    }
                    _ => {}
                }
            }
        }
        IssuesViewMode::Create => {
            // Create mode: form controls
            if let Some(create_form_state) = issues_state.create_form_state_mut() {
                let form = create_form_state.form_state_mut();

                match key_code {
                    // Field navigation
                    KeyCode::Tab | KeyCode::Down => {
                        form.focus_next();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.focus_previous();
                    }
                    // Text input
                    KeyCode::Char(c) => {
                        form.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        form.delete_char();
                    }
                    // Cursor movement
                    KeyCode::Left => {
                        form.move_cursor_left();
                    }
                    KeyCode::Right => {
                        form.move_cursor_right();
                    }
                    KeyCode::Home => {
                        form.move_cursor_to_start();
                    }
                    KeyCode::End => {
                        form.move_cursor_to_end();
                    }
                    // Submit/Cancel
                    KeyCode::Enter => {
                        // Validate and submit
                        if create_form_state.validate() {
                            if let Some(data) = app.issues_view_state.save_create() {
                                // Create a tokio runtime to execute the async call
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                let client = &app.beads_client;
                                
                                match rt.block_on(
                                    client.create_issue_full(
                                        &data.title,
                                        data.issue_type,
                                        data.priority,
                                        Some(&data.status),
                                        data.assignee.as_deref(),
                                        &data.labels,
                                        data.description.as_deref(),
                                    )
                                ) {
                                    Ok(issue_id) => {
                                        // Successfully created
                                        tracing::info!("Successfully created issue: {}", issue_id);
                                        
                                        // Reload issues list
                                        app.reload_issues();
                                        
                                        // Return to list
                                        app.issues_view_state.cancel_create();
                                        
                                        // TODO: Select the newly created issue in the list
                                    }
                                    Err(e) => {
                                        // TODO: Show error message to user in UI
                                        tracing::error!("Failed to create issue: {:?}", e);
                                        // For now, stay in create mode so user can fix and retry
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        issues_state.cancel_create();
                    }
                    _ => {}
                }
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
fn handle_help_view_event(key_code: KeyCode, app: &mut models::AppState) {
    match key_code {
        KeyCode::Right | KeyCode::Tab | KeyCode::Char('l') => {
            app.next_help_section();
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.previous_help_section();
        }
        _ => {}
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut models::AppState,
) -> Result<()> {
    loop {
        // Only render if state has changed (dirty checking)
        if app.is_dirty() {
            let start = Instant::now();
            terminal.draw(|f| ui(f, app))?;
            let render_time = start.elapsed();
            app.perf_stats.record_render(render_time);
            app.clear_dirty();
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Check for performance stats toggle (Ctrl+P or F12)
                if (key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL))
                    || key.code == KeyCode::F(12)
                {
                    app.toggle_perf_stats();
                    continue;
                }
                // Global key bindings
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                        continue;
                    }
                    KeyCode::Char('1') => {
                        app.selected_tab = 0;
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('2') => {
                        app.selected_tab = 1;
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('3') => {
                        app.selected_tab = 2;
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('4') => {
                        app.selected_tab = 3;
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('5') => {
                        app.selected_tab = 4;
                        app.mark_dirty();
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

                // Mark dirty after any key event handling
                app.mark_dirty();
            }
            Event::Resize(_, _) => {
                // Terminal was resized, need to redraw
                app.mark_dirty();
            }
            _ => {}
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
            let all_issues: Vec<_> = app
                .issues_view_state
                .search_state()
                .filtered_issues()
                .iter()
                .collect();
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
        _ => {
            // Help view (tab 4 and beyond)
            let help_view = HelpView::new().selected_section(app.help_section);
            f.render_widget(help_view, tabs_chunks[1]);
        }
    }

    // Status bar with optional performance stats
    let status_text = if app.show_perf_stats {
        let perf_info = app.perf_stats.format_stats();
        let mut lines: Vec<Line> = perf_info
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        // Add help text at the end
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Press Ctrl+P or F12 to toggle perf stats | 'q' to quit | Tab to switch",
        ));
        Paragraph::new(lines)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Performance"))
    } else {
        Paragraph::new(
            "Press 'q' to quit | Tab/Shift+Tab to switch tabs | 1-5 for direct tab access | Ctrl+P/F12 for perf stats",
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL))
    };
    f.render_widget(status_text, chunks[2]);
}
