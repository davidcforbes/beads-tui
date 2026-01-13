//! Kanban board configuration with column management and persistence

use serde::{Deserialize, Serialize};

/// Grouping mode for Kanban columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum GroupingMode {
    /// Group by issue status (default)
    #[default]
    Status,
    /// Group by assignee
    Assignee,
    /// Group by label
    Label,
    /// Group by priority
    Priority,
}


impl GroupingMode {
    /// Returns the display name for this grouping mode
    pub fn display_name(&self) -> &'static str {
        match self {
            GroupingMode::Status => "Status",
            GroupingMode::Assignee => "Assignee",
            GroupingMode::Label => "Label",
            GroupingMode::Priority => "Priority",
        }
    }
}

/// Column identifier for the Kanban board
/// The column type depends on the active grouping mode
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnId {
    /// Status-based columns
    StatusOpen,
    StatusInProgress,
    StatusBlocked,
    StatusClosed,

    /// Assignee-based columns (dynamic, identified by assignee name)
    Assignee(String),

    /// Label-based columns (dynamic, identified by label name)
    Label(String),

    /// Priority-based columns
    PriorityP0,
    PriorityP1,
    PriorityP2,
    PriorityP3,
    PriorityP4,

    /// Unassigned/unlabeled items
    Unassigned,
}

impl ColumnId {
    /// Returns the default label for this column
    pub fn default_label(&self) -> String {
        match self {
            ColumnId::StatusOpen => "Open".to_string(),
            ColumnId::StatusInProgress => "In Progress".to_string(),
            ColumnId::StatusBlocked => "Blocked".to_string(),
            ColumnId::StatusClosed => "Closed".to_string(),
            ColumnId::Assignee(name) => name.clone(),
            ColumnId::Label(label) => label.clone(),
            ColumnId::PriorityP0 => "P0 - Critical".to_string(),
            ColumnId::PriorityP1 => "P1 - High".to_string(),
            ColumnId::PriorityP2 => "P2 - Medium".to_string(),
            ColumnId::PriorityP3 => "P3 - Low".to_string(),
            ColumnId::PriorityP4 => "P4 - Backlog".to_string(),
            ColumnId::Unassigned => "Unassigned".to_string(),
        }
    }

    /// Returns true if this column is mandatory for the current grouping mode
    pub fn is_mandatory(&self, mode: GroupingMode) -> bool {
        match mode {
            GroupingMode::Status => matches!(
                self,
                ColumnId::StatusOpen
                    | ColumnId::StatusInProgress
                    | ColumnId::StatusBlocked
                    | ColumnId::StatusClosed
            ),
            GroupingMode::Priority => matches!(
                self,
                ColumnId::PriorityP0
                    | ColumnId::PriorityP1
                    | ColumnId::PriorityP2
                    | ColumnId::PriorityP3
                    | ColumnId::PriorityP4
            ),
            // Assignee and Label columns are dynamic and not mandatory
            GroupingMode::Assignee | GroupingMode::Label => false,
        }
    }
}

/// Width constraints for a column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidthConstraints {
    /// Minimum width in characters
    pub min: u16,
    /// Maximum width in characters (None = unlimited)
    pub max: Option<u16>,
    /// Preferred width in characters
    pub preferred: u16,
}

impl WidthConstraints {
    pub fn new(min: u16, max: Option<u16>, preferred: u16) -> Self {
        Self {
            min,
            max,
            preferred: preferred.max(min),
        }
    }

    /// Clamp a width value to the constraints
    pub fn clamp(&self, width: u16) -> u16 {
        let clamped = width.max(self.min);
        if let Some(max) = self.max {
            clamped.min(max)
        } else {
            clamped
        }
    }
}

impl Default for WidthConstraints {
    fn default() -> Self {
        Self::new(15, Some(80), 30)
    }
}

/// Sort order for cards within a column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CardSort {
    /// Sort by priority (P0 first)
    #[default]
    Priority,
    /// Sort by title alphabetically
    Title,
    /// Sort by creation date (newest first)
    Created,
    /// Sort by update date (most recent first)
    Updated,
}


