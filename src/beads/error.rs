/// Error types for beads-rs library
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BeadsError>;

#[derive(Error, Debug)]
pub enum BeadsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("Timeout error: operation took longer than {0}ms")]
    Timeout(u64),

    #[error("Cancelled: {0}")]
    Cancelled(String),

    #[error("Invalid issue ID: {0}")]
    InvalidIssueId(String),

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Beads CLI not found in PATH")]
    BeadsNotFound,
}
