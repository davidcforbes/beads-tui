# Keyboard Shortcuts Reference Guide

This guide outlines the intended keyboard shortcuts for the application. Some shortcuts are
context-specific; the same key may do different things depending on the active view or dialog.

## UI Layout Overview

The beads-tui interface consists of five main sections:

1. **TITLE Container** (Rows 1-3): Displays project name, issue count summary (Open/In Progress/Blocked/Closed), active search status, and daemon status
2. **VIEWS Container** (Rows 4-6): Tab navigation bar for switching between views (Issues, Split, Kanban, Dependencies, Labels, Gantt, PERT, Molecular, Statistics, Utilities, Help)
3. **FILTERS Container** (Rows 7-9): Quick filter controls with hotkeys - Stat[u]s, T[y]pe, [L]abels, Pr[i]ority. This container appears directly below the VIEWS container
4. **Issues Section** (Row 10 onwards): Main content display area showing the current view (issue list, Kanban board, dependency tree, etc.)
5. **ACTIONS Container** (Bottom 3 rows): Context-sensitive keyboard shortcuts and navigation hints displayed in a columnar format with vertical separators

The FILTERS container (rows 7-9) provides quick access to common filters via single-key hotkeys. Press the letter shown in square brackets to activate each filter dropdown.

The ACTIONS Bar dynamically updates to show relevant shortcuts for the current view and context, organized into navigation actions and context-specific action items.

## Global Actions

| Key | Action   | Description                            |
| --- | -------- | -------------------------------------- |
|  1  | Issues   | Switch to the Issues View              |
|  2  | Split    | Switch to the Split View               |
|  3  | Split    | Switch to the Kanban View              |
|  4  | Depend   | Switch to the Dependencies View        |
|  5  | Labels   | Switch to the Labels View              |
|  6  | Ghantt   | Switch to the Ghantt  View             |
|  7  | Pert     | Switch to the Pert View                |
|  8  | Molecule | Switch to the Molecule View            |
|  9  | Stats    | Switch to the Statistics View          |
|  0  | Utilities| Switch to the Utilities View           |
|  ?  | Help     | Switch to the Help View                |

## Filter ACTIONS

| Key | Action   | Description                            |
| --- | -------- | -------------------------------------- |
|  S  | Status   | Open the Status Filter Dropdown        |
|  T  | Status   | Open the Type Filter Dropdown          |
|  P  | Priority | Open the Priority Filter Dropdown      |
|  L  | Labels   | Open the Labels Filter Dropdown        |
|  C  | Created  | Open the Created Date Filter Dropdown  |
|  U  | Updated  | Open the Updated Date Filter Dropdown  |
| F11 | Reset    | Reset the Filters to "All"             |

## List ACTIONS

| Key  | Action      | Description                            |
| ---- | ----------- | -------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                     |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                   |
| PgUp | Page Up 1   | Navigate up 1 Page of Rows             |
| PgDn | Page Down 1 | Navigate down 1 Page of Rows           |
| Home | List Top    | Navigate to the Top of the List        |
| End  | List Bottom | Navigate to the Bottom of the List     |

## Screen Actions
| Key  | Action      | Description                             |
| ---- | ----------- | --------------------------------------- |
| Ctrl+↑ | Scroll Up 1 Row | Scroll the Viewport up 1 Row      |
| Ctrl+↓ | Scroll Dn 1 Row | Scroll the Viewport down 1 Row    |
| Ctrl+→ | Scroll Rt 1 Row | Scroll the Viewport right 1 frame |
| Ctrl+← | Scroll Lf 1 Row | Scroll the Viewport left 1 frame  |
| F10    | Maximize Screen | Maximize the view to full screen  |
| F11    | Restore Screen | Restore view to normal screen size |
| F12    | Screen shot   | Save the screen to clipboard or file |


## Record ACTIONS

| Key | Action | Description                            |
| --- | ------ | -------------------------------------- |
|  R  | Read   | Read Mode for Selected Record Detail   |
|  N  | New    | Create Mode for New Record Detail      |
|  E  | Edit   | Edit Mode for Selected Record Detail   |
|  D  | Delete | Delete (Soft) The Selected Record      |
|  F  | Find   | Enter a Find String to Match Records   |
|  O  | Open   | Mark the Selected Record Status=Open   |
|  X  | Close  | Mark the Selected Record Status=Closed |
|  B  | Block  | Mark the Selected Record Status=Blocked|
|  Q  | Quit   | Exit the application                   |
| --- | ------ | -------------------------------------- |

