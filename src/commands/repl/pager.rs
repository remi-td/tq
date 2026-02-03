//! Custom Result Pager for Large Result Sets
//!
//! Sprint 30: Architectural refactor to accept QueryResult directly instead of
//! pre-formatted strings. This fixes the fundamental Sprint 29 bug where the
//! pager received 1221-character-wide pre-formatted tables for 117-char terminals.
//!
//! ## Architecture Change
//!
//! Sprint 29 (broken): Executor → format table → string → Pager parses string
//! Sprint 30 (fixed):  Executor → QueryResult → Pager formats at render time
//!
//! By accepting structured data, the pager can:
//! - Calculate column widths based on actual terminal width
//! - Select which columns fit without pre-rendering all columns
//! - Navigate horizontally at the column level, not character level
//!
//! ## Features
//!
//! - Column windowing: Show columns that fit terminal, navigate with h/l
//! - Row paging: Navigate with j/k, Space/b, g/G
//! - Cell truncation: Long values truncated with ellipsis
//! - Safe exit: 'q' returns to REPL, never exits program
//!
//! ## Key Bindings
//!
//! Vertical: j/k (row), Space/b (page), g/G (first/last)
//! Horizontal: h/l (column), H/L (first/last column)
//! Help: ? (show help overlay)
//! Exit: q/Esc (return to REPL)

use crate::db::{Alignment, QueryResult};
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
use unicode_width::UnicodeWidthStr;

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
            min_cols_for_scrolling: 0, // Use terminal width
            page_size: 0,              // Auto-detect
            visible_width: 0,          // Auto-detect
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
            terminal_size()
                .map(|(_, h)| (h as usize).saturating_sub(5))
                .unwrap_or(20)
        }
    }

    /// Get effective visible width (auto-detect if 0)
    pub fn effective_visible_width(&self) -> usize {
        if self.visible_width > 0 {
            self.visible_width
        } else {
            terminal_size().map(|(w, _)| w as usize).unwrap_or(120)
        }
    }
}

/// A single column with its calculated display properties
#[derive(Debug, Clone)]
struct ColumnInfo {
    /// Column name (header)
    name: String,
    /// Calculated display width for this column (content width, not including borders)
    display_width: usize,
    /// Alignment for this column
    alignment: Alignment,
}

/// Represents the structured table data for paging
///
/// Sprint 30: Built directly from QueryResult, no string parsing
#[derive(Debug)]
pub struct TableData {
    /// Column information with calculated widths
    columns: Vec<ColumnInfo>,
    /// Cell values as strings (pre-truncated)
    /// Indexed as [row_index][column_index]
    cell_values: Vec<Vec<String>>,
    /// Total number of rows
    row_count: usize,
}

impl TableData {
    /// Create TableData directly from QueryResult
    ///
    /// Sprint 30: This is the NEW method that replaces parse_from_content().
    /// By accepting QueryResult directly, we can calculate proper column widths
    /// based on actual data without the overhead of string parsing.
    ///
    /// # Arguments
    /// * `result` - The query result with columns and rows
    /// * `max_col_width` - Maximum width for any column (typically MAX_COLUMN_WIDTH)
    pub fn from_query_result(result: &QueryResult, max_col_width: usize) -> Self {
        let mut columns = Vec::with_capacity(result.columns.len());
        let mut cell_values: Vec<Vec<String>> = vec![Vec::new(); result.rows.len()];

        for (col_idx, col_meta) in result.columns.iter().enumerate() {
            // Truncate header if needed
            let header = truncate_cell(&col_meta.name, max_col_width.saturating_sub(2));
            let header_width = header.width();

            // Calculate max value width for this column
            let mut max_value_width = header_width;

            for (row_idx, row) in result.rows.iter().enumerate() {
                let value = if col_idx < row.len() {
                    row[col_idx].display()
                } else {
                    "[NULL]".to_string()
                };

                // Truncate cell value
                let truncated = truncate_cell(&value, MAX_CELL_LENGTH);
                let value_width = truncated.width();

                max_value_width = max_value_width.max(value_width);
                cell_values[row_idx].push(truncated);
            }

            // Apply width constraints
            let display_width = max_value_width
                .max(MIN_COLUMN_WIDTH)
                .min(max_col_width);

            columns.push(ColumnInfo {
                name: header,
                display_width,
                alignment: col_meta.data_type.alignment(),
            });
        }

        TableData {
            columns,
            cell_values,
            row_count: result.rows.len(),
        }
    }

