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
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Assign Y positions within each bucket to avoid overlap
        let mut lane_assignments: HashMap<u16, HashSet<u16>> = HashMap::new();

        for (_id, node) in &mut self.nodes {
            let bucket = (node.earliest_start / bucket_size) as u16;
            node.x = bucket * 4; // Horizontal spacing

            // Find available lane (Y position) in this bucket
            let used_lanes = lane_assignments.entry(bucket).or_insert_with(HashSet::new);
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
}
