//! Dependency graph view with pan and zoom controls
//!
//! ## Known Limitations
//!
//! - **Performance**: Very wide graphs (>1000 nodes at same level) may cause slowdown
//!   - Layout algorithm is O(n²) for positioning nodes within layers
//!   - Rendering is optimized with viewport clipping
//!
//! - **Layout**: Node label truncation occurs when labels exceed node width (max ~50 chars)
//!   - Long labels show ellipsis (…) to indicate truncation
//!   - Hover tooltips could be added in future for full text
//!
//! - **Deep Hierarchies**: Very deep graphs (>200 levels) are supported but may require
//!   significant panning to navigate
//!   - Consider implementing minimap or overview panel for large graphs
//!
//! - **Zoom**: Zoom functionality is planned but not yet implemented
//!   - Current workaround: Use terminal font size adjustment
//!
//! - **Cycles**: Cyclic dependencies are detected and rendered without infinite loops
//!   - Cycle visualization shows all edges including back edges
//!   - Consider adding cycle highlighting in future
//!
//! ## Testing
//!
//! Comprehensive edge case tests in `tests/integration/dependency_edge_cases.rs` cover:
//! - Empty graphs, single nodes, disconnected components
//! - Very wide graphs (150+ children), deep hierarchies (120+ levels)
//! - Cyclic dependencies, self-referencing nodes
//! - Large labels, special characters, Unicode
//! - Viewport edge cases (tiny viewport, large offsets)

use crate::beads::models::Issue;
use crate::graph::{GraphLayout, GraphRenderer, LayoutOptions, RenderOptions};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use std::collections::HashMap;

/// State for the dependency graph view
#[derive(Debug, Clone)]
pub struct DependencyGraphState {
    /// Current viewport offset X
    offset_x: isize,
    /// Current viewport offset Y
    offset_y: isize,
    /// Selected node ID
    selected_node: Option<String>,
    /// Index of selected node in node list (for keyboard navigation)
    selected_index: usize,
    /// List of node IDs for navigation
    node_ids: Vec<String>,
    /// Zoom level (not yet implemented, for future enhancement)
    zoom: f32,
}

impl Default for DependencyGraphState {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraphState {
    /// Create a new dependency graph state
    pub fn new() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            selected_node: None,
            selected_index: 0,
            node_ids: Vec::new(),
            zoom: 1.0,
        }
    }

    /// Pan the viewport
    pub fn pan(&mut self, dx: isize, dy: isize) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    /// Pan left
    pub fn pan_left(&mut self, amount: isize) {
        self.offset_x -= amount;
    }

    /// Pan right
    pub fn pan_right(&mut self, amount: isize) {
        self.offset_x += amount;
    }

    /// Pan up
    pub fn pan_up(&mut self, amount: isize) {
        self.offset_y -= amount;
    }

    /// Pan down
    pub fn pan_down(&mut self, amount: isize) {
        self.offset_y += amount;
    }

    /// Reset view to origin
    pub fn reset_view(&mut self) {
        self.offset_x = 0;
        self.offset_y = 0;
    }

    /// Get current offset
    pub fn offset(&self) -> (isize, isize) {
        (self.offset_x, self.offset_y)
    }

    /// Set selected node
    pub fn select_node(&mut self, node_id: Option<String>) {
        self.selected_node = node_id.clone();
        if let Some(id) = node_id {
            if let Some(idx) = self.node_ids.iter().position(|n| n == &id) {
                self.selected_index = idx;
            }
        }
    }

    /// Get selected node
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_deref()
    }

    /// Select next node (for keyboard navigation)
    pub fn select_next(&mut self) {
        if !self.node_ids.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.node_ids.len();
            self.selected_node = Some(self.node_ids[self.selected_index].clone());
        }
    }

    /// Select previous node (for keyboard navigation)
    pub fn select_previous(&mut self) {
        if !self.node_ids.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.node_ids.len() - 1
            } else {
                self.selected_index - 1
            };
            self.selected_node = Some(self.node_ids[self.selected_index].clone());
        }
    }

    /// Update node IDs (called when issues change)
    pub fn update_nodes(&mut self, node_ids: Vec<String>) {
        self.node_ids = node_ids;
        if !self.node_ids.is_empty() && self.selected_node.is_none() {
            self.selected_index = 0;
            self.selected_node = Some(self.node_ids[0].clone());
        }
    }

    /// Center view on selected node
    pub fn center_on_selected(&mut self, layout: &GraphLayout, viewport_width: u16, viewport_height: u16) {
        if let Some(node_id) = &self.selected_node {
            if let Some(node) = layout.get_node(node_id) {
                // Calculate offset to center the node
                self.offset_x = -(node.x as isize) + (viewport_width as isize / 2) - (node.width as isize / 2);
                self.offset_y = -(node.y as isize) + (viewport_height as isize / 2) - (node.height as isize / 2);
            }
        }
    }
}

/// Widget for rendering the dependency graph
pub struct DependencyGraphView<'a> {
    /// Issues to display
    issues: &'a [Issue],
    /// Block for the widget
    block: Option<Block<'a>>,
    /// Whether to show help text
    show_help: bool,
}

impl<'a> DependencyGraphView<'a> {
    /// Create a new dependency graph view
    pub fn new(issues: &'a [Issue]) -> Self {
        Self {
            issues,
            block: None,
            show_help: true,
        }
    }

    /// Set the block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set whether to show help text
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    /// Build graph layout from issues
    fn build_layout(&self) -> GraphLayout {
        let mut nodes = HashMap::new();
        let mut dependencies = Vec::new();

        // Add all issues as nodes
        for issue in self.issues {
            nodes.insert(
                issue.id.clone(),
                format!("{} ({})", issue.id, issue.title),
            );
        }

        // Add dependencies as edges
        for issue in self.issues {
            for dep in &issue.dependencies {
                dependencies.push((issue.id.clone(), dep.clone()));
            }
            for blocked in &issue.blocks {
                dependencies.push((issue.id.clone(), blocked.clone()));
            }
        }

        GraphLayout::new(nodes, dependencies, LayoutOptions::default())
    }
}

