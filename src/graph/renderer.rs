//! Graph renderer using box-drawing characters
//!
//! Renders a laid-out graph using Unicode box-drawing characters for nodes and edges.

use super::layout::{GraphLayout, LayoutEdge, LayoutNode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
};

/// Options for rendering
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Style for normal nodes
    pub node_style: Style,
    /// Style for selected nodes
    pub selected_style: Style,
    /// Style for edges
    pub edge_style: Style,
    /// Viewport offset X (for panning)
    pub offset_x: isize,
    /// Viewport offset Y (for panning)
    pub offset_y: isize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            node_style: Style::default().fg(Color::Cyan),
            selected_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            edge_style: Style::default().fg(Color::DarkGray),
            offset_x: 0,
            offset_y: 0,
        }
    }
}

/// Graph renderer
pub struct GraphRenderer {
    /// The graph layout to render
    layout: GraphLayout,
    /// Selected node ID
    selected_node: Option<String>,
}

impl GraphRenderer {
    /// Create a new graph renderer
    pub fn new(layout: GraphLayout) -> Self {
        Self {
            layout,
            selected_node: None,
        }
    }

    /// Set the selected node
    pub fn select_node(&mut self, node_id: Option<String>) {
        self.selected_node = node_id;
    }

    /// Get the selected node
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_deref()
    }

    /// Render the graph to a buffer
    pub fn render(&self, area: Rect, buf: &mut Buffer, options: &RenderOptions) {
        if self.layout.nodes.is_empty() {
            return;
        }

        // First pass: draw edges
        for edge in &self.layout.edges {
            self.render_edge(edge, area, buf, options);
        }

        // Second pass: draw nodes (so they appear on top of edges)
        for node in &self.layout.nodes {
            let is_selected = self
                .selected_node
                .as_ref()
                .map(|s| s == &node.id)
                .unwrap_or(false);
            self.render_node(node, is_selected, area, buf, options);
        }
    }

    /// Render a single node
    fn render_node(
        &self,
        node: &LayoutNode,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        options: &RenderOptions,
    ) {
        // Apply viewport offset
        let x = (node.x as isize + options.offset_x) as i32;
        let y = (node.y as isize + options.offset_y) as i32;

        // Skip if outside viewport
        if x < 0 || y < 0 || x >= area.width as i32 || y >= area.height as i32 {
            return;
        }

        let screen_x = (area.x as i32 + x) as u16;
        let screen_y = (area.y as i32 + y) as u16;

        let style = if is_selected {
            options.selected_style
        } else {
            options.node_style
        };

        // Draw box top
        if screen_y < area.bottom() && screen_x < area.right() {
            let top_line = format!(
                "╭{}╮",
                "─".repeat(
                    node.width
                        .saturating_sub(2)
                        .min((area.right() - screen_x - 2) as usize)
                )
            );
            buf.set_line(
                screen_x,
                screen_y,
                &Line::styled(top_line, style),
                node.width as u16,
            );
        }

        // Draw box middle with text
        if screen_y + 1 < area.bottom() && screen_x < area.right() {
            let truncated_text = if node.text.len() > node.width.saturating_sub(2) {
                format!("{}…", &node.text[..node.width.saturating_sub(3)])
            } else {
                format!(
                    "{:^width$}",
                    node.text,
                    width = node.width.saturating_sub(2)
                )
            };
            let middle_line = format!("│{}│", truncated_text);
            buf.set_line(
                screen_x,
                screen_y + 1,
                &Line::styled(middle_line, style),
                node.width as u16,
            );
        }

        // Draw box bottom
        if screen_y + 2 < area.bottom() && screen_x < area.right() {
            let bottom_line = format!(
                "╰{}╯",
                "─".repeat(
                    node.width
                        .saturating_sub(2)
                        .min((area.right() - screen_x - 2) as usize)
                )
            );
            buf.set_line(
                screen_x,
                screen_y + 2,
                &Line::styled(bottom_line, style),
                node.width as u16,
            );
        }
    }

    /// Render an edge between two nodes
    fn render_edge(
        &self,
        edge: &LayoutEdge,
        area: Rect,
        buf: &mut Buffer,
        options: &RenderOptions,
    ) {
        let from_node = match self.layout.get_node(&edge.from) {
            Some(n) => n,
            None => return,
        };
        let to_node = match self.layout.get_node(&edge.to) {
            Some(n) => n,
            None => return,
        };

        // Apply viewport offset
        let from_x = from_node.x as isize + options.offset_x;
        let from_y = from_node.y as isize + options.offset_y;
        let to_x = to_node.x as isize + options.offset_x;
        let to_y = to_node.y as isize + options.offset_y;

        // Start from bottom-center of source node
        let start_x = from_x + (from_node.width / 2) as isize;
        let start_y = from_y + from_node.height as isize;

        // End at top-center of target node
        let end_x = to_x + (to_node.width / 2) as isize;
        let end_y = to_y;

        // Draw a simple vertical-then-horizontal line
        // For downward edges, draw: vertical down, then horizontal, then vertical down to target
        if start_y < end_y {
            // Draw vertical segment from source
            let mid_y = start_y + (end_y - start_y) / 2;
            for y in start_y..=mid_y.min(area.height as isize - 1) {
                if y >= 0 && start_x >= 0 && start_x < area.width as isize {
                    let screen_x = (area.x as isize + start_x) as u16;
                    let screen_y = (area.y as isize + y) as u16;
                    if screen_x < area.right() && screen_y < area.bottom() {
                        buf.set_line(
                            screen_x,
                            screen_y,
                            &Line::styled("│", options.edge_style),
                            1,
                        );
                    }
                }
            }

            // Draw horizontal segment
            let min_x = start_x.min(end_x);
            let max_x = start_x.max(end_x);
            for x in min_x..=max_x {
                if x >= 0 && x < area.width as isize && mid_y >= 0 && mid_y < area.height as isize {
                    let screen_x = (area.x as isize + x) as u16;
                    let screen_y = (area.y as isize + mid_y) as u16;
                    if screen_x < area.right() && screen_y < area.bottom() {
                        buf.set_line(
                            screen_x,
                            screen_y,
                            &Line::styled("─", options.edge_style),
                            1,
                        );
                    }
                }
            }

            // Draw vertical segment to target
            for y in mid_y..=end_y.min(area.height as isize - 1) {
                if y >= 0 && end_x >= 0 && end_x < area.width as isize {
                    let screen_x = (area.x as isize + end_x) as u16;
                    let screen_y = (area.y as isize + y) as u16;
                    if screen_x < area.right() && screen_y < area.bottom() {
                        buf.set_line(
                            screen_x,
                            screen_y,
                            &Line::styled("│", options.edge_style),
                            1,
                        );
                    }
                }
            }

            // Draw arrow at target
            if end_y > 0 && end_x >= 0 && end_x < area.width as isize {
                let screen_x = (area.x as isize + end_x) as u16;
                let screen_y = (area.y as isize + end_y - 1) as u16;
                if screen_x < area.right() && screen_y < area.bottom() {
                    buf.set_line(
                        screen_x,
                        screen_y,
                        &Line::styled("v", options.edge_style),
                        1,
                    );
                }
            }
        }
    }

    /// Get the layout
    pub fn layout(&self) -> &GraphLayout {
        &self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::layout::LayoutOptions;
    use std::collections::HashMap;

    #[test]
    fn test_renderer_creation() {
        let layout = GraphLayout::new(HashMap::new(), Vec::new(), LayoutOptions::default());
        let renderer = GraphRenderer::new(layout);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_node_selection() {
        let layout = GraphLayout::new(HashMap::new(), Vec::new(), LayoutOptions::default());
        let mut renderer = GraphRenderer::new(layout);

        renderer.select_node(Some("node1".to_string()));
        assert_eq!(renderer.selected_node(), Some("node1"));

        renderer.select_node(None);
        assert_eq!(renderer.selected_node(), None);
    }

    #[test]
    fn test_render_empty_graph() {
        let layout = GraphLayout::new(HashMap::new(), Vec::new(), LayoutOptions::default());
        let renderer = GraphRenderer::new(layout);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let options = RenderOptions::default();

        // Should not panic on empty graph
        renderer.render(area, &mut buf, &options);
    }

    #[test]
    fn test_render_single_node() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());

        let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());
        let renderer = GraphRenderer::new(layout);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let options = RenderOptions::default();

        renderer.render(area, &mut buf, &options);

        // Verify the buffer is not empty (node was drawn)
        let has_content = (0..area.height).any(|y| {
            (0..area.width).any(|x| {
                let cell = buf.get(x, y);
                !cell.symbol().is_empty() && cell.symbol() != " "
            })
        });
        assert!(has_content);
    }
}
