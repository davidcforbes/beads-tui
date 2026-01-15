# Configuration

beads-tui loads a YAML config file at startup and falls back to defaults
if the file does not exist or omits fields.

## File Location

- Linux/macOS: `~/.config/beads-tui/config.yaml`
- Windows: `%APPDATA%\beads-tui\config.yaml`

## Format

The config file uses YAML with snake_case field names. Unknown fields are
ignored by serde.

## Minimal Example

```yaml
theme:
  name: dark

behavior:
  auto_refresh: true
  refresh_interval_secs: 60
```

## Top-Level Keys

| Key | Type | Default | Notes |
| --- | --- | --- | --- |
| `theme` | object | `{ name: "dark" }` | UI theme name (5 themes available) |
| `behavior` | object | see below | Refresh behavior and UI preferences |
| `keybindings` | object | `{}` | Custom keybinding overrides (66 actions available) |
| `table` | object | see below | Issue list table layout |
| `kanban` | object | see below | Kanban board layout |
| `gantt` | object | see below | Gantt chart configuration |
| `pert` | object | see below | PERT chart configuration |
| `filters` | object | see below | Saved filter configurations |

## Theme

```yaml
theme:
  name: dark
```

`theme.name` is a string. Available themes:

- **`dark`** (default): Dark theme with good contrast
- **`high_contrast`**: WCAG AAA compliant high-contrast theme for accessibility
- **`deuteranopia`**: Red-green colorblind friendly theme (deuteranopia/protanopia)
- **`protanopia`**: Red-green colorblind friendly theme (alternative variant)
- **`tritanopia`**: Blue-yellow colorblind friendly theme

All themes are designed to work well in terminal environments and support 256-color terminals.

## Behavior

```yaml
behavior:
  auto_refresh: true
  refresh_interval_secs: 60
```

- `auto_refresh` (bool): enable periodic refresh.
- `refresh_interval_secs` (number): refresh interval in seconds.

## Table Configuration

```yaml
table:
  row_height: 1
  sort:
    column: updated
    ascending: false
  columns:
    - id: id
      label: ID
      width_constraints:
        min: 10
        max: 20
        preferred: 15
      width: 15
      alignment: left
      wrap: truncate
      visible: true
```

### Fields

- `columns` (list): ordered column definitions.
- `row_height` (number): row height in lines (min 1).
- `sort` (object):
  - `column` (string): one of `id`, `title`, `status`, `priority`, `type`,
    `assignee`, `labels`, `updated`, `created`.
  - `ascending` (bool): sort direction.
- `filters` (map): per-column filter strings (optional).
- `version` (number): config version (default 1).

### Column Definition

Each column supports:

- `id`: same values as `sort.column`.
- `label`: display label string.
- `width_constraints`:
  - `min` (number), `max` (number or null), `preferred` (number).
- `width` (number): current width (clamped to constraints).
- `alignment`: `left`, `center`, or `right`.
- `wrap`: `truncate`, `wrap`, or `wrap_anywhere`.
- `visible`: bool.

## Kanban Configuration

```yaml
kanban:
  grouping_mode: status
  card_height: 3
  columns:
    - id: status_open
      label: Open
      width_constraints:
        min: 15
        max: 80
        preferred: 30
      width: 30
      visible: true
      card_sort: priority
      wip_limit: null
```

### Fields

- `grouping_mode`: `status`, `assignee`, `label`, `priority`.
- `columns` (list): ordered column definitions for the active mode.
- `card_height` (number): card height in lines (min 1).
- `filters` (object): optional filters.
- `version` (number): config version (default 1).

### Kanban Column Definition

- `id` values depend on `grouping_mode`:
  - `status_open`, `status_in_progress`, `status_blocked`, `status_closed`
  - `priority_p0` .. `priority_p4`
  - `unassigned`
  - `assignee: "<name>"` (dynamic, YAML map form)
  - `label: "<name>"` (dynamic, YAML map form)
- `label`: display label string.
- `width_constraints`: same shape as table columns.
- `width`: current width.
- `visible`: bool (mandatory columns cannot be hidden).
- `card_sort`: `priority`, `title`, `created`, or `updated`.
- `wip_limit`: number or null.

## Saved Filters

```yaml
filters:
  saved:
    - name: "My Issues"
      hotkey: F3
      search_text: ""
      status_filter: [open, in_progress]
      assignee: "@me"
    - name: "Blocked"
      hotkey: F4
      status_filter: [blocked]
```

Saved filters can be:
- Accessed via `Alt+F` (saved filters menu)
- Applied directly with `F3`-`F11` hotkeys
- Created via `Ctrl+Shift+S` in the Issues view

### Filter Fields

- `name` (string): Display name for the filter
- `hotkey` (string): Function key (`F3`-`F11`)
- `search_text` (string): Text search query
- `status_filter` (list): Status values to include
- `priority_filter` (list): Priority levels to include
- `type_filter` (list): Issue types to include
- `label_filter` (list): Labels to filter by
- `assignee` (string): Assignee username or `@me`

## Keybindings

```yaml
keybindings:
  # Override default keybindings (66 actions available)
  # See KEYBOARD_SHORTCUTS.md for the full list of actions
  actions:
    quit: ["q", "ctrl+q", "ctrl+c"]
    search: ["/", "s"]
    create_issue: ["n"]
```

Custom keybindings override the defaults. Each action can have multiple key combinations.
See [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md) for the complete list of 66 available actions.

## Gantt Chart Configuration

```yaml
gantt:
  zoom_level: day
  grouping_mode: none
  show_milestones: true
  show_dependencies: true
```

### Fields

- `zoom_level`: `hour`, `day`, `week`, `month`
- `grouping_mode`: `none`, `assignee`, `label`, `priority`
- `show_milestones`: bool (highlight milestone issues)
- `show_dependencies`: bool (draw dependency lines)

## PERT Chart Configuration

```yaml
pert:
  layout_algorithm: hierarchical
  show_critical_path: true
  node_spacing: 3
```

### Fields

- `layout_algorithm`: `hierarchical`, `force_directed`, `radial`
- `show_critical_path`: bool (highlight critical path in red)
- `node_spacing`: number (spacing between nodes)

## Notes

- Missing fields fall back to defaults.
- Table and Kanban configs auto-migrate: missing mandatory columns are added
  and widths are clamped to constraints.
- Config changes are written atomically with retry logic.
- Invalid YAML results in a warning and fallback to defaults.
- The config file is created with default values on first run.
