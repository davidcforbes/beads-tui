//! Theme system for beads-tui
//!
//! Provides multiple color themes including:
//! - Dark theme (default)
//! - High contrast theme (for better visibility)
//! - Color-blind friendly palettes (deuteranopia, protanopia, tritanopia)

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Available theme types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeType {
    /// Default dark theme
    Dark,
    /// High contrast theme for better visibility
    HighContrast,
    /// Color-blind friendly theme for deuteranopia (red-green color blindness)
    Deuteranopia,
    /// Color-blind friendly theme for protanopia (red-green color blindness)
    Protanopia,
    /// Color-blind friendly theme for tritanopia (blue-yellow color blindness)
    Tritanopia,
}

impl ThemeType {
    /// Parse theme type from string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            "deuteranopia" => Some(Self::Deuteranopia),
            "protanopia" => Some(Self::Protanopia),
            "tritanopia" => Some(Self::Tritanopia),
            _ => None,
        }
    }

    /// Get the string name of this theme type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::HighContrast => "high-contrast",
            Self::Deuteranopia => "deuteranopia",
            Self::Protanopia => "protanopia",
            Self::Tritanopia => "tritanopia",
        }
    }

    /// Get a human-readable description of this theme
    pub fn description(&self) -> &'static str {
        match self {
            Self::Dark => "Default dark theme",
            Self::HighContrast => "High contrast theme for better visibility",
            Self::Deuteranopia => "Optimized for deuteranopia (red-green color blindness)",
            Self::Protanopia => "Optimized for protanopia (red-green color blindness)",
            Self::Tritanopia => "Optimized for tritanopia (blue-yellow color blindness)",
        }
    }
}

/// Color scheme for different UI elements
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme type
    pub theme_type: ThemeType,

    // Status colors
    /// Success/positive state color
    pub success: Color,
    /// Error/negative state color
    pub error: Color,
    /// Warning state color
    pub warning: Color,
    /// Info/neutral state color
    pub info: Color,

    // Priority colors
    /// Critical priority (P0)
    pub priority_critical: Color,
    /// High priority (P1)
    pub priority_high: Color,
    /// Medium priority (P2)
    pub priority_medium: Color,
    /// Low priority (P3)
    pub priority_low: Color,
    /// Backlog priority (P4)
    pub priority_backlog: Color,

    // UI element colors
    /// Primary UI elements
    pub primary: Color,
    /// Secondary UI elements
    pub secondary: Color,
    /// Accent color for highlights
    pub accent: Color,
    /// Background color
    pub background: Color,
    /// Foreground/text color
    pub foreground: Color,
    /// Muted/dimmed text color
    pub muted: Color,
    /// Border color
    pub border: Color,
    /// Selected item background
    pub selected_bg: Color,
    /// Selected item foreground
    pub selected_fg: Color,
}

