#!/usr/bin/env bash
# Generate deterministic test data fixtures for beads-tui
#
# Usage:
#   ./generate-fixtures.sh [fixture-name]
#
# Arguments:
#   fixture-name    Specific fixture to generate (test-small, test-medium, test-large, test-deps, test-edge, or all)
#
# Examples:
#   ./generate-fixtures.sh                  # Generate all fixtures
#   ./generate-fixtures.sh test-small       # Generate only the small fixture

set -e

# Color codes
CYAN='\033[0;36m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${CYAN}→ $1${NC}"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
}

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FIXTURES_DIR="$SCRIPT_DIR/../tests/fixtures"

# Create fixtures directory if it doesn't exist
mkdir -p "$FIXTURES_DIR"

# Fixture descriptions
get_description() {
    case "$1" in
        test-small) echo "Small dataset for smoke tests and basic functionality validation" ;;
        test-medium) echo "Medium dataset for standard integration testing and realistic workflows" ;;
        test-large) echo "Large dataset for performance testing and scalability validation" ;;
        test-deps) echo "Dependency-heavy dataset for graph visualization and dependency management testing" ;;
        test-edge) echo "Edge cases and boundary conditions for error handling validation" ;;
        *) echo "Test fixture for beads-tui" ;;
    esac
}

# Fixture focus areas (as JSON array)
get_focus_areas() {
    case "$1" in
        test-small) echo '["Basic CRUD operations","Simple filtering","Quick validation"]' ;;
        test-medium) echo '["Full workflows","Filter validation","Dependency graphs"]' ;;
        test-large) echo '["Performance benchmarks","Render optimization","Large lists"]' ;;
        test-deps) echo '["Dependency trees","Cycle detection","Critical paths"]' ;;
        test-edge) echo '["Input validation","Error handling","Unicode support"]' ;;
        *) echo '[]' ;;
    esac
}

# Get fixture statistics
get_stats() {
    local total=$(bd list --status all --limit 0 2>/dev/null | grep -E '^○|^✓' | wc -l || echo 0)
    local open=$(bd list --status open --limit 0 2>/dev/null | grep '^○' | wc -l || echo 0)
    local closed=$(bd list --status closed --limit 0 2>/dev/null | grep '^✓' | wc -l || echo 0)
    local blocked=$(bd blocked 2>/dev/null | grep '^○' | wc -l || echo 0)

    cat << EOF
{
  "total": $total,
  "open": $open,
  "closed": $closed,
  "blocked": $blocked,
  "dep_count": 0,
  "max_depth": 0
}
EOF
}

# Generate fixture
generate_fixture() {
    local name="$1"
    local fixture_path="$FIXTURES_DIR/$name"

    section "Generating $name"

    # Remove old fixture if it exists
    if [ -d "$fixture_path" ]; then
        info "Removing old fixture at $fixture_path"
        rm -rf "$fixture_path"
    fi

    # Create fixture directory
    mkdir -p "$fixture_path"

    # Initialize beads database in fixture
    cd "$fixture_path"

    info "Initializing beads database"
    bd init > /dev/null 2>&1

    # Generate fixture data
    info "Generating fixture data"
    case "$name" in
        test-small) generate_test_small ;;
        test-medium) generate_test_medium ;;
        test-large) generate_test_large ;;
        test-deps) generate_test_deps ;;
        test-edge) generate_test_edge ;;
    esac

    # Export to JSONL
    info "Exporting to JSONL"
    bd export --format jsonl --output issues.jsonl > /dev/null 2>&1

    # Get statistics
    local stats=$(get_stats)
    local total=$(echo "$stats" | jq -r '.total')

    # Create fixture manifest
    info "Creating fixture manifest"
    cat > fixture.json << EOF
{
  "name": "$name",
  "description": "$(get_description $name)",
  "issue_count": $total,
  "created": "$(date +%Y-%m-%d)",
  "version": "1.0.0",
  "statistics": {
    "open": $(echo "$stats" | jq -r '.open'),
    "closed": $(echo "$stats" | jq -r '.closed'),
    "blocked": $(echo "$stats" | jq -r '.blocked'),
    "by_type": {
      "task": 0,
      "bug": 0,
      "feature": 0,
      "epic": 0,
      "chore": 0
    },
    "by_priority": {
      "P0": 0,
      "P1": 0,
      "P2": 0,
      "P3": 0,
      "P4": 0
    }
  },
  "dependencies": {
    "total": 0,
    "max_depth": 0,
    "cycles": 0
  },
  "focus_areas": $(get_focus_areas $name),
  "known_issues": []
}
EOF

    # Create README
    cat > README.md << EOF
# $name

