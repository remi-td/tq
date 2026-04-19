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
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as terminal_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

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
                .map(|(_, h)| (h as usize).saturating_sub(6))
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
    /// Sprint 31: CRITICAL FIX - Cell values must be truncated to display_width,
    /// not MAX_CELL_LENGTH. The previous code could store 100-char values but
    /// set display_width to 40, causing line overflow when format! expanded.
    ///
    /// # Arguments
    /// * `result` - The query result with columns and rows
    /// * `max_col_width` - Maximum width for any column (typically MAX_COLUMN_WIDTH)
    pub fn from_query_result(result: &QueryResult, max_col_width: usize) -> Self {
        // PASS 1: Calculate display_width for each column
        let mut column_widths: Vec<usize> = Vec::with_capacity(result.columns.len());

        for (col_idx, col_meta) in result.columns.iter().enumerate() {
            // Header width (truncated to max_col_width - 2 for padding)
            let header = truncate_cell(&col_meta.name, max_col_width.saturating_sub(2));
            let header_width = header.width();

            // Find max value width in this column
            let mut max_value_width = header_width;
            for row in &result.rows {
                let value = if col_idx < row.len() {
                    row[col_idx].display()
                } else {
                    "[NULL]".to_string()
                };
                let value_width = value.trim().width();
                max_value_width = max_value_width.max(value_width);
            }

            // Apply width constraints: MIN <= display_width <= max_col_width
            let display_width = max_value_width.max(MIN_COLUMN_WIDTH).min(max_col_width);
            column_widths.push(display_width);
        }

        // PASS 2: Build columns and truncate cell values to display_width
        let mut columns = Vec::with_capacity(result.columns.len());
        let mut cell_values: Vec<Vec<String>> = vec![Vec::new(); result.rows.len()];

        for (col_idx, col_meta) in result.columns.iter().enumerate() {
            let display_width = column_widths[col_idx];

            // Truncate header to display_width
            let header = truncate_cell(&col_meta.name, display_width);

            // Truncate cell values to display_width (not MAX_CELL_LENGTH!)
            for (row_idx, row) in result.rows.iter().enumerate() {
                let value = if col_idx < row.len() {
                    row[col_idx].display()
                } else {
                    "[NULL]".to_string()
                };

                // CRITICAL: Truncate to display_width, not MAX_CELL_LENGTH
                let truncated = truncate_cell(&value, display_width);
                cell_values[row_idx].push(truncated);
            }

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

/// Pad a string to the specified display width using visual width calculation
///
/// Sprint 33: Fixes Issue #14 - format! width specifier uses character count,
/// not display width. This function correctly pads based on visual width,
/// handling CJK characters and emoji that take 2 display columns per character.
///
/// # Arguments
/// * `value` - The string to pad
/// * `width` - Target display width (visual columns)
/// * `alignment` - How to align the content within the padded space
///
/// # Returns
/// A string with leading space, content, padding, and trailing space.
/// Total visual width = width + 2 (for the surrounding spaces).
fn pad_to_display_width(value: &str, width: usize, alignment: Alignment) -> String {
    let visual_width = value.width();
    let padding = width.saturating_sub(visual_width);

    match alignment {
        Alignment::Left => format!(" {}{} ", value, " ".repeat(padding)),
        Alignment::Right => format!(" {}{} ", " ".repeat(padding), value),
        Alignment::Center => {
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!(" {}{}{} ", " ".repeat(left_pad), value, " ".repeat(right_pad))
        }
    }
}

// -- Search data model (Pager Search feature) --
//
// REQ-PAGER-SEARCH-001..012: forward literal-substring search with `/`, `n`, `N`,
// case-insensitive by default, `\c` suffix to opt into case-sensitive matching.

/// A single match location within the result set.
///
/// `byte_start` / `byte_end` index into the *displayed* (post-truncation) cell
/// text returned by `TableData::get_cell`, NOT the underlying Teradata value.
/// See REQ-PAGER-SEARCH-002.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Match {
    row: usize,
    col: usize,
    byte_start: usize,
    byte_end: usize,
}

/// Active-search state. `None` on the `Pager` when no search has been submitted.
#[derive(Debug, Clone)]
struct SearchState {
    /// Pattern exactly as it will be displayed in the status bar (with any `\c`
    /// suffix stripped — see REQ-PAGER-SEARCH-006.3).
    pattern: String,
    /// True if `\c` suffix was present. Informational: all matching already
    /// happened at submit time via `find_all_matches`, so runtime rendering
    /// does not need to re-check sensitivity. Retained for debug output and
    /// so a future re-scan (on data change) has the original flag.
    #[allow(dead_code)]
    case_sensitive: bool,
    /// All matches in the full result set, sorted by `(row, col, byte_start)`.
    matches: Vec<Match>,
    /// Index into `matches` of the currently active match. `None` when
    /// `matches` is empty.
    current: Option<usize>,
}

/// Outcome of the status-bar rendering; drives whether matches were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchStatus {
    /// No search has been submitted in this pager session.
    None,
    /// A search was submitted and produced one or more matches.
    Matches,
    /// A search was submitted and produced zero matches.
    NotFound,
}

/// Direction for `n` (Next) and `N` (Previous) match navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Next,
    Prev,
}

/// REPL-style input mode for the pager event loop.
///
/// Implemented as a state-machine flag on `Pager`, NOT a nested
/// `event::read()` sub-loop — consistent with the Sprint 33 discipline of a
/// single flat event loop.
#[derive(Debug, Clone)]
enum InputMode {
    /// Normal pager navigation (`j`/`k`, `h`/`l`, `/`, `n`, etc.).
    Normal,
    /// Search prompt open, accumulating typed characters into `buffer`.
    SearchPrompt { buffer: String },
}

/// Strip the literal `\c` suffix from a search input and report case sensitivity.
///
/// REQ-PAGER-SEARCH-006.2/.3: `"Foo\\c"` -> `("Foo", true)`; any other suffix
/// -> `(input, false)`. The `\c` suffix is stripped before matching and before
/// status-bar display. An input of just `"\\c"` produces `("", true)` — empty
/// pattern is NOT special-cased, the normal no-match flow handles it.
fn parse_search_input(input: &str) -> (String, bool) {
    if let Some(stripped) = input.strip_suffix("\\c") {
        (stripped.to_string(), true)
    } else {
        (input.to_string(), false)
    }
}

/// Case-insensitive byte-equality check, ASCII-only.
///
/// Case fold is ASCII-only by design: non-ASCII bytes compare exactly. Calling
/// `str::to_lowercase` would allocate and is Unicode-aware overkill for this
/// use case; most SQL identifiers and data are ASCII.
fn bytes_eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Scan every cell of the displayed table data for the literal pattern.
///
/// Searches the post-truncation displayed cell text. Matches in truncated
/// tails are intentionally invisible to preserve user-visible consistency.
///
/// Returns matches sorted by `(row, col, byte_start)`. An empty pattern
/// returns an empty list — callers rely on this to render the `not found`
/// status for `"/\c"` -> empty pattern.
pub(crate) fn find_all_matches(
    data: &TableData,
    pattern: &str,
    case_sensitive: bool,
) -> Vec<Match> {
    let mut out = Vec::new();
    if pattern.is_empty() {
        return out;
    }
    let pat_bytes = pattern.as_bytes();
    for (row_idx, row) in data.cell_values.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let cell_bytes = cell.as_bytes();
            if cell_bytes.len() < pat_bytes.len() {
                continue;
            }
            let mut start = 0usize;
            while start + pat_bytes.len() <= cell_bytes.len() {
                let slice = &cell_bytes[start..start + pat_bytes.len()];
                let hit = if case_sensitive {
                    slice == pat_bytes
                } else {
                    bytes_eq_ignore_ascii_case(slice, pat_bytes)
                };
                if hit {
                    out.push(Match {
                        row: row_idx,
                        col: col_idx,
                        byte_start: start,
                        byte_end: start + pat_bytes.len(),
                    });
                    start += pat_bytes.len();
                } else {
                    start += 1;
                }
            }
        }
    }
    out
}

