# REPL Mode Design

This document explains the interactive Read-Eval-Print Loop implementation in tq.

**Related Specification**: `docs/specifications/repl.md`

## Overview

The REPL provides an interactive SQL environment with command history, syntax highlighting, tab completion, and intelligent result paging.

## Architecture

```
REPL Loop:

Initialize → Connect → Setup Editor → Show Prompt
                                          ↓
                                      Read Input
                                          ↓
                                  ┌───────┴───────┐
                                  ↓               ↓
                            SQL Statement    Metacommand
                                  ↓               ↓
                              Execute         Process
                                  ↓               ↓
                            Format Result    Update State
                                  ↓               ↓
                              Page Output        ↓
                                  ↓               ↓
                              Show Prompt ←───────┘
```

## Module Structure

```
src/commands/repl/
├── mod.rs              # REPL orchestration
├── executor.rs         # Statement execution
├── metadata_completer.rs  # Tab completion
├── highlighter.rs      # Syntax highlighting
├── pager.rs            # Result paging
└── state.rs            # Session state
```

## Core Components

### REPL State

```rust
// src/commands/repl/state.rs

pub struct ReplState {
    pub connection: Connection,
    pub config: FormatOptions,
    pub timing: bool,
    pub pager_enabled: bool,
    pub metadata_cache: MetadataCache,
}

pub struct MetadataCache {
    pub databases: Vec<String>,
    pub tables: HashMap<String, Vec<TableInfo>>,
    pub columns: HashMap<String, Vec<ColumnInfo>>,
    pub last_refresh: Instant,
}
```

### Editor Setup

Uses `reedline` for line editing:

```rust
let mut editor = Reedline::create()
    .with_history(Box::new(
        FileBackedHistory::with_file(100, history_path)?
    ))
    .with_completer(Box::new(
        MetadataCompleter::new(state.metadata_cache.clone())
    ))
    .with_highlighter(Box::new(
        SqlHighlighter::new()
    ))
    .with_validator(Box::new(
        StatementValidator::new()
    ));
```

### Tab Completion

Context-aware suggestions:

```rust
// src/commands/repl/metadata_completer.rs

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let context = parse_completion_context(line, pos);

        match context {
            CompletionContext::AfterFrom | CompletionContext::AfterJoin => {
                // Suggest table names
                self.cache.tables.keys()
                    .map(|t| Suggestion::new(t.clone()))
                    .collect()
            }
            CompletionContext::AfterSelect => {
                // Suggest column names from tables in FROM clause
                let tables = extract_tables_from_query(line);
                tables.iter()
                    .flat_map(|t| self.cache.columns.get(t))
                    .flatten()
                    .map(|c| Suggestion::new(c.name.clone()))
                    .collect()
            }
            CompletionContext::Keyword => {
                // Suggest SQL keywords
                SQL_KEYWORDS.iter()
                    .map(|k| Suggestion::new(k.to_string()))
                    .collect()
            }
            _ => vec![],
        }
    }
}
```

### Syntax Highlighting

```rust
// src/commands/repl/highlighter.rs

impl Highlighter for SqlHighlighter {
    fn highlight(&self, line: &str) -> StyledText {
        let mut styled = StyledText::new();

        for token in tokenize_sql(line) {
            let style = match token.kind {
                TokenKind::Keyword => Style::new().fg(Color::Blue).bold(),
                TokenKind::String => Style::new().fg(Color::Green),
                TokenKind::Number => Style::new().fg(Color::Cyan),
                TokenKind::Comment => Style::new().fg(Color::DarkGray),
                TokenKind::Function => Style::new().fg(Color::Yellow),
                _ => Style::new(),
            };

            styled.push((style, token.text));
        }

        styled
    }
}
```

### Result Paging

```rust
// src/commands/repl/pager.rs

pub fn display_with_pager(output: &str, rows: usize) -> Result<()> {
    if should_page(rows) {
        let mut pager = Pager::new()?;
        pager.set_text(output)?;
        pager.run()?;
    } else {
        println!("{}", output);
    }
    Ok(())
}

fn should_page(rows: usize) -> bool {
    if let Some((_, height)) = terminal::size().ok() {
        rows as u16 > height - 5  // Leave room for prompt
    } else {
        false
    }
}
```

Uses `crossterm` crate for terminal control with vi-like navigation keys.

### Horizontal Paging

This section documents the technical design for interactive horizontal paging, enabling users to navigate wide result sets that exceed terminal width.

#### Overview

Horizontal paging extends the existing pager infrastructure to provide column-by-column navigation through wide tables. The design integrates seamlessly with vertical paging, allowing users to navigate both dimensions independently.

**Related Specification**: `docs/specifications/repl.md#large-result-handling--result-paging`

#### Architecture (Sprint 30 Refactor)

**IMPORTANT**: Sprint 30 refactored the pager architecture to fix a fundamental bug where pre-formatted table strings (1221+ chars wide) caused garbled output on narrow terminals (117 chars).

**Previous Architecture (Sprint 29 - BROKEN)**:
```
Executor -> Format ALL columns to string -> Pager parses string -> Wrapped/garbled output
```

**New Architecture (Sprint 30 - FIXED)**:
```
Executor -> QueryResult -> Pager calculates widths at render time -> Clean output
```

The pager now accepts `QueryResult` directly instead of pre-formatted strings. Column widths are calculated at render time based on actual terminal dimensions, ensuring output never exceeds terminal width.

```
Horizontal Paging Flow (Sprint 30):

Query Result (N columns, M rows)
        |
        v
Pager Activation Check (should_page)
        |
    +---+---+
    |       |
    v       v
Pager ON   Pager OFF
    |       |
    v       v
Pager::new(&result)  Direct Table Format
    |                (width-constrained)
    v
TableData::from_query_result()
    |
    v
Calculate Visible Columns (term_width aware)
    |
    v
Interactive Event Loop:
    +--> Render visible window (col_offset to col_offset+visible)
    |    (formats from structured data, respects terminal width)
    |        |
    |        v
    |    Key Event
    |        |
    |    +---+---+
    |    |   |   |
    |    v   v   v
    |   h/l  j/k  q
    |   |    |    |
    |   v    v    v
    |  col   row  Exit
    +---+----+
```

#### TableData Construction (Sprint 30)

Sprint 30 introduced `TableData::from_query_result()` which builds structured table data directly from `QueryResult`, eliminating the need to parse pre-formatted strings.

```rust
// src/commands/repl/pager.rs

/// Create TableData directly from QueryResult
///
/// Sprint 30: This method calculates column widths from actual data
/// at construction time, enabling proper width management at render time.
pub fn from_query_result(result: &QueryResult, max_col_width: usize) -> Self {
    let mut columns = Vec::with_capacity(result.columns.len());
    let mut cell_values: Vec<Vec<String>> = vec![Vec::new(); result.rows.len()];

    for (col_idx, col_meta) in result.columns.iter().enumerate() {
        // Truncate header if needed
        let header = truncate_cell(&col_meta.name, max_col_width.saturating_sub(2));
        let mut max_value_width = header.width();

        for (row_idx, row) in result.rows.iter().enumerate() {
            let value = row[col_idx].display();
            let truncated = truncate_cell(&value, MAX_CELL_LENGTH);
            max_value_width = max_value_width.max(truncated.width());
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

    TableData { columns, cell_values, row_count: result.rows.len() }
}
```

#### Column Windowing Algorithm

The pager calculates how many columns can fit in the terminal width at any given time, accounting for column position indicators when columns are hidden.

```rust
// src/commands/repl/pager.rs

/// Calculate how many columns fit in the terminal width
/// Accounts for indicator cells when columns are hidden
fn visible_column_count(&self) -> usize {
    let hidden_left = self.hidden_columns_left();
    let hidden_right_possible = self.data.columns.len().saturating_sub(self.col_offset + 1) > 0;

    // Reserve space for indicator cells if columns are hidden
    let left_indicator_width = if hidden_left > 0 { INDICATOR_WIDTH + 3 } else { 0 };
    let right_indicator_width = if hidden_right_possible { INDICATOR_WIDTH + 3 } else { 0 };

    let mut total_width = 1 + left_indicator_width; // Left border + left indicator
    let mut count = 0;

    for col in self.data.columns.iter().skip(self.col_offset) {
        let col_width = col.display_width + 3; // " " + value + " " + "│"
        let available_width = self.term_width.saturating_sub(right_indicator_width);
        if total_width + col_width > available_width && count > 0 {
            break;
        }
        total_width += col_width;
        count += 1;
    }

    count.max(1) // Always show at least 1 column
}
```

#### Column Offset Calculation

```rust
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
```

#### Key Bindings

Horizontal navigation is handled in the `handle_key()` method:

```rust
fn handle_key(&mut self, key: KeyEvent) -> bool {
    match key.code {
        // Exit pager
        KeyCode::Char('q') | KeyCode::Esc => return false,

        // Vertical navigation (existing)
        KeyCode::Char('j') | KeyCode::Down => { /* scroll down */ }
        KeyCode::Char('k') | KeyCode::Up => { /* scroll up */ }
        KeyCode::Char(' ') | KeyCode::PageDown => { /* page down */ }
        KeyCode::Char('b') | KeyCode::PageUp => { /* page up */ }
        KeyCode::Char('g') | KeyCode::Home => { /* first row */ }
        KeyCode::Char('G') | KeyCode::End => { /* last row */ }

        // Horizontal navigation
        KeyCode::Left | KeyCode::Char('h') => {
            self.col_offset = self.col_offset.saturating_sub(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if self.col_offset + self.visible_column_count() < self.data.columns.len() {
                self.col_offset += 1;
            }
        }

        // Jump navigation
        KeyCode::Char('H') => {
            self.col_offset = 0; // First column
        }
        KeyCode::Char('L') => {
            let visible = self.visible_column_count();
            self.col_offset = self.data.columns.len().saturating_sub(visible); // Last columns
        }

        // Help
        KeyCode::Char('?') => {
            self.show_help(); // Display navigation help
        }

        _ => {}
    }
    true // Continue paging
}
```

#### Column Position Indicators

When columns are hidden, visual indicators show the user how many columns exist off-screen.

```rust
/// Render column indicators in header and data rows
fn render_header(&self, stdout: &mut impl Write, start_col: usize, end_col: usize) -> io::Result<()> {
    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();

    let mut row_str = String::from("│");

    // Left indicator cell (if columns hidden to left)
    if hidden_left > 0 {
        let indicator = format!("(+{} cols)", hidden_left);
        write_dim(stdout, &format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH))?;
        row_str = "│".to_string();
    }

    // Data column headers...

    // Right indicator cell (if columns hidden to right)
    if hidden_right > 0 {
        let indicator = format!("(+{} cols)", hidden_right);
        write!(stdout, "│")?;
        write_dim(stdout, &format!(" {:^width$} ", indicator, width = INDICATOR_WIDTH))?;
    }

    write!(stdout, "│")?;
    writeln!(stdout)
}
```

#### Status Bar Design

The status bar shows current position and available navigation:

```rust
fn render_status_bar(&self, stdout: &mut impl Write) -> io::Result<()> {
    let visible_cols = self.visible_column_count();
    let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
    let end_row = (self.row_offset + self.page_size).min(self.data.row_count);
    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();

    let col_status = format!(
        "Columns {}-{} of {}",
        self.col_offset + 1, end_col, self.data.columns.len()
    );

    let row_status = format!(
        "Rows {}-{} of {} ({}%)",
        self.row_offset + 1, end_row, self.total_rows, progress
    );

    // Navigation hints (only show horizontal hints if columns are hidden)
    let mut nav_parts = Vec::new();
    if hidden_left > 0 || hidden_right > 0 {
        nav_parts.push("h/l <-/->: scroll cols");
        nav_parts.push("H/L: first/last col");
    }
    nav_parts.push("j/k Space/b: rows");
    nav_parts.push("g/G: first/last");
    nav_parts.push("?: help");
    nav_parts.push("q/Esc: exit");

    writeln!(stdout, "{} | {} | {}", col_status, row_status, nav_parts.join(" | "))?;
    Ok(())
}
```

#### Help Display

Pressing `?` displays available navigation keys:

```rust
fn show_help(&mut self) {
    // Clear screen and show help overlay
    let help_text = r#"
Navigation Keys:

  Horizontal (Column) Navigation:
    <- or h     Scroll left one column
    -> or l     Scroll right one column
    H           Jump to first column
    L           Jump to last column

  Vertical (Row) Navigation:
    j or Down   Next row
    k or Up     Previous row
    Space       Page down
    b           Page up
    g           Jump to first row
    G           Jump to last row

  Control:
    ?           Show this help
    q or Esc    Exit pager

Press any key to return...
"#;
    // Display help_text and wait for keypress
}
```

#### Pager Search

Interactive less-style forward search layered on top of the existing
horizontal-paging pager. Implements `REQ-PAGER-SEARCH-001..012` from
`docs/specifications/repl.md`.

**Scope:** literal substring search, case-insensitive by default with
`\c` opt-in, forward-initiated (`/`), bidirectional navigation via `n` / `N`,
highlights rendered via the terminal `Reverse` attribute, status-bar match
count. Regex patterns, backward-initiated search (`?`), and cross-line matches
are explicitly out of scope.

##### Data model

Private to `src/commands/repl/pager.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Match {
    row: usize,
    col: usize,
    byte_start: usize,
    byte_end: usize,
}

struct SearchState {
    pattern: String,       // `\c` suffix stripped
    case_sensitive: bool,  // informational; matches already resolved
    matches: Vec<Match>,   // sorted by (row, col, byte_start)
    current: Option<usize>,
}

enum SearchStatus { None, Matches, NotFound }

enum InputMode {
    Normal,
    SearchPrompt { buffer: String },
}
```

New fields on `Pager`:

```rust
mode: InputMode,                       // state machine for event loop
search: Option<SearchState>,           // None until first submission
search_status: SearchStatus,           // drives status-bar text
transient_status: Option<String>,      // one-frame overlay (wrap notice)
```

`Match::byte_start` / `byte_end` index into the **post-truncation displayed
cell text** returned by `TableData::get_cell`, not the underlying Teradata
value. Matches in truncated tails are intentionally invisible — the user sees
only what the pager renders, and search honors that.

##### Match scanning

`find_all_matches` is exposed as `pub(crate)` so unit tests can drive it with
a raw `TableData` fixture:

```rust
pub(crate) fn find_all_matches(
    data: &TableData,
    pattern: &str,
    case_sensitive: bool,
) -> Vec<Match>;
```

Algorithm:

1. Iterate every `(row_idx, col_idx)` in `data.cell_values`.
2. For each cell's `&str`, scan byte-by-byte. Compare `&cell_bytes[start..start + pat_len]`
   to the pattern bytes using either exact byte equality (case-sensitive) or
   `eq_ignore_ascii_case` (case-insensitive).
3. On hit, push a `Match` and advance `start` by `pat_len` (non-overlapping
   matches). On miss, advance `start` by 1.

**Case folding is ASCII-only by design.** `str::to_lowercase` would allocate
and is Unicode-aware overkill for a SQL-result-text search; most Teradata
data and identifiers are ASCII. Non-ASCII bytes require exact match.

**Empty pattern** returns an empty match list — callers rely on this so that
submitting `"/\c"` (pattern `""`, case-sensitive) routes through the normal
`not found` flow without special-casing.

##### Input-mode state machine

`Pager::handle_key` branches on `self.mode`. A single flat event loop — no
nested `event::read()` sub-loop — consistent with the Sprint 33 discipline:

```rust
match &mut self.mode {
    InputMode::SearchPrompt { buffer } => match key.code {
        KeyCode::Enter     => submit (empty buffer == cancel),
        KeyCode::Esc       => cancel (keep prior search),
        KeyCode::Backspace => buffer.pop(),  // stays open when empty
        KeyCode::Char(c)   => buffer.push(c),
        _ => {}                              // ignore other keys
    },
    InputMode::Normal => match key.code {
        KeyCode::Char('/')                     => open prompt,
        KeyCode::Char('n') if self.search.is_some() => jump_match(Next),
        KeyCode::Char('N') if self.search.is_some() => jump_match(Prev),
        // ... existing arms ...
    },
}
```

`transient_status` is cleared at the top of `handle_key` BEFORE processing the
event, so the `wrapped to first match` overlay shows for exactly one render
cycle after the triggering `n`/`N` and disappears on the next keypress.

##### Scroll-to-match

`scroll_to_match_index` reuses the existing `row_offset` / `col_offset`
viewport fields:

- **Vertical:** place the matched row at the top of the viewport, clamped to
  `data.row_count.saturating_sub(page_size)` to avoid over-scrolling past the
  last page.
- **Horizontal:** if `match.col < col_offset`, set `col_offset = match.col`
  (leftmost visible); if `match.col >= col_offset + visible_column_count`,
  set `col_offset = match.col + 1 - visible_column_count` (rightmost visible).
  Uses a fixed shift rather than iterating — the next render cycle settles
  `visible_column_count` to the new offset.

This design piggybacks on the existing `visible_column_count` calculation
without extending it, keeping the column-windowing algorithm (from the
Horizontal Paging section) unchanged.

##### Highlight rendering

Matches in the current viewport are rendered with the crossterm `Reverse`
terminal attribute, NOT `SetBackgroundColor`:

```rust
execute!(stdout, SetAttribute(Attribute::Reverse))?;
stdout.write_all(&cell_bytes[start..end])?;
execute!(stdout, SetAttribute(Attribute::Reset))?;
```

`Reverse` composes cleanly on top of the existing foreground colors (cyan
header, DarkGrey `[NULL]`) without a color-specific clash. `SetBackgroundColor`
would hard-code a background that fights with terminal theming.

Cells are rendered through `render_cell_with_highlights`, which reconstructs
the padded layout (`" " + left_pad + value-with-highlights + right_pad + " "`)
so match byte-ranges are applied to the value only, never to the padding
whitespace. All visible matches — not just the active `current` match — are
highlighted, consistent with `less` and `vim`.

NULL cells (`value == "[NULL]"`) skip the highlight path entirely: they are
formatting artifacts, not user data, and would never realistically match a
search pattern.

##### Status-bar extensions

`render_status_bar_to_buffer` is a pure, writer-injected variant of the
status rendering path. Priority order (highest first):

1. **`InputMode::SearchPrompt`** — render `/{buffer}` literally. Empty buffer
   shows `/` with nothing after it; the terminal's own cursor provides the
   blink position.
2. **`transient_status`** — the one-frame wrap notice (`wrapped to first match`
   or `wrapped to last match`).
3. **`SearchStatus::Matches`** — `"Pattern: {pattern}  ({M} matches)"` with
   exactly two spaces before the `(`.
4. **`SearchStatus::NotFound`** — `"Pattern: {pattern}  not found"` with
   exactly two spaces before `not`.
5. **Default** — the existing `Columns X-Y of Z | Rows X-Y of Z (P%)` status
   plus a new `/: search` nav hint.

The status line is emitted via `render_status_bar_to_buffer` so tests can
assert on exact byte output without needing a terminal; the live
`render_status_bar` wraps it in `SetForegroundColor(DarkGrey)` / `ResetColor`.

##### Help-overlay integration

`show_help` reads from a single `const HELP_TEXT` associated constant. A
writer-injected `render_help_text(writer)` helper is exposed for tests
(mirroring the `render_to_buffer` / `render_border_plain` pattern already in
the file). The Search block is placed BEFORE the `Exit:` block:

```
Search:
  /pattern    Search forward for pattern (case-insensitive)
  /pattern\c  Search forward (case-sensitive)
  n           Next match
  N           Previous match

Exit:
  q / Esc     Exit pager and return to REPL prompt
```

##### Exclusions

- **Regex patterns:** out of scope. Only literal substrings are matched.
- **Backward-initiated search (`?pattern`):** out of scope. `N` already
  navigates backward through forward-initiated matches.
- **Highlights in the plain-text exit snapshot:** `render_exit_snapshot`
  uses the existing plain-text helpers (`render_row_plain`, etc.) and does
  not apply highlights. The snapshot is a copyable artifact; highlights
  would leak terminal escape sequences into pasted content.
- **Cross-cell or cross-line matches:** each cell is scanned independently.
  A pattern spanning a cell boundary cannot match.

##### Testability

Three pure functions are `#[cfg(test)]`-accessible at the module level
(tests in the same `mod tests` block):

- `find_all_matches(&TableData, &str, bool) -> Vec<Match>` — scans a
  fixture directly, asserting match byte-ranges, sort order, and
  case sensitivity.
- `parse_search_input(&str) -> (String, bool)` — `\c` suffix stripping and
  edge cases (`""`, `"\\c"`, `"foo\\c\\c"`).
- `pick_initial_match(&[Match], cursor_row) -> Option<usize>` — initial
  match selection including past-all-matches wrap.

Two writer-injected variants enable byte-level status / help assertions:

- `render_status_bar_to_buffer(&self, &mut impl Write)` — AC-9 exact
  format verification, including the double-space verbatim requirement.
- `render_help_text(&mut impl Write)` — AC-12 Search-block presence and
  placement (before `Exit:`).

##### Exit-path interaction

No change to `Pager::run` is required for exit. `q` / `Esc` in `InputMode::Normal`
returns `Ok(false)` as before; the `RawModeGuard`-style cleanup is unchanged.
`search: Option<SearchState>` is dropped with the `Pager`, so no state persists
across pager invocations (REQ-PAGER-SEARCH-011.5).

`Esc` in `InputMode::SearchPrompt` only closes the prompt (REQ-PAGER-SEARCH-001.5);
a second `Esc` in `InputMode::Normal` exits the pager. Users exit a partially
typed search with two `Esc` keystrokes — consistent with `less` and `vim`.

#### Integration with Executor (Sprint 30)

The executor passes `QueryResult` directly to the pager instead of pre-formatted strings:

```rust
// src/commands/repl/executor.rs

pub fn execute_sql_with_state<W: Write>(
    client: &DatabaseClient,
    state: &mut ReplState,
    sql: &str,
    writer: &mut W,
    default_limit: usize,
) -> Result<usize> {
    // ... execute query ...

    let pager_enabled = state.is_pager_enabled();

    if pager_enabled {
        // Sprint 30: Pass QueryResult directly to pager - NO pre-formatting!
        // The pager calculates column widths at render time based on terminal size.
        let pager_config = PagerConfig::default();
        match display_with_pager(&result_clone, &pager_config) {
            Ok(true) => {
                // Pager was used, output already displayed
            }
            Ok(false) => {
                // Pager not needed (small result), format and write directly
                write_output_with_timing(&result_clone, writer, OutputFormat::Table, &format_options, true)?;
            }
            Err(e) => {
                // Pager failed, fall back to direct output
                log::warn!("Pager failed: {}", e);
                write_output_with_timing(&result_clone, writer, OutputFormat::Table, &format_options, true)?;
            }
        }
    } else {
        // Pager disabled - format and write output directly
        write_output_with_timing(&result_clone, writer, OutputFormat::Table, &format_options, true)?;
    }

    Ok(row_count)
}
```

**Key Difference from Sprint 29**:
- Sprint 29: `display_with_pager(&formatted_string, row_count, &config)` - received pre-formatted table
- Sprint 30: `display_with_pager(&result, &config)` - receives structured data

This ensures the pager always has access to the original column metadata and can calculate proper widths for the current terminal dimensions.

#### Terminal Resize Handling

The pager responds to terminal resize events:

```rust
// In event loop
if let Event::Resize(w, h) = event::read()? {
    self.term_width = w as usize;
    self.term_height = h as usize;
    self.page_size = self.term_height.saturating_sub(5);
    // Recalculate visible columns after resize
    self.render()?;
}
```

Note: `col_offset` is preserved during resize - only the visible window size changes.

#### Data Structures

```rust
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
    term_height: usize,
    /// Total row count in result
    total_rows: usize,
}

/// Parsed table data for paging
struct TableData {
    /// Columns with their data
    columns: Vec<ColumnData>,
    /// Total number of rows
    row_count: usize,
}

/// A single column with its data
struct ColumnData {
    /// Column header name
    header: String,
    /// Cell values for each row
    values: Vec<String>,
    /// Calculated display width for this column
    display_width: usize,
}
```

#### Design Trade-offs

##### Column Offset vs Character Offset

**Chosen**: Column-level offset (scroll by whole columns)
**Alternative**: Character-level offset (scroll by characters within cells)
**Rationale**:
- Column-level scrolling keeps data aligned and readable
- Users think in terms of columns, not characters
- Simpler implementation with predictable behavior
- Matches behavior of database tools like `psql` expanded mode

##### Preserve Column Offset on Vertical Scroll

**Chosen**: Yes, preserve `col_offset` when scrolling vertically
**Alternative**: Reset to column 0 on vertical scroll
**Rationale**:
- Users may be examining a specific column across many rows
- Resetting would be disorienting
- No performance impact (render always uses current offsets)

##### Indicator Cell Design

**Chosen**: Dedicated indicator cells at edges with `(+N cols)` format
**Alternative**: Status bar only, no inline indicators
**Rationale**:
- Inline indicators provide immediate context
- Arrow indicators (`<--` / `-->`) in data rows reinforce navigation direction
- Does not require users to look at status bar

#### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Pager state | `src/commands/repl/pager.rs` | `Pager::new()`, `Pager::run()` |
| Column calculation | `src/commands/repl/pager.rs` | `visible_column_count()` |
| Navigation | `src/commands/repl/pager.rs` | `handle_key()` |
| Rendering | `src/commands/repl/pager.rs` | `render()`, `render_header()`, `render_row()` |
| Status bar | `src/commands/repl/pager.rs` | `render_status_bar()` |
| Help display | `src/commands/repl/pager.rs` | `show_help()` (to be added) |
| Configuration | `src/commands/repl/pager.rs` | `PagerConfig` |
| Activation check | `src/commands/repl/pager.rs` | `should_page()`, `display_with_pager()` |
| Executor integration | `src/commands/repl/executor.rs` | `execute_sql_with_state()` |
| State management | `src/commands/repl/state.rs` | `ReplState::is_pager_enabled()` |

### Statement Execution

```rust
// src/commands/repl/executor.rs

pub fn execute_statement(
    state: &mut ReplState,
    sql: &str,
) -> Result<ExecutionResult> {
    let start = Instant::now();

    // Execute query
    let result = state.connection.execute(sql)?;

    // Format output
    let output = format_result(&result, &state.config)?;

    let execution_result = ExecutionResult {
        output,
        row_count: result.row_count,
        execution_time: start.elapsed(),
    };

    // Display with paging if needed
    display_with_pager(&execution_result.output, result.row_count)?;

    // Show timing if enabled
    if state.timing {
        eprintln!(
            "\n{} rows in set ({:.3}s)",
            result.row_count,
            execution_result.execution_time.as_secs_f64()
        );
    }

    Ok(execution_result)
}
```

## Metacommands

Special commands prefixed with backslash:

```rust
pub enum Metacommand {
    ListDatabases,          // \l
    ListTables(Option<String>), // \dt [pattern]
    Describe(String),       // \d table_name
    ToggleTiming,          // \timing
    ToggleExpanded,        // \x
    Help(Option<String>),  // \h [topic]
    Quit,                  // \q
}

pub fn parse_metacommand(input: &str) -> Option<Metacommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('\\') {
        return None;
    }

    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    match parts.first()? {
        &"l" => Some(Metacommand::ListDatabases),
        &"dt" => Some(Metacommand::ListTables(parts.get(1).map(|s| s.to_string()))),
        &"d" => Some(Metacommand::Describe(parts.get(1)?.to_string())),
        &"timing" => Some(Metacommand::ToggleTiming),
        &"x" => Some(Metacommand::ToggleExpanded),
        &"h" => Some(Metacommand::Help(parts.get(1).map(|s| s.to_string()))),
        &"q" => Some(Metacommand::Quit),
        _ => None,
    }
}
```

## Metadata Caching

Fetch metadata at startup for tab completion:

```rust
impl MetadataCache {
    pub fn refresh(&mut self, conn: &Connection) -> Result<()> {
        // List databases
        self.databases = conn.list_databases()?;

        // List tables in current database
        self.tables.clear();
        let tables = conn.list_tables(None)?;
        for table in tables {
            self.tables.insert(table.name.clone(), table);

            // Get columns for each table
            let columns = conn.describe_table(&table.name)?;
            self.columns.insert(table.name.clone(), columns);
        }

        self.last_refresh = Instant::now();
        Ok(())
    }

    pub fn is_stale(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(300)  // 5 minutes
    }
}
```

## Error Handling

