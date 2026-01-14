//! Issue detail view

use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
use crate::ui::widgets::{MarkdownViewer, MarkdownViewerState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidget, Widget},
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

    fn priority_description(priority: &Priority) -> &'static str {
        match priority {
            Priority::P0 => "Critical",
            Priority::P1 => "High",
            Priority::P2 => "Medium",
            Priority::P3 => "Low",
            Priority::P4 => "Backlog",
        }
    }

    fn type_symbol(issue_type: &IssueType) -> &'static str {
        match issue_type {
            IssueType::Bug => "🐛",
            IssueType::Feature => "✨",
            IssueType::Task => "📋",
            IssueType::Epic => "🎯",
            IssueType::Chore => "🔧",
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        use crate::ui::themes::Theme;

        let default_theme = Theme::default();
        let theme_ref = self.theme.unwrap_or(&default_theme);

        let status_symbol = Theme::status_symbol(&self.issue.status);
        let status_color = theme_ref.status_color(&self.issue.status);
        let priority_symbol = Theme::priority_symbol(&self.issue.priority);
        let priority_color = theme_ref.priority_color(&self.issue.priority);

        let header_lines = vec![
            Line::from(vec![
                Span::styled(Self::type_symbol(&self.issue.issue_type), Style::default()),
                Span::raw(" "),
                Span::styled(
                    &self.issue.id,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(
                    &self.issue.title,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} {:?}", status_symbol, self.issue.status),
                    Style::default().fg(status_color),
                ),
                Span::raw("  "),
                Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} {} ({})",
                        priority_symbol,
                        self.issue.priority,
                        Self::priority_description(&self.issue.priority)
                    ),
                    Style::default().fg(priority_color),
                ),
                Span::raw("  "),
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:?}", self.issue.issue_type), Style::default()),
            ]),
        ];

        let header = Paragraph::new(header_lines)
            .block(Block::default().borders(Borders::ALL).title("Issue"));

        header.render(area, buf);
    }

    fn render_description(&self, area: Rect, buf: &mut Buffer) {
        let description_text = if let Some(ref desc) = self.issue.description {
            desc.clone()
        } else {
            "No description provided.".to_string()
        };

        // Use markdown viewer for rich text rendering
        let mut markdown_state = MarkdownViewerState::new(description_text.clone());
        let markdown_viewer = MarkdownViewer::new()
            .block(Block::default().borders(Borders::ALL).title("Description"))
            .style(if self.issue.description.is_none() {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC)
            } else {
                Style::default()
            });

        StatefulWidget::render(markdown_viewer, area, buf, &mut markdown_state);
    }

    fn render_metadata(&self, area: Rect, buf: &mut Buffer) {
        use crate::ui::themes::Theme;

        let default_theme = Theme::default();
        let theme_ref = self.theme.unwrap_or(&default_theme);

        let status_symbol = Theme::status_symbol(&self.issue.status);
        let status_color = theme_ref.status_color(&self.issue.status);
        let priority_symbol = Theme::priority_symbol(&self.issue.priority);
        let priority_color = theme_ref.priority_color(&self.issue.priority);

        let mut metadata_lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
                Span::raw(&self.issue.id),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:?}", self.issue.issue_type)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} {:?}", status_symbol, self.issue.status),
                    Style::default().fg(status_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} {} ({})",
                        priority_symbol,
                        self.issue.priority,
                        Self::priority_description(&self.issue.priority)
                    ),
                    Style::default().fg(priority_color),
                ),
            ]),
        ];

        if let Some(ref assignee) = self.issue.assignee {
            metadata_lines.push(Line::from(vec![
                Span::styled("Assignee: ", Style::default().fg(Color::DarkGray)),
                Span::raw(assignee),
            ]));
        }

        if !self.issue.labels.is_empty() {
            metadata_lines.push(Line::from(vec![
                Span::styled("Labels: ", Style::default().fg(Color::DarkGray)),
                Span::raw(self.issue.labels.join(", ")),
            ]));
        }

        metadata_lines.push(Line::from("")); // Spacer

        metadata_lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
            Span::raw(self.issue.created.format("%Y-%m-%d %H:%M").to_string()),
        ]));

        metadata_lines.push(Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
            Span::raw(self.issue.updated.format("%Y-%m-%d %H:%M").to_string()),
        ]));

        if let Some(closed) = self.issue.closed {
            metadata_lines.push(Line::from(vec![
                Span::styled("Closed: ", Style::default().fg(Color::DarkGray)),
                Span::raw(closed.format("%Y-%m-%d %H:%M").to_string()),
            ]));
        }

        let metadata = Paragraph::new(metadata_lines)
            .block(Block::default().borders(Borders::ALL).title("Metadata"));

        metadata.render(area, buf);
    }

    fn render_dependencies(&self, area: Rect, buf: &mut Buffer) {
        let mut items = Vec::new();

        if !self.issue.dependencies.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Depends on:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))));
            for dep in &self.issue.dependencies {
                items.push(ListItem::new(format!("  → {dep}")));
            }
        }

        if !self.issue.blocks.is_empty() {
            if !items.is_empty() {
                items.push(ListItem::new("")); // Spacer
            }
            items.push(ListItem::new(Line::from(Span::styled(
                "Blocks:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))));
            for blocked in &self.issue.blocks {
                items.push(ListItem::new(format!("  ← {blocked}")));
            }
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "No dependencies",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))));
        }

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Dependencies"));

        Widget::render(list, area, buf);
    }

    fn render_notes(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = if self.issue.notes.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No notes",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            self.issue
                .notes
                .iter()
                .map(|note| {
                    let timestamp = note.timestamp.format("%Y-%m-%d %H:%M").to_string();
                    let author = &note.author;
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                            Span::raw(" - "),
                            Span::styled(author.clone(), Style::default().fg(Color::Cyan)),
                        ]),
                        Line::from(format!("  {}", note.content)),
                    ])
                })
                .collect()
        };

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Notes"));

        Widget::render(list, area, buf);
    }
}

