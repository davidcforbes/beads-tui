/// Data models for beads issues and related structures
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: IssueStatus,
    pub priority: Priority,
    pub issue_type: IssueType,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub blocks: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(default)]
    pub closed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    Open,
    InProgress,
    Blocked,
    Closed,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Open => write!(f, "open"),
            IssueStatus::InProgress => write!(f, "in_progress"),
            IssueStatus::Blocked => write!(f, "blocked"),
            IssueStatus::Closed => write!(f, "closed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
            Priority::P4 => write!(f, "P4"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Epic,
    Feature,
    Task,
    Bug,
    Chore,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::Epic => write!(f, "epic"),
            IssueType::Feature => write!(f, "feature"),
            IssueType::Task => write!(f, "task"),
            IssueType::Bug => write!(f, "bug"),
            IssueType::Chore => write!(f, "chore"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStats {
    pub total_issues: usize,
    pub open: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub closed: usize,
    pub ready_to_work: usize,
    pub avg_lead_time_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    #[serde(default)]
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Blocks,
    DependsOn,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Blocks => write!(f, "blocks"),
            DependencyType::DependsOn => write!(f, "depends_on"),
        }
    }
}

/// Parameters for creating a new issue
#[derive(Debug, Clone)]
pub struct CreateIssueParams<'a> {
    pub title: &'a str,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub status: Option<&'a str>,
    pub assignee: Option<&'a str>,
    pub labels: &'a [String],
    pub description: Option<&'a str>,
}

impl<'a> CreateIssueParams<'a> {
    /// Create new params with required fields
    pub fn new(title: &'a str, issue_type: IssueType, priority: Priority) -> Self {
        Self {
            title,
            issue_type,
            priority,
            status: None,
            assignee: None,
            labels: &[],
            description: None,
        }
    }

    /// Set status
    pub fn with_status(mut self, status: &'a str) -> Self {
        self.status = Some(status);
        self
    }

    /// Set assignee
    pub fn with_assignee(mut self, assignee: &'a str) -> Self {
        self.assignee = Some(assignee);
        self
    }

