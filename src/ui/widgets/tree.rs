//! Tree widget with expand/collapse functionality

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};
use std::fmt::Display;

/// A node in the tree
#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    /// The data for this node
    pub data: T,
    /// Child nodes
    pub children: Vec<TreeNode<T>>,
    /// Unique identifier for this node (used for tracking expansion state)
    pub id: String,
}

impl<T> TreeNode<T> {
    /// Create a new tree node
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            id: id.into(),
        }
    }

    /// Create a new tree node with children
    pub fn with_children(id: impl Into<String>, data: T, children: Vec<TreeNode<T>>) -> Self {
        Self {
            data,
            children,
            id: id.into(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// State for the tree widget
#[derive(Debug, Clone)]
pub struct TreeState {
    /// Set of expanded node IDs
    expanded: std::collections::HashSet<String>,
    /// List state for selection and scrolling
    list_state: ListState,
    /// Flattened list of visible items (updated on render)
    visible_items: Vec<VisibleItem>,
}

/// A visible item in the flattened tree
#[derive(Debug, Clone)]
struct VisibleItem {
    /// Node ID
    id: String,
    /// Display text
    text: String,
    /// Indentation level
    level: usize,
    /// Whether this node has children
    has_children: bool,
    /// Whether this node is expanded
    is_expanded: bool,
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeState {
    /// Create a new tree state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            expanded: std::collections::HashSet::new(),
            list_state,
            visible_items: Vec::new(),
        }
    }

    /// Toggle expansion of a node by ID
    pub fn toggle_expansion(&mut self, node_id: &str) {
        if self.expanded.contains(node_id) {
            self.expanded.remove(node_id);
        } else {
            self.expanded.insert(node_id.to_string());
        }
    }

    /// Check if a node is expanded
    pub fn is_expanded(&self, node_id: &str) -> bool {
        self.expanded.contains(node_id)
    }

    /// Expand a node
    pub fn expand(&mut self, node_id: &str) {
        self.expanded.insert(node_id.to_string());
    }

    /// Collapse a node
    pub fn collapse(&mut self, node_id: &str) {
        self.expanded.remove(node_id);
    }

    /// Expand all nodes
    pub fn expand_all<T>(&mut self, nodes: &[TreeNode<T>]) {
        fn expand_recursive<T>(state: &mut TreeState, node: &TreeNode<T>) {
            state.expand(&node.id);
            for child in &node.children {
                expand_recursive(state, child);
            }
        }

        for node in nodes {
            expand_recursive(self, node);
        }
    }

    /// Collapse all nodes
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Get the currently selected item index
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Select an item by index
    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.visible_items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select previous item
    pub fn select_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.visible_items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Toggle expansion of the currently selected node
    pub fn toggle_selected(&mut self) {
        if let Some(index) = self.selected() {
            if let Some(item) = self.visible_items.get(index) {
                if item.has_children {
                    let id = item.id.clone();
                    self.toggle_expansion(&id);
                }
            }
        }
    }

    /// Get the ID of the currently selected node
    pub fn selected_id(&self) -> Option<&str> {
        self.selected()
            .and_then(|i| self.visible_items.get(i))
            .map(|item| item.id.as_str())
    }
}

/// Tree widget for displaying hierarchical data
pub struct Tree<'a, T> {
    /// Root nodes
    nodes: &'a [TreeNode<T>],
    /// Block for the widget
    block: Option<Block<'a>>,
    /// Style for normal items
    style: Style,
    /// Style for selected items
    highlight_style: Style,
    /// Symbol for selected items
    highlight_symbol: &'a str,
    /// Style for expanded indicator
    expanded_style: Style,
    /// Style for collapsed indicator
    collapsed_style: Style,
}

impl<'a, T> Tree<'a, T> {
    /// Create a new tree widget
    pub fn new(nodes: &'a [TreeNode<T>]) -> Self {
        Self {
            nodes,
            block: None,
            style: Style::default(),
            highlight_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            highlight_symbol: ">> ",
            expanded_style: Style::default().fg(Color::Cyan),
            collapsed_style: Style::default().fg(Color::DarkGray),
        }
    }

    /// Set the block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the highlight style
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Set the highlight symbol
    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = symbol;
        self
    }

    /// Set the expanded indicator style
    pub fn expanded_style(mut self, style: Style) -> Self {
        self.expanded_style = style;
        self
    }

    /// Set the collapsed indicator style
    pub fn collapsed_style(mut self, style: Style) -> Self {
        self.collapsed_style = style;
        self
    }
}

impl<'a, T: Display> StatefulWidget for Tree<'a, T> {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Flatten the tree into a list of visible items
        state.visible_items.clear();
        flatten_tree(self.nodes, &state.expanded, 0, &mut state.visible_items);

