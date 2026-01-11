//! Main application views

pub mod create_issue;
pub mod description_editor;
pub mod issue_detail;
pub mod issue_editor;

pub use create_issue::{CreateIssueData, CreateIssueForm, CreateIssueFormState};
pub use description_editor::{DescriptionEditorState, DescriptionEditorView, EditorMode};
pub use issue_detail::IssueDetailView;
pub use issue_editor::{IssueEditorState, IssueEditorView};

// Future views will be added here:
// - Issues view
// - Issue detail view
// - Dependencies view
// - Labels view
// - Database view
// - Help view
