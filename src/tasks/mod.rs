//! Background task management system
//!
//! This module provides infrastructure for running long-running operations in the background
//! without blocking the UI. Tasks can report progress, be cancelled, and notify the main thread
//! when they complete.
//!
//! # Architecture
//!
//! - **TaskHandle**: Handle to a running or completed task
//! - **TaskManager**: Manages spawning and tracking tasks
//! - **ProgressTracker**: Reports progress for long-running tasks
//! - **TaskError**: Error type for task failures
//!
//! # Usage
//!
//! ```no_run
//! # use beads_tui::tasks::*;
//! # use beads_tui::beads::client::BeadsClient;
//! let mut manager = TaskManager::new();
//! let client = BeadsClient::new();
//!
//! // Spawn a background task
//! let handle = manager.spawn_task(
//!     "Compacting database",
//!     client,
//!     |client| async move {
//!         client.compact_database().await?;
//!         Ok(TaskOutput::DatabaseCompacted)
//!     }
//! );
//!
//! // Check status later
//! if handle.is_complete() {
//!     println!("Task finished!");
//! }
//! ```

pub mod error;
pub mod handle;
pub mod manager;
pub mod progress;

pub use error::TaskError;
pub use handle::{TaskHandle, TaskId, TaskOutput, TaskResult, TaskStatus};
pub use manager::{TaskManager, TaskMessage};
pub use progress::ProgressTracker;
