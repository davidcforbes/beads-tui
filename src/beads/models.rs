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
