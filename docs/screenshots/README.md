# Screenshot Capture Guide for Beads-TUI UI Reference

This directory contains screenshots for the comprehensive UI Reference documentation.

## Quick Start

1. **Setup test data**:
   ```bash
   cd beads-tui
   bd import < docs/tutorial_sample_data/issues.jsonl
   ```

2. **Launch beads-tui**:
   ```bash
   cargo run --release
   ```

3. **Capture screenshots** following the checklist below

4. **Save to this directory** with exact filenames

## Screenshot Checklist

### Main Views (14 screenshots)

- [ ] `screenshot-01-main-layout.png` - Main layout with tab bar and status bar
- [ ] `screenshot-02-status-bar.png` - Status bar close-up
- [ ] `screenshot-03-issues-view.png` - Issues view with issue table
- [ ] `screenshot-04-issues-view-filtered.png` - Issues view with filters
- [ ] `screenshot-05-issues-view-bulk-select.png` - Multiple issues selected
- [ ] `screenshot-06-search-active.png` - Search bar with results
- [ ] `screenshot-07-dependencies-view.png` - Dependency tree
- [ ] `screenshot-08-dependencies-expanded.png` - All nodes expanded
- [ ] `screenshot-09-dependency-cycle.png` - Circular dependency highlighted
- [ ] `screenshot-10-labels-view.png` - Label statistics
- [ ] `screenshot-11-labels-search.png` - Label search active
- [ ] `screenshot-12-database-view.png` - Database health metrics
- [ ] `screenshot-13-database-sync.png` - Sync operation in progress
- [ ] `screenshot-14-help-view.png` - Help view with shortcuts

### Help and Details (6 screenshots)

- [ ] `screenshot-15-help-search.png` - Help view with search filter
- [ ] `screenshot-16-issue-detail.png` - Full issue details
- [ ] `screenshot-17-issue-detail-dependencies.png` - Issue detail showing dependencies
- [ ] `screenshot-18-issue-editor-create.png` - Create issue form
- [ ] `screenshot-19-issue-editor-validation.png` - Form validation errors
- [ ] `screenshot-20-issue-editor-autocomplete.png` - Label autocomplete

### Advanced Views (6 screenshots)

- [ ] `screenshot-21-gantt-view.png` - Gantt chart timeline
- [ ] `screenshot-22-gantt-zoom.png` - Gantt zoomed to month view
- [ ] `screenshot-23-kanban-view.png` - Kanban board
- [ ] `screenshot-24-kanban-swimlanes.png` - Kanban with swimlanes
- [ ] `screenshot-25-pert-view.png` - PERT network diagram
- [ ] `screenshot-26-pert-critical-path.png` - PERT with critical path

### Molecular Operations (5 screenshots)

- [ ] `screenshot-27-formula-browser.png` - Formula browser
- [ ] `screenshot-28-pour-wizard.png` - Pour wizard interface
- [ ] `screenshot-29-wisp-manager.png` - Wisp template manager
- [ ] `screenshot-30-bonding-interface.png` - Issue bonding interface
- [ ] `screenshot-31-history-ops.png` - History operations

### Dialogs and Overlays (11 screenshots)

- [ ] `screenshot-32-column-manager.png` - Column customization dialog
- [ ] `screenshot-33-filter-builder.png` - Filter builder dialog
- [ ] `screenshot-34-filter-quick-select.png` - Filter quick select menu
- [ ] `screenshot-35-dependency-dialog.png` - Dependency management dialog
- [ ] `screenshot-36-label-picker.png` - Label picker with autocomplete
- [ ] `screenshot-37-confirmation-dialog.png` - Delete confirmation
- [ ] `screenshot-38-help-overlay.png` - Help keyboard shortcuts overlay
- [ ] `screenshot-39-notification-history.png` - Notification history panel
- [ ] `screenshot-40-issue-history.png` - Issue change timeline
- [ ] `screenshot-41-undo-history.png` - Undo/redo history overlay
- [ ] `screenshot-42-tab-bar.png` - Tab bar close-up

### Widgets and Components (9 screenshots)

