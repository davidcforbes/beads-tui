use crate::common::TestHarness;
use beads_tui::graph::{GraphLayout, GraphRenderer, LayoutOptions, RenderOptions};
use beads_tui::ui::views::{DependencyGraphState, DependencyGraphView};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use std::collections::HashMap;
use std::process::Command;

#[tokio::test]
async fn test_dependency_cycle_detection() {
    let harness = TestHarness::new();
    harness.init().await;

    // Create cycle: A -> B -> C -> A
    let root = harness.root.path();

    let create = |title: &str| {
        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .expect("Failed to create issue");
        let s = String::from_utf8_lossy(&output.stdout);
        // Extract ID
        s.split_whitespace()
            .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
            .unwrap()
            .trim_end_matches(':')
            .to_string()
    };

    let id_a = create("Task A");
    let id_b = create("Task B");
    let id_c = create("Task C");

    let add_dep = |from: &str, to: &str| {
        Command::new("bd")
            .args(["dep", "add", from, to])
            .current_dir(root)
            .status()
            .expect("Failed to add dependency");
    };

    add_dep(&id_a, &id_b);
    add_dep(&id_b, &id_c);
    add_dep(&id_c, &id_a); // Cycle!

    // Load into state
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    // Check if app logic detects cycle (assuming we have a cycle detection helper)
    // For now, we verify that we can at least list them without crashing.
    assert_eq!(issues.len(), 3);
}

#[tokio::test]
async fn test_deep_dependency_chain() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    let mut prev_id: Option<String> = None;
    for i in 0..20 {
        let title = format!("Chain Task {}", i);
        let id = Command::new("bd")
            .args([
                "create",
                "--title",
                &title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .find(|p| p.contains(".tmp") || p.contains("beads-"))
                    .unwrap()
                    .trim_end_matches(':')
                    .to_string()
            })
            .expect("Failed to create issue");

        if let Some(p) = prev_id {
            Command::new("bd")
                .args(["dep", "add", &p, &id])
                .current_dir(root)
                .status()
                .unwrap();
        }
        prev_id = Some(id);
    }

    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list chain");
    assert_eq!(issues.len(), 20);
}

// Edge case tests for dependency graph layout and rendering

#[test]
fn test_empty_graph_layout() {
    // Test with no nodes
    let nodes = HashMap::new();
    let dependencies = Vec::new();
    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    assert_eq!(layout.nodes.len(), 0);
    assert_eq!(layout.edges.len(), 0);
    assert_eq!(layout.width, 0);
    assert_eq!(layout.height, 0);
}

#[test]
fn test_single_node_graph() {
    // Test with one isolated node
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Single Node".to_string());
    let dependencies = Vec::new();

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    assert_eq!(layout.nodes.len(), 1);
    assert_eq!(layout.edges.len(), 0);
    assert!(layout.width > 0);
    assert!(layout.height > 0);

    let node = &layout.nodes[0];
    assert_eq!(node.id, "A");
    assert!(node.width > 0);
    assert!(node.height > 0);
}

#[test]
fn test_disconnected_components() {
    // Test with multiple disconnected components
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Component 1 Node A".to_string());
    nodes.insert("B".to_string(), "Component 1 Node B".to_string());
    nodes.insert("C".to_string(), "Component 2 Node C".to_string());
    nodes.insert("D".to_string(), "Component 2 Node D".to_string());

    let dependencies = vec![
        ("A".to_string(), "B".to_string()),
        ("C".to_string(), "D".to_string()),
    ];

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    // All nodes should be positioned
    assert_eq!(layout.nodes.len(), 4);
    assert_eq!(layout.edges.len(), 2);
}

#[test]
fn test_very_wide_graph() {
    // Test with many nodes at the same level (>100 children)
    let mut nodes = HashMap::new();
    let mut dependencies = Vec::new();

    // Create root node
    nodes.insert("root".to_string(), "Root".to_string());

    // Create 150 child nodes
    for i in 0..150 {
        let child_id = format!("child_{}", i);
        nodes.insert(child_id.clone(), format!("Child {}", i));
        dependencies.push(("root".to_string(), child_id));
    }

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    assert_eq!(layout.nodes.len(), 151); // 1 root + 150 children
    assert_eq!(layout.edges.len(), 150);
    assert!(layout.width > 0, "Very wide graph should have positive width");
}

#[test]
fn test_deep_hierarchy_graph() {
    // Test with deep hierarchy (>100 levels)
    let mut nodes = HashMap::new();
    let mut dependencies = Vec::new();

    // Create chain of 120 nodes
    for i in 0..120 {
        let node_id = format!("node_{}", i);
        nodes.insert(node_id.clone(), format!("Node {}", i));

        if i > 0 {
            let parent_id = format!("node_{}", i - 1);
            dependencies.push((parent_id, node_id));
        }
    }

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    assert_eq!(layout.nodes.len(), 120);
    assert_eq!(layout.edges.len(), 119);
    assert!(layout.height > 0, "Deep hierarchy should have positive height");
}

