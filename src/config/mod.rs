/// Configuration management for beads-tui
use crate::models::{KanbanConfig, TableConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeybindingsConfig {
    // Future: Custom keybindings
}

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

    /// Get the path to the configuration file
    fn config_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("com", "davidcforbes", "beads-tui")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        Ok(config_dir.config_dir().join("config.yaml"))
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
}
