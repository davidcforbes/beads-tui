use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Priority};
use beads_tui::models::AppState;
use beads_tui::ui::views::{DatabaseView, HelpView, IssuesView, IssuesViewState};
use chrono::{TimeZone, Utc};
use insta::Settings;
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
use serial_test::serial;

fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut output = String::new();

    for y in 0..area.height {
        for x in 0..area.width {
            output.push_str(buffer.get(x, y).symbol());
        }
        if y + 1 < area.height {
            output.push('\n');
        }
    }

    output
}

fn snapshot_settings() -> Settings {
    let mut settings = Settings::new();
    let snapshot_dir = std::env::var("BEADS_TUI_SNAPSHOT_DIR").unwrap_or_else(|_| {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        root.join("tests")
            .join("snapshots")
            .to_string_lossy()
            .to_string()
    });
    settings.set_snapshot_path(snapshot_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings
}

fn render_to_string<F>(width: u16, height: u16, draw: F) -> String
where
    F: FnOnce(&mut Terminal<TestBackend>),
{
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    draw(&mut terminal);
    buffer_to_string(terminal.backend().buffer())
}

fn snapshot_view<F>(name: &str, width: u16, height: u16, draw: F)
where
    F: FnOnce(&mut Terminal<TestBackend>),
{
    let rendered = render_to_string(width, height, draw);
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_snapshot!(name, rendered);
    });
}

fn sample_issues() -> Vec<Issue> {
    let created = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
    let updated = Utc.with_ymd_and_hms(2026, 1, 2, 10, 30, 0).unwrap();

    vec![
        Issue {
            id: "beads-100".to_string(),
            title: "Add state selector".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P1,
            issue_type: IssueType::Feature,
            description: Some("Add status selector in Issues view.".to_string()),
            assignee: Some("alex".to_string()),
            labels: vec!["ui".to_string(), "state:patrol".to_string()],
            dependencies: vec![],
            blocks: vec![],
            created,
            updated,
            closed: None,
            notes: vec![],
        },
        Issue {
            id: "beads-101".to_string(),
            title: "Fix label validation".to_string(),
            status: IssueStatus::InProgress,
            priority: Priority::P0,
            issue_type: IssueType::Bug,
            description: Some("Reject invalid label characters.".to_string()),
            assignee: Some("sam".to_string()),
            labels: vec!["bug".to_string(), "labels".to_string()],
            dependencies: vec!["beads-100".to_string()],
            blocks: vec![],
            created,
            updated,
            closed: None,
            notes: vec![],
        },
        Issue {
            id: "beads-102".to_string(),
            title: "Document workflow examples".to_string(),
            status: IssueStatus::Closed,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: Some("Add workflow examples to docs.".to_string()),
            assignee: None,
            labels: vec!["docs".to_string()],
            dependencies: vec![],
            blocks: vec![],
            created,
            updated,
            closed: Some(updated),
            notes: vec![],
        },
    ]
}

#[test]
#[serial]
fn test_help_view_snapshot_80x24() {
    let app = AppState::new();

    snapshot_view("help_view_80x24", 80, 24, |terminal| {
        terminal
            .draw(|f| {
                let help_view = HelpView::new().selected_section(app.help_section);
                f.render_widget(help_view, f.size());
            })
            .unwrap();
    });
}

#[test]
#[serial]
fn test_help_view_snapshot_120x40() {
    let app = AppState::new();

    snapshot_view("help_view_120x40", 120, 40, |terminal| {
        terminal
            .draw(|f| {
                let help_view = HelpView::new().selected_section(app.help_section);
                f.render_widget(help_view, f.size());
            })
            .unwrap();
    });
}

#[test]
#[serial]
fn test_database_view_snapshot_80x24() {
    use beads_tui::ui::views::{DatabaseStats, DatabaseStatus, DatabaseViewState};

    let mut database_view_state = DatabaseViewState::default();
    let database_stats = DatabaseStats {
        total_issues: 50,
        open_issues: 0,
        closed_issues: 0,
        blocked_issues: 0,
        database_size: 0,
        last_sync: None,
    };
    let database_status = DatabaseStatus::Ready;
    let daemon_running = true;

    snapshot_view("database_view_80x24", 80, 24, |terminal| {
        terminal
            .draw(|f| {
                let database_view = DatabaseView::new()
                    .status(database_status)
                    .stats(database_stats.clone())
                    .daemon_running(daemon_running);
                f.render_stateful_widget(database_view, f.size(), &mut database_view_state);
            })
            .unwrap();
    });
}

#[test]
#[serial]
fn test_database_view_snapshot_120x40() {
    use beads_tui::ui::views::{DatabaseStats, DatabaseStatus, DatabaseViewState};

    let mut database_view_state = DatabaseViewState::default();
    let database_stats = DatabaseStats {
        total_issues: 50,
        open_issues: 0,
        closed_issues: 0,
        blocked_issues: 0,
        database_size: 0,
        last_sync: None,
    };
    let database_status = DatabaseStatus::Ready;
    let daemon_running = true;

    snapshot_view("database_view_120x40", 120, 40, |terminal| {
        terminal
            .draw(|f| {
                let database_view = DatabaseView::new()
                    .status(database_status)
                    .stats(database_stats.clone())
                    .daemon_running(daemon_running);
                f.render_stateful_widget(database_view, f.size(), &mut database_view_state);
            })
            .unwrap();
    });
}

#[test]
#[serial]
fn test_issues_view_snapshot_80x24() {
    let mut issues_state = IssuesViewState::new(sample_issues());

    snapshot_view("issues_view_80x24", 80, 24, |terminal| {
        terminal
            .draw(|f| {
                let issues_view = IssuesView::new();
                f.render_stateful_widget(issues_view, f.size(), &mut issues_state);
            })
            .unwrap();
    });
}

#[test]
#[serial]
fn test_issues_view_snapshot_120x40() {
    let mut issues_state = IssuesViewState::new(sample_issues());

    snapshot_view("issues_view_120x40", 120, 40, |terminal| {
        terminal
            .draw(|f| {
                let issues_view = IssuesView::new();
                f.render_stateful_widget(issues_view, f.size(), &mut issues_state);
            })
            .unwrap();
    });
}
