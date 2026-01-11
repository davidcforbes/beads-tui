//! Reusable UI widgets

pub mod autocomplete;
pub mod checkbox_list;
pub mod date_picker;
pub mod dialog;
pub mod field_editor;
pub mod filter_builder;
pub mod filter_panel;
pub mod filter_save_dialog;
pub mod form;
pub mod issue_list;
pub mod label_picker;
pub mod progress;
pub mod search_input;
pub mod selector;
pub mod status_bar;
pub mod tab_bar;
pub mod text_editor;

pub use autocomplete::{Autocomplete, AutocompleteState};
pub use checkbox_list::{CheckboxList, CheckboxListState};
pub use date_picker::{DateRange, DateRangePicker, DateRangePickerState, DateRangePreset};
pub use dialog::{Dialog, DialogButton, DialogState, DialogType};
pub use field_editor::{EditorMode, FieldEditor, FieldEditorState};
pub use filter_builder::{FilterBuilder, FilterBuilderState, FilterSection};
pub use filter_panel::{FilterCriteria, FilterPanel};
pub use filter_save_dialog::{FilterSaveDialog, FilterSaveDialogState, FilterSaveField};
pub use form::{FieldType, Form, FormField, FormState};
pub use issue_list::{IssueList, IssueListState, SortColumn, SortDirection};
pub use label_picker::{LabelPicker, LabelPickerState};
pub use progress::{LoadingIndicator, ProgressBar, Spinner};
pub use search_input::{SearchInput, SearchInputState};
pub use selector::{PrioritySelector, SelectorState, StatusSelector, TypeSelector};
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;
pub use text_editor::{TextEditor, TextEditorState};

// Future widgets:
// - Issue list widget
// - Filter builder widget
// - Dependency tree widget
// - Command palette widget