REPL continues on errors (unlike batch mode):

```rust
loop {
    match editor.readline(&prompt) {
        Ok(Signal::Success(input)) => {
            match execute_statement(&mut state, &input) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error: {}", e);
                    // Continue REPL loop
                }
            }
        }
        Ok(Signal::CtrlC) => {
            // Cancel current input, show new prompt
            continue;
        }
        Ok(Signal::CtrlD) | Err(_) => {
            // Exit REPL
            break;
        }
    }
}
```

## Code Linkage

| Component | File Path | Key Types |
|-----------|-----------|-----------|
| REPL orchestration | `src/commands/repl/mod.rs` | `execute()`, main loop |
| Statement executor | `src/commands/repl/executor.rs` | `execute_statement()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `MetadataCompleter` |
| Syntax highlighting | `src/commands/repl/highlighter.rs` | `SqlHighlighter` |
| Result pager | `src/commands/repl/pager.rs` | `display_with_pager()` |
| REPL state | `src/commands/repl/state.rs` | `ReplState`, `MetadataCache` |

## Design Trade-offs

### Persistent Connection
**Chosen**: Single connection for entire session
**Alternative**: Reconnect per query
**Rationale**: Performance, session state preservation

### Metadata Caching
**Chosen**: Cache at startup, manual refresh
**Alternative**: Query on-demand
**Rationale**: Fast completion, reduced network calls

### Paging Strategy
**Chosen**: Automatic paging based on terminal height
**Alternative**: Always page or never page
**Rationale**: Optimal UX for different result sizes

## Tab Completion Caching Architecture

### Overview

Tab completion provides context-aware suggestions for database names, table names, and column names. The completion system uses a multi-tiered caching strategy to balance responsiveness with data freshness.

**Sprint 20 Critical Fix**: Database names are now loaded at REPL startup, BEFORE the reedline editor is initialized. This prevents TTY conflicts that caused "Page 1: records 0 - 0" output during tab completion.

### Architecture

```
REPL Startup Sequence (Sprint 20):

Connect to Database
        ↓
Create CompletionState
        ↓
Load Database Names (CRITICAL: Before Editor Init)
        ↓
Show Banner
        ↓
Initialize reedline Editor
        ↓
Start REPL Loop

Tab Completion Flow:

User presses TAB
        ↓
Analyze SQL Context
        ↓
┌───────┴───────────────────┐
↓                           ↓
After FROM/JOIN?     After SELECT/WHERE?
↓                           ↓
Need Database/Table    Need Columns
↓                           ↓
Check Database Cache   Check Column Cache
(pre-loaded at startup)
↓                           ↓
┌───────┴───────┐    ┌───────┴───────┐
↓               ↓    ↓               ↓
Always Hit   Need Tables  Cache Hit  Cache Miss
(pre-loaded)  (lazy load)     ↓          ↓
    ↓              ↓      Return    Load from
Return         Load from  Suggestions  Database
Databases      Database      (suppressed output)
    ↓         (suppressed)
    +              ↓
Return        Cache Tables
Tables in         ↓
Current DB   Return Tables
```

### Cache Structure

```rust
// src/db/metadata.rs

pub struct MetadataCache {
    /// Database names - LOADED AT STARTUP (Sprint 20)
    /// Pre-loaded before editor init to avoid TTY conflicts
    databases: Option<Vec<String>>,

    /// Tables list - loaded lazily on first completion
    tables: Option<Vec<TableInfo>>,

    /// Columns per table: HashMap<"database.table", Vec<ColumnInfo>>
    /// Loaded lazily when user needs column completion
    columns: HashMap<String, Vec<ColumnInfo>>,

    /// Timestamp of database list load
    databases_loaded_at: Option<Instant>,

    /// Timestamp of table list load
    tables_loaded_at: Option<Instant>,

    /// Current database context (for unqualified table names)
    current_database: String,
}
```

### Loading Strategy

#### Database Names (Sprint 20 - Pre-loaded at Startup)
- **When**: At REPL startup, BEFORE editor initialization
- **Why**: Prevents TTY conflicts - the teradatarustapi may output pager messages during queries, which interferes with reedline's terminal handling
- **Query**: `SELECT TRIM(DatabaseName) FROM DBC.DatabasesV WHERE DatabaseName NOT IN (...system databases...)`
- **Cache Duration**: Session lifetime (cleared on /logon)
- **Rationale**: Database list rarely changes, pre-loading ensures no queries during completion

#### Table Names
- **When**: User presses TAB after FROM/JOIN keyword (lazy load)
- **Scope**: Load all user tables (excluding system databases)
- **Query**: `SELECT TRIM(DatabaseName), TRIM(TableName), TableKind FROM DBC.TablesV WHERE ...`
- **Cache Duration**: Session lifetime
- **Rationale**: Tables may be large, lazy load balances startup time vs. completion latency
- **Output Suppression**: Uses `OutputSuppressor` to redirect stdout/stderr during query

#### Table Names for Specific Database
- **When**: User types `database.` and presses TAB
- **Scope**: Filter from cached table list
- **No Additional Query**: Uses already-cached table data
- **Rationale**: Avoids additional network roundtrip

#### Column Names
- **When**: User presses TAB in column context (SELECT, WHERE, etc.)
- **Scope**: Load columns for specific table only when needed
- **Query**: `SELECT TRIM(ColumnName), ColumnType FROM DBC.ColumnsV WHERE DatabaseName = ? AND TableName = ?`
- **Cache Duration**: Session lifetime per table
- **Rationale**: Column lists can be large, lazy load minimizes memory

### Output Suppression and TTY Conflict Resolution

The teradatarustapi library (Go-based FFI) may print debug output during query execution:
```
Page 1: records 0 - 0  total: 0
```

**Sprint 20 Solution (Iteration 2)**: The fix is to pre-load ALL metadata at REPL startup, BEFORE the reedline editor is initialized, AND ensure that tab completion NEVER triggers database queries.

**Key Design Principles:**
1. **All metadata pre-loaded at startup**: Both database names AND table metadata are loaded before reedline initializes
2. **Zero queries during completion**: The completion code uses ONLY cached data - if data isn't cached, it returns empty rather than querying
3. **Startup loading is safe**: Any driver output during startup is harmless (before terminal is in raw mode)

**Implementation in `src/commands/repl/mod.rs`:**
```rust
pub fn execute(...) -> Result<()> {
    // Create completion state
    let completion_state = Arc::new(Mutex::new(CompletionState::new(client, database)));

    // Pre-load ALL metadata BEFORE editor initialization
    {
        let mut cs = completion_state.lock().unwrap();
        cs.ensure_databases_loaded();  // Load database names
        cs.ensure_tables_loaded();     // Load table metadata
    }

    // NOW initialize reedline (after all queries complete)
    let mut editor = create_editor(args, writer, Arc::clone(&completion_state))?;
    // ...
}
```

**Implementation in `src/commands/repl/metadata_completer.rs`:**
```rust
fn complete_tables(&self, prefix: &str) -> Vec<Suggestion> {
    // Use ONLY cached data - NO queries
    if state.cache().has_databases() {
        // Return cached database completions
    }
    if state.cache().has_tables() {
        // Return cached table completions
    }
    // If not cached, return empty (don't query!)
}
```

**Why File Descriptor Redirection Failed**: The `OutputSuppressor` approach (redirecting stdout/stderr to `/dev/null`) did not reliably suppress the pager output. This may be because:
- The Go library buffers output before the redirect takes effect
- There are timing issues with the CGO bridge
- The library may write directly to the controlling TTY in some circumstances

**The Robust Solution**: By ensuring tab completion NEVER triggers queries, we eliminate the problem at its source. The trade-off is slightly longer startup time, but this provides a much better user experience during interactive use.

### Completion Context Analysis

The `sql_context` module analyzes the SQL being typed to determine what completions are relevant:

```rust
// src/commands/repl/sql_context.rs

pub enum CompletionContext {
    /// After FROM, JOIN keywords - suggest databases + tables
    TableName { prefix: String },

    /// After "database." - suggest tables in that database
    SchemaQualifiedTable { schema: String, prefix: String },

    /// After SELECT, WHERE, ORDER BY - suggest columns
    ColumnName {
        tables: Vec<TableReference>,
        prefix: String,
        table_qualifier: Option<String>,
    },

    /// Generic keyword completion
    Keyword,
}

pub fn analyze_context(sql: &str, cursor_pos: usize) -> CompletionContext {
    // Tokenize SQL
    // Find cursor position relative to keywords
    // Determine appropriate context
}
```

### Integration with reedline

```rust
// src/commands/repl/metadata_completer.rs

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let context = analyze_context(line, pos);

        match context {
            CompletionContext::TableName { prefix } => {
                // Ensure databases loaded
                // Return databases + tables in current db
                self.complete_tables(&prefix)
            }
            CompletionContext::SchemaQualifiedTable { schema, prefix } => {
                // Ensure tables loaded for this database
                // Return tables in specified database
                self.complete_schema_tables(&schema, &prefix)
            }
            CompletionContext::ColumnName { tables, prefix, .. } => {
                // Ensure columns loaded for referenced tables
                // Return matching columns
                self.complete_columns(&tables, &prefix)
            }
            CompletionContext::Keyword => {
                // Return empty - keyword completion disabled
                vec![]
            }
        }
    }
}
```

### Design Trade-offs

#### Lazy Loading vs Eager Loading
**Chosen**: Lazy loading for tables and columns
**Alternative**: Load all metadata at startup
**Rationale**:
- Large databases may have thousands of tables
- Users typically work with subset of databases
- Reduces startup time and memory usage
- Trade-off: First completion in new context slightly slower

#### Per-Database vs Global Table Cache
**Chosen**: Per-database caching
**Alternative**: Single global table list
**Rationale**:
- Teradata uses `database.table` naming model
- Reduces memory for multi-database environments
- Enables targeted cache invalidation

#### Output Suppression Strategy
**Chosen**: File descriptor redirection (dup2 to /dev/null)
**Alternative**: Environment variable, connection parameter
**Rationale**:
- Works without driver modifications
- RAII pattern ensures cleanup even on panic
- Platform-specific (Unix only), Windows no-op

### Error Handling

Metadata loading failures should NOT block tab completion:

```rust
pub fn ensure_tables_loaded(&mut self) -> bool {
    if self.cache.has_tables() {
        return true;
    }

    match self.cache.load_tables(&self.client) {
        true => {
            log::debug!("Tables loaded successfully");
            true
        }
        false => {
            // Log error but don't crash
            log::warn!("Failed to load tables: {}",
                self.cache.last_error().unwrap_or("unknown"));
            false
        }
    }
}
```

### Cache Invalidation

Cache is cleared on:
1. `/logon` - New connection established
2. Session change to different database
3. (Future) After DDL statements (CREATE, DROP, ALTER)

### Performance Targets

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Cache hit completion | < 50ms | Instant feel |
| Database list load | < 500ms | Acceptable first-time delay |
| Table list load | < 500ms | Per-database, lazy |
| Column list load | < 300ms | Per-table, lazy |

## Sprint 21: Tab Completion Quality Enhancements

This section documents the technical design for Sprint 21 tab completion improvements.

### Feature 1: Complete Database Metadata Fetching (P0)

**Issue**: System databases like `dbc` are missing from tab completion.

**Root Cause Analysis**: The current query in `src/db/metadata.rs` (`load_databases()`) explicitly excludes system databases via a hardcoded exclusion list:

```sql
SELECT TRIM(DatabaseName)
FROM DBC.DatabasesV
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', 'DBC', ...)
```

The exclusion of `DBC` was intentional to reduce noise, but users need `dbc` for system queries.

**Solution Design**:

1. **Remove `DBC` from exclusion list**: The `dbc` database is a legitimate completion target for advanced users querying system views.

2. **Keep other system databases excluded**: Databases like `Console`, `Crashdumps`, `SYSBAR`, etc. provide no value for completion and add noise.

3. **Implementation location**: `src/db/metadata.rs`, function `load_databases()`, lines 461-474.

**Code Change**:
```rust
// Before (Sprint 20):
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', 'DBC', ...)

// After (Sprint 21):
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', ...)
// Note: 'DBC' REMOVED from exclusion list
```

**Complexity**: Low (single line SQL modification)

---

### Feature 2: Universal Table Metadata Fetching (P0)

**Issue**: Some databases like `demo_user` show "NO RECORDS FOUND" even when tables exist.

**Root Cause Analysis**: The current query in `src/db/metadata.rs` (`load_tables()`) has two filtering issues:

1. **System database exclusion**: Same exclusion list as database names, but for tables. If a user database name happens to match a pattern, it could be filtered.

2. **SAMPLE limit**: The query uses `SAMPLE 10000` which may not capture all tables in large environments, especially if those 10000 samples come from other databases first.

**Investigation Required**: Need to verify if `demo_user` is in the exclusion list or if there's a SAMPLE issue.

**Solution Design**:

1. **Verify database name**: Ensure `demo_user` is not accidentally filtered.

2. **Increase SAMPLE or remove limit**: For environments with many tables, 10000 may be insufficient. Consider removing the SAMPLE limit or increasing it substantially.

3. **Alternative: On-demand loading per database**: Instead of loading ALL tables at startup, load tables for a specific database when user types `database.` + TAB. This is more scalable.

**Recommended Approach - On-Demand Loading**:

```
Current Flow:
REPL Start → Load ALL tables (up to 10000) → Cache

New Flow (Sprint 21):
REPL Start → Load Database Names only
User types "demo_user." + TAB → Load tables for demo_user → Cache
Subsequent "demo_user." + TAB → Use cached data
```

**Implementation**:

1. **Keep startup database loading** (Feature 1 fix)
2. **Make table loading per-database on-demand**:
   - Add `load_tables_for_database(&self, client: &DatabaseClient, database: &str) -> bool`
   - Cache structure: `tables_by_database: HashMap<String, Vec<TableInfo>>`
   - On `SchemaQualifiedTable { schema, prefix }` context, check if `schema` is cached, load if not

**Code Structure**:
```rust
// src/db/metadata.rs
impl MetadataCache {
    /// Per-database table cache
    tables_by_database: HashMap<String, Vec<TableInfo>>,

    /// Load tables for a specific database (on-demand)
    pub fn load_tables_for_database(&mut self, client: &DatabaseClient, database: &str) -> bool {
        if self.tables_by_database.contains_key(&database.to_uppercase()) {
            return true; // Already cached
        }

        let sql = format!(r#"
            SELECT TRIM(TableName), TableKind
            FROM DBC.TablesV
            WHERE UPPER(DatabaseName) = UPPER('{}')
              AND TableKind IN ('T', 'V', 'O')
            ORDER BY TableName
        "#, escape_sql_string(database));

        // Execute and cache...
    }
}
```

**Complexity**: Medium (requires architecture change to per-database caching)

---

### Feature 3: Second TAB Accepts Selection (P1)

**Issue**: Second TAB moves to next item instead of accepting selection (bash/zsh behavior).

**Root Cause Analysis**: The current TAB keybinding uses:

```rust
// src/commands/repl/mod.rs, add_completion_keybinding()
keybindings.add_binding(
    KeyModifiers::NONE,
    KeyCode::Tab,
    ReedlineEvent::UntilFound(vec![
        ReedlineEvent::Menu("completion_menu".to_string()),
        ReedlineEvent::MenuNext,
    ]),
);
```

This means:
- **First TAB**: `Menu("completion_menu")` activates menu
- **Second TAB**: Menu already active, so `UntilFound` tries `Menu` (inapplicable), then executes `MenuNext` (move to next item)

**Reedline Investigation Results**:

After investigating reedline v0.38.0 source code and GitHub issues:

1. **No `MenuAccept` event exists**: reedline does not have a dedicated event for accepting menu selection ([GitHub Issue #624](https://github.com/nushell/reedline/issues/624) - OPEN as of 2024).

2. **`Enter` when menu is active**: This calls `replace_in_buffer()` then `MenuEvent::Deactivate`, which IS the accept behavior (reedline engine.rs lines 1096-1107).

3. **Problem**: We cannot distinguish "second TAB while menu open" from "first TAB" at the keybinding level.

**Feasibility Assessment**: **AT RISK / NOT FEASIBLE with current reedline**

The requested behavior (second TAB accepts selection) requires one of:
- A new `MenuAccept` event in reedline (upstream change)
- Custom menu implementation that tracks TAB press count
- Fork of reedline with custom event handling

**Alternative Approaches Investigated**:

| Approach | Feasibility | Complexity | Notes |
|----------|-------------|------------|-------|
| Upstream PR to add `MenuAccept` | Medium | High | Would require reedline maintainer buy-in |
| Fork reedline | High | Very High | Maintenance burden, version drift |
| Custom `EditMode` | Low | Very High | Would need to reimplement all keybindings |
| Track state in completer | Not Possible | N/A | Completer doesn't control keybindings |
| Bind TAB to `Enter` when menu open | Not Possible | N/A | No conditional binding mechanism |

**Recommendation**: **DEFER TO FUTURE SPRINT**

This feature requires upstream reedline changes. The recommended path is:
1. Document limitation clearly for user
2. Submit feature request to reedline (reference existing issue #624)
3. Consider contributing a PR to reedline if prioritized
4. Interim workaround: Users can press Enter to accept selection

**User Communication**:
```
Current behavior: TAB cycles through completions, Enter accepts
Bash/zsh behavior: Second TAB accepts
Status: Requires reedline library enhancement (tracked upstream)
```

---

### Feature 4: Smart Database-Dot-TAB Completion (P1)

**Issue**: User wants `dem` + TAB to complete to `demo_user.` and immediately show tables.

**Current Behavior**:
1. `dem` + TAB → Shows `demo_user` in menu
2. User navigates, presses Enter → `demo_user` inserted
3. User types `.` + TAB → Shows tables in `demo_user`

**Desired Behavior**:
1. `dem` + TAB → If only one match (`demo_user`), auto-complete to `demo_user.` and immediately show tables

**Feasibility Assessment**: **FEASIBLE**

This can be achieved by modifying the completion logic.

**Solution Design**:

1. **Detect unique database match**: In `complete_tables()`, if:
   - Context is `TableName` (after FROM/JOIN)
   - Exactly ONE database matches the prefix
   - No tables match the prefix in current database

   Then: Return the database name WITH trailing dot, and trigger table completion.

2. **Challenge**: reedline completer returns suggestions, it doesn't control follow-up actions.

3. **Alternative Implementation**: Use `append_whitespace: false` for database suggestions (already done), and enhance `SchemaQualifiedTable` handling to work seamlessly when user types the dot.

**Detailed Design**:

The key insight is that Feature 4 is partially solved if:
- Database completions already have `append_whitespace: false` (they do)
- After accepting `demo_user`, user types `.`
- On `.` + TAB, we enter `SchemaQualifiedTable { schema: "demo_user", prefix: "" }` context
- This triggers table loading for that database (Feature 2 on-demand loading)

**What's Missing**: The "auto-add dot and show tables" part. This requires the completer to:
1. Recognize single-match scenario
2. Append `.` to the completion value
3. Somehow trigger immediate re-completion

**Approach**: Modify suggestion value to include the dot when appropriate.

```rust
// In complete_tables():
if databases.len() == 1 && prefix_matches_database_exactly {
    // Single database match - append dot to enable quick table access
    suggestions.push(Suggestion {
        value: format!("{}.", db_name), // Include dot
        description: Some("(database - press TAB for tables)".to_string()),
        append_whitespace: false,
        ...
    });
}
```

**After selection**: User sees `demo_user.` and can immediately TAB again for tables.

**Complexity**: Medium (requires careful edge case handling)

**Edge Cases**:
- Multiple database matches: Don't add dot (user needs to disambiguate first)
- Database prefix also matches table name: Show both options
- Empty prefix: Show all databases without dots

---

### Feature 5: Automated Regression Testing (P2)

**Design Guidance for quality-validator**:

**Test Categories**:

1. **Unit Tests** (in `src/db/metadata.rs`, `src/commands/repl/metadata_completer.rs`):
   - `test_load_databases_includes_dbc` - Verify DBC not filtered
   - `test_load_tables_for_database` - Verify per-database loading
   - `test_completion_context_analysis` - Context detection
   - `test_database_suggestion_format` - Verify dot handling

2. **Integration Tests** (with mock or test database):
   - `test_tab_completion_shows_dbc`
   - `test_tab_completion_for_user_database`
   - `test_schema_qualified_completion`

3. **Manual Validation Required** (due to reedline TTY interaction):
   - Visual verification of menu display
   - TAB key behavior (navigation vs acceptance)
   - No pager output during completion

**Test File Locations**:
- `src/db/metadata.rs` - Unit tests in `mod tests`
- `tests/cases/TC-TAB-*.md` - Test case documentation
- `tests/results/sprint-21/` - Execution evidence

---

## Sprint 21 Implementation Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Feature 1: Include `dbc` | IMPLEMENTED | Removed 'DBC' from exclusion list in `load_databases()` and `load_tables()` |
| Feature 2: Universal tables | IMPLEMENTED | On-demand per-database loading via `load_tables_for_database()` |
| Feature 3: Second TAB accepts | DEFERRED | Blocked by reedline Issue #624 - no `MenuAccept` event |
| Feature 4: Smart database.TAB | IMPLEMENTED | Appends '.' to single-match database suggestions |
| Feature 5: Testability | IMPLEMENTED | Added unit tests for all new functionality |

**Implementation Details**:

1. **Feature 1** (lines 464, 392 in `src/db/metadata.rs`):
   - Removed 'DBC' from the exclusion lists in both `load_databases()` and `load_tables()` queries
   - `dbc` database now appears in tab completion

2. **Feature 2** (new methods in `src/db/metadata.rs`):
   - Added `tables_by_database: HashMap<String, Vec<TableInfo>>` to `MetadataCache`
   - Added `load_tables_for_database()` for on-demand loading
   - Added `has_tables_for_database()`, `get_tables_for_database()`, `find_tables_in_database_by_prefix()`
   - Updated `complete_schema_tables()` in `metadata_completer.rs` to trigger on-demand loading

3. **Feature 4** (in `complete_tables()` in `metadata_completer.rs`):
   - When exactly one database matches prefix and no tables match
   - Appends '.' to suggestion value: `format!("{}.", db_name)`
   - Description shows "(database - TAB for tables)"

4. **Feature 5** (unit tests added):
   - `test_has_tables_for_database`
   - `test_get_tables_for_database`
   - `test_find_tables_in_database_by_prefix`
   - `test_metadata_cache_clear_clears_per_database_tables`
   - `test_dbc_not_in_exclusion_list`

---

## Metacommand Tab Completion

This section documents the technical design for metacommand tab completion, enabling users to type `/des<TAB>` and see `/describe` in the completion menu.

### Architecture

Metacommand completion integrates with the existing `MetadataCompleter` by detecting when the input starts with `/` or `\` and providing metacommand suggestions instead of SQL completions.

```
Tab Completion Decision Flow:

User presses TAB
        |
        v
Check if line starts with '/' or '\'
        |
    +---+---+
    |       |
    v       v
  YES       NO
    |       |
    v       v
Metacommand   SQL Context
Completion    Completion
    |       (existing)
    v
Filter metacommands by prefix
    |
    v
Return suggestions with descriptions
```

### Implementation Location

**Primary file**: `src/commands/repl/metadata_completer.rs`

**Extension points**:
1. Add metacommand detection in `complete()` method (around line 489)
2. Add `complete_metacommands()` helper method
3. Define metacommand registry with names, aliases, and descriptions

### Metacommand Registry

```rust
// src/commands/repl/metadata_completer.rs

/// Metacommand definition for completion
struct MetacommandDef {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
}

/// Registry of all available metacommands
const METACOMMANDS: &[MetacommandDef] = &[
    MetacommandDef { name: "help", aliases: &["?"], description: "Show help message" },
    MetacommandDef { name: "quit", aliases: &["q", "exit"], description: "Exit the REPL" },
    MetacommandDef { name: "session", aliases: &[], description: "Show session information" },
    MetacommandDef { name: "ping", aliases: &[], description: "Test database connection" },
    MetacommandDef { name: "describe", aliases: &["d"], description: "Describe table structure" },
    MetacommandDef { name: "export", aliases: &[], description: "Export query results" },
    MetacommandDef { name: "pager", aliases: &[], description: "Toggle result paging" },
    MetacommandDef { name: "colors", aliases: &[], description: "Toggle syntax highlighting" },
    MetacommandDef { name: "logon", aliases: &[], description: "Switch database connection" },
    // Sprint 22 additions:
    MetacommandDef { name: "list databases", aliases: &["l"], description: "List all databases" },
    MetacommandDef { name: "list tables", aliases: &["dt"], description: "List tables in database" },
    MetacommandDef { name: "list views", aliases: &["dv"], description: "List views in database" },
];
```

### Completion Logic

```rust
/// Complete metacommands
fn complete_metacommands(&self, prefix: &str) -> Vec<Suggestion> {
    let prefix_lower = prefix.to_lowercase();

    METACOMMANDS
        .iter()
        .filter(|cmd| {
            cmd.name.starts_with(&prefix_lower) ||
            cmd.aliases.iter().any(|a| a.starts_with(&prefix_lower))
        })
        .map(|cmd| Suggestion {
            value: format!("/{}", cmd.name),
            description: Some(cmd.description.to_string()),
            style: None,
            extra: None,
            span: reedline::Span { start: 0, end: 0 }, // Set by caller
            append_whitespace: cmd.name.contains(' '), // Space for commands with args
        })
        .collect()
}
```

### Multi-word Metacommand Handling

Commands like `/list tables` require special handling:

1. First TAB after `/list` shows subcommands: `databases`, `tables`, `views`
2. Subcommand completion uses same registry pattern
3. Space-separated parts treated as single command

```rust
/// Check if completing a multi-word metacommand
fn complete_metacommand_subcommand(&self, prefix: &str) -> Vec<Suggestion> {
    let parts: Vec<&str> = prefix.split_whitespace().collect();

    match parts.as_slice() {
        ["list"] | ["list", ""] => {
            // Show subcommands: databases, tables, views
            vec![
                Suggestion { value: "/list databases".into(), description: Some("List all databases".into()), .. },
                Suggestion { value: "/list tables".into(), description: Some("List tables".into()), .. },
                Suggestion { value: "/list views".into(), description: Some("List views".into()), .. },
            ]
        }
        ["list", partial] => {
            // Filter subcommands by partial match
            let subcommands = ["databases", "tables", "views"];
            subcommands.iter()
                .filter(|s| s.starts_with(&partial.to_lowercase()))
                .map(|s| Suggestion { value: format!("/list {}", s), .. })
                .collect()
        }
        _ => Vec::new()
    }
}
```

---

## Schema Inspection Commands

This section documents the technical design for `/list databases`, `/list tables [pattern]`, and `/list views` commands.

### Architecture

Schema commands query Teradata system catalog views and format results for display. They integrate with the existing metacommand handler in `src/commands/repl/metacommands.rs`.

```
Schema Command Flow:

User types "/list tables emp%"
        |
        v
Parse metacommand (handle_metacommand_with_state)
        |
        v
Match "list" command with args ["tables", "emp%"]
        |
        v
Call execute_list_tables(client, pattern, writer)
        |
        v
Build SQL query for DBC.TablesV
        |
        v
Execute query via DatabaseClient
        |
        v
Format results as columnar output
        |
        v
Display to user
```

### Implementation Location

**Primary file**: `src/commands/repl/metacommands.rs`

**New functions**:
- `execute_list_databases()`
- `execute_list_tables(pattern: Option<&str>)`
- `execute_list_views()`

### `/list databases` Implementation

```rust
/// Execute /list databases
///
/// Queries DBC.DatabasesV and displays all accessible databases.
fn execute_list_databases<W: Write>(
    client: &DatabaseClient,
    writer: &mut W,
) -> Result<()> {
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name,
               OwnerName,
               CommentString
        FROM DBC.DatabasesV
        WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', ...)
        ORDER BY DatabaseName
    "#;

    match client.execute(sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Databases ({} total):", result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                let owner = row.get(1).map(|v| v.display()).unwrap_or_default();
                writeln!(writer, "  {:<30} (owner: {})", name, owner)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing databases: {}", e)?;
        }
    }
    Ok(())
}
```

### `/list tables [pattern]` Implementation

```rust
/// Execute /list tables [pattern]
///
/// Lists tables in current database, with optional glob pattern filtering.
/// Pattern supports:
/// - `*` matches any characters
/// - `?` matches single character
/// - `dbc.*` matches database prefix
fn execute_list_tables<W: Write>(
    client: &DatabaseClient,
    pattern: Option<&str>,
    current_database: &str,
    writer: &mut W,
) -> Result<()> {
    // Determine database context and table filter
    let (database, table_pattern) = parse_table_pattern(pattern, current_database);

    // Convert glob to SQL LIKE pattern
    let like_pattern = glob_to_sql_like(&table_pattern);

    let sql = format!(r#"
        SELECT TRIM(TableName) AS table_name,
               TableKind,
               CommentString
        FROM DBC.TablesV
        WHERE UPPER(DatabaseName) = UPPER('{}')
          AND TableKind IN ('T', 'V', 'O')
          AND TableName LIKE '{}'
        ORDER BY TableName
    "#, escape_sql_string(&database), like_pattern);

    match client.execute(&sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Tables in '{}' ({} found):", database, result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                let kind = row.get(1).map(|v| format_table_kind(&v.display())).unwrap_or_default();
                writeln!(writer, "  {:<40} ({})", name, kind)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing tables: {}", e)?;
        }
    }
    Ok(())
}