| Key | Action | Description |
|---|---|---|
| `q`, `Ctrl+Q`, `Ctrl+C` | Quit | Exit the application |
| `?` | Shortcut Help | Toggle keyboard shortcuts overlay |
| `F1` | Context Help | Toggle context-sensitive help |
| `Ctrl+Z` | Undo | Undo last action |
| `Ctrl+Y` | Redo | Redo last undone action |
| `Esc` | Dismiss | Dismiss notification / close overlays |
| `Ctrl+H` | Notifications | Show notification history |

## Navigation
| Key | Action | Description |
|---|---|---|
| `Tab` | Next Tab | Switch to the next top-level tab |
| `Shift+Tab` | Prev Tab | Switch to the previous tab |
| `1`-`9` | Jump Tab | Switch directly to a tab by number (implemented for 1-5) |
| `Up`/`Down` or `j`/`k` | Move | Move selection up/down |
| `Left`/`Right` or `h`/`l` | Move | Move selection left/right or collapse/expand |
| `PageUp`/`PageDown` or `Ctrl+U`/`Ctrl+D` | Page | Page up/down in lists |
| `Home`/`End` or `g`/`G` | Jump | Jump to top/bottom |

## General Operations
| Key | Action | Description |
|---|---|---|
| `Ctrl+Z` | Undo | Undo last action |
| `Ctrl+Y` | Redo | Redo last undone action |
| `Esc` | Dismiss | Dismiss notification / close overlays |
| `Ctrl+H` | Notifications | Show notification history |
| `Enter` | Confirm/View | Open details, confirm dialog, or toggle expand |
| `Esc` | Cancel/Back | Close dialogs, clear search, go back |
| `r` or `F5` | Refresh | Refresh data |


## Issues View - List
| Key | Action | Description |
|---|---|---|
| `n` | Create | Create a new issue |
| `e` | Edit | Edit selected issue |
| `d` | Delete | Delete selected issue (with confirmation) |
| `x` | Close | Close selected issue |
| `o` | Reopen | Reopen selected issue |
| `F2` | Rename | Quick edit issue title |
| `p` | Priority | Change priority of selected issue |
| `Shift+S` | Status | Change status of selected issue |
| `l` | Labels | Edit labels for selected issue |
| `a` | Assignee | Edit assignee for selected issue |
| `+` | Add Dep | Add dependency to selected issue |
| `-` | Remove Dep | Remove dependency from selected issue |
| `>` | Indent | Indent issue (make child of previous) |
| `<` | Outdent | Outdent issue (promote to parent level) |
| `Space` | Toggle Select | Select/deselect issue (when multi-select is supported) |
| `Ctrl+A` | Select All | Select all issues |
| `Ctrl+N` | Deselect All | Clear selection |
| `c` | Columns | Open column manager |
| `v` | Scope | Cycle issue scope (All/Ready/Blocked/My/Recent/Stale) |

## Issues View - Detail / Split
| Key | Action | Description |
|---|---|---|
| `Enter` | Full View | Open full detail view |
| `e` | Edit | Edit selected issue |
| `d` | Delete | Delete selected issue |
| `Esc` or `q` | Back | Return to list view |
| `Alt+H` | History | Toggle issue history panel |

## Issue Forms (Create/Edit)
| Key | Action | Description |
|---|---|---|
| `Tab` | Next Field | Move focus to next form field |
| `Shift+Tab` | Prev Field | Move focus to previous form field |
| `Enter` | Submit | Save and close the form |
| `Esc` | Cancel | Close form without saving |
| `Ctrl+L` | Load File | Load description content from file path |

