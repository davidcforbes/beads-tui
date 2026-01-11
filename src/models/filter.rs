/// Filter models for issue list filtering
use crate::beads::models::{IssueStatus, IssueType, Priority};

#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    pub status: Option<IssueStatus>,
    pub priority: Option<Priority>,
    pub issue_type: Option<IssueType>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub search_text: Option<String>,
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

#[derive(Debug, Clone)]
pub struct SavedFilter {
    pub name: String,
    pub filter: IssueFilter,
    pub hotkey: Option<char>, // F1-F12 mapped to chars
}