/// Parse table pattern to extract database and table filter
fn parse_table_pattern(pattern: Option<&str>, current_db: &str) -> (String, String) {
    match pattern {
        Some(p) if p.contains('.') => {
            let parts: Vec<&str> = p.splitn(2, '.').collect();
            (parts[0].to_string(), parts.get(1).unwrap_or(&"*").to_string())
        }
        Some(p) => (current_db.to_string(), p.to_string()),
        None => (current_db.to_string(), "*".to_string()),
    }
}

/// Convert glob pattern to SQL LIKE pattern
fn glob_to_sql_like(pattern: &str) -> String {
    pattern
        .replace('*', "%")
        .replace('?', "_")
}
```

### `/list views` Implementation

```rust
/// Execute /list views
///
/// Lists views in current database (TableKind = 'V').
fn execute_list_views<W: Write>(
    client: &DatabaseClient,
    current_database: &str,
    writer: &mut W,
) -> Result<()> {
    let sql = format!(r#"
        SELECT TRIM(TableName) AS view_name,
               CommentString
        FROM DBC.TablesV
        WHERE UPPER(DatabaseName) = UPPER('{}')
          AND TableKind = 'V'
        ORDER BY TableName
    "#, escape_sql_string(current_database));

    match client.execute(&sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Views in '{}' ({} found):", current_database, result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                writeln!(writer, "  {}", name)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing views: {}", e)?;
        }
    }
    Ok(())
}
```

### Metacommand Handler Integration

Update `handle_metacommand_with_state()` to handle the new commands:

```rust
// In handle_metacommand_with_state()
match command.as_str() {
    // ... existing commands ...

    "list" => {
        if args.is_empty() {
            writeln!(writer, "Usage: /list databases | tables [pattern] | views")?;
        } else {
            match args[0].to_lowercase().as_str() {
                "databases" | "database" => {
                    execute_list_databases(completion_state.client(), writer)?;
                }
                "tables" | "table" => {
                    let pattern = args.get(1).map(|s| *s);
                    let current_db = &state.connection_info().database;
                    execute_list_tables(completion_state.client(), pattern, current_db, writer)?;
                }
                "views" | "view" => {
                    let current_db = &state.connection_info().database;
                    execute_list_views(completion_state.client(), current_db, writer)?;
                }
                _ => {
                    writeln!(writer, "Unknown list target: {}", args[0])?;
                    writeln!(writer, "Usage: /list databases | tables [pattern] | views")?;
                }
            }
        }
    }

    // Aliases
    "l" => { /* delegate to list databases */ }
    "dt" => { /* delegate to list tables */ }
    "dv" => { /* delegate to list views */ }
}
```

---

## Loading Indicator for Slow Metadata Fetches

This section documents the technical design for displaying a loading indicator when metadata queries take longer than 500ms.

### Architecture

The loading indicator uses a background thread to display progress while the main thread executes the metadata query. This provides user feedback during slow network operations.

```
Loading Indicator Flow:

User types "database." + TAB
        |
        v
Check if database tables cached
        |
    +---+---+
    |       |
    v       v
  YES       NO
    |       |
    v       v
Return     Start loading indicator thread
cached     |
           v
           Display "Loading tables from <database>..."
           |
           v
           Execute metadata query
           |
           v
           Stop indicator, clear line
           |
           v
           Show completions
```

### Implementation Location

**Primary file**: `src/db/metadata.rs` (loading logic)
**Secondary file**: `src/commands/repl/metadata_completer.rs` (UI feedback)

### Design Approach

Two approaches were considered:

1. **Background thread with channel** (Chosen)
   - Spawn thread to display spinner
   - Main thread executes query
   - Signal thread when complete
   - Pros: Non-blocking, responsive
   - Cons: Thread overhead, complexity

2. **Timeout-based display**
   - Start timer before query
   - If timer exceeds threshold, print message
   - Pros: Simple
   - Cons: Message may appear mid-query, flicker

### Implementation

```rust
// src/db/metadata.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Threshold for showing loading indicator
const LOADING_INDICATOR_THRESHOLD: Duration = Duration::from_millis(500);

/// Execute a query with loading indicator for slow operations
fn execute_with_loading_indicator<T, F>(
    message: &str,
    threshold: Duration,
    operation: F,
) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let indicator_shown = Arc::new(AtomicBool::new(false));
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Clone for thread
    let indicator_shown_clone = Arc::clone(&indicator_shown);
    let stop_signal_clone = Arc::clone(&stop_signal);
    let message = message.to_string();

    // Spawn indicator thread
    let handle = thread::spawn(move || {
        thread::sleep(threshold);

        if !stop_signal_clone.load(Ordering::Relaxed) {
            eprint!("\r{}", message);
            let _ = std::io::Write::flush(&mut std::io::stderr());
            indicator_shown_clone.store(true, Ordering::Relaxed);
        }
    });

    // Execute operation
    let result = operation();

    // Stop indicator
    stop_signal.store(true, Ordering::Relaxed);
    let _ = handle.join();

    // Clear indicator line if shown
    if indicator_shown.load(Ordering::Relaxed) {
        eprint!("\r{}\r", " ".repeat(message.len()));
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    result
}
```

### Integration with On-Demand Loading

```rust
// src/db/metadata.rs

impl MetadataCache {
    pub fn load_tables_for_database(&mut self, client: &DatabaseClient, database: &str) -> bool {
        let db_upper = database.to_uppercase();

        if self.tables_by_database.contains_key(&db_upper) {
            return true;
        }

        let message = format!("Loading tables from {}...", database);

        execute_with_loading_indicator(
            &message,
            LOADING_INDICATOR_THRESHOLD,
            || {
                // Existing query logic...
                let sql = format!(r#"
                    SELECT TRIM(TableName), TableKind
                    FROM DBC.TablesV
                    WHERE UPPER(DatabaseName) = UPPER('{}')
                    AND TableKind IN ('T', 'V', 'O')
                    ORDER BY TableName
                "#, escape_sql_string(database));

                // Execute and cache...
            }
        )
    }
}
```

### Considerations

1. **Thread safety**: Uses atomics for cross-thread communication
2. **Terminal handling**: Writes to stderr to avoid interfering with completion output
3. **Cleanup**: Clears indicator line on completion
4. **reedline compatibility**: Must not interfere with terminal raw mode

---

## Multi-line Command History

This section documents the technical design for storing and recalling complete multi-line SQL statements as single history entries (Sprint 24).

### Problem Statement

**Current Behavior (Pre-Sprint 24):**
1. User types multi-line SQL statement across multiple lines
2. Each line is individually saved to history by reedline
3. Pressing UP arrow recalls only individual lines, not complete statements
4. User cannot easily re-execute or edit previous complex queries

**Desired Behavior:**
1. Multi-line SQL statements (until `;` terminator) stored as single history entry
2. UP/DOWN arrows recall complete multi-line commands
3. Cursor navigation works within recalled multi-line commands
4. Backward compatible with existing `~/.tq_history` files

### Solution Architecture

The solution leverages reedline's `Validator` trait, which controls when input is considered "complete":

```
Multi-line History Architecture:

User types line     →  reedline receives input
        |
        v
Validator::validate(line) called
        |
    +---+---+
    |       |
    v       v
Does line      Does line NOT
end with ';'?  end with ';'?
    |              |
    v              v
Return         Return
Complete       Incomplete
    |              |
    v              v
reedline       reedline
saves ENTIRE   continues
buffer to      accepting
history        input
    |              |
    v              v
Returns        Shows
Signal::       multi-line
Success        prompt
(buffer)       (repeats)
```

**Key Insight**: When `Validator` returns `Incomplete`:
- reedline does NOT save partial input to history
- reedline continues accepting input on new lines
- reedline accumulates all lines into single buffer
- When `Complete` is returned, the ENTIRE buffer is saved as one history entry

### Implementation Components

#### 1. SqlStatementValidator

New validator that checks for SQL statement completion:

```rust
// src/commands/repl/validator.rs

use reedline::{ValidationResult, Validator};

/// Validates SQL statement completion for multi-line history support
///
/// Returns `Incomplete` until a semicolon terminator is found,
/// causing reedline to accumulate multi-line input as a single
/// history entry.
pub struct SqlStatementValidator;

impl Validator for SqlStatementValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim();

        // Empty input is complete (allows pressing Enter on empty line)
        if trimmed.is_empty() {
            return ValidationResult::Complete;
        }

        // Metacommands are always complete (single line)
        if trimmed.starts_with('/') || trimmed.starts_with('\\') {
            return ValidationResult::Complete;
        }

