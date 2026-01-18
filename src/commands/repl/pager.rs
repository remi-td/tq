//! Result Paging for Large Result Sets
//!
//! Provides both vertical and horizontal paging for query results
//! that exceed terminal dimensions. Based on the minus crate for
//! less-like paging functionality.
//!
//! Features:
//! - Vertical paging: Navigate through long result sets with j/k, Page Up/Down
//! - Horizontal paging: Scroll wide tables with h/l, arrow keys
//! - Search: Find text in results with /pattern
//! - Status line: Shows position, total rows, and hints

use crossterm::terminal;
use unicode_width::UnicodeWidthStr;

/// Configuration for the pager
#[derive(Debug, Clone)]
pub struct PagerConfig {
    /// Enable vertical paging for long results
    pub vertical_paging: bool,
    /// Enable horizontal scrolling for wide results
    pub horizontal_scrolling: bool,
    /// Minimum rows before paging activates (0 = always page)
    pub min_rows_for_paging: usize,
    /// Minimum columns before horizontal scrolling activates
    pub min_cols_for_scrolling: usize,
    /// Number of rows to show per page (0 = auto-detect from terminal)
    pub page_size: usize,
    /// Visible column width before scrolling (0 = auto-detect from terminal)
    pub visible_width: usize,
}

impl Default for PagerConfig {
    fn default() -> Self {
        Self {
            vertical_paging: true,
            horizontal_scrolling: true,
            min_rows_for_paging: 25,
            min_cols_for_scrolling: 120,
            page_size: 0,  // Auto-detect
            visible_width: 0,  // Auto-detect
        }
    }
}

impl PagerConfig {
    /// Create a config with paging disabled
    pub fn disabled() -> Self {
        Self {
            vertical_paging: false,
            horizontal_scrolling: false,
            ..Default::default()
        }
    }

    /// Get effective page size (auto-detect if 0)
    pub fn effective_page_size(&self) -> usize {
        if self.page_size > 0 {
            self.page_size
        } else {
            // Try to get terminal height, default to 24
            terminal::size()
                .map(|(_, h)| (h as usize).saturating_sub(3))  // Reserve for header and status
                .unwrap_or(24)
        }
    }

    /// Get effective visible width (auto-detect if 0)
    pub fn effective_visible_width(&self) -> usize {
        if self.visible_width > 0 {
            self.visible_width
        } else {
            // Try to get terminal width, default to 120
            terminal::size()
                .map(|(w, _)| w as usize)
                .unwrap_or(120)
        }
    }
}

/// Represents paged output with navigation state
pub struct PagedOutput {
    /// The formatted table lines
    lines: Vec<String>,
    /// Current vertical scroll position (line index)
    scroll_y: usize,
    /// Current horizontal scroll position (column index)
    scroll_x: usize,
    /// Total width of the widest line
    max_line_width: usize,
    /// Pager configuration
    config: PagerConfig,
    /// Whether content needs paging
    needs_vertical_paging: bool,
    /// Whether content needs horizontal scrolling
    needs_horizontal_scrolling: bool,
}

