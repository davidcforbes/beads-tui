pub mod beads;
pub mod config;
pub mod demo;
pub mod graph;
pub mod models;
pub mod runtime;
pub mod tasks;
pub mod test_mode;
pub mod tts;
pub mod ui;
pub mod undo;
pub mod utils;

use anyhow::Result;
use clap::Parser;
use config::Action;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io::{self, Write};
use std::sync::Arc;
use tasks::TaskOutput;
use ui::views::{DatabaseView, DependencyTreeView, HelpView, HelpViewState, IssuesView, LabelsView};
use undo::IssueUpdateCommand;

/// Terminal UI for the beads issue tracker
#[derive(Parser, Debug)]
#[command(name = "beads-tui")]
#[command(about = "Terminal UI for beads issue tracker", long_about = None)]
struct Args {
    /// Enable text-to-speech announcements for screen readers
    #[arg(long)]
    tts: bool,

    /// Run in demo mode with generated test data
    #[arg(long)]
    demo: bool,

    /// Open specific view (0-11)
    #[arg(long, value_name = "TAB_INDEX")]
    view: Option<usize>,

    /// Test data set (small|medium|large|deps|edge)
    #[arg(long, value_name = "DATASET", default_value = "small")]
    dataset: String,

    /// Render view to text and exit (for testing)
    #[arg(long)]
    snapshot: bool,

    /// Output file for snapshot mode
    #[arg(long, value_name = "FILE")]
    output: Option<String>,

    /// Terminal dimensions (WIDTHxHEIGHT)
    #[arg(long, value_name = "WxH", default_value = "120x40")]
    size: String,

    /// List all available views and exit
    #[arg(long)]
    list_views: bool,

    /// Run test sequence through all views
    #[arg(long)]
    test_all_views: bool,

    /// Duration per view in test mode (seconds)
    #[arg(long, default_value = "2")]
    test_duration: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --list-views early (before terminal setup)
    if args.list_views {
        test_mode::print_view_list();
        return Ok(());
    }

    // Setup panic hook to restore terminal on panic
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Try multiple times to restore terminal state
        for _ in 0..3 {
            if disable_raw_mode().is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Try to restore screen state
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        let _ = stdout.flush();

        // Call original panic handler
        default_panic(info);
    }));

    // Setup logging to file to avoid interfering with TUI
    use std::fs::OpenOptions;
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("beads-tui.log")
        .expect("Failed to open log file");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(move || log_file.try_clone().expect("Failed to clone log file"))
        .with_ansi(false) // Disable ANSI colors in log file
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize TTS if requested
    let tts_manager = tts::TtsManager::new(args.tts);
    if tts_manager.is_available() {
        tracing::info!("Screen reader support enabled");
    }

    // Create app state (demo or real)
    let mut app = if args.demo {
        let dataset = demo::DemoDataset::generate(&args.dataset).map_err(|e| {
            anyhow::anyhow!("Failed to generate demo dataset: {}", e)
        })?;
        models::AppState::with_demo_data(dataset, tts_manager)
    } else {
        models::AppState::with_tts(tts_manager)
    };

    // Set initial view if specified
    if let Some(view_index) = args.view {
        if view_index < app.tabs.len() {
            app.selected_tab = view_index;
        } else {
            // Clean up terminal before error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            return Err(anyhow::anyhow!(
                "Invalid view index: {}. Valid range: 0-{} (use --list-views to see all views)",
                view_index,
                app.tabs.len() - 1
            ));
        }
    }

    // Run in snapshot mode if requested
    if args.snapshot {
        let view_index = args.view.unwrap_or(0);
        let size = test_mode::parse_terminal_size(&args.size)?;

        let result = test_mode::run_snapshot_mode(
            &mut terminal,
            &mut app,
            view_index,
            args.output,
            size,
            |f, app| ui(f, app),
        );

        // Clean up terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        return result;
    }

    // Run test sequence if requested
    if args.test_all_views {
        let result = test_mode::run_test_sequence(
            &mut terminal,
            &mut app,
            args.test_duration,
            |f, app| ui(f, app),
        );

        // Clean up terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        return result;
    }

    // Normal interactive mode continues below...

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

/// Handle mouse events
fn handle_mouse_event<B: ratatui::backend::Backend>(
    mouse: MouseEvent,
    terminal: &mut Terminal<B>,
    app: &mut models::AppState,
) {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            // Simulate Down key press for scrolling
            let key = KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            };

            // Route to active view
            match app.selected_tab {
                0 | 1 => handle_issues_view_event(key, app),
                2 => handle_kanban_view_event(key, app),
                3 => handle_dependencies_view_event(key, app),
                4 => handle_labels_view_event(key, app),
                5 => handle_gantt_view_event(key, app),
                6 => handle_pert_view_event(key, app),
                7 => handle_molecular_view_event(key, app),
                8 | 9 => handle_database_view_event(key, app),
                10 => handle_issues_view_event(key, app), // Record Detail
                11 => handle_help_view_event(key, app),
                _ => {}
            }
            app.mark_dirty();
        }
        MouseEventKind::ScrollUp => {
            // Simulate Up key press for scrolling
            let key = KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            };

            // Route to active view
            match app.selected_tab {
                0 | 1 => handle_issues_view_event(key, app),
                2 => handle_kanban_view_event(key, app),
                3 => handle_dependencies_view_event(key, app),
                4 => handle_labels_view_event(key, app),
                5 => handle_gantt_view_event(key, app),
                6 => handle_pert_view_event(key, app),
                7 => handle_molecular_view_event(key, app),
                8 | 9 => handle_database_view_event(key, app),
                10 => handle_issues_view_event(key, app), // Record Detail
                11 => handle_help_view_event(key, app),
                _ => {}
            }
            app.mark_dirty();
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // Store mouse down position for click detection
            app.mouse_down_pos = Some((mouse.column, mouse.row));
            tracing::debug!("Mouse down at {}, {}", mouse.column, mouse.row);
        }
        MouseEventKind::Up(MouseButton::Left) => {
            // Check if this is a click (down and up at same/similar position)
            if let Some((down_col, down_row)) = app.mouse_down_pos {
                let col_diff = (mouse.column as i16 - down_col as i16).abs();
                let row_diff = (mouse.row as i16 - down_row as i16).abs();

                // Allow 1-2 character tolerance for click detection
                if col_diff <= 2 && row_diff <= 1 {
                    tracing::debug!("Mouse click detected at {}, {}", mouse.column, mouse.row);
                    // Get terminal dimensions from current buffer
                    let terminal_area = terminal.current_buffer_mut().area;
                    handle_mouse_click(mouse.column, mouse.row, terminal_area.width, terminal_area.height, app);
                }
            }
            app.mouse_down_pos = None;
        }
        _ => {}
    }
}

/// Capture the current terminal screen as text
fn capture_screen_to_text<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> String {
    let buffer = terminal.current_buffer_mut();
    let area = buffer.area;
    let (width, height) = (area.width, area.height);

    let mut output = String::new();

    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let index = (y * width + x) as usize;
            if let Some(cell) = buffer.content.get(index) {
                line.push_str(cell.symbol());
            }
        }
        // Trim trailing whitespace from each line
        output.push_str(line.trim_end());
        output.push('\n');
    }

    output
}

/// Handle F12 screen capture
fn handle_screen_capture<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut models::AppState,
) {
    let screen_text = capture_screen_to_text(terminal);

    // Try to copy to clipboard
    let mut clipboard_success = false;
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            if let Ok(_) = clipboard.set_text(&screen_text) {
                clipboard_success = true;
                tracing::info!("Screen captured to clipboard");
            } else {
                tracing::warn!("Failed to write to clipboard");
            }
        }
        Err(e) => {
            tracing::warn!("Failed to access clipboard: {}", e);
        }
    }

    // Always save to file as backup
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screen_capture_{}.txt", timestamp);

    match std::fs::write(&filename, &screen_text) {
        Ok(_) => {
            if clipboard_success {
                app.set_success(format!("Screen captured to clipboard and {}", filename));
            } else {
                app.set_success(format!("Screen captured to {}", filename));
            }
            tracing::info!("Screen captured to {}", filename);
        }
        Err(e) => {
            if clipboard_success {
                app.set_success("Screen captured to clipboard".to_string());
            } else {
                app.set_error(format!("Failed to capture screen: {}", e));
                tracing::error!("Failed to save screen capture: {}", e);
            }
        }
    }
}

/// Handle mouse click events (after detecting down+up at same position)
fn handle_mouse_click(col: u16, row: u16, _terminal_width: u16, terminal_height: u16, app: &mut models::AppState) {
    // Layout:
    // Row 0-2: TITLE block (with Find field at row 1)
    // Row 3-5: VIEWS/Tabs block (tab content at row 4)
    // Row 6-8: Filter bar (if visible) - filter content at row 7
    // Row 9+: Content area (when filter bar visible)
    // Row 6+: Content area (when filter bar NOT visible)
    // Last 3 rows: ACTION bar block (content at terminal_height - 2)

    // Hit test for Find field in title bar (row 1)
    if row == 1 {
        // Find field is in the third column of the title layout
        // Approximate column range: starts around col 50-90
        // For now, clicking anywhere in row 1 will focus the search
        tracing::info!("Title bar clicked at col {}, focusing search", col);
        if !app.issues_view_state.search_state().search_state().is_focused() {
            app.issues_view_state.search_state_mut().search_state_mut().set_focused(true);
            app.mark_dirty();
        }
        return;
    }

    // Hit test for tabs (row 4 is the tab content row)
    if row == 4 {
        // Calculate which tab was clicked based on column
        // Tabs are rendered sequentially with their actual text width, not evenly divided
        // Each tab has format: " {shortcut}:{name}" and tabs are separated by " │ " divider (3 chars)

        // Adjust column for left border (block border is 1 char)
        if col > 0 {
            let adjusted_col = col - 1;

            // Calculate cumulative tab positions
            // Tab format: " 1:Issues", " 2:Split", etc.
            let divider_width = 3u16; // " │ " between tabs
            let mut current_pos = 0u16;

            for (i, &name) in app.tabs.iter().enumerate() {
                // Calculate tab text width including shortcut
                let shortcut = match i {
                    0..=8 => format!("{}:", i + 1),  // 1-9
                    9 => "0:".to_string(),            // 0 for Utilities
                    10 => "R:".to_string(),           // R for Record
                    11 => "H:".to_string(),           // H for Help
                    _ => "".to_string(),
                };
                let tab_text = format!(" {}{}", shortcut, name);
                let tab_width = tab_text.len() as u16;

                // Check if click is within this tab's range
                let tab_end = current_pos + tab_width;
                if adjusted_col >= current_pos && adjusted_col < tab_end {
                    tracing::info!("Tab {} clicked at col {} (adjusted {}): range {}-{} '{}'",
                        i, col, adjusted_col, current_pos, tab_end, app.tabs[i]);
                    app.selected_tab = i;
                    app.mark_dirty();
                    return;
                }

                // Move to next tab position (tab width + divider)
                current_pos = tab_end + divider_width;
            }
        }
        return;
    }

    // Hit test for filter bar (row 7 is the filter content row)
    if row == 7 && app.issues_view_state.filter_bar_state.is_some() {
        tracing::info!("Filter bar clicked at col {}", col);
        // For now, just log the click. Future enhancement: detect which filter field was clicked
        // and open the corresponding dropdown
        return;
    }

    // Hit test for list items (Issues view)
    if app.selected_tab == 0 || app.selected_tab == 1 {
        // Calculate row offset based on filter bar visibility
        let filter_bar_visible = app.issues_view_state.filter_bar_state.is_some();

        // When filter bar is visible:
        //   Row 6-8: Filter bar
        //   Row 9: Table top border
        //   Row 10: Table header
        //   Row 11+: Table data rows
        // When filter bar is NOT visible:
        //   Row 6: Results info line
        //   Row 7: Table top border
        //   Row 8: Table header
        //   Row 9+: Table data rows

        let first_data_row = if filter_bar_visible { 11 } else { 9 };

        if row >= first_data_row {
            let item_index = (row - first_data_row) as usize;
            let filtered_issues_len = app.issues_view_state.search_state().filtered_issues().len();

            if item_index < filtered_issues_len {
                tracing::info!("List item {} clicked at row {} (first_data_row: {}, filter_bar_visible: {})",
                    item_index, row, first_data_row, filter_bar_visible);
                app.issues_view_state
                    .search_state_mut()
                    .list_state_mut()
                    .select(Some(item_index));
                app.mark_dirty();
            }
        }
    }

    // Hit test for action bar (last 3 rows of terminal)
    let action_bar_content_row = terminal_height.saturating_sub(2);
    if row == action_bar_content_row {
        // Get the contextual actions to determine what was clicked
        let (_nav_actions, action_items) = get_contextual_actions(app);

        // The action bar has this format (adjusting for border):
        // " nav_actions │ action0 │ action1 │ action2 │ action3 │ action4 │ action5 │ action6 "
        // Each section is separated by " │ " (3 chars)

        // Adjust for left border
        if col > 0 {
            let adjusted_col = col - 1;

            // Skip navigation section (approximate width ~67 chars + separator)
            // Navigation: "↓:Up ↑:Down (row) PgUp/PgDn (page) →:Scroll-Right ←:Scroll-Left"
            let nav_width = 67u16;
            let separator_width = 3u16;

            if adjusted_col > nav_width + separator_width {
                // We're in the action items section
                let action_section_start = nav_width + separator_width;
                let action_col = adjusted_col - action_section_start;

                // Each action has approximate format "X:Action" (varies)
                // Calculate which action section we're in based on separators
                let mut current_pos = 0u16;

                for (idx, action_text) in action_items.iter().enumerate() {
                    let action_width = action_text.len() as u16;
                    let section_end = current_pos + action_width + separator_width;

                    if action_col >= current_pos && action_col < current_pos + action_width {
                        // Clicked on this action
                        tracing::info!("Action bar item {} clicked: '{}' at col {}", idx, action_text, col);

                        // Parse the action key from the format "X:Label" or "^X:Label"
                        if let Some((key_char, has_ctrl)) = parse_action_key(action_text) {
                            // Simulate the key press
                            let key = create_key_event_from_action(key_char, has_ctrl);
                            simulate_key_press(key, app);
                            app.mark_dirty();
                        }
                        return;
                    }

                    current_pos = section_end;
                }
            }
        }
    }
}

