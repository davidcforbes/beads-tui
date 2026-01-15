# Beads-TUI User Interface Reference

**Version:** 0.1.0
**Last Updated:** 2026-01-14

This comprehensive guide documents all user interface screens, components, and interactions in beads-tui. Each section includes detailed descriptions and screenshot placeholders for visual reference.

---

## Table of Contents

1. [Overview](#overview)
2. [Main Navigation](#main-navigation)
3. [Status Bar](#status-bar)
4. [Main Views](#main-views)
   - [Issues View](#issues-view)
   - [Dependencies View](#dependencies-view)
   - [Labels View](#labels-view)
   - [Database View](#database-view)
   - [Help View](#help-view)
5. [Issue Management](#issue-management)
   - [Issue Detail View](#issue-detail-view)
   - [Issue Editor](#issue-editor)
6. [Advanced Views](#advanced-views)
   - [Gantt Chart View](#gantt-chart-view)
   - [Kanban Board View](#kanban-board-view)
   - [PERT Chart View](#pert-chart-view)
7. [Molecular Chemistry Operations](#molecular-chemistry-operations)
8. [Dialogs and Overlays](#dialogs-and-overlays)
9. [Widgets and Components](#widgets-and-components)
10. [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Overview

Beads-TUI is a Rust-based terminal user interface for the beads issue tracking system. It provides a rich, interactive experience with keyboard-first navigation, real-time updates, and visual dependency management.

### Key Features

- **Tab-based Navigation**: Switch between Issues, Dependencies, Labels, Database, and Help views
- **Keyboard-First Design**: Efficient navigation with vim-style keybindings
- **Visual Dependency Trees**: Interactive tree visualization of issue dependencies
- **Advanced Filtering**: Full-text search with status, label, and priority filters
- **Column Customization**: Show, hide, and reorder table columns
- **Markdown Support**: Rich text rendering for issue descriptions
- **Screen Reader Support**: Text-to-speech announcements (enable with `--tts` flag)
- **Undo/Redo**: Reversible operations for safety

---

## Main Navigation

The main application interface uses a tabbed layout with the following structure:

```
┌─────────────────────────────────────────────────────────────┐
│ [Issues] [Dependencies] [Labels] [Database] [Help]          │ ← Tab Bar
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                                                             │
│                   Main View Content                         │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Status: Ready | Issues: 42 | q: Quit  ?: Help              │ ← Status Bar
└─────────────────────────────────────────────────────────────┘
```

**Screenshot Placeholder:** `screenshot-01-main-layout.png`
*Caption: Main application layout showing tab bar, content area, and status bar*

### Navigation Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `1-5` | Jump to tab by number (1=Issues, 2=Dependencies, etc.) |
| `Esc` | Go back / Close dialog |
| `q` | Quit application (from main views) |
| `?` | Show help overlay |

---

## Status Bar

The status bar appears at the bottom of the screen and displays:

- **Status Message**: Current operation or ready state
- **Issue Count**: Total number of issues
- **Daemon Status**: Beads daemon running status
- **Quick Shortcuts**: Context-sensitive keyboard hints
- **Performance Stats**: Optional FPS and render time (toggle with Debug mode)

**Screenshot Placeholder:** `screenshot-02-status-bar.png`
*Caption: Status bar showing application state and shortcuts*

---

## Main Views

### Issues View

The Issues view is the primary workspace for browsing, creating, and managing issues.

#### Features

- **Issue Table**: Sortable, filterable list of all issues
- **Column Customization**: Hide/show/reorder columns via Column Manager
- **Bulk Operations**: Select multiple issues for batch updates
- **Quick Actions**: Status change, priority update, label management
- **Search and Filter**: Full-text search with advanced filter builder

**Screenshot Placeholder:** `screenshot-03-issues-view.png`
*Caption: Issues view showing the issue table with multiple columns*

#### Table Columns

Default columns (customizable via Column Manager):

| Column | Description | Width |
|--------|-------------|-------|
| ID | Issue identifier (beads-XXX) | 12 chars |
| Title | Issue title/summary | Flexible |
| Status | Current status (open, in_progress, closed) | 12 chars |
| Priority | Priority level (P0-P4) | 8 chars |
| Type | Issue type (task, bug, feature, epic) | 10 chars |
| Labels | Comma-separated labels | Flexible |
| Assignee | Assigned user | 15 chars |
| Created | Creation date | 10 chars |
| Updated | Last update date | 10 chars |

#### Keyboard Shortcuts (Issues View)

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate up/down |
| `Enter` | View issue details |
| `c` | Create new issue |
| `e` | Edit selected issue |
| `d` | Delete selected issue (with confirmation) |
| `Shift+C` | Close selected issue |
| `Shift+S` | Change issue status |
| `p` | Change issue priority |
| `Shift+L` | Manage labels |
| `Space` | Toggle selection (bulk mode) |
| `a` | Select all visible issues |
| `Shift+A` | Deselect all |
| `/` | Focus search bar |
| `f` | Open filter builder |
| `Shift+F` | Save current filter |
| `Ctrl+M` | Open column manager |
| `r` | Refresh issue list |

**Screenshot Placeholder:** `screenshot-04-issues-view-filtered.png`
*Caption: Issues view with active filters applied*

**Screenshot Placeholder:** `screenshot-05-issues-view-bulk-select.png`
*Caption: Issues view with multiple issues selected for bulk operations*

#### Search and Filtering

The search bar supports multiple query types:

- **Text search**: `authentication` (searches title, description, comments)
- **Status filter**: `status:open`, `status:in_progress`, `status:closed`
- **Label filter**: `label:bug`, `label:"needs review"`
- **Priority filter**: `priority:0`, `priority:high` (0-4 or P0-P4)
- **Type filter**: `type:bug`, `type:feature`, `type:task`, `type:epic`
- **Assignee filter**: `assignee:username`, `assignee:@me`
- **Combined**: `status:open label:bug authentication`

**Screenshot Placeholder:** `screenshot-06-search-active.png`
*Caption: Search bar with active filter showing results*

---

### Dependencies View

The Dependencies view provides visual representation and management of issue dependencies.

#### Features

- **Dependency Tree**: Hierarchical tree view of issue relationships
- **Cycle Detection**: Automatic detection and highlighting of circular dependencies
- **Interactive Navigation**: Navigate through dependency chains
- **Add/Remove Dependencies**: Dialog-based dependency management
- **Blocked Issues**: Highlight issues blocked by dependencies

**Screenshot Placeholder:** `screenshot-07-dependencies-view.png`
*Caption: Dependencies view showing dependency tree structure*

#### Tree Representation

```
beads-001: Implement authentication system
├─ beads-002: Create user model [BLOCKS beads-001]
│  └─ beads-003: Setup database schema [BLOCKS beads-002]
└─ beads-004: Design login UI [BLOCKS beads-001]
```

#### Keyboard Shortcuts (Dependencies View)

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate tree nodes |
| `←`/`→` or `h`/`l` | Collapse/expand nodes |
| `Enter` | View issue details |
| `a` | Add dependency for selected issue |
| `d` | Remove dependency (with confirmation) |
| `r` | Refresh dependency tree |
| `e` | Expand all nodes |
| `c` | Collapse all nodes |

**Screenshot Placeholder:** `screenshot-08-dependencies-expanded.png`
*Caption: Dependencies view with all nodes expanded*

**Screenshot Placeholder:** `screenshot-09-dependency-cycle.png`
*Caption: Dependencies view highlighting a circular dependency*

---

### Labels View

The Labels view provides label management and analytics.

#### Features

- **Label Statistics**: Usage count, issue count per label
- **Label Search**: Filter labels by name or pattern
- **Label Dimensions**: Support for namespaced labels (e.g., `state:patrol`, `priority:high`)
- **Usage Analytics**: Most/least used labels
- **Label Health**: Detect unused or misspelled labels

**Screenshot Placeholder:** `screenshot-10-labels-view.png`
*Caption: Labels view showing label statistics and usage*

#### Label Statistics Table

| Label | Issues | Percentage | Last Used |
|-------|--------|------------|-----------|
| bug | 15 | 35% | 2 days ago |
| feature | 12 | 28% | 1 day ago |
| needs-review | 8 | 19% | 3 hours ago |
| documentation | 5 | 12% | 1 week ago |

#### Keyboard Shortcuts (Labels View)

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate labels |
| `Enter` | Filter issues by selected label |
| `/` | Search labels |
| `r` | Refresh label statistics |

**Screenshot Placeholder:** `screenshot-11-labels-search.png`
*Caption: Labels view with search filter active*

---

### Database View

The Database view provides monitoring and maintenance tools for the beads database.

#### Features

- **Database Stats**: Issue counts, storage size, index health
- **Sync Status**: Remote sync state and last sync time
- **Daemon Status**: Beads daemon running state and PID
- **Maintenance Operations**: Compaction, vacuum, integrity check
- **Import/Export**: Database backup and restore

**Screenshot Placeholder:** `screenshot-12-database-view.png`
*Caption: Database view showing health metrics and sync status*

#### Displayed Information

**Database Health**
- Total issues: 42
- Open issues: 18
- Closed issues: 24
- Database size: 2.4 MB
- Index status: Healthy

**Sync Status**
- Remote: origin/main
- Last sync: 2 minutes ago
- Status: Up to date
- Uncommitted changes: 0

**Daemon Status**
- Status: Running
- PID: 12345
- Uptime: 2 hours 15 minutes

#### Keyboard Shortcuts (Database View)

| Key | Action |
|-----|--------|
| `s` | Sync with remote |
| `c` | Compact database |
| `v` | Vacuum database |
| `i` | Run integrity check |
| `e` | Export database |
| `Shift+I` | Import database |
| `d` | Start/stop daemon |
| `r` | Refresh stats |

**Screenshot Placeholder:** `screenshot-13-database-sync.png`
*Caption: Database view during sync operation*

---

### Help View

The Help view provides context-sensitive keyboard shortcuts and usage tips.

#### Features

- **Categorized Shortcuts**: Organized by view and function
- **Search**: Filter shortcuts by key or description
- **Context-Sensitive**: Shows relevant shortcuts for current view
- **Quick Reference**: Printable cheat sheet format

**Screenshot Placeholder:** `screenshot-14-help-view.png`
*Caption: Help view showing categorized keyboard shortcuts*

#### Help Sections

1. **Global Shortcuts**: Available in all views
2. **Navigation**: Tab switching and view navigation
3. **Issues View**: Issue management shortcuts
4. **Dependencies View**: Dependency tree shortcuts
5. **Labels View**: Label management shortcuts
6. **Database View**: Database operation shortcuts
7. **Editing**: Form and text editor shortcuts

**Screenshot Placeholder:** `screenshot-15-help-search.png`
*Caption: Help view with search filter active*

---

## Issue Management

### Issue Detail View

The Issue Detail view displays comprehensive information about a single issue.

#### Features

- **Full Metadata**: All issue fields with inline editing
- **Markdown Rendering**: Rich text display for description
- **Dependency Display**: Shows blocking and blocked-by relationships
- **Label Display**: Visual label chips
- **Issue History**: Timeline of changes and comments
- **Quick Actions**: Edit, close, reopen, delete

**Screenshot Placeholder:** `screenshot-16-issue-detail.png`
*Caption: Issue detail view showing full issue information*

#### Displayed Information

```
┌─ Issue: beads-042 ──────────────────────────────────────┐
│ Title: Implement user authentication                    │
│ Status: in_progress           Priority: P1              │
│ Type: feature                Assignee: alice            │
│ Labels: [backend] [security] [api]                      │
│                                                          │
│ Created: 2026-01-10           Updated: 2026-01-14       │
│                                                          │
│ Description:                                             │
│ ─────────────────────────────────────────────────────── │
│ Implement JWT-based authentication for the API.         │
│                                                          │
│ Requirements:                                            │
│ - User login with email/password                        │
│ - JWT token generation                                  │
│ - Token refresh mechanism                               │
│ - Logout functionality                                  │
│                                                          │
│ Dependencies:                                            │
│ ─────────────────────────────────────────────────────── │
│ Depends on:                                              │
│   • beads-040: Setup database schema                    │
│   • beads-041: Create user model                        │
│                                                          │
│ Blocks:                                                  │
│   • beads-043: Implement authorization middleware       │
│   • beads-044: Add protected API endpoints              │
│                                                          │
│ [e] Edit  [c] Close  [l] Labels  [d] Dependencies       │
└──────────────────────────────────────────────────────────┘
```

#### Keyboard Shortcuts (Issue Detail)

| Key | Action |
|-----|--------|
| `Esc` | Back to issue list |
| `e` | Edit issue |
| `c` | Close issue |
| `Shift+R` | Reopen issue |
| `Shift+L` | Manage labels |
| `d` | Manage dependencies |
| `h` | Show issue history |
| `Delete` | Delete issue (with confirmation) |

**Screenshot Placeholder:** `screenshot-17-issue-detail-dependencies.png`
*Caption: Issue detail view highlighting dependency relationships*

---

### Issue Editor

The Issue Editor is a form-based interface for creating and editing issues.

#### Features

- **Form Validation**: Required field validation
- **Field Types**: Text input, textarea, dropdown selectors
- **Autocomplete**: Label and assignee autocomplete
- **Markdown Editor**: Description field with markdown support
- **Date Picker**: For due dates and milestones
- **Real-time Validation**: Immediate feedback on invalid input

**Screenshot Placeholder:** `screenshot-18-issue-editor-create.png`
*Caption: Issue editor form for creating a new issue*

#### Form Fields

**Required Fields:**
- **Title**: Issue summary (max 200 characters)
- **Type**: Issue type (task, bug, feature, epic)

**Optional Fields:**
- **Description**: Detailed description with markdown support
- **Priority**: Priority level (P0-P4, default: P2)
- **Assignee**: Username of assigned person
- **Labels**: Comma-separated labels with autocomplete
- **Due Date**: Target completion date
- **Estimated Hours**: Time estimate

**Screenshot Placeholder:** `screenshot-19-issue-editor-validation.png`
*Caption: Issue editor showing field validation errors*

#### Keyboard Shortcuts (Issue Editor)

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Ctrl+S` | Save issue |
| `Esc` | Cancel and close |
| `Ctrl+Enter` | Save and create another |
| `Ctrl+D` | Insert current date |
| `Ctrl+L` | Focus label field |
| `Ctrl+P` | Open priority selector |

**Screenshot Placeholder:** `screenshot-20-issue-editor-autocomplete.png`
*Caption: Issue editor showing label autocomplete suggestions*

---

## Advanced Views

### Gantt Chart View

The Gantt Chart view provides a timeline-based visualization of issues and milestones.

#### Features

- **Timeline View**: Calendar-based issue scheduling
- **Dependencies**: Visual dependency arrows between issues
- **Milestones**: Key project milestones
- **Date Ranges**: Zoom in/out to view different time periods
- **Drag and Drop**: Reschedule issues by dragging
- **Progress Tracking**: Visual progress bars

**Screenshot Placeholder:** `screenshot-21-gantt-view.png`
*Caption: Gantt chart showing issue timeline and dependencies*

#### Timeline Elements

```
Jan 2026           Feb 2026           Mar 2026
├──────────────────┼──────────────────┼──────────────────┤
│ beads-040: Database schema                            │
│ ████████████░░░░ (75%)                                │
│                                                        │
│     beads-041: User model                             │
│     ███████████████░░ (85%)                           │
│                                                        │
│         beads-042: Authentication ──────┐             │
│         ██████░░░░░░░░░░ (40%)          │             │
│                                         │             │
│                                         ▼             │
│             beads-043: Authorization                  │
│             ░░░░░░░░░░░░ (0%)                         │
│                                                        │
│ ◆ Milestone: Auth Complete                            │
└────────────────────────────────────────────────────────┘
```

#### Keyboard Shortcuts (Gantt View)

| Key | Action |
|-----|--------|
| `←`/`→` | Scroll timeline left/right |
| `+`/`-` | Zoom in/out |
| `t` | Go to today |
| `m` | Add milestone |
| `Enter` | Edit selected issue |
| `r` | Refresh timeline |

**Screenshot Placeholder:** `screenshot-22-gantt-zoom.png`
*Caption: Gantt chart zoomed to month view*

---

### Kanban Board View

The Kanban Board view displays issues in a column-based workflow.

#### Features

- **Column Layout**: Customizable status columns
- **Drag and Drop**: Move issues between columns
- **WIP Limits**: Work-in-progress limits per column
- **Card Customization**: Show/hide card fields
- **Swimlanes**: Group by assignee, priority, or label
- **Quick Edit**: Inline card editing

**Screenshot Placeholder:** `screenshot-23-kanban-view.png`
*Caption: Kanban board showing issues organized by status*

#### Default Columns

```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│   Backlog   │  To Do      │ In Progress │    Done     │
│   (12)      │   (8)       │   (5)       │   (24)      │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ ┌─────────┐ │ ┌─────────┐ │ ┌─────────┐ │ ┌─────────┐ │
│ │beads-050│ │ │beads-042│ │ │beads-040│ │ │beads-035│ │
│ │Feature  │ │ │P1 Auth  │ │ │Database │ │ │Login UI │ │
│ │@alice   │ │ │@bob     │ │ │@alice   │ │ │@charlie │ │
│ └─────────┘ │ └─────────┘ │ └─────────┘ │ └─────────┘ │
│             │             │             │             │
│ ┌─────────┐ │ ┌─────────┐ │ ┌─────────┐ │ ┌─────────┐ │
│ │beads-051│ │ │beads-043│ │ │beads-041│ │ │beads-036│ │
│ │Bug Fix  │ │ │P2 API   │ │ │Models   │ │ │Tests    │ │
│ │@bob     │ │ │@alice   │ │ │@bob     │ │ │@alice   │ │
│ └─────────┘ │ └─────────┘ │ └─────────┘ │ └─────────┘ │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

#### Keyboard Shortcuts (Kanban View)

| Key | Action |
|-----|--------|
| `←`/`→` or `h`/`l` | Switch columns |
| `↑`/`↓` or `j`/`k` | Navigate cards |
| `Enter` | Edit selected card |
| `Shift+→` | Move card right |
| `Shift+←` | Move card left |
| `c` | Create issue in current column |
| `s` | Toggle swimlanes |
| `r` | Refresh board |

**Screenshot Placeholder:** `screenshot-24-kanban-swimlanes.png`
*Caption: Kanban board with assignee swimlanes*

---

### PERT Chart View

The PERT Chart view displays a network diagram of issue dependencies.

#### Features

- **Network Diagram**: Visual graph of issue relationships
- **Critical Path**: Highlight longest dependency chain
- **Node Information**: Issue details in graph nodes
- **Auto Layout**: Automatic graph layout algorithms
- **Cycle Detection**: Visual highlighting of circular dependencies
- **Zoom and Pan**: Navigate large dependency graphs

**Screenshot Placeholder:** `screenshot-25-pert-view.png`
*Caption: PERT chart showing network diagram of dependencies*

#### Graph Representation

```
     ┌───────────┐
     │ beads-040 │
     │ Database  │
     │  Schema   │
     └─────┬─────┘
           │
           ▼
     ┌───────────┐     ┌───────────┐
     │ beads-041 │     │ beads-045 │
     │   User    │────▶│    API    │
     │   Model   │     │   Routes  │
     └─────┬─────┘     └─────┬─────┘
           │                 │
           ▼                 ▼
     ┌───────────┐     ┌───────────┐
     │ beads-042 │     │ beads-046 │
     │   Auth    │────▶│   Tests   │
     │  System   │     │           │
     └───────────┘     └───────────┘
     
     [CRITICAL PATH highlighted]
```

#### Keyboard Shortcuts (PERT View)

| Key | Action |
|-----|--------|
| `←`/`→`/`↑`/`↓` | Pan view |
| `+`/`-` | Zoom in/out |
| `0` | Fit to screen |
| `c` | Highlight critical path |
| `l` | Change layout algorithm |
| `Enter` | View selected node details |
| `r` | Refresh graph |

**Screenshot Placeholder:** `screenshot-26-pert-critical-path.png`
*Caption: PERT chart with critical path highlighted*

---

## Molecular Chemistry Operations

Molecular chemistry operations provide advanced issue management capabilities inspired by atomic operations.

### Formula Browser

Browse and apply saved molecular formulas for common operations.

**Screenshot Placeholder:** `screenshot-27-formula-browser.png`
*Caption: Formula browser showing available molecular formulas*

#### Available Formulas

- **Pour**: Copy issues between projects
- **Wisp**: Create lightweight issue templates
- **Bond**: Merge related issues
- **Squash**: Consolidate issue history
- **Burn**: Permanently delete archived issues

---

### Pour Wizard

The Pour Wizard guides you through copying issues between projects.

**Screenshot Placeholder:** `screenshot-28-pour-wizard.png`
*Caption: Pour wizard interface for copying issues*

#### Steps

1. **Select Source**: Choose issues to copy
2. **Select Destination**: Choose target project
3. **Configure Mapping**: Map labels and statuses
4. **Preview**: Review changes before execution
5. **Execute**: Perform the pour operation

---

### Wisp Manager

Manage lightweight issue templates (wisps) for rapid issue creation.

**Screenshot Placeholder:** `screenshot-29-wisp-manager.png`
*Caption: Wisp manager showing available templates*

#### Features

- **Template Library**: Pre-defined issue templates
- **Custom Wisps**: Create custom templates
- **Quick Create**: One-click issue creation from template
- **Field Defaults**: Pre-filled field values

---

### Bonding Interface

Merge related issues using the bonding interface.

**Screenshot Placeholder:** `screenshot-30-bonding-interface.png`
*Caption: Bonding interface for merging issues*

#### Features

- **Issue Selection**: Select issues to merge
- **Conflict Resolution**: Resolve field conflicts
- **History Preservation**: Maintain complete audit trail
- **Dependency Update**: Automatically update dependencies

---

### History Operations

View and manage issue history, including compaction and squashing.

**Screenshot Placeholder:** `screenshot-31-history-ops.png`
*Caption: History operations interface*

#### Operations

- **View History**: Browse issue change timeline
- **Squash**: Consolidate history entries
- **Compact**: Remove redundant history
- **Export**: Save history to file

---

## Dialogs and Overlays

### Column Manager

The Column Manager allows customization of table columns in the Issues view.

**Screenshot Placeholder:** `screenshot-32-column-manager.png`
*Caption: Column manager dialog showing available columns*

#### Features

- **Show/Hide**: Toggle column visibility
- **Reorder**: Drag to reorder columns
- **Resize**: Adjust column widths
- **Presets**: Save column configurations
- **Reset**: Restore default layout

#### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate columns |
| `Space` | Toggle visibility |
| `Shift+↑`/`↓` | Reorder columns |
| `Enter` | Apply changes |
| `Esc` | Cancel |

---

### Filter Builder

The Filter Builder provides advanced query construction for filtering issues.

**Screenshot Placeholder:** `screenshot-33-filter-builder.png`
*Caption: Filter builder dialog with multiple filter conditions*

#### Filter Types

- **Status**: Filter by issue status
- **Priority**: Filter by priority level
- **Type**: Filter by issue type
- **Labels**: Filter by labels (AND/OR logic)
- **Assignee**: Filter by assigned user
- **Date Range**: Filter by creation or update date
- **Text Search**: Full-text search in title and description

#### Query Builder

```
┌─ Filter Builder ─────────────────────────────────────┐
│                                                       │
│ Condition 1: Status = open                           │
│ Condition 2: Priority <= P2                          │
│ Condition 3: Label contains "bug"                    │
│ Condition 4: Assignee = @me                          │
│                                                       │
│ Logic: AND [✓]  OR [ ]                               │
│                                                       │
│ Preview: 8 issues match                              │
│                                                       │
│ [Save Filter] [Apply] [Cancel]                       │
└───────────────────────────────────────────────────────┘
```

---

### Filter Quick Select

Quick access to saved filters.

**Screenshot Placeholder:** `screenshot-34-filter-quick-select.png`
*Caption: Filter quick select menu*

#### Features

- **Saved Filters**: Access named filter presets
- **Recent Filters**: Last 5 used filters
- **Quick Apply**: One-click filter activation
- **Keyboard Shortcuts**: F1-F12 for saved filters

---

### Dependency Dialog

Add or remove dependencies between issues.

**Screenshot Placeholder:** `screenshot-35-dependency-dialog.png`
*Caption: Dependency dialog for managing issue relationships*

#### Features

- **Issue Search**: Find issues by ID or title
- **Dependency Type**: Choose "depends on" or "blocks"
- **Cycle Detection**: Prevent circular dependencies
- **Preview**: Show resulting dependency graph

---

### Label Picker

Select labels for issues with autocomplete and suggestions.

**Screenshot Placeholder:** `screenshot-36-label-picker.png`
*Caption: Label picker dialog with autocomplete*

#### Features

- **Autocomplete**: Type-ahead label suggestions
- **Multi-Select**: Toggle multiple labels
- **Usage Stats**: Show label usage count
- **Create New**: Add new labels inline
- **Label Dimensions**: Support namespaced labels

---

### Dialog Confirmation

Standard confirmation dialogs for destructive operations.

**Screenshot Placeholder:** `screenshot-37-confirmation-dialog.png`
*Caption: Confirmation dialog for deleting an issue*

#### Features

- **Clear Message**: Describes action consequences
- **Destructive Warning**: Highlights irreversible actions
- **Keyboard Shortcuts**: Enter to confirm, Esc to cancel
- **Focus Handling**: Default focus on safe option

Example:
```
┌─ Confirm Delete ─────────────────────────────────────┐
│                                                       │
│  Are you sure you want to delete this issue?         │
│                                                       │
│  Issue: beads-042 - Implement authentication         │
│                                                       │
│  ⚠️  This action cannot be undone.                    │
│                                                       │
│  [Cancel]  [Delete]                                   │
│     ▲                                                 │
│  (default)                                            │
└───────────────────────────────────────────────────────┘
```

---

### Help Overlay

Context-sensitive help overlay showing keyboard shortcuts.

**Screenshot Placeholder:** `screenshot-38-help-overlay.png`
*Caption: Help overlay showing keyboard shortcuts for current view*

#### Features

- **Context-Aware**: Shows shortcuts relevant to current view
- **Categories**: Organized by function
- **Search**: Filter shortcuts by key or action
- **Quick Reference**: Compact cheat sheet format

---

### Notification History

View past notifications and messages.

**Screenshot Placeholder:** `screenshot-39-notification-history.png`
*Caption: Notification history panel*

#### Features

- **Timeline View**: Chronological notification list
- **Filter by Type**: Show errors, warnings, info, or success
- **Click to Dismiss**: Remove individual notifications
- **Clear All**: Remove all notifications
- **Auto-expire**: Notifications fade after timeout

---

### Issue History Panel

View timeline of changes for an issue.

**Screenshot Placeholder:** `screenshot-40-issue-history.png`
*Caption: Issue history panel showing change timeline*

#### Features

- **Timeline Display**: Chronological change list
- **Field Changes**: Show before/after values
- **User Attribution**: Who made each change
- **Timestamps**: When changes occurred
- **Comment History**: Inline comments and discussions

Example:
```
┌─ Issue History: beads-042 ───────────────────────────┐
│                                                       │
│ 2026-01-14 10:30 AM - alice                          │
│   Status: open → in_progress                         │
│                                                       │
│ 2026-01-13 3:15 PM - bob                             │
│   Priority: P2 → P1                                  │
│   Comment: "Bumping priority - needed for release"   │
│                                                       │
│ 2026-01-12 9:00 AM - alice                           │
│   Labels: Added "security", "api"                    │
│                                                       │
│ 2026-01-10 2:45 PM - alice                           │
│   Created issue                                       │
│                                                       │
└───────────────────────────────────────────────────────┘
```

---

### Undo History Overlay

View and navigate undo/redo history.

**Screenshot Placeholder:** `screenshot-41-undo-history.png`
*Caption: Undo history overlay showing reversible operations*

#### Features

- **Operation List**: All reversible operations
- **Undo/Redo**: Navigate operation history
- **Branch Visualization**: Show undo branches
- **Operation Details**: What each operation changed

---

## Widgets and Components

### Tab Bar

The tab bar provides navigation between main views.

**Screenshot Placeholder:** `screenshot-42-tab-bar.png`
*Caption: Tab bar showing all main views*

#### Features

- **Active Indicator**: Highlight current tab
- **Badge Count**: Show notification counts per tab
- **Keyboard Navigation**: 1-5 for direct tab access
- **Mouse Support**: Click to switch tabs

---

### Status Bar

The status bar displays application state and quick actions.

**Screenshot Placeholder:** `screenshot-43-status-bar-detailed.png`
*Caption: Status bar with all information displayed*

#### Sections

- **Left**: Current operation or status message
- **Center**: Issue counts and statistics
- **Right**: Quick action hints and shortcuts

#### Status Messages

- Loading operations: "Loading issues..."
- Success: "Issue created successfully"
- Errors: "Failed to sync with remote"
- Info: "42 issues loaded"

---

### Toast Notifications

Temporary notification messages for user feedback.

**Screenshot Placeholder:** `screenshot-44-toast-notification.png`
*Caption: Toast notification showing success message*

#### Types

- **Success** (Green): Operation completed successfully
- **Error** (Red): Operation failed with error details
- **Warning** (Yellow): Potential issues or cautions
- **Info** (Blue): General information

#### Behavior

- Auto-dismiss after 3-5 seconds
- Click to dismiss immediately
- Stack multiple notifications
- Queue when screen is full

---

### Progress Indicators

Visual feedback for long-running operations.

**Screenshot Placeholder:** `screenshot-45-progress-indicator.png`
*Caption: Progress bar during database sync*

#### Types

- **Spinner**: Indeterminate progress
- **Progress Bar**: Determinate progress with percentage
- **Status Text**: Operation description

---

### Search Input

Search bar with autocomplete and filter suggestions.

**Screenshot Placeholder:** `screenshot-46-search-input.png`
*Caption: Search input with autocomplete suggestions*

#### Features

- **Live Search**: Results update as you type
- **History**: Recent search queries
- **Suggestions**: Autocomplete for labels, status, etc.
- **Clear Button**: Quick clear with Esc key

---

### Tree Widget

Hierarchical tree visualization for dependencies.

**Screenshot Placeholder:** `screenshot-47-tree-widget.png`
*Caption: Tree widget showing issue hierarchy*

#### Features

- **Expand/Collapse**: Toggle node visibility
- **Indentation**: Visual hierarchy levels
- **Icons**: Status and type indicators
- **Navigation**: Keyboard navigation through nodes

---

### Markdown Viewer

Rich text rendering for issue descriptions.

**Screenshot Placeholder:** `screenshot-48-markdown-viewer.png`
*Caption: Markdown viewer rendering formatted text*

#### Supported Elements

- **Headers**: H1-H6 with different sizes
- **Lists**: Ordered and unordered lists
- **Code Blocks**: Syntax highlighting
- **Links**: Clickable URLs
- **Emphasis**: Bold, italic, strikethrough
- **Blockquotes**: Indented quote blocks
- **Tables**: Formatted table rendering

---

### Skeleton Loaders

Placeholder content while loading data.

**Screenshot Placeholder:** `screenshot-49-skeleton-loader.png`
*Caption: Skeleton loader showing loading state*

#### Features

- **Animated**: Pulsing or shimmer effect
- **Shape-matching**: Mimics final content layout
- **Progressive**: Shows structure before content
- **Cancellable**: Can be interrupted

---

## Keyboard Shortcuts

### Global Shortcuts

Available in all views:

| Key | Action |
|-----|--------|
| `?` | Show help overlay |
| `q` | Quit (from main views) |
| `Ctrl+C` | Force quit |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `1-5` | Jump to tab by number |
| `Esc` | Go back / Close dialog |
| `Ctrl+U` | Undo last action |
| `Ctrl+R` | Redo last undone action |
| `Ctrl+H` | Show undo history |
| `Ctrl+N` | Show notification history |
| `F5` | Refresh current view |

---

### Navigation Shortcuts

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `h` or `←` | Move left / Collapse |
| `l` or `→` | Move right / Expand |
| `g` | Go to top |
| `Shift+G` | Go to bottom |
| `Ctrl+D` | Page down |
| `Ctrl+U` | Page up |
| `Enter` | Select / Activate |

---

### View-Specific Shortcuts Summary

#### Issues View

| Key | Action |
|-----|--------|
| `c` | Create new issue |
| `e` | Edit selected issue |
| `d` | Delete selected issue |
| `Shift+C` | Close selected issue |
| `Shift+S` | Change status |
| `p` | Change priority |
| `Shift+L` | Manage labels |
| `/` | Focus search |
| `f` | Open filter builder |
| `Shift+F` | Save filter |
| `Ctrl+M` | Column manager |
| `Space` | Toggle selection |
| `a` | Select all |
| `Shift+A` | Deselect all |

#### Dependencies View

| Key | Action |
|-----|--------|
| `a` | Add dependency |
| `d` | Remove dependency |
| `e` | Expand all |
| `c` | Collapse all |

#### Labels View

| Key | Action |
|-----|--------|
| `/` | Search labels |
| `Enter` | Filter by label |

#### Database View

| Key | Action |
|-----|--------|
| `s` | Sync with remote |
| `c` | Compact database |
| `v` | Vacuum database |
| `i` | Integrity check |
| `d` | Start/stop daemon |

#### Editor Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Esc` | Cancel |
| `Ctrl+Enter` | Save and continue |
| `Ctrl+L` | Focus labels |
| `Ctrl+P` | Priority selector |

---

## Accessibility Features

### Screen Reader Support

Enable screen reader support with the `--tts` flag:

```bash
beads-tui --tts
```

**Screenshot Placeholder:** `screenshot-50-accessibility.png`
*Caption: Accessibility mode enabled with screen reader announcements*

#### Features

- **Text-to-Speech**: Announces UI changes and navigation
- **Status Announcements**: Reads operation results
- **Navigation Hints**: Describes available actions
- **Error Messages**: Speaks error details clearly

#### Announced Events

- Tab changes: "Switched to Issues view"
- Issue selection: "Selected issue beads-042: Implement authentication"
- Operations: "Issue created successfully"
- Errors: "Failed to save issue: Title is required"
- Navigation: "5 items in list, item 3 selected"

---

### Keyboard-Only Navigation

All functionality is accessible via keyboard:

- **No Mouse Required**: Complete keyboard navigation
- **Focus Indicators**: Clear visual focus indicators
- **Tab Order**: Logical tab order through elements
- **Shortcuts**: Comprehensive keyboard shortcuts
- **Escape Hatch**: Esc key always cancels/goes back

---

### Visual Indicators

- **Color with Symbols**: Don't rely on color alone
- **High Contrast**: Clear contrast ratios
- **Focus Rings**: Visible focus indicators
- **Status Icons**: Icons supplement color coding

---

## Tips and Tricks

### Efficiency Tips

1. **Use Number Keys**: Press 1-5 to jump directly to tabs
2. **Vim Navigation**: Use j/k for up/down navigation
3. **Bulk Operations**: Use Space to select multiple issues, then apply batch actions
4. **Saved Filters**: Create filters for common queries and use F1-F12 for quick access
5. **Column Manager**: Hide unused columns to reduce clutter
6. **Undo/Redo**: Don't fear mistakes - Ctrl+U to undo any operation

### Search Tips

1. **Combine Filters**: `status:open label:bug priority:high`
2. **Use @me**: `assignee:@me` to find your issues
3. **Date Ranges**: `created:2026-01-01..2026-01-31`
4. **Negative Filters**: `status:!closed` (everything except closed)
5. **Wildcards**: `label:feature-*` (all labels starting with feature-)

### Workflow Tips

1. **Daily Standup**: Use saved filter "My Open Issues" (F1)
2. **Bug Triage**: Filter by `status:open type:bug`, sort by priority
3. **Sprint Planning**: Use Kanban view with assignee swimlanes
4. **Release Planning**: Use Gantt view to visualize timeline
5. **Dependency Audit**: Check Dependencies view for cycles

---

## Performance Optimization

### Large Datasets

For repositories with thousands of issues:

1. **Use Filters**: Narrow results before viewing
2. **Column Reduction**: Hide unnecessary columns
3. **Pagination**: Results are automatically paginated
4. **Lazy Loading**: Trees expand on demand
5. **Background Refresh**: Operations run in background

### Performance Stats

Enable performance stats (development mode):

```bash
RUST_LOG=debug beads-tui
```

Press `Shift+P` to toggle performance overlay showing:
- FPS (frames per second)
- Render time
- Event processing time
- Memory usage

**Screenshot Placeholder:** `screenshot-51-performance-stats.png`
*Caption: Performance stats overlay in debug mode*

---

## Troubleshooting

### Common Issues

#### Application Won't Start

- Check beads CLI is installed: `bd --version`
- Ensure .beads folder exists in current directory
- Run from project root: `cd /path/to/project && beads-tui`

#### UI Rendering Issues

- Verify terminal supports 256 colors
- Try different terminal emulator
- Check terminal size: minimum 80x24
- Update terminal fonts

#### Slow Performance

- Reduce visible columns
- Apply filters to limit results
- Check database size: might need compaction
- Close unused views

#### Keyboard Shortcuts Not Working

- Check terminal isn't capturing keys
- Disable terminal key bindings that conflict
- Try different terminal emulator

---

## Screenshot Capture Guide

To capture screenshots for this documentation:

### Required Screenshots

This guide references 51 screenshots. Use the following process to capture them:

#### Prerequisites

- Terminal emulator with screenshot capability (iTerm2, Windows Terminal, or use external tools)
- Sample beads repository with test data
- Screen capture tool (built-in or third-party)

#### Recommended Tools

**macOS:**
- iTerm2 built-in screenshot (Cmd+Shift+4, then select iTerm2 window)
- Screenshot.app (Cmd+Shift+5)

**Windows:**
- Snipping Tool (Win+Shift+S)
- ShareX (free, powerful screenshot tool)
- Windows Terminal has built-in screenshot support

**Linux:**
- gnome-screenshot
- flameshot
- scrot

**Cross-platform:**
- [Termshot](https://github.com/homeport/termshot) - Terminal screenshot tool
- [Carbon](https://carbon.now.sh) - Code snippet screenshots
- [Asciinema](https://asciinema.org) - Terminal recording (can export frames)

#### Screenshot Naming Convention

Save screenshots with the exact names referenced in this document:

- `screenshot-01-main-layout.png`
- `screenshot-02-status-bar.png`
- etc.

#### Capture Process

1. **Setup Test Data**: Import tutorial sample data
   ```bash
   bd import < docs/tutorial_sample_data/issues.jsonl
   ```

2. **Launch Application**:
   ```bash
   cargo run --release
   ```

3. **Navigate to Each View**: Follow the document order
   - Main layout (screenshot-01)
   - Status bar close-up (screenshot-02)
   - Issues view (screenshot-03)
   - etc.

4. **Capture Screenshots**: Use consistent terminal size (120x40 recommended)

5. **Post-Processing** (optional):
   - Crop to remove excess
   - Add annotations if needed
   - Optimize file size
   - Ensure readable text

6. **Save to Directory**:
   ```
   docs/screenshots/
   ├── screenshot-01-main-layout.png
   ├── screenshot-02-status-bar.png
   ├── screenshot-03-issues-view.png
   └── ...
   ```

7. **Update Document**: Replace placeholders with actual images
   ```markdown
   **Screenshot Placeholder:** `screenshot-01-main-layout.png`
   ```
   becomes:
   ```markdown
   ![Main Layout](screenshots/screenshot-01-main-layout.png)
   ```

#### Screenshot Checklist

Use this checklist to ensure all screenshots are captured:

**Main Views (14 screenshots)**
- [ ] 01: Main layout
- [ ] 02: Status bar
- [ ] 03: Issues view
- [ ] 04: Issues view filtered
- [ ] 05: Issues view bulk select
- [ ] 06: Search active
- [ ] 07: Dependencies view
- [ ] 08: Dependencies expanded
- [ ] 09: Dependency cycle
- [ ] 10: Labels view
- [ ] 11: Labels search
- [ ] 12: Database view
- [ ] 13: Database sync
- [ ] 14: Help view

**Help and Details (6 screenshots)**
- [ ] 15: Help search
- [ ] 16: Issue detail
- [ ] 17: Issue detail dependencies
- [ ] 18: Issue editor create
- [ ] 19: Issue editor validation
- [ ] 20: Issue editor autocomplete

**Advanced Views (6 screenshots)**
- [ ] 21: Gantt view
- [ ] 22: Gantt zoom
- [ ] 23: Kanban view
- [ ] 24: Kanban swimlanes
- [ ] 25: PERT view
- [ ] 26: PERT critical path

**Molecular Operations (5 screenshots)**
- [ ] 27: Formula browser
- [ ] 28: Pour wizard
- [ ] 29: Wisp manager
- [ ] 30: Bonding interface
- [ ] 31: History ops

**Dialogs and Overlays (11 screenshots)**
- [ ] 32: Column manager
- [ ] 33: Filter builder
- [ ] 34: Filter quick select
- [ ] 35: Dependency dialog
- [ ] 36: Label picker
- [ ] 37: Confirmation dialog
- [ ] 38: Help overlay
- [ ] 39: Notification history
- [ ] 40: Issue history
- [ ] 41: Undo history
- [ ] 42: Tab bar

**Widgets and Components (9 screenshots)**
- [ ] 43: Status bar detailed
- [ ] 44: Toast notification
- [ ] 45: Progress indicator
- [ ] 46: Search input
- [ ] 47: Tree widget
- [ ] 48: Markdown viewer
- [ ] 49: Skeleton loader
- [ ] 50: Accessibility mode
- [ ] 51: Performance stats

---

## Quick Reference Card

### Essential Shortcuts

```
Global               Issues View          Editing
─────────────────    ─────────────────    ─────────────────
q      Quit          c      Create        Ctrl+S  Save
?      Help          e      Edit          Esc     Cancel
Tab    Next tab      d      Delete        Tab     Next field
Esc    Go back       /      Search
1-5    Jump to tab   Space  Select        
                     a      Select all    
Navigation           p      Priority
─────────────────    Shift+S Status
j/k    Up/Down       Shift+L Labels
h/l    Left/Right    
Enter  Select        Dependencies
g      Top           ─────────────────
Shift+G Bottom       a      Add
                     d      Remove
                     e      Expand all
                     c      Collapse all
```

---

## Conclusion

This comprehensive UI reference documents all screens, components, and interactions in beads-tui. The application is designed for keyboard-first navigation with an intuitive, efficient workflow.

### Key Highlights

- **Rich Terminal UI**: Modern, responsive interface in the terminal
- **Keyboard Efficiency**: Complete keyboard navigation with vim-style bindings
- **Visual Dependency Management**: Interactive trees, Gantt charts, PERT diagrams
- **Powerful Filtering**: Advanced search and filter capabilities
- **Accessibility**: Screen reader support and keyboard-only operation
- **Extensibility**: Molecular chemistry operations for advanced workflows

### Next Steps

1. Capture screenshots using the guide above
2. Review with users for feedback
3. Update based on new features
4. Create video tutorials
5. Build interactive demo

### Related Documentation

- [USER_GUIDE.md](USER_GUIDE.md) - Getting started guide
- [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md) - Complete shortcut reference
- [FILTERING_GUIDE.md](FILTERING_GUIDE.md) - Advanced filtering patterns
- [TUTORIAL.md](TUTORIAL.md) - Step-by-step tutorial
- [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) - Contributing guide

---

**Document Version:** 1.0
**Last Updated:** 2026-01-14
**Status:** Ready for screenshots
**Total Screenshots Needed:** 51

*This document will be updated as new features are added to beads-tui.*
