//! PERT chart widget for visualizing dependency graphs and critical paths

use crate::models::pert_layout::{PertEdge, PertGraph, PertNode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

/// PERT chart rendering configuration
#[derive(Debug, Clone)]
pub struct PertChartConfig {
    /// Viewport offset X (for panning)
    pub offset_x: i32,
    /// Viewport offset Y (for panning)
    pub offset_y: i32,
    /// Zoom level (multiplier for node spacing)
    pub zoom: f32,
    /// Show critical path highlighting
    pub show_critical_path: bool,
    /// Currently selected node ID
    pub selected_node: Option<String>,
    /// Style for normal nodes
    pub normal_style: Style,
    /// Style for selected nodes
    pub selected_style: Style,
    /// Style for critical path nodes
    pub critical_style: Style,
    /// Style for edges
    pub edge_style: Style,
    /// Style for critical path edges
    pub critical_edge_style: Style,
    /// Node box width
    pub node_width: u16,
    /// Node box height
    pub node_height: u16,
}

impl Default for PertChartConfig {
    fn default() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            zoom: 1.0,
            show_critical_path: true,
            selected_node: None,
            normal_style: Style::default().fg(Color::Cyan),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            critical_style: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            edge_style: Style::default().fg(Color::Gray),
            critical_edge_style: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            node_width: 20,
            node_height: 5,
        }
    }
}

impl PertChartConfig {
    /// Create a new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set pan offset
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Pan by delta
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    /// Set zoom level
    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.5).min(3.0);
        self
    }

    /// Adjust zoom by factor
    pub fn adjust_zoom(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).max(0.5).min(3.0);
    }

    /// Toggle critical path highlighting
    pub fn toggle_critical_path(&mut self) {
        self.show_critical_path = !self.show_critical_path;
    }

    /// Set selected node
    pub fn select_node(&mut self, node_id: Option<String>) {
        self.selected_node = node_id;
    }

    /// Select next node in topological order
    pub fn select_next(&mut self, graph: &PertGraph) {
        if let Some(current) = &self.selected_node {
            let order = &graph.topological_order;
            if let Some(idx) = order.iter().position(|id| id == current) {
                if idx + 1 < order.len() {
                    self.selected_node = Some(order[idx + 1].clone());
                }
            }
        } else if !graph.topological_order.is_empty() {
            self.selected_node = Some(graph.topological_order[0].clone());
        }
    }

    /// Select previous node in topological order
    pub fn select_prev(&mut self, graph: &PertGraph) {
        if let Some(current) = &self.selected_node {
            let order = &graph.topological_order;
            if let Some(idx) = order.iter().position(|id| id == current) {
                if idx > 0 {
                    self.selected_node = Some(order[idx - 1].clone());
                }
            }
        } else if !graph.topological_order.is_empty() {
            self.selected_node = Some(graph.topological_order.last().unwrap().clone());
        }
    }

    /// Select adjacent node (for arrow key navigation)
    pub fn select_adjacent(&mut self, graph: &PertGraph, direction: Direction) {
        if let Some(current) = &self.selected_node {
            if let Some(current_node) = graph.nodes.get(current) {
                // Find nodes in the direction
                let candidates: Vec<(&String, &PertNode)> = graph
                    .nodes
                    .iter()
                    .filter(|(id, node)| {
                        *id != current
                            && match direction {
                                Direction::Up => node.y < current_node.y,
                                Direction::Down => node.y > current_node.y,
                                Direction::Left => node.x < current_node.x,
                                Direction::Right => node.x > current_node.x,
                            }
                    })
                    .collect();

                // Find closest candidate
                if let Some(closest) = candidates.iter().min_by_key(|(_, node)| {
                    let dx = (node.x as i32 - current_node.x as i32).abs();
                    let dy = (node.y as i32 - current_node.y as i32).abs();
                    dx + dy // Manhattan distance
                }) {
                    self.selected_node = Some(closest.0.clone());
                }
            }
        } else if !graph.nodes.is_empty() {
            // Select first node if none selected
            self.selected_node = graph.topological_order.first().cloned();
        }
    }
}

/// Navigation direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// PERT chart widget
#[derive(Debug, Clone)]
pub struct PertChart<'a> {
    /// The PERT graph to render
    graph: &'a PertGraph,
    /// Rendering configuration
    config: PertChartConfig,
}

impl<'a> PertChart<'a> {
    /// Create a new PERT chart widget
    pub fn new(graph: &'a PertGraph) -> Self {
        Self {
            graph,
            config: PertChartConfig::default(),
        }
    }

    /// Set configuration
    pub fn config(mut self, config: PertChartConfig) -> Self {
        self.config = config;
        self
    }

    /// Apply zoom to coordinate
    fn apply_zoom(&self, coord: u16) -> u16 {
        ((coord as f32) * self.config.zoom) as u16
    }