/// Choose the initial match index after a `/pattern` submission.
///
/// REQ-PAGER-SEARCH-002.1: pick the first match at or after `cursor_row`.
/// If no such match exists (cursor is past all matches), wraps to the first
/// match. Returns `None` iff `matches` is empty.
fn pick_initial_match(matches: &[Match], cursor_row: usize) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    for (i, m) in matches.iter().enumerate() {
        if m.row >= cursor_row {
            return Some(i);
        }
    }
    Some(0)
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
    /// Current input mode: Normal navigation or SearchPrompt accumulation.
    mode: InputMode,
    /// Active search, if any. `None` means no search has been submitted.
    search: Option<SearchState>,
    /// Summary status: whether the last submitted search found matches.
    search_status: SearchStatus,
    /// One-render-cycle overlay status (e.g. `wrapped to first match`).
    /// Cleared at the top of `handle_key` before the next event is processed.
    transient_status: Option<String>,
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
            mode: InputMode::Normal,
            search: None,
            search_status: SearchStatus::None,
            transient_status: None,
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

        // Raw mode disables kernel ONLCR translation, so \n alone won't
        // return the cursor to column 0. Use \r\n for proper line breaks.
        write!(stdout, "{}\r\n", border)
    }

    /// Render the header row
    ///
    /// Sprint 30: Formats directly from TableData columns, no string parsing
    /// Sprint 33: Uses pad_to_display_width() for correct Unicode width handling
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
            // Indicators are ASCII-only, format! is safe here
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        // Data column headers - use display-width-aware padding
        for col in &self.data.columns[start_col..end_col] {
            let padded = pad_to_display_width(&col.name, col.display_width, Alignment::Center);
            execute!(stdout, SetForegroundColor(Color::Cyan))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            let indicator = format!("(+{} cols)", hidden_right);
            // Indicators are ASCII-only, format! is safe here
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{}", padded)?;
            execute!(stdout, ResetColor)?;
            write!(stdout, "│")?;
        }

        write!(stdout, "\r\n")
    }

    /// Render a data row
    ///
    /// Sprint 30: Formats directly from TableData cell values with proper alignment
    /// Sprint 33: Uses pad_to_display_width() for correct Unicode width handling
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

            // REQ-PAGER-SEARCH-008: collect match byte-ranges in this cell
            // (only non-empty when a search is active with matches).
            let cell_matches = self.cell_match_ranges(row_idx, col_idx);

            if is_null {
                // NULL cells don't participate in search highlighting — they
                // render as the literal string `[NULL]` which won't contain
                // typical search patterns.
                let padded = pad_to_display_width(value, col.display_width, col.alignment);
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{}", padded)?;
                execute!(stdout, ResetColor)?;
            } else if cell_matches.is_empty() {
                let padded = pad_to_display_width(value, col.display_width, col.alignment);
                write!(stdout, "{}", padded)?;
            } else {
                self.render_cell_with_highlights(
                    stdout,
                    value,
                    col.display_width,
                    col.alignment,
                    &cell_matches,
                )?;
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

        write!(stdout, "\r\n")
    }

    /// Return `(byte_start, byte_end)` ranges in the cell's raw text that are
    /// match occurrences for the active search. Empty when no search is
    /// active or the cell has no matches.
    fn cell_match_ranges(&self, row_idx: usize, col_idx: usize) -> Vec<(usize, usize)> {
        let Some(search) = self.search.as_ref() else {
            return Vec::new();
        };
        if search.matches.is_empty() {
            return Vec::new();
        }
        search
            .matches
            .iter()
            .filter(|m| m.row == row_idx && m.col == col_idx)
            .map(|m| (m.byte_start, m.byte_end))
            .collect()
    }

    /// Render a single cell with in-line match highlights applied via the
    /// terminal `Reverse` attribute.
    ///
    /// Padding and alignment are preserved: the leading/trailing single-space
    /// padding characters are never highlighted, only the matched substring
    /// inside the cell value. The `Reverse` attribute composes on top of any
    /// existing foreground color (see header cyan / NULL DarkGrey) without
    /// clashing.
    fn render_cell_with_highlights(
        &self,
        stdout: &mut impl Write,
        value: &str,
        display_width: usize,
        alignment: Alignment,
        matches: &[(usize, usize)],
    ) -> io::Result<()> {
        let visual_width = value.width();
        let padding = display_width.saturating_sub(visual_width);
        let (left_pad, right_pad) = match alignment {
            Alignment::Left => (0, padding),
            Alignment::Right => (padding, 0),
            Alignment::Center => {
                let l = padding / 2;
                (l, padding - l)
            }
        };

        // Leading space + left padding (no highlight).
        write!(stdout, " ")?;
        if left_pad > 0 {
            write!(stdout, "{}", " ".repeat(left_pad))?;
        }

        // Value with highlighted match ranges.
        self.write_value_with_highlights(stdout, value, matches)?;

        // Right padding + trailing space.
        if right_pad > 0 {
            write!(stdout, "{}", " ".repeat(right_pad))?;
        }
        write!(stdout, " ")
    }

    /// Emit the cell value with matched byte-ranges wrapped in `SetAttribute(Reverse)`.
    ///
    /// Ranges are assumed sorted and non-overlapping (guaranteed by
    /// `find_all_matches`, which advances past each match).
    fn write_value_with_highlights(
        &self,
        stdout: &mut impl Write,
        value: &str,
        matches: &[(usize, usize)],
    ) -> io::Result<()> {
        let bytes = value.as_bytes();
        let mut cursor = 0usize;
        for &(start, end) in matches {
            if start < cursor || end > bytes.len() {
                // Out-of-range or overlapping — skip defensively.
                continue;
            }
            if start > cursor {
                // Non-matching prefix.
                stdout.write_all(&bytes[cursor..start])?;
            }
            execute!(stdout, SetAttribute(Attribute::Reverse))?;
            stdout.write_all(&bytes[start..end])?;
            // Use NoReverse (SGR 27) instead of Reset (SGR 0) so any
            // caller-set foreground color (e.g. DarkGrey on NULL cells)
            // survives the highlight toggle. Reset strips ALL attributes
            // including fg color.
            execute!(stdout, SetAttribute(Attribute::NoReverse))?;
            cursor = end;
        }
        if cursor < bytes.len() {
            stdout.write_all(&bytes[cursor..])?;
        }
        Ok(())
    }

    /// Render the status bar
    fn render_status_bar(&self, stdout: &mut impl Write) -> io::Result<()> {
        write!(stdout, "\r\n")?;
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        self.render_status_bar_to_buffer(stdout)?;
        execute!(stdout, ResetColor)?;
        Ok(())
    }

    /// Render the status bar content (no ANSI escapes) to a writer.
    ///
    /// Writer-injected variant used by unit tests to assert on exact byte
    /// output without needing a terminal. Output is terminated with `\r\n`
    /// to match the live render path; tests that care about line endings
    /// trim as needed.
    fn render_status_bar_to_buffer(&self, writer: &mut impl Write) -> io::Result<()> {
        // -- Search-prompt state takes priority: show the literal `/pattern`.
        if let InputMode::SearchPrompt { buffer } = &self.mode {
            // REQ-PAGER-SEARCH-001.1/.2: show `/` + typed chars, nothing else.
            // REQ-PAGER-SEARCH-001.3: empty buffer renders as `/` with
            // terminal cursor handling the blink position.
            return write!(writer, "/{}\r\n", buffer);
        }

        // -- Transient overlay (wrap notice) takes priority over match-count.
        if let Some(msg) = &self.transient_status {
            return write!(writer, "{}\r\n", msg);
        }

        // -- Search status (after a submitted pattern) --
        match self.search_status {
            SearchStatus::Matches => {
                if let Some(search) = &self.search {
                    let n = search.matches.len();
                    let noun = if n == 1 { "match" } else { "matches" };
                    return write!(
                        writer,
                        "Pattern: {}  ({} {})\r\n",
                        search.pattern, n, noun
                    );
                }
            }
            SearchStatus::NotFound => {
                if let Some(search) = &self.search {
                    return write!(writer, "Pattern: {}  not found\r\n", search.pattern);
                }
            }
            SearchStatus::None => {}
        }

        // -- Default status line --
        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        // Calculate progress percentage
        let progress = (end_row * 100)
            .checked_div(self.data.row_count)
            .unwrap_or(100);

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

        let mut nav_parts = Vec::new();
        if hidden_left > 0 || hidden_right > 0 {
            nav_parts.push("<- ->: scroll cols");
        }
        nav_parts.push("j/k Space/b: rows");
        nav_parts.push("g/G: first/last");
        nav_parts.push("/: search");
        nav_parts.push("?: help");
        nav_parts.push("q/Esc: exit");
        let nav_hints = nav_parts.join(" | ");

        write!(
            writer,
            "{} | {} | {} | {}\r\n",
            col_status, row_status, timing, nav_hints
        )
    }

    /// Handle navigation input
    /// Returns Ok(true) to continue paging, Ok(false) to exit pager
    fn handle_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        // REQ-PAGER-SEARCH: clear any one-render-cycle overlay status before
        // processing this event (e.g. the `wrapped to first match` notice).
        self.transient_status = None;

        // Split the state machine by input mode. Using `take` + rewrite keeps
        // the borrow discipline simple: we own `InputMode` for the duration of
        // the match and write back the new mode at the end.
        let current_mode = std::mem::replace(&mut self.mode, InputMode::Normal);
        match current_mode {
            InputMode::SearchPrompt { mut buffer } => {
                match key.code {
                    KeyCode::Enter => {
                        // REQ-PAGER-SEARCH-001.4: submit pattern.
                        // REQ-PAGER-SEARCH-001.6: empty pattern (or a pattern
                        // that reduces to empty after stripping the `\c`
                        // suffix, e.g. `\c` alone) is treated as cancel — no
                        // search performed, prior state untouched. This
                        // avoids rendering a visually broken
                        // `Pattern:   not found` with a blank pattern.
                        let (parsed, _) = parse_search_input(&buffer);
                        if !parsed.is_empty() {
                            self.submit_search(&buffer);
                        }
                        // Back to normal mode either way.
                        self.mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        // REQ-PAGER-SEARCH-001.5: cancel prompt, keep prior search.
                        self.mode = InputMode::Normal;
                    }
                    KeyCode::Backspace => {
                        // REQ-PAGER-SEARCH-001.3: if buffer becomes empty the
                        // prompt STAYS open showing `/`.
                        buffer.pop();
                        self.mode = InputMode::SearchPrompt { buffer };
                    }
                    KeyCode::Char(c) => {
                        buffer.push(c);
                        self.mode = InputMode::SearchPrompt { buffer };
                    }
                    _ => {
                        // Other keys (arrows, function keys) are ignored while
                        // the prompt is open.
                        self.mode = InputMode::SearchPrompt { buffer };
                    }
                }
                Ok(true)
            }
            InputMode::Normal => {
                // Remain in Normal mode unless a key explicitly enters another.
                self.mode = InputMode::Normal;
                match key.code {
                    // Exit pager
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(false),

                    // -- Search (REQ-PAGER-SEARCH-001, -004, -005) --
                    KeyCode::Char('/') => {
                        self.mode = InputMode::SearchPrompt {
                            buffer: String::new(),
                        };
                    }
                    KeyCode::Char('n') if self.search.is_some() => {
                        self.jump_match(Direction::Next);
                    }
                    KeyCode::Char('N') if self.search.is_some() => {
                        self.jump_match(Direction::Prev);
                    }

                    // Vertical navigation
                    KeyCode::Char('j') | KeyCode::Down
                        if self.row_offset + self.page_size < self.data.row_count =>
                    {
                        self.row_offset += 1;
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
                        self.row_offset =
                            self.data.row_count.saturating_sub(self.page_size);
                    }

                    // Horizontal navigation (column windowing)
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.col_offset = self.col_offset.saturating_sub(1);
                    }
                    KeyCode::Right | KeyCode::Char('l')
                        if self.col_offset + self.visible_column_count()
                            < self.data.columns.len() =>
                    {
                        self.col_offset += 1;
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
        }
    }

    /// Submit a `/pattern` from the search prompt.
    ///
    /// Parses the `\c` suffix, scans all cells, updates `self.search`, and
    /// scrolls to the initial match (first match at or after the current
    /// row_offset, per REQ-PAGER-SEARCH-002.1/.2).
    fn submit_search(&mut self, raw_input: &str) {
        let (pattern, case_sensitive) = parse_search_input(raw_input);
        let matches = find_all_matches(&self.data, &pattern, case_sensitive);
        let current = pick_initial_match(&matches, self.row_offset);

        if matches.is_empty() {
            // REQ-PAGER-SEARCH-003: no match. Retain viewport, display status.
            // REQ-PAGER-SEARCH-008.4: no highlights when no search is active
            // OR when the new search has zero matches — we keep the search
            // field populated (with empty matches) so the status bar shows
            // the `not found` state; highlight rendering checks `matches`
            // is non-empty.
            self.search = Some(SearchState {
                pattern,
                case_sensitive,
                matches,
                current: None,
            });
            self.search_status = SearchStatus::NotFound;
        } else {
            self.search_status = SearchStatus::Matches;
            self.search = Some(SearchState {
                pattern,
                case_sensitive,
                matches,
                current,
            });
            // Scroll to the initial match (we just set current, so unwrap is safe).
            if let Some(idx) = current {
                self.scroll_to_match_index(idx);
            }
        }
    }

    /// Move to the next or previous match, wrapping with a one-frame overlay.
    ///
    /// REQ-PAGER-SEARCH-004 / -005: wrap at end/start and show a transient
    /// `wrapped to first/last match` notice for the next render.
    fn jump_match(&mut self, dir: Direction) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }
        let total = search.matches.len();
        let current = search.current.unwrap_or(0);
        let (new_idx, wrapped_msg) = match dir {
            Direction::Next => {
                if current + 1 >= total {
                    (0, Some("wrapped to first match".to_string()))
                } else {
                    (current + 1, None)
                }
            }
            Direction::Prev => {
                if current == 0 {
                    (total - 1, Some("wrapped to last match".to_string()))
                } else {
                    (current - 1, None)
                }
            }
        };
        search.current = Some(new_idx);
        self.transient_status = wrapped_msg;
        self.scroll_to_match_index(new_idx);
    }

    /// Scroll the viewport so match `idx` is visible.
    ///
    /// Vertical: put the matched row at the top of the viewport, clamped so
    /// we don't scroll past the last page. Horizontal: if the matched column
    /// is outside `[col_offset, col_offset + visible_column_count)`, shift
    /// `col_offset` minimally so the column is visible (REQ-PAGER-SEARCH-007).
    fn scroll_to_match_index(&mut self, idx: usize) {
        let Some(search) = self.search.as_ref() else {
            return;
        };
        let Some(m) = search.matches.get(idx).copied() else {
            return;
        };

        // Vertical: prefer placing the match at the top of the viewport,
        // clamped to the last-page offset so we don't over-scroll.
        let max_row_offset = self.data.row_count.saturating_sub(self.page_size);
        self.row_offset = m.row.min(max_row_offset);

        // Horizontal: make the matched column visible.
        let visible = self.visible_column_count();
        if m.col < self.col_offset {
            // Scroll left so the matched column is the leftmost visible.
            self.col_offset = m.col;
        } else if m.col >= self.col_offset + visible {
            // Scroll right so the matched column is visible. Minimal shift:
            // align the matched column to the rightmost visible position.
            // Recompute visible after a trial shift because visible count is
            // col_offset dependent; we take a conservative fixed shift that
            // brings m.col into view and let the next render settle widths.
            let new_offset = m.col + 1 - visible.max(1);
            self.col_offset = new_offset;
        }
    }

    /// Help overlay text, authoritative source for both the live render and
    /// the unit-test writer-injected variant.
    ///
    /// The Search block placement (before `Exit:`) is REQ-PAGER-SEARCH-012.
    const HELP_TEXT: &'static str = r#"
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

