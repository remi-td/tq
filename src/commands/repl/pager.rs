//! Custom Result Pager for Large Result Sets
//!
//! Sprint 8: Complete rewrite to fix critical bugs:
//! - Bug 3.1: 'q' now returns to REPL instead of exiting program
//! - Bug 3.2: Column windowing for wide tables (shows 4-6 readable columns)
//! - Bug 3.3: Cell truncation at 100 chars for long values
//!
//! This pager uses crossterm for terminal control and implements
//! a custom event loop that properly returns control to the REPL.
//!
//! Features:
//! - Column windowing: Show readable columns, navigate with Left/Right
//! - Row paging: Navigate with j/k, Space/b, g/G
//! - Cell truncation: Long values truncated with ellipsis
//! - Safe exit: 'q' returns to REPL, never exits program

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as terminal_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Maximum characters to display in a cell before truncation
const MAX_CELL_LENGTH: usize = 100;

/// Minimum column width (including padding)
const MIN_COLUMN_WIDTH: usize = 8;

/// Maximum column width (including padding)
const MAX_COLUMN_WIDTH: usize = 40;

/// Width of the column position indicator cells (e.g., "(+3 cols)")
const INDICATOR_WIDTH: usize = 10;

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
            min_cols_for_scrolling: 0,  // Use terminal width (fixed in Sprint 29.1)
            page_size: 0,     // Auto-detect
            visible_width: 0, // Auto-detect
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
            terminal_size()
                .map(|(_, h)| (h as usize).saturating_sub(5)) // Reserve for header and status
                .unwrap_or(20)
        }
    }

    /// Get effective visible width (auto-detect if 0)
    pub fn effective_visible_width(&self) -> usize {
        if self.visible_width > 0 {
            self.visible_width
        } else {
            // Try to get terminal width, default to 120
            terminal_size().map(|(w, _)| w as usize).unwrap_or(120)
        }
    }
}

/// A single column with its data
#[derive(Debug, Clone)]
struct ColumnData {
    /// Column header name
    header: String,
    /// Cell values for each row
    values: Vec<String>,
    /// Calculated display width for this column
    display_width: usize,
}

/// Represents the parsed and processed table data for paging
#[derive(Debug)]
pub struct TableData {
    /// Columns with their data
    columns: Vec<ColumnData>,
    /// Total number of rows
    row_count: usize,
}

impl TableData {
    /// Parse table content from formatted string
    ///
    /// This parses the comfy-table output format and extracts columns and values.
    /// It applies cell truncation during parsing.
    pub fn parse_from_content(content: &str) -> Option<Self> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 3 {
            return None;
        }

        // Find header row (first row with actual content between separators)
        let header_idx = lines.iter().position(|l| {
            l.contains('│')
                && !l.chars().all(|c| {
                    c == '─'
                        || c == '│'
                        || c == '┌'
                        || c == '┐'
                        || c == '├'
                        || c == '┤'
                        || c == '└'
                        || c == '┘'
                        || c == '╭'
                        || c == '╮'
                        || c == '╰'
                        || c == '╯'
                        || c == '┼'
                        || c.is_whitespace()
                })
        })?;

        // Parse header
        let header_line = lines[header_idx];
        let headers: Vec<String> = parse_row_cells(header_line);
        if headers.is_empty() {
            return None;
        }

        // Find data rows
        let mut rows: Vec<Vec<String>> = Vec::new();
        for line in lines.iter().skip(header_idx + 1) {
            if line.contains('│') && !is_separator_line(line) {
                let cells = parse_row_cells(line);
                if cells.len() == headers.len() {
                    // Apply truncation to each cell
                    let truncated_cells: Vec<String> = cells
                        .into_iter()
                        .map(|c| truncate_cell(&c, MAX_CELL_LENGTH))
                        .collect();
                    rows.push(truncated_cells);
                }
            }
        }

        // Build column data
        let mut columns: Vec<ColumnData> = headers
            .into_iter()
            .map(|h| {
                let truncated_header = truncate_cell(&h, MAX_COLUMN_WIDTH - 2);
                ColumnData {
                    display_width: truncated_header.width().max(MIN_COLUMN_WIDTH),
                    header: truncated_header,
                    values: Vec::new(),
                }
            })
            .collect();

