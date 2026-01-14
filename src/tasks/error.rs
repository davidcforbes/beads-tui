//! Error types for background tasks

use crate::beads::error::BeadsError;
use std::fmt;

/// Task error type
#[derive(Debug, Clone)]
pub enum TaskError {
    /// Task execution failed
    ExecutionFailed(String),
    /// Task was cancelled
    Cancelled,
    /// Invalid task state
    InvalidState(String),
    /// Beads client error
    ClientError(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionFailed(msg) => write!(f, "Task execution failed: {}", msg),
            Self::Cancelled => write!(f, "Task was cancelled"),
            Self::InvalidState(msg) => write!(f, "Invalid task state: {}", msg),
            Self::ClientError(msg) => write!(f, "Beads client error: {}", msg),
        }
    }
}

impl std::error::Error for TaskError {}

impl From<anyhow::Error> for TaskError {
    fn from(err: anyhow::Error) -> Self {
        Self::ClientError(err.to_string())
    }
}

impl From<BeadsError> for TaskError {
    fn from(err: BeadsError) -> Self {
        Self::ClientError(err.to_string())
    }
}
