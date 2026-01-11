# Development Environment Setup

This guide covers setting up a complete Rust development and debugging environment for beads-tui.

## Current Status

✅ Rust 1.92.0 installed
✅ Cargo configured with all dependencies
✅ Git repository initialized
✅ Beads database with 1,031 issues
✅ Basic proof-of-concept TUI application

## Recommended Development Tools

### 1. Essential Cargo Extensions

Install these cargo subcommands for enhanced development workflow:

```powershell
# Watch for file changes and auto-rebuild
cargo install cargo-watch

# Enhanced code formatting
# (already available via rustfmt)

# Advanced linting
# (already available via clippy)

# Security audit for dependencies
cargo install cargo-audit

# Check for outdated dependencies
cargo install cargo-outdated

# Code coverage (Windows-compatible)
cargo install cargo-llvm-cov

# Macro expansion debugging
cargo install cargo-expand

# Better dependency tree visualization
# (already available via cargo tree)

# Benchmark runner (already in dev-dependencies via criterion)
```

### 2. VS Code Extensions (Recommended)

If using VS Code, install these extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",           // Rust language server
    "vadimcn.vscode-lldb",               // Debugger
    "tamasfe.even-better-toml",          // TOML syntax
    "serayuzgur.crates",                 // Cargo.toml helper
    "swellaby.vscode-rust-test-adapter", // Test explorer
    "usernamehw.errorlens",              // Inline errors
    "mutantdino.resourcemonitor",        // Resource usage
    "wayou.vscode-todo-highlight"        // TODO highlighting
  ]
}
```

### 3. Debugger Configuration

Create `.vscode/launch.json` for debugging:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug beads-tui",
      "cargo": {
        "args": ["build", "--bin=beads-tui"],
        "filter": {
          "name": "beads-tui",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "RUST_BACKTRACE": "1"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug tests",
      "cargo": {
        "args": ["test", "--no-run"],
        "filter": {
          "name": "beads-tui",
          "kind": "lib"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### 4. VS Code Settings

Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true,
  "rust-analyzer.lens.run.enable": true,
  "rust-analyzer.lens.debug.enable": true,
  "editor.formatOnSave": true,
  "editor.formatOnPaste": false,
  "files.watcherExclude": {
    "**/target/**": true,
    "**/.beads/**": true
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.tabSize": 4
  }
}
```

### 5. Git Configuration

Configure git line endings for Windows:

```powershell
git config core.autocrlf true
git config core.eol lf
```

### 6. Pre-commit Hooks

Create `.git/hooks/pre-commit` (PowerShell version):

```powershell
#!/usr/bin/env pwsh
# Run checks before allowing commit

Write-Host "Running pre-commit checks..." -ForegroundColor Cyan

# Format check
Write-Host "Checking formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "Formatting failed. Run 'cargo fmt' to fix." -ForegroundColor Red
    exit 1
}

# Clippy check
Write-Host "Running clippy..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "Clippy found issues." -ForegroundColor Red
    exit 1
}

# Tests
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test --all
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tests failed." -ForegroundColor Red
    exit 1
}

Write-Host "All checks passed!" -ForegroundColor Green
exit 0
```

Make it executable:
```powershell
# Note: On Windows, git hooks need to be .ps1 or .bat files
# The above is saved as pre-commit.ps1 and configured in git
```

## Development Workflows

### Daily Development

```powershell
# Start development with auto-rebuild on file changes
cargo watch -x check -x test -x run

# Or with clear screen between runs
cargo watch -c -x check -x test -x run

# Run with debug logging
$env:RUST_LOG="debug"; cargo run

# Run with trace logging for specific module
$env:RUST_LOG="beads_tui::ui=trace"; cargo run
```

### Code Quality

```powershell
# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with auto-fix (when possible)
cargo clippy --fix

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test --lib ui::widgets

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Performance Analysis

```powershell
# Run benchmarks (once created in benches/)
cargo bench

# Profile with flamegraph (requires cargo-flamegraph)
cargo flamegraph --bin beads-tui

# Check compile times
cargo clean
cargo build --timings

# Check binary size
cargo build --release
ls -l target\release\beads-tui.exe
```

### Building

```powershell
# Development build (fast, unoptimized)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check

# Clean build artifacts
cargo clean
```

## Debugging Tips

### Terminal TUI Debugging

Debugging TUI applications is tricky since they take over the terminal. Options:

**Option 1: Log to file**
```rust
use tracing_subscriber::fmt::writer::MakeWriter;
let file = std::fs::File::create("debug.log").unwrap();
tracing_subscriber::fmt()
    .with_writer(file)
    .init();
```

**Option 2: Use a second terminal**
```powershell
# Terminal 1: Run tui-logger or tail logs
tail -f debug.log

# Terminal 2: Run application
cargo run
```

**Option 3: VS Code debugger**
- Set breakpoints in VS Code
- Press F5 to start debugging
- The debugger will pause at breakpoints
- Step through code with F10/F11

### Common Issues

**Issue: Terminal not restored after panic**
```powershell
# Reset terminal manually
reset
# or in PowerShell
Clear-Host
```

**Issue: Colors not working**
```powershell
# Enable ANSI colors in Windows Terminal
$env:TERM="xterm-256color"
```

**Issue: Slow compile times**
```toml
# Add to Cargo.toml for faster incremental builds
[profile.dev.package."*"]
opt-level = 2  # Already configured
```

## Testing Strategy

### Test Organization

```text
tests/
├── integration/         # Integration tests
│   ├── issue_crud.rs
│   ├── dependency_tree.rs
│   └── filter_search.rs
├── ui/                  # UI snapshot tests
│   ├── widgets.rs
│   └── views.rs
└── fixtures/            # Test data
    ├── test_beads.db
    └── sample_issues.json
```

### Writing Tests

```rust
// Unit test in module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.selected_tab, 0);
    }
}

// Integration test in tests/
#[test]
fn test_issue_list_loads() {
    // Setup test beads database
    // Run application
    // Assert UI state
}
```

## Beads Workflow Integration

### Finding Work

```powershell
# See ready work
bd ready

# See all epics
bd list --type epic

# Start work on an issue
bd update <issue-id> --status in_progress

# View dependency tree
bd dep tree <epic-id>
```

### Tracking Progress

```powershell
# After completing a task
bd close <issue-id>

# Commit changes
git add .
git commit -m "feat: implement feature X

Closes bd-<issue-id>"

# Sync beads
bd sync --from-main
```

## Next Steps

1. **Run the proof-of-concept**:
   ```powershell
   cargo run
   ```
   Press Tab to switch between tabs, 'q' to quit

2. **Start with first task**:
   ```powershell
   bd ready  # Find ready work
   ```

3. **Set up your preferred IDE** with extensions

4. **Create your first branch**:
   ```powershell
   git checkout -b feature/bd-<issue-id>-short-description
   ```

5. **Begin development!**

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Documentation](https://ratatui.rs/)
- [Crossterm Docs](https://docs.rs/crossterm/)
- [Beads Repository](https://github.com/steveyegge/beads)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

## Troubleshooting

### Cargo build errors
```powershell
# Clear cargo cache
cargo clean
rm -r ~/.cargo/registry/cache
cargo build
```

### VS Code rust-analyzer issues
```powershell
# Restart rust-analyzer
# Ctrl+Shift+P > "Rust Analyzer: Restart Server"

# Or reload VS Code window
# Ctrl+Shift+P > "Developer: Reload Window"
```

### Git issues
```powershell
# Check git status
git status

# Reset to last commit
git reset --hard HEAD

# Clean untracked files
git clean -fd
```
