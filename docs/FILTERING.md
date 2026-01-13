# Filtering and Search Guide

This guide covers the comprehensive filtering and search capabilities in beads-tui.

## Quick Start

- Press `/` to focus the search input
- Type your search query
- Use column filters (status, priority, type, labels) to narrow results
- Press `Ctrl+S` to save the current filter
- Press `f` to open the quick-select menu
- Press `F1-F12` to apply saved filters with hotkeys

## Filter Types

### Column Filters

Column filters allow you to filter issues by specific fields:

#### Status Filter
Filter issues by their status:
- **Open** - Issues that are open and not yet started
- **In Progress** - Issues currently being worked on
- **Blocked** - Issues waiting on dependencies or external factors
- **Closed** - Completed issues

#### Priority Filter
Filter by priority level:
- **P0** - Critical priority (most urgent)
- **P1** - High priority
- **P2** - Medium priority (default)
- **P3** - Low priority
- **P4** - Backlog (lowest priority)

#### Type Filter
Filter by issue type:
- **Epic** - Large multi-issue initiatives
- **Feature** - New functionality
- **Task** - Concrete work items
- **Bug** - Defects or issues
- **Chore** - Maintenance work

#### Label Filters
Filter by one or more labels with flexible matching:
- **AND logic** - Issue must have ALL specified labels
- **OR logic** - Issue must have ANY of the specified labels
- Support for label aliases (e.g., "bug-fix" and "bugfix" treated as same)

#### Assignee Filter
- Filter by assignee username
- Special filter: **no-assignee** - Show unassigned issues only

### Search Text Filters

Search for text within issues with multiple modes:

#### Search Modes

1. **Plain Text Search** (default)
   - Simple substring matching
   - Case-insensitive by default
   - Example: `authentication` matches "Authentication", "authenticate", etc.

2. **Regex Search**
   - Enable with regex toggle
   - Full regular expression support
   - Example: `^[Bb]ug.*fix$` matches issues starting with "Bug" or "bug" and ending with "fix"

3. **Fuzzy Search**
   - Enable with fuzzy toggle
   - Approximate string matching
   - Handles typos and minor variations
   - Example: `autentication` still matches "authentication"

#### Search Scopes

Control which fields are searched:

- **All** (default) - Search across title, description, and notes
- **Title** - Search only in issue titles
- **Description** - Search only in issue descriptions
- **Notes** - Search only in issue notes

Press `Tab` while in search input to cycle through scopes.

## Smart Views

Smart views provide pre-configured filters for common workflows:

### All View
- Shows all issues regardless of status
- Default view when no filters applied

### Ready View
- Shows issues ready to work on
- Filters: Status = Open, No blocking dependencies
- Perfect for finding your next task

### Blocked View
- Shows issues that are blocked
- Filters: Status = Blocked OR has blocking dependencies
- Helps identify bottlenecks

### My Issues View
- Shows issues assigned to current user
- Filters: Assignee = current username
- Your personal task list

### Recently View
- Shows recently updated issues
- Filters: Updated within last 7 days
- Stay current with active work

### Stale View
- Shows issues not updated recently
- Filters: Updated more than 30 days ago (configurable)
- Identify neglected work

## Saved Filters

Save frequently used filter combinations for quick access.

### Creating Saved Filters

1. Apply desired filters (column filters, search, view, etc.)
2. Press `Ctrl+S` to open the save dialog
3. Enter a filter name
4. Optionally assign a hotkey (A-Z)
5. Press `Enter` to save

### Using Saved Filters

**Quick-Select Menu** (`f` key):
- Press `f` to open the filter quick-select menu
- Use `↑`/`↓` or `j`/`k` to navigate
- Press `1-9` for quick selection by number
- Press assigned hotkey letter to instantly apply
- Press `e` to edit selected filter
- Press `d` to delete selected filter (with confirmation)
- Press `Enter` to apply selected filter
- Press `Esc` to close menu

**Hotkey Access** (`F1-F12`):
- Assign F1-F12 hotkeys when saving filters
- Press `F1-F12` anywhere in Issues view to instantly apply
- Hotkeys displayed in quick-select menu

### Editing Filters

1. Press `f` to open quick-select menu
2. Navigate to desired filter
3. Press `e` to edit
4. Modify name or hotkey
5. Press `Enter` to save changes

### Deleting Filters

1. Press `f` to open quick-select menu
2. Navigate to filter to delete
3. Press `d` for delete
4. Confirm deletion in dialog

## Filter Persistence

All filters are automatically saved:
- **Saved Filters** - Stored in `~/.config/beads-tui/config.json`
- **Current Filters** - Remembered during session
- **View State** - Persisted across restarts

## Filter Combination Rules

Filters combine using AND logic:
- Column filters AND search text
- All active filters must match for issue to appear
- Within label filters, use AND/OR logic setting

