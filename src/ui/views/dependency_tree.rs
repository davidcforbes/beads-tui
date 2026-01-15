//! Hierarchical dependency tree view showing all issues with relationships
//!
//! This view displays a tree structure where:
//! - Top level: Issues that have dependencies or blocks
//! - Second level: "Dependencies" and "Blocks" sections
//! - Third level: Individual dependency/block items
//!
//! Users can navigate and expand/collapse nodes to explore relationships.

use crate::beads::models::{Issue, IssueStatus, Priority};
use crate::ui::widgets::tree::{TreeNode, TreeState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;
use std::fmt;

/// Node type in the dependency tree
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyNodeType {
    /// Top-level issue with relationships
    Issue(String),
    /// Section header for dependencies
    DependenciesSection(String),
    /// Section header for blocks
    BlocksSection(String),
    /// A specific dependency item
    DependencyItem {
        parent_id: String,
        dep_id: String,
    },
    /// A specific block item
    BlockItem {
        parent_id: String,
        blocked_id: String,
    },
}

/// Data for a node in the dependency tree
#[derive(Debug, Clone)]
pub struct DependencyNodeData {
    pub node_type: DependencyNodeType,
    pub display_text: String,
    pub issue: Option<Issue>,
}

impl fmt::Display for DependencyNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_text)
    }
}

/// State for the dependency tree view
#[derive(Debug)]
pub struct DependencyTreeState {
    tree_state: TreeState,
    /// Map of node IDs to issue IDs for quick lookup
    node_to_issue: HashMap<String, String>,
    /// Currently selected issue ID
    selected_issue_id: Option<String>,
}

impl Default for DependencyTreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyTreeState {
    pub fn new() -> Self {
        Self {
            tree_state: TreeState::new(),
            node_to_issue: HashMap::new(),
            selected_issue_id: None,
        }
    }

    /// Get the tree state
    pub fn tree_state_mut(&mut self) -> &mut TreeState {
        &mut self.tree_state
    }

    /// Get the currently selected node ID
    pub fn selected_node_id(&self) -> Option<&str> {
        self.tree_state.selected_id()
    }

    /// Get the currently selected issue ID
    pub fn selected_issue_id(&self) -> Option<&str> {
        self.selected_issue_id.as_deref()
    }

    /// Navigate to next item
    pub fn select_next(&mut self) {
        self.tree_state.select_next();
        self.update_selected_issue();
    }

    /// Navigate to previous item
    pub fn select_previous(&mut self) {
        self.tree_state.select_previous();
        self.update_selected_issue();
    }

    /// Toggle expansion of current node
    pub fn toggle_expansion(&mut self) {
        self.tree_state.toggle_selected();
    }

    /// Expand current node
    pub fn expand_current(&mut self) {
        if let Some(node_id) = self.selected_node_id().map(|s| s.to_string()) {
            self.tree_state.expand(&node_id);
        }
    }

    /// Collapse current node
    pub fn collapse_current(&mut self) {
        if let Some(node_id) = self.selected_node_id().map(|s| s.to_string()) {
            self.tree_state.collapse(&node_id);
        }
    }

    /// Update the selected issue ID based on current node
    fn update_selected_issue(&mut self) {
        if let Some(node_id) = self.selected_node_id() {
            self.selected_issue_id = self.node_to_issue.get(node_id).cloned();
        }
    }

    /// Build node ID to issue ID mapping
    pub fn build_node_mapping(&mut self, nodes: &[TreeNode<DependencyNodeData>]) {
        self.node_to_issue.clear();
        Self::build_mapping_recursive(nodes, &mut self.node_to_issue);
    }

