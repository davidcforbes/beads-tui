/// Beads CLI client for executing commands and parsing results
use super::{error::*, models::*, parser};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Client for interacting with the beads CLI
#[derive(Debug, Clone)]
pub struct BeadsClient {
    command_timeout: Duration,
    bd_path: String,
}

impl Default for BeadsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BeadsClient {
    /// Create a new beads client with default settings
    pub fn new() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            bd_path: "bd".to_string(),
        }
    }

    /// Create a new beads client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            command_timeout: timeout,
            bd_path: "bd".to_string(),
        }
    }

    /// Set a custom path to the bd executable
    pub fn with_bd_path(mut self, path: String) -> Self {
        self.bd_path = path;
        self
    }

    /// Check if beads CLI is available
    pub fn check_available() -> Result<bool> {
        match Command::new("bd")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => Ok(status.success()),
            Err(_) => Ok(false),
        }
    }

    /// List issues with optional filters
    pub async fn list_issues(
        &self,
        status: Option<IssueStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<Issue>> {
        let mut args = vec!["list".to_string(), "--json".to_string()];

        if let Some(s) = status {
            args.push("--status".to_string());
            args.push(s.to_string());
        }

        if let Some(l) = limit {
            args.push("--limit".to_string());
            args.push(l.to_string());
        }

        let output = self.execute_command(&args).await?;
        parser::parse_issue_list(&output)
    }

    /// Get a specific issue by ID
    pub async fn get_issue(&self, id: &str) -> Result<Issue> {
        let args = vec!["show".to_string(), id.to_string(), "--json".to_string()];
        let output = self.execute_command(&args).await?;
        parser::parse_issue(&output)
    }

    /// Create a new issue
    pub async fn create_issue(
        &self,
        title: &str,
        issue_type: IssueType,
        priority: Priority,
    ) -> Result<String> {
        let args = vec![
            "create".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--type".to_string(),
            issue_type.to_string(),
            "--priority".to_string(),
            priority.to_string(),
        ];

        let output = self.execute_command(&args).await?;
        parser::parse_create_response(&output)
    }

    /// Create a new issue with full options
    pub async fn create_issue_full(&self, params: super::models::CreateIssueParams<'_>) -> Result<String> {
        let mut args = vec![
            "create".to_string(),
            "--title".to_string(),
            params.title.to_string(),
            "--type".to_string(),
            params.issue_type.to_string(),
            "--priority".to_string(),
            params.priority.to_string(),
        ];

        if let Some(s) = params.status {
            args.push("--status".to_string());
            args.push(s.to_string());
        }

        if let Some(a) = params.assignee {
            args.push("--assignee".to_string());
            args.push(a.to_string());
        }

        if !params.labels.is_empty() {
            args.push("--label".to_string());
            args.push(params.labels.join(","));
        }

        if let Some(d) = params.description {
            args.push("--description".to_string());
            args.push(d.to_string());
        }

        let output = self.execute_command(&args).await?;
        parser::parse_create_response(&output)
    }

    /// Update an issue
    pub async fn update_issue(&self, id: &str, updates: IssueUpdate) -> Result<()> {
        let mut args = vec!["update".to_string(), id.to_string()];

        if let Some(title) = updates.title {
            args.push("--title".to_string());
            args.push(title);
        }

        if let Some(issue_type) = updates.issue_type {
            args.push("--type".to_string());
            args.push(issue_type.to_string());
        }

        if let Some(status) = updates.status {
            args.push("--status".to_string());
            args.push(status.to_string());
        }

        if let Some(priority) = updates.priority {
            args.push("--priority".to_string());
            args.push(priority.to_string());
        }

        if let Some(assignee) = updates.assignee {
            args.push("--assignee".to_string());
            args.push(assignee);
        }

        if let Some(labels) = updates.labels {
            if !labels.is_empty() {
                args.push("--label".to_string());
                args.push(labels.join(","));
            }
        }

        if let Some(description) = updates.description {
            args.push("--description".to_string());
            args.push(description);
        }

        self.execute_command(&args).await?;
        Ok(())
    }

    /// Close an issue
    pub async fn close_issue(&self, id: &str, reason: Option<&str>) -> Result<()> {
        let mut args = vec!["close".to_string(), id.to_string()];

        if let Some(r) = reason {
            args.push("--reason".to_string());
            args.push(r.to_string());
        }

        self.execute_command(&args).await?;
        Ok(())
    }

    /// Reopen a closed issue
    pub async fn reopen_issue(&self, id: &str) -> Result<()> {
        let args = vec!["reopen".to_string(), id.to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Delete an issue
    pub async fn delete_issue(&self, id: &str) -> Result<()> {
        let args = vec!["delete".to_string(), id.to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Get issue statistics
    pub async fn get_stats(&self) -> Result<IssueStats> {
        let args = vec!["stats".to_string(), "--json".to_string()];
        let output = self.execute_command(&args).await?;
        parser::parse_stats(&output)
    }

    /// List all labels
    pub async fn list_labels(&self) -> Result<Vec<Label>> {
        let args = vec!["labels".to_string(), "--json".to_string()];
        let output = self.execute_command(&args).await?;
        parser::parse_labels(&output)
    }

    /// Add a dependency between issues
    pub async fn add_dependency(&self, issue: &str, depends_on: &str) -> Result<()> {
        let args = vec![
            "dep".to_string(),
            "add".to_string(),
            issue.to_string(),
            depends_on.to_string(),
        ];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Remove a dependency
    pub async fn remove_dependency(&self, issue: &str, depends_on: &str) -> Result<()> {
        let args = vec![
            "dep".to_string(),
            "remove".to_string(),
            issue.to_string(),
            depends_on.to_string(),
        ];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Check if beads daemon is running
    pub async fn check_daemon_status(&self) -> Result<bool> {
        let args = vec!["daemon".to_string(), "--status".to_string()];
        match self.execute_command(&args).await {
            Ok(output) => {
                // Check if output indicates daemon is running
                Ok(output.contains("running") || output.contains("active"))
            }
            Err(_) => {
                // If command fails, assume daemon is not running
                Ok(false)
            }
        }
    }

    /// Start the beads daemon
    pub async fn start_daemon(&self) -> Result<()> {
        let args = vec!["daemon".to_string(), "--start".to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Stop the beads daemon
    pub async fn stop_daemon(&self) -> Result<()> {
        let args = vec!["daemon".to_string(), "--stop".to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Sync beads database with remote
    pub async fn sync_database(&self) -> Result<String> {
        let args = vec!["sync".to_string()];
        self.execute_command(&args).await
    }

    /// Export issues to a file
    pub async fn export_issues(&self, path: &str) -> Result<()> {
        let args = vec!["export".to_string(), path.to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Import issues from a file
    pub async fn import_issues(&self, path: &str) -> Result<()> {
        let args = vec!["import".to_string(), path.to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Verify database integrity
    pub async fn verify_database(&self) -> Result<String> {
        let args = vec!["doctor".to_string()];
        self.execute_command(&args).await
    }

    /// Compact database (remove history)
    pub async fn compact_database(&self) -> Result<()> {
        let args = vec!["compact".to_string()];
        self.execute_command(&args).await?;
        Ok(())
    }

    /// Execute a bd command with timeout
    async fn execute_command(&self, args: &[String]) -> Result<String> {
        let mut cmd = TokioCommand::new(&self.bd_path);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BeadsError::BeadsNotFound
            } else {
                BeadsError::Io(e)
            }
        })?;

        let output = timeout(self.command_timeout, child.wait_with_output())
            .await
            .map_err(|_| BeadsError::Timeout(self.command_timeout.as_millis() as u64))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Builder for issue updates
#[derive(Debug, Default, Clone)]
pub struct IssueUpdate {
    pub title: Option<String>,
    pub issue_type: Option<IssueType>,
    pub status: Option<IssueStatus>,
    pub priority: Option<Priority>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    pub description: Option<String>,
}

impl IssueUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn issue_type(mut self, issue_type: IssueType) -> Self {
        self.issue_type = Some(issue_type);
        self
    }

    pub fn status(mut self, status: IssueStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn assignee(mut self, assignee: String) -> Self {
        self.assignee = Some(assignee);
        self
    }

    pub fn labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = BeadsClient::new();
        assert_eq!(client.bd_path, "bd");
    }

    #[tokio::test]
    async fn test_client_with_timeout() {
        let client = BeadsClient::with_timeout(Duration::from_secs(60));
        assert_eq!(client.command_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_client_default() {
        let client = BeadsClient::default();
        assert_eq!(client.bd_path, "bd");
        assert_eq!(client.command_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_client_with_bd_path() {
        let client = BeadsClient::new().with_bd_path("/usr/local/bin/bd".to_string());
        assert_eq!(client.bd_path, "/usr/local/bin/bd");
    }

    #[test]
    fn test_client_builder_chain() {
        let client = BeadsClient::with_timeout(Duration::from_secs(120))
            .with_bd_path("/custom/bd".to_string());
        assert_eq!(client.command_timeout, Duration::from_secs(120));
        assert_eq!(client.bd_path, "/custom/bd");
    }

    #[test]
    fn test_client_clone() {
        let client1 = BeadsClient::new().with_bd_path("/test/bd".to_string());
        let client2 = client1.clone();
        assert_eq!(client1.bd_path, client2.bd_path);
        assert_eq!(client1.command_timeout, client2.command_timeout);
    }

    // IssueUpdate tests
    #[test]
    fn test_issue_update_new() {
        let update = IssueUpdate::new();
        assert!(update.title.is_none());
        assert!(update.issue_type.is_none());
        assert!(update.status.is_none());
        assert!(update.priority.is_none());
        assert!(update.assignee.is_none());
        assert!(update.labels.is_none());
        assert!(update.description.is_none());
    }

    #[test]
    fn test_issue_update_default() {
        let update = IssueUpdate::default();
        assert!(update.title.is_none());
        assert!(update.issue_type.is_none());
        assert!(update.status.is_none());
        assert!(update.priority.is_none());
        assert!(update.assignee.is_none());
        assert!(update.labels.is_none());
        assert!(update.description.is_none());
    }

    #[test]
    fn test_issue_update_title() {
        let update = IssueUpdate::new().title("New Title".to_string());
        assert_eq!(update.title, Some("New Title".to_string()));
    }

    #[test]
    fn test_issue_update_issue_type() {
        let update = IssueUpdate::new().issue_type(IssueType::Bug);
        assert_eq!(update.issue_type, Some(IssueType::Bug));
    }

    #[test]
    fn test_issue_update_status() {
        let update = IssueUpdate::new().status(IssueStatus::InProgress);
        assert_eq!(update.status, Some(IssueStatus::InProgress));
    }

    #[test]
    fn test_issue_update_priority() {
        let update = IssueUpdate::new().priority(Priority::P1);
        assert_eq!(update.priority, Some(Priority::P1));
    }

    #[test]
    fn test_issue_update_assignee() {
        let update = IssueUpdate::new().assignee("user@example.com".to_string());
        assert_eq!(update.assignee, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_issue_update_labels() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        let update = IssueUpdate::new().labels(labels.clone());
        assert_eq!(update.labels, Some(labels));
    }

    #[test]
    fn test_issue_update_description() {
        let update = IssueUpdate::new().description("A detailed description".to_string());
        assert_eq!(update.description, Some("A detailed description".to_string()));
    }

    #[test]
    fn test_issue_update_builder_chain() {
        let update = IssueUpdate::new()
            .title("Fix Bug".to_string())
            .issue_type(IssueType::Bug)
            .status(IssueStatus::Open)
            .priority(Priority::P1)
            .assignee("dev@example.com".to_string())
            .labels(vec!["critical".to_string()])
            .description("Fix critical bug".to_string());

        assert_eq!(update.title, Some("Fix Bug".to_string()));
        assert_eq!(update.issue_type, Some(IssueType::Bug));
        assert_eq!(update.status, Some(IssueStatus::Open));
        assert_eq!(update.priority, Some(Priority::P1));
        assert_eq!(update.assignee, Some("dev@example.com".to_string()));
        assert_eq!(update.labels, Some(vec!["critical".to_string()]));
        assert_eq!(update.description, Some("Fix critical bug".to_string()));
    }

    #[test]
    fn test_issue_update_partial() {
        let update = IssueUpdate::new()
            .title("Partial Update".to_string())
            .priority(Priority::P2);

        assert_eq!(update.title, Some("Partial Update".to_string()));
        assert!(update.issue_type.is_none());
        assert!(update.status.is_none());
        assert_eq!(update.priority, Some(Priority::P2));
        assert!(update.assignee.is_none());
        assert!(update.labels.is_none());
        assert!(update.description.is_none());
    }

    #[test]
    fn test_issue_update_clone() {
        let update1 = IssueUpdate::new()
            .title("Test".to_string())
            .priority(Priority::P1);
        let update2 = update1.clone();

        assert_eq!(update1.title, update2.title);
        assert_eq!(update1.priority, update2.priority);
    }

    #[test]
    fn test_issue_update_empty_labels() {
        let update = IssueUpdate::new().labels(vec![]);
        assert_eq!(update.labels, Some(vec![]));
    }

    #[test]
    fn test_issue_update_multiple_labels() {
        let labels = vec![
            "bug".to_string(),
            "urgent".to_string(),
            "backend".to_string(),
        ];
        let update = IssueUpdate::new().labels(labels.clone());
        assert_eq!(update.labels, Some(labels));
    }

    #[test]
    fn test_issue_update_long_title() {
        let long_title = "A".repeat(200);
        let update = IssueUpdate::new().title(long_title.clone());
        assert_eq!(update.title, Some(long_title));
    }

    #[test]
    fn test_issue_update_long_description() {
        let long_desc = "Description ".repeat(100);
        let update = IssueUpdate::new().description(long_desc.clone());
        assert_eq!(update.description, Some(long_desc));
    }

    #[test]
    fn test_issue_update_special_chars_in_title() {
        let title = "Fix: Handle edge case with 'quotes' & \"symbols\"".to_string();
        let update = IssueUpdate::new().title(title.clone());
        assert_eq!(update.title, Some(title));
    }

    #[test]
    fn test_client_timeout_values() {
        let client1 = BeadsClient::with_timeout(Duration::from_millis(100));
        assert_eq!(client1.command_timeout, Duration::from_millis(100));

        let client2 = BeadsClient::with_timeout(Duration::from_secs(300));
        assert_eq!(client2.command_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_client_default_timeout() {
        let client = BeadsClient::new();
        assert_eq!(client.command_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_issue_update_all_issue_types() {
        let bug = IssueUpdate::new().issue_type(IssueType::Bug);
        assert_eq!(bug.issue_type, Some(IssueType::Bug));

        let feature = IssueUpdate::new().issue_type(IssueType::Feature);
        assert_eq!(feature.issue_type, Some(IssueType::Feature));

        let task = IssueUpdate::new().issue_type(IssueType::Task);
        assert_eq!(task.issue_type, Some(IssueType::Task));

        let epic = IssueUpdate::new().issue_type(IssueType::Epic);
        assert_eq!(epic.issue_type, Some(IssueType::Epic));

        let chore = IssueUpdate::new().issue_type(IssueType::Chore);
        assert_eq!(chore.issue_type, Some(IssueType::Chore));
    }

    #[test]
    fn test_issue_update_all_statuses() {
        let open = IssueUpdate::new().status(IssueStatus::Open);
        assert_eq!(open.status, Some(IssueStatus::Open));

        let in_progress = IssueUpdate::new().status(IssueStatus::InProgress);
        assert_eq!(in_progress.status, Some(IssueStatus::InProgress));

        let blocked = IssueUpdate::new().status(IssueStatus::Blocked);
        assert_eq!(blocked.status, Some(IssueStatus::Blocked));

        let closed = IssueUpdate::new().status(IssueStatus::Closed);
        assert_eq!(closed.status, Some(IssueStatus::Closed));
    }

    #[test]
    fn test_issue_update_all_priorities() {
        let p0 = IssueUpdate::new().priority(Priority::P0);
        assert_eq!(p0.priority, Some(Priority::P0));

        let p1 = IssueUpdate::new().priority(Priority::P1);
        assert_eq!(p1.priority, Some(Priority::P1));

        let p2 = IssueUpdate::new().priority(Priority::P2);
        assert_eq!(p2.priority, Some(Priority::P2));

        let p3 = IssueUpdate::new().priority(Priority::P3);
        assert_eq!(p3.priority, Some(Priority::P3));

        let p4 = IssueUpdate::new().priority(Priority::P4);
        assert_eq!(p4.priority, Some(Priority::P4));
    }

    #[test]
    fn test_client_bd_path_variations() {
        let client1 = BeadsClient::new().with_bd_path("bd".to_string());
        assert_eq!(client1.bd_path, "bd");

        let client2 = BeadsClient::new().with_bd_path("/usr/bin/bd".to_string());
        assert_eq!(client2.bd_path, "/usr/bin/bd");

        let client3 = BeadsClient::new().with_bd_path("C:\\Program Files\\bd.exe".to_string());
        assert_eq!(client3.bd_path, "C:\\Program Files\\bd.exe");
    }

    #[test]
    fn test_issue_update_unicode_title() {
        let title = "Fix 🐛 in 日本語 feature".to_string();
        let update = IssueUpdate::new().title(title.clone());
        assert_eq!(update.title, Some(title));
    }

    #[test]
    fn test_client_multiple_bd_path_chains() {
        let client = BeadsClient::new()
            .with_bd_path("/first/bd".to_string())
            .with_bd_path("/second/bd".to_string());
        assert_eq!(client.bd_path, "/second/bd"); // Last one wins
    }

    #[test]
    fn test_issue_update_overwrite_values() {
        let update = IssueUpdate::new()
            .title("First Title".to_string())
            .title("Second Title".to_string());
        assert_eq!(update.title, Some("Second Title".to_string()));
    }

    #[test]
    fn test_client_zero_timeout() {
        let client = BeadsClient::with_timeout(Duration::from_secs(0));
        assert_eq!(client.command_timeout, Duration::from_secs(0));
    }

    #[test]
    fn test_issue_update_single_label() {
        let update = IssueUpdate::new().labels(vec!["single".to_string()]);
        assert_eq!(update.labels, Some(vec!["single".to_string()]));
    }
}
