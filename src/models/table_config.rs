//! Table configuration for issue list with column management and persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fixed column widths for horizontal scrolling mode
/// These widths are used when displaying all columns with horizontal scrolling enabled
pub const FIXED_COLUMN_WIDTHS: [(ColumnId, u16); 9] = [
    (ColumnId::Id, 12),
    (ColumnId::Title, 40),
    (ColumnId::Status, 12),
    (ColumnId::Priority, 8),
    (ColumnId::Type, 10),
    (ColumnId::Assignee, 15),
    (ColumnId::Labels, 20),
    (ColumnId::Created, 10),
    (ColumnId::Updated, 10),
];

/// Column identifier for the issue table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnId {
    Id,
    Title,
    Status,
    Priority,
    Type,
    Assignee,
    Labels,
    Updated,
    Created,
}

impl ColumnId {
    /// Returns true if this column is mandatory and cannot be fully hidden
    pub fn is_mandatory(&self) -> bool {
        matches!(self, ColumnId::Id | ColumnId::Title)
    }

    /// Returns the default label for this column
    pub fn default_label(&self) -> &'static str {
        match self {
            ColumnId::Id => "ID",
            ColumnId::Title => "Title",
            ColumnId::Status => "Status",
            ColumnId::Priority => "Priority",
            ColumnId::Type => "Type",
            ColumnId::Assignee => "Assignee",
            ColumnId::Labels => "Labels",
            ColumnId::Updated => "Updated",
            ColumnId::Created => "Created",
        }
    }
}

/// Text alignment for column content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// Word wrap behavior for column content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapBehavior {
    /// No wrapping, truncate with ellipsis (at end)
    Truncate,
    /// Truncate at start (keep end characters)
    TruncateStart,
    /// Wrap at word boundaries
    Wrap,
    /// Wrap anywhere
    WrapAnywhere,
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

/// Column definition for the issue table
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
    /// Text alignment
    pub alignment: Alignment,
    /// Wrap behavior
    pub wrap: WrapBehavior,
    /// Whether the column is visible
    pub visible: bool,
}

impl ColumnDefinition {
    /// Create a new column definition with defaults
    pub fn new(id: ColumnId) -> Self {
        let (width_constraints, alignment, wrap) = match id {
            ColumnId::Id => (
                WidthConstraints::new(10, Some(20), 15),
                Alignment::Left,
                WrapBehavior::Truncate,
            ),
            ColumnId::Title => (
                WidthConstraints::new(20, None, 40),
                Alignment::Left,
                WrapBehavior::Wrap,
            ),
            ColumnId::Status => (
                WidthConstraints::new(8, Some(15), 12),
                Alignment::Left,
                WrapBehavior::Truncate,
            ),
            ColumnId::Priority => (
                WidthConstraints::new(6, Some(10), 8),
                Alignment::Center,
                WrapBehavior::Truncate,
            ),
            ColumnId::Type => (
                WidthConstraints::new(6, Some(10), 8),
                Alignment::Left,
                WrapBehavior::Truncate,
            ),
            ColumnId::Assignee => (
                WidthConstraints::new(8, Some(30), 15),
                Alignment::Left,
                WrapBehavior::Truncate,
            ),
            ColumnId::Labels => (
                WidthConstraints::new(10, None, 20),
                Alignment::Left,
                WrapBehavior::Truncate,
            ),
            ColumnId::Updated => (
                WidthConstraints::new(10, Some(20), 16),
                Alignment::Right,
                WrapBehavior::Truncate,
            ),
            ColumnId::Created => (
                WidthConstraints::new(10, Some(20), 16),
                Alignment::Right,
                WrapBehavior::Truncate,
            ),
        };

        Self {
            id,
            label: id.default_label().to_string(),
            width_constraints,
            width: width_constraints.preferred,
            alignment,
            wrap,
            visible: true,
        }
    }

    /// Set the column width (clamped to constraints)
    pub fn set_width(&mut self, width: u16) {
        self.width = self.width_constraints.clamp(width);
    }

    /// Set visibility (mandatory columns cannot be fully hidden)
    pub fn set_visible(&mut self, visible: bool) {
        if !self.id.is_mandatory() {
            self.visible = visible;
        }
    }
}

/// Sort configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortConfig {
    /// Column to sort by
    pub column: ColumnId,
    /// Sort direction
    pub ascending: bool,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            column: ColumnId::Updated,
            ascending: false,
        }
    }
}

/// Filter configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Active filters by column
    pub filters: HashMap<ColumnId, String>,
}