        // SQL statements complete when ending with semicolon
        // Note: We use simple terminator detection for performance.
        // Edge cases (semicolons in strings/comments) are rare in practice
        // and can be handled by adding a space after the closing quote.
        if trimmed.ends_with(';') {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}
```

#### 2. Editor Configuration Update

Integrate validator into reedline setup:

```rust
// src/commands/repl/mod.rs (in create_editor function)

use crate::commands::repl::validator::SqlStatementValidator;

fn create_editor(...) -> Result<Reedline> {
    let mut editor = Reedline::create();

    // Add validator for multi-line history support
    editor = editor.with_validator(Box::new(SqlStatementValidator));

    // ... rest of editor configuration ...
}
```

#### 3. REPL Loop Simplification

With the validator handling multi-line accumulation, the REPL loop simplifies:

```rust
// Simplified REPL loop (validator handles accumulation)
match editor.read_line(&current_prompt) {
    Ok(Signal::Success(buffer)) => {
        // Buffer contains complete multi-line statement
        // (including newlines preserved from user input)

        if buffer.trim().is_empty() {
            continue;
        }

        if is_metacommand(&buffer) {
            handle_metacommand(&buffer, state, writer)?;
        } else {
            // Execute complete SQL statement
            execute_sql(&buffer, state, writer)?;
        }
    }
    // ... handle Ctrl-C, Ctrl-D ...
}
```

**Note:** The current `ReplState.input_buffer` accumulation logic becomes redundant when using the validator. The `Signal::Success(buffer)` from reedline already contains the complete multi-line input.

### History File Format

reedline's `FileBackedHistory` already supports multi-line entries via newline escaping:

```
File format (~/.tq_history):
-----------------------------
SELECT 1;
SELECT<\n>  col1,<\n>  col2<\n>FROM table<\n>WHERE x = 1;
SELECT * FROM users;
```

**Key Points:**
- Newlines within entries are escaped as `<\n>`
- Backward compatible: existing single-line entries work unchanged
- When loaded, `<\n>` is decoded back to actual newlines
- History search (Ctrl-R) works with full command text

### Cursor Navigation Within Multi-line Commands

When a multi-line command is recalled from history:

1. reedline displays the complete command with actual newlines
2. Standard line editing keys work:
   - Left/Right arrows: Move within current line
   - Home/End: Jump to line start/end
   - Ctrl-A/Ctrl-E: Beginning/end of line
3. Vertical navigation within multi-line buffer:
   - When at top line, UP recalls previous history entry
   - When at bottom line, DOWN recalls next history entry
   - Within multi-line buffer, cursor moves between lines

This is native reedline behavior - no additional implementation needed.

### Edge Cases and Mitigations

| Edge Case | Handling | Rationale |
|-----------|----------|-----------|
| Semicolon in string literal | May cause early termination | User can add space after closing quote to continue |
| Semicolon in comment | May cause early termination | Rare in interactive use; user can adjust |
| Very long statements | Works correctly | reedline handles arbitrary buffer sizes |
| Escaped newlines in history | Preserved correctly | reedline's `<\n>` encoding handles this |
| Existing history file | Backward compatible | Single-line entries have no `<\n>` |
| Ctrl-C during multi-line | Clears accumulated buffer | reedline handles this automatically |

### Testing Strategy

**Unit Tests:**
- `test_validator_empty_input_complete` - Empty returns Complete
- `test_validator_metacommand_complete` - Metacommands return Complete
- `test_validator_semicolon_complete` - Statements with `;` return Complete
- `test_validator_no_semicolon_incomplete` - Partial statements return Incomplete
- `test_validator_semicolon_in_middle` - Only trailing `;` counts

**Integration Tests (with mock):**
- History saves multi-line statement as single entry
- History recall returns complete multi-line command
- Ctrl-C clears accumulated multi-line buffer

**PTY Tests (manual validation primary):**
- Type multi-line SQL, verify single history entry created
- Press UP, verify complete statement recalled
- Edit recalled multi-line statement, verify cursor navigation
- Verify history file contains escaped newlines

**Manual Validation Required:**
- Visual appearance of multi-line continuation prompt
- Cursor movement within recalled multi-line command
- Keyboard behavior for UP/DOWN at buffer boundaries

### Design Trade-offs

#### Validator-Based vs Manual Accumulation
**Chosen**: reedline Validator (new in Sprint 24)
**Previous**: Manual `ReplState.input_buffer` accumulation
**Rationale**:
- Validator integrates with reedline's history mechanism
- Single history entry for complete statement (desired behavior)
- Simpler REPL loop (reedline handles accumulation)
- Better multi-line editing experience

#### Simple Semicolon Detection vs SQL Parsing
**Chosen**: Simple `ends_with(';')` check
**Alternative**: Full SQL lexer to detect semicolons in context
**Rationale**:
- Performance: No parsing overhead per keystroke
- Simplicity: Easy to understand and maintain
- Pragmatism: Edge cases rare in interactive use
- Escape hatch: User can work around by adjusting input

### Code Linkage

| Component | File Path | Key Changes |
|-----------|-----------|-------------|
| SQL Validator | `src/commands/repl/validator.rs` (NEW) | `SqlStatementValidator` struct |
| Editor Setup | `src/commands/repl/mod.rs` | Add `.with_validator()` call |
| Module Export | `src/commands/repl/mod.rs` | Add `mod validator;` |
| REPL Loop | `src/commands/repl/mod.rs` | Simplify to use validator buffer |
| State | `src/commands/repl/state.rs` | `input_buffer` may become redundant |

### Migration Notes

**From Pre-Sprint 24:**
- Existing `~/.tq_history` files are backward compatible
- Single-line entries continue to work unchanged
- New multi-line entries use `<\n>` escaping
- No migration script needed

**Removed/Deprecated:**
- `ReplState.input_buffer` accumulation logic (redundant with validator)
- `ReplState.has_input()` checks in REPL loop (validator handles this)
- Manual multi-line prompt state (reedline manages this)

### Implementation Status (Sprint 24)

**Status:** IMPLEMENTED

**Files Changed:**
- `src/commands/repl/validator.rs` (NEW) - SqlStatementValidator implementing reedline::Validator
- `src/commands/repl/mod.rs` - Added validator module, integrated into create_editor(), simplified repl_loop()

**Key Implementation Details:**
1. SqlStatementValidator returns `Complete` for empty input, metacommands, and statements ending with `;`
2. SqlStatementValidator returns `Incomplete` for partial SQL statements
3. REPL loop simplified - no longer needs manual accumulation, uses validator-provided buffer
4. Comprehensive unit tests for validator logic (13 tests)

---

## Sessions Command (Sprint 26)

This section documents the technical design for the `/sessions` metacommand, which displays active Teradata sessions with performance metrics.

### Overview

The `/sessions` command queries the Teradata `MonitorSession` table function to display real-time session activity. This is valuable for DBAs and developers who need visibility into system utilization, running queries, and performance issues like CPU/IO skew.

### Architecture

```
Sessions Command Flow:

User types "/sessions" or "tq --sessions"
        |
        v
Parse command (REPL or batch mode)
        |
        v
Build MonitorSession SQL query
        |
        v
Execute via DatabaseClient.execute()
        |
        v
Format results (calculate skew %, format timestamps)
        |
        v
Display using standard table formatter
```

### Implementation Location

**Primary file**: `src/commands/repl/metacommands.rs`

**New function**: `execute_sessions()`

**Related changes**:
- `src/cli.rs` - Add `Sessions` command variant
- `src/main.rs` - Handle `--sessions` flag in batch mode
- `src/commands/mod.rs` - Add `sessions()` function
- `src/commands/repl/metadata_completer.rs` - Add `/sessions` to metacommand completion

### SQL Query Design

The query uses Teradata's `MonitorSession` table function:

```sql
SELECT
    SessionNo,
    UserName,
    LogonTime,
    PEState,
    AMPState,
    AMPCPUSec,
    AMPIO,
    ReqSpool,
    AvgAmpCPUSec,
    HotAmp1CPU,
    AvgAmpIOCnt,
    HotAmp1IO
FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
ORDER BY SessionNo
```

**Query Parameters:**
- `-1`: Query all sessions (not just current user's sessions)
- `'*'`: All users (wildcard)
- `0`: Include all session types

**Design Decision:** The skew calculation is performed in Rust rather than SQL to:
1. Keep the SQL query simple and portable
2. Handle NULL values explicitly in the display layer
3. Allow flexible formatting of skew percentages

### Skew Calculation Algorithm

Skew measures how unevenly work is distributed across AMPs (parallel processing units).

**CPU Skew Formula:**
```rust
cpu_skew = if hot_amp1_cpu > 0.0 {
    Some(100.0 * (1.0 - (avg_amp_cpu_sec / hot_amp1_cpu)))
} else {
    None  // Display as [NULL] for idle sessions
}
```

**IO Skew Formula:**
```rust
io_skew = if hot_amp1_io > 0.0 {
    Some(100.0 * (1.0 - (avg_amp_io_cnt / hot_amp1_io)))
} else {
    None  // Display as [NULL] for idle sessions
}
```

**Interpretation:**
- `0%` = Perfect balance (all AMPs doing equal work)
- Higher `%` = More skewed (one AMP doing disproportionate work)
- `NULL` = Session is idle (no AMP activity to measure)

### LogonTime Formatting

The `LogonTime` column from Teradata is a TIMESTAMP. Format as specified:

```rust
fn format_logon_time(ts: &str) -> String {
    // Input: "2026-01-27 15:33:26.00" (Teradata TIMESTAMP)
    // Output: "2026/01/27 15:33:26.00" (User-friendly format)
    ts.replace('-', "/")
}
```

### Implementation Details

#### Metacommand Handler Integration

```rust
// In handle_metacommand_with_state()
match command.as_str() {
    // ... existing commands ...

    // Sprint 26: Sessions command
    "sessions" | "s" => {
        execute_sessions(completion_state.client(), writer)?;
    }

    // ... rest of commands ...
}
```

#### Execute Sessions Function

```rust
/// Execute /sessions metacommand
///
/// Lists active Teradata sessions with performance metrics including
/// CPU/IO skew percentages.
///
/// Uses MonitorSession(-1, '*', 0) table function which requires
/// SELECT privilege on DBC.MonitorSession.
fn execute_sessions<W: Write>(
    client: &DatabaseClient,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let sql = r#"
        SELECT
            SessionNo,
            UserName,
            LogonTime,
            PEState,
            AMPState,
            AMPCPUSec,
            AMPIO,
            ReqSpool,
            AvgAmpCPUSec,
            HotAmp1CPU,
            AvgAmpIOCnt,
            HotAmp1IO
        FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
        ORDER BY SessionNo
    "#;

    match client.execute(sql) {
        Ok(result) => {
            // Process rows and calculate skew
            let sessions: Vec<SessionInfo> = result.rows.iter()
                .filter_map(|row| SessionInfo::from_row(row))
                .collect();

            // Display results using table formatter
            display_sessions(&sessions, writer)?;

            writeln!(writer)?;
            writeln!(writer, "{} active session(s)", sessions.len())?;
        }
        Err(e) => {
            // Handle privilege errors gracefully
            if e.to_string().contains("privilege") ||
               e.to_string().contains("access") {
                writeln!(writer, "Error: Insufficient privileges to query sessions.")?;
                writeln!(writer)?;
                writeln!(writer, "Required: SELECT privilege on DBC.MonitorSession")?;
                writeln!(writer, "Contact your DBA to grant access.")?;
            } else {
                writeln!(writer, "Error listing sessions: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}
```

#### SessionInfo Struct

```rust
/// Session information extracted from MonitorSession result
struct SessionInfo {
    session_no: i64,
    user_name: String,
    logon_time: String,
    pe_state: String,
    amp_state: String,
    amp_cpu_sec: f64,
    amp_io: i64,
    req_spool: i64,
    cpu_skew: Option<f64>,  // None for idle sessions
    io_skew: Option<f64>,   // None for idle sessions
}

impl SessionInfo {
    fn from_row(row: &[Value]) -> Option<Self> {
        // Extract values with proper null handling
        let session_no = row.get(0)?.as_integer()?;
        let user_name = row.get(1)?.as_string()?.trim().to_string();
        let logon_time = format_logon_time(row.get(2)?.as_timestamp()?);
        let pe_state = row.get(3)?.as_string()?.trim().to_string();
        let amp_state = row.get(4)?.as_string()?.trim().to_string();
        let amp_cpu_sec = row.get(5)?.as_decimal().unwrap_or(0.0);
        let amp_io = row.get(6)?.as_integer().unwrap_or(0);
        let req_spool = row.get(7)?.as_integer().unwrap_or(0);

        // Calculate skew percentages
        let avg_amp_cpu = row.get(8)?.as_decimal().unwrap_or(0.0);
        let hot_amp1_cpu = row.get(9)?.as_decimal().unwrap_or(0.0);
        let avg_amp_io = row.get(10)?.as_decimal().unwrap_or(0.0);
        let hot_amp1_io = row.get(11)?.as_decimal().unwrap_or(0.0);

        let cpu_skew = calculate_skew(avg_amp_cpu, hot_amp1_cpu);
        let io_skew = calculate_skew(avg_amp_io, hot_amp1_io);

        Some(Self {
            session_no,
            user_name,
            logon_time,
            pe_state,
            amp_state,
            amp_cpu_sec,
            amp_io,
            req_spool,
            cpu_skew,
            io_skew,
        })
    }
}

fn calculate_skew(avg: f64, hot: f64) -> Option<f64> {
    if hot > 0.0 {
        Some(100.0 * (1.0 - (avg / hot)))
    } else {
        None
    }
}
```

### Batch Mode Integration

The `--sessions` flag provides the same functionality in batch mode.

#### CLI Definition (src/cli.rs)

```rust
/// Available commands for tq
#[derive(Subcommand, Debug)]
pub enum Command {
    // ... existing commands ...

    /// List active database sessions with performance metrics
    ///
    /// Displays active Teradata sessions including user, state, and
    /// performance metrics (CPU, IO, skew percentages).
    Sessions(SessionsArgs),
}

/// Arguments for the sessions command
#[derive(Parser, Debug)]
pub struct SessionsArgs {
    /// Output format
    #[arg(short, long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

#### Main Handler (src/main.rs)

```rust
Command::Sessions(args) => {
    let mut stdout = io::stdout();
    commands::sessions(&client, &args, &mut stdout, use_color)?;
}
```

### Tab Completion Integration

Add `/sessions` to the metacommand completion registry:

```rust
// In metadata_completer.rs, METACOMMANDS array
MetacommandDef {
    name: "sessions",
    aliases: &["s"],
    description: "List active sessions with performance metrics"
},
```

### Error Handling

#### Privilege Error

```
tq> /sessions

Error: Insufficient privileges to query sessions.

Required: SELECT privilege on DBC.MonitorSession
Contact your DBA to grant access.
```

**Detection:** Check error message for "privilege", "access", or error code 3523.

#### Connection Error

```
tq> /sessions

Error listing sessions: Connection lost to host:port
Use /reconnect to establish new connection
```

#### Empty Result Set

```
tq> /sessions

Sessions:
(no active sessions found)

0 active session(s)
```

Note: This is rare since the current session would normally appear.

### Teradata Compatibility

**Required Teradata Version:** 14.10+

The `MonitorSession` table function was introduced in Teradata 14.10. Earlier versions do not support this syntax.

**Version Detection:** If the query fails with a syntax error mentioning "MonitorSession", display:

```
Error: MonitorSession function not available.

This feature requires Teradata 14.10 or later.
Your system may be running an earlier version.
```

### Performance Considerations

- **Query Speed:** MonitorSession(-1) queries all sessions - typically <1 second
- **Result Size:** Usually small (tens to hundreds of sessions)
- **System Impact:** Minimal - reads from session control structures
- **No Caching:** Results are always live (no caching)

### Output Format

**Table Format (default):**
```
Sessions:
┌───────────┬──────────┬────────────────────────┬────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEState    │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE       │ IDLE     │         0 │     6 │           0 │         [NULL] │       [NULL] │
│      1077 │ DBC      │ 2026/01/27 15:33:27.00 │ IDLE       │ IDLE     │     0.376 │  6782 │           0 │         [NULL] │       [NULL] │
│      1078 │ DBC      │ 2026/01/27 15:33:28.00 │ DISPATCHING│ ACTIVE   │   366.736 │ 75335 │ 26753187840 │           2.87 │         3.78 │
└───────────┴──────────┴────────────────────────┴────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

3 active session(s)
```

**JSON Format:**
```json
[
  {
    "SessionNo": 1076,
    "UserName": "DBC",
    "LogonTime": "2026/01/27 15:33:26.00",
    "PEState": "IDLE",
    "AMPState": "IDLE",
    "AMPCPUSec": 0,
    "AMPIO": 6,
    "ReqSpool": 0,
    "AmpCPUSkew": null,
    "AmpIOSkew": null
  }
]
```

**CSV Format:**
```csv
SessionNo,UserName,LogonTime,PEState,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %
1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
1077,DBC,2026/01/27 15:33:27.00,IDLE,IDLE,0.376,6782,0,,
1078,DBC,2026/01/27 15:33:28.00,DISPATCHING,ACTIVE,366.736,75335,26753187840,2.87,3.78
```

### Code Linkage

| Component | File Path | Key Changes |
|-----------|-----------|-------------|
| Metacommand handler | `src/commands/repl/metacommands.rs` | Add `execute_sessions()` function |
| Help text | `src/commands/repl/metacommands.rs` | Update `print_help_extended()` |
| Metacommand completion | `src/commands/repl/metadata_completer.rs` | Add `/sessions` to registry |
| Batch mode CLI | `src/cli.rs` | Add `Sessions` command variant |
| Batch mode handler | `src/main.rs` | Handle `Command::Sessions` |
| Sessions command | `src/commands/sessions.rs` (NEW) | Batch mode implementation |
| Commands export | `src/commands/mod.rs` | Export `sessions()` function |

### Design Trade-offs

#### SQL Calculation vs Rust Calculation for Skew
**Chosen:** Calculate skew in Rust
**Alternative:** Use Teradata's DECIMAL casting and NULLIFZERO in SQL
**Rationale:**
- Simpler SQL query (easier to debug and maintain)
- Explicit NULL handling in display layer
- Flexible formatting without SQL FORMAT clauses
- Better testability (unit tests for skew calculation)

#### Separate SessionsArgs vs Reusing QueryArgs
**Chosen:** Separate `SessionsArgs` struct
**Alternative:** Reuse `QueryArgs` with pre-defined SQL
**Rationale:**
- Cleaner CLI interface (no SQL argument needed)
- `--sessions` is a standalone action, not a query
- Simpler user experience for DBAs

#### Monolithic Function vs Trait-based Design
**Chosen:** Simple `execute_sessions()` function
**Alternative:** Create `MetaCommand` trait with `execute()` method
**Rationale:**
- Follows existing metacommand pattern in codebase
- Lower implementation complexity
- Can refactor to trait-based if more commands added

### Sprint 27 Bug Fix: Missing Sessions (#10)

#### Problem Description

The `/sessions` command was incorrectly showing 2 sessions when 3 actually existed. Active sessions with `DISPATCHING/ACTIVE` states were being silently dropped from the output.

**User Evidence:**
- SQL query `SELECT ... FROM TABLE (MonitorSession(-1,'*',0))` returned 3 rows
- `/sessions` command only displayed 2 rows
- Missing session had `PEState = 'DISPATCHING'` and `AMPState = 'ACTIVE'`

#### Root Cause Analysis

The bug was in `SessionInfo::from_row()` in `src/commands/sessions.rs`. The function used strict type matching for `PEState` and `AMPState` columns that returned `None` (silently dropping the row) when the value type was unexpected:

```rust
// BUGGY CODE (Sprint 26):
let pe_state = match &row[3] {
    Value::String(s) => s.trim().to_string(),
    Value::Null => "[NULL]".to_string(),
    _ => return None,  // BUG: Silently drops entire row!
};

let amp_state = match &row[4] {
    Value::String(s) => s.trim().to_string(),
    Value::Null => "[NULL]".to_string(),
    _ => return None,  // BUG: Silently drops entire row!
};
```

**Why this caused the bug:**
1. The Teradata driver may return state values as different `Value` types depending on the column metadata or data characteristics
2. IDLE states were being returned as `Value::String` and worked correctly
3. Some active states (like `DISPATCHING`) were being returned as a different type
4. When the match arm hit the `_` wildcard, `return None` caused `filter_map()` to skip that session entirely

**The silent failure pattern:**
```rust
let sessions: Vec<SessionInfo> = result.rows.iter()
    .filter_map(|row| SessionInfo::from_row(row))  // Silently drops None
    .collect();
```

#### Solution Design

**Principle:** Never silently drop rows due to unexpected value types. Instead, convert any value type to a displayable string using the `Value::display()` method.

**Fixed Code:**
```rust
// FIXED CODE (Sprint 27):
let pe_state = match &row[3] {
    Value::String(s) => s.trim().to_string(),
    Value::Null => "[NULL]".to_string(),
    other => other.display(),  // Convert any type to string
};

let amp_state = match &row[4] {
    Value::String(s) => s.trim().to_string(),
    Value::Null => "[NULL]".to_string(),
    other => other.display(),  // Convert any type to string
};
```

**Design Rationale:**
1. **Defensive Programming:** Never assume database driver returns specific types
2. **Graceful Degradation:** Display something meaningful rather than dropping data
3. **Consistency:** The `Value::display()` method already handles all value types correctly
4. **User Visibility:** Users see all sessions, even if some field formatting is unexpected

#### Regression Prevention

**Unit Test Added:**
```rust
#[test]
fn test_session_info_from_row_non_string_state() {
    // Test that non-String state values don't cause row to be dropped
    let row = vec![
        Value::Integer(1232),
        Value::String("DBC".to_string()),
        Value::Timestamp("2026-01-27 19:31:25.00".to_string()),
        Value::Integer(5),  // PEState as unexpected type
        Value::Boolean(true),  // AMPState as unexpected type
        // ... rest of row
    ];

    let session = SessionInfo::from_row(&row);
    assert!(session.is_some(), "Row should not be dropped for non-string state");
}
```

#### Lessons Learned

1. **Pattern Matching Pitfall:** Using `_ => return None` in match arms can silently drop data
2. **Test Coverage Gap:** Unit tests only tested with expected value types
3. **Database Driver Variability:** Teradata driver type mapping may vary by database version, client configuration, or data characteristics
4. **Defensive Parsing:** When parsing database rows, prefer converting to display format over rejecting data

### Testing Strategy

**Unit Tests:**
- `test_calculate_skew_active_session` - Non-zero hot values
- `test_calculate_skew_idle_session` - Zero hot values return None
- `test_format_logon_time` - Date format conversion
- `test_session_info_from_row` - Row parsing with various values
- `test_session_info_from_row_with_nulls` - NULL handling
- `test_session_info_from_row_non_string_state` - Non-string state handling (Sprint 27)

**Integration Tests:**
- `test_sessions_command_execution` - With mock database
- `test_sessions_privilege_error` - Error handling
- `test_sessions_empty_result` - No sessions case

**PTY Tests:**
- `/sessions` command execution in REPL
- Tab completion includes `/sessions`
- Help text displays correctly

**Manual Validation:**
- Visual verification of output format
- Skew calculation accuracy against known values
- Error message clarity for privilege issues

### Implementation Checklist

1. [ ] Add `execute_sessions()` to `metacommands.rs`
2. [ ] Add `/sessions` and `/s` to metacommand match in `handle_metacommand_with_state()`
3. [ ] Update `print_help_extended()` with `/sessions` description
4. [ ] Add `/sessions` to metacommand completion registry
5. [ ] Add `Sessions` variant to `Command` enum in `cli.rs`
6. [ ] Add `SessionsArgs` struct to `cli.rs`
7. [ ] Create `src/commands/sessions.rs` for batch mode
8. [ ] Update `src/commands/mod.rs` to export sessions
9. [ ] Handle `Command::Sessions` in `main.rs`
10. [ ] Add unit tests for skew calculation
11. [ ] Add unit tests for SessionInfo parsing
12. [ ] Verify output format matches specification

---

## Sprint 28: Interactive Horizontal Paging Enhancement

This section documents the technical design for Sprint 28 interactive horizontal paging and startup warnings fix.

### Background

**GitHub Issue #7:** Users cannot effectively explore wide result sets. While the pager has basic horizontal navigation (Left/Right arrow keys), it lacks:
1. Visual indicators showing hidden columns on both sides (`(+N cols)`)
2. Clear status bar showing column navigation capabilities
3. Discoverability - users don't realize horizontal scrolling is available

**GitHub Issue #11:** Cargo build warnings pollute REPL startup with messages like:
```
warning: tq@1.12.0: Successfully copied teradatasql.dylib to ...
```

### Feature #7: Interactive Horizontal Paging

#### Current Pager State Analysis

The existing pager at `src/commands/repl/pager.rs` provides:

**What Works:**
- Left/Right arrow key navigation (lines 528-535)
- `h`/`l` vim-style navigation (same lines)
- `H`/`L` jump to first/last column (lines 538-545)
- Status bar shows `Columns X-Y of Z` (lines 473-478)
- `visible_column_count()` calculates fitting columns (lines 322-336)
- `col_offset` tracks current horizontal position

**What's Missing (per specification):**
- `(+N cols)` indicators on left/right borders when columns hidden
- Clear visual cue that more columns exist beyond current view
- Enhanced status bar with explicit horizontal navigation hints

#### Solution Architecture

##### 1. Column Indicator Design

**Implementation in `render_header()` and `render_row()`:**

Add indicator columns on borders when columns are hidden:

```rust
/// Calculate hidden column counts
fn hidden_columns_left(&self) -> usize {
    self.col_offset
}

fn hidden_columns_right(&self) -> usize {
    let visible = self.visible_column_count();
    let end_col = (self.col_offset + visible).min(self.data.columns.len());
    self.data.columns.len().saturating_sub(end_col)
}
```

**Left Indicator (when `col_offset > 0`):**
- Display format: `(+N cols)` as first pseudo-column
- Width: Fixed 10 characters (fits `(+999 cols)`)
- Alignment: Right-aligned
- Color: Dim/gray to distinguish from data

**Right Indicator (when more columns hidden):**
- Display format: `(+N cols)` as last pseudo-column
- Width: Fixed 10 characters
- Alignment: Left-aligned
- Color: Dim/gray to distinguish from data

**Visual Example:**
```
╭────────────┬─────────────────────────┬──────────────┬────────────╮
│   (+2 cols)│ Column3                 │ Column4      │   (+15 cols)│
├────────────┼─────────────────────────┼──────────────┼────────────┤
│         ...│ Value3                  │ Value4       │ ...        │
╰────────────┴─────────────────────────┴──────────────┴────────────╯
```

##### 2. Render Method Updates

**File:** `src/commands/repl/pager.rs`

**Update `render_border()`:**
```rust
fn render_border(&self, stdout: &mut impl Write, position: &str) -> io::Result<()> {
    let (left, middle, right, line) = match position {
        "top" => ('╭', '┬', '╮', '─'),
        "middle" => ('├', '┼', '┤', '─'),
        "bottom" => ('╰', '┴', '╯', '─'),
        _ => ('├', '┼', '┤', '─'),
    };

    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();
    let visible_cols = self.visible_column_count();
    let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());

    let mut border = String::new();
    border.push(left);

    // Left indicator column (if columns hidden on left)
    if hidden_left > 0 {
        border.push_str(&line.to_string().repeat(INDICATOR_WIDTH));
        border.push(middle);
    }

    // Data columns
    for (i, col) in self.data.columns[self.col_offset..end_col]
        .iter()
        .enumerate()
    {
        border.push_str(&line.to_string().repeat(col.display_width + 2));
        let is_last_data_col = i == end_col - self.col_offset - 1;
        if !is_last_data_col || hidden_right > 0 {
            border.push(middle);
        }
    }

    // Right indicator column (if columns hidden on right)
    if hidden_right > 0 {
        border.push_str(&line.to_string().repeat(INDICATOR_WIDTH));
    }

    border.push(right);
    writeln!(stdout, "{}", border)
}
```

**Update `render_header()`:**
```rust
fn render_header(
    &self,
    stdout: &mut impl Write,
    start_col: usize,
    end_col: usize,
) -> io::Result<()> {
    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();

    let mut row_str = String::from("│");

    // Left indicator
    if hidden_left > 0 {
        let indicator = format!("(+{} cols)", hidden_left);
        let padded = format!(" {:>width$} ", indicator, width = INDICATOR_WIDTH - 2);
        // Use dim color
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        write!(stdout, "{}", padded)?;
        execute!(stdout, ResetColor)?;
        write!(stdout, "│")?;
    }

    // Data column headers
    for col in &self.data.columns[start_col..end_col] {
        let padded = format!(" {:^width$} ", col.header, width = col.display_width);
        row_str.push_str(&padded);
        row_str.push('│');
    }

    // Right indicator
    if hidden_right > 0 {
        let indicator = format!("(+{} cols)", hidden_right);
        let padded = format!(" {:<width$} ", indicator, width = INDICATOR_WIDTH - 2);
        // Use dim color
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        write!(stdout, "{}", padded)?;
        execute!(stdout, ResetColor)?;
        write!(stdout, "│")?;
    }

    // Header text (cyan)
    execute!(stdout, SetForegroundColor(Color::Cyan))?;
    write!(stdout, "{}", row_str)?;
    execute!(stdout, ResetColor)?;
    writeln!(stdout)
}
```

**Update `render_row()`:**
Similar pattern - add `(+N cols)` / `...` indicator cells on left/right.

##### 3. Status Bar Enhancement

**Current (line 488):**
```rust
let nav_hints = "j/k:scroll  Space/b:page  Left/Right:columns  g/G:first/last  q:exit";
```

**Updated:**
```rust
let nav_hints = "Arrows/hjkl: scroll | Space/b: page | g/G: jump | q: exit";
```

**Rationale:**
- "Arrows" is more intuitive than "Left/Right"
- Combined format is more concise
- Clearer that both horizontal and vertical scrolling work

##### 4. Help Text Integration

**File:** `src/commands/repl/metacommands.rs`
**Function:** `print_help_extended()`

Add new section:
```rust
writeln!(writer)?;
writeln!(writer, "Result Paging (when viewing large results):")?;
writeln!(writer, "  Up/Down, j/k       Scroll rows")?;
writeln!(writer, "  Left/Right, h/l    Scroll columns (wide tables)")?;
writeln!(writer, "  Space, Page Down   Next page")?;
writeln!(writer, "  b, Page Up         Previous page")?;
writeln!(writer, "  g, Home            Jump to first row")?;
writeln!(writer, "  G, End             Jump to last row")?;
writeln!(writer, "  H                  Jump to first column")?;
writeln!(writer, "  L                  Jump to last column")?;
writeln!(writer, "  q, Esc             Exit pager (return to prompt)")?;
```

##### 5. Constants and Configuration

```rust
/// Width of the column indicator pseudo-column (fits "(+999 cols)")
const INDICATOR_WIDTH: usize = 12;
```

#### Code Linkage for Feature #7

| Component | File Path | Function | Changes |
|-----------|-----------|----------|---------|
| Indicator calculation | `pager.rs` | NEW: `hidden_columns_left()`, `hidden_columns_right()` | Add methods |
| Border rendering | `pager.rs` | `render_border()` | Add indicator columns to borders |
| Header rendering | `pager.rs` | `render_header()` | Add indicator cells |
| Row rendering | `pager.rs` | `render_row()` | Add indicator cells |
| Column width calc | `pager.rs` | `visible_column_count()` | Account for indicator width |
| Status bar | `pager.rs` | `render_status_bar()` | Update nav hints text |
| Help text | `metacommands.rs` | `print_help_extended()` | Add pager section |

### Feature #11: Clean REPL Startup

#### Problem Analysis

**Source of Warning:**
The warning originates from `build.rs` (lines 50-59):
```rust
println!(
    "cargo:warning=Successfully copied {} to {}",
    lib_name,
    lib_dest.display()
);
```

Also a fallback warning (lines 81-85):
```rust
println!("cargo:warning=Could not find teradatasql library in cargo cache");
```

**Cargo Output:**
The "Finished" and "Running" messages are standard cargo output, not from build.rs.

#### Solution Design

**Option 1: Remove informational warning from build.rs (RECOMMENDED)**

Change the success message from `cargo:warning=` to a silent operation. The warning was informational, not an actual warning about a problem.

**File:** `build.rs`

**Before:**
```rust
println!(
    "cargo:warning=Successfully copied {} to {}",
    lib_name,
    lib_dest.display()
);
```

**After:**
```rust
// Success message is informational - use rerun-if-changed for cargo visibility
// instead of warning which pollutes user output
eprintln!("build.rs: Copied {} to {}", lib_name, lib_dest.display());
```

Or simply remove the println entirely - silent success.

**Keep the failure warning** - that's a legitimate warning users need to see:
```rust
println!("cargo:warning=Could not find teradatasql library in cargo cache");
```

**Option 2: Suppress via cargo flag (NOT RECOMMENDED)**

Users could use `cargo run --quiet` but this also hides genuine errors.

**Option 3: Documentation workaround (FALLBACK)**

Document that developers should use release builds or add cargo config.

#### Recommended Approach

1. **Remove success warning** from build.rs - silent success is appropriate
2. **Keep failure warning** - users need to know if library wasn't found
3. **Test** that release builds show no warnings
4. **Document** that dev builds may show cargo's "Finished/Running" output (this is standard cargo behavior)

#### Code Changes for Feature #11

**File:** `build.rs`

```rust
// Line 55-59: Remove or convert to silent
// BEFORE:
println!(
    "cargo:warning=Successfully copied {} to {}",
    lib_name,
    lib_dest.display()
);

// AFTER (Option A - silent):
// Just remove the println

// AFTER (Option B - debug only):
#[cfg(debug_assertions)]
eprintln!("build.rs: Copied {} to {}", lib_name, lib_dest.display());
```

### Complexity Assessment

| Feature | Complexity | Estimate | Risk |
|---------|------------|----------|------|
| #7 Horizontal Paging Indicators | Medium | 4-6 hours | Low |
| #7 Status Bar Update | Low | 0.5 hours | Low |
| #7 Help Text Integration | Low | 0.5 hours | Low |
| #11 Startup Warnings | Low | 0.5 hours | Low |
| Testing (both features) | Medium | 2-3 hours | Low |
| **Total** | **Medium** | **8-10 hours** | **Low** |

### Testing Strategy

#### Feature #7 Tests

**Unit Tests (pager.rs):**
- `test_hidden_columns_left_none` - No hidden columns when offset is 0
- `test_hidden_columns_left_some` - Correct count when offset > 0
- `test_hidden_columns_right_none` - No hidden columns when all visible
- `test_hidden_columns_right_some` - Correct count when columns overflow
- `test_indicator_width_constant` - Verify fits expected patterns

**Integration Tests:**
- Execute query with >20 columns, verify `(+N cols)` appears
- Navigate left/right, verify indicator counts update
- Verify status bar shows updated text

**Manual Validation:**
- Wide table rendering appearance
- Indicator visibility and readability
- Navigation responsiveness

#### Feature #11 Tests

**Manual Validation:**
- `cargo build` - verify no warnings on success
- `cargo run -- repl` - verify clean startup (dev mode still shows Finished/Running)
- `cargo build --release && ./target/release/tq repl` - verify completely clean

### Implementation Checklist

#### Feature #7: Interactive Horizontal Paging
- [ ] Add `INDICATOR_WIDTH` constant
- [ ] Add `hidden_columns_left()` method
- [ ] Add `hidden_columns_right()` method
- [ ] Update `visible_column_count()` to account for indicator width
- [ ] Update `render_border()` for indicator columns
- [ ] Update `render_header()` for indicator cells
- [ ] Update `render_row()` for indicator cells
- [ ] Update `render_status_bar()` nav hints
- [ ] Add pager section to `/help` output
- [ ] Add unit tests for indicator calculations
- [ ] Manual testing with wide result sets

#### Feature #11: Clean Startup
- [ ] Remove success warning from `build.rs`
- [ ] Verify failure warning still works
- [ ] Test `cargo build` output
- [ ] Test `cargo run -- repl` output
- [ ] Test release build startup

---

## Sprint 33: Pager Bug Fix and Data Sampling Commands

This section documents the technical design for Sprint 33, which addresses the pager rendering bug (Issue #14) and implements data sampling commands (`/sample`, `/peek`).

### Pager Bug Fix - Root Cause Analysis

**GitHub Issue:** #14 - [BUG] Pager broken and on by default

**Problem Statement:** Despite Sprint 31's two-pass truncation fix, the pager still produces garbled output with misaligned columns and improper line breaks.

#### Root Cause #1: format! Width Specifier Uses Character Count, Not Display Width

The pager's cell rendering uses Rust's `format!` macro with width specifier:

```rust
// Current code (render_row, lines 512-516)
let padded = match col.alignment {
    Alignment::Right => format!(" {:>width$} ", value, width = col.display_width),
    Alignment::Center => format!(" {:^width$} ", value, width = col.display_width),
    Alignment::Left => format!(" {:width$} ", value, width = col.display_width),
};
```

**The Bug:** `format!` pads to `width` **characters**, not `width` display columns. But `display_width` was calculated using `UnicodeWidthStr::width()` which returns **visual width**.

**For ASCII data:** This works because character count equals visual width.

**For wide characters (CJK, emoji):** This creates misalignment:
- "日本" has 2 chars but 4 visual width
- `format!("{:10}", "日本")` adds 8 spaces (to reach 10 chars)
- Result: 4 visual + 8 spaces = 12 visual width (expected 10)

**Demonstration:**
```
ASCII "aaaaa" (5 chars, 5 visual) padded to 10:
"aaaaa     " = 10 visual width (correct)

CJK "日本語" (3 chars, 6 visual) padded to 10:
"日本語       " = 13 visual width (WRONG - expected 10)
```

**Fix:** Replace `format!` width specifier with manual display-width-aware padding:

```rust
/// Pad a string to the specified display width
fn pad_to_display_width(value: &str, width: usize, alignment: Alignment) -> String {
    let visual_width = UnicodeWidthStr::width(value);
    let padding = width.saturating_sub(visual_width);
    match alignment {
        Alignment::Left => format!(" {}{} ", value, " ".repeat(padding)),
        Alignment::Right => format!(" {}{} ", " ".repeat(padding), value),
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!(" {}{}{} ", " ".repeat(left), value, " ".repeat(right))
        }
    }
}
```

#### Root Cause #2: Event Loop Bug

The `run()` method has a bug where it calls `event::read()` twice after a single `event::poll()`:

```rust
// Current code (run, lines 916-928) - BUGGY
if event::poll(std::time::Duration::from_millis(100))? {
    if let Event::Key(key) = event::read()? {
        // handle key...
    }
    if let Event::Resize(w, h) = event::read().unwrap_or(Event::FocusGained) {
        // handle resize...
    }
}
```

After `poll()` returns true, there's only ONE event in the queue. The second `event::read()` either blocks or fails.

**Fix:** Use a single `match` statement:

```rust
// Fixed code
if event::poll(std::time::Duration::from_millis(100))? {
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
        _ => {}
    }
}
```

#### Why Tests Passed But Real Data Failed

1. **Tests use ASCII-only data** where character count equals visual width
2. **Tests use `render_to_buffer()`** which doesn't involve actual terminal rendering
3. **Tests mock terminal width** rather than using real terminal detection
4. **Real Teradata data** may contain non-ASCII characters or data patterns that expose the width mismatch

#### Mitigation Strategy

1. **Immediate:** Disable pager by default (`pager_enabled: false` in state.rs)
2. **Fix #1:** Implement display-width-aware padding function
3. **Fix #2:** Correct the event loop bug
4. **Testing:** Add tests with Unicode/CJK data to catch width calculation issues
5. **User option:** Users can still enable pager with `/pager on` if desired

#### Files to Modify

| File | Change |
|------|--------|
| `src/commands/repl/state.rs` | Set `pager_enabled: false` |
| `src/commands/repl/pager.rs` | Add `pad_to_display_width()` function |
| `src/commands/repl/pager.rs` | Update `render_row()` to use new padding |
| `src/commands/repl/pager.rs` | Update `render_header()` to use new padding |
| `src/commands/repl/pager.rs` | Fix event loop in `run()` |
| `src/commands/repl/pager.rs` | Add Unicode width tests |

---

### Data Sampling Commands Design

**Related Specification:** `docs/specifications/repl.md#data-sampling-commands` (REQ-SAMPLE-001 through REQ-SAMPLE-015)

#### Overview

Data sampling commands (`/sample` and `/peek`) provide fast exploratory data analysis without writing full SQL queries. They target data analysts and DBAs who need quick table inspection during REPL sessions.

| Command | Purpose | SQL Generated |
|---------|---------|---------------|
| `/sample <table> [n]` | Random sample (default 10 rows) | `SELECT * FROM <table> SAMPLE <n>` |
| `/peek <table>` | First 5 rows + column info | `SELECT TOP 5 * FROM <table>` |

#### Architecture

```
User Input: /sample employees 20
       │
       v
Metacommand Parser (mod.rs)
       │
       v
SampleCommand::execute()
       │
       ├─→ Parse arguments (table name, optional count)
       │
       ├─→ Validate table name (qualified or unqualified)
       │
       ├─→ Validate sample size (1-1000)
       │
       ├─→ Generate SQL: SELECT * FROM employees SAMPLE 20
       │
       ├─→ Execute query via connection
       │
       ├─→ Format result (table/csv/json based on current format)
       │
       └─→ Display with header/footer
```

#### Implementation Location

```
src/commands/repl/
├── mod.rs              # Add /sample and /peek to metacommand dispatch
├── metacommands.rs     # Implement SampleCommand and PeekCommand
└── executor.rs         # Reuse existing query execution infrastructure
```

#### SampleCommand Implementation

```rust
/// /sample <table> [n] - Display random sample of rows
pub fn handle_sample(
    args: &str,
    conn: &mut Connection,
    state: &mut ReplState,
) -> Result<MetacommandResult, Box<dyn std::error::Error + Send + Sync>> {
    // Parse arguments
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Usage: /sample <table> [n]\nExample: /sample employees 20".into());
    }

    let table_name = parts[0];
    let sample_size: usize = parts.get(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(10);

    // Validate sample size (REQ-SAMPLE-003)
    if sample_size == 0 || sample_size > 1000 {
        return Err(format!(
            "Sample size must be between 1 and 1000 (got {})\n\
             Example: /sample {} 100",
            sample_size, table_name
        ).into());
    }

    // Resolve qualified table name (REQ-SAMPLE-005)
    let qualified_name = resolve_table_name(table_name, state)?;

    // Generate and execute SQL (REQ-SAMPLE-002)
    let sql = format!("SELECT * FROM {} SAMPLE {}", qualified_name, sample_size);
    let result = execute_query(conn, &sql)?;

    // Display with header (REQ-SAMPLE-014)
    println!("\nRandom sample from {} ({} rows):", qualified_name, result.row_count);
    display_result(&result, state)?;
    println!("{} rows sampled (Query time: {:.3}s)",
             result.row_count,
             result.execution_time.as_secs_f64());

    Ok(MetacommandResult::Continue)
}
```

#### PeekCommand Implementation

```rust
/// /peek <table> - Display first 5 rows with column metadata
pub fn handle_peek(
    args: &str,
    conn: &mut Connection,
    state: &mut ReplState,
) -> Result<MetacommandResult, Box<dyn std::error::Error + Send + Sync>> {
    let table_name = args.trim();
    if table_name.is_empty() {
        return Err("Usage: /peek <table>\nExample: /peek employees".into());
    }

    // Resolve qualified table name
    let qualified_name = resolve_table_name(table_name, state)?;

    // Display column metadata (REQ-SAMPLE-004.3, 004.4)
    println!("\nTable: {}", qualified_name);

    // Fetch and display column info (reuse /describe infrastructure)
    let column_info = fetch_column_metadata(conn, &qualified_name)?;
    display_column_metadata(&column_info)?;

    // Fetch first 5 rows (REQ-SAMPLE-004.1, 004.2)
    let sql = format!("SELECT TOP 5 * FROM {}", qualified_name);
    let result = execute_query(conn, &sql)?;

    if result.row_count == 0 {
        println!("\nTable is empty");
    } else {
        println!("\nFirst {} rows:", result.row_count);
        display_result(&result, state)?;
    }

    println!("(Query time: {:.3}s)", result.execution_time.as_secs_f64());

    Ok(MetacommandResult::Continue)
}
```

#### Table Name Resolution

```rust
/// Resolve table name to fully qualified form
fn resolve_table_name(
    name: &str,
    state: &ReplState
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if name.contains('.') {
        // Already qualified: database.table
        Ok(name.to_string())
    } else {
        // Need current database
        let current_db = state.connection_info().database.clone();
        if current_db.is_empty() {
            return Err(format!(
                "No current database. Use qualified name: /sample DATABASE.{}",
                name
            ).into());
        }
        Ok(format!("{}.{}", current_db, name))
    }
}
```

#### Tab Completion Integration (REQ-SAMPLE-009)

Add to `MetadataCompleter`:

```rust
// In metadata_completer.rs

fn complete_metacommand_args(&self, cmd: &str, partial: &str) -> Vec<Suggestion> {
    match cmd {
        "/sample" | "/peek" => {
            // Complete table names from current database
            self.complete_table_names(partial)
        }
        _ => vec![]
    }
}
```

#### Help Text Integration (REQ-SAMPLE-010)

Add to metacommand help:

```rust
MetacommandDef {
    name: "sample",
    aliases: &[],
    description: "Show random sample of rows from table"
},
MetacommandDef {
    name: "peek",
    aliases: &[],
    description: "Show first 5 rows and column info"
},
```

#### Batch Mode Integration (REQ-SAMPLE-011)

Add subcommands to CLI:

```rust
// In cli.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Show random sample of rows from table
    Sample {
        /// Table name (database.table or table)
        table: String,
        /// Number of rows to sample (default: 10, max: 1000)
        #[arg(default_value = "10")]
        count: usize,
    },

    /// Show first rows and column info
    Peek {
        /// Table name (database.table or table)
        table: String,
    },
}
```

#### Error Handling (REQ-SAMPLE-006, 007)

```rust
/// Handle sampling errors with user-friendly messages
fn handle_sample_error(
    err: &TeradataError,
    table_name: &str
) -> String {
    match err.code() {
        Some(3807) => format!(
            "Table '{}' not found.\n\
             Use /list tables to see available tables.",
            table_name
        ),
        Some(3523) => format!(
            "Permission denied on '{}'.\n\
             You need SELECT privilege on this table.\n\
             Contact your DBA or try: GRANT SELECT ON {} TO <your_user>;",
            table_name, table_name
        ),
        _ => format!("Error sampling '{}': {}", table_name, err)
    }
}
```

#### Code Linkage

| Component | Location | Function |
|-----------|----------|----------|
| Metacommand dispatch | `mod.rs` | Add `/sample`, `/peek` to match |
| Sample handler | `metacommands.rs` | `handle_sample()` |
| Peek handler | `metacommands.rs` | `handle_peek()` |
| Table resolution | `metacommands.rs` | `resolve_table_name()` |
| Tab completion | `metadata_completer.rs` | `complete_metacommand_args()` |
| Batch commands | `cli.rs` | `Commands::Sample`, `Commands::Peek` |
| Batch handlers | `main.rs` | Route to new command handlers |

#### Design Trade-offs

**Sample vs. Random Selection:**
- Using `SAMPLE n` instead of `ORDER BY RANDOM() LIMIT n`
- SAMPLE is Teradata-specific but much more efficient (no sort)
- Non-deterministic: each execution may return different rows

**Peek Row Count:**
- Fixed at 5 rows (not configurable) per spec
- Rationale: `/peek` is for quick inspection, use `/sample` for more rows
- Reduces complexity and user confusion

**Column Metadata Display:**
- Reuses `/describe` infrastructure for column info
- Displayed before data rows for context
- Includes: name, type, nullable, precision (if applicable)

---

## Shared Utilities for SQL Generation

This section documents the shared utilities for SQL generation and Teradata type formatting, enabling consistent behavior across REPL metacommands and batch mode commands.

### Overview

The data sampling commands (`/sample`, `/peek`) require common functionality for:
1. **Teradata type formatting**: Converting Teradata type codes to human-readable strings
2. **SQL identifier quoting**: Preventing SQL injection and handling special characters in identifiers
3. **SQL string escaping**: Escaping single quotes in string literals

These utilities are shared between:
- `src/commands/sample.rs` (batch mode)
- `src/commands/repl/metacommands.rs` (REPL mode)
- `src/db/metadata.rs` (metadata queries)

### Module Organization

```
src/sql/
├── mod.rs              # Module exports
├── parser.rs           # Statement parsing (existing)
├── identifiers.rs      # SQL identifier utilities (NEW)
└── types.rs            # Teradata type formatting (NEW)
```

The `src/sql/` module is the natural home for SQL-related utilities, extending the existing `parser.rs` functionality.

### Teradata Type Formatting

The `format_column_type()` function converts Teradata type codes from DBC.ColumnsV to human-readable type strings.

```rust
// src/sql/types.rs

/// Format Teradata column type from type code and dimensions.
///
/// Converts Teradata internal type codes (from DBC.ColumnsV) to
/// human-readable SQL type names with appropriate precision/scale.
///
/// # Arguments
/// * `type_code` - Teradata type code (e.g., "CV", "I", "D")
/// * `length` - Column length (for VARCHAR, CHAR, BYTE types)
/// * `precision` - Decimal total digits (for DECIMAL type)
/// * `scale` - Decimal fractional digits (for DECIMAL type)
///
/// # Examples
/// ```
/// use tq::sql::types::format_column_type;
///
/// assert_eq!(format_column_type("CV", Some(100), None, None), "VARCHAR(100)");
/// assert_eq!(format_column_type("I", None, None, None), "INTEGER");
/// assert_eq!(format_column_type("D", None, Some(10), Some(2)), "DECIMAL(10,2)");
/// ```
pub fn format_column_type(
    type_code: &str,
    length: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> String {
    match type_code.trim() {
        // Character types
        "CV" => format!("VARCHAR({})", length.unwrap_or(0)),
        "CF" => format!("CHAR({})", length.unwrap_or(0)),

        // Integer types
        "I" => "INTEGER".to_string(),
        "I1" => "BYTEINT".to_string(),
        "I2" => "SMALLINT".to_string(),
        "I8" => "BIGINT".to_string(),

        // Numeric types
        "D" => {
            if let (Some(p), Some(s)) = (precision, scale) {
                format!("DECIMAL({},{})", p, s)
            } else {
                "DECIMAL".to_string()
            }
        }
        "F" => "FLOAT".to_string(),

        // Date/time types
        "DA" => "DATE".to_string(),
        "TS" => "TIMESTAMP".to_string(),
        "TZ" => "TIMESTAMP WITH TIME ZONE".to_string(),
        "AT" => "TIME".to_string(),

        // Binary types
        "BV" => format!("VARBYTE({})", length.unwrap_or(0)),
        "BF" => format!("BYTE({})", length.unwrap_or(0)),

        // LOB types
        "CO" => "CLOB".to_string(),
        "BO" => "BLOB".to_string(),

        // Special types
        "JN" => "JSON".to_string(),

        // Unknown - pass through
        other => other.to_string(),
    }
}
```

**Teradata Type Code Reference:**

| Code | Type | Dimensions |
|------|------|------------|
| CV | VARCHAR | length |
| CF | CHAR | length |
| I | INTEGER | none |
| I1 | BYTEINT | none |
| I2 | SMALLINT | none |
| I8 | BIGINT | none |
| D | DECIMAL | precision, scale |
| F | FLOAT | none |
| DA | DATE | none |
| TS | TIMESTAMP | none |
| TZ | TIMESTAMP WITH TIME ZONE | none |
| AT | TIME | none |
| BV | VARBYTE | length |
| BF | BYTE | length |
| CO | CLOB | none |
| BO | BLOB | none |
| JN | JSON | none |

### SQL Identifier Quoting

The `quote_identifier()` function properly quotes SQL identifiers to prevent injection and handle special characters.

```rust
// src/sql/identifiers.rs

/// Quote a SQL identifier for safe use in dynamic SQL.
///
/// Teradata uses double quotes for delimited identifiers. This function:
/// 1. Wraps the identifier in double quotes
/// 2. Escapes any existing double quotes by doubling them
///
/// # Arguments
/// * `identifier` - The identifier (database, table, or column name)
///
/// # Examples
/// ```
/// use tq::sql::identifiers::quote_identifier;
///
/// // Simple identifier
/// assert_eq!(quote_identifier("employees"), "\"employees\"");
///
/// // Identifier with spaces
/// assert_eq!(quote_identifier("my table"), "\"my table\"");
///
/// // Identifier with quotes
/// assert_eq!(quote_identifier("my\"table"), "\"my\"\"table\"");
/// ```
pub fn quote_identifier(identifier: &str) -> String {
    // Escape embedded double quotes by doubling them
    let escaped = identifier.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Quote a fully qualified table name (database.table).
///
/// Parses the qualified name and quotes each component separately.
///
/// # Arguments
/// * `qualified_name` - The table name, optionally with database prefix
///
/// # Examples
/// ```
/// use tq::sql::identifiers::quote_qualified_name;
///
/// assert_eq!(quote_qualified_name("db.table"), "\"db\".\"table\"");
/// assert_eq!(quote_qualified_name("my db.my table"), "\"my db\".\"my table\"");
/// ```
pub fn quote_qualified_name(qualified_name: &str) -> String {
    if let Some(dot_pos) = qualified_name.find('.') {
        let database = &qualified_name[..dot_pos];
        let table = &qualified_name[dot_pos + 1..];
        format!("{}.{}", quote_identifier(database), quote_identifier(table))
    } else {
        quote_identifier(qualified_name)
    }
}
```

**Quoting Rules:**

1. **Always quote**: All identifiers in generated SQL should be quoted to prevent injection
2. **Escape embedded quotes**: Double any existing `"` characters within the identifier
3. **Preserve case**: Teradata preserves case for quoted identifiers
4. **Handle qualified names**: Parse `database.table` and quote each component

### SQL String Escaping

The `escape_sql_string()` function escapes single quotes in string literals for WHERE clauses.

```rust
// src/sql/identifiers.rs

/// Escape a string value for use in SQL WHERE clauses.
///
/// This is for string LITERALS (values), not identifiers.
/// Escapes single quotes by doubling them.
///
/// # Arguments
/// * `value` - The string value to escape
///
/// # Examples
/// ```
/// use tq::sql::identifiers::escape_sql_string;
///
/// assert_eq!(escape_sql_string("O'Brien"), "O''Brien");
/// assert_eq!(escape_sql_string("test"), "test");
/// ```
pub fn escape_sql_string(value: &str) -> String {
    value.replace('\'', "''")
}
```

**Important Distinction:**

| Function | Purpose | Delimiter | Example Input | Example Output |
|----------|---------|-----------|---------------|----------------|
| `quote_identifier()` | Table/column names | Double quotes | `my table` | `"my table"` |
| `escape_sql_string()` | String literals in WHERE | Single quotes | `O'Brien` | `O''Brien` |

### Integration Pattern

Commands that generate SQL should use these utilities consistently:

```rust
// src/commands/sample.rs

use crate::sql::identifiers::{quote_qualified_name, escape_sql_string};
use crate::sql::types::format_column_type;

pub fn execute_sample(...) -> Result<()> {
    // Quote table name for safe SQL
    let quoted_table = quote_qualified_name(&qualified_name);

    // Generate safe SQL
    let sql = format!(
        "SELECT * FROM {} SAMPLE {}",
        quoted_table, sample_size
    );

    // For metadata queries, escape string values
    let columns_sql = format!(
        "SELECT ColumnName, ColumnType FROM DBC.ColumnsV \
         WHERE DatabaseName = '{}'",
        escape_sql_string(database)
    );

    // Format column types for display
    for row in result.rows {
        let type_display = format_column_type(&type_code, length, precision, scale);
    }
}
```

### Security Considerations

1. **SQL Injection Prevention**: Always use `quote_identifier()` for table/column names in generated SQL
2. **No User SQL in Metacommands**: The `/sample` and `/peek` commands construct SQL programmatically, never executing user-provided SQL directly
3. **String Values**: Use `escape_sql_string()` for WHERE clause values when querying system views
4. **Defense in Depth**: Even though tq is a local CLI tool, proper quoting prevents accidents with unusual table names

### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Type formatting | `src/sql/types.rs` | `format_column_type()` |
| Identifier quoting | `src/sql/identifiers.rs` | `quote_identifier()`, `quote_qualified_name()` |
| String escaping | `src/sql/identifiers.rs` | `escape_sql_string()` |
| Module exports | `src/sql/mod.rs` | Re-exports for public API |
| Batch sample | `src/commands/sample.rs` | Uses shared utilities |
| REPL sample | `src/commands/repl/metacommands.rs` | Uses shared utilities |
| Metadata queries | `src/db/metadata.rs` | Uses `escape_sql_string()` |

### Migration Plan

1. **Create new modules**: Add `src/sql/types.rs` and `src/sql/identifiers.rs`
2. **Move functions**: Extract `format_column_type()` from `sample.rs`, `escape_sql_string()` from `metacommands.rs` and `metadata.rs`
3. **Update imports**: Change imports in consuming modules to use shared utilities
4. **Add identifier quoting**: Update SQL generation to use `quote_identifier()`
5. **Consolidate tests**: Move tests to new modules, remove duplicates

---

## Query Editing Commands

### `/repeat` Command

The `/repeat` metacommand re-executes the most recently executed SQL query without requiring the user to retype or recall it from history.

**Related Specification**: `docs/specifications/repl.md` (Query Editing section)

#### Design Approach

The `/repeat` command leverages existing infrastructure in `ReplState` that already tracks the last executed SQL query.

#### Implementation Details

**State Tracking:**
```rust
// src/commands/repl/state.rs

pub struct ReplState {
    // ... other fields ...

    /// Last SQL query executed (for /export and /repeat)
    last_sql: Option<String>,

    /// Whether last result was limited by default REPL limit
    was_limited: bool,
}

impl ReplState {
    /// Get the last SQL query
    pub fn last_sql(&self) -> Option<&str> {
        self.last_sql.as_deref()
    }

    /// Set the last SQL query and whether it was limited
    pub fn set_last_query(&mut self, sql: String, was_limited: bool) {
        self.last_sql = Some(sql);
        self.was_limited = was_limited;
    }
}
```

**Command Handler:**
```rust
// src/commands/repl/metacommands.rs

pub fn handle_metacommand_with_state<W: Write>(
    input: &str,
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<bool> {
    // ... existing command parsing ...

    match command.as_str() {
        // ... other commands ...

        "repeat" | "r" => {
            execute_repeat(state, completion_state, writer)?;
        }

        // ... remaining commands ...
    }

    Ok(true)
}

/// Execute the /repeat metacommand
///
/// Re-executes the most recently executed SQL query.
fn execute_repeat<W: Write>(
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    // Check if there's a previous query
    let sql = match state.last_sql() {
        Some(s) => s.to_string(),
        None => {
            writeln!(writer)?;
            writeln!(writer, "No previous query to repeat")?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    // Display what we're repeating
    writeln!(writer)?;
    writeln!(writer, "Repeating: {}", sql)?;
    writeln!(writer)?;

    // Execute the query using standard execution pipeline
    let result = completion_state.client().execute(&sql)?;

    // Format and display results (same as normal query execution)
    let formatter = crate::output::TableFormatter::new();
    let output = formatter.format(&result)?;

    if state.is_pager_enabled() {
        crate::commands::repl::pager::display_with_pager(&output, result.row_count)?;
    } else {
        write!(writer, "{}", output)?;
    }

    // Update state (keep last_sql unchanged - it's the same query)
    state.set_last_result(result);

    Ok(())
}
```

#### Tab Completion

The `/repeat` command is added to the metacommand completion list:

```rust
// src/commands/repl/metadata_completer.rs

const METACOMMANDS: &[(&str, &str)] = &[
    // ... existing metacommands ...
    ("/repeat", "Re-execute last query"),
    // ... remaining metacommands ...
];
```

Short alias `\r` is supported through the command parser:
```rust
match command.as_str() {
    "repeat" | "r" => { /* ... */ }
}
```

#### Help Text

Updated help output:
```rust
fn print_help_extended<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "Query Editing:")?;
    writeln!(writer, "  /repeat, /r            Re-execute last query")?;
    // ... rest of help ...
}
```

#### Usage Examples

```sql
tq> SELECT COUNT(*) FROM employees WHERE department = 'IT';
+-------+
| count |
+-------+
|   142 |
+-------+
(1 row)

tq> /repeat
Repeating: SELECT COUNT(*) FROM employees WHERE department = 'IT'

+-------+
| count |
+-------+
|   142 |
+-------+
(1 row)

tq> \r
Repeating: SELECT COUNT(*) FROM employees WHERE department = 'IT'
[same output]
```

**No previous query:**
```sql
tq> /repeat

No previous query to repeat

tq>
```

#### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| State tracking | `src/commands/repl/state.rs` | `last_sql()`, `set_last_query()` |
| Command handler | `src/commands/repl/metacommands.rs` | `execute_repeat()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | METACOMMANDS list |
| Help text | `src/commands/repl/metacommands.rs` | `print_help_extended()` |

#### Error Handling

1. **No previous query**: Show clear message "No previous query to repeat"
2. **Query execution fails**: Standard error handling (connection lost, syntax error, permission denied) - same as regular query execution
3. **State consistency**: Last SQL is preserved on error - user can fix connection and retry

---

### `/edit` Command (Sprint 37)

The `/edit` metacommand opens the last executed SQL query in an external editor, allowing users to modify and re-execute it. This is the natural companion to `/repeat`, completing the query editing feature set.

**Related Specification**: `docs/specifications/repl.md` (Query Editing section, line 3175)

#### Design Approach

The `/edit` command follows a five-step workflow:

1. **Retrieve** the last SQL query from `ReplState.last_sql`
2. **Create** a temporary file with `.sql` extension
3. **Launch** the user's preferred editor (resolved from environment)
4. **Read** the edited content after editor exits
5. **Execute** the modified query (if changed and non-empty)

This design leverages the same state management infrastructure as `/repeat` and follows established patterns for temporary file handling and external process management.

#### Implementation Details

**Editor Resolution Chain:**

The command resolves the editor in the following priority order:

```rust
// src/commands/repl/metacommands.rs

/// Resolve the editor to use for /edit command
///
/// Priority: $VISUAL → $EDITOR → vi (fallback)
fn resolve_editor() -> Result<String> {
    if let Ok(visual) = std::env::var("VISUAL") {
        if !visual.trim().is_empty() {
            return Ok(visual);
        }
    }

    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.trim().is_empty() {
            return Ok(editor);
        }
    }

    // Fallback to vi (available on all UNIX-like systems)
    Ok("vi".to_string())
}
```

**Temporary File Management:**

Uses the `tempfile` crate (already a project dependency) to create a secure temporary file with proper cleanup semantics:

```rust
use tempfile::Builder;

/// Create temporary file with .sql extension for editor
///
/// The file is created with a descriptive prefix and .sql extension
/// for proper syntax highlighting in editors.
fn create_temp_sql_file(content: &str) -> Result<(tempfile::NamedTempFile, PathBuf)> {
    let temp_file = Builder::new()
        .prefix("tq_edit_")
        .suffix(".sql")
        .tempfile()
        .map_err(|e| Error::from(format!("Failed to create temp file: {}", e)))?;

    // Write last SQL to temp file
    let path = temp_file.path().to_path_buf();
    std::fs::write(&path, content)
        .map_err(|e| Error::from(format!("Failed to write temp file: {}", e)))?;

    Ok((temp_file, path))
}
```

**Editor Launch and Exit Code Handling:**

```rust
use std::process::Command;

/// Launch editor and wait for exit
///
/// Returns Ok(()) if editor exits successfully (exit code 0),
/// Err otherwise.
fn launch_editor(editor: &str, file_path: &Path) -> Result<()> {
    let status = Command::new(editor)
        .arg(file_path)
        .status()
        .map_err(|e| Error::from(format!("Failed to launch editor '{}': {}", editor, e)))?;

    if !status.success() {
        return Err(Error::from(format!(
            "Editor '{}' exited with non-zero status: {}",
            editor,
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}
```

**Change Detection:**

```rust
/// Check if edited content differs from original
fn content_changed(original: &str, edited: &str) -> bool {
    original.trim() != edited.trim()
}
```

**Command Handler:**

```rust
// src/commands/repl/metacommands.rs

pub fn handle_metacommand_with_state<W: Write>(
    input: &str,
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<bool> {
    // ... existing command parsing ...

    match command.as_str() {
        // ... other commands ...

        "edit" | "e" => {
            execute_edit(state, completion_state, writer)?;
        }

        // ... remaining commands ...
    }

    Ok(true)
}

/// Execute the /edit metacommand (Sprint 37)
///
/// Opens the last SQL query in an external editor, then executes
/// the modified query if changes were made.
fn execute_edit<W: Write>(
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    // 1. Check if there's a previous query
    let original_sql = match state.last_sql() {
        Some(s) => s.to_string(),
        None => {
            writeln!(writer, "No previous query to edit.")?;
            return Ok(());
        }
    };

    // 2. Resolve editor
    let editor = match resolve_editor() {
        Ok(e) => e,
        Err(e) => {
            writeln!(writer, "Error: {}", e)?;
            writeln!(writer, "Set $EDITOR or $VISUAL environment variable.")?;
            return Ok(());
        }
    };

    // 3. Create temp file with original SQL
    let (_temp_file, temp_path) = create_temp_sql_file(&original_sql)?;

    // 4. Launch editor
    writeln!(writer, "Opening editor: {}", editor)?;
    if let Err(e) = launch_editor(&editor, &temp_path) {
        writeln!(writer, "Error: {}", e)?;
        return Ok(());
    }

    // 5. Read edited content
    let edited_sql = std::fs::read_to_string(&temp_path)
        .map_err(|e| Error::from(format!("Failed to read edited file: {}", e)))?;

    // 6. Check if content changed
    if !content_changed(&original_sql, &edited_sql) {
        writeln!(writer, "No changes made.")?;
        return Ok(());
    }

    // 7. Check if result is empty
    let trimmed = edited_sql.trim();
    if trimmed.is_empty() {
        writeln!(writer, "Edited query is empty. No execution.")?;
        return Ok(());
    }

    // 8. Execute the edited query
    writeln!(writer)?;
    writeln!(writer, "Executing edited query:")?;
    writeln!(writer, "{}", trimmed)?;
    writeln!(writer)?;

    let default_limit = state.default_limit();
    let client = completion_state.client();

    match execute_sql_with_state(client, state, trimmed, writer, default_limit) {
        Ok(row_count) => {
            // Store edited query as new last_sql (enables /repeat after /edit)
            state.set_last_query(trimmed.to_string(), default_limit > 0);
            state.record_query(row_count);
        }
        Err(e) => {
            writeln!(writer, "\nError: {}", e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}
```

#### Tab Completion

The `/edit` command is added to the metacommand completion list with its alias:

```rust
// src/commands/repl/metadata_completer.rs

const METACOMMANDS: &[MetacommandDef] = &[
    // ... existing metacommands ...
    MetacommandDef {
        name: "edit",
        aliases: &["e"],
        description: "Edit last query in $EDITOR",
    },
    // ... remaining metacommands ...
];
```

The alias `\e` is supported through the command parser:
```rust
match command.as_str() {
    "edit" | "e" => { /* ... */ }
}
```

#### Help Text

Updated help output to include `/edit`:

```rust
fn print_help_extended<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "Query Editing:")?;
    writeln!(writer, "  /edit, /e              Edit last query in $EDITOR")?;
    writeln!(writer, "  /repeat, /r            Re-execute last query")?;
    // ... rest of help ...
}
```

#### Usage Examples

**Normal workflow:**
```sql
tq> SELECT COUNT(*) FROM employees WHERE status = 'active';
+-------+
| count |
+-------+
|   142 |
+-------+
(1 row)

tq> /edit
Opening editor: vim
[vim opens with: SELECT COUNT(*) FROM employees WHERE status = 'active']
[user edits to: SELECT COUNT(*) FROM employees WHERE status = 'inactive']
[user saves and exits]

Executing edited query:
SELECT COUNT(*) FROM employees WHERE status = 'inactive'

+-------+
| count |
+-------+
|    18 |
+-------+
(1 row)

tq> /repeat
Repeating: SELECT COUNT(*) FROM employees WHERE status = 'inactive'
[executes the edited version]
```

**Short alias:**
```sql
tq> SELECT * FROM customers LIMIT 5;
[results...]

tq> \e
Opening editor: nano
[nano opens with query]
```

**No previous query:**
```sql
tq> /edit
No previous query to edit.
tq>
```

**No changes made:**
```sql
tq> SELECT 1;
+---+
| 1 |
+---+
| 1 |
+---+

tq> /edit
Opening editor: vi
[user exits without making changes]
No changes made.
tq>
```

**Empty after edit:**
```sql
tq> SELECT * FROM test;
[results...]

tq> /edit
Opening editor: emacs
[user deletes all content and saves]
Edited query is empty. No execution.
tq>
```

**Editor not found:**
```sql
tq> /edit
Error: Failed to launch editor 'nonexistent': No such file or directory
Set $EDITOR or $VISUAL environment variable.
tq>
```

#### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| State tracking | `src/commands/repl/state.rs` | `last_sql()`, `set_last_query()` |
| Command handler | `src/commands/repl/metacommands.rs` | `execute_edit()`, `resolve_editor()`, `create_temp_sql_file()`, `launch_editor()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | METACOMMANDS list |
| Help text | `src/commands/repl/metacommands.rs` | `print_help_extended()` |
| SQL execution | `src/commands/repl/executor.rs` | `execute_sql_with_state()` |

#### Error Handling

1. **No previous query**: Show clear message "No previous query to edit."
2. **Editor not found**: Show error with suggestion to set $EDITOR or $VISUAL
3. **Editor exits with error**: Show exit status, don't execute query
4. **Temp file creation fails**: Show descriptive error (disk full, permissions)
5. **Temp file read fails**: Show descriptive error
6. **Empty result**: Show message, don't execute
7. **No changes**: Show message, don't execute
8. **Query execution fails**: Standard error handling (same as regular query execution)

#### Integration with `/repeat`

The edited query is stored as `last_sql`, enabling seamless workflow:

```sql
tq> SELECT * FROM orders WHERE date = '2024-01-01';
[results for Jan 1]

tq> /edit
[edit date to '2024-01-02']
[results for Jan 2]

tq> /repeat
[results for Jan 2 again]

tq> /edit
[edit date to '2024-01-03']
[results for Jan 3]
```

Each `/edit` updates `last_sql`, so `/repeat` always executes the most recent query (whether originally typed or edited).

#### Cross-Platform Considerations

**Editor Fallback:**
- On UNIX-like systems (Linux, macOS): fallback to `vi` (universally available)
- On Windows: the fallback to `vi` may fail if not installed; user must set $EDITOR

**Path Handling:**
- Use `PathBuf` and `Path` for cross-platform path manipulation
- `tempfile` crate handles platform-specific temp directory resolution

**Process Spawning:**
- `std::process::Command` is cross-platform
- Exit code checking works consistently across platforms

#### Testing Strategy

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_resolve_editor_visual() {
        // Set VISUAL and verify it takes priority
    }

    #[test]
    fn test_resolve_editor_editor() {
        // Clear VISUAL, set EDITOR, verify fallback
    }

    #[test]
    fn test_resolve_editor_fallback() {
        // Clear both, verify vi fallback
    }

    #[test]
    fn test_content_changed() {
        // Verify change detection (trim whitespace differences)
    }

    #[test]
    fn test_edit_no_previous_query() {
        // Verify error message when last_sql is None
    }

    #[test]
    fn test_help_includes_edit_command() {
        // Verify /edit appears in help text
    }
}
```

**Integration Tests:**
- Mock editor using shell script that modifies the file
- Test full workflow: create query → edit → verify execution
- Test edge cases: empty file, no changes, invalid SQL

---

## Schema Inspection Commands

### `/show indexes` Command

The `/show indexes <table>` metacommand displays index information for a specified table by querying the Teradata system catalog.

**Related Specification**: `docs/specifications/repl.md` (Schema Inspection Commands)

#### Design Approach

The `/show indexes` command follows the established pattern used by `/describe`, querying Teradata's `DBC.IndicesV` system view to retrieve index metadata.

#### Teradata Catalog Query

Teradata stores index information in the `DBC.IndicesV` view, which provides comprehensive details about all indexes in the database.

**Key columns:**
- `IndexNumber` - Unique index identifier
- `IndexType` - Type code (P=Primary, S=Secondary, K=Primary Key, U=Unique, V=Value-ordered, etc.)
- `ColumnName` - Column included in the index
- `ColumnPosition` - Position of column within the index
- `UniqueFlag` - Whether index enforces uniqueness (Y/N)

**Query structure:**
```sql
SELECT
    IndexNumber,
    CASE IndexType
        WHEN 'P' THEN 'Primary Index'
        WHEN 'Q' THEN 'Partitioned Primary Index'
        WHEN 'S' THEN 'Secondary Index'
        WHEN 'K' THEN 'Primary Key'
        WHEN 'U' THEN 'Unique Constraint'
        WHEN 'V' THEN 'Value-ordered Secondary Index'
        WHEN 'H' THEN 'Hash-ordered Covering Secondary Index'
        ELSE IndexType
    END AS IndexType,
    ColumnName,
    ColumnPosition,
    CASE UniqueFlag
        WHEN 'Y' THEN 'Unique'
        ELSE 'Not Unique'
    END AS Uniqueness
FROM DBC.IndicesV
WHERE DatabaseName = <database>
  AND TableName = <table>
ORDER BY IndexNumber, ColumnPosition
```

#### Implementation Details

**Command Handler:**
```rust
// src/commands/repl/metacommands.rs

pub fn handle_metacommand_with_state<W: Write>(
    input: &str,
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<bool> {
    // ... existing command parsing ...

    match command.as_str() {
        // ... other commands ...

        "show" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /show <subcommand> [options]")?;
                writeln!(writer)?;
                writeln!(writer, "Subcommands:")?;
                writeln!(writer, "  indexes <table>    Display index information")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /show indexes employees")?;
                writeln!(writer, "  /show indexes mydb.customers")?;
                writeln!(writer)?;
            } else if args[0].to_lowercase() == "indexes" {
                if args.len() < 2 {
                    writeln!(writer, "Usage: /show indexes <table_name>")?;
                    writeln!(writer, "       /show indexes <database>.<table_name>")?;
                } else {
                    execute_show_indexes(completion_state, args[1], writer)?;
                }
            } else {
                writeln!(writer, "Unknown subcommand: {}", args[0])?;
                writeln!(writer, "Type /show for available subcommands.")?;
            }
        }

        // Short alias
        "di" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /di <table_name>")?;
            } else {
                execute_show_indexes(completion_state, args[0], writer)?;
            }
        }

        // ... remaining commands ...
    }

    Ok(true)
}

/// Execute the /show indexes metacommand
///
/// Shows index information for a table from DBC.IndicesV
fn execute_show_indexes<W: Write>(
    completion_state: &CompletionState,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // Parse table name - may be qualified (database.table) or unqualified
    let (database, table) = if let Some(dot_pos) = table_name.find('.') {
        let db = &table_name[..dot_pos];
        let tbl = &table_name[dot_pos + 1..];
        (Some(db), tbl)
    } else {
        (None, table_name)
    };

    // Build the query to fetch index information from DBC.IndicesV
    let sql = if let Some(db) = database {
        format!(
            r#"SELECT
                IndexNumber,
                CASE IndexType
                    WHEN 'P' THEN 'Primary Index'
                    WHEN 'Q' THEN 'Partitioned Primary Index'
                    WHEN 'S' THEN 'Secondary Index'
                    WHEN 'K' THEN 'Primary Key'
                    WHEN 'U' THEN 'Unique Constraint'
                    WHEN 'V' THEN 'Value-ordered Secondary Index'
                    WHEN 'H' THEN 'Hash-ordered Covering Secondary Index'
                    WHEN 'O' THEN 'Value-ordered ALL Covering Secondary Index'
                    WHEN 'I' THEN 'Ordering Column (Composite Secondary Index)'
                    WHEN 'G' THEN 'Geospatial Nonunique Secondary Index'
                    ELSE IndexType
                END AS IndexType,
                ColumnName,
                ColumnPosition,
                CASE UniqueFlag
                    WHEN 'Y' THEN 'Unique'
                    ELSE 'Not Unique'
                END AS Uniqueness
               FROM DBC.IndicesV
               WHERE DatabaseName = '{}'
                 AND TableName = '{}'
               ORDER BY IndexNumber, ColumnPosition"#,
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            r#"SELECT
                IndexNumber,
                CASE IndexType
                    WHEN 'P' THEN 'Primary Index'
                    WHEN 'Q' THEN 'Partitioned Primary Index'
                    WHEN 'S' THEN 'Secondary Index'
                    WHEN 'K' THEN 'Primary Key'
                    WHEN 'U' THEN 'Unique Constraint'
                    WHEN 'V' THEN 'Value-ordered Secondary Index'
                    WHEN 'H' THEN 'Hash-ordered Covering Secondary Index'
                    WHEN 'O' THEN 'Value-ordered ALL Covering Secondary Index'
                    WHEN 'I' THEN 'Ordering Column (Composite Secondary Index)'
                    WHEN 'G' THEN 'Geospatial Nonunique Secondary Index'
                    ELSE IndexType
                END AS IndexType,
                ColumnName,
                ColumnPosition,
                CASE UniqueFlag
                    WHEN 'Y' THEN 'Unique'
                    ELSE 'Not Unique'
                END AS Uniqueness
               FROM DBC.IndicesV
               WHERE TableName = '{}'
                 AND DatabaseName = DATABASE
               ORDER BY IndexNumber, ColumnPosition"#,
            escape_sql_string(table)
        )
    };

    // Execute the query
    match completion_state.client().execute(&sql) {
        Ok(result) => {
            if result.row_count == 0 {
                writeln!(
                    writer,
                    "Table '{}' not found or has no indexes.",
                    table_name
                )?;
                writeln!(writer)?;
                writeln!(writer, "Suggestions:")?;
                writeln!(writer, "  - Check the table name spelling")?;
                writeln!(writer, "  - Use qualified name: database.table")?;
                writeln!(writer, "  - Use /list tables to see available tables")?;
                writeln!(writer)?;
            } else {
                // Display the results using table formatter
                writeln!(writer, "Indexes for table '{}':", table_name)?;
                writeln!(writer)?;

                let formatter = crate::output::TableFormatter::new();
                let output = formatter.format(&result)?;
                write!(writer, "{}", output)?;

                writeln!(writer)?;
            }
        }
        Err(e) => {
            // Handle common errors
            let error_msg = e.to_string();

            if error_msg.contains("does not exist") || error_msg.contains("not found") {
                writeln!(writer, "Table '{}' not found.", table_name)?;
                writeln!(writer)?;
                writeln!(writer, "Use /list tables to see available tables.")?;
            } else if error_msg.contains("permission") || error_msg.contains("access denied") {
                writeln!(writer, "Permission denied accessing table '{}'.", table_name)?;
                writeln!(writer)?;
                writeln!(writer, "You may not have SELECT access to this table or DBC.IndicesV.")?;
                writeln!(writer, "Contact your database administrator for access.")?;
            } else {
                writeln!(writer, "Error retrieving index information: {}", e)?;
            }
            writeln!(writer)?;
        }
    }

    Ok(())
}
```

#### Tab Completion

The `/show indexes` command is added to the metacommand completion list:

```rust
// src/commands/repl/metadata_completer.rs

const METACOMMANDS: &[(&str, &str)] = &[
    // ... existing metacommands ...
    ("/show indexes", "Display index information for a table"),
    // ... remaining metacommands ...
];
```

Short alias `\di` is supported as a separate metacommand entry:
```rust
match command.as_str() {
    "show" => { /* handle /show subcommands */ }
    "di" => { /* shortcut for /show indexes */ }
}
```

#### Help Text

Updated help output:
```rust
fn print_help_extended<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "Schema Inspection:")?;
    writeln!(writer, "  /describe <table>, /d  Show table structure")?;
    writeln!(writer, "  /show indexes <table>  Display index information")?;
    writeln!(writer, "  /di <table>            Shortcut for /show indexes")?;
    writeln!(writer, "  /list databases        List all accessible databases")?;
    writeln!(writer, "  /list tables [pattern] List tables (optional glob pattern)")?;
    writeln!(writer, "  /list views            List views in current database")?;
    writeln!(writer, "  /dt                    Shortcut for /list tables")?;
    writeln!(writer, "  /dv                    Shortcut for /list views")?;
    // ... rest of help ...
}
```

#### Usage Examples

**Basic usage:**
```sql
tq> /show indexes employees

Indexes for table 'employees':

+-------------+--------------------+--------------+----------------+-------------+
| IndexNumber | IndexType          | ColumnName   | ColumnPosition | Uniqueness  |
+-------------+--------------------+--------------+----------------+-------------+
|           1 | Primary Index      | employee_id  |              1 | Not Unique  |
|           2 | Secondary Index    | last_name    |              1 | Not Unique  |
|           2 | Secondary Index    | first_name   |              2 | Not Unique  |
|           3 | Primary Key        | employee_id  |              1 | Unique      |
+-------------+--------------------+--------------+----------------+-------------+
(4 rows)
```

**Qualified table name:**
```sql
tq> /show indexes hr.employees

Indexes for table 'hr.employees':
[same output format]
```

**Short alias:**
```sql
tq> \di employees
[same output as /show indexes]
```

**Table not found:**
```sql
tq> /show indexes nonexistent

Table 'nonexistent' not found or has no indexes.

Suggestions:
  - Check the table name spelling
  - Use qualified name: database.table
  - Use /list tables to see available tables
```

**Permission denied:**
```sql
tq> /show indexes restricted_table

Permission denied accessing table 'restricted_table'.

You may not have SELECT access to this table or DBC.IndicesV.
Contact your database administrator for access.
```

#### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Command handler | `src/commands/repl/metacommands.rs` | `execute_show_indexes()` |
| SQL utilities | `src/sql/identifiers.rs` | `escape_sql_string()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | METACOMMANDS list |
| Help text | `src/commands/repl/metacommands.rs` | `print_help_extended()` |
| Output formatting | `src/output/table.rs` | `TableFormatter` |

#### Error Handling

1. **Table not found**: Show helpful message with suggestions
2. **Permission denied**: Clear message explaining access requirements
3. **Invalid table name**: Caught by SQL execution error
4. **Connection lost**: Standard database error handling
5. **System view unavailable**: Error message with alternative suggestion

#### Design Decisions

**Why DBC.IndicesV instead of DBC.Indices?**
- `IndicesV` is a view that provides a more user-friendly interface
- Consistent with `/describe` which uses `DBC.ColumnsV`
- Both views handle qualified names and permissions consistently

**Why separate `/show` parent command?**
- Extensible pattern for future schema commands (`/show constraints`, `/show stats`, etc.)
- Consistent with PostgreSQL's `\d+` pattern
- Clean namespace organization

**Why include ColumnPosition?**
- Multi-column indexes require positional information to understand key structure
- Helps users understand composite index column order (important for query optimization)

**Why translate IndexType codes?**
- Raw codes (P, Q, S, K, U, etc.) are cryptic for users
- Human-readable descriptions improve usability
- Follows pattern established by other database tools (pgcli, mycli)

---

## System Configuration Command (/sysconfig)

This section documents the technical design for the `/sysconfig` metacommand, which displays Teradata system topology and configuration in a compact summary format.

### Overview

The `/sysconfig` command queries multiple Teradata system views to build a consolidated system configuration summary. Unlike the tabular output of `/sessions`, this command produces a key-value summary display that gives DBAs immediate visibility into the system topology (version, nodes, AMPs, PEs) without running multiple queries.

**User Stories:** US-1.1, US-1.2, US-1.3 (Configuration Summary)

### Architecture

```
Sysconfig Command Flow:

/sysconfig (REPL) or tq sysconfig (batch)
        |
        v
Execute multiple SQL queries:
  1. DBC.DBCInfoV -> version, release
  2. HASHAMP()+1 -> total AMP count
  3. Node topology query -> node count
        |
        v
Parse results into SystemConfig struct
        |
        v
Format output:
  REPL: Compact key-value summary
  Batch: table/csv/json per --format flag
```

### Module Structure

New file: `src/commands/sysconfig.rs`

Following the established `sessions.rs` pattern:
- SQL constants for each query
- Parsed data struct (`SystemConfig`)
- `execute()` for batch mode
- `execute_for_repl()` for REPL mode
- `display_table()`, `display_csv()`, `display_json()` formatters

### SQL Query Design

Three separate queries are executed sequentially within the same connection:

#### Query 1: System Version and Release

```sql
SELECT InfoKey, CAST(InfoData AS VARCHAR(256)) AS InfoData
FROM DBC.DBCInfoV
ORDER BY InfoKey
```

This returns rows with `InfoKey` values including:
- `VERSION` - The database engine version (e.g., `16.20.53.30`)
- `RELEASE` - The release identifier (e.g., `16.20.53.30`)
- `LANGUAGE SUPPORT MODE` - Character set mode

The query uses `CAST(InfoData AS VARCHAR(256))` to ensure the `InfoData` column (which may be a longer type) is returned as a manageable string.

#### Query 2: Total AMP Count

```sql
SELECT HASHAMP()+1 AS TotalAMPs
```

`HASHAMP()` is a built-in Teradata function that returns the highest AMP number (zero-indexed). Adding 1 gives the total AMP count. This is the canonical way to determine AMP count and works on all Teradata versions.

#### Query 3: Node Count (with fallback)

Primary query:
```sql
SELECT COUNT(DISTINCT NodeID) AS NodeCount
FROM DBC.ResUsageSpma
WHERE TheDate = DATE
```

This queries `DBC.ResUsageSpma` (System Performance Measurement Architecture) for today's data to determine the number of physical nodes. The `WHERE TheDate = DATE` filter limits the scan to recent data.

Fallback query (if ResUsageSpma is unavailable or empty):
```sql
SELECT COUNT(DISTINCT NodeID) AS NodeCount
FROM DBC.ResCpuUsageByAmpView
```

If both fail, the node count is reported as "N/A" rather than causing a command failure.

#### PE Count

PE count is derived from the node count using a heuristic (1 PE per node for typical configurations), or if available, queried from the system. Since PE count is not reliably available from standard DBC views on all configurations, the implementation reports it when available and omits it gracefully when not.

### Data Model

```rust
/// System configuration information extracted from DBC views
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// Teradata software version string (e.g., "16.20.53.30")
    pub version: String,
    /// Teradata release string
    pub release: String,
    /// Total number of AMPs in the system
    pub amp_count: i64,
    /// Number of physical/logical nodes (None if unavailable)
    pub node_count: Option<i64>,
    /// Number of Parsing Engines (None if unavailable)
    pub pe_count: Option<i64>,
    /// Additional info key-value pairs from DBCInfoV
    pub info_entries: Vec<(String, String)>,
}
```

The struct uses `Option<i64>` for fields that may not be available due to view permissions or system configuration. The `info_entries` vector captures all key-value pairs from `DBC.DBCInfoV` for completeness.

### Row Parsing

```rust
impl SystemConfig {
    /// Build SystemConfig from DBC.DBCInfoV query result
    pub fn from_dbcinfo_result(result: &QueryResult) -> Self {
        let mut version = String::from("Unknown");
        let mut release = String::from("Unknown");
        let mut info_entries = Vec::new();

        for row in &result.rows {
            if row.len() < 2 { continue; }

            let key = match &row[0] {
                Value::String(s) => s.trim().to_string(),
                _ => continue,
            };
            let value = match &row[1] {
                Value::String(s) => s.trim().to_string(),
                Value::Null => "[NULL]".to_string(),
                other => other.display(),
            };

            match key.to_uppercase().as_str() {
                "VERSION" => version = value.clone(),
                "RELEASE" => release = value.clone(),
                _ => {}
            }
            info_entries.push((key, value));
        }

        Self {
            version,
            release,
            amp_count: 0,  // Set separately
            node_count: None,
            pe_count: None,
            info_entries,
        }
    }
}
```

### REPL Display Format

The REPL display uses a compact key-value summary format rather than a table:

```
System Configuration:
  Version:    16.20.53.30
  Release:    16.20.53.30
  Nodes:      4
  AMPs:       64
  AMPs/Node:  16

(Query time: 0.234s)
```

This is implemented using plain `writeln!` formatting for alignment:

```rust
pub fn execute_for_repl<W: Write>(client: &DatabaseClient, writer: &mut W) -> Result<()> {
    writeln!(writer)?;

    match build_system_config(client) {
        Ok(config) => {
            writeln!(writer, "System Configuration:")?;
            writeln!(writer, "  Version:    {}", config.version)?;
            writeln!(writer, "  Release:    {}", config.release)?;
            if let Some(nodes) = config.node_count {
                writeln!(writer, "  Nodes:      {}", nodes)?;
            }
            writeln!(writer, "  AMPs:       {}", config.amp_count)?;
            if let (Some(nodes), amp_count) = (config.node_count, config.amp_count) {
                if nodes > 0 {
                    writeln!(writer, "  AMPs/Node:  {}", amp_count / nodes)?;
                }
            }
            if let Some(pes) = config.pe_count {
                writeln!(writer, "  PEs:        {}", pes)?;
            }
        }
        Err(e) => {
            // Error handling follows sessions.rs pattern
            handle_sysconfig_error(&e, writer)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}
```

### Batch Mode Display

For batch mode, three output formats are supported:

**Table format:** Uses `comfy_table` with a key-value layout:

```
System Configuration:
+-----------+---------------+
| Property  | Value         |
+-----------+---------------+
| Version   | 16.20.53.30   |
| Release   | 16.20.53.30   |
| Nodes     | 4             |
| AMPs      | 64            |
| AMPs/Node | 16            |
+-----------+---------------+
```

**CSV format:**
```
Property,Value
Version,16.20.53.30
Release,16.20.53.30
Nodes,4
AMPs,64
AMPs/Node,16
```

**JSON format:**
```json
{
  "Version": "16.20.53.30",
  "Release": "16.20.53.30",
  "Nodes": 4,
  "AMPs": 64,
  "AMPsPerNode": 16
}
```

### Error Handling

Privilege errors are detected and presented with actionable guidance, following the sessions.rs pattern:

```rust
fn handle_sysconfig_error<W: Write>(e: &crate::error::TqError, writer: &mut W) -> Result<()> {
    let error_str = e.to_string().to_lowercase();

    if error_str.contains("privilege") || error_str.contains("access")
        || error_str.contains("permission") || error_str.contains("3523")
    {
        writeln!(writer, "Error: Insufficient privileges to query system configuration.")?;
        writeln!(writer)?;
        writeln!(writer, "Required: SELECT privilege on DBC.DBCInfoV")?;
        writeln!(writer)?;
        writeln!(writer, "To grant access, a DBA can run:")?;
        writeln!(writer, "  GRANT SELECT ON DBC.DBCInfoV TO <username>;")?;
    } else {
        writeln!(writer, "Error querying system configuration: {}", e)?;
    }
    Ok(())
}
```

The multi-query approach is resilient: if the AMP count query succeeds but the node count query fails (e.g., due to `ResUsageSpma` not being collected), the command still displays available information rather than failing entirely.

### CLI Integration

```rust
// In src/cli.rs - Command enum
/// Display system configuration and topology
///
/// Shows Teradata version, node count, AMP count, and system topology.
/// Requires SELECT privilege on DBC.DBCInfoV.
Sysconfig(SysconfigArgs),

/// Arguments for the sysconfig command
#[derive(Parser, Debug)]
pub struct SysconfigArgs {
    /// Output format
    #[arg(
        short, long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

### Tab Completion Integration

Add to the `METACOMMANDS` array in `metadata_completer.rs`:

```rust
MetacommandDef {
    name: "sysconfig",
    aliases: &["sc"],
    description: "Show system configuration and topology",
},
```

### Metacommand Handler Integration

In `metacommands.rs`, both `handle_metacommand` and `handle_metacommand_with_state`:

```rust
// System configuration command
"sysconfig" | "sc" => {
    crate::commands::sysconfig::execute_for_repl(completion_state.client(), writer)?;
}
```

### Help Text Integration

Add to the "System Monitoring" section in `print_help_extended()`:

```rust
writeln!(writer, "System Monitoring:")?;
writeln!(writer, "  /sessions              List active sessions with performance metrics")?;
writeln!(writer, "  /sysconfig, /sc        Show system configuration and topology")?;
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_config_from_dbcinfo_result() {
        // Test parsing of DBC.DBCInfoV result rows
    }

    #[test]
    fn test_system_config_version_extraction() {
        // Test that VERSION key is correctly extracted
    }

    #[test]
    fn test_system_config_missing_keys() {
        // Test graceful handling when expected keys are absent
    }

    #[test]
    fn test_amp_count_parsing() {
        // Test HASHAMP()+1 result parsing
    }

    #[test]
    fn test_node_count_parsing() {
        // Test node count from ResUsageSpma
    }

    #[test]
    fn test_display_table_format() {
        // Test table formatter output
    }

    #[test]
    fn test_display_csv_format() {
        // Test CSV formatter output
    }

    #[test]
    fn test_display_json_format() {
        // Test JSON formatter output
    }

    #[test]
    fn test_repl_display_format() {
        // Test compact REPL display output
    }

    #[test]
    fn test_repl_display_with_missing_node_count() {
        // Test REPL display when node count is unavailable
    }
}
```

### Design Decisions

**Why multiple queries instead of one combined query?**
- The three data sources (`DBC.DBCInfoV`, `HASHAMP()`, node topology views) cannot be efficiently joined in a single query
- Sequential execution allows partial results if some views are unavailable
- Each query is lightweight (no table scans, instant results)

**Why compact key-value format instead of table format for REPL?**
- System configuration is a small fixed set of properties, not a variable-length list
- Key-value format is more readable for summary data
- Follows the pattern of database CLI tools like `psql`'s `\conninfo`

**Why include AMPs/Node derived field?**
- This ratio is valuable for DBAs to verify even distribution across nodes
- Teradata documentation frequently references this metric
- It is trivially computed from available data

---

## Lock Information Command (/locks)

This section documents the technical design for the `/locks` metacommand, which displays current lock contention and blocking chains to help DBAs diagnose session blocking issues.

### Overview

The `/locks` command queries the `DBC.LockInfoV` view to display current lock information including locked objects, lock types, lock modes, and waiting sessions. It consolidates raw lock rows into a display-friendly format and identifies blocking chains to help DBAs diagnose contention.

**User Stories:** US-3.2, US-3.3, US-3.5, US-3.6 (Session and Lock Information)

### Architecture

```
Locks Command Flow:

/locks (REPL) or tq locks (batch)
        |
        v
Execute SQL query:
  DBC.LockInfoV with CASE expressions for human-readable names
        |
        v
Parse into LockInfo structs (one per raw lock row)
        |
        v
Build display rows (aggregate waiters per lock holder)
        |
        v
Identify blocking chains (sessions with waiters)
        |
        v
Format output:
  REPL: Table + blocking chain annotations + summary footer
  Batch: table/csv/json per --format flag
```

### Module Structure

**File**: `src/commands/locks.rs`

Following the established monitoring command pattern:
- SQL constant (`LOCKS_SQL`) for the DBC.LockInfoV query
- `LockInfo` struct for raw parsed rows
- `LockDisplayRow` struct for consolidated display rows
- `BlockingChain` struct for blocker-to-blocked relationships
- `build_display_rows()` aggregation function
- `identify_blocking_chains()` analysis function
- `execute()` for batch mode
- `execute_for_repl()` for REPL mode
- `display_table()`, `display_csv()`, `display_json()` formatters

### SQL Query Design

The query uses `DBC.LockInfoV` with CASE expressions to translate codes into human-readable names:

```sql
SELECT
    TRIM(DatabaseName) || '.' || TRIM(TableName) AS LockedObject,
    CASE LockType
        WHEN 'T' THEN 'Table'
        WHEN 'R' THEN 'Row Hash'
        WHEN 'D' THEN 'Database'
        WHEN 'V' THEN 'View'
        ELSE TRIM(LockType)
    END AS LockTypeName,
    CASE ModeGranted
        WHEN 'A' THEN 'ACCESS'
        WHEN 'R' THEN 'READ'
        WHEN 'W' THEN 'WRITE'
        WHEN 'E' THEN 'EXCLUSIVE'
        ELSE TRIM(ModeGranted)
    END AS LockModeName,
    GrantorSessionId,
    LockerSessionId,
    CASE ModeWanting
        WHEN ' ' THEN NULL
        WHEN '' THEN NULL
        ELSE ModeWanting
    END AS ModeWanting
FROM DBC.LockInfoV
ORDER BY LockerSessionId, LockedObject
```

**Key columns:**
- `LockedObject` - The database.table name that is locked
- `LockTypeName` - Lock granularity (Table, Row Hash, Database, View)
- `LockModeName` - Lock mode (ACCESS, READ, WRITE, EXCLUSIVE)
- `GrantorSessionId` - Session that holds/granted the lock
- `LockerSessionId` - Session that holds or is requesting the lock
- `ModeWanting` - Non-NULL when the session is waiting for a lock (distinguishes holders from waiters)

**Design Decision: DBC.LockInfoV over MonitorSession**

The implementation uses `DBC.LockInfoV` rather than `MonitorSession` because:
1. It provides detailed lock information (object name, lock type, lock level) that MonitorSession does not
2. It directly shows which object is locked and at what level
3. It distinguishes lock holders from waiters via the `ModeWanting` column
4. It requires only SELECT privilege on DBC.LockInfoV (standard DBC view access)

### Data Model

```rust
// src/commands/locks.rs

/// Lock information extracted from DBC.LockInfoV
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// Locked object name (database.table)
    pub locked_object: String,
    /// Lock type (Table, Row Hash, Database, View)
    pub lock_type: String,
    /// Lock mode (ACCESS, READ, WRITE, EXCLUSIVE)
    pub lock_mode: String,
    /// Session ID that holds the lock
    pub locking_session: i64,
    /// Session ID of the grantor
    pub grantor_session: i64,
    /// Whether this row represents a waiting session
    pub is_waiting: bool,
}

impl LockInfo {
    /// Create LockInfo from a DBC.LockInfoV query result row
    ///
    /// Expected columns: LockedObject, LockTypeName, LockModeName,
    /// GrantorSessionId, LockerSessionId, ModeWanting
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 6 { return None; }
        // ModeWanting non-NULL indicates a waiting session
        let is_waiting = !matches!(&row[5], Value::Null);
        // ... extraction logic
    }
}
```

### Display Row Aggregation

Raw `LockInfo` rows are consolidated into display rows where each row represents one lock with all its waiting sessions:

```rust
/// A consolidated lock display row for output
#[derive(Debug, Clone)]
pub struct LockDisplayRow {
    pub locked_object: String,
    pub lock_type: String,
    pub lock_mode: String,
    pub locking_session: i64,
    pub waiting_sessions: Vec<i64>,
}
```

The `build_display_rows()` function performs a two-pass aggregation:
1. **First pass**: Collect lock holders (rows where `is_waiting` is false)
2. **Second pass**: Attach waiters to their corresponding holders using the `grantor_session` as the join key

### Blocking Chain Logic

```rust
/// Blocking chain information
#[derive(Debug, Clone)]
pub struct BlockingChain {
    pub blocker_session: i64,
    pub blocked_sessions: Vec<i64>,
}
```

The `identify_blocking_chains()` function aggregates all waiting sessions per blocking session across all display rows. A session that holds multiple locks may block different sessions on each lock; the chain groups all blocked sessions under one blocker entry.

### REPL Display Format

```
Lock Information:
+----------------+-----------+-----------+--------------+--------------+
| Locked Object  | Lock Type | Lock Mode | Locking Sess | Waiting Sess |
+----------------+-----------+-----------+--------------+--------------+
| PROD.orders    | Table     | WRITE     |         1023 | 1045, 1067   |
| PROD.customers | Table     | READ      |         1078 | (none)       |
+----------------+-----------+-----------+--------------+--------------+

2 lock(s) found - 1 blocking chain(s) detected (Query time: 0.156s)

Blocking Chain:
  Session 1023 blocks sessions: 1045, 1067
```

When no locks exist:

```
Lock Information:
No locks currently held.

(Query time: 0.089s)
```

### Batch Mode Display

**Table format:** Same as REPL table with summary footer.

**CSV format:**
```
Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess
PROD.orders,Table,WRITE,1023,"1045, 1067"
PROD.customers,Table,READ,1078,
```

**Note:** The CSV format uses empty string for locks with no waiters, which is the standard convention for machine-parseable CSV. The table display format uses "(none)" for human readability. This distinction is handled by `format_waiting_sessions()` (table) vs `format_waiting_sessions_csv()` (CSV).

**JSON format:**
```json
[
  {
    "Locked Object": "PROD.orders",
    "Lock Type": "Table",
    "Lock Mode": "WRITE",
    "Locking Sess": 1023,
    "Waiting Sess": [1045, 1067]
  }
]
```

### Error Handling

The error handling follows the standard monitoring command pattern, detecting privilege and availability errors:

```rust
// Privilege error -> suggest GRANT SELECT ON DBC.LockInfoV
// View not found -> inform that DBC.LockInfoV is not accessible
// Generic error -> display error message
```

The specific privilege guidance references `DBC.LockInfoV` (not MonitorSession):
```
Error: Unable to retrieve lock information.

This command requires SELECT access to DBC lock views.

To grant access, a DBA can run:
  GRANT SELECT ON DBC.LockInfoV TO <your_username>;
```

### CLI Integration

```rust
// In src/cli.rs - Command enum
/// Display current lock contention and blocking chains
///
/// Shows locked objects, lock types, locking sessions, and waiting sessions.
/// Requires SELECT privilege on DBC.LockInfoV.
Locks(LocksArgs),

/// Arguments for the locks command
#[derive(Parser, Debug)]
pub struct LocksArgs {
    /// Output format
    #[arg(short, long, env = "TQ_FORMAT", default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

### Tab Completion Integration

In the `METACOMMANDS` array in `metadata_completer.rs`:

```rust
MetacommandDef {
    name: "locks",
    aliases: &["lk"],
    description: "Show lock contention and blocking sessions",
},
```

### Metacommand Handler Integration

In `metacommands.rs`, both `handle_metacommand` and `handle_metacommand_with_state`:

```rust
"locks" | "lk" => {
    crate::commands::locks::execute_for_repl(completion_state.client(), writer)?;
}
```

### Design Decisions

**Why DBC.LockInfoV?**
- Provides object-level detail (which table is locked, lock type, lock level)
- Standard DBC view available on modern Teradata installations
- Clear waiter detection via `ModeWanting` column
- Requires only SELECT privilege (standard DBC access)

**Why aggregate into display rows?**
- Raw LockInfoV rows have one row per lock participant (holder + each waiter)
- Users need a consolidated view: one row per lock with all waiters listed
- Aggregation simplifies blocking chain identification

**Why the /lk short alias?**
- `/l` is already taken by `/list`
- `/lk` is intuitive (first two consonants of "locks")
- Follows the short alias pattern established by `/sc` for sysconfig

---

## Monitoring Commands: Shared Patterns

This section documents common patterns shared across the monitoring command family (`/sessions`, `/sysconfig`, `/locks`, `/query`).

### Module Pattern

Each monitoring command follows this structure:

```
src/commands/<command>.rs
  - SQL constant(s)
  - Parsed data struct
  - from_row() parser
  - execute() for batch mode
  - execute_for_repl() for REPL mode
  - display_table() using comfy_table
  - display_csv() using writeln!
  - display_json() using serde_json
  - Privilege error handling (inline or via shared pattern)
  - #[cfg(test)] mod tests
```

### Integration Touchpoints

Adding a new monitoring command requires changes to exactly these files:

1. `src/commands/<command>.rs` (NEW) - Command implementation
2. `src/commands/mod.rs` - Register module and re-export
3. `src/cli.rs` - Add `<Command>Args` struct and `Command` enum variant
4. `src/main.rs` - Handle new command variant in `run()` function
5. `src/commands/repl/metacommands.rs` - Add to both handler functions and help text
6. `src/commands/repl/metadata_completer.rs` - Add to `METACOMMANDS` array

### Error Handling Pattern

All monitoring commands follow the same privilege error handling pattern:

1. Detect privilege/access errors via string matching on error message
2. Display clear error message identifying the required privilege
3. Provide the exact GRANT statement needed to fix the issue
4. Handle version/availability errors separately
5. Fall back to generic error display for unexpected errors

### Output Format Pattern

All monitoring commands support the same three output formats:

- **Table** (`comfy_table`): Human-readable, used by default
- **CSV**: Machine-readable, pipe-friendly
- **JSON**: Structured output for programmatic consumption

The format is controlled by:
- REPL mode: Always uses table/summary format (no format flag)
- Batch mode: `--format` flag with `table`/`csv`/`json` values

---

## Monitoring Utilities Module

This section documents the shared utilities module that eliminates code duplication across monitoring commands.

### Problem

The monitoring commands (`sessions.rs`, `sysconfig.rs`, `locks.rs`, `sample.rs`) each contain duplicated utility functions for value extraction and CSV escaping. This creates maintenance burden and risks inconsistent behavior across commands.

**Duplicated functions identified:**

| Function | sessions.rs | sysconfig.rs | locks.rs | sample.rs |
|----------|:-----------:|:------------:|:--------:|:---------:|
| `extract_integer()` | Yes | Yes | Yes | No |
| `extract_decimal()` | Yes | No | No | No |
| `extract_trimmed_string()` | No | Yes | Yes | No |
| `escape_csv()` | Yes | Yes | Yes | Yes |

### Module Design

**File**: `src/commands/monitoring_utils.rs`

This module provides shared utility functions used by monitoring command implementations.

```rust
// src/commands/monitoring_utils.rs

use crate::db::Value;

/// Extract integer value from a Value, returning None for NULL
///
/// Handles Value::Integer and Value::Decimal (truncated to i64).
/// Returns None for NULL or non-numeric types.
pub fn extract_integer(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(v) => Some(*v),
        Value::Decimal(v) => Some(*v as i64),
        Value::Null => None,
        _ => None,
    }
}

/// Extract decimal value from a Value, returning None for NULL
///
/// Handles Value::Decimal and Value::Integer (promoted to f64).
/// Returns None for NULL or non-numeric types.
pub fn extract_decimal(value: &Value) -> Option<f64> {
    match value {
        Value::Decimal(v) => Some(*v),
        Value::Integer(v) => Some(*v as f64),
        Value::Null => None,
        _ => None,
    }
}

/// Extract a trimmed string from a Value
///
/// Returns the specified `null_display` string for NULL values.
/// For non-string types, calls `Value::display()` and trims.
///
/// # Arguments
/// * `value` - The database value to extract
/// * `null_display` - String to return for NULL values (e.g., "[NULL]" or "[unavailable]")
pub fn extract_trimmed_string(value: &Value, null_display: &str) -> String {
    match value {
        Value::String(s) => s.trim().to_string(),
        Value::Null => null_display.to_string(),
        other => other.display().trim().to_string(),
    }
}

/// Escape a string for CSV output
///
/// Wraps the string in double quotes and escapes internal quotes
/// if the string contains commas, double quotes, or newlines.
pub fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

### Design Decisions

**`extract_trimmed_string` parameterization:**

The sysconfig and locks modules use different NULL display strings (`"[unavailable]"` vs `"[NULL]"`). Rather than creating two separate functions, the shared version accepts a `null_display` parameter. Each caller specifies the appropriate string for its context.

**`extract_decimal` consolidation:**

Although only `sessions.rs` currently uses `extract_decimal()`, it is included in the shared module because:
1. It follows the same pattern as `extract_integer`
2. Future monitoring commands (e.g., `/query`) may need it for metrics
3. Having all extraction helpers in one place improves discoverability

### Refactoring Strategy

The refactoring is mechanical and low-risk:

1. Create `src/commands/monitoring_utils.rs` with the shared functions
2. Register the module in `src/commands/mod.rs` (`pub mod monitoring_utils;`)
3. For each consumer module:
   - Add `use super::monitoring_utils::{...};` (or `use crate::commands::monitoring_utils::{...};`)
   - Remove the local function definitions
   - For `extract_trimmed_string` callers, add the `null_display` argument to each call site
4. Run `cargo test --lib` after each module to verify no regressions

**Call site changes for `extract_trimmed_string`:**

```rust
// sysconfig.rs - before:
let key = extract_trimmed_string(&row[0]);
// sysconfig.rs - after:
let key = extract_trimmed_string(&row[0], "[unavailable]");

// locks.rs - before:
let locked_object = extract_trimmed_string(&row[0]);
// locks.rs - after:
let locked_object = extract_trimmed_string(&row[0], "[NULL]");
```

### Integration with Module Registry

```rust
// src/commands/mod.rs
pub mod locks;
pub mod monitoring_utils;  // NEW: shared monitoring utilities
pub mod ping;
pub mod query;
pub mod repl;
pub mod sample;
pub mod sessions;
pub mod sysconfig;
```

### Unit Tests

The shared module carries its own unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // extract_integer tests
    fn test_extract_integer_from_integer();
    fn test_extract_integer_from_decimal();
    fn test_extract_integer_from_null();
    fn test_extract_integer_from_string();

    // extract_decimal tests
    fn test_extract_decimal_from_decimal();
    fn test_extract_decimal_from_integer();
    fn test_extract_decimal_from_null();

    // extract_trimmed_string tests
    fn test_extract_trimmed_string_from_string();
    fn test_extract_trimmed_string_from_null_custom_display();
    fn test_extract_trimmed_string_from_other_type();
    fn test_extract_trimmed_string_trims_whitespace();

    // escape_csv tests
    fn test_escape_csv_simple();
    fn test_escape_csv_with_comma();
    fn test_escape_csv_with_quotes();
    fn test_escape_csv_with_newline();
    fn test_escape_csv_empty_string();
}
```

Existing tests in each consumer module continue to work because they test behavior through the public API (e.g., `from_row()`, `display_csv()`), not the utility functions directly. The utility function tests that currently exist in consumer modules can be removed since the shared module's tests cover them.

---

## Query Inspection Command (/query)

This section documents the technical design for the `/query` metacommand, which displays the SQL text and execution metadata of a session's most recent query.

### Overview

The `/query <session_id>` command queries `DBC.QryLogV` (and optionally `DBC.DBQLSqlTbl` for full SQL text) to show what SQL a given session is or was recently executing. This is the natural next step in the PMON workflow: see sessions -> see locks -> inspect the SQL.

**User Stories:** US-9.1, US-9.3 (Query Drill-Down and Analysis)

### Architecture

```
Query Inspection Command Flow:

/query <session_id> (REPL) or tq query-session <session_id> (batch)
        |
        v
Validate session_id is a positive integer
        |
        v
Execute SQL query:
  DBC.QryLogV WHERE SessionID = <session_id>
  ORDER BY StartTime DESC
        |
        v
Parse into QueryLogInfo struct(s)
        |
        v
If QueryText is truncated (200 char default), optionally
fetch full text from DBC.DBQLSqlTbl via ProcID + QueryID join
        |
        v
Format output:
  REPL: Key-value summary with SQL text
  Batch: table/csv/json per --format flag
```

### Module Structure

**File**: `src/commands/query_inspect.rs`

**Note on naming**: The existing `src/commands/query.rs` handles the general `tq query "SELECT ..."` command. The query inspection command is a different feature -- it inspects a session's queries. The module is named `query_inspect.rs` to avoid collision. The REPL metacommand is `/query <session_id>` (alias `/qi`) and the batch command is `tq query-inspect <session_id>`.

Following the established monitoring command pattern:
- `build_query_sql()` function for the DBC.QryLogV query (parameterized by session_id)
- `QueryInfo` struct for parsed result
- `execute()` for batch mode
- `execute_for_repl()` for REPL mode
- `display_table()`, `display_csv()`, `display_json()` formatters
- Uses shared `monitoring_utils` functions (extract_integer, extract_trimmed_string, escape_csv)

### SQL Query Design

**Primary query** - Fetch the most recent queries for a session:

```sql
SELECT TOP 5
    SessionID,
    CAST(QueryText AS VARCHAR(10000)) AS QueryText,
    CAST(StartTime AS VARCHAR(30)) AS StartTime,
    CAST(TotalElapsedTime AS VARCHAR(30)) AS TotalElapsedTime,
    CASE
        WHEN AbortFlag = 'Y' THEN 'Aborted'
        WHEN ErrorCode <> 0 THEN 'Error'
        ELSE 'Complete'
    END AS QueryStatus
FROM DBC.QryLogV
WHERE SessionID = <session_id>
  AND CollectTimeStamp >= CURRENT_TIMESTAMP - INTERVAL '1' DAY
ORDER BY CollectTimeStamp DESC
```

**Key columns:**
- `SessionID` - Session that ran the query
- `QueryText` - SQL text (cast to VARCHAR(10000) for extended text beyond default 200 chars)
- `StartTime` - When the query started
- `TotalElapsedTime` - Total elapsed execution time
- `QueryStatus` - Derived status: 'Complete', 'Aborted', or 'Error' based on AbortFlag and ErrorCode

**Why TOP 5?**
- Shows recent query history for the session, not just the latest
- Helps DBAs see the pattern of activity (e.g., repeated failing queries)
- Keeps output manageable in REPL mode

**Why filter by last 1 day?**
- DBQL can accumulate large volumes of data
- Most monitoring use cases involve recent queries
- Reduces query execution time on systems with extensive DBQL history

**Design Decision: CAST to VARCHAR(10000)**

The default `QueryText` column in `DBC.QryLogV` is limited to 200 characters. By casting to `VARCHAR(10000)`, we retrieve significantly more text without needing a secondary query to `DBQLSqlTbl` in most cases. The CAST approach is simpler and avoids the join complexity.

### Data Model

```rust
// src/commands/query_inspect.rs

/// Query information extracted from DBC.QryLogV
#[derive(Debug, Clone)]
pub struct QueryInfo {
    /// Session ID that ran the query
    pub session_id: i64,
    /// SQL text of the query
    pub query_text: String,
    /// Query start time (formatted)
    pub start_time: String,
    /// Total elapsed time (formatted)
    pub total_elapsed: String,
    /// Query status (Complete, Aborted, Error)
    pub status: String,
}

impl QueryInfo {
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 { return None; }
        let session_id = extract_integer(&row[0])?;
        let query_text = extract_trimmed_string(&row[1], "");
        let start_time = extract_trimmed_string(&row[2], "[unknown]");
        let total_elapsed = extract_trimmed_string(&row[3], "[unknown]");
        let status = extract_trimmed_string(&row[4], "Unknown");
        Some(Self { session_id, query_text, start_time, total_elapsed, status })
    }
}
```

### REPL Display Format

The REPL display shows each recent query as a key-value property table:

```
Recent Queries for Session 1078:

+----------+-------------------------------------------+
| Property | Value                                     |
+----------+-------------------------------------------+
| Query #  | 1                                         |
| Start Time | 2026-01-27 15:33:26                     |
| Elapsed Time | 00:00:02.456                          |
| Status   | Complete                                  |
| SQL      | SELECT o.order_id, c.customer_name...     |
+----------+-------------------------------------------+

2 recent query(ies) for session 1078 (Query time: 0.089s)
```

SQL text is truncated at 200 characters in table display; CSV and JSON formats include full text.

When no queries found:

```
No queries found for session 9999.

(Query time: 0.045s)
```

### Long SQL Text Handling

SQL text can be very long (thousands of characters). The display strategy:

1. **Table format (REPL and batch)**: Truncate SQL at 200 characters with "..." suffix. Whitespace is normalized (newlines/tabs replaced with spaces).
2. **CSV format**: Include complete SQL text (no truncation). Properly escaped per RFC 4180.
3. **JSON format**: Include complete SQL text (no truncation).

### Batch Mode

**CLI definition:**

```rust
// In src/cli.rs
/// Inspect recent SQL queries for a session
#[command(name = "query-inspect")]
QueryInspect(QueryInspectArgs),

/// Arguments for the query-inspect command
#[derive(Parser, Debug)]
pub struct QueryInspectArgs {
    /// Session ID to inspect
    pub session_id: i64,

    /// Output format
    #[arg(short, long, env = "TQ_FORMAT", default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

**Batch CSV format:**
```
SessionID,StartTime,ElapsedTime,Status,QueryText
1234,2026-01-27 15:33:26,00:00:02.456,Complete,"SELECT o.order_id, c.customer_name..."
```

**Batch JSON format:**
```json
[
  {
    "SessionID": 1234,
    "StartTime": "2026-01-27 15:33:26",
    "ElapsedTime": "00:00:02.456",
    "Status": "Complete",
    "QueryText": "SELECT o.order_id, c.customer_name..."
  }
]
```

### Tab Completion Integration

```rust
MetacommandDef {
    name: "query",
    aliases: &["qi"],
    description: "Show recent SQL queries for a session",
},
```

### Metacommand Handler Integration

The `/query` metacommand requires an argument (session_id), so the handler includes argument parsing:

```rust
"query" | "qi" => {
    if args.is_empty() {
        writeln!(writer, "Usage: /query <session_id>")?;
        writeln!(writer, "       /qi <session_id>")?;
        writeln!(writer)?;
        writeln!(writer, "Shows recent SQL queries for the specified session.")?;
        writeln!(writer, "Use /sessions to find session IDs.")?;
    } else {
        match args[0].parse::<i64>() {
            Ok(session_id) if session_id > 0 => {
                crate::commands::query_inspect::execute_for_repl(
                    completion_state.client(),
                    session_id,
                    writer,
                )?;
            }
            _ => {
                writeln!(writer, "Error: Invalid session ID '{}'. Must be a positive integer.", args[0])?;
            }
        }
    }
}
```

### Error Handling

```rust
// DBC.QryLogV privilege error
"Error: Unable to retrieve query information.

This command requires SELECT access to DBC query log views.

To grant access, a DBA can run:
  GRANT SELECT ON DBC.QryLogV TO <your_username>;"

// DBQL not enabled
"Error: No query log data available.

DBQL (Database Query Logging) may not be enabled.
Contact your DBA to enable query logging:
  BEGIN QUERY LOGGING ON ALL;"

// Session not found (no rows returned, not an error)
"No queries found in DBQL for session <id>.

This may mean:
  - The session ID does not exist
  - DBQL logging is not enabled for this user
  - The query log has been purged"
```

### Design Decisions

**Why DBC.QryLogV?**
- Standard DBQL view available on all Teradata systems with DBQL enabled
- Contains query text, timing, and execution metrics in a single view
- SessionID column provides direct filtering without joins
- CAST to VARCHAR(10000) retrieves extended SQL text without secondary queries

**Why not use MonitorSession for current query text?**
- MonitorSession does not expose SQL text
- QryLogV provides historical queries, not just the current one
- QryLogV includes execution metrics (IO, rows) that MonitorSession does not

**Why CAST QueryText to VARCHAR(10000)?**
- Default QueryText in QryLogV is limited to 200 characters
- CAST extends the retrieved text to cover most real-world queries
- Avoids the complexity of joining to DBQLSqlTbl for routine use
- The --full-sql flag provides a fallback for extremely long queries

**Why TOP 5 instead of TOP 1?**
- DBAs troubleshooting a session often need context beyond the current query
- Seeing the last 5 queries reveals patterns (repeated failures, query sequence)
- The count is configurable via `--count` flag in batch mode
- REPL mode always shows up to 5 for quick diagnosis

**Why query_inspect.rs instead of extending query.rs?**
- The existing `query.rs` handles `tq query "SELECT ..."` (executing user SQL)
- Query inspection is a completely different feature (inspecting DBQL)
- Separate modules maintain single responsibility
- The REPL metacommand `/query` routes to query_inspect, not the general query command

---

## Variable Substitution in REPL (`/params`)

This section documents the `/params` metacommand for managing YAML-based variable substitution at runtime within the REPL.

See `docs/design/params.md` for the full substitution engine design.

### State Management

The `ParamStore` is held in `ReplState`:

```rust
// src/commands/repl/state.rs
pub struct ReplState {
    // ... existing fields ...
    pub params: ParamStore,
}
```

When the REPL starts, `ParamStore` is initialized from any `--params`/`-p` CLI flags:

```rust
// In REPL initialization (mod.rs)
let mut params = ParamStore::new();
for path in &global.params {
    params.load_file(path)?;
}
state.params = params;
```

### Metacommand Handler

**Primary file**: `src/commands/repl/metacommands.rs`

The `/params` metacommand (alias `/p`) supports three subcommands:

| Subcommand | Syntax | Behavior |
|------------|--------|----------|
| `show` (default) | `/params` or `/params show` | Display loaded files and available variables |
| `load` | `/params load <file>` | Load and merge a YAML parameter file |
| `unload` | `/params unload` | Clear all loaded parameters |

Handler integration in the main match block:

```rust
// In handle_metacommand or handle_metacommand_with_state
"params" | "p" => {
    handle_params(&args, state, writer)?;
}
```

The handler function:

```rust
fn handle_params<W: Write>(
    args: &[&str],
    state: &mut ReplState,
    writer: &mut W,
) -> Result<()> {
    match args.first().copied() {
        Some("load") => {
            let path_str = args.get(1).ok_or_else(|| TqError::InvalidConfig(
                "Usage: /params load <file>\n\nProvide a path to a YAML parameter file.".to_string()
            ))?;
            let path = Path::new(path_str);
            state.params.load_file(path)?;
            writeln!(writer, "Loaded parameters from '{}'", path.display())?;
            // Show count of available variables
            let paths = state.params.list_available_paths();
            writeln!(writer, "{} variable(s) available.", paths.len())?;
        }
        Some("unload") => {
            state.params.clear();
            writeln!(writer, "All parameters cleared.")?;
        }
        Some("show") | None => {
            if state.params.is_empty() {
                writeln!(writer, "No parameters loaded.")?;
                writeln!(writer)?;
                writeln!(writer, "Use /params load <file> to load a YAML parameter file.")?;
            } else {
                writeln!(writer, "Loaded files:")?;
                for f in state.params.loaded_files() {
                    writeln!(writer, "  {}", f.display())?;
                }
                writeln!(writer)?;
                writeln!(writer, "Available variables:")?;
                for var_path in state.params.list_available_paths() {
                    writeln!(writer, "  {{{{{}}}}}", var_path)?;
                }
            }
        }
        Some(other) => {
            writeln!(writer, "Unknown /params subcommand: {}", other)?;
            writeln!(writer, "Usage: /params [load <file> | unload | show]")?;
        }
    }
    Ok(())
}
```

### SQL Substitution Hook

Substitution is applied in `executor.rs` before any SQL is sent to Teradata:

```rust
// src/commands/repl/executor.rs - in execute_sql()
let substituted = if !state.params.is_empty() {
    match state.params.substitute(trimmed) {
        Ok(s) => s,
        Err(e) => {
            writeln!(writer, "Parameter substitution error: {}", e)?;
            return Ok(0);
        }
    }
} else {
    trimmed.to_string()
};
// Execute substituted SQL...
```

In REPL mode, substitution errors are non-fatal: they print the error and return to the prompt.

### Tab Completion

Add `/params` to the metacommand completion registry in `metadata_completer.rs`:

```rust
MetacommandDef {
    name: "params",
    aliases: &["p"],
    description: "Manage variable substitution parameters",
},
```

Subcommand completion for `/params`:
- After `/params `, suggest `load`, `unload`, `show`
- After `/params load `, file path completion (using reedline's file completer if available)

### Help Text

Add to `print_help_extended()`:

```
  /params [show]           Show loaded parameter files and available variables
  /params load <file>      Load a YAML parameter file for variable substitution
  /params unload           Clear all loaded parameters
  /p                       Alias for /params
```

---

## Future Enhancements

- Query history search (Ctrl-R) - already supported by reedline
- Result export from REPL (\export)
- Session transcripts (\spool)
- Transaction control (\begin, \commit, \rollback)
- Async metadata loading (background thread)
- DDL-triggered cache invalidation
- Fuzzy matching for completion (like pgcli)
- Second TAB accepts selection (requires reedline enhancement)
- Session filtering for `/sessions` (by user, state, etc.)
- Additional `/show` subcommands (constraints, statistics, partitions)
- Performance resource monitoring (`/perf`) using ResUsage views
- Session history tracking (`/history`) using DBQL data
- Real-time auto-refresh for monitoring commands
- Explain plan inspection (`/explain <session_id>`) using DBQL step data
- AMP skew analysis (`/skew <session_id>`) using DBQL step-level metrics
- Full SQL text retrieval from DBQLSqlTbl (`--full-sql` flag for `/query`)

---

## /inspect Command

The `/inspect` command provides comprehensive object inspection, consolidating and extending the
existing `/describe`, `/show indexes`, and schema metadata commands into a single, rich view.

### Design Overview

`/inspect <object>` produces a multi-section report for any Teradata object (table, view, macro,
procedure). Each section is fetched independently; if one DBC view is inaccessible, the remaining
sections still render. This graceful-degradation strategy ensures the command is useful even in
environments with restricted DBC access.

```
/inspect employees
/inspect dbc.tables
\i orders
```

Alias: `\i` (mirrors the `\d` alias for `/describe`).

### Module: `src/commands/inspect.rs`

The inspect logic lives in a dedicated module following the same structural pattern as
`sessions.rs`, `locks.rs`, and `sysconfig.rs`. The module exposes two public entry points:

```rust
// src/commands/inspect.rs

/// REPL entry point — writes formatted multi-section output to writer.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()>

/// Batch-mode entry point — used by `tq inspect` CLI command.
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &InspectArgs,
    writer: &mut W,
    use_color: bool,
) -> Result<()>
```

### Object Name Resolution

Object names follow the `[database.]object` pattern already established by `/describe` and
`/show indexes`. The same inline split pattern used in `execute_describe()` and
`execute_show_indexes()` is applied:

```rust
// Inline qualified-name split (consistent with existing metacommands)
let (database, obj) = if let Some(dot_pos) = object_name.find('.') {
    (Some(&object_name[..dot_pos]), &object_name[dot_pos + 1..])
} else {
    (None, object_name)
};
```

When no database qualifier is present, the inspect module queries DBC without a database filter,
relying on the session default database. This matches the existing behaviour in `/describe`.
String values used in WHERE clauses are wrapped with `escape_sql_string()` from `src/sql.rs`
(already imported by `metacommands.rs`) to prevent injection through crafted object names.

### Section Architecture

Each section is an independent query function. Failures are caught per-section and surfaced as
`"(unavailable: <reason>)"` without aborting the whole report.

```rust
// Internal helpers

fn query_object_type(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<ObjectTypeInfo>

fn query_columns(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<QueryResult>

fn query_indexes(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<QueryResult>

fn query_storage(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<StorageInfo>

fn query_definition(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
    kind: &str,
) -> Result<String>

/// Human-readable byte counts: 1.5 KB, 2.3 MB, 1.1 GB, 4.2 TB
fn format_size(bytes: i64) -> String
```

### Data Types

```rust
/// Object kind returned by DBC.TablesV
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKind {
    Table,
    View,
    Macro,
    StoredProcedure,
    JoinIndex,
    HashIndex,
    Other(String),
}

impl ObjectKind {
    fn from_table_kind(kind: &str) -> Self {
        match kind.trim() {
            "T" | "O" => ObjectKind::Table,
            "V"       => ObjectKind::View,
            "M"       => ObjectKind::Macro,
            "P"       => ObjectKind::StoredProcedure,
            "J"       => ObjectKind::JoinIndex,
            "N"       => ObjectKind::HashIndex,
            other     => ObjectKind::Other(other.to_string()),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            ObjectKind::Table          => "Table",
            ObjectKind::View           => "View",
            ObjectKind::Macro          => "Macro",
            ObjectKind::StoredProcedure => "Stored Procedure",
            ObjectKind::JoinIndex      => "Join Index",
            ObjectKind::HashIndex      => "Hash Index",
            ObjectKind::Other(s)       => s.as_str(),
        }
    }
}

/// Result of the object-type lookup
pub struct ObjectTypeInfo {
    pub kind: ObjectKind,
    pub created: Option<String>,
    pub comment: Option<String>,
}

/// Aggregated storage metrics
pub struct StorageInfo {
    pub total_bytes: i64,
    pub peak_bytes: i64,
    pub max_amp_bytes: i64,
    pub avg_amp_bytes: f64,
    pub amp_count: i64,
    pub skew_pct: Option<f64>,
}
```

### SQL Queries

#### Object Type (DBC.TablesV)

```sql
SELECT TableKind,
       CAST(CreateTimeStamp AS VARCHAR(26)) AS Created,
       CommentString
FROM DBC.TablesV
WHERE DatabaseName = '{db}'
  AND TableName    = '{obj}'
```

SQL strings are built using `escape_sql_string()` from `src/sql.rs` (already imported in
`metacommands.rs`) to prevent injection through object names containing single quotes.

#### Columns (DBC.ColumnsV) — reuses `/describe` query

```sql
SELECT ColumnName, ColumnType, Nullable, DefaultValue, CommentString
FROM DBC.ColumnsV
WHERE DatabaseName = '{db}'
  AND TableName    = '{obj}'
ORDER BY ColumnId
```

#### Indexes (DBC.IndicesV) — reuses `/show indexes` query

```sql
SELECT IndexNumber, IndexType, UniqueFlag, IndexName, ColumnName, ColumnPosition
FROM DBC.IndicesV
WHERE DatabaseName = '{db}'
  AND TableName    = '{obj}'
ORDER BY IndexNumber, ColumnPosition
```

#### Storage (DBC.TableSizeV)

```sql
SELECT SUM(CurrentPerm)       AS TotalSize,
       SUM(PeakPerm)          AS PeakSize,
       MAX(CurrentPerm)       AS MaxAmpSize,
       AVG(CAST(CurrentPerm AS FLOAT)) AS AvgAmpSize,
       COUNT(*)               AS AmpCount
FROM DBC.TableSizeV
WHERE DatabaseName = '{db}'
  AND TableName    = '{obj}'
```

Skew is calculated in Rust:

```rust
fn compute_skew(max_amp: i64, avg_amp: f64) -> Option<f64> {
    if avg_amp > 0.0 {
        Some(((max_amp as f64 / avg_amp) - 1.0) * 100.0)
    } else {
        None
    }
}
```

A skew of 0% means perfectly even distribution. Values above ~20% indicate meaningful skew.

#### Definition (views and macros)

For `ObjectKind::View`, execute `SHOW VIEW {db}.{obj}`. For `ObjectKind::Macro`, execute
`SHOW MACRO {db}.{obj}`. For other object types, this section is omitted.

The SHOW statement returns the CREATE text as a character result. The definition section is
rendered verbatim, which lets users read dependencies directly from the text.

### Output Format

REPL and batch table output share the same section-oriented format:

```
Object: dbc.employees  (Table)
Created: 2023-08-14 09:12:33.00
Comment: Employee master table

=== Columns (7) ===
 # | Column Name  | Type         | Nullable | Default
---+--------------+--------------+----------+---------
 1 | EmployeeId   | INTEGER      | N        |
 2 | FirstName    | VARCHAR(50)  | Y        |
...

=== Indexes ===
Index 1 (Unique Primary Index):  EmployeeId
Index 2 (Non-unique Secondary Index: emp_dept_idx):  DepartmentId

=== Storage ===
Total size : 4.2 MB
Peak size  : 4.8 MB
AMP count  : 16
Skew       : 3.2%

(Size information unavailable)    ← shown when DBC.TableSizeV query fails
```

Sections are separated by blank lines. Section headers use the `=== Header ===` convention
already established by `/show indexes`.

### REPL Integration

Add the `/inspect` handler to `handle_metacommand_with_state()` in `metacommands.rs`:

```rust
// In the match block of handle_metacommand_with_state()
"inspect" | "i" => {
    if args.is_empty() {
        writeln!(writer, "Usage: /inspect <table_name>")?;
        writeln!(writer, "       /inspect <database>.<table_name>")?;
    } else {
        crate::commands::inspect::execute_for_repl(
            completion_state.client(),
            args[0],
            writer,
        )?;
    }
}
```

Add to `print_help_extended()`:

```
  /inspect <table>, /i   Comprehensive object inspection (type, columns, indexes, size)
```

### Tab Completion Integration

Add to the `METACOMMAND_REGISTRY` constant in `metadata_completer.rs`:

```rust
MetacommandDef {
    name: "inspect",
    aliases: &["i"],
    description: "Comprehensive object inspection",
},
```

The completion context for `/inspect ` after a space already resolves to
`CompletionContext::MetacommandArg`, which triggers table name completion using the existing
`complete_table_names()` path. No changes to completion context analysis are needed.

### Semicolon Stripping (Bug #32)

Both `handle_metacommand()` and `handle_metacommand_with_state()` currently call `input.trim()`
but do not strip trailing semicolons before splitting into command and arguments. Users who type
`/inspect employees;` will have `employees;` passed as the object name, causing a DBC query
failure.

The fix adds `trim_end_matches(';')` immediately after the leading prefix is stripped:

```rust
// Before (in both handler functions):
let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');

// After:
let without_prefix = trimmed
    .trim_start_matches('/')
    .trim_start_matches('\\')
    .trim_end_matches(';')
    .trim();
```

Applying `.trim()` again after `trim_end_matches(';')` handles the unusual case where the user
types `/ describe table ;` (spaces around the trailing semicolon). This matches the pattern
already used in `executor.rs`:

```rust
// src/commands/repl/executor.rs (reference pattern)
let trimmed = input.trim();
let sql = trimmed.trim_end_matches(';').trim();
```

The fix must be applied in both functions to keep them consistent:
- `handle_metacommand()` at approximately line 46
- `handle_metacommand_with_state()` at approximately line 255

## REPL-Batch Shared Logic Pattern

Several metacommands have batch-mode equivalents (CLI subcommands). To avoid code duplication,
the shared logic pattern extracts the core query and rendering code into dedicated command modules
that both REPL and batch modes call.

### Established Pattern

Commands that already follow this pattern:
- `/sessions` -> `crate::commands::sessions::execute_for_repl()` / `tq sessions`
- `/inspect` -> `crate::commands::inspect::execute_for_repl()` / `tq inspect`
- `/sample` -> `crate::commands::sample::execute_sample()` / `tq sample`
- `/peek` -> `crate::commands::sample::execute_peek()` / `tq peek`
- `/sysconfig` -> `crate::commands::sysconfig::execute_for_repl()` / `tq sysconfig`
- `/locks` -> `crate::commands::locks::execute_for_repl()` / `tq locks`

### Migration Status

**Already delegating** (completed in prior sprints):
- `/show indexes` -> `crate::commands::show_indexes::execute_for_repl()`

**Sprint 47 delegation targets:**

1. `/describe` -> `crate::commands::describe::execute_for_repl()`
2. `/list` -> `crate::commands::list::execute_for_repl()` (new function)

Each delegation follows the same steps:
1. The batch module provides `execute_for_repl()` that accepts `&DatabaseClient` and `&mut W`
2. The REPL metacommand handler delegates execution: `crate::commands::describe::execute_for_repl(client, table_name, writer)?`
3. The REPL handler retains argument parsing, usage help, and error wrapping
4. Output format is always `Table` for REPL mode

### `/describe` Delegation Detail

The existing `describe::execute_for_repl()` already has the correct signature:
```rust
pub fn execute_for_repl<W: Write>(client: &DatabaseClient, table_name: &str, writer: &mut W) -> Result<()>
```

The REPL handler `execute_describe()` (approximately 130 lines of inline SQL and formatting)
is replaced with a single delegation call. The batch module handles all SQL queries,
column type translation, and output formatting.

### `/list` Delegation Detail

The `list.rs` module currently lacks `execute_for_repl()`. A new function is added:
```rust
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    subcommand: &str,
    pattern: Option<&str>,
    database: Option<&str>,
    writer: &mut W,
) -> Result<()>
```

The REPL handler `execute_list()` dispatches to subcommand aliases (databases/db/dbs,
tables/table/t, views/view/v) and then delegates to the batch module. The `CompletionState`
cache lookup for `/list databases` is removed -- the batch module queries `DBC.DatabasesV`
directly. This simplifies the code at the cost of a database round-trip, which is acceptable
for an interactive command.

### Note on CompletionState

The REPL metacommand handlers receive `CompletionState` which wraps the `DatabaseClient`.
The extracted modules accept `&DatabaseClient` directly (obtained via `completion_state.client()`).
The `/list databases` cache optimization in `CompletionState` is preserved for tab completion
but is no longer used for the `/list databases` command output itself.

## Pager Exit Snapshot

When the user exits the pager (pressing `q` or `Esc`), the alternate screen is discarded and the terminal restores its previous content. The user loses all visual context of what they were looking at. The pager exit snapshot feature prints a static, plain-text reproduction of the last visible pager viewport to the normal terminal immediately after leaving the alternate screen.

**Related Specification**: `docs/specifications/repl.md`

### Architecture

```
Pager::run() event loop
        |
        v
    User presses q/Esc
        |
        v
    execute!(stdout, Show, LeaveAlternateScreen)
    disable_raw_mode()
        |
        v
    render_exit_snapshot(&mut io::stdout())   <-- NEW
        |
        v
    Return Ok(()) to caller
```

The snapshot renders AFTER the alternate screen is closed and raw mode is disabled. This means:
- Normal terminal semantics apply (newlines produce CR+LF automatically)
- No crossterm commands are needed or allowed
- The output is plain text that persists in the terminal scrollback

### Design Approach

The pager already has `#[cfg(test)]` methods (`render_border_to_buffer`, `render_header_to_buffer`, `render_row_to_buffer`) that produce plain-text output without ANSI escapes. The exit snapshot follows the same approach but is available in production builds and writes to a generic `&mut impl Write`.

Rather than duplicating code, the design extracts the plain-text rendering logic into shared methods that are used by both the test buffer rendering and the exit snapshot. The `#[cfg(test)]` attribute is removed from these methods, and they are refactored to write to `&mut impl Write` instead of `&mut Vec<u8>`.

### Method Signature

```rust
// src/commands/repl/pager.rs

impl Pager {
    /// Render a static snapshot of the current viewport to the given writer.
    ///
    /// Called after LeaveAlternateScreen and disable_raw_mode(), so normal
    /// terminal semantics apply. Uses \n line endings (not \r\n).
    /// No ANSI escape sequences. No crossterm commands.
    ///
    /// The snapshot includes:
    /// 1. The table (borders, header, visible data rows) using box-drawing chars
    /// 2. Hidden columns footer (if any columns are off-screen)
    /// 3. Row count and timing footer
    pub fn render_exit_snapshot(&self, writer: &mut impl Write) -> io::Result<()>
}
```

### Rendering Logic

The method computes the visible window from the pager's current state and renders each component sequentially.

#### 1. Visible Window Calculation

```rust
let visible_cols = self.visible_column_count();
let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
let end_row = (self.row_offset + self.page_size).min(self.data.row_count);
let hidden_left = self.hidden_columns_left();
let hidden_right = self.hidden_columns_right();
```

These are the same calculations used by `render()` and the existing `render_to_buffer()`.

#### 2. Table Body

The table body is rendered using plain-text helper methods that write borders, headers, and data rows. Each helper writes directly to the `impl Write` parameter.

**Border rendering** (`write_snapshot_border`):
- Uses the same box-drawing characters as the pager: `╭─┬╮ ├─┼┤ ╰─┴╯`
- Includes left/right indicator cell borders when columns are hidden
- Uses `\n` line endings (not `\r\n`)

**Header rendering** (`write_snapshot_header`):
- Column names centered using `pad_to_display_width()` (same function as pager)
- Left indicator shows `(+N cols)` when columns are hidden to the left
- Right indicator shows `(+N cols)` when columns are hidden to the right
- No color or styling

**Data row rendering** (`write_snapshot_row`):
- Cell values padded with `pad_to_display_width()` using each column's alignment
- Left indicator shows `<--` arrows, right indicator shows `-->`
- No NULL styling (plain `[NULL]` text)

#### 3. Hidden Columns Footer

When columns are hidden (either left or right), a footer message lists the hidden column names. This matches the format used by `src/format/table.rs`:

```rust
// Collect all hidden column names
let total_hidden = hidden_left + hidden_right;
if total_hidden > 0 {
    writeln!(writer)?;
    // Collect names from columns before col_offset and after end_col
    let hidden_names: Vec<&str> = self.data.columns.iter().enumerate()
        .filter(|(i, _)| *i < self.col_offset || *i >= end_col)
        .map(|(_, col)| col.name.as_str())
        .collect();
    writeln!(writer, "{} columns hidden: {}", total_hidden, hidden_names.join(", "))?;
    writeln!(writer, "Use \\format csv or \\format json to see all columns")?;
}
```

Note: The REPL uses `\format` (backslash metacommand), not `--format` (batch flag), so the hint text adapts to the REPL context.

#### 4. Row Count and Timing Footer

```rust
writeln!(writer, "{} row(s) in set ({:.3}s)", self.total_rows, self.execution_time.as_secs_f64())?;
```

This uses `self.total_rows` (the total result count, not just visible rows) and `self.execution_time` (already stored on the Pager struct).

### Integration into `Pager::run()`

The snapshot call is inserted after the alternate screen teardown:

```rust
pub fn run(&mut self) -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // ... existing event loop ...

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    // NEW: Print static snapshot of last viewport
    self.render_exit_snapshot(&mut io::stdout())?;

    Ok(())
}
```

### Refactoring: Shared Plain-Text Helpers

The existing `#[cfg(test)]` methods (`render_border_to_buffer`, `render_header_to_buffer`, `render_row_to_buffer`) are refactored:

1. Remove `#[cfg(test)]` attribute from the plain-text rendering helpers
2. Change signature from `(&self, buffer: &mut Vec<u8>, ...)` to `(&self, writer: &mut impl Write, ...) -> io::Result<()>`
3. The `render_to_buffer()` test method continues to exist as a thin wrapper that creates a `Vec<u8>` and calls the shared helpers
4. The `render_exit_snapshot()` production method calls the same shared helpers plus the footer logic

This eliminates code duplication and ensures the test rendering and production snapshot use identical logic.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| All columns visible | No hidden columns footer. No indicator cells in table. |
| Scrolled to rightmost column | Left indicator shows `(+N cols)`, no right indicator. Hidden footer lists left-hidden columns. |
| Scrolled to leftmost column (default) | No left indicator. Right indicator if columns overflow. Hidden footer lists right-hidden columns. |
| Empty result set | Should not reach pager (pager activation requires rows > threshold). Defensive: render empty table with header only. |
| Single row | Render normally. Row count shows "1 row(s) in set". |
| Single column | Render normally. No hidden columns. |
| Very wide terminal | All columns may fit. No indicators, no hidden footer. |

### Unit Testing Strategy

All snapshot tests write to a `Vec<u8>` buffer and verify the output string:

```rust
#[test]
fn test_exit_snapshot_basic() {
    let result = create_test_result(3, 5);
    let pager = create_pager_with_width(&result, 120);
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Verify structure
    assert!(output.starts_with("╭"));           // Top border
    assert!(output.contains("col0"));            // Header
    assert!(output.contains("val_0_0"));         // Data
    assert!(output.contains("row(s) in set"));   // Timing footer
}

#[test]
fn test_exit_snapshot_hidden_columns() {
    let result = create_wide_test_result(15, 3);
    let mut pager = create_pager_with_width(&result, 80);
    pager.col_offset = 3;
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("columns hidden:"));
    assert!(output.contains("\\format csv or \\format json"));
}

#[test]
fn test_exit_snapshot_all_columns_visible() {
    let result = create_test_result(2, 2);
    let pager = create_pager_with_width(&result, 200);
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    assert!(!output.contains("columns hidden"));
    assert!(!output.contains("<--"));
    assert!(!output.contains("-->"));
}

#[test]
fn test_exit_snapshot_no_ansi() {
    let result = create_test_result(3, 3);
    let pager = create_pager_with_width(&result, 120);
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Must not contain ANSI escape sequences
    assert!(!output.contains("\x1b["));
}

#[test]
fn test_exit_snapshot_uses_newline_not_cr() {
    let result = create_test_result(2, 2);
    let pager = create_pager_with_width(&result, 120);
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Must use \n, not \r\n
    assert!(!output.contains("\r\n"));
    assert!(output.contains("\n"));
}

#[test]
fn test_exit_snapshot_row_count_shows_total() {
    // Verify that the footer shows total rows, not just visible page
    let result = create_test_result(2, 100);
    let mut pager = create_pager_with_width(&result, 120);
    pager.page_size = 20;
    let mut buf = Vec::new();
    pager.render_exit_snapshot(&mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("100 row(s) in set"));
}
```

### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Snapshot entry point | `src/commands/repl/pager.rs` | `Pager::render_exit_snapshot()` |
| Plain-text border | `src/commands/repl/pager.rs` | `Pager::write_snapshot_border()` |
| Plain-text header | `src/commands/repl/pager.rs` | `Pager::write_snapshot_header()` |
| Plain-text row | `src/commands/repl/pager.rs` | `Pager::write_snapshot_row()` |
| Integration point | `src/commands/repl/pager.rs` | `Pager::run()` (after LeaveAlternateScreen) |
| Test buffer rendering | `src/commands/repl/pager.rs` | `Pager::render_to_buffer()` (test-only wrapper) |

### Design Decisions

**Why not reuse `format/table.rs`?**
The table formatter in `format/table.rs` uses `comfy-table` style rendering and its own column selection logic that starts from column 0. The pager snapshot must render from an arbitrary `col_offset` using the pager's own column width calculations and indicator cells. Reusing the table formatter would require passing the pager's state through an incompatible interface. The code duplication is minimal since the plain-text helpers already exist in pager.rs for testing.

**Why `&mut impl Write` instead of returning a `String`?**
Writing to a generic `Write` implementor follows the established pattern in `format/table.rs` and enables:
- Direct output to stdout without intermediate allocation
- Unit testing via `Vec<u8>` buffer
- Potential future use with file output

**Why show `\format` instead of `--format` in the hint?**
The pager only runs in REPL mode. In the REPL, users change format with the `\format` metacommand, not the `--format` CLI flag. The hint should match the available action.

**Why not strip indicator cells from the snapshot?**
The indicators (`(+N cols)`, `<--`, `-->`) provide valuable context about what portion of the result is visible. Stripping them would make the snapshot less informative than the pager view. The hidden columns footer provides the complete list of column names for reference.

---

## Watch Mode for Monitoring Commands

**Related Specification**: `docs/specifications/repl.md` (monitoring commands), GitHub Issue #25

### Overview

Watch mode is an auto-refreshing display for monitoring metacommands. A monitoring command (e.g. `/sessions`, `/locks`, `/resources`) invoked with `--watch` re-renders its output at a configurable interval until the user presses `q`, `Esc`, or `Ctrl-C`. On exit, the last rendered frame is left on the terminal as a copy-paste friendly plain-text snapshot — a direct parallel to the pager exit snapshot pattern.

Watch mode is shared infrastructure, not per-command. Each monitoring command supplies a render closure; `run_watch` owns the loop, the raw-mode lifecycle, keystroke handling, and the exit snapshot.

### Architecture

```
User types "/sessions --watch --interval 10"
        |
        v
metacommands.rs "sessions" handler
        |
        v
parse_watch_args(&args, default=6)  --> Some(10)
        |
        v
run_watch(10, |buf| sessions::execute_for_repl(client, buf))
        |
        v
┌────────── run_watch ──────────────────────────────────┐
│                                                       │
│  RawModeGuard::enable()   (RAII; restores on drop)    │
│                                                       │
│  loop {                                               │
│    clear screen, move to (0,0)                        │
│    render(&mut frame_buf)                             │
│    write frame_buf to stdout                          │
│    write header line (timestamp | interval | hint)    │
│                                                       │
│    poll loop until tick elapsed:                      │
│      event::poll(min(remaining, 100ms))               │
│      if key in {q, Q, Esc, Ctrl-C} -> break           │
│  }                                                    │
│                                                       │
│  <RawModeGuard drops here, even on panic>             │
│                                                       │
│  write last_frame_buf to stdout (exit snapshot)       │
└───────────────────────────────────────────────────────┘
        |
        v
Return to REPL prompt
```

### Implementation Location

**Primary file**: `src/commands/watch.rs` (already present; Sprint 65 hardens it)

**Callers** (all in `src/commands/repl/metacommands.rs`):
- `/sessions` handler (line ~544)
- `/locks` handler (line ~637)
- `/resources` handler (line ~716)

**Design principle**: All watch-mode behavior lives in `watch.rs`. The monitoring commands only supply a render closure; they know nothing about raw-mode, polling, or snapshots.

### Argument Parsing

`watch::parse_watch_args(&args, default_interval) -> Option<u64>` already exists and is reused as-is:

- `--watch`                    -> `Some(default)` (6 for `/sessions`)
- `--watch 10`                 -> `Some(10)` (positional shorthand)
- `--watch --interval 10`      -> `Some(10)` (explicit form)
- `--interval` without `--watch` -> `None` (watch mode not engaged)
- Clamped to `[2, 300]` seconds — minimum 2s protects the DB from accidental `--watch 0`; 300s ceiling prevents a "forgotten terminal" from holding a cached session indefinitely without any visible lifecycle cue.

**Why a dedicated `u64` seconds arg instead of the Sprint 61 `parse_duration`?** The `parse_duration` path supports `h`/`m`/`s` suffixes useful for *logoff thresholds* (which are in hours). Refresh intervals are always sub-minute; a plain integer is friction-free (`--interval 10`), and the existing `parse_watch_args` already ships with this contract. Reusing `parse_duration` would widen the grammar without a user-facing win.

### Loop Architecture: Polling Keystrokes While Ticking

The standard crossterm pattern — `event::poll(timeout)` in an inner loop until the tick interval elapses — is what `watch_loop` uses:

```rust
let start = Instant::now();
while start.elapsed() < interval {
    let remaining = interval.saturating_sub(start.elapsed());
    let poll_timeout = remaining.min(Duration::from_millis(100));
    if event::poll(poll_timeout)? {
        if let Event::Key(key) = event::read()? {
            if should_quit(&key) { return Ok(()); }
        }
    }
}
```

**Why 100ms inner poll bound instead of the full remaining interval?** Responsiveness on `q`/`Ctrl-C`: with a 10-second refresh, a single blocking `poll(10s)` would make the exit key feel unresponsive. Capping at 100ms guarantees ≤100ms exit latency regardless of interval.

**Why not also consume `Event::Resize`?** The next tick re-renders from scratch against the current terminal size, so resize events are naturally absorbed. No special handling needed, unlike the pager (which maintains persistent layout state between renders).

### Raw-Mode / Alternate-Screen Lifecycle (RAII)

**Current gap (Sprint 65 must fix)**: `run_watch` calls `terminal::enable_raw_mode()` and then relies on `let _ = terminal::disable_raw_mode()` at the end of the function. If `watch_loop` panics, the panic unwinds *past* the `let _ = ...` line and the user is left in a broken terminal. The pager has the same vulnerability but is better-isolated; watch mode is *more* exposed because it runs indefinitely.

**Fix**: introduce a `RawModeGuard` RAII type whose `Drop` impl unconditionally disables raw mode and leaves the alternate screen. This is idiomatic Rust and is the only correct pattern for terminal state spanning potentially-panicking code.

```rust
// src/commands/watch.rs

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Best-effort restore; ignore errors because Drop cannot propagate.
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
```

`run_watch` becomes:

```rust
pub fn run_watch<F>(interval_secs: u64, render: F) -> Result<()>
where F: Fn(&mut Vec<u8>) -> Result<()> {
    let _guard = RawModeGuard::enter()?;   // guard drops on any return path
    let last_frame = watch_loop(interval_secs, &render)?;
    drop(_guard);                           // explicit: leave alt-screen first
    print_exit_snapshot(&last_frame)?;      // then snapshot on primary screen
    Ok(())
}
```

**Alternate screen**: Watch mode enters the alternate screen on start and leaves it on exit. This matches the pager contract and prevents the scrollback buffer from being spammed with dozens of stale frames. The exit snapshot is printed *after* leaving the alternate screen, so it lands in the user's persistent scrollback — exactly the Sprint 63 pattern.

### Query Execution Per Tick

Each tick, the render closure executes the command's existing `execute_for_repl` function against the shared `DatabaseClient`. No query caching, no delta computation — the monitoring queries (`MonitorSession`, lock view, `MonitorPhysicalResource`) are already cheap, and watch mode's whole point is fresh data.

**Session reuse**: The `DatabaseClient` reference captured by the closure is the REPL's live connection. No reconnect per tick. If the session dies mid-watch, the closure returns an error — see error handling below.

**Table formatting**: Each command's existing `execute_for_repl` already produces formatted output (comfy-table + footer). The watch loop does not touch this — it writes the rendered bytes to stdout verbatim. This guarantees watch-mode output matches non-watch output column-for-column.

### Error Handling Per Tick

**Current gap (Sprint 65 must fix)**: `watch_loop` propagates errors from `render(&mut buf)?` out of the loop, aborting watch mode on the first transient DB hiccup. Acceptance criterion explicitly requires: "If a refresh query fails … display the error in the frame header and keep trying on the next tick."

**Fix**: catch the render error and display it in the frame header instead of propagating:

```rust
let mut buf = Vec::new();
let render_status = match render(&mut buf) {
    Ok(()) => None,
    Err(e) => Some(format!("Query error: {} (retrying...)", e)),
};

execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
if let Some(err_msg) = &render_status {
    // Red header line
    writeln!(stdout, "\x1b[31m[!] {}\x1b[0m", err_msg)?;
}
stdout.write_all(&buf)?;   // may be partial; that's fine
```

**Which errors count?** Any `crate::error::Error` returned from the closure is caught and displayed. I/O errors on stdout itself (from `write_all`) propagate — these indicate a broken terminal and watch mode cannot recover. Panics are caught by `RawModeGuard`'s `Drop` and then re-propagate past `run_watch`.

### Exit Snapshot: Sharing the Pager Pattern

The pager's `render_exit_snapshot` is specific to `Pager` state (column offsets, paging window). It cannot be called directly from watch mode, which has no such state. However, the *pattern* is identical: after leaving the alternate screen and disabling raw mode, print one last plain-text version of the content to the user's persistent scrollback.

**Design choice**: **duplicate the pattern, not the code.** Watch mode's content is already plain text (produced by `execute_for_repl` — comfy-table output with no ANSI, no raw-mode-specific `\r\n`). We simply retain the last rendered frame buffer in memory and write it to stdout after the raw-mode guard drops.

```rust
// Inside watch_loop, keep the most recently rendered buffer
let mut last_frame: Vec<u8> = Vec::new();
loop {
    let mut buf = Vec::new();
    // ... render into buf, display ...
    last_frame = buf;   // retain for snapshot
    // ... poll for exit ...
}
// return last_frame from watch_loop; run_watch prints it after guard drops
```

**Why not wrap `Pager::render_exit_snapshot`?** That method owns formatting logic specific to Pager state. Watch mode's content is already formatted by the command's own formatter. Sharing code here would require either (a) passing a `QueryResult` into watch mode (violates the "watch mode is transport-only" boundary), or (b) extracting a generic "plain-text table" helper from the pager that neither caller actually needs. The current design keeps watch mode as pure transport and defers all formatting to the monitoring command.

**Header line excluded from snapshot**: The `Last updated: HH:MM:SS | Refreshing every Ns | Press q ...` footer is watch-mode UI chrome, not data. It is written to the alternate screen only. The exit snapshot contains just the last frame's rendered command output plus a single `Exited watch mode at HH:MM:SS` line.

### Frame Header Line (Within Alternate Screen)

Acceptance criterion: "Each refresh shows … plus a header line with timestamp and the configured interval."

```
Last updated: 14:22:36 | Refreshing every 6s | Press q or Ctrl-C to stop
```

This is the *bottom* status line today; Sprint 65 keeps it there — users expect the freshest data at the top and the status chrome at the bottom (matches `top`, `watch(1)`, `htop`). The timestamp uses `chrono::Local::now().format("%H:%M:%S")` — already implemented in `format_timestamp()`.

### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Watch entry point | `src/commands/watch.rs` | `run_watch` |
| RAII guard | `src/commands/watch.rs` | `RawModeGuard` (new) |
| Inner loop | `src/commands/watch.rs` | `watch_loop` (returns last frame) |
| Argument parsing | `src/commands/watch.rs` | `parse_watch_args` (reused unchanged) |
| Exit key detection | `src/commands/watch.rs` | `should_quit` |
| Session integration | `src/commands/repl/metacommands.rs` | `/sessions` arm (~L544) |
| Locks integration | `src/commands/repl/metacommands.rs` | `/locks` arm (~L637) |
| Resources integration | `src/commands/repl/metacommands.rs` | `/resources` arm (~L716) |

### Unit Testing Strategy

Headless interactive tests are notoriously flaky, so the testable surface is kept small:

- `parse_watch_args`: already comprehensively tested; extend with `--interval 0` -> clamped to 2 (minimum), `--interval -5` -> rejected as non-u64.
- `should_quit`: already tested for `q`/`Q`/`Esc`/`Ctrl-C`; extend with `Ctrl-D` (does not quit), `Shift+Q` (does quit).
- `RawModeGuard` drop behavior: unit test cannot reliably toggle real raw mode, but a mockable variant (behind a trait) can verify Drop is called on panic via `std::panic::catch_unwind`.
- Render-closure error propagation: inject a closure that returns `Err` on tick 1 and `Ok` on tick 2; assert the loop does NOT exit and that the error header is written to the output sink.
- Last-frame retention: inject a closure that writes incrementing counters; after the loop exits, assert the returned last-frame buffer matches the final counter.

For end-to-end exit-snapshot behavior, an integration test drives watch mode for 2 ticks via a scripted keystroke injection and verifies the exit snapshot contains the last frame's session data.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Interval `0` / `1` via `--interval 0` | Clamped to 2 seconds (DB safety floor) |
| Interval `9999` via `--interval 9999` | Clamped to 300 seconds |
| Non-numeric interval (`--watch foo`) | Falls back to default (6) |
| DB connection drops mid-watch | Error rendered in frame header; loop continues; next tick retries and recovers if DB is back |
| Terminal resized mid-watch | Next tick re-renders at new size; no layout corruption |
| `Ctrl-C` inside render closure | `watch_loop` catches the render error (if surfaced as `Err`), displays it, continues. Bare-process SIGINT signal handling is Rust/terminal default behavior; raw-mode disables the kernel Ctrl-C -> SIGINT translation so Ctrl-C becomes a keystroke and exits watch cleanly via `should_quit`. |
| Panic inside render closure | Propagates up through `watch_loop`; `RawModeGuard::drop` restores terminal state before the panic continues unwinding |
| Zero rows in result (no active sessions) | Command's own formatter handles this ("0 active sessions") |
| Very large result set (1000+ sessions) | Rendered in full each tick; no pagination inside watch — user should filter via SQL if needed (deferred) |

### Design Decisions

**Why share `watch.rs` across `/sessions`, `/locks`, `/resources`?**
All three are monitoring commands with the same interaction shape: re-render periodically, quit on keystroke. The closure-based design (`F: Fn(&mut Vec<u8>) -> Result<()>`) keeps the per-command cost to one line of wiring. Adding `/cpu` or `/gpu` in future sprints costs zero new watch infrastructure.

**Why not `tokio` + async polling?**
The rest of `tq` is synchronous — adding a Tokio runtime for this one feature pulls in a large dependency and a whole different error-handling idiom. Crossterm's `event::poll` with a timeout gives us the same behavior in 20 lines of synchronous code.

**Why not a full TUI via `ratatui`?**
Explicitly out of scope per sprint planning. Watch mode is an incremental, low-risk extension of the existing REPL. A ratatui TUI is a separate sprint if demand emerges.

**Why retain the last frame in memory rather than re-querying on exit?**
Re-querying on exit would show the user something *different* from what they just saw before pressing `q`, and would incur a final DB round-trip after the user has indicated they want to leave. The Sprint 63 pager pattern is clear: show the *exact last frame*, no surprises.

**Why hide the cursor (`Hide`) during watch?**
The cursor would blink over the rendered table, visually noisy. `Show` is restored by the RAII guard on exit.
