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
- **Unified Form System**: Consistent form layouts across create, edit, detail, and split views
- **Issue List View**: Browse issues with sorting, filtering, and column customization
- **Issue Detail View**: View full issue details with metadata and relationships
- **Split Screen Mode**: View details alongside the issue list
- **Visual Dependency Trees**: See dependency relationships as interactive trees
- **Dependencies View**: Manage dependencies with tree visualization and cycle detection
- **Label Management**: Browse and manage labels with autocomplete
- **Database Dashboard**: Monitor database health, sync status, and operations
- **Column Manager**: Hide, show, and reorder table columns
- **Advanced Search**: Full-text search with fuzzy matching, regex, and saved filters
- **Smart Filtering**: Multi-criteria filtering with status, priority, type, and label filters
- **Markdown Rendering**: Rich text display for issue descriptions
- **Gantt Chart View**: Timeline visualization with scheduling and date derivation
- **Kanban Board View**: Column-based workflow with WIP limits and drag support
- **PERT Chart View**: Network diagram for dependencies with critical path analysis
- **Theme Support**: 5 themes including accessibility themes (High-Contrast, Deuteranopia, Protanopia, Tritanopia)
- **Undo/Redo System**: Command history tracking with 50-command capacity
- **Keyboard-First Design**: Complete keyboard navigation with 66+ keybindable actions
- **Form Validation**: Comprehensive field validation for required fields, formats, and constraints
- **Help System**: Context-sensitive help and keyboard shortcuts overlay
- **Notification System**: Toast notifications with history panel (Ctrl+H)
- **Task Management**: Background async operations with progress tracking
- **Molecular Chemistry UI**: Advanced wizards for bonding, formulas, pour, and wisp operations

### In Progress 🚧

- **Bulk Operations**: Enhanced multi-select and batch operations
- **Performance Optimization**: Incremental rendering and caching improvements
- **Test Coverage**: Expanding unit and integration test suite
- **Documentation**: Comprehensive user and developer guides

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
3. Press `Enter` to save or `Esc` to cancel

#### Editing Issues

1. Select an issue in the list
2. Press `e` to edit
3. Modify fields as needed
4. Press `Enter` to save changes

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
│   ├── main.rs              # Entry point, event loop, and rendering orchestration
│   ├── lib.rs               # Library exports and public API
│   ├── runtime.rs           # Global async runtime singleton
│   ├── models/              # Domain models and application state
│   │   ├── app_state.rs     # Central AppState (single source of truth)
│   │   ├── filter.rs        # Issue filtering with saved filters
│   │   ├── table_config.rs  # Column definitions and visibility
│   │   ├── kanban_config.rs # Kanban board configuration
│   │   ├── gantt_schedule.rs # Timeline scheduling
│   │   ├── pert_layout.rs   # PERT chart layout
│   │   ├── issue_cache.rs   # Issue caching with statistics
│   │   └── undo_history.rs  # Command history tracking
│   ├── ui/                  # UI components and widgets
│   │   ├── views/           # 15+ screen-level views
│   │   │   ├── issues_view.rs        # Main issue list view
│   │   │   ├── issue_detail.rs       # Issue detail view
│   │   │   ├── issue_editor.rs       # Issue edit view
│   │   │   ├── create_issue.rs       # Issue creation view
│   │   │   ├── issue_form_builder.rs # Unified form builder
│   │   │   ├── kanban_view.rs        # Kanban board
│   │   │   ├── gantt_view.rs         # Gantt chart
│   │   │   ├── pert_view.rs          # PERT chart
│   │   │   ├── dependencies_view.rs  # Dependency management
│   │   │   ├── dependency_graph.rs   # Dependency graph visualization
│   │   │   ├── labels_view.rs        # Label management
│   │   │   ├── database_view.rs      # Database dashboard
│   │   │   ├── search_interface.rs   # Search interface
│   │   │   ├── help_view.rs          # Help and shortcuts
│   │   │   └── molecular/            # Molecular chemistry views
│   │   ├── widgets/         # 34+ reusable widgets
│   │   │   ├── form.rs      # Form widget with validation
│   │   │   ├── dialog.rs    # Modal dialogs
│   │   │   ├── filter_bar.rs # Filter bar widget
│   │   │   ├── kanban_card.rs # Kanban card widget
│   │   │   ├── gantt_chart.rs # Gantt chart widget
│   │   │   └── ...          # Many more widgets
│   │   └── themes/          # 5 theme definitions
│   ├── beads/               # Beads CLI integration layer
│   │   ├── client.rs        # Async BeadsClient with retry logic
│   │   ├── models.rs        # Issue, Status, Priority, Type, Note models
│   │   ├── parser.rs        # Defensive JSON parsing
│   │   ├── error.rs         # Custom error types
│   │   └── mock.rs          # Mock backend for testing
│   ├── config/              # Configuration management
│   │   ├── keybindings.rs   # 66 customizable actions
│   │   └── mod.rs           # YAML-based config
│   ├── tasks/               # Background task management
│   │   ├── manager.rs       # TaskManager for async tasks
│   │   └── types.rs         # TaskHandle, TaskId, TaskStatus
│   ├── undo/                # Undo/redo system
│   │   └── ...              # Command pattern implementation
│   ├── graph/               # Dependency graph algorithms
│   │   └── ...              # Layout, cycle detection, topological sort
│   ├── tts/                 # Text-to-speech support
│   └── utils/               # Utility functions
├── docs/                    # Documentation
│   ├── ARCHITECTURE.md      # Architecture and design decisions
│   ├── SEARCH_ARCHITECTURE.md # Search V2 design
│   ├── FILTERING_GUIDE.md   # Filter system details
│   ├── widgets.md           # Widget catalog
│   └── USER_GUIDE.md        # End-user documentation
├── tests/                   # Integration tests
├── Cargo.toml              # Rust dependencies
├── KEYBOARD_SHORTCUTS.md   # Complete keyboard reference
└── WORKPLAN.md             # Development roadmap
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