- [ ] `screenshot-43-status-bar-detailed.png` - Status bar with all info
- [ ] `screenshot-44-toast-notification.png` - Toast notification message
- [ ] `screenshot-45-progress-indicator.png` - Progress bar during operation
- [ ] `screenshot-46-search-input.png` - Search with autocomplete
- [ ] `screenshot-47-tree-widget.png` - Tree widget structure
- [ ] `screenshot-48-markdown-viewer.png` - Markdown rendered text
- [ ] `screenshot-49-skeleton-loader.png` - Loading skeleton placeholder
- [ ] `screenshot-50-accessibility.png` - Accessibility mode enabled
- [ ] `screenshot-51-performance-stats.png` - Performance stats overlay

## Capture Instructions

### Terminal Setup

**Recommended terminal size:** 120 columns x 40 rows

```bash
# Resize terminal (if supported)
printf '\e[8;40;120t'
```

### Screenshot Tools

**Windows:**
- Windows Snipping Tool (Win+Shift+S)
- Windows Terminal: Right-click → Export screenshot
- [ShareX](https://getsharex.com/) (recommended)

**macOS:**
- Screenshot.app (Cmd+Shift+5)
- iTerm2: Select window with Cmd+Shift+4

**Linux:**
- gnome-screenshot
- flameshot
- scrot

### Navigation Map

```
Tab 1: Issues
├─ Press c → Create issue form (18, 19, 20)
├─ Press / → Search bar (06, 46)
├─ Press f → Filter builder (33)
├─ Press Ctrl+M → Column manager (32)
├─ Select issue + Enter → Issue detail (16, 17)
└─ Press Space → Bulk select (05)

Tab 2: Dependencies
├─ Default view → Tree (07)
├─ Press e → Expand all (08)
└─ Create cycle → Cycle detection (09)

Tab 3: Labels
├─ Default view → Stats (10)
└─ Press / → Search (11)

Tab 4: Database
├─ Default view → Stats (12)
└─ Press s → Sync (13)

Tab 5: Help
├─ Default view → Shortcuts (14)
└─ Press / → Search (15)

Overlays (Press ? anywhere)
├─ Help overlay (38)
├─ Notification history: Ctrl+N (39)
├─ Issue history: h in detail (40)
└─ Undo history: Ctrl+H (41)
```

### Tips for High-Quality Screenshots

1. **Consistent Size**: Use 120x40 terminal for all screenshots
2. **Clean Data**: Use sample data that looks realistic
3. **Focus**: Ensure the relevant UI element is visible
4. **Timing**: Capture animations at peak visibility
5. **Contrast**: Use default theme for consistency
6. **Text**: Ensure all text is readable
7. **Crop**: Remove unnecessary borders
8. **Format**: Save as PNG for best quality

### Post-Processing

Optional steps to improve screenshots:

1. **Crop**: Remove excess space around terminal
2. **Annotate**: Add arrows or highlights if needed
3. **Optimize**: Compress PNG files without quality loss
   ```bash
   # Using optipng
   optipng screenshot-*.png

   # Using pngquant
   pngquant --quality=80-95 screenshot-*.png
   ```

### Updating Documentation

Once screenshots are captured, update `UI_REFERENCE.md`:

**Before:**
```markdown
**Screenshot Placeholder:** `screenshot-01-main-layout.png`
*Caption: Main application layout showing tab bar, content area, and status bar*
```

**After:**
```markdown
![Main application layout](screenshots/screenshot-01-main-layout.png)
*Main application layout showing tab bar, content area, and status bar*
```

### Batch Update Script

Use this script to update all placeholders at once:

```bash
# update-screenshots.sh
#!/bin/bash

cd "$(dirname "$0")/.."

sed -i 's/\*\*Screenshot Placeholder:\*\* `\(screenshot-[0-9][0-9]-[^`]*\.png\)`/![\1](screenshots\/\1)/g' docs/UI_REFERENCE.md
sed -i 's/^\*Caption: \(.*\)\*$/\*\1\*/g' docs/UI_REFERENCE.md

echo "Screenshots updated in UI_REFERENCE.md"
```

## Progress Tracking

Track your progress:

```bash
# Count captured screenshots
ls screenshot-*.png 2>/dev/null | wc -l

# Expected: 51
```

## Questions?

See the full [UI_REFERENCE.md](../UI_REFERENCE.md) for detailed context on each screenshot.
