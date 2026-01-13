# Contributing to Beads-TUI

Thank you for your interest in contributing to Beads-TUI! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Issue Management](#issue-management)

## Code of Conduct

Be respectful, inclusive, and constructive. We welcome contributors of all skill levels.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Beads CLI installed and configured
- Git
- A GitHub account

### Setup

1. **Fork the repository**

   ```bash
   # Click "Fork" on GitHub, then clone your fork
   git clone https://github.com/davidcforbes/beads-tui
   cd beads-tui
   ```

2. **Add upstream remote**

   ```bash
   git remote add upstream https://github.com/davidcforbes/beads-tui
   ```

3. **Initialize beads**

   ```bash
   bd init
   python generate-issues.py  # Import work plan
   ```

4. **Install development tools**

   ```bash
   cargo install cargo-watch cargo-edit cargo-tarpaulin
   rustup component add rustfmt clippy
   ```

## Development Process

### 1. Find an Issue

```bash
# Find ready work in beads
bd ready --json

# Or browse GitHub issues
# https://github.com/davidcforbes/beads-tui/issues
```

Look for issues tagged with:

- `good first issue` - Great for beginners
- `help wanted` - Community help needed
- `documentation` - Writing and docs
- `bug` - Something isn't working

### 2. Claim the Issue

Comment on the GitHub issue to let others know you're working on it:

```text
I'd like to work on this issue.
```

Update beads status:

```bash
bd update <issue-id> --status in_progress --assignee your-name
```

### 3. Create a Branch

```bash
# Create feature branch from main
git checkout main
git pull upstream main
git checkout -b feature/bd-<id>-short-description
```

Branch naming convention:

- `feature/bd-123-add-filter-ui` - New features
- `fix/bd-456-search-crash` - Bug fixes
- `docs/bd-789-update-readme` - Documentation
- `refactor/bd-012-cleanup-parser` - Code refactoring
- `test/bd-345-add-widget-tests` - Tests

### 4. Make Changes

```bash
# Make your changes
code .

# Run in watch mode for fast feedback
cargo watch -x check -x test -x run
```

### 5. Commit Your Changes

Follow conventional commits format:

```bash
git add .
git commit -m "feat(ui): add filter builder widget

- Implement filter criteria UI
- Add live preview of filtered results
- Support AND/OR logic for labels

Closes bd-123"
```

Commit message format:

- `feat(scope): description` - New feature
- `fix(scope): description` - Bug fix
- `docs(scope): description` - Documentation
- `test(scope): description` - Tests
- `refactor(scope): description` - Code refactoring
- `chore(scope): description` - Maintenance

Scopes: `ui`, `beads`, `config`, `events`, `widgets`, `views`, `cli`

### 6. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_filter_builder

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Generate coverage report
cargo tarpaulin --out Html
```

### 7. Update Documentation

- Update README.md if adding features
- Add/update code comments
- Update WORKPLAN.md if completing tasks
- Add examples if appropriate

### 8. Push and Create PR

```bash
# Push to your fork
git push origin feature/bd-123-add-filter-ui

# Create pull request on GitHub
```

## Code Standards

See [CODE_STYLE.md](CODE_STYLE.md) for the project style guide.

### Rust Style

Follow Rust standard style:

```bash
cargo fmt
```

### Linting

Fix all clippy warnings:

```bash
cargo clippy --fix
```

### Code Organization

```text
src/
├── main.rs              # Entry point
├── app.rs               # Application state
├── ui/                  # UI components
│   ├── mod.rs
│   ├── layout.rs
│   ├── widgets/         # Reusable widgets
│   │   ├── mod.rs
│   │   ├── issue_list.rs
│   │   └── filter_builder.rs
│   └── views/           # Full-screen views
│       ├── mod.rs
│       ├── issues.rs
│       └── dependencies.rs
├── beads/               # Beads wrapper
│   ├── mod.rs
│   ├── client.rs
│   ├── models.rs
│   └── parser.rs
├── config.rs
├── events.rs
└── keybindings.rs
```

### Error Handling

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

fn read_config() -> Result<Config> {
    let path = config_path()
        .context("Failed to determine config path")?;

    std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {:?}", path))?
        .parse()
        .context("Failed to parse config")
}
```

### Naming Conventions

- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Descriptive names over short names

### Documentation

Add doc comments for public items:

```rust
/// Creates a new filter builder widget.
///
/// # Arguments
///
/// * `filters` - Initial filter criteria
///
/// # Examples
///
/// ```
/// let builder = FilterBuilder::new(vec![]);
/// ```
pub fn new(filters: Vec<FilterCriteria>) -> Self {
    // ...
}
```

## Testing

For the full testing strategy and plan, see TEST_STRATEGY.md and TEST_PLAN.md.
For harness and fixtures, see TEST_HARNESS.md and TEST_DATA.md.

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder_creation() {
        let builder = FilterBuilder::new(vec![]);
        assert_eq!(builder.filters().len(), 0);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/issue_management.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_create_issue() {
    let mut cmd = Command::cargo_bin("beads-tui").unwrap();
    cmd.assert().success();
}
```

### Property-Based Tests

Use proptest for edge cases:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_issue_id_valid(id in "[a-z0-9-]+") {
        assert!(is_valid_issue_id(&id));
    }
}
```

## Pull Request Process

### Before Submitting

- [ ] Tests pass: `cargo test`
- [ ] Linter passes: `cargo clippy`
- [ ] Formatted: `cargo fmt`
- [ ] Documentation updated
- [ ] Beads issue updated
- [ ] CHANGELOG.md updated (if applicable)

### PR Template

When creating a PR, include:

```markdown
## Description
Brief description of changes

