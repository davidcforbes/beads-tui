//! Reusable UI widgets

pub mod issue_list;
pub mod progress;
pub mod status_bar;
pub mod tab_bar;

pub use issue_list::{IssueList, IssueListState, SortColumn, SortDirection};
pub use progress::{LoadingIndicator, ProgressBar, Spinner};
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;

// Future widgets:
// - Issue list widget
// - Filter builder widget
// - Dependency tree widget
// - Command palette widget
