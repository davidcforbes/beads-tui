#!/bin/bash
# Launch interactive tour of all views

BUILD_MODE="${1:-release}"
DURATION="${2:-2}"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Build flag
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_FLAG="--release"
    BIN_PATH="target/release/beads-tui"
else
    BUILD_FLAG=""
    BIN_PATH="target/debug/beads-tui"
fi

# Check if binary exists
if [ ! -f "$BIN_PATH" ]; then
    echo -e "${CYAN}Building project...${NC}"
    cargo build $BUILD_FLAG
fi

echo -e "${CYAN}Starting view tour ($DURATION seconds per view)${NC}"
echo -e "${GREEN}Press 'q' to exit early${NC}"
echo ""

"$BIN_PATH" --demo --test-all-views --test-duration "$DURATION"
