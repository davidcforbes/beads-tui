# User Guide

## Getting started
1. Install beads CLI and initialize a beads workspace.
2. Run beads-tui from a directory that contains a .beads folder.
3. Press ? at any time to open the shortcut overlay.

## Navigation basics
- Tab and Shift+Tab switch between views.
- j/k or the arrow keys move selection.
- Enter opens details or activates actions.
- Esc dismisses dialogs or exits a sub-view.
- 1-9 jump directly to a tab by index.

See KEYBOARD_SHORTCUTS.md for the full list.

## Views
### Issues
The primary work area for creating, editing, and closing issues.
- Press c to create a new issue.
- Press e to edit the selected issue.
- Press Shift+S to change status.
- Press p to change priority.
- Press Shift+L to edit labels.
- Press / to open the search and filter bar.

### Dependencies
Visualize and edit dependency graphs. Use the dependency dialog to add or
remove edges and verify there are no cycles.

### Labels
Browse label usage, search for label patterns, and audit label health.
Use label dimensions like state:patrol to keep taxonomy consistent.

### Database
Monitor database health, daemon status, and sync state. Use this view when
troubleshooting performance or consistency issues.

### Help
The help overlay lists all current shortcuts and view-specific actions.

## Common tasks
### Create an issue
1. Press c in the Issues view.
2. Fill out the form fields.
3. Press Ctrl+S to save or Esc to cancel.

### Update status and priority
1. Select an issue.
2. Press Shift+S to open the status selector.
3. Press p to open the priority selector.

### Apply labels
1. Select an issue.
2. Press Shift+L to open the label picker.
3. Toggle labels and press Enter to confirm.

### Filter and search
1. Press / to focus the search bar.
2. Enter text, labels, or status queries.
3. Press Esc to clear the search.

## References
- CONFIGURATION.md for config options.
- docs/FILTERING_GUIDE.md for filter patterns.
- docs/labels.md for label naming guidance.