    fn build_mapping_recursive(
        nodes: &[TreeNode<DependencyNodeData>],
        mapping: &mut HashMap<String, String>,
    ) {
        for node in nodes {
            match &node.data.node_type {
                DependencyNodeType::Issue(issue_id) => {
                    mapping.insert(node.id.clone(), issue_id.clone());
                }
                DependencyNodeType::DependencyItem { parent_id: _, dep_id } => {
                    mapping.insert(node.id.clone(), dep_id.clone());
                }
                DependencyNodeType::BlockItem {
                    parent_id: _,
                    blocked_id,
                } => {
                    mapping.insert(node.id.clone(), blocked_id.clone());
                }
                _ => {}
            }

            if !node.children.is_empty() {
                Self::build_mapping_recursive(&node.children, mapping);
            }
        }
    }
}

/// Dependency tree view widget
pub struct DependencyTreeView<'a> {
    issues: &'a [Issue],
    block_style: Style,
}

impl<'a> DependencyTreeView<'a> {
    pub fn new(issues: &'a [Issue]) -> Self {
        Self {
            issues,
            block_style: Style::default().fg(Color::Cyan),
        }
    }

    pub fn block_style(mut self, style: Style) -> Self {
        self.block_style = style;
        self
    }

    /// Build the tree structure from issues
    fn build_tree(&self) -> Vec<TreeNode<DependencyNodeData>> {
        let mut nodes = Vec::new();
        let issue_map: HashMap<String, &Issue> =
            self.issues.iter().map(|i| (i.id.clone(), i)).collect();

        // Find all issues that have dependencies or blocks
        for issue in self.issues {
            if !issue.dependencies.is_empty() || !issue.blocks.is_empty() {
                let issue_node = self.build_issue_node(issue, &issue_map);
                nodes.push(issue_node);
            }
        }

        nodes
    }

    /// Build a node for a single issue with its dependencies and blocks
    fn build_issue_node(
        &self,
        issue: &Issue,
        issue_map: &HashMap<String, &Issue>,
    ) -> TreeNode<DependencyNodeData> {
        let mut children = Vec::new();

        // Add dependencies section
        if !issue.dependencies.is_empty() {
            children.push(self.build_dependencies_section(issue, issue_map));
        }

        // Add blocks section
        if !issue.blocks.is_empty() {
            children.push(self.build_blocks_section(issue, issue_map));
        }

        // Format issue display with status indicators
        let status_indicator = self.get_status_indicator(&issue.status);
        let priority_str = format!("{:?}", issue.priority);
        let dep_count = issue.dependencies.len();
        let block_count = issue.blocks.len();

        let display_text = format!(
            "{} {} {} {} ↑{} ↓{}",
            issue.id, priority_str, issue.title, status_indicator, dep_count, block_count
        );

        TreeNode::with_children(
            issue.id.clone(),
            DependencyNodeData {
                node_type: DependencyNodeType::Issue(issue.id.clone()),
                display_text,
                issue: Some(issue.clone()),
            },
            children,
        )
    }

    /// Build the dependencies section node
    fn build_dependencies_section(
        &self,
        issue: &Issue,
        issue_map: &HashMap<String, &Issue>,
    ) -> TreeNode<DependencyNodeData> {
        let section_id = format!("{}-deps", issue.id);
        let count = issue.dependencies.len();

        let mut children = Vec::new();
        for (idx, dep_id) in issue.dependencies.iter().enumerate() {
            if let Some(dep_issue) = issue_map.get(dep_id) {
                let dep_display = format!(
                    "{} {} [{}]",
                    dep_issue.id,
                    dep_issue.title,
                    self.get_status_indicator(&dep_issue.status)
                );

                let dep_node = TreeNode::new(
                    format!("{}-dep-{}", issue.id, idx),
                    DependencyNodeData {
                        node_type: DependencyNodeType::DependencyItem {
                            parent_id: issue.id.clone(),
                            dep_id: dep_id.clone(),
                        },
                        display_text: dep_display,
                        issue: Some((*dep_issue).clone()),
                    },
                );
                children.push(dep_node);
            }
        }

        TreeNode::with_children(
            section_id,
            DependencyNodeData {
                node_type: DependencyNodeType::DependenciesSection(issue.id.clone()),
                display_text: format!("↑ Dependencies ({})", count),
                issue: None,
            },
            children,
        )
    }

