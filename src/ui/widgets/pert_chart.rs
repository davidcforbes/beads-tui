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
    /// Focus mode: show only subgraph around focused node
    pub focus_mode: bool,
    /// Focused node ID (for subgraph focus)
    pub focus_node: Option<String>,
    /// Focus depth (how many levels up/downstream to show)
    pub focus_depth: usize,
    /// Focus direction: "upstream", "downstream", "both"
    pub focus_direction: String,
    /// Show legend
    pub show_legend: bool,
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
            critical_style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            edge_style: Style::default().fg(Color::Gray),
            critical_edge_style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            node_width: 20,
            node_height: 5,
            focus_mode: false,
            focus_node: None,
            focus_depth: 1,
            focus_direction: "both".to_string(),
            show_legend: true,
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
        self.zoom = zoom.clamp(0.5, 3.0);
        self
    }

    /// Adjust zoom by factor
    pub fn adjust_zoom(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.5, 3.0);
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

    /// Toggle focus mode
    pub fn toggle_focus_mode(&mut self) {
        self.focus_mode = !self.focus_mode;
        if self.focus_mode && self.focus_node.is_none() {
            // Set focus to selected node if entering focus mode
            self.focus_node = self.selected_node.clone();
        }
    }

    /// Set focus on a specific node
    pub fn focus_on_node(&mut self, node_id: Option<String>) {
        self.focus_node = node_id;
        if self.focus_node.is_some() {
            self.focus_mode = true;
        }
    }

    /// Set focus depth
    pub fn set_focus_depth(&mut self, depth: usize) {
        self.focus_depth = depth;
    }

    /// Set focus direction
    pub fn set_focus_direction(&mut self, direction: &str) {
        self.focus_direction = direction.to_string();
    }

    /// Toggle legend visibility
    pub fn toggle_legend(&mut self) {
        self.show_legend = !self.show_legend;
    }

    /// Exit focus mode (restore full graph view)
    pub fn exit_focus_mode(&mut self) {
        self.focus_mode = false;
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
        x >= 0 && y >= 0 && (x as u16) < area.width && (y as u16) < area.height
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
        let is_selected = self.config.selected_node.as_ref() == Some(&node.issue_id);
        let is_critical = node.is_critical && self.config.show_critical_path;

        let style = if is_selected {
            self.config.selected_style
        } else if is_critical {
            self.config.critical_style
        } else {
            self.config.normal_style
        };

        // Render box borders
        let width = self
            .config
            .node_width
            .min(area.width.saturating_sub(vx as u16));
        let height = self
            .config
            .node_height
            .min(area.height.saturating_sub(vy as u16));

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
        let is_critical =
            self.config.show_critical_path && from_node.is_critical && to_node.is_critical;

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
        let (to_vx, to_vy) =
            self.transform_position(to_node.x, to_node.y + self.config.node_height / 2);

        // Skip if edge goes backward horizontally (shouldn't happen in DAG)
        if from_vx >= to_vx {
            return;
        }

        // Simple horizontal line with arrow (same Y level)
        if from_vy == to_vy {
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
        } else {
            // Non-horizontal edge: draw H-V-H pattern with corners
            // Turn point is halfway between source and target X
            let turn_x = from_vx + (to_vx - from_vx) / 2;

            // Horizontal segment 1: from source to turn point
            let y1 = area.y + from_vy as u16;
            if y1 >= area.y && y1 < area.y + area.height {
                for x in (from_vx + 1)..turn_x {
                    let x_pos = area.x + x as u16;
                    if x_pos >= area.x && x_pos < area.x + area.width {
                        buf.set_string(x_pos, y1, "─", style);
                    }
                }
            }

            // Vertical segment: from turn point at source Y to turn point at target Y
            let x_turn = area.x + turn_x as u16;
            if x_turn >= area.x && x_turn < area.x + area.width {
                let (y_start, y_end) = if from_vy < to_vy {
                    (from_vy, to_vy)
                } else {
                    (to_vy, from_vy)
                };

                for y in (y_start + 1)..y_end {
                    let y_pos = area.y + y as u16;
                    if y_pos >= area.y && y_pos < area.y + area.height {
                        buf.set_string(x_turn, y_pos, "│", style);
                    }
                }

                // Corner characters
                if from_vy < to_vy {
                    // Going down: use ┐ at top, └ at bottom
                    if y1 >= area.y && y1 < area.y + area.height {
                        buf.set_string(x_turn, y1, "┐", style);
                    }
                    let y_bottom = area.y + to_vy as u16;
                    if y_bottom >= area.y && y_bottom < area.y + area.height {
                        buf.set_string(x_turn, y_bottom, "└", style);
                    }
                } else {
                    // Going up: use ┘ at bottom, ┌ at top
                    if y1 >= area.y && y1 < area.y + area.height {
                        buf.set_string(x_turn, y1, "┘", style);
                    }
                    let y_top = area.y + to_vy as u16;
                    if y_top >= area.y && y_top < area.y + area.height {
                        buf.set_string(x_turn, y_top, "┌", style);
                    }
                }
            }

            // Horizontal segment 2: from turn point to target (with arrow)
            let y2 = area.y + to_vy as u16;
            if y2 >= area.y && y2 < area.y + area.height {
                for x in (turn_x + 1)..to_vx {
                    let x_pos = area.x + x as u16;
                    if x_pos >= area.x && x_pos < area.x + area.width {
                        buf.set_string(x_pos, y2, "─", style);
                    }
                }
                // Arrow head
                let arrow_x = area.x + to_vx as u16;
                if arrow_x > area.x && arrow_x <= area.x + area.width {
                    buf.set_string(arrow_x.saturating_sub(1), y2, "→", style);
                }
            }
        }
    }
}