impl<'a> Widget for IssueDetailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create main layout: header + body
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Header
                Constraint::Min(10),   // Body
            ])
            .split(area);

        // Render header
        self.render_header(main_chunks[0], buf);

        // Create body layout: left (description) + right (metadata + deps + notes)
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Description
                Constraint::Percentage(40), // Sidebar
            ])
            .split(main_chunks[1]);

        // Render description
        self.render_description(body_chunks[0], buf);

        // Create sidebar layout
        let mut sidebar_constraints = vec![Constraint::Length(12)]; // Metadata

        if self.show_dependencies {
            sidebar_constraints.push(Constraint::Min(8)); // Dependencies
        }

        if self.show_notes {
            sidebar_constraints.push(Constraint::Min(8)); // Notes
        }

        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(sidebar_constraints)
            .split(body_chunks[1]);

        // Render metadata
        self.render_metadata(sidebar_chunks[0], buf);

        // Render dependencies
        let mut chunk_index = 1;
        if self.show_dependencies && sidebar_chunks.len() > chunk_index {
            self.render_dependencies(sidebar_chunks[chunk_index], buf);
            chunk_index += 1;
        }

        // Render notes
        if self.show_notes && sidebar_chunks.len() > chunk_index {
            self.render_notes(sidebar_chunks[chunk_index], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_priority_color() {
        assert_eq!(IssueDetailView::priority_color(&Priority::P0), Color::Red);
        assert_eq!(
            IssueDetailView::priority_color(&Priority::P1),
            Color::LightRed
        );
        assert_eq!(
            IssueDetailView::priority_color(&Priority::P2),
            Color::Yellow
        );
        assert_eq!(IssueDetailView::priority_color(&Priority::P3), Color::Blue);
        assert_eq!(IssueDetailView::priority_color(&Priority::P4), Color::Gray);
    }

    #[test]
    fn test_status_color() {
        assert_eq!(
            IssueDetailView::status_color(&IssueStatus::Open),
            Color::Green
        );
        assert_eq!(
            IssueDetailView::status_color(&IssueStatus::InProgress),
            Color::Cyan
        );
        assert_eq!(
            IssueDetailView::status_color(&IssueStatus::Blocked),
            Color::Red
        );
        assert_eq!(
            IssueDetailView::status_color(&IssueStatus::Closed),
            Color::Gray
        );
    }

    #[test]
    fn test_type_symbol() {
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Bug), "🐛");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Feature), "✨");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Task), "📋");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Epic), "🎯");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Chore), "🔧");
    }

    #[test]
    fn test_priority_description() {
        assert_eq!(
            IssueDetailView::priority_description(&Priority::P0),
            "Critical"
        );
        assert_eq!(IssueDetailView::priority_description(&Priority::P1), "High");
        assert_eq!(
            IssueDetailView::priority_description(&Priority::P2),
            "Medium"
        );
        assert_eq!(IssueDetailView::priority_description(&Priority::P3), "Low");
        assert_eq!(
            IssueDetailView::priority_description(&Priority::P4),
            "Backlog"
        );
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
                timestamp: Utc::now(),
                author: "alice".to_string(),
                content: "First note".to_string(),
            },
            Note {
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
                timestamp: Utc::now(),
                author: "bob".to_string(),
                content: "Note content".to_string(),
            }],
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
                timestamp: Utc::now(),
                author: "alice".to_string(),
                content: "First note from alice".to_string(),
            },
            Note {
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