Example:
```
Status: Open
Priority: P1
Labels: bug, ui (AND logic)
Search: "rendering"
```
Shows only P1 Open bugs with BOTH "bug" AND "ui" labels containing "rendering"

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Focus search input |
| `Esc` | Clear search / Unfocus search input |
| `Tab` | Cycle search scope (while focused) |
| `Ctrl+S` | Save current filter |
| `f` | Open filter quick-select menu |
| `F1-F12` | Apply saved filter by hotkey |
| `Ctrl+X` | Clear all filters |

### Quick-Select Menu Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate filters |
| `1-9` | Quick select by number |
| `a-z` or `A-Z` | Quick select by hotkey |
| `Enter` | Apply selected filter |
| `e` | Edit selected filter |
| `d` | Delete selected filter |
| `Esc` | Close menu |

## Advanced Techniques

### Combining Filters Effectively

1. **Start Broad, Narrow Down**
   - Begin with a Smart View (Ready, My Issues, etc.)
   - Add column filters to narrow scope
   - Use search for specific issues

2. **Label Organization**
   - Use hierarchical labels: `area:frontend`, `area:backend`
   - Combine with OR logic to see all areas
   - Use AND logic for specific combinations

3. **Priority Triage**
   - Filter by P0/P1 + Open status
   - Save as "Critical Work" filter
   - Use F1 hotkey for instant access

### Filter Patterns

**Daily Standup View**:
```
View: My Issues
Status: In Progress OR Blocked
Sort: Updated (newest first)
```

**Bug Triage**:
```
Type: Bug
Status: Open
Priority: P0 OR P1
Sort: Created (oldest first)
```

**Sprint Planning**:
```
View: Ready
Priority: P1 OR P2
Labels: sprint-candidate (OR)
```

**Code Review**:
```
Labels: needs-review
Assignee: (your username)
Status: In Progress
```

## Filter Syntax Summary

### IssueFilter Structure

Filters are stored as JSON with this structure:

```json
{
  "name": "Filter Name",
  "filter": {
    "status": "Open" | "InProgress" | "Blocked" | "Closed" | null,
    "priority": "P0" | "P1" | "P2" | "P3" | "P4" | null,
    "issue_type": "Epic" | "Feature" | "Task" | "Bug" | "Chore" | null,
    "assignee": "username" | null,
    "labels": ["label1", "label2"],
    "label_logic": "And" | "Or",
    "search_text": "query" | null,
    "search_scope": "All" | "Title" | "Description" | "Notes",
    "view_type": "All" | "Ready" | "Blocked" | "MyIssues" | "Recently" | "Stale",
    "use_regex": true | false,
    "use_fuzzy": true | false
  },
  "hotkey": "1" | null
}
```

## Troubleshooting

### Filters Not Working

1. **Check Filter Combination**
   - Multiple filters might be too restrictive
   - Try removing filters one by one
   - Press `Ctrl+X` to clear all and start fresh

2. **Regex Errors**
   - Invalid regex patterns are silently ignored
   - Test regex patterns in a regex tester first
   - Escape special characters: `\`, `.`, `*`, `+`, `?`, `^`, `$`, `{`, `}`, `[`, `]`, `|`, `(`, `)`

3. **No Results**
   - Check if view filter is too restrictive
   - Verify label names match exactly
   - Search is case-insensitive but must match substrings

### Performance Issues

- Complex regex patterns may slow search
- Disable fuzzy search for large datasets
- Use specific search scopes instead of "All"
- Combine multiple filters instead of complex regex

## Examples

### Example 1: Find Critical Bugs
```
Type: Bug
Priority: P0
Status: Open
Save as: "Critical Bugs" with F1 hotkey
```

### Example 2: My Current Sprint
```
View: My Issues
Status: Open OR In Progress
Labels: sprint-42
Save as: "Sprint 42" with F2 hotkey
```

### Example 3: Documentation Tasks
```
Type: Task
Labels: documentation (OR), docs (OR)
Status: Open
Save as: "Docs Work" with F3 hotkey
```

### Example 4: Stale P1/P2 Work
```
View: Stale
Priority: P1 OR P2
Status: Open
Save as: "Stale Priority" with F4 hotkey
```

### Example 5: Frontend Bugs
```
Type: Bug
Labels: frontend, ui (OR logic)
Status: Open
Search: (empty)
Save as: "Frontend Bugs" with F5 hotkey
```

## Best Practices

1. **Name Filters Clearly**
   - Use descriptive names: "P0 Bugs" not "Filter 1"
   - Include key criteria in name
   - Keep names under 20 characters

2. **Assign Hotkeys Strategically**
   - F1-F4: Most frequently used filters
   - F5-F8: Context-specific filters
   - F9-F12: Occasional use filters

3. **Regular Filter Maintenance**
   - Remove obsolete filters
   - Update filter names as workflow changes
   - Consolidate similar filters

4. **Share Filter Configs**
   - Export `~/.config/beads-tui/config.json`
   - Share with team members
   - Document team filter conventions

## Related Documentation

- [Keyboard Shortcuts](../KEYBOARD_SHORTCUTS.md) - All keyboard shortcuts
- [Label Management](labels.md) - Label system details
- [Configuration](../CONFIGURATION.md) - Config file reference
