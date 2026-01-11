# Beads-TUI Quick Start Guide

Get your beads-tui development environment up and running in minutes.

## Prerequisites

1. **Install Rust** (if not already installed)

   ```bash
   # Linux/macOS
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Windows
   # Download and run: https://rustup.rs/
   ```

2. **Install Beads CLI**

   ```bash
   # Follow installation instructions at:
   # https://github.com/steveyegge/beads
   ```

3. **Verify installations**

   ```bash
   rustc --version  # Should show 1.70+
   cargo --version
   bd --version     # Should show beads CLI version
   ```

## Setup Steps

### 1. Initialize the Project

```bash
cd C:\Development\beads-tui  # Or your project directory

# Initialize Cargo project (if not done)
cargo init --name beads-tui

# The Cargo.toml is already configured with dependencies
```

### 2. Create Basic Directory Structure

```bash
# Create source directories
mkdir -p src/ui/widgets
mkdir -p src/ui/views
mkdir -p src/beads
mkdir -p tests
mkdir -p benches

# Create placeholder files
touch src/app.rs
touch src/config.rs
touch src/events.rs
touch src/keybindings.rs
touch src/ui/mod.rs
touch src/ui/layout.rs
touch src/ui/widgets/mod.rs
touch src/ui/views/mod.rs
touch src/beads/mod.rs
touch src/beads/client.rs
touch src/beads/models.rs
touch src/beads/parser.rs
```

### 3. Create Minimal main.rs

Create `src/main.rs`:

```rust
use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear screen and show message
    terminal.clear()?;
    terminal.draw(|f| {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::layout::{Layout, Constraint, Direction};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let welcome = Paragraph::new("Welcome to Beads-TUI!\n\nPress 'q' to quit.")
            .block(Block::default().title("Beads-TUI v0.1.0").borders(Borders::ALL));

        f.render_widget(welcome, chunks[0]);
    })?;

    // Wait for 'q' key
    loop {
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}
```

### 4. Build and Run

```bash
# Build the project
cargo build

# Run the application
cargo run

# You should see a welcome message. Press 'q' to quit.
```

## Import Work Plan to Beads

To track development using beads itself:

```bash
# Make sure you're in a beads-initialized directory
cd C:\Development\beads-tui
bd init

# Generate and execute issue creation script
python generate-issues.py --dry-run  # Preview first
python generate-issues.py             # Create all issues

# Or generate a shell script to review
python generate-issues.py --output create-issues.sh
# Review the script, then:
bash create-issues.sh  # Windows: run in Git Bash or WSL
```

This creates:

- 14 epics for major areas
- 60+ features
- 350+ tasks
- 50+ chores
- 40+ potential bugs

### View Your Work Plan in Beads

```bash
# List all epics
bd list --type epic --json | jq -r '.[] | "\(.id): \(.title)"'

# View a specific epic's structure
bd dep tree <epic-id>

# Find ready work
bd ready

# Start working on the first task
bd update <task-id> --status in_progress
```

## Development Workflow

### 1. Pick an Issue

```bash
# Find ready work (no blockers)
bd ready --json

# Or browse by epic
bd list --type epic
bd dep tree bd-<epic-id>
```

### 2. Start Working

```bash
# Mark as in progress
bd update bd-<id> --status in_progress
```

### 3. Implement

```bash
# Create feature branch
git checkout -b feature/bd-<id>-description

# Make changes, commit often
git add .
git commit -m "bd-<id>: Implement feature"
```

### 4. Test

```bash
# Run tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

### 5. Complete

```bash
# Close the issue
bd close bd-<id> --reason "Implemented and tested"

# Sync changes
bd sync
```

## Useful Commands

### Development

```bash
# Watch mode (auto-rebuild on changes)
cargo install cargo-watch
cargo watch -x run

# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check

# Build release version
cargo build --release
```

### Beads

```bash
# View issue details
bd show bd-<id> --json

# Search issues
bd list --title "search term"

# Add labels
bd label add bd-<id> in-progress,rust,ui

# Add dependency
bd dep add bd-child bd-parent

# View dependency tree
bd dep tree bd-<id>
```

### Testing

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test '*'

# Run benchmarks
cargo bench

# Generate test coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Next Steps

1. **Read the Work Plan**: Review [WORKPLAN.md](WORKPLAN.md) to understand the full scope

2. **Choose Your Starting Point**:
   - Epic 1: Project Setup & Foundation (foundational work)
   - Epic 2: Core TUI Framework (get basic UI working)
   - Epic 3: Issue Management (first user-facing feature)

3. **Set Up Development Tools**:

   ```bash
   # Install helpful cargo extensions
   cargo install cargo-watch    # Auto-rebuild
   cargo install cargo-edit     # Manage dependencies
   cargo install cargo-expand   # View macro expansions
   cargo install cargo-tree     # View dependency tree
   ```

4. **Join the Community**:
   - Beads Discussions: <https://github.com/steveyegge/beads/discussions>
   - Ratatui Discord: <https://discord.gg/pMCEU9hNEj>

5. **Start Coding**!

   ```bash
   # Find first task
   bd ready --priority 0 --type task --json

   # Start working
   bd update bd-<id> --status in_progress

   # Build something awesome!
   cargo run
   ```

## Troubleshooting

### Rust Build Issues

```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build
```

### Beads Issues

```bash
# Check beads status
bd info

# Check daemon
bd daemons list

# Reset database (CAUTION: destructive)
rm .beads/beads.db
bd init
```

### Terminal Issues

```bash
# If terminal gets corrupted
reset

# Or in Rust panic:
# We set up panic hooks to restore terminal automatically
```

## Resources

- **Ratatui Documentation**: <https://docs.rs/ratatui/>
- **Ratatui Book**: <https://ratatui.rs/>
- **Beads Documentation**: <https://github.com/steveyegge/beads>
- **Rust Book**: <https://doc.rust-lang.org/book/>
- **Crossterm Docs**: <https://docs.rs/crossterm/>

---

**Happy Coding!** 🚀

Questions? Issues? Check the [main README](README.md) or open an issue on GitHub.
