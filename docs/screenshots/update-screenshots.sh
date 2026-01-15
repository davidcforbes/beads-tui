#!/bin/bash
# Script to update screenshot placeholders in UI_REFERENCE.md with actual image references

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="$(dirname "$SCRIPT_DIR")"
UI_REF="$DOCS_DIR/UI_REFERENCE.md"

echo "Updating screenshot references in UI_REFERENCE.md..."

# Create backup
cp "$UI_REF" "$UI_REF.backup"
echo "Created backup: UI_REFERENCE.md.backup"

# Count total placeholders
TOTAL=$(grep -c "Screenshot Placeholder:" "$UI_REF" || true)
echo "Found $TOTAL screenshot placeholders"

# Update placeholders to markdown images
# Transform: **Screenshot Placeholder:** `screenshot-XX-name.png`
# To: ![Description](screenshots/screenshot-XX-name.png)
sed -i 's/\*\*Screenshot Placeholder:\*\* `\(screenshot-[0-9][0-9]-[^`]*\.png\)`/![\1](screenshots\/\1)/g' "$UI_REF"

# Update caption format
# Transform: *Caption: Some description*
# To: *Some description*
sed -i 's/^\*Caption: \(.*\)\*$/\*\1\*/g' "$UI_REF"

# Count remaining placeholders
REMAINING=$(grep -c "Screenshot Placeholder:" "$UI_REF" || true)

echo ""
echo "Update complete!"
echo "  Original placeholders: $TOTAL"
echo "  Remaining placeholders: $REMAINING"
echo "  Updated references: $((TOTAL - REMAINING))"

if [ $REMAINING -eq 0 ]; then
    echo ""
    echo "✅ All screenshot placeholders have been replaced!"
else
    echo ""
    echo "⚠️  Some placeholders remain - check for formatting issues"
fi

# Count actual screenshot files
SCREENSHOT_COUNT=$(ls "$SCRIPT_DIR"/screenshot-*.png 2>/dev/null | wc -l || echo "0")
echo ""
echo "Screenshot files in directory: $SCREENSHOT_COUNT / 51"

if [ "$SCREENSHOT_COUNT" -lt 51 ]; then
    MISSING=$((51 - SCREENSHOT_COUNT))
    echo "⚠️  Still need to capture $MISSING screenshots"
fi

echo ""
echo "Backup saved at: UI_REFERENCE.md.backup"
echo "To restore: cp UI_REFERENCE.md.backup UI_REFERENCE.md"