        // Populate column values and calculate widths
        for row in &rows {
            for (i, value) in row.iter().enumerate() {
                if i < columns.len() {
                    let value_width = value.width();
                    columns[i].display_width = columns[i]
                        .display_width
                        .max(value_width)
                        .min(MAX_COLUMN_WIDTH);
                    columns[i].values.push(value.clone());
                }
            }
        }

        Some(TableData {
            columns,
            row_count: rows.len(),
        })
    }
}

/// Parse cells from a table row line
///
/// Sprint 8 Bug Fix: Simplified and more robust parsing logic
fn parse_row_cells(line: &str) -> Vec<String> {
    let parts: Vec<&str> = line.split('│').collect();

    // Skip first (before first │) and last (after last │) which are empty or borders
    if parts.len() <= 2 {
        return vec![];
    }

    parts[1..parts.len() - 1]
        .iter()
        .map(|s| s.trim().to_string())
        .collect()
}

/// Check if a line is a separator line (borders only)
fn is_separator_line(line: &str) -> bool {
    line.chars().all(|c| {
        c == '─'
            || c == '│'
            || c == '┌'
            || c == '┐'
            || c == '├'
            || c == '┤'
            || c == '└'
            || c == '┘'
            || c == '╭'
            || c == '╮'
            || c == '╰'
            || c == '╯'
            || c == '┼'
            || c == '┬'
            || c == '┴'
            || c.is_whitespace()
    })
}

/// Truncate a cell value to max_length with ellipsis
fn truncate_cell(value: &str, max_length: usize) -> String {
    let trimmed = value.trim();
    if trimmed.width() <= max_length {
        trimmed.to_string()
    } else {
        // Find a safe truncation point
        let mut result = String::new();
        let mut width = 0;
        for c in trimmed.chars() {
            let char_width = c.width().unwrap_or(1);
            if width + char_width + 1 > max_length {
                // Leave room for ellipsis
                break;
            }
            result.push(c);
            width += char_width;
        }
        result.push('…');
        result
    }
}

/// Pager state for navigation
pub struct Pager {
    /// Table data
    data: TableData,
    /// Current row offset (first visible row)
    row_offset: usize,
    /// Current column offset (first visible column)
    col_offset: usize,
    /// Page size (rows per page)
    page_size: usize,
    /// Terminal width
    term_width: usize,
    /// Terminal height
    #[allow(dead_code)]
    term_height: usize,
    /// Total row count in result
    total_rows: usize,
}

impl Pager {
    /// Create a new pager with parsed table data
    pub fn new(content: String, row_count: usize, config: &PagerConfig) -> Self {
        let (term_width, term_height) = terminal_size().unwrap_or((120, 24));
        let page_size = config.effective_page_size();

        let data = TableData::parse_from_content(&content).unwrap_or_else(|| {
            // Fallback: create single-column display
            TableData {
                columns: vec![ColumnData {
                    header: "Output".to_string(),
                    values: content.lines().map(|s| s.to_string()).collect(),
                    display_width: term_width as usize - 4,
                }],
                row_count: content.lines().count(),
            }
        });

        Pager {
            data,
            row_offset: 0,
            col_offset: 0,
            page_size,
            term_width: term_width as usize,
            term_height: term_height as usize,
            total_rows: row_count,
        }
    }

    /// Calculate how many columns can fit in the terminal width
    /// Sprint 28: Accounts for indicator cells when columns are hidden
    fn visible_column_count(&self) -> usize {
        let hidden_left = self.hidden_columns_left();
        let hidden_right_possible = self.data.columns.len().saturating_sub(self.col_offset + 1) > 0;

        // Reserve space for indicator cells if columns are hidden
        // Indicator rendering: " " + centered(10) + " " + "│" = 13 chars total
        let left_indicator_width = if hidden_left > 0 { INDICATOR_WIDTH + 3 } else { 0 };
        let right_indicator_width = if hidden_right_possible { INDICATOR_WIDTH + 3 } else { 0 };

        // Start with leading border (1 char) + left indicator (if any)
        let mut total_width = 1 + left_indicator_width;
        let mut count = 0;

        for col in self.data.columns.iter().skip(self.col_offset) {
            // Column rendering: " " + value(width) + " " + "│" = width + 3
            let col_width = col.display_width + 3;
            // Account for right indicator when checking if column fits
            let available_width = self.term_width.saturating_sub(right_indicator_width);
            if total_width + col_width > available_width && count > 0 {
                break;
            }
            total_width += col_width;
            count += 1;
        }

        count.max(1) // Always show at least 1 column
    }

