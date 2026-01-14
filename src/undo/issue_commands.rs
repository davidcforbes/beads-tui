//! Concrete command implementations for issue operations

use super::{Command, CommandError, CommandMetadata, CommandResult};
use crate::beads::client::{BeadsClient, IssueUpdate};
use std::sync::Arc;

/// Command for updating issue fields
///
/// Stores both old and new values to enable undo/redo.
/// Supports updating title, status, priority, assignee, labels, and description.
#[derive(Debug)]
pub struct IssueUpdateCommand {
    /// Client for executing beads commands
    client: Arc<BeadsClient>,
    /// ID of the issue to update
    issue_id: String,
    /// New field values to apply
    new_values: IssueUpdate,
    /// Old field values for undo (captured during execute)
    old_values: Option<IssueUpdate>,
    /// Command metadata
    metadata: CommandMetadata,
    /// Whether the command has been executed
    executed: bool,
}

impl IssueUpdateCommand {
    /// Create a new issue update command
    ///
    /// # Arguments
    /// * `client` - BeadsClient for executing updates
    /// * `issue_id` - ID of the issue to update
    /// * `updates` - Field updates to apply
    pub fn new(
        client: Arc<BeadsClient>,
        issue_id: impl Into<String>,
        updates: IssueUpdate,
    ) -> Self {
        let issue_id = issue_id.into();
        let description = Self::build_description(&issue_id, &updates);

        // Estimate size: issue_id + field values
        let size_bytes = issue_id.len()
            + updates
                .title
                .as_ref()
                .map(|s: &String| s.len())
                .unwrap_or(0)
            + updates
                .description
                .as_ref()
                .map(|s: &String| s.len())
                .unwrap_or(0)
            + updates
                .assignee
                .as_ref()
                .map(|s: &String| s.len())
                .unwrap_or(0)
            + updates
                .labels
                .as_ref()
                .map(|v: &Vec<String>| v.iter().map(|s: &String| s.len()).sum::<usize>())
                .unwrap_or(0)
            + 100; // Overhead for other fields

        Self {
            client,
            issue_id,
            new_values: updates,
            old_values: None,
            metadata: CommandMetadata::new(description).with_size(size_bytes),
            executed: false,
        }
    }

    /// Build a human-readable description of the update
    fn build_description(issue_id: &str, updates: &IssueUpdate) -> String {
        let mut parts = Vec::new();

        if updates.title.is_some() {
            parts.push("title");
        }
        if updates.issue_type.is_some() {
            parts.push("type");
        }
        if updates.status.is_some() {
            parts.push("status");
        }
        if updates.priority.is_some() {
            parts.push("priority");
        }
        if updates.assignee.is_some() {
            parts.push("assignee");
        }
        if updates.labels.is_some() {
            parts.push("labels");
        }
        if updates.description.is_some() {
            parts.push("description");
        }

        if parts.is_empty() {
            format!("Update issue {}", issue_id)
        } else {
            format!("Update {} on issue {}", parts.join(", "), issue_id)
        }
    }

    /// Fetch current issue state to capture old values
    async fn fetch_current_state(&self) -> CommandResult<IssueUpdate> {
        // Fetch the current issue
        let issue =
            self.client.get_issue(&self.issue_id).await.map_err(|e| {
                CommandError::ExecutionFailed(format!("Failed to fetch issue: {}", e))
            })?;

        // Build IssueUpdate from current values (only for fields we're updating)
        let mut old_values = IssueUpdate::new();

        if self.new_values.title.is_some() {
            old_values.title = Some(issue.title.clone());
        }
        if self.new_values.issue_type.is_some() {
            old_values.issue_type = Some(issue.issue_type);
        }
        if self.new_values.status.is_some() {
            old_values.status = Some(issue.status);
        }
        if self.new_values.priority.is_some() {
            old_values.priority = Some(issue.priority);
        }
        if self.new_values.assignee.is_some() {
            old_values.assignee = issue.assignee.clone();
        }
        if self.new_values.labels.is_some() {
            old_values.labels = Some(issue.labels.clone());
        }
        if self.new_values.description.is_some() {
            old_values.description = Some(issue.description.clone().unwrap_or_default());
        }

        Ok(old_values)
    }
}

