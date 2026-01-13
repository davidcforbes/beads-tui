# Keyboard Shortcuts Reference

Complete reference for all keyboard shortcuts in beads-tui.

## Global Shortcuts

These shortcuts work from any view:

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `1-5` | Jump to tab directly (1=Issues, 2=Dependencies, 3=Labels, 4=Database, 5=Help) |
| `?` | Toggle help |
| `Ctrl+C` | Force quit |

## Issues View

Navigation and search:

| Key | Action |
|-----|--------|
| `/` | Focus search input |
| `Tab` | Cycle search scope (when in search) |
| `Esc` | Clear search / Cancel operation |
| `j` or `↓` | Navigate down in list |
| `k` or `↑` | Navigate up in list |
| `Enter` | View issue details |

Issue management:

| Key | Action |
|-----|--------|
| `n` | Create new issue |
| `e` | Edit selected issue |
| `d` | Delete selected issue |
| `c` | Close selected issue |
| `r` | Reopen closed issue |

## Dependencies View

Navigation:

| Key | Action |
|-----|--------|
| `j` or `↓` | Navigate down in tree |
| `k` or `↑` | Navigate up in tree |
| `Enter` | Expand/collapse node |

Dependency management:

| Key | Action |
|-----|--------|
| `a` | Add dependency |
| `d` | Remove dependency |
| `g` | Show dependency graph |
| `c` | Check for cycles |

## Labels View

| Key | Action |
|-----|--------|
| `j` or `↓` | Navigate down in label list |
| `k` or `↑` | Navigate up in label list |
| `a` | Add new label |
| `d` | Delete label |
| `e` | Edit label |

## Database View

| Key | Action |
|-----|--------|
| `r` | Refresh database statistics |
| `s` | Sync with remote |
| `i` | Import issues |
| `x` | Export issues |

## Form Editing

When editing forms (new issue, edit issue, etc.):

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Enter` | Confirm / Next line (in text areas) |
| `Esc` | Cancel editing |
| `Ctrl+S` | Save changes |

## Text Input

When in text input fields:

| Key | Action |
|-----|--------|
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` / `→` | Move cursor left/right |
| `Home` / `Ctrl+A` | Move to start of line |
| `End` / `Ctrl+E` | Move to end of line |
| `Ctrl+U` | Clear line |
| `Ctrl+W` | Delete word before cursor |

## Dialog Shortcuts

When dialogs are open:

| Key | Action |
|-----|--------|
| `Enter` | Confirm action |
| `Esc` | Cancel / Close dialog |
| `Tab` | Next button |
| `y` | Yes (in confirmation dialogs) |
| `n` | No (in confirmation dialogs) |

## Tips

- **Vim-style navigation**: Use `j`/`k` keys for up/down navigation in lists
- **Quick access**: Number keys 1-5 provide instant tab switching
- **Context help**: Press `?` at any time to see view-specific shortcuts
- **Force quit**: `Ctrl+C` immediately exits without saving
- **Search everywhere**: `/` activates search in most list views

## Advanced Features

### Search Scopes

In Issues view, after pressing `/`, use `Tab` to cycle through:
- **All**: Search all issues
- **Open**: Search only open issues
- **Closed**: Search only closed issues
- **Blocked**: Search only blocked issues

### Multi-line Text Editing

In description fields:
- `Enter` creates a new line
- `Ctrl+S` saves the complete text
- `Esc` discards changes

### Navigation Patterns

Common navigation patterns across views:
- `j`/`k` or arrow keys for vertical movement
- `Enter` to select/expand/confirm
- `Esc` to go back/cancel
- `Tab` to move forward through options
