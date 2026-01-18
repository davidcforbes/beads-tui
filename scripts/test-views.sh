#!/bin/bash
# Test all views and generate snapshots

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

BUILD_MODE="${1:-release}"

echo -e "${CYAN}====================================${NC}"
echo -e "${CYAN}  Beads-TUI View Testing Suite${NC}"
echo -e "${CYAN}====================================${NC}"
echo ""

views=(0 1 2 3 4 5 6 7 8 9 10)
sizes=("80x24" "120x40" "160x50")
datasets=("small" "medium")

# Create output directory
mkdir -p test_output
echo -e "${GREEN}Created output directory: test_output/${NC}"
echo ""

# Build flag
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_FLAG="--release"
    BIN_PATH="target/release/beads-tui"
else
    BUILD_FLAG=""
    BIN_PATH="target/debug/beads-tui"
fi

echo -e "${CYAN}Using build mode: $BUILD_MODE${NC}"
echo -e "${CYAN}Binary path: $BIN_PATH${NC}"
echo ""

# Check if binary exists
if [ ! -f "$BIN_PATH" ]; then
    echo -e "${RED}Binary not found at $BIN_PATH${NC}"
    echo -e "${YELLOW}Building project...${NC}"
    cargo build $BUILD_FLAG
fi

echo -e "${CYAN}Starting view tests...${NC}"
echo ""

TOTAL_TESTS=$((${#views[@]} * ${#sizes[@]} * ${#datasets[@]}))
CURRENT_TEST=0
SUCCESS_COUNT=0
FAIL_COUNT=0

for dataset in "${datasets[@]}"; do
    echo -e "${MAGENTA}Testing dataset: $dataset${NC}"
    echo "$(printf '%.0s-' {1..50})"

    for view in "${views[@]}"; do
        for size in "${sizes[@]}"; do
            CURRENT_TEST=$((CURRENT_TEST + 1))
            OUTPUT="test_output/view_${view}_${dataset}_${size}.txt"

            printf "  [%d/%d] View %d @ %s " "$CURRENT_TEST" "$TOTAL_TESTS" "$view" "$size"

            if "$BIN_PATH" --demo --dataset "$dataset" --view "$view" --snapshot --size "$size" --output "$OUTPUT" >/dev/null 2>&1; then
                echo -e "[${GREEN}OK${NC}]"
                SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
            else
                echo -e "[${RED}FAILED${NC}]"
                FAIL_COUNT=$((FAIL_COUNT + 1))
            fi
        done
    done
    echo ""
done

echo ""
echo -e "${CYAN}====================================${NC}"
echo -e "${CYAN}  Test Summary${NC}"
echo -e "${CYAN}====================================${NC}"
echo "Total tests:    $TOTAL_TESTS"
echo -e "Successful:     ${GREEN}$SUCCESS_COUNT${NC}"
if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "Failed:         ${GREEN}$FAIL_COUNT${NC}"
else
    echo -e "Failed:         ${RED}$FAIL_COUNT${NC}"
fi
echo -e "Output directory: ${CYAN}test_output/${NC}"
echo ""

# Count generated files
FILE_COUNT=$(ls -1 test_output/*.txt 2>/dev/null | wc -l)
echo -e "${CYAN}Generated $FILE_COUNT snapshot files${NC}"

if [ $FAIL_COUNT -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed. Check the output above for details.${NC}"
    exit 1
fi