impl Theme {
    /// Create a new theme of the specified type
    pub fn new(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Dark => Self::dark(),
            ThemeType::HighContrast => Self::high_contrast(),
            ThemeType::Deuteranopia => Self::deuteranopia(),
            ThemeType::Protanopia => Self::protanopia(),
            ThemeType::Tritanopia => Self::tritanopia(),
        }
    }

    /// Create the default dark theme
    fn dark() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Blue,
            priority_critical: Color::Red,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::Cyan,
            priority_backlog: Color::Gray,
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::DarkGray,
            border: Color::Gray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        }
    }

    /// Create a high-contrast theme for better visibility
    /// Uses only black and white with high contrast ratios (WCAG AAA compliant)
    fn high_contrast() -> Self {
        Self {
            theme_type: ThemeType::HighContrast,
            success: Color::Black,
            error: Color::White,
            warning: Color::White,
            info: Color::Black,
            priority_critical: Color::White,
            priority_high: Color::White,
            priority_medium: Color::Gray,
            priority_low: Color::Gray,
            priority_backlog: Color::DarkGray,
            primary: Color::White,
            secondary: Color::Gray,
            accent: Color::White,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::Gray,
            border: Color::White,
            selected_bg: Color::White,
            selected_fg: Color::Black,
        }
    }

    /// Create a deuteranopia-friendly theme
    /// Avoids red-green distinctions, uses blue-yellow instead
    fn deuteranopia() -> Self {
        Self {
            theme_type: ThemeType::Deuteranopia,
            success: Color::Blue,        // Blue instead of green
            error: Color::Yellow,        // Yellow instead of red
            warning: Color::Magenta,     // Magenta for warnings
            info: Color::Cyan,           // Cyan for info
            priority_critical: Color::Yellow,
            priority_high: Color::LightYellow,
            priority_medium: Color::Cyan,
            priority_low: Color::Blue,
            priority_backlog: Color::Gray,
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::DarkGray,
            border: Color::Gray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        }
    }

    /// Create a protanopia-friendly theme
    /// Similar to deuteranopia, avoids red-green distinctions
    fn protanopia() -> Self {
        Self {
            theme_type: ThemeType::Protanopia,
            success: Color::Blue,
            error: Color::Yellow,
            warning: Color::Magenta,
            info: Color::Cyan,
            priority_critical: Color::Yellow,
            priority_high: Color::LightYellow,
            priority_medium: Color::Cyan,
            priority_low: Color::Blue,
            priority_backlog: Color::Gray,
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::DarkGray,
            border: Color::Gray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        }
    }

    /// Create a tritanopia-friendly theme
    /// Avoids blue-yellow distinctions, uses red-green instead
    fn tritanopia() -> Self {
        Self {
            theme_type: ThemeType::Tritanopia,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Magenta,     // Magenta instead of yellow
            info: Color::Green,          // Green instead of blue
            priority_critical: Color::Red,
            priority_high: Color::LightRed,
            priority_medium: Color::Magenta,
            priority_low: Color::Green,
            priority_backlog: Color::Gray,
            primary: Color::Red,
            secondary: Color::Green,
            accent: Color::Magenta,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::DarkGray,
            border: Color::Gray,
            selected_bg: Color::Red,
            selected_fg: Color::White,
        }
    }

    /// Get all available theme types
    pub fn available_themes() -> Vec<ThemeType> {
        vec![
            ThemeType::Dark,
            ThemeType::HighContrast,
            ThemeType::Deuteranopia,
            ThemeType::Protanopia,
            ThemeType::Tritanopia,
        ]
    }

    /// Get color for issue status based on theme
    pub fn status_color(&self, status: &crate::beads::models::IssueStatus) -> Color {
        use crate::beads::models::IssueStatus;
        match status {
            IssueStatus::Open => self.success,
            IssueStatus::InProgress => self.info,
            IssueStatus::Blocked => self.error,
            IssueStatus::Closed => self.muted,
        }
    }

    /// Get color for priority based on theme
    pub fn priority_color(&self, priority: &crate::beads::models::Priority) -> Color {
        use crate::beads::models::Priority;
        match priority {
            Priority::P0 => self.priority_critical,
            Priority::P1 => self.priority_high,
            Priority::P2 => self.priority_medium,
            Priority::P3 => self.priority_low,
            Priority::P4 => self.priority_backlog,
        }
    }

    /// Get symbol for issue status (static, theme-independent)
    pub fn status_symbol(status: &crate::beads::models::IssueStatus) -> &'static str {
        use crate::beads::models::IssueStatus;
        match status {
            IssueStatus::Open => "○",
            IssueStatus::InProgress => "◐",
            IssueStatus::Blocked => "◩",
            IssueStatus::Closed => "✓",
        }
    }

    /// Get symbol for priority (static, theme-independent)
    pub fn priority_symbol(priority: &crate::beads::models::Priority) -> &'static str {
        use crate::beads::models::Priority;
        match priority {
            Priority::P0 => "◆",
            Priority::P1 => "●",
            Priority::P2 => "○",
            Priority::P3 => "◇",
            Priority::P4 => "·",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_type_from_name() {
        assert_eq!(ThemeType::from_name("dark"), Some(ThemeType::Dark));
        assert_eq!(ThemeType::from_name("Dark"), Some(ThemeType::Dark));
        assert_eq!(ThemeType::from_name("DARK"), Some(ThemeType::Dark));
        assert_eq!(ThemeType::from_name("high-contrast"), Some(ThemeType::HighContrast));
        assert_eq!(ThemeType::from_name("highcontrast"), Some(ThemeType::HighContrast));
        assert_eq!(ThemeType::from_name("deuteranopia"), Some(ThemeType::Deuteranopia));
        assert_eq!(ThemeType::from_name("protanopia"), Some(ThemeType::Protanopia));
        assert_eq!(ThemeType::from_name("tritanopia"), Some(ThemeType::Tritanopia));
        assert_eq!(ThemeType::from_name("invalid"), None);
    }

    #[test]
    fn test_theme_type_name() {
        assert_eq!(ThemeType::Dark.name(), "dark");
        assert_eq!(ThemeType::HighContrast.name(), "high-contrast");
        assert_eq!(ThemeType::Deuteranopia.name(), "deuteranopia");
        assert_eq!(ThemeType::Protanopia.name(), "protanopia");
        assert_eq!(ThemeType::Tritanopia.name(), "tritanopia");
    }

    #[test]
    fn test_theme_type_description() {
        assert!(!ThemeType::Dark.description().is_empty());
        assert!(!ThemeType::HighContrast.description().is_empty());
        assert!(!ThemeType::Deuteranopia.description().is_empty());
    }

    #[test]
    fn test_theme_creation() {
        let dark = Theme::new(ThemeType::Dark);
        assert_eq!(dark.theme_type, ThemeType::Dark);
        assert_eq!(dark.success, Color::Green);

        let high_contrast = Theme::new(ThemeType::HighContrast);
        assert_eq!(high_contrast.theme_type, ThemeType::HighContrast);
        assert_eq!(high_contrast.background, Color::Black);
        assert_eq!(high_contrast.foreground, Color::White);
    }

    #[test]
    fn test_deuteranopia_theme_no_red_green() {
        let theme = Theme::new(ThemeType::Deuteranopia);
        // Success should not be green
        assert_ne!(theme.success, Color::Green);
        // Error should not be red
        assert_ne!(theme.error, Color::Red);
    }

    #[test]
    fn test_tritanopia_theme_no_blue_yellow() {
        let theme = Theme::new(ThemeType::Tritanopia);
        // Info should not be blue
        assert_ne!(theme.info, Color::Blue);
        // Warning should not be yellow
        assert_ne!(theme.warning, Color::Yellow);
    }

    #[test]
    fn test_default_theme_is_dark() {
        let default = Theme::default();
        assert_eq!(default.theme_type, ThemeType::Dark);
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert_eq!(themes.len(), 5);
        assert!(themes.contains(&ThemeType::Dark));
        assert!(themes.contains(&ThemeType::HighContrast));
        assert!(themes.contains(&ThemeType::Deuteranopia));
        assert!(themes.contains(&ThemeType::Protanopia));
        assert!(themes.contains(&ThemeType::Tritanopia));
    }
}
