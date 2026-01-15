//! Task manager for spawning and tracking background tasks

use super::handle::{TaskHandle, TaskId, TaskResult, TaskStatus};
use crate::beads::client::BeadsClient;
use crate::runtime::RUNTIME;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Message sent from background tasks to the main thread
#[derive(Debug, Clone)]
pub enum TaskMessage {
    /// Task has completed
    Completed { id: TaskId, result: TaskResult },
    /// Task status changed
    StatusChanged { id: TaskId, status: TaskStatus },
}

/// Manager for background tasks
#[derive(Debug)]
pub struct TaskManager {
    /// Channel receiver for task messages
    rx: UnboundedReceiver<TaskMessage>,
    /// Channel sender for task messages (cloned for each task)
    tx: UnboundedSender<TaskMessage>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self { rx, tx }
    }

    /// Spawn a background task
    pub fn spawn_task<F, Fut>(&self, name: &str, client: BeadsClient, f: F) -> TaskHandle
    where
        F: FnOnce(BeadsClient) -> Fut + Send + 'static,
        Fut: Future<Output = TaskResult> + Send + 'static,
    {
        let handle = TaskHandle::new(name.to_string());
        let task_id = handle.id();
        let status_arc = Arc::new(handle.clone());
        let tx = self.tx.clone();

        // Spawn the task on the global tokio runtime
        RUNTIME.spawn(async move {
            // Update status to running
            status_arc.set_status(TaskStatus::Running);
            let _ = tx.send(TaskMessage::StatusChanged {
                id: task_id,
                status: TaskStatus::Running,
            });

            // Execute the task
            let result = f(client).await;

            // Update status based on result
            let final_status = match &result {
                Ok(_) => TaskStatus::Completed,
                Err(_) => TaskStatus::Failed,
            };

            status_arc.set_status(final_status);
            status_arc.set_result(result.clone());

            // Send completion message
            let _ = tx.send(TaskMessage::Completed {
                id: task_id,
                result,
            });
        });

        handle
    }

    /// Spawn a background task with progress tracking
    pub fn spawn_task_with_progress<F, Fut>(
        &self,
        name: &str,
        client: BeadsClient,
        f: F,
    ) -> TaskHandle
    where
        F: FnOnce(BeadsClient) -> Fut + Send + 'static,
        Fut: Future<Output = TaskResult> + Send + 'static,
    {
        let handle = TaskHandle::with_progress(name.to_string());
        let task_id = handle.id();
        let status_arc = Arc::new(handle.clone());
        let tx = self.tx.clone();

        // Spawn the task on tokio runtime
        tokio::spawn(async move {
            // Update status to running
            status_arc.set_status(TaskStatus::Running);
            let _ = tx.send(TaskMessage::StatusChanged {
                id: task_id,
                status: TaskStatus::Running,
            });

            // Execute the task
            let result = f(client).await;

            // Update status based on result
            let final_status = match &result {
                Ok(_) => TaskStatus::Completed,
                Err(_) => TaskStatus::Failed,
            };

            status_arc.set_status(final_status);
            status_arc.set_result(result.clone());

            // Send completion message
            let _ = tx.send(TaskMessage::Completed {
                id: task_id,
                result,
            });
        });

        handle
    }

    /// Poll for task messages (non-blocking)
    /// Returns all available messages
    pub fn poll(&mut self) -> Vec<TaskMessage> {
        let mut messages = Vec::new();

        // Drain all available messages without blocking
        while let Ok(msg) = self.rx.try_recv() {
            messages.push(msg);
        }

        messages
    }

    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        !self.rx.is_empty()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::TaskOutput;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_task_spawn_and_complete() {
        let manager = TaskManager::new();
        let client = BeadsClient::new();

        let handle = manager.spawn_task("test task", client, |_client| async move {
            sleep(Duration::from_millis(10)).await;
            Ok(TaskOutput::Success("done".to_string()))
        });

        // Wait for task to complete
        sleep(Duration::from_millis(50)).await;

        // Handle should be marked complete
        assert!(handle.is_complete());
    }

    #[tokio::test]
    async fn test_task_manager_poll() {
        let mut manager = TaskManager::new();
        let client = BeadsClient::new();

        let _handle = manager.spawn_task("test task", client, |_client| async move {
            Ok(TaskOutput::Success("done".to_string()))
        });

        // Wait a bit for task to execute
        sleep(Duration::from_millis(50)).await;

        // Poll for messages
        let messages = manager.poll();

        // Should have at least one message (status change and completion)
        assert!(!messages.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let manager = TaskManager::new();
        let client = BeadsClient::new();

        // Spawn multiple tasks
        let handle1 = manager.spawn_task("task 1", client.clone(), |_client| async move {
            sleep(Duration::from_millis(10)).await;
            Ok(TaskOutput::Success("1".to_string()))
        });

        let handle2 = manager.spawn_task("task 2", client, |_client| async move {
            sleep(Duration::from_millis(20)).await;
            Ok(TaskOutput::Success("2".to_string()))
        });

        // Wait for both to complete
        sleep(Duration::from_millis(100)).await;

        assert!(handle1.is_complete());
        assert!(handle2.is_complete());
    }
}
