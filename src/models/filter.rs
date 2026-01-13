/// Filter models for issue list filtering
use crate::beads::models::{IssueStatus, IssueType, Priority};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogicOp {
    #[default]
    And,
    Or,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueFilter {
    pub status: Option<IssueStatus>,
    pub priority: Option<Priority>,
    pub issue_type: Option<IssueType>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub label_logic: LogicOp,
    pub search_text: Option<String>,
    pub use_regex: bool,
    pub use_fuzzy: bool,
}

impl IssueFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.priority.is_none()
            && self.issue_type.is_none()
            && self.assignee.is_none()
            && self.labels.is_empty()
            && self.search_text.is_none()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    pub name: String,
    pub filter: IssueFilter,
    pub hotkey: Option<char>, // F1-F12 mapped to chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_filter_new() {
        let filter = IssueFilter::new();
        assert!(filter.status.is_none());
        assert!(filter.priority.is_none());
        assert!(filter.issue_type.is_none());
        assert!(filter.assignee.is_none());
        assert!(filter.labels.is_empty());
        assert_eq!(filter.label_logic, LogicOp::And);
        assert!(filter.search_text.is_none());
        assert!(!filter.use_regex);
        assert!(!filter.use_fuzzy);
    }

    #[test]
    fn test_issue_filter_default() {
        let filter = IssueFilter::default();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_empty_when_new() {
        let filter = IssueFilter::new();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_status() {
        let mut filter = IssueFilter::new();
        filter.status = Some(IssueStatus::Open);
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_priority() {
        let mut filter = IssueFilter::new();
        filter.priority = Some(Priority::P1);
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_issue_type() {
        let mut filter = IssueFilter::new();
        filter.issue_type = Some(IssueType::Bug);
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_assignee() {
        let mut filter = IssueFilter::new();
        filter.assignee = Some("developer".to_string());
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_labels() {
        let mut filter = IssueFilter::new();
        filter.labels.push("bug".to_string());
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_is_not_empty_with_search_text() {
        let mut filter = IssueFilter::new();
        filter.search_text = Some("search term".to_string());
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_issue_filter_clear() {
        let mut filter = IssueFilter::new();
        filter.status = Some(IssueStatus::Closed);
        filter.priority = Some(Priority::P2);
        filter.issue_type = Some(IssueType::Feature);
        filter.assignee = Some("user".to_string());
        filter.labels.push("label1".to_string());
        filter.search_text = Some("query".to_string());

        assert!(!filter.is_empty());

        filter.clear();

        assert!(filter.is_empty());
        assert!(filter.status.is_none());
        assert!(filter.priority.is_none());
        assert!(filter.issue_type.is_none());
        assert!(filter.assignee.is_none());
        assert!(filter.labels.is_empty());
        assert!(filter.search_text.is_none());
    }

    #[test]
    fn test_saved_filter_creation() {
        let filter = IssueFilter::new();
        let saved = SavedFilter {
            name: "My Filter".to_string(),
            filter: filter.clone(),
            hotkey: Some('1'),
        };

        assert_eq!(saved.name, "My Filter");
        assert!(saved.filter.is_empty());
        assert_eq!(saved.hotkey, Some('1'));
    }

    #[test]
    fn test_saved_filter_with_configured_filter() {
        let mut filter = IssueFilter::new();
        filter.status = Some(IssueStatus::Open);
        filter.priority = Some(Priority::P1);

        let saved = SavedFilter {
            name: "High Priority Open".to_string(),
            filter: filter.clone(),
            hotkey: None,
        };

        assert_eq!(saved.name, "High Priority Open");
        assert!(!saved.filter.is_empty());
        assert_eq!(saved.filter.status, Some(IssueStatus::Open));
        assert_eq!(saved.filter.priority, Some(Priority::P1));
        assert!(saved.hotkey.is_none());
    }
}
