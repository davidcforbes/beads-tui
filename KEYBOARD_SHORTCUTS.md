# Keyboard Shortcuts Reference Guide

This guide outlines the intended keyboard shortcuts for the application. Some shortcuts are
context-specific; the same key may do different things depending on the active view or dialog.

## Screen Actions (Active on all views)
| Key       | Action          | Description                         |
| --------- | --------------- | ----------------------------------- |
| Tab       | Next Tab        | Switch to the next top-level tab    |
| Shift+Tab | Prev Tab        | Switch to the previous tab          |
| Ctrl+↑    | Scroll Up 1 Row | Scroll the Viewport up 1 Row        |
| Ctrl+↓    | Scroll Dn 1 Row | Scroll the Viewport down 1 Row      |
| Ctrl+→    | Scroll Rt 1 Row | Scroll the Viewport right 1 frame   |
| Ctrl+←    | Scroll Lf 1 Row | Scroll the Viewport left 1 frame    |
| F10       | Maximize Screen | Maximize the view to full screen    |
| F11       | Restore Screen  | Restore view to normal screen size  |
| F12       | Screen shot     | Save screen to clipboard or file    |

## View Actions (Active on all Views)

| Key | Action   | Description                            |
| --- | -------- | -------------------------------------- |
|  1  | Issues   | Switch to the Issues View              |
|  2  | Split    | Switch to the Split View               |
|  3  | Kanban   | Switch to the Kanban View              |
|  4  | Depend   | Switch to the Dependencies View        |
|  5  | Labels   | Switch to the Labels View              |
|  6  | Gantt    | Switch to the Gantt View               |
|  7  | Pert     | Switch to the Pert View                |
|  8  | Molecule | Switch to the Molecule View            |
|  9  | Stats    | Switch to the Statistics View          |
|  0  | Utility  | Switch to the Utilities View           |
|  H  | History  | Switch to the Notice History View      |

## Filter ACTIONS (Active on Issues View, Split View, Kanban View)

| Key | Action       | Description                           |
| --- | ------------ | ------------------------------------- |
|  S  | Status       | Open the Status Filter Dropdown       |
|  T  | Status       | Open the Type Filter Dropdown         |
|  P  | Priority     | Open the Priority Filter Dropdown     |
|  L  | Labels       | Open the Labels Filter Dropdown       |
|  C  | Created      | Open the Created Date Filter Dropdown |
|  U  | Updated      | Open the Updated Date Filter Dropdown |
| F1  | Reset Filter | Reset the Filters to "All"            |
| F2  | Save Filter  | Save current filter configuration     |
| F3  | Filter Menu  | Open saved filters menu & choose      |

## Find Actions (Active on Issues View, Split View, Kanban View)

| Key     | Action      | Description                             |
| ------- | ----------- | --------------------------------------- |
|    F    | Find        | Focus search bar and enter search text  |
| Shift+F | Clear Find  | Clear search input and return focus     |
| Shift+N | Next Result | Jump to next search result              |
| Shift+P | Prev Result | Jump to previous search result          |
| Shift+Z | Fuzzy Find  | Toggle fuzzy search                     |
| Shift+R | Regex Find  | Toggle regex search                     |

## Issue List ACTIONS (Active on Issues View, Split View, and Kanban View)

| Key  | Action      | Description                            |
| ---- | ----------- | -------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                     |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                   |
| PgUp | Page Up 1   | Navigate up 1 Page of Rows             |
| PgDn | Page Down 1 | Navigate down 1 Page of Rows           |
| Home | List Top    | Navigate to the Top of the List        |
| End  | List Bottom | Navigate to the Bottom of the List     |
|  M   | Manage Cols | Open column manager                    |

## Selected Issue ACTIONS (Active on Issues View, Split View, and Kanban View)

