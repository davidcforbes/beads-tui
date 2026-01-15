# Architecture and Design Decisions

## Overview
Beads-TUI is a professional-grade Rust terminal UI that wraps the beads CLI and presents a
stateful, keyboard-first interface with rich visualization capabilities. The architecture keeps UI rendering
pure, centralizes state in AppState, isolates CLI interactions, and supports async task management.

## Project Statistics
- **Total Rust Files**: 101 source files
- **Lines of Code**: ~73,000 LOC
- **Source Size**: 2.6 MB
- **UI Views**: 15+ specialized views
- **Reusable Widgets**: 34 components
- **Actions**: 66 keybindable actions
- **Themes**: 5 (including accessibility themes)

## High-Level Module Layout

### Core Modules
- **src/main.rs**: Application bootstrap, event loop, command routing, and rendering orchestration
- **src/lib.rs**: Library exports and public API
- **src/runtime.rs**: Global async runtime singleton (prevents 5-50ms overhead per operation)

### Business Logic
- **src/beads/**: Beads CLI integration layer
  - `client.rs` - Async BeadsClient with 30s timeout and retry logic
  - `models.rs` - Issue, Status, Priority, Type, Note data structures
  - `parser.rs` - Defensive JSON parsing with validation
  - `error.rs` - Custom error types with context
  - `mock.rs` - Mock backend for testing

### State Management
- **src/models/**: Domain models and application state
  - `app_state.rs` - Central AppState (single source of truth)
  - `filter.rs` - Issue filtering with saved filters (F3-F11 hotkeys)
  - `table_config.rs` - Column definitions and visibility
  - `kanban_config.rs` - Kanban board configuration with WIP limits
  - `gantt_schedule.rs` - Timeline scheduling and date derivation
  - `pert_layout.rs` - PERT chart layout with critical path analysis
  - `labels.rs` - Label parsing and normalization
  - `navigation.rs` - View navigation history
  - `issue_cache.rs` - Issue caching with statistics
  - `undo_history.rs` - Command history tracking (50 command capacity)

### User Interface
- **src/ui/views/**: Screen-level views (15+ views)
  - Issues, Dependencies, Labels, Kanban, Gantt, PERT, Database views
  - Help, Search, Detail, Editor, Graph views
  - Molecular views (BondingInterface, FormulaBrowser, PourWizard, WispManager, HistoryOps)
  - `issue_form_builder.rs` - Unified form builder for consistent layouts

- **src/ui/widgets/**: 34 reusable widgets
  - Core: Form, Dialog, TextEditor, SearchInput, TabBar, StatusBar, Toast
  - Lists: IssueList, CheckboxList, Tree, Selector, LabelPicker, Autocomplete
  - Complex: GanttChart, PertChart, KanbanCard, MarkdownViewer, ColumnManager
  - Filters: FilterBar, FilterBuilder, FilterPanel, FilterQuickSelect, FilterSaveDialog
  - Data: DatePicker, InlineMetadata, Progress, Skeleton
  - Overlays: HelpOverlay, BulkActionMenu, DependencyDialog, FieldEditor, IssueHistory, NotificationHistory, UndoHistory

- **src/ui/themes/**: Theme system with 5 themes
  - Dark (default), High-Contrast (WCAG AAA), Deuteranopia, Protanopia, Tritanopia

### Supporting Systems
- **src/config/**: Configuration management
  - `keybindings.rs` - 66 customizable actions with key mappings
  - `mod.rs` - YAML-based config with atomic writes and retry logic

- **src/tasks/**: Background task management
  - `manager.rs` - TaskManager for spawning and tracking async tasks
  - `types.rs` - TaskHandle, TaskId, TaskStatus, TaskOutput
  - Progress tracking, cancellation tokens, error propagation

- **src/undo/**: Undo/redo system
  - Command pattern implementation with 50-command stack
  - IssueUpdateCommand, IssueCreateCommand
  - Reversible operations with before/after snapshots

- **src/graph/**: Dependency graph algorithms
  - Layout engine for visual dependency graphs
  - Cycle detection
  - Topological sorting

- **src/tts/**: Text-to-speech support
  - Optional TTS manager for screen readers
  - Accessibility integration

- **src/utils/**: Utility functions
  - String manipulation, date formatting, validation helpers

## Data Flow Architecture

### Input Flow
```
User Input → Event Handler → Action Router → State Mutation → Render Queue
```

### Data Flow
```
Beads CLI → JSON Output → Defensive Parser → Models → AppState → View Rendering
```

### Rendering Pipeline
```
AppState → Pure Render Functions → Ratatui Widgets → Terminal Buffer → Screen
```

### Command Execution
```
User Action → Validation → CLI Command → Async Task → Result Handler → Notification
```

## Core Design Patterns

### 1. Stateful Widget Pattern
Views follow ratatui's StatefulWidget pattern: widgets are pure rendering functions that accept separate state structs.

```rust
impl StatefulWidget for IssuesView {
    type State = IssuesViewState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Pure rendering from state
    }
}
```

### 2. Command Pattern
The undo system uses the Command pattern for reversible operations:

```rust
trait Command {
    fn execute(&mut self, app: &mut AppState) -> Result<()>;
    fn undo(&mut self, app: &mut AppState) -> Result<()>;
}
```

### 3. Builder Pattern
Complex form construction uses the builder pattern for clarity:

```rust
FormField::text("title", "Title")
    .required()
    .placeholder("Brief description")
    .with_validation(ValidationRule::MaxLength(256))
```

### 4. Observer Pattern
Task notifications propagate to the UI through the observer pattern:

```rust
task_manager.spawn_task(|progress| {
    // Report progress updates
    progress.report(0.5, "Syncing...");
});
```

## Design Decisions

### CLI-First Architecture
All data mutations go through the beads CLI, not direct database writes. This ensures:
- Consistency with beads' transaction model
- Audit trail through git commits
- Compatibility with multi-user scenarios

### View Isolation
Each view owns its own state to limit coupling:
- IssuesViewState, KanbanViewState, GanttViewState are independent
- Views can be tested in isolation
- State serialization for persistence

### Defensive Parsing
All CLI responses are validated before entering AppState:
- JSON schema validation
- Type checking with serde
- Default values for missing fields
- Error recovery for malformed responses

### Predictable Rendering
Render functions avoid allocation-heavy work:
- Pre-computed layouts cached in state
- Lazy evaluation for expensive calculations
- Incremental rendering with dirty flags
- 60 FPS target for smooth interactions

### Async Task Management
Long-running operations use the global tokio runtime:
- Background task spawning without blocking UI
- Progress reporting and cancellation
- Error propagation with context
- Task history (last 20 completed tasks)

### Error Handling and Feedback
- CLI failures surface as notifications rather than panics
- Long operations show progress indicators
- Operations can be canceled (Ctrl+C in task context)
- Notification history panel (Ctrl+H) for review

## State Management Deep Dive

### AppState Structure
```rust
pub struct AppState {
    // Navigation
    selected_tab: usize,
    tabs: Vec<&'static str>,

    // Beads Integration
    beads_client: BeadsClient,

    // View States (15+ specialized states)
    issues_view_state: IssuesViewState,
    dependencies_view_state: DependenciesViewState,
    labels_view_state: LabelsViewState,
    kanban_view_state: KanbanViewState,
    gantt_view_state: GanttViewState,
    pert_view_state: PertViewState,
    database_view_state: DatabaseViewState,
    // ... more view states

    // UI Overlays
    dialog_state: Option<DialogState>,
    show_notification_history: bool,
    show_issue_history: bool,
    show_label_picker: bool,
    column_manager_state: Option<ColumnManagerState>,

    // Task Management
    task_manager: TaskManager,
    loading_spinner: Option<Spinner>,
    loading_message: Option<String>,

    // Notifications
    notifications: Vec<NotificationMessage>,
    notification_history: VecDeque<NotificationMessage>, // max 100

    // Undo/Redo
    undo_stack: UndoStack, // 50 command capacity

    // Configuration
    config: Config,
    theme: Theme,

    // Dirty Tracking
    dirty: bool, // Triggers re-render
}
```

### View States
Each major view has its own state struct:

**IssuesViewState**:
- Mode: List, Detail, Edit, Create, SplitScreen
- SearchState with filtering
- SplitScreenFocus (List or Detail panel)
- Selected issue index
- Scroll positions
- Editor state (when in Edit mode)
- Create form state (when in Create mode)

**KanbanViewState**:
- GroupingMode: Status, Assignee, Label, Priority
- Column definitions with WIP limits
- Card positions and drag state
- Column collapse state
- Selected card index

**GanttViewState**:
- Timeline configuration
- Zoom level
- Swim lane grouping
- Schedule data (derived from estimates and dates)
- Visible date range

## Form System Architecture

### Unified Form Builder
The `issue_form_builder.rs` module provides consistent form layouts across all views:

```rust
pub enum IssueFormMode {
    Read,   // Display-only in Detail view
    Edit,   // Editable in Edit mode
    Create, // New issue creation
}

pub fn build_issue_form(mode: IssueFormMode, issue: Option<&Issue>) -> Vec<FormField>
```

This ensures identical field ordering and labels in:
- Detail View (read-only)
- Edit View (editable with validation)
- Create View (new issue with defaults)
- Split Screen Detail (read-only right panel)

### Form Field Types
```rust
pub enum FieldType {
    Text,       // Single-line input
    Password,   // Masked input
    TextArea,   // Multi-line input
    Selector,   // Dropdown menu
    ReadOnly,   // Display-only field
}
```

### Validation Rules
```rust
pub enum ValidationRule {
    Required,
    Enum(Vec<String>),
    PositiveInteger,
    BeadsIdFormat,       // beads-xxx-xxxx pattern
    NoSpaces,
    Date,
    FutureDate,
    MaxLength(usize),
}
```

## Search Architecture

### Current Implementation (V1)
- Full-text search with multiple modes
- Substring matching (case-insensitive by default)
- Fuzzy search using SkimMatcherV2
- Regex search with safe_regex_match
- Label-based filtering (Any/All logic)
- Status/Priority/Type filtering
- Cached filtering for performance

### Planned V2 Features
See [SEARCH_ARCHITECTURE.md](SEARCH_ARCHITECTURE.md) for details on:
- Unified query language (key:value syntax)
- Query parser for structured searches
- Hybrid scoring (exact, fuzzy, BM25/TF-IDF)
- Recency boosting
- Semantic search (future with fastembed-rs)

## Testing Architecture

### Test Infrastructure
- `insta` - Snapshot testing for UI layouts
- `proptest` - Property-based testing for algorithms
- `criterion` - Performance benchmarking
- `mockall` - Mocking for unit tests
- `assert_cmd` - CLI integration tests
- `serial_test` - Sequential test execution

### Test Strategy
- Unit tests for models and helpers
- Integration tests for CLI workflows
- Snapshot tests for UI rendering
- Property tests for search algorithms
- Benchmarks for performance-critical code

See [TEST_STRATEGY.md](../TEST_STRATEGY.md) for complete testing approach.

## Performance Considerations

### Optimization Techniques
1. **Dirty Flag Pattern**: Only re-render when state changes
2. **Lazy Evaluation**: Defer expensive computations until needed
3. **Caching**: Cache filtered issue lists, computed layouts
4. **Batch Operations**: Group multiple updates into single render
5. **Global Runtime**: Reuse tokio runtime to avoid startup overhead

### Performance Targets
- 60 FPS rendering for smooth interactions
- <100ms response time for user actions
- <1s for CLI operations (sync, list, filter)
- <5s for complex operations (graph layout, PERT analysis)

## Accessibility Features

### Visual Accessibility
- High-contrast theme (WCAG AAA compliant)
- Color-blind friendly themes (Deuteranopia, Protanopia, Tritanopia)
- Clear visual indicators for focus and selection
- Status conveyed through multiple channels (color + icon + text)

### Keyboard Accessibility
- Complete keyboard navigation (no mouse required)
- Vim-style shortcuts (j/k, h/l, g/G)
- Context-sensitive help (F1)
- Keyboard shortcut overlay (?)

### Screen Reader Support
- Optional TTS manager (--tts flag)
- Descriptive labels for all UI elements
- Announcement of state changes

## Future Architecture Improvements

### Planned Enhancements
1. **Plugin System**: Extensible architecture for custom views and widgets
2. **Event Sourcing**: Complete audit trail with replay capability
3. **Offline Mode**: Work without network connectivity
4. **Collaboration**: Real-time multi-user support
5. **AI Integration**: Semantic search, auto-labeling, prediction

### Technical Debt
- Molecular views need refactoring for clarity
- Some views have duplicate code (opportunity for abstraction)
- Test coverage could be improved (target: 80%+)
- Documentation for internal APIs

## Related Documentation

- [SEARCH_ARCHITECTURE.md](SEARCH_ARCHITECTURE.md) - Search V2 design
- [TEST_STRATEGY.md](../TEST_STRATEGY.md) - Testing approach
- [FILTERING_GUIDE.md](FILTERING_GUIDE.md) - Filter system details
- [widgets.md](widgets.md) - Widget catalog
- [USER_GUIDE.md](USER_GUIDE.md) - End-user documentation
