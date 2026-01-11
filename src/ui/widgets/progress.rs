//! Progress indicators and loading spinners for beads-tui

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, Widget},
};
use std::time::{Duration, Instant};

/// Spinner animation frames
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Loading spinner widget for indeterminate progress
#[derive(Debug, Clone)]
pub struct Spinner {
    start_time: Instant,
    frame_duration: Duration,
    style: Style,
    label: Option<String>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Create a new spinner
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_duration: Duration::from_millis(80),
            style: Style::default().fg(Color::Cyan),
            label: None,
        }
    }

    /// Set the spinner style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set a label for the spinner
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Get the current frame index
    fn current_frame(&self) -> usize {
        let elapsed = self.start_time.elapsed();
        let frames_elapsed = elapsed.as_millis() / self.frame_duration.as_millis();
        (frames_elapsed % SPINNER_FRAMES.len() as u128) as usize
    }

    /// Get the current spinner character
    pub fn frame_char(&self) -> &'static str {
        SPINNER_FRAMES[self.current_frame()]
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 1 || area.height < 1 {
            return;
        }

        let frame = self.frame_char();
        let mut spans = vec![Span::styled(frame, self.style)];

        if let Some(label) = self.label {
            spans.push(Span::raw(" "));
            spans.push(Span::raw(label));
        }

        let line = Line::from(spans);
        line.render(area, buf);
    }
}

/// Progress bar widget for determinate progress
#[derive(Debug, Clone)]
pub struct ProgressBar {
    ratio: f64,
    label: Option<String>,
    style: Style,
    gauge_style: Style,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(ratio: f64) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            label: None,
            style: Style::default(),
            gauge_style: Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Set the progress ratio (0.0 to 1.0)
    pub fn ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set a label for the progress bar
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the style for the container
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for the filled portion
    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }

    /// Get the percentage (0-100)
    pub fn percentage(&self) -> u16 {
        (self.ratio * 100.0) as u16
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = if let Some(ref custom_label) = self.label {
            format!("{} ({}%)", custom_label, self.percentage())
        } else {
            format!("{}%", self.percentage())
        };

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(self.gauge_style)
            .ratio(self.ratio)
            .label(label);

        gauge.render(area, buf);
    }
}

/// Combined loading state widget
#[derive(Debug, Clone)]
pub struct LoadingIndicator {
    message: String,
    progress: Option<f64>,
    style: Style,
    start_time: Instant,
}

impl LoadingIndicator {
    /// Create a new loading indicator with indeterminate progress
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            progress: None,
            style: Style::default(),
            start_time: Instant::now(),
        }
    }

    /// Create a loading indicator with determinate progress
    pub fn with_progress<S: Into<String>>(message: S, progress: f64) -> Self {
        Self {
            message: message.into(),
            progress: Some(progress.clamp(0.0, 1.0)),
            style: Style::default(),
            start_time: Instant::now(),
        }
    }

    /// Set the loading indicator style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Update the progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = Some(progress.clamp(0.0, 1.0));
    }

    /// Update the message
    pub fn set_message<S: Into<String>>(&mut self, message: S) {
        self.message = message.into();
    }
}

impl Widget for LoadingIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        if let Some(progress) = self.progress {
            // Show progress bar with message
            let progress_bar = ProgressBar::new(progress)
                .label(self.message)
                .style(self.style);
            progress_bar.render(area, buf);
        } else {
            // Show spinner with message
            let spinner = Spinner {
                start_time: self.start_time,
                frame_duration: Duration::from_millis(80),
                style: self.style.fg(Color::Cyan),
                label: Some(self.message),
            };
            spinner.render(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new();
        assert!(spinner.label.is_none());
    }

    #[test]
    fn test_spinner_with_label() {
        let spinner = Spinner::new().label("Loading...");
        assert_eq!(spinner.label, Some("Loading...".to_string()));
    }

    #[test]
    fn test_spinner_frame() {
        let spinner = Spinner::new();
        let frame = spinner.frame_char();
        assert!(SPINNER_FRAMES.contains(&frame));
    }

    #[test]
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new(0.5);
        assert_eq!(bar.percentage(), 50);
    }

    #[test]
    fn test_progress_bar_clamping() {
        let bar1 = ProgressBar::new(-0.5);
        assert_eq!(bar1.percentage(), 0);

        let bar2 = ProgressBar::new(1.5);
        assert_eq!(bar2.percentage(), 100);
    }

    #[test]
    fn test_progress_bar_with_label() {
        let bar = ProgressBar::new(0.75).label("Downloading");
        assert_eq!(bar.label, Some("Downloading".to_string()));
        assert_eq!(bar.percentage(), 75);
    }

    #[test]
    fn test_loading_indicator_indeterminate() {
        let indicator = LoadingIndicator::new("Loading data...");
        assert!(indicator.progress.is_none());
        assert_eq!(indicator.message, "Loading data...");
    }

    #[test]
    fn test_loading_indicator_determinate() {
        let indicator = LoadingIndicator::with_progress("Downloading", 0.6);
        assert_eq!(indicator.progress, Some(0.6));
        assert_eq!(indicator.message, "Downloading");
    }

    #[test]
    fn test_loading_indicator_update() {
        let mut indicator = LoadingIndicator::new("Processing");
        indicator.set_progress(0.5);
        assert_eq!(indicator.progress, Some(0.5));

        indicator.set_message("Almost done");
        assert_eq!(indicator.message, "Almost done");
    }
}
