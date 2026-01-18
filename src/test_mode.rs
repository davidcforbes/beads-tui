/// Test mode functionality for CLI-based view testing
use crate::models::AppState;
use anyhow::Result;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::Frame;
use ratatui::Terminal;
use std::io::Write;
use std::time::Duration;

/// Information about a view in the application
#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub index: usize,
    pub name: &'static str,
    pub description: &'static str,
}

/// Get metadata for all available views
pub fn all_views() -> Vec<ViewInfo> {
    vec![
        ViewInfo {
            index: 0,
            name: "Issues",
            description: "Main issue list view with filtering and sorting",
        },
        ViewInfo {
            index: 1,
            name: "Split",
            description: "Split screen: issue list + detail view",
        },
        ViewInfo {
            index: 2,
            name: "Kanban",
            description: "Kanban board grouped by status",
        },
        ViewInfo {
            index: 3,
            name: "Dependencies",
            description: "Dependency tree visualization",
        },
        ViewInfo {
            index: 4,
            name: "Labels",
            description: "Label statistics and management",
        },
        ViewInfo {
            index: 5,
            name: "Gantt",
            description: "Gantt chart timeline view",
        },
        ViewInfo {
            index: 6,
            name: "PERT",
            description: "PERT diagram for critical path analysis",
        },
        ViewInfo {
            index: 7,
            name: "Molecular",
            description: "Molecular formulas and wisps management",
        },
        ViewInfo {
            index: 8,
            name: "Statistics",
            description: "Database statistics dashboard",
        },
        ViewInfo {
            index: 9,
            name: "Utilities",
            description: "Database maintenance and utility tools",
        },
        ViewInfo {
            index: 10,
            name: "Record",
            description: "Record detail view (full-screen issue details)",
        },
        ViewInfo {
            index: 11,
            name: "Help",
            description: "Help and keyboard shortcuts",
        },
    ]
}

/// Run snapshot mode: render a view to text and save to file
pub fn run_snapshot_mode<B, F>(
    _terminal: &mut Terminal<B>,
    app: &mut AppState,
    view_index: usize,
    output_path: Option<String>,
    size: (u16, u16),
    render_fn: F,
) -> Result<()>
where
    B: Backend,
    F: Fn(&mut Frame, &mut AppState),
{
    // Set the selected view
    if view_index < app.tabs.len() {
        app.selected_tab = view_index;
    } else {
        return Err(anyhow::anyhow!(
            "Invalid view index: {}. Valid range: 0-{}",
            view_index,
            app.tabs.len() - 1
        ));
    }

    // Create a test backend with specified dimensions
    use ratatui::backend::TestBackend;
    let test_backend = TestBackend::new(size.0, size.1);
    let mut test_terminal = Terminal::new(test_backend)?;

    // Render the view
    test_terminal.draw(|f| render_fn(f, app))?;

    // Convert buffer to text
    let buffer = test_terminal.backend().buffer();
    let text = buffer_to_string(buffer);

    // Determine output path
    let output_file = output_path.unwrap_or_else(|| {
        format!(
            "snapshot_view_{}_{}.txt",
            view_index,
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )
    });

    // Write to file
    let mut file = std::fs::File::create(&output_file)?;
    file.write_all(text.as_bytes())?;

    println!("Snapshot saved to: {}", output_file);

    Ok(())
}

/// Run test sequence mode: cycle through all views with delays
pub fn run_test_sequence<B, F>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    duration_secs: u64,
    render_fn: F,
) -> Result<()>
where
    B: Backend,
    F: Fn(&mut Frame, &mut AppState),
{
    use crossterm::event::{self, Event, KeyCode};
    use std::time::Instant;

    let views = all_views();
    println!("Starting test sequence: {} views, {} seconds each", views.len(), duration_secs);
    println!("Press 'q' or Ctrl+C to stop\n");

    for (i, view_info) in views.iter().enumerate() {
        app.selected_tab = view_info.index;

        println!("View {}/{}: {} - {}", i + 1, views.len(), view_info.name, view_info.description);

        let start = Instant::now();
        let duration = Duration::from_secs(duration_secs);

        while start.elapsed() < duration {
            // Render the view
            terminal.draw(|f| render_fn(f, app))?;

            // Check for exit key (non-blocking with timeout)
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c') {
                        println!("\nTest sequence interrupted by user");
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("\nTest sequence completed successfully!");
    println!("Summary:");
    println!("  Total views tested: {}", views.len());
    println!("  Duration per view: {} seconds", duration_secs);
    println!("  Total time: {} seconds", views.len() as u64 * duration_secs);

    Ok(())
}

/// Convert a ratatui buffer to a string representation
fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut result = String::new();

    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buffer.get(x, y);
            result.push_str(cell.symbol());
        }
        // Don't add newline after the last line
        if y < area.height - 1 {
            result.push('\n');
        }
    }

    result
}

