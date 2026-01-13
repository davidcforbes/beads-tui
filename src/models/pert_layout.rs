//! PERT chart layout model with DAG computation and critical path analysis

use crate::beads::models::{Issue, IssueStatus};
use std::collections::{HashMap, HashSet, VecDeque};

/// PERT node representing an issue in the dependency graph
#[derive(Debug, Clone)]
pub struct PertNode {
    /// Issue ID
    pub issue_id: String,
    /// Issue title for display
    pub title: String,
    /// Issue status
    pub status: IssueStatus,
    /// Duration in hours (from estimate or default)
    pub duration: f64,
    /// Earliest start time
    pub earliest_start: f64,
    /// Earliest finish time
    pub earliest_finish: f64,
    /// Latest start time
    pub latest_start: f64,
    /// Latest finish time
    pub latest_finish: f64,
    /// Slack time (float)
    pub slack: f64,
    /// Whether this node is on the critical path
    pub is_critical: bool,
    /// Layout coordinates
    pub x: u16,
    pub y: u16,
}

/// PERT edge representing a dependency between issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PertEdge {
    /// From issue ID (blocker)
    pub from: String,
    /// To issue ID (blocked)
    pub to: String,
}

/// Cycle detection result
#[derive(Debug, Clone)]
pub struct CycleDetection {
    /// Whether a cycle was detected
    pub has_cycle: bool,
    /// Edges that form cycles
    pub cycle_edges: Vec<PertEdge>,
}

/// PERT graph structure
#[derive(Debug, Clone)]
pub struct PertGraph {
    /// Nodes indexed by issue ID
    pub nodes: HashMap<String, PertNode>,
    /// Adjacency list: issue_id -> list of dependent issue IDs
    pub adjacency: HashMap<String, Vec<String>>,
    /// Reverse adjacency list: issue_id -> list of blocker issue IDs
    pub reverse_adjacency: HashMap<String, Vec<String>>,
    /// All edges in the graph
    pub edges: Vec<PertEdge>,
    /// Topological order of nodes (empty if cycle detected)
    pub topological_order: Vec<String>,
    /// Critical path (sequence of issue IDs)
    pub critical_path: Vec<String>,
    /// Cycle detection result
    pub cycle_detection: CycleDetection,
}

impl PertGraph {
    /// Create a new PERT graph from issues
    /// default_duration: default duration in hours if estimate is not available
    pub fn new(issues: &[Issue], default_duration: f64) -> Self {
        let mut nodes = HashMap::new();
        let mut adjacency = HashMap::new();
        let mut reverse_adjacency = HashMap::new();
        let mut edges = Vec::new();

        // Build nodes
        for issue in issues {
            // Use default duration for all nodes
            // In the future, could derive from beads metadata or custom fields
            let duration = default_duration;

            let node = PertNode {
                issue_id: issue.id.clone(),
                title: issue.title.clone(),
                status: issue.status,
                duration,
                earliest_start: 0.0,
                earliest_finish: 0.0,
                latest_start: 0.0,
                latest_finish: 0.0,
                slack: 0.0,
                is_critical: false,
                x: 0,
                y: 0,
            };

            nodes.insert(issue.id.clone(), node);
            adjacency.insert(issue.id.clone(), Vec::new());
            reverse_adjacency.insert(issue.id.clone(), Vec::new());
        }

        // Build edges from dependencies
        // dependencies: list of issues this issue depends on (blockers)
        for issue in issues {
            for dep in &issue.dependencies {
                // issue depends on dep, so dep blocks issue
                // Edge: dep -> issue
                if nodes.contains_key(dep) {
                    adjacency
                        .entry(dep.clone())
                        .or_insert_with(Vec::new)
                        .push(issue.id.clone());

                    reverse_adjacency
                        .entry(issue.id.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());

                    edges.push(PertEdge {
                        from: dep.clone(),
                        to: issue.id.clone(),
                    });
                }
            }
        }

        let mut graph = Self {
            nodes,
            adjacency,
            reverse_adjacency,
            edges,
            topological_order: Vec::new(),
            critical_path: Vec::new(),
            cycle_detection: CycleDetection {
                has_cycle: false,
                cycle_edges: Vec::new(),
            },
        };

        // Detect cycles
        graph.detect_cycles();

        // Only compute timing and layout if no cycles
        if !graph.cycle_detection.has_cycle {
            graph.compute_topological_order();
            graph.compute_timing();
            graph.compute_critical_path();
            graph.compute_layout();
        }

        graph
    }

