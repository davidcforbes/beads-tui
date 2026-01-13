#!/usr/bin/env bash
# Cross-platform test runner for beads-tui
#
# Usage:
#   ./test.sh [suite] [options]
#
# Arguments:
#   suite           Test suite to run: unit, integration, snapshot, property, or all (default: all)
#
# Options:
#   --fixture NAME  Test fixture to use (sets BD_DB environment variable)
#   --update-snapshots  Enable snapshot update mode for UI tests
#   --verbose       Show detailed test output
#   --nocapture     Don't capture test output (shows println! immediately)
#   --help          Show this help message
#
# Examples:
#   ./test.sh                           # Run all tests
#   ./test.sh unit                      # Run only unit tests
#   ./test.sh snapshot --update-snapshots  # Run snapshot tests and update snapshots
#   ./test.sh --fixture test-data-1     # Run tests with specific fixture

set -e  # Exit on error

# Color codes
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${CYAN}→ $1${NC}"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

error() {
    echo -e "${RED}✗ $1${NC}"
}

section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
    echo ""
}

show_help() {
    cat << EOF
Cross-platform test runner for beads-tui

Usage:
  ./test.sh [suite] [options]

Arguments:
  suite           Test suite to run: unit, integration, snapshot, property, or all (default: all)

Options:
  --fixture NAME          Test fixture to use (sets BD_DB environment variable)
  --update-snapshots      Enable snapshot update mode for UI tests
  --verbose               Show detailed test output
  --nocapture             Don't capture test output (shows println! immediately)
  --help                  Show this help message

Examples:
  ./test.sh                                   # Run all tests
  ./test.sh unit                              # Run only unit tests
  ./test.sh snapshot --update-snapshots       # Run snapshot tests and update snapshots
  ./test.sh --fixture test-data-1             # Run tests with specific fixture
  ./test.sh all --verbose --nocapture         # Run all tests with verbose, uncaptured output

Test Suites:
  unit            Unit tests (cargo test --lib)
  integration     Integration tests (cargo test --test)
  snapshot        Snapshot tests (cargo test --test integration ui_snapshots)
  property        Property-based tests (cargo test --lib proptest)
  all             All test suites (default)
EOF
    exit 0
}

# Parse arguments
SUITE="all"
FIXTURE=""
UPDATE_SNAPSHOTS=false
VERBOSE=false
NOCAPTURE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --fixture)
            FIXTURE="$2"
            shift 2
            ;;
        --update-snapshots)
            UPDATE_SNAPSHOTS=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --nocapture)
            NOCAPTURE=true
            shift
            ;;
        --help|-h)
            show_help
            ;;
        unit|integration|snapshot|property|all)
            SUITE="$1"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment variables
if [ -n "$FIXTURE" ]; then
    info "Using test fixture: $FIXTURE"
    export BD_DB="$FIXTURE"
fi

if [ "$UPDATE_SNAPSHOTS" = true ]; then
    info "Snapshot update mode enabled"
    export UPDATE_SNAPSHOTS=1
    export INSTA_UPDATE=always
fi

# Build base cargo command
CARGO_ARGS="test"

if [ "$VERBOSE" = true ]; then
    CARGO_ARGS="$CARGO_ARGS --verbose"
fi

if [ "$NOCAPTURE" = true ]; then
    CARGO_ARGS="$CARGO_ARGS -- --nocapture"
fi

# Track test results
EXIT_CODE=0

# Run requested test suites
case $SUITE in
    unit)
        section "Running Unit Tests"
        info "Command: cargo $CARGO_ARGS --lib"
        if ! cargo $CARGO_ARGS --lib; then
            EXIT_CODE=1
        fi
        ;;

    integration)
        section "Running Integration Tests"
        info "Command: cargo $CARGO_ARGS --test '*'"
        if ! cargo $CARGO_ARGS --test '*'; then
            EXIT_CODE=1
        fi
        ;;

    snapshot)
        section "Running Snapshot Tests"
        info "Command: cargo $CARGO_ARGS --test integration ui_snapshots"
        if ! cargo $CARGO_ARGS --test integration ui_snapshots; then
            EXIT_CODE=1
        fi
        ;;

    property)
        section "Running Property-Based Tests"
        info "Command: cargo $CARGO_ARGS --lib proptest"
        if ! cargo $CARGO_ARGS --lib proptest; then
            EXIT_CODE=1
        fi
        ;;

    all)
        # Run all test types
        section "Running All Tests"

        # Unit tests
        info "1/4 - Unit tests"
        if ! cargo $CARGO_ARGS --lib; then
            EXIT_CODE=1
        fi
        echo ""

        # Integration tests
        info "2/4 - Integration tests"
        if ! cargo $CARGO_ARGS --test '*'; then
            EXIT_CODE=1
        fi
        echo ""

        # Snapshot tests
        info "3/4 - Snapshot tests"
        if ! cargo $CARGO_ARGS --test integration ui_snapshots; then
            EXIT_CODE=1
        fi
        echo ""

        # Property tests
        info "4/4 - Property-based tests"
        if ! cargo $CARGO_ARGS --lib proptest; then
            EXIT_CODE=1
        fi
        ;;

    *)
        error "Unknown suite: $SUITE"
        echo "Valid suites: unit, integration, snapshot, property, all"
        exit 1
        ;;
esac

# Clean up environment variables
unset BD_DB 2>/dev/null || true
unset UPDATE_SNAPSHOTS 2>/dev/null || true

# Summary
echo ""
if [ $EXIT_CODE -eq 0 ]; then
    success "All tests passed!"
else
    error "Some tests failed"
fi

exit $EXIT_CODE
