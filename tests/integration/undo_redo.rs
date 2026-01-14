//! Integration tests for undo/redo functionality
//!
//! Tests the undo/redo system integration and edge cases.
//! Individual command and stack tests are in the unit tests.

use beads_tui::undo::{Command, CommandError, CommandMetadata, CommandResult, UndoStack};
use std::fmt;

/// Simple test command that tracks execute/undo state
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

    fn get_value(&self) -> i32 {
        self.value
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
fn test_undo_stack_integration() {
    let mut stack = UndoStack::new();

    // Create and execute commands
    let mut cmd1 = TestCommand::new("Command 1", 0);
    let mut cmd2 = TestCommand::new("Command 2", 100);

    cmd1.execute().expect("Failed to execute cmd1");
    cmd2.execute().expect("Failed to execute cmd2");

    assert_eq!(cmd1.get_value(), 10);
    assert_eq!(cmd2.get_value(), 110);

    // Push to stack
    stack.push(Box::new(cmd1));
    stack.push(Box::new(cmd2));

    assert_eq!(stack.undo_count(), 2);
    assert_eq!(stack.redo_count(), 0);

    // Undo cmd2
    stack.undo().expect("Failed to undo");
    assert_eq!(stack.undo_count(), 1);
    assert_eq!(stack.redo_count(), 1);

    // Undo cmd1
    stack.undo().expect("Failed to undo");
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 2);

    // Redo cmd1
    stack.redo().expect("Failed to redo");
    assert_eq!(stack.undo_count(), 1);
    assert_eq!(stack.redo_count(), 1);

    // Redo cmd2
    stack.redo().expect("Failed to redo");
    assert_eq!(stack.undo_count(), 2);
    assert_eq!(stack.redo_count(), 0);
}

#[test]
fn test_undo_empty_stack() {
    let mut stack = UndoStack::new();

    // Attempting to undo on empty stack should fail
    let result = stack.undo();
    assert!(result.is_err(), "Undo should fail on empty stack");
    assert!(result.unwrap_err().to_string().contains("Nothing to undo"));
}

#[test]
fn test_redo_empty_stack() {
    let mut stack = UndoStack::new();

    // Attempting to redo on empty stack should fail
    let result = stack.redo();
    assert!(result.is_err(), "Redo should fail on empty stack");
    assert!(result.unwrap_err().to_string().contains("Nothing to redo"));
}

#[test]
fn test_redo_stack_cleared_on_new_command() {
    let mut stack = UndoStack::new();

    // Execute and push cmd1
    let mut cmd1 = TestCommand::new("Command 1", 0);
    cmd1.execute().expect("Failed to execute");
    stack.push(Box::new(cmd1));

    // Undo
    stack.undo().expect("Failed to undo");
    assert!(stack.can_redo(), "Should be able to redo");

    // Push a new command (this should clear the redo stack)
    let mut cmd2 = TestCommand::new("Command 2", 100);
    cmd2.execute().expect("Failed to execute");
    stack.push(Box::new(cmd2));

    // Redo should now fail (stack was cleared)
    assert!(!stack.can_redo(), "Redo stack should be cleared after new command");
}

#[test]
fn test_undo_stack_size_limit() {
    let mut stack = UndoStack::with_capacity(3);

    // Push more commands than the capacity
    for i in 1..=5 {
        let mut cmd = TestCommand::new(&format!("Command {}", i), i * 10);
        cmd.execute().expect("Failed to execute");
        stack.push(Box::new(cmd));
    }

    // Stack should only contain the last 3 commands
    assert_eq!(stack.undo_count(), 3, "Stack should only hold 3 commands");

    // Undo 3 times (should succeed)
    for _ in 0..3 {
        stack.undo().expect("Failed to undo");
    }

    // 4th undo should fail
    let result = stack.undo();
    assert!(result.is_err(), "Should not be able to undo more than 3 times");
}

#[test]
fn test_multiple_undo_redo_cycles() {
    let mut stack = UndoStack::new();

    // Execute and push 3 commands
    for i in 1..=3 {
        let mut cmd = TestCommand::new(&format!("Command {}", i), i * 10);
        cmd.execute().expect("Failed to execute");
        stack.push(Box::new(cmd));
    }

    assert_eq!(stack.undo_count(), 3);

    // Undo all
    stack.undo().expect("Failed to undo");
    stack.undo().expect("Failed to undo");
    stack.undo().expect("Failed to undo");
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 3);

    // Redo all
    stack.redo().expect("Failed to redo");
    stack.redo().expect("Failed to redo");
    stack.redo().expect("Failed to redo");
    assert_eq!(stack.undo_count(), 3);
    assert_eq!(stack.redo_count(), 0);
}

#[test]
fn test_peek_operations() {
    let mut stack = UndoStack::new();

    let mut cmd = TestCommand::new("Test Command", 0);
    cmd.execute().expect("Failed to execute");
    stack.push(Box::new(cmd));

    // Test peek_undo
    assert_eq!(stack.peek_undo(), Some("Test Command"));

    // Undo and test peek_redo
    stack.undo().expect("Failed to undo");
    assert_eq!(stack.peek_redo(), Some("Test Command"));
}

#[test]
fn test_stack_clear() {
    let mut stack = UndoStack::new();

    // Add some commands
    for i in 1..=3 {
        let mut cmd = TestCommand::new(&format!("Command {}", i), i * 10);
        cmd.execute().expect("Failed to execute");
        stack.push(Box::new(cmd));
    }

    stack.undo().expect("Failed to undo");
    assert!(stack.can_undo());
    assert!(stack.can_redo());

    // Clear should remove all commands
    stack.clear();
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 0);
}

#[test]
fn test_command_metadata() {
    let cmd = TestCommand::new("Test Metadata", 42);

    assert_eq!(cmd.description(), "Test Metadata");
    assert!(cmd.can_undo()); // By default, commands are reversible
    // Size bytes might be 0 for simple test commands
    assert!(cmd.size_bytes() >= 0); // Should have non-negative size
}
