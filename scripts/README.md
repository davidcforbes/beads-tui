# Test Runner Scripts

Cross-platform test runners for beads-tui that wrap `cargo test` with support for different test suites, fixtures, and snapshot updates.

## Quick Start

### Windows (PowerShell)

```powershell
# Run all tests
.\scripts\test.ps1

# Run specific test suite
.\scripts\test.ps1 unit
.\scripts\test.ps1 integration
.\scripts\test.ps1 snapshot
.\scripts\test.ps1 property

# Update UI snapshots
.\scripts\test.ps1 snapshot -UpdateSnapshots

# Use test fixture
.\scripts\test.ps1 -Fixture test-small

# Verbose output
.\scripts\test.ps1 unit -ShowVerbose -NoCapture
```

### Linux / macOS (Bash)

```bash
# Make script executable (first time only)
chmod +x scripts/test.sh

# Run all tests
./scripts/test.sh

# Run specific test suite
./scripts/test.sh unit
./scripts/test.sh integration
./scripts/test.sh snapshot
./scripts/test.sh property

# Update UI snapshots
./scripts/test.sh snapshot --update-snapshots

# Use test fixture
./scripts/test.sh --fixture test-small

# Verbose output
./scripts/test.sh unit --verbose --nocapture
```

## Test Suites

### Unit Tests (`unit`)
Runs all unit tests in the library code (`--lib`). These are the fastest tests and test individual functions and modules in isolation.

**Example:**
```bash
./scripts/test.sh unit
```

### Integration Tests (`integration`)
Runs integration tests that test the application as a whole. These tests are in the `tests/` directory.

**Example:**
```bash
./scripts/test.sh integration
```

### Snapshot Tests (`snapshot`)
Runs UI snapshot tests that compare rendered terminal output against saved snapshots. Useful for catching unexpected UI changes.

**Example:**
```bash
# Run snapshot tests
./scripts/test.sh snapshot

# Update snapshots after intentional UI changes
./scripts/test.sh snapshot --update-snapshots
```

### Property-Based Tests (`property`)
Runs property-based tests using `proptest`. These tests generate random inputs to find edge cases.

**Example:**
```bash
./scripts/test.sh property
```

### All Tests (`all`)
Runs all test suites in sequence. This is the default if no suite is specified.

**Example:**
```bash
./scripts/test.sh
# or explicitly:
./scripts/test.sh all
```

## Options

### Test Fixtures

Use `--fixture` (bash) or `-Fixture` (PowerShell) to set a specific test database:

```bash
# Bash
./scripts/test.sh --fixture test-small

# PowerShell
.\scripts\test.ps1 -Fixture test-small
```

This sets the `BD_DB` environment variable, which the beads CLI uses to determine which database to use.

### Snapshot Updates

When UI code changes intentionally, update the snapshot baselines:

```bash
# Bash
./scripts/test.sh snapshot --update-snapshots

# PowerShell
.\scripts\test.ps1 snapshot -UpdateSnapshots
```

This sets the `UPDATE_SNAPSHOTS=1` environment variable.

### Verbose Output

Show detailed cargo test output:

```bash
# Bash
./scripts/test.sh --verbose

# PowerShell
.\scripts\test.ps1 -ShowVerbose
```

### No Capture

Don't capture test output (shows `println!` statements immediately):

```bash
# Bash
./scripts/test.sh --nocapture

# PowerShell
.\scripts\test.ps1 -NoCapture
```

## Continuous Integration

These scripts are designed to work in CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Run tests
  run: ./scripts/test.sh all --verbose
```

## Exit Codes

- `0`: All tests passed
- `1`: One or more tests failed

## Environment Variables

The test scripts manage these environment variables:

- `BD_DB`: Test database/fixture name (set by `--fixture` flag)
- `UPDATE_SNAPSHOTS`: Enable snapshot update mode (set by `--update-snapshots` flag)

These are automatically cleaned up after tests complete.

## Validating Fixtures

Validate fixture layout and metadata before running integration tests:

```bash
./scripts/validate-fixtures.sh
```

```powershell
.\scripts\validate-fixtures.ps1
```

## Examples

### Quick feedback during development
```bash
# Run just the tests for the module you're working on
./scripts/test.sh unit --nocapture
```

### Before committing
```bash
# Run all tests to ensure nothing broke
./scripts/test.sh
```

### After UI changes
```bash
# Update snapshots for changed UI
./scripts/test.sh snapshot --update-snapshots

# Then run all tests to confirm
./scripts/test.sh
```

### Testing with different data
```bash
# Test with multiple fixtures
for fixture in test-small test-medium test-large; do
    echo "Testing with $fixture"
    ./scripts/test.sh --fixture $fixture
done
```

## Help

For full help and all options:

```bash
# Bash
./scripts/test.sh --help

# PowerShell
Get-Help .\scripts\test.ps1 -Full
```