## Keyboard Shortcuts

Beads-TUI supports 66+ keybindable actions with intuitive shortcuts. Here are the most common:

### Global

- `?` or `F1` - Show help / keyboard shortcuts
- `q`, `Ctrl+Q`, or `Ctrl+C` - Quit
- `Tab` / `Shift+Tab` - Switch tabs
- `Ctrl+Z` / `Ctrl+Y` - Undo / Redo
- `Ctrl+H` - Show notification history
- `Esc` - Dismiss notifications / close overlays

### Issue Management

- `n` - Create new issue
- `e` - Edit selected issue
- `d` - Delete selected issue
- `x` - Close selected issue
- `o` - Reopen selected issue
- `Enter` - View details / Confirm action
- `c` - Open column manager

### Navigation

- `j`/`k` or `↓`/`↑` - Move up/down
- `h`/`l` or `←`/`→` - Move left/right
- `g`/`G` - Jump to top/bottom
- `Ctrl+U` / `Ctrl+D` - Page up/down

### Search & Filters

- `/` or `s` - Focus search bar
- `f` - Toggle filters
- `Shift+F` - Clear filters
- `Alt+Z` - Toggle fuzzy search
- `Alt+R` - Toggle regex search
- `Alt+S` / `Alt+P` / `Alt+T` / `Alt+L` - Open status/priority/type/labels filters
- `F3`-`F11` - Apply saved filters

### Issue Operations

- `p` - Update priority
- `Shift+S` - Update status
- `l` - Update labels
- `a` - Update assignee
- `+` / `-` - Add/remove dependency
- `>` / `<` - Indent/outdent issue

See [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md) for the complete reference.

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

- ✅ Implemented unified form system across all views
- ✅ Completed Gantt, Kanban, and PERT chart views
- ✅ Added 5-theme system with accessibility support
- ✅ Implemented advanced search with fuzzy and regex matching
- ✅ Built molecular chemistry UI wizards
- ✅ Added undo/redo system with command history
- ✅ Implemented notification system with history panel
- ✅ Created comprehensive keyboard shortcuts (66+ actions)

**Next Steps**:

1. Enhance bulk operations and multi-select functionality
2. Improve test coverage (unit and integration tests)
3. Optimize performance with incremental rendering
4. Complete comprehensive user and developer documentation
5. Add CI/CD pipeline and release automation
