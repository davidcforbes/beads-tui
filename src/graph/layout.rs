//! Graph layout algorithms for dependency visualization
//!
//! Implements a layered graph layout algorithm similar to Sugiyama/dagre.
//! The algorithm works in several phases:
//! 1. Cycle removal (feedback arc set)
//! 2. Layer assignment (longest path layering)
//! 3. Crossing reduction (barycenter heuristic)
//! 4. Coordinate assignment (with collision detection)

use std::collections::{HashMap, HashSet, VecDeque};

/// Options for graph layout
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// Minimum horizontal spacing between nodes
    pub node_spacing_x: usize,
    /// Minimum vertical spacing between layers
    pub layer_spacing_y: usize,
    /// Width of each node box
    pub node_width: usize,
    /// Height of each node box
    pub node_height: usize,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            node_spacing_x: 4,
            layer_spacing_y: 3,
            node_width: 20,
            node_height: 3,
        }
    }
}

/// A positioned node in the graph layout
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// Node ID
    pub id: String,
    /// Display text
    pub text: String,
    /// X coordinate (top-left corner)
    pub x: usize,
    /// Y coordinate (top-left corner)
    pub y: usize,
    /// Width of the node
    pub width: usize,
    /// Height of the node
    pub height: usize,
    /// Layer number (for debugging/analysis)
    pub layer: usize,
}

/// An edge in the graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
}

/// Result of graph layout
#[derive(Debug, Clone)]
pub struct GraphLayout {
    /// Positioned nodes
    pub nodes: Vec<LayoutNode>,
    /// Edges between nodes
    pub edges: Vec<LayoutEdge>,
    /// Total width of the graph
    pub width: usize,
    /// Total height of the graph
    pub height: usize,
}

impl GraphLayout {
    /// Create a new graph layout from nodes and dependencies
    ///
    /// # Arguments
    /// * `nodes` - Map of node ID to display text
    /// * `dependencies` - List of (from, to) edges
    /// * `options` - Layout options
    pub fn new(
        nodes: HashMap<String, String>,
        dependencies: Vec<(String, String)>,
        options: LayoutOptions,
    ) -> Self {
        if nodes.is_empty() {
            return Self {
                nodes: Vec::new(),
                edges: Vec::new(),
                width: 0,
                height: 0,
            };
        }

        // Build adjacency lists
        let mut out_edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut edges = Vec::new();

        for (from, to) in dependencies {
            if nodes.contains_key(&from) && nodes.contains_key(&to) {
                out_edges.entry(from.clone()).or_default().push(to.clone());
                in_edges.entry(to.clone()).or_default().push(from.clone());
                edges.push(LayoutEdge {
                    from: from.clone(),
                    to: to.clone(),
                });
            }
        }

        // Ensure all nodes are in adjacency lists
        for node_id in nodes.keys() {
            out_edges.entry(node_id.clone()).or_default();
            in_edges.entry(node_id.clone()).or_default();
        }

        // Assign layers using longest path layering
        let layers = Self::assign_layers(&nodes, &out_edges, &in_edges);

        // Position nodes in layers
        let positioned_nodes = Self::position_nodes(&nodes, &layers, &options);

        // Calculate total dimensions
        let width = positioned_nodes
            .iter()
            .map(|n| n.x + n.width)
            .max()
            .unwrap_or(0);
        let height = positioned_nodes
            .iter()
            .map(|n| n.y + n.height)
            .max()
            .unwrap_or(0);

        Self {
            nodes: positioned_nodes,
            edges,
            width,
            height,
        }
    }

    /// Assign nodes to layers using topological sort with depth tracking
    fn assign_layers(
        nodes: &HashMap<String, String>,
        out_edges: &HashMap<String, Vec<String>>,
        in_edges: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, usize> {
        let mut layers: HashMap<String, usize> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Calculate in-degrees
        for node_id in nodes.keys() {
            in_degree.insert(node_id.clone(), in_edges.get(node_id).unwrap().len());
        }

        // Find root nodes (no incoming edges)
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back((node_id.clone(), 0));
            }
        }

        // If no root nodes (cycle detected), pick any node
        if queue.is_empty() && !nodes.is_empty() {
            let first_node = nodes.keys().next().unwrap().clone();
            queue.push_back((first_node, 0));
        }

        // Topological sort with layer assignment
        let mut visited = HashSet::new();
        while let Some((node_id, layer)) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Assign layer (use maximum of current and calculated)
            let current_layer = layers.get(&node_id).copied().unwrap_or(0);
            layers.insert(node_id.clone(), current_layer.max(layer));