impl Command for IssueUpdateCommand {
    fn execute(&mut self) -> CommandResult<()> {
        if self.executed {
            return Err(CommandError::InvalidState(
                "Command already executed".to_string(),
            ));
        }

        // Use tokio runtime to run async code
        let result = crate::runtime::RUNTIME.block_on(async {
            // Capture current state before applying changes
            let old_values = self.fetch_current_state().await?;
            self.old_values = Some(old_values);

            // Apply the update
            self.client
                .update_issue(&self.issue_id, self.new_values.clone())
                .await
                .map_err(|e| {
                    CommandError::ExecutionFailed(format!("Failed to update issue: {}", e))
                })?;

            Ok(())
        });

        if result.is_ok() {
            self.executed = true;
        }

        result
    }

    fn undo(&mut self) -> CommandResult<()> {
        if !self.executed {
            return Err(CommandError::InvalidState(
                "Command not executed".to_string(),
            ));
        }

        let old_values = self
            .old_values
            .clone()
            .ok_or_else(|| CommandError::InvalidState("No old values captured".to_string()))?;

        // Use tokio runtime to run async code
        let result = crate::runtime::RUNTIME.block_on(async {
            self.client
                .update_issue(&self.issue_id, old_values)
                .await
                .map_err(|e| CommandError::UndoFailed(format!("Failed to restore issue: {}", e)))?;

            Ok(())
        });

        if result.is_ok() {
            self.executed = false;
        }

        result
    }

    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
}

/// Command for creating a new issue
///
/// Stores creation parameters and the ID of the created issue for undo (deletion).
#[derive(Debug)]
pub struct IssueCreateCommand {
    /// Client for executing beads commands
    client: Arc<BeadsClient>,
    /// Title of the issue to create
    title: String,
    /// Type of the issue
    issue_type: crate::beads::models::IssueType,
    /// Priority of the issue
    priority: crate::beads::models::Priority,
    /// ID of the created issue (captured during execute)
    created_id: Option<String>,
    /// Command metadata
    metadata: CommandMetadata,
    /// Whether the command has been executed
    executed: bool,
}

impl IssueCreateCommand {
    /// Create a new issue creation command
    ///
    /// # Arguments
    /// * `client` - BeadsClient for executing commands
    /// * `title` - Title of the issue to create
    /// * `issue_type` - Type of the issue (task, bug, feature, epic)
    /// * `priority` - Priority of the issue (P0-P4)
    pub fn new(
        client: Arc<BeadsClient>,
        title: impl Into<String>,
        issue_type: crate::beads::models::IssueType,
        priority: crate::beads::models::Priority,
    ) -> Self {
        let title = title.into();
        let description = format!(
            "Create {} issue: {} ({})",
            issue_type.to_string().to_lowercase(),
            title,
            priority
        );

        // Estimate size: title + overhead
        let size_bytes = title.len() + 50;

        Self {
            client,
            title,
            issue_type,
            priority,
            created_id: None,
            metadata: CommandMetadata::new(description).with_size(size_bytes),
            executed: false,
        }
    }
}

impl Command for IssueCreateCommand {
    fn execute(&mut self) -> CommandResult<()> {
        if self.executed {
            return Err(CommandError::InvalidState(
                "Command already executed".to_string(),
            ));
        }

        // Use tokio runtime to run async code
        let result = crate::runtime::RUNTIME.block_on(async {
            // Create the issue
            let issue_id = self
                .client
                .create_issue(&self.title, self.issue_type, self.priority)
                .await
                .map_err(|e| {
                    CommandError::ExecutionFailed(format!("Failed to create issue: {}", e))
                })?;

            self.created_id = Some(issue_id);
            Ok(())
        });

        if result.is_ok() {
            self.executed = true;
        }

        result
    }

