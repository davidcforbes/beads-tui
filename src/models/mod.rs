/// Application-level models and state management
///
/// This module contains UI state models that are distinct from
/// the beads data models in the beads module.
pub mod app_state;
pub mod filter;
pub mod kanban_config;
pub mod navigation;
pub mod perf;
pub mod table_config;

pub use app_state::*;
pub use filter::*;
pub use navigation::*;
pub use perf::PerfStats;

// Re-export table and kanban configs with their full types to avoid ambiguity
pub use table_config::TableConfig;
pub use kanban_config::KanbanConfig;
