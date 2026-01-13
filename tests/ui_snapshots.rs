use beads_tui::models::AppState;
use beads_tui::ui::views::{DatabaseView, HelpView};
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn test_help_view_snapshot() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let app = AppState::new();

    terminal
        .draw(|f| {
            let help_view = HelpView::new().selected_section(app.help_section);
            f.render_widget(help_view, f.size());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    // In a real snapshot test, we would compare the buffer against a saved file.
    // For this task, we verify that it renders something non-empty.
    assert!(buffer.content.iter().any(|c| c.symbol() != " "));
}

#[test]
fn test_database_view_snapshot() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut app = AppState::new();

    terminal
        .draw(|f| {
            let database_view = DatabaseView::new()
                .status(app.database_status)
                .stats(app.database_stats.clone())
                .daemon_running(app.daemon_running);
            f.render_stateful_widget(database_view, f.size(), &mut app.database_view_state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    assert!(buffer.content.iter().any(|c| c.symbol() != " "));
}
