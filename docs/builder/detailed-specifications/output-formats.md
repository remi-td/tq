# Output Format Specifications

**Version:** 2.0.0 (Sprint 11 Simplified Table Design)
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification - Critical Update for Sprint 11

---

## Table of Contents

1. [Format Selection](#81-format-selection)
2. [Table Format](#82-table-format)
3. [Implementation Guide for Table Truncation](#82x-implementation-guide-for-table-truncation)
4. [JSON Format](#83-json-format)
5. [CSV Format](#84-csv-format)
6. [Format Comparison](#85-format-comparison)

---

## 8.1 Format Selection

### 8.1.1 Selection Priority

1. **Command-line flag**: `--format table`
2. **Environment variable**: `TQ_FORMAT=json`
3. **Configuration file**: `format = "csv"`
4. **Context-based default**:
   - TTY (interactive): `table`
   - Piped: `csv` or `json` (configurable)

### 8.1.2 Format Types

| Format | Use Case | File Extension | MIME Type |
|--------|----------|----------------|-----------|
| `table` | Human-readable, interactive | - | `text/plain` |
| `json` | Scripting, APIs, parsing | `.json` | `application/json` |
| `csv` | Data export, Excel, analysis | `.csv` | `text/csv` |

## 8.2 Table Format

### 8.2.1 Design Philosophy

**IMPORTANT:** The table formatting approach has been simplified to focus on terminal width awareness rather than complex padding logic. This design prioritizes:

1. **Simplicity** - No complex padding calculations that are prone to breaking
2. **Terminal Width Awareness** - Detect terminal width and show columns that fit
3. **Clear Truncation** - Explicitly indicate when columns are hidden
4. **Batch Mode Compatibility** - Show all columns in non-TTY contexts

**Why No Padding?**
- Padding logic has repeatedly broken in Sprints 6, 8, and 11
- Proper padding requires visual regression testing framework (not yet available)
- Terminal width detection + column truncation is simpler and more robust
- Padding feature is postponed until proper testing infrastructure exists

### 8.2.2 Terminal Width Detection

**Interactive Mode (TTY):**
- Detect terminal width using `terminal_size` crate
- Default to 80 columns if detection fails
- Recalculate on window resize (best effort)

**Batch Mode (Non-TTY):**
- Show ALL columns regardless of width
- No truncation or column hiding
- Optimized for piping and file redirection

### 8.2.3 Column Truncation Strategy

When columns don't fit in terminal width:

1. **Prioritize Leftmost Columns** - Show columns from left to right until width exhausted
2. **Calculate Minimum Width** - Each column gets minimum width based on content:
   - Column name length + 2 (for padding)
   - Or longest value in first N rows (sample-based, max 100 rows)
3. **Add Truncation Indicator** - When columns are hidden:
   - Header: Show `| (+n cols) |` in rightmost position
   - Body: Show `| ... |` in rightmost position
4. **No Padding** - Columns are NOT padded, just basic spacing

**Example with 80-column terminal:**

```
┌─────┬──────────┬────────┬──────────┬─────────────┐
│ id  │ username │ active │ dept     │ (+3 cols)   │
├─────┼──────────┼────────┼──────────┼─────────────┤
│ 1   │ alice    │ true   │ eng      │ ...         │
│ 2   │ bob      │ false  │ sales    │ ...         │
│ 3   │ charlie  │ true   │ ops      │ ...         │
└─────┴──────────┴────────┴──────────┴─────────────┘

3 columns hidden: email, created_at, last_login
3 rows in set (0.045s)
```

**Footer Message:**
- When columns are truncated, show: `n columns hidden: col1, col2, col3`
- Suggest: `Use --format csv or --format json to see all columns`

### 8.2.4 ASCII Table (Default)

```
┌────┬───────┬────────┐
│ id │ name  │ active │
├────┼───────┼────────┤
│ 1  │ Alice │ true   │
│ 2  │ Bob   │ false  │
└────┴───────┴────────┘

2 rows in set (0.123s)
```

**Features**:
- Box-drawing characters (┌─┬─┐ │ ├─┼─┤ └─┴─┘)
- Minimal spacing (no padding)
- Basic column alignment
- Row count and timing footer
- Terminal width awareness

### 8.2.5 Simple Table (--table-style simple)

```
id | name  | active
---+-------+-------
 1 | Alice | true
 2 | Bob   | false

(2 rows)
```

**Use Case**: Better for copying/pasting, terminal compatibility

### 8.2.6 Compact Table (--table-style compact)

```
id name  active
 1 Alice true
 2 Bob   false
```

**Use Case**: Dense output, logs

### 8.2.7 Markdown Table (--table-style markdown)

```
| id | name  | active |
|----|-------|--------|
| 1  | Alice | true   |
| 2  | Bob   | false  |
```

**Use Case**: Documentation, GitHub issues

### 8.2.8 Column Alignment

**Basic alignment only (no padding):**
- **Numbers**: Right-aligned within minimum space
- **Text**: Left-aligned within minimum space
- **Booleans**: Left-aligned within minimum space
- **Dates**: Left-aligned within minimum space

**Note**: No fancy padding or column width calculations. Just basic alignment.

### 8.2.9 NULL Representation

```
┌────┬───────┬────────┐
│ id │ name  │ email  │
├────┼───────┼────────┤
│ 1  │ Alice │ a@e.co │
│ 2  │ Bob   │ [NULL] │
└────┴───────┴────────┘
```

**NULL Values:**
- Display as `[NULL]` in table cells
- Future enhancement: gray/italic styling (low priority)

---

## 8.2.x Implementation Guide for Table Truncation

### Purpose

This section provides **unambiguous implementation requirements** for the rust-teradata-architect agent. The goal is to remove all broken padding logic and implement a simple, robust terminal-width-aware column truncation feature.

### Implementation Requirements

#### 1. Remove All Padding Logic (MANDATORY)

**Action:** Delete or disable all code related to column padding calculations.

**Why:** Padding logic has broken repeatedly in Sprints 6, 8, and 11. It is not worth the maintenance burden until we have proper visual testing infrastructure.

**What to Remove:**
- Any code calculating column widths based on content
- Any code adding spaces to align columns
- Any complex table layout algorithms
- Any "smart" column sizing logic

**What to Keep:**
- Basic box-drawing characters (┌─┬─┐ │ ├─┼─┤ └─┴─┘)
- Basic column separators (│)
- Row separators (horizontal lines)

#### 2. Detect Terminal Width

**Requirement:** Use the `terminal_size` crate to detect terminal width.

**Implementation:**

```rust
use terminal_size::{Width, terminal_size};

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80  // Default fallback
    }
}
```

**Behavior:**
- **TTY (Interactive):** Detect terminal width, default to 80 if detection fails
- **Non-TTY (Batch Mode):** Return `usize::MAX` (no truncation)

**Detection Strategy:**
```rust
fn should_truncate_columns() -> bool {
    atty::is(atty::Stream::Stdout)  // Only truncate in TTY mode
}
```

#### 3. Calculate Minimum Column Widths

**Requirement:** For each column, calculate the minimum width needed to display content.

**Algorithm:**

```rust
fn calculate_min_column_width(column_name: &str, sample_values: &[String]) -> usize {
    let header_width = column_name.len();
    let max_value_width = sample_values
        .iter()
        .take(100)  // Sample first 100 rows only
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    // Minimum width is max of header and content, plus 2 for spacing
    std::cmp::max(header_width, max_value_width) + 2
}
```

**Important Notes:**
- Sample **at most 100 rows** to avoid performance issues
- Add 2 characters for basic spacing around content (1 space on each side)
- Do NOT add extra padding beyond this basic spacing

#### 4. Select Columns to Display

**Requirement:** Prioritize leftmost columns and add truncation indicator if needed.

**Algorithm:**

```rust
struct ColumnSelection {
    visible_columns: Vec<usize>,  // Indices of columns to show
    hidden_count: usize,           // Number of hidden columns
    truncation_indicator_width: usize,  // Width needed for "(+n cols)" column
}

fn select_visible_columns(
    column_widths: &[usize],
    terminal_width: usize
) -> ColumnSelection {
    if !should_truncate_columns() {
        // Batch mode: show all columns
        return ColumnSelection {
            visible_columns: (0..column_widths.len()).collect(),
            hidden_count: 0,
            truncation_indicator_width: 0,
        };
    }

    let mut visible = Vec::new();
    let mut used_width = 0;

    // Reserve space for truncation indicator: "| (+n cols) |"
    // Estimate: 15 characters should be enough
    let truncation_width = 15;

    // Account for table borders and separators
    let border_width = 2;  // Left and right borders

    for (idx, &col_width) in column_widths.iter().enumerate() {
        let separator_width = if visible.is_empty() { 0 } else { 1 };  // │ between columns
        let new_width = used_width + border_width + col_width + separator_width;

        // Check if we can fit this column + potential truncation indicator
        if visible.len() < column_widths.len() - 1 {
            // Not the last column, need room for truncation indicator
            if new_width + truncation_width <= terminal_width {
                visible.push(idx);
                used_width = new_width;
            } else {
                break;  // Stop adding columns
            }
        } else {
            // Last column, no truncation indicator needed
            if new_width <= terminal_width {
                visible.push(idx);
                used_width = new_width;
            } else {
                break;
            }
        }
    }

    let hidden_count = column_widths.len() - visible.len();

    ColumnSelection {
        visible_columns: visible,
        hidden_count,
        truncation_indicator_width: if hidden_count > 0 { truncation_width } else { 0 },
    }
}
```

#### 5. Render Table with Truncation Indicator

**Header Rendering:**

```rust
fn render_header(
    column_names: &[String],
    selection: &ColumnSelection
) -> String {
    let mut header = String::from("┌");

    // Visible columns
    for (i, &col_idx) in selection.visible_columns.iter().enumerate() {
        if i > 0 { header.push_str("┬"); }
        let name = &column_names[col_idx];
        header.push_str(&"─".repeat(name.len() + 2));
    }

    // Truncation indicator column
    if selection.hidden_count > 0 {
        header.push_str("┬");
        header.push_str(&"─".repeat(selection.truncation_indicator_width));
    }

    header.push_str("┐\n│");

    // Column names
    for (i, &col_idx) in selection.visible_columns.iter().enumerate() {
        if i > 0 { header.push_str("│"); }
        let name = &column_names[col_idx];
        header.push_str(&format!(" {} ", name));
    }

    // Truncation indicator: "(+n cols)"
    if selection.hidden_count > 0 {
        header.push_str("│");
        header.push_str(&format!(" (+{} cols) ", selection.hidden_count));
    }

    header.push_str("│\n");
    // ... separator line ...

    header
}
```

**Body Row Rendering:**

```rust
fn render_row(
    row_values: &[String],
    selection: &ColumnSelection
) -> String {
    let mut row = String::from("│");

    // Visible columns
    for (i, &col_idx) in selection.visible_columns.iter().enumerate() {
        if i > 0 { row.push_str("│"); }
        let value = &row_values[col_idx];
        row.push_str(&format!(" {} ", value));
    }

    // Truncation indicator: "..."
    if selection.hidden_count > 0 {
        row.push_str("│");
        row.push_str(" ... ");
    }

    row.push_str("│\n");
    row
}
```

**Footer Message:**

```rust
fn render_footer(
    row_count: usize,
    execution_time: f64,
    hidden_columns: &[String]  // Names of hidden columns
) -> String {
    let mut footer = String::new();

    if !hidden_columns.is_empty() {
        footer.push_str(&format!(
            "\n{} columns hidden: {}\n",
            hidden_columns.len(),
            hidden_columns.join(", ")
        ));
        footer.push_str("Use --format csv or --format json to see all columns\n");
    }

    footer.push_str(&format!("\n{} rows in set ({:.3}s)\n", row_count, execution_time));
    footer
}
```

#### 6. Testing Requirements

**Unit Tests Required:**

1. **Test: Terminal width detection**
   - Verify TTY detection works
   - Verify batch mode returns usize::MAX
   - Verify fallback to 80 columns

2. **Test: Column selection with various terminal widths**
   - 80 columns: Show 3 columns, hide 5
   - 120 columns: Show 5 columns, hide 3
   - 200 columns: Show all 8 columns
   - Batch mode: Always show all columns

3. **Test: Truncation indicator rendering**
   - Header shows "| (+n cols) |" when columns hidden
   - Body shows "| ... |" when columns hidden
   - Footer lists hidden column names
   - No indicator when all columns visible

4. **Test: Edge cases**
   - 1 column (no truncation possible)
   - All columns fit exactly (no truncation)
   - Terminal width too small (show at least 1 column)
   - Very long column names

**Integration Tests Required:**

1. Execute query with 10 columns in 80-column terminal
2. Execute same query in batch mode (all columns visible)
3. Execute query with columns that exactly fit terminal width
4. Verify footer message appears when columns are truncated

#### 7. No Padding - Just Basic Spacing

**CRITICAL:** Do NOT implement any padding beyond the basic spacing in step 3.

**What this means:**
- Add 1 space before content
- Add 1 space after content
- Total: `content.len() + 2`
- **NO** additional alignment
- **NO** column width normalization
- **NO** smart formatting

**Example (Correct):**
```
│ id │ name │ active │
│ 1  │ Alice │ true  │
│ 42 │ Bob  │ false │
```

**Example (Incorrect - DO NOT DO THIS):**
```
│ id │ name  │ active │
│  1 │ Alice │ true   │
│ 42 │ Bob   │ false  │
```

The incorrect example shows padding/alignment. We do NOT want this. Keep it simple.

### Summary Checklist for Implementation

- [ ] Remove ALL existing padding logic from codebase
- [ ] Implement terminal width detection (TTY vs batch mode)
- [ ] Calculate minimum column widths (header + sample content + 2)
- [ ] Select visible columns (leftmost columns that fit)
- [ ] Render header with "(+n cols)" indicator if needed
- [ ] Render body rows with "..." indicator if needed
- [ ] Render footer with hidden column names and suggestion
- [ ] Add unit tests for column selection algorithm
- [ ] Add integration tests for TTY vs batch mode behavior
- [ ] Verify NO padding logic remains in codebase
- [ ] Verify batch mode shows ALL columns
- [ ] Verify TTY mode truncates correctly

---

## 8.3 JSON Format

### 8.3.1 Array of Objects (Default)

```json
[
  {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com",
    "active": true,
    "created_at": "2024-01-15T10:30:00Z"
  },
  {
    "id": 2,
    "name": "Bob",
    "email": "bob@example.com",
    "active": false,
    "created_at": "2024-01-16T11:45:00Z"
  }
]
```

**Features**:
- Each row is a JSON object
- Column names as keys
- Type preservation (numbers, booleans, null)
- ISO 8601 for dates/timestamps

### 8.3.2 Streaming JSONL (--json-format lines)

```jsonl
{"id":1,"name":"Alice","email":"alice@example.com","active":true}
{"id":2,"name":"Bob","email":"bob@example.com","active":false}
```

**Use Case**: Large datasets, streaming processing

### 8.3.3 Metadata Wrapper (--json-format wrapped)

```json
{
  "query": "SELECT id, name FROM users",
  "execution_time_ms": 123,
  "row_count": 2,
  "columns": [
    {"name": "id", "type": "INTEGER"},
    {"name": "name", "type": "VARCHAR"}
  ],
  "rows": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ]
}
```

**Use Case**: APIs, complete metadata

### 8.3.4 Type Mapping

| Teradata Type | JSON Type | Example |
|---------------|-----------|---------|
| INTEGER, BIGINT | number | `42` |
| DECIMAL, FLOAT | number | `3.14` |
| VARCHAR, CHAR | string | `"text"` |
| DATE | string | `"2024-01-15"` |
| TIMESTAMP | string | `"2024-01-15T10:30:00Z"` |
| BOOLEAN | boolean | `true`, `false` |
| NULL | null | `null` |
| BLOB, CLOB | string (base64) | `"YWJjMTIz"` |

## 8.4 CSV Format

### 8.4.1 Standard CSV (RFC 4180)

```csv
id,name,email,active,created_at
1,Alice,alice@example.com,true,2024-01-15T10:30:00Z
2,Bob,bob@example.com,false,2024-01-16T11:45:00Z
```

**Features**:
- Header row (optional with `--no-header`)
- Double-quote escaping for special characters
- Comma separator (configurable with `--delimiter`)
- LF line endings (`\n`)

### 8.4.2 Excel-Compatible CSV

```bash
tq query --format csv --excel "SELECT * FROM users" > users.csv
```

**Differences**:
- BOM (Byte Order Mark) for UTF-8
- CRLF line endings (`\r\n`)
- Date format: `YYYY-MM-DD`

### 8.4.3 Custom Delimiter (TSV)

```bash
tq query --format csv --delimiter '\t' "SELECT * FROM data" > data.tsv
```

Output:
```tsv
id      name    email
1       Alice   alice@example.com
2       Bob     bob@example.com
```

### 8.4.4 Escaping Rules

```csv
id,name,description
1,Alice,"Senior Engineer, Team Lead"
2,Bob,"Quote: ""Hello World"""
3,Charlie,"Line 1
Line 2"
```

**Rules**:
- Fields with commas → quoted
- Fields with quotes → quoted, quotes doubled
- Fields with newlines → quoted

### 8.4.5 NULL Representation

```csv
id,name,email
1,Alice,alice@example.com
2,Bob,
```

**Options**:
- Empty field (default)
- `--null-string "NULL"` → explicit marker

## 8.5 Format Comparison

| Feature | Table | JSON | CSV |
|---------|-------|------|-----|
| Human-readable | ✅ Excellent | ⚠️ Okay | ⚠️ Okay |
| Machine-parseable | ❌ Poor | ✅ Excellent | ✅ Good |
| Type preservation | ❌ No | ✅ Yes | ❌ No (all strings) |
| Streaming friendly | ✅ Yes | ⚠️ JSONL only | ✅ Yes |
| Excel compatible | ❌ No | ❌ No | ✅ Yes |
| File size | N/A | Medium | Small |
| Processing speed | Fast | Medium | Fast |

---

## Sprint 11 Implementation Summary

### What Changed

**Version 2.0.0** introduces a **simplified table truncation approach** that removes all broken padding logic in favor of terminal-width-aware column display.

### Key Changes

1. **Padding Removed** - All column padding logic deleted (repeatedly broke in Sprints 6, 8, 11)
2. **Terminal Width Detection** - Use `terminal_size` crate to detect terminal width
3. **Column Truncation** - Show leftmost columns that fit, hide the rest
4. **Clear Indicators** - Display "(+n cols)" in header and "..." in body when columns are hidden
5. **Batch Mode Exception** - Show ALL columns in non-TTY contexts (no truncation)

### Why This Change

The user explicitly requested this change after padding broke multiple times:
- "Broken AGAIN with the padding!!!"
- "Please stop the padding for now and postpone it for much later"
- "You will need a test framework that will enable you to 'see' like the user"

### Implementation Approach

The implementation is **deliberately simple** to avoid the complexity that caused previous failures:

- NO complex width calculations
- NO padding or alignment (just basic spacing)
- YES to terminal width detection
- YES to column selection based on available width
- YES to clear truncation indicators

### For rust-teradata-architect Agent

**Read Section 8.2.x** carefully - it contains:
- Complete implementation requirements with code examples
- Explicit algorithm specifications
- Testing requirements checklist
- Edge case handling
- What NOT to do (no padding!)

**Success Criteria:**
- All padding code removed from codebase
- Terminal width detection working (TTY vs batch mode)
- Column truncation working correctly
- "(+n cols)" indicator appears when needed
- Footer message lists hidden columns
- Batch mode shows all columns (no truncation)
- All tests pass (unit + integration)

### Future Work (Postponed)

**Proper Padding Implementation** is postponed until:
- Visual regression testing framework is built
- Extensive research on table rendering best practices
- Comprehensive test coverage for visual output

Until then, the simplified truncation approach is the specification.

---
