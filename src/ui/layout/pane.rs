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
}

impl PaneManager {
    /// Create a new pane manager with a single pane
    pub fn new(area: Rect) -> Self {
        Self {
            root: Pane::new(0, area),
            next_id: 1,
            focused_id: 0,
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
}
