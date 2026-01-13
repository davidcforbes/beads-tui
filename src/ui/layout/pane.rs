//! Pane management for split views

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Pane split orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

/// A pane in the layout that can be split
#[derive(Debug, Clone)]
pub struct Pane {
    pub id: usize,
    pub area: Rect,
    pub is_focused: bool,
    pub split: Option<Box<PaneSplit>>,
}

/// Represents a split pane configuration
#[derive(Debug, Clone)]
pub struct PaneSplit {
    pub orientation: SplitOrientation,
    pub ratio: u16, // Percentage for first pane (0-100)
    pub first: Pane,
    pub second: Pane,
}

impl Pane {
    /// Create a new simple pane
    pub fn new(id: usize, area: Rect) -> Self {
        Self {
            id,
            area,
            is_focused: false,
            split: None,
        }
    }

    /// Split this pane horizontally
    pub fn split_horizontal(&mut self, ratio: u16, next_id: &mut usize) {
        let first_id = *next_id;
        *next_id += 1;
        let second_id = *next_id;
        *next_id += 1;

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(ratio),
                Constraint::Percentage(100 - ratio),
            ])
            .split(self.area);

        self.split = Some(Box::new(PaneSplit {
            orientation: SplitOrientation::Horizontal,
            ratio,
            first: Pane::new(first_id, areas[0]),
            second: Pane::new(second_id, areas[1]),
        }));
    }

    /// Split this pane vertically
    pub fn split_vertical(&mut self, ratio: u16, next_id: &mut usize) {
        let first_id = *next_id;
        *next_id += 1;
        let second_id = *next_id;
        *next_id += 1;

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(ratio),
                Constraint::Percentage(100 - ratio),
            ])
            .split(self.area);

        self.split = Some(Box::new(PaneSplit {
            orientation: SplitOrientation::Vertical,
            ratio,
            first: Pane::new(first_id, areas[0]),
            second: Pane::new(second_id, areas[1]),
        }));
    }

    /// Check if this pane is split
    pub fn is_split(&self) -> bool {
        self.split.is_some()
    }

    /// Get all leaf pane IDs (non-split panes)
    pub fn get_leaf_ids(&self) -> Vec<usize> {
        if let Some(split) = &self.split {
            let mut ids = split.first.get_leaf_ids();
            ids.extend(split.second.get_leaf_ids());
            ids
        } else {
            vec![self.id]
        }
    }
}

/// Pane manager for handling multiple panes
pub struct PaneManager {
    root: Pane,
    next_id: usize,
    focused_id: usize,
    fullscreen_pane_id: Option<usize>,
}

impl PaneManager {
    /// Create a new pane manager with a single pane
    pub fn new(area: Rect) -> Self {
        Self {
            root: Pane::new(0, area),
            next_id: 1,
            focused_id: 0,
            fullscreen_pane_id: None,
        }
    }

    /// Get the currently focused pane ID
    pub fn focused_id(&self) -> usize {
        self.focused_id
    }

    /// Set the focused pane
    pub fn set_focused(&mut self, id: usize) {
        self.focused_id = id;
    }

    /// Get all leaf pane IDs
    pub fn get_pane_ids(&self) -> Vec<usize> {
        self.root.get_leaf_ids()
    }

    /// Focus the next pane
    pub fn focus_next(&mut self) {
        let ids = self.get_pane_ids();
        if let Some(current_idx) = ids.iter().position(|&id| id == self.focused_id) {
            self.focused_id = ids[(current_idx + 1) % ids.len()];
        }
    }

