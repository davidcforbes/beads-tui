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
- Press `n` to create a new issue.
- Press `e` to edit the selected issue.
- Press `Shift+S` to change status.
- Press `p` to change priority.
- Press `l` to edit labels.
- Press `/` or `s` to open the search and filter bar.
- Press `c` to open the column manager.
- Press `Enter` to view details in split screen mode.
- Press `f` to toggle filters.

### Dependencies
Visualize and edit dependency graphs. Use the dependency dialog to add or
remove edges and verify there are no cycles.
- Press `+` to add a dependency.
- Press `-` to remove a dependency.
- Press `g` to view the dependency graph visualization.
- Press `c` to check for circular dependencies.

### Labels
Browse label usage, search for label patterns, and audit label health.
Use label dimensions like state:patrol to keep taxonomy consistent.
- Press `a` to add a new label.
- Press `e` to edit a label.
- Press `d` to delete a label.
- Press `s` to show label statistics.

### Kanban
Column-based workflow visualization with WIP limits.
- Navigate between cards with arrow keys or `h`/`j`/`k`/`l`.
- Press `Space` to move a card to the next column.
- Press `c` to configure board columns.
- Press `Ctrl+F1-F4` to toggle column collapse.

### Gantt
Timeline visualization with scheduling and date derivation.
- Navigate rows with `j`/`k` or arrow keys.
- Press `+`/`-` to zoom in/out.
- Press `g` to change grouping mode.
- Press `c` to configure chart settings.

### PERT
Network diagram for dependencies with critical path analysis.
- Navigate nodes with arrow keys.
- Press `+`/`-` to zoom in/out.
- Press `c` to configure chart settings.

### Database
Monitor database health, sync status, and operations.
- Press `r` or `F5` to refresh status.
- Press `s` to sync database with remote.
- Press `x` to export issues to JSONL.
- Press `i` to import issues from JSONL.
- Press `v` to verify database integrity.
- Press `c` to compact database.

### Help
The help overlay lists all current shortcuts and view-specific actions.
- Press `?` or `F1` to toggle help.
- Press `Left`/`Right` or `h`/`l` to switch help sections.

## Common tasks
### Create an issue
1. Press `n` in the Issues view.
2. Fill out the form fields (use `Tab` to move between fields).
3. Press `Enter` to save or `Esc` to cancel.

### Edit an issue
1. Select an issue in the list.
2. Press `e` to open the edit form.
3. Modify fields as needed.
4. Press `Enter` to save changes or `Esc` to cancel.

### View issue details
1. Select an issue in the list.
2. Press `Enter` to open split screen mode with details on the right.
3. Press `Enter` again to view full detail.
4. Press `Esc` or `q` to return to list view.

### Update status and priority
1. Select an issue.
2. Press `Shift+S` to open the status selector.
3. Press `p` to open the priority selector.
4. Use arrow keys to select and `Enter` to confirm.

### Apply labels
1. Select an issue.
2. Press `l` to open the label picker.
3. Toggle labels and press `Enter` to confirm.

### Filter and search
1. Press `/` or `s` to focus the search bar.
2. Enter text to search across all fields.
3. Press `Alt+Z` to toggle fuzzy search.
4. Press `Alt+R` to toggle regex search.
5. Press `f` to open the filter bar.
6. Press `Alt+S`/`Alt+P`/`Alt+T`/`Alt+L` for specific filter dialogs.
7. Press `Esc` to clear the search.

### Use saved filters
1. Press `Alt+F` to open the saved filters menu.
2. Press `Ctrl+Shift+S` to save the current filter.
3. Press `F3`-`F11` to quickly apply saved filters.

### Manage dependencies
1. Select an issue.
2. Press `+` to add a dependency (opens dependency dialog).
3. Press `-` to remove a dependency.
4. Switch to Dependencies view to see the full dependency tree.

### Undo and redo
1. Press `Ctrl+Z` to undo the last action.
2. Press `Ctrl+Y` to redo the last undone action.
3. The undo system tracks the last 50 commands.

### View notifications
1. Press `Ctrl+H` to open the notification history panel.
2. Review recent notifications and errors.
3. Press `Esc` to close the panel.

### Customize columns
1. Press `c` in the Issues view to open the column manager.
2. Toggle column visibility with checkboxes.
3. Reorder columns by moving them up/down.
4. Press `Enter` to apply changes.

## Advanced features

### Split screen mode
View issue details alongside the list without leaving the Issues view:
1. Press `Enter` on an issue to toggle split screen.
2. The detail panel appears on the right side.
3. Press `Alt+H` to view issue history in the detail panel.
4. Press `Esc` to close split screen and return to full list.

### Form system
All forms (Create, Edit, Detail) use a unified layout:
- `Tab` / `Shift+Tab` - Move between form fields
- `Ctrl+L` - Load description content from a file path
- `Ctrl+P` - Toggle preview mode (Create form only)
- `Enter` - Submit the form
- `Esc` - Cancel and close

### Theme support
Switch between 5 themes including accessibility themes:
- Dark (default)
- High-Contrast (WCAG AAA compliant)
- Deuteranopia (red-green colorblind friendly)
- Protanopia (red-green colorblind friendly)
- Tritanopia (blue-yellow colorblind friendly)

Configure the theme in your config file (see CONFIGURATION.md).

### Background tasks
Long-running operations run in the background with progress tracking:
- Watch for progress indicators in the status bar
- Operations can be canceled with `Ctrl+C` (in task context)
- Check notification history (`Ctrl+H`) for task completion status

## Tips and tricks

### Keyboard efficiency
- Learn the vim-style navigation (`j`/`k` for up/down, `g`/`G` for top/bottom)
- Use `/` or `s` for quick search access
- Memorize `F3`-`F11` for your most-used saved filters
- Use `Tab` number keys (`1`-`9`) to jump directly to a tab

### Workflow optimization
1. Set up saved filters for common views (In Progress, Blocked, My Issues)
2. Use the Kanban view for workflow visualization
3. Use the Gantt view for timeline planning
4. Use the PERT view to identify critical path and dependencies
5. Enable split screen mode for quick issue review

### Search power
- Use fuzzy search (`Alt+Z`) for approximate matching
- Use regex search (`Alt+R`) for complex patterns
- Combine text search with filter dialogs for precise results
- Save frequently-used search + filter combinations

## Troubleshooting

### Application not responding
- Press `Ctrl+C` to interrupt long operations
- Check Database view for sync status
- Verify beads CLI is working: `bd list` in terminal

### Data not updating
- Press `r` or `F5` to refresh the current view
- Switch to Database view and press `s` to sync
- Check notification history (`Ctrl+H`) for errors

### Performance issues
- Reduce the number of visible columns (column manager: `c`)
- Use filters to reduce the number of displayed issues
- Switch to Database view and press `c` to compact the database

## References
- [KEYBOARD_SHORTCUTS.md](../KEYBOARD_SHORTCUTS.md) - Complete keyboard reference
- [CONFIGURATION.md](CONFIGURATION.md) - Configuration options
- [FILTERING_GUIDE.md](FILTERING_GUIDE.md) - Advanced filter patterns
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture details
- [widgets.md](widgets.md) - UI widget catalog