    /// Set labels
    pub fn with_labels(mut self, labels: &'a [String]) -> Self {
        self.labels = labels;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // IssueStatus tests
    #[test]
    fn test_issue_status_display() {
        assert_eq!(IssueStatus::Open.to_string(), "open");
        assert_eq!(IssueStatus::InProgress.to_string(), "in_progress");
        assert_eq!(IssueStatus::Blocked.to_string(), "blocked");
        assert_eq!(IssueStatus::Closed.to_string(), "closed");
    }

    #[test]
    fn test_issue_status_ordering() {
        assert!(IssueStatus::Open < IssueStatus::InProgress);
        assert!(IssueStatus::InProgress < IssueStatus::Blocked);
        assert!(IssueStatus::Blocked < IssueStatus::Closed);
    }

    #[test]
    fn test_issue_status_equality() {
        assert_eq!(IssueStatus::Open, IssueStatus::Open);
        assert_ne!(IssueStatus::Open, IssueStatus::Closed);
    }

    #[test]
    fn test_issue_status_serialization() {
        let status = IssueStatus::Open;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"open\"");

        let status = IssueStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"inprogress\"");
    }

    #[test]
    fn test_issue_status_deserialization() {
        let status: IssueStatus = serde_json::from_str("\"open\"").unwrap();
        assert_eq!(status, IssueStatus::Open);

        let status: IssueStatus = serde_json::from_str("\"closed\"").unwrap();
        assert_eq!(status, IssueStatus::Closed);
    }

    // Priority tests
    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::P0.to_string(), "P0");
        assert_eq!(Priority::P1.to_string(), "P1");
        assert_eq!(Priority::P2.to_string(), "P2");
        assert_eq!(Priority::P3.to_string(), "P3");
        assert_eq!(Priority::P4.to_string(), "P4");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::P0 < Priority::P1);
        assert!(Priority::P1 < Priority::P2);
        assert!(Priority::P2 < Priority::P3);
        assert!(Priority::P3 < Priority::P4);
    }

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::P0, Priority::P0);
        assert_ne!(Priority::P0, Priority::P4);
    }

    #[test]
    fn test_priority_serialization() {
        let priority = Priority::P2;
        let json = serde_json::to_string(&priority).unwrap();
        assert!(json.contains("P2"));
    }

    // IssueType tests
    #[test]
    fn test_issue_type_display() {
        assert_eq!(IssueType::Epic.to_string(), "epic");
        assert_eq!(IssueType::Feature.to_string(), "feature");
        assert_eq!(IssueType::Task.to_string(), "task");
        assert_eq!(IssueType::Bug.to_string(), "bug");
        assert_eq!(IssueType::Chore.to_string(), "chore");
    }

    #[test]
    fn test_issue_type_ordering() {
        assert!(IssueType::Epic < IssueType::Feature);
        assert!(IssueType::Feature < IssueType::Task);
        assert!(IssueType::Task < IssueType::Bug);
        assert!(IssueType::Bug < IssueType::Chore);
    }

    #[test]
    fn test_issue_type_equality() {
        assert_eq!(IssueType::Bug, IssueType::Bug);
        assert_ne!(IssueType::Bug, IssueType::Feature);
    }

    #[test]
    fn test_issue_type_serialization() {
        let issue_type = IssueType::Feature;
        let json = serde_json::to_string(&issue_type).unwrap();
        assert_eq!(json, "\"feature\"");
    }

    #[test]
    fn test_issue_type_deserialization() {
        let issue_type: IssueType = serde_json::from_str("\"bug\"").unwrap();
        assert_eq!(issue_type, IssueType::Bug);

        let issue_type: IssueType = serde_json::from_str("\"epic\"").unwrap();
        assert_eq!(issue_type, IssueType::Epic);
    }

    // DependencyType tests
    #[test]
    fn test_dependency_type_display() {
        assert_eq!(DependencyType::Blocks.to_string(), "blocks");
        assert_eq!(DependencyType::DependsOn.to_string(), "depends_on");
    }

    #[test]
    fn test_dependency_type_equality() {
        assert_eq!(DependencyType::Blocks, DependencyType::Blocks);
        assert_ne!(DependencyType::Blocks, DependencyType::DependsOn);
    }

    #[test]
    fn test_dependency_type_serialization() {
        let dep_type = DependencyType::Blocks;
        let json = serde_json::to_string(&dep_type).unwrap();
        assert_eq!(json, "\"blocks\"");
    }

    #[test]
    fn test_dependency_type_deserialization() {
        let dep_type: DependencyType = serde_json::from_str("\"blocks\"").unwrap();
        assert_eq!(dep_type, DependencyType::Blocks);

        let dep_type: DependencyType = serde_json::from_str("\"dependson\"").unwrap();
        assert_eq!(dep_type, DependencyType::DependsOn);
    }

    // CreateIssueParams tests
    #[test]
    fn test_create_issue_params_new() {
        let params = CreateIssueParams::new("Test Issue", IssueType::Task, Priority::P2);
        assert_eq!(params.title, "Test Issue");
        assert_eq!(params.issue_type, IssueType::Task);
        assert_eq!(params.priority, Priority::P2);
        assert!(params.status.is_none());
        assert!(params.assignee.is_none());
        assert!(params.labels.is_empty());
        assert!(params.description.is_none());
    }

    #[test]
    fn test_create_issue_params_with_status() {
        let params = CreateIssueParams::new("Test", IssueType::Bug, Priority::P1)
            .with_status("in_progress");
        assert_eq!(params.status, Some("in_progress"));
    }

    #[test]
    fn test_create_issue_params_with_assignee() {
        let params = CreateIssueParams::new("Test", IssueType::Feature, Priority::P2)
            .with_assignee("developer");
        assert_eq!(params.assignee, Some("developer"));
    }

    #[test]
    fn test_create_issue_params_with_labels() {
        let labels = vec!["bug".to_string(), "urgent".to_string()];
        let params = CreateIssueParams::new("Test", IssueType::Bug, Priority::P0)
            .with_labels(&labels);
        assert_eq!(params.labels.len(), 2);
        assert_eq!(params.labels[0], "bug");
        assert_eq!(params.labels[1], "urgent");
    }

    #[test]
    fn test_create_issue_params_with_description() {
        let params = CreateIssueParams::new("Test", IssueType::Task, Priority::P3)
            .with_description("This is a test description");
        assert_eq!(params.description, Some("This is a test description"));
    }

    #[test]
    fn test_create_issue_params_builder_chain() {
        let labels = vec!["frontend".to_string()];
        let params = CreateIssueParams::new("Full Test", IssueType::Feature, Priority::P1)
            .with_status("open")
            .with_assignee("john")
            .with_labels(&labels)
            .with_description("Full description");

        assert_eq!(params.title, "Full Test");
        assert_eq!(params.issue_type, IssueType::Feature);
        assert_eq!(params.priority, Priority::P1);
        assert_eq!(params.status, Some("open"));
        assert_eq!(params.assignee, Some("john"));
        assert_eq!(params.labels.len(), 1);
        assert_eq!(params.description, Some("Full description"));
    }

    // Label tests
    #[test]
    fn test_label_creation() {
        let label = Label {
            name: "bug".to_string(),
            count: 5,
        };
        assert_eq!(label.name, "bug");
        assert_eq!(label.count, 5);
    }

    // Dependency tests
    #[test]
    fn test_dependency_creation() {
        let dep = Dependency {
            from: "beads-001".to_string(),
            to: "beads-002".to_string(),
            dependency_type: DependencyType::Blocks,
        };
        assert_eq!(dep.from, "beads-001");
        assert_eq!(dep.to, "beads-002");
        assert_eq!(dep.dependency_type, DependencyType::Blocks);
    }

    // IssueStats tests
    #[test]
    fn test_issue_stats_creation() {
        let stats = IssueStats {
            total_issues: 100,
            open: 30,
            in_progress: 20,
            blocked: 10,
            closed: 40,
            ready_to_work: 15,
            avg_lead_time_hours: 48.5,
        };
        assert_eq!(stats.total_issues, 100);
        assert_eq!(stats.open, 30);
        assert_eq!(stats.closed, 40);
        assert_eq!(stats.avg_lead_time_hours, 48.5);
    }
}
