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
| `theme` | object | `{ name: "dark" }` | UI theme name |
| `behavior` | object | see below | Refresh behavior |
| `keybindings` | object | `{}` | Reserved for future custom bindings |
| `table` | object | see below | Issue list table layout |
| `kanban` | object | see below | Kanban board layout |

## Theme

```yaml
theme:
  name: dark
```

`theme.name` is a string. The default is `dark`.

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

## Notes

- Missing fields fall back to defaults.
- Table and Kanban configs auto-migrate: missing mandatory columns are added
  and widths are clamped to constraints.
