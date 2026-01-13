//! Skeleton loading widgets for displaying placeholder content while data loads
//!
//! Skeleton screens improve perceived performance by showing the structure of content
//! before the actual data is available, reducing layout shift and providing visual feedback.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

/// Animation phase for skeleton shimmer effect
#[derive(Debug, Clone, Copy)]
pub struct SkeletonAnimation {
    /// Current frame (0-7)
    frame: u8,
}

impl SkeletonAnimation {
    /// Create a new animation state
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    /// Create from timestamp for synchronized animation
    pub fn from_instant(instant: std::time::Instant) -> Self {
        let elapsed = instant.elapsed().as_millis();
        let frame = ((elapsed / 150) % 8) as u8;
        Self { frame }
    }

    /// Get the current shimmer style based on animation frame
    fn get_shimmer_style(&self) -> Style {
        // Create subtle shimmer effect with varying brightness
        let brightness = match self.frame {
            0 | 7 => Color::Rgb(60, 60, 60),
            1 | 6 => Color::Rgb(70, 70, 70),
            2 | 5 => Color::Rgb(80, 80, 80),
            3 | 4 => Color::Rgb(90, 90, 90),
            _ => Color::Rgb(70, 70, 70),
        };
        Style::default().fg(brightness)
    }
}

impl Default for SkeletonAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Skeleton text widget for placeholder text lines
pub struct SkeletonText {
    /// Width percentage of the line (0-100)
    width_percent: u16,
    /// Whether to show shimmer animation
    animate: bool,
    /// Animation state
    animation: SkeletonAnimation,
    /// Custom style (overrides animation if set)
    style: Option<Style>,
}

impl SkeletonText {
    /// Create a new skeleton text widget
    pub fn new() -> Self {
        Self {
            width_percent: 80,
            animate: true,
            animation: SkeletonAnimation::new(),
            style: None,
        }
    }

    /// Set the width percentage (0-100)
    pub fn width_percent(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Enable or disable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Set animation from timestamp
    pub fn animation(mut self, instant: std::time::Instant) -> Self {
        self.animation = SkeletonAnimation::from_instant(instant);
        self
    }

    /// Set custom style (disables animation)
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self.animate = false;
        self
    }
}

impl Default for SkeletonText {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SkeletonText {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = (area.width as u32 * self.width_percent as u32 / 100) as u16;
        let text = "▓".repeat(width as usize);

        let style = if let Some(s) = self.style {
            s
        } else if self.animate {
            self.animation.get_shimmer_style()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let paragraph = Paragraph::new(text).style(style);
        paragraph.render(area, buf);
    }
}

/// Skeleton list widget for placeholder list items
pub struct SkeletonList {
    /// Number of skeleton items to show
    count: usize,
    /// Width percentage for each line (0-100)
    width_percent: u16,
    /// Whether to vary line widths
    vary_widths: bool,
    /// Whether to show shimmer animation
    animate: bool,
    /// Animation state
    animation: SkeletonAnimation,
    /// Optional block for borders
    block: Option<Block<'static>>,
}

impl SkeletonList {
    /// Create a new skeleton list widget
    pub fn new(count: usize) -> Self {
        Self {
            count,
            width_percent: 90,
            vary_widths: true,
            animate: true,
            animation: SkeletonAnimation::new(),
            block: None,
        }
    }