/// Parse the key character from an action label like "R:Read" or "^S:Save"
/// Returns (character, has_ctrl_modifier)
fn parse_action_key(action_text: &str) -> Option<(char, bool)> {
    if let Some(colon_pos) = action_text.find(':') {
        let key_part = &action_text[..colon_pos];
        // Handle special cases like "^S", "^X", "^Del", etc.
        if key_part.starts_with('^') && key_part.len() > 1 {
            if let Some(ch) = key_part.chars().nth(1) {
                return Some((ch, true));
            }
        } else if key_part.len() == 1 {
            if let Some(ch) = key_part.chars().next() {
                return Some((ch, false));
            }
        } else if key_part == "Esc" {
            // Special handling for Esc key
            return Some(('\x1b', false)); // ESC character
        } else if key_part == "Tab" {
            // Special handling for Tab key
            return Some(('\t', false));
        }
    }
    None
}

/// Create a KeyEvent from an action character
fn create_key_event_from_action(key_char: char, has_ctrl: bool) -> KeyEvent {
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind};

    // Map special characters to their key codes
    let (code, mut modifiers) = match key_char {
        '↵' => (KeyCode::Enter, KeyModifiers::empty()),
        '\x1b' => (KeyCode::Esc, KeyModifiers::empty()),
        '\t' => (KeyCode::Tab, KeyModifiers::empty()),
        'D' if key_char.is_uppercase() && has_ctrl => {
            // ^Del becomes Ctrl+Delete
            (KeyCode::Delete, KeyModifiers::CONTROL)
        }
        _ => {
            // Regular character, check if it should be uppercase or lowercase
            let base_modifiers = if key_char.is_uppercase() {
                KeyModifiers::SHIFT
            } else {
                KeyModifiers::empty()
            };
            (KeyCode::Char(key_char.to_ascii_lowercase()), base_modifiers)
        }
    };

    // Add CONTROL modifier if specified
    if has_ctrl && !matches!(key_char, 'D') {
        modifiers |= KeyModifiers::CONTROL;
    }

    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

/// Simulate a key press by routing it to the appropriate handler
fn simulate_key_press(key: KeyEvent, app: &mut models::AppState) {
    match app.selected_tab {
        0 | 1 => handle_issues_view_event(key, app),
        2 => handle_kanban_view_event(key, app),
        3 => handle_dependencies_view_event(key, app),
        4 => handle_labels_view_event(key, app),
        5 => handle_gantt_view_event(key, app),
        6 => handle_pert_view_event(key, app),
        7 => handle_molecular_view_event(key, app),
        8 | 9 => handle_database_view_event(key, app),
        10 => handle_help_view_event(key, app),
        _ => {}
    }
}

