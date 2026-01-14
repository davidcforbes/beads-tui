//! Text-to-speech support for screen reader accessibility

use std::sync::{Arc, Mutex};
use tts::Tts;

/// TTS manager for screen reader support
#[derive(Clone)]
pub struct TtsManager {
    inner: Option<Arc<Mutex<Tts>>>,
    enabled: bool,
}

impl std::fmt::Debug for TtsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TtsManager")
            .field("enabled", &self.enabled)
            .field("available", &self.inner.is_some())
            .finish()
    }
}

impl TtsManager {
    /// Create a new TTS manager
    pub fn new(enabled: bool) -> Self {
        if !enabled {
            return Self {
                inner: None,
                enabled: false,
            };
        }

        let tts = match Tts::default() {
            Ok(tts) => {
                tracing::info!("TTS initialized successfully");
                Some(Arc::new(Mutex::new(tts)))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize TTS: {}. Screen reader support will be disabled.", e);
                None
            }
        };

        Self {
            inner: tts,
            enabled,
        }
    }

    /// Announce a message to screen readers
    pub fn announce(&self, message: &str) {
        if !self.enabled {
            return;
        }

        let Some(ref tts) = self.inner else {
            return;
        };

        // Try to speak, but don't fail if it doesn't work
        if let Ok(mut tts_lock) = tts.try_lock() {
            // Stop any currently speaking text first
            let _ = tts_lock.stop();

            if let Err(e) = tts_lock.speak(message, false) {
                tracing::warn!("Failed to announce '{}': {}", message, e);
            }
        }
    }

    /// Check if TTS is enabled and available
    pub fn is_available(&self) -> bool {
        self.enabled && self.inner.is_some()
    }
}

impl Default for TtsManager {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_manager_disabled() {
        let tts = TtsManager::new(false);
        assert!(!tts.is_available());
    }

    #[test]
    fn test_tts_manager_announce_when_disabled() {
        let tts = TtsManager::new(false);
        // Should not panic
        tts.announce("Test message");
    }

    #[test]
    fn test_tts_manager_default() {
        let tts = TtsManager::default();
        assert!(!tts.is_available());
    }
}
