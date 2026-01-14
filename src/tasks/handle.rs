//! Task handle types for tracking background task status

use super::error::TaskError;
use super::progress::ProgressTracker;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Unique identifier for a task
pub type TaskId = Uuid;

/// Status of a background task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending (not yet started)
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Output of a completed task
#[derive(Debug, Clone)]
pub enum TaskOutput {
    /// Generic success with message
    Success(String),
    /// Issues were updated
    IssuesUpdated,
    /// Database was compacted
    DatabaseCompacted,
    /// Stats were computed
    StatsComputed,
    /// Database was synced
    DatabaseSynced,
    /// Issues were exported
    IssuesExported(String), // File path
    /// Issues were imported
    IssuesImported(usize), // Count
    /// Issue was deleted
    IssueDeleted(String), // Issue ID
    /// Issue was created
    IssueCreated(String), // Issue ID
    /// Issue was updated
    IssueUpdated(String), // Issue ID
    /// Dependency was added
    DependencyAdded,
    /// Dependency was removed
    DependencyRemoved,
}

impl fmt::Display for TaskOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success(msg) => write!(f, "{}", msg),
            Self::IssuesUpdated => write!(f, "Issues updated"),
            Self::DatabaseCompacted => write!(f, "Database compacted"),
            Self::StatsComputed => write!(f, "Stats computed"),
            Self::DatabaseSynced => write!(f, "Database synced"),
            Self::IssuesExported(path) => write!(f, "Exported to {}", path),
            Self::IssuesImported(count) => write!(f, "Imported {} issues", count),
            Self::IssueDeleted(id) => write!(f, "Deleted issue {}", id),
            Self::IssueCreated(id) => write!(f, "Created issue {}", id),
            Self::IssueUpdated(id) => write!(f, "Updated issue {}", id),
            Self::DependencyAdded => write!(f, "Dependency added"),
            Self::DependencyRemoved => write!(f, "Dependency removed"),
        }
    }
}

/// Result type for tasks
pub type TaskResult = Result<TaskOutput, TaskError>;

/// Handle to a background task
#[derive(Clone)]
pub struct TaskHandle {
    id: TaskId,
    name: String,
    status: Arc<Mutex<TaskStatus>>,
    result: Arc<Mutex<Option<TaskResult>>>,
    cancellation_token: CancellationToken,
    created_at: Instant,
    progress: Option<ProgressTracker>,
}

impl TaskHandle {
    /// Create a new task handle
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            status: Arc::new(Mutex::new(TaskStatus::Pending)),
            result: Arc::new(Mutex::new(None)),
            cancellation_token: CancellationToken::new(),
            created_at: Instant::now(),
            progress: None,
        }
    }

    /// Create a new task handle with progress tracking
    pub fn with_progress(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            status: Arc::new(Mutex::new(TaskStatus::Pending)),
            result: Arc::new(Mutex::new(None)),
            cancellation_token: CancellationToken::new(),
            created_at: Instant::now(),
            progress: Some(ProgressTracker::new()),
        }
    }

    /// Get the task ID
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Get the task name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current status
    pub fn status(&self) -> TaskStatus {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Update the status
    pub(crate) fn set_status(&self, status: TaskStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }

    /// Get the task result (if completed)
    pub fn result(&self) -> Option<TaskResult> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Set the task result
    pub(crate) fn set_result(&self, result: TaskResult) {
        *self
            .result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
    }

    /// Get the cancellation token
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Request cancellation
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
        self.set_status(TaskStatus::Cancelled);
    }

    /// Check if cancellation was requested
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get the progress tracker (if available)
    pub fn progress(&self) -> Option<&ProgressTracker> {
        self.progress.as_ref()
    }

    /// Get time since task creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Check if the task is complete (success, failure, or cancelled)
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status(),
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

impl fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskHandle")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("status", &self.status())
            .field("elapsed", &self.elapsed())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_handle_creation() {
        let handle = TaskHandle::new("Test task".to_string());
        assert_eq!(handle.name(), "Test task");
        assert_eq!(handle.status(), TaskStatus::Pending);
        assert!(!handle.is_complete());
    }

    #[test]
    fn test_task_status_updates() {
        let handle = TaskHandle::new("Test task".to_string());

        handle.set_status(TaskStatus::Running);
        assert_eq!(handle.status(), TaskStatus::Running);

        handle.set_status(TaskStatus::Completed);
        assert_eq!(handle.status(), TaskStatus::Completed);
        assert!(handle.is_complete());
    }

    #[test]
    fn test_task_cancellation() {
        let handle = TaskHandle::new("Test task".to_string());

        assert!(!handle.is_cancelled());

        handle.cancel();

        assert!(handle.is_cancelled());
        assert_eq!(handle.status(), TaskStatus::Cancelled);
    }

    #[test]
    fn test_task_result() {
        let handle = TaskHandle::new("Test task".to_string());

        assert!(handle.result().is_none());

        handle.set_result(Ok(TaskOutput::Success("Done".to_string())));

        assert!(handle.result().is_some());
    }

    #[test]
    fn test_task_with_progress() {
        let handle = TaskHandle::with_progress("Test task".to_string());

        assert!(handle.progress().is_some());

        if let Some(progress) = handle.progress() {
            progress.set_progress(50, 100);
            assert_eq!(progress.percentage(), 50.0);
        }
    }
}
