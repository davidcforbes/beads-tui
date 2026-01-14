//! Undo/redo stack implementation

use super::{Command, CommandResult};
use std::collections::VecDeque;

/// Undo/redo stack with configurable size limits
///
/// Manages a stack of executed commands with support for undo and redo operations.
/// When the stack reaches its size limit, the oldest commands are automatically removed.
#[derive(Debug)]
pub struct UndoStack {
    /// Stack of commands that can be undone (most recent at the end)
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Stack of commands that can be redone (most recent at the end)
    redo_stack: VecDeque<Box<dyn Command>>,
    /// Maximum number of commands to keep in the undo stack
    max_size: usize,
}

impl UndoStack {
    /// Create a new undo stack with the default size limit (50 commands)
    pub fn new() -> Self {
        Self::with_capacity(50)
    }

    /// Create a new undo stack with a custom size limit
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of commands to keep in the undo stack
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_size),
            redo_stack: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a new command onto the undo stack
    ///
    /// This should be called after a command has been executed successfully.
    /// Pushing a new command clears the redo stack (since we're creating a new timeline).
    /// If the stack is at capacity, the oldest command is removed.
    pub fn push(&mut self, command: Box<dyn Command>) {
        // Clear redo stack when pushing a new command
        self.redo_stack.clear();

        // Add command to undo stack
        self.undo_stack.push_back(command);

        // Enforce size limit
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.pop_front();
        }
    }

    /// Undo the most recent command
    ///
    /// Returns an error if there are no commands to undo or if the undo operation fails.
    pub fn undo(&mut self) -> CommandResult<()> {
        // Pop the most recent command from the undo stack
        let mut command = self
            .undo_stack
            .pop_back()
            .ok_or_else(|| super::CommandError::InvalidState("Nothing to undo".to_string()))?;

        // Undo the command
        command.undo()?;

        // Move command to redo stack
        self.redo_stack.push_back(command);

        Ok(())
    }

    /// Redo the most recently undone command
    ///
    /// Returns an error if there are no commands to redo or if the redo operation fails.
    pub fn redo(&mut self) -> CommandResult<()> {
        // Pop the most recent command from the redo stack
        let mut command = self
            .redo_stack
            .pop_back()
            .ok_or_else(|| super::CommandError::InvalidState("Nothing to redo".to_string()))?;

        // Re-execute the command
        command.execute()?;

        // Move command back to undo stack
        self.undo_stack.push_back(command);

        Ok(())
    }

    /// Check if there are any commands that can be undone
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if there are any commands that can be redone
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of commands in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get the maximum capacity of the undo stack
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Clear both undo and redo stacks
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the description of the most recent undoable command
    pub fn peek_undo(&self) -> Option<&str> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Get the description of the most recent redoable command
    pub fn peek_redo(&self) -> Option<&str> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Get the total memory size of all commands in bytes
    pub fn total_size_bytes(&self) -> usize {
        let undo_size: usize = self.undo_stack.iter().map(|cmd| cmd.size_bytes()).sum();
        let redo_size: usize = self.redo_stack.iter().map(|cmd| cmd.size_bytes()).sum();
        undo_size + redo_size
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::undo::{CommandError, CommandMetadata};

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
    fn test_stack_creation() {
        let stack = UndoStack::new();
        assert_eq!(stack.capacity(), 50);
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 0);
    }

    #[test]
    fn test_stack_with_custom_capacity() {
        let stack = UndoStack::with_capacity(10);
        assert_eq!(stack.capacity(), 10);
    }

    #[test]
    fn test_push_command() {
        let mut stack = UndoStack::new();
        let cmd = Box::new(TestCommand::new("Test", 0));

        stack.push(cmd);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 1);
    }

    #[test]
    fn test_undo_command() {
        let mut stack = UndoStack::new();
        let mut cmd = TestCommand::new("Test", 0);
        cmd.execute().unwrap();

        stack.push(Box::new(cmd));
        assert_eq!(stack.undo_count(), 1);

        stack.undo().unwrap();
        assert!(!stack.can_undo());
        assert!(stack.can_redo());
        assert_eq!(stack.redo_count(), 1);
    }

    #[test]
    fn test_redo_command() {
        let mut stack = UndoStack::new();
        let mut cmd = TestCommand::new("Test", 0);
        cmd.execute().unwrap();

        stack.push(Box::new(cmd));
        stack.undo().unwrap();
        stack.redo().unwrap();

        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 1);
    }

    #[test]
    fn test_undo_empty_stack() {
        let mut stack = UndoStack::new();
        let result = stack.undo();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    #[test]
    fn test_redo_empty_stack() {
        let mut stack = UndoStack::new();
        let result = stack.redo();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut stack = UndoStack::new();
        let mut cmd1 = TestCommand::new("Test 1", 0);
        let mut cmd2 = TestCommand::new("Test 2", 0);
        cmd1.execute().unwrap();
        cmd2.execute().unwrap();

        stack.push(Box::new(cmd1));
        stack.undo().unwrap();
        assert!(stack.can_redo());

        // Pushing a new command should clear the redo stack
        stack.push(Box::new(cmd2));
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_size_limit() {
        let mut stack = UndoStack::with_capacity(3);

        for i in 0..5 {
            let mut cmd = TestCommand::new(&format!("Test {}", i), i);
            cmd.execute().unwrap();
            stack.push(Box::new(cmd));
        }

        // Stack should only contain the last 3 commands
        assert_eq!(stack.undo_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut stack = UndoStack::new();
        let mut cmd = TestCommand::new("Test", 0);
        cmd.execute().unwrap();

        stack.push(Box::new(cmd));
        stack.undo().unwrap();

        assert!(stack.can_redo());
        stack.clear();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_peek_undo() {
        let mut stack = UndoStack::new();
        let mut cmd = TestCommand::new("Test command", 0);
        cmd.execute().unwrap();

        stack.push(Box::new(cmd));
        assert_eq!(stack.peek_undo(), Some("Test command"));
    }

    #[test]
    fn test_peek_redo() {
        let mut stack = UndoStack::new();
        let mut cmd = TestCommand::new("Test command", 0);
        cmd.execute().unwrap();

        stack.push(Box::new(cmd));
        stack.undo().unwrap();

        assert_eq!(stack.peek_redo(), Some("Test command"));
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut stack = UndoStack::new();

        for i in 0..5 {
            let mut cmd = TestCommand::new(&format!("Command {}", i), i);
            cmd.execute().unwrap();
            stack.push(Box::new(cmd));
        }

        assert_eq!(stack.undo_count(), 5);

        // Undo 3 commands
        stack.undo().unwrap();
        stack.undo().unwrap();
        stack.undo().unwrap();

        assert_eq!(stack.undo_count(), 2);
        assert_eq!(stack.redo_count(), 3);

        // Redo 2 commands
        stack.redo().unwrap();
        stack.redo().unwrap();

        assert_eq!(stack.undo_count(), 4);
        assert_eq!(stack.redo_count(), 1);
    }
}
