# Developer Guide

## Quick start
```powershell
cargo run
```

Run the full local suite:
```powershell
pwsh ./scripts/test.ps1 -Suite all
```

## Project layout
- src/main.rs: event loop and rendering.
- src/models: domain models and AppState.
- src/ui/views: screen-level views.
- src/ui/widgets: reusable widgets.
- src/beads: CLI client and parsers.

See docs/ARCHITECTURE.md for deeper context.

## Adding a new view
1. Create a new view in src/ui/views.
2. Add view state to src/models/app_state.rs.
3. Wire the view into src/ui/views/mod.rs.
4. Add rendering and input handling in src/main.rs.
5. Add snapshot or integration tests as needed.

## Adding a widget
1. Create a widget in src/ui/widgets.
2. Define a state struct and implement StatefulWidget if needed.
3. Keep render functions pure and avoid side effects.
4. Add unit tests for layout and behavior.

See docs/WIDGET_DEVELOPMENT.md for patterns and examples.

## Testing
- TEST_STRATEGY.md defines coverage expectations.
- TEST_HARNESS.md documents the runner and fixtures.
- tests/ contains integration, snapshot, and property-based suites.

## Debugging
```powershell
$env:RUST_LOG="debug"; cargo run
```

Use trace logging for UI modules:
```powershell
$env:RUST_LOG="beads_tui::ui=trace"; cargo run
```

## Contributing
Follow CONTRIBUTING.md for style, linting, and PR requirements.