/// Complete table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// Column definitions in display order
    pub columns: Vec<ColumnDefinition>,
    /// Row height in lines
    pub row_height: u16,
    /// Sort configuration
    pub sort: SortConfig,
    /// Filter configuration
    pub filters: FilterConfig,
    /// Config version for migration
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for TableConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TableConfig {
    /// Create a new table configuration with default columns
    pub fn new() -> Self {
        Self {
            columns: Self::default_columns(),
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        }
    }

    /// Get the default column order and definitions
    fn default_columns() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition::new(ColumnId::Id),
            ColumnDefinition::new(ColumnId::Title),
            ColumnDefinition::new(ColumnId::Status),
            ColumnDefinition::new(ColumnId::Priority),
            ColumnDefinition::new(ColumnId::Type),
            ColumnDefinition::new(ColumnId::Assignee),
            ColumnDefinition::new(ColumnId::Labels),
            ColumnDefinition::new(ColumnId::Created),
            ColumnDefinition::new(ColumnId::Updated),
        ]
    }

    /// Validate and migrate configuration
    pub fn validate_and_migrate(mut self) -> Self {
        // Ensure mandatory columns are present and visible
        let has_id = self.columns.iter().any(|c| c.id == ColumnId::Id);
        let has_title = self.columns.iter().any(|c| c.id == ColumnId::Title);

        if !has_id {
            self.columns.insert(0, ColumnDefinition::new(ColumnId::Id));
        }
        if !has_title {
            // Insert Title after Id (position 1) since we just added Id if it was missing
            let insert_pos = if has_id {
                // Id was already present, find its position
                self.columns
                    .iter()
                    .position(|c| c.id == ColumnId::Id)
                    .map(|pos| pos + 1)
                    .unwrap_or(1)
            } else {
                // Id was just added at position 0
                1
            };
            self.columns
                .insert(insert_pos, ColumnDefinition::new(ColumnId::Title));
        }

        // Ensure mandatory columns are visible
        for col in &mut self.columns {
            if col.id.is_mandatory() {
                col.visible = true;
            }
        }

        // Validate widths are within constraints
        for col in &mut self.columns {
            col.width = col.width_constraints.clamp(col.width);
        }

        // Ensure row height is at least 1
        self.row_height = self.row_height.max(1);

        // Add any missing default columns at the end
        let existing_ids: Vec<ColumnId> = self.columns.iter().map(|c| c.id).collect();
        for default_col in Self::default_columns() {
            if !existing_ids.contains(&default_col.id) {
                self.columns.push(default_col);
            }
        }

        self
    }

    /// Get visible columns in display order
    pub fn visible_columns(&self) -> Vec<&ColumnDefinition> {
        self.columns.iter().filter(|c| c.visible).collect()
    }

    /// Get a column by ID
    pub fn get_column(&self, id: ColumnId) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|c| c.id == id)
    }

    /// Get a mutable column by ID
    pub fn get_column_mut(&mut self, id: ColumnId) -> Option<&mut ColumnDefinition> {
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
    pub fn set_column_width(&mut self, id: ColumnId, width: u16) {
        if let Some(col) = self.get_column_mut(id) {
            col.set_width(width);
        }
    }

    /// Toggle column visibility
    pub fn toggle_column_visibility(&mut self, id: ColumnId) {
        if let Some(col) = self.get_column_mut(id) {
            col.set_visible(!col.visible);
        }
    }

    /// Save table configuration to a file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, json)
    }

    /// Load table configuration from a file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let config: TableConfig = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Validate and migrate the loaded config
        Ok(config.validate_and_migrate())
    }

    /// Get fixed widths for all columns (for horizontal scrolling mode)
    /// Returns a HashMap mapping column IDs to their fixed widths
    pub fn get_fixed_widths() -> HashMap<ColumnId, u16> {
        FIXED_COLUMN_WIDTHS.iter().copied().collect()
    }

    /// Calculate the total width required for all visible columns using fixed widths
    pub fn total_fixed_width(&self) -> u16 {
        let fixed_widths = Self::get_fixed_widths();
        self.columns
            .iter()
            .filter(|c| c.visible)
            .filter_map(|c| fixed_widths.get(&c.id))
            .sum()
    }

    /// Apply fixed widths to all visible columns (for horizontal scrolling mode)
    pub fn apply_fixed_widths(&mut self) {
        let fixed_widths = Self::get_fixed_widths();
        for col in &mut self.columns {
            if let Some(&fixed_width) = fixed_widths.get(&col.id) {
                col.width = fixed_width;
            }
        }
    }

    /// Get the fixed width for a specific column ID
    pub fn get_fixed_width(id: ColumnId) -> Option<u16> {
        FIXED_COLUMN_WIDTHS
            .iter()
            .find(|(col_id, _)| *col_id == id)
            .map(|(_, width)| *width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_id_mandatory() {
        assert!(ColumnId::Id.is_mandatory());
        assert!(ColumnId::Title.is_mandatory());
        assert!(!ColumnId::Status.is_mandatory());
        assert!(!ColumnId::Priority.is_mandatory());
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
        let mut col = ColumnDefinition::new(ColumnId::Id);
        col.set_width(5); // Below min
        assert_eq!(col.width, col.width_constraints.min);

        col.set_width(100); // Above max
        assert_eq!(col.width, col.width_constraints.max.unwrap());
    }

    #[test]
    fn test_column_definition_set_visible() {
        // Non-mandatory column can be hidden
        let mut col = ColumnDefinition::new(ColumnId::Status);
        col.set_visible(false);
        assert!(!col.visible);

        // Mandatory column cannot be hidden
        let mut col = ColumnDefinition::new(ColumnId::Id);
        col.set_visible(false);
        assert!(col.visible);
    }

    #[test]
    fn test_table_config_default() {
        let config = TableConfig::default();
        assert_eq!(config.columns.len(), 9);
        assert_eq!(config.row_height, 1);
        assert_eq!(config.sort.column, ColumnId::Updated);
        assert!(!config.sort.ascending);
    }

    #[test]
    fn test_table_config_validate_mandatory_columns() {
        let mut config = TableConfig {
            columns: vec![ColumnDefinition::new(ColumnId::Status)],
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Should have added ID and Title at the beginning
        assert!(config.columns.iter().any(|c| c.id == ColumnId::Id));
        assert!(config.columns.iter().any(|c| c.id == ColumnId::Title));
        assert!(config.columns[0].id == ColumnId::Id);
        assert!(config.columns[1].id == ColumnId::Title);
    }

    #[test]
    fn test_table_config_validate_row_height() {
        let mut config = TableConfig {
            columns: TableConfig::default_columns(),
            row_height: 0,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        config = config.validate_and_migrate();
        assert_eq!(config.row_height, 1);
    }

    #[test]
    fn test_table_config_visible_columns() {
        let mut config = TableConfig::default();
        config.columns[2].visible = false; // Hide Status

        let visible = config.visible_columns();
        assert_eq!(visible.len(), 7);
        assert!(!visible.iter().any(|c| c.id == ColumnId::Status));
    }

    #[test]
    fn test_table_config_reorder_column() {
        let mut config = TableConfig::default();
        let first_id = config.columns[0].id;
        let third_id = config.columns[2].id;

        config.reorder_column(2, 0);

        assert_eq!(config.columns[0].id, third_id);
        assert_eq!(config.columns[1].id, first_id);
    }

    #[test]
    fn test_table_config_set_column_width() {
        let mut config = TableConfig::default();
        config.set_column_width(ColumnId::Title, 50);

        let title_col = config.get_column(ColumnId::Title).unwrap();
        assert_eq!(title_col.width, 50);
    }

    #[test]
    fn test_table_config_toggle_visibility() {
        let mut config = TableConfig::default();

        // Toggle non-mandatory column
        config.toggle_column_visibility(ColumnId::Status);
        assert!(!config.get_column(ColumnId::Status).unwrap().visible);

        config.toggle_column_visibility(ColumnId::Status);
        assert!(config.get_column(ColumnId::Status).unwrap().visible);

        // Toggle mandatory column (should remain visible)
        config.toggle_column_visibility(ColumnId::Id);
        assert!(config.get_column(ColumnId::Id).unwrap().visible);
    }

    #[test]
    fn test_serialization() {
        let config = TableConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TableConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.columns.len(), deserialized.columns.len());
        assert_eq!(config.row_height, deserialized.row_height);
    }

    #[test]
    fn test_column_id_equality() {
        assert_eq!(ColumnId::Id, ColumnId::Id);
        assert_eq!(ColumnId::Status, ColumnId::Status);
        assert_ne!(ColumnId::Id, ColumnId::Title);
        assert_ne!(ColumnId::Priority, ColumnId::Type);
    }

    #[test]
    fn test_column_id_clone() {
        let id = ColumnId::Updated;
        let cloned = id;
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_column_id_default_label_all_variants() {
        assert_eq!(ColumnId::Id.default_label(), "ID");
        assert_eq!(ColumnId::Title.default_label(), "Title");
        assert_eq!(ColumnId::Status.default_label(), "Status");
        assert_eq!(ColumnId::Priority.default_label(), "Priority");
        assert_eq!(ColumnId::Type.default_label(), "Type");
        assert_eq!(ColumnId::Assignee.default_label(), "Assignee");
        assert_eq!(ColumnId::Labels.default_label(), "Labels");
        assert_eq!(ColumnId::Updated.default_label(), "Updated");
        assert_eq!(ColumnId::Created.default_label(), "Created");
    }

    #[test]
    fn test_alignment_equality() {
        assert_eq!(Alignment::Left, Alignment::Left);
        assert_eq!(Alignment::Center, Alignment::Center);
        assert_ne!(Alignment::Left, Alignment::Right);
    }

    #[test]
    fn test_alignment_clone() {
        let align = Alignment::Right;
        let cloned = align;
        assert_eq!(align, cloned);
    }

    #[test]
    fn test_alignment_all_variants() {
        let left = Alignment::Left;
        let center = Alignment::Center;
        let right = Alignment::Right;
        assert_eq!(left, Alignment::Left);
        assert_eq!(center, Alignment::Center);
        assert_eq!(right, Alignment::Right);
    }

    #[test]
    fn test_wrap_behavior_equality() {
        assert_eq!(WrapBehavior::Truncate, WrapBehavior::Truncate);
        assert_eq!(WrapBehavior::Wrap, WrapBehavior::Wrap);
        assert_ne!(WrapBehavior::Truncate, WrapBehavior::WrapAnywhere);
    }

    #[test]
    fn test_wrap_behavior_clone() {
        let wrap = WrapBehavior::Wrap;
        let cloned = wrap;
        assert_eq!(wrap, cloned);
    }

    #[test]
    fn test_wrap_behavior_all_variants() {
        let truncate = WrapBehavior::Truncate;
        let wrap = WrapBehavior::Wrap;
        let wrap_anywhere = WrapBehavior::WrapAnywhere;
        assert_eq!(truncate, WrapBehavior::Truncate);
        assert_eq!(wrap, WrapBehavior::Wrap);
        assert_eq!(wrap_anywhere, WrapBehavior::WrapAnywhere);
    }

    #[test]
    fn test_width_constraints_new_min_greater_than_preferred() {
        let constraints = WidthConstraints::new(30, Some(50), 20);
        // Preferred should be clamped to min
        assert_eq!(constraints.preferred, 30);
        assert_eq!(constraints.min, 30);
        assert_eq!(constraints.max, Some(50));
    }

    #[test]
    fn test_width_constraints_clamp_no_max() {
        let constraints = WidthConstraints::new(10, None, 30);
        assert_eq!(constraints.clamp(5), 10); // Below min
        assert_eq!(constraints.clamp(100), 100); // No max, so 100 is OK
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
    fn test_column_definition_new_all_column_ids() {
        let id_col = ColumnDefinition::new(ColumnId::Id);
        assert_eq!(id_col.id, ColumnId::Id);
        assert_eq!(id_col.label, "ID");
        assert!(id_col.visible);

        let title_col = ColumnDefinition::new(ColumnId::Title);
        assert_eq!(title_col.id, ColumnId::Title);
        assert_eq!(title_col.alignment, Alignment::Left);
        assert_eq!(title_col.wrap, WrapBehavior::Wrap);

        let priority_col = ColumnDefinition::new(ColumnId::Priority);
        assert_eq!(priority_col.alignment, Alignment::Center);

        let updated_col = ColumnDefinition::new(ColumnId::Updated);
        assert_eq!(updated_col.alignment, Alignment::Right);
    }

    #[test]
    fn test_column_definition_clone() {
        let col = ColumnDefinition::new(ColumnId::Status);
        let cloned = col.clone();
        assert_eq!(col.id, cloned.id);
        assert_eq!(col.label, cloned.label);
        assert_eq!(col.visible, cloned.visible);
    }

    #[test]
    fn test_sort_config_default() {
        let sort = SortConfig::default();
        assert_eq!(sort.column, ColumnId::Updated);
        assert!(!sort.ascending);
    }

    #[test]
    fn test_sort_config_clone() {
        let sort = SortConfig {
            column: ColumnId::Priority,
            ascending: true,
        };
        let cloned = sort;
        assert_eq!(sort.column, cloned.column);
        assert_eq!(sort.ascending, cloned.ascending);
    }

    #[test]
    fn test_sort_config_equality() {
        let s1 = SortConfig {
            column: ColumnId::Status,
            ascending: true,
        };
        let s2 = SortConfig {
            column: ColumnId::Status,
            ascending: true,
        };
        let s3 = SortConfig {
            column: ColumnId::Priority,
            ascending: true,
        };
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_filter_config_default() {
        let filter = FilterConfig::default();
        assert!(filter.filters.is_empty());
    }

    #[test]
    fn test_filter_config_clone() {
        let mut filter = FilterConfig::default();
        filter.filters.insert(ColumnId::Status, "open".to_string());
        let cloned = filter.clone();
        assert_eq!(cloned.filters.len(), 1);
        assert_eq!(
            cloned.filters.get(&ColumnId::Status),
            Some(&"open".to_string())
        );
    }

    #[test]
    fn test_table_config_new() {
        let config = TableConfig::new();
        assert_eq!(config.columns.len(), 9);
        assert_eq!(config.row_height, 1);
        assert_eq!(config.sort.column, ColumnId::Updated);
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_table_config_clone() {
        let config = TableConfig::default();
        let cloned = config.clone();
        assert_eq!(config.columns.len(), cloned.columns.len());
        assert_eq!(config.row_height, cloned.row_height);
        assert_eq!(config.version, cloned.version);
    }

    #[test]
    fn test_table_config_get_column_none() {
        let mut config = TableConfig::default();
        // Remove Created column
        config.columns.retain(|c| c.id != ColumnId::Created);
        assert!(config.get_column(ColumnId::Created).is_none());
    }

    #[test]
    fn test_table_config_get_column_mut_none() {
        let mut config = TableConfig::default();
        config.columns.retain(|c| c.id != ColumnId::Labels);
        assert!(config.get_column_mut(ColumnId::Labels).is_none());
    }

    #[test]
    fn test_table_config_reorder_column_out_of_bounds() {
        let mut config = TableConfig::default();
        let original_len = config.columns.len();
        let original_first_id = config.columns[0].id;

        // Try to reorder with out of bounds indices
        config.reorder_column(100, 0);
        assert_eq!(config.columns.len(), original_len);
        assert_eq!(config.columns[0].id, original_first_id); // No change

        config.reorder_column(0, 100);
        assert_eq!(config.columns.len(), original_len);
        assert_eq!(config.columns[0].id, original_first_id); // No change
    }

    #[test]
    fn test_table_config_validate_missing_id_only() {
        let mut config = TableConfig {
            columns: vec![
                ColumnDefinition::new(ColumnId::Title),
                ColumnDefinition::new(ColumnId::Status),
            ],
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Should have added ID at position 0
        assert_eq!(config.columns[0].id, ColumnId::Id);
        assert_eq!(config.columns[1].id, ColumnId::Title);
        assert!(config.columns.iter().all(|c| {
            if c.id.is_mandatory() {
                c.visible
            } else {
                true
            }
        }));
    }

    #[test]
    fn test_table_config_validate_missing_title_only() {
        let mut config = TableConfig {
            columns: vec![
                ColumnDefinition::new(ColumnId::Id),
                ColumnDefinition::new(ColumnId::Status),
            ],
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Should have added Title at position 1 (after Id)
        assert_eq!(config.columns[0].id, ColumnId::Id);
        assert_eq!(config.columns[1].id, ColumnId::Title);
    }

    #[test]
    fn test_table_config_validate_widths() {
        let mut config = TableConfig::default();
        // Manually set a width outside constraints
        if let Some(col) = config.columns.iter_mut().find(|c| c.id == ColumnId::Id) {
            col.width = 5; // Below min (should be clamped to min=10)
        }

        config = config.validate_and_migrate();

        let id_col = config.get_column(ColumnId::Id).unwrap();
        assert_eq!(id_col.width, id_col.width_constraints.min);
    }

    #[test]
    fn test_column_id_copy_trait() {
        let id1 = ColumnId::Title;
        let id2 = id1;
        assert_eq!(id1, id2);
        // Both should still be usable after copy
        assert_eq!(id1, ColumnId::Title);
        assert_eq!(id2, ColumnId::Title);
    }

    #[test]
    fn test_all_column_id_inequalities() {
        let ids = vec![
            ColumnId::Id,
            ColumnId::Title,
            ColumnId::Status,
            ColumnId::Priority,
            ColumnId::Type,
            ColumnId::Assignee,
            ColumnId::Labels,
            ColumnId::Updated,
            ColumnId::Created,
        ];

        for (i, id1) in ids.iter().enumerate() {
            for (j, id2) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id1, id2);
                }
            }
        }
    }

    #[test]
    fn test_alignment_copy_trait() {
        let align1 = Alignment::Center;
        let align2 = align1;
        assert_eq!(align1, align2);
        assert_eq!(align1, Alignment::Center);
        assert_eq!(align2, Alignment::Center);
    }

    #[test]
    fn test_all_alignment_inequalities() {
        let aligns = [Alignment::Left, Alignment::Center, Alignment::Right];

        for (i, a1) in aligns.iter().enumerate() {
            for (j, a2) in aligns.iter().enumerate() {
                if i != j {
                    assert_ne!(a1, a2);
                }
            }
        }
    }

    #[test]
    fn test_wrap_behavior_copy_trait() {
        let wrap1 = WrapBehavior::Wrap;
        let wrap2 = wrap1;
        assert_eq!(wrap1, wrap2);
        assert_eq!(wrap1, WrapBehavior::Wrap);
        assert_eq!(wrap2, WrapBehavior::Wrap);
    }

    #[test]
    fn test_all_wrap_behavior_inequalities() {
        let wraps = [
            WrapBehavior::Truncate,
            WrapBehavior::Wrap,
            WrapBehavior::WrapAnywhere,
        ];

        for (i, w1) in wraps.iter().enumerate() {
            for (j, w2) in wraps.iter().enumerate() {
                if i != j {
                    assert_ne!(w1, w2);
                }
            }
        }
    }

    #[test]
    fn test_width_constraints_min_equals_max() {
        let constraints = WidthConstraints::new(20, Some(20), 25);
        assert_eq!(constraints.min, 20);
        assert_eq!(constraints.max, Some(20));
        assert_eq!(constraints.preferred, 25);

        // Clamping should work correctly
        assert_eq!(constraints.clamp(15), 20);
        assert_eq!(constraints.clamp(20), 20);
        assert_eq!(constraints.clamp(25), 20);
    }

    #[test]
    fn test_width_constraints_clamp_at_max() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        assert_eq!(constraints.clamp(60), 50);
        assert_eq!(constraints.clamp(50), 50);
        assert_eq!(constraints.clamp(40), 40);
    }

    #[test]
    fn test_column_definition_set_width_above_max() {
        let mut col = ColumnDefinition::new(ColumnId::Id);
        col.set_width(100); // Above max of 20
        assert_eq!(col.width, 20);
    }

    #[test]
    fn test_column_definition_set_width_below_min() {
        let mut col = ColumnDefinition::new(ColumnId::Id);
        col.set_width(5); // Below min of 10
        assert_eq!(col.width, 10);
    }

    #[test]
    fn test_column_definition_set_visible_non_mandatory() {
        let mut col = ColumnDefinition::new(ColumnId::Status);
        assert!(col.visible);

        col.set_visible(false);
        assert!(!col.visible);

        col.set_visible(true);
        assert!(col.visible);
    }

    #[test]
    fn test_column_definition_set_visible_mandatory_id() {
        let mut col = ColumnDefinition::new(ColumnId::Id);
        assert!(col.visible);

        col.set_visible(false);
        assert!(col.visible); // Should remain visible
    }

    #[test]
    fn test_column_definition_set_visible_mandatory_title() {
        let mut col = ColumnDefinition::new(ColumnId::Title);
        assert!(col.visible);

        col.set_visible(false);
        assert!(col.visible); // Should remain visible
    }

    #[test]
    fn test_sort_config_all_columns() {
        let columns = vec![
            ColumnId::Id,
            ColumnId::Title,
            ColumnId::Status,
            ColumnId::Priority,
            ColumnId::Type,
            ColumnId::Assignee,
            ColumnId::Labels,
            ColumnId::Updated,
            ColumnId::Created,
        ];

        for column in columns {
            let sort = SortConfig {
                column,
                ascending: true,
            };
            assert_eq!(sort.column, column);
            assert!(sort.ascending);
        }
    }

    #[test]
    fn test_sort_config_ascending_false() {
        let sort = SortConfig {
            column: ColumnId::Updated,
            ascending: false,
        };
        assert_eq!(sort.column, ColumnId::Updated);
        assert!(!sort.ascending);
    }

    #[test]
    fn test_filter_config_empty() {
        let config = FilterConfig::default();
        assert!(config.filters.is_empty());
    }

    #[test]
    fn test_filter_config_single_filter() {
        let mut config = FilterConfig::default();
        config.filters.insert(ColumnId::Status, "open".to_string());

        assert_eq!(config.filters.len(), 1);
        assert_eq!(
            config.filters.get(&ColumnId::Status),
            Some(&"open".to_string())
        );
    }

    #[test]
    fn test_filter_config_multiple_filters() {
        let mut config = FilterConfig::default();
        config.filters.insert(ColumnId::Status, "open".to_string());
        config
            .filters
            .insert(ColumnId::Priority, "high".to_string());
        config.filters.insert(ColumnId::Type, "bug".to_string());

        assert_eq!(config.filters.len(), 3);
        assert_eq!(
            config.filters.get(&ColumnId::Status),
            Some(&"open".to_string())
        );
        assert_eq!(
            config.filters.get(&ColumnId::Priority),
            Some(&"high".to_string())
        );
        assert_eq!(
            config.filters.get(&ColumnId::Type),
            Some(&"bug".to_string())
        );
    }

    #[test]
    fn test_table_config_all_columns_hidden_except_mandatory() {
        let mut config = TableConfig::default();

        // Try to hide all columns
        for col in &mut config.columns {
            col.set_visible(false);
        }

        // Mandatory columns should remain visible
        assert!(
            config
                .columns
                .iter()
                .find(|c| c.id == ColumnId::Id)
                .unwrap()
                .visible
        );
        assert!(
            config
                .columns
                .iter()
                .find(|c| c.id == ColumnId::Title)
                .unwrap()
                .visible
        );

        // Non-mandatory should be hidden
        if let Some(status) = config.columns.iter().find(|c| c.id == ColumnId::Status) {
            assert!(!status.visible);
        }
    }

    #[test]
    fn test_table_config_custom_column_order() {
        let config = TableConfig {
            columns: vec![
                ColumnDefinition::new(ColumnId::Priority),
                ColumnDefinition::new(ColumnId::Title),
                ColumnDefinition::new(ColumnId::Id),
                ColumnDefinition::new(ColumnId::Status),
            ],
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        assert_eq!(config.columns[0].id, ColumnId::Priority);
        assert_eq!(config.columns[1].id, ColumnId::Title);
        assert_eq!(config.columns[2].id, ColumnId::Id);
        assert_eq!(config.columns[3].id, ColumnId::Status);
    }

    #[test]
    fn test_table_config_very_large_row_height() {
        let config = TableConfig {
            columns: TableConfig::default_columns(),
            row_height: 1000,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        assert_eq!(config.row_height, 1000);
    }

    #[test]
    fn test_table_config_row_height_zero() {
        let config = TableConfig {
            columns: TableConfig::default_columns(),
            row_height: 0,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        assert_eq!(config.row_height, 0);
    }

    #[test]
    fn test_width_constraints_preferred_less_than_min() {
        let constraints = WidthConstraints::new(20, Some(50), 10);
        // Preferred should be clamped to min
        assert_eq!(constraints.preferred, 20);
    }

    #[test]
    fn test_column_definition_all_alignments() {
        let mut col = ColumnDefinition::new(ColumnId::Id);

        col.alignment = Alignment::Left;
        assert_eq!(col.alignment, Alignment::Left);

        col.alignment = Alignment::Center;
        assert_eq!(col.alignment, Alignment::Center);

        col.alignment = Alignment::Right;
        assert_eq!(col.alignment, Alignment::Right);
    }

    #[test]
    fn test_column_definition_all_wrap_behaviors() {
        let mut col = ColumnDefinition::new(ColumnId::Title);

        col.wrap = WrapBehavior::Truncate;
        assert_eq!(col.wrap, WrapBehavior::Truncate);

        col.wrap = WrapBehavior::Wrap;
        assert_eq!(col.wrap, WrapBehavior::Wrap);

        col.wrap = WrapBehavior::WrapAnywhere;
        assert_eq!(col.wrap, WrapBehavior::WrapAnywhere);
    }

    #[test]
    fn test_table_config_get_column_by_id() {
        let config = TableConfig::default();

        let id_col = config.get_column(ColumnId::Id);
        assert!(id_col.is_some());
        assert_eq!(id_col.unwrap().id, ColumnId::Id);

        let title_col = config.get_column(ColumnId::Title);
        assert!(title_col.is_some());
        assert_eq!(title_col.unwrap().id, ColumnId::Title);
    }

    #[test]
    fn test_table_config_get_column_mut_by_id() {
        let mut config = TableConfig::default();

        if let Some(col) = config.get_column_mut(ColumnId::Id) {
            col.width = 15;
        }

        let id_col = config.get_column(ColumnId::Id);
        assert_eq!(id_col.unwrap().width, 15);
    }

    #[test]
    fn test_table_config_reorder_column_to_end() {
        let mut config = TableConfig::default();
        let original_len = config.columns.len();

        // Find Id column index
        let id_index = config
            .columns
            .iter()
            .position(|c| c.id == ColumnId::Id)
            .unwrap();

        // Move Id column to end
        config.reorder_column(id_index, original_len - 1);

        assert_eq!(config.columns[original_len - 1].id, ColumnId::Id);
    }

    #[test]
    fn test_table_config_reorder_column_to_start() {
        let mut config = TableConfig::default();

        // Find Updated column index
        let updated_index = config
            .columns
            .iter()
            .position(|c| c.id == ColumnId::Updated)
            .unwrap();

        // Move Updated column to start
        config.reorder_column(updated_index, 0);

        assert_eq!(config.columns[0].id, ColumnId::Updated);
    }

    #[test]
    fn test_table_config_toggle_visibility_multiple_times() {
        let mut config = TableConfig::default();

        config.toggle_column_visibility(ColumnId::Status);
        let status = config.get_column(ColumnId::Status).unwrap();
        assert!(!status.visible);

        config.toggle_column_visibility(ColumnId::Status);
        let status = config.get_column(ColumnId::Status).unwrap();
        assert!(status.visible);

        config.toggle_column_visibility(ColumnId::Status);
        let status = config.get_column(ColumnId::Status).unwrap();
        assert!(!status.visible);
    }

    #[test]
    fn test_table_config_toggle_visibility_mandatory() {
        let mut config = TableConfig::default();

        config.toggle_column_visibility(ColumnId::Id);
        let id_col = config.get_column(ColumnId::Id).unwrap();
        assert!(id_col.visible); // Should remain visible

        config.toggle_column_visibility(ColumnId::Title);
        let title_col = config.get_column(ColumnId::Title).unwrap();
        assert!(title_col.visible); // Should remain visible
    }

    #[test]
    fn test_table_config_version() {
        let config = TableConfig::default();
        assert_eq!(config.version, 1);

        let custom_config = TableConfig {
            columns: TableConfig::default_columns(),
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 2,
        };
        assert_eq!(custom_config.version, 2);
    }

    #[test]
    fn test_column_id_hash_different() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ColumnId::Id);
        set.insert(ColumnId::Title);
        set.insert(ColumnId::Status);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&ColumnId::Id));
        assert!(set.contains(&ColumnId::Title));
        assert!(set.contains(&ColumnId::Status));
    }

    #[test]
    fn test_width_constraints_debug_trait() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        let debug_str = format!("{:?}", constraints);
        assert!(debug_str.contains("WidthConstraints"));
    }

    #[test]
    fn test_column_definition_debug_trait() {
        let col = ColumnDefinition::new(ColumnId::Id);
        let debug_str = format!("{:?}", col);
        assert!(debug_str.contains("ColumnDefinition"));
    }

    #[test]
    fn test_sort_config_copy_trait() {
        let sort1 = SortConfig::default();
        let sort2 = sort1;
        assert_eq!(sort1, sort2);
        assert_eq!(sort1.column, sort2.column);
        assert_eq!(sort1.ascending, sort2.ascending);
    }

    #[test]
    fn test_table_config_validate_both_mandatory_missing() {
        let mut config = TableConfig {
            columns: vec![
                ColumnDefinition::new(ColumnId::Status),
                ColumnDefinition::new(ColumnId::Priority),
            ],
            row_height: 1,
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        config = config.validate_and_migrate();

        // Should have added both Id and Title
        assert_eq!(config.columns[0].id, ColumnId::Id);
        assert_eq!(config.columns[1].id, ColumnId::Title);
        assert!(config.columns[0].visible);
        assert!(config.columns[1].visible);
    }

    #[test]
    fn test_width_constraints_clamp_exact_min() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        assert_eq!(constraints.clamp(10), 10);
    }

    #[test]
    fn test_width_constraints_clamp_exact_max() {
        let constraints = WidthConstraints::new(10, Some(50), 30);
        assert_eq!(constraints.clamp(50), 50);
    }

    #[test]
    fn test_filter_config_update_existing() {
        let mut config = FilterConfig::default();
        config.filters.insert(ColumnId::Status, "open".to_string());
        assert_eq!(
            config.filters.get(&ColumnId::Status),
            Some(&"open".to_string())
        );

        config
            .filters
            .insert(ColumnId::Status, "closed".to_string());
        assert_eq!(
            config.filters.get(&ColumnId::Status),
            Some(&"closed".to_string())
        );
    }

    #[test]
    fn test_column_definition_label_customization() {
        let mut col = ColumnDefinition::new(ColumnId::Id);
        assert_eq!(col.label, "ID");

        col.label = "Identifier".to_string();
        assert_eq!(col.label, "Identifier");
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("beads_tui_test_config.json");

        // Create a custom config
        let mut config = TableConfig::default();
        config.row_height = 3;
        config.set_column_width(ColumnId::Title, 60);
        config.toggle_column_visibility(ColumnId::Labels);

        // Save
        config.save_to_file(&config_path).unwrap();

        // Load
        let loaded = TableConfig::load_from_file(&config_path).unwrap();

        // Verify
        assert_eq!(loaded.row_height, 3);
        assert_eq!(loaded.get_column(ColumnId::Title).unwrap().width, 60);
        assert!(!loaded.get_column(ColumnId::Labels).unwrap().visible);

        // Clean up
        std::fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_load_config_creates_parent_dir() {
        let temp_dir = std::env::temp_dir();
        let nested_path = temp_dir
            .join("beads_tui_test")
            .join("nested")
            .join("config.json");

        // Clean up first if it exists
        if let Some(parent) = nested_path.parent() {
            std::fs::remove_dir_all(parent).ok();
        }

        let config = TableConfig::default();
        config.save_to_file(&nested_path).unwrap();

        assert!(nested_path.exists());

        // Clean up
        if let Some(parent) = nested_path.parent() {
            std::fs::remove_dir_all(parent.parent().unwrap()).ok();
        }
    }

    #[test]
    fn test_load_config_validates() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("beads_tui_test_invalid_config.json");

        // Create an invalid config (missing mandatory columns)
        let invalid_config = TableConfig {
            columns: vec![ColumnDefinition::new(ColumnId::Status)],
            row_height: 0, // Invalid: should be at least 1
            sort: SortConfig::default(),
            filters: FilterConfig::default(),
            version: 1,
        };

        // Save the invalid config
        invalid_config.save_to_file(&config_path).unwrap();

        // Load and validate
        let loaded = TableConfig::load_from_file(&config_path).unwrap();

        // Should have mandatory columns added
        assert!(loaded.columns.iter().any(|c| c.id == ColumnId::Id));
        assert!(loaded.columns.iter().any(|c| c.id == ColumnId::Title));
        // Should have fixed row height
        assert_eq!(loaded.row_height, 1);

        // Clean up
        std::fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("nonexistent_beads_tui_config.json");

        let result = TableConfig::load_from_file(&config_path);
        assert!(result.is_err());
    }
}