    /// Transform node coordinates to viewport coordinates
    fn transform_position(&self, x: u16, y: u16) -> (i32, i32) {
        let zoomed_x = self.apply_zoom(x);
        let zoomed_y = self.apply_zoom(y);
        let viewport_x = zoomed_x as i32 - self.config.offset_x;
        let viewport_y = zoomed_y as i32 - self.config.offset_y;
        (viewport_x, viewport_y)
    }

    /// Check if a position is visible in the viewport
    fn is_visible(&self, x: i32, y: i32, area: &Rect) -> bool {
        x >= 0
            && y >= 0
            && (x as u16) < area.width
            && (y as u16) < area.height
    }

    /// Render a node box
    fn render_node(&self, node: &PertNode, area: Rect, buf: &mut Buffer) {
        let (vx, vy) = self.transform_position(node.x, node.y);

        // Check if node is visible
        if !self.is_visible(vx, vy, &area) {
            return;
        }

        let x = area.x + vx as u16;
        let y = area.y + vy as u16;

        // Determine node style
        let is_selected = self
            .config
            .selected_node
            .as_ref()
            .map_or(false, |id| id == &node.issue_id);
        let is_critical = node.is_critical && self.config.show_critical_path;

        let style = if is_selected {
            self.config.selected_style
        } else if is_critical {
            self.config.critical_style
        } else {
            self.config.normal_style
        };

        // Render box borders
        let width = self.config.node_width.min(area.width.saturating_sub(vx as u16));
        let height = self.config.node_height.min(area.height.saturating_sub(vy as u16));

        if width < 4 || height < 3 {
            return; // Too small to render
        }

        // Top border
        if y < area.y + area.height {
            buf.set_string(x, y, "┌", style);
            for dx in 1..width - 1 {
                if x + dx < area.x + area.width {
                    buf.set_string(x + dx, y, "─", style);
                }
            }
            if x + width - 1 < area.x + area.width {
                buf.set_string(x + width - 1, y, "┐", style);
            }
        }

        // Middle rows with content
        for dy in 1..height - 1 {
            if y + dy >= area.y + area.height {
                break;
            }

            // Left border
            buf.set_string(x, y + dy, "│", style);

            // Content
            let content = match dy {
                1 => {
                    // Line 1: Issue ID
                    truncate_string(&node.issue_id, (width - 2) as usize)
                }
                2 => {
                    // Line 2: Title (truncated)
                    truncate_string(&node.title, (width - 2) as usize)
                }
                3 => {
                    // Line 3: Timing info
                    format!("ES:{:.1}", node.earliest_start)
                }
                _ => String::new(),
            };

            if !content.is_empty() {
                buf.set_string(x + 1, y + dy, &content, style);
            }

            // Right border
            if x + width - 1 < area.x + area.width {
                buf.set_string(x + width - 1, y + dy, "│", style);
            }
        }

        // Bottom border
        if y + height - 1 < area.y + area.height {
            buf.set_string(x, y + height - 1, "└", style);
            for dx in 1..width - 1 {
                if x + dx < area.x + area.width {
                    buf.set_string(x + dx, y + height - 1, "─", style);
                }
            }
            if x + width - 1 < area.x + area.width {
                buf.set_string(x + width - 1, y + height - 1, "┘", style);
            }
        }
    }

    /// Render an edge between two nodes
    fn render_edge(&self, edge: &PertEdge, area: Rect, buf: &mut Buffer) {
        let from_node = match self.graph.nodes.get(&edge.from) {
            Some(n) => n,
            None => return,
        };
        let to_node = match self.graph.nodes.get(&edge.to) {
            Some(n) => n,
            None => return,
        };

        // Check if edge is on critical path
        let is_critical = self.config.show_critical_path
            && from_node.is_critical
            && to_node.is_critical;

        let style = if is_critical {
            self.config.critical_edge_style
        } else {
            self.config.edge_style
        };

        // Calculate start and end positions (from right of from_node to left of to_node)
        let (from_vx, from_vy) = self.transform_position(
            from_node.x + self.config.node_width,
            from_node.y + self.config.node_height / 2,
        );
        let (to_vx, to_vy) = self.transform_position(to_node.x, to_node.y + self.config.node_height / 2);

        // Simple horizontal line with arrow
        if from_vy == to_vy && from_vx < to_vx {
            let y_pos = area.y + from_vy as u16;
            if y_pos >= area.y && y_pos < area.y + area.height {
                for x in (from_vx + 1)..to_vx {
                    let x_pos = area.x + x as u16;
                    if x_pos >= area.x && x_pos < area.x + area.width {
                        buf.set_string(x_pos, y_pos, "─", style);
                    }
                }
                // Arrow head
                let arrow_x = area.x + to_vx as u16;
                if arrow_x > area.x && arrow_x <= area.x + area.width {
                    buf.set_string(arrow_x.saturating_sub(1), y_pos, "→", style);
                }
            }
        }
        // TODO: Handle non-horizontal edges with corner characters
    }
}

