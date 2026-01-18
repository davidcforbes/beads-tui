use beads_tui::*;
use anyhow::Result;
use clap::Parser;
use beads_tui::config::Action;
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
use ui::views::{
    DatabaseView, DatabaseViewState, DependenciesViewState, DependencyTreeView,
    FormulaBrowserState, GanttView, GanttViewState, HelpView, HelpViewState,
    IssuesView, IssuesViewState, KanbanView, KanbanViewState, LabelsView, LabelsViewState,
    PertView, PertViewState, ViewEventHandler
};
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

    // Initialize filter_bar_state for fullscreen Issues View (tab 0)
    // This ensures it exists before the event loop starts
    if app.selected_tab == 0 && app.issues_view_state.filter_bar_state.is_none() {
        let filter_bar_state = ui::widgets::FilterBarState::new(
            beads_tui::helpers::collect_unique_statuses(&app.issues_view_state),
            beads_tui::helpers::collect_unique_priorities(&app.issues_view_state),
            beads_tui::helpers::collect_unique_types(&app.issues_view_state),
            beads_tui::helpers::collect_unique_labels(&app.issues_view_state),
            beads_tui::helpers::collect_unique_assignees(&app.issues_view_state),
            beads_tui::helpers::collect_unique_created_dates(&app.issues_view_state),
            beads_tui::helpers::collect_unique_updated_dates(&app.issues_view_state),
            beads_tui::helpers::collect_unique_closed_dates(&app.issues_view_state),
        );
        app.issues_view_state.filter_bar_state = Some(filter_bar_state);
    }

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
            use beads_tui::ui::views::ViewEventHandler;
            let _ = match app.selected_tab {
                0 | 1 => IssuesViewState::handle_key_event(app, key),
                2 => KanbanViewState::handle_key_event(app, key),
                3 => DependenciesViewState::handle_key_event(app, key),
                4 => LabelsViewState::handle_key_event(app, key),
                5 => GanttViewState::handle_key_event(app, key),
                6 => PertViewState::handle_key_event(app, key),
                7 => FormulaBrowserState::handle_key_event(app, key),
                8 | 9 => DatabaseViewState::handle_key_event(app, key),
                10 => IssuesViewState::handle_key_event(app, key), // Record Detail
                11 => HelpViewState::handle_key_event(app, key),
                _ => false,
            };
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
            use beads_tui::ui::views::ViewEventHandler;
            let _ = match app.selected_tab {
                0 | 1 => IssuesViewState::handle_key_event(app, key),
                2 => KanbanViewState::handle_key_event(app, key),
                3 => DependenciesViewState::handle_key_event(app, key),
                4 => LabelsViewState::handle_key_event(app, key),
                5 => GanttViewState::handle_key_event(app, key),
                6 => PertViewState::handle_key_event(app, key),
                7 => FormulaBrowserState::handle_key_event(app, key),
                8 | 9 => DatabaseViewState::handle_key_event(app, key),
                10 => IssuesViewState::handle_key_event(app, key), // Record Detail
                11 => HelpViewState::handle_key_event(app, key),
                _ => false,
            };
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
    // FULLSCREEN ISSUES VIEW (tab 0) - New layout:
    // Row 0: Title bar
    // Row 1: Tab bar
    // Rows 2-3: FILTERS section
    // Row 4+: Issues table
    // Last row: Action bar
    if app.selected_tab == 0 {
        // Handle tab bar clicks (row 1)
        if row == 1 {
            // Tab bar layout: " Issues | Record | Split | ..."
            // Parse tab positions: leading space + tab name + trailing space + "|"
            // Pattern for each tab: " TabName " + "|" (except last has no "|")
            let mut current_col = 1u16; // Start after leading space (where first tab name begins)

            for (i, &tab_name) in app.tabs.iter().enumerate() {
                // Each tab occupies: name.len() characters starting at current_col
                let tab_end = current_col + tab_name.len() as u16;

                // Check if click is within this tab's name
                if col >= current_col && col < tab_end {
                    tracing::info!("Fullscreen tab {} '{}' clicked at col {}", i, tab_name, col);

                    // "Quit" tab should quit the app
                    if tab_name == "Quit" {
                        app.should_quit = true;
                    } else {
                        app.selected_tab = i;
                        app.tts_manager.announce(&format!("{} tab", tab_name));
                    }
                    app.mark_dirty();
                    return;
                }

                // Move to next tab position:
                // +1 for trailing space after tab name
                // +1 for "|" separator
                // +1 for leading space before next tab
                // = +3 total
                current_col = tab_end + 3;
            }
            return;
        }

        // Handle filter bar clicks (row 3 - the second line of FILTERS section)
        if row == 3 && app.issues_view_state.filter_bar_state.is_some() {
            // Filter bar layout: "1:Status [ALL ▼] | 2:Type [ALL ▼] | ..."
            // Approximate click zones for each filter
            if col < 20 {
                // Status filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Status);
                    app.mark_dirty();
                }
            } else if col >= 20 && col < 40 {
                // Type filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Type);
                    app.mark_dirty();
                }
            } else if col >= 40 && col < 63 {
                // Priority filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Priority);
                    app.mark_dirty();
                }
            } else if col >= 63 && col < 85 {
                // Labels filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Labels);
                    app.mark_dirty();
                }
            } else if col >= 85 && col < 107 {
                // Created filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Created);
                    app.mark_dirty();
                }
            } else if col >= 107 && col < 129 {
                // Updated filter
                if let Some(ref mut filter_bar_state) = app.issues_view_state.filter_bar_state {
                    filter_bar_state.toggle_dropdown(ui::widgets::FilterDropdownType::Updated);
                    app.mark_dirty();
                }
            }
            return;
        }

        // For other rows in fullscreen view, continue to existing handling below
    }

    // OLD LAYOUT (for other tabs):
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
        // Calculate row offset based on layout type
        let first_data_row = if app.selected_tab == 0 {
            // FULLSCREEN Issues View layout:
            // Row 0: Title bar
            // Row 1: Tab bar
            // Rows 2-3: FILTERS section
            // Row 4: [ISSUES] header
            // Row 5: Table header (column names)
            // Row 6+: Table data rows
            6
        } else {
            // Split view layout (tab 1): use old calculations
            let filter_bar_visible = app.issues_view_state.filter_bar_state.is_some();
            if filter_bar_visible { 11 } else { 9 }
        };

        if row >= first_data_row {
            let item_index = (row - first_data_row) as usize;
            let filtered_issues_len = app.issues_view_state.search_state().filtered_issues().len();

            if item_index < filtered_issues_len {
                tracing::info!("List item {} clicked at row {} (first_data_row: {}, tab: {})",
                    item_index, row, first_data_row, app.selected_tab);
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
    use ui::views::ViewEventHandler;
    let _ = match app.selected_tab {
        0 | 1 => IssuesViewState::handle_key_event(app, key),
        2 => KanbanViewState::handle_key_event(app, key),
        3 => DependenciesViewState::handle_key_event(app, key),
        4 => LabelsViewState::handle_key_event(app, key),
        5 => GanttViewState::handle_key_event(app, key),
        6 => PertViewState::handle_key_event(app, key),
        7 => FormulaBrowserState::handle_key_event(app, key),
        8 | 9 => DatabaseViewState::handle_key_event(app, key),
        10 => IssuesViewState::handle_key_event(app, key), // Record Detail
        11 => HelpViewState::handle_key_event(app, key),
        _ => false,
    };
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
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 0;
                            app.tts_manager.announce("Issues tab");
                            // Initialize filter_bar_state for fullscreen Issues View
                            if app.issues_view_state.filter_bar_state.is_none() {
                                let filter_bar_state = ui::widgets::FilterBarState::new(
                                    beads_tui::helpers::collect_unique_statuses(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_priorities(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_types(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_labels(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_assignees(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_created_dates(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                    beads_tui::helpers::collect_unique_closed_dates(&app.issues_view_state),
                                );
                                app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                            }
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('2') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 1;
                            app.tts_manager.announce("Split tab");
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('3') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 2;
                            app.tts_manager.announce("Kanban tab");
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('4') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 3;
                            app.tts_manager.announce("Dependencies tab");
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('5') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 4;
                            app.tts_manager.announce("Labels tab");
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('6') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 5;
                            app.tts_manager.announce("Gantt tab");
                            app.mark_dirty();
                            continue;
                        }
                    }
                    KeyCode::Char('7') => {
                        // On Issues view (tab 0), keys 1-7 are for filters, not tab switching
                        if app.selected_tab != 0 {
                            app.selected_tab = 6;
                            app.tts_manager.announce("Pert tab");
                            app.mark_dirty();
                            continue;
                        }
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
                    // Letter shortcuts for tab navigation
                    // i=Issues, r=Record Detail, s=Split, k=Kanban, d=Dependencies, l=Labels
                    // g=Gantt, p=Pert, m=Molecular, t=Statistics, u=Utilities, h=Help
                    KeyCode::Char('i') => {
                        // Issues view - same as '1' but always works
                        app.selected_tab = 0;
                        app.tts_manager.announce("Issues tab");
                        // Initialize filter_bar_state for fullscreen Issues View
                        if app.issues_view_state.filter_bar_state.is_none() {
                            let filter_bar_state = ui::widgets::FilterBarState::new(
                                beads_tui::helpers::collect_unique_statuses(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_priorities(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_types(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_labels(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_assignees(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_created_dates(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_updated_dates(&app.issues_view_state),
                                beads_tui::helpers::collect_unique_closed_dates(&app.issues_view_state),
                            );
                            app.issues_view_state.filter_bar_state = Some(filter_bar_state);
                        }
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('r') => {
                        // 'r' switches to Record Detail tab
                        app.selected_tab = 10;
                        app.tts_manager.announce("Record Detail tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('s') => {
                        // 's' switches to Split View tab
                        app.selected_tab = 1;
                        app.tts_manager.announce("Split View tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('k') => {
                        // 'k' switches to Kanban tab
                        app.selected_tab = 2;
                        app.tts_manager.announce("Kanban tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('d') => {
                        app.selected_tab = 3;
                        app.tts_manager.announce("Dependencies tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('l') => {
                        app.selected_tab = 4;
                        app.tts_manager.announce("Labels tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('g') => {
                        app.selected_tab = 5;
                        app.tts_manager.announce("Gantt tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('p') => {
                        app.selected_tab = 6;
                        app.tts_manager.announce("Pert tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('m') => {
                        app.selected_tab = 7;
                        app.tts_manager.announce("Molecular tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('t') => {
                        app.selected_tab = 8;
                        app.tts_manager.announce("Statistics tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('u') => {
                        app.selected_tab = 9;
                        app.tts_manager.announce("Utilities tab");
                        app.mark_dirty();
                        continue;
                    }
                    KeyCode::Char('h') => {
                        app.selected_tab = 11;
                        app.tts_manager.announce("Help tab");
                        app.mark_dirty();
                        continue;
                    }
                    _ => {}
                }

                // Tab-specific key bindings using ViewEventHandler trait
                use ui::views::ViewEventHandler;
                let _handled = match app.selected_tab {
                    0 | 1 => IssuesViewState::handle_key_event(app, key), // Issues & Split
                    2 => KanbanViewState::handle_key_event(app, key),     // Kanban
                    3 => DependenciesViewState::handle_key_event(app, key), // Dependencies
                    4 => LabelsViewState::handle_key_event(app, key),     // Labels
                    5 => GanttViewState::handle_key_event(app, key),      // Gantt
                    6 => PertViewState::handle_key_event(app, key),       // PERT
                    7 => FormulaBrowserState::handle_key_event(app, key), // Molecular
                    8 | 9 => DatabaseViewState::handle_key_event(app, key), // Statistics & Utilities
                    10 => IssuesViewState::handle_key_event(app, key), // Record Detail
                    11 => HelpViewState::handle_key_event(app, key),      // Help
                    _ => false,
                };

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

/// Render full-screen Issues View with new design (solid backgrounds, no borders)
fn render_issues_view_fullscreen(f: &mut Frame, app: &mut models::AppState) {
    let area = f.size();

    // Initialize filter_bar_state if not already initialized (needed for filter hotkeys 1-7)
    if app.issues_view_state.filter_bar_state.is_none() {
        let filter_bar_state = ui::widgets::FilterBarState::new(
            beads_tui::helpers::collect_unique_statuses(&app.issues_view_state),
            beads_tui::helpers::collect_unique_priorities(&app.issues_view_state),
            beads_tui::helpers::collect_unique_types(&app.issues_view_state),
            beads_tui::helpers::collect_unique_labels(&app.issues_view_state),
            beads_tui::helpers::collect_unique_assignees(&app.issues_view_state),
            beads_tui::helpers::collect_unique_created_dates(&app.issues_view_state),
            beads_tui::helpers::collect_unique_updated_dates(&app.issues_view_state),
            beads_tui::helpers::collect_unique_closed_dates(&app.issues_view_state),
        );
        app.issues_view_state.filter_bar_state = Some(filter_bar_state);
    }

    // Layout: Title(1) | Tabs(1) | Filters(2) | Issues(min) | Actions(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Tab bar
            Constraint::Length(2), // FILTERS section (2 lines)
            Constraint::Min(5),    // Issues table
            Constraint::Length(1), // Action bar
        ])
        .split(area);

    // 1. TITLE BAR - Blue background, single line
    let open_count = app.database_stats.open_issues;
    let in_progress_count = app.database_stats.in_progress_issues;
    let blocked_count = app.database_stats.blocked_issues;
    let closed_count = app.database_stats.closed_issues;

    let daemon_text = if app.daemon_running {
        "{Daemon: Running}"
    } else {
        "{Daemon: Stopped}"
    };

    // Build title with right-justified daemon status
    let left_part = format!(
        "[BEADS-TUI v.1.6.0]—(Open: {}, In-Progress: {}, Blocked: {}, Closed: {})",
        open_count, in_progress_count, blocked_count, closed_count
    );

    // Calculate padding to push daemon status to the right edge (add 2 to move it 2 chars closer)
    let total_content = left_part.len() + daemon_text.len();
    let padding_len = if total_content + 2 <= area.width as usize {
        area.width as usize - total_content + 2
    } else {
        1 // At least 1 character spacing
    };

    let title_text = format!(
        "{}{}{}",
        left_part,
        "—".repeat(padding_len),
        daemon_text
    );

    let title = Paragraph::new(Line::from(Span::styled(
        title_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    )));
    f.render_widget(title, chunks[0]);

    // 2. TAB BAR - Cyan background with yellow for selected, first letter bold+underlined
    let mut tab_spans = Vec::new();
    for (i, &tab_name) in app.tabs.iter().enumerate() {
        let bg_color = if i == app.selected_tab { Color::Yellow } else { Color::Cyan };
        let base_style = Style::default().fg(Color::Black).bg(bg_color);

        // Add leading space
        tab_spans.push(Span::styled(" ", base_style));

        // Split tab name to find and style the hotkey character
        // For most tabs, the first character is the hotkey (uppercase)
        // For special cases like "sTatistics", find the first uppercase letter
        if let Some(first_char) = tab_name.chars().next() {
            if first_char.is_uppercase() {
                // Standard case: first character is the hotkey
                if let Some((hotkey, rest)) = tab_name.split_at_checked(1) {
                    tab_spans.push(Span::styled(
                        hotkey,
                        base_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    ));
                    if i == app.selected_tab {
                        tab_spans.push(Span::styled(rest, base_style.add_modifier(Modifier::BOLD)));
                    } else {
                        tab_spans.push(Span::styled(rest, base_style));
                    }
                }
            } else {
                // Special case: first char is lowercase, find first uppercase as hotkey
                if let Some(hotkey_pos) = tab_name.chars().position(|c| c.is_uppercase()) {
                    let (before, from_hotkey) = tab_name.split_at(hotkey_pos);
                    let (hotkey, after) = from_hotkey.split_at_checked(1).unwrap_or((from_hotkey, ""));

                    // Before hotkey: normal
                    if i == app.selected_tab {
                        tab_spans.push(Span::styled(before, base_style.add_modifier(Modifier::BOLD)));
                    } else {
                        tab_spans.push(Span::styled(before, base_style));
                    }
                    // Hotkey: bold and underlined
                    tab_spans.push(Span::styled(
                        hotkey,
                        base_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    ));
                    // After hotkey: normal
                    if i == app.selected_tab {
                        tab_spans.push(Span::styled(after, base_style.add_modifier(Modifier::BOLD)));
                    } else {
                        tab_spans.push(Span::styled(after, base_style));
                    }
                } else {
                    // Fallback: no uppercase found, style first character
                    if let Some((hotkey, rest)) = tab_name.split_at_checked(1) {
                        tab_spans.push(Span::styled(
                            hotkey,
                            base_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                        ));
                        if i == app.selected_tab {
                            tab_spans.push(Span::styled(rest, base_style.add_modifier(Modifier::BOLD)));
                        } else {
                            tab_spans.push(Span::styled(rest, base_style));
                        }
                    }
                }
            }
        } else {
            // Fallback if tab name is empty
            tab_spans.push(Span::styled(tab_name, base_style));
        }

        // Add trailing space
        tab_spans.push(Span::styled(" ", base_style));

        // Add separator
        if i < app.tabs.len() - 1 {
            tab_spans.push(Span::styled(
                "|",
                Style::default().fg(Color::Black).bg(Color::Cyan)
            ));
        }
    }

    // Fill remaining space with cyan background
    // Calculate how much space the tabs take
    let tabs_text_len: usize = app.tabs.iter().enumerate().map(|(i, name)| {
        let tab_len = name.len() + 2; // " tab "
        let sep_len = if i < app.tabs.len() - 1 { 1 } else { 0 }; // "|"
        tab_len + sep_len
    }).sum();

    // Add padding at the end to fill the line with cyan
    if tabs_text_len < area.width as usize {
        tab_spans.push(Span::styled(
            " ".repeat(area.width as usize - tabs_text_len),
            Style::default().bg(Color::Cyan)
        ));
    }

    let tabs_line = Line::from(tab_spans);
    let tabs = Paragraph::new(tabs_line);
    f.render_widget(tabs, chunks[1]);

    // 3. FILTERS SECTION - Green background, 2 lines
    let filter_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "FILTERS" label
            Constraint::Length(1), // Filter controls
        ])
        .split(chunks[2]);

    // Line 1: FILTERS label (no leading space, fill to end)
    let filters_label_text = format!("FILTERS{}", " ".repeat(area.width.saturating_sub(7) as usize));
    let filters_label = Paragraph::new(Line::from(Span::styled(
        filters_label_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    )));
    f.render_widget(filters_label, filter_chunks[0]);

    // Line 2: Filter controls (no leading space, fill to end with green background)
    let filter_controls_text = "1:Status [ALL ▼] | 2:Type [ALL ▼] | 3:Priority [ALL ▼] | 4:Labels [ALL ▼] | 5:Created [ALL ▼] | 6:Updated [ALL ▼] | 7:Reset";
    let padding_len = area.width.saturating_sub(filter_controls_text.len() as u16);
    let filter_text = format!("{}{}", filter_controls_text, " ".repeat(padding_len as usize));
    let filters_controls = Paragraph::new(Line::from(Span::styled(
        filter_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Green)
    ))).style(Style::default().bg(Color::Green)); // Ensure background fills entire line
    f.render_widget(filters_controls, filter_chunks[1]);

    // 4. ISSUES TABLE - Render using IssuesView but in a custom area
    // For now, render a placeholder to see the layout
    let issues_area = chunks[3];

    // Issues header - Blue background
    let issues_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // [ISSUES] header
            Constraint::Min(0),    // Table content
        ])
        .split(issues_area);

    let total_issues = app.issues_view_state.search_state().all_issues().len();
    let filtered_issues = app.issues_view_state.search_state().result_count();

    // Calculate exact length of prefix to fill rest of line with "—"
    let issues_prefix = format!("[ISSUES] ({}/{})", filtered_issues, total_issues);
    let dash_count = area.width.saturating_sub(issues_prefix.len() as u16);
    let issues_header_text = format!("{}{}", issues_prefix, "—".repeat(dash_count as usize));

    let issues_header = Paragraph::new(Line::from(Span::styled(
        issues_header_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    )));
    f.render_widget(issues_header, issues_chunks[0]);

    // Render actual issues list using borderless IssueList directly
    use crate::ui::widgets::IssueList;

    // Get issues from the search state
    let issues = app.issues_view_state.search_state().filtered_issues();
    let issue_refs: Vec<&beads::models::Issue> = issues.iter().collect();

    // Create borderless issue list
    let issue_list = IssueList::new(issue_refs)
        .show_border(false)
        .show_title(false);

    f.render_stateful_widget(
        issue_list,
        issues_chunks[1],
        app.issues_view_state.search_state_mut().list_state_mut()
    );

    // 5. ACTION BAR - Blue background (fill to end with spaces)
    let action_bar_text = "↓:Up ↑:Down (row) PgUp/PgDn (page) →/←:Scroll-Right/Left | Ctrl+New | Ctrl+Delete | Ctrl+Find | Ctrl+Open | Ctrl+Close | ?:Help";
    let action_padding_len = area.width.saturating_sub(action_bar_text.len() as u16);
    let action_text = format!("{}{}", action_bar_text, " ".repeat(action_padding_len as usize));
    let action_bar = Paragraph::new(Line::from(Span::styled(
        action_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
    )));
    f.render_widget(action_bar, chunks[4]);
}

fn ui(f: &mut Frame, app: &mut models::AppState) {
    // Check if we're on Issues tab (tab 0) - render full-screen Issues view
    if app.selected_tab == 0 {
        render_issues_view_fullscreen(f, app);
        return;
    }

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