impl PagedOutput {
    /// Create a new paged output from formatted table lines
    pub fn new(content: String, config: PagerConfig) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let max_line_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);

        let page_size = config.effective_page_size();
        let visible_width = config.effective_visible_width();

        let needs_vertical_paging = config.vertical_paging
            && lines.len() > config.min_rows_for_paging
            && lines.len() > page_size;

        let needs_horizontal_scrolling = config.horizontal_scrolling
            && max_line_width > config.min_cols_for_scrolling
            && max_line_width > visible_width;

        Self {
            lines,
            scroll_y: 0,
            scroll_x: 0,
            max_line_width,
            config,
            needs_vertical_paging,
            needs_horizontal_scrolling,
        }
    }

    /// Check if this output needs any paging
    pub fn needs_paging(&self) -> bool {
        self.needs_vertical_paging || self.needs_horizontal_scrolling
    }

    /// Get the total number of lines
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get the current page of content with horizontal scrolling applied
    pub fn current_page(&self) -> Vec<&str> {
        let page_size = self.config.effective_page_size();
        let end = (self.scroll_y + page_size).min(self.lines.len());

        self.lines[self.scroll_y..end]
            .iter()
            .map(|s| s.as_str())
            .collect()
    }

    /// Get the visible portion of a line accounting for horizontal scroll
    pub fn visible_line(&self, line: &str) -> String {
        if self.scroll_x == 0 && !self.needs_horizontal_scrolling {
            return line.to_string();
        }

        let visible_width = self.config.effective_visible_width();

        // Handle horizontal scrolling by character position
        let chars: Vec<char> = line.chars().collect();

        if self.scroll_x >= chars.len() {
            return String::new();
        }

        let visible_chars: String = chars[self.scroll_x..]
            .iter()
            .take(visible_width)
            .collect();

        // Add scroll indicators
        let left_indicator = if self.scroll_x > 0 { "<" } else { " " };
        let right_indicator = if self.scroll_x + visible_width < chars.len() {
            ">"
        } else {
            " "
        };

        format!("{}{}{}", left_indicator, visible_chars, right_indicator)
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        let page_size = self.config.effective_page_size();
        if self.scroll_y + page_size < self.lines.len() {
            self.scroll_y += 1;
        }
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        if self.scroll_y > 0 {
            self.scroll_y -= 1;
        }
    }

    /// Scroll down by one page
    pub fn page_down(&mut self) {
        let page_size = self.config.effective_page_size();
        let max_scroll = self.lines.len().saturating_sub(page_size);
        self.scroll_y = (self.scroll_y + page_size).min(max_scroll);
    }

    /// Scroll up by one page
    pub fn page_up(&mut self) {
        let page_size = self.config.effective_page_size();
        self.scroll_y = self.scroll_y.saturating_sub(page_size);
    }

    /// Scroll left by one column
    pub fn scroll_left(&mut self) {
        if self.scroll_x > 0 {
            self.scroll_x -= 1;
        }
    }

    /// Scroll right by one column
    pub fn scroll_right(&mut self) {
        let visible_width = self.config.effective_visible_width();
        if self.scroll_x + visible_width < self.max_line_width {
            self.scroll_x += 1;
        }
    }

    /// Scroll to the beginning
    pub fn scroll_home(&mut self) {
        self.scroll_y = 0;
        self.scroll_x = 0;
    }

    /// Scroll to the end
    pub fn scroll_end(&mut self) {
        let page_size = self.config.effective_page_size();
        self.scroll_y = self.lines.len().saturating_sub(page_size);
    }

    /// Get the status line for the pager
    pub fn status_line(&self) -> String {
        let page_size = self.config.effective_page_size();
        let visible_end = (self.scroll_y + page_size).min(self.lines.len());
        let progress = if self.lines.is_empty() {
            100
        } else {
            (visible_end * 100) / self.lines.len()
        };

        let mut status = format!(
            "Lines {}-{} of {} ({}%)",
            self.scroll_y + 1,
            visible_end,
            self.lines.len(),
            progress
        );

        if self.needs_horizontal_scrolling {
            let visible_width = self.config.effective_visible_width();
            status.push_str(&format!(
                " | Cols {}-{} of {}",
                self.scroll_x + 1,
                (self.scroll_x + visible_width).min(self.max_line_width),
                self.max_line_width
            ));
        }

        status.push_str(" | q: quit");
        if self.needs_vertical_paging {
            status.push_str(" | j/k: scroll");
        }
        if self.needs_horizontal_scrolling {
            status.push_str(" | h/l: scroll horiz");
        }

        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_config_default() {
        let config = PagerConfig::default();
        assert!(config.vertical_paging);
        assert!(config.horizontal_scrolling);
        assert_eq!(config.min_rows_for_paging, 25);
    }

    #[test]
    fn test_pager_config_disabled() {
        let config = PagerConfig::disabled();
        assert!(!config.vertical_paging);
        assert!(!config.horizontal_scrolling);
    }

    #[test]
    fn test_paged_output_no_paging_needed() {
        let content = "Line 1\nLine 2\nLine 3".to_string();
        let config = PagerConfig::default();
        let paged = PagedOutput::new(content, config);

        // With default min_rows_for_paging=25, 3 lines shouldn't need paging
        assert!(!paged.needs_paging());
    }

    #[test]
    fn test_paged_output_scroll() {
        let mut lines = Vec::new();
        for i in 0..100 {
            lines.push(format!("Line {}", i));
        }
        let content = lines.join("\n");

        let mut config = PagerConfig::default();
        config.page_size = 10;
        config.min_rows_for_paging = 5;

        let mut paged = PagedOutput::new(content, config);
        assert!(paged.needs_paging());
        assert_eq!(paged.scroll_y, 0);

        paged.scroll_down();
        assert_eq!(paged.scroll_y, 1);

        paged.scroll_up();
        assert_eq!(paged.scroll_y, 0);

        paged.page_down();
        assert_eq!(paged.scroll_y, 10);

        paged.page_up();
        assert_eq!(paged.scroll_y, 0);
    }

    #[test]
    fn test_paged_output_horizontal_scroll() {
        let long_line = "A".repeat(200);
        let content = format!("{}\n{}\n{}", long_line, long_line, long_line);

        let mut config = PagerConfig::default();
        config.visible_width = 80;
        config.min_cols_for_scrolling = 50;

        let mut paged = PagedOutput::new(content, config);
        assert!(paged.needs_horizontal_scrolling);
        assert_eq!(paged.scroll_x, 0);

        paged.scroll_right();
        assert_eq!(paged.scroll_x, 1);

        paged.scroll_left();
        assert_eq!(paged.scroll_x, 0);
    }

    #[test]
    fn test_status_line() {
        let mut lines = Vec::new();
        for i in 0..50 {
            lines.push(format!("Line {}", i));
        }
        let content = lines.join("\n");

        let mut config = PagerConfig::default();
        config.page_size = 10;
        config.min_rows_for_paging = 5;

        let paged = PagedOutput::new(content, config);
        let status = paged.status_line();

        assert!(status.contains("Lines 1-10 of 50"));
        assert!(status.contains("q: quit"));
    }

    #[test]
    fn test_visible_line_with_scroll() {
        let line = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let mut config = PagerConfig::default();
        config.visible_width = 10;
        config.min_cols_for_scrolling = 5;
        config.horizontal_scrolling = true;

        let mut paged = PagedOutput::new(line.to_string(), config);
        paged.scroll_x = 5;

        let visible = paged.visible_line(line);
        // Should show scroll indicators and visible portion
        assert!(visible.starts_with('<'));  // Left indicator shows more content
    }
}
