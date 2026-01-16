//! Issue detail view

use crate::beads::models::Issue;
use crate::ui::views::issue_form_builder::{build_issue_form_with_sections, IssueFormMode};
use crate::ui::widgets::{FormState, Form};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, StatefulWidget, Widget},
};

/// Issue detail view widget
pub struct IssueDetailView<'a> {
    issue: &'a Issue,
    show_dependencies: bool,
    show_notes: bool,
    theme: Option<&'a crate::ui::themes::Theme>,
}

impl<'a> IssueDetailView<'a> {
    /// Create a new issue detail view
    pub fn new(issue: &'a Issue) -> Self {
        Self {
            issue,
            show_dependencies: true,
            show_notes: true,
            theme: None,
        }
    }

    /// Show or hide dependencies section
    pub fn show_dependencies(mut self, show: bool) -> Self {
        self.show_dependencies = show;
        self
    }

    /// Show or hide notes section
    pub fn show_notes(mut self, show: bool) -> Self {
        self.show_notes = show;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: &'a crate::ui::themes::Theme) -> Self {
        self.theme = Some(theme);
        self
    }

}

impl<'a> StatefulWidget for IssueDetailView<'a> {
    type State = u16; // Scroll offset

    fn render(self, area: Rect, buf: &mut Buffer, scroll: &mut Self::State) {
        if area.width < 2 || area.height < 2 {
            return;
        }

        // Build form fields using sectioned form builder
        let mut fields = build_issue_form_with_sections(IssueFormMode::Read, Some(self.issue));

        // Filter fields based on show_dependencies and show_notes flags
        if !self.show_dependencies {
            fields.retain(|f| f.id != "dependencies" && f.id != "blocks");
        }
        if !self.show_notes {
            fields.retain(|f| f.id != "notes");
        }

        // Create form state and set scroll offset
        let mut form_state = FormState::new(fields);
        form_state.set_scroll_offset(*scroll as usize);

        let form = Form::new()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title("Record Details")
                    .style(Style::default().bg(Color::Black)),
            );

        StatefulWidget::render(form, area, buf, &mut form_state);

        // Update scroll offset back to state
        *scroll = form_state.scroll_offset() as u16;
    }
}