impl<'a> StatefulWidget for DependencyGraphView<'a> {
    type State = DependencyGraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Apply block if provided
        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        // Reserve space for help text if enabled
        let (graph_area, help_area) = if self.show_help && inner_area.height > 3 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(inner_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (inner_area, None)
        };

        // Build layout
        let layout = self.build_layout();

        // Update state with current nodes
        let node_ids: Vec<String> = layout.nodes.iter().map(|n| n.id.clone()).collect();
        if state.node_ids != node_ids {
            state.update_nodes(node_ids);
        }

        // Create renderer
        let mut renderer = GraphRenderer::new(layout);
        renderer.select_node(state.selected_node.clone());

        // Render graph
        let render_options = RenderOptions {
            node_style: Style::default().fg(Color::Cyan),
            selected_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            edge_style: Style::default().fg(Color::DarkGray),
            offset_x: state.offset_x,
            offset_y: state.offset_y,
        };

        renderer.render(graph_area, buf, &render_options);

        // Render help text
        if let Some(help_area) = help_area {
            let help_text = if self.issues.is_empty() {
                "No issues to display"
            } else {
                "↑↓←→: Pan | Tab/Shift+Tab: Select node | C: Center on selected | R: Reset view | Q: Quit"
            };

            let help_line = Line::from(Span::styled(
                help_text,
                Style::default().fg(Color::DarkGray),
            ));
            let help_para = Paragraph::new(help_line);
            help_para.render(help_area, buf);
        }

        // Render status bar with offset info
        if graph_area.height > 0 && self.issues.len() > 0 {
            let status_y = graph_area.top();
            let status_text = format!(
                " Offset: ({}, {}) | Issues: {} | Selected: {} ",
                state.offset_x,
                state.offset_y,
                self.issues.len(),
                state.selected_node.as_deref().unwrap_or("None")
            );
            let status_line = Line::from(Span::styled(
                status_text,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ));
            if status_y < buf.area.bottom() {
                buf.set_line(graph_area.left(), status_y, &status_line, graph_area.width);
            }
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
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description: None,
            assignee: None,
            labels: Vec::new(),
            dependencies: Vec::new(),
            blocks: Vec::new(),
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: Vec::new(),
        }
    }

    #[test]
    fn test_graph_state_new() {
        let state = DependencyGraphState::new();
        assert_eq!(state.offset(), (0, 0));
        assert_eq!(state.selected_node(), None);
    }

    #[test]
    fn test_graph_state_pan() {
        let mut state = DependencyGraphState::new();

        state.pan(10, 20);
        assert_eq!(state.offset(), (10, 20));

        state.pan_left(5);
        assert_eq!(state.offset(), (5, 20));

        state.pan_right(10);
        assert_eq!(state.offset(), (15, 20));

        state.pan_up(10);
        assert_eq!(state.offset(), (15, 10));

        state.pan_down(5);
        assert_eq!(state.offset(), (15, 15));

        state.reset_view();
        assert_eq!(state.offset(), (0, 0));
    }

    #[test]
    fn test_graph_state_selection() {
        let mut state = DependencyGraphState::new();

        state.update_nodes(vec![
            "node1".to_string(),
            "node2".to_string(),
            "node3".to_string(),
        ]);

        assert_eq!(state.selected_node(), Some("node1"));

        state.select_next();
        assert_eq!(state.selected_node(), Some("node2"));

        state.select_next();
        assert_eq!(state.selected_node(), Some("node3"));

        state.select_next();
        assert_eq!(state.selected_node(), Some("node1")); // Wraps around

        state.select_previous();
        assert_eq!(state.selected_node(), Some("node3")); // Wraps around

        state.select_node(Some("node2".to_string()));
        assert_eq!(state.selected_node(), Some("node2"));

        state.select_node(None);
        assert_eq!(state.selected_node(), None);
    }

    #[test]
    fn test_graph_view_empty_issues() {
        let issues: Vec<Issue> = vec![];
        let view = DependencyGraphView::new(&issues);
        let mut state = DependencyGraphState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // Should not panic on empty issues
        view.render(area, &mut buf, &mut state);
    }

    #[test]
    fn test_graph_view_single_issue() {
        let issues = vec![create_test_issue("issue-1", "Test Issue")];
        let view = DependencyGraphView::new(&issues);
        let mut state = DependencyGraphState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf, &mut state);

        // State should have selected the first node
        assert_eq!(state.selected_node(), Some("issue-1"));
    }

    #[test]
    fn test_graph_view_with_dependencies() {
        let mut issue1 = create_test_issue("issue-1", "First Issue");
        let mut issue2 = create_test_issue("issue-2", "Second Issue");
        let issue3 = create_test_issue("issue-3", "Third Issue");

        issue1.dependencies.push("issue-2".to_string());
        issue2.blocks.push("issue-3".to_string());

        let issues = vec![issue1, issue2, issue3];
        let view = DependencyGraphView::new(&issues);
        let mut state = DependencyGraphState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        view.render(area, &mut buf, &mut state);

        // Should have all three nodes
        assert_eq!(state.node_ids.len(), 3);
    }

    #[test]
    fn test_center_on_selected() {
        let mut state = DependencyGraphState::new();
        state.update_nodes(vec!["A".to_string()]);

        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());

        state.center_on_selected(&layout, 80, 24);

        // Should have adjusted offset to center the node
        assert_ne!(state.offset(), (0, 0));
    }
}