#[test]
fn test_cyclic_graph_rendering() {
    // Test that cyclic dependencies don't cause infinite loops
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Node A".to_string());
    nodes.insert("B".to_string(), "Node B".to_string());
    nodes.insert("C".to_string(), "Node C".to_string());

    // Create cycle: A -> B -> C -> A
    let dependencies = vec![
        ("A".to_string(), "B".to_string()),
        ("B".to_string(), "C".to_string()),
        ("C".to_string(), "A".to_string()),
    ];

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    // Should complete without hanging
    assert_eq!(layout.nodes.len(), 3);
    assert_eq!(layout.edges.len(), 3);
}

#[test]
fn test_self_referencing_node() {
    // Test node that depends on itself
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Self-referencing".to_string());

    let dependencies = vec![("A".to_string(), "A".to_string())];

    let layout = GraphLayout::new(nodes, dependencies, LayoutOptions::default());

    // Should handle gracefully
    assert_eq!(layout.nodes.len(), 1);
}

#[test]
fn test_graph_renderer_empty() {
    // Test rendering empty graph doesn't crash
    let layout = GraphLayout::new(HashMap::new(), Vec::new(), LayoutOptions::default());
    let renderer = GraphRenderer::new(layout);

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    let options = RenderOptions::default();

    // Should not panic
    renderer.render(area, &mut buf, &options);
}

#[test]
fn test_graph_renderer_with_offset() {
    // Test rendering with large viewport offsets
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Node A".to_string());

    let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());
    let renderer = GraphRenderer::new(layout);

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    // Large negative offset (scrolled far right/down)
    let options = RenderOptions {
        offset_x: -1000,
        offset_y: -1000,
        ..Default::default()
    };

    // Should not crash with nodes off-screen
    renderer.render(area, &mut buf, &options);
}

#[test]
fn test_graph_renderer_tiny_viewport() {
    // Test rendering to very small viewport
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Node A with long text".to_string());

    let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());
    let renderer = GraphRenderer::new(layout);

    let tiny_area = Rect::new(0, 0, 5, 3);
    let mut buf = Buffer::empty(tiny_area);
    let options = RenderOptions::default();

    // Should not crash with tiny viewport
    renderer.render(tiny_area, &mut buf, &options);
}

#[test]
fn test_dependency_graph_view_empty_issues() {
    // Test DependencyGraphView with empty issues list
    let issues = vec![];
    let view = DependencyGraphView::new(&issues);
    let mut state = DependencyGraphState::new();

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    // Should not panic
    view.render(area, &mut buf, &mut state);
}

#[test]
fn test_dependency_graph_state_navigation() {
    // Test navigation through nodes
    let mut state = DependencyGraphState::new();

    // Update with some nodes
    let node_ids = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    state.update_nodes(node_ids);

    // Should auto-select first node
    assert_eq!(state.selected_node(), Some("A"));

    // Navigate next
    state.select_next();
    assert_eq!(state.selected_node(), Some("B"));

    state.select_next();
    assert_eq!(state.selected_node(), Some("C"));

    // Should wrap around
    state.select_next();
    assert_eq!(state.selected_node(), Some("A"));

    // Navigate previous
    state.select_previous();
    assert_eq!(state.selected_node(), Some("C"));
}

#[test]
fn test_dependency_graph_panning() {
    // Test panning operations
    let mut state = DependencyGraphState::new();

    assert_eq!(state.offset(), (0, 0));

    state.pan_right(10);
    assert_eq!(state.offset(), (10, 0));

    state.pan_down(20);
    assert_eq!(state.offset(), (10, 20));

    state.pan_left(5);
    assert_eq!(state.offset(), (5, 20));

    state.pan_up(10);
    assert_eq!(state.offset(), (5, 10));

    state.reset_view();
    assert_eq!(state.offset(), (0, 0));
}

#[test]
fn test_large_node_labels() {
    // Test with very long node labels
    let mut nodes = HashMap::new();
    let long_label = "A".repeat(500); // 500 character label
    nodes.insert("A".to_string(), long_label);

    let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());

    // Should handle without crashing
    assert_eq!(layout.nodes.len(), 1);
    let node = &layout.nodes[0];
    // Width should be reasonable even with long label
    assert!(node.width < 200, "Node width should be capped");
}

#[test]
fn test_special_characters_in_labels() {
    // Test with special characters and Unicode
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), "Node with 中文 and émojis 🎉".to_string());
    nodes.insert("B".to_string(), "Path/with\\special:chars".to_string());

    let layout = GraphLayout::new(nodes, Vec::new(), LayoutOptions::default());

    assert_eq!(layout.nodes.len(), 2);
    // Should render without crashing
}
