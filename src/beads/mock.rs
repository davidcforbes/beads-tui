//! Mock backend for testing without requiring actual beads CLI
//!
//! This module provides a fake beads backend that can be used in tests
//! to simulate beads CLI behavior without spawning actual processes.

use super::{error::*, models::*};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock backend that simulates beads CLI behavior
#[derive(Debug, Clone)]
pub struct MockBeadsBackend {
    issues: Arc<Mutex<HashMap<String, Issue>>>,
    labels: Arc<Mutex<Vec<Label>>>,
    stats: Arc<Mutex<IssueStats>>,
    next_id: Arc<Mutex<u32>>,
}

impl Default for MockBeadsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBeadsBackend {
    /// Create a new mock backend with empty data
    pub fn new() -> Self {
        Self {
            issues: Arc::new(Mutex::new(HashMap::new())),
            labels: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(IssueStats {
                total_issues: 0,
                open: 0,
                in_progress: 0,
                blocked: 0,
                closed: 0,
                ready_to_work: 0,
                avg_lead_time_hours: 0.0,
            })),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Create a mock backend pre-populated with test data
    pub fn with_test_data() -> Self {
        let backend = Self::new();

        // Add some test issues
        let test_issues = vec![
            Issue {
                id: "beads-test-001".to_string(),
                title: "Test Issue 1".to_string(),
                status: IssueStatus::Open,
                priority: Priority::P1,
                issue_type: IssueType::Task,
                assignee: Some("test_user".to_string()),
                labels: vec!["test".to_string()],
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                description: Some("This is a test issue".to_string()),
                dependencies: Vec::new(),
                blocks: Vec::new(),
                closed: None,
                notes: Vec::new(),
            },
            Issue {
                id: "beads-test-002".to_string(),
                title: "Test Bug Fix".to_string(),
                status: IssueStatus::InProgress,
                priority: Priority::P0,
                issue_type: IssueType::Bug,
                assignee: Some("test_user".to_string()),
                labels: vec!["bug".to_string(), "critical".to_string()],
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                description: Some("Critical bug that needs fixing".to_string()),
                dependencies: Vec::new(),
                blocks: Vec::new(),
                closed: None,
                notes: Vec::new(),
            },
            Issue {
                id: "beads-test-003".to_string(),
                title: "Completed Feature".to_string(),
                status: IssueStatus::Closed,
                priority: Priority::P2,
                issue_type: IssueType::Feature,
                assignee: Some("test_user".to_string()),
                labels: vec!["feature".to_string()],
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                description: Some("A completed feature".to_string()),
                dependencies: Vec::new(),
                blocks: Vec::new(),
                closed: None,
                notes: Vec::new(),
            },
        ];

        {
            let mut issues = backend.issues.lock().unwrap();
            for issue in test_issues {
                issues.insert(issue.id.clone(), issue);
            }
        }

        // Add test labels
        {
            let mut labels = backend.labels.lock().unwrap();
            labels.extend(vec![
                Label {
                    name: "test".to_string(),
                    count: 1,
                },
                Label {
                    name: "bug".to_string(),
                    count: 1,
                },
                Label {
                    name: "feature".to_string(),
                    count: 1,
                },
                Label {
                    name: "critical".to_string(),
                    count: 1,
                },
            ]);
        }

        // Update stats
        {
            let mut stats = backend.stats.lock().unwrap();
            stats.total_issues = 3;
            stats.open = 1;
            stats.in_progress = 1;
            stats.closed = 1;
            stats.ready_to_work = 1;
        }

        backend
    }

    /// List issues with optional filters
    pub fn list_issues(
        &self,
        status: Option<IssueStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<Issue>> {
        let issues = self.issues.lock().unwrap();
        let mut result: Vec<Issue> = issues
            .values()
            .filter(|issue| match &status {
                Some(s) => issue.status == *s,
                None => true,
            })
            .cloned()
            .collect();

        // Sort by created date (newest first)
        result.sort_by(|a, b| b.created.cmp(&a.created));

        if let Some(l) = limit {
            result.truncate(l);
        }

        Ok(result)
    }

    /// Get a specific issue by ID
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let issues = self.issues.lock().unwrap();
        issues
            .get(id)
            .cloned()
            .ok_or(BeadsError::IssueNotFound(id.to_string()))
    }

    /// Create a new issue
    pub fn create_issue(
        &self,
        title: &str,
        issue_type: IssueType,
        priority: Priority,
    ) -> Result<String> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = format!("beads-mock-{:04}", *next_id);
            *next_id += 1;
            id
        };

        let issue = Issue {
            id: id.clone(),
            title: title.to_string(),
            status: IssueStatus::Open,
            priority,
            issue_type,
            assignee: None,
            labels: Vec::new(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            description: None,
            dependencies: Vec::new(),
            blocks: Vec::new(),
            closed: None,
            notes: Vec::new(),
        };

        let mut issues = self.issues.lock().unwrap();
        issues.insert(id.clone(), issue);

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.total_issues += 1;
        stats.open += 1;
        stats.ready_to_work += 1;

        Ok(id)
    }