    /// Calculate number of columns hidden to the left
    /// Sprint 28: For column position indicators
    fn hidden_columns_left(&self) -> usize {
        self.col_offset
    }

    /// Calculate number of columns hidden to the right
    /// Sprint 28: For column position indicators
    fn hidden_columns_right(&self) -> usize {
        let visible = self.visible_column_count();
        let end_col = (self.col_offset + visible).min(self.data.columns.len());
        self.data.columns.len().saturating_sub(end_col)
    }

    /// Render the current view
    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Clear screen and move to top
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);

        // Render top border
        self.render_border(&mut stdout, "top")?;

        // Render header row
        self.render_header(&mut stdout, self.col_offset, end_col)?;

        // Render header separator
        self.render_border(&mut stdout, "middle")?;

        // Render data rows
        for row_idx in self.row_offset..end_row {
            self.render_row(&mut stdout, row_idx, self.col_offset, end_col)?;
        }

        // Render bottom border
        self.render_border(&mut stdout, "bottom")?;

        // Render status bar
        self.render_status_bar(&mut stdout)?;

        stdout.flush()
    }

    /// Render a table border
    /// Sprint 28: Updated to include indicator cell borders when columns are hidden
    fn render_border(&self, stdout: &mut impl Write, position: &str) -> io::Result<()> {
        let (left, middle, right, line) = match position {
            "top" => ('╭', '┬', '╮', '─'),
            "middle" => ('├', '┼', '┤', '─'),
            "bottom" => ('╰', '┴', '╯', '─'),
            _ => ('├', '┼', '┤', '─'),
        };

        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        let mut border = String::new();
        border.push(left);

        // Left indicator cell border (if columns hidden to left)
        if hidden_left > 0 {
            border.push_str(&line.to_string().repeat(INDICATOR_WIDTH + 2));
            border.push(middle);
        }

        // Data column borders
        for (i, col) in self.data.columns[self.col_offset..end_col]
            .iter()
            .enumerate()
        {
            border.push_str(&line.to_string().repeat(col.display_width + 2));
            if i < end_col - self.col_offset - 1 {
                border.push(middle);
            }
        }

        // Right indicator cell border (if columns hidden to right)
        if hidden_right > 0 {
            border.push(middle);
            border.push_str(&line.to_string().repeat(INDICATOR_WIDTH + 2));
        }

        border.push(right);

        writeln!(stdout, "{}", border)
    }

    /// Render the header row
    /// Sprint 28: Updated to include column position indicator cells
    fn render_header(
        &self,
        stdout: &mut impl Write,
        start_col: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        // Write leading border FIRST (matches render_row pattern)
        write!(stdout, "│")?;

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            let indicator = format!("(+{} cols)", hidden_left);
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            // Write indicator with dim color
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        // Data column headers
        for col in &self.data.columns[start_col..end_col] {
            let padded = format!(" {:^width$} ", col.header, width = col.display_width);
            // Use bold/cyan for header
            execute!(stdout, SetForegroundColor(Color::Cyan))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            let indicator = format!("(+{} cols)", hidden_right);
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        writeln!(stdout)
    }

    /// Render a data row
    ///
    /// Sprint 8 Bug Fix: Write leading border, simplified logic
    /// Sprint 28: Updated to include column position indicator cells
    fn render_row(
        &self,
        stdout: &mut impl Write,
        row_idx: usize,
        start_col: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        // Sprint 8 Fix: Write leading border FIRST
        write!(stdout, "│")?;

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            // Show left arrow indicator in data rows
            let indicator = "    <--   ";
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, " {} ", indicator)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        for col in &self.data.columns[start_col..end_col] {
            let value = col.values.get(row_idx).map(|s| s.as_str()).unwrap_or("");
            let is_null = value == "[NULL]";

            // Pad value to column width
            let padded = format!(" {:width$} ", value, width = col.display_width);

            if is_null {
                // Dim color for NULL values
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{}", padded)?;
                execute!(stdout, ResetColor)?;
            } else {
                write!(stdout, "{}", padded)?;
            }

            // Write column separator
            write!(stdout, "│")?;
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            // Show right arrow indicator in data rows
            let indicator = "   -->    ";
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, " {} ", indicator)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        writeln!(stdout)
    }

    /// Render the status bar
    /// Sprint 28: Updated with clearer navigation hints and column indicators
    fn render_status_bar(&self, stdout: &mut impl Write) -> io::Result<()> {
        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        // Calculate progress percentage
        let progress = if self.data.row_count == 0 {
            100
        } else {
            (end_row * 100) / self.data.row_count
        };

        // Build status line with column and row positions
        let col_status = format!(
            "Columns {}-{} of {}",
            self.col_offset + 1,
            end_col,
            self.data.columns.len()
        );

        let row_status = format!(
            "Rows {}-{} of {} ({}%)",
            self.row_offset + 1,
            end_row,
            self.total_rows,
            progress
        );

        // Build navigation hints based on what navigation is possible
        let mut nav_parts = Vec::new();

        // Horizontal navigation (only show if columns are hidden)
        if hidden_left > 0 || hidden_right > 0 {
            nav_parts.push("<- ->: scroll cols");
        }

        // Vertical navigation
        nav_parts.push("j/k Space/b: rows");
        nav_parts.push("g/G: first/last");
        nav_parts.push("?: help");
        nav_parts.push("q/Esc: exit");

        let nav_hints = nav_parts.join(" | ");

        writeln!(stdout)?;
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        writeln!(stdout, "{} | {} | {}", col_status, row_status, nav_hints)?;
        execute!(stdout, ResetColor)?;

        Ok(())
    }

    /// Handle navigation input
    /// Returns Ok(true) to continue paging, Ok(false) to exit pager
    fn handle_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            // Exit pager
            KeyCode::Char('q') | KeyCode::Esc => return Ok(false),

            // Vertical navigation
            KeyCode::Char('j') | KeyCode::Down => {
                if self.row_offset + self.page_size < self.data.row_count {
                    self.row_offset += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.row_offset = self.row_offset.saturating_sub(1);
            }
            KeyCode::Char(' ') | KeyCode::PageDown => {
                let max_offset = self.data.row_count.saturating_sub(self.page_size);
                self.row_offset = (self.row_offset + self.page_size).min(max_offset);
            }
            KeyCode::Char('b') | KeyCode::PageUp => {
                self.row_offset = self.row_offset.saturating_sub(self.page_size);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.row_offset = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.row_offset = self.data.row_count.saturating_sub(self.page_size);
            }

            // Horizontal navigation (column windowing)
            KeyCode::Left | KeyCode::Char('h') => {
                self.col_offset = self.col_offset.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.col_offset + self.visible_column_count() < self.data.columns.len() {
                    self.col_offset += 1;
                }
            }

            // Ctrl-Left: Jump to first column
            KeyCode::Char('H') => {
                self.col_offset = 0;
            }
            // Ctrl-Right: Jump to last column window
            KeyCode::Char('L') => {
                let visible = self.visible_column_count();
                self.col_offset = self.data.columns.len().saturating_sub(visible);
            }

            // Help display
            KeyCode::Char('?') => {
                self.show_help()?;
            }

            _ => {}
        }
        Ok(true) // Continue paging
    }

    /// Display help overlay showing all navigation keys
    /// REQ-PAGER-HORIZ-011: Help text documents horizontal navigation keys
    fn show_help(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Clear screen and show help
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        let help_text = r#"
Navigation Keys
===============

Vertical (Row) Navigation:
  ↑ or k      Scroll up one row
  ↓ or j      Scroll down one row
  Space       Page down (next page)
  b           Page up (previous page)
  g / Home    Jump to first row
  G / End     Jump to last row

Horizontal (Column) Navigation:
  ← or h      Scroll left one column
  → or l      Scroll right one column
  H           Jump to first column
  L           Jump to last column

Column Indicators:
  (+N cols)   Shows N hidden columns in that direction
  <--         Arrow pointing left indicates more columns to the left
  -->         Arrow pointing right indicates more columns to the right

Note: Column position is preserved when scrolling vertically.

Exit:
  q / Esc     Exit pager and return to REPL prompt

Press any key to return to results..."#;

        execute!(stdout, SetForegroundColor(Color::Cyan))?;
        writeln!(stdout, "{}", help_text)?;
        execute!(stdout, ResetColor)?;
        stdout.flush()?;

        // Wait for any key press
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }

        // Re-render the table
        self.render()?;

        Ok(())
    }

    /// Run the pager event loop
    pub fn run(&mut self) -> io::Result<()> {
        // Enter alternate screen and raw mode
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;

        // Initial render
        self.render()?;

        // Event loop
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key(key)? {
                        break; // Exit pager
                    }
                    self.render()?;
                }
                if let Event::Resize(w, h) = event::read().unwrap_or(Event::FocusGained) {
                    self.term_width = w as usize;
                    self.term_height = h as usize;
                    self.page_size = self.term_height.saturating_sub(5);
                    self.render()?;
                }
            }
        }

        // Leave alternate screen and disable raw mode
        execute!(stdout, Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
}

