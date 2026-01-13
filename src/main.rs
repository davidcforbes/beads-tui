pub mod beads;
pub mod config;
pub mod models;
pub mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Instant;
use ui::views::{
    BondType, BondingInterface, DatabaseView, DependenciesView, FormulaBrowser, GanttView,
    HelpView, HistoryOps, IssuesView, KanbanView, LabelsView, PertView, PourStep, PourWizard,
    PourWizardState, WispManager,
};
use ui::widgets::TabBar;

/// Interactive terminal UI for Beads issue management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to beads repository (defaults to current directory)
    #[arg(short, long)]
    path: Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI arguments (handles --version, --help automatically)
    let _cli = Cli::parse();

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
fn handle_issues_view_event(key: KeyEvent, app: &mut models::AppState) {
    use ui::views::IssuesViewMode;

    let key_code = key.code;

    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    // Clear errors when entering create or edit mode
    if matches!(key_code, KeyCode::Char('c') | KeyCode::Char('e')) {
        app.clear_error();
    }

    // Handle dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.dialog_state {
        match key_code {
            KeyCode::Left => {
                dialog_state.select_previous(2); // Yes/No = 2 buttons
                return;
            }
            KeyCode::Right | KeyCode::Tab => {
                dialog_state.select_next(2);
                return;
            }
            KeyCode::Enter => {
                // Execute pending action based on selected button
                let selected = dialog_state.selected_button();
                if selected == 0 {
                    // Yes was selected
                    if let Some(action) = app.pending_action.take() {
                        if let Some(issue_id) = action.strip_prefix("delete:") {
                            tracing::info!("Confirmed delete for issue: {}", issue_id);

                            // Create a tokio runtime to execute the async call
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let client = &app.beads_client;

                            match rt.block_on(client.delete_issue(issue_id)) {
                                Ok(()) => {
                                    tracing::info!("Successfully deleted issue: {}", issue_id);
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to delete issue: {:?}", e);
                                }
                            }
                        } else if action == "compact_database" {
                            tracing::info!("Confirmed compact database");

                            // Create a tokio runtime to execute the async call
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let client = &app.beads_client;

                            match rt.block_on(client.compact_database()) {
                                Ok(()) => {
                                    tracing::info!("Successfully compacted database");
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to compact database: {:?}", e);
                                    app.set_error(format!("Failed to compact database: {e}"));
                                }
                            }
                        }
                    }
                }
                // Close dialog (both Yes and No)
                app.dialog_state = None;
                app.pending_action = None;
                return;
            }
            KeyCode::Esc => {
                // Cancel dialog
                tracing::debug!("Dialog cancelled");
                app.dialog_state = None;
                app.pending_action = None;
                return;
            }
            _ => {
                // Ignore other keys when dialog is active
                return;
            }
        }
    }

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
            } else if issues_state.search_state().list_state().is_editing() {
                // In-place editing mode: handle title editing
                match key_code {
                    KeyCode::Enter => {
                        // Save the edited title
                        let list_state = issues_state.search_state_mut().list_state_mut();
                        if let Some(new_title) = list_state.finish_editing() {
                            // Get the selected issue
                            if let Some(issue) = issues_state.search_state().selected_issue() {
                                let issue_id = issue.id.clone();
                                tracing::info!("Saving title edit for {}: {}", issue_id, new_title);

                                // Create IssueUpdate with only title
                                let update =
                                    crate::beads::client::IssueUpdate::new().title(new_title);

                                // Execute the update
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                let client = &app.beads_client;

                                match rt.block_on(client.update_issue(&issue_id, update)) {
                                    Ok(()) => {
                                        tracing::info!(
                                            "Successfully updated title for: {}",
                                            issue_id
                                        );
                                        app.reload_issues();
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to update title: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Cancel editing
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .cancel_editing();
                    }
                    KeyCode::Char(ch) => {
                        // Insert character
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .insert_char_at_cursor(ch);
                    }
                    KeyCode::Backspace => {
                        // Delete character before cursor
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .delete_char_before_cursor();
                    }
                    KeyCode::Left => {
                        // Move cursor left
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_cursor_left();
                    }
                    KeyCode::Right => {
                        // Move cursor right
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_cursor_right();
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
                        // Delete selected issue with confirmation dialog
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            let issue_title = issue.title.clone();
                            tracing::info!("Requesting confirmation to delete issue: {}", issue_id);

                            // Show confirmation dialog
                            app.dialog_state = Some(ui::widgets::DialogState::new());
                            app.pending_action = Some(format!("delete:{issue_id}"));

                            // Store issue title for dialog message (we'll need to format it in rendering)
                            tracing::debug!("Showing delete confirmation for: {}", issue_title);
                        }
                    }
                    KeyCode::Char('r') => {
                        // Start in-place editing of title
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let title = issue.title.clone();
                            if let Some(selected_idx) =
                                issues_state.search_state().list_state().selected()
                            {
                                tracing::info!(
                                    "Starting in-place edit for {}: {}",
                                    issue.id,
                                    title
                                );
                                issues_state
                                    .search_state_mut()
                                    .list_state_mut()
                                    .start_editing(selected_idx, title);
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        // Toggle quick filters
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .toggle_filters();
                        issues_state.search_state_mut().update_filtered_issues();
                        let enabled = issues_state.search_state().list_state().filters_enabled();
                        tracing::info!(
                            "Quick filters toggled: {}",
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }
                    KeyCode::Char('/') => {
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(true);
                    }
                    KeyCode::Char('v') => {
                        // Cycle to next view
                        issues_state.search_state_mut().next_view();
                        let view_name = issues_state.search_state().current_view().display_name();
                        tracing::info!("Switched to view: {}", view_name);
                    }
                    KeyCode::Esc => {
                        issues_state.search_state_mut().clear_search();
                    }
                    KeyCode::Left
                        if key
                            .modifiers
                            .contains(KeyModifiers::ALT | KeyModifiers::SHIFT) =>
                    {
                        // Alt+Shift+Left: Shrink focused column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .shrink_focused_column();
                        tracing::debug!("Shrinking focused column");
                    }
                    KeyCode::Right
                        if key
                            .modifiers
                            .contains(KeyModifiers::ALT | KeyModifiers::SHIFT) =>
                    {
                        // Alt+Shift+Right: Grow focused column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .grow_focused_column();
                        tracing::debug!("Growing focused column");
                    }
                    KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Left: Move focused column left
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_left();
                        tracing::debug!("Moving focused column left");
                    }
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Right: Move focused column right
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_right();
                        tracing::debug!("Moving focused column right");
                    }
                    KeyCode::Tab if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Tab: Focus next column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .focus_next_column();
                        tracing::debug!("Focusing next column");
                    }
                    KeyCode::BackTab if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Shift+Tab: Focus previous column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .focus_previous_column();
                        tracing::debug!("Focusing previous column");
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

                // Check for Ctrl+L first (before generic Char handler)
                if key_code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Load from file (Ctrl+L)
                    // Get the current field value as the file path
                    if let Some(focused_field) = form.focused_field() {
                        let file_path = focused_field.value.trim().to_string();

                        if file_path.is_empty() {
                            // Set error on focused field
                            if let Some(field) = form.focused_field_mut() {
                                field.error = Some(
                                    "Enter a file path first, then press Ctrl+L to load it"
                                        .to_string(),
                                );
                            }
                        } else {
                            // Try to load from file
                            match form.load_from_file(&file_path) {
                                Ok(()) => {
                                    tracing::info!("Loaded content from file: {}", file_path);
                                }
                                Err(err) => {
                                    tracing::error!("Failed to load file {}: {}", file_path, err);
                                    // Error is already set in the field by load_from_file
                                }
                            }
                        }
                    }
                } else {
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
                                let changes = editor_state.get_changes();
                                if changes.is_empty() {
                                    tracing::info!("No changes detected, returning to list");
                                    issues_state.return_to_list();
                                } else {
                                    // Log changes
                                    tracing::info!("Changes detected: {:?}", changes);

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
                                                tracing::info!(
                                                    "Successfully updated issue: {}",
                                                    issue_id
                                                );

                                                // Clear any previous errors
                                                app.clear_error();

                                                // Reload issues list
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to update issue: {:?}", e);
                                                app.set_error(format!(
                                                    "Failed to update issue: {e}"
                                                ));
                                                // Stay in edit mode so user can fix and retry
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
        }
        IssuesViewMode::Create => {
            // Create mode: form controls
            if let Some(create_form_state) = issues_state.create_form_state_mut() {
                let form = create_form_state.form_state_mut();

                // Check for Ctrl+L first (before generic Char handler)
                if key_code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Load from file (Ctrl+L)
                    // Get the current field value as the file path
                    if let Some(focused_field) = form.focused_field() {
                        let file_path = focused_field.value.trim().to_string();

                        if file_path.is_empty() {
                            // Set error on focused field
                            if let Some(field) = form.focused_field_mut() {
                                field.error = Some(
                                    "Enter a file path first, then press Ctrl+L to load it"
                                        .to_string(),
                                );
                            }
                        } else {
                            // Try to load from file
                            match form.load_from_file(&file_path) {
                                Ok(()) => {
                                    tracing::info!("Loaded content from file: {}", file_path);
                                }
                                Err(err) => {
                                    tracing::error!("Failed to load file {}: {}", file_path, err);
                                    // Error is already set in the field by load_from_file
                                }
                            }
                        }
                    }
                } else {
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

                                    // Build create params
                                    let mut params = beads::models::CreateIssueParams::new(
                                        &data.title,
                                        data.issue_type,
                                        data.priority,
                                    );
                                    params.status = Some(&data.status);
                                    params.assignee = data.assignee.as_deref();
                                    params.labels = &data.labels;
                                    params.description = data.description.as_deref();

                                    match rt.block_on(client.create_issue_full(params)) {
                                        Ok(issue_id) => {
                                            // Successfully created
                                            tracing::info!(
                                                "Successfully created issue: {}",
                                                issue_id
                                            );

                                            // Clear any previous errors
                                            app.clear_error();

                                            // Reload issues list
                                            app.reload_issues();

                                            // Return to list
                                            app.issues_view_state.cancel_create();

                                            // Select the newly created issue in the list
                                            let created_issue = app
                                                .issues_view_state
                                                .search_state()
                                                .filtered_issues()
                                                .iter()
                                                .find(|issue| issue.id == issue_id)
                                                .cloned();

                                            if let Some(issue) = created_issue {
                                                app.issues_view_state
                                                    .set_selected_issue(Some(issue));
                                                tracing::debug!(
                                                    "Selected newly created issue: {}",
                                                    issue_id
                                                );
                                            } else {
                                                tracing::warn!(
                                                    "Could not find newly created issue {} in list",
                                                    issue_id
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to create issue: {:?}", e);
                                            app.set_error(format!("Failed to create issue: {e}"));
                                            // Stay in create mode so user can fix and retry
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
}

/// Handle keyboard events for the Dependencies view
fn handle_dependencies_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    let selected_issue = app.issues_view_state.selected_issue();

    match key_code {
        KeyCode::Char('j') | KeyCode::Down => {
            // Get the length of the focused list
            let len = if let Some(issue) = selected_issue {
                match app.dependencies_view_state.focus() {
                    ui::views::DependencyFocus::Dependencies => issue.dependencies.len(),
                    ui::views::DependencyFocus::Blocks => issue.blocks.len(),
                }
            } else {
                0
            };
            app.dependencies_view_state.select_next(len);
            app.mark_dirty();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let len = if let Some(issue) = selected_issue {
                match app.dependencies_view_state.focus() {
                    ui::views::DependencyFocus::Dependencies => issue.dependencies.len(),
                    ui::views::DependencyFocus::Blocks => issue.blocks.len(),
                }
            } else {
                0
            };
            app.dependencies_view_state.select_previous(len);
            app.mark_dirty();
        }
        KeyCode::Tab => {
            // Toggle focus between dependencies and blocks
            app.dependencies_view_state.toggle_focus();
            app.mark_dirty();
        }
        KeyCode::Char('a') => {
            // Add dependency
            app.set_info("Add dependency: Not yet implemented (requires input dialog)".to_string());
            tracing::info!("Add dependency requested");
        }
        KeyCode::Char('d') => {
            // Remove dependency
            if let Some(issue) = selected_issue {
                match app.dependencies_view_state.focus() {
                    ui::views::DependencyFocus::Dependencies => {
                        if let Some(selected_idx) =
                            app.dependencies_view_state.selected_dependency()
                        {
                            if selected_idx < issue.dependencies.len() {
                                let dep_id = issue.dependencies[selected_idx].clone();
                                app.set_info(format!(
                                    "Remove dependency '{}': Not yet implemented",
                                    dep_id
                                ));
                                tracing::info!("Remove dependency requested: {}", dep_id);
                            }
                        }
                    }
                    ui::views::DependencyFocus::Blocks => {
                        if let Some(selected_idx) = app.dependencies_view_state.selected_block() {
                            if selected_idx < issue.blocks.len() {
                                let blocked_id = issue.blocks[selected_idx].clone();
                                app.set_info(format!(
                                    "Remove block relationship with '{}': Not yet implemented",
                                    blocked_id
                                ));
                                tracing::info!("Remove block requested: {}", blocked_id);
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Char('g') => {
            // Show dependency graph
            app.set_info("Show dependency graph: Not yet implemented".to_string());
            tracing::info!("Show dependency graph requested");
        }
        KeyCode::Char('c') => {
            // Check for circular dependencies
            app.set_info("Check circular dependencies: Not yet implemented".to_string());
            tracing::info!("Check circular dependencies requested");
        }
        _ => {}
    }
}

/// Handle keyboard events for the Labels view
fn handle_labels_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    let labels_len = app.label_stats.len();

    match key_code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.labels_view_state.select_next(labels_len);
            app.mark_dirty();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.labels_view_state.select_previous(labels_len);
            app.mark_dirty();
        }
        KeyCode::Char('a') => {
            // Add label - show notification for now (needs input dialog widget)
            app.set_info("Add label: Not yet implemented (requires input dialog)".to_string());
            tracing::info!("Add label requested");
        }
        KeyCode::Char('d') => {
            // Delete selected label
            if let Some(selected_idx) = app.labels_view_state.selected() {
                if selected_idx < app.label_stats.len() {
                    let label_name = app.label_stats[selected_idx].name.clone();
                    app.set_info(format!(
                        "Delete label '{}': Not yet implemented",
                        label_name
                    ));
                    tracing::info!("Delete label requested: {}", label_name);
                }
            }
        }
        KeyCode::Char('e') => {
            // Edit selected label
            if let Some(selected_idx) = app.labels_view_state.selected() {
                if selected_idx < app.label_stats.len() {
                    let label_name = app.label_stats[selected_idx].name.clone();
                    app.set_info(format!("Edit label '{}': Not yet implemented", label_name));
                    tracing::info!("Edit label requested: {}", label_name);
                }
            }
        }
        KeyCode::Char('s') => {
            // Show statistics - already visible in the view
            app.set_info("Label statistics are displayed in the summary panel".to_string());
        }
        KeyCode::Char('/') => {
            // Search labels
            app.set_info(
                "Search labels: Not yet implemented (requires search input widget)".to_string(),
            );
            tracing::info!("Search labels requested");
        }
        _ => {}
    }
}

/// Handle keyboard events for the Database view
fn handle_database_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    // Handle input mode
    if app.database_view_state.is_input_focused {
        match key_code {
            KeyCode::Char(c) => {
                app.database_view_state.input_value.push(c);
                app.mark_dirty();
            }
            KeyCode::Backspace => {
                app.database_view_state.input_value.pop();
                app.mark_dirty();
            }
            KeyCode::Enter => {
                let prompt = app.database_view_state.input_prompt.clone();
                let filename = app.database_view_state.finish_input();

                let rt = tokio::runtime::Runtime::new().unwrap();

                if prompt.contains("Export") {
                    app.database_view_state
                        .add_sync_log(format!("Exporting to {}...", filename));
                    app.database_view_state
                        .start_operation("Exporting".to_string());
                    app.mark_dirty();
                    let client = &app.beads_client;
                    match rt.block_on(client.export_issues(&filename)) {
                        Ok(_) => app.set_success(format!("Exported to {}", filename)),
                        Err(e) => app.set_error(format!("Export failed: {}", e)),
                    }
                } else if prompt.contains("Import") {
                    app.database_view_state
                        .add_sync_log(format!("Importing from {}...", filename));
                    app.database_view_state
                        .start_operation("Importing".to_string());
                    app.mark_dirty();
                    let client = &app.beads_client;
                    match rt.block_on(client.import_issues(&filename)) {
                        Ok(_) => app.set_success(format!("Imported from {}", filename)),
                        Err(e) => app.set_error(format!("Import failed: {}", e)),
                    }
                }
                app.database_view_state.finish_operation();
                app.mark_dirty();
            }
            KeyCode::Esc => {
                app.database_view_state.cancel_input();
                app.mark_dirty();
            }
            _ => {}
        }
        return;
    }

    // Mode switching
    if key_code == KeyCode::Char('/') {
        use ui::views::DatabaseViewMode;
        let modes = DatabaseViewMode::all();
        let current_idx = modes
            .iter()
            .position(|m| *m == app.database_view_state.mode)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % modes.len();
        app.database_view_state.set_mode(modes[next_idx]);
        app.mark_dirty();
        return;
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = app.beads_client.clone();

    match key_code {
        KeyCode::Char('r') => {
            // Refresh database status
            tracing::info!("Refreshing database status");
            app.database_view_state
                .add_sync_log("Refreshing status...".to_string());
            app.reload_issues();
        }
        KeyCode::Char('d') => {
            // Toggle daemon (start/stop)
            tracing::info!("Toggling daemon");
            app.database_view_state
                .add_daemon_log("Toggling daemon...".to_string());

            if app.daemon_running {
                // Stop daemon
                match rt.block_on(client.stop_daemon()) {
                    Ok(_) => {
                        tracing::info!("Daemon stopped successfully");
                        app.daemon_running = false;
                        app.database_view_state
                            .add_daemon_log("Daemon stopped.".to_string());
                        app.mark_dirty();
                    }
                    Err(e) => {
                        tracing::error!("Failed to stop daemon: {:?}", e);
                        app.database_view_state
                            .add_daemon_log(format!("Failed to stop: {}", e));
                        app.set_error(format!("Failed to stop daemon: {e}"));
                    }
                }
            } else {
                // Start daemon
                match rt.block_on(client.start_daemon()) {
                    Ok(_) => {
                        tracing::info!("Daemon started successfully");
                        app.daemon_running = true;
                        app.database_view_state
                            .add_daemon_log("Daemon started.".to_string());
                        app.mark_dirty();
                    }
                    Err(e) => {
                        tracing::error!("Failed to start daemon: {:?}", e);
                        app.database_view_state
                            .add_daemon_log(format!("Failed to start: {}", e));
                        app.set_error(format!("Failed to start daemon: {e}"));
                    }
                }
            }
        }
        KeyCode::Char('s') => {
            // Sync database with remote
            tracing::info!("Syncing database with remote");
            app.database_view_state
                .add_sync_log("Starting sync...".to_string());
            app.database_view_state
                .start_operation("Syncing database".to_string());
            app.mark_dirty();

            match rt.block_on(client.sync_database()) {
                Ok(output) => {
                    tracing::info!("Database synced successfully: {}", output);
                    app.database_view_state
                        .add_sync_log(format!("Sync success: {output}"));
                    app.reload_issues();
                }
                Err(e) => {
                    tracing::error!("Failed to sync database: {:?}", e);
                    app.database_view_state
                        .add_sync_log(format!("Sync failed: {e}"));
                    app.set_error(format!("Failed to sync database: {e}"));
                }
            }
            app.database_view_state.finish_operation();
            app.mark_dirty();
        }
        KeyCode::Char('e') => {
            // Export issues to file
            app.database_view_state
                .start_input("Export Filename".to_string(), "beads_export.jsonl".to_string());
            app.mark_dirty();
        }
        KeyCode::Char('i') => {
            // Import issues from file
            app.database_view_state
                .start_input("Import Filename".to_string(), "beads_import.jsonl".to_string());
            app.mark_dirty();
        }
        KeyCode::Char('v') => {
            // Verify database integrity
            tracing::info!("Verifying database integrity");
            app.database_view_state
                .add_sync_log("Verifying integrity...".to_string());

            match rt.block_on(client.verify_database()) {
                Ok(output) => {
                    tracing::info!("Database verification result: {}", output);
                    app.database_view_state
                        .add_sync_log(format!("Integrity check: {output}"));
                    app.database_view_state.set_integrity_report(output.clone());
                    if output.contains("error") {
                        app.set_error(format!("Database check: {output}"));
                    } else if output.contains("issue") || output.contains("warning") {
                        app.set_warning(format!("Database check: {output}"));
                    } else {
                        app.set_success("Database verification completed successfully".to_string());
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to verify database: {:?}", e);
                    app.set_error(format!("Failed to verify database: {e}"));
                }
            }
        }
        KeyCode::Char('c') => {
            // Compact database (requires confirmation)
            tracing::info!("Compact database requested - showing confirmation dialog");

            // Set up confirmation dialog for compact operation
            app.dialog_state = Some(ui::widgets::DialogState::new());
            app.pending_action = Some("compact_database".to_string());
            app.mark_dirty();
        }
        KeyCode::Char('k') => {
            // Kill all (force)
            tracing::info!("Kill all beads processes requested");
            app.database_view_state
                .add_sync_log("Killing all beads processes...".to_string());
            // No direct client method for killall, could use shell
            app.set_warning("Killall not yet implemented via API".to_string());
        }
        _ => {}
    }
}

/// Handle keyboard events for the Molecular view
fn handle_molecular_view_event(key: KeyEvent, app: &mut models::AppState) {
    let key_code = key.code;

    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    // 1. Handle Wizard Events if active
    if let Some(ref mut wizard_state) = app.pour_wizard_state {
        match key_code {
            KeyCode::Enter => {
                if wizard_state.step == PourStep::Execution {
                    app.pour_wizard_state = None;
                } else {
                    wizard_state.next_step();
                    if wizard_state.step == PourStep::Execution {
                        // Trigger execution!
                        tracing::info!("Pouring formula: {}", wizard_state.formula.name);
                        // TODO: Implement actual issue creation via beads client
                        wizard_state.result_message = Some(format!(
                            "Success! Poured '{}' formula into database.",
                            wizard_state.formula.name
                        ));
                    }
                }
                app.mark_dirty();
            }
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::SHIFT) => {
                wizard_state.prev_step();
                app.mark_dirty();
            }
            KeyCode::Backspace => {
                wizard_state.form_state.delete_char();
                app.mark_dirty();
            }
            KeyCode::Esc => {
                app.pour_wizard_state = None;
                app.mark_dirty();
            }
            // Form input handling for Variables step
            KeyCode::Tab | KeyCode::Down => {
                wizard_state.form_state.focus_next();
                app.mark_dirty();
            }
            KeyCode::BackTab | KeyCode::Up => {
                wizard_state.form_state.focus_previous();
                app.mark_dirty();
            }
            KeyCode::Char(c) => {
                wizard_state.form_state.insert_char(c);
                app.mark_dirty();
            }
            KeyCode::Delete => {
                wizard_state.form_state.delete_char();
                app.mark_dirty();
            }
            _ => {}
        }
        return;
    }

    // 2. Handle Search input if searching in Browser
    if app.formula_browser_state.is_searching() {
        match key_code {
            KeyCode::Char(c) => {
                app.formula_browser_state.insert_char(c);
                app.mark_dirty();
            }
            KeyCode::Backspace => {
                app.formula_browser_state.backspace();
                app.mark_dirty();
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.formula_browser_state.set_searching(false);
                app.mark_dirty();
            }
            _ => {}
        }
        return;
    }

    // 3. Handle sub-tab navigation
    match key_code {
        KeyCode::Char('[') => {
            if app.selected_molecular_tab > 0 {
                app.selected_molecular_tab -= 1;
            } else {
                app.selected_molecular_tab = app.molecular_tabs.len() - 1;
            }
            app.mark_dirty();
            return;
        }
        KeyCode::Char(']') => {
            app.selected_molecular_tab =
                (app.selected_molecular_tab + 1) % app.molecular_tabs.len();
            app.mark_dirty();
            return;
        }
        _ => {}
    }

    // 4. Sub-tab specific handling
    match app.selected_molecular_tab {
        0 => {
            // Formula Browser
            let formulas_len = app.formulas.len();
            match key_code {
                KeyCode::Char('j') | KeyCode::Down => {
                    app.formula_browser_state.select_next(formulas_len);
                    app.mark_dirty();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.formula_browser_state.select_previous(formulas_len);
                    app.mark_dirty();
                }
                KeyCode::Char('/') => {
                    app.formula_browser_state.set_searching(true);
                    app.mark_dirty();
                }
                KeyCode::Enter => {
                    if let Some(idx) = app.formula_browser_state.selected() {
                        if let Some(formula) = app.formulas.get(idx) {
                            app.pour_wizard_state = Some(PourWizardState::new(formula.clone()));
                            app.mark_dirty();
                        }
                    }
                }
                KeyCode::Esc => {
                    app.formula_browser_state.clear_search();
                    app.mark_dirty();
                }
                _ => {}
            }
        }
        1 => {
            // Wisp Manager
            let wisps_len = app
                .issues_view_state
                .all_issues()
                .iter()
                .filter(|i| i.labels.contains(&"#wisp".to_string()))
                .count();
            match key_code {
                KeyCode::Char('j') | KeyCode::Down => {
                    app.wisp_manager_state.select_next(wisps_len);
                    app.mark_dirty();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.wisp_manager_state.select_previous(wisps_len);
                    app.mark_dirty();
                }
                KeyCode::Char('n') => {
                    // Create Wisp - Coming Soon (requires quick-entry dialog)
                    app.set_info("Create Wisp: Not yet implemented".to_string());
                }
                KeyCode::Char('d') => {
                    // Dissolve Wisp - Coming Soon
                    app.set_info("Dissolve Wisp: Not yet implemented".to_string());
                }
                _ => {}
            }
        }
        2 => {
            // Bonding Interface
            let issues_len = app.issues_view_state.all_issues().len();
            match key_code {
                KeyCode::Char('j') | KeyCode::Down => {
                    app.bonding_interface_state.select_next(issues_len);
                    app.mark_dirty();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.bonding_interface_state.select_previous(issues_len);
                    app.mark_dirty();
                }
                KeyCode::Tab => {
                    app.bonding_interface_state.toggle_focus();
                    app.mark_dirty();
                }
                KeyCode::Char('t') => {
                    // Cycle bond type
                    let next_type = match app.bonding_interface_state.bond_type {
                        BondType::Sequential => BondType::Parallel,
                        BondType::Parallel => BondType::Conditional,
                        BondType::Conditional => BondType::Sequential,
                    };
                    app.bonding_interface_state.bond_type = next_type;
                    app.mark_dirty();
                }
                KeyCode::Enter => {
                    // Create Bond
                    app.set_info("Create Bond: Not yet implemented".to_string());
                }
                _ => {}
            }
        }
        3 => {
            // Squash/Burn
            match key_code {
                KeyCode::Char('s') => {
                    app.set_info("Squash History: Not yet implemented".to_string());
                }
                KeyCode::Char('b') => {
                    app.set_info("Burn (Hard Delete): Not yet implemented".to_string());
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Handle keyboard events for the Help view
fn handle_help_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if app.notification.is_some() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

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
        // Check for notification auto-dismiss
        app.check_notification_timeout();

        // Only render if state has changed (dirty checking)
        if app.is_dirty() {
            let start = Instant::now();
            terminal.draw(|f| ui(f, app))?;
            let render_time = start.elapsed();
            app.perf_stats.record_render(render_time);
            app.clear_dirty();
        }

        // Poll for events with timeout to enable periodic notification checks
        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
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
                    0 => handle_issues_view_event(key, app),
                    1 => handle_dependencies_view_event(key.code, app),
                    2 => handle_labels_view_event(key.code, app),
                    6 => handle_molecular_view_event(key, app),
                    7 => handle_database_view_event(key.code, app),
                    8 => handle_help_view_event(key.code, app),
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

    // Title with daemon status
    let daemon_status = if app.daemon_running {
        Span::styled(
            " [Daemon: Running]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [Daemon: Stopped]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    };
    let title_line = Line::from(vec![
        Span::styled(
            format!("Beads-TUI v{}", env!("CARGO_PKG_VERSION")),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        daemon_status,
    ]);
    let title = Paragraph::new(title_line).block(Block::default().borders(Borders::ALL));
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

            // Add issue count for Issues tab (index 0)
            let label = if i == 0 {
                let filtered_count = app.issues_view_state.search_state().filtered_issues().len();
                let total_count = app.database_stats.total_issues;
                if filtered_count < total_count {
                    format!(" {} {} ({}/{}) ", i + 1, name, filtered_count, total_count)
                } else {
                    format!(" {} {} ({}) ", i + 1, name, total_count)
                }
            } else {
                format!(" {} {} ", i + 1, name)
            };

            ListItem::new(label).style(style)
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
            f.render_stateful_widget(
                dependencies_view,
                tabs_chunks[1],
                &mut app.dependencies_view_state,
            );
        }
        2 => {
            // Labels view
            let labels_view = LabelsView::new().labels(app.label_stats.clone());
            f.render_stateful_widget(labels_view, tabs_chunks[1], &mut app.labels_view_state);
        }
        3 => {
            // PERT Chart view
            PertView::render_with_state(tabs_chunks[1], f.buffer_mut(), &app.pert_view_state);
        }
        4 => {
            // Gantt Chart view
            GanttView::render_with_state(tabs_chunks[1], f.buffer_mut(), &app.gantt_view_state);
        }
        5 => {
            // Kanban Board view
            KanbanView::render_with_state(tabs_chunks[1], f.buffer_mut(), &app.kanban_view_state);
        }
        6 => {
            // Molecular Chemistry view
            if let Some(ref mut wizard_state) = app.pour_wizard_state {
                let wizard = PourWizard::new();
                f.render_stateful_widget(wizard, tabs_chunks[1], wizard_state);
            } else {
                // Render sub-tabs
                let mol_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(tabs_chunks[1]);

                let sub_tab_bar = TabBar::new(app.molecular_tabs.clone())
                    .selected(app.selected_molecular_tab)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Molecular Operations"),
                    );
                sub_tab_bar.render(mol_chunks[0], f.buffer_mut());

                match app.selected_molecular_tab {
                    0 => {
                        let formula_browser = FormulaBrowser::new().formulas(app.formulas.clone());
                        f.render_stateful_widget(
                            formula_browser,
                            mol_chunks[1],
                            &mut app.formula_browser_state,
                        );
                    }
                    1 => {
                        // Wisp Manager
                        let wisps: Vec<&crate::beads::models::Issue> = app
                            .issues_view_state
                            .all_issues()
                            .iter()
                            .filter(|i| i.labels.contains(&"#wisp".to_string()))
                            .collect();
                        let wisp_manager = WispManager::new(wisps);
                        f.render_stateful_widget(
                            wisp_manager,
                            mol_chunks[1],
                            &mut app.wisp_manager_state,
                        );
                    }
                    2 => {
                        // Bonding Interface
                        let all_issues: Vec<&crate::beads::models::Issue> =
                            app.issues_view_state.all_issues().iter().collect();
                        let bonding_interface = BondingInterface::new(all_issues);
                        f.render_stateful_widget(
                            bonding_interface,
                            mol_chunks[1],
                            &mut app.bonding_interface_state,
                        );
                    }
                    3 => {
                        // Squash/Burn
                        let history_ops = HistoryOps::new();
                        f.render_stateful_widget(
                            history_ops,
                            mol_chunks[1],
                            &mut app.history_ops_state,
                        );
                    }
                    _ => {}
                }
            }
        }
        7 => {
            // Database view
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(app.daemon_running);
            f.render_stateful_widget(database_view, tabs_chunks[1], &mut app.database_view_state);
        }
        _ => {
            // Help view (tab 8 and beyond)
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

    // Render dialog overlay if active
    if let Some(ref dialog_state) = app.dialog_state {
        if let Some(ref action) = app.pending_action {
            // Parse action to get issue ID and construct message
            if let Some(issue_id) = action.strip_prefix("delete:") {
                let message = format!("Are you sure you want to delete issue {issue_id}?");
                let dialog = ui::widgets::Dialog::confirm("Confirm Delete", &message);

                // Render dialog centered on screen
                let area = f.size();
                let dialog_area = centered_rect(60, 30, area);

                // Clear and render dialog
                f.render_widget(Clear, dialog_area);
                dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
            } else if action == "compact_database" {
                let message = "WARNING: Compacting will remove issue history.\nThis operation cannot be undone.\n\nContinue?";
                let dialog = ui::widgets::Dialog::confirm("Compact Database", message)
                    .dialog_type(ui::widgets::DialogType::Warning);

                // Render dialog centered on screen
                let area = f.size();
                let dialog_area = centered_rect(60, 30, area);

                // Clear and render dialog
                f.render_widget(Clear, dialog_area);
                dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
            }
        }
    }

    // Render notification banner if present
    if let Some(ref notification) = app.notification {
        let area = f.size();
        let notification_area = Rect {
            x: 0,
            y: 0,
            width: area.width,
            height: 3,
        };

        // Determine colors and icon based on notification type
        let (bg_color, border_color, icon) = match notification.notification_type {
            models::NotificationType::Error => (Color::Red, Color::Red, "✖"),
            models::NotificationType::Success => (Color::Green, Color::Green, "✓"),
            models::NotificationType::Info => (Color::Blue, Color::Blue, "ℹ"),
            models::NotificationType::Warning => (Color::Yellow, Color::Yellow, "⚠"),
        };

        let notification_text = Paragraph::new(format!(
            " {} {} (Press Esc to dismiss)",
            icon, notification.message
        ))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(bg_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default().borders(Borders::ALL).border_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        f.render_widget(Clear, notification_area);
        f.render_widget(notification_text, notification_area);
    }
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

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