    /// Get cell value for a specific row and column index
    fn get_cell(&self, row_idx: usize, col_idx: usize) -> &str {
        self.cell_values
            .get(row_idx)
            .and_then(|row| row.get(col_idx))
            .map(|s| s.as_str())
            .unwrap_or("")
    }
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
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if width + char_width + 1 > max_length {
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
    /// Table data (structured from QueryResult)
    data: TableData,
    /// Query execution time for status bar
    execution_time: std::time::Duration,
    /// Current row offset (first visible row)
    row_offset: usize,
    /// Current column offset (first visible column)
    col_offset: usize,
    /// Page size (rows per page)
    page_size: usize,
    /// Terminal width
    term_width: usize,
    /// Terminal height
    term_height: usize,
    /// Total row count in result (may differ from data.row_count if limited)
    total_rows: usize,
}

impl Pager {
    /// Create a new pager from QueryResult
    ///
    /// Sprint 30: NEW constructor that accepts QueryResult directly.
    /// This eliminates the string parsing step that caused Sprint 29 issues.
    ///
    /// # Arguments
    /// * `result` - The query result to display
    /// * `config` - Pager configuration
    pub fn new(result: &QueryResult, config: &PagerConfig) -> Self {
        let (term_width, term_height) = terminal_size().unwrap_or((120, 24));
        let page_size = config.effective_page_size();

        // Build table data from QueryResult with appropriate column width limit
        let data = TableData::from_query_result(result, MAX_COLUMN_WIDTH);

        log::debug!(
            "Pager initialized: {} columns, {} rows, term_width={}, page_size={}",
            data.columns.len(),
            data.row_count,
            term_width,
            page_size
        );

        Pager {
            data,
            execution_time: result.execution_time,
            row_offset: 0,
            col_offset: 0,
            page_size,
            term_width: term_width as usize,
            term_height: term_height as usize,
            total_rows: result.row_count,
        }
    }

    /// Calculate how many columns can fit in the terminal width
    ///
    /// Sprint 30: Uses actual column widths from TableData, calculated at
    /// construction time from QueryResult.
    fn visible_column_count(&self) -> usize {
        let hidden_left = self.hidden_columns_left();
        let hidden_right_possible =
            self.data.columns.len().saturating_sub(self.col_offset + 1) > 0;

        // Reserve space for indicator cells if columns are hidden
        // Indicator rendering: " " + centered(10) + " " + "│" = 13 chars total
        let left_indicator_width = if hidden_left > 0 {
            INDICATOR_WIDTH + 3
        } else {
            0
        };
        let right_indicator_width = if hidden_right_possible {
            INDICATOR_WIDTH + 3
        } else {
            0
        };

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
    fn hidden_columns_left(&self) -> usize {
        self.col_offset
    }

    /// Calculate number of columns hidden to the right
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
        self.render_border(&mut stdout, BorderType::Top)?;

        // Render header row
        self.render_header(&mut stdout, self.col_offset, end_col)?;

        // Render header separator
        self.render_border(&mut stdout, BorderType::Middle)?;

        // Render data rows
        for row_idx in self.row_offset..end_row {
            self.render_row(&mut stdout, row_idx, self.col_offset, end_col)?;
        }

        // Render bottom border
        self.render_border(&mut stdout, BorderType::Bottom)?;

        // Render status bar
        self.render_status_bar(&mut stdout)?;

        stdout.flush()
    }

