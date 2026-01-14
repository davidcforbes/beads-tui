//! Progress tracking for background tasks

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Progress tracker for long-running tasks
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    current: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
    message: Arc<Mutex<String>>,
    stage: Arc<Mutex<String>>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            current: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
            message: Arc::new(Mutex::new(String::new())),
            stage: Arc::new(Mutex::new(String::new())),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the progress (current/total)
    pub fn set_progress(&self, current: u64, total: u64) {
        self.current.store(current, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);

        // Initialize start time on first progress update
        let mut start = self
            .start_time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if start.is_none() {
            *start = Some(Instant::now());
        }
    }

    /// Set the progress message
    pub fn set_message(&self, message: String) {
        *self
            .message
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = message;
    }

    /// Set the current stage
    pub fn set_stage(&self, stage: String) {
        *self
            .stage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = stage;
    }

    /// Get current progress
    pub fn get_progress(&self) -> (u64, u64) {
        (
            self.current.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }

    /// Get progress percentage (0.0 - 100.0)
    pub fn percentage(&self) -> f64 {
        let (current, total) = self.get_progress();
        if total == 0 {
            0.0
        } else {
            (current as f64 / total as f64) * 100.0
        }
    }

    /// Get the current message
    pub fn get_message(&self) -> String {
        self.message
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Get the current stage
    pub fn get_stage(&self) -> String {
        self.stage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Estimate time remaining (ETA)
    pub fn eta(&self) -> Option<Duration> {
        let (current, total) = self.get_progress();
        if current == 0 || total == 0 || current >= total {
            return None;
        }

        let start = self
            .start_time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let start_time = start.as_ref()?;

        let elapsed = start_time.elapsed();
        let rate = current as f64 / elapsed.as_secs_f64();
        if rate == 0.0 {
            return None;
        }

        let remaining = (total - current) as f64 / rate;
        Some(Duration::from_secs_f64(remaining))
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_progress_percentage() {
        let tracker = ProgressTracker::new();
        tracker.set_progress(0, 100);
        assert_eq!(tracker.percentage(), 0.0);

        tracker.set_progress(50, 100);
        assert_eq!(tracker.percentage(), 50.0);

        tracker.set_progress(100, 100);
        assert_eq!(tracker.percentage(), 100.0);
    }

    #[test]
    fn test_progress_message() {
        let tracker = ProgressTracker::new();
        tracker.set_message("Processing...".to_string());
        assert_eq!(tracker.get_message(), "Processing...");
    }

    #[test]
    fn test_progress_stage() {
        let tracker = ProgressTracker::new();
        tracker.set_stage("Loading data".to_string());
        assert_eq!(tracker.get_stage(), "Loading data");
    }

    #[test]
    fn test_eta_calculation() {
        let tracker = ProgressTracker::new();
        tracker.set_progress(0, 100);

        // Wait a bit and update progress
        sleep(Duration::from_millis(10));
        tracker.set_progress(10, 100);

        // Should have an ETA now
        let eta = tracker.eta();
        assert!(eta.is_some());
    }

    #[test]
    fn test_eta_when_complete() {
        let tracker = ProgressTracker::new();
        tracker.set_progress(100, 100);

        // ETA should be None when complete
        assert!(tracker.eta().is_none());
    }
}
