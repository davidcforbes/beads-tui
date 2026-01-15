//! Reusable UI widgets

pub mod autocomplete;
pub mod bulk_action_menu;
pub mod checkbox_list;
pub mod column_manager;
pub mod date_picker;
pub mod dependency_dialog;
pub mod dialog;
pub mod field_editor;
pub mod filter_bar;
pub mod filter_builder;
pub mod filter_panel;
pub mod filter_quick_select;
pub mod filter_save_dialog;
pub mod form;
pub mod gantt_chart;
pub mod help_overlay;
pub mod inline_metadata;
pub mod issue_history;
pub mod issue_list;
pub mod kanban_card;
pub mod label_picker;
pub mod markdown_viewer;
pub mod notification_history;
pub mod pert_chart;
pub mod progress;
pub mod search_input;
pub mod selector;
pub mod skeleton;
pub mod status_bar;
pub mod tab_bar;
pub mod text_editor;
pub mod toast;
pub mod tree;
pub mod undo_history;

pub use autocomplete::{Autocomplete, AutocompleteState};
pub use bulk_action_menu::{BulkAction, BulkActionMenu, BulkActionMenuState};
pub use checkbox_list::{CheckboxList, CheckboxListState};
pub use column_manager::{ColumnManager, ColumnManagerAction, ColumnManagerState};
pub use date_picker::{DateRange, DateRangePicker, DateRangePickerState, DateRangePreset};
pub use dependency_dialog::{
    DependencyDialog, DependencyDialogFocus, DependencyDialogState, DependencyType,
};
pub use dialog::{Dialog, DialogButton, DialogState, DialogType};
pub use field_editor::{EditorMode, FieldEditor, FieldEditorState};
pub use filter_bar::{
    ActiveDropdownMut, FilterBar, FilterBarState, FilterDropdown, FilterDropdownType,
    MultiSelectDropdownState,
};
pub use filter_builder::{FilterBuilder, FilterBuilderState, FilterSection};
pub use filter_panel::{FilterCriteria, FilterPanel};
pub use filter_quick_select::{FilterQuickSelectMenu, FilterQuickSelectState};
pub use filter_save_dialog::{FilterSaveDialog, FilterSaveDialogState, FilterSaveField};
pub use form::{FieldType, Form, FormField, FormState, ValidationRule};
pub use gantt_chart::{GanttChart, GanttChartConfig, GroupingMode};
pub use help_overlay::{HelpOverlay, HelpOverlayPosition, KeyBinding};
pub use inline_metadata::{
    build_metadata_spans, format_age, format_assignee, format_labels, MetadataDisplayConfig,
};
pub use issue_history::{IssueHistoryPanel, IssueHistoryState};
pub use issue_list::{ColumnFilters, IssueList, IssueListState, SortColumn, SortDirection};
pub use kanban_card::{render_kanban_card, CardMode, KanbanCardConfig};
pub use label_picker::{LabelPicker, LabelPickerState};
pub use markdown_viewer::{MarkdownViewer, MarkdownViewerState};
pub use notification_history::{NotificationHistoryPanel, NotificationHistoryState};
pub use pert_chart::{Direction, PertChart, PertChartConfig};
pub use progress::{LoadingIndicator, ProgressBar, Spinner};
pub use search_input::{SearchInput, SearchInputState};
pub use selector::{PrioritySelector, SelectorState, StatusSelector, TypeSelector};
pub use skeleton::{SkeletonAnimation, SkeletonList, SkeletonTable, SkeletonText, SkeletonTree};
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;
pub use text_editor::{TextEditor, TextEditorState};
pub use toast::{Toast, ToastConfig, ToastPosition, ToastStack};
pub use tree::{Tree, TreeNode, TreeState};
pub use undo_history::{HistoryEntry, UndoHistoryView};

// Future widgets:
// - Issue list widget
// - Filter builder widget
// - Dependency tree widget
// - Command palette widget