    /// Focus the previous pane
    pub fn focus_previous(&mut self) {
        let ids = self.get_pane_ids();
        if let Some(current_idx) = ids.iter().position(|&id| id == self.focused_id) {
            let prev_idx = if current_idx == 0 {
                ids.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_id = ids[prev_idx];
        }
    }

    /// Split the focused pane horizontally
    pub fn split_focused_horizontal(&mut self, ratio: u16) -> bool {
        Self::split_pane_horizontal(&mut self.root, self.focused_id, ratio, &mut self.next_id)
    }

    /// Split the focused pane vertically
    pub fn split_focused_vertical(&mut self, ratio: u16) -> bool {
        Self::split_pane_vertical(&mut self.root, self.focused_id, ratio, &mut self.next_id)
    }

    /// Helper to find and split a specific pane horizontally
    fn split_pane_horizontal(
        pane: &mut Pane,
        target_id: usize,
        ratio: u16,
        next_id: &mut usize,
    ) -> bool {
        if pane.id == target_id && !pane.is_split() {
            pane.split_horizontal(ratio, next_id);
            true
        } else if let Some(split) = &mut pane.split {
            Self::split_pane_horizontal(&mut split.first, target_id, ratio, next_id)
                || Self::split_pane_horizontal(&mut split.second, target_id, ratio, next_id)
        } else {
            false
        }
    }

    /// Helper to find and split a specific pane vertically
    fn split_pane_vertical(
        pane: &mut Pane,
        target_id: usize,
        ratio: u16,
        next_id: &mut usize,
    ) -> bool {
        if pane.id == target_id && !pane.is_split() {
            pane.split_vertical(ratio, next_id);
            true
        } else if let Some(split) = &mut pane.split {
            Self::split_pane_vertical(&mut split.first, target_id, ratio, next_id)
                || Self::split_pane_vertical(&mut split.second, target_id, ratio, next_id)
        } else {
            false
        }
    }

    /// Toggle fullscreen for the focused pane
    pub fn toggle_fullscreen(&mut self) {
        if self.fullscreen_pane_id.is_some() {
            // Exit fullscreen
            self.fullscreen_pane_id = None;
        } else {
            // Enter fullscreen with focused pane
            self.fullscreen_pane_id = Some(self.focused_id);
        }
    }

    /// Check if a pane is in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen_pane_id.is_some()
    }

    /// Get the fullscreen pane ID if in fullscreen mode
    pub fn fullscreen_pane_id(&self) -> Option<usize> {
        self.fullscreen_pane_id
    }

    /// Get the area for rendering a specific pane
    pub fn get_pane_area(&self, pane_id: usize, full_area: Rect) -> Option<Rect> {
        // If in fullscreen mode and this is the fullscreen pane, return full area
        if let Some(fs_id) = self.fullscreen_pane_id {
            if fs_id == pane_id {
                return Some(full_area);
            } else {
                // Pane is hidden in fullscreen mode
                return None;
            }
        }

        // Not in fullscreen, find the pane's normal area
        Self::find_pane_area(&self.root, pane_id)
    }

    /// Helper to find a pane's area in the tree
    fn find_pane_area(pane: &Pane, target_id: usize) -> Option<Rect> {
        if pane.id == target_id {
            Some(pane.area)
        } else if let Some(split) = &pane.split {
            Self::find_pane_area(&split.first, target_id)
                .or_else(|| Self::find_pane_area(&split.second, target_id))
        } else {
            None
        }
    }

    /// Update layout when terminal is resized
    pub fn resize(&mut self, new_area: Rect) {
        self.root.area = new_area;
        Self::recalculate_layout(&mut self.root);
    }

    /// Recalculate layout for split panes
    fn recalculate_layout(pane: &mut Pane) {
        if let Some(split) = &mut pane.split {
            match split.orientation {
                SplitOrientation::Horizontal => {
                    let areas = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(split.ratio),
                            Constraint::Percentage(100 - split.ratio),
                        ])
                        .split(pane.area);

                    split.first.area = areas[0];
                    split.second.area = areas[1];
                }
                SplitOrientation::Vertical => {
                    let areas = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(split.ratio),
                            Constraint::Percentage(100 - split.ratio),
                        ])
                        .split(pane.area);

