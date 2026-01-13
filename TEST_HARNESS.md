# Test Harness

## Goals
- Provide one command to run all test suites with consistent setup.
- Keep tests deterministic across terminals and platforms.
- Enable snapshot updates and fixture selection via flags.

## Components

### Runner Scripts
Create platform scripts that call cargo test with the right flags and
environment variables.

- `scripts/test.ps1` (Windows)
- `scripts/test.sh` (macOS and Linux)

Suggested flags:
- `--suite unit|integration|snapshot|property|all`
- `--fixture beads-mini|beads-small|beads-large`
- `--update-snapshots`
- `--profile` (optional performance timing)

### Rust Harness Module
Provide a small harness in `tests/support/` with helpers for:
- `spawn_app()` and `drive_events()` for UI state tests.
- `load_fixture_db()` to copy fixtures into temp directories.
- `run_cli()` to invoke beads with deterministic env settings.
- `render_view()` and `assert_frame()` for snapshot capture.

### Snapshot Backend
Use a test backend for ratatui and a snapshot library such as `insta`.
Store snapshots under `tests/snapshots/` and include size and theme in
snapshot names. Update snapshots only when `INSTA_UPDATE=always` is set
or when the test scripts pass the update flag.

### CLI Isolation
Default to direct storage mode to avoid cross-test interference:
- `BD_DB` points at the temp fixture DB.
- `BD_NO_DAEMON=1` for deterministic behavior.
Add a separate test run that enables the daemon for coverage.

## Environment Variables

| Name | Purpose |
| --- | --- |
| `BD_DB` | Path to the fixture database for the test run |
| `BD_NO_DAEMON` | Force direct storage mode (1 or 0) |
| `BEADS_TUI_TEST_MODE` | Disable animations and timing-dependent UI |
| `BEADS_TUI_SNAPSHOT_DIR` | Override snapshot output location |
| `BEADS_TUI_FIXTURE` | Fixture name for scripts and harness helpers |
| `INSTA_UPDATE` | Control snapshot updates (`always` to rewrite) |

## Directory Layout

```text
tests/
  support/                # Harness helpers
  integration/            # Integration tests
  ui/                     # UI snapshot tests
  fixtures/               # Test data
scripts/
  test.ps1                # Windows runner
  test.sh                 # macOS/Linux runner
```

## Design Notes
- Avoid global state in tests; always use temp directories.
- Capture terminal size in helpers to guarantee deterministic layouts.
- Keep snapshot updates behind explicit flags in scripts.