/// Parse terminal size from string format "WIDTHxHEIGHT"
pub fn parse_terminal_size(size_str: &str) -> Result<(u16, u16)> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid size format: '{}'. Expected format: WIDTHxHEIGHT (e.g., 120x40)",
            size_str
        ));
    }

    let width: u16 = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid width: {}", parts[0]))?;
    let height: u16 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid height: {}", parts[1]))?;

    if width < 40 || width > 500 {
        return Err(anyhow::anyhow!("Width must be between 40 and 500"));
    }
    if height < 20 || height > 200 {
        return Err(anyhow::anyhow!("Height must be between 20 and 200"));
    }

    Ok((width, height))
}

/// Print a formatted list of all available views
pub fn print_view_list() {
    let views = all_views();

    println!("Available Views:");
    println!("{:<7} {:<15} {}", "Index", "Name", "Description");
    println!("{}", "-".repeat(70));

    for view in views {
        println!("{:<7} {:<15} {}", view.index, view.name, view.description);
    }

    println!("\nUsage:");
    println!("  beads-tui --demo --view <INDEX>           # Open specific view with demo data");
    println!("  beads-tui --demo --test-all-views         # Cycle through all views");
    println!("  beads-tui --demo --view <INDEX> --snapshot # Generate snapshot");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_views_count() {
        let views = all_views();
        assert_eq!(views.len(), 11);
    }

    #[test]
    fn test_all_views_indices() {
        let views = all_views();
        for (i, view) in views.iter().enumerate() {
            assert_eq!(view.index, i);
        }
    }

    #[test]
    fn test_parse_terminal_size_valid() {
        assert_eq!(parse_terminal_size("120x40").unwrap(), (120, 40));
        assert_eq!(parse_terminal_size("80x24").unwrap(), (80, 24));
        assert_eq!(parse_terminal_size("160x50").unwrap(), (160, 50));
    }

    #[test]
    fn test_parse_terminal_size_invalid() {
        assert!(parse_terminal_size("120").is_err());
        assert!(parse_terminal_size("120x").is_err());
        assert!(parse_terminal_size("x40").is_err());
        assert!(parse_terminal_size("abc x def").is_err());
        assert!(parse_terminal_size("30x40").is_err()); // Width too small
        assert!(parse_terminal_size("120x10").is_err()); // Height too small
        assert!(parse_terminal_size("600x40").is_err()); // Width too large
        assert!(parse_terminal_size("120x300").is_err()); // Height too large
    }

    #[test]
    fn test_buffer_to_string() {
        use ratatui::backend::TestBackend;
        use ratatui::widgets::{Block, Borders, Paragraph};

        let mut backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let block = Block::default().title("Test").borders(Borders::ALL);
                f.render_widget(block, f.size());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let text = buffer_to_string(buffer);

        // Should contain the border and title
        assert!(text.contains("Test"));
        // Should have the right number of newlines (height - 1)
        assert_eq!(text.chars().filter(|&c| c == '\n').count(), 4);
    }

    #[test]
    fn test_view_info_structure() {
        let view = ViewInfo {
            index: 0,
            name: "Test",
            description: "Test description",
        };

        assert_eq!(view.index, 0);
        assert_eq!(view.name, "Test");
        assert_eq!(view.description, "Test description");
    }

    #[test]
    fn test_all_views_have_unique_names() {
        let views = all_views();
        let mut names = std::collections::HashSet::new();

        for view in views {
            assert!(
                names.insert(view.name),
                "Duplicate view name: {}",
                view.name
            );
        }
    }
}