impl<'a> Widget for PertChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render edges first (so they appear behind nodes)
        for edge in &self.graph.edges {
            self.render_edge(edge, area, buf);
        }

        // Render nodes
        for node in self.graph.nodes.values() {
            self.render_node(node, area, buf);
        }

        // Render cycle warning if any
        if self.graph.cycle_detection.has_cycle {
            let warning = format!(
                "⚠ {} cycle(s) detected",
                self.graph.cycle_detection.cycle_edges.len()
            );
            if area.height > 0 {
                buf.set_string(
                    area.x,
                    area.y,
                    &warning,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                );
            }
        }

        // Render status line at bottom
        if area.height > 1 {
            let status = if let Some(selected) = &self.config.selected_node {
                if let Some(node) = self.graph.nodes.get(selected) {
                    format!(
                        "Selected: {} | ES: {:.1} LS: {:.1} Slack: {:.1} | Zoom: {:.1}x",
                        node.issue_id,
                        node.earliest_start,
                        node.latest_start,
                        node.slack,
                        self.config.zoom
                    )
                } else {
                    format!("Zoom: {:.1}x", self.config.zoom)
                }
            } else {
                format!(
                    "{} nodes, {} edges | Zoom: {:.1}x | [c]ritical path: {}",
                    self.graph.nodes.len(),
                    self.graph.edges.len(),
                    self.config.zoom,
                    if self.config.show_critical_path {
                        "ON"
                    } else {
                        "OFF"
                    }
                )
            };
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                &status,
                Style::default().fg(Color::Gray),
            );
        }
    }
}

/// Truncate string to fit width, adding ellipsis if needed
fn truncate_string(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{Issue, IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_pert_chart_config_default() {
        let config = PertChartConfig::default();
        assert_eq!(config.offset_x, 0);
        assert_eq!(config.offset_y, 0);
        assert_eq!(config.zoom, 1.0);
        assert!(config.show_critical_path);
        assert!(config.selected_node.is_none());
    }

    #[test]
    fn test_pert_chart_config_pan() {
        let mut config = PertChartConfig::default();
        config.pan(10, 20);
        assert_eq!(config.offset_x, 10);
        assert_eq!(config.offset_y, 20);

        config.pan(-5, -10);
        assert_eq!(config.offset_x, 5);
        assert_eq!(config.offset_y, 10);
    }

    #[test]
    fn test_pert_chart_config_zoom() {
        let mut config = PertChartConfig::default();
        config.adjust_zoom(2.0);
        assert_eq!(config.zoom, 2.0);

        config.adjust_zoom(0.5);
        assert_eq!(config.zoom, 1.0);

        // Test zoom limits
        config.adjust_zoom(5.0);
        assert_eq!(config.zoom, 3.0); // Max zoom

        config.zoom = 1.0;
        config.adjust_zoom(0.1);
        assert_eq!(config.zoom, 0.5); // Min zoom
    }

    #[test]
    fn test_pert_chart_config_toggle_critical_path() {
        let mut config = PertChartConfig::default();
        assert!(config.show_critical_path);

        config.toggle_critical_path();
        assert!(!config.show_critical_path);

        config.toggle_critical_path();
        assert!(config.show_critical_path);
    }

    #[test]
    fn test_pert_chart_config_select_next() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let issue3 = create_test_issue("C", "Task C");

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        let mut config = PertChartConfig::default();

        // Initially no selection
        assert!(config.selected_node.is_none());

        // First call selects first node in topological order
        config.select_next(&graph);
        let first_node = graph.topological_order[0].clone();
        assert_eq!(config.selected_node, Some(first_node.clone()));

        // Move to next
        config.select_next(&graph);
        let second_node = graph.topological_order[1].clone();
        assert_eq!(config.selected_node, Some(second_node));

        // Move to next
        config.select_next(&graph);
        let third_node = graph.topological_order[2].clone();
        assert_eq!(config.selected_node, Some(third_node.clone()));

        // At end, stays at last node
        config.select_next(&graph);
        assert_eq!(config.selected_node, Some(third_node));
    }

    #[test]
    fn test_pert_chart_config_select_prev() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let issue3 = create_test_issue("C", "Task C");

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        let mut config = PertChartConfig::default();

        // Initially no selection, prev selects last in topological order
        config.select_prev(&graph);
        let last_node = graph.topological_order[2].clone();
        assert_eq!(config.selected_node, Some(last_node.clone()));

        // Move to prev
        config.select_prev(&graph);
        let second_node = graph.topological_order[1].clone();
        assert_eq!(config.selected_node, Some(second_node));

        // Move to prev
        config.select_prev(&graph);
        let first_node = graph.topological_order[0].clone();
        assert_eq!(config.selected_node, Some(first_node.clone()));

        // At start, stays at first node
        config.select_prev(&graph);
        assert_eq!(config.selected_node, Some(first_node));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
        assert_eq!(truncate_string("test", 3), "...");
        assert_eq!(truncate_string("test", 2), "...");
    }

    #[test]
    fn test_pert_chart_creation() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let chart = PertChart::new(&graph);

        assert_eq!(chart.graph.nodes.len(), 1);
        assert!(chart.config.show_critical_path);
    }
}
