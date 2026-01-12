//! Markdown viewer widget for rendering markdown text

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Markdown element type
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkdownElement {
    Heading(usize, String), // level, text
    Paragraph(String),
    ListItem(String),
    CodeBlock(String),
    #[allow(dead_code)]
    InlineCode(String),
    HorizontalRule,
}

/// Markdown viewer state
#[derive(Debug, Clone)]
pub struct MarkdownViewerState {
    content: String,
    scroll_offset: usize,
    total_lines: usize,
}

impl MarkdownViewerState {
    /// Create a new markdown viewer state
    pub fn new(content: String) -> Self {
        Self {
            content,
            scroll_offset: 0,
            total_lines: 0,
        }
    }

    /// Set content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.scroll_offset = 0;
    }

    /// Get content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize) {
        if self.total_lines > 0 {
            self.scroll_offset =
                (self.scroll_offset + amount).min(self.total_lines.saturating_sub(1));
        }
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Reset scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        if self.total_lines > 0 {
            self.scroll_offset = self.total_lines.saturating_sub(1);
        }
    }
}

impl Default for MarkdownViewerState {
    fn default() -> Self {
        Self::new(String::new())
    }
}

/// Parse markdown content into elements
fn parse_markdown(content: &str) -> Vec<MarkdownElement> {
    let mut elements = Vec::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for line in content.lines() {
        // Check for code block delimiters
        if line.trim().starts_with("```") {
            if in_code_block {
                // End code block
                elements.push(MarkdownElement::CodeBlock(code_block_content.clone()));
                code_block_content.clear();
                in_code_block = false;
            } else {
                // Start code block
                in_code_block = true;
            }
            continue;
        }

        // If we're in a code block, accumulate lines
        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
            continue;
        }

        // Parse other markdown elements
        let trimmed = line.trim();

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            elements.push(MarkdownElement::HorizontalRule);
            continue;
        }

        // Headings
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 {
                let text = trimmed[level..].trim().to_string();
                if !text.is_empty() {
                    elements.push(MarkdownElement::Heading(level, text));
                    continue;
                }
            }
        }

        // List items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let text = trimmed[2..].trim().to_string();
            elements.push(MarkdownElement::ListItem(text));
            continue;
        }

        // Numbered lists
        if let Some(pos) = trimmed.find(". ") {
            let prefix = &trimmed[..pos];
            if prefix.chars().all(|c| c.is_ascii_digit()) {
                let text = trimmed[pos + 2..].trim().to_string();
                elements.push(MarkdownElement::ListItem(format!("{prefix}. {text}")));
                continue;
            }
        }

        // Regular paragraph (including empty lines)
        if !trimmed.is_empty() {
            elements.push(MarkdownElement::Paragraph(line.to_string()));
        }
    }

    // Handle unclosed code block
    if in_code_block && !code_block_content.is_empty() {
        elements.push(MarkdownElement::CodeBlock(code_block_content));
    }

    elements
}

/// Parse inline markdown formatting (bold, italic, code, links)
fn parse_inline_formatting(text: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Bold (**text** or __text__)
            '*' if chars.peek() == Some(&'*') => {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }
                chars.next(); // consume second *

                let mut bold_text = String::new();
                let mut found_end = false;

                while let Some(c2) = chars.next() {
                    if c2 == '*' && chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        found_end = true;
                        break;
                    }
                    bold_text.push(c2);
                }

                if found_end && !bold_text.is_empty() {
                    spans.push(Span::styled(
                        bold_text,
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                } else {
                    current.push_str("**");
                    current.push_str(&bold_text);
                }
            }

            // Italic (*text* or _text_)
            '*' | '_' => {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }

                let delimiter = c;
                let mut italic_text = String::new();
                let mut found_end = false;

                while let Some(c2) = chars.next() {
                    if c2 == delimiter && chars.peek() != Some(&delimiter) {
                        found_end = true;
                        break;
                    }
                    italic_text.push(c2);
                }

                if found_end && !italic_text.is_empty() {
                    spans.push(Span::styled(
                        italic_text,
                        Style::default().add_modifier(Modifier::ITALIC),
                    ));
                } else {
                    current.push(delimiter);
                    current.push_str(&italic_text);
                }
            }

            // Inline code (`text`)
            '`' => {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }

                let mut code_text = String::new();
                let mut found_end = false;

                for c2 in chars.by_ref() {
                    if c2 == '`' {
                        found_end = true;
                        break;
                    }
                    code_text.push(c2);
                }

                if found_end && !code_text.is_empty() {
                    spans.push(Span::styled(
                        code_text,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::ITALIC),
                    ));
                } else {
                    current.push('`');
                    current.push_str(&code_text);
                }
            }

            // Regular character
            _ => {
                current.push(c);
            }
        }
    }

    // Add remaining text
    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    // Return default span if empty
    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    spans
}