$(get_description $name)

## Statistics

- Total Issues: $total
- Open: $(echo "$stats" | jq -r '.open')
- Closed: $(echo "$stats" | jq -r '.closed')
- Blocked: $(echo "$stats" | jq -r '.blocked')

## Usage

\`\`\`rust
let (temp_dir, db_path) = setup_fixture("$name");
\`\`\`

See docs/TEST_DATA.md for more information.
EOF

    success "Generated $name with $total issues"
}

# Fixture generators

generate_test_small() {
    # Create 12 issues with simple dependencies

    # Epic
    local epic1=$(bd create --title "User Authentication" --type epic --priority 1 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    # Tasks for epic
    local task1=$(bd create --title "Implement login form" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local task2=$(bd create --title "Add password validation" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local task3=$(bd create --title "Create session management" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    # Bugs
    local bug1=$(bd create --title "Login button not clickable" --type bug --priority 0 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local bug2=$(bd create --title "Password field shows plain text" --type bug --priority 1 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    # Features
    bd create --title "Add remember me checkbox" --type feature --priority 3 > /dev/null 2>&1
    bd create --title "Implement OAuth login" --type feature --priority 3 > /dev/null 2>&1

    # Chore
    bd create --title "Update authentication docs" --type chore --priority 4 > /dev/null 2>&1

    # More tasks
    local task4=$(bd create --title "Write tests for login" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local task5=$(bd create --title "Add error messages" --type task --priority 3 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    # Create dependencies
    [ -n "$bug1" ] && [ -n "$task1" ] && bd dep add "$bug1" "$task1" 2>/dev/null || true
    [ -n "$bug2" ] && [ -n "$task2" ] && bd dep add "$bug2" "$task2" 2>/dev/null || true
    [ -n "$task3" ] && [ -n "$task1" ] && bd dep add "$task3" "$task1" 2>/dev/null || true
    [ -n "$task4" ] && [ -n "$task1" ] && bd dep add "$task4" "$task1" 2>/dev/null || true
    [ -n "$task4" ] && [ -n "$task2" ] && bd dep add "$task4" "$task2" 2>/dev/null || true

    # Close some issues
    [ -n "$task1" ] && bd close "$task1" --reason "Completed" 2>/dev/null || true
    [ -n "$bug1" ] && bd close "$bug1" --reason "Fixed" 2>/dev/null || true
    [ -n "$task5" ] && bd close "$task5" --reason "Done" 2>/dev/null || true
}

generate_test_medium() {
    info "Generating medium dataset (this may take a minute)..."

    # Create 3 epics and tasks
    for i in {1..3}; do
        bd create --title "Epic $i: Feature Area $i" --type epic --priority 1 > /dev/null 2>&1

        # Create 8-10 tasks per epic
        for j in {1..9}; do
            local priority=$((RANDOM % 3 + 1))
            local task_id=$(bd create --title "Task $i.$j" --type task --priority $priority 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

            # Randomly close 40% of tasks
            if [ $((RANDOM % 100)) -le 40 ] && [ -n "$task_id" ]; then
                bd close "$task_id" --reason "Completed" 2>/dev/null || true
            fi
        done
    done

    # Add bugs
    for i in {1..10}; do
        local priority=$((RANDOM % 3))
        bd create --title "Bug $i: Issue description" --type bug --priority $priority > /dev/null 2>&1
    done

    # Add features
    for i in {1..8}; do
        bd create --title "Feature $i" --type feature --priority 2 > /dev/null 2>&1
    done

    # Add chores
    for i in {1..5}; do
        bd create --title "Chore $i" --type chore --priority 3 > /dev/null 2>&1
    done
}

generate_test_large() {
    info "Generating large dataset (this will take several minutes)..."

    # Create 15 epics
    for i in {1..15}; do
        bd create --title "Epic $i: Major Feature Area" --type epic --priority 1 > /dev/null 2>&1
    done

    # Create 200 tasks
    for i in {1..200}; do
        local priority=$((RANDOM % 4 + 1))
        local task_id=$(bd create --title "Task $i: Detailed task description" --type task --priority $priority 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

        # Close 60% of tasks
        if [ $((RANDOM % 100)) -le 60 ] && [ -n "$task_id" ]; then
            bd close "$task_id" --reason "Completed" 2>/dev/null || true
        fi

        # Progress indicator
        if [ $((i % 20)) -eq 0 ]; then
            info "Created $i/200 tasks..."
        fi
    done

    # Create 50 bugs
    for i in {1..50}; do
        local priority=$((RANDOM % 4))
        bd create --title "Bug $i: Issue with detailed description" --type bug --priority $priority > /dev/null 2>&1

        if [ $((i % 10)) -eq 0 ]; then
            info "Created $i/50 bugs..."
        fi
    done

    # Create 30 features
    for i in {1..30}; do
        bd create --title "Feature $i: New capability" --type feature --priority 2 > /dev/null 2>&1
    done

    # Create 20 chores
    for i in {1..20}; do
        bd create --title "Chore $i: Maintenance task" --type chore --priority 3 > /dev/null 2>&1
    done
}

generate_test_deps() {
    info "Generating dependency-heavy dataset..."

    # Create a complex dependency tree
    local root=$(bd create --title "Root Epic" --type epic --priority 1 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    # Level 1: 4 tasks
    declare -a level1
    for i in {1..4}; do
        level1[i]=$(bd create --title "Level 1 Task $i" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    done

    # Level 2: 8 tasks depending on level 1
    declare -a level2
    for i in {1..8}; do
        level2[i]=$(bd create --title "Level 2 Task $i" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
        local parent_idx=$((RANDOM % 4 + 1))
        [ -n "${level2[i]}" ] && [ -n "${level1[parent_idx]}" ] && bd dep add "${level2[i]}" "${level1[parent_idx]}" 2>/dev/null || true
    done

    # Level 3: 12 tasks depending on level 2
    for i in {1..12}; do
        local task_id=$(bd create --title "Level 3 Task $i" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
        local parent_idx=$((RANDOM % 8 + 1))
        [ -n "$task_id" ] && [ -n "${level2[parent_idx]}" ] && bd dep add "$task_id" "${level2[parent_idx]}" 2>/dev/null || true
    done

    # Add diamond dependencies
    local diamond1=$(bd create --title "Diamond Task 1" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local diamond2=$(bd create --title "Diamond Task 2" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    local diamond_child=$(bd create --title "Diamond Child" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")

    [ -n "$diamond_child" ] && [ -n "$diamond1" ] && bd dep add "$diamond_child" "$diamond1" 2>/dev/null || true
    [ -n "$diamond_child" ] && [ -n "$diamond2" ] && bd dep add "$diamond_child" "$diamond2" 2>/dev/null || true
}

generate_test_edge() {
    info "Generating edge case dataset..."

    # Empty title (may not be allowed)
    bd create --title " " --type task --priority 2 > /dev/null 2>&1 || true

    # Very long title
    bd create --title "$(printf 'A%.0s' {1..200})" --type task --priority 2 > /dev/null 2>&1 || true

    # Unicode and emoji
    bd create --title "Task with emoji 🚀 🎉 ✨" --type task --priority 2 > /dev/null 2>&1
    bd create --title "Task with unicode: 日本語 中文 한글 العربية" --type task --priority 2 > /dev/null 2>&1

    # Special characters
    bd create --title "Task with <HTML> & \"quotes\" and \\backslashes\\" --type task --priority 2 > /dev/null 2>&1

    # All priorities
    bd create --title "P0 Critical" --type bug --priority 0 > /dev/null 2>&1
    bd create --title "P1 High" --type task --priority 1 > /dev/null 2>&1
    bd create --title "P2 Medium" --type task --priority 2 > /dev/null 2>&1
    bd create --title "P3 Low" --type chore --priority 3 > /dev/null 2>&1
    bd create --title "P4 Backlog" --type task --priority 4 > /dev/null 2>&1

    # Closed immediately
    local closed_id=$(bd create --title "Immediately closed" --type task --priority 2 2>&1 | grep -o 'beads-[a-z0-9]*' | head -1 || echo "")
    [ -n "$closed_id" ] && bd close "$closed_id" --reason "Closed immediately" 2>/dev/null || true

    # Whitespace variations
    bd create --title $'Task\twith\ttabs' --type task --priority 2 > /dev/null 2>&1 || true
    bd create --title $'Task\nwith\nnewlines' --type task --priority 2 > /dev/null 2>&1 || true
}

# Main execution

FIXTURE="${1:-all}"

case "$FIXTURE" in
    all)
        generate_fixture "test-small"
        generate_fixture "test-medium"
        generate_fixture "test-large"
        generate_fixture "test-deps"
        generate_fixture "test-edge"
        ;;
    test-small|test-medium|test-large|test-deps|test-edge)
        generate_fixture "$FIXTURE"
        ;;
    *)
        echo "Unknown fixture: $FIXTURE"
        echo "Valid options: all, test-small, test-medium, test-large, test-deps, test-edge"
        exit 1
        ;;
esac

echo ""
success "Fixture generation complete!"
info "Fixtures location: $FIXTURES_DIR"
info "See docs/TEST_DATA.md for usage information"