    /// Set the width percentage for lines
    pub fn width_percent(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Set whether to vary line widths
    pub fn vary_widths(mut self, vary: bool) -> Self {
        self.vary_widths = vary;
        self
    }

    /// Enable or disable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Set animation from timestamp
    pub fn animation(mut self, instant: std::time::Instant) -> Self {
        self.animation = SkeletonAnimation::from_instant(instant);
        self
    }

    /// Set a block for borders
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.animate {
            self.animation.get_shimmer_style()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items: Vec<ListItem> = (0..self.count)
            .map(|i| {
                let width = if self.vary_widths {
                    // Vary widths between 60-100% for more natural look
                    let variation = (i * 13) % 40; // Pseudo-random variation
                    (self.width_percent as i32 - 20 + variation as i32).max(60).min(100) as u16
                } else {
                    self.width_percent
                };

                let length = (area.width as u32 * width as u32 / 100) as usize;
                let text = "▓".repeat(length);
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(items).block(self.block.unwrap_or_default());
        list.render(area, buf);
    }
}

/// Skeleton table widget for placeholder table rows
pub struct SkeletonTable {
    /// Number of rows to show
    rows: usize,
    /// Number of columns
    columns: usize,
    /// Column widths as percentages (sum should be <= 100)
    column_widths: Vec<u16>,
    /// Whether to show header row
    show_header: bool,
    /// Whether to show shimmer animation
    animate: bool,
    /// Animation state
    animation: SkeletonAnimation,
    /// Optional block for borders
    block: Option<Block<'static>>,
}

impl SkeletonTable {
    /// Create a new skeleton table widget
    pub fn new(rows: usize, columns: usize) -> Self {
        // Default equal column widths
        let width_per_column = 90 / columns as u16;
        let column_widths = vec![width_per_column; columns];

        Self {
            rows,
            columns,
            column_widths,
            show_header: true,
            animate: true,
            animation: SkeletonAnimation::new(),
            block: None,
        }
    }

    /// Set column widths as percentages
    pub fn column_widths(mut self, widths: Vec<u16>) -> Self {
        if widths.len() == self.columns {
            self.column_widths = widths;
        }
        self
    }

    /// Set whether to show header row
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Enable or disable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Set animation from timestamp
    pub fn animation(mut self, instant: std::time::Instant) -> Self {
        self.animation = SkeletonAnimation::from_instant(instant);
        self
    }

    /// Set a block for borders
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.animate {
            self.animation.get_shimmer_style()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let header_style = if self.animate {
            self.animation.get_shimmer_style().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        };

        let mut items = Vec::new();

        // Add header row if enabled
        if self.show_header {
            let header_spans: Vec<Span> = self
                .column_widths
                .iter()
                .enumerate()
                .map(|(i, &width)| {
                    let col_width = (area.width as u32 * width as u32 / 100) as usize;
                    let text = "▓".repeat(col_width.saturating_sub(1));
                    let separator = if i < self.columns - 1 { " " } else { "" };
                    vec![Span::styled(text, header_style), Span::raw(separator)]
                })
                .flatten()
                .collect();
            items.push(ListItem::new(Line::from(header_spans)));
        }

        // Add data rows
        for _ in 0..self.rows {
            let row_spans: Vec<Span> = self
                .column_widths
                .iter()
                .enumerate()
                .map(|(i, &width)| {
                    let col_width = (area.width as u32 * width as u32 / 100) as usize;
                    let text = "▓".repeat(col_width.saturating_sub(1));
                    let separator = if i < self.columns - 1 { " " } else { "" };
                    vec![Span::styled(text, style), Span::raw(separator)]
                })
                .flatten()
                .collect();
            items.push(ListItem::new(Line::from(row_spans)));
        }

        let list = List::new(items).block(self.block.unwrap_or_default());
        list.render(area, buf);
    }
}

/// Skeleton tree widget for placeholder tree structure
pub struct SkeletonTree {
    /// Number of nodes to show
    nodes: usize,
    /// Maximum depth level
    max_depth: usize,
    /// Whether to show expand/collapse indicators
    show_indicators: bool,
    /// Whether to show shimmer animation
    animate: bool,
    /// Animation state
    animation: SkeletonAnimation,
    /// Optional block for borders
    block: Option<Block<'static>>,
}

impl SkeletonTree {
    /// Create a new skeleton tree widget
    pub fn new(nodes: usize) -> Self {
        Self {
            nodes,
            max_depth: 3,
            show_indicators: true,
            animate: true,
            animation: SkeletonAnimation::new(),
            block: None,
        }
    }

    /// Set maximum depth level
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set whether to show expand/collapse indicators
    pub fn show_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    /// Enable or disable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Set animation from timestamp
    pub fn animation(mut self, instant: std::time::Instant) -> Self {
        self.animation = SkeletonAnimation::from_instant(instant);
        self
    }

    /// Set a block for borders
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonTree {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.animate {
            self.animation.get_shimmer_style()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let indicator_style = style;

        let items: Vec<ListItem> = (0..self.nodes)
            .map(|i| {
                // Calculate depth (pseudo-random pattern)
                let depth = (i * 7) % (self.max_depth + 1);

                let mut spans = Vec::new();

                // Add indentation
                if depth > 0 {
                    spans.push(Span::raw("  ".repeat(depth)));
                }

                // Add indicator
                if self.show_indicators && depth < self.max_depth {
                    let indicator = if i % 3 == 0 { "▼ " } else { "▶ " };
                    spans.push(Span::styled(indicator, indicator_style));
                } else if self.show_indicators {
                    spans.push(Span::raw("  "));
                }

                // Add skeleton text with varying width
                let base_width = 60;
                let variation = (i * 17) % 30;
                let width = (base_width + variation).min(90);
                let available_width =
                    area.width.saturating_sub((depth * 2 + if self.show_indicators { 2 } else { 0 }) as u16);
                let text_length = (available_width as u32 * width as u32 / 100) as usize;
                let text = "▓".repeat(text_length);
                spans.push(Span::styled(text, style));

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items).block(self.block.unwrap_or_default());
        list.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_animation_creation() {
        let anim = SkeletonAnimation::new();
        assert_eq!(anim.frame, 0);
    }

    #[test]
    fn test_skeleton_animation_from_instant() {
        let instant = std::time::Instant::now();
        let anim = SkeletonAnimation::from_instant(instant);
        assert!(anim.frame < 8);
    }

    #[test]
    fn test_skeleton_text_builder() {
        let skeleton = SkeletonText::new()
            .width_percent(70)
            .animate(false);
        assert_eq!(skeleton.width_percent, 70);
        assert!(!skeleton.animate);
    }

    #[test]
    fn test_skeleton_text_width_clamping() {
        let skeleton = SkeletonText::new().width_percent(150);
        assert_eq!(skeleton.width_percent, 100);
    }

    #[test]
    fn test_skeleton_list_builder() {
        let skeleton = SkeletonList::new(5)
            .width_percent(80)
            .vary_widths(false)
            .animate(false);
        assert_eq!(skeleton.count, 5);
        assert_eq!(skeleton.width_percent, 80);
        assert!(!skeleton.vary_widths);
        assert!(!skeleton.animate);
    }

    #[test]
    fn test_skeleton_table_builder() {
        let skeleton = SkeletonTable::new(10, 3)
            .show_header(false)
            .animate(false);
        assert_eq!(skeleton.rows, 10);
        assert_eq!(skeleton.columns, 3);
        assert!(!skeleton.show_header);
        assert!(!skeleton.animate);
    }

    #[test]
    fn test_skeleton_table_column_widths() {
        let widths = vec![30, 50, 20];
        let skeleton = SkeletonTable::new(5, 3)
            .column_widths(widths.clone());
        assert_eq!(skeleton.column_widths, widths);
    }

    #[test]
    fn test_skeleton_table_column_widths_wrong_count() {
        let widths = vec![50, 50]; // Wrong count (2 instead of 3)
        let skeleton = SkeletonTable::new(5, 3)
            .column_widths(widths);
        // Should keep default widths
        assert_eq!(skeleton.column_widths.len(), 3);
    }

    #[test]
    fn test_skeleton_tree_builder() {
        let skeleton = SkeletonTree::new(8)
            .max_depth(2)
            .show_indicators(false)
            .animate(false);
        assert_eq!(skeleton.nodes, 8);
        assert_eq!(skeleton.max_depth, 2);
        assert!(!skeleton.show_indicators);
        assert!(!skeleton.animate);
    }

    #[test]
    fn test_skeleton_text_render_empty_area() {
        let skeleton = SkeletonText::new();
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        skeleton.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_skeleton_text_custom_style() {
        let custom_style = Style::default().fg(Color::Red);
        let skeleton = SkeletonText::new().style(custom_style);
        assert!(skeleton.style.is_some());
        assert!(!skeleton.animate); // Should disable animation
    }

    #[test]
    fn test_skeleton_animation_shimmer_style() {
        let anim = SkeletonAnimation { frame: 0 };
        let style = anim.get_shimmer_style();
        assert!(matches!(style.fg, Some(Color::Rgb(60, 60, 60))));

        let anim = SkeletonAnimation { frame: 3 };
        let style = anim.get_shimmer_style();
        assert!(matches!(style.fg, Some(Color::Rgb(90, 90, 90))));
    }
}
