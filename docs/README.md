# beads-tui Documentation

Welcome to the beads-tui documentation! beads-tui is a powerful Terminal User Interface (TUI) for managing issues with the [beads](https://github.com/davidcforbes/beads) issue tracker.

## What is beads-tui?

beads-tui provides a rich, interactive terminal interface for working with beads issues. It offers:

- **Powerful Filtering**: Advanced filtering with regex, fuzzy search, and saved filter presets
- **Dependency Management**: Visual dependency graphs, PERT charts, and Gantt charts
- **Keyboard-First UX**: Extensive keyboard shortcuts with vim-style bindings
- **Rich Widgets**: Markdown rendering, syntax highlighting, and interactive forms
- **Theme Support**: Multiple color schemes including accessibility-focused themes for color blindness
- **Undo/Redo**: Full command history with undo/redo support
- **Performance**: Optimized for large issue repositories with thousands of issues

## Quick Start

Install beads-tui:

```bash
cargo install beads-tui
```

Initialize a beads repository:

```bash
bd init
```

Launch the TUI:

```bash
beads-tui
```

## Documentation Sections

### User Guide

Learn how to use beads-tui effectively:

- [Getting Started](./USER_GUIDE.md) - Installation and basic usage
- [Tutorial](./TUTORIAL.md) - Step-by-step walkthrough
- [Advanced Features](./ADVANCED_FEATURES.md) - Power user features
- [Filtering Guide](./FILTERING_GUIDE.md) - Master the filtering system

### Developer Guide

Contributing to beads-tui:

- [Architecture](./ARCHITECTURE.md) - System design and structure
- [Developer Guide](./DEVELOPER_GUIDE.md) - Setup and contribution workflow
- [Widget Development](./WIDGET_DEVELOPMENT.md) - Creating UI components

### API Reference

- [Rust API Documentation](https://docs.rs/beads-tui) - Auto-generated from source code

## Features

### Filtering & Search

beads-tui provides a comprehensive filtering system:

- Filter by status, priority, assignee, labels
- Text search with regex and fuzzy matching
- Saved filter presets with hotkeys
- Quick filter menu for common filters

See the [Filtering Guide](./FILTERING_GUIDE.md) for details.

### Visualization

Understand your issue dependencies:

- **Gantt Chart**: Timeline view of issues with dependencies
- **PERT Chart**: Critical path analysis
- **Dependency Tree**: Hierarchical dependency visualization
- **Kanban Board**: Status-based workflow view

### Keyboard Shortcuts

beads-tui is keyboard-first with extensive shortcuts:

- Vim-style navigation (h/j/k/l, gg/G)
- Quick actions (n: new, e: edit, d: delete)
- Filter shortcuts (f: toggle filter, /: search)
- Undo/redo (Ctrl+Z, Ctrl+Y)

Press `?` in the app to see all shortcuts.

### Themes

Multiple color schemes:

- Dark (default)
- High Contrast
- Deuteranopia (red-green color blindness)
- Protanopia (red-green color blindness)
- Tritanopia (blue-yellow color blindness)

Cycle themes with `Ctrl+T`.

## Platform Support

beads-tui runs on:

- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

## Getting Help

- [FAQ](./FAQ.md) - Common questions and answers
- [GitHub Issues](https://github.com/davidcforbes/beads-tui/issues) - Report bugs or request features
- [Discussions](https://github.com/davidcforbes/beads-tui/discussions) - Ask questions and share ideas

## License

beads-tui is licensed under the MIT License. See [LICENSE](../LICENSE) for details.
