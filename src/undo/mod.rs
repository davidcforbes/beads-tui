//! Undo/redo system for beads-tui
//!
//! Provides comprehensive undo/redo functionality using the Command pattern.
//! Supports persistent undo history and memory-efficient command storage.

mod command;

pub use command::{Command, CommandError, CommandMetadata, CommandResult};
