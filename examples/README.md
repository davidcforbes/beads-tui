# Beads-TUI Examples

This directory contains example code demonstrating various features and use cases of beads-tui.

## Running Examples

To run an example, use:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example basic_ui
```

## Available Examples

### basic_ui.rs
Demonstrates creating a basic TUI application using beads-tui components.
Shows how to:
- Set up the terminal
- Create a simple layout
- Handle keyboard input
- Render widgets (TabBar, StatusBar, Dialog)
- Navigate between tabs
- Display lists and paragraphs

This example is fully self-contained and doesn't require beads CLI to be installed.

## Requirements

All examples require:
- Rust 1.70+
- Terminal with 256 color support

Some future examples may require:
- Beads CLI installed (for real backend examples)
- A beads repository initialized (for integration examples)

## Learning Path

1. Start with `basic_ui.rs` to understand the UI framework
2. Check the main application code in `src/` for advanced patterns
3. Review widget implementations in `src/ui/widgets/` for custom components

## Contributing Examples

If you create a useful example, please consider contributing it!
See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