    /// Detect cycles in the graph using DFS
    fn detect_cycles(&mut self) {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycle_edges = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_detect_cycle(
                    node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut cycle_edges,
                );
            }
        }

        self.cycle_detection = CycleDetection {
            has_cycle: !cycle_edges.is_empty(),
            cycle_edges,
        };
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycle(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        cycle_edges: &mut Vec<PertEdge>,
    ) {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        if let Some(neighbors) = self.adjacency.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_detect_cycle(neighbor, visited, rec_stack, cycle_edges);
                } else if rec_stack.contains(neighbor) {
                    // Found a back edge (cycle)
                    cycle_edges.push(PertEdge {
                        from: node_id.to_string(),
                        to: neighbor.clone(),
                    });
                }
            }
        }

        rec_stack.remove(node_id);
    }

    /// Compute topological order using Kahn's algorithm
    fn compute_topological_order(&mut self) {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut order = Vec::new();

        // Calculate in-degrees
        for node_id in self.nodes.keys() {
            in_degree.insert(
                node_id.clone(),
                self.reverse_adjacency.get(node_id).map_or(0, |v| v.len()),
            );
        }

        // Add nodes with no dependencies to queue
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // Process queue
        while let Some(node_id) = queue.pop_front() {
            order.push(node_id.clone());

            if let Some(neighbors) = self.adjacency.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        self.topological_order = order;
    }

    /// Compute earliest and latest start/finish times
    fn compute_timing(&mut self) {
        // Forward pass: compute earliest start/finish
        for node_id in &self.topological_order.clone() {
            let mut max_earliest_finish: f64 = 0.0;

            // Find the maximum earliest finish of all predecessors
            if let Some(predecessors) = self.reverse_adjacency.get(node_id) {
                for pred in predecessors {
                    if let Some(pred_node) = self.nodes.get(pred) {
                        max_earliest_finish = max_earliest_finish.max(pred_node.earliest_finish);
                    }
                }
            }

            // Update node timing
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.earliest_start = max_earliest_finish;
                node.earliest_finish = node.earliest_start + node.duration;
            }
        }

        // Find project completion time (max earliest finish)
        let project_completion = self
            .nodes
            .values()
            .map(|n| n.earliest_finish)
            .fold(0.0, f64::max);

        // Backward pass: compute latest start/finish
        // Initialize all nodes with project completion time
        for node in self.nodes.values_mut() {
            node.latest_finish = project_completion;
            node.latest_start = node.latest_finish - node.duration;
        }

        // Process in reverse topological order
        for node_id in self.topological_order.iter().rev() {
            let mut min_latest_start = project_completion;

            // Find the minimum latest start of all successors
            if let Some(successors) = self.adjacency.get(node_id) {
                for succ in successors {
                    if let Some(succ_node) = self.nodes.get(succ) {
                        min_latest_start = min_latest_start.min(succ_node.latest_start);
                    }
                }
            }

            // Update node timing and slack
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.latest_finish = min_latest_start;
                node.latest_start = node.latest_finish - node.duration;
                node.slack = node.latest_start - node.earliest_start;
            }
        }
    }

    /// Compute critical path (nodes with zero slack)
    fn compute_critical_path(&mut self) {
        let critical_nodes: Vec<String> = self
            .nodes
            .iter_mut()
            .filter_map(|(id, node)| {
                let is_critical = node.slack.abs() < 0.001; // floating point tolerance
                node.is_critical = is_critical;
                if is_critical {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        // Build critical path in topological order
        self.critical_path = self
            .topological_order
            .iter()
            .filter(|id| critical_nodes.contains(id))
            .cloned()
            .collect();
    }

    /// Compute layout coordinates for visualization
    /// Uses time-based X axis and lane-based Y axis
    fn compute_layout(&mut self) {
        // Determine time range
        let max_time = self
            .nodes
            .values()
            .map(|n| n.earliest_finish)
            .fold(0.0, f64::max);

        // Group nodes into time buckets based on earliest start
        let bucket_size = (max_time / 20.0).max(1.0); // Aim for ~20 buckets
        let mut time_buckets: HashMap<u16, Vec<String>> = HashMap::new();

        for (id, node) in &self.nodes {
            let bucket = (node.earliest_start / bucket_size) as u16;
            time_buckets
                .entry(bucket)
                .or_default()
                .push(id.clone());
        }

        // Assign Y positions within each bucket to avoid overlap
        let mut lane_assignments: HashMap<u16, HashSet<u16>> = HashMap::new();

        for node in self.nodes.values_mut() {
            let bucket = (node.earliest_start / bucket_size) as u16;
            node.x = bucket * 4; // Horizontal spacing

            // Find available lane (Y position) in this bucket
            let used_lanes = lane_assignments.entry(bucket).or_default();
            let mut lane = 0;
            while used_lanes.contains(&lane) {
                lane += 1;
            }
            used_lanes.insert(lane);
            node.y = lane * 3; // Vertical spacing
        }
    }

    /// Get nodes in topological order
    pub fn nodes_in_order(&self) -> Vec<&PertNode> {
        self.topological_order
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Get critical path nodes
    pub fn critical_path_nodes(&self) -> Vec<&PertNode> {
        self.critical_path
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Compute a subgraph focused on a specific node
    /// Returns nodes and edges within the specified depth
    /// direction: "upstream" (dependencies), "downstream" (blocks), "both"
    pub fn compute_subgraph(
        &self,
        focus_node: &str,
        depth: usize,
        direction: &str,
    ) -> (HashSet<String>, Vec<PertEdge>) {
        let mut included_nodes = HashSet::new();

        if !self.nodes.contains_key(focus_node) {
            return (included_nodes, Vec::new());
        }

        // Start with the focus node
        included_nodes.insert(focus_node.to_string());

        // BFS to find nodes within depth
        match direction {
            "upstream" => {
                self.traverse_direction(&mut included_nodes, focus_node, depth, true);
            }
            "downstream" => {
                self.traverse_direction(&mut included_nodes, focus_node, depth, false);
            }
            "both" => {
                self.traverse_direction(&mut included_nodes, focus_node, depth, true);
                self.traverse_direction(&mut included_nodes, focus_node, depth, false);
            }
            _ => {}
        }

        // Filter edges to only include those between included nodes
        let filtered_edges: Vec<PertEdge> = self
            .edges
            .iter()
            .filter(|edge| included_nodes.contains(&edge.from) && included_nodes.contains(&edge.to))
            .cloned()
            .collect();

        (included_nodes, filtered_edges)
    }

    /// Helper to traverse the graph in a specific direction (upstream or downstream)
    fn traverse_direction(
        &self,
        included_nodes: &mut HashSet<String>,
        start_node: &str,
        depth: usize,
        upstream: bool,
    ) {
        let mut queue = VecDeque::new();
        queue.push_back((start_node.to_string(), 0));

        while let Some((node_id, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }

            let neighbors = if upstream {
                self.reverse_adjacency.get(&node_id)
            } else {
                self.adjacency.get(&node_id)
            };

            if let Some(neighbors) = neighbors {
                for neighbor in neighbors {
                    if included_nodes.insert(neighbor.clone()) {
                        queue.push_back((neighbor.clone(), current_depth + 1));
                    }
                }
            }
        }
    }

    /// Create a filtered view of the graph based on node IDs
    /// Returns a new PertGraph containing only the specified nodes and their edges
    pub fn filter_by_nodes(&self, node_ids: &HashSet<String>) -> Self {
        let mut filtered_graph = Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
            critical_path: Vec::new(),
            cycle_detection: CycleDetection {
                has_cycle: false,
                cycle_edges: Vec::new(),
            },
        };

        // Copy nodes
        for (id, node) in &self.nodes {
            if node_ids.contains(id) {
                filtered_graph.nodes.insert(id.clone(), node.clone());
            }
        }

        // Copy edges
        for edge in &self.edges {
            if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
                filtered_graph.edges.push(edge.clone());

                // Update adjacency lists
                filtered_graph
                    .adjacency
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());

                filtered_graph
                    .reverse_adjacency
                    .entry(edge.to.clone())
                    .or_default()
                    .push(edge.from.clone());
            }
        }

        // Filter topological order
        filtered_graph.topological_order = self
            .topological_order
            .iter()
            .filter(|id| node_ids.contains(id.as_str()))
            .cloned()
            .collect();

        // Filter critical path
        filtered_graph.critical_path = self
            .critical_path
            .iter()
            .filter(|id| node_ids.contains(id.as_str()))
            .cloned()
            .collect();

        filtered_graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueType, Priority};
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
    fn test_pert_graph_simple_chain() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Chain: A -> B -> C
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0); // 1 hour duration

        // Check topological order
        assert_eq!(graph.topological_order.len(), 3);
        assert_eq!(graph.topological_order[0], "A");
        assert_eq!(graph.topological_order[1], "B");
        assert_eq!(graph.topological_order[2], "C");

        // Check timing (all have 1 hour duration)
        let node_a = graph.nodes.get("A").unwrap();
        assert_eq!(node_a.earliest_start, 0.0);
        assert_eq!(node_a.earliest_finish, 1.0);

        let node_b = graph.nodes.get("B").unwrap();
        assert_eq!(node_b.earliest_start, 1.0);
        assert_eq!(node_b.earliest_finish, 2.0);

        let node_c = graph.nodes.get("C").unwrap();
        assert_eq!(node_c.earliest_start, 2.0);
        assert_eq!(node_c.earliest_finish, 3.0);

        // All nodes on critical path (chain)
        assert!(node_a.is_critical);
        assert!(node_b.is_critical);
        assert!(node_c.is_critical);

        assert_eq!(graph.critical_path.len(), 3);
    }

    #[test]
    fn test_pert_graph_parallel_paths() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");

        // Diamond: A -> B -> D
        //          A -> C -> D
        // With equal durations, all paths are critical
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string(), "C".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4], 1.0);

        // With equal durations, all nodes are on critical path
        let node_a = graph.nodes.get("A").unwrap();
        let node_b = graph.nodes.get("B").unwrap();
        let node_c = graph.nodes.get("C").unwrap();
        let node_d = graph.nodes.get("D").unwrap();

        assert!(node_a.is_critical);
        assert!(node_b.is_critical);
        assert!(node_c.is_critical);
        assert!(node_d.is_critical);
    }

    #[test]
    fn test_pert_graph_cycle_detection() {
        let mut issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Cycle: A -> B -> C -> A
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];
        issue1.dependencies = vec!["C".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 8.0);

        // Check cycle detection
        assert!(graph.cycle_detection.has_cycle);
        assert!(!graph.cycle_detection.cycle_edges.is_empty());

        // No topological order or timing computed
        assert!(graph.topological_order.is_empty());
    }

    #[test]
    fn test_pert_graph_no_dependencies() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let issue3 = create_test_issue("C", "Task C");

        let graph = PertGraph::new(&[issue1, issue2, issue3], 8.0);

        // All nodes can start at time 0
        for node in graph.nodes.values() {
            assert_eq!(node.earliest_start, 0.0);
        }

        // All nodes are critical (no dependencies to create slack)
        for node in graph.nodes.values() {
            assert!(node.is_critical);
        }
    }

    #[test]
    fn test_pert_graph_default_duration() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");

        let graph = PertGraph::new(&[issue1, issue2], 8.0);

        // Check default duration is used
        let node_a = graph.nodes.get("A").unwrap();
        assert_eq!(node_a.duration, 8.0);
    }

    #[test]
    fn test_pert_graph_layout_coordinates() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        issue2.dependencies = vec!["A".to_string()];

        let graph = PertGraph::new(&[issue1, issue2], 8.0);

        // Check layout coordinates are assigned
        let node_a = graph.nodes.get("A").unwrap();
        let node_b = graph.nodes.get("B").unwrap();

        // B should be positioned after A (higher x)
        assert!(node_b.x >= node_a.x);
    }

    #[test]
    fn test_critical_path_ordering() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 8.0);

        let critical_nodes = graph.critical_path_nodes();

        // Critical path should be in topological order
        assert_eq!(critical_nodes.len(), 3);
        assert_eq!(critical_nodes[0].issue_id, "A");
        assert_eq!(critical_nodes[1].issue_id, "B");
        assert_eq!(critical_nodes[2].issue_id, "C");
    }

    #[test]
    fn test_nodes_in_order() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        issue2.dependencies = vec!["A".to_string()];

        let graph = PertGraph::new(&[issue1, issue2], 8.0);

        let ordered_nodes = graph.nodes_in_order();

        assert_eq!(ordered_nodes.len(), 2);
        assert_eq!(ordered_nodes[0].issue_id, "A");
        assert_eq!(ordered_nodes[1].issue_id, "B");
    }

    #[test]
    fn test_compute_subgraph_upstream() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Chain: A -> B -> C
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Focus on C, upstream depth 1 (should include B and C)
        let (nodes, edges) = graph.compute_subgraph("C", 1, "upstream");
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains("B"));
        assert!(nodes.contains("C"));
        assert_eq!(edges.len(), 1); // B -> C

        // Focus on C, upstream depth 2 (should include A, B, and C)
        let (nodes, edges) = graph.compute_subgraph("C", 2, "upstream");
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));
        assert!(nodes.contains("C"));
        assert_eq!(edges.len(), 2); // A -> B, B -> C
    }

    #[test]
    fn test_compute_subgraph_downstream() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Chain: A -> B -> C
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Focus on A, downstream depth 1 (should include A and B)
        let (nodes, edges) = graph.compute_subgraph("A", 1, "downstream");
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));
        assert_eq!(edges.len(), 1); // A -> B

        // Focus on A, downstream depth 2 (should include A, B, and C)
        let (nodes, edges) = graph.compute_subgraph("A", 2, "downstream");
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));
        assert!(nodes.contains("C"));
        assert_eq!(edges.len(), 2); // A -> B, B -> C
    }

    #[test]
    fn test_compute_subgraph_both() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // Chain: A -> B -> C
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Focus on B, both directions depth 1 (should include all three)
        let (nodes, edges) = graph.compute_subgraph("B", 1, "both");
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));
        assert!(nodes.contains("C"));
        assert_eq!(edges.len(), 2); // A -> B, B -> C
    }

    #[test]
    fn test_filter_by_nodes() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Filter to only include A and B
        let mut filter_nodes = HashSet::new();
        filter_nodes.insert("A".to_string());
        filter_nodes.insert("B".to_string());

        let filtered = graph.filter_by_nodes(&filter_nodes);

        assert_eq!(filtered.nodes.len(), 2);
        assert!(filtered.nodes.contains_key("A"));
        assert!(filtered.nodes.contains_key("B"));
        assert!(!filtered.nodes.contains_key("C"));

        assert_eq!(filtered.edges.len(), 1); // Only A -> B
        assert_eq!(filtered.topological_order.len(), 2);
    }

    #[test]
    fn test_pert_graph_empty_issues() {
        let graph = PertGraph::new(&[], 8.0);
        
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.topological_order.len(), 0);
        assert_eq!(graph.critical_path.len(), 0);
        assert!(!graph.cycle_detection.has_cycle);
    }

    #[test]
    fn test_pert_graph_single_issue() {
        let issue = create_test_issue("A", "Single Task");
        let graph = PertGraph::new(&[issue], 5.0);
        
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.topological_order.len(), 1);
        
        let node = graph.nodes.get("A").unwrap();
        assert_eq!(node.duration, 5.0);
        assert_eq!(node.earliest_start, 0.0);
        assert_eq!(node.earliest_finish, 5.0);
        assert!(node.is_critical);
    }

    #[test]
    fn test_pert_edge_equality() {
        let edge1 = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let edge2 = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let edge3 = PertEdge {
            from: "B".to_string(),
            to: "C".to_string(),
        };
        
        assert_eq!(edge1, edge2);
        assert_ne!(edge1, edge3);
    }

    #[test]
    fn test_pert_edge_hash() {
        use std::collections::HashSet;
        
        let mut edges = HashSet::new();
        let edge1 = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let edge2 = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        
        edges.insert(edge1);
        edges.insert(edge2); // Duplicate should not be added
        
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_cycle_detection_self_loop() {
        let mut issue = create_test_issue("A", "Task A");
        issue.dependencies = vec!["A".to_string()]; // Self-loop
        
        let graph = PertGraph::new(&[issue], 1.0);
        
        assert!(graph.cycle_detection.has_cycle);
    }

    #[test]
    fn test_cycle_detection_multiple_cycles() {
        let mut issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");
        
        // Cycle: A -> B -> A
        issue2.dependencies = vec!["A".to_string()];
        issue1.dependencies = vec!["B".to_string()];
        
        // Cycle: C -> D -> C
        issue4.dependencies = vec!["C".to_string()];
        issue3.dependencies = vec!["D".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4], 1.0);
        
        assert!(graph.cycle_detection.has_cycle);
        assert!(!graph.cycle_detection.cycle_edges.is_empty());
    }

    #[test]
    fn test_slack_calculation_non_critical() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");
        
        // Diamond: A -> B -> D (1 hour each)
        //          A -> C -> D (C is 3 hours - critical path)
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string(), "C".to_string()];
        
        let mut graph = PertGraph::new(&[issue1.clone(), issue2, issue3, issue4], 1.0);
        
        // Manually set C to have 3 hour duration to create slack on B path
        if let Some(node_c) = graph.nodes.get_mut("C") {
            node_c.duration = 3.0;
        }
        
        // Recompute timing and critical path
        graph.compute_timing();
        graph.compute_critical_path();
        
        let node_b = graph.nodes.get("B").unwrap();
        // B path: A(0-1) -> B(1-2) -> D(3-4)
        // C path: A(0-1) -> C(1-4) -> D(4-5) [critical]
        // B should have slack (can delay without affecting project completion)
        assert!(node_b.slack > 0.0);
        assert!(!node_b.is_critical);
    }

    #[test]
    fn test_missing_dependency() {
        let mut issue = create_test_issue("A", "Task A");
        issue.dependencies = vec!["NONEXISTENT".to_string()];
        
        let graph = PertGraph::new(&[issue], 1.0);
        
        // Should not panic, dependency to nonexistent node is ignored
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_compute_subgraph_nonexistent_node() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        
        let (nodes, edges) = graph.compute_subgraph("NONEXISTENT", 1, "both");
        
        assert_eq!(nodes.len(), 0);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_compute_subgraph_depth_zero() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        issue2.dependencies = vec!["A".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2], 1.0);
        
        let (nodes, edges) = graph.compute_subgraph("A", 0, "downstream");
        
        // Depth 0 should only include the focus node itself
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains("A"));
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_compute_subgraph_invalid_direction() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        
        let (nodes, edges) = graph.compute_subgraph("A", 1, "invalid");
        
        // Invalid direction should only include focus node
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains("A"));
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_filter_by_nodes_empty_set() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        
        let filter_nodes = HashSet::new();
        let filtered = graph.filter_by_nodes(&filter_nodes);
        
        assert_eq!(filtered.nodes.len(), 0);
        assert_eq!(filtered.edges.len(), 0);
        assert_eq!(filtered.topological_order.len(), 0);
    }

    #[test]
    fn test_filter_by_nodes_all_nodes() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        issue2.dependencies = vec!["A".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2], 1.0);
        
        let mut filter_nodes = HashSet::new();
        filter_nodes.insert("A".to_string());
        filter_nodes.insert("B".to_string());
        
        let filtered = graph.filter_by_nodes(&filter_nodes);
        
        assert_eq!(filtered.nodes.len(), 2);
        assert_eq!(filtered.edges.len(), 1);
        assert_eq!(filtered.topological_order.len(), 2);
    }

    #[test]
    fn test_nodes_in_order_empty() {
        let graph = PertGraph::new(&[], 1.0);
        
        let ordered = graph.nodes_in_order();
        assert_eq!(ordered.len(), 0);
    }

    #[test]
    fn test_critical_path_nodes_empty() {
        let graph = PertGraph::new(&[], 1.0);
        
        let critical = graph.critical_path_nodes();
        assert_eq!(critical.len(), 0);
    }

    #[test]
    fn test_layout_multiple_nodes_same_bucket() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let issue3 = create_test_issue("C", "Task C");
        
        // All start at time 0, so they're in the same time bucket
        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        
        let node_a = graph.nodes.get("A").unwrap();
        let node_b = graph.nodes.get("B").unwrap();
        let node_c = graph.nodes.get("C").unwrap();
        
        // All should have same x (same bucket)
        assert_eq!(node_a.x, 0);
        assert_eq!(node_b.x, 0);
        assert_eq!(node_c.x, 0);
        
        // Different y positions to avoid overlap
        let y_positions = [node_a.y, node_b.y, node_c.y];
        let unique_y: HashSet<_> = y_positions.iter().collect();
        assert_eq!(unique_y.len(), 3); // All different Y positions
    }

    #[test]
    fn test_timing_complex_dag() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");
        let mut issue5 = create_test_issue("E", "Task E");
        
        // Complex DAG:
        // A -> B -> D -> E
        // A -> C -> E
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string()];
        issue5.dependencies = vec!["D".to_string(), "C".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4, issue5], 1.0);
        
        // E depends on both D and C, should start after latest finish
        let node_e = graph.nodes.get("E").unwrap();
        let node_d = graph.nodes.get("D").unwrap();
        let node_c = graph.nodes.get("C").unwrap();
        
        let max_predecessor_finish = node_d.earliest_finish.max(node_c.earliest_finish);
        assert_eq!(node_e.earliest_start, max_predecessor_finish);
    }

    #[test]
    fn test_different_durations() {
        let graph = PertGraph::new(&[], 5.0);
        
        // Verify default duration can vary
        let graph2 = PertGraph::new(&[], 10.0);
        
        // Just verify they were created with different default durations
        // (actual duration verification happens in other tests)
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph2.nodes.len(), 0);
    }

    #[test]
    fn test_adjacency_lists() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        
        // A should have 2 outgoing edges (to B and C)
        let a_adjacency = graph.adjacency.get("A").unwrap();
        assert_eq!(a_adjacency.len(), 2);
        assert!(a_adjacency.contains(&"B".to_string()));
        assert!(a_adjacency.contains(&"C".to_string()));
        
        // B should have 1 incoming edge (from A)
        let b_reverse = graph.reverse_adjacency.get("B").unwrap();
        assert_eq!(b_reverse.len(), 1);
        assert!(b_reverse.contains(&"A".to_string()));
    }

    #[test]
    fn test_pert_node_fields() {
        let issue = create_test_issue("test-123", "Test Node");
        let graph = PertGraph::new(&[issue], 8.5);
        
        let node = graph.nodes.get("test-123").unwrap();
        
        assert_eq!(node.issue_id, "test-123");
        assert_eq!(node.title, "Test Node");
        assert_eq!(node.status, IssueStatus::Open);
        assert_eq!(node.duration, 8.5);
        assert!(node.earliest_start >= 0.0);
        assert!(node.earliest_finish >= node.earliest_start);
        assert!(node.latest_start >= 0.0);
        assert!(node.latest_finish >= node.latest_start);
        assert!(node.slack >= 0.0);
    }

    #[test]
    fn test_partial_critical_path() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        
        let mut graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        
        // Make C longer to create critical path
        if let Some(node_c) = graph.nodes.get_mut("C") {
            node_c.duration = 5.0;
        }
        
        graph.compute_timing();
        graph.compute_critical_path();
        
        let critical = graph.critical_path_nodes();
        
        // A and C should be critical, B should not
        assert!(critical.iter().any(|n| n.issue_id == "A"));
        assert!(critical.iter().any(|n| n.issue_id == "C"));
        
        let node_b = graph.nodes.get("B").unwrap();
        assert!(!node_b.is_critical);
    }

    #[test]
    fn test_topological_order_multiple_start_nodes() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        
        // A and B are independent, C depends on both
        issue3.dependencies = vec!["A".to_string(), "B".to_string()];
        
        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);
        
        assert_eq!(graph.topological_order.len(), 3);
        
        // C must come after both A and B
        let a_pos = graph.topological_order.iter().position(|id| id == "A").unwrap();
        let b_pos = graph.topological_order.iter().position(|id| id == "B").unwrap();
        let c_pos = graph.topological_order.iter().position(|id| id == "C").unwrap();
        
        assert!(c_pos > a_pos);
        assert!(c_pos > b_pos);
    }

    #[test]
    fn test_pert_node_debug() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let node = graph.nodes.get("A").unwrap();

        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("PertNode"));
        assert!(debug_str.contains("Task A"));
    }

    #[test]
    fn test_pert_edge_debug() {
        let edge = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };

        let debug_str = format!("{:?}", edge);
        assert!(debug_str.contains("PertEdge"));
    }

    #[test]
    fn test_cycle_detection_debug() {
        let cycle = CycleDetection {
            has_cycle: true,
            cycle_edges: vec![],
        };

        let debug_str = format!("{:?}", cycle);
        assert!(debug_str.contains("CycleDetection"));
    }

    #[test]
    fn test_pert_graph_debug() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);

        let debug_str = format!("{:?}", graph);
        assert!(debug_str.contains("PertGraph"));
    }

    #[test]
    fn test_pert_node_clone() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);
        let node = graph.nodes.get("A").unwrap();

        let cloned = node.clone();
        assert_eq!(node.issue_id, cloned.issue_id);
        assert_eq!(node.title, cloned.title);
        assert_eq!(node.duration, cloned.duration);
    }

    #[test]
    fn test_pert_edge_clone() {
        let edge = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };

        let cloned = edge.clone();
        assert_eq!(edge, cloned);
    }

    #[test]
    fn test_cycle_detection_clone() {
        let edge = PertEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };

        let cycle = CycleDetection {
            has_cycle: true,
            cycle_edges: vec![edge.clone()],
        };

        let cloned = cycle.clone();
        assert_eq!(cycle.has_cycle, cloned.has_cycle);
        assert_eq!(cycle.cycle_edges.len(), cloned.cycle_edges.len());
    }

    #[test]
    fn test_pert_graph_clone() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);

        let cloned = graph.clone();
        assert_eq!(graph.nodes.len(), cloned.nodes.len());
        assert_eq!(graph.edges.len(), cloned.edges.len());
    }

    #[test]
    fn test_complex_dependency_graph() {
        // Create a complex graph with multiple dependency levels
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");
        let mut issue5 = create_test_issue("E", "Task E");
        let mut issue6 = create_test_issue("F", "Task F");

        // Complex dependencies:
        // A -> B, C
        // B -> D
        // C -> D, E
        // D -> F
        // E -> F
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string(), "C".to_string()];
        issue5.dependencies = vec!["C".to_string()];
        issue6.dependencies = vec!["D".to_string(), "E".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4, issue5, issue6], 1.0);

        assert_eq!(graph.nodes.len(), 6);
        assert_eq!(graph.topological_order.len(), 6);
        assert!(!graph.cycle_detection.has_cycle);

        // F should be last in topological order
        assert_eq!(graph.topological_order.last().unwrap(), "F");
    }

    #[test]
    fn test_deep_dependency_chain() {
        // Create a very deep chain: A -> B -> C -> D -> E -> F -> G -> H
        let mut issues = Vec::new();
        let nodes = ["A", "B", "C", "D", "E", "F", "G", "H"];

        for (i, &id) in nodes.iter().enumerate() {
            let mut issue = create_test_issue(id, &format!("Task {}", id));
            if i > 0 {
                issue.dependencies = vec![nodes[i - 1].to_string()];
            }
            issues.push(issue);
        }

        let graph = PertGraph::new(&issues, 1.0);

        assert_eq!(graph.nodes.len(), 8);
        assert_eq!(graph.topological_order.len(), 8);

        // Check timing for deep chain
        let last_node = graph.nodes.get("H").unwrap();
        assert_eq!(last_node.earliest_finish, 8.0); // 8 nodes * 1 hour each
    }

    #[test]
    fn test_large_parallel_graph() {
        // Create a large graph with many parallel tasks
        let issue_a = create_test_issue("A", "Start");
        let mut issues = vec![issue_a];

        // Create 20 tasks that all depend on A
        for i in 1..=20 {
            let mut issue = create_test_issue(&format!("T{}", i), &format!("Task {}", i));
            issue.dependencies = vec!["A".to_string()];
            issues.push(issue);
        }

        let graph = PertGraph::new(&issues, 1.0);

        assert_eq!(graph.nodes.len(), 21);
        assert!(!graph.cycle_detection.has_cycle);

        // All T* nodes should start after A finishes
        for i in 1..=20 {
            let node = graph.nodes.get(&format!("T{}", i)).unwrap();
            assert_eq!(node.earliest_start, 1.0);
        }
    }

    #[test]
    fn test_subgraph_with_complex_structure() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");
        let mut issue5 = create_test_issue("E", "Task E");

        // Structure: A -> B -> D
        //            A -> C -> E
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string()];
        issue5.dependencies = vec!["C".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4, issue5], 1.0);

        // Focus on B with upstream depth 2
        let (nodes, _edges) = graph.compute_subgraph("B", 2, "upstream");
        assert_eq!(nodes.len(), 2); // A and B
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));

        // Focus on D with both directions depth 2
        let (nodes, _edges) = graph.compute_subgraph("D", 2, "both");
        assert!(nodes.contains("A"));
        assert!(nodes.contains("B"));
        assert!(nodes.contains("D"));
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn test_filter_preserves_structure() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        let mut filter_nodes = HashSet::new();
        filter_nodes.insert("A".to_string());
        filter_nodes.insert("C".to_string());

        let filtered = graph.filter_by_nodes(&filter_nodes);

        assert_eq!(filtered.nodes.len(), 2);
        assert_eq!(filtered.edges.len(), 0); // No edge between A and C directly
        assert_eq!(filtered.topological_order.len(), 2);
    }

    #[test]
    fn test_critical_path_with_equal_paths() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");

        // Two equal paths: A -> B -> D and A -> C -> D
        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["B".to_string(), "C".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4], 1.0);

        // All nodes should be critical with equal durations
        assert!(graph.nodes.get("A").unwrap().is_critical);
        assert!(graph.nodes.get("B").unwrap().is_critical);
        assert!(graph.nodes.get("C").unwrap().is_critical);
        assert!(graph.nodes.get("D").unwrap().is_critical);
    }

    #[test]
    fn test_layout_with_different_start_times() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // A and B are independent, C depends on B
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 5.0);

        let node_a = graph.nodes.get("A").unwrap();
        let node_b = graph.nodes.get("B").unwrap();
        let node_c = graph.nodes.get("C").unwrap();

        // A and B should be in same time bucket (both start at 0)
        assert_eq!(node_a.x, node_b.x);

        // C should be in a different time bucket (starts at 5.0)
        assert!(node_c.x > node_a.x);
    }

    #[test]
    fn test_adjacency_with_multiple_dependencies() {
        let issue1 = create_test_issue("A", "Task A");
        let issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        // C depends on both A and B
        issue3.dependencies = vec!["A".to_string(), "B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // C should have 2 incoming edges
        let c_reverse = graph.reverse_adjacency.get("C").unwrap();
        assert_eq!(c_reverse.len(), 2);
        assert!(c_reverse.contains(&"A".to_string()));
        assert!(c_reverse.contains(&"B".to_string()));

        // A and B should each have C in their adjacency
        assert!(graph.adjacency.get("A").unwrap().contains(&"C".to_string()));
        assert!(graph.adjacency.get("B").unwrap().contains(&"C".to_string()));
    }

    #[test]
    fn test_cycle_in_partial_graph() {
        // Create a graph where only part has a cycle
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");
        let mut issue4 = create_test_issue("D", "Task D");

        // A -> B (valid)
        // C -> D -> C (cycle)
        issue2.dependencies = vec!["A".to_string()];
        issue4.dependencies = vec!["C".to_string()];
        issue3.dependencies = vec!["D".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3, issue4], 1.0);

        assert!(graph.cycle_detection.has_cycle);
        assert!(!graph.cycle_detection.cycle_edges.is_empty());
    }

    #[test]
    fn test_compute_subgraph_large_depth() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");
        let mut issue3 = create_test_issue("C", "Task C");

        issue2.dependencies = vec!["A".to_string()];
        issue3.dependencies = vec!["B".to_string()];

        let graph = PertGraph::new(&[issue1, issue2, issue3], 1.0);

        // Very large depth should include all reachable nodes
        let (nodes, edges) = graph.compute_subgraph("A", 100, "downstream");
        assert_eq!(nodes.len(), 3); // All nodes reachable
        assert_eq!(edges.len(), 2); // All edges
    }

    #[test]
    fn test_node_coordinates_assignment() {
        let issue1 = create_test_issue("A", "Task A");
        let mut issue2 = create_test_issue("B", "Task B");

        issue2.dependencies = vec!["A".to_string()];

        let graph = PertGraph::new(&[issue1, issue2], 10.0);

        let node_a = graph.nodes.get("A").unwrap();
        let node_b = graph.nodes.get("B").unwrap();

        // All nodes should have coordinates assigned
        assert!(node_a.x < u16::MAX);
        assert!(node_a.y < u16::MAX);
        assert!(node_b.x < u16::MAX);
        assert!(node_b.y < u16::MAX);
    }

    #[test]
    fn test_empty_graph_operations() {
        let graph = PertGraph::new(&[], 1.0);

        let ordered = graph.nodes_in_order();
        let critical = graph.critical_path_nodes();
        let (nodes, edges) = graph.compute_subgraph("A", 1, "both");

        assert_eq!(ordered.len(), 0);
        assert_eq!(critical.len(), 0);
        assert_eq!(nodes.len(), 0);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_filter_with_nonexistent_nodes() {
        let issue = create_test_issue("A", "Task A");
        let graph = PertGraph::new(&[issue], 1.0);

        let mut filter_nodes = HashSet::new();
        filter_nodes.insert("A".to_string());
        filter_nodes.insert("NONEXISTENT".to_string());

        let filtered = graph.filter_by_nodes(&filter_nodes);

        assert_eq!(filtered.nodes.len(), 1); // Only A
        assert!(filtered.nodes.contains_key("A"));
    }
}