/// Represents paged output with navigation state (legacy API compatibility)
#[allow(dead_code)]
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

    /// Get the full content as a string
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }
}

/// Display content using the custom pager for interactive scrolling
///
/// Sprint 8: Complete rewrite using crossterm instead of minus.
/// CRITICAL: 'q' now returns to REPL instead of exiting the program.
///
/// # Arguments
/// * `content` - The content to display (typically formatted table output)
/// * `row_count` - Number of rows in the result (for the status bar)
/// * `config` - Pager configuration
///
/// # Returns
/// * `Ok(true)` if paging was used
/// * `Ok(false)` if content didn't need paging (should be displayed directly)
/// * `Err` if paging failed
pub fn display_with_pager(
    content: &str,
    row_count: usize,
    config: &PagerConfig,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let paged = PagedOutput::new(content.to_string(), config.clone());

    if !paged.needs_paging() {
        // Content doesn't need paging - return false so caller can display directly
        return Ok(false);
    }

    log::debug!(
        "Starting custom pager for {} lines ({} rows)",
        paged.total_lines(),
        row_count
    );

    // Create and run the pager
    let mut pager = Pager::new(content.to_string(), row_count, config);

    // Run pager - this blocks until user presses 'q'
    // CRITICAL: This returns normally, it does NOT exit the program
    pager.run()?;

    log::debug!("Pager exited normally, returning to REPL");

    Ok(true)
}