    /// Build the blocks section node
    fn build_blocks_section(
        &self,
        issue: &Issue,
        issue_map: &HashMap<String, &Issue>,
    ) -> TreeNode<DependencyNodeData> {
        let section_id = format!("{}-blocks", issue.id);
        let count = issue.blocks.len();

        let mut children = Vec::new();
        for (idx, blocked_id) in issue.blocks.iter().enumerate() {
            if let Some(blocked_issue) = issue_map.get(blocked_id) {
                let blocked_display = format!(
                    "{} {} [{}]",
                    blocked_issue.id,
                    blocked_issue.title,
                    self.get_status_indicator(&blocked_issue.status)
                );

                let blocked_node = TreeNode::new(
                    format!("{}-block-{}", issue.id, idx),
                    DependencyNodeData {
                        node_type: DependencyNodeType::BlockItem {
                            parent_id: issue.id.clone(),
                            blocked_id: blocked_id.clone(),
                        },
                        display_text: blocked_display,
                        issue: Some((*blocked_issue).clone()),
                    },
                );
                children.push(blocked_node);
            }
        }

        TreeNode::with_children(
            section_id,
            DependencyNodeData {
                node_type: DependencyNodeType::BlocksSection(issue.id.clone()),
                display_text: format!("↓ Blocks ({})", count),
                issue: None,
            },
            children,
        )
    }

    /// Get status indicator symbol
    fn get_status_indicator(&self, status: &IssueStatus) -> &str {
        match status {
            IssueStatus::Open => "○",
            IssueStatus::InProgress => "●●●",
            IssueStatus::Blocked => "⊗",
            IssueStatus::Closed => "✓",
        }
    }

    /// Get status color
    fn get_status_color(&self, status: &IssueStatus) -> Color {
        match status {
            IssueStatus::Open => Color::Green,
            IssueStatus::InProgress => Color::Yellow,
            IssueStatus::Blocked => Color::Red,
            IssueStatus::Closed => Color::Gray,
        }
    }

    /// Get priority color
    fn get_priority_color(&self, priority: &Priority) -> Color {
        match priority {
            Priority::P0 => Color::Red,
            Priority::P1 => Color::LightRed,
            Priority::P2 => Color::Yellow,
            Priority::P3 => Color::Blue,
            Priority::P4 => Color::Gray,
        }
    }