Search:
  /pattern    Search forward for pattern (case-insensitive)
  /pattern\c  Search forward (case-sensitive)
  n           Next match
  N           Previous match
  Esc         Cancel prompt (keeps previous search)

Exit:
  q / Esc     Exit pager and return to REPL prompt

Press any key to return to results..."#;

    /// Write the help overlay text to a writer without ANSI escapes and
    /// without blocking on a keypress.
    ///
    /// Unit tests assert on the returned bytes for AC-12 (help documents
    /// `/pattern`, `n`, `N`, and `\c`). Line endings match the live render
    /// (`\r\n`) for string-equality parity.
    fn render_help_text(writer: &mut impl Write) -> io::Result<()> {
        let help_raw = Self::HELP_TEXT.replace('\n', "\r\n");
        write!(writer, "{}\r\n", help_raw)
    }

    /// Display help overlay showing all navigation keys
    fn show_help(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        execute!(stdout, SetForegroundColor(Color::Cyan))?;
        Self::render_help_text(&mut stdout)?;
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

    /// Render the current view to a string buffer for testing/debugging.
    ///
    /// Thin wrapper around the plain rendering methods. Writing to Vec<u8>
    /// is infallible, so unwrap() is safe here.
    #[cfg(test)]
    pub fn render_to_buffer(&self) -> String {
        let mut buffer = Vec::new();

        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);

        self.render_border_plain(&mut buffer, BorderType::Top).unwrap();
        self.render_header_plain(&mut buffer, self.col_offset, end_col).unwrap();
        self.render_border_plain(&mut buffer, BorderType::Middle).unwrap();
        for row_idx in self.row_offset..end_row {
            self.render_row_plain(&mut buffer, row_idx, self.col_offset, end_col).unwrap();
        }
        self.render_border_plain(&mut buffer, BorderType::Bottom).unwrap();

        String::from_utf8_lossy(&buffer).to_string()
    }

    /// Render a table border as plain text (no ANSI escapes)
    ///
    /// Writes to a generic writer for both production use (exit snapshot)
    /// and testing (Vec<u8> buffer).
    fn render_border_plain(&self, writer: &mut impl Write, border_type: BorderType) -> io::Result<()> {
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
        writeln!(writer, "{}", border)
    }

    /// Render header row as plain text (no ANSI escapes)
    ///
    /// Uses pad_to_display_width() for correct Unicode width handling.
    fn render_header_plain(&self, writer: &mut impl Write, start_col: usize, end_col: usize) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        let mut line = String::new();
        line.push('│');

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            let indicator = format!("(+{} cols)", hidden_left);
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            line.push_str(&padded);
            line.push('│');
        }

        // Data column headers - use display-width-aware padding
        for col in &self.data.columns[start_col..end_col] {
            let padded = pad_to_display_width(&col.name, col.display_width, Alignment::Center);
            line.push_str(&padded);
            line.push('│');
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            let indicator = format!("(+{} cols)", hidden_right);
            let padded = format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH);
            line.push_str(&padded);
            line.push('│');
        }

        writeln!(writer, "{}", line)
    }

    /// Render data row as plain text (no ANSI escapes)
    ///
    /// Uses pad_to_display_width() for correct Unicode width handling.
    fn render_row_plain(
        &self,
        writer: &mut impl Write,
        row_idx: usize,
        start_col: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        let mut line = String::new();
        line.push('│');

        // Left indicator cell (if columns hidden to left)
        if hidden_left > 0 {
            let indicator = "    <--   ";
            line.push_str(&format!(" {} ", indicator));
            line.push('│');
        }

        for (vis_idx, col) in self.data.columns[start_col..end_col].iter().enumerate() {
            let col_idx = start_col + vis_idx;
            let value = self.data.get_cell(row_idx, col_idx);

            let padded = pad_to_display_width(value, col.display_width, col.alignment);
            line.push_str(&padded);
            line.push('│');
        }

        // Right indicator cell (if columns hidden to right)
        if hidden_right > 0 {
            let indicator = "   -->    ";
            line.push_str(&format!(" {} ", indicator));
            line.push('│');
        }

        writeln!(writer, "{}", line)
    }

    /// Render a static snapshot of the current pager viewport as plain text.
    ///
    /// Called after exiting the pager (after LeaveAlternateScreen and disable_raw_mode)
    /// to leave a plain-text copy of the last viewed content on the user's terminal.
    /// Output has no ANSI escape codes, uses \n line endings, and includes:
    /// - Box-drawing table with the current viewport's rows and columns
    /// - Hidden columns footer (if any columns are off-screen)
    /// - Row count and timing footer
    pub fn render_exit_snapshot(&self, writer: &mut impl Write) -> io::Result<()> {
        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);

        // Render table
        self.render_border_plain(writer, BorderType::Top)?;
        self.render_header_plain(writer, self.col_offset, end_col)?;
        self.render_border_plain(writer, BorderType::Middle)?;
        for row_idx in self.row_offset..end_row {
            self.render_row_plain(writer, row_idx, self.col_offset, end_col)?;
        }
        self.render_border_plain(writer, BorderType::Bottom)?;

        // Hidden columns footer
        let mut hidden_names: Vec<&str> = Vec::new();
        // Columns hidden to the left (before current viewport)
        for col in &self.data.columns[..self.col_offset] {
            hidden_names.push(&col.name);
        }
        // Columns hidden to the right (after current viewport)
        for col in &self.data.columns[end_col..] {
            hidden_names.push(&col.name);
        }

        if !hidden_names.is_empty() {
            writeln!(writer)?;
            writeln!(
                writer,
                "{} columns hidden: {}",
                hidden_names.len(),
                hidden_names.join(", ")
            )?;
            writeln!(
                writer,
                "Use /format csv or /format json to see all columns"
            )?;
        }

        // Row count and timing footer
        writeln!(
            writer,
            "{} row(s) in set ({:.3}s)",
            self.total_rows,
            self.execution_time.as_secs_f64()
        )?;

        Ok(())
    }

    /// Calculate expected line width (for debugging/testing)
    #[cfg(test)]
    fn calculate_expected_line_width(&self) -> usize {
        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let hidden_left = self.hidden_columns_left();
        let hidden_right = self.hidden_columns_right();

        let mut width = 1; // Leading border

        if hidden_left > 0 {
            width += INDICATOR_WIDTH + 3; // " " + indicator + " " + "│"
        }

        for col in &self.data.columns[self.col_offset..end_col] {
            width += col.display_width + 3; // " " + content + " " + "│"
        }

        if hidden_right > 0 {
            width += INDICATOR_WIDTH + 3;
        }

        width
    }

    /// Run the pager event loop
    ///
    /// Sprint 33: Fixed bug where event::read() was called twice after single poll().
    /// After poll() returns true, there is only ONE event in the queue.
    pub fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;

        // Initial render
        self.render()?;

        // Event loop
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                // Sprint 33 Fix: Only call event::read() ONCE per poll() success.
                // Previous code had a bug where it called read() twice.
                match event::read()? {
                    Event::Key(key) => {
                        if !self.handle_key(key)? {
                            break;
                        }
                        self.render()?;
                    }
                    Event::Resize(w, h) => {
                        self.term_width = w as usize;
                        self.term_height = h as usize;
                        self.page_size = self.term_height.saturating_sub(5);
                        self.render()?;
                    }
                    _ => {
                        // Ignore other events (FocusGained, FocusLost, Mouse, Paste)
                    }
                }
            }
        }

        execute!(stdout, Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        // Print a static snapshot of the last viewport so the user can refer to it
        self.render_exit_snapshot(&mut stdout)?;

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

/// Estimate the rendered table width for a QueryResult.
///
/// Computes the width the table would occupy using the same column-width
/// logic as `TableData::from_query_result`. Used by auto-pager mode to
/// decide whether the result needs horizontal scrolling.
pub fn estimate_table_width(result: &QueryResult) -> usize {
    if result.columns.is_empty() {
        return 0;
    }

    // Leading border
    let mut width: usize = 1;

    for (col_idx, col_meta) in result.columns.iter().enumerate() {
        let header_width = col_meta.name.trim().width();
        let mut max_value_width = header_width;

        for row in &result.rows {
            let value = if col_idx < row.len() {
                row[col_idx].display()
            } else {
                "[NULL]".to_string()
            };
            max_value_width = max_value_width.max(value.trim().width());
        }

        let display_width = max_value_width.clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH);
        // Each column renders as: " " + content(display_width) + " " + "│"
        width += display_width + 3;
    }

    width
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
    use crossterm::event::KeyModifiers;
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

    // Sprint 31: Width validation tests using render_to_buffer

    /// Create a test result with wide columns to force horizontal scrolling
    fn create_wide_test_result(num_cols: usize, num_rows: usize) -> QueryResult {
        let columns: Vec<ColumnMetadata> = (0..num_cols)
            .map(|i| {
                ColumnMetadata::new(
                    format!("column_name_with_longer_text_{}", i),
                    TeradataType::Varchar,
                    true,
                )
            })
            .collect();

        let rows: Vec<Vec<Value>> = (0..num_rows)
            .map(|r| {
                (0..num_cols)
                    .map(|c| Value::String(format!("value_row{}_col{}_data", r, c)))
                    .collect()
            })
            .collect();

        QueryResult::new(columns, rows, Duration::from_millis(100))
    }

    /// Helper to create a pager with a specific terminal width
    fn create_pager_with_width(result: &QueryResult, term_width: usize) -> Pager {
        let config = PagerConfig::default();
        let mut pager = Pager::new(result, &config);
        pager.term_width = term_width;
        pager.page_size = 10;
        pager
    }

    /// Assert no line in the rendered output exceeds terminal width
    fn assert_no_overflow(rendered: &str, term_width: usize) {
        for (line_num, line) in rendered.lines().enumerate() {
            let line_width = UnicodeWidthStr::width(line);
            assert!(
                line_width <= term_width,
                "Line {} overflows: {} chars > {} terminal width.\nLine: '{}'",
                line_num + 1,
                line_width,
                term_width,
                line
            );
        }
    }

    #[test]
    fn test_render_width_80_columns() {
        let result = create_wide_test_result(20, 5);
        let pager = create_pager_with_width(&result, 80);

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 80);
        assert!(
            rendered.contains('│'),
            "Rendered output should contain table borders"
        );
    }

    #[test]
    fn test_render_width_117_columns() {
        // 117 is the user-reported problematic width
        let result = create_wide_test_result(20, 5);
        let pager = create_pager_with_width(&result, 117);

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 117);
    }

    #[test]
    fn test_render_width_120_columns() {
        let result = create_wide_test_result(20, 5);
        let pager = create_pager_with_width(&result, 120);

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 120);
    }

    #[test]
    fn test_render_width_160_columns() {
        let result = create_wide_test_result(20, 5);
        let pager = create_pager_with_width(&result, 160);

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 160);
    }

    #[test]
    fn test_expected_vs_actual_width() {
        let result = create_wide_test_result(10, 3);
        let pager = create_pager_with_width(&result, 100);

        let expected = pager.calculate_expected_line_width();
        let rendered = pager.render_to_buffer();

        // Check that actual matches expected for each line
        for (line_num, line) in rendered.lines().enumerate() {
            let actual_width = UnicodeWidthStr::width(line);
            // Allow some variance due to status bar, but data lines should match
            if line.starts_with('│') || line.starts_with('╭') || line.starts_with('├') || line.starts_with('╰') {
                assert!(
                    actual_width <= expected + 2, // Small tolerance for unicode char differences
                    "Line {}: actual width {} != expected width {}\nLine: '{}'",
                    line_num + 1,
                    actual_width,
                    expected,
                    line
                );
            }
        }
    }

    #[test]
    fn test_render_with_hidden_columns_left() {
        let result = create_wide_test_result(15, 3);
        let mut pager = create_pager_with_width(&result, 80);

        // Scroll right so there are hidden columns on the left
        pager.col_offset = 3;

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 80);
        // Should contain left indicator
        assert!(
            rendered.contains("<--"),
            "Should show left indicator when columns hidden to left"
        );
    }

    #[test]
    fn test_render_with_hidden_columns_right() {
        let result = create_wide_test_result(15, 3);
        let pager = create_pager_with_width(&result, 80);

        // At col_offset 0, should have columns hidden to the right
        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 80);
        // Should contain right indicator
        assert!(
            rendered.contains("-->"),
            "Should show right indicator when columns hidden to right"
        );
    }

    #[test]
    fn test_render_all_columns_visible() {
        // Small result that fits entirely
        let result = create_test_result(2, 2);
        let pager = create_pager_with_width(&result, 120);

        let rendered = pager.render_to_buffer();

        assert_no_overflow(&rendered, 120);
        // Should NOT contain indicators
        assert!(
            !rendered.contains("<--") && !rendered.contains("-->"),
            "Should not show indicators when all columns visible"
        );
    }

    #[test]
    fn test_narrow_terminal_minimum_one_column() {
        // Very narrow terminal
        let result = create_wide_test_result(10, 2);
        let pager = create_pager_with_width(&result, 30);

        // Should still be able to render at least one column
        let visible = pager.visible_column_count();
        assert!(visible >= 1, "Should always show at least one column");

        let rendered = pager.render_to_buffer();
        // Even if it overflows, it should render something
        assert!(!rendered.is_empty(), "Should render something");
    }

    #[test]
    fn test_render_debug_at_117_width() {
        // Debug test for the user-reported problematic width
        let result = create_wide_test_result(20, 3);
        let pager = create_pager_with_width(&result, 117);

        eprintln!("\n=== DEBUG: Pager at 117 char width ===");
        eprintln!("Terminal width: {}", pager.term_width);
        eprintln!("Visible columns: {}", pager.visible_column_count());
        eprintln!("Hidden left: {}", pager.hidden_columns_left());
        eprintln!("Hidden right: {}", pager.hidden_columns_right());
        eprintln!("Expected line width: {}", pager.calculate_expected_line_width());

        let rendered = pager.render_to_buffer();

        eprintln!("\n--- Rendered output (lines) ---");
        for (i, line) in rendered.lines().enumerate() {
            let width = UnicodeWidthStr::width(line);
            let status = if width <= 117 { "OK" } else { "OVERFLOW" };
            eprintln!("Line {}: {} chars [{}] '{}'", i + 1, width, status, line);
        }
        eprintln!("--- End rendered output ---\n");

        // Validate
        for (i, line) in rendered.lines().enumerate() {
            let width = UnicodeWidthStr::width(line);
            assert!(
                width <= 117,
                "Line {} overflow: {} > 117",
                i + 1,
                width
            );
        }
    }

    #[test]
    fn test_unicode_box_char_width() {
        // Verify box drawing characters have width 1
        assert_eq!(UnicodeWidthStr::width("│"), 1, "Vertical bar should be width 1");
        assert_eq!(UnicodeWidthStr::width("─"), 1, "Horizontal bar should be width 1");
        assert_eq!(UnicodeWidthStr::width("╭"), 1, "Top-left corner should be width 1");
        assert_eq!(UnicodeWidthStr::width("╮"), 1, "Top-right corner should be width 1");
        assert_eq!(UnicodeWidthStr::width("╰"), 1, "Bottom-left corner should be width 1");
        assert_eq!(UnicodeWidthStr::width("╯"), 1, "Bottom-right corner should be width 1");
        assert_eq!(UnicodeWidthStr::width("┬"), 1, "T-down should be width 1");
        assert_eq!(UnicodeWidthStr::width("┴"), 1, "T-up should be width 1");
        assert_eq!(UnicodeWidthStr::width("├"), 1, "T-right should be width 1");
        assert_eq!(UnicodeWidthStr::width("┤"), 1, "T-left should be width 1");
        assert_eq!(UnicodeWidthStr::width("┼"), 1, "Cross should be width 1");
    }

    #[test]
    fn test_cell_value_exceeds_display_width() {
        // This test exposes the bug: cell values may be wider than display_width
        // because truncation happens at MAX_CELL_LENGTH (100), not display_width (40)
        let long_value = "This is a value that is longer than MAX_COLUMN_WIDTH of forty characters";
        let columns = vec![ColumnMetadata::new("col", TeradataType::Varchar, true)];
        let rows = vec![vec![Value::String(long_value.to_string())]];
        let result = QueryResult::new(columns, rows, Duration::from_millis(50));

        let data = TableData::from_query_result(&result, MAX_COLUMN_WIDTH);

        // The display_width should be capped at MAX_COLUMN_WIDTH
        assert_eq!(data.columns[0].display_width, MAX_COLUMN_WIDTH);

        // The cell value should be truncated to fit within display_width
        let cell_value = data.get_cell(0, 0);
        let cell_width = UnicodeWidthStr::width(cell_value);

        // THIS ASSERTION CATCHES THE BUG:
        // Cell value must be <= display_width, otherwise format! will expand
        assert!(
            cell_width <= MAX_COLUMN_WIDTH,
            "Cell value width {} exceeds display_width {}. Value: '{}'",
            cell_width,
            MAX_COLUMN_WIDTH,
            cell_value
        );
    }

    // Sprint 63: Pager Exit Snapshot tests

    /// Helper: render exit snapshot to a String
    fn snapshot_to_string(pager: &Pager) -> String {
        let mut buffer = Vec::new();
        pager.render_exit_snapshot(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn test_exit_snapshot_basic_all_columns_visible() {
        let result = create_test_result(3, 5);
        let pager = create_pager_with_width(&result, 120);

        let output = snapshot_to_string(&pager);

        // Should contain table borders
        assert!(output.contains('╭'), "Should have top border");
        assert!(output.contains('╰'), "Should have bottom border");
        assert!(output.contains('│'), "Should have column separators");

        // Should contain all column headers
        assert!(output.contains("col0"), "Should contain col0 header");
        assert!(output.contains("col1"), "Should contain col1 header");
        assert!(output.contains("col2"), "Should contain col2 header");

        // Should contain data values
        assert!(output.contains("val_0_0"), "Should contain first cell");
        assert!(output.contains("val_4_2"), "Should contain last cell");

        // Should NOT contain hidden columns footer (all visible)
        assert!(
            !output.contains("columns hidden"),
            "Should not show hidden columns when all are visible"
        );

        // Should contain row count and timing
        assert!(
            output.contains("5 row(s) in set (0.100s)"),
            "Should show total row count and timing"
        );
    }

    #[test]
    fn test_exit_snapshot_horizontal_scroll() {
        let result = create_wide_test_result(15, 3);
        let mut pager = create_pager_with_width(&result, 80);
        pager.col_offset = 3;

        let output = snapshot_to_string(&pager);

        // Should NOT contain columns 0-2 in the table data
        // (they are hidden to the left)
        assert!(
            output.contains("<--"),
            "Should show left scroll indicator"
        );

        // Should contain hidden columns footer listing hidden column names
        assert!(
            output.contains("columns hidden:"),
            "Should list hidden columns"
        );

        // The hidden names should include the first 3 columns
        assert!(
            output.contains("column_name_with_longer_text_0"),
            "Should list col 0 as hidden"
        );
        assert!(
            output.contains("column_name_with_longer_text_1"),
            "Should list col 1 as hidden"
        );
        assert!(
            output.contains("column_name_with_longer_text_2"),
            "Should list col 2 as hidden"
        );

        // Should contain the hint
        assert!(
            output.contains("Use /format csv or /format json to see all columns"),
            "Should show format hint"
        );
    }

    #[test]
    fn test_exit_snapshot_vertical_scroll() {
        let result = create_test_result(3, 20);
        let mut pager = create_pager_with_width(&result, 120);
        pager.page_size = 5;
        pager.row_offset = 10;

        let output = snapshot_to_string(&pager);

        // Should contain rows 10-14
        assert!(output.contains("val_10_0"), "Should contain row 10");
        assert!(output.contains("val_14_0"), "Should contain row 14");

        // Should NOT contain rows outside the viewport
        assert!(!output.contains("val_0_0"), "Should not contain row 0");
        assert!(!output.contains("val_9_0"), "Should not contain row 9");
        assert!(!output.contains("val_15_0"), "Should not contain row 15");

        // Should show total row count (not just visible rows)
        assert!(
            output.contains("20 row(s) in set"),
            "Should show total row count, not just visible"
        );
    }

    #[test]
    fn test_exit_snapshot_both_offsets() {
        let result = create_wide_test_result(15, 20);
        let mut pager = create_pager_with_width(&result, 80);
        pager.col_offset = 2;
        pager.row_offset = 5;
        pager.page_size = 3;

        let output = snapshot_to_string(&pager);

        // Should show rows 5-7 only
        assert!(output.contains("value_row5_col"), "Should contain row 5 data");
        assert!(output.contains("value_row7_col"), "Should contain row 7 data");
        assert!(!output.contains("value_row4_col"), "Should not contain row 4");
        assert!(!output.contains("value_row8_col"), "Should not contain row 8");

        // Should have hidden columns
        assert!(
            output.contains("columns hidden:"),
            "Should show hidden columns footer"
        );

        // Total row count
        assert!(
            output.contains("20 row(s) in set"),
            "Should show total row count"
        );
    }

    #[test]
    fn test_exit_snapshot_no_ansi_escapes() {
        let result = create_test_result(3, 5);
        let pager = create_pager_with_width(&result, 120);

        let mut buffer = Vec::new();
        pager.render_exit_snapshot(&mut buffer).unwrap();

        // ANSI escape sequences start with 0x1B (ESC)
        assert!(
            !buffer.contains(&0x1B),
            "Snapshot output must not contain ANSI escape codes"
        );
    }

    #[test]
    fn test_exit_snapshot_uses_newline_not_crlf() {
        let result = create_test_result(3, 5);
        let pager = create_pager_with_width(&result, 120);

        let output = snapshot_to_string(&pager);

        assert!(
            !output.contains("\r\n"),
            "Snapshot must use \\n line endings, not \\r\\n"
        );
        assert!(
            output.contains('\n'),
            "Snapshot must contain newlines"
        );
    }

    #[test]
    fn test_exit_snapshot_hidden_columns_in_schema_order() {
        // Create a result where we scroll to the middle, hiding columns on both sides
        let result = create_wide_test_result(10, 2);
        let mut pager = create_pager_with_width(&result, 80);
        pager.col_offset = 3;

        let output = snapshot_to_string(&pager);

        // Find the hidden columns line
        let hidden_line = output
            .lines()
            .find(|l| l.contains("columns hidden:"))
            .expect("Should have hidden columns line");

        // Columns 0, 1, 2 are hidden left; some columns are hidden right
        // All should appear in schema order (0, 1, 2 first, then right-hidden)
        let col0_pos = hidden_line.find("column_name_with_longer_text_0").unwrap();
        let col1_pos = hidden_line.find("column_name_with_longer_text_1").unwrap();
        let col2_pos = hidden_line.find("column_name_with_longer_text_2").unwrap();

        assert!(col0_pos < col1_pos, "col0 should appear before col1");
        assert!(col1_pos < col2_pos, "col1 should appear before col2");
    }

    #[test]
    fn test_exit_snapshot_timing_format() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
        ];
        let rows = vec![vec![Value::Integer(42)]];
        let result = QueryResult::new(columns, rows, Duration::from_millis(1234));

        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;
        pager.page_size = 10;

        let output = snapshot_to_string(&pager);

        assert!(
            output.contains("1 row(s) in set (1.234s)"),
            "Timing should use 3 decimal places. Got: {}",
            output
        );
    }

    #[test]
    fn test_exit_snapshot_box_drawing_characters() {
        let result = create_test_result(2, 2);
        let pager = create_pager_with_width(&result, 120);

        let output = snapshot_to_string(&pager);

        // Should use the same box-drawing characters as the pager
        assert!(output.contains('╭'), "Top-left corner");
        assert!(output.contains('╮'), "Top-right corner");
        assert!(output.contains('╰'), "Bottom-left corner");
        assert!(output.contains('╯'), "Bottom-right corner");
        assert!(output.contains('┬'), "Top T-junction");
        assert!(output.contains('┴'), "Bottom T-junction");
        assert!(output.contains('├'), "Left T-junction");
        assert!(output.contains('┤'), "Right T-junction");
        assert!(output.contains('┼'), "Cross junction");
        assert!(output.contains('─'), "Horizontal line");
        assert!(output.contains('│'), "Vertical line");
    }

    // ---------------------------------------------------------------------------
    // Sprint 67: Pager Search tests (REQ-PAGER-SEARCH-001..012)
    // ---------------------------------------------------------------------------

    /// Build a TableData with explicit cell contents, bypassing column-width
    /// calculation so we can drive `find_all_matches` with known inputs.
    fn make_table_data(cells: Vec<Vec<&str>>) -> TableData {
        let cols_count = cells.first().map(|r| r.len()).unwrap_or(0);
        let columns: Vec<ColumnInfo> = (0..cols_count)
            .map(|i| ColumnInfo {
                name: format!("c{}", i),
                display_width: 40,
                alignment: Alignment::Left,
            })
            .collect();
        let cell_values: Vec<Vec<String>> = cells
            .into_iter()
            .map(|row| row.into_iter().map(|s| s.to_string()).collect())
            .collect();
        let row_count = cell_values.len();
        TableData {
            columns,
            cell_values,
            row_count,
        }
    }

    // -- parse_search_input --

    #[test]
    fn parse_search_empty_input() {
        assert_eq!(parse_search_input(""), (String::new(), false));
    }

    #[test]
    fn parse_search_no_suffix() {
        assert_eq!(parse_search_input("foo"), ("foo".to_string(), false));
    }

    #[test]
    fn parse_search_c_suffix_strips() {
        // REQ-PAGER-SEARCH-006.2/.3: `\c` suffix => case-sensitive, stripped.
        assert_eq!(parse_search_input("foo\\c"), ("foo".to_string(), true));
    }

    #[test]
    fn parse_search_just_c() {
        // Locked decision 3: `"\c"` -> `("", true)`, NOT special-cased.
        assert_eq!(parse_search_input("\\c"), (String::new(), true));
    }

    #[test]
    fn parse_search_double_c_only_strips_outer() {
        // `foo\c\c` -> outer `\c` is the suffix marker; inner `\c` stays.
        // So result is ("foo\\c", true).
        assert_eq!(
            parse_search_input("foo\\c\\c"),
            ("foo\\c".to_string(), true)
        );
    }

    #[test]
    fn parse_search_multibyte_safe() {
        // A pattern ending in a multi-byte char (no `\c` suffix) must not
        // panic or split a codepoint.
        assert_eq!(
            parse_search_input("café"),
            ("café".to_string(), false)
        );
    }

    // -- find_all_matches --

    #[test]
    fn find_matches_case_insensitive_multi_cell() {
        let data = make_table_data(vec![
            vec!["Hello", "WORLD", "[NULL]"],
            vec!["hello world", "foo", "42"],
        ]);
        let matches = find_all_matches(&data, "hello", false);
        // Row 0 col 0: "Hello" -> 1 match
        // Row 1 col 0: "hello world" -> 1 match at start
        assert_eq!(matches.len(), 2, "Expected 2 hello matches, got {:?}", matches);
        assert_eq!(matches[0].row, 0);
        assert_eq!(matches[0].col, 0);
        assert_eq!(matches[1].row, 1);
        assert_eq!(matches[1].col, 0);
    }

    #[test]
    fn find_matches_case_sensitive() {
        let data = make_table_data(vec![
            vec!["Hello", "hello", "HELLO"],
        ]);
        // Case-sensitive "hello" matches only the middle cell
        let matches = find_all_matches(&data, "hello", true);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].col, 1);
    }

    #[test]
    fn find_matches_no_match() {
        let data = make_table_data(vec![vec!["Hello", "World"]]);
        let matches = find_all_matches(&data, "xyzzy", false);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_matches_empty_pattern() {
        // Locked decision 3: empty pattern yields zero matches, NOT special-cased.
        let data = make_table_data(vec![vec!["Hello", "World"]]);
        let matches = find_all_matches(&data, "", false);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_matches_multi_per_cell() {
        // Pattern "o" must match both Hello (1x) and world (1x) in row 1.
        let data = make_table_data(vec![
            vec!["Hello", "WORLD"],
            vec!["hello world", "foo"],
        ]);
        let matches = find_all_matches(&data, "o", false);
        // Hello: 1 'o'; WORLD: 1 'O'; "hello world": 2 'o's; "foo": 2 'o's = 6
        assert_eq!(matches.len(), 6, "Got: {:?}", matches);
        // Verify matches are sorted by (row, col, byte_start)
        for w in matches.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(
                (a.row, a.col, a.byte_start) <= (b.row, b.col, b.byte_start),
                "Matches should be sorted: {:?} before {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn find_matches_byte_ranges_exact() {
        let data = make_table_data(vec![vec!["hello world"]]);
        let matches = find_all_matches(&data, "world", false);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].byte_start, 6);
        assert_eq!(matches[0].byte_end, 11);
    }

    // -- pick_initial_match --

    #[test]
    fn pick_initial_empty_list() {
        assert_eq!(pick_initial_match(&[], 0), None);
    }

    #[test]
    fn pick_initial_cursor_at_zero() {
        let ms = vec![
            Match { row: 0, col: 0, byte_start: 0, byte_end: 1 },
            Match { row: 2, col: 0, byte_start: 0, byte_end: 1 },
        ];
        // Cursor at row 0 -> first match (at or after row 0).
        assert_eq!(pick_initial_match(&ms, 0), Some(0));
    }

    #[test]
    fn pick_initial_cursor_in_middle() {
        let ms = vec![
            Match { row: 0, col: 0, byte_start: 0, byte_end: 1 },
            Match { row: 5, col: 0, byte_start: 0, byte_end: 1 },
            Match { row: 10, col: 0, byte_start: 0, byte_end: 1 },
        ];
        // Cursor at row 3 -> first match with row >= 3 is row 5 (index 1).
        assert_eq!(pick_initial_match(&ms, 3), Some(1));
    }

    #[test]
    fn pick_initial_cursor_past_all() {
        let ms = vec![
            Match { row: 0, col: 0, byte_start: 0, byte_end: 1 },
            Match { row: 5, col: 0, byte_start: 0, byte_end: 1 },
        ];
        // Cursor past last match -> wrap to first match.
        assert_eq!(pick_initial_match(&ms, 100), Some(0));
    }

    #[test]
    fn pick_initial_cursor_exact_match_row() {
        let ms = vec![
            Match { row: 5, col: 0, byte_start: 0, byte_end: 1 },
            Match { row: 10, col: 0, byte_start: 0, byte_end: 1 },
        ];
        // Cursor at row 5 -> returns the match at row 5 (not 10).
        assert_eq!(pick_initial_match(&ms, 5), Some(0));
    }

    // -- Status-bar rendering (writer-injected, AC-9) --

    fn make_pager_with_data(rows: usize, cols: usize) -> Pager {
        let result = create_test_result(cols, rows);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;
        pager.page_size = 10;
        pager
    }

    fn status_bar_to_string(pager: &Pager) -> String {
        let mut buf: Vec<u8> = Vec::new();
        pager.render_status_bar_to_buffer(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn status_bar_default_no_search() {
        let pager = make_pager_with_data(5, 3);
        let out = status_bar_to_string(&pager);
        // Default status line includes Columns/Rows/timing/nav hints.
        assert!(out.contains("Columns 1-"), "Should show column status: {}", out);
        assert!(out.contains("Rows 1-"), "Should show row status: {}", out);
        assert!(out.contains("/: search"), "Nav hints should advertise `/`: {}", out);
    }

    #[test]
    fn status_bar_search_prompt_open_empty_buffer() {
        let mut pager = make_pager_with_data(5, 3);
        pager.mode = InputMode::SearchPrompt {
            buffer: String::new(),
        };
        let out = status_bar_to_string(&pager);
        // Locked decision 2: empty buffer renders literal `/` only.
        assert!(out.starts_with('/'), "Should start with /: {:?}", out);
        // Should NOT include any placeholder or navigation hints.
        assert!(!out.contains("Columns"), "No column hints while prompt open");
    }

    #[test]
    fn status_bar_search_prompt_with_buffer() {
        let mut pager = make_pager_with_data(5, 3);
        pager.mode = InputMode::SearchPrompt {
            buffer: "foo".to_string(),
        };
        let out = status_bar_to_string(&pager);
        assert!(out.starts_with("/foo"), "Should echo typed chars: {:?}", out);
    }

    #[test]
    fn status_bar_matches_format_exact() {
        // REQ-PAGER-SEARCH-009.1: `Pattern: <pat>  (M matches)` with two spaces.
        // Use a pattern that hits multiple cells so plural `matches` is used.
        let mut pager = make_pager_with_data(5, 3);
        pager.submit_search("val_");
        let out = status_bar_to_string(&pager);
        // Must contain literal double space between pattern and `(`.
        assert!(
            out.contains("Pattern: val_  ("),
            "Expected two spaces before `(`: {:?}",
            out
        );
        assert!(out.contains("matches)"), "Expected plural `matches)`: {:?}", out);
    }

    #[test]
    fn status_bar_singular_match_uses_match_not_matches() {
        // REQ-PAGER-SEARCH-009.1: `(1 match)` (singular) vs `(N matches)` (plural).
        // A single hit must NOT read `(1 matches)`.
        let mut pager = make_pager_with_data(5, 3);
        pager.submit_search("val_2_1");
        let out = status_bar_to_string(&pager);
        assert!(
            out.contains("(1 match)"),
            "Singular match should read `(1 match)`, not `(1 matches)`: {:?}",
            out
        );
        assert!(
            !out.contains("(1 matches)"),
            "Singular must not read `(1 matches)`: {:?}",
            out
        );
    }

    #[test]
    fn status_bar_not_found_format_exact() {
        // REQ-PAGER-SEARCH-003.3: `Pattern: <pat>  not found` with two spaces.
        let mut pager = make_pager_with_data(5, 3);
        pager.submit_search("xyzzy_absolutely_not_in_data");
        let out = status_bar_to_string(&pager);
        assert!(
            out.contains("Pattern: xyzzy_absolutely_not_in_data  not found"),
            "Expected verbatim not-found format: {:?}",
            out
        );
    }

    #[test]
    fn status_bar_empty_pattern_submit_search_renders_not_found() {
        // Defensive low-level behavior: if `submit_search` is called
        // directly with a pattern that strips to empty (e.g. `\c`), the
        // status shows `Pattern:   not found`. The ENTER handler in
        // `handle_key` guards against this user-facing case by parsing
        // the buffer first and cancelling when the post-parse pattern
        // is empty — see `enter_on_c_only_buffer_cancels_instead_of_submitting`.
        let mut pager = make_pager_with_data(5, 3);
        pager.submit_search("\\c");
        let out = status_bar_to_string(&pager);
        assert!(
            out.contains("Pattern:   not found"),
            "Expected defensive `Pattern:   not found` at submit_search: {:?}",
            out
        );
    }

    #[test]
    fn enter_on_c_only_buffer_cancels_instead_of_submitting() {
        // REQ-PAGER-SEARCH-001.6 extended: ENTER with a buffer that
        // reduces to empty after stripping `\c` is treated as cancel,
        // NOT submitted. Otherwise the status bar would render a
        // broken-looking `Pattern:   not found` with a blank pattern.
        let mut pager = make_pager_with_data(5, 3);
        pager.mode = InputMode::SearchPrompt {
            buffer: "\\c".to_string(),
        };
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let cont = pager.handle_key(key).expect("handle_key should succeed");
        assert!(cont, "Pager should continue running after cancel");
        assert!(
            matches!(pager.mode, InputMode::Normal),
            "Mode should be Normal after cancel"
        );
        assert!(
            pager.search.is_none(),
            "No search state should be created on cancel"
        );
    }

    #[test]
    fn status_bar_transient_wrap_notice() {
        let mut pager = make_pager_with_data(5, 3);
        pager.transient_status = Some("wrapped to first match".to_string());
        let out = status_bar_to_string(&pager);
        assert!(
            out.starts_with("wrapped to first match"),
            "Transient status takes priority: {:?}",
            out
        );
    }

    // -- Help text (writer-injected, AC-12) --

    #[test]
    fn help_text_documents_search_keys() {
        // AC-12: `/pattern`, `n`, `N`, and `\c` must all appear in help.
        let mut buf: Vec<u8> = Vec::new();
        Pager::render_help_text(&mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("Search:"), "Help has Search section: {}", out);
        assert!(out.contains("/pattern"), "Help documents /pattern");
        assert!(
            out.contains("case-insensitive"),
            "Help explains case-insensitive default"
        );
        assert!(out.contains("\\c"), "Help mentions \\c suffix");
        assert!(
            out.contains("Next match"),
            "Help documents `n` Next match"
        );
        assert!(
            out.contains("Previous match"),
            "Help documents `N` Previous match"
        );
    }

    #[test]
    fn help_text_search_before_exit() {
        // Locked decision 7: Search block is BEFORE Exit block.
        let mut buf: Vec<u8> = Vec::new();
        Pager::render_help_text(&mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let search_pos = out.find("Search:").expect("has Search:");
        let exit_pos = out.find("Exit:").expect("has Exit:");
        assert!(
            search_pos < exit_pos,
            "Search section must appear before Exit section"
        );
    }

    // -- Integration: submit_search scrolls correctly --

    #[test]
    fn submit_search_scrolls_to_first_match() {
        // Create a result with a known value in row 3.
        let cols = vec![ColumnMetadata::new("id", TeradataType::Integer, false)];
        let rows: Vec<Vec<Value>> = (0..10)
            .map(|r| vec![Value::Integer(r as i64)])
            .collect();
        let result = QueryResult::new(cols, rows, Duration::from_millis(10));
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;
        pager.page_size = 3;

        // Search for "5" -> matches row 5 exactly.
        pager.submit_search("5");
        assert_eq!(pager.row_offset, 5, "Should scroll to row 5");
        assert_eq!(pager.search_status, SearchStatus::Matches);
    }

    #[test]
    fn submit_search_no_match_does_not_scroll() {
        let mut pager = make_pager_with_data(10, 3);
        pager.row_offset = 2;
        pager.submit_search("absolutely_not_present_12345");
        assert_eq!(pager.row_offset, 2, "REQ-PAGER-SEARCH-003.1: no scroll on no match");
        assert_eq!(pager.search_status, SearchStatus::NotFound);
    }

    #[test]
    fn jump_match_next_wraps() {
        // Result with 3 matches -> pressing n three times wraps.
        let cols = vec![ColumnMetadata::new("label", TeradataType::Varchar, true)];
        let rows: Vec<Vec<Value>> = (0..5)
            .map(|r| {
                vec![Value::String(if r % 2 == 0 {
                    "hit".to_string()
                } else {
                    "miss".to_string()
                })]
            })
            .collect();
        let result = QueryResult::new(cols, rows, Duration::from_millis(10));
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;
        pager.page_size = 2;

        pager.submit_search("hit"); // 3 matches (rows 0, 2, 4)
        assert_eq!(pager.search.as_ref().unwrap().matches.len(), 3);
        assert_eq!(pager.search.as_ref().unwrap().current, Some(0));

        pager.jump_match(Direction::Next);
        assert_eq!(pager.search.as_ref().unwrap().current, Some(1));
        assert!(pager.transient_status.is_none());

        pager.jump_match(Direction::Next);
        assert_eq!(pager.search.as_ref().unwrap().current, Some(2));
        assert!(pager.transient_status.is_none());

        // Third next -> wraps to 0.
        pager.jump_match(Direction::Next);
        assert_eq!(pager.search.as_ref().unwrap().current, Some(0));
        assert_eq!(
            pager.transient_status.as_deref(),
            Some("wrapped to first match")
        );
    }

    #[test]
    fn jump_match_prev_wraps() {
        let cols = vec![ColumnMetadata::new("label", TeradataType::Varchar, true)];
        let rows: Vec<Vec<Value>> = (0..3)
            .map(|_| vec![Value::String("hit".to_string())])
            .collect();
        let result = QueryResult::new(cols, rows, Duration::from_millis(10));
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;
        pager.page_size = 1;

        pager.submit_search("hit"); // 3 matches, current = 0
        pager.jump_match(Direction::Prev);
        // Wraps to last index (2).
        assert_eq!(pager.search.as_ref().unwrap().current, Some(2));
        assert_eq!(
            pager.transient_status.as_deref(),
            Some("wrapped to last match")
        );
    }

    // ---------------------------------------------------------------------------
    // Sprint 67: bytes_eq_ignore_ascii_case
    // ---------------------------------------------------------------------------

    #[test]
    fn bytes_eq_ignore_case_basic() {
        assert!(bytes_eq_ignore_ascii_case(b"Hello", b"HELLO"));
        assert!(bytes_eq_ignore_ascii_case(b"foo", b"FOO"));
        assert!(!bytes_eq_ignore_ascii_case(b"foo", b"bar"));
    }

    #[test]
    fn bytes_eq_ignore_case_length_mismatch() {
        assert!(!bytes_eq_ignore_ascii_case(b"foo", b"foos"));
        assert!(!bytes_eq_ignore_ascii_case(b"foos", b"foo"));
    }
}
