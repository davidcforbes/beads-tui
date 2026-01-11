//! Reusable UI widgets

pub mod progress;
pub mod status_bar;
pub mod tab_bar;

pub use progress::{LoadingIndicator, ProgressBar, Spinner};
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;

// Future widgets:
// - Issue list widget
// - Filter builder widget
// - Dependency tree widget
// - Command palette widget
