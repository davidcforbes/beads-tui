use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Priority};
use beads_tui::beads::parser;
use chrono::Utc;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_issue_serialization_roundtrip(
        id in "[a-z0-9-]{10}",
        title in "\\PC*",
        description in proptest::option::of("\\PC*"),
        assignee in proptest::option::of("[a-z0-9_]+"),
    ) {
        let issue = Issue {
            id: id.clone(),
            title: title.clone(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: description.clone(),
            assignee: assignee.clone(),
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        };

        let json = serde_json::to_string(&issue).unwrap();
        let parsed = serde_json::from_str::<Issue>(&json).unwrap();

        assert_eq!(issue.id, parsed.id);
        assert_eq!(issue.title, parsed.title);
        assert_eq!(issue.assignee, parsed.assignee);
    }

    #[test]
    fn test_parser_does_not_panic_on_random_input(s in "\\PC*") {
        let _ = parser::parse_issue(&s);
        let _ = parser::parse_issue_list(&s);
        let _ = parser::parse_create_response(&s);
    }
}
