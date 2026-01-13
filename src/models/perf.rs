/// Performance profiling and metrics tracking for the render loop
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Performance statistics for render loop profiling
#[derive(Debug, Clone)]
pub struct PerfStats {
    /// Recent render times (last N frames)
    render_times: VecDeque<Duration>,
    /// Maximum number of samples to keep
    max_samples: usize,
    /// Total frames rendered
    total_frames: u64,
    /// Start time of profiling session
    session_start: Instant,
    /// Whether profiling is enabled
    enabled: bool,
}

impl PerfStats {
    /// Create new performance stats tracker
    pub fn new() -> Self {
        Self {
            render_times: VecDeque::with_capacity(100),
            max_samples: 100,
            total_frames: 0,
            session_start: Instant::now(),
            enabled: false,
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled && self.render_times.is_empty() {
            // Reset session start when enabling
            self.session_start = Instant::now();
        }
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a render time
    pub fn record_render(&mut self, duration: Duration) {
        if !self.enabled {
            return;
        }

        self.total_frames += 1;
        self.render_times.push_back(duration);

        // Keep only the most recent samples
        if self.render_times.len() > self.max_samples {
            self.render_times.pop_front();
        }
    }

    /// Get average render time over recent samples
    pub fn avg_render_time(&self) -> Option<Duration> {
        if self.render_times.is_empty() {
            return None;
        }

        let sum: Duration = self.render_times.iter().sum();
        Some(sum / self.render_times.len() as u32)
    }

    /// Get minimum render time
    pub fn min_render_time(&self) -> Option<Duration> {
        self.render_times.iter().min().copied()
    }

    /// Get maximum render time
    pub fn max_render_time(&self) -> Option<Duration> {
        self.render_times.iter().max().copied()
    }

    /// Get the 95th percentile render time
    pub fn p95_render_time(&self) -> Option<Duration> {
        if self.render_times.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = self.render_times.iter().copied().collect();
        sorted.sort();

        let idx = (sorted.len() as f64 * 0.95) as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Get the 99th percentile render time
    pub fn p99_render_time(&self) -> Option<Duration> {
        if self.render_times.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = self.render_times.iter().copied().collect();
        sorted.sort();

        let idx = (sorted.len() as f64 * 0.99) as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Get total frames rendered
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Get average FPS over the session
    pub fn avg_fps(&self) -> f64 {
        let elapsed = self.session_start.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.total_frames as f64 / elapsed
    }

    /// Get current FPS based on recent render times
    pub fn current_fps(&self) -> f64 {
        match self.avg_render_time() {
            Some(avg) => {
                let secs = avg.as_secs_f64();
                if secs == 0.0 {
                    0.0
                } else {
                    1.0 / secs
                }
            }
            None => 0.0,
        }
    }

    /// Format statistics as a multi-line string
    pub fn format_stats(&self) -> String {
        if !self.enabled || self.render_times.is_empty() {
            return "Performance profiling disabled or no data".to_string();
        }

        let avg = self
            .avg_render_time()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        let min = self
            .min_render_time()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        let max = self
            .max_render_time()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        let p95 = self
            .p95_render_time()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        let p99 = self
            .p99_render_time()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        format!(
            "Perf Stats (last {} frames):\n\
             Total frames: {}\n\
             Avg FPS: {:.1}\n\
             Current FPS: {:.1}\n\
             Render time - Avg: {} | Min: {} | Max: {} | P95: {} | P99: {}",
            self.render_times.len(),
            self.total_frames,
            self.avg_fps(),
            self.current_fps(),
            avg,
            min,
            max,
            p95,
            p99
        )
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.render_times.clear();
        self.total_frames = 0;
        self.session_start = Instant::now();
    }
}

impl Default for PerfStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_stats_creation() {
        let stats = PerfStats::new();
        assert!(!stats.is_enabled());
        assert_eq!(stats.total_frames(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut stats = PerfStats::new();
        stats.set_enabled(true);
        assert!(stats.is_enabled());
        stats.set_enabled(false);
        assert!(!stats.is_enabled());
    }

    #[test]
    fn test_record_when_disabled() {
        let mut stats = PerfStats::new();
        stats.record_render(Duration::from_millis(16));
        assert_eq!(stats.total_frames(), 0);
        assert!(stats.avg_render_time().is_none());
    }

    #[test]
    fn test_record_when_enabled() {
        let mut stats = PerfStats::new();
        stats.set_enabled(true);

        stats.record_render(Duration::from_millis(10));
        stats.record_render(Duration::from_millis(20));
        stats.record_render(Duration::from_millis(15));

        assert_eq!(stats.total_frames(), 3);
        assert!(stats.avg_render_time().is_some());
        assert_eq!(stats.avg_render_time().unwrap(), Duration::from_millis(15));
    }

    #[test]
    fn test_min_max() {
        let mut stats = PerfStats::new();
        stats.set_enabled(true);

        stats.record_render(Duration::from_millis(10));
        stats.record_render(Duration::from_millis(25));
        stats.record_render(Duration::from_millis(15));

        assert_eq!(stats.min_render_time(), Some(Duration::from_millis(10)));
        assert_eq!(stats.max_render_time(), Some(Duration::from_millis(25)));
    }

    #[test]
    fn test_percentiles() {
        let mut stats = PerfStats::new();
        stats.set_enabled(true);

        for i in 1..=100 {
            stats.record_render(Duration::from_millis(i));
        }

        assert!(stats.p95_render_time().is_some());
        assert!(stats.p99_render_time().is_some());
    }

    #[test]
    fn test_reset() {
        let mut stats = PerfStats::new();
        stats.set_enabled(true);

        stats.record_render(Duration::from_millis(10));
        stats.record_render(Duration::from_millis(20));

        assert_eq!(stats.total_frames(), 2);

        stats.reset();

        assert_eq!(stats.total_frames(), 0);
        assert!(stats.avg_render_time().is_none());
    }

    #[test]
    fn test_max_samples() {
        let mut stats = PerfStats::new();
        stats.max_samples = 10;
        stats.set_enabled(true);

        // Record more than max_samples
        for i in 1..=20 {
            stats.record_render(Duration::from_millis(i));
        }

        // Should only keep the last 10
        assert_eq!(stats.render_times.len(), 10);
        assert_eq!(stats.total_frames(), 20);
    }
}
