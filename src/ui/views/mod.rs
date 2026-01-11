//! Main application views

pub mod create_issue;
pub mod issue_detail;

pub use create_issue::{CreateIssueData, CreateIssueForm, CreateIssueFormState};
pub use issue_detail::IssueDetailView;

// Future views will be added here:
// - Issues view
// - Issue detail view
// - Dependencies view
// - Labels view
// - Database view
// - Help view
