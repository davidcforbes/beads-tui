# Code Style Guide

This guide captures the house style for beads-tui. It complements the
contribution workflow in CONTRIBUTING.md.

## Formatting

- Use `cargo fmt` (rustfmt) for all Rust files.
- Keep line lengths reasonable; prefer readability over squeezing code.

## Linting

- Run `cargo clippy -- -D warnings` before landing changes.
- Address warnings instead of suppressing them unless there is a clear reason.

## Naming

- `snake_case` for functions, modules, and variables.
- `PascalCase` for types and traits.
- `SCREAMING_SNAKE_CASE` for constants.
- Prefer descriptive names to abbreviations.

## Errors and Results

- Use `anyhow` for application-level errors and `thiserror` for library types.
- Add context with `with_context` or `context` when returning errors.

## Tests

- New logic should include unit tests where it is easy and deterministic.
- Integration tests belong under `tests/`.
- Keep tests deterministic and avoid time-based flakiness.

## Documentation

- Public APIs should have doc comments (`///`).
- Update docs when behavior changes.
