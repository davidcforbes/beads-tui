# Beads TUI Widget Library Documentation

This document provides a comprehensive reference for all widgets available in the beads-tui application. The widget library is built on [Ratatui](https://github.com/ratatui-org/ratatui), a Rust library for building terminal user interfaces.

## Table of Contents

1. [Overview](#overview)
2. [Core Widgets](#core-widgets)
   - [StatusBar](#statusbar)
   - [TabBar](#tabbar)
   - [Progress](#progress)
3. [List & Selection Widgets](#list--selection-widgets)
   - [IssueList](#issuelist)
   - [CheckboxList](#checkboxlist)
   - [Selector](#selector)
   - [LabelPicker](#labelpicker)
4. [Input Widgets](#input-widgets)
   - [SearchInput](#searchinput)
   - [FieldEditor](#fieldeditor)
   - [TextEditor](#texteditor)
   - [Autocomplete](#autocomplete)
5. [Form & Dialog Widgets](#form--dialog-widgets)
   - [Form](#form)
   - [Dialog](#dialog)
   - [FilterSaveDialog](#filtersavedialog)
6. [Filter Widgets](#filter-widgets)
   - [FilterBuilder](#filterbuilder)
   - [FilterPanel](#filterpanel)
7. [Display Widgets](#display-widgets)
   - [MarkdownViewer](#markdownviewer)
   - [InlineMetadata](#inlinemetadata)
   - [DatePicker](#datepicker)
8. [Action Widgets](#action-widgets)
   - [BulkActionMenu](#bulkactionmenu)
9. [Widget Patterns](#widget-patterns)

---

## Overview

The beads-tui widget library follows consistent patterns across all widgets:

- **StatefulWidget Pattern**: Most widgets use the StatefulWidget trait, separating widget configuration (the widget struct) from runtime state (the state struct).
- **Builder Pattern**: Widgets use method chaining for configuration.
- **State Management**: Widget state is maintained separately from the widget itself, allowing for clean separation of concerns.
- **Color Coding**: Consistent use of colors for different issue types, statuses, and priorities.
- **Focus States**: Input widgets have distinct visual styles when focused.

---

## Core Widgets

### StatusBar

A three-section status bar for displaying context information, current mode, and statistics.

#### Purpose
Displays contextual information at the bottom of the screen with left, center, and right sections.

#### Use Cases
- Showing current view context (e.g., "Issue List", "Viewing Issue #123")
- Displaying current application mode or state
- Showing statistics (e.g., "5 of 20 issues")

#### Structure

No state struct - this is a stateless widget.

**Widget: `StatusBar<'a>`**
- `left: Vec<Span<'a>>` - Spans for left section
- `center: Vec<Span<'a>>` - Spans for center section
- `right: Vec<Span<'a>>` - Spans for right section

#### Key Methods

```rust
pub fn new() -> Self
pub fn left(mut self, spans: Vec<Span<'a>>) -> Self
pub fn center(mut self, spans: Vec<Span<'a>>) -> Self
pub fn right(mut self, spans: Vec<Span<'a>>) -> Self
pub fn render(&self, area: Rect, buf: &mut Buffer)
```

#### Example Usage

```rust
use ratatui::text::Span;
use ratatui::style::{Color, Style};

let status = StatusBar::new()
    .left(vec![Span::styled("Issue List", Style::default().fg(Color::White))])
    .center(vec![Span::styled("NORMAL", Style::default().fg(Color::Cyan))])
    .right(vec![Span::raw("5 of 20 issues")]);

// Render using Widget trait
status.render(status_area, buf);
```

#### Styling/Configuration
- Left section: Gray text, left-aligned
- Center section: Cyan text, center-aligned
- Right section: Gray text, right-aligned
- Three equal sections (33%, 34%, 33% of width)

---

### TabBar

A horizontal tab navigation widget for switching between views.

#### Purpose
Provides tabbed navigation with visual indication of the selected tab.

#### Use Cases
- Main application navigation (Issues, Search, Filters, etc.)
- View switching within a section

#### Structure

No state struct - tab selection is passed via builder method.

**Widget: `TabBar<'a>`**
- `tabs: Vec<&'a str>` - Tab labels
- `selected: usize` - Currently selected tab index
- `block: Option<Block<'a>>` - Optional border block

#### Key Methods

```rust
pub fn new(tabs: Vec<&'a str>) -> Self
pub fn selected(mut self, index: usize) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn render(&self, area: Rect, buf: &mut Buffer)
```

#### Example Usage

```rust
use ratatui::widgets::Block;

let tabs = TabBar::new(vec!["Issues", "Search", "Filters"])
    .selected(0)
    .block(Block::default().borders(Borders::ALL).title("Navigation"));

tabs.render(tab_area, buf);
```

#### Styling/Configuration
- Selected tab: Yellow text, bold
- Unselected tabs: White text
- Tab format: " {number} {name} " (e.g., " 1 Issues ")

---

### Progress

Progress indicators including spinners, progress bars, and loading indicators.

#### Purpose
Display progress for long-running operations, both determinate and indeterminate.

#### Use Cases
- Loading data from files
- Processing batch operations
- Network requests
- Background tasks

#### Components

**Spinner**
- `start_time: Instant` - When spinner started
- `frame_duration: Duration` - Time between animation frames (default: 80ms)
- `style: Style` - Spinner color/style
- `label: Option<String>` - Optional text label

**ProgressBar**
- `ratio: f64` - Progress ratio (0.0 to 1.0)
- `label: Option<String>` - Optional label
- `style: Style` - Container style
- `gauge_style: Style` - Filled portion style

**LoadingIndicator**
- `message: String` - Loading message
- `progress: Option<f64>` - Optional progress (None = spinner, Some = bar)
- `style: Style` - Widget style
- `start_time: Instant` - Start time for spinner animation

#### Key Methods

**Spinner:**
```rust
pub fn new() -> Self
pub fn style(mut self, style: Style) -> Self
pub fn label<S: Into<String>>(mut self, label: S) -> Self
pub fn frame_char(&self) -> &'static str
```

**ProgressBar:**
```rust
pub fn new(ratio: f64) -> Self
pub fn ratio(mut self, ratio: f64) -> Self
pub fn label<S: Into<String>>(mut self, label: S) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn gauge_style(mut self, style: Style) -> Self
pub fn percentage(&self) -> u16
```

**LoadingIndicator:**
```rust
pub fn new<S: Into<String>>(message: S) -> Self
pub fn with_progress<S: Into<String>>(message: S, progress: f64) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn set_progress(&mut self, progress: f64)
pub fn set_message<S: Into<String>>(&mut self, message: S)
```

#### Example Usage

```rust
// Indeterminate progress (spinner)
let spinner = Spinner::new()
    .label("Loading issues...")
    .style(Style::default().fg(Color::Cyan));
spinner.render(area, buf);

// Determinate progress (progress bar)
let progress = ProgressBar::new(0.75)
    .label("Processing")
    .gauge_style(Style::default().bg(Color::Blue).fg(Color::White));
progress.render(area, buf);

// Combined loading indicator
let loading = LoadingIndicator::with_progress("Syncing...", 0.6);
loading.render(area, buf);
```

#### Styling/Configuration
- **Spinner**: Cyan color, 10-frame animation
- **Progress Bar**: Blue background for filled portion, white text
- **Percentage**: Automatically displayed (0-100%)

---

## List & Selection Widgets

### IssueList

A sortable, filterable table displaying issues with multi-column layout.

#### Purpose
Display a list of issues with sorting, filtering, and search highlighting capabilities.

#### Use Cases
- Main issue list view
- Filtered issue results
- Search results
- Dependency/blocker lists

#### State Structure

**IssueListState**
- `table_state: TableState` - Ratatui table selection state
- `sort_column: SortColumn` - Currently sorted column
- `sort_direction: SortDirection` - Ascending or descending

**Enums:**
```rust
enum SortColumn { Id, Title, Status, Priority, Type, Created, Updated }
enum SortDirection { Ascending, Descending }
```

#### Key Methods

**State:**
```rust
pub fn new() -> Self
pub fn select_next(&mut self, len: usize)
pub fn select_previous(&mut self, len: usize)
pub fn selected(&self) -> Option<usize>
pub fn select(&mut self, index: Option<usize>)
pub fn sort_by(&mut self, column: SortColumn)
pub fn sort_column(&self) -> SortColumn
pub fn sort_direction(&self) -> SortDirection
```

**Widget:**
```rust
pub fn new(issues: Vec<&'a Issue>) -> Self
pub fn with_sort(mut self, column: SortColumn, direction: SortDirection) -> Self
pub fn show_details(mut self, show: bool) -> Self
pub fn search_query(mut self, query: Option<String>) -> Self
```

**Helper Methods:**
```rust
fn highlight_text(text: &str, query: &str) -> Vec<Span<'static>>
fn priority_color(priority: &Priority) -> Color
fn status_color(status: &IssueStatus) -> Color
fn type_symbol(issue_type: &IssueType) -> &'static str
```

#### Example Usage

```rust
let mut issues: Vec<&Issue> = all_issues.iter().collect();
let mut state = IssueListState::new();

let list = IssueList::new(issues)
    .with_sort(SortColumn::Updated, SortDirection::Descending)
    .search_query(Some("authentication".to_string()));

// Render
StatefulWidget::render(list, area, buf, &mut state);

// Navigation
state.select_next(issues.len());
state.select_previous(issues.len());

// Sorting
state.sort_by(SortColumn::Priority);

// Get selection
if let Some(idx) = state.selected() {
    let selected_issue = &issues[idx];
}
```

#### Styling/Configuration
- **Columns**: Type (emoji), ID, Title, Status, Priority
- **Column Widths**: Type(6), ID(15), Title(Min 30), Status(12), Priority(10)
- **Sort Indicator**: ▲ (ascending) / ▼ (descending) in column headers
- **Selection**: Dark gray background, bold text, ">> " prefix
- **Search Highlighting**: Yellow background, black text, bold
- **Priority Colors**: P0=Red, P1=LightRed, P2=Yellow, P3=Blue, P4=Gray
- **Status Colors**: Open=Green, InProgress=Cyan, Blocked=Red, Closed=Gray
- **Type Symbols**: Bug=🐛, Feature=✨, Task=📋, Epic=🎯, Chore=🔧

---

### CheckboxList

A multi-selection list widget with checkbox indicators.

#### Purpose
Allow users to select multiple items from a list using checkboxes.

#### Use Cases
- Bulk issue selection
- Multi-label selection
- Batch operation item selection

#### State Structure

**CheckboxListState<T>** (generic over item type)
- `items: Vec<T>` - List items
- `selected: HashSet<usize>` - Selected item indices
- `list_state: ListState` - Current highlight position
- `selection_mode: bool` - Whether selection mode is active

#### Key Methods

```rust
pub fn new(items: Vec<T>) -> Self
pub fn items(&self) -> &[T]
pub fn set_items(&mut self, items: Vec<T>)
pub fn selected_indices(&self) -> Vec<usize>
pub fn selected_items(&self) -> Vec<T>
pub fn is_selected(&self, index: usize) -> bool
pub fn is_selection_mode(&self) -> bool
pub fn set_selection_mode(&mut self, enabled: bool)
pub fn toggle_selection_mode(&mut self)
pub fn toggle_selected(&mut self)
pub fn select_all(&mut self)
pub fn deselect_all(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn item_count(&self) -> usize
pub fn selected_count(&self) -> usize
pub fn highlighted_index(&self) -> Option<usize>
```

**Widget:**
```rust
pub fn new(item_formatter: F) -> Self  // F: Fn(&T) -> String
pub fn title(mut self, title: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn selected_style(mut self, style: Style) -> Self
pub fn checkbox_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn show_count(mut self, show: bool) -> Self
```

#### Example Usage

```rust
let items = vec!["Bug #123", "Feature #124", "Task #125"];
let mut state: CheckboxListState<&str> = CheckboxListState::new(items);

// Enable selection mode
state.set_selection_mode(true);

// Create widget with formatter
let list = CheckboxList::new(|item: &&str| item.to_string())
    .title("Select Issues")
    .show_count(true);

StatefulWidget::render(list, area, buf, &mut state);

// Toggle selection of highlighted item
state.toggle_selected();

// Select all
state.select_all();

// Get selected items
let selected = state.selected_items();
```

#### Styling/Configuration
- **Checked**: [✓] Green checkbox
- **Unchecked**: [ ] Green checkbox outline
- **Title Format**: "Items [2/5]" (selected/total) when in selection mode
- **Highlight**: Dark gray background, bold
- **Highlight Symbol**: "> "
- **Empty State**: "No items" in gray italic

---

### Selector

Dropdown selector widgets for Priority, Status, and Type enums.

#### Purpose
Provide dropdown selection for enumerated values (Priority, Status, Type).

#### Use Cases
- Issue creation/editing forms
- Quick status changes
- Filtering by single value

#### State Structure

**SelectorState<T>** (generic over enum type)
- `options: Vec<T>` - Available options
- `selected_index: usize` - Currently selected option
- `list_state: ListState` - Dropdown list state
- `is_open: bool` - Whether dropdown is shown

#### Key Methods

```rust
pub fn new(options: Vec<T>, selected: usize) -> Self
pub fn selected(&self) -> &T
pub fn selected_index(&self) -> usize
pub fn set_selected(&mut self, value: T)
pub fn is_open(&self) -> bool
pub fn toggle(&mut self)
pub fn open(&mut self)
pub fn close(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn confirm_selection(&mut self)
```

**Specialized Widgets:**
- `PrioritySelector<'a>` - For Priority enum
- `StatusSelector<'a>` - For IssueStatus enum
- `TypeSelector<'a>` - For IssueType enum

#### Example Usage

```rust
// Priority selector
let priorities = vec![Priority::P0, Priority::P1, Priority::P2, Priority::P3, Priority::P4];
let mut state = SelectorState::new(priorities, 2);  // Default to P2

let selector = PrioritySelector::new()
    .title("Priority")
    .placeholder("Select priority...");

StatefulWidget::render(selector, area, buf, &mut state);

// Toggle dropdown
state.toggle();

// Navigate
state.select_next();
state.select_previous();

// Confirm selection
state.confirm_selection();
let selected_priority = state.selected();
```

#### Styling/Configuration
- **Closed**: Shows selected value or placeholder
- **Open**: Dropdown list with up to 5 items visible
- **Colors**:
  - Priority: P0=Red, P1=LightRed, P2=Yellow, P3=Blue, P4=Gray
  - Status: Open=Green, InProgress=Cyan, Blocked=Red, Closed=Gray
  - Type: Bug=Red, Feature=Green, Task=Blue, Epic=Magenta, Chore=Yellow
- **Focus**: Cyan border, bold
- **Highlight**: Dark gray background
- **Symbol**: "▼" when closed, "▲" when open

---

### LabelPicker

A multi-select label picker with real-time filtering.

#### Purpose
Allow users to select multiple labels with search/filter capabilities.

#### Use Cases
- Adding labels to issues
- Filtering by multiple labels
- Label management

#### State Structure

**LabelPickerState**
- `available_labels: Vec<String>` - All available labels
- `selected_labels: HashSet<String>` - Currently selected labels
- `filter_query: String` - Current filter text
- `cursor_position: usize` - Cursor position in filter field
- `list_state: ListState` - List selection state
- `is_filtering: bool` - Whether in filter mode

#### Key Methods

```rust
pub fn new(available_labels: Vec<String>) -> Self
pub fn with_selected(mut self, selected: Vec<String>) -> Self
pub fn selected_labels(&self) -> Vec<String>
pub fn is_selected(&self, label: &str) -> bool
pub fn toggle_selected(&mut self, label: String)
pub fn add_label(&mut self, label: String)
pub fn remove_label(&mut self, label: &str)
pub fn clear_selected(&mut self)
pub fn filtered_labels(&self) -> Vec<String>
pub fn filter_query(&self) -> &str
pub fn start_filtering(&mut self)
pub fn stop_filtering(&mut self)
pub fn is_filtering(&self) -> bool
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn show_filter(mut self, show: bool) -> Self
pub fn max_visible(mut self, max: usize) -> Self
```

#### Example Usage

```rust
let available = vec!["bug".to_string(), "urgent".to_string(), "frontend".to_string()];
let mut state = LabelPickerState::new(available)
    .with_selected(vec!["bug".to_string()]);

let picker = LabelPicker::new()
    .title("Labels")
    .show_filter(true)
    .max_visible(10);

StatefulWidget::render(picker, area, buf, &mut state);

// Start filtering
state.start_filtering();
state.insert_char('u');
state.insert_char('r');

// Toggle selection
state.toggle_selected("urgent".to_string());

// Get selected
let selected = state.selected_labels();
```

#### Styling/Configuration
- **Selected**: ✓ prefix, green color
- **Unselected**: No prefix, white color
- **Filter Input**: Shows when `is_filtering` is true
- **Filter Placeholder**: "Type to filter..."
- **Cursor**: White background when filtering
- **Highlight**: Dark gray background
- **Case-insensitive** filtering

---

## Input Widgets

### SearchInput

A search input field with history navigation.

#### Purpose
Provide search functionality with command-line-like history navigation.

#### Use Cases
- Issue search
- Text filtering
- Any text search with history

#### State Structure

**SearchInputState**
- `query: String` - Current search query
- `cursor_position: usize` - Cursor position
- `history: Vec<String>` - Search history (max 50)
- `history_index: Option<usize>` - Current position in history
- `temp_query: String` - Temporary storage when navigating history

#### Key Methods

```rust
pub fn new() -> Self
pub fn query(&self) -> &str
pub fn set_query<S: Into<String>>(&mut self, query: S)
pub fn cursor_position(&self) -> usize
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn move_cursor_to_start(&mut self)
pub fn move_cursor_to_end(&mut self)
pub fn clear(&mut self)
pub fn history(&self) -> &[String]
pub fn add_to_history(&mut self, query: String)
pub fn history_previous(&mut self)
pub fn history_next(&mut self)
pub fn confirm_query(&mut self)
```

**Widget:**
```rust
pub fn new() -> Self
pub fn placeholder(mut self, placeholder: &'a str) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
```

#### Example Usage

```rust
let mut state = SearchInputState::new();

let input = SearchInput::new()
    .placeholder("Search issues...")
    .focused_style(Style::default().fg(Color::Cyan));

// Handle input
state.insert_char('t');
state.insert_char('e');
state.insert_char('s');
state.insert_char('t');

// Confirm and add to history
state.confirm_query();

// Navigate history
state.history_previous();  // Get previous search
state.history_next();      // Move forward in history

// Render
StatefulWidget::render(input, area, buf, &mut state);
```

#### Styling/Configuration
- **Default**: Gray border
- **Focused**: Cyan border, bold
- **Placeholder**: Dark gray, italic
- **Cursor**: White background, black text
- **History**: Max 50 entries, duplicates removed
- **History Navigation**: Up/Down arrow keys (application logic)

---

### FieldEditor

A single-line or multi-line text editor field.

#### Purpose
Simple text input field for forms and dialogs.

#### Use Cases
- Form fields (title, description)
- Inline text editing
- Dialog inputs

#### State Structure

**FieldEditorState**
- `content: String` - Field content
- `cursor_position: usize` - Cursor position
- `mode: EditorMode` - SingleLine or MultiLine
- `is_focused: bool` - Focus state

**EditorMode:**
```rust
enum EditorMode {
    SingleLine,  // Ignores newlines
    MultiLine,   // Accepts newlines
}
```

#### Key Methods

```rust
pub fn new() -> Self
pub fn with_content<S: Into<String>>(mut self, content: S) -> Self
pub fn with_mode(mut self, mode: EditorMode) -> Self
pub fn content(&self) -> &str
pub fn set_content<S: Into<String>>(&mut self, content: S)
pub fn clear(&mut self)
pub fn cursor_position(&self) -> usize
pub fn set_focused(&mut self, focused: bool)
pub fn is_focused(&self) -> bool
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn delete_char_forward(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn move_cursor_to_start(&mut self)
pub fn move_cursor_to_end(&mut self)
pub fn cursor_line_col(&self) -> (usize, usize)
```

**Widget:**
```rust
pub fn new() -> Self
pub fn label(mut self, label: &'a str) -> Self
pub fn placeholder(mut self, placeholder: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
```

#### Example Usage

```rust
// Single-line field
let mut title_state = FieldEditorState::new()
    .with_mode(EditorMode::SingleLine);

let title_field = FieldEditor::new()
    .label("Title")
    .placeholder("Enter issue title");

// Multi-line field
let mut desc_state = FieldEditorState::new()
    .with_mode(EditorMode::MultiLine)
    .with_content("Initial content");

let desc_field = FieldEditor::new()
    .label("Description")
    .placeholder("Enter description");

// Render
StatefulWidget::render(title_field, area, buf, &mut title_state);

// Handle input
title_state.insert_char('T');
title_state.insert_char('e');
title_state.insert_char('s');
title_state.insert_char('t');
```

#### Styling/Configuration
- **Default**: White border
- **Focused**: Cyan border, bold, "[editing]" in title
- **Placeholder**: Dark gray, italic
- **Cursor**: White background, black text (when focused)
- **SingleLine**: Newlines ignored
- **MultiLine**: Newlines preserved, shows line/column

---

### TextEditor

A full-featured multi-line text editor with scrolling.

#### Purpose
Edit large blocks of text with proper line management and scrolling.

#### Use Cases
- Issue descriptions
- Notes editing
- Multi-line form fields
- Code snippets

#### State Structure

**TextEditorState**
- `lines: Vec<String>` - Text content as lines
- `cursor_line: usize` - Current line (0-based)
- `cursor_col: usize` - Column in current line
- `scroll_offset: usize` - First visible line
- `is_focused: bool` - Focus state
- `max_lines: Option<usize>` - Optional line limit

#### Key Methods

```rust
pub fn new() -> Self
pub fn text(&self) -> String
pub fn set_text<S: Into<String>>(&mut self, text: S)
pub fn lines(&self) -> &[String]
pub fn cursor_position(&self) -> (usize, usize)
pub fn set_focused(&mut self, focused: bool)
pub fn is_focused(&self) -> bool
pub fn set_max_lines(&mut self, max: Option<usize>)
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn delete_char_forward(&mut self)
pub fn move_cursor_up(&mut self)
pub fn move_cursor_down(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn move_cursor_to_line_start(&mut self)
pub fn move_cursor_to_line_end(&mut self)
pub fn move_cursor_to_start(&mut self)
pub fn move_cursor_to_end(&mut self)
pub fn clear(&mut self)
pub fn line_count(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn scroll_offset(&self) -> usize
```

**Widget:**
```rust
pub fn new() -> Self
pub fn placeholder(mut self, placeholder: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn show_line_numbers(mut self, show: bool) -> Self
pub fn wrap(mut self, wrap: bool) -> Self
```

#### Example Usage

```rust
let mut state = TextEditorState::new();
state.set_text("Line 1\nLine 2\nLine 3");

let editor = TextEditor::new()
    .placeholder("Enter text...")
    .show_line_numbers(true)
    .wrap(true)
    .block(Block::default().borders(Borders::ALL).title("Description"));

StatefulWidget::render(editor, area, buf, &mut state);

// Multi-line navigation
state.insert_char('H');
state.insert_char('i');
state.insert_char('\n');  // New line
state.insert_char('B');
state.insert_char('y');
state.insert_char('e');

// Line merging (backspace at line start)
state.move_cursor_to_line_start();
state.delete_char();  // Merges with previous line
```

#### Styling/Configuration
- **Title Format**: "Text [5L]" (shows line count when focused)
- **Placeholder**: Dark gray, italic
- **Line Numbers**: Dark gray, right-aligned with padding
- **Cursor**: White background, black text
- **Auto-scrolling**: Keeps cursor visible
- **Word Wrap**: Optional (default: enabled)

---

### Autocomplete

Text input with autocomplete suggestions dropdown.

#### Purpose
Provide autocomplete functionality for text input with live filtering.

#### Use Cases
- Assignee selection
- Label input with suggestions
- Any text field with known options

#### State Structure

**AutocompleteState**
- `options: Vec<String>` - Available suggestions
- `input: String` - Current input text
- `cursor_position: usize` - Cursor position
- `list_state: ListState` - Suggestion list state
- `is_focused: bool` - Focus state
- `show_suggestions: bool` - Whether dropdown is shown
- `selected_value: Option<String>` - Confirmed selection

#### Key Methods

```rust
pub fn new() -> Self
pub fn set_options(&mut self, options: Vec<String>)
pub fn input(&self) -> &str
pub fn selected_value(&self) -> Option<&str>
pub fn set_selected_value<S: Into<String>>(&mut self, value: Option<S>)
pub fn clear_selected(&mut self)
pub fn set_focused(&mut self, focused: bool)
pub fn is_focused(&self) -> bool
pub fn set_show_suggestions(&mut self, show: bool)
pub fn is_showing_suggestions(&self) -> bool
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn filtered_suggestions(&self) -> Vec<&str>
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn confirm_selection(&mut self)
pub fn cursor_position(&self) -> usize
```

**Widget:**
```rust
pub fn new() -> Self
pub fn placeholder(mut self, placeholder: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
pub fn selected_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn max_suggestions(mut self, max: usize) -> Self
```

#### Example Usage

```rust
let mut state = AutocompleteState::new();
state.set_options(vec![
    "alice".to_string(),
    "bob".to_string(),
    "charlie".to_string(),
]);

let autocomplete = Autocomplete::new()
    .placeholder("Type to search...")
    .max_suggestions(5);

StatefulWidget::render(autocomplete, area, buf, &mut state);

// Type to filter
state.insert_char('a');
state.insert_char('l');
// Suggestions now filtered to "alice"

// Navigate suggestions
state.select_next();
state.select_previous();

// Confirm selection
state.confirm_selection();
let assignee = state.selected_value();
```

#### Styling/Configuration
- **Input Field**: Cyan when focused
- **Title States**: "Assignee", "Assignee [typing]", "Assignee [selected]"
- **Placeholder**: Dark gray, italic
- **Cursor**: White background
- **Suggestions**: Up to 5 visible (configurable)
- **Suggestion Highlight**: Dark gray background
- **Empty State**: "No matches" in gray italic
- **Case-insensitive** filtering

---

## Form & Dialog Widgets

### Form

Multi-field form widget with validation.

#### Purpose
Create forms with multiple fields, validation, and focus management.

#### Use Cases
- Issue creation/editing
- Settings forms
- Multi-field data entry

#### State Structure

**FormState**
- `fields: Vec<FormField>` - All form fields
- `focused_index: usize` - Currently focused field index
- `cursor_position: usize` - Cursor position in focused field

**FormField:**
- `id: String` - Unique field identifier
- `label: String` - Field label
- `field_type: FieldType` - Text, TextArea, Selector, ReadOnly
- `value: String` - Current value
- `required: bool` - Validation requirement
- `error: Option<String>` - Validation error message
- `placeholder: Option<String>` - Placeholder text
- `options: Vec<String>` - Options for selector fields

**FieldType:**
```rust
enum FieldType {
    Text,        // Single-line input
    TextArea,    // Multi-line input
    Selector,    // Dropdown
    ReadOnly,    // Display only
}
```

#### Key Methods

**FormState:**
```rust
pub fn new(fields: Vec<FormField>) -> Self
pub fn focused_field(&self) -> Option<&FormField>
pub fn focused_field_mut(&mut self) -> Option<&mut FormField>
pub fn fields(&self) -> &[FormField]
pub fn fields_mut(&mut self) -> &mut [FormField]
pub fn get_field(&self, id: &str) -> Option<&FormField>
pub fn get_field_mut(&mut self, id: &str) -> Option<&mut FormField>
pub fn set_value(&mut self, id: &str, value: String)
pub fn get_value(&self, id: &str) -> Option<&str>
pub fn focus_next(&mut self)
pub fn focus_previous(&mut self)
pub fn focused_index(&self) -> usize
pub fn set_focused_index(&mut self, index: usize)
pub fn cursor_position(&self) -> usize
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn move_cursor_to_start(&mut self)
pub fn move_cursor_to_end(&mut self)
pub fn validate(&mut self) -> bool
pub fn has_errors(&self) -> bool
pub fn clear_errors(&mut self)
```

**FormField:**
```rust
pub fn text<S: Into<String>>(id: S, label: S) -> Self
pub fn text_area<S: Into<String>>(id: S, label: S) -> Self
pub fn selector<S: Into<String>>(id: S, label: S, options: Vec<String>) -> Self
pub fn read_only<S: Into<String>>(id: S, label: S, value: S) -> Self
pub fn required(mut self) -> Self
pub fn placeholder<S: Into<String>>(mut self, placeholder: S) -> Self
pub fn value<S: Into<String>>(mut self, value: S) -> Self
pub fn validate(&mut self) -> bool
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
pub fn error_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
```

#### Example Usage

```rust
// Create form fields
let fields = vec![
    FormField::text("title", "Title")
        .required()
        .placeholder("Enter issue title"),
    FormField::text_area("description", "Description")
        .placeholder("Describe the issue"),
    FormField::selector("priority", "Priority", vec![
        "P0".to_string(), "P1".to_string(), "P2".to_string()
    ]),
    FormField::read_only("id", "ID", "beads-123"),
];

let mut state = FormState::new(fields);

let form = Form::new()
    .title("Create Issue")
    .focused_style(Style::default().fg(Color::Cyan))
    .error_style(Style::default().fg(Color::Red));

StatefulWidget::render(form, area, buf, &mut state);

// Field navigation
state.focus_next();
state.focus_previous();

// Set values
state.set_value("title", "Fix login bug".to_string());

// Validate
if state.validate() {
    // Form is valid
    let title = state.get_value("title").unwrap();
} else {
    // Show errors
}
```

#### Styling/Configuration
- **Field Heights**: Text=3 lines, TextArea=5+ lines
- **Required Fields**: Asterisk (*) in label
- **Focused**: Cyan border, "[editing]" suffix
- **Error**: Red border, "⚠ {message}" below field
- **Placeholder**: Dark gray, italic
- **Cursor**: White background
- **Read-only**: No cursor, non-editable

---

### Dialog

Modal dialog for confirmations, alerts, and messages.

#### Purpose
Display modal dialogs for user confirmations, errors, warnings, and information.

#### Use Cases
- Confirm destructive actions
- Show error messages
- Display success notifications
- Get yes/no user input

#### State Structure

**DialogState**
- `selected_button: usize` - Currently selected button index

**DialogButton:**
- `label: String` - Button text
- `action: String` - Action identifier

**DialogType:**
```rust
enum DialogType {
    Info,      // Blue
    Warning,   // Yellow
    Error,     // Red
    Success,   // Green
    Confirm,   // Cyan
}
```

#### Key Methods

**DialogState:**
```rust
pub fn new() -> Self
pub fn selected_button(&self) -> usize
pub fn select_next(&mut self, button_count: usize)
pub fn select_previous(&mut self, button_count: usize)
pub fn reset(&mut self)
```

**Dialog:**
```rust
pub fn new(title: &'a str, message: &'a str) -> Self
pub fn confirm(title: &'a str, message: &'a str) -> Self
pub fn save_cancel(title: &'a str, message: &'a str) -> Self
pub fn error(title: &'a str, message: &'a str) -> Self
pub fn warning(title: &'a str, message: &'a str) -> Self
pub fn info(title: &'a str, message: &'a str) -> Self
pub fn success(title: &'a str, message: &'a str) -> Self
pub fn dialog_type(mut self, dialog_type: DialogType) -> Self
pub fn buttons(mut self, buttons: Vec<DialogButton>) -> Self
pub fn width(mut self, width: u16) -> Self
pub fn height(mut self, height: u16) -> Self
pub fn render_with_state(self, area: Rect, buf: &mut Buffer, state: &DialogState)
```

**DialogButton:**
```rust
pub fn new<S: Into<String>>(label: S, action: S) -> Self
```

#### Example Usage

```rust
// Confirmation dialog
let mut state = DialogState::new();
let dialog = Dialog::confirm(
    "Delete Issue",
    "Are you sure you want to delete this issue? This cannot be undone."
);
dialog.render_with_state(area, buf, &state);

// Error dialog
let error_dialog = Dialog::error(
    "Error",
    "Failed to save issue: Network timeout"
);

// Custom dialog
let custom_dialog = Dialog::new("Custom", "Choose an option")
    .dialog_type(DialogType::Info)
    .buttons(vec![
        DialogButton::new("Option 1", "opt1"),
        DialogButton::new("Option 2", "opt2"),
        DialogButton::new("Cancel", "cancel"),
    ])
    .width(60)
    .height(12);

// Navigate buttons
state.select_next(dialog.buttons.len());
state.select_previous(dialog.buttons.len());
```

#### Styling/Configuration
- **Default Size**: 50x10
- **Centered**: Auto-centered on screen
- **Type Colors**: Info=Blue, Warning=Yellow, Error=Red, Success=Green, Confirm=Cyan
- **Type Symbols**: Info=ℹ, Warning=⚠, Error=✖, Success=✓, Confirm=?
- **Selected Button**: Colored background, `[ {label} ]` format
- **Unselected Button**: Colored text, `  {label}  ` format
- **Button Width**: 12 characters
- **Destructive Actions**: Close, Delete (marked with [!])

---

### FilterSaveDialog

Dialog for saving filter presets with name, description, and hotkey.

#### Purpose
Allow users to save custom filters with metadata for quick access.

#### Use Cases
- Saving frequently used filter combinations
- Creating named filter presets
- Assigning hotkeys to filters

#### State Structure

**FilterSaveDialogState**
- `name: String` - Filter name
- `description: String` - Optional description
- `hotkey: Option<String>` - Optional hotkey (1-9, F1-F12)
- `focused_field: FilterSaveField` - Current field focus
- `name_cursor: usize` - Cursor in name field
- `description_cursor: usize` - Cursor in description field

**FilterSaveField:**
```rust
enum FilterSaveField {
    Name,
    Description,
    Hotkey,
}
```

#### Key Methods

```rust
pub fn new() -> Self
pub fn name(&self) -> &str
pub fn set_name<S: Into<String>>(&mut self, name: S)
pub fn description(&self) -> &str
pub fn set_description<S: Into<String>>(&mut self, description: S)
pub fn hotkey(&self) -> Option<&str>
pub fn set_hotkey<S: Into<String>>(&mut self, hotkey: Option<S>)
pub fn focused_field(&self) -> FilterSaveField
pub fn focus_next(&mut self)
pub fn focus_previous(&mut self)
pub fn insert_char(&mut self, c: char)
pub fn delete_char(&mut self)
pub fn move_cursor_left(&mut self)
pub fn move_cursor_right(&mut self)
pub fn validate(&self) -> Result<(), String>
pub fn clear(&mut self)
pub fn has_data(&self) -> bool
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn show_hotkey(mut self, show: bool) -> Self
pub fn width(mut self, width: u16) -> Self
pub fn height(mut self, height: u16) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn focused_style(mut self, style: Style) -> Self
pub fn render_with_state(self, area: Rect, buf: &mut Buffer, state: &FilterSaveDialogState)
```

#### Example Usage

```rust
let mut state = FilterSaveDialogState::new();

let dialog = FilterSaveDialog::new()
    .title("Save Filter")
    .show_hotkey(true)
    .width(60)
    .height(14);

dialog.render_with_state(area, buf, &state);

// Set values
state.set_name("My Custom Filter");
state.set_description("Shows all high priority bugs");
state.set_hotkey(Some("F1"));

// Field navigation
state.focus_next();
state.focus_previous();

// Validate
match state.validate() {
    Ok(_) => {
        // Save filter
        let name = state.name();
        let description = state.description();
        let hotkey = state.hotkey();
    }
    Err(msg) => {
        // Show error
    }
}
```

#### Styling/Configuration
- **Default Size**: 60x14
- **Centered**: Auto-centered on screen
- **Fields**: Name (required), Description (optional), Hotkey (optional)
- **Validation**:
  - Name: required, max 50 chars
  - Description: max 200 chars
- **Focus**: Cyan border on focused field
- **Placeholder**: Dark gray, italic
- **Cursor**: White background
- **Help Text**: "Tab Next | Ctrl+S Save | Esc Cancel"

---

## Filter Widgets

### FilterBuilder

Interactive filter builder for creating complex filter criteria.

#### Purpose
Build multi-criteria filters by selecting from available options.

#### Use Cases
- Advanced issue filtering
- Building custom filter presets
- Combining multiple filter conditions

#### State Structure

**FilterBuilderState**
- `criteria: FilterCriteria` - Current filter criteria
- `list_state: ListState` - List selection state
- `section: FilterSection` - Current filter section

**FilterCriteria:**
- `statuses: HashSet<IssueStatus>`
- `priorities: HashSet<Priority>`
- `types: HashSet<IssueType>`
- `labels: HashSet<String>`

**FilterSection:**
```rust
enum FilterSection {
    Status,
    Priority,
    Type,
    Labels,
}
```

#### Key Methods

**FilterBuilderState:**
```rust
pub fn new() -> Self
pub fn with_criteria(mut self, criteria: FilterCriteria) -> Self
pub fn criteria(&self) -> &FilterCriteria
pub fn criteria_mut(&mut self) -> &mut FilterCriteria
pub fn section(&self) -> FilterSection
pub fn set_section(&mut self, section: FilterSection)
pub fn next_section(&mut self)
pub fn previous_section(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn toggle_selected(&mut self)
pub fn clear_section(&mut self)
pub fn clear_all(&mut self)
```

**FilterCriteria:**
```rust
pub fn new() -> Self
pub fn is_active(&self) -> bool
pub fn add_status(&mut self, status: IssueStatus)
pub fn remove_status(&mut self, status: &IssueStatus)
pub fn toggle_status(&mut self, status: IssueStatus)
pub fn add_priority(&mut self, priority: Priority)
pub fn remove_priority(&mut self, priority: &Priority)
pub fn toggle_priority(&mut self, priority: Priority)
pub fn add_type(&mut self, issue_type: IssueType)
pub fn remove_type(&mut self, issue_type: &IssueType)
pub fn toggle_type(&mut self, issue_type: IssueType)
pub fn add_label(&mut self, label: impl Into<String>)
pub fn remove_label(&mut self, label: &str)
pub fn toggle_label(&mut self, label: impl Into<String>)
pub fn clear(&mut self)
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn show_section_tabs(mut self, show: bool) -> Self
```

#### Example Usage

```rust
let mut state = FilterBuilderState::new();

let builder = FilterBuilder::new()
    .title("Build Filter")
    .show_section_tabs(true);

StatefulWidget::render(builder, area, buf, &mut state);

// Navigate sections
state.next_section();     // Status -> Priority
state.previous_section(); // Priority -> Status

// Toggle selections
state.toggle_selected();  // Toggle current item

// Access criteria
let criteria = state.criteria();
let has_filters = criteria.is_active();

// Clear specific section
state.clear_section();

// Clear all
state.clear_all();
```

#### Styling/Configuration
- **Sections**: Status, Priority, Type, Labels
- **Section Navigation**: Tab-like interface
- **Selected Items**: ✓ prefix, green color
- **Active Section**: Cyan highlight
- **Item Colors**: Match issue list colors
- **Empty State**: "No items selected"

---

### FilterPanel

Display active filters in a compact panel.

#### Purpose
Show currently active filter criteria in a readable format.

#### Use Cases
- Showing active filters alongside issue list
- Filter summary display
- Quick reference for active filters

#### Structure

No state - uses `FilterCriteria` directly.

**Widget: `FilterPanel<'a>`**
- `criteria: &'a FilterCriteria` - Filter criteria to display
- `style: Style` - Panel style
- `active_style: Style` - Style for active filter labels
- `show_empty_message: bool` - Show message when no filters
- `result_count: Option<usize>` - Optional result count

#### Key Methods

```rust
pub fn new(criteria: &'a FilterCriteria) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn active_style(mut self, style: Style) -> Self
pub fn show_empty_message(mut self, show: bool) -> Self
pub fn result_count(mut self, count: Option<usize>) -> Self
```

#### Example Usage

```rust
let mut criteria = FilterCriteria::new();
criteria.add_status(IssueStatus::Open);
criteria.add_priority(Priority::P0);
criteria.add_label("bug");

let panel = FilterPanel::new(&criteria)
    .show_empty_message(true)
    .result_count(Some(15));

// Render as stateless widget
panel.render(area, buf);
```

#### Styling/Configuration
- **Active Indicator**: "Filters (Active)" in title
- **Empty Message**: "No active filters" in gray italic
- **Sections**: Status, Priority, Type, Labels, Assignee, Search, Conditions
- **Icons**: Color-coded for each filter type
- **Result Count**: "15 results" at bottom
- **Colors**: Match filter builder colors

---

## Display Widgets

### MarkdownViewer

Markdown content viewer with syntax highlighting and scrolling.

#### Purpose
Display and scroll through markdown-formatted content.

#### Use Cases
- Issue descriptions
- Help text
- Documentation
- Release notes

#### State Structure

**MarkdownViewerState**
- `content: String` - Markdown content
- `scroll_offset: usize` - First visible line
- `total_lines: usize` - Total rendered lines

**MarkdownElement:**
```rust
enum MarkdownElement {
    Heading(usize, String),    // Level, text
    Paragraph(String),
    ListItem(String),
    CodeBlock(String),
    InlineCode(String),
    HorizontalRule,
}
```

#### Key Methods

```rust
pub fn new(content: String) -> Self
pub fn content(&self) -> &str
pub fn set_content(&mut self, content: String)
pub fn scroll_offset(&self) -> usize
pub fn scroll_up(&mut self, amount: usize)
pub fn scroll_down(&mut self, amount: usize)
pub fn scroll_to_top(&mut self)
pub fn scroll_to_bottom(&mut self)
pub fn can_scroll_up(&self) -> bool
pub fn can_scroll_down(&self) -> bool
```

**Widget:**
```rust
pub fn new() -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn wrap(mut self, wrap: bool) -> Self
pub fn style(mut self, style: Style) -> Self
```

**Helper Functions:**
```rust
fn parse_markdown(content: &str) -> Vec<MarkdownElement>
fn parse_inline_formatting(text: &str) -> Vec<Span<'static>>
```

#### Example Usage

```rust
let markdown = r#"
# Issue Description

This is a **bold** and *italic* text.

## Details

- Item 1
- Item 2

```rust
fn example() {
    println!("Code block");
}
```

---
"#;

let mut state = MarkdownViewerState::new(markdown.to_string());

let viewer = MarkdownViewer::new()
    .wrap(true)
    .block(Block::default().borders(Borders::ALL).title("Description"));

StatefulWidget::render(viewer, area, buf, &mut state);

// Scrolling
state.scroll_down(5);
state.scroll_up(2);
state.scroll_to_bottom();
```

#### Styling/Configuration
- **Headings**:
  - H1: Cyan, bold, "# " prefix
  - H2: Blue, bold, "## " prefix
  - H3: Green, bold, "### " prefix
- **List Items**: Yellow bullet "•", indented
- **Code Blocks**: Green, italic, ``` delimiters in dark gray
- **Inline Code**: Cyan, italic
- **Bold**: Bold modifier
- **Italic**: Italic modifier
- **Horizontal Rules**: Dark gray "─" line
- **Word Wrap**: Optional (default: enabled)

---

### InlineMetadata

Compact inline display of issue metadata (labels, assignee, age).

#### Purpose
Show issue metadata in a compact, inline format.

#### Use Cases
- Issue list details
- Issue headers
- Quick reference display

#### Structure

No state - stateless widget.

**MetadataDisplayConfig:**
- `show_labels: bool` - Show labels
- `show_assignee: bool` - Show assignee
- `show_age: bool` - Show issue age
- `max_labels: Option<usize>` - Max labels to display
- `label_style: Style` - Label styling
- `assignee_style: Style` - Assignee styling
- `age_style: Style` - Age styling

**Widget: `InlineMetadata<'a>`**
- `issue: &'a Issue` - Issue to display
- `config: MetadataDisplayConfig` - Display configuration

#### Key Methods

**MetadataDisplayConfig:**
```rust
pub fn new() -> Self
pub fn show_labels(mut self, show: bool) -> Self
pub fn show_assignee(mut self, show: bool) -> Self
pub fn show_age(mut self, show: bool) -> Self
pub fn max_labels(mut self, max: usize) -> Self
pub fn label_style(mut self, style: Style) -> Self
pub fn assignee_style(mut self, style: Style) -> Self
pub fn age_style(mut self, style: Style) -> Self
```

**InlineMetadata:**
```rust
pub fn new(issue: &'a Issue) -> Self
pub fn config(mut self, config: MetadataDisplayConfig) -> Self
```

**Helper Functions:**
```rust
fn format_age(created: &DateTime<Utc>) -> String
fn format_labels(labels: &[String], max: Option<usize>) -> String
fn format_assignee(assignee: &Option<String>) -> String
fn build_metadata_spans(issue: &Issue, config: &MetadataDisplayConfig) -> Vec<Span>
```

#### Example Usage

```rust
let issue = /* ... */;

let config = MetadataDisplayConfig::new()
    .show_labels(true)
    .show_assignee(true)
    .show_age(true)
    .max_labels(3);

let metadata = InlineMetadata::new(&issue)
    .config(config);

// Render as stateless widget
metadata.render(area, buf);

// Custom format examples:
// format_age() -> "2h ago", "3d ago", "1w ago"
// format_labels() -> "bug, urgent, +2 more"
// format_assignee() -> "@john"
```

#### Styling/Configuration
- **Labels**: 🏷 icon, blue color, comma-separated
- **Assignee**: 👤 icon, cyan color, @username format
- **Age**: 🕒 icon, gray color, relative time
- **Overflow**: "+N more" for excess labels
- **Separator**: " | " between sections
- **Compact**: Single line display

---

### DatePicker

Date range picker with presets and custom ranges.

#### Purpose
Select date ranges for filtering and reporting.

#### Use Cases
- Date range filtering
- Report generation
- Time-based queries

#### State Structure

**DateRangePickerState**
- `preset: DateRangePreset` - Selected preset
- `range: DateRange` - Current date range
- `list_state: ListState` - Preset list state
- `is_custom_mode: bool` - Custom range mode

**DateRangePreset:**
```rust
enum DateRangePreset {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    Custom,
}
```

**DateRange:**
- `start: Option<NaiveDate>` - Start date
- `end: Option<NaiveDate>` - End date

#### Key Methods

**DateRangePickerState:**
```rust
pub fn new() -> Self
pub fn with_preset(mut self, preset: DateRangePreset) -> Self
pub fn preset(&self) -> DateRangePreset
pub fn date_range(&self) -> &DateRange
pub fn set_date_range(&mut self, start: Option<NaiveDate>, end: Option<NaiveDate>)
pub fn clear(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn apply_selected(&mut self)
pub fn is_custom_mode(&self) -> bool
pub fn set_custom_mode(&mut self, enabled: bool)
```

**DateRange:**
```rust
pub fn new() -> Self
pub fn from_dates(start: NaiveDate, end: NaiveDate) -> Self
pub fn is_empty(&self) -> bool
pub fn contains(&self, date: &NaiveDate) -> bool
pub fn format(&self) -> String
```

**DateRangePreset:**
```rust
pub fn name(&self) -> &str
pub fn date_range(&self) -> Option<(NaiveDate, NaiveDate)>
pub fn all() -> Vec<DateRangePreset>
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn selected_style(mut self, style: Style) -> Self
```

#### Example Usage

```rust
let mut state = DateRangePickerState::new()
    .with_preset(DateRangePreset::Last7Days);

let picker = DateRangePicker::new()
    .title("Date Range")
    .selected_style(Style::default().bg(Color::DarkGray));

StatefulWidget::render(picker, area, buf, &mut state);

// Navigate presets
state.select_next();
state.select_previous();

// Apply preset
state.apply_selected();

// Custom range
use chrono::NaiveDate;
let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
state.set_date_range(Some(start), Some(end));

// Get range
let range = state.date_range();
if !range.is_empty() {
    let formatted = range.format();
}
```

#### Styling/Configuration
- **Presets**: List with date preview
- **Format Examples**:
  - Same day: "2024-01-15"
  - Range: "2024-01-01 to 2024-01-31"
  - Partial: "From 2024-01-01" or "Until 2024-01-31"
  - Empty: "Any date"
- **Current Range**: Green text when selected
- **Highlight**: Dark gray background
- **Two Panels**: Preset list and current range display

---

## Action Widgets

### BulkActionMenu

Menu for selecting bulk actions on multiple issues.

#### Purpose
Provide a menu of available bulk operations for selected issues.

#### Use Cases
- Closing multiple issues
- Changing status in bulk
- Adding labels to multiple issues
- Bulk deletions

#### State Structure

**BulkActionMenuState**
- `actions: Vec<BulkAction>` - Available actions
- `list_state: ListState` - Menu selection state
- `selected_count: usize` - Number of selected issues
- `confirmed_action: Option<BulkAction>` - Confirmed action

**BulkAction:**
```rust
enum BulkAction {
    Close,
    Reopen,
    SetInProgress,
    SetBlocked,
    SetPriority,
    AddLabels,
    RemoveLabels,
    SetAssignee,
    ClearAssignee,
    Delete,
    Export,
    Cancel,
}
```

#### Key Methods

**BulkActionMenuState:**
```rust
pub fn new(selected_count: usize) -> Self
pub fn with_actions(actions: Vec<BulkAction>, selected_count: usize) -> Self
pub fn selected_count(&self) -> usize
pub fn set_selected_count(&mut self, count: usize)
pub fn actions(&self) -> &[BulkAction]
pub fn highlighted_action(&self) -> Option<BulkAction>
pub fn confirmed_action(&self) -> Option<BulkAction>
pub fn clear_confirmed(&mut self)
pub fn select_next(&mut self)
pub fn select_previous(&mut self)
pub fn confirm_selection(&mut self) -> Option<BulkAction>
pub fn reset(&mut self)
```

**BulkAction:**
```rust
pub fn display_name(&self) -> &str
pub fn icon(&self) -> &str
pub fn color(&self) -> Color
pub fn is_destructive(&self) -> bool
pub fn requires_input(&self) -> bool
pub fn all() -> Vec<BulkAction>
```

**Widget:**
```rust
pub fn new() -> Self
pub fn title(mut self, title: &'a str) -> Self
pub fn style(mut self, style: Style) -> Self
pub fn selected_style(mut self, style: Style) -> Self
pub fn block(mut self, block: Block<'a>) -> Self
pub fn show_icons(mut self, show: bool) -> Self
pub fn show_count(mut self, show: bool) -> Self
```

#### Example Usage

```rust
let mut state = BulkActionMenuState::new(5);  // 5 issues selected

let menu = BulkActionMenu::new()
    .title("Bulk Actions")
    .show_icons(true)
    .show_count(true);

StatefulWidget::render(menu, area, buf, &mut state);

// Navigate
state.select_next();
state.select_previous();

// Confirm action
if let Some(action) = state.confirm_selection() {
    match action {
        BulkAction::Close => { /* close selected issues */ }
        BulkAction::Delete => { /* show confirmation */ }
        _ => {}
    }
}

// Custom action list
let custom_actions = vec![
    BulkAction::Close,
    BulkAction::SetPriority,
    BulkAction::Cancel,
];
let state = BulkActionMenuState::with_actions(custom_actions, 3);
```

#### Styling/Configuration
- **Title**: "Bulk Actions (5 selected)"
- **Icons**: ✓ ↻ ▶ ⊘ ! + - @ ∅ ✗ ↓ ←
- **Colors**:
  - Close: Green
  - Reopen: Cyan
  - SetInProgress: Yellow
  - SetBlocked: Red
  - Delete: Red
  - Export: Green
  - Others: Type-specific
- **Destructive Actions**: Red text, bold, [!] suffix
- **Requires Input**: "..." suffix in display name
- **Highlight**: Dark gray background

---

## Widget Patterns

### Common Patterns

#### StatefulWidget Pattern

Most widgets follow this pattern:

```rust
// State struct - holds mutable state
pub struct MyWidgetState {
    // ... state fields
}

// Widget struct - holds configuration
pub struct MyWidget<'a> {
    // ... config fields (references, not owned)
}

// Implementation
impl<'a> StatefulWidget for MyWidget<'a> {
    type State = MyWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // ... rendering logic
    }
}
```

#### Builder Pattern

All widgets use method chaining:

```rust
let widget = MyWidget::new()
    .title("Title")
    .style(Style::default().fg(Color::Cyan))
    .some_option(true);
```

#### Focus Management

Input widgets follow this pattern:

```rust
// State tracks focus
state.set_focused(true);

// Widget styling changes based on focus
let style = if state.is_focused() {
    self.focused_style
} else {
    self.style
};
```

#### Color Coding

Consistent colors across widgets:

- **Priority**: P0=Red, P1=LightRed, P2=Yellow, P3=Blue, P4=Gray
- **Status**: Open=Green, InProgress=Cyan, Blocked=Red, Closed=Gray
- **Type**: Bug=Red, Feature=Green, Task=Blue, Epic=Magenta, Chore=Yellow
- **Focus**: Cyan
- **Error**: Red
- **Success**: Green
- **Info**: Blue
- **Warning**: Yellow

#### List Navigation

Standard navigation pattern:

```rust
// Wrapping navigation
pub fn select_next(&mut self) {
    let count = self.items.len();
    let i = match self.selected() {
        Some(i) if i >= count - 1 => 0,  // Wrap to start
        Some(i) => i + 1,
        None => 0,
    };
    self.select(Some(i));
}
```

### Error Handling

Validation pattern:

```rust
pub fn validate(&mut self) -> Result<(), String> {
    if self.required_field.is_empty() {
        return Err("Field is required".to_string());
    }
    Ok(())
}
```

### Performance Considerations

- **Lazy Rendering**: Widgets only render visible content
- **Scroll Optimization**: Only render lines within viewport
- **State Separation**: Widget config is cheap to clone, state is not
- **Immutable Widgets**: Widgets consumed on render (moved, not borrowed)

---

## Conclusion

This widget library provides a comprehensive set of components for building terminal user interfaces in Rust. All widgets follow consistent patterns for state management, styling, and user interaction, making them easy to learn and compose together.

For examples of widget usage in context, see the `src/ui/screens/` directory, which contains complete screen implementations using these widgets.
