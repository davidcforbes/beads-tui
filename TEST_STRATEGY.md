# Test Strategy

## Purpose
Beads-TUI should behave predictably across terminals, preserve data integrity
when calling the beads CLI, and remain fast for common workflows. This
strategy defines the test levels, environments, and gates needed to reach
that bar.

## Quality Goals
- Prevent data loss or corruption when creating, editing, or closing issues.
- Keep rendering consistent across terminals, themes, and window sizes.
- Ensure navigation and state transitions are reliable and keyboard-first.
- Maintain predictable performance for list loads, filters, and graphs.
- Keep tests deterministic so they can run locally and in CI.

## Scope
### In Scope
- UI state and rendering for all views and widgets.
- beads CLI integration, including error handling and retries.
- Data mapping between CLI JSON and internal models.
- Multi-instance behavior with separate workspaces.

### Out of Scope (for now)
- Network failure simulation for GitHub sync.
- Long-running daemon performance tuning.
- External integrations beyond beads CLI.

## Test Levels
### Unit Tests
- Pure logic modules: parsing, formatting, filtering, sorting, layout math.
- State reducers and event handlers.
- CLI command construction and response parsing.

### Integration Tests
- End-to-end workflows that invoke the real beads CLI against fixtures.
- CRUD operations, filters, dependency edits, sync and import/export.
- Multi-instance behavior with isolated fixture databases.

### UI Snapshot Tests
- Golden snapshots for key views, forms, and dialogs.
- Snapshots at multiple terminal sizes and themes.
- Snapshot diffs are reviewed as part of pull requests.

### Property-Based Tests
- Generative tests for filters, IDs, and dependency graphs.
- Randomized UI event sequences for state stability.

### Manual and Exploratory Tests
- Keyboard navigation and focus management.
- Visual inspection of complex layouts (kanban, gantt, pert).
- Cross-terminal smoke testing.

### Non-Functional Tests
- Performance baselines for startup, list render, and large datasets.
- Resource usage checks for memory growth during long sessions.

## Test Environments
- OS: Windows, Linux, MacOS.
- Terminals: Windows Terminal, Powershell, wezterm, iTerm2, GNOME Terminal (or equivalents).
- Terminal sizes: 80x24, 120x40, 160x50, 200x60.
- Themes: default, high contrast, light.

## Test Data
See TEST_DATA.md for fixture definitions, sizes, and generation notes.

## Automation and Tooling
See TEST_HARNESS.md for runner scripts, harness modules, and environment
variables. All suites should be runnable via a single entry point with flags.

## Gates and Metrics
- Unit and integration tests pass on all supported OS targets.
- Snapshot changes are reviewed and baselines updated intentionally.
- Coverage target: 80 percent on core modules once harness is in place.
- No P0 or P1 test failures before release.

## Risk Areas and Mitigations
- Terminal rendering differences: keep per-backend baselines and size variants.
- CLI timing and concurrency: use deterministic fixtures and retries where safe.
- Shared state leaks: reset state between tests and use isolated temp dirs.
- Daemon vs direct mode: test both with explicit toggles.

## Reporting
Test failures become beads issues with reproduction steps, environment data,
and expected vs actual behavior.

## Maintenance
Review the strategy quarterly or when adding major UI views or data flows.
