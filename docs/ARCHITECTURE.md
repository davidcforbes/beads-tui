# Architecture and Design Decisions

## Overview
Beads-TUI is a Rust terminal UI that wraps the beads CLI and presents a
stateful, keyboard-first interface. The architecture keeps UI rendering
pure, centralizes state in AppState, and isolates CLI interactions.

## High-level layout
- src/main.rs: app bootstrap, event loop, command routing, and rendering.
- src/models: domain models plus AppState and view-specific state.
- src/ui/views: screen-level views (issues, labels, dependencies, etc).
- src/ui/widgets: reusable widgets used by multiple views.
- src/beads: CLI client, JSON parsing, and API adapters.

## Data flow
1. The app loads issues via the beads client.
2. CLI responses are parsed into models.
3. AppState owns view state and caches derived data.
4. Views render from state without side effects.
5. User input updates AppState and may enqueue CLI commands.

## Design decisions
- CLI-first: all mutations go through the beads CLI, not direct DB writes.
- Stateful widgets: widgets are mostly stateless and accept separate state
  structs, following ratatui patterns.
- View isolation: each view owns its own state to limit coupling.
- Predictable rendering: render functions avoid allocation-heavy work.
- Defensive parsing: CLI responses are validated before entering AppState.

## Error handling and feedback
- CLI failures surface as notifications rather than panics.
- Long operations show progress indicators and can be canceled.

## Testing alignment
Architecture supports snapshot tests for UI, unit tests for models and
helpers, and integration tests for CLI workflows. See TEST_STRATEGY.md.
