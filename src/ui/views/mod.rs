//! Main application views

pub mod create_issue;
pub mod database_view;
pub mod description_editor;
pub mod help_view;
pub mod issue_detail;
pub mod issue_editor;
pub mod issues_view;
pub mod search_interface;

pub use create_issue::{CreateIssueData, CreateIssueForm, CreateIssueFormState};
pub use database_view::{DatabaseStats, DatabaseStatus, DatabaseView};
pub use description_editor::{DescriptionEditorState, DescriptionEditorView, EditorMode};
pub use help_view::{HelpSection, HelpView};
pub use issue_detail::IssueDetailView;
pub use issue_editor::{IssueEditorState, IssueEditorView};
pub use issues_view::{IssuesView, IssuesViewMode, IssuesViewState};
pub use search_interface::{SearchInterfaceState, SearchInterfaceView, SearchScope};

// Future views will be added here:
// - Dependencies view
// - Labels view