    /// Render the tree with custom styling
    fn render_tree(
        &self,
        tree_nodes: &[TreeNode<DependencyNodeData>],
        area: Rect,
        buf: &mut Buffer,
        state: &mut DependencyTreeState,
    ) {
        // Build the tree widget
        use crate::ui::widgets::tree::Tree;

        let tree = Tree::new(tree_nodes)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Dependency Tree ({})", tree_nodes.len()))
                    .style(self.block_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
            .expanded_style(Style::default().fg(Color::Cyan))
            .collapsed_style(Style::default().fg(Color::DarkGray));

        StatefulWidget::render(tree, area, buf, &mut state.tree_state);
    }

    /// Render the quick info panel
    fn render_info_panel(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &DependencyTreeState,
    ) {
        let lines = if let Some(issue_id) = state.selected_issue_id() {
            // Find the issue
            if let Some(issue) = self.issues.iter().find(|i| i.id == issue_id) {
                vec![
                    Line::from(Span::styled("Selected:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled(&issue.id, Style::default().fg(Color::Yellow))),
                    Line::from(issue.title.as_str()),
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("Status: "),
                        Span::styled(
                            format!("{:?}", issue.status),
                            Style::default().fg(self.get_status_color(&issue.status)),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Priority: "),
                        Span::styled(
                            format!("{:?}", issue.priority),
                            Style::default().fg(self.get_priority_color(&issue.priority)),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(format!("Depends on: {}", issue.dependencies.len())),
                    Line::from(format!("Blocks: {}", issue.blocks.len())),
                    Line::from(""),
                    Line::from(Span::styled("Navigate:", Style::default().fg(Color::DarkGray))),
                    Line::from(Span::styled("j/k: Move", Style::default().fg(Color::DarkGray))),
                    Line::from(Span::styled("h/l: Collapse/Expand", Style::default().fg(Color::DarkGray))),
                    Line::from(Span::styled("Space: Toggle", Style::default().fg(Color::DarkGray))),
                    Line::from(Span::styled("Enter: View detail", Style::default().fg(Color::DarkGray))),
                ]
            } else {
                vec![Line::from("No issue selected")]
            }
        } else {
            vec![
                Line::from("No issue selected"),
                Line::from(""),
                Line::from(Span::styled("Navigate:", Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled("j/k or ↑/↓: Move", Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled("h/l or ←/→: Collapse/Expand", Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled("Space: Toggle", Style::default().fg(Color::DarkGray))),
            ]
        };

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Quick Info")
                .style(self.block_style),
        );

        paragraph.render(area, buf);
    }
}

impl<'a> StatefulWidget for DependencyTreeView<'a> {
    type State = DependencyTreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Split the area into tree and info panel
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Build the tree
        let tree_nodes = self.build_tree();

        // Build node mapping
        state.build_node_mapping(&tree_nodes);

        // Render tree
        self.render_tree(&tree_nodes, chunks[0], buf, state);

        // Render info panel
        self.render_info_panel(chunks[1], buf, state);

        // Update selected issue based on current node
        state.update_selected_issue();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueType};
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
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
    fn test_dependency_tree_state_new() {
        let state = DependencyTreeState::new();
        assert!(state.selected_node_id().is_none() || state.selected_node_id() == Some(""));
    }

    #[test]
    fn test_build_tree_empty() {
        let issues = vec![];
        let view = DependencyTreeView::new(&issues);
        let tree = view.build_tree();
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_build_tree_with_dependencies() {
        let mut issue1 = create_test_issue("issue-1", "First");
        let issue2 = create_test_issue("issue-2", "Second");
        issue1.dependencies.push("issue-2".to_string());

        let issues = vec![issue1, issue2];
        let view = DependencyTreeView::new(&issues);
        let tree = view.build_tree();

        assert_eq!(tree.len(), 1); // Only issue1 has dependencies
        assert!(tree[0].has_children());
    }

    #[test]
    fn test_build_tree_with_blocks() {
        let issue1 = create_test_issue("issue-1", "First");
        let mut issue2 = create_test_issue("issue-2", "Second");
        issue2.blocks.push("issue-1".to_string());

        let issues = vec![issue1, issue2];
        let view = DependencyTreeView::new(&issues);
        let tree = view.build_tree();

        assert_eq!(tree.len(), 1); // Only issue2 has blocks
    }

    #[test]
    fn test_status_indicators() {
        let view = DependencyTreeView::new(&[]);
        assert_eq!(view.get_status_indicator(&IssueStatus::Open), "○");
        assert_eq!(view.get_status_indicator(&IssueStatus::InProgress), "●●●");
        assert_eq!(view.get_status_indicator(&IssueStatus::Blocked), "⊗");
        assert_eq!(view.get_status_indicator(&IssueStatus::Closed), "✓");
    }

    #[test]
    fn test_node_mapping() {
        let mut issue1 = create_test_issue("issue-1", "First");
        let issue2 = create_test_issue("issue-2", "Second");
        issue1.dependencies.push("issue-2".to_string());

        let issues = vec![issue1, issue2];
        let view = DependencyTreeView::new(&issues);
        let tree = view.build_tree();

        let mut state = DependencyTreeState::new();
        state.build_node_mapping(&tree);

        assert!(state.node_to_issue.contains_key("issue-1"));
    }
}