impl<'a> Widget for PertChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Apply focus mode filtering if enabled
        let (nodes_to_render, edges_to_render) = if self.config.focus_mode {
            if let Some(focus_node) = &self.config.focus_node {
                self.graph.compute_subgraph(
                    focus_node,
                    self.config.focus_depth,
                    &self.config.focus_direction,
                )
            } else {
                // No focus node set, render full graph
                (
                    self.graph.nodes.keys().cloned().collect(),
                    self.graph.edges.clone(),
                )
            }
        } else {
            // Normal mode, render full graph
            (
                self.graph.nodes.keys().cloned().collect(),
                self.graph.edges.clone(),
            )
        };

        // Render edges first (so they appear behind nodes)
        for edge in &edges_to_render {
            self.render_edge(edge, area, buf);
        }

        // Render nodes
        for node_id in &nodes_to_render {
            if let Some(node) = self.graph.nodes.get(node_id) {
                self.render_node(node, area, buf);
            }
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

        // Render legend if enabled
        if self.config.show_legend && area.height > 3 {
            let legend_y = area.y + 1;
            let legend_items = vec![
                ("─→", "Dependency", self.config.edge_style),
                ("─→", "Critical Path", self.config.critical_edge_style),
                ("□", "Normal", self.config.normal_style),
                ("■", "Selected", self.config.selected_style),
            ];

            let mut x_offset = area.x + 2;
            for (symbol, label, style) in legend_items {
                let text = format!("{symbol} {label} ");
                if x_offset + text.len() as u16 <= area.x + area.width {
                    buf.set_string(x_offset, legend_y, &text, style);
                    x_offset += text.len() as u16;
                }
            }
        }

        // Render status line at bottom
        if area.height > 1 {
            let status = if let Some(selected) = &self.config.selected_node {
                if let Some(node) = self.graph.nodes.get(selected) {
                    let focus_info = if self.config.focus_mode {
                        format!(
                            " | Focus: {} depth {}",
                            self.config.focus_direction, self.config.focus_depth
                        )
                    } else {
                        String::new()
                    };
                    format!(
                        "Selected: {} | ES: {:.1} LS: {:.1} Slack: {:.1} | Zoom: {:.1}x{}",
                        node.issue_id,
                        node.earliest_start,
                        node.latest_start,
                        node.slack,
                        self.config.zoom,
                        focus_info
                    )
                } else {
                    format!("Zoom: {:.1}x", self.config.zoom)
                }
            } else {
                let focus_info = if self.config.focus_mode {
                    if let Some(focus) = &self.config.focus_node {
                        format!(
                            " | Focus: {} on {} (depth {})",
                            self.config.focus_direction, focus, self.config.focus_depth
                        )
                    } else {
                        " | Focus mode (no node)".to_string()
                    }
                } else {
                    String::new()
                };
                format!(
                    "{} nodes, {} edges | Zoom: {:.1}x | [c]ritical: {}{}",
                    self.graph.nodes.len(),
                    self.graph.edges.len(),
                    self.config.zoom,
                    if self.config.show_critical_path {
                        "ON"
                    } else {
                        "OFF"
                    },
                    focus_info
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

    #[test]
    fn test_pert_chart_config_focus_mode() {
        let mut config = PertChartConfig::default();
        assert!(!config.focus_mode);
        assert!(config.focus_node.is_none());

        // Set selected node first
        config.selected_node = Some("A".to_string());

        // Toggle focus mode should enable and set focus to selected
        config.toggle_focus_mode();
        assert!(config.focus_mode);
        assert_eq!(config.focus_node, Some("A".to_string()));

        // Toggle again to disable
        config.toggle_focus_mode();
        assert!(!config.focus_mode);
    }

    #[test]
    fn test_pert_chart_config_focus_on_node() {
        let mut config = PertChartConfig::default();

        config.focus_on_node(Some("B".to_string()));
        assert!(config.focus_mode);
        assert_eq!(config.focus_node, Some("B".to_string()));

        config.exit_focus_mode();
        assert!(!config.focus_mode);
    }

    #[test]
    fn test_pert_chart_config_focus_settings() {
        let mut config = PertChartConfig::default();
        assert_eq!(config.focus_depth, 1);
        assert_eq!(config.focus_direction, "both");

        config.set_focus_depth(3);
        assert_eq!(config.focus_depth, 3);

        config.set_focus_direction("upstream");
        assert_eq!(config.focus_direction, "upstream");
    }

    #[test]
    fn test_pert_chart_config_toggle_legend() {
        let mut config = PertChartConfig::default();
        assert!(config.show_legend);

        config.toggle_legend();
        assert!(!config.show_legend);

        config.toggle_legend();
        assert!(config.show_legend);
    }

    #[test]
    fn test_non_horizontal_edge_rendering() {
        // Create issues that will be placed in different lanes (different Y positions)
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Make B and C depend on A (parallel paths)
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];

        let mut graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Manually set different Y positions to force non-horizontal edges
        // (In real layout, this happens when nodes are in different lanes)
        if let Some(node_a) = graph.nodes.get_mut("A") {
            node_a.y = 0;
        }
        if let Some(node_b) = graph.nodes.get_mut("B") {
            node_b.y = 3; // Different lane (lane * 3)
        }
        if let Some(node_c) = graph.nodes.get_mut("C") {
            node_c.y = 6; // Another lane
        }

        let chart = PertChart::new(&graph);

        // Render to a buffer - this should not panic
        // Non-horizontal edges should be rendered with corners and vertical segments
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        ratatui::widgets::Widget::render(chart, Rect::new(0, 0, 80, 24), &mut buf);

        // Verify nodes are at different Y positions (testing our setup)
        assert_eq!(graph.nodes.get("A").unwrap().y, 0);
        assert_eq!(graph.nodes.get("B").unwrap().y, 3);
        assert_eq!(graph.nodes.get("C").unwrap().y, 6);

        // The test passes if we reach here without panic
        // (Before fix, non-horizontal edges were silently skipped)
    }

    #[test]
    fn test_non_horizontal_edge_with_critical_path() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        issue2.dependencies = vec!["A".to_string()];

        let mut graph = PertGraph::new(&[issue1, issue2], 1.0);

        // Set different Y positions
        if let Some(node_a) = graph.nodes.get_mut("A") {
            node_a.y = 0;
            node_a.is_critical = true;
        }
        if let Some(node_b) = graph.nodes.get_mut("B") {
            node_b.y = 3;
            node_b.is_critical = true;
        }

        let config = PertChartConfig {
            show_critical_path: true,
            ..Default::default()
        };
        let chart = PertChart::new(&graph).config(config);

        // Render to a buffer - should not panic
        // Critical path styling should apply to non-horizontal edges
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        ratatui::widgets::Widget::render(chart, Rect::new(0, 0, 80, 24), &mut buf);

        // Verify both nodes are marked as critical
        assert!(graph.nodes.get("A").unwrap().is_critical);
        assert!(graph.nodes.get("B").unwrap().is_critical);

        // The test passes if we reach here without panic
        // (Verifies critical path styling works with non-horizontal edges)
    }

    #[test]
    fn test_backward_edge_skipped() {
        // Test that edges going backward (from_vx >= to_vx) are skipped
        // This shouldn't happen in a proper DAG layout, but we handle it gracefully
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        issue2.dependencies = vec!["A".to_string()];

        let mut graph = PertGraph::new(&[issue1, issue2], 1.0);

        // Manually create an invalid situation where target is left of source
        if let Some(node_a) = graph.nodes.get_mut("A") {
            node_a.x = 20;
            node_a.y = 0;
        }
        if let Some(node_b) = graph.nodes.get_mut("B") {
            node_b.x = 10; // B is to the left of A (invalid for DAG)
            node_b.y = 3;
        }

        let chart = PertChart::new(&graph);

        // This should not panic, even though the edge is invalid
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        ratatui::widgets::Widget::render(chart, Rect::new(0, 0, 80, 24), &mut buf);

        // The test passes if we reach here without panic
        // (The edge is skipped in render_edge when from_vx >= to_vx)
    }

    #[test]
    fn test_pert_chart_config_clone() {
        let config = PertChartConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.offset_x, config.offset_x);
        assert_eq!(cloned.zoom, config.zoom);
        assert_eq!(cloned.show_critical_path, config.show_critical_path);
    }

    #[test]
    fn test_pert_chart_config_offset_builder() {
        let config = PertChartConfig::new().offset(15, 25);
        assert_eq!(config.offset_x, 15);
        assert_eq!(config.offset_y, 25);
    }

    #[test]
    fn test_pert_chart_config_zoom_builder() {
        let config = PertChartConfig::new().zoom(2.5);
        assert_eq!(config.zoom, 2.5);
    }

    #[test]
    fn test_pert_chart_config_zoom_clamp_high() {
        let config = PertChartConfig::new().zoom(5.0);
        assert_eq!(config.zoom, 3.0); // Clamped to max
    }

    #[test]
    fn test_pert_chart_config_zoom_clamp_low() {
        let config = PertChartConfig::new().zoom(0.1);
        assert_eq!(config.zoom, 0.5); // Clamped to min
    }

    #[test]
    fn test_pert_chart_config_select_node() {
        let mut config = PertChartConfig::default();
        config.select_node(Some("test-node".to_string()));
        assert_eq!(config.selected_node, Some("test-node".to_string()));

        config.select_node(None);
        assert!(config.selected_node.is_none());
    }

    #[test]
    fn test_pert_chart_config_select_adjacent_empty_graph() {
        let graph = PertGraph::new(&[], 1.0);
        let mut config = PertChartConfig::default();

        config.select_adjacent(&graph, Direction::Right);
        assert!(config.selected_node.is_none());
    }

    #[test]
    fn test_pert_chart_config_select_adjacent_no_selection() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let mut config = PertChartConfig::default();

        config.select_adjacent(&graph, Direction::Right);
        // Should select first node when nothing selected
        assert!(config.selected_node.is_some());
    }

    #[test]
    fn test_pert_chart_builder_config() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let config = PertChartConfig::new().zoom(2.0);
        let chart = PertChart::new(&graph).config(config);

        assert_eq!(chart.config.zoom, 2.0);
    }

    #[test]
    fn test_direction_equality() {
        assert_eq!(Direction::Up, Direction::Up);
        assert_eq!(Direction::Down, Direction::Down);
        assert_eq!(Direction::Left, Direction::Left);
        assert_eq!(Direction::Right, Direction::Right);
        assert_ne!(Direction::Up, Direction::Down);
    }

    #[test]
    fn test_direction_clone() {
        let dir = Direction::Up;
        let cloned = dir;
        assert_eq!(dir, cloned);
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(truncate_string("exactly", 7), "exactly");
    }

    #[test]
    fn test_truncate_string_one_char() {
        assert_eq!(truncate_string("test", 1), "...");
    }

    #[test]
    fn test_pert_chart_config_pan_multiple_times() {
        let mut config = PertChartConfig::default();
        config.pan(10, 20);
        config.pan(-5, -10);
        assert_eq!(config.offset_x, 5);
        assert_eq!(config.offset_y, 10);
    }

    #[test]
    fn test_pert_chart_config_adjust_zoom_multiple() {
        let mut config = PertChartConfig::default();
        config.adjust_zoom(2.0);
        config.adjust_zoom(1.5);
        assert_eq!(config.zoom, 3.0); // 1.0 * 2.0 * 1.5 = 3.0 (clamped)
    }

    #[test]
    fn test_pert_chart_config_focus_depth_zero() {
        let mut config = PertChartConfig::default();
        config.set_focus_depth(0);
        assert_eq!(config.focus_depth, 0);
    }

    #[test]
    fn test_pert_chart_config_focus_direction_variations() {
        let mut config = PertChartConfig::default();

        config.set_focus_direction("upstream");
        assert_eq!(config.focus_direction, "upstream");

        config.set_focus_direction("downstream");
        assert_eq!(config.focus_direction, "downstream");

        config.set_focus_direction("both");
        assert_eq!(config.focus_direction, "both");
    }

    #[test]
    fn test_pert_chart_config_exit_focus_mode() {
        let mut config = PertChartConfig {
            focus_mode: true,
            ..Default::default()
        };

        config.exit_focus_mode();
        assert!(!config.focus_mode);
    }

    #[test]
    fn test_pert_chart_with_cycle_detection() {
        let mut issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        // Create circular dependency
        issue1.dependencies = vec!["B".to_string()];
        issue2.dependencies = vec!["A".to_string()];

        let graph = PertGraph::new(&[issue1, issue2], 1.0);
        let chart = PertChart::new(&graph);

        // Should detect cycle
        assert!(graph.cycle_detection.has_cycle);

        // Rendering should not panic even with cycles
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        ratatui::widgets::Widget::render(chart, Rect::new(0, 0, 80, 24), &mut buf);
    }

    #[test]
    fn test_direction_copy() {
        let dir = Direction::Left;
        let copied = dir;
        assert_eq!(dir, copied);
    }

    #[test]
    fn test_truncate_string_empty() {
        assert_eq!(truncate_string("", 10), "");
    }

    #[test]
    fn test_truncate_string_zero_width() {
        assert_eq!(truncate_string("test", 0), "...");
    }

    #[test]
    fn test_pert_chart_apply_zoom() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let config = PertChartConfig {
            zoom: 2.0,
            ..Default::default()
        };
        let chart = PertChart::new(&graph).config(config);

        assert_eq!(chart.apply_zoom(10), 20);
        assert_eq!(chart.apply_zoom(5), 10);
    }

    #[test]
    fn test_pert_chart_transform_position() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let config = PertChartConfig {
            offset_x: 10,
            offset_y: 5,
            ..Default::default()
        };
        let chart = PertChart::new(&graph).config(config);

        let (vx, vy) = chart.transform_position(20, 15);
        assert_eq!(vx, 10); // 20 - 10
        assert_eq!(vy, 10); // 15 - 5
    }

    #[test]
    fn test_pert_chart_is_visible() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let chart = PertChart::new(&graph);
        let area = Rect::new(0, 0, 80, 24);

        assert!(chart.is_visible(10, 10, &area));
        assert!(chart.is_visible(0, 0, &area));
        assert!(!chart.is_visible(-1, 10, &area));
        assert!(!chart.is_visible(10, -1, &area));
        assert!(!chart.is_visible(80, 10, &area));
        assert!(!chart.is_visible(10, 24, &area));
    }

    #[test]
    fn test_select_adjacent_all_directions() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");

        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string(), "C".to_string()];

        let mut graph = PertGraph::new(&[issue1, issue2, issue3, issue4], 1.0);

        // Manually position nodes in a grid pattern
        if let Some(node) = graph.nodes.get_mut("A") {
            node.x = 10;
            node.y = 10;
        }
        if let Some(node) = graph.nodes.get_mut("B") {
            node.x = 30;
            node.y = 10;
        }
        if let Some(node) = graph.nodes.get_mut("C") {
            node.x = 10;
            node.y = 20;
        }
        if let Some(node) = graph.nodes.get_mut("D") {
            node.x = 30;
            node.y = 20;
        }

        let config = PertChartConfig {
            selected_node: Some("A".to_string()),
            ..Default::default()
        };

        // Test Right direction
        let mut test_config = config.clone();
        test_config.select_adjacent(&graph, Direction::Right);
        assert_eq!(test_config.selected_node, Some("B".to_string()));

        // Test Down direction
        let mut test_config = config.clone();
        test_config.select_adjacent(&graph, Direction::Down);
        assert_eq!(test_config.selected_node, Some("C".to_string()));
    }

    #[test]
    fn test_select_next_empty_graph() {
        let graph = PertGraph::new(&[], 1.0);
        let mut config = PertChartConfig::default();

        config.select_next(&graph);
        assert!(config.selected_node.is_none());
    }

    #[test]
    fn test_select_prev_empty_graph() {
        let graph = PertGraph::new(&[], 1.0);
        let mut config = PertChartConfig::default();

        config.select_prev(&graph);
        assert!(config.selected_node.is_none());
    }

    #[test]
    fn test_focus_on_node_none() {
        let mut config = PertChartConfig {
            focus_mode: true,
            ..Default::default()
        };

        config.focus_on_node(None);
        assert!(config.focus_node.is_none());
        assert!(config.focus_mode); // focus_mode stays true, focus_node is None
    }

    #[test]
    fn test_pert_chart_config_builder_chain() {
        let config = PertChartConfig::new().offset(10, 20).zoom(2.5);

        assert_eq!(config.offset_x, 10);
        assert_eq!(config.offset_y, 20);
        assert_eq!(config.zoom, 2.5);
    }

    #[test]
    fn test_node_size_configuration() {
        let mut config = PertChartConfig::default();
        assert_eq!(config.node_width, 20);
        assert_eq!(config.node_height, 5);

        config.node_width = 30;
        config.node_height = 7;
        assert_eq!(config.node_width, 30);
        assert_eq!(config.node_height, 7);
    }

    #[test]
    fn test_zoom_boundary_exact_limits() {
        let config1 = PertChartConfig::new().zoom(0.5);
        assert_eq!(config1.zoom, 0.5);

        let config2 = PertChartConfig::new().zoom(3.0);
        assert_eq!(config2.zoom, 3.0);
    }

    #[test]
    fn test_toggle_focus_mode_without_selected() {
        let mut config = PertChartConfig::default();
        assert!(config.selected_node.is_none());

        config.toggle_focus_mode();
        assert!(config.focus_mode);
        assert!(config.focus_node.is_none()); // No node to focus on
    }

    #[test]
    fn test_pert_chart_clone() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let chart = PertChart::new(&graph);
        let cloned = chart.clone();

        assert_eq!(chart.graph.nodes.len(), cloned.graph.nodes.len());
        assert_eq!(chart.config.zoom, cloned.config.zoom);
    }

    #[test]
    fn test_pert_chart_config_new() {
        let config = PertChartConfig::new();
        assert_eq!(config.offset_x, 0);
        assert_eq!(config.offset_y, 0);
        assert_eq!(config.zoom, 1.0);
    }
}
