#!/usr/bin/env bash
set -euo pipefail

VERBOSE=false
NOCAPTURE=false

show_help() {
  cat << EOF
Validate test fixtures for beads-tui

Usage:
  ./scripts/validate-fixtures.sh [--verbose] [--nocapture]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
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
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

args=(test --test validate_fixtures)
if [ "$VERBOSE" = true ]; then
  args+=(--verbose)
fi
if [ "$NOCAPTURE" = true ]; then
  args+=(-- --nocapture)
fi

echo "Running: cargo ${args[*]}"
cargo "${args[@]}"