/// Column definition for the Kanban board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column identifier
    pub id: ColumnId,
    /// Display label
    pub label: String,
    /// Width constraints
    pub width_constraints: WidthConstraints,
    /// Current width (within constraints)
    pub width: u16,
    /// Whether the column is visible
    pub visible: bool,
    /// Sort order for cards in this column
    #[serde(default)]
    pub card_sort: CardSort,
    /// Work-in-progress limit (None = no limit)
    #[serde(default)]
    pub wip_limit: Option<usize>,
}

impl ColumnDefinition {
    /// Create a new column definition with defaults
    pub fn new(id: ColumnId) -> Self {
        let width_constraints = WidthConstraints::default();

        Self {
            label: id.default_label(),
            id,
            width_constraints,
            width: width_constraints.preferred,
            visible: true,
            card_sort: CardSort::default(),
            wip_limit: None,
        }
    }

    /// Set the column width (clamped to constraints)
    pub fn set_width(&mut self, width: u16) {
        self.width = self.width_constraints.clamp(width);
    }

    /// Set visibility (mandatory columns cannot be hidden for their grouping mode)
    pub fn set_visible(&mut self, visible: bool, mode: GroupingMode) {
        if !self.id.is_mandatory(mode) {
            self.visible = visible;
        }
    }
}

/// Filter configuration for the board
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoardFilters {
    /// Filter by issue type
    pub issue_types: Vec<String>,
    /// Filter by priority
    pub priorities: Vec<String>,
    /// Filter by labels
    pub labels: Vec<String>,
    /// Filter by assignees
    pub assignees: Vec<String>,
    /// Filter by status
    pub statuses: Vec<String>,
    /// Search query for title/description
    pub search_query: Option<String>,
}

impl BoardFilters {
    /// Check if any filters are active
    pub fn is_active(&self) -> bool {
        !self.issue_types.is_empty()
            || !self.priorities.is_empty()
            || !self.labels.is_empty()
            || !self.assignees.is_empty()
            || !self.statuses.is_empty()
            || self.search_query.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.issue_types.clear();
        self.priorities.clear();
        self.labels.clear();
        self.assignees.clear();
        self.statuses.clear();
        self.search_query = None;
    }
}

/// Complete Kanban board configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanConfig {
    /// Active grouping mode
    #[serde(default)]
    pub grouping_mode: GroupingMode,
    /// Column definitions in display order
    pub columns: Vec<ColumnDefinition>,
    /// Card height in lines
    #[serde(default = "default_card_height")]
    pub card_height: u16,
    /// Filter configuration
    #[serde(default)]
    pub filters: BoardFilters,
    /// Config version for migration
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_card_height() -> u16 {
    3
}

fn default_version() -> u32 {
    1
}

impl Default for KanbanConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl KanbanConfig {
    /// Create a new Kanban configuration with default columns
    pub fn new() -> Self {
        Self {
            grouping_mode: GroupingMode::default(),
            columns: Self::default_columns(GroupingMode::default()),
            card_height: default_card_height(),
            filters: BoardFilters::default(),
            version: 1,
        }
    }

    /// Get the default columns for a given grouping mode
    fn default_columns(mode: GroupingMode) -> Vec<ColumnDefinition> {
        match mode {
            GroupingMode::Status => vec![
                ColumnDefinition::new(ColumnId::StatusOpen),
                ColumnDefinition::new(ColumnId::StatusInProgress),
                ColumnDefinition::new(ColumnId::StatusBlocked),
                ColumnDefinition::new(ColumnId::StatusClosed),
            ],
            GroupingMode::Priority => vec![
                ColumnDefinition::new(ColumnId::PriorityP0),
                ColumnDefinition::new(ColumnId::PriorityP1),
                ColumnDefinition::new(ColumnId::PriorityP2),
                ColumnDefinition::new(ColumnId::PriorityP3),
                ColumnDefinition::new(ColumnId::PriorityP4),
            ],
            // Assignee and Label columns are dynamically created based on actual data
            GroupingMode::Assignee | GroupingMode::Label => vec![
                ColumnDefinition::new(ColumnId::Unassigned),
            ],
        }
    }

