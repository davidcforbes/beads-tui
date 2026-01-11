//! Issue detail view

use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};

/// Issue detail view widget
pub struct IssueDetailView<'a> {
    issue: &'a Issue,
    show_dependencies: bool,
    show_notes: bool,
}

impl<'a> IssueDetailView<'a> {
    /// Create a new issue detail view
    pub fn new(issue: &'a Issue) -> Self {
        Self {
            issue,
            show_dependencies: true,
            show_notes: true,
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

    fn priority_color(priority: &Priority) -> Color {
        match priority {
            Priority::P0 => Color::Red,
            Priority::P1 => Color::LightRed,
            Priority::P2 => Color::Yellow,
            Priority::P3 => Color::Blue,
            Priority::P4 => Color::Gray,
        }
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

    fn status_color(status: &IssueStatus) -> Color {
        match status {
            IssueStatus::Open => Color::Green,
            IssueStatus::InProgress => Color::Cyan,
            IssueStatus::Blocked => Color::Red,
            IssueStatus::Closed => Color::Gray,
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
        let header_lines = vec![
            Line::from(vec![
                Span::styled(
                    Self::type_symbol(&self.issue.issue_type),
                    Style::default(),
                ),
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
                    Style::default()
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:?}", self.issue.status),
                    Style::default().fg(Self::status_color(&self.issue.status)),
                ),
                Span::raw("  "),
                Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ({})", self.issue.priority, Self::priority_description(&self.issue.priority)),
                    Style::default().fg(Self::priority_color(&self.issue.priority)),
                ),
                Span::raw("  "),
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:?}", self.issue.issue_type),
                    Style::default(),
                ),
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

        let description = Paragraph::new(description_text)
            .block(Block::default().borders(Borders::ALL).title("Description"))
            .wrap(Wrap { trim: false })
            .style(if self.issue.description.is_none() {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            } else {
                Style::default()
            });

        description.render(area, buf);
    }

    fn render_metadata(&self, area: Rect, buf: &mut Buffer) {
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
                    format!("{:?}", self.issue.status),
                    Style::default().fg(Self::status_color(&self.issue.status)),
                ),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ({})", self.issue.priority, Self::priority_description(&self.issue.priority)),
                    Style::default().fg(Self::priority_color(&self.issue.priority)),
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))));
            for dep in &self.issue.dependencies {
                items.push(ListItem::new(format!("  → {}", dep)));
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
                items.push(ListItem::new(format!("  ← {}", blocked)));
            }
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "No dependencies",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ))));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Dependencies"));

        list.render(area, buf);
    }

    fn render_notes(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = if self.issue.notes.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No notes",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
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

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Notes"));

        list.render(area, buf);
    }
}

impl<'a> Widget for IssueDetailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create main layout: header + body
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Header
                Constraint::Min(10),    // Body
            ])
            .split(area);

        // Render header
        self.render_header(main_chunks[0], buf);

        // Create body layout: left (description) + right (metadata + deps + notes)
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Description
                Constraint::Percentage(40),  // Sidebar
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
        assert_eq!(IssueDetailView::priority_color(&Priority::P1), Color::LightRed);
        assert_eq!(IssueDetailView::priority_color(&Priority::P2), Color::Yellow);
        assert_eq!(IssueDetailView::priority_color(&Priority::P3), Color::Blue);
        assert_eq!(IssueDetailView::priority_color(&Priority::P4), Color::Gray);
    }

    #[test]
    fn test_status_color() {
        assert_eq!(IssueDetailView::status_color(&IssueStatus::Open), Color::Green);
        assert_eq!(IssueDetailView::status_color(&IssueStatus::InProgress), Color::Cyan);
        assert_eq!(IssueDetailView::status_color(&IssueStatus::Blocked), Color::Red);
        assert_eq!(IssueDetailView::status_color(&IssueStatus::Closed), Color::Gray);
    }

    #[test]
    fn test_type_symbol() {
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Bug), "🐛");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Feature), "✨");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Task), "📋");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Epic), "🎯");
        assert_eq!(IssueDetailView::type_symbol(&IssueType::Chore), "🔧");
    }
}