    /// Render a table border
    fn render_border(&self, stdout: &mut impl Write, border_type: BorderType) -> io::Result<()> {
        let (left, middle, right, line) = match border_type {
            BorderType::Top => ('╭', '┬', '╮', '─'),
            BorderType::Middle => ('├', '┼', '┤', '─'),
            BorderType::Bottom => ('╰', '┴', '╯', '─'),
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
    ///
    /// Sprint 30: Formats directly from TableData columns, no string parsing
    fn render_header(
        &self,
        stdout: &mut impl Write,
        start_col: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        write!(stdout, "│")?;

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            let indicator = format!("(+{} cols)", hidden_left);
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        // Data column headers
        for col in &self.data.columns[start_col..end_col] {
            let padded = format!(" {:^width$} ", col.name, width = col.display_width);
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
    /// Sprint 30: Formats directly from TableData cell values with proper alignment
    fn render_row(
        &self,
        stdout: &mut impl Write,
        row_idx: usize,
        start_col: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        write!(stdout, "│")?;

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            let indicator = "    <--   ";
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, " {} ", indicator)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        for (vis_idx, col) in self.data.columns[start_col..end_col].iter().enumerate() {
            let col_idx = start_col + vis_idx;
            let value = self.data.get_cell(row_idx, col_idx);
            let is_null = value == "[NULL]";

            // Format value with alignment
            let padded = match col.alignment {
                Alignment::Right => format!(" {:>width$} ", value, width = col.display_width),
                Alignment::Center => format!(" {:^width$} ", value, width = col.display_width),
                Alignment::Left => format!(" {:width$} ", value, width = col.display_width),
            };

            if is_null {
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{}", padded)?;
                execute!(stdout, ResetColor)?;
            } else {
                write!(stdout, "{}", padded)?;
            }

            write!(stdout, "│")?;
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            let indicator = "   -->    ";
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, " {} ", indicator)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        writeln!(stdout)
    }

    /// Render the status bar
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

        let timing = format!("{:.3}s", self.execution_time.as_secs_f64());

        // Build navigation hints based on what navigation is possible
        let mut nav_parts = Vec::new();

        if hidden_left > 0 || hidden_right > 0 {
            nav_parts.push("<- ->: scroll cols");
        }

        nav_parts.push("j/k Space/b: rows");
        nav_parts.push("g/G: first/last");
        nav_parts.push("?: help");
        nav_parts.push("q/Esc: exit");

        let nav_hints = nav_parts.join(" | ");

        writeln!(stdout)?;
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        writeln!(
            stdout,
            "{} | {} | {} | {}",
            col_status, row_status, timing, nav_hints
        )?;
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

            // Jump to first column
            KeyCode::Char('H') => {
                self.col_offset = 0;
            }
            // Jump to last column window
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
        Ok(true)
    }

    /// Display help overlay showing all navigation keys
    fn show_help(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

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
                        break;
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

        execute!(stdout, Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
}

/// Border type for rendering
#[derive(Debug, Clone, Copy)]
enum BorderType {
    Top,
    Middle,
    Bottom,
}

/// Check if content should be paged based on configuration and result size
pub fn should_page(result: &QueryResult, config: &PagerConfig) -> bool {
    if !config.vertical_paging && !config.horizontal_scrolling {
        return false;
    }

    let page_size = config.effective_page_size();
    let visible_width = config.effective_visible_width();

    // Check vertical paging threshold
    let needs_vertical = config.vertical_paging
        && result.row_count > config.min_rows_for_paging
        && result.row_count > page_size;

    // Check horizontal scrolling - estimate if columns would exceed terminal width
    // Use MAX_COLUMN_WIDTH as upper bound for column width estimation
    let estimated_width: usize = result
        .columns
        .iter()
        .map(|c| c.name.len().clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH) + 3)
        .sum();
    let needs_horizontal = config.horizontal_scrolling
        && result.columns.len() > config.min_cols_for_scrolling
        && estimated_width > visible_width;

    needs_vertical || needs_horizontal
}

/// Display QueryResult using the pager
///
/// Sprint 30: New entry point that accepts QueryResult directly.
///
/// # Arguments
/// * `result` - The query result to display
/// * `config` - Pager configuration
///
/// # Returns
/// * `Ok(true)` if paging was used
/// * `Ok(false)` if content didn't need paging
/// * `Err` if paging failed
pub fn display_with_pager(
    result: &QueryResult,
    config: &PagerConfig,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if !should_page(result, config) {
        return Ok(false);
    }

    log::debug!(
        "Starting pager for {} columns, {} rows",
        result.columns.len(),
        result.row_count
    );

    let mut pager = Pager::new(result, config);
    pager.run()?;

    log::debug!("Pager exited normally, returning to REPL");

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ColumnMetadata, TeradataType, Value};
    use std::time::Duration;

    fn create_test_result(num_cols: usize, num_rows: usize) -> QueryResult {
        let columns: Vec<ColumnMetadata> = (0..num_cols)
            .map(|i| ColumnMetadata::new(format!("col{}", i), TeradataType::Varchar, true))
            .collect();

        let rows: Vec<Vec<Value>> = (0..num_rows)
            .map(|r| {
                (0..num_cols)
                    .map(|c| Value::String(format!("val_{}_{}", r, c)))
                    .collect()
            })
            .collect();

        QueryResult::new(columns, rows, Duration::from_millis(100))
    }

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
    fn test_table_data_from_query_result() {
        let result = create_test_result(3, 2);
        let data = TableData::from_query_result(&result, MAX_COLUMN_WIDTH);

        assert_eq!(data.columns.len(), 3);
        assert_eq!(data.row_count, 2);
        assert_eq!(data.cell_values.len(), 2);
        assert_eq!(data.cell_values[0].len(), 3);
    }

    #[test]
    fn test_table_data_cell_access() {
        let result = create_test_result(2, 2);
        let data = TableData::from_query_result(&result, MAX_COLUMN_WIDTH);

        assert_eq!(data.get_cell(0, 0), "val_0_0");
        assert_eq!(data.get_cell(0, 1), "val_0_1");
        assert_eq!(data.get_cell(1, 0), "val_1_0");
        assert_eq!(data.get_cell(1, 1), "val_1_1");
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
        assert!(truncated.width() <= 20);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn test_truncate_cell_exact() {
        let value = "ExactLen";
        assert_eq!(truncate_cell(value, 8), "ExactLen");
    }

    #[test]
    fn test_should_page_small_result() {
        let result = create_test_result(3, 5);
        let config = PagerConfig::default();
        assert!(!should_page(&result, &config));
    }

    #[test]
    fn test_should_page_many_rows() {
        let result = create_test_result(3, 100);
        let config = PagerConfig::default();
        // Should need paging due to row count
        assert!(should_page(&result, &config));
    }

    #[test]
    fn test_pager_initial_state() {
        let result = create_test_result(5, 10);
        let config = PagerConfig::default();
        let pager = Pager::new(&result, &config);

        assert_eq!(pager.row_offset, 0);
        assert_eq!(pager.col_offset, 0);
        assert_eq!(pager.total_rows, 10);
        assert_eq!(pager.data.columns.len(), 5);
    }

    #[test]
    fn test_pager_hidden_columns_at_start() {
        let result = create_test_result(3, 2);
        let config = PagerConfig::default();
        let pager = Pager::new(&result, &config);

        // At start, no columns hidden to left
        assert_eq!(pager.hidden_columns_left(), 0);
    }

    #[test]
    fn test_pager_visible_column_count_minimum_one() {
        let result = create_test_result(3, 2);
        let config = PagerConfig::default();
        let pager = Pager::new(&result, &config);

        // Should always show at least one column
        assert!(pager.visible_column_count() >= 1);
    }

    #[test]
    fn test_indicator_width_constant() {
        // Verify INDICATOR_WIDTH can hold typical indicators like "(+99 cols)"
        let sample_indicator = "(+99 cols)";
        assert!(
            INDICATOR_WIDTH >= sample_indicator.len(),
            "INDICATOR_WIDTH {} must be >= {} to hold indicators",
            INDICATOR_WIDTH,
            sample_indicator.len()
        );
    }

    #[test]
    fn test_column_alignment_preserved() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
            ColumnMetadata::new("active", TeradataType::Boolean, false),
        ];
        let rows = vec![vec![
            Value::Integer(1),
            Value::String("Alice".into()),
            Value::Boolean(true),
        ]];
        let result = QueryResult::new(columns, rows, Duration::from_millis(50));

        let data = TableData::from_query_result(&result, MAX_COLUMN_WIDTH);

        assert_eq!(data.columns[0].alignment, Alignment::Right); // Integer
        assert_eq!(data.columns[1].alignment, Alignment::Left); // Varchar
        assert_eq!(data.columns[2].alignment, Alignment::Center); // Boolean
    }

    #[test]
    fn test_null_value_display() {
        let columns = vec![ColumnMetadata::new("col", TeradataType::Varchar, true)];
        let rows = vec![vec![Value::Null]];
        let result = QueryResult::new(columns, rows, Duration::from_millis(50));

        let data = TableData::from_query_result(&result, MAX_COLUMN_WIDTH);

        assert_eq!(data.get_cell(0, 0), "[NULL]");
    }
}
