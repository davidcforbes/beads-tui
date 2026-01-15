/// Application-level models and state management
///
/// This module contains UI state models that are distinct from
/// the beads data models in the beads module.
pub mod app_state;
pub mod filter;
pub mod gantt_schedule;
pub mod issue_cache;
pub mod kanban_config;
pub mod labels;
pub mod navigation;
pub mod pert_layout;
pub mod table_config;
pub mod undo_history;

pub use app_state::*;
pub use filter::*;
pub use issue_cache::{IssueCache, IssueCacheStats};
pub use labels::*;
pub use navigation::*;
pub use undo_history::{UndoEntry, UndoHistory};

// Re-export table and kanban configs with their full types to avoid ambiguity
pub use kanban_config::KanbanConfig;
pub use table_config::TableConfig;

// Re-export gantt schedule types
pub use gantt_schedule::{IssueSchedule, ScheduleData, TimeEstimate, TimelineConfig, ZoomLevel};

// Re-export PERT layout types
pub use pert_layout::{CycleDetection, PertEdge, PertGraph, PertNode};
