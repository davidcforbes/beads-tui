pub mod beads;
pub mod config;
pub mod models;
pub mod runtime;
pub mod ui;
pub mod utils;

use anyhow::Result;
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
fn handle_issues_view_event(key: KeyEvent, app: &mut models::AppState) {
    use ui::views::IssuesViewMode;

    let key_code = key.code;

    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    // Handle undo (Ctrl+Z)
    if key_code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
        runtime::RUNTIME.block_on(app.undo()).ok();
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

                            // Show loading indicator
                            app.start_loading(format!("Deleting issue {}...", issue_id));

                            // Create a tokio runtime to execute the async call
                            // Using global runtime instead of creating new runtime
                            let client = &app.beads_client;

                            match runtime::RUNTIME.block_on(client.delete_issue(issue_id)) {
                                Ok(()) => {
                                    tracing::info!("Successfully deleted issue: {}", issue_id);
                                    app.reload_issues();
                                    app.stop_loading();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to delete issue: {:?}", e);
                                }
                            }
                        } else if let Some(issue_id) = action.strip_prefix("close:") {
                            tracing::info!("Confirmed close for issue: {}", issue_id);

                            // Capture current status for undo before closing
                            let old_status = app
                                .issues_view_state
                                .search_state()
                                .selected_issue()
                                .map(|issue| issue.status.to_string());

                            // Show loading indicator
                            app.start_loading(format!("Closing issue {}...", issue_id));

                            let client = &app.beads_client;

                            match runtime::RUNTIME.block_on(client.close_issue(issue_id, None)) {
                                Ok(()) => {
                                    tracing::info!("Successfully closed issue: {}", issue_id);

                                    // Add undo entry if we captured the old status
                                    if let Some(status) = old_status {
                                        app.undo_history.push(crate::models::UndoEntry {
                                            description: format!("Closed issue {}", issue_id),
                                            reverse_command: vec![
                                                "update".to_string(),
                                                issue_id.to_string(),
                                                format!("--status={}", status),
                                            ],
                                            timestamp: std::time::SystemTime::now(),
                                        });
                                    }

                                    app.reload_issues();
                                    app.stop_loading();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to close issue: {:?}", e);
                                    app.set_error(format!("Failed to close issue: {e}\n\nTry:\n• Verify the issue exists with 'bd show {issue_id}'\n• Check network connectivity\n• Run 'bd doctor' to diagnose issues"));
                                }
                            }
                        } else if action == "compact_database" {
                            tracing::info!("Confirmed compact database");

                            // Show loading indicator
                            app.start_loading("Compacting database...");

                            // Create a tokio runtime to execute the async call
                            // Using global runtime instead of creating new runtime
                            let client = &app.beads_client;

                            match runtime::RUNTIME.block_on(client.compact_database()) {
                                Ok(()) => {
                                    tracing::info!("Successfully compacted database");
                                    app.reload_issues();
                                    app.stop_loading();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to compact database: {:?}", e);
                                    app.set_error(format!(
                                        "Failed to compact database: {e}\n\nTry:\n• Run 'bd doctor' to diagnose issues\n• Check available disk space\n• Restart the application"
                                    ));
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

    // Handle column manager events if active
    if let Some(ref mut cm_state) = app.column_manager_state {
        match key_code {
            KeyCode::Up => {
                cm_state.select_previous();
                return;
            }
            KeyCode::Down => {
                cm_state.select_next();
                return;
            }
            KeyCode::Char(' ') => {
                cm_state.toggle_visibility();
                return;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reset to defaults
                let defaults = crate::models::table_config::TableConfig::default().columns;
                cm_state.reset(defaults);
                return;
            }
            KeyCode::Enter => {
                // Apply changes
                if cm_state.is_modified() {
                    // Get modified columns
                    let new_columns = cm_state.columns().to_vec();

                    // Update table config with new columns
                    let mut table_config = app
                        .issues_view_state
                        .search_state()
                        .list_state()
                        .table_config()
                        .clone();
                    table_config.columns = new_columns;

                    // Apply to state
                    app.issues_view_state
                        .search_state_mut()
                        .list_state_mut()
                        .set_table_config(table_config);

                    // Save to disk
                    if let Err(e) = app.save_table_config() {
                        tracing::warn!("Failed to save table config: {}", e);
                        app.set_warning(format!("Column changes applied but not saved: {}", e));
                    } else {
                        app.set_success("Column configuration saved".to_string());
                    }
                }
                // Close column manager
                app.column_manager_state = None;
                return;
            }
            KeyCode::Esc => {
                // Cancel without applying
                app.column_manager_state = None;
                return;
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column up
                cm_state.move_up();
                return;
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column down
                cm_state.move_down();
                return;
            }
            _ => {
                // Ignore other keys when column manager is active
                return;
            }
        }
    }

    // Handle filter save dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.filter_save_dialog_state {
        match key_code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    dialog_state.focus_previous();
                } else {
                    dialog_state.focus_next();
                }
                return;
            }
            KeyCode::Left => {
                dialog_state.move_cursor_left();
                return;
            }
            KeyCode::Right => {
                dialog_state.move_cursor_right();
                return;
            }
            KeyCode::Backspace => {
                dialog_state.delete_char();
                return;
            }
            KeyCode::Char(c) => {
                dialog_state.insert_char(c);
                return;
            }
            KeyCode::Enter => {
                // Save or update the filter depending on mode
                if app.is_editing_filter() {
                    // Update existing filter
                    match app.save_edited_filter() {
                        Ok(()) => {
                            tracing::info!("Filter updated successfully");
                            app.set_success("Filter updated".to_string());
                        }
                        Err(e) => {
                            tracing::error!("Failed to update filter: {}", e);
                            app.set_error(e);
                        }
                    }
                } else {
                    // Save new filter
                    match app.save_current_filter() {
                        Ok(()) => {
                            tracing::info!("Filter saved successfully");
                            app.set_success("Filter saved".to_string());
                        }
                        Err(e) => {
                            tracing::error!("Failed to save filter: {}", e);
                            app.set_error(e);
                        }
                    }
                }
                return;
            }
            KeyCode::Esc => {
                // Cancel dialog
                tracing::debug!("Filter save dialog cancelled");
                app.hide_filter_save_dialog();
                return;
            }
            _ => {
                // Ignore other keys when dialog is active
                return;
            }
        }
    }

    // Handle dependency dialog events if dialog is open
    if app.dependency_dialog_state.is_open() {
        use ui::widgets::DependencyDialogFocus;

        match key_code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    app.dependency_dialog_state.focus_previous();
                } else {
                    app.dependency_dialog_state.focus_next();
                }
                app.mark_dirty();
                return;
            }
            KeyCode::Left => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_previous_button();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Right => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_next_button();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Up => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state
                        .autocomplete_state
                        .select_previous();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Down => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state.autocomplete_state.select_next();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char(' ') => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Type {
                    app.dependency_dialog_state.toggle_type();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Backspace => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state.autocomplete_state.delete_char();
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char(c) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state
                        .autocomplete_state
                        .insert_char(c);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Enter => {
                // Handle confirmation
                if app.dependency_dialog_state.is_ok_selected()
                    || app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId
                {
                    // Confirm selection and add dependency
                    if let Some(target_issue_id) = app.dependency_dialog_state.selected_issue_id() {
                        if let Some(current_issue) = app.issues_view_state.selected_issue() {
                            let current_id = current_issue.id.clone();
                            let dep_type = app.dependency_dialog_state.dependency_type();

                            match dep_type {
                                ui::widgets::DependencyType::RelatesTo => {
                                    // Bidirectional "see also" relationship - no cycle check needed
                                    // Using global runtime instead of creating new runtime
                                    app.start_loading("Linking issues...");

                                    match runtime::RUNTIME.block_on(
                                        app.beads_client
                                            .relate_issues(&current_id, &target_issue_id),
                                    ) {
                                        Ok(()) => {
                                            app.stop_loading();
                                            tracing::info!(
                                                "Created relates_to link: {} <-> {}",
                                                current_id,
                                                target_issue_id
                                            );
                                            app.set_success(format!(
                                                "Linked issues: {} <-> {}",
                                                current_id, target_issue_id
                                            ));
                                            app.reload_issues();
                                        }
                                        Err(e) => {
                                            app.stop_loading();
                                            tracing::error!(
                                                "Failed to create relates_to link: {}",
                                                e
                                            );
                                            app.set_error(format!("Failed to link issues: {}\n\nCommon causes:\n• One or both issue IDs are invalid\n• Issues not found - verify with 'bd list'\n• Network connectivity issues", e));
                                        }
                                    }
                                }
                                ui::widgets::DependencyType::DependsOn
                                | ui::widgets::DependencyType::Blocks => {
                                    // Blocking dependency - check for cycles
                                    let (from_id, to_id) = match dep_type {
                                        ui::widgets::DependencyType::DependsOn => {
                                            // Current depends on target (target blocks current)
                                            (current_id.clone(), target_issue_id.clone())
                                        }
                                        ui::widgets::DependencyType::Blocks => {
                                            // Current blocks target (target depends on current)
                                            (target_issue_id.clone(), current_id.clone())
                                        }
                                        _ => unreachable!(),
                                    };

                                    // Check if this would create a cycle
                                    let all_issues: Vec<beads::Issue> = app
                                        .issues_view_state
                                        .search_state()
                                        .filtered_issues()
                                        .to_vec();

                                    if models::PertGraph::would_create_cycle(
                                        &all_issues,
                                        &from_id,
                                        &to_id,
                                    ) {
                                        app.set_error(format!(
                                            "Cannot add dependency: would create a cycle. {} → {} would form a circular dependency.",
                                            from_id, to_id
                                        ));
                                        tracing::warn!(
                                            "Prevented cycle: {} depends on {} would create cycle",
                                            from_id,
                                            to_id
                                        );
                                    } else {
                                        // Call CLI to add dependency synchronously
                                        // Using global runtime instead of creating new runtime
                                        app.start_loading("Adding dependency...");

                                        match runtime::RUNTIME.block_on(
                                            app.beads_client.add_dependency(&from_id, &to_id),
                                        ) {
                                            Ok(()) => {
                                                app.stop_loading();
                                                tracing::info!(
                                                    "Added dependency: {} depends on {}",
                                                    from_id,
                                                    to_id
                                                );

                                                // Add undo entry
                                                app.undo_history.push(crate::models::UndoEntry {
                                                    description: format!(
                                                        "Added dependency: {} depends on {}",
                                                        from_id, to_id
                                                    ),
                                                    reverse_command: vec![
                                                        "dep".to_string(),
                                                        "remove".to_string(),
                                                        from_id.clone(),
                                                        to_id.clone(),
                                                    ],
                                                    timestamp: std::time::SystemTime::now(),
                                                });

                                                app.set_success(format!(
                                                    "Added dependency: {} depends on {}",
                                                    from_id, to_id
                                                ));
                                                // Reload issues to reflect the change
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                app.stop_loading();
                                                tracing::error!("Failed to add dependency: {}", e);
                                                app.set_error(format!(
                                                    "Failed to add dependency: {}\n\nCommon causes:\n• Dependency would create a cycle\n• Invalid issue ID format\n• Issue not found",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        app.set_info("Please select an issue first".to_string());
                    }
                }
                // Close dialog in either case
                app.dependency_dialog_state.close();
                app.mark_dirty();
                return;
            }
            KeyCode::Esc => {
                // Cancel dialog
                tracing::debug!("Dependency dialog cancelled");
                app.dependency_dialog_state.close();
                app.mark_dirty();
                return;
            }
            _ => {
                // Ignore other keys when dialog is active
                return;
            }
        }
    }

    // Handle delete confirmation dialog events if active
    if app.is_delete_confirmation_visible() {
        if let Some(ref mut dialog_state) = app.delete_dialog_state {
            match key_code {
                KeyCode::Left | KeyCode::Char('h') => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return;
                }
                KeyCode::Enter => {
                    // Confirm action based on selected button
                    let selected = dialog_state.selected_button();
                    if selected == 0 {
                        // Yes button - confirm deletion
                        match app.confirm_delete_filter() {
                            Ok(()) => {
                                tracing::info!("Filter deleted");
                                app.set_success("Filter deleted".to_string());
                            }
                            Err(e) => {
                                tracing::error!("Failed to delete filter: {}", e);
                                app.set_error(e);
                            }
                        }
                    } else {
                        // No button - cancel
                        tracing::debug!("Delete confirmation cancelled");
                        app.cancel_delete_filter();
                    }
                    return;
                }
                KeyCode::Esc => {
                    // Cancel deletion
                    tracing::debug!("Delete confirmation cancelled");
                    app.cancel_delete_filter();
                    return;
                }
                _ => {
                    // Ignore other keys when dialog is active
                    return;
                }
            }
        }
    }

    // Handle dependency removal confirmation dialog events if active
    if app.is_dependency_removal_confirmation_visible() {
        if let Some(ref mut dialog_state) = app.dependency_removal_dialog_state {
            match key_code {
                KeyCode::Left | KeyCode::Char('h') => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return;
                }
                KeyCode::Enter => {
                    // Confirm action based on selected button
                    let selected = dialog_state.selected_button();
                    if selected == 0 {
                        // Yes button - confirm removal
                        match app.confirm_remove_dependency() {
                            Ok(()) => {
                                tracing::info!("Dependency removed");
                                app.set_success("Dependency removed successfully".to_string());
                            }
                            Err(e) => {
                                tracing::error!("Failed to remove dependency: {}", e);
                                app.set_error(e);
                            }
                        }
                    } else {
                        // No button - cancel
                        tracing::debug!("Dependency removal cancelled");
                        app.cancel_remove_dependency();
                    }
                    return;
                }
                KeyCode::Esc => {
                    // Cancel removal
                    tracing::debug!("Dependency removal cancelled");
                    app.cancel_remove_dependency();
                    return;
                }
                _ => {
                    // Ignore other keys when dialog is active
                    return;
                }
            }
        }
    }

    let issues_state = &mut app.issues_view_state;
    let view_mode = issues_state.view_mode();

    // Handle filter menu events if open
    if issues_state.search_state().is_filter_menu_open() {
        match key_code {
            KeyCode::Char('j') | KeyCode::Down => {
                issues_state.search_state_mut().filter_menu_next();
                return;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                issues_state.search_state_mut().filter_menu_previous();
                return;
            }
            KeyCode::Enter => {
                issues_state.search_state_mut().filter_menu_confirm();
                app.set_success("Filter applied".to_string());
                return;
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                // Delete filter
                if let Some(i) = issues_state.search_state().filter_menu_state().selected() {
                    issues_state.search_state_mut().delete_saved_filter(i);
                    // Sync to config
                    let filters = issues_state.search_state().saved_filters().to_vec();
                    let _ = issues_state; // Release borrow before using app
                    app.config.filters = filters;
                    let _ = app.config.save();
                    app.set_success("Filter deleted".to_string());
                }
                return;
            }
            KeyCode::Esc | KeyCode::Char('m') => {
                issues_state.search_state_mut().toggle_filter_menu();
                return;
            }
            _ => return, // Sink other keys
        }
    }

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
                                // Using global runtime instead of creating new runtime
                                let client = app.beads_client.clone();

                                app.start_loading("Updating title...");

                                match runtime::RUNTIME
                                    .block_on(client.update_issue(&issue_id, update))
                                {
                                    Ok(()) => {
                                        app.stop_loading();
                                        tracing::info!(
                                            "Successfully updated title for: {}",
                                            issue_id
                                        );
                                        app.reload_issues();
                                    }
                                    Err(e) => {
                                        app.stop_loading();
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
                    KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        reorder_child_issue(app, -1);
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        reorder_child_issue(app, 1);
                    }
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
                    KeyCode::Char('C') => {
                        // Open column manager
                        let current_columns = issues_state
                            .search_state()
                            .list_state()
                            .table_config()
                            .columns
                            .clone();
                        app.column_manager_state =
                            Some(crate::ui::widgets::ColumnManagerState::new(current_columns));
                        tracing::debug!("Opened column manager");
                    }
                    KeyCode::Char('x') => {
                        // Close selected issue with confirmation
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            let issue_title = issue.title.clone();
                            tracing::info!("Requesting confirmation to close issue: {}", issue_id);

                            // Show confirmation dialog
                            app.dialog_state = Some(ui::widgets::DialogState::new());
                            app.pending_action = Some(format!("close:{issue_id}"));

                            tracing::debug!("Showing close confirmation for: {}", issue_title);
                        }
                    }
                    KeyCode::Char('o') => {
                        // Reopen selected issue
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            let old_status = issue.status.to_string();
                            tracing::info!("Reopening issue: {}", issue_id);

                            // Create a tokio runtime to execute the async call
                            // Using global runtime instead of creating new runtime
                            let client = app.beads_client.clone();

                            app.start_loading("Reopening issue...");

                            match runtime::RUNTIME.block_on(client.reopen_issue(&issue_id)) {
                                Ok(()) => {
                                    app.stop_loading();
                                    tracing::info!("Successfully reopened issue: {}", issue_id);

                                    // Add undo entry
                                    app.undo_history.push(crate::models::UndoEntry {
                                        description: format!("Reopened issue {}", issue_id),
                                        reverse_command: vec![
                                            "update".to_string(),
                                            issue_id.to_string(),
                                            format!("--status={}", old_status),
                                        ],
                                        timestamp: std::time::SystemTime::now(),
                                    });

                                    // Reload issues list
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    app.stop_loading();
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
                    KeyCode::Char('>') => {
                        // Indent: Make selected issue a child of the previous issue
                        if let Some(selected_issue) = issues_state.search_state().selected_issue() {
                            let selected_id = selected_issue.id.clone();
                            let selected_idx = issues_state.search_state().list_state().selected();

                            // Get the previous issue in the filtered list
                            if let Some(idx) = selected_idx {
                                if idx > 0 {
                                    let filtered_issues =
                                        issues_state.search_state().filtered_issues();
                                    if let Some(prev_issue) = filtered_issues.get(idx - 1) {
                                        let prev_id = prev_issue.id.clone();

                                        tracing::info!(
                                            "Indenting {} under {}",
                                            selected_id,
                                            prev_id
                                        );

                                        // Create a tokio runtime to execute the async call
                                        // Using global runtime instead of creating new runtime
                                        let client = app.beads_client.clone();

                                        app.start_loading("Indenting issue...");

                                        // Add dependency: selected depends on previous (previous blocks selected)
                                        match runtime::RUNTIME
                                            .block_on(client.add_dependency(&selected_id, &prev_id))
                                        {
                                            Ok(()) => {
                                                app.stop_loading();
                                                tracing::info!(
                                                    "Successfully indented {} under {}",
                                                    selected_id,
                                                    prev_id
                                                );
                                                app.set_success(format!(
                                                    "Indented issue under {}",
                                                    prev_id
                                                ));

                                                // Reload issues list to reflect the hierarchy change
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                app.stop_loading();
                                                tracing::error!("Failed to indent issue: {:?}", e);
                                                app.set_error(format!("Failed to indent: {e}\n\nTip: Ensure the previous issue is valid and can be a parent.\nCheck 'bd show' to verify issue hierarchy."));
                                            }
                                        }
                                    } else {
                                        app.set_error(
                                            "No previous issue to indent under".to_string(),
                                        );
                                    }
                                } else {
                                    app.set_error("Cannot indent first issue".to_string());
                                }
                            }
                        }
                    }
                    KeyCode::Char('<') => {
                        // Outdent: Remove selected issue from its parent
                        if let Some(selected_issue) = issues_state.search_state().selected_issue() {
                            let selected_id = selected_issue.id.clone();

                            // Find the parent (issue that blocks the selected issue)
                            let all_issues = issues_state.all_issues();
                            let parent_id = all_issues
                                .iter()
                                .find(|issue| issue.blocks.contains(&selected_id))
                                .map(|issue| issue.id.clone());

                            if let Some(parent_id) = parent_id {
                                tracing::info!(
                                    "Outdenting {} from parent {}",
                                    selected_id,
                                    parent_id
                                );

                                // Create a tokio runtime to execute the async call
                                // Using global runtime instead of creating new runtime
                                let client = app.beads_client.clone();

                                app.start_loading("Outdenting issue...");

                                // Remove dependency: selected no longer depends on parent
                                match runtime::RUNTIME
                                    .block_on(client.remove_dependency(&selected_id, &parent_id))
                                {
                                    Ok(()) => {
                                        app.stop_loading();
                                        tracing::info!(
                                            "Successfully outdented {} from {}",
                                            selected_id,
                                            parent_id
                                        );
                                        app.set_success(format!(
                                            "Outdented issue from {}",
                                            parent_id
                                        ));

                                        // Reload issues list to reflect the hierarchy change
                                        app.reload_issues();
                                    }
                                    Err(e) => {
                                        app.stop_loading();
                                        tracing::error!("Failed to outdent issue: {:?}", e);
                                        app.set_error(format!("Failed to outdent: {e}\n\nTip: Check the issue hierarchy with 'bd show'.\nEnsure dependencies allow this change."));
                                    }
                                }
                            } else {
                                app.set_error("Issue has no parent to outdent from".to_string());
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
                    KeyCode::Char('v') => {
                        // Cycle view
                        issues_state.search_state_mut().next_view();
                        tracing::debug!(
                            "Cycled to next view: {:?}",
                            issues_state.search_state().current_view()
                        );
                    }
                    KeyCode::Char('s') => {
                        // Cycle search scope
                        issues_state.search_state_mut().next_search_scope();
                        tracing::debug!(
                            "Cycled to next scope: {:?}",
                            issues_state.search_state().search_scope()
                        );
                    }
                    KeyCode::Char('g') => {
                        // Toggle regex
                        issues_state.search_state_mut().toggle_regex();
                        let enabled = issues_state.search_state().is_regex_enabled();
                        app.set_info(format!(
                            "Regex search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    KeyCode::Char('z') => {
                        // Toggle fuzzy
                        issues_state.search_state_mut().toggle_fuzzy();
                        let enabled = issues_state.search_state().is_fuzzy_enabled();
                        app.set_info(format!(
                            "Fuzzy search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    KeyCode::Char('l') => {
                        // Toggle label logic
                        issues_state.search_state_mut().toggle_label_logic();
                        let logic = issues_state.search_state().label_logic();
                        app.set_info(format!("Label logic set to {:?}", logic));
                    }
                    KeyCode::Char('m') => {
                        // Toggle filter quick select menu
                        issues_state.search_state_mut().toggle_filter_menu();
                    }
                    KeyCode::Char('p') => {
                        // Open priority selector for selected issue
                        if issues_state.selected_issue().is_some() {
                            app.priority_selector_state.toggle();
                        }
                    }
                    KeyCode::Char('L') => {
                        // Open label picker for selected issue (Shift+L)
                        if let Some(issue) = issues_state.selected_issue() {
                            // Clone current issue's labels first (to release borrow)
                            let current_labels = issue.labels.clone();

                            // Collect all unique labels from all issues
                            let all_labels: std::collections::HashSet<String> = app
                                .issues_view_state
                                .all_issues()
                                .iter()
                                .flat_map(|i| i.labels.iter().cloned())
                                .collect();
                            let mut available_labels: Vec<String> =
                                all_labels.into_iter().collect();
                            available_labels.sort();

                            // Set available labels and current issue's labels as selected
                            app.label_picker_state
                                .set_available_labels(available_labels);
                            app.label_picker_state.set_selected_labels(current_labels);

                            // Open picker
                            app.show_label_picker = true;
                        }
                    }
                    KeyCode::Char('S') => {
                        // Open status selector for selected issue (Shift+S)
                        if issues_state.selected_issue().is_some() {
                            app.status_selector_state.toggle();
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
                        let _ = issues_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
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
                        let _ = issues_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
                        tracing::debug!("Growing focused column");
                    }
                    KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Left: Move focused column left
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_left();
                        let _ = issues_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
                        tracing::debug!("Moving focused column left");
                    }
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Right: Move focused column right
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .move_focused_column_right();
                        let _ = issues_state; // Release borrow
                        if let Err(e) = app.save_table_config() {
                            tracing::warn!("Failed to save table config: {}", e);
                        }
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
                                if !editor_state.is_dirty() {
                                    tracing::info!("No changes detected, returning to list");
                                    issues_state.return_to_list();
                                } else {
                                    // Get change summary for logging
                                    let changes = editor_state.get_changes();
                                    tracing::info!("Changes detected: {:?}", changes);

                                    // Get IssueUpdate with only changed fields
                                    if let Some(update) = editor_state.get_issue_update() {
                                        let issue_id = editor_state.issue_id().to_string();

                                        // Mark as saved and return to list before reloading
                                        editor_state.save();
                                        issues_state.return_to_list();

                                        // Create a tokio runtime to execute the async call
                                        // Using global runtime instead of creating new runtime
                                        let client = app.beads_client.clone();

                                        app.start_loading("Updating issue...");

                                        match runtime::RUNTIME
                                            .block_on(client.update_issue(&issue_id, update))
                                        {
                                            Ok(()) => {
                                                app.stop_loading();
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
                                                app.stop_loading();
                                                tracing::error!("Failed to update issue: {:?}", e);
                                                app.set_error(format!(
                                                "Failed to update issue: {e}\n\nStay in edit mode to fix and retry.\nVerify your changes and try again."
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
                // Check for Ctrl+P first (toggle preview)
                if key_code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    create_form_state.toggle_preview();
                    return;
                }

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
                                    // Using global runtime instead of creating new runtime
                                    let client = app.beads_client.clone();

                                    let mut dependency_targets: Vec<String> = Vec::new();
                                    if let Some(parent) = data.parent.clone() {
                                        if !dependency_targets.contains(&parent) {
                                            dependency_targets.push(parent);
                                        }
                                    }
                                    for dep in data.dependencies.clone() {
                                        if !dependency_targets.contains(&dep) {
                                            dependency_targets.push(dep);
                                        }
                                    }

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

                                    // Show loading indicator
                                    app.start_loading("Creating issue...");

                                    match runtime::RUNTIME
                                        .block_on(client.create_issue_full(params))
                                    {
                                        Ok(issue_id) => {
                                            app.stop_loading();
                                            // Successfully created
                                            tracing::info!(
                                                "Successfully created issue: {}",
                                                issue_id
                                            );

                                            // Clear any previous errors
                                            app.clear_error();

                                            if !dependency_targets.is_empty() {
                                                let mut failures = Vec::new();
                                                for dep_id in &dependency_targets {
                                                    if dep_id == &issue_id {
                                                        continue;
                                                    }
                                                    if let Err(e) = runtime::RUNTIME.block_on(
                                                        client.add_dependency(&issue_id, dep_id),
                                                    ) {
                                                        failures.push(format!("{dep_id}: {e}"));
                                                    }
                                                }
                                                if !failures.is_empty() {
                                                    app.set_error(format!(
                                                    "Issue created, but dependencies failed: {}",
                                                    failures.join(", ")
                                                ));
                                                }
                                            }

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
                                            app.stop_loading();
                                            tracing::error!("Failed to create issue: {:?}", e);
                                            app.set_error(format!(
                                            "Failed to create issue: {e}\n\nStay in create mode to fix and retry.\nCheck that all required fields are filled correctly."
                                        ));
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
    if !app.notifications.is_empty() && key_code == KeyCode::Esc {
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
            // Add dependency - open dialog
            if let Some(current_issue) = selected_issue {
                let current_id = current_issue.id.clone();

                // Get all issue IDs except the current one
                let all_issues: Vec<String> = app
                    .issues_view_state
                    .search_state()
                    .filtered_issues()
                    .iter()
                    .map(|issue| issue.id.clone())
                    .filter(|id| id != &current_id)
                    .collect();

                app.dependency_dialog_state.open(all_issues);
                app.mark_dirty();
                tracing::info!("Add dependency dialog opened for issue: {}", current_id);
            } else {
                app.set_info("No issue selected".to_string());
            }
        }
        KeyCode::Char('d') => {
            // Remove dependency (with confirmation)
            if let Some(issue) = selected_issue {
                let current_id = issue.id.clone();
                match app.dependencies_view_state.focus() {
                    ui::views::DependencyFocus::Dependencies => {
                        if let Some(selected_idx) =
                            app.dependencies_view_state.selected_dependency()
                        {
                            if selected_idx < issue.dependencies.len() {
                                let dep_id = issue.dependencies[selected_idx].clone();
                                // Show confirmation dialog before removing
                                app.show_dependency_removal_confirmation(&current_id, &dep_id);
                            }
                        }
                    }
                    ui::views::DependencyFocus::Blocks => {
                        if let Some(selected_idx) = app.dependencies_view_state.selected_block() {
                            if selected_idx < issue.blocks.len() {
                                let blocked_id = issue.blocks[selected_idx].clone();
                                // Show confirmation dialog before removing
                                app.show_dependency_removal_confirmation(&blocked_id, &current_id);
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
    if !app.notifications.is_empty() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    let filtered_labels = app.labels_view_state.filtered_labels(&app.label_stats);
    let labels_len = filtered_labels.len();

    if app.labels_view_state.is_searching() {
        match key_code {
            KeyCode::Esc => {
                app.labels_view_state.stop_search();
                app.labels_view_state.clear_search();
            }
            KeyCode::Enter => {
                app.labels_view_state.stop_search();
            }
            KeyCode::Backspace => {
                app.labels_view_state.delete_search_char();
            }
            KeyCode::Char(c) => {
                app.labels_view_state.insert_search_char(c);
            }
            _ => {}
        }
        app.mark_dirty();
        return;
    }

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
                if let Some(label_stat) = filtered_labels.get(selected_idx) {
                    let label_name = label_stat.name.clone();
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
                if let Some(label_stat) = filtered_labels.get(selected_idx) {
                    let label_name = label_stat.name.clone();
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
            app.labels_view_state.start_search();
            tracing::info!("Search labels started");
        }
        KeyCode::Esc => {
            if !app.labels_view_state.search_query().is_empty() {
                app.labels_view_state.clear_search();
            }
        }
        _ => {}
    }
}

/// Handle keyboard events for the Database view
fn handle_database_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && key_code == KeyCode::Esc {
        app.clear_notification();
        return;
    }

    // Using global runtime instead of creating new runtime
    let client = app.beads_client.clone();

    match key_code {
        KeyCode::Char('r') => {
            // Refresh database status
            tracing::info!("Refreshing database status");
            app.start_loading("Refreshing database...");
            app.reload_issues();
            app.stop_loading();
        }
        KeyCode::Char('d') => {
            // Toggle daemon (start/stop)
            tracing::info!("Toggling daemon");

            if app.daemon_running {
                // Stop daemon
                match runtime::RUNTIME.block_on(client.stop_daemon()) {
                    Ok(_) => {
                        tracing::info!("Daemon stopped successfully");
                        app.daemon_running = false;
                        app.mark_dirty();
                    }
                    Err(e) => {
                        tracing::error!("Failed to stop daemon: {:?}", e);
                        app.set_error(format!(
                            "Failed to stop daemon: {e}\n\nThe daemon may not be running or may be unresponsive.\nCheck 'bd doctor' for status."
                        ));
                    }
                }
            } else {
                // Start daemon
                match runtime::RUNTIME.block_on(client.start_daemon()) {
                    Ok(_) => {
                        tracing::info!("Daemon started successfully");
                        app.daemon_running = true;
                        app.mark_dirty();
                    }
                    Err(e) => {
                        tracing::error!("Failed to start daemon: {:?}", e);
                        app.set_error(format!(
                            "Failed to start daemon: {e}\n\nTry:\n• Check if daemon is already running\n• Verify beads is installed correctly\n• Run 'bd doctor' for diagnostics"
                        ));
                    }
                }
            }
        }
        KeyCode::Char('s') => {
            // Sync database with remote
            tracing::info!("Syncing database with remote");

            app.start_loading("Syncing database...");

            match runtime::RUNTIME.block_on(app.beads_client.sync_database()) {
                Ok(output) => {
                    app.stop_loading();
                    tracing::info!("Database synced successfully: {}", output);
                    app.reload_issues();
                }
                Err(e) => {
                    app.stop_loading();
                    tracing::error!("Failed to sync database: {:?}", e);
                    app.set_error(format!(
                        "Failed to sync database: {e}\n\nTry:\n• Check network connectivity\n• Verify git remote is configured\n• Run 'bd doctor' to diagnose issues"
                    ));
                }
            }
        }
        KeyCode::Char('e') => {
            // Export issues to file
            tracing::info!("Exporting issues to beads_export.jsonl");

            app.start_loading("Exporting issues...");

            match runtime::RUNTIME.block_on(app.beads_client.export_issues("beads_export.jsonl")) {
                Ok(_) => {
                    app.stop_loading();
                    tracing::info!("Issues exported successfully");
                    app.set_success("Issues exported to beads_export.jsonl".to_string());
                }
                Err(e) => {
                    app.stop_loading();
                    tracing::error!("Failed to export issues: {:?}", e);
                    app.set_error(format!("Failed to export issues: {e}\n\nTry:\n• Check write permissions for the output file\n• Ensure sufficient disk space\n• Verify the file path is valid"));
                }
            }
        }
        KeyCode::Char('i') => {
            // Import issues from file
            tracing::info!("Importing issues from beads_import.jsonl");

            app.start_loading("Importing issues...");

            match runtime::RUNTIME.block_on(app.beads_client.import_issues("beads_import.jsonl")) {
                Ok(_) => {
                    app.stop_loading();
                    tracing::info!("Issues imported successfully");
                    app.reload_issues();
                }
                Err(e) => {
                    app.stop_loading();
                    tracing::error!("Failed to import issues: {:?}", e);
                    app.set_error(format!("Failed to import issues: {e}\n\nTry:\n• Verify the import file exists and is readable\n• Check the file format is valid JSON\n• Ensure the file contains properly formatted issue data"));
                }
            }
        }
        KeyCode::Char('v') => {
            // Verify database integrity
            tracing::info!("Verifying database integrity");

            app.start_loading("Verifying database...");

            match runtime::RUNTIME.block_on(app.beads_client.verify_database()) {
                Ok(output) => {
                    app.stop_loading();
                    tracing::info!("Database verification result: {}", output);
                    if output.contains("error") {
                        app.set_error(format!("Database check: {output}"));
                    } else if output.contains("issue") || output.contains("warning") {
                        app.set_warning(format!("Database check: {output}"));
                    } else {
                        app.set_success("Database verification completed successfully".to_string());
                    }
                }
                Err(e) => {
                    app.stop_loading();
                    tracing::error!("Failed to verify database: {:?}", e);
                    app.set_error(format!("Failed to verify database: {e}\n\nTry:\n• Run 'bd doctor' to diagnose issues\n• Check if beads CLI is accessible\n• Verify database integrity with 'bd stats'"));
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
        _ => {}
    }
}

/// Handle keyboard events for the Help view
fn handle_help_view_event(key_code: KeyCode, app: &mut models::AppState) {
    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && key_code == KeyCode::Esc {
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

fn reorder_child_issue(app: &mut models::AppState, direction: i32) {
    use crate::beads::models::Issue;
    let selected_issue = match app.issues_view_state.search_state().selected_issue() {
        Some(issue) => Issue::clone(issue),
        None => {
            app.set_info("No issue selected".to_string());
            return;
        }
    };

    let all_issues = app.issues_view_state.all_issues();
    let parent = all_issues
        .iter()
        .find(|issue| issue.blocks.contains(&selected_issue.id));

    let parent = match parent {
        Some(issue) => issue,
        None => {
            app.set_info("Selected issue has no parent".to_string());
            return;
        }
    };

    let mut new_order = parent.blocks.clone();
    let current_idx = match new_order.iter().position(|id| id == &selected_issue.id) {
        Some(idx) => idx,
        None => {
            app.set_error("Selected issue not found in parent blocks".to_string());
            return;
        }
    };

    if new_order.len() < 2 {
        app.set_info("Parent has only one child".to_string());
        return;
    }

    let target_idx = if direction < 0 {
        if current_idx == 0 {
            app.set_info("Already at the top".to_string());
            return;
        }
        current_idx - 1
    } else {
        if current_idx + 1 >= new_order.len() {
            app.set_info("Already at the bottom".to_string());
            return;
        }
        current_idx + 1
    };

    new_order.swap(current_idx, target_idx);

    let parent_id = parent.id.clone();
    let current_children = parent.blocks.clone();
    // Using global runtime instead of creating new runtime
    let client = &app.beads_client;

    let mut removed: Vec<String> = Vec::new();
    for child_id in &current_children {
        if let Err(e) = runtime::RUNTIME.block_on(client.remove_dependency(child_id, &parent_id)) {
            for restored_id in &removed {
                let _ = runtime::RUNTIME.block_on(client.add_dependency(restored_id, &parent_id));
            }
            app.set_error(format!("Failed to reorder children: {e}"));
            return;
        }
        removed.push(child_id.clone());
    }

    let mut added: Vec<String> = Vec::new();
    for child_id in &new_order {
        if let Err(e) = runtime::RUNTIME.block_on(client.add_dependency(child_id, &parent_id)) {
            for added_id in &added {
                let _ = runtime::RUNTIME.block_on(client.remove_dependency(added_id, &parent_id));
            }
            for restored_id in &current_children {
                let _ = runtime::RUNTIME.block_on(client.add_dependency(restored_id, &parent_id));
            }
            app.set_error(format!("Failed to reorder children: {e}"));
            return;
        }
        added.push(child_id.clone());
    }

    app.set_success(format!("Reordered children under {}", parent_id));
    app.reload_issues();

    let search_state = app.issues_view_state.search_state_mut();
    if let Some(idx) = search_state
        .filtered_issues()
        .iter()
        .position(|issue| issue.id == selected_issue.id)
    {
        search_state.list_state_mut().select(Some(idx));
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

                // Check for ESC during loading operations to request cancellation
                if key.code == KeyCode::Esc && app.is_loading() {
                    app.request_cancellation();
                    continue;
                }

                // Check for saved filter hotkeys (F1-F11)
                if let KeyCode::F(num) = key.code {
                    if (1..=11).contains(&num) {
                        // Map F-key to hotkey char: F1='1', F2='2', ..., F9='9', F10='A', F11='B'
                        let hotkey = if num <= 9 {
                            char::from_digit(num as u32, 10).unwrap()
                        } else if num == 10 {
                            'A'
                        } else {
                            'B'
                        };

                        // Try to load and apply the filter
                        if let Some(saved_filter) = app.config.get_filter_by_hotkey(hotkey).cloned()
                        {
                            // Apply filter to issues view (only if on Issues tab)
                            if app.selected_tab == 0 {
                                app.issues_view_state
                                    .search_state_mut()
                                    .apply_filter(&saved_filter.filter);
                                app.set_success(format!("Applied filter: {}", saved_filter.name));
                                app.mark_dirty();
                            }
                        }
                        continue;
                    }
                }

                // Check for filter save shortcut (Ctrl+S)
                if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    if app.selected_tab == 0 {
                        // Show filter save dialog on Issues tab
                        app.show_filter_save_dialog();
                        app.mark_dirty();
                    }
                    continue;
                }

                // Check for keyboard shortcut help toggle ('?')
                if key.code == KeyCode::Char('?') && key.modifiers.is_empty() {
                    if app.is_shortcut_help_visible() {
                        app.hide_shortcut_help();
                    } else {
                        app.show_shortcut_help();
                    }
                    continue;
                }

                // Check for context-sensitive help (F1)
                if key.code == KeyCode::F(1) {
                    if app.is_context_help_visible() {
                        app.hide_context_help();
                    } else {
                        app.show_context_help();
                    }
                    continue;
                }

                // Handle Esc key for dismissing overlays
                if key.code == KeyCode::Esc {
                    if app.is_shortcut_help_visible() {
                        app.hide_shortcut_help();
                        continue;
                    }
                    if app.is_context_help_visible() {
                        app.hide_context_help();
                        continue;
                    }
                    if app.show_notification_history {
                        app.toggle_notification_history();
                        continue;
                    }
                    if app.show_issue_history {
                        app.show_issue_history = false;
                        continue;
                    }
                    // Fall through to other Esc handlers if shortcut help is not visible
                }

                // Handle Ctrl+H for toggling issue history panel (only in Issues tab with selected issue)
                if key.code == KeyCode::Char('h')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                    && app.selected_tab == 0
                {
                    // Only show history if an issue is selected
                    if app.issues_view_state.selected_issue().is_some() {
                        app.show_issue_history = !app.show_issue_history;
                        continue;
                    }
                }

                // Handle notification history panel events if visible
                if app.show_notification_history {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.notification_history_state.select_previous();
                            continue;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let len = app.notification_history.len();
                            app.notification_history_state.select_next(len);
                            continue;
                        }
                        _ => {}
                    }
                }

                // Handle issue history panel events if visible
                if app.show_issue_history {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.issue_history_state.select_previous();
                            continue;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            // Count history events for the selected issue
                            if let Some(issue) = app.issues_view_state.selected_issue() {
                                let len = 2
                                    + issue.notes.len()
                                    + if issue.updated != issue.created { 1 } else { 0 }
                                    + if issue.closed.is_some() { 1 } else { 0 };
                                app.issue_history_state.select_next(len);
                            }
                            continue;
                        }
                        _ => {}
                    }
                }

                // Handle priority selector events if open
                if app.priority_selector_state.is_open() {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.priority_selector_state.select_previous(5); // 5 priority levels
                            continue;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.priority_selector_state.select_next(5);
                            continue;
                        }
                        KeyCode::Enter => {
                            // Apply selected priority to current issue
                            if let Some(selected_idx) = app.priority_selector_state.selected() {
                                use crate::beads::client::IssueUpdate;
                                use crate::beads::models::Priority;
                                let priorities = [
                                    Priority::P0,
                                    Priority::P1,
                                    Priority::P2,
                                    Priority::P3,
                                    Priority::P4,
                                ];
                                if let Some(&new_priority) = priorities.get(selected_idx) {
                                    if let Some(issue) = app.issues_view_state.selected_issue() {
                                        let issue_id = issue.id.clone();

                                        // Update priority via beads client
                                        // Using global runtime instead of creating new runtime
                                        let client = app.beads_client.clone();
                                        let update = IssueUpdate::new().priority(new_priority);

                                        app.start_loading("Updating priority...");

                                        match runtime::RUNTIME
                                            .block_on(client.update_issue(&issue_id, update))
                                        {
                                            Ok(()) => {
                                                app.stop_loading();
                                                app.set_success(format!(
                                                    "Updated priority to {} for issue {}",
                                                    new_priority, issue_id
                                                ));
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                app.stop_loading();
                                                app.set_error(format!("Failed to update priority: {}\n\nTip: Verify the issue exists and you have permission to modify it.\nUse valid priority values: P0, P1, P2, P3, or P4", e));
                                            }
                                        }
                                    }
                                }
                            }
                            app.priority_selector_state.close();
                            continue;
                        }
                        KeyCode::Esc => {
                            app.priority_selector_state.close();
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                // Handle label picker events if open
                if app.show_label_picker {
                    match key.code {
                        KeyCode::Esc => {
                            // If filtering, stop filtering; otherwise close picker
                            if app.label_picker_state.is_filtering() {
                                app.label_picker_state.stop_filtering();
                            } else {
                                app.show_label_picker = false;
                            }
                            continue;
                        }
                        KeyCode::Char('/') if !app.label_picker_state.is_filtering() => {
                            app.label_picker_state.start_filtering();
                            continue;
                        }
                        KeyCode::Char(c) if app.label_picker_state.is_filtering() => {
                            app.label_picker_state.insert_char(c);
                            continue;
                        }
                        KeyCode::Backspace if app.label_picker_state.is_filtering() => {
                            app.label_picker_state.delete_char();
                            continue;
                        }
                        KeyCode::Up | KeyCode::Char('k')
                            if !app.label_picker_state.is_filtering() =>
                        {
                            app.label_picker_state.select_previous();
                            continue;
                        }
                        KeyCode::Down | KeyCode::Char('j')
                            if !app.label_picker_state.is_filtering() =>
                        {
                            app.label_picker_state.select_next();
                            continue;
                        }
                        KeyCode::Char(' ') if !app.label_picker_state.is_filtering() => {
                            app.label_picker_state.toggle_selected();
                            continue;
                        }
                        KeyCode::Enter => {
                            // Apply selected labels to current issue
                            if let Some(issue) = app.issues_view_state.selected_issue() {
                                let issue_id = issue.id.clone();
                                let new_labels = app.label_picker_state.selected_labels().to_vec();

                                // Update labels via beads client
                                use crate::beads::client::IssueUpdate;
                                // Using global runtime instead of creating new runtime
                                let client = app.beads_client.clone();
                                let update = IssueUpdate::new().labels(new_labels.clone());

                                app.start_loading("Updating labels...");

                                match runtime::RUNTIME
                                    .block_on(client.update_issue(&issue_id, update))
                                {
                                    Ok(()) => {
                                        app.stop_loading();
                                        app.set_success(format!(
                                            "Updated labels for issue {} ({})",
                                            issue_id,
                                            if new_labels.is_empty() {
                                                "removed all".to_string()
                                            } else {
                                                format!("{} labels", new_labels.len())
                                            }
                                        ));
                                        app.reload_issues();
                                    }
                                    Err(e) => {
                                        app.stop_loading();
                                        app.set_error(format!("Failed to update labels: {}\n\nTip: Verify the issue exists and label format is correct.\nLabels should be single words or hyphenated (e.g., 'bug', 'high-priority')", e));
                                    }
                                }
                            }
                            app.show_label_picker = false;
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                // Handle status selector events if open
                if app.status_selector_state.is_open() {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.status_selector_state.select_previous(3); // 3 status options: Open, InProgress, Closed
                            continue;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.status_selector_state.select_next(3);
                            continue;
                        }
                        KeyCode::Enter => {
                            // Apply selected status to current issue
                            if let Some(selected_idx) = app.status_selector_state.selected() {
                                use crate::beads::client::IssueUpdate;
                                use crate::beads::models::IssueStatus;
                                let statuses = [
                                    IssueStatus::Open,
                                    IssueStatus::InProgress,
                                    IssueStatus::Closed,
                                ];
                                if let Some(&new_status) = statuses.get(selected_idx) {
                                    if let Some(issue) = app.issues_view_state.selected_issue() {
                                        let issue_id = issue.id.clone();

                                        // Update status via beads client
                                        // Using global runtime instead of creating new runtime
                                        let client = app.beads_client.clone();
                                        let update = IssueUpdate::new().status(new_status);

                                        app.start_loading("Updating status...");

                                        match runtime::RUNTIME
                                            .block_on(client.update_issue(&issue_id, update))
                                        {
                                            Ok(()) => {
                                                app.stop_loading();
                                                app.set_success(format!(
                                                    "Updated status to {} for issue {}",
                                                    new_status, issue_id
                                                ));
                                                app.reload_issues();
                                            }
                                            Err(e) => {
                                                app.stop_loading();
                                                app.set_error(format!("Failed to update status: {}\n\nTip: Valid statuses are: open, in_progress, blocked, closed.\nVerify the issue exists with 'bd show <issue-id>'", e));
                                            }
                                        }
                                    }
                                }
                            }
                            app.status_selector_state.close();
                            continue;
                        }
                        KeyCode::Esc => {
                            app.status_selector_state.close();
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                // Global key bindings
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                        continue;
                    }
                    KeyCode::Char('N') => {
                        app.toggle_notification_history();
                        continue;
                    }
                    KeyCode::F(1) => {
                        // Toggle context-sensitive help
                        if app.is_context_help_visible() {
                            app.hide_context_help();
                        } else {
                            app.show_context_help();
                        }
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
            "Beads-TUI v0.1.0",
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
            // Database view
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(app.daemon_running);
            f.render_stateful_widget(database_view, tabs_chunks[1], &mut app.database_view_state);
        }
        _ => {
            // Help view (tab 4 and beyond)
            let help_view = HelpView::new().selected_section(app.help_section);
            f.render_widget(help_view, tabs_chunks[1]);
        }
    }

    // Status bar with optional performance stats, loading indicator, or action hints
    let status_text = if let Some(ref spinner) = app.loading_spinner {
        // Show loading indicator using Spinner widget
        let label = app
            .loading_message
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Loading...");
        let spinner_text = format!("{} {}", spinner.frame_char(), label);
        Paragraph::new(spinner_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Loading"))
    } else if app.show_perf_stats {
        let perf_info = app.perf_stats.format_stats();
        let mut lines: Vec<Line> = perf_info
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        // Add context-sensitive action hints at the end
        lines.push(Line::from(""));
        lines.push(Line::from(get_action_hints(app)));
        Paragraph::new(lines)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Performance"))
    } else {
        // Show context-sensitive action hints
        Paragraph::new(get_action_hints(app))
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Actions"))
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
            } else if let Some(issue_id) = action.strip_prefix("close:") {
                let message = format!("Are you sure you want to close issue {issue_id}?\nThis will mark the issue as resolved.");
                let dialog = ui::widgets::Dialog::confirm("Confirm Close", &message);

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

    // Render filter save dialog overlay if active
    if let Some(ref dialog_state) = app.filter_save_dialog_state {
        use ui::widgets::FilterSaveDialog;

        let dialog = FilterSaveDialog::new();

        // Render dialog centered on screen
        let area = f.size();
        let dialog_area = centered_rect(70, 40, area);

        // Clear and render dialog
        f.render_widget(Clear, dialog_area);
        dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
    }

    // Render filter quick-select menu overlay if active
    if let Some(ref mut quick_select_state) = app.filter_quick_select_state {
        use ui::widgets::FilterQuickSelectMenu;

        let menu = FilterQuickSelectMenu::new();

        // Render menu centered on screen
        let area = f.size();
        let menu_area = centered_rect(80, 60, area);

        // Clear and render menu
        f.render_widget(Clear, menu_area);
        menu.render_with_state(menu_area, f.buffer_mut(), quick_select_state);
    }

    // Render delete filter confirmation dialog overlay if active
    if let Some(ref filter_name) = app.delete_confirmation_filter {
        if let Some(ref dialog_state) = app.delete_dialog_state {
            let message = format!(
                "Are you sure you want to delete the filter '{}'?\n\nThis action cannot be undone.",
                filter_name
            );
            let dialog = ui::widgets::Dialog::confirm("Delete Filter", &message)
                .dialog_type(ui::widgets::DialogType::Warning);

            // Render dialog centered on screen
            let area = f.size();
            let dialog_area = centered_rect(60, 30, area);

            // Clear and render dialog
            f.render_widget(Clear, dialog_area);
            dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
        }
    }

    // Render dependency removal confirmation dialog overlay if active
    if let Some((issue_id, depends_on_id)) = &app.pending_dependency_removal {
        if let Some(ref dialog_state) = app.dependency_removal_dialog_state {
            let message = format!(
                "Are you sure you want to remove this dependency?\n\n{} will no longer depend on {}\n\nThis action cannot be undone.",
                issue_id, depends_on_id
            );
            let dialog = ui::widgets::Dialog::confirm("Remove Dependency", &message)
                .dialog_type(ui::widgets::DialogType::Warning);

            // Render dialog centered on screen
            let area = f.size();
            let dialog_area = centered_rect(60, 30, area);

            // Clear and render dialog
            f.render_widget(Clear, dialog_area);
            dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
        }
    }

    // Render dependency dialog overlay if open
    if app.dependency_dialog_state.is_open() {
        use ui::widgets::DependencyDialog;

        // Get current issue title for dialog
        let current_issue_title = app
            .issues_view_state
            .selected_issue()
            .map(|issue| issue.title.as_str())
            .unwrap_or("Unknown Issue");

        let dialog = DependencyDialog::new(current_issue_title);

        // Render dialog overlay
        f.render_stateful_widget(dialog, f.size(), &mut app.dependency_dialog_state);
    }

    // Render toast notifications if present
    if !app.notifications.is_empty() {
        let toast_stack = ui::widgets::ToastStack::new(&app.notifications);
        f.render_widget(toast_stack, f.size());
    }

    // Render notification history panel if visible
    if app.show_notification_history {
        use ui::widgets::NotificationHistoryPanel;

        // Convert VecDeque to Vec for panel rendering
        let notifications_vec: Vec<_> = app.notification_history.iter().cloned().collect();
        let panel = NotificationHistoryPanel::new(&notifications_vec);
        f.render_stateful_widget(panel, f.size(), &mut app.notification_history_state);
    }

    // Render issue history panel if visible
    if app.show_issue_history {
        use ui::widgets::IssueHistoryPanel;

        let selected_issue = app.issues_view_state.selected_issue();
        let panel = IssueHistoryPanel::new(selected_issue);
        f.render_stateful_widget(panel, f.size(), &mut app.issue_history_state);
    }

    // Render priority selector if open
    if app.priority_selector_state.is_open() {
        use ratatui::widgets::Clear;
        use ui::widgets::PrioritySelector;

        // Get current priority of selected issue
        let current_priority = app
            .issues_view_state
            .selected_issue()
            .map(|issue| issue.priority)
            .unwrap_or(beads::models::Priority::P2);

        // Create a centered rect for the selector
        let area = f.size();
        let selector_area = centered_rect(40, 30, area);

        // Clear and render selector
        f.render_widget(Clear, selector_area);
        let selector = PrioritySelector::new(current_priority);
        f.render_stateful_widget(selector, selector_area, &mut app.priority_selector_state);
    }

    // Render label picker if open
    if app.show_label_picker {
        use ratatui::widgets::Clear;
        use ui::widgets::LabelPicker;

        // Create a centered rect for the picker
        let area = f.size();
        let picker_area = centered_rect(60, 70, area);

        // Clear and render picker
        f.render_widget(Clear, picker_area);
        let picker = LabelPicker::new();
        f.render_stateful_widget(picker, picker_area, &mut app.label_picker_state);
    }

    // Render status selector if open
    if app.status_selector_state.is_open() {
        use ratatui::widgets::Clear;
        use ui::widgets::StatusSelector;

        // Get current status of selected issue
        let current_status = app
            .issues_view_state
            .selected_issue()
            .map(|issue| issue.status)
            .unwrap_or(beads::models::IssueStatus::Open);

        // Create a centered rect for the selector
        let area = f.size();
        let selector_area = centered_rect(40, 30, area);

        // Clear and render selector
        f.render_widget(Clear, selector_area);
        let selector = StatusSelector::new(current_status);
        f.render_stateful_widget(selector, selector_area, &mut app.status_selector_state);
    }

    // Render column manager if open
    if let Some(ref mut cm_state) = app.column_manager_state {
        use ratatui::widgets::Clear;
        use ui::widgets::ColumnManager;

        // Create a centered rect for the column manager
        let area = f.size();
        let cm_area = centered_rect(60, 70, area);

        // Clear and render column manager
        f.render_widget(Clear, cm_area);
        let column_manager = ColumnManager::new();
        f.render_stateful_widget(column_manager, cm_area, cm_state);
    }

    // Render keyboard shortcut help overlay if visible
    if app.is_shortcut_help_visible() {
        use ui::widgets::{HelpOverlay, HelpOverlayPosition};

        let help = HelpOverlay::new("Keyboard Shortcuts")
            .subtitle("Press ? or Esc to close")
            .position(HelpOverlayPosition::Center)
            .width_percent(60)
            .height_percent(70)
            // Global shortcuts
            .key_binding("?", "Toggle this help")
            .key_binding("q", "Quit application")
            .key_binding("Esc", "Dismiss overlays/dialogs")
            .key_binding("Tab", "Next tab")
            .key_binding("Shift+Tab", "Previous tab")
            .key_binding("1-9", "Switch to tab by number")
            .key_binding("Shift+N", "Notification history")
            .key_binding("Ctrl+P / F12", "Toggle performance stats")
            // Issues view shortcuts
            .key_binding("↑/↓ or j/k", "Navigate issues")
            .key_binding("Enter", "View/edit issue")
            .key_binding("c", "Create new issue")
            .key_binding("e", "Edit selected issue")
            .key_binding("d", "Delete selected issue")
            .key_binding("Space", "Select/deselect issue")
            .key_binding("a", "Select all issues")
            .key_binding("x", "Clear selection")
            .key_binding("/", "Search/filter issues")
            .key_binding("f", "Quick filter menu")
            .key_binding("Ctrl+S", "Save current filter")
            .key_binding("F1-F11", "Apply saved filter")
            // View shortcuts
            .key_binding("h", "Show full help")
            .key_binding("r", "Refresh data");

        f.render_widget(help, f.size());
    }

    // Render context-sensitive help overlay if visible
    if app.is_context_help_visible() {
        use ui::widgets::{HelpOverlay, HelpOverlayPosition};

        let (title, subtitle, bindings) = get_context_help_content(app);
        let subtitle_text = format!("{} | Press F1 or Esc to close", subtitle);

        let mut help = HelpOverlay::new(&title)
            .subtitle(&subtitle_text)
            .position(HelpOverlayPosition::Center)
            .width_percent(65)
            .height_percent(75);

        // Add all key bindings from the context
        for (key, description) in bindings {
            help = help.key_binding(key, description);
        }

        f.render_widget(help, f.size());
    }
}

/// Generate context-sensitive action hints based on current application state
fn get_action_hints(app: &models::AppState) -> String {
    // If dialog is visible, show dialog-specific hints
    if app.dialog_state.is_some() || app.delete_dialog_state.is_some() {
        return "←/→: Navigate | Enter: Confirm | Esc: Cancel".to_string();
    }

    // If filter save dialog is visible
    if app.is_filter_save_dialog_visible() {
        return "Type to edit | Tab: Next field | Enter: Save | Esc: Cancel".to_string();
    }

    // If filter quick-select is visible
    if app.issues_view_state.search_state().is_filter_menu_open() {
        return "↑/↓: Navigate | Enter: Apply filter | e: Edit | d: Delete | Esc: Cancel"
            .to_string();
    }

    // If dependency dialog is visible
    if app.dependency_dialog_state.is_open() {
        return "Tab: Next field | ↑/↓: Navigate | Space: Select | Enter: Add | Esc: Cancel"
            .to_string();
    }

    // If keyboard shortcut help is visible
    if app.is_shortcut_help_visible() {
        return "Esc or ?: Close help".to_string();
    }

    // Tab-specific action hints
    match app.selected_tab {
        0 => {
            // Issues view
            let mode = app.issues_view_state.view_mode();
            match mode {
                ui::views::IssuesViewMode::List => {
                    "↑/↓/j/k: Navigate | Enter: View | c: Create | e: Edit | d: Delete | /: Search | f: Filter | Space: Select | ?: Help".to_string()
                }
                ui::views::IssuesViewMode::Create => {
                    "Tab: Next field | Shift+Tab: Previous | Ctrl+S: Save | Esc: Cancel".to_string()
                }
                ui::views::IssuesViewMode::Edit => {
                    "Tab: Next field | Shift+Tab: Previous | Ctrl+S: Save | Esc: Cancel".to_string()
                }
                ui::views::IssuesViewMode::Detail => {
                    "e: Edit | d: Delete | Esc: Back to list".to_string()
                }
            }
        }
        1 => {
            // Dependencies view
            "↑/↓/j/k: Navigate | a: Add dependency | r: Remove | Enter: View | Esc: Back | ?: Help"
                .to_string()
        }
        2 => {
            // Labels view
            "↑/↓/j/k: Navigate | Enter: Select | a: Add label | d: Delete | Esc: Back | ?: Help"
                .to_string()
        }
        3 => {
            // PERT view
            "↑/↓: Navigate | +/-: Zoom | c: Configure | Esc: Back | ?: Help".to_string()
        }
        4 => {
            // Gantt view
            "↑/↓: Navigate | +/-: Zoom | g: Group by | c: Configure | Esc: Back | ?: Help"
                .to_string()
        }
        5 => {
            // Kanban view
            "↑/↓/←/→: Navigate | Space: Move card | c: Configure | Esc: Back | ?: Help".to_string()
        }
        6 => {
            // Molecular view
            "↑/↓: Navigate | Tab: Switch molecular tab | Enter: Select | Esc: Back | ?: Help"
                .to_string()
        }
        7 => {
            // Database view
            "↑/↓: Navigate | r: Refresh | c: Compact | v: Verify | Esc: Back | ?: Help".to_string()
        }
        8 => {
            // Help view
            "←/→/h/l: Navigate sections | Esc: Back | ?: Quick reference".to_string()
        }
        _ => "Press 'q' to quit | Tab/Shift+Tab: Switch tabs | 1-9: Direct tab access | ?: Help"
            .to_string(),
    }
}

/// Generate context-sensitive help content based on current application state
fn get_context_help_content(
    app: &models::AppState,
) -> (String, String, Vec<(&'static str, &'static str)>) {
    // If dialog is visible, show dialog-specific help
    if app.dialog_state.is_some() || app.delete_dialog_state.is_some() {
        return (
            "Dialog Help".to_string(),
            "Confirmation Dialog".to_string(),
            vec![
                ("←/→", "Navigate between buttons"),
                ("Enter", "Confirm current selection"),
                ("Esc", "Cancel and close dialog"),
                ("Tab", "Move to next button"),
            ],
        );
    }

    // If filter save dialog is visible
    if app.is_filter_save_dialog_visible() {
        return (
            "Filter Save Dialog Help".to_string(),
            "Save or Edit Filter".to_string(),
            vec![
                ("Type", "Enter filter name and hotkey"),
                ("Tab", "Move to next field"),
                ("Shift+Tab", "Move to previous field"),
                ("Enter", "Save filter"),
                ("Esc", "Cancel"),
                ("F1-F11", "Available hotkey options"),
            ],
        );
    }

    // If filter quick-select is visible
    if app.issues_view_state.search_state().is_filter_menu_open() {
        return (
            "Quick Filter Menu Help".to_string(),
            "Apply, Edit, or Delete Filters".to_string(),
            vec![
                ("↑/↓", "Navigate through filters"),
                ("Enter", "Apply selected filter"),
                ("e", "Edit filter"),
                ("d", "Delete filter"),
                ("Esc", "Close menu"),
            ],
        );
    }

    // If dependency dialog is visible
    if app.dependency_dialog_state.is_open() {
        return (
            "Dependency Dialog Help".to_string(),
            "Manage Issue Dependencies".to_string(),
            vec![
                ("Tab", "Move to next field"),
                ("↑/↓", "Navigate through issues"),
                ("Space", "Select/deselect issue"),
                ("Enter", "Add dependency"),
                ("Esc", "Cancel"),
            ],
        );
    }

    // Tab-specific contextual help
    match app.selected_tab {
        0 => {
            // Issues view - mode-specific help
            let mode = app.issues_view_state.view_mode();
            match mode {
                ui::views::IssuesViewMode::List => (
                    "Issues List View Help".to_string(),
                    "Navigate and Manage Issues".to_string(),
                    vec![
                        ("↑/↓ or j/k", "Navigate through issues"),
                        ("Enter", "View issue details"),
                        ("c", "Create new issue"),
                        ("e", "Edit selected issue"),
                        ("d", "Delete selected issue"),
                        ("Space", "Select/deselect issue"),
                        ("a", "Select all issues"),
                        ("x", "Clear selection"),
                        ("/", "Search/filter issues"),
                        ("f", "Open quick filter menu"),
                        ("Ctrl+S", "Save current filter"),
                        ("F1-F11", "Apply saved filter"),
                        ("?", "Show all keyboard shortcuts"),
                        ("Esc", "Clear search or go back"),
                    ],
                ),
                ui::views::IssuesViewMode::Detail => (
                    "Issue Detail View Help".to_string(),
                    "View Issue Information".to_string(),
                    vec![
                        ("e", "Edit this issue"),
                        ("d", "Delete this issue"),
                        ("Esc", "Back to issues list"),
                        ("?", "Show all keyboard shortcuts"),
                    ],
                ),
                ui::views::IssuesViewMode::Edit => (
                    "Issue Edit Mode Help".to_string(),
                    "Edit Issue Fields".to_string(),
                    vec![
                        ("Tab", "Move to next field"),
                        ("Shift+Tab", "Move to previous field"),
                        ("Ctrl+S", "Save changes"),
                        ("Esc", "Cancel editing"),
                        ("?", "Show all keyboard shortcuts"),
                    ],
                ),
                ui::views::IssuesViewMode::Create => (
                    "Create Issue Help".to_string(),
                    "Create New Issue".to_string(),
                    vec![
                        ("Tab", "Move to next field"),
                        ("Shift+Tab", "Move to previous field"),
                        ("Ctrl+S", "Create issue"),
                        ("Esc", "Cancel creation"),
                        ("?", "Show all keyboard shortcuts"),
                    ],
                ),
            }
        }
        1 => (
            "Dependencies View Help".to_string(),
            "Manage Issue Dependencies".to_string(),
            vec![
                ("↑/↓ or j/k", "Navigate through dependencies"),
                ("a", "Add new dependency"),
                ("r", "Remove dependency"),
                ("Enter", "View issue details"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        2 => (
            "Labels View Help".to_string(),
            "Manage Issue Labels".to_string(),
            vec![
                ("↑/↓ or j/k", "Navigate through labels"),
                ("Enter", "Select/apply label"),
                ("a", "Add new label"),
                ("d", "Delete label"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        3 => (
            "PERT Chart View Help".to_string(),
            "Project Evaluation and Review Technique".to_string(),
            vec![
                ("↑/↓", "Navigate through nodes"),
                ("+/-", "Zoom in/out"),
                ("c", "Configure chart settings"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        4 => (
            "Gantt Chart View Help".to_string(),
            "Timeline and Dependencies".to_string(),
            vec![
                ("↑/↓", "Navigate through tasks"),
                ("+/-", "Zoom timeline"),
                ("g", "Change grouping mode"),
                ("c", "Configure chart settings"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        5 => (
            "Kanban Board View Help".to_string(),
            "Drag and Drop Task Management".to_string(),
            vec![
                ("↑/↓/←/→", "Navigate between cards"),
                ("Space", "Move card to different column"),
                ("c", "Configure board"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        6 => (
            "Molecular View Help".to_string(),
            "Advanced Issue Visualization".to_string(),
            vec![
                ("↑/↓", "Navigate through items"),
                ("Tab", "Switch between molecular tabs"),
                ("Enter", "Select item"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        7 => (
            "Database View Help".to_string(),
            "Database Management".to_string(),
            vec![
                ("↑/↓", "Navigate through operations"),
                ("r", "Refresh database"),
                ("c", "Compact database"),
                ("v", "Verify database integrity"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        8 => (
            "Help View".to_string(),
            "Documentation and Guides".to_string(),
            vec![
                ("←/→ or h/l", "Navigate between sections"),
                ("Esc", "Go back"),
                ("?", "Quick keyboard reference"),
                ("F1", "Context-sensitive help"),
            ],
        ),
        _ => (
            "General Help".to_string(),
            "Global Navigation".to_string(),
            vec![
                ("q", "Quit application"),
                ("Tab", "Next tab"),
                ("Shift+Tab", "Previous tab"),
                ("1-9", "Jump to tab by number"),
                ("Ctrl+P or F12", "Toggle performance stats"),
                ("?", "Show all keyboard shortcuts"),
                ("F1", "Context-sensitive help"),
            ],
        ),
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