        // Build list items with proper indentation and expand/collapse indicators
        let items: Vec<ListItem> = state
            .visible_items
            .iter()
            .map(|item| {
                let mut spans = Vec::new();

                // Add indentation
                if item.level > 0 {
                    let indent = "  ".repeat(item.level);
                    spans.push(Span::raw(indent));
                }

                // Add expand/collapse indicator
                if item.has_children {
                    let indicator = if item.is_expanded { "▼ " } else { "▶ " };
                    let indicator_style = if item.is_expanded {
                        self.expanded_style
                    } else {
                        self.collapsed_style
                    };
                    spans.push(Span::styled(indicator, indicator_style));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Add text
                spans.push(Span::raw(&item.text));

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Create and render the list
        let list = List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .highlight_symbol(self.highlight_symbol);

        let list = if let Some(block) = self.block {
            list.block(block)
        } else {
            list
        };

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}

/// Flatten the tree structure into a list of visible items
fn flatten_tree<T: Display>(
    nodes: &[TreeNode<T>],
    expanded: &std::collections::HashSet<String>,
    level: usize,
    output: &mut Vec<VisibleItem>,
) {
    for node in nodes {
        let is_expanded = expanded.contains(&node.id);
        let has_children = node.has_children();

        // Add this node
        output.push(VisibleItem {
            id: node.id.clone(),
            text: node.data.to_string(),
            level,
            has_children,
            is_expanded,
        });

        // Add children if expanded
        if is_expanded && has_children {
            flatten_tree(&node.children, expanded, level + 1, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("node1", "Test Node");
        assert_eq!(node.id, "node1");
        assert_eq!(node.data, "Test Node");
        assert!(!node.has_children());
    }

    #[test]
    fn test_tree_node_with_children() {
        let child1 = TreeNode::new("child1", "Child 1");
        let child2 = TreeNode::new("child2", "Child 2");
        let parent = TreeNode::with_children("parent", "Parent", vec![child1, child2]);

        assert_eq!(parent.id, "parent");
        assert!(parent.has_children());
        assert_eq!(parent.children.len(), 2);
    }

    #[test]
    fn test_tree_state_expansion() {
        let mut state = TreeState::new();

        assert!(!state.is_expanded("node1"));

        state.expand("node1");
        assert!(state.is_expanded("node1"));

        state.collapse("node1");
        assert!(!state.is_expanded("node1"));

        state.toggle_expansion("node1");
        assert!(state.is_expanded("node1"));

        state.toggle_expansion("node1");
        assert!(!state.is_expanded("node1"));
    }

    #[test]
    fn test_tree_state_selection() {
        let mut state = TreeState::new();

        // Initially selects index 0
        assert_eq!(state.selected(), Some(0));

        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_tree_state_navigation() {
        let mut state = TreeState::new();

        // Create some visible items for navigation
        state.visible_items = vec![
            VisibleItem {
                id: "1".to_string(),
                text: "Item 1".to_string(),
                level: 0,
                has_children: false,
                is_expanded: false,
            },
            VisibleItem {
                id: "2".to_string(),
                text: "Item 2".to_string(),
                level: 0,
                has_children: false,
                is_expanded: false,
            },
            VisibleItem {
                id: "3".to_string(),
                text: "Item 3".to_string(),
                level: 0,
                has_children: false,
                is_expanded: false,
            },
        ];

        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.selected(), Some(1));

        state.select_next();
        assert_eq!(state.selected(), Some(2));

        // Should wrap to 0
        state.select_next();
        assert_eq!(state.selected(), Some(0));

        state.select_previous();
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_tree_state_expand_all() {
        let child1 = TreeNode::new("child1", "Child 1");
        let child2 = TreeNode::new("child2", "Child 2");
        let grandchild = TreeNode::new("grandchild", "Grandchild");

        let mut child3 = TreeNode::new("child3", "Child 3");
        child3.add_child(grandchild);

        let parent = TreeNode::with_children("parent", "Parent", vec![child1, child2, child3]);

        let mut state = TreeState::new();
        state.expand_all(&[parent]);

        assert!(state.is_expanded("parent"));
        assert!(state.is_expanded("child3"));
        assert!(state.is_expanded("grandchild"));
    }

    #[test]
    fn test_tree_state_collapse_all() {
        let mut state = TreeState::new();
        state.expand("node1");
        state.expand("node2");
        state.expand("node3");

        assert!(state.is_expanded("node1"));
        assert!(state.is_expanded("node2"));
        assert!(state.is_expanded("node3"));

        state.collapse_all();

        assert!(!state.is_expanded("node1"));
        assert!(!state.is_expanded("node2"));
        assert!(!state.is_expanded("node3"));
    }

    #[test]
    fn test_flatten_tree() {
        let child1 = TreeNode::new("child1", "Child 1");
        let child2 = TreeNode::new("child2", "Child 2");
        let parent = TreeNode::with_children("parent", "Parent", vec![child1, child2]);

        let mut state = TreeState::new();
        let mut output = Vec::new();

        // Without expansion, should only show parent
        flatten_tree(std::slice::from_ref(&parent), &state.expanded, 0, &mut output);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].id, "parent");
        assert_eq!(output[0].level, 0);

        // With expansion, should show parent and children
        state.expand("parent");
        output.clear();
        flatten_tree(&[parent], &state.expanded, 0, &mut output);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0].id, "parent");
        assert_eq!(output[0].level, 0);
        assert_eq!(output[1].id, "child1");
        assert_eq!(output[1].level, 1);
        assert_eq!(output[2].id, "child2");
        assert_eq!(output[2].level, 1);
    }

    #[test]
    fn test_selected_id() {
        let mut state = TreeState::new();
        state.visible_items = vec![
            VisibleItem {
                id: "node1".to_string(),
                text: "Node 1".to_string(),
                level: 0,
                has_children: false,
                is_expanded: false,
            },
            VisibleItem {
                id: "node2".to_string(),
                text: "Node 2".to_string(),
                level: 0,
                has_children: false,
                is_expanded: false,
            },
        ];

        state.select(Some(0));
        assert_eq!(state.selected_id(), Some("node1"));

        state.select(Some(1));
        assert_eq!(state.selected_id(), Some("node2"));

        state.select(None);
        assert_eq!(state.selected_id(), None);
    }

    #[test]
    fn test_toggle_selected() {
        let mut state = TreeState::new();

        // Create a tree structure
        let child1 = TreeNode::new("child1", "Child 1");
        let child2 = TreeNode::new("child2", "Child 2");
        let parent = TreeNode::with_children("parent", "Parent", vec![child1, child2]);

        // Populate visible items
        let mut output = Vec::new();
        flatten_tree(&[parent], &state.expanded, 0, &mut output);
        state.visible_items = output;

        // Select the parent node (index 0)
        state.select(Some(0));
        assert_eq!(state.selected_id(), Some("parent"));
        assert!(!state.is_expanded("parent"));

        // Toggle expansion
        state.toggle_selected();
        assert!(state.is_expanded("parent"));

        // Toggle again to collapse
        state.toggle_selected();
        assert!(!state.is_expanded("parent"));
    }

    #[test]
    fn test_deep_hierarchy() {
        // Create a deep tree structure
        let leaf = TreeNode::new("leaf", "Leaf Node");
        let level2 = TreeNode::with_children("level2", "Level 2", vec![leaf]);
        let level1 = TreeNode::with_children("level1", "Level 1", vec![level2]);
        let root = TreeNode::with_children("root", "Root", vec![level1]);

        let mut state = TreeState::new();

        // Initially only root is visible
        let mut output = Vec::new();
        flatten_tree(std::slice::from_ref(&root), &state.expanded, 0, &mut output);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].id, "root");

        // Expand all
        state.expand_all(std::slice::from_ref(&root));

        // Now all nodes should be visible
        output.clear();
        flatten_tree(&[root], &state.expanded, 0, &mut output);
        assert_eq!(output.len(), 4);
        assert_eq!(output[0].id, "root");
        assert_eq!(output[0].level, 0);
        assert_eq!(output[1].id, "level1");
        assert_eq!(output[1].level, 1);
        assert_eq!(output[2].id, "level2");
        assert_eq!(output[2].level, 2);
        assert_eq!(output[3].id, "leaf");
        assert_eq!(output[3].level, 3);
    }