/// Markdown viewer widget
pub struct MarkdownViewer<'a> {
    block: Option<Block<'a>>,
    wrap: bool,
    style: Style,
}

impl<'a> MarkdownViewer<'a> {
    /// Create a new markdown viewer
    pub fn new() -> Self {
        Self {
            block: None,
            wrap: true,
            style: Style::default(),
        }
    }

    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Enable or disable text wrapping
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Render markdown elements to lines
    fn render_elements<'b>(&self, elements: &'b [MarkdownElement]) -> Vec<Line<'b>> {
        let mut lines = Vec::new();

        for element in elements {
            match element {
                MarkdownElement::Heading(level, text) => {
                    let (color, symbol) = match level {
                        1 => (Color::Cyan, "# ".to_string()),
                        2 => (Color::Blue, "## ".to_string()),
                        3 => (Color::Green, "### ".to_string()),
                        _ => (Color::Yellow, "#".repeat(*level) + " "),
                    };

                    let spans = vec![
                        Span::styled(
                            symbol,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            text.clone(),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                    ];
                    lines.push(Line::from(spans));
                    lines.push(Line::from("")); // Add spacing after headings
                }

                MarkdownElement::Paragraph(text) => {
                    let spans = parse_inline_formatting(text);
                    lines.push(Line::from(spans));
                }

                MarkdownElement::ListItem(text) => {
                    let mut spans = vec![Span::styled("  • ", Style::default().fg(Color::Yellow))];
                    spans.extend(parse_inline_formatting(text));
                    lines.push(Line::from(spans));
                }

                MarkdownElement::CodeBlock(code) => {
                    lines.push(Line::from(Span::styled(
                        "```",
                        Style::default().fg(Color::DarkGray),
                    )));

                    for code_line in code.lines() {
                        lines.push(Line::from(Span::styled(
                            code_line.to_string(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::ITALIC),
                        )));
                    }

                    lines.push(Line::from(Span::styled(
                        "```",
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from("")); // Add spacing after code blocks
                }

                MarkdownElement::InlineCode(code) => {
                    lines.push(Line::from(Span::styled(
                        code.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                MarkdownElement::HorizontalRule => {
                    lines.push(Line::from(Span::styled(
                        "─".repeat(50),
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from("")); // Add spacing
                }
            }
        }

        lines
    }
}

impl<'a> Default for MarkdownViewer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for MarkdownViewer<'a> {
    type State = MarkdownViewerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Copy values that will be used later
        let wrap = self.wrap;
        let style = self.style;

        // Parse markdown
        let elements = parse_markdown(&state.content);
        let lines = self.render_elements(&elements);

        // Extract block after using self
        let block = self.block;

        // Update total lines for scrolling
        state.total_lines = lines.len();

        // Apply scroll offset
        let visible_lines: Vec<Line> = if state.scroll_offset < lines.len() {
            lines.into_iter().skip(state.scroll_offset).collect()
        } else {
            Vec::new()
        };

        // Create paragraph
        let mut paragraph = Paragraph::new(visible_lines).style(style);

        if let Some(block) = block {
            paragraph = paragraph.block(block);
        }

        if wrap {
            paragraph = paragraph.wrap(Wrap { trim: false });
        }

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_viewer_state_creation() {
        let state = MarkdownViewerState::new("# Test".to_string());
        assert_eq!(state.content(), "# Test");
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_set_content() {
        let mut state = MarkdownViewerState::new("Original".to_string());
        state.set_content("New content".to_string());
        assert_eq!(state.content(), "New content");
        assert_eq!(state.scroll_offset(), 0); // Should reset scroll
    }

    #[test]
    fn test_scrolling() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 100;

        state.scroll_down(10);
        assert_eq!(state.scroll_offset(), 10);

        state.scroll_down(20);
        assert_eq!(state.scroll_offset(), 30);

        state.scroll_up(15);
        assert_eq!(state.scroll_offset(), 15);

        state.scroll_up(100); // Should not go below 0
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 100;

        state.scroll_down(50);
        assert_eq!(state.scroll_offset(), 50);

        state.scroll_to_top();
        assert_eq!(state.scroll_offset(), 0);

        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset(), 99);
    }

    #[test]
    fn test_parse_headings() {
        let content = "# Heading 1\n## Heading 2\n### Heading 3";
        let elements = parse_markdown(content);

        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0],
            MarkdownElement::Heading(1, "Heading 1".to_string())
        );
        assert_eq!(
            elements[1],
            MarkdownElement::Heading(2, "Heading 2".to_string())
        );
        assert_eq!(
            elements[2],
            MarkdownElement::Heading(3, "Heading 3".to_string())
        );
    }

    #[test]
    fn test_parse_lists() {
        let content = "- Item 1\n- Item 2\n* Item 3";
        let elements = parse_markdown(content);

        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::ListItem(_)));
        assert!(matches!(elements[1], MarkdownElement::ListItem(_)));
        assert!(matches!(elements[2], MarkdownElement::ListItem(_)));
    }

    #[test]
    fn test_parse_code_block() {
        let content = "```\ncode line 1\ncode line 2\n```";
        let elements = parse_markdown(content);

        assert_eq!(elements.len(), 1);
        if let MarkdownElement::CodeBlock(code) = &elements[0] {
            assert!(code.contains("code line 1"));
            assert!(code.contains("code line 2"));
        } else {
            panic!("Expected code block");
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let content = "---\n***\n___";
        let elements = parse_markdown(content);

        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0], MarkdownElement::HorizontalRule);
        assert_eq!(elements[1], MarkdownElement::HorizontalRule);
        assert_eq!(elements[2], MarkdownElement::HorizontalRule);
    }

    #[test]
    fn test_parse_inline_bold() {
        let spans = parse_inline_formatting("This is **bold** text");
        assert_eq!(spans.len(), 3);

        // Check that middle span has bold modifier
        if let Some(style) = spans.get(1).map(|s| s.style) {
            assert!(style.add_modifier.contains(Modifier::BOLD));
        }
    }

    #[test]
    fn test_parse_inline_italic() {
        let spans = parse_inline_formatting("This is *italic* text");
        assert_eq!(spans.len(), 3);

        // Check that middle span has italic modifier
        if let Some(style) = spans.get(1).map(|s| s.style) {
            assert!(style.add_modifier.contains(Modifier::ITALIC));
        }
    }

    #[test]
    fn test_parse_inline_code() {
        let spans = parse_inline_formatting("This is `code` text");
        assert_eq!(spans.len(), 3);

        // Check that middle span has code styling
        if let Some(style) = spans.get(1).map(|s| s.style) {
            assert!(style.add_modifier.contains(Modifier::ITALIC));
            assert_eq!(style.fg, Some(Color::Cyan));
        }
    }

    #[test]
    fn test_parse_multiple_inline_formats() {
        let spans = parse_inline_formatting("**bold** and *italic* and `code`");
        assert!(spans.len() >= 5); // Should have multiple spans
    }

    #[test]
    fn test_empty_content() {
        let state = MarkdownViewerState::new(String::new());
        assert_eq!(state.content(), "");

        let elements = parse_markdown("");
        assert_eq!(elements.len(), 0);
    }

    #[test]
    fn test_markdown_viewer_state_default() {
        let state = MarkdownViewerState::default();
        assert_eq!(state.content(), "");
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_markdown_viewer_state_clone() {
        let state = MarkdownViewerState::new("Test content".to_string());
        let cloned = state.clone();
        assert_eq!(cloned.content(), state.content());
        assert_eq!(cloned.scroll_offset(), state.scroll_offset());
    }

    #[test]
    fn test_scroll_down_with_zero_lines() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 0;
        state.scroll_down(10);
        assert_eq!(state.scroll_offset(), 0); // Should not scroll
    }

    #[test]
    fn test_scroll_down_at_boundary() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 10;
        state.scroll_down(100); // Try to scroll past end
        assert_eq!(state.scroll_offset(), 9); // Should clamp to total_lines - 1
    }

    #[test]
    fn test_scroll_to_bottom_with_zero_lines() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 0;
        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_set_content_resets_scroll() {
        let mut state = MarkdownViewerState::new("Original".to_string());
        state.total_lines = 100;
        state.scroll_down(50);
        assert_eq!(state.scroll_offset(), 50);

        state.set_content("New".to_string());
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_parse_numbered_list() {
        let content = "1. First item\n2. Second item\n10. Tenth item";
        let elements = parse_markdown(content);
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], MarkdownElement::ListItem(_)));
        assert!(matches!(elements[1], MarkdownElement::ListItem(_)));
        assert!(matches!(elements[2], MarkdownElement::ListItem(_)));
    }

    #[test]
    fn test_parse_all_heading_levels() {
        let content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
        let elements = parse_markdown(content);
        assert_eq!(elements.len(), 6);
        
        for (i, element) in elements.iter().enumerate() {
            if let MarkdownElement::Heading(level, _) = element {
                assert_eq!(*level, i + 1);
            } else {
                panic!("Expected heading");
            }
        }
    }

    #[test]
    fn test_parse_heading_without_space() {
        let content = "#NoSpace";
        let elements = parse_markdown(content);
        // Should still parse as heading
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0], MarkdownElement::Heading(1, _)));
    }

    #[test]
    fn test_parse_empty_heading() {
        let content = "##\n### \n";
        let elements = parse_markdown(content);
        // Empty headings (just #) are treated as paragraphs
        assert_eq!(elements.len(), 2);
        assert!(matches!(elements[0], MarkdownElement::Paragraph(_)));
        assert!(matches!(elements[1], MarkdownElement::Paragraph(_)));
    }

    #[test]
    fn test_parse_paragraph() {
        let content = "This is a regular paragraph.";
        let elements = parse_markdown(content);
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0], MarkdownElement::Paragraph(_)));
    }

    #[test]
    fn test_parse_unclosed_code_block() {
        let content = "```\ncode line 1\ncode line 2";
        let elements = parse_markdown(content);
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0], MarkdownElement::CodeBlock(_)));
    }

    #[test]
    fn test_parse_empty_code_block() {
        let content = "```\n```";
        let elements = parse_markdown(content);
        // Empty code blocks are still added (as empty string)
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0], MarkdownElement::CodeBlock(_)));
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = "# Title\n\nParagraph text\n\n- List item\n\n```\ncode\n```";
        let elements = parse_markdown(content);
        assert!(elements.len() >= 4);
    }

    #[test]
    fn test_parse_inline_unclosed_bold() {
        let spans = parse_inline_formatting("This is **unclosed");
        // Should treat as literal
        assert!(spans.iter().any(|s| s.content.contains("**")));
    }

    #[test]
    fn test_parse_inline_unclosed_italic() {
        let spans = parse_inline_formatting("This is *unclosed");
        // Should treat as literal
        assert!(spans.iter().any(|s| s.content.contains("*")));
    }

    #[test]
    fn test_parse_inline_unclosed_code() {
        let spans = parse_inline_formatting("This is `unclosed");
        // Should treat as literal
        assert!(spans.iter().any(|s| s.content.contains("`")));
    }

    #[test]
    fn test_parse_inline_empty_bold() {
        let spans = parse_inline_formatting("****");
        // Empty bold should be treated as literal
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_parse_inline_plain_text() {
        let spans = parse_inline_formatting("Plain text with no formatting");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "Plain text with no formatting");
    }

    #[test]
    fn test_markdown_element_eq() {
        let h1 = MarkdownElement::Heading(1, "Title".to_string());
        let h1_same = MarkdownElement::Heading(1, "Title".to_string());
        let h2 = MarkdownElement::Heading(2, "Title".to_string());
        
        assert_eq!(h1, h1_same);
        assert_ne!(h1, h2);
        assert_eq!(MarkdownElement::HorizontalRule, MarkdownElement::HorizontalRule);
    }

    #[test]
    fn test_markdown_element_clone() {
        let element = MarkdownElement::Heading(1, "Test".to_string());
        let cloned = element.clone();
        assert_eq!(element, cloned);
    }

    #[test]
    fn test_markdown_viewer_new() {
        let viewer = MarkdownViewer::new();
        assert!(viewer.block.is_none());
        assert!(viewer.wrap);
    }

    #[test]
    fn test_markdown_viewer_default() {
        let viewer = MarkdownViewer::default();
        assert!(viewer.block.is_none());
        assert!(viewer.wrap);
    }

    #[test]
    fn test_markdown_viewer_block() {
        let block = Block::default().title("Test");
        let viewer = MarkdownViewer::new().block(block);
        assert!(viewer.block.is_some());
    }

    #[test]
    fn test_markdown_viewer_wrap() {
        let viewer = MarkdownViewer::new().wrap(false);
        assert!(!viewer.wrap);

        let viewer = MarkdownViewer::new().wrap(true);
        assert!(viewer.wrap);
    }

    #[test]
    fn test_markdown_viewer_style() {
        let style = Style::default().fg(Color::Red);
        let viewer = MarkdownViewer::new().style(style);
        assert_eq!(viewer.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_markdown_viewer_builder_chain() {
        let block = Block::default().title("Doc");
        let style = Style::default().fg(Color::Blue);
        
        let viewer = MarkdownViewer::new()
            .block(block)
            .wrap(false)
            .style(style);

        assert!(viewer.block.is_some());
        assert!(!viewer.wrap);
        assert_eq!(viewer.style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_parse_list_variants() {
        let content = "- Dash item\n* Star item\n+ Plus item";
        let elements = parse_markdown(content);
        assert_eq!(elements.len(), 3);
        
        for element in elements {
            assert!(matches!(element, MarkdownElement::ListItem(_)));
        }
    }

    #[test]
    fn test_parse_horizontal_rule_variants() {
        let hr1 = parse_markdown("---");
        let hr2 = parse_markdown("***");
        let hr3 = parse_markdown("___");
        
        assert_eq!(hr1[0], MarkdownElement::HorizontalRule);
        assert_eq!(hr2[0], MarkdownElement::HorizontalRule);
        assert_eq!(hr3[0], MarkdownElement::HorizontalRule);
    }

    #[test]
    fn test_scroll_offset_after_multiple_operations() {
        let mut state = MarkdownViewerState::new("Test".to_string());
        state.total_lines = 50;

        state.scroll_down(10);
        state.scroll_down(5);
        state.scroll_up(3);
        assert_eq!(state.scroll_offset(), 12);

        state.scroll_to_top();
        assert_eq!(state.scroll_offset(), 0);

        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset(), 49);
    }
}