    fn undo(&mut self) -> CommandResult<()> {
        if !self.executed {
            return Err(CommandError::InvalidState(
                "Command not executed".to_string(),
            ));
        }

        let issue_id = self
            .created_id
            .as_ref()
            .ok_or_else(|| CommandError::InvalidState("No issue ID captured".to_string()))?;

        // Use tokio runtime to run async code
        let result = crate::runtime::RUNTIME.block_on(async {
            self.client
                .delete_issue(issue_id)
                .await
                .map_err(|e| CommandError::UndoFailed(format!("Failed to delete issue: {}", e)))?;

            Ok(())
        });

        if result.is_ok() {
            self.executed = false;
            self.created_id = None;
        }

        result
    }

    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, Priority};

    fn create_test_client() -> Arc<BeadsClient> {
        Arc::new(BeadsClient::new())
    }

    #[test]
    fn test_build_description_single_field() {
        let updates = IssueUpdate::new().title("New title".to_string());
        let desc = IssueUpdateCommand::build_description("issue-123", &updates);
        assert_eq!(desc, "Update title on issue issue-123");
    }

    #[test]
    fn test_build_description_multiple_fields() {
        let updates = IssueUpdate::new()
            .title("New title".to_string())
            .status(IssueStatus::InProgress)
            .priority(Priority::P1);
        let desc = IssueUpdateCommand::build_description("issue-123", &updates);
        assert!(desc.contains("title"));
        assert!(desc.contains("status"));
        assert!(desc.contains("priority"));
        assert!(desc.contains("issue-123"));
    }

    #[test]
    fn test_build_description_empty_update() {
        let updates = IssueUpdate::new();
        let desc = IssueUpdateCommand::build_description("issue-123", &updates);
        assert_eq!(desc, "Update issue issue-123");
    }

    #[test]
    fn test_command_creation() {
        let client = create_test_client();
        let updates = IssueUpdate::new().title("New title".to_string());
        let cmd = IssueUpdateCommand::new(client, "issue-123", updates);

        assert_eq!(cmd.issue_id, "issue-123");
        assert!(!cmd.executed);
        assert!(cmd.old_values.is_none());
        assert!(cmd.can_undo());
    }

    #[test]
    fn test_command_metadata() {
        let client = create_test_client();
        let updates = IssueUpdate::new().title("New title".to_string());
        let cmd = IssueUpdateCommand::new(client, "issue-123", updates);

        let desc = cmd.description();
        assert!(desc.contains("title"));
        assert!(desc.contains("issue-123"));

        // Size should account for issue_id and title
        assert!(cmd.size_bytes() > "issue-123".len());
        assert!(cmd.size_bytes() > "New title".len());
    }

    #[test]
    fn test_double_execute_fails() {
        let client = create_test_client();
        let updates = IssueUpdate::new().title("New title".to_string());
        let mut cmd = IssueUpdateCommand::new(client, "issue-123", updates);

        // Mock execution state (we can't actually execute without a real issue)
        cmd.executed = true;

        let result = cmd.execute();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    #[test]
    fn test_undo_without_execute_fails() {
        let client = create_test_client();
        let updates = IssueUpdate::new().title("New title".to_string());
        let mut cmd = IssueUpdateCommand::new(client, "issue-123", updates);

        let result = cmd.undo();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    // IssueCreateCommand tests

    #[test]
    fn test_create_command_creation() {
        use crate::beads::models::{IssueType, Priority};

        let client = create_test_client();
        let cmd = IssueCreateCommand::new(client, "Test issue", IssueType::Task, Priority::P2);

        assert_eq!(cmd.title, "Test issue");
        assert_eq!(cmd.issue_type, IssueType::Task);
        assert_eq!(cmd.priority, Priority::P2);
        assert!(!cmd.executed);
        assert!(cmd.created_id.is_none());
        assert!(cmd.can_undo());
    }

    #[test]
    fn test_create_command_metadata() {
        use crate::beads::models::{IssueType, Priority};

        let client = create_test_client();
        let cmd = IssueCreateCommand::new(client, "Test issue", IssueType::Feature, Priority::P1);

        let desc = cmd.description();
        assert!(desc.contains("Create"));
        assert!(desc.contains("feature"));
        assert!(desc.contains("Test issue"));
        assert!(desc.contains("P1"));

        // Size should account for title
        assert!(cmd.size_bytes() > "Test issue".len());
    }

    #[test]
    fn test_create_command_double_execute_fails() {
        use crate::beads::models::{IssueType, Priority};

        let client = create_test_client();
        let mut cmd = IssueCreateCommand::new(client, "Test issue", IssueType::Bug, Priority::P0);

        // Mock execution state
        cmd.executed = true;

        let result = cmd.execute();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }

    #[test]
    fn test_create_command_undo_without_execute_fails() {
        use crate::beads::models::{IssueType, Priority};

        let client = create_test_client();
        let mut cmd = IssueCreateCommand::new(client, "Test issue", IssueType::Task, Priority::P3);

        let result = cmd.undo();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::InvalidState(_)));
    }
}
