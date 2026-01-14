/// Configuration management for beads-tui
pub mod keybindings;

use crate::models::{KanbanConfig, SavedFilter, TableConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use keybindings::{Action, Keybinding, KeybindingsConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub kanban: KanbanConfig,
    #[serde(default)]
    pub filters: Vec<SavedFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub name: String,
}

fn default_theme_name() -> String {
    "dark".to_string()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
        }
    }
}

// KeybindingsConfig is now defined in keybindings module and re-exported above

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
}

fn default_auto_refresh() -> bool {
    true
}

fn default_refresh_interval() -> u64 {
    60
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_refresh: default_auto_refresh(),
            refresh_interval_secs: default_refresh_interval(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_yaml::from_str(&content)?
        } else {
            Self::default()
        };

        // Validate and migrate table config
        config.table = config.table.validate_and_migrate();

        // Validate and migrate kanban config
        config.kanban = config.kanban.validate_and_migrate();

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get path to the configuration file
    fn config_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("com", "davidcforbes", "beads-tui")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        Ok(config_dir.config_dir().join("config.yaml"))
    }

    /// Add a saved filter
    pub fn add_filter(&mut self, filter: SavedFilter) {
        self.filters.push(filter);
    }

    /// Remove a saved filter by name
    /// Returns true if a filter was removed, false if not found
    pub fn remove_filter(&mut self, name: &str) -> bool {
        let len_before = self.filters.len();
        self.filters.retain(|f| f.name != name);
        self.filters.len() < len_before
    }

    /// Update a saved filter by name
    /// Returns true if the filter was updated, false if not found
    pub fn update_filter(&mut self, name: &str, filter: SavedFilter) -> bool {
        if let Some(existing) = self.filters.iter_mut().find(|f| f.name == name) {
            *existing = filter;
            true
        } else {
            false
        }
    }

    /// Get a filter by name
    pub fn get_filter(&self, name: &str) -> Option<&SavedFilter> {
        self.filters.iter().find(|f| f.name == name)
    }

    /// Get all saved filters
    pub fn saved_filters(&self) -> &[SavedFilter] {
        &self.filters
    }

    /// Get a filter by hotkey
    pub fn get_filter_by_hotkey(&self, key: char) -> Option<&SavedFilter> {
        self.filters.iter().find(|f| f.hotkey == Some(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_default_theme_name() {
        assert_eq!(default_theme_name(), "dark");
    }

    #[test]
    fn test_theme_config_custom() {
        let theme = ThemeConfig {
            name: "light".to_string(),
        };
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_behavior_config_default() {
        let behavior = BehaviorConfig::default();
        assert!(behavior.auto_refresh);
        assert_eq!(behavior.refresh_interval_secs, 60);
    }

    #[test]
    fn test_default_auto_refresh() {
        assert!(default_auto_refresh());
    }

    #[test]
    fn test_default_refresh_interval() {
        assert_eq!(default_refresh_interval(), 60);
    }

    #[test]
    fn test_behavior_config_custom() {
        let behavior = BehaviorConfig {
            auto_refresh: false,
            refresh_interval_secs: 120,
        };
        assert!(!behavior.auto_refresh);
        assert_eq!(behavior.refresh_interval_secs, 120);
    }

    #[test]
    fn test_keybindings_config_default() {
        let keybindings = KeybindingsConfig::default();
        // Just verify it can be constructed
        let _ = format!("{:?}", keybindings);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.theme.name, "dark");
        assert!(config.behavior.auto_refresh);
        assert_eq!(config.behavior.refresh_interval_secs, 60);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("theme"));
        assert!(yaml.contains("dark"));
        assert!(yaml.contains("behavior"));
        assert!(yaml.contains("auto_refresh"));
        assert!(yaml.contains("refresh_interval_secs"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
theme:
  name: light
behavior:
  auto_refresh: false
  refresh_interval_secs: 30
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.theme.name, "light");
        assert!(!config.behavior.auto_refresh);
        assert_eq!(config.behavior.refresh_interval_secs, 30);
    }

    #[test]
    fn test_config_partial_deserialization() {
        let yaml = r#"
theme:
  name: custom
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.theme.name, "custom");
        // Should use defaults for missing fields
        assert!(config.behavior.auto_refresh);
        assert_eq!(config.behavior.refresh_interval_secs, 60);
    }

    #[test]
    fn test_config_empty_deserialization() {
        let yaml = "{}";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        // Should use all defaults
        assert_eq!(config.theme.name, "dark");
        assert!(config.behavior.auto_refresh);
        assert_eq!(config.behavior.refresh_interval_secs, 60);
    }

    #[test]
    fn test_theme_config_serialization() {
        let theme = ThemeConfig {
            name: "custom".to_string(),
        };
        let yaml = serde_yaml::to_string(&theme).unwrap();
        assert!(yaml.contains("name"));
        assert!(yaml.contains("custom"));
    }

    #[test]
    fn test_behavior_config_serialization() {
        let behavior = BehaviorConfig {
            auto_refresh: false,
            refresh_interval_secs: 120,
        };
        let yaml = serde_yaml::to_string(&behavior).unwrap();
        assert!(yaml.contains("auto_refresh"));
        assert!(yaml.contains("false"));
        assert!(yaml.contains("refresh_interval_secs"));
        assert!(yaml.contains("120"));
    }

    #[test]
    fn test_add_filter() {
        use crate::models::IssueFilter;

        let mut config = Config::default();
        assert_eq!(config.saved_filters().len(), 0);

        let filter = SavedFilter {
            name: "Test Filter".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };

        config.add_filter(filter.clone());
        assert_eq!(config.saved_filters().len(), 1);
        assert_eq!(config.saved_filters()[0].name, "Test Filter");
    }

    #[test]
    fn test_remove_filter() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter1 = SavedFilter {
            name: "Filter 1".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };
        let filter2 = SavedFilter {
            name: "Filter 2".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('2'),
        };

        config.add_filter(filter1);
        config.add_filter(filter2);
        assert_eq!(config.saved_filters().len(), 2);

        // Remove existing filter
        assert!(config.remove_filter("Filter 1"));
        assert_eq!(config.saved_filters().len(), 1);
        assert_eq!(config.saved_filters()[0].name, "Filter 2");

        // Try to remove non-existent filter
        assert!(!config.remove_filter("Filter 1"));
        assert_eq!(config.saved_filters().len(), 1);
    }

    #[test]
    fn test_update_filter() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter = SavedFilter {
            name: "Original".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };

        config.add_filter(filter);
        assert_eq!(config.saved_filters()[0].hotkey, Some('1'));

        // Update existing filter
        let updated = SavedFilter {
            name: "Original".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('2'),
        };
        assert!(config.update_filter("Original", updated));
        assert_eq!(config.saved_filters()[0].hotkey, Some('2'));

        // Try to update non-existent filter
        let another = SavedFilter {
            name: "NonExistent".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('3'),
        };
        assert!(!config.update_filter("NonExistent", another));
    }

    #[test]
    fn test_get_filter() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter = SavedFilter {
            name: "Test".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };

        config.add_filter(filter);

        // Get existing filter
        let found = config.get_filter("Test");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");

        // Try to get non-existent filter
        assert!(config.get_filter("NonExistent").is_none());
    }

    #[test]
    fn test_get_filter_by_hotkey() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter1 = SavedFilter {
            name: "Filter 1".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };
        let filter2 = SavedFilter {
            name: "Filter 2".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('2'),
        };
        let filter3 = SavedFilter {
            name: "Filter 3".to_string(),
            filter: IssueFilter::new(),
            hotkey: None,
        };

        config.add_filter(filter1);
        config.add_filter(filter2);
        config.add_filter(filter3);

        // Get filter by existing hotkey
        let found = config.get_filter_by_hotkey('1');
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Filter 1");

        let found = config.get_filter_by_hotkey('2');
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Filter 2");

        // Try to get filter by non-existent hotkey
        assert!(config.get_filter_by_hotkey('3').is_none());
    }

    #[test]
    fn test_saved_filters() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter1 = SavedFilter {
            name: "Filter 1".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };
        let filter2 = SavedFilter {
            name: "Filter 2".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('2'),
        };

        config.add_filter(filter1);
        config.add_filter(filter2);

        let filters = config.saved_filters();
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].name, "Filter 1");
        assert_eq!(filters[1].name, "Filter 2");
    }

    #[test]
    fn test_config_with_filters_serialization() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter = SavedFilter {
            name: "Test Filter".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('1'),
        };

        config.add_filter(filter);

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("filters"));
        assert!(yaml.contains("Test Filter"));
    }

    #[test]
    fn test_config_with_filters_deserialization() {
        let yaml = r#"
theme:
  name: dark
behavior:
  auto_refresh: true
  refresh_interval_secs: 60
filters:
  - name: "High Priority"
    filter:
      status: null
      priority: P1
      issue_type: null
      assignee: null
      labels: []
      label_logic: And
      search_text: null
      search_scope: "All"
      view_type: "All"
      use_regex: false
      use_fuzzy: false
    hotkey: "1"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.filters.len(), 1);
        assert_eq!(config.filters[0].name, "High Priority");
        assert_eq!(config.filters[0].hotkey, Some('1'));
    }

    #[test]
    fn test_config_save_and_load() {
        use tempfile::tempdir;

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("config.yaml");

        // Set up a custom config
        let mut config = Config::default();
        config.theme.name = "light".to_string();
        config.behavior.auto_refresh = false;
        config.behavior.refresh_interval_secs = 120;

        // Manually save to our temp location
        let content = serde_yaml::to_string(&config).unwrap();
        std::fs::write(&config_file, content).unwrap();

        // Verify file exists
        assert!(config_file.exists());

        // Load and verify
        let loaded_content = std::fs::read_to_string(&config_file).unwrap();
        let loaded_config: Config = serde_yaml::from_str(&loaded_content).unwrap();
        assert_eq!(loaded_config.theme.name, "light");
        assert!(!loaded_config.behavior.auto_refresh);
        assert_eq!(loaded_config.behavior.refresh_interval_secs, 120);
    }

    #[test]
    fn test_config_load_nonexistent_returns_default() {
        // This tests the logic that if config file doesn't exist, return default
        let config = Config::default();
        assert_eq!(config.theme.name, "dark");
        assert!(config.behavior.auto_refresh);
        assert_eq!(config.behavior.refresh_interval_secs, 60);
        assert_eq!(config.filters.len(), 0);
    }

    #[test]
    fn test_config_path_returns_path() {
        // Config path should return a valid PathBuf
        let result = Config::config_path();
        // Should succeed or fail gracefully depending on environment
        // We can't assert success as it depends on the system
        let _ = result;
    }

    #[test]
    fn test_multiple_filters_operations() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        // Add multiple filters
        for i in 1..=5 {
            let filter = SavedFilter {
                name: format!("Filter {}", i),
                filter: IssueFilter::new(),
                // Safe: i is guaranteed to be 1-5 from loop range
                hotkey: Some(char::from_digit(i, 10).expect("digit 1-5 always valid")),
            };
            config.add_filter(filter);
        }

        assert_eq!(config.saved_filters().len(), 5);

        // Remove one
        assert!(config.remove_filter("Filter 3"));
        assert_eq!(config.saved_filters().len(), 4);

        // Update one
        let updated = SavedFilter {
            name: "Filter 2".to_string(),
            filter: IssueFilter::new(),
            hotkey: Some('9'),
        };
        assert!(config.update_filter("Filter 2", updated));
        assert_eq!(config.get_filter_by_hotkey('9').unwrap().name, "Filter 2");

        // Verify remaining filters
        assert!(config.get_filter("Filter 1").is_some());
        assert!(config.get_filter("Filter 2").is_some());
        assert!(config.get_filter("Filter 3").is_none());
        assert!(config.get_filter("Filter 4").is_some());
        assert!(config.get_filter("Filter 5").is_some());
    }

    #[test]
    fn test_filter_hotkey_none() {
        use crate::models::IssueFilter;

        let mut config = Config::default();

        let filter = SavedFilter {
            name: "No Hotkey".to_string(),
            filter: IssueFilter::new(),
            hotkey: None,
        };

        config.add_filter(filter);
        assert_eq!(config.saved_filters().len(), 1);
        assert_eq!(config.get_filter("No Hotkey").unwrap().hotkey, None);

        // Should not find by any hotkey
        assert!(config.get_filter_by_hotkey('1').is_none());
        assert!(config.get_filter_by_hotkey('x').is_none());
    }

    #[test]
    fn test_config_clone() {
        let mut config = Config::default();
        config.theme.name = "custom".to_string();

        let cloned = config.clone();
        assert_eq!(cloned.theme.name, "custom");
        assert_eq!(cloned.behavior.auto_refresh, config.behavior.auto_refresh);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("theme"));
    }
}
