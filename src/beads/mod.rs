/// Beads-rs: Rust wrapper library for beads CLI commands
///
/// This module provides a type-safe interface for executing beads commands
/// and parsing their JSON output.
pub mod client;
pub mod error;
pub mod models;
pub mod parser;

#[cfg(test)]
pub mod mock;

pub use client::BeadsClient;
pub use error::{BeadsError, Result};
pub use models::*;

#[cfg(test)]
pub use mock::MockBeadsBackend;
