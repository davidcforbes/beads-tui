# View Testing Guide

This guide explains how to use the CLI-based view testing system for beads-tui. This system allows you to test, inspect, and validate all 11 views without requiring a real beads database.

## Table of Contents

- [Quick Start](#quick-start)
- [Available Views](#available-views)
- [Demo Datasets](#demo-datasets)
- [CLI Commands](#cli-commands)
- [Test Scripts](#test-scripts)
- [Use Cases](#use-cases)
- [CI Integration](#ci-integration)

## Quick Start

List all available views:
```bash
beads-tui --list-views
```

Open a specific view with demo data:
```bash
beads-tui --demo --view 2  # Opens Kanban view
```

Generate a snapshot for testing:
```bash
beads-tui --demo --view 2 --snapshot --output kanban.txt
```

Run an interactive tour of all views:
```bash
./scripts/view-tour.sh  # or view-tour.ps1 on Windows
```

## Available Views

| Index | Name         | Description                                    |
|-------|--------------|------------------------------------------------|
| 0     | Issues       | Main issue list view with filtering/sorting   |
| 1     | Split        | Split screen: issue list + detail view         |
| 2     | Kanban       | Kanban board grouped by status                 |
| 3     | Dependencies | Dependency tree visualization                  |
| 4     | Labels       | Label statistics and management                |
| 5     | Gantt        | Gantt chart timeline view                      |
| 6     | PERT         | PERT diagram for critical path analysis        |
| 7     | Molecular    | Molecular formulas and wisps management        |
| 8     | Statistics   | Database statistics dashboard                  |
| 9     | Utilities    | Database maintenance and utility tools         |
| 10    | Help         | Help and keyboard shortcuts                    |

## Demo Datasets

The demo mode provides several pre-generated datasets with realistic test data:

### small (default)
- **15 issues** with simple dependencies
- **Best for**: Screenshots, quick visual checks, development
- **Features**: Simple epic/feature/task/bug hierarchy
- **Labels**: Basic set (frontend, backend, auth, design, etc.)

### medium
- **50 issues** with realistic workflows
- **Best for**: Integration testing, workflow validation
- **Features**: Multiple epics, feature chains, complex dependencies
- **Labels**: Comprehensive set covering multiple project areas

### large
- **300 issues** for stress testing
- **Best for**: Performance validation, UI scalability testing
- **Features**: 10 epics, 30 features, 150 tasks, 80 bugs, 30 chores
- **Labels**: Extensive label coverage

### deps
- **60 issues** with complex dependency graphs
- **Best for**: Testing dependency view, PERT diagrams, Gantt charts
- **Features**: Multi-layer dependency trees, chains, blocked tasks
- **Labels**: Component-based labeling

### edge
- **25 issues** with edge cases
- **Best for**: UI robustness testing, rendering edge cases
- **Features**:
  - Unicode characters (Chinese, Russian, Arabic, Emoji)
  - Very long titles and descriptions
  - Special characters (\`\`, <>, &, \\, /)
  - Empty fields
  - Many labels per issue
  - Extreme time estimates
  - Duplicate titles

## CLI Commands

### List Views

Show all available views with descriptions:
```bash
beads-tui --list-views
```

### Demo Mode

Run in demo mode with generated test data:
```bash
beads-tui --demo [--dataset <TYPE>] [--view <INDEX>]
```

Examples:
```bash
# Default small dataset, Issues view
beads-tui --demo

# Medium dataset, Kanban view
beads-tui --demo --dataset medium --view 2

# Edge cases dataset, Dependencies view
beads-tui --demo --dataset edge --view 3
```

### Snapshot Mode

Render a view to text and save to file:
```bash
beads-tui --demo --view <INDEX> --snapshot [--output <FILE>] [--size <WxH>] [--dataset <TYPE>]
```

Examples:
```bash
# Generate snapshot with auto-generated filename
beads-tui --demo --view 2 --snapshot

# Custom output file and size
beads-tui --demo --view 2 --snapshot --output kanban_120x40.txt --size 120x40

# Large dataset at high resolution
beads-tui --demo --dataset large --view 0 --snapshot --size 160x50 --output issues_large.txt
```

**Supported sizes:**
- Width: 40-500 characters
- Height: 20-200 lines
- Common sizes: `80x24`, `120x40`, `160x50`

### Test All Views

Cycle through all 11 views with a specified duration per view:
```bash
beads-tui --demo --test-all-views [--test-duration <SECONDS>] [--dataset <TYPE>]
```

Examples:
```bash
# 2 seconds per view (default)
beads-tui --demo --test-all-views

# 5 seconds per view
beads-tui --demo --test-all-views --test-duration 5

# Quick 1-second tour with large dataset
beads-tui --demo --dataset large --test-all-views --test-duration 1
```

Press `q` to exit early.

## Test Scripts

### Comprehensive View Tests

Generate snapshots for all views at multiple sizes and datasets:

**PowerShell (Windows):**
```powershell
.\scripts\test-views.ps1 [release|debug]
```

**Bash (Linux/Mac):**
```bash
./scripts/test-views.sh [release|debug]
```

This script:
- Tests all 11 views
- At 3 different sizes (80x24, 120x40, 160x50)
- With 2 datasets (small, medium)
- Generates 66 snapshot files total
- Creates `test_output/` directory with all snapshots

Example output:
```
Testing dataset: small
--------------------------------------------------
  [1/66] View 0 @ 80x24 [OK]
  [2/66] View 0 @ 120x40 [OK]
  ...

Total tests:    66
Successful:     66
Failed:         0
Generated 66 snapshot files
```

### Interactive View Tour

Launch an interactive tour that cycles through all views:

**PowerShell (Windows):**
```powershell
.\scripts\view-tour.ps1 [-BuildMode release|debug] [-Duration <seconds>]
```

**Bash (Linux/Mac):**
```bash
./scripts/view-tour.sh [release|debug] [duration_seconds]
```

Examples:
```bash
# Default: 2 seconds per view, release build
./scripts/view-tour.sh

# 5 seconds per view, debug build
./scripts/view-tour.sh debug 5
```

## Use Cases

### Development & Debugging

Quickly test a specific view during development:
```bash
beads-tui --demo --view 2
```

Generate a snapshot to inspect rendering:
```bash
beads-tui --demo --view 2 --snapshot --output debug.txt
cat debug.txt  # or type debug.txt on Windows
```

### Screenshot Generation

Generate screenshots for documentation:
```bash
# Small dataset for clean, simple screenshots
beads-tui --demo --dataset small --view 2 --snapshot --size 120x40 --output docs/kanban_view.txt

# All views at standard size
for i in {0..10}; do
    beads-tui --demo --view $i --snapshot --size 120x40 --output "docs/view_$i.txt"
done
```

### Regression Testing

Test that all views still render without errors:
```bash
./scripts/test-views.sh
```

Compare snapshots over time:
```bash
# Before changes
./scripts/test-views.sh
mv test_output test_output_before

# After changes
./scripts/test-views.sh
diff -r test_output_before test_output
```

### UI/UX Review

Review all views quickly:
```bash
beads-tui --demo --dataset medium --test-all-views --test-duration 3
```

### Performance Testing

Test with large dataset:
```bash
# Stress test with 300 issues
beads-tui --demo --dataset large --view 0

# Generate snapshot (may be slow)
time beads-tui --demo --dataset large --view 0 --snapshot
```

### Edge Case Testing

Test UI robustness:
```bash
# Unicode, long text, special characters
beads-tui --demo --dataset edge --view 0

# Generate snapshots of all edge cases
for i in {0..10}; do
    beads-tui --demo --dataset edge --view $i --snapshot --output "edge_view_$i.txt"
done
```

## CI Integration

### GitHub Actions

Add to your CI workflow (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  test-views:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: cargo build --release

      - name: Test all views
        run: ./scripts/test-views.sh release

      - name: Upload snapshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: view-snapshots
          path: test_output/
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
test-views:
  stage: test
  script:
    - cargo build --release
    - ./scripts/test-views.sh release
  artifacts:
    when: on_failure
    paths:
      - test_output/
```

### Local Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Testing views before commit..."
cargo build --release && ./scripts/test-views.sh release

if [ $? -ne 0 ]; then
    echo "View tests failed! Commit aborted."
    exit 1
fi
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting

### Binary not found

If you see "Binary not found" errors, build the project first:
```bash
cargo build --release
```

Or specify debug mode:
```bash
./scripts/test-views.sh debug
```

### Terminal size errors

If you see "Invalid size format" or dimension errors:
- Width must be 40-500
- Height must be 20-200
- Format must be `WIDTHxHEIGHT` (e.g., `120x40`)

### Dataset errors

Valid dataset types: `small`, `medium`, `large`, `deps`, `edge`

Check your spelling:
```bash
# Wrong
beads-tui --demo --dataset SMALL

# Correct
beads-tui --demo --dataset small
```

### View index out of range

Valid view indices: 0-10

Check available views:
```bash
beads-tui --list-views
```

## Advanced Usage

### Custom Test Workflow

Create a custom test script:

```bash
#!/bin/bash
# custom-test.sh

# Build
cargo build --release

# Test specific views of interest
VIEWS=(0 2 3 8)  # Issues, Kanban, Dependencies, Statistics

for view in "${VIEWS[@]}"; do
    echo "Testing view $view..."
    beads-tui --demo --dataset medium --view $view --snapshot \
        --size 120x40 --output "test_view_$view.txt"

    # Validate output
    if [ -s "test_view_$view.txt" ]; then
        echo "  OK"
    else
        echo "  FAILED: Empty output"
        exit 1
    fi
done

echo "All tests passed!"
```

### Automated Screenshot Generation

Generate all view screenshots for documentation:

```bash
#!/bin/bash
# generate-screenshots.sh

mkdir -p docs/screenshots

for i in {0..10}; do
    beads-tui --demo --dataset small --view $i --snapshot \
        --size 120x40 --output "docs/screenshots/view_$i.txt"
done

echo "Screenshots saved to docs/screenshots/"
```

## Tips & Best Practices

1. **Use appropriate datasets**: `small` for dev, `medium` for testing, `large` for performance
2. **Save snapshots**: Use `--snapshot` to save reference renders for comparison
3. **Automate testing**: Add view tests to your CI pipeline
4. **Test edge cases**: Regularly test with the `edge` dataset to catch UI bugs
5. **Check all sizes**: Test at multiple terminal sizes (80x24, 120x40, 160x50)
6. **Version snapshots**: Keep snapshot files in git to track UI changes over time

## Related Documentation

- [Beads TUI User Guide](../README.md)
- [Development Guide](./DEVELOPMENT.md)
- [Testing Guide](./TESTING.md)

## Support

For issues or questions:
- Open an issue on GitHub
- Check existing documentation
- Run `beads-tui --help` for CLI help