// Backward compatibility wrapper for Widget trait (renders with 0 scroll)
impl<'a> Widget for IssueDetailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut scroll = 0;
        StatefulWidget::render(self, area, buf, &mut scroll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue() -> Issue {
        Issue {
            id: "beads-001".to_string(),
            title: "Test Issue".to_string(),
            description: Some("This is a test issue".to_string()),
            issue_type: IssueType::Task,
            status: IssueStatus::Open,
            priority: Priority::P2,
            labels: vec!["test".to_string(), "demo".to_string()],
            assignee: Some("john".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec!["beads-000".to_string()],
            blocks: vec!["beads-002".to_string()],
            notes: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_issue_detail_view_creation() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue);
        assert!(view.show_dependencies);
        assert!(view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_toggle_sections() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(false);
        assert!(!view.show_dependencies);
        assert!(!view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_builder_chain() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(true);

        assert!(!view.show_dependencies);
        assert!(view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_with_no_assignee() {
        let mut issue = create_test_issue();
        issue.assignee = None;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.assignee, None);
    }

    #[test]
    fn test_issue_detail_view_with_no_labels() {
        let mut issue = create_test_issue();
        issue.labels = vec![];

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.labels.is_empty());
    }

    #[test]
    fn test_issue_detail_view_with_no_description() {
        let mut issue = create_test_issue();
        issue.description = None;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.description, None);
    }

    #[test]
    fn test_issue_detail_view_with_closed_issue() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::Closed;
        issue.closed = Some(Utc::now());

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.status, IssueStatus::Closed);
        assert!(view.issue.closed.is_some());
    }

    #[test]
    fn test_issue_detail_view_with_no_dependencies() {
        let mut issue = create_test_issue();
        issue.dependencies = vec![];
        issue.blocks = vec![];

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.dependencies.is_empty());
        assert!(view.issue.blocks.is_empty());
    }

    #[test]
    fn test_issue_detail_view_with_multiple_dependencies() {
        let mut issue = create_test_issue();
        issue.dependencies = vec![
            "beads-001".to_string(),
            "beads-002".to_string(),
            "beads-003".to_string(),
        ];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.dependencies.len(), 3);
    }

    #[test]
    fn test_issue_detail_view_with_multiple_blocks() {
        let mut issue = create_test_issue();
        issue.blocks = vec!["beads-004".to_string(), "beads-005".to_string()];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.blocks.len(), 2);
    }

    #[test]
    fn test_issue_detail_view_with_no_notes() {
        let issue = create_test_issue();

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.notes.is_empty());
    }

    #[test]
    fn test_issue_detail_view_with_notes() {
        use crate::beads::models::Note;

        let mut issue = create_test_issue();
        issue.notes = vec![
            Note {
                id: "test-note-1".to_string(),
                timestamp: Utc::now(),
                author: "alice".to_string(),
                content: "First note".to_string(),
            },
            Note {
                id: "test-note-2".to_string(),
                timestamp: Utc::now(),
                author: "bob".to_string(),
                content: "Second note".to_string(),
            },
        ];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.notes.len(), 2);
        assert_eq!(view.issue.notes[0].author, "alice");
        assert_eq!(view.issue.notes[1].author, "bob");
    }

    #[test]
    fn test_issue_detail_view_hide_dependencies_section() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue).show_dependencies(false);

        assert!(!view.show_dependencies);
        assert!(view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_hide_notes_section() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue).show_notes(false);

        assert!(view.show_dependencies);
        assert!(!view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_hide_both_sections() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(false);

        assert!(!view.show_dependencies);
        assert!(!view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_with_epic_type() {
        let mut issue = create_test_issue();
        issue.issue_type = IssueType::Epic;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.issue_type, IssueType::Epic);
    }

    #[test]
    fn test_issue_detail_view_with_bug_type() {
        let mut issue = create_test_issue();
        issue.issue_type = IssueType::Bug;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.issue_type, IssueType::Bug);
    }

    #[test]
    fn test_issue_detail_view_with_feature_type() {
        let mut issue = create_test_issue();
        issue.issue_type = IssueType::Feature;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.issue_type, IssueType::Feature);
    }

    #[test]
    fn test_issue_detail_view_with_blocked_status() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::Blocked;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.status, IssueStatus::Blocked);
    }

    #[test]
    fn test_issue_detail_view_with_in_progress_status() {
        let mut issue = create_test_issue();
        issue.status = IssueStatus::InProgress;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.status, IssueStatus::InProgress);
    }

    #[test]
    fn test_issue_detail_view_with_p0_priority() {
        let mut issue = create_test_issue();
        issue.priority = Priority::P0;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.priority, Priority::P0);
    }

    #[test]
    fn test_issue_detail_view_with_p4_priority() {
        let mut issue = create_test_issue();
        issue.priority = Priority::P4;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.priority, Priority::P4);
    }

    #[test]
    fn test_issue_detail_view_with_multiple_labels() {
        let mut issue = create_test_issue();
        issue.labels = vec![
            "bug".to_string(),
            "urgent".to_string(),
            "security".to_string(),
            "p0".to_string(),
        ];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.labels.len(), 4);
        assert!(view.issue.labels.contains(&"security".to_string()));
    }

    #[test]
    fn test_issue_detail_view_defaults_show_all_sections() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue);

        assert!(view.show_dependencies);
        assert!(view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_show_dependencies_true() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue).show_dependencies(true);

        assert!(view.show_dependencies);
    }

    #[test]
    fn test_issue_detail_view_show_notes_true() {
        let issue = create_test_issue();
        let view = IssueDetailView::new(&issue).show_notes(true);

        assert!(view.show_notes);
    }

    #[test]
    fn test_issue_detail_view_with_chore_type() {
        let mut issue = create_test_issue();
        issue.issue_type = IssueType::Chore;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.issue_type, IssueType::Chore);
    }

    #[test]
    fn test_issue_detail_view_with_p1_priority() {
        let mut issue = create_test_issue();
        issue.priority = Priority::P1;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.priority, Priority::P1);
    }

    #[test]
    fn test_issue_detail_view_with_p3_priority() {
        let mut issue = create_test_issue();
        issue.priority = Priority::P3;

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.priority, Priority::P3);
    }

    #[test]
    fn test_builder_chain_all_combinations() {
        let issue = create_test_issue();

        // Test all four combinations
        let view1 = IssueDetailView::new(&issue)
            .show_dependencies(true)
            .show_notes(true);
        assert!(view1.show_dependencies && view1.show_notes);

        let view2 = IssueDetailView::new(&issue)
            .show_dependencies(true)
            .show_notes(false);
        assert!(view2.show_dependencies && !view2.show_notes);

        let view3 = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(true);
        assert!(!view3.show_dependencies && view3.show_notes);

        let view4 = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(false);
        assert!(!view4.show_dependencies && !view4.show_notes);
    }

    #[test]
    fn test_issue_with_only_dependencies_no_blocks() {
        let mut issue = create_test_issue();
        issue.dependencies = vec!["beads-001".to_string(), "beads-002".to_string()];
        issue.blocks = vec![];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.dependencies.len(), 2);
        assert!(view.issue.blocks.is_empty());
    }

    #[test]
    fn test_issue_with_only_blocks_no_dependencies() {
        let mut issue = create_test_issue();
        issue.dependencies = vec![];
        issue.blocks = vec!["beads-003".to_string()];

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.dependencies.is_empty());
        assert_eq!(view.issue.blocks.len(), 1);
    }

    #[test]
    fn test_issue_with_single_dependency() {
        let mut issue = create_test_issue();
        issue.dependencies = vec!["beads-001".to_string()];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.dependencies.len(), 1);
        assert_eq!(view.issue.dependencies[0], "beads-001");
    }

    #[test]
    fn test_issue_with_single_block() {
        let mut issue = create_test_issue();
        issue.blocks = vec!["beads-002".to_string()];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.blocks.len(), 1);
        assert_eq!(view.issue.blocks[0], "beads-002");
    }

    #[test]
    fn test_issue_with_single_note() {
        use crate::beads::models::Note;

        let mut issue = create_test_issue();
        issue.notes = vec![Note {
            id: "test-note-1".to_string(),
            timestamp: Utc::now(),
            author: "alice".to_string(),
            content: "Single note".to_string(),
        }];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.notes.len(), 1);
        assert_eq!(view.issue.notes[0].author, "alice");
        assert_eq!(view.issue.notes[0].content, "Single note");
    }

    #[test]
    fn test_issue_with_single_label() {
        let mut issue = create_test_issue();
        issue.labels = vec!["urgent".to_string()];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.labels.len(), 1);
        assert_eq!(view.issue.labels[0], "urgent");
    }

    #[test]
    fn test_issue_minimal_fields() {
        let mut issue = create_test_issue();
        issue.description = None;
        issue.assignee = None;
        issue.labels = vec![];
        issue.dependencies = vec![];
        issue.blocks = vec![];
        issue.notes = vec![];
        issue.closed = None;

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.description.is_none());
        assert!(view.issue.assignee.is_none());
        assert!(view.issue.labels.is_empty());
        assert!(view.issue.dependencies.is_empty());
        assert!(view.issue.blocks.is_empty());
        assert!(view.issue.notes.is_empty());
        assert!(view.issue.closed.is_none());
    }

    #[test]
    fn test_issue_all_fields_populated() {
        use crate::beads::models::Note;

        let issue = Issue {
            id: "beads-999".to_string(),
            title: "Full Issue".to_string(),
            description: Some("Comprehensive description".to_string()),
            issue_type: IssueType::Bug,
            status: IssueStatus::InProgress,
            priority: Priority::P0,
            labels: vec!["urgent".to_string(), "security".to_string()],
            assignee: Some("alice".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
            closed: Some(Utc::now()),
            dependencies: vec!["beads-001".to_string()],
            blocks: vec!["beads-002".to_string()],
            notes: vec![Note {
                id: "test-note-1".to_string(),
                timestamp: Utc::now(),
                author: "bob".to_string(),
                content: "Note content".to_string(),
            }],
            ..Default::default()
        };

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.description.is_some());
        assert!(view.issue.assignee.is_some());
        assert!(!view.issue.labels.is_empty());
        assert!(!view.issue.dependencies.is_empty());
        assert!(!view.issue.blocks.is_empty());
        assert!(!view.issue.notes.is_empty());
        assert!(view.issue.closed.is_some());
    }

    #[test]
    fn test_notes_with_same_author() {
        use crate::beads::models::Note;

        let mut issue = create_test_issue();
        issue.notes = vec![
            Note {
                id: "test-note-1".to_string(),
                timestamp: Utc::now(),
                author: "alice".to_string(),
                content: "First note from alice".to_string(),
            },
            Note {
                id: "test-note-2".to_string(),
                timestamp: Utc::now(),
                author: "alice".to_string(),
                content: "Second note from alice".to_string(),
            },
        ];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.notes.len(), 2);
        assert_eq!(view.issue.notes[0].author, view.issue.notes[1].author);
    }

    #[test]
    fn test_builder_methods_independent() {
        let issue = create_test_issue();

        // Test that builder methods are independent
        let view1 = IssueDetailView::new(&issue).show_dependencies(false);
        assert!(!view1.show_dependencies);
        assert!(view1.show_notes); // Should remain default

        let view2 = IssueDetailView::new(&issue).show_notes(false);
        assert!(view2.show_dependencies); // Should remain default
        assert!(!view2.show_notes);
    }

    #[test]
    fn test_multiple_dependencies_and_blocks() {
        let mut issue = create_test_issue();
        issue.dependencies = vec![
            "beads-001".to_string(),
            "beads-002".to_string(),
            "beads-003".to_string(),
        ];
        issue.blocks = vec!["beads-004".to_string(), "beads-005".to_string()];

        let view = IssueDetailView::new(&issue);
        assert_eq!(view.issue.dependencies.len(), 3);
        assert_eq!(view.issue.blocks.len(), 2);
    }

    #[test]
    fn test_issue_with_long_description() {
        let mut issue = create_test_issue();
        issue.description = Some("This is a very long description that contains multiple sentences. It should test how the view handles longer text content. The description can span multiple lines and include various formatting.".to_string());

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.description.is_some());
        assert!(view.issue.description.as_ref().unwrap().len() > 100);
    }

    #[test]
    fn test_issue_with_empty_string_description() {
        let mut issue = create_test_issue();
        issue.description = Some("".to_string());

        let view = IssueDetailView::new(&issue);
        assert!(view.issue.description.is_some());
        assert_eq!(view.issue.description.as_ref().unwrap(), "");
    }

    #[test]
    fn test_builder_method_chaining_order() {
        let issue = create_test_issue();

        // Test that order doesn't matter
        let view1 = IssueDetailView::new(&issue)
            .show_dependencies(false)
            .show_notes(true);

        let view2 = IssueDetailView::new(&issue)
            .show_notes(true)
            .show_dependencies(false);

        assert_eq!(view1.show_dependencies, view2.show_dependencies);
        assert_eq!(view1.show_notes, view2.show_notes);
    }
}