    /// Validate and migrate configuration
    pub fn validate_and_migrate(mut self) -> Self {
        // Ensure card height is at least 1
        self.card_height = self.card_height.max(1);

        // Validate widths are within constraints
        for col in &mut self.columns {
            col.width = col.width_constraints.clamp(col.width);
        }

        // Ensure mandatory columns for the current grouping mode are present and visible
        let mandatory_columns = Self::default_columns(self.grouping_mode);
        for mandatory in &mandatory_columns {
            let exists = self.columns.iter().any(|c| c.id == mandatory.id);
            if !exists {
                self.columns.push(mandatory.clone());
            }
        }

        // Ensure mandatory columns are visible
        for col in &mut self.columns {
            if col.id.is_mandatory(self.grouping_mode) {
                col.visible = true;
            }
        }

        // Remove columns that don't match the current grouping mode
        let mode = self.grouping_mode;
        self.columns.retain(|col| {
            match mode {
                GroupingMode::Status => matches!(
                    col.id,
                    ColumnId::StatusOpen
                        | ColumnId::StatusInProgress
                        | ColumnId::StatusBlocked
                        | ColumnId::StatusClosed
                ),
                GroupingMode::Priority => matches!(
                    col.id,
                    ColumnId::PriorityP0
                        | ColumnId::PriorityP1
                        | ColumnId::PriorityP2
                        | ColumnId::PriorityP3
                        | ColumnId::PriorityP4
                ),
                GroupingMode::Assignee => {
                    matches!(col.id, ColumnId::Assignee(_) | ColumnId::Unassigned)
                }
                GroupingMode::Label => matches!(col.id, ColumnId::Label(_) | ColumnId::Unassigned),
            }
        });

        // If no columns remain after filtering, restore defaults
        if self.columns.is_empty() {
            self.columns = Self::default_columns(self.grouping_mode);
        }

        self
    }

    /// Check if a column ID is valid for the current grouping mode
    #[cfg_attr(not(test), allow(dead_code))]
    fn is_valid_for_mode(&self, id: &ColumnId) -> bool {
        match self.grouping_mode {
            GroupingMode::Status => matches!(
                id,
                ColumnId::StatusOpen
                    | ColumnId::StatusInProgress
                    | ColumnId::StatusBlocked
                    | ColumnId::StatusClosed
            ),
            GroupingMode::Priority => matches!(
                id,
                ColumnId::PriorityP0
                    | ColumnId::PriorityP1
                    | ColumnId::PriorityP2
                    | ColumnId::PriorityP3
                    | ColumnId::PriorityP4
            ),
            GroupingMode::Assignee => {
                matches!(id, ColumnId::Assignee(_) | ColumnId::Unassigned)
            }
            GroupingMode::Label => matches!(id, ColumnId::Label(_) | ColumnId::Unassigned),
        }
    }

    /// Get visible columns in display order
    pub fn visible_columns(&self) -> Vec<&ColumnDefinition> {
        self.columns.iter().filter(|c| c.visible).collect()
    }