## Record Detail Form
| Key | Action | Description |
|---|---|---|
| `r` or `R` | Read Mode | Open selected issue in read-only mode |
| `e` or `E` | Edit Mode | Open selected issue in edit mode |
| `Tab` | Switch Focus | Switch focus between list and detail (split view) |
| `Ctrl+S` | Save | Save changes in edit mode |
| `Ctrl+X` | Cancel | Cancel editing and revert changes |
| `Ctrl+Del` | Soft Delete | Soft delete issue |
| `Ctrl+J` | Copy JSON | Copy issue as JSON to clipboard |
| `Ctrl+P` | Export Markdown | Export issue to Markdown file |
| `Up`/`Down` | Scroll | Scroll detail view |
| `PgUp`/`PgDn` | Page Scroll | Page up/down in detail view |
| `Home`/`End` | Jump | Jump to start/end of detail view |

## Issues View - Search and Filters
| Key | Action | Description |
|---|---|---|
| `/` or `s` | Search | Focus search bar |
| `Esc` | Clear Search | Clear search input and return focus |
| `Shift+N` | Next Result | Jump to next search result |
| `Alt+N` | Prev Result | Jump to previous search result |
| `Alt+Z` | Fuzzy | Toggle fuzzy search |
| `Alt+R` | Regex | Toggle regex search |
| `f` | Filters | Toggle quick filters on/off |
| `Shift+F` | Clear Filters | Clear current filters |
| `Ctrl+Shift+S` | Save Filter | Save current filter configuration |
| `Alt+F` | Filter Menu | Open saved filters menu |
| `u` | Status Filter | Open Status filter dropdown (Stat[u]s) |
| `y` | Type Filter | Open Type filter dropdown (T[y]pe) |
| `L` | Labels Filter | Open Labels filter dropdown ([L]abels) |
| `i` | Priority Filter | Open Priority filter dropdown (Pr[i]ority) |
| `F3`-`F11` | Saved Filters | Apply saved filter hotkeys |

## Dependencies View
| Key | Action | Description |
|---|---|---|
| `Up`/`Down` or `j`/`k` | Navigate | Move selection |
| `Tab` | Focus | Switch focus between Dependencies and Blocks |
| `a` | Add | Add dependency (opens dialog) |
| `d` | Remove | Remove dependency (with confirmation) |
| `g` | Graph | Show dependency graph (stub) |
| `c` | Cycle Check | Check circular dependencies (stub) |
| `Enter` | View | View selected issue details |
| `Esc` | Back | Return to Issues view |

## Labels View
| Key | Action | Description |
|---|---|---|
| `Up`/`Down` or `j`/`k` | Navigate | Move selection |
| `/` | Search | Search labels |
| `a` | Add | Add label |
| `e` | Edit | Edit label |
| `d` | Delete | Delete label |
| `s` | Stats | Show label stats info |
| `Esc` | Back | Return to Issues view |

## Kanban View
| Key | Action | Description |
|---|---|---|
| `Up`/`Down`/`Left`/`Right` or `h`/`j`/`k`/`l` | Navigate | Move between cards/columns |
| `Space` | Move | Move card to next column |
| `c` | Configure | Configure board columns |
| `Ctrl+F1` | Toggle Column | Toggle Open column collapse |
| `Ctrl+F2` | Toggle Column | Toggle In Progress column collapse |
| `Ctrl+F3` | Toggle Column | Toggle Blocked column collapse |
| `Ctrl+F4` | Toggle Column | Toggle Closed column collapse |

## PERT View
| Key | Action | Description |
|---|---|---|
| `Up`/`Down` | Navigate | Move selection |
| `+` / `-` | Zoom | Zoom in/out |
| `c` | Configure | Configure chart settings |

## Gantt View
| Key | Action | Description |
|---|---|---|
| `Up`/`Down` | Navigate | Move selection |
| `+` / `-` | Zoom | Zoom in/out |
| `g` | Group | Change grouping mode |
| `c` | Configure | Configure chart settings |

## Database View
| Key | Action | Description |
|---|---|---|
| `r` or `F5` | Refresh | Refresh database status |
| `s` | Sync | Sync database with remote |
| `x` | Export | Export issues to JSONL |
| `i` | Import | Import issues from JSONL |
| `v` | Verify | Verify database integrity |
| `c` | Compact | Compact database |
| `Esc` | Back | Return to Issues view |

## Help View
| Key | Action | Description |
|---|---|---|
| `Left`/`Right` or `h`/`l` | Navigate | Switch help sections |
| `Esc` | Back | Return to previous view |