                    split.first.area = areas[0];
                    split.second.area = areas[1];
                }
            }

            Self::recalculate_layout(&mut split.first);
            Self::recalculate_layout(&mut split.second);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_split() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        assert!(!pane.is_split());

        pane.split_horizontal(50, &mut next_id);
        assert!(pane.is_split());

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 2);
    }

    #[test]
    fn test_pane_manager() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        assert_eq!(manager.focused_id(), 0);
        assert_eq!(manager.get_pane_ids().len(), 1);

        manager.focus_next();
        assert_eq!(manager.focused_id(), 0); // Only one pane, wraps around
    }

    #[test]
    fn test_fullscreen_toggle() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        assert!(!manager.is_fullscreen());
        assert_eq!(manager.fullscreen_pane_id(), None);

        manager.toggle_fullscreen();
        assert!(manager.is_fullscreen());
        assert_eq!(manager.fullscreen_pane_id(), Some(0));

        manager.toggle_fullscreen();
        assert!(!manager.is_fullscreen());
        assert_eq!(manager.fullscreen_pane_id(), None);
    }

    #[test]
    fn test_fullscreen_pane_area() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Normal mode - pane gets its area
        let pane_area = manager.get_pane_area(0, area);
        assert_eq!(pane_area, Some(area));

        // Enter fullscreen
        manager.toggle_fullscreen();
        let fs_area = manager.get_pane_area(0, area);
        assert_eq!(fs_area, Some(area));
    }

    #[test]
    fn test_pane_vertical_split() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        assert!(!pane.is_split());

        pane.split_vertical(60, &mut next_id);
        assert!(pane.is_split());

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 2);
        assert_eq!(leaf_ids[0], 1);
        assert_eq!(leaf_ids[1], 2);
        assert_eq!(next_id, 3);
    }

    #[test]
    fn test_pane_nested_splits() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        // First horizontal split
        pane.split_horizontal(50, &mut next_id);
        assert_eq!(next_id, 3);

        // Split the first child vertically
        if let Some(split) = &mut pane.split {
            split.first.split_vertical(40, &mut next_id);
            assert_eq!(next_id, 5);
        }

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 3); // Two from first split, one from second
        assert!(leaf_ids.contains(&2)); // Original second pane
        assert!(leaf_ids.contains(&3)); // First nested pane
        assert!(leaf_ids.contains(&4)); // Second nested pane
    }

    #[test]
    fn test_pane_get_leaf_ids_complex_tree() {
        let area = Rect::new(0, 0, 200, 80);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        // Create complex nested structure
        pane.split_horizontal(50, &mut next_id);
        if let Some(split) = &mut pane.split {
            split.first.split_vertical(50, &mut next_id);
            split.second.split_horizontal(50, &mut next_id);
        }

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 4);
        assert_eq!(next_id, 7);
    }

    #[test]
    fn test_pane_manager_focus_next_multiple_panes() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split to create multiple panes
        assert!(manager.split_focused_horizontal(50));

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 2);
        // After split, the original pane 0 is split and IDs 1 and 2 are created
        assert_eq!(ids[0], 1);
        assert_eq!(ids[1], 2);

        // Focus is still 0 (the split container), need to set focus to a leaf
        manager.set_focused(ids[0]);
        assert_eq!(manager.focused_id(), ids[0]);

        // Focus next should go to second pane
        manager.focus_next();
        assert_eq!(manager.focused_id(), ids[1]);

        // Focus next should wrap back to first
        manager.focus_next();
        assert_eq!(manager.focused_id(), ids[0]);
    }

    #[test]
    fn test_pane_manager_focus_previous() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // First split
        manager.split_focused_horizontal(50);

        // Focus on one of the new leaf panes before second split
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);

        // Second split
        manager.split_focused_vertical(50);

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 3);

        // Start at first pane
        let start_id = ids[0];
        manager.set_focused(start_id);

        // Focus previous should wrap to last pane
        manager.focus_previous();
        assert_eq!(manager.focused_id(), ids[2]);

        // Focus previous again
        manager.focus_previous();
        assert_eq!(manager.focused_id(), ids[1]);

        // One more to get back to start
        manager.focus_previous();
        assert_eq!(manager.focused_id(), start_id);
    }

    #[test]
    fn test_pane_manager_split_focused_vertical() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        assert!(manager.split_focused_vertical(70));
        assert_eq!(manager.get_pane_ids().len(), 2);

        // Focus on one of the leaf panes before splitting again
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);

        // Split one of the resulting panes
        assert!(manager.split_focused_vertical(30));
        assert_eq!(manager.get_pane_ids().len(), 3);
    }

    #[test]
    fn test_pane_manager_split_already_split_pane_fails() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // First split succeeds
        assert!(manager.split_focused_horizontal(50));

        // Trying to split the root again should fail (it's already split)
        let original_count = manager.get_pane_ids().len();
        manager.set_focused(0);
        assert!(!manager.split_focused_horizontal(50));
        assert_eq!(manager.get_pane_ids().len(), original_count);
    }

    #[test]
    fn test_pane_manager_set_focused() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();

        manager.set_focused(ids[0]);
        assert_eq!(manager.focused_id(), ids[0]);

        manager.set_focused(ids[1]);
        assert_eq!(manager.focused_id(), ids[1]);
    }

    #[test]
    fn test_pane_manager_get_pane_area_nested() {
        let area = Rect::new(0, 0, 200, 100);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        manager.focus_next();
        manager.split_focused_vertical(60);

        let ids = manager.get_pane_ids();
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, area);
            assert!(pane_area.is_some());
        }
    }

    #[test]
    fn test_pane_manager_get_pane_area_non_existent() {
        let area = Rect::new(0, 0, 100, 40);
        let manager = PaneManager::new(area);

        let non_existent_area = manager.get_pane_area(999, area);
        assert_eq!(non_existent_area, None);
    }

    #[test]
    fn test_pane_manager_fullscreen_hides_other_panes() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 2);

        // Enter fullscreen on first pane
        manager.set_focused(ids[0]);
        manager.toggle_fullscreen();

        // First pane should get full area
        assert_eq!(manager.get_pane_area(ids[0], area), Some(area));

        // Second pane should be hidden
        assert_eq!(manager.get_pane_area(ids[1], area), None);
    }

    #[test]
    fn test_pane_manager_resize_single_pane() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        let new_area = Rect::new(0, 0, 120, 50);
        manager.resize(new_area);

        let pane_area = manager.get_pane_area(0, new_area);
        assert_eq!(pane_area, Some(new_area));
    }

    #[test]
    fn test_pane_manager_resize_with_splits() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();

        let new_area = Rect::new(0, 0, 200, 80);
        manager.resize(new_area);

        // Check that all panes got updated areas
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, new_area);
            assert!(pane_area.is_some());
            let rect = pane_area.unwrap();
            assert!(rect.width > 0);
            assert!(rect.height > 0);
        }
    }

    #[test]
    fn test_pane_manager_resize_nested_splits() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);

        // Focus on a leaf pane before second split
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);

        manager.split_focused_vertical(60);

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 3);

        let new_area = Rect::new(0, 0, 150, 60);
        manager.resize(new_area);

        // All panes should have valid areas after resize
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, new_area);
            assert!(pane_area.is_some());
        }
    }

    #[test]
    fn test_split_orientation_equality() {
        assert_eq!(SplitOrientation::Horizontal, SplitOrientation::Horizontal);
        assert_eq!(SplitOrientation::Vertical, SplitOrientation::Vertical);
        assert_ne!(SplitOrientation::Horizontal, SplitOrientation::Vertical);
    }

    #[test]
    fn test_pane_split_ratio_edge_cases() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        // Test with 0% ratio (first pane gets nothing)
        pane.split_horizontal(0, &mut next_id);
        assert!(pane.is_split());

        let mut pane2 = Pane::new(5, area);
        let mut next_id2 = 6;

        // Test with 100% ratio (second pane gets nothing)
        pane2.split_vertical(100, &mut next_id2);
        assert!(pane2.is_split());
    }

    #[test]
    fn test_pane_new_initialization() {
        let area = Rect::new(10, 20, 100, 50);
        let pane = Pane::new(42, area);

        assert_eq!(pane.id, 42);
        assert_eq!(pane.area, area);
        assert!(!pane.is_focused);
        assert!(!pane.is_split());
    }

    #[test]
    fn test_pane_manager_new_initialization() {
        let area = Rect::new(0, 0, 100, 40);
        let manager = PaneManager::new(area);

        assert_eq!(manager.focused_id(), 0);
        assert_eq!(manager.get_pane_ids(), vec![0]);
        assert!(!manager.is_fullscreen());
        assert_eq!(manager.fullscreen_pane_id(), None);
    }

    #[test]
    fn test_split_orientation_clone() {
        let orientation = SplitOrientation::Horizontal;
        let cloned = orientation;
        assert_eq!(orientation, cloned);
    }

    #[test]
    fn test_split_orientation_copy() {
        let orientation = SplitOrientation::Vertical;
        let copied = orientation;
        assert_eq!(orientation, copied);
    }

    #[test]
    fn test_pane_clone() {
        let area = Rect::new(0, 0, 100, 40);
        let pane = Pane::new(5, area);
        let cloned = pane.clone();

        assert_eq!(pane.id, cloned.id);
        assert_eq!(pane.area, cloned.area);
        assert_eq!(pane.is_focused, cloned.is_focused);
        assert_eq!(pane.is_split(), cloned.is_split());
    }

    #[test]
    fn test_pane_split_clone() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        pane.split_horizontal(50, &mut next_id);
        let cloned = pane.clone();

        assert_eq!(pane.is_split(), cloned.is_split());
        assert_eq!(pane.get_leaf_ids(), cloned.get_leaf_ids());
    }

    #[test]
    fn test_pane_split_struct_clone() {
        let area = Rect::new(0, 0, 100, 40);
        let split = PaneSplit {
            orientation: SplitOrientation::Horizontal,
            ratio: 50,
            first: Pane::new(1, area),
            second: Pane::new(2, area),
        };

        let cloned = split.clone();
        assert_eq!(split.orientation, cloned.orientation);
        assert_eq!(split.ratio, cloned.ratio);
        assert_eq!(split.first.id, cloned.first.id);
        assert_eq!(split.second.id, cloned.second.id);
    }

    #[test]
    fn test_pane_manager_focus_next_single_pane() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        let initial_focus = manager.focused_id();
        manager.focus_next();
        assert_eq!(manager.focused_id(), initial_focus); // Wraps to same pane
    }

    #[test]
    fn test_pane_manager_focus_previous_single_pane() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        let initial_focus = manager.focused_id();
        manager.focus_previous();
        assert_eq!(manager.focused_id(), initial_focus); // Wraps to same pane
    }

    #[test]
    fn test_pane_manager_split_with_various_ratios() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split with 25%
        assert!(manager.split_focused_horizontal(25));

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 2);

        // Focus and split second pane with 75%
        manager.set_focused(ids[1]);
        assert!(manager.split_focused_vertical(75));

        assert_eq!(manager.get_pane_ids().len(), 3);
    }

    #[test]
    fn test_pane_manager_get_pane_area_after_fullscreen_exit() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();

        // Enter and exit fullscreen
        manager.set_focused(ids[0]);
        manager.toggle_fullscreen();
        manager.toggle_fullscreen();

        // Both panes should be visible again
        assert!(manager.get_pane_area(ids[0], area).is_some());
        assert!(manager.get_pane_area(ids[1], area).is_some());
    }

    #[test]
    fn test_pane_manager_fullscreen_with_different_panes() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();

        // Fullscreen first pane
        manager.set_focused(ids[0]);
        manager.toggle_fullscreen();
        assert_eq!(manager.fullscreen_pane_id(), Some(ids[0]));

        // Exit and fullscreen second pane
        manager.toggle_fullscreen();
        manager.set_focused(ids[1]);
        manager.toggle_fullscreen();
        assert_eq!(manager.fullscreen_pane_id(), Some(ids[1]));
    }

    #[test]
    fn test_pane_get_leaf_ids_single_pane() {
        let area = Rect::new(0, 0, 100, 40);
        let pane = Pane::new(42, area);

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids, vec![42]);
    }

    #[test]
    fn test_pane_get_leaf_ids_after_horizontal_split() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        pane.split_horizontal(50, &mut next_id);

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 2);
        assert_eq!(leaf_ids[0], 1);
        assert_eq!(leaf_ids[1], 2);
    }

    #[test]
    fn test_pane_get_leaf_ids_after_vertical_split() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        pane.split_vertical(60, &mut next_id);

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 2);
        assert_eq!(leaf_ids[0], 1);
        assert_eq!(leaf_ids[1], 2);
    }

    #[test]
    fn test_pane_manager_multiple_focus_next_cycles() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);

        // Cycle through multiple times
        for _ in 0..5 {
            manager.focus_next();
            manager.focus_next();
        }

        // Should be back at first pane
        assert_eq!(manager.focused_id(), ids[0]);
    }

    #[test]
    fn test_pane_manager_multiple_focus_previous_cycles() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);

        // Cycle through multiple times
        for _ in 0..5 {
            manager.focus_previous();
            manager.focus_previous();
        }

        // Should be back at first pane
        assert_eq!(manager.focused_id(), ids[0]);
    }

    #[test]
    fn test_pane_manager_resize_maintains_split_orientation() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();

        let new_area = Rect::new(0, 0, 200, 80);
        manager.resize(new_area);

        // Verify both panes still exist and have valid areas
        assert_eq!(manager.get_pane_ids().len(), 2);
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, new_area);
            assert!(pane_area.is_some());
        }
    }

    #[test]
    fn test_pane_manager_split_non_leaf_fails() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split the root pane
        assert!(manager.split_focused_horizontal(50));

        // Trying to split pane 0 (now a split container) should fail
        manager.set_focused(0);
        assert!(!manager.split_focused_horizontal(50));
    }

    #[test]
    fn test_pane_manager_get_pane_area_in_fullscreen_returns_full_area() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(30);
        let ids = manager.get_pane_ids();

        manager.set_focused(ids[1]);
        manager.toggle_fullscreen();

        let fs_area = manager.get_pane_area(ids[1], area);
        assert_eq!(fs_area, Some(area));
    }

    #[test]
    fn test_pane_split_horizontal_updates_next_id() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 10;

        pane.split_horizontal(50, &mut next_id);
        assert_eq!(next_id, 12); // Should increment by 2
    }

    #[test]
    fn test_pane_split_vertical_updates_next_id() {
        let area = Rect::new(0, 0, 100, 40);
        let mut pane = Pane::new(0, area);
        let mut next_id = 20;

        pane.split_vertical(50, &mut next_id);
        assert_eq!(next_id, 22); // Should increment by 2
    }

    #[test]
    fn test_pane_manager_complex_focus_navigation() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Create 3 panes
        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);
        manager.split_focused_vertical(50);

        let all_ids = manager.get_pane_ids();
        assert_eq!(all_ids.len(), 3);

        // Navigate through all panes
        manager.set_focused(all_ids[0]);
        manager.focus_next();
        assert_eq!(manager.focused_id(), all_ids[1]);
        manager.focus_next();
        assert_eq!(manager.focused_id(), all_ids[2]);
        manager.focus_next();
        assert_eq!(manager.focused_id(), all_ids[0]); // Wrapped
    }

    #[test]
    fn test_pane_is_focused_field_default() {
        let area = Rect::new(0, 0, 100, 40);
        let pane = Pane::new(5, area);
        assert!(!pane.is_focused);
    }

    // ========== Additional comprehensive tests ==========

    #[test]
    fn test_split_orientation_debug() {
        let horizontal = SplitOrientation::Horizontal;
        let vertical = SplitOrientation::Vertical;

        let h_debug = format!("{:?}", horizontal);
        let v_debug = format!("{:?}", vertical);

        assert!(h_debug.contains("Horizontal"));
        assert!(v_debug.contains("Vertical"));
    }

    #[test]
    fn test_pane_debug() {
        let area = Rect::new(0, 0, 100, 40);
        let pane = Pane::new(5, area);
        let debug_str = format!("{:?}", pane);

        assert!(debug_str.contains("Pane"));
        assert!(debug_str.contains("id"));
    }

    #[test]
    fn test_pane_split_debug() {
        let area = Rect::new(0, 0, 100, 40);
        let split = PaneSplit {
            orientation: SplitOrientation::Horizontal,
            ratio: 50,
            first: Pane::new(1, area),
            second: Pane::new(2, area),
        };

        let debug_str = format!("{:?}", split);
        assert!(debug_str.contains("PaneSplit"));
        assert!(debug_str.contains("orientation"));
        assert!(debug_str.contains("ratio"));
    }

    #[test]
    fn test_pane_split_orientation_field() {
        let area = Rect::new(0, 0, 100, 40);
        let split = PaneSplit {
            orientation: SplitOrientation::Vertical,
            ratio: 60,
            first: Pane::new(1, area),
            second: Pane::new(2, area),
        };

        assert_eq!(split.orientation, SplitOrientation::Vertical);
    }

    #[test]
    fn test_pane_split_ratio_field() {
        let area = Rect::new(0, 0, 100, 40);
        let split = PaneSplit {
            orientation: SplitOrientation::Horizontal,
            ratio: 75,
            first: Pane::new(1, area),
            second: Pane::new(2, area),
        };

        assert_eq!(split.ratio, 75);
    }

    #[test]
    fn test_pane_manager_fullscreen_with_three_panes() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Create 3 panes
        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);
        manager.split_focused_vertical(50);

        let all_ids = manager.get_pane_ids();
        assert_eq!(all_ids.len(), 3);

        // Fullscreen the middle pane
        manager.set_focused(all_ids[1]);
        manager.toggle_fullscreen();

        // Only middle pane should be visible
        assert_eq!(manager.get_pane_area(all_ids[1], area), Some(area));
        assert_eq!(manager.get_pane_area(all_ids[0], area), None);
        assert_eq!(manager.get_pane_area(all_ids[2], area), None);
    }

    #[test]
    fn test_pane_manager_resize_preserves_split_ratios() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split with specific ratio
        manager.split_focused_horizontal(30);

        let new_area = Rect::new(0, 0, 200, 80);
        manager.resize(new_area);

        // Get areas and verify approximate ratio (30/70)
        let ids = manager.get_pane_ids();
        let area1 = manager.get_pane_area(ids[0], new_area).unwrap();
        let area2 = manager.get_pane_area(ids[1], new_area).unwrap();

        // First pane should be approximately 30% of width (60 out of 200)
        assert!(area1.width > 0 && area1.width < area2.width);
    }

    #[test]
    fn test_pane_deeply_nested_splits() {
        let area = Rect::new(0, 0, 200, 100);
        let mut pane = Pane::new(0, area);
        let mut next_id = 1;

        // Create deep nesting: split 4 levels deep
        pane.split_horizontal(50, &mut next_id);

        if let Some(ref mut split) = pane.split {
            split.first.split_vertical(50, &mut next_id);

            if let Some(ref mut nested_split) = split.first.split {
                nested_split.first.split_horizontal(50, &mut next_id);

                if let Some(ref mut deep_split) = nested_split.first.split {
                    deep_split.first.split_vertical(50, &mut next_id);
                }
            }
        }

        let leaf_ids = pane.get_leaf_ids();
        assert_eq!(leaf_ids.len(), 5); // 1 + 2 + 2 from successive splits
    }

    #[test]
    fn test_pane_manager_focus_navigation_with_many_panes() {
        let area = Rect::new(0, 0, 200, 100);
        let mut manager = PaneManager::new(area);

        // Create 4 panes
        manager.split_focused_horizontal(50);
        let ids = manager.get_pane_ids();
        manager.set_focused(ids[0]);
        manager.split_focused_vertical(50);

        let ids = manager.get_pane_ids();
        manager.set_focused(ids[2]);
        manager.split_focused_horizontal(50);

        let all_ids = manager.get_pane_ids();
        assert_eq!(all_ids.len(), 4);

        // Navigate forward through all
        manager.set_focused(all_ids[0]);
        for id in all_ids.iter().skip(1) {
            manager.focus_next();
            assert_eq!(manager.focused_id(), *id);
        }

        // Wrap back to first
        manager.focus_next();
        assert_eq!(manager.focused_id(), all_ids[0]);
    }

    #[test]
    fn test_pane_manager_get_pane_ids_consistency() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        manager.split_focused_horizontal(50);
        let ids1 = manager.get_pane_ids();

        // Calling multiple times should return same IDs
        let ids2 = manager.get_pane_ids();
        let ids3 = manager.get_pane_ids();

        assert_eq!(ids1, ids2);
        assert_eq!(ids2, ids3);
    }

    #[test]
    fn test_pane_manager_split_ratio_1_percent() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split with very small ratio
        assert!(manager.split_focused_horizontal(1));

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 2);

        // Both panes should have valid (though small for first) areas
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, area);
            assert!(pane_area.is_some());
        }
    }

    #[test]
    fn test_pane_manager_split_ratio_99_percent() {
        let area = Rect::new(0, 0, 100, 40);
        let mut manager = PaneManager::new(area);

        // Split with very large ratio
        assert!(manager.split_focused_vertical(99));

        let ids = manager.get_pane_ids();
        assert_eq!(ids.len(), 2);

        // Both panes should have valid (though small for second) areas
        for &id in &ids {
            let pane_area = manager.get_pane_area(id, area);
            assert!(pane_area.is_some());
        }
    }
}