| Key | Action    | Description                            |
| --- | --------- | -------------------------------------- |
|  R  | Read      | Read Mode for Selected Record Detail   |
|  N  | New       | Create Mode for New Record Detail      |
|  E  | Edit      | Edit Mode for Selected Record Detail   |
|  D  | Delete    | Delete (Soft) The Selected Record      |
|  F  | Find      | Enter a Find String to Match Records   |
|  O  | Open      | Mark the Selected Record Status=Open   |
|  X  | Close     | Mark the Selected Record Status=Closed |
|  B  | Block     | Mark the Selected Record Status=Blocked|
|  Q  | Quit      | Exit the application                   |
|  H  | History   | Toggle issue history panel             |
| --- | --------- | -------------------------------------- |

## Record Detail Form Create/Edit Actions

| Key       | Action      | Description                             |
| --------- | ----------- | --------------------------------------- |
| Tab       | Next Field  | Move focus to next form field           |
| Shift+Tab | Prev Field  | Move focus to previous form field       |
| Esc       | Cancel Edit | Dismiss notification / close overlays   |
| Ctrl+S    | Save Edit   | Save changes in edit mode               |
| Ctrl+Z    | Undo Edit   | Undo last action                        |
| Ctrl+Y    | Redo Edit   | Redo last undone action                 |
| Ctrl+L    | Load File   | Load description content from file path |
| Ctrl+X    | Cancel      | Cancel editing and revert changes       |
| Ctrl+Del  | Soft Delete | Soft delete issue                       |
| Ctrl+J    | Copy JSON   | Copy issue as JSON to clipboard         |
| Ctrl+P    | Markdown    | Export issue to Markdown file           |

## Kanban View
| Key     | Action          | Description                             |
| ------- | --------------- | --------------------------------------- |
|   F4    | Open Toggle     | Toggle Open column collapse             |
|   F5    | Progress Toggle | Toggle In Progress column collapse      |
|   F6    | Blocked Toggle  | Toggle Blocked column collapse          |
|   F7    | Closed Toggle   | Toggle Closed column collapse           |

## Dependencies View
| Key     | Action        | Description                             |
| ------- | ------------- | --------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                           |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                         |
| Tab  | Focus       | Switch focus between Dependencies and Blocks |
| Ins  | Insert     | Add a dependency (opens dialog)               |
| Del  | Delete      | Remove dependency (with confirmation)        |
|  \   | Edit        | Edit the selected dependencies               |
|  G   | Graph       | Show dependency graph (stub)                 |
|  V   | Validate    | Check for circular dependencies (stub)       |
|  R   | Read        | Record Detail View for selected Record       |

## Labels View
| Key     | Action   | Description                                  |
| ------- | -------- | -------------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                           |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                         |
| Ins  | Insert      | Add a label                                  |
| Del  | Delete      | Delete label                                 |
|  \   | Edit        | Edit label                                   |

## PERT View
| Key  | Action      | Description                                  |
| ---- | ----------- | -------------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                           |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                         |
|  +   | Zoom In     | Zoom in                                      |
|  -   | Zoom Out    | Zoom out                                     |
| F8   | Configure   | Configure Pert Chart Settings                |

## Gantt View
| Key     | Action   | Description                                  |
| ------- | -------- | -------------------------------------------- |
|  ↑   | Up 1 Row    | Navigate up 1 Rows                           |
|  ↓   | Down 1 Row  | Navigate down 1 Rows                         |
|  +   | Zoom In     | Zoom in                                      |
|  -   | Zoom Out    | Zoom out                                     |
| F9   | Configure   | Configure Ghantt Chart Settings              |
|  G   | Group       | Change grouping mode                         |

## Database View
| Key     | Action        | Description                             |
| ------- | ------------- | --------------------------------------- |
|   R     | Refresh DB    | Refresh database status                 |
|   S     | Sync DB       | Sync database with remote               |
|   X     | Export DB     | Export issues to JSONL                  |
|   I     | Import DB     | Import issues from JSONL                |
|   V     | Verify DB     | Verify database integrity               |
|   C     | Compact DB    | Compact database                        |

## Help View
| Key     | Action          | Description                           |
| ------- | --------------- | ------------------------------------- |
| Ctrl+↑  | Scroll Up 1 Row | Scroll the Viewport up 1 Row          |
| Ctrl+↓  | Scroll Dn 1 Row | Scroll the Viewport down 1 Row        |