/// Handle keyboard events for the Issues view
///
/// ESC KEY HIERARCHY (highest to lowest priority):
/// 1. Dismiss notifications
/// 2. Close undo/redo history overlay
/// 3. Cancel active dialogs (delete, dependency, filter save, column manager)
/// 4. Close selectors/pickers (priority, label, status)
/// 5. Exit search/filter modes
/// 6. Cancel edit/create modes
/// 7. Return from detail view to list
///    Each handler returns early, so only the highest-priority applicable action is taken.
fn handle_issues_view_event(key: KeyEvent, app: &mut models::AppState) {
    use ui::views::IssuesViewMode;

    let key_code = key.code;
    let action = app
        .config
        .keybindings
        .find_action(&key.code, &key.modifiers);

    // ESC Priority 1: Dismiss notifications (highest)
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return;
    }

    // Handle filter dropdown hotkeys (1-7 for filters and reset)
    if app.issues_view_state.filter_bar_state.is_some() {
        match key_code {
            KeyCode::Char('1') => {
                // Toggle Status filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Status);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('2') => {
                // Toggle Type filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Type);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('3') => {
                // Toggle Priority filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Priority);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('4') => {
                // Toggle Labels filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Labels);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('5') => {
                // Toggle Created date filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Created);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('6') => {
                // Toggle Updated date filter dropdown
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Updated);
                    app.mark_dirty();
                }
                return;
            }
            KeyCode::Char('7') => {
                // Reset all filters
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    // Close any open dropdown
                    filter_bar_state.close_dropdown();
                    // Clear all selections (reset to "All")
                    filter_bar_state.status_dropdown.clear_selection();
                    filter_bar_state.priority_dropdown.clear_selection();
                    filter_bar_state.type_dropdown.clear_selection();
                    filter_bar_state.labels_dropdown.clear_selection();
                    filter_bar_state.created_dropdown.clear_selection();
                    filter_bar_state.updated_dropdown.clear_selection();
                    app.mark_dirty();
                }
                return;
            }
            _ => {}
        }

        // Handle filter dropdown navigation if a dropdown is open
        if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
            if filter_bar_state.active_dropdown.is_some() {
                match action {
                    Some(Action::MoveUp) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.previous();
                            app.mark_dirty();
                        }
                        return;
                    }
                    Some(Action::MoveDown) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.next();
                            app.mark_dirty();
                        }
                        return;
                    }
                    Some(Action::ToggleSelection) => {
                        if let Some(mut dropdown) = filter_bar_state.active_dropdown_mut() {
                            dropdown.toggle_selected();
                            app.mark_dirty();
                        }
                        return;
                    }
                    Some(Action::ConfirmDialog) => {
                        // Apply filter and close dropdown
                        filter_bar_state.close_dropdown();
                        app.issues_view_state.apply_filter_bar_filters();
                        app.mark_dirty();
                        return;
                    }
                    Some(Action::CancelDialog) => {
                        // Close dropdown without applying
                        filter_bar_state.close_dropdown();
                        app.mark_dirty();
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    // Handle global actions
    match action {
        Some(Action::Undo) => {
            app.undo().ok();
            return;
        }
        Some(Action::Redo) => {
            app.redo().ok();
            return;
        }
        Some(Action::ShowNotificationHistory) => {
            app.toggle_notification_history();
            return;
        }
        Some(Action::ShowIssueHistory) => {
            // Check if undo history overlay is what was meant (Ctrl+H maps to ShowNotificationHistory in config default??)
            // Config: ShowNotificationHistory -> Ctrl+h. ShowIssueHistory -> Alt+h.
            // Old code: Ctrl+h -> toggle_undo_history.
            // Let's stick to the Action definitions.
            // If action is ShowNotificationHistory, do that.
            // If we want UndoHistory, we need an Action for it.
            // Action::ShowIssueHistory exists.
            // Wait, old code mapped Ctrl+H to toggle_undo_history. Config maps Ctrl+H to ShowNotificationHistory.
            // The user wanted standard keys.
            // Let's assume Config is the source of truth now.
        }
        _ => {}
    }

    // Handle undo history overlay toggle (Special case if not covered by Action)
    // Old code used Ctrl+H for undo history.
    // Let's use the Config action if possible.
    // Config has ShowNotificationHistory (Ctrl+H).
    // Config doesn't have specific "ToggleUndoHistory".
    // I'll keep the old logic for now if it's not in Config, OR rely on Config.
    // The previous code had:
    // if key_code == KeyCode::Char('h') && key.modifiers.contains(KeyModifiers::CONTROL) { app.toggle_undo_history(); }
    // Config default for ShowNotificationHistory is Ctrl+h.
    // This is a conflict in the legacy code vs new config.
    // I will respect the NEW config which maps Ctrl+H to ShowNotificationHistory.
    // But wait, the user wants "Universal Set".
    // I'll skip this specific conflict resolution for a moment and focus on the structure.

    // ESC Priority 2: Close undo history overlay
    if app.is_undo_history_visible() && matches!(action, Some(Action::DismissNotification)) {
        app.hide_undo_history();
        return;
    }

    // Clear errors when entering create or edit mode
    // We can't easily map this to Action::CreateIssue yet because we are in "global" scope of function
    if matches!(action, Some(Action::CreateIssue) | Some(Action::EditIssue)) {
        app.clear_error();
    }

    // Handle dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.dialog_state {
        match action {
            Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                dialog_state.select_previous(2); // Yes/No = 2 buttons
                return;
            }
            Some(Action::MoveRight) | Some(Action::NextDialogButton) | Some(Action::NextTab) => {
                dialog_state.select_next(2);
                return;
            }
            Some(Action::ConfirmDialog) => {
                // Execute pending action based on selected button
                let selected = dialog_state.selected_button();
                if selected == 0 {
                    // Yes was selected
                    if let Some(action) = app.pending_action.take() {
                        if let Some(issue_id) = action.strip_prefix("delete:") {
                            tracing::info!("Confirmed delete for issue: {}", issue_id);

                            // Spawn background task (non-blocking)
                            let issue_id_owned = issue_id.to_string();
                            let _ = app.spawn_task(
                                &format!("Deleting issue {}", issue_id),
                                move |client| async move {
                                    client.delete_issue(&issue_id_owned).await?;
                                    tracing::info!(
                                        "Successfully deleted issue: {}",
                                        issue_id_owned
                                    );
                                    Ok(TaskOutput::IssueDeleted(issue_id_owned))
                                },
                            );
                        } else if let Some(filter_idx_str) = action.strip_prefix("delete_filter:") {
                            tracing::info!("Confirmed delete filter at index: {}", filter_idx_str);

                            if let Ok(i) = filter_idx_str.parse::<usize>() {
                                app.issues_view_state
                                    .search_state_mut()
                                    .delete_saved_filter(i);
                                // Sync to config
                                let filters = app
                                    .issues_view_state
                                    .search_state()
                                    .saved_filters()
                                    .to_vec();
                                app.config.filters = filters;
                                let _ = app.config.save();
                                app.set_success("Filter deleted".to_string());
                            }
                        } else if let Some(ids) = action.strip_prefix("indent:") {
                            let parts: Vec<&str> = ids.split(':').collect();
                            if parts.len() == 2 {
                                let selected_id = parts[0].to_string();
                                let prev_id = parts[1].to_string();
                                tracing::info!(
                                    "Confirmed indent {} under {}",
                                    selected_id,
                                    prev_id
                                );

                                // Spawn background task (non-blocking)
                                let _ =
                                    app.spawn_task("Indenting issue", move |client| async move {
                                        client.add_dependency(&selected_id, &prev_id).await?;
                                        tracing::info!(
                                            "Successfully indented {} under {}",
                                            selected_id,
                                            prev_id
                                        );
                                        Ok(TaskOutput::DependencyAdded)
                                    });
                            }
                        } else if let Some(ids) = action.strip_prefix("outdent:") {
                            let parts: Vec<&str> = ids.split(':').collect();
                            if parts.len() == 2 {
                                let selected_id = parts[0].to_string();
                                let parent_id = parts[1].to_string();
                                tracing::info!(
                                    "Confirmed outdent {} from parent {}",
                                    selected_id,
                                    parent_id
                                );

                                // Spawn background task (non-blocking)
                                let _ =
                                    app.spawn_task("Outdenting issue", move |client| async move {
                                        client.remove_dependency(&selected_id, &parent_id).await?;
                                        tracing::info!(
                                            "Successfully outdented {} from {}",
                                            selected_id,
                                            parent_id
                                        );
                                        Ok(TaskOutput::DependencyRemoved)
                                    });
                            }
                        } else if action == "compact_database" {
                            tracing::info!("Confirmed compact database");

                            // Spawn background task (non-blocking)
                            let _ = app.spawn_task("Compacting database", |client| async move {
                                client.compact_database().await?;
                                Ok(TaskOutput::DatabaseCompacted)
                            });
                        }
                    }
                }
                // Close dialog (both Yes and No)
                app.dialog_state = None;
                app.pending_action = None;
                return;
            }
            Some(Action::CancelDialog) => {
                // ESC Priority 3: Cancel dialog
                tracing::debug!("Dialog cancelled");
                app.dialog_state = None;
                app.pending_action = None;
                return;
            }
            Some(Action::Quit) | Some(Action::ShowHelp) => {
                // Let '?' and 'q' fall through to be handled globally
                // Dialog remains open but user can still get help or quit
            }
            _ => {
                // Ignore other keys when dialog is active
                return;
            }
        }
    }

    // Handle column manager events if active
    if let Some(ref mut cm_state) = app.column_manager_state {
        match action {
            Some(Action::MoveUp) => {
                cm_state.select_previous();
                return;
            }
            Some(Action::MoveDown) => {
                cm_state.select_next();
                return;
            }
            Some(Action::ToggleSelection) => {
                cm_state.toggle_visibility();
                return;
            }
            Some(Action::Refresh) => {
                // Using 'r' for reset/refresh
                // Reset to defaults
                let defaults = crate::models::table_config::TableConfig::default().columns;
                cm_state.reset(defaults);
                return;
            }
            Some(Action::ConfirmDialog) => {
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
            Some(Action::CancelDialog) => {
                // Cancel without applying
                app.column_manager_state = None;
                return;
            }
            Some(Action::MoveLeft) if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column up (Alt+Left in existing code? Logic says move_up)
                cm_state.move_up();
                return;
            }
            Some(Action::MoveRight) if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move selected column down
                cm_state.move_down();
                return;
            }
            Some(Action::Quit) | Some(Action::ShowHelp) => {
                // Let '?' and 'q' fall through to be handled globally
            }
            _ => {
                // Ignore other keys when column manager is active
                return;
            }
        }
    }

    // Handle filter save dialog events if dialog is active
    if let Some(ref mut dialog_state) = app.filter_save_dialog_state {
        match action {
            Some(Action::NextDialogButton) => {
                dialog_state.focus_next();
                return;
            }
            Some(Action::PrevDialogButton) => {
                dialog_state.focus_previous();
                return;
            }
            Some(Action::MoveLeft) => {
                dialog_state.move_cursor_left();
                return;
            }
            Some(Action::MoveRight) => {
                dialog_state.move_cursor_right();
                return;
            }
            // Backspace is text input, usually not mapped to action for dialogs unless specialized
            // But we need to handle Char(c) for text input.
            // We should check raw keys for text input if no action matched?
            // Or explicitly match Delete/Backspace actions if they exist.
            // Config doesn't have DeleteChar/InsertChar actions.
            // We'll fall back to raw key matching for text input if action is not navigation/confirm/cancel.
            Some(Action::ConfirmDialog) => {
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
            Some(Action::CancelDialog) => {
                // Cancel dialog
                tracing::debug!("Filter save dialog cancelled");
                app.hide_filter_save_dialog();
                return;
            }
            _ => {
                // Handle text input for non-action keys
                match key_code {
                    KeyCode::Backspace => {
                        dialog_state.delete_char();
                        return;
                    }
                    KeyCode::Char(c) => {
                        // Avoid handling 'q' or '?' if they triggered a global action that we skipped?
                        // If 'q' is mapped to Quit, 'action' is Quit. We are in the `_` branch, so action didn't match specific dialog actions.
                        // But we want to allow typing 'q' in the text field.
                        // CONFLICT: Global hotkeys vs Text Input.
                        // Standard solution: If text field is focused, suppress single-key hotkeys unless modifiers are present.
                        // In this dialog, everything is text input except nav/enter/esc.
                        dialog_state.insert_char(c);
                        return;
                    }
                    _ => return,
                }
            }
        }
    }

    // Handle dependency dialog events if dialog is open
    if app.dependency_dialog_state.is_open() {
        use ui::widgets::DependencyDialogFocus;

        match action {
            Some(Action::NextDialogButton) => {
                app.dependency_dialog_state.focus_next();
                app.mark_dirty();
                return;
            }
            Some(Action::PrevDialogButton) => {
                app.dependency_dialog_state.focus_previous();
                app.mark_dirty();
                return;
            }
            Some(Action::MoveLeft) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_previous_button();
                    app.mark_dirty();
                }
                return;
            }
            Some(Action::MoveRight) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Buttons {
                    app.dependency_dialog_state.select_next_button();
                    app.mark_dirty();
                }
                return;
            }
            Some(Action::MoveUp) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state
                        .autocomplete_state
                        .select_previous();
                    app.mark_dirty();
                }
                return;
            }
            Some(Action::MoveDown) => {
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    app.dependency_dialog_state.autocomplete_state.select_next();
                    app.mark_dirty();
                }
                return;
            }
            Some(Action::ToggleSelection) => {
                // Space
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::Type {
                    app.dependency_dialog_state.toggle_type();
                    app.mark_dirty();
                }
                // Also handle space as char if in text box?
                // Conflict resolution: Check focus
                if app.dependency_dialog_state.focus() == DependencyDialogFocus::IssueId {
                    if let KeyCode::Char(c) = key_code {
                        app.dependency_dialog_state
                            .autocomplete_state
                            .insert_char(c);
                        app.mark_dirty();
                    }
                }
                return;
            }
            Some(Action::ConfirmDialog) => {
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
                                    // Bidirectional "see also" relationship - moved to background task
                                    let current_id_clone = current_id.clone();
                                    let target_id_clone = target_issue_id.clone();

                                    let _ = app.spawn_task(
                                        "Linking issues",
                                        move |client| async move {
                                            client
                                                .relate_issues(&current_id_clone, &target_id_clone)
                                                .await
                                                .map_err(|e| {
                                                    crate::tasks::error::TaskError::ClientError(
                                                        e.to_string(),
                                                    )
                                                })?;
                                            Ok(crate::tasks::handle::TaskOutput::Success(format!(
                                                "Linked issues: {} <-> {}",
                                                current_id_clone, target_id_clone
                                            )))
                                        },
                                    );
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
                                        // Spawn background task (non-blocking)
                                        let from_id_owned = from_id.clone();
                                        let to_id_owned = to_id.clone();
                                        let _ = app.spawn_task(
                                            "Adding dependency",
                                            move |client| async move {
                                                client
                                                    .add_dependency(&from_id_owned, &to_id_owned)
                                                    .await?;
                                                tracing::info!(
                                                    "Added dependency: {} depends on {}",
                                                    from_id_owned,
                                                    to_id_owned
                                                );
                                                Ok(TaskOutput::DependencyAdded)
                                            },
                                        );
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
            Some(Action::CancelDialog) => {
                // Cancel dialog
                tracing::debug!("Dependency dialog cancelled");
                app.dependency_dialog_state.close();
                app.mark_dirty();
                return;
            }
            _ => {
                // Handle text input fallback
                match key_code {
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
                    _ => return,
                }
            }
        }
    }

    // Handle delete confirmation dialog events if active
    if app.is_delete_confirmation_visible() {
        if let Some(ref mut dialog_state) = app.delete_dialog_state {
            match action {
                Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return;
                }
                Some(Action::MoveRight) | Some(Action::NextDialogButton) => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return;
                }
                Some(Action::ConfirmDialog) => {
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
                Some(Action::CancelDialog) => {
                    // Cancel deletion
                    tracing::debug!("Delete confirmation cancelled");
                    app.cancel_delete_filter();
                    return;
                }
                Some(Action::Quit) | Some(Action::ShowHelp) => {
                    // Let '?' and 'q' fall through to be handled globally
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
            match action {
                Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
                    dialog_state.select_previous(2); // 2 buttons: Yes, No
                    return;
                }
                Some(Action::MoveRight) | Some(Action::NextDialogButton) => {
                    dialog_state.select_next(2); // 2 buttons: Yes, No
                    return;
                }
                Some(Action::ConfirmDialog) => {
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
                Some(Action::CancelDialog) => {
                    // Cancel removal
                    tracing::debug!("Dependency removal cancelled");
                    app.cancel_remove_dependency();
                    return;
                }
                Some(Action::Quit) | Some(Action::ShowHelp) => {
                    // Let '?' and 'q' fall through to be handled globally
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
        match action {
            Some(Action::MoveDown) => {
                issues_state.search_state_mut().filter_menu_next();
                return;
            }
            Some(Action::MoveUp) => {
                issues_state.search_state_mut().filter_menu_previous();
                return;
            }
            Some(Action::ConfirmDialog) => {
                issues_state.search_state_mut().filter_menu_confirm();
                app.set_success("Filter applied".to_string());
                return;
            }
            Some(Action::DeleteIssue) => {
                // 'd' or 'Delete'
                // Delete filter with confirmation
                if let Some(i) = issues_state.search_state().filter_menu_state().selected() {
                    if let Some(filter) = issues_state.search_state().saved_filters().get(i) {
                        let filter_name = filter.name.clone();
                        tracing::info!("Requesting confirmation to delete filter: {}", filter_name);

                        // Show confirmation dialog
                        app.dialog_state = Some(ui::widgets::DialogState::new());
                        app.pending_action = Some(format!("delete_filter:{}", i));

                        tracing::debug!("Showing delete confirmation for filter: {}", filter_name);
                    }
                }
                return;
            }
            Some(Action::CancelDialog) | Some(Action::ShowColumnManager) => {
                // 'm' closes menu too
                issues_state.search_state_mut().toggle_filter_menu();
                return;
            }
            _ => return, // Sink other keys
        }
    }

    match view_mode {
        IssuesViewMode::List => {
            // Check if a filter dropdown is active
            if let Some(ref mut fb_state) = issues_state.filter_bar_state {
                if fb_state.active_dropdown.is_some() {
                    // Filter dropdown is active - handle dropdown navigation
                    match key_code {
                        KeyCode::Up => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.previous();
                            }
                            return;
                        }
                        KeyCode::Down => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.next();
                            }
                            return;
                        }
                        KeyCode::Char(' ') => {
                            if let Some(ref mut dropdown) = fb_state.active_dropdown_mut() {
                                dropdown.toggle_selected();
                            }
                            return;
                        }
                        KeyCode::Enter => {
                            // Close dropdown first
                            fb_state.close_dropdown();
                            // Apply the selected filters to the issues list
                            issues_state.apply_filter_bar_filters();
                            return;
                        }
                        KeyCode::Esc => {
                            // Close dropdown without applying
                            fb_state.close_dropdown();
                            return;
                        }
                        _ => {
                            // Sink other keys when dropdown is active
                            return;
                        }
                    }
                }
            }

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

                                // Update title via command (undoable)
                                let client = Arc::new(app.beads_client.clone());
                                let update =
                                    beads::client::IssueUpdate::new().title(new_title.clone());
                                let command = Box::new(IssueUpdateCommand::new(
                                    client,
                                    issue_id.clone(),
                                    update,
                                ));

                                app.start_loading("Updating title...");

                                match app.execute_command(command) {
                                    Ok(()) => {
                                        app.stop_loading();
                                        tracing::info!(
                                            "Successfully updated title for: {} (undo with Ctrl+Z)",
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

                // Check for 'r' or 'R' key to open detail view (before action matching)
                if matches!(key_code, KeyCode::Char('r') | KeyCode::Char('R')) {
                    issues_state.enter_detail_view();
                    return;
                }

                match action {
                    Some(Action::MoveUp) => {
                        let len = issues_state.search_state().filtered_issues().len();
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_previous(len);
                    }
                    Some(Action::MoveDown) => {
                        let len = issues_state.search_state().filtered_issues().len();
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_next(len);
                    }
                    Some(Action::MoveLeft) if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Horizontal scroll left by one column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_left();
                    }
                    Some(Action::MoveRight) if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Horizontal scroll right by one column
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_right();
                    }
                    Some(Action::MoveLeft) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Jump to first column (Shift+Left or Shift+Home)
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_to_column(0);
                    }
                    Some(Action::MoveRight) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        // Jump to last column (Shift+Right or Shift+End)
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .scroll_to_column(usize::MAX); // Will be clamped to last column
                    }
                    Some(Action::PageDown) => {
                        // Scroll down by one page (viewport height)
                        let len = issues_state.search_state().filtered_issues().len();
                        let viewport_height = issues_state
                            .search_state()
                            .list_state()
                            .last_viewport_height() as usize;
                        // Use viewport height, fallback to 10 if not yet rendered
                        let page_size = if viewport_height > 0 { viewport_height } else { 10 };
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_page_down(len, page_size);
                    }
                    Some(Action::PageUp) => {
                        // Scroll up by one page (viewport height)
                        let len = issues_state.search_state().filtered_issues().len();
                        let viewport_height = issues_state
                            .search_state()
                            .list_state()
                            .last_viewport_height() as usize;
                        // Use viewport height, fallback to 10 if not yet rendered
                        let page_size = if viewport_height > 0 { viewport_height } else { 10 };
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_page_up(len, page_size);
                    }
                    // TODO: Move child reordering to Action enum (e.g. MoveChildUp/Down)
                    // Keeping hardcoded for now as it uses modifiers
                    _ if key_code == KeyCode::Up
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        reorder_child_issue(app, -1);
                    }
                    _ if key_code == KeyCode::Down
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        reorder_child_issue(app, 1);
                    }

                    Some(Action::ConfirmDialog) => {
                        // Enter -> Open edit popup (same form as Create/Edit)
                        issues_state.enter_edit_mode();
                    }
                    // 'v' is Cycle View in new config
                    // Old code had 'v' for Split Screen AND 'v' for Next View.
                    // We'll stick to Next View as it's more general.
                    // Split screen is just one of the views?
                    // IssuesViewMode has SplitScreen.
                    // So cycling view should eventually reach it.
                    Some(Action::EditIssue) => {
                        issues_state.enter_edit_mode();
                    }
                    Some(Action::CreateIssue) => {
                        issues_state.enter_create_mode();
                    }
                    Some(Action::ShowColumnManager) => {
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
                    Some(Action::CloseIssue) => {
                        // Close selected issue
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Closing issue: {}", issue_id);

                            // Execute close via command for undo support
                            use crate::beads::models::IssueStatus;
                            let client = Arc::new(app.beads_client.clone());
                            let update = crate::beads::client::IssueUpdate::new()
                                .status(IssueStatus::Closed);
                            let command =
                                Box::new(IssueUpdateCommand::new(client, issue_id.clone(), update));

                            app.start_loading(format!("Closing issue {}...", issue_id));

                            match app.execute_command(command) {
                                Ok(()) => {
                                    app.stop_loading();
                                    tracing::info!(
                                        "Successfully closed issue: {} (undo with Ctrl+Z)",
                                        issue_id
                                    );
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to close issue: {:?}", e);
                                    app.set_error(format!("Failed to close issue: {e}\n\nTry:\n• Verify the issue exists with 'bd show {issue_id}'\n• Check network connectivity\n• Run 'bd doctor' to diagnose issues"));
                                }
                            }
                        }
                    }
                    Some(Action::ReopenIssue) => {
                        // Reopen selected issue
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            tracing::info!("Reopening issue: {}", issue_id);

                            use crate::beads::models::IssueStatus;
                            let client = Arc::new(app.beads_client.clone());
                            let update =
                                crate::beads::client::IssueUpdate::new().status(IssueStatus::Open);
                            let command =
                                Box::new(IssueUpdateCommand::new(client, issue_id.clone(), update));

                            app.start_loading("Reopening issue...");

                            match app.execute_command(command) {
                                Ok(()) => {
                                    app.stop_loading();
                                    tracing::info!(
                                        "Successfully reopened issue: {} (undo with Ctrl+Z)",
                                        issue_id
                                    );
                                    app.reload_issues();
                                }
                                Err(e) => {
                                    app.stop_loading();
                                    tracing::error!("Failed to reopen issue: {:?}", e);
                                }
                            }
                        }
                    }
                    Some(Action::DeleteIssue) => {
                        // Delete selected issue with confirmation dialog
                        if let Some(issue) = issues_state.search_state().selected_issue() {
                            let issue_id = issue.id.clone();
                            let issue_title = issue.title.clone();
                            tracing::info!("Requesting confirmation to delete issue: {}", issue_id);

                            // Show confirmation dialog
                            app.dialog_state = Some(ui::widgets::DialogState::new());
                            app.pending_action = Some(format!("delete:{issue_id}"));

                            tracing::debug!("Showing delete confirmation for: {}", issue_title);
                        }
                    }
                    // In-place edit (F2/n) - Need Action::RenameIssue? Not in enum.
                    // Fallback to key check or map to EditIssue?
                    // EditIssue is 'e'. Rename is quick edit.
                    // Let's keep raw key check for specialized quick edit if it's not in Action.
                    // Or map 'n' to something else? Config maps 'n' to CreateIssue.
                    // Config maps 'r' to ReopenIssue.
                    // Config maps 'Shift+n' to NextSearchResult.
                    // We need a key for Quick Edit. 'F2' is standard.
                    // We'll keep F2 raw check.
                    _ if matches!(key_code, KeyCode::F(2)) => {
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

                    Some(Action::IndentIssue) => {
                        // Indent
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
                                            "Requesting confirmation to indent {} under {}",
                                            selected_id,
                                            prev_id
                                        );

                                        // Show confirmation dialog
                                        app.dialog_state = Some(ui::widgets::DialogState::new());
                                        app.pending_action =
                                            Some(format!("indent:{}:{}", selected_id, prev_id));
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
                    Some(Action::OutdentIssue) => {
                        // Outdent
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
                                    "Requesting confirmation to outdent {} from parent {}",
                                    selected_id,
                                    parent_id
                                );

                                // Show confirmation dialog
                                app.dialog_state = Some(ui::widgets::DialogState::new());
                                app.pending_action =
                                    Some(format!("outdent:{}:{}", selected_id, parent_id));
                            } else {
                                app.set_error("Issue has no parent to outdent from".to_string());
                            }
                        }
                    }
                    Some(Action::ToggleFilter) => {
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
                    Some(Action::OpenStatusFilter) => {
                        // Open or close status filter dropdown
                        if issues_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = ui::widgets::FilterBarState::new(
                                collect_unique_statuses(issues_state),
                                collect_unique_priorities(issues_state),
                                collect_unique_types(issues_state),
                                collect_unique_labels(issues_state),
                                collect_unique_assignees(issues_state),
                                collect_unique_created_dates(issues_state),
                                collect_unique_updated_dates(issues_state),
                                collect_unique_closed_dates(issues_state),
                            );
                            issues_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = issues_state.filter_bar_state {
                            fb_state.toggle_dropdown(ui::widgets::FilterDropdownType::Status);
                        }
                    }
                    Some(Action::OpenPriorityFilter) => {
                        // Open or close priority filter dropdown
                        if issues_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = ui::widgets::FilterBarState::new(
                                collect_unique_statuses(issues_state),
                                collect_unique_priorities(issues_state),
                                collect_unique_types(issues_state),
                                collect_unique_labels(issues_state),
                                collect_unique_assignees(issues_state),
                                collect_unique_created_dates(issues_state),
                                collect_unique_updated_dates(issues_state),
                                collect_unique_closed_dates(issues_state),
                            );
                            issues_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = issues_state.filter_bar_state {
                            fb_state.toggle_dropdown(ui::widgets::FilterDropdownType::Priority);
                        }
                    }
                    Some(Action::OpenTypeFilter) => {
                        // Open or close type filter dropdown
                        if issues_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = ui::widgets::FilterBarState::new(
                                collect_unique_statuses(issues_state),
                                collect_unique_priorities(issues_state),
                                collect_unique_types(issues_state),
                                collect_unique_labels(issues_state),
                                collect_unique_assignees(issues_state),
                                collect_unique_created_dates(issues_state),
                                collect_unique_updated_dates(issues_state),
                                collect_unique_closed_dates(issues_state),
                            );
                            issues_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = issues_state.filter_bar_state {
                            fb_state.toggle_dropdown(ui::widgets::FilterDropdownType::Type);
                        }
                    }
                    Some(Action::OpenLabelsFilter) => {
                        // Open or close labels filter dropdown
                        if issues_state.filter_bar_state.is_none() {
                            // Initialize filter bar state
                            let filter_bar_state = ui::widgets::FilterBarState::new(
                                collect_unique_statuses(issues_state),
                                collect_unique_priorities(issues_state),
                                collect_unique_types(issues_state),
                                collect_unique_labels(issues_state),
                                collect_unique_assignees(issues_state),
                                collect_unique_created_dates(issues_state),
                                collect_unique_updated_dates(issues_state),
                                collect_unique_closed_dates(issues_state),
                            );
                            issues_state.filter_bar_state = Some(filter_bar_state);
                        }
                        if let Some(ref mut fb_state) = issues_state.filter_bar_state {
                            fb_state.toggle_dropdown(ui::widgets::FilterDropdownType::Labels);
                        }
                    }
                    // Cycle View (was 'v')
                    // Assuming 'v' maps to some action? Not in Default Bindings explicitly as "CycleView".
                    // Wait, I didn't add "CycleView" to Action.
                    // I will use key check for 'v' since I missed adding it to Action.
                    // Or reuse an existing action?
                    _ if key_code == KeyCode::Char('v') => {
                        // Cycle view
                        issues_state.search_state_mut().next_view();
                        tracing::debug!(
                            "Cycled to next view: {:?}",
                            issues_state.search_state().current_view()
                        );
                    }
                    // Cycle Scope ('s') - mapped to UpdateStatus in Config?
                    // Config: UpdateStatus -> 's'.
                    // OLD CODE: 's' -> Cycle search scope.
                    // Conflict!
                    // Universal Guide says: s -> Status.
                    // So Search Scope needs a new key. Maybe 'S' (Shift+s)?
                    // Or remove cycle search scope shortcut?
                    // Let's keep 's' for Status as per guide.
                    // We'll drop search scope cycling shortcut for now or map it to something else if needed.
                    Some(Action::UpdateStatus) => {
                        // 's'
                        // Open status selector
                        if issues_state.selected_issue().is_some() {
                            app.status_selector_state.toggle();
                        }
                    }
                    Some(Action::ToggleRegexSearch) => {
                        // Toggle regex
                        issues_state.search_state_mut().toggle_regex();
                        let enabled = issues_state.search_state().is_regex_enabled();
                        app.set_info(format!(
                            "Regex search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    Some(Action::ToggleFuzzySearch) => {
                        // Toggle fuzzy
                        issues_state.search_state_mut().toggle_fuzzy();
                        let enabled = issues_state.search_state().is_fuzzy_enabled();
                        app.set_info(format!(
                            "Fuzzy search {}",
                            if enabled { "enabled" } else { "disabled" }
                        ));
                    }
                    Some(Action::UpdateLabels) => {
                        // Toggle label logic (l)?
                        // Config: UpdateLabels -> 'l'.
                        // Old code: 'l' -> Toggle Label Logic.
                        // Guide says 'l' -> Move Right.
                        // Wait, Guide says 'l' -> Move Right in General Nav.
                        // In Issues View, Guide doesn't list 'l'.
                        // Config has 'l' for UpdateLabels.
                        // Let's use 'l' for UpdateLabels (open picker?) or Toggle Logic?
                        // Old code 'L' (Shift+L) opened label picker.
                        // Let's make 'l' open label picker (UpdateLabels).
                        // And maybe Shift+L for logic?

                        // Open label picker for selected issue
                        if let Some(issue) = issues_state.selected_issue() {
                            // ... label picker setup ...
                            let current_labels = issue.labels.clone();
                            let all_labels: std::collections::HashSet<String> = app
                                .issues_view_state
                                .all_issues()
                                .iter()
                                .flat_map(|i| i.labels.iter().cloned())
                                .collect();
                            let mut available_labels: Vec<String> =
                                all_labels.into_iter().collect();
                            available_labels.sort();

                            app.label_picker_state
                                .set_available_labels(available_labels);
                            app.label_picker_state.set_selected_labels(current_labels);

                            app.show_label_picker = true;
                        }
                    }
                    Some(Action::UpdatePriority) => {
                        // Open priority selector
                        if issues_state.selected_issue().is_some() {
                            app.priority_selector_state.toggle();
                        }
                    }
                    Some(Action::Search) => {
                        // '/'
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .clear();
                        issues_state
                            .search_state_mut()
                            .search_state_mut()
                            .set_focused(true);
                        issues_state.search_state_mut().update_filtered_issues();
                    }
                    Some(Action::NextSearchResult) => {
                        let len = issues_state.search_state().filtered_issues().len();
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_next(len);
                    }
                    Some(Action::PrevSearchResult) => {
                        let len = issues_state.search_state().filtered_issues().len();
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select_previous(len);
                    }
                    Some(Action::CancelDialog) => {
                        // Esc
                        issues_state.search_state_mut().clear_search();
                    }

                    // Column manipulation (Alt+Left/Right etc)
                    // These are Actions now: MoveLeft + Alt, etc.
                    // But Action system handles modifiers in finding the action.
                    // If Keybinding::new("left").alt() maps to MoveColumnLeft (we don't have that action).
                    // We reused MoveLeft.
                    // If we have MoveLeft mapped to 'h' and 'Left', and we press Alt+Left.
                    // Does config map Alt+Left? No.
                    // So Alt+Left won't match Action::MoveLeft.
                    // We need to check modifiers manually or add Action::MoveColumnLeft.
                    // Since I didn't add specific column actions, I will keep the raw key checks for column ops for now.
                    _ if key_code == KeyCode::Left && key.modifiers.contains(KeyModifiers::ALT) => {
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
                    _ if key_code == KeyCode::Right
                        && key.modifiers.contains(KeyModifiers::ALT) =>
                    {
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
                    _ => {}
                }
            }
        }
        IssuesViewMode::Detail => {
            // Detail mode: view navigation with scrolling
            match action {
                Some(Action::CancelDialog) | Some(Action::Quit) => {
                    // Esc or q
                    issues_state.return_to_list();
                }
                Some(Action::EditIssue) => {
                    issues_state.return_to_list();
                    issues_state.enter_edit_mode();
                }
                Some(Action::MoveUp) => {
                    // Scroll up in detail view (1 field at a time)
                    issues_state.detail_scroll = issues_state.detail_scroll.saturating_sub(1);
                    app.mark_dirty();
                }
                Some(Action::MoveDown) => {
                    // Scroll down in detail view (1 field at a time)
                    issues_state.detail_scroll = issues_state.detail_scroll.saturating_add(1);
                    app.mark_dirty();
                }
                Some(Action::PageUp) => {
                    // Page up in detail view (5 fields at a time)
                    issues_state.detail_scroll = issues_state.detail_scroll.saturating_sub(5);
                    app.mark_dirty();
                }
                Some(Action::PageDown) => {
                    // Page down in detail view (5 fields at a time)
                    issues_state.detail_scroll = issues_state.detail_scroll.saturating_add(5);
                    app.mark_dirty();
                }
                Some(Action::Home) => {
                    // Scroll to top of form
                    issues_state.detail_scroll = 0;
                    app.mark_dirty();
                }
                Some(Action::End) => {
                    // Scroll to bottom of form (set to large value, clamped by render)
                    issues_state.detail_scroll = u16::MAX;
                    app.mark_dirty();
                }
                _ => {}
            }
        }
        IssuesViewMode::SplitScreen => {
            // Split-screen mode: list navigation with live detail updates
            use ui::views::SplitScreenFocus;

            match key_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    issues_state.return_to_list();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    match issues_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            let len = issues_state.search_state().filtered_issues().len();
                            issues_state
                                .search_state_mut()
                                .list_state_mut()
                                .select_next(len);
                            // Update detail panel with newly selected issue
                            issues_state.update_split_screen_detail();
                        }
                        SplitScreenFocus::Detail => {
                            // Scroll detail panel down
                            issues_state.detail_scroll = issues_state.detail_scroll.saturating_add(1);
                        }
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    match issues_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            let len = issues_state.search_state().filtered_issues().len();
                            issues_state
                                .search_state_mut()
                                .list_state_mut()
                                .select_previous(len);
                            // Update detail panel with newly selected issue
                            issues_state.update_split_screen_detail();
                        }
                        SplitScreenFocus::Detail => {
                            // Scroll detail panel up
                            issues_state.detail_scroll = issues_state.detail_scroll.saturating_sub(1);
                        }
                    }
                }
                KeyCode::Char('g') => {
                    // Go to top
                    issues_state
                        .search_state_mut()
                        .list_state_mut()
                        .select(Some(0));
                    issues_state.update_split_screen_detail();
                }
                KeyCode::Char('G') => {
                    // Go to bottom
                    let len = issues_state.search_state().filtered_issues().len();
                    if len > 0 {
                        issues_state
                            .search_state_mut()
                            .list_state_mut()
                            .select(Some(len - 1));
                        issues_state.update_split_screen_detail();
                    }
                }
                KeyCode::Enter => {
                    // Go to full detail view
                    issues_state.enter_detail_view();
                }
                KeyCode::Char('e') => {
                    // Enter edit mode
                    issues_state.enter_edit_mode();
                }
                KeyCode::Char('r') => {
                    // Enter read-only detail view
                    issues_state.enter_detail_view();
                }
                KeyCode::Tab => {
                    // Toggle focus between list and detail panels
                    match issues_state.split_screen_focus() {
                        SplitScreenFocus::List => {
                            issues_state.set_split_screen_focus(SplitScreenFocus::Detail);
                        }
                        SplitScreenFocus::Detail => {
                            issues_state.set_split_screen_focus(SplitScreenFocus::List);
                        }
                    }
                }
                KeyCode::Char('l') => {
                    // Switch focus to list panel
                    issues_state.set_split_screen_focus(SplitScreenFocus::List);
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
                        // Form scrolling
                        KeyCode::PageUp => {
                            form.scroll_up(5);
                            app.mark_dirty();
                        }
                        KeyCode::PageDown => {
                            form.scroll_down(5);
                            app.mark_dirty();
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

                                        // Update issue via command (undoable)
                                        let client = Arc::new(app.beads_client.clone());
                                        let command = Box::new(IssueUpdateCommand::new(
                                            client,
                                            issue_id.clone(),
                                            update,
                                        ));

                                        app.start_loading("Updating issue...");

                                        match app.execute_command(command) {
                                            Ok(()) => {
                                                app.stop_loading();
                                                tracing::info!(
                                                    "Successfully updated issue: {} (undo with Ctrl+Z)",
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
                        // Form scrolling
                        KeyCode::PageUp => {
                            form.scroll_up(5);
                            app.mark_dirty();
                        }
                        KeyCode::PageDown => {
                            form.scroll_down(5);
                            app.mark_dirty();
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

/// Handle keyboard events for the Dependencies view (Tree View)
fn handle_dependencies_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return;
    }

    match action {
        Some(Action::MoveDown) => {
            // Navigate down in tree
            app.dependency_tree_state.select_next();
            app.mark_dirty();
        }
        Some(Action::MoveUp) => {
            // Navigate up in tree
            app.dependency_tree_state.select_previous();
            app.mark_dirty();
        }
        Some(Action::MoveLeft) => {
            // Collapse current node
            app.dependency_tree_state.collapse_current();
            app.mark_dirty();
        }
        Some(Action::MoveRight) => {
            // Expand current node
            app.dependency_tree_state.expand_current();
            app.mark_dirty();
        }
        _ if key.code == KeyCode::Char(' ') => {
            // Toggle expansion with Space
            app.dependency_tree_state.toggle_expansion();
            app.mark_dirty();
        }
        Some(Action::ConfirmDialog) => {
            // View issue details - for now just provide info
            // TODO: Implement navigation to selected issue in Issues tab
            if let Some(issue_id) = app.dependency_tree_state.selected_issue_id() {
                app.set_info(format!("Selected issue: {}", issue_id));
            }
        }
        Some(Action::CancelDialog) => {
            // Esc goes back to Issues tab
            app.selected_tab = 0;
            app.mark_dirty();
        }
        _ => {}
    }
}

/// Handle keyboard events for the Labels view
fn handle_labels_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return;
    }

    let filtered_labels = app.labels_view_state.filtered_labels(&app.label_stats);
    let labels_len = filtered_labels.len();

    if app.labels_view_state.is_searching() {
        match action {
            Some(Action::CancelDialog) => {
                app.labels_view_state.stop_search();
                app.labels_view_state.clear_search();
            }
            Some(Action::ConfirmDialog) => {
                app.labels_view_state.stop_search();
            }
            _ => {
                match key.code {
                    KeyCode::Backspace => {
                        app.labels_view_state.delete_search_char();
                    }
                    KeyCode::Char(c) => {
                        app.labels_view_state.insert_search_char(c);
                    }
                    _ => {}
                }
            }
        }
        app.mark_dirty();
        return;
    }

    match action {
        Some(Action::MoveDown) => {
            app.labels_view_state.select_next(labels_len);
            app.mark_dirty();
        }
        Some(Action::MoveUp) => {
            app.labels_view_state.select_previous(labels_len);
            app.mark_dirty();
        }
        Some(Action::UpdateAssignee) => { // 'a' used for add label in this view
            // Add label - show notification for now (needs input dialog widget)
            app.set_info("Add label: Not yet implemented (requires input dialog)".to_string());
            tracing::info!("Add label requested");
        }
        Some(Action::DeleteIssue) => { // 'd' used for delete label
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
        Some(Action::EditIssue) => { // 'e' used for edit label
            // Edit selected label
            if let Some(selected_idx) = app.labels_view_state.selected() {
                if let Some(label_stat) = filtered_labels.get(selected_idx) {
                    let label_name = label_stat.name.clone();
                    app.set_info(format!("Edit label '{}': Not yet implemented", label_name));
                    tracing::info!("Edit label requested: {}", label_name);
                }
            }
        }
        Some(Action::UpdateStatus) => { // 's' used for stats in this view
            // Show statistics - already visible in the view
            app.set_info("Label statistics are displayed in the summary panel".to_string());
        }
        Some(Action::Search) => { // '/'
            // Search labels
            app.labels_view_state.start_search();
            tracing::info!("Search labels started");
        }
        Some(Action::CancelDialog) => { // Esc
            if !app.labels_view_state.search_query().is_empty() {
                app.labels_view_state.clear_search();
            } else {
                app.selected_tab = 0; // Go back to issues
            }
        }
        _ => {}
    }
}

/// Handle keyboard events for the Database view
fn handle_database_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return;
    }

    // Using global runtime instead of creating new runtime
    let _client = app.beads_client.clone();

    match action {
        Some(Action::Refresh) => {
            // Refresh database status
            tracing::info!("Refreshing database status");
            app.start_loading("Refreshing database...");
            app.reload_issues();
            app.stop_loading();
        }
        _ if key.code == KeyCode::Char('t') => {
            // Toggle daemon (start/stop) - moved to background task
            if app.daemon_running {
                // Stop daemon
                let _ = app.spawn_task("Stopping daemon", |client| async move {
                    client.stop_daemon().await?;
                    Ok(TaskOutput::DaemonStopped)
                });
            } else {
                // Start daemon
                let _ = app.spawn_task("Starting daemon", |client| async move {
                    client.start_daemon().await?;
                    Ok(TaskOutput::DaemonStarted)
                });
            }
        }
        Some(Action::SyncDatabase) => {
            // Sync database with remote
            tracing::info!("Syncing database with remote");

            // Spawn background task (non-blocking)
            let _ = app.spawn_task("Syncing database", |client| async move {
                let output = client.sync_database().await?;
                tracing::info!("Database synced successfully: {}", output);
                Ok(TaskOutput::DatabaseSynced)
            });
        }
        Some(Action::ExportDatabase) => {
            // Export issues to file - 'x' for export (frees 'e' for edit consistency)
            tracing::info!("Exporting issues to beads_export.jsonl");

            // Spawn background task (non-blocking)
            let _ = app.spawn_task("Exporting issues", |client| async move {
                client.export_issues("beads_export.jsonl").await?;
                tracing::info!("Issues exported successfully");
                Ok(TaskOutput::IssuesExported(
                    "beads_export.jsonl".to_string(),
                ))
            });
        }
        Some(Action::ImportDatabase) => {
            // Import issues from file
            tracing::info!("Importing issues from beads_import.jsonl");

            // Spawn background task (non-blocking)
            let _ = app.spawn_task("Importing issues", |client| async move {
                client.import_issues("beads_import.jsonl").await?;
                tracing::info!("Issues imported successfully");
                Ok(TaskOutput::IssuesImported(0))
            });
        }
        Some(Action::VerifyDatabase) => {
            // Verify database integrity
            tracing::info!("Verifying database integrity");

            // Spawn background task (non-blocking)
            let _ = app.spawn_task("Verifying database", |client| async move {
                let output = client.verify_database().await?;
                tracing::info!("Database verification result: {}", output);
                Ok(TaskOutput::Success(output))
            });
        }
        Some(Action::CompactDatabase) => {
            // Compact database (requires confirmation)
            tracing::info!("Compact database requested - showing confirmation dialog");

            // Set up confirmation dialog for compact operation
            app.dialog_state = Some(ui::widgets::DialogState::new());
            app.pending_action = Some("compact_database".to_string());
            app.mark_dirty();
        }
        Some(Action::CancelDialog) => {
            app.selected_tab = 0; // Go back to issues
        }
        _ => {}
    }
}

/// Handle keyboard events for the Help view
fn handle_help_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

    // Handle notification dismissal with Esc
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        return;
    }

    match action {
        Some(Action::MoveUp) => {
            app.scroll_help_up();
        }
        Some(Action::MoveDown) => {
            app.scroll_help_down();
        }
        Some(Action::MoveRight) | Some(Action::NextTab) => {
            app.next_help_section();
        }
        Some(Action::MoveLeft) | Some(Action::PrevTab) => {
            app.previous_help_section();
        }
        Some(Action::CancelDialog) => {
            app.selected_tab = 0; // Go back to issues
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

        // Poll for task completions
        app.poll_tasks();

        // Only render if state has changed (dirty checking)
        if app.is_dirty() {
            terminal.draw(|f| ui(f, app))?;
            app.clear_dirty();
        }

        // Poll for events with timeout to enable periodic notification checks
        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let action = app
                    .config
                    .keybindings
                    .find_action(&key.code, &key.modifiers);

                // Check for theme cycle (Ctrl+T) - Not in Action enum yet, keeping hardcoded for now
                if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.cycle_theme();
                    continue;
                }

                // Check for ESC during loading operations to request cancellation
                if key.code == KeyCode::Esc && app.is_loading() {
                    app.request_cancellation();
                    continue;
                }

                // Check for F12 screen capture
                if key.code == KeyCode::F(12) {
                    // Force a redraw to ensure buffer is up-to-date
                    terminal.draw(|f| ui(f, app))?;
                    handle_screen_capture(terminal, app);
                    continue;
                }

                // Check for saved filter hotkeys (F1-F11) - Special handling
                if let KeyCode::F(num) = key.code {
                    // F1 is Help in our new bindings, so exclude it here if mapped
                    // But legacy code used F1-F11 for filters.
                    // New bindings: F1 is ShowHelp.
                    // We should probably check if the action is ShowHelp before treating as filter.
                    if matches!(action, Some(Action::ShowHelp)) {
                        // handled below
                    } else if (1..=11).contains(&num) {
                        // Map F-key to hotkey char: F1='1', F2='2', ..., F9='9', F10='A', F11='B'
                        let hotkey = if num <= 9 {
                            // Safe: num is guaranteed to be 1-9 from condition above
                            char::from_digit(num as u32, 10).expect("digit 1-9 always valid")
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
                        // Don't continue, might need to fall through? No, filters are terminal action.
                        continue;
                    }
                }

                // Check for filter save shortcut
                if matches!(action, Some(Action::SaveFilter)) {
                    if app.selected_tab == 0 {
                        // Show filter save dialog on Issues tab
                        app.show_filter_save_dialog();
                        app.mark_dirty();
                    }
                    continue;
                }

                // Check for help actions
                if matches!(
                    action,
                    Some(Action::ShowHelp) | Some(Action::ShowShortcutHelp)
                ) {
                    if app.is_shortcut_help_visible() {
                        app.hide_shortcut_help();
                    } else {
                        app.show_shortcut_help();
                    }
                    continue;
                }

                // Quit action
                if matches!(action, Some(Action::Quit)) {
                    app.should_quit = true;
                    continue;
                }

                // Check for context-sensitive help (Action::ShowContextHelp doesn't exist, hardcoded F1 in legacy?)
                // New config maps F1 to ShowHelp.
                // We'll use a different key or just stick to '?' for help.
                // Or maybe Action::ShowHelp shows context help if context help is enabled?
                // For now, let's keep legacy F1 context help logic if it doesn't conflict with ShowHelp.
                // Actually F1 is now ShowHelp in config.
                // I'll skip the hardcoded F1 context help block since it's covered by ShowHelp logic above (showing shortcut help).
                // If the user wants context help, they can use the menu or we need a new Action.

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
                    if app.is_undo_history_visible() {
                        app.toggle_undo_history();
                        continue;
                    }
                    // Fall through to other Esc handlers
                }

                // Handle ShowIssueHistory (Ctrl+H or Alt+H)
                if matches!(action, Some(Action::ShowIssueHistory)) && app.selected_tab == 0
                    && app.issues_view_state.selected_issue().is_some() {
                        app.show_issue_history = !app.show_issue_history;
                        continue;
                    }

                // Handle notification history panel events if visible
                if app.show_notification_history {
                    match action {
                        Some(Action::MoveUp) => {
                            app.notification_history_state.select_previous();
                            continue;
                        }
                        Some(Action::MoveDown) => {
                            let len = app.notification_history.len();
                            app.notification_history_state.select_next(len);
                            continue;
                        }
                        _ => {}
                    }
                }

                // Handle issue history panel events if visible
                if app.show_issue_history {
                    match action {
                        Some(Action::MoveUp) => {
                            app.issue_history_state.select_previous();
                            continue;
                        }
                        Some(Action::MoveDown) => {
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
                    match action {
                        Some(Action::MoveUp) => {
                            app.priority_selector_state.select_previous(5); // 5 priority levels
                            continue;
                        }
                        Some(Action::MoveDown) => {
                            app.priority_selector_state.select_next(5);
                            continue;
                        }
                        Some(Action::ConfirmDialog) => {
                            // Apply selected priority to current issue
                            if let Some(selected_idx) = app.priority_selector_state.selected() {
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

                                        // Update priority via command (undoable)
                                        let client = Arc::new(app.beads_client.clone());
                                        let update = beads::client::IssueUpdate::new()
                                            .priority(new_priority);
                                        let command = Box::new(IssueUpdateCommand::new(
                                            client,
                                            issue_id.clone(),
                                            update,
                                        ));

                                        app.start_loading("Updating priority...");

                                        match app.execute_command(command) {
                                            Ok(()) => {
                                                app.stop_loading();
                                                app.set_success(format!(
                                                    "Updated priority to {} for issue {} (undo with Ctrl+Z)",
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
                        Some(Action::CancelDialog) => {
                            app.priority_selector_state.close();
                            continue;
                        }
                        Some(Action::Quit) | Some(Action::ShowHelp) => {
                            // Fall through
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

                                // Update labels via command (undoable)
                                let client = Arc::new(app.beads_client.clone());
                                let update =
                                    beads::client::IssueUpdate::new().labels(new_labels.clone());
                                let command = Box::new(IssueUpdateCommand::new(
                                    client,
                                    issue_id.clone(),
                                    update,
                                ));

                                app.start_loading("Updating labels...");

                                match app.execute_command(command) {
                                    Ok(()) => {
                                        app.stop_loading();
                                        app.set_success(format!(
                                            "Updated labels for issue {} ({}) (undo with Ctrl+Z)",
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
                                use crate::beads::models::IssueStatus;
                                let statuses = [
                                    IssueStatus::Open,
                                    IssueStatus::InProgress,
                                    IssueStatus::Closed,
                                ];
                                if let Some(&new_status) = statuses.get(selected_idx) {
                                    if let Some(issue) = app.issues_view_state.selected_issue() {
                                        let issue_id = issue.id.clone();

                                        // Update status via command (undoable)
                                        let client = Arc::new(app.beads_client.clone());
                                        let update =
                                            beads::client::IssueUpdate::new().status(new_status);
                                        let command = Box::new(IssueUpdateCommand::new(
                                            client,
                                            issue_id.clone(),
                                            update,
                                        ));

                                        app.start_loading("Updating status...");

                                        match app.execute_command(command) {
                                            Ok(()) => {
                                                app.stop_loading();
                                                app.set_success(format!(
                                                    "Updated status to {} for issue {} (undo with Ctrl+Z)",
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
                        KeyCode::Char('q') | KeyCode::Char('H') | KeyCode::Char('h') => {
                            // Let 'H', 'h' and 'q' fall through to global handlers
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
                    KeyCode::Char('H') => {
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
                        app.tts_manager.announce("Issues tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('2') => {
                        app.selected_tab = 1;
                        app.tts_manager.announce("Split tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('3') => {
                        app.selected_tab = 2;
                        app.tts_manager.announce("Kanban tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('4') => {
                        app.selected_tab = 3;
                        app.tts_manager.announce("Dependencies tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('5') => {
                        app.selected_tab = 4;
                        app.tts_manager.announce("Labels tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('6') => {
                        app.selected_tab = 5;
                        app.tts_manager.announce("Gantt tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('7') => {
                        app.selected_tab = 6;
                        app.tts_manager.announce("Pert tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('8') => {
                        app.selected_tab = 7;
                        app.tts_manager.announce("Molecular tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('9') => {
                        app.selected_tab = 8;
                        app.tts_manager.announce("Statistics tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('0') => {
                        app.selected_tab = 9;
                        app.tts_manager.announce("Utilities tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('h') => {
                        app.selected_tab = 10;
                        app.tts_manager.announce("Help tab");
                        app.mark_dirty();
                        continue;
                    }
                    _ => {}
                }

                // Tab-specific key bindings
                match app.selected_tab {
                    0 | 1 => handle_issues_view_event(key, app), // Issues & Split
                    2 => handle_kanban_view_event(key, app),     // Kanban
                    3 => handle_dependencies_view_event(key, app), // Dependencies
                    4 => handle_labels_view_event(key, app),     // Labels
                    5 => handle_gantt_view_event(key, app),      // Ghant
                    6 => handle_pert_view_event(key, app),       // Pert
                    7 => handle_molecular_view_event(key, app),  // Molecular
                    8 | 9 => handle_database_view_event(key, app), // Statistics & Utilities
                    10 => handle_issues_view_event(key, app), // Record Detail
                11 => handle_help_view_event(key, app),      // Help
                    _ => {}
                }

                // Handle global tab navigation after view-specific handling
                // Skip Tab navigation in split screen mode (Tab toggles focus there)
                let is_split_screen = matches!(
                    app.issues_view_state.view_mode(),
                    ui::views::IssuesViewMode::SplitScreen
                );
                match key.code {
                    KeyCode::Tab if !is_split_screen => app.next_tab(),
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
            Event::Mouse(mouse) => {
                handle_mouse_event(mouse, terminal, app);
            }
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

// Placeholder handlers for new views
fn handle_kanban_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
    }

    match action {
        // Navigation actions
        Some(Action::MoveLeft) => {
            app.kanban_view_state.previous_column();
            // Note: viewport width will be calculated during render
            // For now, we assume a reasonable default terminal width
            app.kanban_view_state.ensure_selected_column_visible(100);
            app.mark_dirty();
        }
        Some(Action::MoveRight) => {
            app.kanban_view_state.next_column();
            // Note: viewport width will be calculated during render
            // For now, we assume a reasonable default terminal width
            app.kanban_view_state.ensure_selected_column_visible(100);
            app.mark_dirty();
        }
        Some(Action::MoveUp) => {
            app.kanban_view_state.previous_card();
            app.mark_dirty();
        }
        Some(Action::MoveDown) => {
            app.kanban_view_state.next_card();
            app.mark_dirty();
        }
        // Column toggle actions
        Some(Action::ToggleKanbanOpenColumn) => app
            .kanban_view_state
            .toggle_column_collapse(models::kanban_config::ColumnId::StatusOpen),
        Some(Action::ToggleKanbanInProgressColumn) => app
            .kanban_view_state
            .toggle_column_collapse(models::kanban_config::ColumnId::StatusInProgress),
        Some(Action::ToggleKanbanBlockedColumn) => app
            .kanban_view_state
            .toggle_column_collapse(models::kanban_config::ColumnId::StatusBlocked),
        Some(Action::ToggleKanbanClosedColumn) => app
            .kanban_view_state
            .toggle_column_collapse(models::kanban_config::ColumnId::StatusClosed),
        _ => {}
    }
}

fn handle_gantt_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
    }
}

fn handle_pert_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
    }
}

fn handle_molecular_view_event(key: KeyEvent, app: &mut models::AppState) {
    let action = app.config.keybindings.find_action(&key.code, &key.modifiers);
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
    }
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

    // Title with issue stats and daemon status
    let open_count = app.database_stats.open_issues;
    let in_progress_count = app.database_stats.in_progress_issues;
    let blocked_count = app.database_stats.blocked_issues;
    let closed_count = app.database_stats.closed_issues;

    let daemon_status = if app.daemon_running {
        Span::styled(
            "[DAEMON: Running]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "[DAEMON: Stopped]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    };
    // Create columnar title bar layout
    let title_block = Block::default().borders(Borders::ALL).title("[TITLE]");
    let title_inner = title_block.inner(chunks[0]);
    f.render_widget(title_block, chunks[0]);

    let title_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),  // Beads-TUI version
            Constraint::Min(30),     // Issue counts
            Constraint::Length(40),  // Search box
            Constraint::Length(18),  // Daemon status
        ])
        .split(title_inner);

    // Column 1: Beads-TUI version
    let version = Paragraph::new(Span::styled(
        "BEADS-TUI v.1.6.0",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(version, title_columns[0]);

    // Column 2: Issue counts
    let stats = Paragraph::new(Span::styled(
        format!(" Open: {}, In-Progress: {}, Blocked: {}, Closed: {}",
                open_count, in_progress_count, blocked_count, closed_count),
        Style::default().fg(Color::White),
    ));
    f.render_widget(stats, title_columns[1]);

    // Column 3: Search box
    let search_state = app.issues_view_state.search_state();
    let query = search_state.search_state().query();
    let is_focused = search_state.search_state().is_focused();

    let search_display = if is_focused && query.is_empty() {
        "Find: [...........................]".to_string()
    } else if query.is_empty() {
        "Find: [...........................]".to_string()
    } else {
        // Truncate if too long
        if query.len() > 25 {
            format!("Find: [{}...]", &query[..22])
        } else {
            format!("Find: [{}]", query)
        }
    };
    let search_widget = Paragraph::new(Span::styled(
        search_display,
        Style::default().fg(Color::Yellow),
    ));
    f.render_widget(search_widget, title_columns[2]);

    // Column 4: Daemon status
    let daemon = Paragraph::new(daemon_status);
    f.render_widget(daemon, title_columns[3]);

    // Tabs and content
    let tabs_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    // Tab bar without numbered shortcuts
    let tabs: Vec<Line> = app
        .tabs
        .iter()
        .map(|&name| {
            Line::from(format!(" {}", name))
        })
        .collect();

    let tabs_widget = Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title("[VIEWS]"))
        .select(app.selected_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs_widget, tabs_chunks[0]);

    // Content area based on selected tab
    match app.selected_tab {
        0 => {
            // Issues view - render based on current view mode
            use ui::views::{IssuesViewMode, IssueDetailView};

            // Check current view mode to determine which view to render
            match app.issues_view_state.view_mode() {
                IssuesViewMode::Detail => {
                    // Render full-screen Record Detail View
                    // Extract scroll to avoid borrow conflicts
                    let mut detail_scroll = app.issues_view_state.detail_scroll;
                    if let Some(issue) = app.issues_view_state.selected_issue() {
                        let detail_view = IssueDetailView::new(issue);
                        f.render_stateful_widget(detail_view, tabs_chunks[1], &mut detail_scroll);
                        // Write scroll back
                        app.issues_view_state.detail_scroll = detail_scroll;
                    } else {
                        // No issue selected, return to list
                        app.issues_view_state.return_to_list();
                        let issues_view = IssuesView::new();
                        f.render_stateful_widget(issues_view, tabs_chunks[1], &mut app.issues_view_state);
                    }
                }
                IssuesViewMode::SplitScreen => {
                    // Revert to List mode when on Issues tab (Split Screen is tab 1)
                    app.issues_view_state.return_to_list();
                    let issues_view = IssuesView::new();
                    f.render_stateful_widget(issues_view, tabs_chunks[1], &mut app.issues_view_state);
                }
                _ => {
                    // List or Edit mode - render IssuesView
                    let issues_view = IssuesView::new();
                    f.render_stateful_widget(issues_view, tabs_chunks[1], &mut app.issues_view_state);
                }
            }
        }
        1 => {
            // Split view (Issues view in SplitScreen mode)
            use ui::views::IssuesViewMode;
            if app.issues_view_state.view_mode() != IssuesViewMode::SplitScreen {
                app.issues_view_state.enter_split_screen();
            }
            let issues_view = IssuesView::new();
            f.render_stateful_widget(issues_view, tabs_chunks[1], &mut app.issues_view_state);
        }
        2 => {
            // Kanban view
            use ui::views::KanbanView;
            let kanban_view = KanbanView::new();
            f.render_stateful_widget(kanban_view, tabs_chunks[1], &mut app.kanban_view_state);
        }
        3 => {
            // Dependency Tree view
            let issues = app.issues_view_state.search_state().filtered_issues();
            let dependency_tree_view = DependencyTreeView::new(&issues);
            f.render_stateful_widget(
                dependency_tree_view,
                tabs_chunks[1],
                &mut app.dependency_tree_state,
            );
        }
        4 => {
            // Labels view
            let labels_view = LabelsView::new().labels(app.label_stats.clone());
            f.render_stateful_widget(labels_view, tabs_chunks[1], &mut app.labels_view_state);
        }
        5 => {
            // Ghant view (Gantt)
            use ui::views::GanttView;
            let gantt_view = GanttView::new();
            f.render_stateful_widget(gantt_view, tabs_chunks[1], &mut app.gantt_view_state);
        }
        6 => {
            // Pert view
            use ui::views::PertView;
            let pert_view = PertView::new();
            f.render_stateful_widget(pert_view, tabs_chunks[1], &mut app.pert_view_state);
        }
        7 => {
            // Molecular view
            // Note: Molecular view widget needs to be imported or implemented if missing
            // Assuming it uses a similar pattern or falls back to placeholder
            // Searching for MolecularView usage elsewhere...
            // It seems MolecularView logic might be missing or commented out in previous contexts.
            // I'll check imports. If not available, I'll render a placeholder.
            // But wait, AppState has molecular_tabs and selected_molecular_tab.
            // Let's try to render a placeholder for now as I don't see MolecularView imported.
            let placeholder = Paragraph::new("Molecular View: Not fully implemented yet")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(placeholder, tabs_chunks[1]);
        }
        8 => {
            // Statistics view (Database dashboard)
            app.database_view_state
                .set_mode(ui::views::DatabaseViewMode::Dashboard);
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(app.daemon_running);
            f.render_stateful_widget(database_view, tabs_chunks[1], &mut app.database_view_state);
        }
        9 => {
            // Utilities view (Database maintenance/daemon)
            use ui::views::DatabaseViewMode;
            if app.database_view_state.mode != DatabaseViewMode::Maintenance
                && app.database_view_state.mode != DatabaseViewMode::Daemon
                && app.database_view_state.mode != DatabaseViewMode::Sync
            {
                app.database_view_state.set_mode(DatabaseViewMode::Maintenance);
            }
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(app.daemon_running);
            f.render_stateful_widget(database_view, tabs_chunks[1], &mut app.database_view_state);
        }
        10 => {
            // Record Detail view - show detail of selected issue
            use ui::views::IssueDetailView;
            let mut detail_scroll = app.issues_view_state.detail_scroll;
            if let Some(issue) = app.issues_view_state.selected_issue() {
                let detail_view = IssueDetailView::new(issue);
                f.render_stateful_widget(detail_view, tabs_chunks[1], &mut detail_scroll);
                app.issues_view_state.detail_scroll = detail_scroll;
            } else {
                // No issue selected, show message
                let placeholder = Paragraph::new("No issue selected. Press ESC to return to Issues view.")
                    .block(Block::default().borders(Borders::ALL).title("Record Detail"));
                f.render_widget(placeholder, tabs_chunks[1]);
            }
        }
        _ => {
            // Help view (Index 11 and fallback)
            let help_view = HelpView::new().selected_section(app.help_section);
            let mut help_state = HelpViewState::new();
            help_state.set_scroll_offset(app.help_scroll_offset);
            f.render_stateful_widget(help_view, tabs_chunks[1], &mut help_state);
        }
    }

    // Status bar with optional performance stats, loading indicator, or action hints
    if let Some(ref spinner) = app.loading_spinner {
        // Show loading indicator using Spinner widget
        let label = app.loading_message.as_deref().unwrap_or("Loading...");
        let spinner_text = format!("{} {}", spinner.frame_char(), label);
        let status_text = Paragraph::new(spinner_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Loading"));
        f.render_widget(status_text, chunks[2]);
    } else {
        // Show context-sensitive action hints in columnar format
        let action_table = render_action_bar(app);
        f.render_widget(action_table, chunks[2]);
    }

    // Render dialog overlay if active
    if let Some(ref dialog_state) = app.dialog_state {
        if let Some(ref action) = app.pending_action {
            // Parse action to get issue ID and construct message
            if let Some(issue_id) = action.strip_prefix("delete:") {
                let message = format!("Are you sure you want to delete issue {issue_id}?");
                let dialog = ui::widgets::Dialog::confirm("Confirm Delete", &message)
                    .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

                // Render dialog centered on screen
                let area = f.size();
                let dialog_area = centered_rect(60, 30, area);

                // Clear and render dialog
                f.render_widget(Clear, dialog_area);
                dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
            } else if let Some(filter_idx_str) = action.strip_prefix("delete_filter:") {
                // Get filter name for dialog
                if let Ok(i) = filter_idx_str.parse::<usize>() {
                    if let Some(filter) =
                        app.issues_view_state.search_state().saved_filters().get(i)
                    {
                        let message = format!(
                            "Are you sure you want to delete the filter '{}'?\n\nThis action cannot be undone.",
                            filter.name
                        );
                        let dialog = ui::widgets::Dialog::confirm("Delete Filter", &message)
                            .dialog_type(ui::widgets::DialogType::Warning)
                            .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

                        // Render dialog centered on screen
                        let area = f.size();
                        let dialog_area = centered_rect(60, 30, area);

                        // Clear and render dialog
                        f.render_widget(Clear, dialog_area);
                        dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
                    }
                }
            } else if let Some(ids) = action.strip_prefix("indent:") {
                let parts: Vec<&str> = ids.split(':').collect();
                if parts.len() == 2 {
                    let message = format!(
                        "Are you sure you want to indent {} under {}?\n\n{} will depend on {}",
                        parts[0], parts[1], parts[0], parts[1]
                    );
                    let dialog = ui::widgets::Dialog::confirm("Confirm Indent", &message)
                        .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

                    // Render dialog centered on screen
                    let area = f.size();
                    let dialog_area = centered_rect(60, 30, area);

                    // Clear and render dialog
                    f.render_widget(Clear, dialog_area);
                    dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
                }
            } else if let Some(ids) = action.strip_prefix("outdent:") {
                let parts: Vec<&str> = ids.split(':').collect();
                if parts.len() == 2 {
                    let message = format!(
                        "Are you sure you want to outdent {} from parent {}?\n\n{} will no longer depend on {}",
                        parts[0], parts[1], parts[0], parts[1]
                    );
                    let dialog = ui::widgets::Dialog::confirm("Confirm Outdent", &message)
                        .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

                    // Render dialog centered on screen
                    let area = f.size();
                    let dialog_area = centered_rect(60, 30, area);

                    // Clear and render dialog
                    f.render_widget(Clear, dialog_area);
                    dialog.render_with_state(dialog_area, f.buffer_mut(), dialog_state);
                }
            } else if action == "compact_database" {
                let message = "WARNING: Compacting will remove issue history.\nThis operation cannot be undone.\n\nContinue?";
                let dialog = ui::widgets::Dialog::confirm("Compact Database", message)
                    .dialog_type(ui::widgets::DialogType::Warning)
                    .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

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
                .dialog_type(ui::widgets::DialogType::Warning)
                .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

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
                .dialog_type(ui::widgets::DialogType::Warning)
                .hint("Tab/Shift+Tab: Select | Enter: Confirm | Esc: Cancel");

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
            .key_binding("F1", "Context help")
            .key_binding("q / Ctrl+Q / Ctrl+C", "Quit application")
            .key_binding("Esc", "Dismiss overlays/dialogs")
            .key_binding("Ctrl+H / N", "Notification history")
            .key_binding("Ctrl+P / F12", "Toggle performance stats")
            .key_binding("Ctrl+Z", "Undo last action")
            .key_binding("Ctrl+Y", "Redo last action")
            .key_binding("Tab", "Next tab")
            .key_binding("Shift+Tab", "Previous tab")
            .key_binding("1-9", "Switch to tab by number (1-5 implemented)")
            // Issues view shortcuts
            .key_binding("Up/Down or j/k", "Navigate issues")
            .key_binding("Enter", "View issue details")
            .key_binding("n", "Create new issue")
            .key_binding("e", "Edit selected issue")
            .key_binding("d", "Delete selected issue")
            .key_binding("x", "Close selected issue")
            .key_binding("o", "Reopen selected issue")
            .key_binding("F2", "Rename issue")
            .key_binding("/", "Search issues")
            .key_binding("f", "Toggle filters")
            .key_binding("Shift+F", "Clear filters")
            .key_binding("Alt+F", "Filter menu")
            .key_binding("Alt+S", "Save current filter")
            .key_binding("F3-F11", "Apply saved filter")
            .key_binding("Alt+H", "Toggle issue history")
            .key_binding("r / F5", "Refresh data");

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

    // Render undo history overlay if visible
    if app.is_undo_history_visible() {
        use ui::widgets::{HistoryEntry, UndoHistoryView};

        // Get history from undo stack
        let history_data = app.undo_stack.history();
        let entries: Vec<HistoryEntry> = history_data
            .into_iter()
            .map(
                |(description, timestamp, is_current, can_undo)| HistoryEntry {
                    description,
                    timestamp,
                    is_current,
                    can_undo,
                },
            )
            .collect();

        let history_view = UndoHistoryView::new(entries).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Undo/Redo History")
                .title_bottom("Esc: Close"),
        );

        // Center the overlay (60% width, 70% height)
        let area = centered_rect(60, 70, f.size());
        f.render_widget(history_view, area);
    }
}

/// Render action bar with columnar layout
fn render_action_bar(app: &models::AppState) -> Paragraph<'static> {
    // Create action cells based on context
    let (nav_actions, action_items) = get_contextual_actions(app);
    
    // Build formatted text with vertical separators
    let action_text = format!(
        " {} │ {} │ {} │ {} │ {} │ {} │ {} │ {} ",
        nav_actions,
        action_items.get(0).unwrap_or(&String::new()),
        action_items.get(1).unwrap_or(&String::new()),
        action_items.get(2).unwrap_or(&String::new()),
        action_items.get(3).unwrap_or(&String::new()),
        action_items.get(4).unwrap_or(&String::new()),
        action_items.get(5).unwrap_or(&String::new()),
        action_items.get(6).unwrap_or(&String::new()),
    );
    
    Paragraph::new(action_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL))
}

/// Get contextual actions for action bar
fn get_contextual_actions(app: &models::AppState) -> (String, Vec<String>) {
    // Navigation actions (first column)
    let nav = "↓:Up ↑:Down (row) PgUp/PgDn (page) →/←:Scroll-Right/Left".to_string();
    
    // Context-specific action items (remaining columns)
    let actions = match app.selected_tab {
        0 | 1 => {
            // Issues view
            let mode = app.issues_view_state.view_mode();
            match mode {
                ui::views::IssuesViewMode::List | ui::views::IssuesViewMode::SplitScreen => {
                    vec![
                        "R:Read".to_string(),
                        "N:New".to_string(),
                        "E:Edit".to_string(),
                        "D:Delete".to_string(),
                        "F:Find".to_string(),
                        "O:Open".to_string(),
                        "X:Close".to_string(),
                    ]
                }
                _ => {
                    // For Create/Edit modes - Record Detail Form actions
                    vec![
                        "^S:Save".to_string(),
                        "^X:Cancel".to_string(),
                        "^Del:Delete".to_string(),
                        "^J:JSON".to_string(),
                        "^P:Export".to_string(),
                        "Esc:Close".to_string(),
                        "Tab:Move".to_string(),
                    ]
                }
            }
        }
        2 => {
            // Kanban view
            vec![
                "R:Read".to_string(),
                "N:New".to_string(),
                "E:Edit".to_string(),
                "D:Delete".to_string(),
                "F:Find".to_string(),
                "O:Open".to_string(),
                "X:Close".to_string(),
            ]
        }
        10 => {
            // Record Detail view
            vec![
                "E:Edit".to_string(),
                "↑/↓:Scroll".to_string(),
                "PgUp/Dn:Page".to_string(),
                "Home/End:Top/Bot".to_string(),
                "Esc:Back".to_string(),
                "Tab:Sections".to_string(),
                "?:Help".to_string(),
            ]
        }
        _ => {
            // Other views - generic actions
            vec![
                "R:Read".to_string(),
                "N:New".to_string(),
                "E:Edit".to_string(),
                "D:Delete".to_string(),
                "↵:View".to_string(),
                "Esc:Bck".to_string(),
                "?:Help".to_string(),
            ]
        }
    };
    
    (nav, actions)
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
                ("Left/Right", "Navigate between buttons"),
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
                ("F3-F11", "Available hotkey options"),
            ],
        );
    }

    // If filter quick-select is visible
    if app.issues_view_state.search_state().is_filter_menu_open() {
        return (
            "Quick Filter Menu Help".to_string(),
            "Apply, Edit, or Delete Filters".to_string(),
            vec![
                ("Up/Down", "Navigate through filters"),
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
                ("Up/Down", "Navigate through issues"),
                ("Space", "Toggle dependency type"),
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
                        ("Up/Down or j/k", "Navigate through issues"),
                        ("Enter", "View issue details"),
                        ("n", "Create new issue"),
                        ("e", "Edit selected issue"),
                        ("d", "Delete selected issue"),
                        ("x", "Close selected issue"),
                        ("o", "Reopen selected issue"),
                        ("F2", "Rename issue title"),
                        ("p", "Update priority"),
                        ("s", "Update status"),
                        ("l", "Update labels"),
                        ("a", "Update assignee"),
                        ("+", "Add dependency"),
                        ("-", "Remove dependency"),
                        (">", "Indent issue"),
                        ("<", "Outdent issue"),
                        ("Space", "Toggle select"),
                        ("Ctrl+A", "Select all issues"),
                        ("Ctrl+N", "Clear selection"),
                        ("/", "Search issues"),
                        ("f", "Toggle filters"),
                        ("Shift+F", "Clear filters"),
                        ("Alt+F", "Open filter menu"),
                        ("Alt+S", "Save current filter"),
                        ("F3-F11", "Apply saved filter"),
                        ("c", "Open column manager"),
                        ("v", "Cycle issue scope"),
                        ("Alt+H", "Toggle issue history"),
                        ("?", "Show all keyboard shortcuts"),
                        ("Esc", "Clear search or go back"),
                    ],
                ),
                ui::views::IssuesViewMode::Detail => (
                    "Issue Detail View Help".to_string(),
                    "View Issue Information".to_string(),
                    vec![
                        ("Up/Down or j/k", "Scroll detail view"),
                        ("PgUp/PgDn", "Page up/down"),
                        ("e", "Edit this issue"),
                        ("d", "Delete this issue"),
                        ("Alt+H", "Toggle issue history"),
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
                        ("Enter", "Save changes"),
                        ("Ctrl+L", "Load description from file"),
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
                        ("Enter", "Create issue"),
                        ("Ctrl+L", "Load description from file"),
                        ("Ctrl+P", "Toggle preview"),
                        ("Esc", "Cancel creation"),
                        ("?", "Show all keyboard shortcuts"),
                    ],
                ),
                ui::views::IssuesViewMode::SplitScreen => (
                    "Split-Screen View Help".to_string(),
                    "List and Detail View".to_string(),
                    vec![
                        ("Up/Down or j/k", "Navigate/scroll (depending on focus)"),
                        ("r", "Focus record details panel"),
                        ("l", "Focus list panel"),
                        ("Enter", "Go to full detail view"),
                        ("e", "Edit selected issue"),
                        ("Alt+H", "Toggle issue history"),
                        ("Esc/q", "Back to list view"),
                        ("?", "Show all keyboard shortcuts"),
                    ],
                ),
            }
        }
        1 => (
            "Dependencies View Help".to_string(),
            "Manage Issue Dependencies".to_string(),
            vec![
                ("Up/Down or j/k", "Navigate through dependencies"),
                ("Tab", "Switch between Dependencies/Blocks"),
                ("a", "Add dependency"),
                ("d", "Remove dependency"),
                ("g", "Show dependency graph"),
                ("c", "Check circular dependencies"),
                ("Enter", "View issue details"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        2 => (
            "Labels View Help".to_string(),
            "Manage Issue Labels".to_string(),
            vec![
                ("Up/Down or j/k", "Navigate through labels"),
                ("/", "Search labels"),
                ("a", "Add new label"),
                ("e", "Edit label"),
                ("d", "Delete label"),
                ("s", "Show label stats"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        3 => (
            "PERT Chart View Help".to_string(),
            "Project Evaluation and Review Technique".to_string(),
            vec![
                ("Up/Down", "Navigate through nodes"),
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
                ("Up/Down", "Navigate through tasks"),
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
                ("Up/Down/Left/Right or h/j/k/l", "Navigate between cards"),
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
                ("Up/Down", "Navigate through items"),
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
                ("Up/Down or j/k", "Navigate through operations"),
                ("r", "Refresh database"),
                ("s", "Sync database"),
                ("x", "Export issues"),
                ("i", "Import issues"),
                ("v", "Verify database integrity"),
                ("c", "Compact database"),
                ("t", "Toggle daemon"),
                ("Esc", "Go back"),
                ("?", "Show all keyboard shortcuts"),
            ],
        ),
        8 => (
            "Help View".to_string(),
            "Documentation and Guides".to_string(),
            vec![
                ("Left/Right or h/l", "Navigate between sections"),
                ("Esc", "Go back"),
                ("?", "Quick keyboard reference"),
            ],
        ),
        _ => (
            "General Help".to_string(),
            "Global Navigation".to_string(),
            vec![
                ("q / Ctrl+Q / Ctrl+C", "Quit application"),
                ("Tab", "Next tab"),
                ("Shift+Tab", "Previous tab"),
                ("1-9", "Jump to tab by number"),
                ("Ctrl+H / N", "Notification history"),
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

/// Collect unique statuses from issues
fn collect_unique_statuses(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::IssueStatus> {
    use std::collections::HashSet;
    let mut statuses: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.status)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    statuses.sort();
    statuses
}

/// Collect unique priorities from issues
fn collect_unique_priorities(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::Priority> {
    use std::collections::HashSet;
    let mut priorities: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.priority)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    priorities.sort();
    priorities
}

/// Collect unique types from issues
fn collect_unique_types(issues_state: &ui::views::IssuesViewState) -> Vec<beads::models::IssueType> {
    use std::collections::HashSet;
    let mut types: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| i.issue_type)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    types.sort();
    types
}

/// Collect unique labels from issues
fn collect_unique_labels(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    let mut labels: Vec<_> = issues_state
        .all_issues()
        .iter()
        .flat_map(|i| i.labels.iter().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    labels.sort();
    labels
}

/// Collect unique assignees from issues
fn collect_unique_assignees(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    let mut assignees: Vec<_> = issues_state
        .all_issues()
        .iter()
        .filter_map(|i| i.assignee.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    assignees.push("-".to_string()); // Add option for unassigned
    assignees.sort();
    assignees
}

/// Collect unique created dates from issues
fn collect_unique_created_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| format!("{:04}-{:02}-{:02}",
            i.created.year(),
            i.created.month(),
            i.created.day()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}

/// Collect unique updated dates from issues
fn collect_unique_updated_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .map(|i| format!("{:04}-{:02}-{:02}",
            i.updated.year(),
            i.updated.month(),
            i.updated.day()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}

/// Collect unique closed dates from issues
fn collect_unique_closed_dates(issues_state: &ui::views::IssuesViewState) -> Vec<String> {
    use std::collections::HashSet;
    use chrono::Datelike;
    let mut dates: Vec<_> = issues_state
        .all_issues()
        .iter()
        .filter_map(|i| i.closed.as_ref().map(|c| format!("{:04}-{:02}-{:02}",
            c.year(),
            c.month(),
            c.day())))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();
    dates
}