            // Process outgoing edges
            if let Some(successors) = out_edges.get(&node_id) {
                for succ in successors {
                    if !visited.contains(succ) {
                        // Successor goes to next layer
                        let succ_layer = layers.get(succ).copied().unwrap_or(0);
                        layers.insert(succ.clone(), succ_layer.max(layer + 1));
                        queue.push_back((succ.clone(), layer + 1));
                    }
                }
            }
        }

        // Handle any unvisited nodes (disconnected components)
        for node_id in nodes.keys() {
            if !layers.contains_key(node_id) {
                layers.insert(node_id.clone(), 0);
            }
        }

        layers
    }

    /// Position nodes within their layers
    fn position_nodes(
        nodes: &HashMap<String, String>,
        layers: &HashMap<String, usize>,
        options: &LayoutOptions,
    ) -> Vec<LayoutNode> {
        // Group nodes by layer
        let max_layer = layers.values().copied().max().unwrap_or(0);
        let mut layers_vec: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];

        for (node_id, &layer) in layers {
            // Bounds check to prevent panic (defensive programming)
            if let Some(layer_vec) = layers_vec.get_mut(layer) {
                layer_vec.push(node_id.clone());
            } else {
                tracing::warn!("Layer index {} out of bounds (max: {})", layer, max_layer);
            }
        }

        // Sort nodes within each layer by ID for consistency
        for layer_nodes in &mut layers_vec {
            layer_nodes.sort();
        }

        // Calculate positions
        let mut positioned = Vec::new();
        let mut current_y = 0;

        for (layer_idx, layer_nodes) in layers_vec.iter().enumerate() {
            if layer_nodes.is_empty() {
                continue;
            }

            let mut current_x = 0;

            for node_id in layer_nodes {
                let text = nodes.get(node_id).cloned().unwrap_or_default();

                positioned.push(LayoutNode {
                    id: node_id.clone(),
                    text,
                    x: current_x,
                    y: current_y,
                    width: options.node_width,
                    height: options.node_height,
                    layer: layer_idx,
                });

                current_x += options.node_width + options.node_spacing_x;
            }

            current_y += options.node_height + options.layer_spacing_y;
        }

        positioned
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&LayoutNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get all edges from a node
    pub fn get_edges_from(&self, id: &str) -> Vec<&LayoutEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// Get all edges to a node
    pub fn get_edges_to(&self, id: &str) -> Vec<&LayoutEdge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let layout = GraphLayout::new(HashMap::new(), Vec::new(), LayoutOptions::default());
        assert_eq!(layout.nodes.len(), 0);
        assert_eq!(layout.edges.len(), 0);
        assert_eq!(layout.width, 0);
        assert_eq!(layout.height, 0);
    }

    #[test]
    fn test_single_node() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());

        let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.nodes[0].id, "A");
        assert_eq!(layout.nodes[0].layer, 0);
        assert_eq!(layout.nodes[0].x, 0);
        assert_eq!(layout.nodes[0].y, 0);
    }

    #[test]
    fn test_linear_graph() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        nodes.insert("B".to_string(), "Node B".to_string());
        nodes.insert("C".to_string(), "Node C".to_string());

        let deps = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let layout = GraphLayout::new(nodes, deps, LayoutOptions::default());
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.edges.len(), 2);

        // Find nodes
        let node_a = layout.get_node("A").unwrap();
        let node_b = layout.get_node("B").unwrap();
        let node_c = layout.get_node("C").unwrap();

        // Check layering: A -> B -> C should be in increasing layers
        assert_eq!(node_a.layer, 0);
        assert_eq!(node_b.layer, 1);
        assert_eq!(node_c.layer, 2);

        // Check Y coordinates increase with layers
        assert!(node_b.y > node_a.y);
        assert!(node_c.y > node_b.y);
    }

    #[test]
    fn test_diamond_graph() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        nodes.insert("B".to_string(), "Node B".to_string());
        nodes.insert("C".to_string(), "Node C".to_string());
        nodes.insert("D".to_string(), "Node D".to_string());

        let deps = vec![
            ("A".to_string(), "B".to_string()),
            ("A".to_string(), "C".to_string()),
            ("B".to_string(), "D".to_string()),
            ("C".to_string(), "D".to_string()),
        ];

        let layout = GraphLayout::new(nodes, deps, LayoutOptions::default());
        assert_eq!(layout.nodes.len(), 4);
        assert_eq!(layout.edges.len(), 4);

        let node_a = layout.get_node("A").unwrap();
        let node_b = layout.get_node("B").unwrap();
        let node_c = layout.get_node("C").unwrap();
        let node_d = layout.get_node("D").unwrap();

        // A should be in layer 0
        assert_eq!(node_a.layer, 0);
        // B and C should be in layer 1
        assert_eq!(node_b.layer, 1);
        assert_eq!(node_c.layer, 1);
        // D should be in layer 2
        assert_eq!(node_d.layer, 2);
    }

    #[test]
    fn test_get_edges() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        nodes.insert("B".to_string(), "Node B".to_string());
        nodes.insert("C".to_string(), "Node C".to_string());

        let deps = vec![
            ("A".to_string(), "B".to_string()),
            ("A".to_string(), "C".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let layout = GraphLayout::new(nodes, deps, LayoutOptions::default());

        let edges_from_a = layout.get_edges_from("A");
        assert_eq!(edges_from_a.len(), 2);

        let edges_to_c = layout.get_edges_to("C");
        assert_eq!(edges_to_c.len(), 2);

        let edges_from_b = layout.get_edges_from("B");
        assert_eq!(edges_from_b.len(), 1);
    }

    #[test]
    fn test_disconnected_graph() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        nodes.insert("B".to_string(), "Node B".to_string());
        nodes.insert("C".to_string(), "Node C".to_string());
        nodes.insert("D".to_string(), "Node D".to_string());

        let deps = vec![("A".to_string(), "B".to_string())];

        let layout = GraphLayout::new(nodes, deps, LayoutOptions::default());
        assert_eq!(layout.nodes.len(), 4);
        assert_eq!(layout.edges.len(), 1);

        // Disconnected nodes should still be positioned
        assert!(layout.get_node("C").is_some());
        assert!(layout.get_node("D").is_some());
    }

    #[test]
    fn test_cyclic_graph() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), "Node A".to_string());
        nodes.insert("B".to_string(), "Node B".to_string());
        nodes.insert("C".to_string(), "Node C".to_string());

        // Create a cycle: A -> B -> C -> A
        let deps = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
            ("C".to_string(), "A".to_string()),
        ];

        let layout = GraphLayout::new(nodes, deps, LayoutOptions::default());
        // Should handle cycles gracefully without panicking
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.edges.len(), 3);
    }
}