## Related Issue
Closes bd-123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] Tests pass
- [ ] Linter passes
- [ ] Documentation updated
- [ ] Beads issue updated

## Screenshots (if applicable)
Terminal screenshots showing changes
```

### Review Process

1. **Automated checks** run (CI/CD)
2. **Code review** by maintainers
3. **Address feedback** by pushing new commits
4. **Approval** from at least one maintainer
5. **Merge** by maintainer

### After Merge

```bash
# Update beads
bd close <issue-id> --reason "Merged to main"
bd sync

# Update your fork
git checkout main
git pull upstream main
git push origin main

# Delete feature branch
git branch -d feature/bd-123-add-filter-ui
```

## Issue Management

### Using Beads for Tracking

All work is tracked in beads:

```bash
# Find work
bd ready

# Start work
bd update bd-123 --status in_progress --assignee yourname

# Link discovered work
bd create "Found edge case bug" -t bug -p 1 \
  --deps discovered-from:bd-123

# Complete work
bd close bd-123 --reason "Implemented and tested"

# Sync changes
bd sync
```

### Issue Labels

GitHub issues use these labels:

- `priority/0-critical` - Security, data loss
- `priority/1-high` - Major features, important bugs
- `priority/2-medium` - Nice-to-have features
- `priority/3-low` - Polish, optimization
- `type/bug` - Something isn't working
- `type/feature` - New functionality
- `type/task` - Work item
- `type/chore` - Maintenance
- `status/blocked` - Waiting on dependencies
- `good first issue` - Great for beginners
- `help wanted` - Community help needed

## Communication

### Where to Ask Questions

- **General questions**: GitHub Discussions
- **Bug reports**: GitHub Issues
- **Feature requests**: GitHub Issues (with proposal template)
- **Development chat**: Discord (if available)

### Getting Help

Stuck? Ask for help!

1. Check existing documentation
2. Search closed issues and PRs
3. Ask in GitHub Discussions
4. Ping maintainers in your PR

## Recognition

Contributors are recognized in:

- CONTRIBUTORS.md file
- Release notes
- Git commit history

Thank you for contributing! 🎉
