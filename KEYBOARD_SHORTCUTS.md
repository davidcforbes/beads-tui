# Keyboard Shortcuts Reference Guide

This guide outlines the intended keyboard shortcuts for the application. Some shortcuts are
context-specific; the same key may do different things depending on the active view or dialog.

## Global Actions
| Key | Action | Description |
|---|---|---|
| `q`, `Ctrl+Q`, `Ctrl+C` | Quit | Exit the application |
| `?` | Shortcut Help | Toggle keyboard shortcuts overlay |
| `F1` | Context Help | Toggle context-sensitive help |
| `Ctrl+P` or `F12` | Performance | Toggle performance statistics |
| `Ctrl+Z` | Undo | Undo last action |
| `Ctrl+Y` | Redo | Redo last undone action |
| `Esc` | Dismiss | Dismiss notification / close overlays |
| `Ctrl+H` or `N` | Notifications | Show notification history |

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
| `s` | Status | Change status of selected issue |
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
| `Ctrl+P` | Preview | Toggle preview mode (Create form only) |

## Issues View - Search and Filters
| Key | Action | Description |
|---|---|---|
| `/` | Search | Focus search bar |
| `Esc` | Clear Search | Clear search input and return focus |
| `Shift+N` | Next Result | Jump to next search result |
| `Alt+N` | Prev Result | Jump to previous search result |
| `Alt+Z` | Fuzzy | Toggle fuzzy search |
| `Alt+R` | Regex | Toggle regex search |
| `f` | Filters | Toggle quick filters on/off |
| `Shift+F` | Clear Filters | Clear current filters |
| `Alt+S` | Save Filter | Save current filter configuration |
| `Alt+F` | Filter Menu | Open saved filters menu |
| `F2`-`F11` | Saved Filters | Apply saved filter hotkeys |

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
| `r` | Refresh | Refresh database status |
| `s` | Sync | Sync database with remote |
| `x` | Export | Export issues to JSONL |
| `i` | Import | Import issues from JSONL |
| `v` | Verify | Verify database integrity |
| `c` | Compact | Compact database |
| `t` | Toggle Daemon | Start/stop daemon |
| `Esc` | Back | Return to Issues view |

## Help View
| Key | Action | Description |
|---|---|---|
| `Left`/`Right` or `h`/`l` | Navigate | Switch help sections |
| `Esc` | Back | Return to previous view |