    #[test]
    fn test_multiple_siblings() {
        // Create a tree with multiple siblings at each level
        let child1_1 = TreeNode::new("child1_1", "Child 1.1");
        let child1_2 = TreeNode::new("child1_2", "Child 1.2");
        let parent1 = TreeNode::with_children("parent1", "Parent 1", vec![child1_1, child1_2]);

        let child2_1 = TreeNode::new("child2_1", "Child 2.1");
        let parent2 = TreeNode::with_children("parent2", "Parent 2", vec![child2_1]);

        let mut state = TreeState::new();

        // Expand only first parent
        state.expand("parent1");

        let mut output = Vec::new();
        flatten_tree(&[parent1.clone(), parent2.clone()], &state.expanded, 0, &mut output);

        // Should show: parent1, child1_1, child1_2, parent2
        assert_eq!(output.len(), 4);
        assert_eq!(output[0].id, "parent1");
        assert_eq!(output[1].id, "child1_1");
        assert_eq!(output[2].id, "child1_2");
        assert_eq!(output[3].id, "parent2");

        // Now expand second parent
        state.expand("parent2");
        output.clear();
        flatten_tree(&[parent1, parent2], &state.expanded, 0, &mut output);

        // Should show all nodes
        assert_eq!(output.len(), 5);
        assert_eq!(output[4].id, "child2_1");
    }
}