/// Check if content should be paged based on configuration and size
pub fn should_page(content: &str, config: &PagerConfig) -> bool {
    if !config.vertical_paging && !config.horizontal_scrolling {
        return false;
    }

    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();
    let max_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);

    let page_size = config.effective_page_size();
    let visible_width = config.effective_visible_width();

    let needs_vertical =
        config.vertical_paging && line_count > config.min_rows_for_paging && line_count > page_size;

    let needs_horizontal = config.horizontal_scrolling
        && max_width > config.min_cols_for_scrolling
        && max_width > visible_width;

    needs_vertical || needs_horizontal
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
    fn test_truncate_cell_short() {
        let value = "Hello";
        assert_eq!(truncate_cell(value, 10), "Hello");
    }

    #[test]
    fn test_truncate_cell_long() {
        let value = "This is a very long string that should be truncated";
        let truncated = truncate_cell(value, 20);
        // The display width should be <= 20, byte length may be longer due to ellipsis
        assert!(truncated.width() <= 20);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn test_truncate_cell_exact() {
        let value = "ExactLen";
        assert_eq!(truncate_cell(value, 8), "ExactLen");
    }

    #[test]
    fn test_parse_row_cells() {
        let line = "│ id │ name │ value │";
        let cells = parse_row_cells(line);
        assert_eq!(cells, vec!["id", "name", "value"]);
    }

    #[test]
    fn test_is_separator_line() {
        assert!(is_separator_line("├────┼────────┼───────┤"));
        assert!(is_separator_line("╭────┬────────┬───────╮"));
        assert!(!is_separator_line("│ id │ name   │ value │"));
    }

    #[test]
    fn test_should_page_small_content() {
        let content = "Line 1\nLine 2\nLine 3";
        let config = PagerConfig::default();
        assert!(!should_page(content, &config));
    }

    // Sprint 28: Tests for column indicator calculations
    #[test]
    fn test_indicator_width_constant() {
        // Verify INDICATOR_WIDTH can hold typical indicators like "(+99 cols)"
        // The indicator format "(+99 cols)" is 10 chars, so INDICATOR_WIDTH must be >= 10
        let sample_indicator = "(+99 cols)";
        assert!(
            INDICATOR_WIDTH >= sample_indicator.len(),
            "INDICATOR_WIDTH {} must be >= {} to hold indicators like '{}'",
            INDICATOR_WIDTH,
            sample_indicator.len(),
            sample_indicator
        );
    }

    #[test]
    fn test_hidden_columns_at_start() {
        // When col_offset is 0, no columns are hidden to the left
        let content = r#"╭──────┬──────┬──────╮
│ col1 │ col2 │ col3 │
├──────┼──────┼──────┤
│ a    │ b    │ c    │
╰──────┴──────┴──────╯"#;
        let config = PagerConfig::default();
        let pager = Pager::new(content.to_string(), 1, &config);
        assert_eq!(pager.hidden_columns_left(), 0);
    }

    #[test]
    fn test_table_data_parse_columns() {
        let content = r#"╭──────┬──────┬──────╮
│ col1 │ col2 │ col3 │
├──────┼──────┼──────┤
│ a    │ b    │ c    │
│ d    │ e    │ f    │
╰──────┴──────┴──────╯"#;
        let data = TableData::parse_from_content(content).unwrap();
        assert_eq!(data.columns.len(), 3);
        assert_eq!(data.row_count, 2);
    }

    // Sprint 29: Tests for horizontal navigation features
    #[test]
    fn test_pager_initial_column_offset() {
        // New pager should start at column offset 0
        let content = r#"╭──────┬──────┬──────╮
│ col1 │ col2 │ col3 │
├──────┼──────┼──────┤
│ a    │ b    │ c    │
╰──────┴──────┴──────╯"#;
        let config = PagerConfig::default();
        let pager = Pager::new(content.to_string(), 1, &config);
        assert_eq!(pager.col_offset, 0);
    }

    #[test]
    fn test_pager_visible_column_count_minimum_one() {
        // Should always show at least one column
        let content = r#"╭──────┬──────┬──────╮
│ col1 │ col2 │ col3 │
├──────┼──────┼──────┤
│ a    │ b    │ c    │
╰──────┴──────┴──────╯"#;
        let config = PagerConfig::default();
        let pager = Pager::new(content.to_string(), 1, &config);
        assert!(pager.visible_column_count() >= 1);
    }

    #[test]
    fn test_pager_hidden_columns_right_calculation() {
        // With 3 columns and starting at offset 0, hidden right depends on terminal width
        let content = r#"╭──────┬──────┬──────╮
│ col1 │ col2 │ col3 │
├──────┼──────┼──────┤
│ a    │ b    │ c    │
╰──────┴──────┴──────╯"#;
        let config = PagerConfig::default();
        let pager = Pager::new(content.to_string(), 1, &config);
        // hidden_columns_right = total_cols - (col_offset + visible_cols)
        let expected_hidden = pager
            .data
            .columns
            .len()
            .saturating_sub(pager.col_offset + pager.visible_column_count());
        assert_eq!(pager.hidden_columns_right(), expected_hidden);
    }

    #[test]
    fn test_status_bar_includes_help_hint() {
        // Verify the status bar nav_parts include help key
        // This is a code structure test - the status bar should mention '?'
        // We test this by verifying the constant behavior in the code
        let mut buffer = Vec::new();
        let content = r#"╭──────┬──────╮
│ col1 │ col2 │
├──────┼──────┤
│ a    │ b    │
╰──────┴──────╯"#;
        let config = PagerConfig::default();
        let pager = Pager::new(content.to_string(), 1, &config);
        // render_status_bar writes to buffer - we just verify no panic
        let _ = pager.render_status_bar(&mut buffer);
        let output = String::from_utf8_lossy(&buffer);
        // Status bar should mention help key
        assert!(output.contains("?") || output.contains("help"));
    }
}