    /// Get a column by ID
    pub fn get_column(&self, id: &ColumnId) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|c| &c.id == id)
    }

    /// Get a mutable column by ID
    pub fn get_column_mut(&mut self, id: &ColumnId) -> Option<&mut ColumnDefinition> {
        let id = id.clone();
        self.columns.iter_mut().find(|c| c.id == id)
    }

    /// Reorder columns
    pub fn reorder_column(&mut self, from_index: usize, to_index: usize) {
        if from_index < self.columns.len() && to_index < self.columns.len() {
            let col = self.columns.remove(from_index);
            self.columns.insert(to_index, col);
        }
    }

    /// Set column width
    pub fn set_column_width(&mut self, id: &ColumnId, width: u16) {
        if let Some(col) = self.get_column_mut(id) {
            col.set_width(width);
        }
    }

    /// Toggle column visibility
    pub fn toggle_column_visibility(&mut self, id: &ColumnId) {
        let mode = self.grouping_mode;
        if let Some(col) = self.get_column_mut(id) {
            let new_visibility = !col.visible;
            col.set_visible(new_visibility, mode);
        }
    }

    /// Add a dynamic column (for Assignee or Label modes)
    pub fn add_dynamic_column(&mut self, id: ColumnId) {
        // Only add if it doesn't already exist
        if !self.columns.iter().any(|c| c.id == id) {
            self.columns.push(ColumnDefinition::new(id));
        }
    }

    /// Switch grouping mode (rebuilds columns)
    pub fn set_grouping_mode(&mut self, mode: GroupingMode) {
        if self.grouping_mode != mode {
            self.grouping_mode = mode;
            self.columns = Self::default_columns(mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grouping_mode_default() {
        assert_eq!(GroupingMode::default(), GroupingMode::Status);
    }

    #[test]
    fn test_column_id_mandatory() {
        assert!(ColumnId::StatusOpen.is_mandatory(GroupingMode::Status));
        assert!(ColumnId::StatusInProgress.is_mandatory(GroupingMode::Status));
        assert!(!ColumnId::StatusOpen.is_mandatory(GroupingMode::Priority));
        assert!(ColumnId::PriorityP0.is_mandatory(GroupingMode::Priority));
        assert!(!ColumnId::Assignee("user".to_string()).is_mandatory(GroupingMode::Assignee));
    }

    #[test]
    fn test_width_constraints_clamp() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        assert_eq!(constraints.clamp(5), 10); // Below min
        assert_eq!(constraints.clamp(30), 30); // Within range
        assert_eq!(constraints.clamp(60), 50); // Above max
    }

    #[test]
    fn test_column_definition_set_width() {
        let mut col = ColumnDefinition::new(ColumnId::StatusOpen);
        col.set_width(5); // Below min
        assert_eq!(col.width, col.width_constraints.min);

        col.set_width(100); // Above max
        if let Some(max) = col.width_constraints.max {
            assert_eq!(col.width, max);
        }
    }

    #[test]
    fn test_column_definition_set_visible() {
        // Mandatory column cannot be hidden
        let mut col = ColumnDefinition::new(ColumnId::StatusOpen);
        col.set_visible(false, GroupingMode::Status);
        assert!(col.visible);

        // Non-mandatory column can be hidden
        let mut col = ColumnDefinition::new(ColumnId::Assignee("user".to_string()));
        col.set_visible(false, GroupingMode::Assignee);
        assert!(!col.visible);
    }

    #[test]
    fn test_kanban_config_default() {
        let config = KanbanConfig::default();
        assert_eq!(config.grouping_mode, GroupingMode::Status);
        assert_eq!(config.columns.len(), 4); // Status columns
        assert_eq!(config.card_height, 3);
    }

    #[test]
    fn test_kanban_config_validate_card_height() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: KanbanConfig::default_columns(GroupingMode::Status),
            card_height: 0,
            filters: BoardFilters::default(),
            version: 1,
        };

        config = config.validate_and_migrate();
        assert_eq!(config.card_height, 1);
    }

    #[test]
    fn test_kanban_config_validate_mandatory_columns() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Should have added all mandatory status columns
        assert!(config.columns.iter().any(|c| c.id == ColumnId::StatusOpen));
        assert!(config
            .columns
            .iter()
            .any(|c| c.id == ColumnId::StatusInProgress));
        assert!(config.columns.iter().any(|c| c.id == ColumnId::StatusBlocked));
        assert!(config.columns.iter().any(|c| c.id == ColumnId::StatusClosed));
    }

    #[test]
    fn test_kanban_config_visible_columns() {
        let mut config = KanbanConfig::default();
        config.columns[1].visible = false; // Hide In Progress

        let visible = config.visible_columns();
        assert_eq!(visible.len(), 3);
        assert!(!visible.iter().any(|c| c.id == ColumnId::StatusInProgress));
    }

    #[test]
    fn test_kanban_config_reorder_column() {
        let mut config = KanbanConfig::default();
        let first_id = config.columns[0].id.clone();
        let second_id = config.columns[1].id.clone();

        config.reorder_column(1, 0);

        assert_eq!(config.columns[0].id, second_id);
        assert_eq!(config.columns[1].id, first_id);
    }

    #[test]
    fn test_kanban_config_set_column_width() {
        let mut config = KanbanConfig::default();
        config.set_column_width(&ColumnId::StatusOpen, 50);

        let col = config.get_column(&ColumnId::StatusOpen).unwrap();
        assert_eq!(col.width, 50);
    }

    #[test]
    fn test_kanban_config_toggle_visibility() {
        let mut config = KanbanConfig::default();

        // Toggle mandatory column (should remain visible)
        config.toggle_column_visibility(&ColumnId::StatusOpen);
        assert!(config.get_column(&ColumnId::StatusOpen).unwrap().visible);
    }

    #[test]
    fn test_kanban_config_set_grouping_mode() {
        let mut config = KanbanConfig::default();
        assert_eq!(config.columns.len(), 4); // Status columns

        config.set_grouping_mode(GroupingMode::Priority);
        assert_eq!(config.grouping_mode, GroupingMode::Priority);
        assert_eq!(config.columns.len(), 5); // Priority columns
        assert!(config.columns.iter().any(|c| c.id == ColumnId::PriorityP0));
    }

    #[test]
    fn test_kanban_config_add_dynamic_column() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Assignee,
            columns: vec![ColumnDefinition::new(ColumnId::Unassigned)],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        config.add_dynamic_column(ColumnId::Assignee("alice".to_string()));
        assert_eq!(config.columns.len(), 2);
        assert!(config
            .columns
            .iter()
            .any(|c| c.id == ColumnId::Assignee("alice".to_string())));

        // Adding the same column again should not duplicate
        config.add_dynamic_column(ColumnId::Assignee("alice".to_string()));
        assert_eq!(config.columns.len(), 2);
    }

    #[test]
    fn test_serialization() {
        let config = KanbanConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: KanbanConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.grouping_mode, deserialized.grouping_mode);
        assert_eq!(config.columns.len(), deserialized.columns.len());
        assert_eq!(config.card_height, deserialized.card_height);
    }

    #[test]
    fn test_is_valid_for_mode() {
        let config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        assert!(config.is_valid_for_mode(&ColumnId::StatusOpen));
        assert!(!config.is_valid_for_mode(&ColumnId::PriorityP0));
    }

    #[test]
    fn test_validate_removes_invalid_columns() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![
                ColumnDefinition::new(ColumnId::StatusOpen),
                ColumnDefinition::new(ColumnId::PriorityP0), // Invalid for Status mode
            ],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Priority column should be removed
        assert!(!config.columns.iter().any(|c| c.id == ColumnId::PriorityP0));
        // Status columns should remain
        assert!(config.columns.iter().any(|c| c.id == ColumnId::StatusOpen));
    }

    #[test]
    fn test_grouping_mode_clone() {
        let mode = GroupingMode::Status;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_grouping_mode_equality() {
        assert_eq!(GroupingMode::Status, GroupingMode::Status);
        assert_eq!(GroupingMode::Assignee, GroupingMode::Assignee);
        assert_ne!(GroupingMode::Status, GroupingMode::Priority);
    }

    #[test]
    fn test_grouping_mode_display_names() {
        assert_eq!(GroupingMode::Status.display_name(), "Status");
        assert_eq!(GroupingMode::Assignee.display_name(), "Assignee");
        assert_eq!(GroupingMode::Label.display_name(), "Label");
        assert_eq!(GroupingMode::Priority.display_name(), "Priority");
    }

    #[test]
    fn test_column_id_clone() {
        let id = ColumnId::StatusOpen;
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_column_id_equality() {
        assert_eq!(ColumnId::StatusOpen, ColumnId::StatusOpen);
        assert_ne!(ColumnId::StatusOpen, ColumnId::StatusClosed);
        assert_eq!(
            ColumnId::Assignee("alice".to_string()),
            ColumnId::Assignee("alice".to_string())
        );
    }

    #[test]
    fn test_column_id_default_labels() {
        assert_eq!(ColumnId::StatusOpen.default_label(), "Open");
        assert_eq!(ColumnId::StatusInProgress.default_label(), "In Progress");
        assert_eq!(ColumnId::StatusBlocked.default_label(), "Blocked");
        assert_eq!(ColumnId::StatusClosed.default_label(), "Closed");
        assert_eq!(
            ColumnId::Assignee("bob".to_string()).default_label(),
            "bob"
        );
        assert_eq!(ColumnId::Label("bug".to_string()).default_label(), "bug");
        assert_eq!(ColumnId::PriorityP0.default_label(), "P0 - Critical");
        assert_eq!(ColumnId::PriorityP1.default_label(), "P1 - High");
        assert_eq!(ColumnId::PriorityP2.default_label(), "P2 - Medium");
        assert_eq!(ColumnId::PriorityP3.default_label(), "P3 - Low");
        assert_eq!(ColumnId::PriorityP4.default_label(), "P4 - Backlog");
        assert_eq!(ColumnId::Unassigned.default_label(), "Unassigned");
    }

    #[test]
    fn test_width_constraints_clone() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        let cloned = constraints;
        assert_eq!(constraints.min, cloned.min);
        assert_eq!(constraints.max, cloned.max);
        assert_eq!(constraints.preferred, cloned.preferred);
    }

    #[test]
    fn test_width_constraints_new_preferred_less_than_min() {
        let constraints = WidthConstraints::new(20, Some(50), 10);
        // Preferred should be clamped to min
        assert_eq!(constraints.preferred, 20);
    }

    #[test]
    fn test_width_constraints_clamp_no_max() {
        let constraints = WidthConstraints::new(10, None, 30);
        assert_eq!(constraints.clamp(5), 10); // Below min
        assert_eq!(constraints.clamp(100), 100); // No max, so no upper limit
    }

    #[test]
    fn test_width_constraints_default() {
        let constraints = WidthConstraints::default();
        assert_eq!(constraints.min, 15);
        assert_eq!(constraints.max, Some(80));
        assert_eq!(constraints.preferred, 30);
    }

    #[test]
    fn test_card_sort_clone() {
        let sort = CardSort::Priority;
        let cloned = sort;
        assert_eq!(sort, cloned);
    }

    #[test]
    fn test_card_sort_equality() {
        assert_eq!(CardSort::Priority, CardSort::Priority);
        assert_eq!(CardSort::Title, CardSort::Title);
        assert_ne!(CardSort::Priority, CardSort::Created);
    }

    #[test]
    fn test_card_sort_default() {
        assert_eq!(CardSort::default(), CardSort::Priority);
    }

    #[test]
    fn test_column_definition_clone() {
        let col = ColumnDefinition::new(ColumnId::StatusOpen);
        let cloned = col.clone();
        assert_eq!(col.id, cloned.id);
        assert_eq!(col.label, cloned.label);
        assert_eq!(col.width, cloned.width);
        assert_eq!(col.visible, cloned.visible);
    }

    #[test]
    fn test_column_definition_new() {
        let col = ColumnDefinition::new(ColumnId::PriorityP2);
        assert_eq!(col.id, ColumnId::PriorityP2);
        assert_eq!(col.label, "P2 - Medium");
        assert!(col.visible);
        assert_eq!(col.width, col.width_constraints.preferred);
        assert_eq!(col.card_sort, CardSort::default());
    }

    #[test]
    fn test_board_filters_clone() {
        let filters = BoardFilters::default();
        let cloned = filters.clone();
        assert_eq!(filters.issue_types.len(), cloned.issue_types.len());
        assert_eq!(filters.priorities.len(), cloned.priorities.len());
        assert_eq!(filters.search_query, cloned.search_query);
    }

    #[test]
    fn test_board_filters_default() {
        let filters = BoardFilters::default();
        assert!(filters.issue_types.is_empty());
        assert!(filters.priorities.is_empty());
        assert!(filters.search_query.is_none());
    }

    #[test]
    fn test_kanban_config_new() {
        let config = KanbanConfig::new();
        assert_eq!(config.grouping_mode, GroupingMode::Status);
        assert_eq!(config.columns.len(), 4);
        assert_eq!(config.card_height, 3);
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_kanban_config_clone() {
        let config = KanbanConfig::default();
        let cloned = config.clone();
        assert_eq!(config.grouping_mode, cloned.grouping_mode);
        assert_eq!(config.columns.len(), cloned.columns.len());
        assert_eq!(config.card_height, cloned.card_height);
    }

    #[test]
    fn test_kanban_config_get_column_none() {
        let config = KanbanConfig::default();
        let col = config.get_column(&ColumnId::PriorityP0);
        assert!(col.is_none());
    }

    #[test]
    fn test_kanban_config_reorder_out_of_bounds() {
        let mut config = KanbanConfig::default();
        let original_order: Vec<_> = config.columns.iter().map(|c| c.id.clone()).collect();

        // Attempt to reorder with out-of-bounds indices
        config.reorder_column(10, 0);

        // Order should be unchanged
        let new_order: Vec<_> = config.columns.iter().map(|c| c.id.clone()).collect();
        assert_eq!(original_order, new_order);
    }

    #[test]
    fn test_default_columns_for_assignee_mode() {
        let cols = KanbanConfig::default_columns(GroupingMode::Assignee);
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].id, ColumnId::Unassigned);
    }

    #[test]
    fn test_default_columns_for_label_mode() {
        let cols = KanbanConfig::default_columns(GroupingMode::Label);
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].id, ColumnId::Unassigned);
    }

    #[test]
    fn test_default_columns_for_priority_mode() {
        let cols = KanbanConfig::default_columns(GroupingMode::Priority);
        assert_eq!(cols.len(), 5);
        assert!(cols.iter().any(|c| c.id == ColumnId::PriorityP0));
        assert!(cols.iter().any(|c| c.id == ColumnId::PriorityP4));
    }

    #[test]
    fn test_validate_enforces_mandatory_visibility() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![ColumnDefinition {
                id: ColumnId::StatusOpen,
                label: "Open".to_string(),
                width_constraints: WidthConstraints::default(),
                width: 30,
                visible: false, // Try to hide mandatory column
                card_sort: CardSort::default(),
            }],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Mandatory column should be forced visible
        let col = config
            .columns
            .iter()
            .find(|c| c.id == ColumnId::StatusOpen)
            .unwrap();
        assert!(col.visible);
    }

    #[test]
    fn test_validate_clamps_widths() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![ColumnDefinition {
                id: ColumnId::StatusOpen,
                label: "Open".to_string(),
                width_constraints: WidthConstraints::new(10, Some(50), 30),
                width: 200, // Way above max
                visible: true,
                card_sort: CardSort::default(),
            }],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        let col = &config.columns[0];
        assert_eq!(col.width, 50); // Clamped to max
    }

    #[test]
    fn test_is_valid_for_mode_priority() {
        let config = KanbanConfig {
            grouping_mode: GroupingMode::Priority,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        assert!(config.is_valid_for_mode(&ColumnId::PriorityP0));
        assert!(config.is_valid_for_mode(&ColumnId::PriorityP4));
        assert!(!config.is_valid_for_mode(&ColumnId::StatusOpen));
    }

    #[test]
    fn test_is_valid_for_mode_assignee() {
        let config = KanbanConfig {
            grouping_mode: GroupingMode::Assignee,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        assert!(config.is_valid_for_mode(&ColumnId::Assignee("bob".to_string())));
        assert!(config.is_valid_for_mode(&ColumnId::Unassigned));
        assert!(!config.is_valid_for_mode(&ColumnId::StatusOpen));
    }

    #[test]
    fn test_is_valid_for_mode_label() {
        let config = KanbanConfig {
            grouping_mode: GroupingMode::Label,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };

        assert!(config.is_valid_for_mode(&ColumnId::Label("bug".to_string())));
        assert!(config.is_valid_for_mode(&ColumnId::Unassigned));
        assert!(!config.is_valid_for_mode(&ColumnId::PriorityP0));
    }

    #[test]
    fn test_grouping_mode_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GroupingMode::Status);
        set.insert(GroupingMode::Assignee);
        set.insert(GroupingMode::Status); // Duplicate
        
        assert_eq!(set.len(), 2); // Only 2 unique values
    }

    #[test]
    fn test_column_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ColumnId::StatusOpen);
        set.insert(ColumnId::PriorityP0);
        set.insert(ColumnId::StatusOpen); // Duplicate
        
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_column_id_assignee_different_names() {
        let id1 = ColumnId::Assignee("alice".to_string());
        let id2 = ColumnId::Assignee("bob".to_string());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_column_id_label_different_names() {
        let id1 = ColumnId::Label("bug".to_string());
        let id2 = ColumnId::Label("feature".to_string());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_width_constraints_equality() {
        let c1 = WidthConstraints::new(10, Some(50), 30);
        let c2 = WidthConstraints::new(10, Some(50), 30);
        let c3 = WidthConstraints::new(15, Some(50), 30);
        
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_width_constraints_clamp_at_boundaries() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        assert_eq!(constraints.clamp(10), 10); // Exactly at min
        assert_eq!(constraints.clamp(50), 50); // Exactly at max
    }

    #[test]
    fn test_column_definition_set_width_clamped() {
        let mut col = ColumnDefinition::new(ColumnId::StatusOpen);
        let constraints = col.width_constraints;
        
        col.set_width(1000); // Very large
        assert!(col.width <= constraints.max.unwrap_or(u16::MAX));
    }

    #[test]
    fn test_column_definition_card_sort_field() {
        let mut col = ColumnDefinition::new(ColumnId::StatusOpen);
        col.card_sort = CardSort::Title;
        assert_eq!(col.card_sort, CardSort::Title);
    }

    #[test]
    fn test_kanban_config_default_columns_all_visible() {
        let columns = KanbanConfig::default_columns(GroupingMode::Status);
        assert!(columns.iter().all(|c| c.visible));
    }

    #[test]
    fn test_kanban_config_validate_restores_empty_columns() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Status,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };
        
        config = config.validate_and_migrate();
        
        // Should have restored default columns
        assert_eq!(config.columns.len(), 4);
    }

    #[test]
    fn test_kanban_config_validate_forces_mandatory_visible() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Priority,
            columns: vec![],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };
        
        config.columns = KanbanConfig::default_columns(GroupingMode::Priority);
        config.columns[0].visible = false; // Try to hide P0
        
        config = config.validate_and_migrate();
        
        // All mandatory columns should be visible
        assert!(config.columns.iter().all(|c| c.visible));
    }

    #[test]
    fn test_kanban_config_get_column_mut_none() {
        let mut config = KanbanConfig::default();
        let col = config.get_column_mut(&ColumnId::PriorityP0);
        assert!(col.is_none());
    }

    #[test]
    fn test_kanban_config_visible_columns_all() {
        let config = KanbanConfig::default();
        let visible = config.visible_columns();
        assert_eq!(visible.len(), config.columns.len());
    }

    #[test]
    fn test_kanban_config_toggle_visibility_non_mandatory() {
        let mut config = KanbanConfig {
            grouping_mode: GroupingMode::Assignee,
            columns: vec![
                ColumnDefinition::new(ColumnId::Unassigned),
                ColumnDefinition::new(ColumnId::Assignee("alice".to_string())),
            ],
            card_height: 3,
            filters: BoardFilters::default(),
            version: 1,
        };
        
        let assignee_id = ColumnId::Assignee("alice".to_string());
        config.toggle_column_visibility(&assignee_id);
        
        assert!(!config.get_column(&assignee_id).unwrap().visible);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = KanbanConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: KanbanConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.grouping_mode, deserialized.grouping_mode);
        assert_eq!(config.card_height, deserialized.card_height);
        assert_eq!(config.version, deserialized.version);
    }

    #[test]
    fn test_grouping_mode_serialization() {
        let mode = GroupingMode::Assignee;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: GroupingMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_column_id_serialization_dynamic() {
        let id = ColumnId::Assignee("test@example.com".to_string());
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ColumnId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_card_sort_serialization() {
        let sort = CardSort::Updated;
        let json = serde_json::to_string(&sort).unwrap();
        let deserialized: CardSort = serde_json::from_str(&json).unwrap();
        assert_eq!(sort, deserialized);
    }
}
