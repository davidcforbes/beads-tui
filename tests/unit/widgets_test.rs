use beads_tui::ui::widgets::StatusBar;
use ratatui::{
    backend::TestBackend,
    style::{Color, Style},
    text::Span,
    Terminal,
};

#[test]
fn test_status_bar_snapshot() {
    let mut terminal = Terminal::new(TestBackend::new(80, 3)).unwrap();

    terminal
        .draw(|f| {
            let status_bar = StatusBar::new()
                .left(vec![Span::raw("Context: Issues")])
                .center(vec![Span::styled(
                    "NORMAL",
                    Style::default().fg(Color::Cyan),
                )])
                .right(vec![Span::raw("Total: 10 | Open: 5")]);

            f.render_widget(status_bar, f.size());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify context
    assert!(buffer
        .content
        .iter()
        .any(|c| c.symbol() == "C" && c.fg == Color::White));
    // Verify mode
    assert!(buffer
        .content
        .iter()
        .any(|c| c.symbol() == "N" && c.fg == Color::Cyan));
    // Verify some content is rendered
    assert!(buffer.content.iter().any(|c| c.symbol() != " "));
}

#[test]
fn test_dialog_snapshot() {
    use beads_tui::ui::widgets::{Dialog, DialogState};
    use ratatui::layout::Rect;

    let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
    let mut state = DialogState::new();

    terminal
        .draw(|f| {
            let dialog = Dialog::confirm("Test Dialog", "Are you sure you want to proceed?");

            let area = Rect::new(0, 0, 80, 10);
            f.render_stateful_widget(dialog, area, &mut state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Debug print buffer content if test fails
    let symbols: String = buffer.content.iter().map(|c| c.symbol()).collect();
    // Use chunks of 80 to see lines
    for line in symbols.as_bytes().chunks(80) {
        println!("{}", String::from_utf8_lossy(line));
    }

    // Verify title
    assert!(
        buffer.content.iter().any(|c| c.symbol().contains('T')),
        "Title character 'T' not found"
    );
    // Verify content
    assert!(
        buffer.content.iter().any(|c| c.symbol().contains('A')),
        "Content character 'A' not found"
    );
    // Verify buttons (they are rendered as "[ Yes ]" and "  No  " when first is selected)
    assert!(
        buffer.content.iter().any(|c| c.symbol().contains('Y')),
        "Button character 'Y' not found"
    );
    assert!(
        buffer.content.iter().any(|c| c.symbol().contains('N')),
        "Button character 'N' not found"
    );
}

#[test]
fn test_tab_bar_snapshot() {
    use beads_tui::ui::widgets::TabBar;
    let mut terminal = Terminal::new(TestBackend::new(80, 5)).unwrap();

    terminal
        .draw(|f| {
            let tab_bar = TabBar::new(vec!["Issues", "Labels", "Database"]).selected(1);
            f.render_widget(tab_bar, f.size());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify labels are present
    assert!(buffer.content.iter().any(|c| c.symbol().contains('I')));
    assert!(buffer.content.iter().any(|c| c.symbol().contains('L')));
    assert!(buffer.content.iter().any(|c| c.symbol().contains('D')));

    // Verify selection highlighting (selected tab "Labels" should have Yellow fg)
    assert!(buffer
        .content
        .iter()
        .any(|c| c.symbol().contains('L') && c.fg == Color::Yellow));
}

#[test]
fn test_spinner_snapshot() {
    use beads_tui::ui::widgets::Spinner;
    let mut terminal = Terminal::new(TestBackend::new(80, 1)).unwrap();

    terminal
        .draw(|f| {
            let spinner = Spinner::new().label("Loading...");
            f.render_widget(spinner, f.size());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify label is present
    assert!(buffer.content.iter().any(|c| c.symbol().contains('L')));
    assert!(buffer.content.iter().any(|c| c.symbol().contains('.')));
    // Spinner char is indeterminate but should be there
}

#[test]
fn test_progress_bar_snapshot() {
    use beads_tui::ui::widgets::ProgressBar;
    let mut terminal = Terminal::new(TestBackend::new(80, 1)).unwrap();

    terminal
        .draw(|f| {
            let bar = ProgressBar::new(0.5).label("Tasks");
            f.render_widget(bar, f.size());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify percentage label
    assert!(buffer.content.iter().any(|c| c.symbol().contains('5')));
    assert!(buffer.content.iter().any(|c| c.symbol().contains('0')));
    assert!(buffer.content.iter().any(|c| c.symbol().contains('%')));
    // Verify label
    assert!(buffer.content.iter().any(|c| c.symbol().contains('T')));
}
