# Test Data Fixtures

Deterministic test datasets for beads-tui integration and end-to-end tests.

## Overview

Test fixtures provide consistent, reproducible data for testing beads-tui functionality. Each fixture is a complete beads database with known issues, dependencies, and metadata.

## Fixture Datasets

### 1. Small (`test-small`)

**Purpose**: Quick smoke tests and basic functionality validation
**Issue Count**: 10-15 issues
**Characteristics**:
- Mix of open/closed issues
- 2-3 simple dependency chains
- All issue types (task, bug, feature, epic, chore)
- All priority levels (P0-P4)
- Minimal labels and assignees
- No complex dependency trees

**Use Cases**:
- Basic CRUD operations
- Simple filtering and search
- Quick validation during development
- Performance baseline

### 2. Medium (`test-medium`)

**Purpose**: Standard integration testing and realistic workflows
**Issue Count**: 50-75 issues
**Characteristics**:
- Realistic distribution of issue states
- 5-8 dependency chains of varying depth
- Multiple epics with child tasks
- Some blocked issues
- Moderate label and assignee diversity
- Representative comments and metadata

**Use Cases**:
- Full workflow testing
- Filter and search validation
- List/detail view testing
- Dependency graph rendering

### 3. Large (`test-large`)

**Purpose**: Performance testing and scalability validation
**Issue Count**: 300-500 issues
**Characteristics**:
- Large issue database
- Multiple long dependency chains
- Many epics with extensive child hierarchies
- High label and assignee diversity
- Complex filtering scenarios
- Long descriptions and comment threads

**Use Cases**:
- Performance benchmarks
- Render optimization validation
- Large list pagination
- Search performance

### 4. Dependency-Heavy (`test-deps`)

**Purpose**: Dependency management and graph visualization testing
**Issue Count**: 40-60 issues
**Characteristics**:
- Complex dependency graphs
- Multiple levels of transitive dependencies
- Circular dependency detection scenarios
- Critical path testing
- Blocked/blocking relationship chains
- Diamond dependencies

**Use Cases**:
- Dependency tree rendering
- Cycle detection
- Critical path computation
- Blocked issue analysis
- Graph layout algorithms

### 5. Edge Cases (`test-edge`)

**Purpose**: Error handling and boundary condition testing
**Issue Count**: 20-30 issues
**Characteristics**:
- Empty fields
- Maximum length descriptions
- Unicode and emoji in all fields
- Issues with many labels/assignees
- Empty dependency graphs
- Malformed data (if supported)
- Timestamp edge cases

**Use Cases**:
- Input validation
- Error message testing
- Unicode handling
- Boundary condition validation
- Graceful degradation

## Fixture Format

Each fixture directory contains:

```
tests/fixtures/{fixture-name}/
  .beads/              # Full beads database
    issues/            # Issue JSONL files
    config.json        # Beads configuration
  fixture.json         # Manifest (see below)
  README.md            # Human-readable overview
```

## Fixture Manifest (`fixture.json`)

Each fixture includes a `fixture.json` manifest with metadata:

```json
{
  "name": "test-small",
  "description": "Small dataset for smoke tests",
  "issue_count": 12,
  "created": "2026-01-11",
  "version": "1.0.0",
  "statistics": {
    "open": 8,
    "closed": 4,
    "blocked": 2,
    "by_type": {
      "task": 5,
      "bug": 3,
      "feature": 2,
      "epic": 1,
      "chore": 1
    },
    "by_priority": {
      "P0": 1,
      "P1": 2,
      "P2": 4,
      "P3": 3,
      "P4": 2
    }
  },
  "dependencies": {
    "total": 8,
    "max_depth": 3,
    "cycles": 0
  },
  "focus_areas": [
    "Basic CRUD operations",
    "Simple filtering",
    "Dependency chains"
  ],
  "known_issues": []
}
```

## Using Fixtures in Tests

### Loading a Fixture

```rust
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

fn setup_fixture(name: &str) -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let fixture_src = PathBuf::from("tests/fixtures").join(name);
    let fixture_dst = temp.path().join(".beads");

    // Copy fixture to temp directory
    fs_extra::dir::copy(&fixture_src, &temp.path(), &Default::default()).unwrap();

    (temp, temp.path().to_path_buf())
}

#[test]
fn test_with_fixture() {
    let (temp_dir, db_path) = setup_fixture("test-small");

    // Set BD_DB to temp path
    std::env::set_var("BD_DB", db_path.to_str().unwrap());

    // Run your test...

    // Temp dir automatically cleaned up
}
```

### Asserting on Fixture Data

```rust
#[test]
fn test_list_issues() {
    let (temp_dir, db_path) = setup_fixture("test-small");
    std::env::set_var("BD_DB", db_path.to_str().unwrap());

    let manifest = load_fixture_manifest("test-small");

    let issues = list_all_issues();
    assert_eq!(issues.len(), manifest.issue_count);
}
```

## Maintenance

### Regenerating Fixtures

Fixtures can be regenerated from scripts:

```bash
# Generate all fixtures
./scripts/generate-fixtures.sh

# Generate specific fixture
./scripts/generate-fixtures.sh test-small
```

### Validating Fixtures

```bash
# Validate all fixtures
cargo test --test validate_fixtures

# Validate specific fixture
cargo test --test validate_fixtures -- test_small
```

## Best Practices

1. **Deterministic**: Fixtures should produce identical results on every run
2. **Version Control**: Commit fixtures to git for reproducibility
3. **Isolation**: Each test should use a temp copy of fixtures
4. **Documentation**: Update fixture manifests when changing data
5. **Minimal Size**: Keep fixtures as small as possible while covering scenarios
6. **Reset State**: Don't modify fixtures during tests without copying first

## Fixture Coverage Matrix

| Scenario | Small | Medium | Large | Deps | Edge |
|----------|-------|--------|-------|------|------|
| Basic CRUD | ✓ | ✓ | ✓ | ✓ | ✓ |
| Simple filters | ✓ | ✓ | ✓ | ✓ | ✓ |
| Complex filters | - | ✓ | ✓ | - | - |
| Dependency trees | ✓ | ✓ | ✓ | ✓✓✓ | - |
| Performance | - | - | ✓✓✓ | - | - |
| Edge cases | - | - | - | - | ✓✓✓ |
| Unicode/i18n | - | ✓ | ✓ | - | ✓✓✓ |

Legend: ✓ = covers, ✓✓✓ = primary focus, - = not applicable
