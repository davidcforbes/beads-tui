//! Reusable UI widgets

pub mod dialog;
pub mod field_editor;
pub mod form;
pub mod issue_list;
pub mod progress;
pub mod selector;
pub mod status_bar;
pub mod tab_bar;

pub use dialog::{Dialog, DialogButton, DialogState, DialogType};
pub use field_editor::{EditorMode, FieldEditor, FieldEditorState};
pub use form::{FieldType, Form, FormField, FormState};
pub use issue_list::{IssueList, IssueListState, SortColumn, SortDirection};
pub use progress::{LoadingIndicator, ProgressBar, Spinner};
pub use selector::{PrioritySelector, SelectorState, StatusSelector, TypeSelector};
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;

// Future widgets:
// - Issue list widget
// - Filter builder widget
// - Dependency tree widget
// - Command palette widget
