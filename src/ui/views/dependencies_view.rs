//! Dependencies view showing issue dependencies and blocks relationships

use crate::beads::models::{Issue, IssueStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

/// Get color for issue status
fn status_color(status: &IssueStatus) -> Color {
    match status {
        IssueStatus::Open => Color::Green,
        IssueStatus::InProgress => Color::Cyan,
        IssueStatus::Blocked => Color::Red,
        IssueStatus::Closed => Color::Gray,
    }
}

/// Dependencies view widget
pub struct DependenciesView<'a> {
    issue: Option<&'a Issue>,
    all_issues: Vec<&'a Issue>,
    block_style: Style,
}

impl<'a> DependenciesView<'a> {
    /// Create a new dependencies view
    pub fn new(all_issues: Vec<&'a Issue>) -> Self {
        Self {
            issue: None,
            all_issues,
            block_style: Style::default().fg(Color::Cyan),
        }
    }

    /// Set the selected issue to view dependencies for
    pub fn issue(mut self, issue: &'a Issue) -> Self {
        self.issue = Some(issue);
        self
    }

    /// Set block style
    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    fn render_no_selection(&self, area: Rect, buf: &mut Buffer) {
        let message = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No issue selected",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Select an issue from the Issues view to view its dependencies",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dependencies")
                .style(self.block_style),
        );

        message.render(area, buf);
    }

    fn render_dependencies(&self, area: Rect, buf: &mut Buffer, issue: &Issue) {
        // Create layout: issue info (3) + dependencies (fill) + blocks (fill) + help (1)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Issue info
                Constraint::Min(5),     // Dependencies
                Constraint::Min(5),     // Blocks
                Constraint::Length(1),  // Help
            ])
            .split(area);

        // Render issue info
        let info_text = format!("{} - {}", issue.id, issue.title);
        let info = Paragraph::new(info_text)
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Selected Issue")
                    .style(self.block_style),
            );
        info.render(chunks[0], buf);

        // Render dependencies (what this issue depends on)
        let dep_items: Vec<ListItem> = if issue.dependencies.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No dependencies",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )))]
        } else {
            issue
                .dependencies
                .iter()
                .filter_map(|dep_id| {
                    self.all_issues
                        .iter()
                        .find(|i| &i.id == dep_id)
                        .map(|dep_issue| {
                            ListItem::new(Line::from(vec![
                                Span::styled(&dep_issue.id, Style::default().fg(Color::Cyan)),
                                Span::raw(" - "),
                                Span::styled(
                                    &dep_issue.title,
                                    Style::default().fg(Color::White),
                                ),
                                Span::raw(" "),
                                Span::styled(
                                    format!("[{}]", dep_issue.status),
                                    Style::default().fg(status_color(&dep_issue.status)),
                                ),
                            ]))
                        })
                })
                .collect()
        };

        let dependencies = List::new(dep_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Depends On ({})", issue.dependencies.len()))
                .style(self.block_style),
        );
        dependencies.render(chunks[1], buf);

        // Render blocks (what this issue blocks)
        let block_items: Vec<ListItem> = if issue.blocks.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "Does not block any issues",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )))]
        } else {
            issue
                .blocks
                .iter()
                .filter_map(|blocked_id| {
                    self.all_issues
                        .iter()
                        .find(|i| &i.id == blocked_id)
                        .map(|blocked_issue| {
                            ListItem::new(Line::from(vec![
                                Span::styled(&blocked_issue.id, Style::default().fg(Color::Yellow)),
                                Span::raw(" - "),
                                Span::styled(
                                    &blocked_issue.title,
                                    Style::default().fg(Color::White),
                                ),
                                Span::raw(" "),
                                Span::styled(
                                    format!("[{}]", blocked_issue.status),
                                    Style::default().fg(status_color(&blocked_issue.status)),
                                ),
                            ]))
                        })
                })
                .collect()
        };

        let blocks = List::new(block_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Blocks ({})", issue.blocks.len()))
                .style(self.block_style),
        );
        blocks.render(chunks[2], buf);

        // Render help
        let help_text = "a: Add Dependency | d: Remove Dependency | g: Show Graph | c: Check Cycles | Esc: Back";
        let help = Paragraph::new(Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )));
        help.render(chunks[3], buf);
    }
}

impl<'a> Widget for DependenciesView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.issue {
            Some(issue) => self.render_dependencies(area, buf, issue),
            None => self.render_no_selection(area, buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            description: Some("Test description".to_string()),
            issue_type: IssueType::Task,
            status: IssueStatus::Open,
            priority: Priority::P2,
            labels: vec![],
            assignee: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies: vec![],
            blocks: vec![],
            notes: vec![],
        }
    }

    #[test]
    fn test_dependencies_view_creation() {
        let issues = vec![];
        let view = DependenciesView::new(issues);
        assert!(view.issue.is_none());
    }

    #[test]
    fn test_dependencies_view_with_issue() {
        let issue1 = create_test_issue("beads-001", "Issue 1");
        let issue2 = create_test_issue("beads-002", "Issue 2");
        let issues = vec![&issue1, &issue2];

        let view = DependenciesView::new(issues).issue(&issue1);
        assert!(view.issue.is_some());
    }

    #[test]
    fn test_dependencies_view_block_style() {
        let issues = vec![];
        let style = Style::default().fg(Color::Red);
        let view = DependenciesView::new(issues).block_style(style);
        assert_eq!(view.block_style, style);
    }
}
