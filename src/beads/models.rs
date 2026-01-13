/// Data models for beads issues and related structures
use chrono::{DateTime, Utc};
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;

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
    #[serde(rename = "created_at")]
    pub created: DateTime<Utc>,
    #[serde(rename = "updated_at")]
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

impl FromStr for IssueStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
            .collect::<String>();

        match normalized.as_str() {
            "open" => Ok(IssueStatus::Open),
            "inprogress" => Ok(IssueStatus::InProgress),
            "blocked" => Ok(IssueStatus::Blocked),
            "closed" => Ok(IssueStatus::Closed),
            _ => Err(format!("Invalid issue status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PriorityVisitor;

        impl<'de> Visitor<'de> for PriorityVisitor {
            type Value = Priority;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer 0-4 or string P0-P4")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Priority, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Priority::P0),
                    1 => Ok(Priority::P1),
                    2 => Ok(Priority::P2),
                    3 => Ok(Priority::P3),
                    4 => Ok(Priority::P4),
                    _ => Err(E::custom(format!("Invalid priority integer: {}", value))),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Priority, E>
            where
                E: de::Error,
            {
                match value {
                    "P0" | "p0" => Ok(Priority::P0),
                    "P1" | "p1" => Ok(Priority::P1),
                    "P2" | "p2" => Ok(Priority::P2),
                    "P3" | "p3" => Ok(Priority::P3),
                    "P4" | "p4" => Ok(Priority::P4),
                    _ => Err(E::custom(format!("Invalid priority string: {}", value))),
                }
            }
        }

        deserializer.deserialize_any(PriorityVisitor)
    }
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

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let normalized = trimmed
            .strip_prefix('p')
            .or_else(|| trimmed.strip_prefix('P'))
            .unwrap_or(trimmed);

        match normalized {
            "0" => Ok(Priority::P0),
            "1" => Ok(Priority::P1),
            "2" => Ok(Priority::P2),
            "3" => Ok(Priority::P3),
            "4" => Ok(Priority::P4),
            _ => Err(format!("Invalid priority: {s}")),
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

impl FromStr for IssueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "epic" => Ok(IssueType::Epic),
            "feature" => Ok(IssueType::Feature),
            "task" => Ok(IssueType::Task),
            "bug" => Ok(IssueType::Bug),
            "chore" => Ok(IssueType::Chore),
            _ => Err(format!("Invalid issue type: {s}")),
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
        let params =
            CreateIssueParams::new("Test", IssueType::Bug, Priority::P1).with_status("in_progress");
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
        let params =
            CreateIssueParams::new("Test", IssueType::Bug, Priority::P0).with_labels(&labels);
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

    // Clone trait tests
    #[test]
    fn test_issue_status_clone() {
        let status = IssueStatus::InProgress;
        let cloned = status; // Copy trait, not clone
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_issue_status_copy() {
        let status = IssueStatus::Blocked;
        let copied = status;
        assert_eq!(status, copied);
    }

    #[test]
    fn test_priority_clone() {
        let priority = Priority::P2;
        let cloned = priority; // Copy trait, not clone
        assert_eq!(priority, cloned);
    }

    #[test]
    fn test_priority_copy() {
        let priority = Priority::P3;
        let copied = priority;
        assert_eq!(priority, copied);
    }

    #[test]
    fn test_issue_type_clone() {
        let issue_type = IssueType::Feature;
        let cloned = issue_type; // Copy trait, not clone
        assert_eq!(issue_type, cloned);
    }

    #[test]
    fn test_issue_type_copy() {
        let issue_type = IssueType::Bug;
        let copied = issue_type;
        assert_eq!(issue_type, copied);
    }

    #[test]
    fn test_dependency_type_clone() {
        let dep_type = DependencyType::DependsOn;
        let cloned = dep_type;
        assert_eq!(dep_type, cloned);
    }

    #[test]
    fn test_dependency_type_copy() {
        let dep_type = DependencyType::Blocks;
        let copied = dep_type;
        assert_eq!(dep_type, copied);
    }

    #[test]
    fn test_note_clone() {
        let note = Note {
            timestamp: Utc::now(),
            author: "test_author".to_string(),
            content: "test content".to_string(),
        };
        let cloned = note.clone();
        assert_eq!(note.author, cloned.author);
        assert_eq!(note.content, cloned.content);
    }

    #[test]
    fn test_label_clone() {
        let label = Label {
            name: "urgent".to_string(),
            count: 10,
        };
        let cloned = label.clone();
        assert_eq!(label.name, cloned.name);
        assert_eq!(label.count, cloned.count);
    }

    #[test]
    fn test_dependency_clone() {
        let dep = Dependency {
            from: "beads-001".to_string(),
            to: "beads-002".to_string(),
            dependency_type: DependencyType::Blocks,
        };
        let cloned = dep.clone();
        assert_eq!(dep.from, cloned.from);
        assert_eq!(dep.to, cloned.to);
        assert_eq!(dep.dependency_type, cloned.dependency_type);
    }

    #[test]
    fn test_issue_stats_clone() {
        let stats = IssueStats {
            total_issues: 50,
            open: 15,
            in_progress: 10,
            blocked: 5,
            closed: 20,
            ready_to_work: 8,
            avg_lead_time_hours: 36.7,
        };
        let cloned = stats.clone();
        assert_eq!(stats.total_issues, cloned.total_issues);
        assert_eq!(stats.avg_lead_time_hours, cloned.avg_lead_time_hours);
    }

    #[test]
    fn test_create_issue_params_clone() {
        let params = CreateIssueParams::new("Test", IssueType::Task, Priority::P2);
        let cloned = params.clone();
        assert_eq!(params.title, cloned.title);
        assert_eq!(params.issue_type, cloned.issue_type);
        assert_eq!(params.priority, cloned.priority);
    }

    // Additional Priority tests
    #[test]
    fn test_priority_deserialization() {
        let priority: Priority = serde_json::from_str("\"P0\"").unwrap();
        assert_eq!(priority, Priority::P0);

        let priority: Priority = serde_json::from_str("\"P4\"").unwrap();
        assert_eq!(priority, Priority::P4);
    }

    // Full Issue roundtrip test
    #[test]
    fn test_issue_serialization_roundtrip() {
        let now = Utc::now();
        let issue = Issue {
            id: "beads-test".to_string(),
            title: "Test Issue".to_string(),
            status: IssueStatus::InProgress,
            priority: Priority::P1,
            issue_type: IssueType::Feature,
            description: Some("A test description".to_string()),
            assignee: Some("developer".to_string()),
            labels: vec!["frontend".to_string(), "urgent".to_string()],
            dependencies: vec!["beads-001".to_string()],
            blocks: vec!["beads-002".to_string()],
            created: now,
            updated: now,
            closed: None,
            notes: vec![],
        };

        let json = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&json).unwrap();

        assert_eq!(issue.id, deserialized.id);
        assert_eq!(issue.title, deserialized.title);
        assert_eq!(issue.status, deserialized.status);
        assert_eq!(issue.priority, deserialized.priority);
        assert_eq!(issue.issue_type, deserialized.issue_type);
        assert_eq!(issue.labels, deserialized.labels);
        assert_eq!(issue.dependencies, deserialized.dependencies);
        assert_eq!(issue.blocks, deserialized.blocks);
    }

    #[test]
    fn test_create_issue_params_empty_labels() {
        let empty_labels: Vec<String> = vec![];
        let params =
            CreateIssueParams::new("Test", IssueType::Bug, Priority::P0).with_labels(&empty_labels);
        assert!(params.labels.is_empty());
    }

    // All enum variants iteration tests
    #[test]
    fn test_all_issue_status_variants() {
        let variants = vec![
            IssueStatus::Open,
            IssueStatus::InProgress,
            IssueStatus::Blocked,
            IssueStatus::Closed,
        ];
        for variant in variants {
            let _display = variant.to_string();
            let json = serde_json::to_string(&variant).unwrap();
            let _deserialized: IssueStatus = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_all_priority_variants() {
        let variants = vec![
            Priority::P0,
            Priority::P1,
            Priority::P2,
            Priority::P3,
            Priority::P4,
        ];
        for variant in variants {
            let _display = variant.to_string();
            let json = serde_json::to_string(&variant).unwrap();
            let _deserialized: Priority = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_all_issue_type_variants() {
        let variants = vec![
            IssueType::Epic,
            IssueType::Feature,
            IssueType::Task,
            IssueType::Bug,
            IssueType::Chore,
        ];
        for variant in variants {
            let _display = variant.to_string();
            let json = serde_json::to_string(&variant).unwrap();
            let _deserialized: IssueType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_all_dependency_type_variants() {
        let variants = vec![DependencyType::Blocks, DependencyType::DependsOn];
        for variant in variants {
            let _display = variant.to_string();
            let json = serde_json::to_string(&variant).unwrap();
            let _deserialized: DependencyType = serde_json::from_str(&json).unwrap();
        }
    }

    // Debug trait tests
    #[test]
    fn test_issue_status_debug() {
        let status = IssueStatus::InProgress;
        let debug_str = format!("{:?}", status);
        assert_eq!(debug_str, "InProgress");
    }

    #[test]
    fn test_priority_debug() {
        let priority = Priority::P2;
        let debug_str = format!("{:?}", priority);
        assert_eq!(debug_str, "P2");
    }

    #[test]
    fn test_issue_type_debug() {
        let issue_type = IssueType::Feature;
        let debug_str = format!("{:?}", issue_type);
        assert_eq!(debug_str, "Feature");
    }

    #[test]
    fn test_dependency_type_debug() {
        let dep_type = DependencyType::Blocks;
        let debug_str = format!("{:?}", dep_type);
        assert_eq!(debug_str, "Blocks");
    }

    #[test]
    fn test_note_debug() {
        let note = Note {
            timestamp: Utc::now(),
            author: "author".to_string(),
            content: "content".to_string(),
        };
        let debug_str = format!("{:?}", note);
        assert!(debug_str.contains("Note"));
        assert!(debug_str.contains("author"));
        assert!(debug_str.contains("content"));
    }

    #[test]
    fn test_label_debug() {
        let label = Label {
            name: "bug".to_string(),
            count: 5,
        };
        let debug_str = format!("{:?}", label);
        assert!(debug_str.contains("Label"));
        assert!(debug_str.contains("bug"));
    }

    #[test]
    fn test_dependency_debug() {
        let dep = Dependency {
            from: "beads-001".to_string(),
            to: "beads-002".to_string(),
            dependency_type: DependencyType::Blocks,
        };
        let debug_str = format!("{:?}", dep);
        assert!(debug_str.contains("Dependency"));
        assert!(debug_str.contains("beads-001"));
        assert!(debug_str.contains("beads-002"));
    }

    #[test]
    fn test_issue_stats_debug() {
        let stats = IssueStats {
            total_issues: 100,
            open: 30,
            in_progress: 20,
            blocked: 10,
            closed: 40,
            ready_to_work: 15,
            avg_lead_time_hours: 48.5,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("IssueStats"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_create_issue_params_debug() {
        let params = CreateIssueParams::new("Test", IssueType::Task, Priority::P2);
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("CreateIssueParams"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_issue_debug() {
        let now = Utc::now();
        let issue = Issue {
            id: "beads-001".to_string(),
            title: "Test".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: now,
            updated: now,
            closed: None,
            notes: vec![],
        };
        let debug_str = format!("{:?}", issue);
        assert!(debug_str.contains("Issue"));
        assert!(debug_str.contains("beads-001"));
    }

    // Issue struct tests
    #[test]
    fn test_issue_creation_with_all_fields() {
        let now = Utc::now();
        let note = Note {
            timestamp: now,
            author: "author".to_string(),
            content: "note content".to_string(),
        };
        let issue = Issue {
            id: "beads-test".to_string(),
            title: "Test Issue".to_string(),
            status: IssueStatus::InProgress,
            priority: Priority::P1,
            issue_type: IssueType::Feature,
            description: Some("Description".to_string()),
            assignee: Some("developer".to_string()),
            labels: vec!["frontend".to_string(), "urgent".to_string()],
            dependencies: vec!["beads-001".to_string()],
            blocks: vec!["beads-002".to_string()],
            created: now,
            updated: now,
            closed: None,
            notes: vec![note],
        };

        assert_eq!(issue.id, "beads-test");
        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.status, IssueStatus::InProgress);
        assert_eq!(issue.priority, Priority::P1);
        assert_eq!(issue.issue_type, IssueType::Feature);
        assert_eq!(issue.description.as_deref(), Some("Description"));
        assert_eq!(issue.assignee.as_deref(), Some("developer"));
        assert_eq!(issue.labels.len(), 2);
        assert_eq!(issue.dependencies.len(), 1);
        assert_eq!(issue.blocks.len(), 1);
        assert_eq!(issue.notes.len(), 1);
        assert!(issue.closed.is_none());
    }

    #[test]
    fn test_issue_creation_minimal_fields() {
        let now = Utc::now();
        let issue = Issue {
            id: "beads-minimal".to_string(),
            title: "Minimal Issue".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P3,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: now,
            updated: now,
            closed: None,
            notes: vec![],
        };

        assert_eq!(issue.id, "beads-minimal");
        assert!(issue.description.is_none());
        assert!(issue.assignee.is_none());
        assert!(issue.labels.is_empty());
        assert!(issue.dependencies.is_empty());
        assert!(issue.blocks.is_empty());
        assert!(issue.notes.is_empty());
        assert!(issue.closed.is_none());
    }

    #[test]
    fn test_issue_with_closed_timestamp() {
        let now = Utc::now();
        let closed_time = now + chrono::Duration::hours(2);
        let issue = Issue {
            id: "beads-closed".to_string(),
            title: "Closed Issue".to_string(),
            status: IssueStatus::Closed,
            priority: Priority::P2,
            issue_type: IssueType::Bug,
            description: None,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: now,
            updated: closed_time,
            closed: Some(closed_time),
            notes: vec![],
        };

        assert_eq!(issue.status, IssueStatus::Closed);
        assert!(issue.closed.is_some());
        assert_eq!(issue.closed.unwrap(), closed_time);
    }

    #[test]
    fn test_note_creation() {
        let now = Utc::now();
        let note = Note {
            timestamp: now,
            author: "test_author".to_string(),
            content: "This is a test note".to_string(),
        };

        assert_eq!(note.author, "test_author");
        assert_eq!(note.content, "This is a test note");
        assert_eq!(note.timestamp, now);
    }

    #[test]
    fn test_label_default_count() {
        let label = Label {
            name: "feature".to_string(),
            count: 0,
        };
        assert_eq!(label.count, 0);
    }

    #[test]
    fn test_issue_stats_all_fields() {
        let stats = IssueStats {
            total_issues: 100,
            open: 25,
            in_progress: 15,
            blocked: 10,
            closed: 50,
            ready_to_work: 20,
            avg_lead_time_hours: 72.5,
        };

        assert_eq!(stats.total_issues, 100);
        assert_eq!(stats.open, 25);
        assert_eq!(stats.in_progress, 15);
        assert_eq!(stats.blocked, 10);
        assert_eq!(stats.closed, 50);
        assert_eq!(stats.ready_to_work, 20);
        assert_eq!(stats.avg_lead_time_hours, 72.5);
    }

    #[test]
    fn test_dependency_with_depends_on_type() {
        let dep = Dependency {
            from: "beads-A".to_string(),
            to: "beads-B".to_string(),
            dependency_type: DependencyType::DependsOn,
        };
        assert_eq!(dep.dependency_type, DependencyType::DependsOn);
        assert_eq!(dep.dependency_type.to_string(), "depends_on");
    }

    #[test]
    fn test_create_issue_params_multiple_labels() {
        let labels = vec![
            "bug".to_string(),
            "urgent".to_string(),
            "frontend".to_string(),
        ];
        let params =
            CreateIssueParams::new("Test", IssueType::Bug, Priority::P0).with_labels(&labels);
        assert_eq!(params.labels.len(), 3);
        assert_eq!(params.labels[2], "frontend");
    }

    #[test]
    fn test_issue_clone() {
        let now = Utc::now();
        let issue = Issue {
            id: "beads-clone".to_string(),
            title: "Clone Test".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: Some("Description".to_string()),
            assignee: None,
            labels: vec!["test".to_string()],
            dependencies: vec![],
            blocks: vec![],
            created: now,
            updated: now,
            closed: None,
            notes: vec![],
        };

        let cloned = issue.clone();
        assert_eq!(issue.id, cloned.id);
        assert_eq!(issue.title, cloned.title);
        assert_eq!(issue.status, cloned.status);
        assert_eq!(issue.labels, cloned.labels);
    }
}
