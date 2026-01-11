//! Responsive layout utilities for different terminal sizes

use ratatui::layout::Rect;

/// Terminal size categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalSize {
    /// Very small terminals (< 80x24)
    Tiny,
    /// Small terminals (80x24 to 120x40)
    Small,
    /// Medium terminals (120x40 to 160x50)
    Medium,
    /// Large terminals (> 160x50)
    Large,
}

impl TerminalSize {
    /// Determine terminal size category from dimensions
    pub fn from_rect(rect: Rect) -> Self {
        match (rect.width, rect.height) {
            (w, h) if w < 80 || h < 24 => Self::Tiny,
            (w, h) if w < 120 || h < 40 => Self::Small,
            (w, h) if w < 160 || h < 50 => Self::Medium,
            _ => Self::Large,
        }
    }

    /// Check if terminal is too small for optimal use
    pub fn is_too_small(&self) -> bool {
        matches!(self, Self::Tiny)
    }

    /// Get recommended number of visible tabs for this size
    pub fn max_visible_tabs(&self) -> usize {
        match self {
            Self::Tiny => 3,
            Self::Small => 5,
            Self::Medium => 7,
            Self::Large => 10,
        }
    }

    /// Get recommended content padding for this size
    pub fn content_padding(&self) -> u16 {
        match self {
            Self::Tiny => 0,
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 2,
        }
    }

    /// Check if side panels should be shown
    pub fn show_side_panels(&self) -> bool {
        !matches!(self, Self::Tiny | Self::Small)
    }
}

/// Check if the terminal size is adequate
pub fn check_terminal_size(rect: Rect) -> Result<(), String> {
    const MIN_WIDTH: u16 = 60;
    const MIN_HEIGHT: u16 = 20;

    if rect.width < MIN_WIDTH || rect.height < MIN_HEIGHT {
        Err(format!(
            "Terminal too small: {}x{} (minimum: {}x{})",
            rect.width, rect.height, MIN_WIDTH, MIN_HEIGHT
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_detection() {
        assert_eq!(
            TerminalSize::from_rect(Rect::new(0, 0, 60, 20)),
            TerminalSize::Tiny
        );
        assert_eq!(
            TerminalSize::from_rect(Rect::new(0, 0, 100, 30)),
            TerminalSize::Small
        );
        assert_eq!(
            TerminalSize::from_rect(Rect::new(0, 0, 140, 45)),
            TerminalSize::Medium
        );
        assert_eq!(
            TerminalSize::from_rect(Rect::new(0, 0, 180, 60)),
            TerminalSize::Large
        );
    }

    #[test]
    fn test_check_terminal_size() {
        assert!(check_terminal_size(Rect::new(0, 0, 80, 24)).is_ok());
        assert!(check_terminal_size(Rect::new(0, 0, 50, 15)).is_err());
    }
}