    /// Update an issue
    pub fn update_issue(
        &self,
        id: &str,
        status: Option<IssueStatus>,
        priority: Option<Priority>,
        assignee: Option<String>,
    ) -> Result<()> {
        let mut issues = self.issues.lock().unwrap();
        let issue = issues
            .get_mut(id)
            .ok_or(BeadsError::IssueNotFound(id.to_string()))?;

        let old_status = issue.status;

        if let Some(s) = status {
            issue.status = s;
        }

        if let Some(p) = priority {
            issue.priority = p;
        }

        if assignee.is_some() {
            issue.assignee = assignee;
        }

        issue.updated = chrono::Utc::now();

        // Update stats if status changed
        if let Some(new_status) = status {
            if new_status != old_status {
                let mut stats = self.stats.lock().unwrap();
                match old_status {
                    IssueStatus::Open => stats.open = stats.open.saturating_sub(1),
                    IssueStatus::InProgress => {
                        stats.in_progress = stats.in_progress.saturating_sub(1)
                    }
                    IssueStatus::Blocked => stats.blocked = stats.blocked.saturating_sub(1),
                    IssueStatus::Closed => stats.closed = stats.closed.saturating_sub(1),
                }
                match new_status {
                    IssueStatus::Open => stats.open += 1,
                    IssueStatus::InProgress => stats.in_progress += 1,
                    IssueStatus::Blocked => stats.blocked += 1,
                    IssueStatus::Closed => stats.closed += 1,
                }
            }
        }

        Ok(())
    }

    /// Close an issue
    pub fn close_issue(&self, id: &str) -> Result<()> {
        self.update_issue(id, Some(IssueStatus::Closed), None, None)
    }

    /// Get issue statistics
    pub fn get_stats(&self) -> Result<IssueStats> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    /// List all labels
    pub fn list_labels(&self) -> Result<Vec<Label>> {
        let labels = self.labels.lock().unwrap();
        Ok(labels.clone())
    }

    /// Add a dependency between issues
    pub fn add_dependency(&self, issue_id: &str, depends_on_id: &str) -> Result<()> {
        let mut issues = self.issues.lock().unwrap();

        // Verify both issues exist
        if !issues.contains_key(issue_id) {
            return Err(BeadsError::IssueNotFound(issue_id.to_string()));
        }
        if !issues.contains_key(depends_on_id) {
            return Err(BeadsError::IssueNotFound(depends_on_id.to_string()));
        }

        // Add dependency
        if let Some(issue) = issues.get_mut(issue_id) {
            if !issue.dependencies.contains(&depends_on_id.to_string()) {
                issue.dependencies.push(depends_on_id.to_string());
            }
        }

        // Add blocks to the dependency
        if let Some(dep_issue) = issues.get_mut(depends_on_id) {
            if !dep_issue.blocks.contains(&issue_id.to_string()) {
                dep_issue.blocks.push(issue_id.to_string());
            }
        }

        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(&self, issue_id: &str, depends_on_id: &str) -> Result<()> {
        let mut issues = self.issues.lock().unwrap();

        // Remove from dependencies
        if let Some(issue) = issues.get_mut(issue_id) {
            issue.dependencies.retain(|id| id != depends_on_id);
        }

        // Remove from blocks
        if let Some(dep_issue) = issues.get_mut(depends_on_id) {
            dep_issue.blocks.retain(|id| id != issue_id);
        }

        Ok(())
    }

    /// Clear all data (useful for test cleanup)
    pub fn clear(&self) {
        self.issues.lock().unwrap().clear();
        self.labels.lock().unwrap().clear();
        let mut stats = self.stats.lock().unwrap();
        *stats = IssueStats {
            total_issues: 0,
            open: 0,
            in_progress: 0,
            blocked: 0,
            closed: 0,
            ready_to_work: 0,
            avg_lead_time_hours: 0.0,
        };
        *self.next_id.lock().unwrap() = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_backend_creation() {
        let backend = MockBeadsBackend::new();
        let issues = backend.list_issues(None, None).unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_mock_backend_with_test_data() {
        let backend = MockBeadsBackend::with_test_data();
        let issues = backend.list_issues(None, None).unwrap();
        assert_eq!(issues.len(), 3);
    }

    #[test]
    fn test_create_issue() {
        let backend = MockBeadsBackend::new();
        let id = backend
            .create_issue("Test Issue", IssueType::Task, Priority::P2)
            .unwrap();
        assert!(id.starts_with("beads-mock-"));

        let issue = backend.get_issue(&id).unwrap();
        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.status, IssueStatus::Open);
    }

    #[test]
    fn test_update_issue() {
        let backend = MockBeadsBackend::with_test_data();
        backend
            .update_issue("beads-test-001", Some(IssueStatus::InProgress), None, None)
            .unwrap();

        let issue = backend.get_issue("beads-test-001").unwrap();
        assert_eq!(issue.status, IssueStatus::InProgress);
    }

    #[test]
    fn test_filter_by_status() {
        let backend = MockBeadsBackend::with_test_data();
        let open_issues = backend.list_issues(Some(IssueStatus::Open), None).unwrap();
        assert_eq!(open_issues.len(), 1);

        let closed_issues = backend
            .list_issues(Some(IssueStatus::Closed), None)
            .unwrap();
        assert_eq!(closed_issues.len(), 1);
    }

    #[test]
    fn test_dependencies() {
        let backend = MockBeadsBackend::with_test_data();
        backend
            .add_dependency("beads-test-001", "beads-test-002")
            .unwrap();

        let issue = backend.get_issue("beads-test-001").unwrap();
        assert!(issue.dependencies.contains(&"beads-test-002".to_string()));

        let dep_issue = backend.get_issue("beads-test-002").unwrap();
        assert!(dep_issue.blocks.contains(&"beads-test-001".to_string()));
    }
}
