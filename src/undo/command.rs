//! Command pattern implementation for undo/redo system

use chrono::{DateTime, Utc};
use std::fmt;

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Errors that can occur during command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    /// Command execution failed
    ExecutionFailed(String),
    /// Command undo failed
    UndoFailed(String),
    /// Command is not reversible
    NotReversible,
    /// Invalid command state
    InvalidState(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::UndoFailed(msg) => write!(f, "Undo failed: {}", msg),
            CommandError::NotReversible => write!(f, "Command is not reversible"),
            CommandError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Metadata about a command
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Human-readable description of the command
    pub description: String,
    /// When the command was created
    pub timestamp: DateTime<Utc>,
    /// Whether this command can be undone
    pub reversible: bool,
    /// Estimated memory size in bytes
    pub size_bytes: usize,
}

impl CommandMetadata {
    /// Create new command metadata
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            timestamp: Utc::now(),
            reversible: true,
            size_bytes: 0,
        }
    }

    /// Create metadata for a non-reversible command
    pub fn non_reversible(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            timestamp: Utc::now(),
            reversible: false,
            size_bytes: 0,
        }
    }

    /// Set the estimated size of this command
    pub fn with_size(mut self, size_bytes: usize) -> Self {
        self.size_bytes = size_bytes;
        self
    }
}

/// Core trait for all undoable commands
///
/// Implements the Command pattern for undo/redo functionality.
/// Each command must be able to execute itself and undo its effects.
pub trait Command: Send + Sync + fmt::Debug {
    /// Execute the command
    ///
    /// Performs the operation and returns the result.
    /// Implementation should be idempotent when possible.
    fn execute(&mut self) -> CommandResult<()>;

    /// Undo the command
    ///
    /// Reverses the effects of execute().
    /// Should restore the state to exactly what it was before execute().
    fn undo(&mut self) -> CommandResult<()>;

    /// Get command metadata
    fn metadata(&self) -> &CommandMetadata;

    /// Check if this command can be undone
    fn can_undo(&self) -> bool {
        self.metadata().reversible
    }

    /// Get a human-readable description of this command
    fn description(&self) -> &str {
        &self.metadata().description
    }

    /// Get the timestamp when this command was created
    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata().timestamp
    }

    /// Get estimated memory size of this command in bytes
    fn size_bytes(&self) -> usize {
        self.metadata().size_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCommand {
        metadata: CommandMetadata,
        value: i32,
        executed: bool,
    }

    impl TestCommand {
        fn new(description: &str, value: i32) -> Self {
            Self {
                metadata: CommandMetadata::new(description),
                value,
                executed: false,
            }
        }
    }

    impl Command for TestCommand {
        fn execute(&mut self) -> CommandResult<()> {
            if self.executed {
                return Err(CommandError::InvalidState("Already executed".to_string()));
            }
            self.value += 10;
            self.executed = true;
            Ok(())
        }

        fn undo(&mut self) -> CommandResult<()> {
            if !self.executed {
                return Err(CommandError::InvalidState("Not executed".to_string()));
            }
            self.value -= 10;
            self.executed = false;
            Ok(())
        }

        fn metadata(&self) -> &CommandMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_command_execute() {
        let mut cmd = TestCommand::new("Test command", 5);
        assert_eq!(cmd.value, 5);
        assert!(!cmd.executed);

        cmd.execute().unwrap();
        assert_eq!(cmd.value, 15);
        assert!(cmd.executed);
    }

    #[test]
    fn test_command_undo() {
        let mut cmd = TestCommand::new("Test command", 5);
        cmd.execute().unwrap();
        assert_eq!(cmd.value, 15);

        cmd.undo().unwrap();
        assert_eq!(cmd.value, 5);
        assert!(!cmd.executed);
    }

    #[test]
    fn test_command_metadata() {
        let cmd = TestCommand::new("Test description", 0);
        assert_eq!(cmd.description(), "Test description");
        assert!(cmd.can_undo());
    }

    #[test]
    fn test_non_reversible_command() {
        let metadata = CommandMetadata::non_reversible("Non-reversible");
        assert!(!metadata.reversible);
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::ExecutionFailed("test error".to_string());
        assert_eq!(err.to_string(), "Execution failed: test error");

        let err = CommandError::NotReversible;
        assert_eq!(err.to_string(), "Command is not reversible");
    }

    #[test]
    fn test_double_execute_fails() {
        let mut cmd = TestCommand::new("Test", 0);
        cmd.execute().unwrap();
        let result = cmd.execute();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    #[test]
    fn test_undo_without_execute_fails() {
        let mut cmd = TestCommand::new("Test", 0);
        let result = cmd.undo();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }
}
