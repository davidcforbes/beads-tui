# Beads-TUI

[![CI](https://github.com/davidcforbes/beads-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/davidcforbes/beads-tui/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/davidcforbes/beads-tui/branch/main/graph/badge.svg)](https://codecov.io/gh/davidcforbes/beads-tui)

Interactive Text User Interface for
[Beads](https://github.com/steveyegge/beads) - A powerful terminal UI
for managing issues, dependencies, and workflows.

## Overview

Beads-TUI is a Rust-based terminal user interface that provides a rich,
interactive experience for all beads CLI functionality. Instead of typing
commands, navigate through issues, manage dependencies visually, and
execute workflows through an intuitive interface.

## Features

### Implemented ✅

- **Interactive Issue Management**: Create, edit, update, and close issues through forms and dialogs
- **Issue List View**: Browse issues with sorting, filtering, and column customization
- **Issue Detail View**: View full issue details with metadata and relationships
- **Visual Dependency Trees**: See dependency relationships as interactive trees
- **Dependencies View**: Manage dependencies with tree visualization
- **Label Management**: Browse and manage labels with autocomplete
- **Database Dashboard**: Monitor database health, sync status, and daemon operations
- **Column Manager**: Hide, show, and reorder table columns
- **Smart Search**: Full-text search with filtering
- **Markdown Rendering**: Rich text display for issue descriptions
- **Keyboard-First Design**: Efficient navigation with intuitive keybindings
- **Form Validation**: Field validation for required fields and formats
- **Help System**: Context-sensitive help and keyboard shortcuts

### In Progress 🚧

- **Gantt Chart View**: Calendar-based timeline visualization
- **Kanban Board View**: Column-based workflow visualization
- **PERT Chart View**: Network diagram for dependencies
- **Bulk Operations**: Select and operate on multiple issues at once
- **Theme Support**: Multiple color themes
- **Molecular Chemistry UI**: Interactive wizards for advanced operations

## Tech Stack

- **Rust**: Core language for performance and reliability
- **Ratatui**: Terminal UI framework for rich interfaces
- **Crossterm**: Cross-platform terminal manipulation
- **Beads-rs**: Rust wrapper library for beads CLI commands

## Project Status

✅ **Active Development** - Core functionality is working!

The application now has a functional terminal UI with issue management, dependency visualization, and database monitoring. See [WORKPLAN.md](WORKPLAN.md) for the comprehensive development roadmap.

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- [Beads CLI](https://github.com/steveyegge/beads) installed and configured
- A terminal emulator with 256 color support

### Building from Source

```bash
# Clone the repository
git clone https://github.com/davidcforbes/beads-tui
cd beads-tui

# Build the project
cargo build --release

# Run the application
./target/release/beads-tui

# Or install locally
cargo install --path .
beads-tui
```

### Usage

Once installed, run `beads-tui` in a directory with a beads repository:

```bash
cd /path/to/your/beads/project
beads-tui
```

#### Basic Navigation

- `Tab` / `Shift+Tab` - Switch between views
- `↑` / `↓` or `j` / `k` - Navigate items
- `Enter` - Select/View details
- `Esc` - Go back
- `?` - Show help
- `q` - Quit

#### Views

- **Issues** (`Tab` → Issues): Browse and manage issues
- **Dependencies** (`Tab` → Dependencies): View dependency trees
- **Labels** (`Tab` → Labels): Manage labels
- **Database** (`Tab` → Database): Monitor database status
- **Help** (`?`): View all keyboard shortcuts

#### Creating Issues

1. Press `n` in the Issues view
2. Fill out the form (title, type, priority, description)
3. Press `Ctrl+S` to save or `Esc` to cancel

#### Editing Issues

1. Select an issue in the list
2. Press `e` to edit
3. Modify fields as needed
4. Press `Ctrl+S` to save changes

### Development Setup

```bash
# Clone the repository
git clone https://github.com/davidcforbes/beads-tui
cd beads-tui

# Run in development mode
cargo run

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Run specific test suite
cargo test --test integration_test
```

## Work Plan Import

To import the complete work plan into your beads database:

```bash
# Generate shell script
python generate-issues.py --output create-issues.sh

# Review the script
cat create-issues.sh

# Execute to create all issues
bash create-issues.sh

# Or run directly (dry-run first recommended)
python generate-issues.py --dry-run
python generate-issues.py
```

This will create:

- 14 epics covering all major areas
- 60+ features defining user-facing functionality
- 350+ tasks breaking down implementation work
- 50+ chores for maintenance and infrastructure
- 40+ potential bugs to watch for

## Architecture

```text
beads-tui/
├── src/
│   ├── main.rs              # Entry point and main loop
│   ├── app.rs               # Application state management
│   ├── ui/                  # UI components and widgets
│   │   ├── mod.rs
│   │   ├── layout.rs        # Layout engine
│   │   ├── widgets/         # Reusable UI widgets
│   │   └── views/           # Main application views
│   ├── beads/               # Beads-rs wrapper library
│   │   ├── mod.rs
│   │   ├── client.rs        # CLI command execution
│   │   ├── models.rs        # Data models
│   │   └── parser.rs        # JSON response parsing
│   ├── config.rs            # Configuration management
│   ├── events.rs            # Event handling
│   └── keybindings.rs       # Keyboard shortcut management
├── tests/                   # Integration tests
├── Cargo.toml              # Rust dependencies
├── WORKPLAN.md             # Complete development roadmap
└── generate-issues.py      # Script to import work plan to beads
```

## Keyboard Shortcuts (Planned)

### Global

- `?` - Show help
- `q` - Quit
- `Ctrl+c` - Force quit
- `/` - Search
- `:` - Command palette
- `Tab` - Next panel
- `Shift+Tab` - Previous panel

### Issue List

- `j`/`k` or `↓`/`↑` - Navigate
- `Enter` - View details
- `n` - Create new issue
- `e` - Edit selected issue
- `d` - Delete selected issue
- `c` - Close selected issue
- `Space` - Toggle selection (bulk mode)
- `a` - Select all
- `A` - Deselect all

### Issue Detail

- `e` - Edit issue
- `c` - Close issue
- `r` - Reopen issue
- `l` - Manage labels
- `d` - Manage dependencies
- `Esc` - Back to list

### Filters

- `f` - Open filter builder
- `F` - Save current filter
- `Ctrl+f` - Quick filter
- `F1-F12` - Saved filter shortcuts

## Development Roadmap

See [WORKPLAN.md](WORKPLAN.md) for the complete development roadmap, organized into 14 epics:

1. **Project Setup & Foundation** - Rust scaffolding, beads-rs wrapper, config
2. **Core TUI Framework** - Event loop, navigation, layouts, status bar
3. **Issue Management Interface** - CRUD operations, forms, bulk actions
4. **List & Filter Interface** - Advanced filtering, search, saved filters
5. **Dependency Management UI** - Tree viewer, editor, cycle detection
6. **Label Management UI** - Label browser, autocomplete, state management
7. **Advanced Operations UI** - Cleanup, duplicates, compaction, migration
8. **Molecular Chemistry UI** - Formulas, pour, wisp, bond, squash, burn
9. **Database & Sync UI** - Dashboard, import/export, daemon, sync
10. **Quality of Life Features** - Help, themes, keybindings, notifications, undo
11. **Performance & Optimization** - Lazy loading, caching, background ops
12. **Testing & Quality Assurance** - Unit, integration, UI, property tests
13. **Documentation & Onboarding** - User guide, dev guide, tutorials
14. **Release & Distribution** - Binary releases, package managers, auto-update

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas Needing Help

- Rust developers for core implementation
- UI/UX designers for interface design
- Technical writers for documentation
- Testers for cross-platform testing

## License

[Choose appropriate license - MIT, Apache 2.0, etc.]

## Acknowledgments

- [Beads](https://github.com/steveyegge/beads) by Steve Yegge - The amazing issue management system this TUI wraps
- [Ratatui](https://github.com/ratatui-org/ratatui) - Excellent terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library

## Related Projects

- [Beads](https://github.com/steveyegge/beads) - The core CLI tool
- [Beads-Viewer](https://github.com/steveyegge/beads_viewer) - Web-based viewer
- [Ratatui](https://github.com/ratatui-org/ratatui) - TUI framework we build on

## Support

- **Issues**: [GitHub Issues](https://github.com/davidcforbes/beads-tui/issues)
- **Discussions**: [GitHub Discussions](https://github.com/davidcforbes/beads-tui/discussions)
- **Beads Community**: [Beads Discussions](https://github.com/steveyegge/beads/discussions)

---

**Status**: ✅ Active Development - Core features functional!

**Recent Milestones**:

- ✅ Implemented beads-rs wrapper library
- ✅ Created interactive issue list view
- ✅ Built issue create/edit forms
- ✅ Added dependency tree visualization
- ✅ Implemented column manager
- ✅ Added markdown rendering

**Next Steps**:

1. Complete Gantt, Kanban, and PERT chart views
2. Add bulk operations support
3. Implement theme system
4. Add molecular chemistry UI
5. Create comprehensive documentation
