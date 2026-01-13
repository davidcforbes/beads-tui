# Keyboard Shortcuts Reference

This reference reflects the current key handling in `src/main.rs`.

## Global Shortcuts

| Key | Action |
| --- | --- |
| `q` | Quit application |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `1-5` | Jump to tab (1 Issues, 2 Dependencies, 3 Labels, 4 Database, 5 Help) |
| `Ctrl+P` / `F12` | Toggle performance stats |
| `Esc` | Dismiss notification |

## Issues View (List)

| Key | Action |
| --- | --- |
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Enter` | Open issue details |
| `e` | Edit selected issue |
| `c` | Create new issue |
| `x` | Close selected issue |
| `o` | Reopen selected issue |
| `d` | Delete selected issue (confirmation dialog) |
| `r` | Edit title inline |
| `f` | Toggle quick filters |
| `/` | Focus search input |
| `v` | Cycle list view |
| `Esc` | Clear search |
| `Alt+Shift+Left` | Shrink focused column |
| `Alt+Shift+Right` | Grow focused column |
| `Alt+Left` | Move focused column left |
| `Alt+Right` | Move focused column right |
| `Alt+Tab` | Focus next column |
| `Alt+Shift+Tab` | Focus previous column |

## Issues View (Search Input Focused)

| Key | Action |
| --- | --- |
| Type | Insert characters |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `Left` / `Right` | Move cursor |
| `Home` / `End` | Move to start/end |
| `Up` / `Down` | Previous/next search history |
| `Enter` | Apply search and exit search input |
| `Esc` | Exit search input |

## Issues View (Inline Title Edit)

| Key | Action |
| --- | --- |
| Type | Insert characters |
| `Backspace` | Delete character before cursor |
| `Left` / `Right` | Move cursor |
| `Enter` | Save title |
| `Esc` | Cancel edit |

## Issues View (Detail)

| Key | Action |
| --- | --- |
| `Esc` / `q` | Return to list |
| `e` | Edit issue |

## Issues View (Create/Edit Form)

| Key | Action |
| --- | --- |
| `Tab` / `Down` | Next field |
| `Shift+Tab` / `Up` | Previous field |
| Type | Insert characters |
| `Backspace` | Delete character before cursor |
| `Left` / `Right` | Move cursor |
| `Home` / `End` | Move to start/end |
| `Enter` | Submit (create) or save (edit) |
| `Esc` | Cancel |
| `Ctrl+L` | Load field content from file path in focused field |

## Dependencies View

| Key | Action |
| --- | --- |
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Tab` | Toggle focus (dependencies vs blocks) |
| `a` | Add dependency (not yet implemented) |
| `d` | Remove dependency/block (not yet implemented) |
| `g` | Show dependency graph (not yet implemented) |
| `c` | Check circular dependencies (not yet implemented) |

## Labels View

| Key | Action |
| --- | --- |
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `a` | Add label (not yet implemented) |
| `d` | Delete label (not yet implemented) |
| `e` | Edit label (not yet implemented) |
| `s` | Show stats summary |
| `/` | Search labels (not yet implemented) |

## Database View (Normal)

| Key | Action |
| --- | --- |
| `/` | Cycle database view mode |
| `r` | Refresh status |
| `d` | Toggle daemon start/stop |
| `s` | Sync database |
| `e` | Export issues (prompts for filename) |
| `i` | Import issues (prompts for filename) |
| `v` | Verify database integrity |
| `c` | Compact database (confirmation dialog) |
| `k` | Kill all processes (not yet implemented) |

## Database View (Filename Prompt)

| Key | Action |
| --- | --- |
| Type | Enter filename |
| `Backspace` | Delete character |
| `Enter` | Confirm |
| `Esc` | Cancel |

## Help View

| Key | Action |
| --- | --- |
| `Right` / `Tab` / `l` | Next section |
| `Left` / `h` | Previous section |

## Dialogs

| Key | Action |
| --- | --- |
| `Left` / `Right` / `Tab` | Change selection |
| `Enter` | Confirm |
| `Esc` | Cancel |
