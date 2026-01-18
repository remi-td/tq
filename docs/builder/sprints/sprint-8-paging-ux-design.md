# Sprint 8: Result Paging UX Design

**Date:** 2026-01-18
**Designer:** cli-ux-designer agent
**Status:** Final Design - Ready for Implementation

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problem Analysis](#problem-analysis)
3. [Design Solution](#design-solution)
4. [Implementation Guidance](#implementation-guidance)
5. [Test Scenarios](#test-scenarios)

---

## Executive Summary

This document provides a complete redesign of tq's result paging UX to address three critical issues discovered in Sprint 8 Round 1 testing:

1. **Issue 3.1 (CRITICAL):** 'q' exits entire program instead of returning to REPL
2. **Issue 3.2 (CRITICAL):** Wide tables (20+ columns) are unreadable when squeezed into terminal width
3. **Issue 3.3 (HIGH):** Long cell values (500+ characters) make tables unusable

The design prioritizes safety (no accidental session loss), usability (readable tables at any width), and familiarity (navigation similar to `psql`, `less`, and other standard pagers).

---

## Problem Analysis

### Why Current Paging Fails

**Issue 3.1: 'q' Kills Program**
- **Root Cause:** `minus::page_all()` treats 'q' as application exit signal, not pager exit
- **Impact:** Users lose REPL session, connection, command history
- **User Expectation:** 'q' should close pager and return to `tq>` prompt (like `psql`, `less`)

**Issue 3.2: Wide Tables Squeezed**
- **Root Cause:** Current implementation displays ALL columns at once, compressed to fit terminal width
- **Impact:** 20+ columns become 2-3 character wide vertical strips, completely unreadable
- **Teradata Reality:** Tables often have 30-50 columns in data warehouses
- **User Expectation:** Show reasonable number of readable columns, use left/right navigation for rest

**Issue 3.3: Long Cell Values Break Layout**
- **Root Cause:** Full cell content displayed regardless of length
- **Impact:** 500+ character VARCHAR values stretch columns, break table formatting
- **User Expectation:** Truncate long values with indicator, provide way to view full content if needed

### Teradata-Specific Challenges

Teradata databases present unique challenges:
- **Wide schemas:** 20-50 columns per table is common in data warehouses
- **Long values:** VARCHAR(2000) columns frequently contain hundreds of characters
- **Analytical queries:** SELECT * is common for data exploration
- **Result volume:** Queries often return hundreds or thousands of rows

These characteristics demand intelligent display strategies, not simple table rendering.

---

## Design Solution

### Design Goals

1. **Safety First:** Never accidentally exit REPL from pager
2. **Readability:** All displayed content must be readable at standard terminal widths (80-120 chars)
3. **Discoverability:** Clear indicators showing navigation options and current position
4. **Familiarity:** Navigation similar to `less`, `psql`, and other standard tools
5. **Performance:** Smooth navigation without lag or flicker

### Design Approach: Three-Layer Strategy

Our solution uses a three-layer approach to make any result set readable:

```
Layer 1: Column Windowing    →  Limit how many columns show at once
Layer 2: Cell Truncation     →  Limit content length within columns
Layer 3: Row Paging          →  Paginate long result sets vertically
```

Each layer independently solves one dimension of the problem.

---

## Design Solution: Layer 1 - Column Windowing

### Objective
Display a manageable subset of columns at once, allowing horizontal navigation through additional columns.

### Column Window Size: Dynamic Calculation

**Algorithm:**
```
1. Start with first column (leftmost)
2. Calculate minimum usable width for column:
   - min_width = max(column_name.length + 2, 8)  // At least 8 chars
   - max_width = 40  // Never exceed 40 chars per column
3. Add columns until total_width > (terminal_width - 10)
   - Reserve 10 chars for borders and margins
4. Show at least 3 columns, even if it means exceeding terminal width slightly
```

**Example (120-char terminal):**
```
Terminal width: 120 chars
Reserve for borders: 10 chars
Available: 110 chars

Columns and widths:
- employee_id: 15 chars
- first_name: 15 chars
- last_name: 15 chars
- email: 25 chars
- hire_date: 15 chars
- salary: 12 chars
- department_id: 18 chars
Total for first 5: 103 chars → Fits!
Adding 6th (department_id, 18): 121 chars → Exceeds, stop at 5 columns

Result: Show columns 1-5 of 23 in window
```

### Column Selection Priority

**First window always shows:**
1. Primary key columns (if identifiable from name patterns: `*_id`, `id`)
2. Leftmost columns in SELECT list order
3. Continue until window full

**User-driven navigation:**
- Left/Right arrows: Shift window by 1 column
- Ctrl-Left/Right: Shift window to start/end
- Home: Jump to first column group
- End: Jump to last column group

### Column Window Indicator

**Status Bar (Bottom of Screen):**
```
┌─────────────┬──────────────┬──────────────┬──────────────┐
│ employee_id │ first_name   │ last_name    │ email        │
├─────────────┼──────────────┼──────────────┼──────────────┤
│ 1           │ Alice        │ Anderson     │ alice@co.com │
│ 2           │ Bob          │ Brown        │ bob@co.com   │
│ 3           │ Charlie      │ Chen         │ charlie@...  │
└─────────────┴──────────────┴──────────────┴──────────────┘

Columns 1-4 of 23 | Rows 1-3 of 150 | ← → : scroll columns | ↑ ↓ : scroll rows | q: exit pager
```

**Key Elements:**
- `Columns 1-4 of 23`: Shows current column window and total columns
- `Rows 1-3 of 150`: Shows current row position and total rows
- Navigation hints: Brief, always visible
- Exit hint: Clear `q: exit pager` (not "q: quit" which sounds like quit program)

### Column Transition Behavior

**Smooth Scrolling:**
- Left arrow: Shift window left by 1 column (if possible)
- Right arrow: Shift window right by 1 column (if possible)
- Window size adjusts dynamically to fit new columns

**Example:**
```
Initial view: Columns 1-5 of 10
[employee_id] [first_name] [last_name] [email] [hire_date]

Press Right Arrow:
[first_name] [last_name] [email] [hire_date] [salary]
Status: Columns 2-6 of 10

Press Right Arrow again:
[last_name] [email] [hire_date] [salary] [department_id]
Status: Columns 3-7 of 10
```

**Edge Cases:**
- At leftmost: Left arrow does nothing, show brief indicator "Already at first column"
- At rightmost: Right arrow does nothing, show brief indicator "Already at last column"
- Single column wider than terminal: Show it anyway, allow horizontal scrolling within cell

---

## Design Solution: Layer 2 - Cell Truncation

### Objective
Prevent individual cell values from breaking table layout by truncating long content.

### Truncation Rules

**Maximum Cell Display Length:**
- **Standard cells:** 100 characters maximum (user-suggested value)
- **Numeric cells:** No truncation (numbers are naturally bounded)
- **Date/Time cells:** No truncation (fixed format)
- **Text cells (VARCHAR, CHAR):** Apply 100-char limit

**Truncation Indicator:**
- Append `…` (single ellipsis) to truncated values
- Use Unicode ellipsis (U+2026) for cleaner look
- Example: `This is a very long description that goes on...` (actual: 99 chars + ellipsis)

**Calculation:**
```
If cell_value.length > 100:
    displayed_value = cell_value[0..99] + "…"
Else:
    displayed_value = cell_value
```

### Column Width After Truncation

**Per-Column Width Calculation:**
```
1. Examine all values in column (up to first 100 rows for performance)
2. Find max display length after truncation:
   - max_length = max(header_length, max(value_lengths))
3. Cap at maximum column width:
   - column_width = min(max_length + 2, 40)  // +2 for padding, max 40
4. Use consistent width for entire column
```

**Example:**
```
Column: description (VARCHAR(2000))
Values:
  Row 1: "Short text" (10 chars)
  Row 2: "This is a much longer description..." (500 chars → truncate to 100)
  Row 3: "Medium length description here" (30 chars)

Column width = min(max(11 "description", 100) + 2, 40) = 40 chars
All cells in this column: 40 chars wide
```

### Full Value Display: Future Enhancement

**Deferred to Future Sprint:**
Currently, users cannot view full cell values from pager. Workarounds:
1. Copy full value from SQL: `SELECT description FROM table WHERE id = 123;`
2. Use `/export` to save full results to file for inspection

**Future Design (Sprint 9+):**
- Press 'v' (view) on cell to open expanded view modal
- Show full cell content in scrollable popup
- Press Escape to return to table view

**Rationale for Deferral:**
- Sprint 8 focuses on making paging safe and usable
- Full value viewer adds complexity (modal UI, cursor positioning)
- Workarounds are acceptable for now

---

## Design Solution: Layer 3 - Vertical Row Paging

### Objective
Navigate long result sets (100+ rows) smoothly with clear position indicators.

### Page Size: Dynamic

**Calculation:**
```
page_size = terminal_height - 5
  - 1 row for header
  - 2 rows for top/bottom borders
  - 1 row for status bar
  - 1 row for breathing room
```

**Example (24-line terminal):**
```
Terminal height: 24 lines
Page size: 19 rows of data
Status bar: 1 line
Total: 20 lines used, 4 lines margin
```

### Vertical Navigation Keys

**Single Row Movement:**
- `j` / Down Arrow: Next row
- `k` / Up Arrow: Previous row

**Page Movement:**
- `Space` / Page Down: Next page (jump by page_size rows)
- `b` / Page Up: Previous page (jump by page_size rows)

**Jump Navigation:**
- `g` / Home: Jump to first row
- `G` / End: Jump to last row
- `50G`: Jump to row 50 (vi-style numeric prefix)

**Search (Future):**
- `/pattern`: Search forward for pattern in visible columns
- `n`: Next match
- `N`: Previous match

### Row Position Indicator

**In Status Bar:**
```
Rows 1-19 of 1,234 (1%) | Space: next page | b: prev | g/G: first/last | q: exit
```

**Elements:**
- `Rows 1-19 of 1,234`: Current visible range and total
- `(1%)`: Percentage through result set
- Navigation hints: Concise, show most common actions
- Exit hint: Always present

### Edge Behavior

**At Top:**
- `k` or Up Arrow: No movement, no error
- Brief flash indicator: "Already at first row" (optional)

**At Bottom:**
- `j` or Down Arrow: No movement, no error
- Brief flash indicator: "Already at last row" (optional)

**Empty Results:**
```
(No results)

Press q to exit pager
```

---

## Design Solution: Safe Pager Exit

### Critical Requirement: Never Exit Program from Pager

**Problem:** Current implementation treats 'q' as program exit signal.

**Solution:** Pager must be a controlled component within REPL, not a separate blocking mode.

### Exit Keys

**Primary Exit:**
- `q` (lowercase): Exit pager, return to `tq>` prompt
- Clear status message: "q: exit pager" (not "q: quit")

**Alternative Exit:**
- `Escape`: Also exits pager, return to prompt
- Ctrl-C: Cancel pager, return to prompt (same as 'q')

**Exit Program (From REPL Prompt Only):**
- `Ctrl-D`: Exit tq program (when at empty prompt)
- `/quit`: Exit tq program metacommand
- These are NEVER available from pager mode

### Exit Flow

**User Experience:**
```
tq> SELECT * FROM employees;
[Query executes, enters pager mode]

┌─────────────┬──────────────┬──────────────┐
│ employee_id │ first_name   │ last_name    │
├─────────────┼──────────────┼──────────────┤
│ 1           │ Alice        │ Anderson     │
│ 2           │ Bob          │ Brown        │
└─────────────┴──────────────┴──────────────┘

Rows 1-20 of 500 | q: exit pager

[User presses 'q']

tq> _
[Back at REPL prompt, session preserved]
```

**Technical Implementation:**
- Pager runs in local event loop, not `minus::page_all()` blocking mode
- 'q' key breaks pager event loop, returns control to REPL
- REPL state (connection, history, settings) fully preserved
- No process exit signals sent

### Mode Indicators

**Pager Mode Visual Cues:**
1. **Status bar present:** Only shown in pager, not in normal REPL output
2. **Clear "exit pager" text:** Distinguishes from "exit program"
3. **No prompt visible:** When in pager, `tq>` prompt is hidden

**REPL Mode Visual Cues:**
1. **Prompt visible:** `tq>` or `tq[dbname]>` shown
2. **No status bar:** Regular terminal output
3. **Cursor ready:** Blinking cursor for input

---

## Combined UX: Complete Pager Interface

### Full Status Bar Design

**Layout (Bottom of Screen):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%) | Navigation: ←→ ↑↓ Space b  │
│ g/G: first/last | /: search | q/Esc: exit pager                           │
└────────────────────────────────────────────────────────────────────────────┘
```

**Elements:**
- **Column position:** `Columns 1-5 of 23`
- **Row position:** `Rows 1-20 of 1,234 (2%)`
- **Navigation hints:** Most common keys
- **Exit hint:** Clear "exit pager" wording
- **Two-line status bar:** Enough space for all information without cramping

### Paging Example: Wide Table

**Scenario:** Table with 23 columns, 1,234 rows, terminal width 100 chars

**Initial View:**
```
┌──────────┬────────────┬───────────┬──────────────┬────────────┐
│ emp_id   │ first_name │ last_name │ email        │ hire_date  │
├──────────┼────────────┼───────────┼──────────────┼────────────┤
│ 1        │ Alice      │ Anderson  │ alice@co.com │ 2020-01-15 │
│ 2        │ Bob        │ Brown     │ bob@co.com   │ 2020-03-22 │
│ 3        │ Charlie    │ Chen      │ charlie@c... │ 2020-07-01 │
│ 4        │ Diana      │ Davis     │ diana@co.com │ 2021-01-10 │
│ 5        │ Edward     │ Evans     │ edward@co... │ 2021-04-05 │
│ ...      │ ...        │ ...       │ ...          │ ...        │
│ 20       │ Tina       │ Turner    │ tina@co.com  │ 2024-12-15 │
└──────────┴────────────┴───────────┴──────────────┴────────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%) | →: more columns             │
│ Space: next page | ↑↓: scroll rows | q: exit pager                         │
└────────────────────────────────────────────────────────────────────────────┘
```

**User Presses Right Arrow:**
```
┌────────────┬──────────┬─────────────┬────────────┬──────────┐
│ last_name  │ email    │ hire_date   │ salary     │ dept_id  │
├────────────┼──────────┼─────────────┼────────────┼──────────┤
│ Anderson   │ alice@.. │ 2020-01-15  │ 75000.00   │ 101      │
│ Brown      │ bob@co.. │ 2020-03-22  │ 68000.00   │ 102      │
│ Chen       │ charlie. │ 2020-07-01  │ 82000.00   │ 101      │
│ Davis      │ diana@.. │ 2021-01-10  │ 71000.00   │ 103      │
│ Evans      │ edward@. │ 2021-04-05  │ 79000.00   │ 102      │
│ ...        │ ...      │ ...         │ ...        │ ...      │
│ Turner     │ tina@co. │ 2024-12-15  │ 65000.00   │ 104      │
└────────────┴──────────┴─────────────┴────────────┴──────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 3-7 of 23 | Rows 1-20 of 1,234 (2%) | ←: prev | →: next           │
│ Space: next page | ↑↓: scroll rows | q: exit pager                         │
└────────────────────────────────────────────────────────────────────────────┘
```

**User Presses Space (Next Page):**
```
┌────────────┬──────────┬─────────────┬────────────┬──────────┐
│ last_name  │ email    │ hire_date   │ salary     │ dept_id  │
├────────────┼──────────┼─────────────┼────────────┼──────────┤
│ Garcia     │ uma@co.. │ 2020-02-12  │ 73000.00   │ 105      │
│ Harris     │ victor@. │ 2020-04-18  │ 76000.00   │ 101      │
│ Irwin      │ wendy@.. │ 2020-08-22  │ 69000.00   │ 102      │
│ ...        │ ...      │ ...         │ ...        │ ...      │
│ Zane       │ xander@. │ 2023-11-30  │ 71000.00   │ 103      │
└────────────┴──────────┴─────────────┴────────────┴──────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 3-7 of 23 | Rows 21-40 of 1,234 (3%) | ←→: columns                │
│ Space: next page | b: prev page | q: exit pager                            │
└────────────────────────────────────────────────────────────────────────────┘
```

**User Presses 'q':**
```
tq> _
[Returns to REPL prompt, session preserved]
```

### Paging Example: Long Cell Values

**Scenario:** Table with long VARCHAR values

**Query:**
```sql
SELECT id, title, description FROM articles LIMIT 5;
```

**Paged Output:**
```
┌─────┬─────────────────────┬──────────────────────────────────────────┐
│ id  │ title               │ description                              │
├─────┼─────────────────────┼──────────────────────────────────────────┤
│ 1   │ Getting Started     │ This article explains how to get star... │
│ 2   │ Advanced Features   │ Learn about advanced features includi... │
│ 3   │ Troubleshooting     │ Common issues and their solutions are... │
│ 4   │ Performance Tuning  │ Optimize your queries for maximum per... │
│ 5   │ Best Practices      │ Follow these best practices to ensure... │
└─────┴─────────────────────┴──────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-3 of 3 | Rows 1-5 of 5 (100%) | q: exit pager                   │
└────────────────────────────────────────────────────────────────────────────┘
```

**Note:** Each description truncated to 100 chars with `...` indicator. Full values require separate query or `/export`.

---

## Implementation Guidance

### Architecture Requirements

**Pager Component Responsibilities:**
1. **Input:** Receive complete result set from query execution
2. **Processing:**
   - Calculate column windows based on terminal width
   - Apply cell truncation rules
   - Determine page size based on terminal height
3. **Rendering:** Draw table with borders, headers, and status bar
4. **Navigation:** Handle keyboard input for scrolling
5. **Exit:** Return control to REPL on 'q' or Escape

**Integration with REPL:**
```rust
// Pseudo-code
fn execute_query(sql: &str) -> Result<ResultSet> {
    let results = database.query(sql)?;

    if results.row_count() > PAGING_THRESHOLD {
        // Enter pager mode
        pager::display_with_navigation(results)?;
        // When pager exits, returns here
    } else {
        // Small result, print directly
        print_table(results);
    }

    // Always returns to REPL prompt after this function
    Ok(())
}
```

**Key Insight:** Pager is a function call, not a mode change. It runs, displays results, handles navigation, then returns. REPL never loses control.

### Technical Implementation Details

**Column Windowing Implementation:**
```rust
struct ColumnWindow {
    start_index: usize,      // First visible column (0-indexed)
    visible_columns: Vec<usize>,  // Indices of visible columns
    total_columns: usize,    // Total columns in result set
}

impl ColumnWindow {
    fn calculate(
        columns: &[Column],
        terminal_width: usize
    ) -> Self {
        let mut visible = Vec::new();
        let mut total_width = 10;  // Borders + margins

        for (i, col) in columns.iter().enumerate() {
            let col_width = calculate_column_width(col);
            if total_width + col_width <= terminal_width {
                visible.push(i);
                total_width += col_width;
            } else {
                break;  // Window full
            }

            // Ensure at least 3 columns visible
            if i < 2 {
                visible.push(i);
            }
        }

        ColumnWindow {
            start_index: 0,
            visible_columns: visible,
            total_columns: columns.len(),
        }
    }

    fn shift_right(&mut self) -> bool {
        // Shift window right by 1 column if possible
        if self.start_index + self.visible_columns.len() < self.total_columns {
            self.start_index += 1;
            self.recalculate_visible();
            true
        } else {
            false  // Already at rightmost position
        }
    }

    fn shift_left(&mut self) -> bool {
        // Shift window left by 1 column if possible
        if self.start_index > 0 {
            self.start_index -= 1;
            self.recalculate_visible();
            true
        } else {
            false  // Already at leftmost position
        }
    }
}
```

**Cell Truncation Implementation:**
```rust
fn truncate_cell_value(value: &str, max_length: usize) -> String {
    if value.len() > max_length {
        format!("{}…", &value[0..max_length - 1])
    } else {
        value.to_string()
    }
}

fn calculate_column_width(column: &Column, rows: &[Row]) -> usize {
    let header_len = column.name.len();
    let max_value_len = rows
        .iter()
        .take(100)  // Sample first 100 rows for performance
        .map(|row| {
            let value = row.get_string(column.index);
            truncate_cell_value(value, 100).len()
        })
        .max()
        .unwrap_or(0);

    let width = max(header_len, max_value_len) + 2;  // +2 for padding
    min(width, 40)  // Cap at 40 chars
}
```

**Pager Event Loop:**
```rust
fn pager_event_loop(
    results: &ResultSet,
    terminal: &mut Terminal
) -> Result<()> {
    let mut column_window = ColumnWindow::calculate(&results.columns, terminal.width());
    let mut row_offset = 0;
    let page_size = terminal.height() - 5;

    loop {
        // Render current view
        render_table(
            results,
            &column_window,
            row_offset,
            page_size,
            terminal
        )?;

        render_status_bar(
            &column_window,
            row_offset,
            page_size,
            results.row_count(),
            terminal
        )?;

        // Handle input
        match read_key()? {
            Key::Char('q') | Key::Esc => break,  // Exit pager
            Key::Char('j') | Key::Down => {
                row_offset = min(row_offset + 1, results.row_count() - 1);
            }
            Key::Char('k') | Key::Up => {
                row_offset = max(row_offset.saturating_sub(1), 0);
            }
            Key::Char(' ') | Key::PageDown => {
                row_offset = min(row_offset + page_size, results.row_count() - 1);
            }
            Key::Char('b') | Key::PageUp => {
                row_offset = row_offset.saturating_sub(page_size);
            }
            Key::Right => {
                if !column_window.shift_right() {
                    show_message("Already at last column");
                }
            }
            Key::Left => {
                if !column_window.shift_left() {
                    show_message("Already at first column");
                }
            }
            Key::Char('g') | Key::Home => {
                row_offset = 0;
            }
            Key::Char('G') | Key::End => {
                row_offset = results.row_count().saturating_sub(page_size);
            }
            _ => {}  // Ignore other keys
        }
    }

    // Pager exited, return control to REPL
    Ok(())
}
```

### Library Recommendations

**Replace `minus` with Custom Pager:**

The current `minus` library does not provide the control needed for:
1. Safe exit (q returns to REPL, not exit program)
2. Custom column windowing
3. Custom cell truncation
4. Teradata-specific features

**Recommended Approach:**
- Use `crossterm` for terminal control (cursor, colors, input)
- Build custom pager with exact behavior specified in this design
- Full control over all UX aspects

**Alternative (If Keeping `minus`):**
- Use `minus::Pager::new()` instead of `minus::page_all()`
- Run pager in controlled mode, not blocking mode
- Override keybindings to prevent program exit
- May still have limitations for column windowing

### Performance Considerations

**Large Result Sets:**
- Don't load all rows into memory at once
- Use streaming or chunked loading (fetch 1000 rows at a time)
- Render only visible rows (current page)
- Background loading indicator if next page not yet fetched

**Wide Tables:**
- Column width calculation: Sample first 100 rows only (don't scan all rows)
- Cache column widths per window for reuse
- Re-render only changed portions when scrolling

**Terminal Resizing:**
- Listen for SIGWINCH signal (terminal resize)
- Recalculate column windows and page size
- Re-render current view with new dimensions

---

## Test Scenarios

### Critical Test Cases (Must Pass)

**Test 1: 'q' Returns to REPL**
```
Steps:
1. Start tq REPL
2. Execute: SELECT * FROM large_table;
3. Pager opens with results
4. Press 'q'
Expected: Returns to tq> prompt, session preserved
Actual: [To be tested]
```

**Test 2: Wide Table Readable**
```
Steps:
1. Query table with 25 columns
2. Verify only 4-6 columns displayed (depending on terminal width)
3. Verify columns are readable (not squeezed)
4. Press Right Arrow
5. Verify new columns appear, old columns scroll off
Expected: All visible columns readable, smooth navigation
Actual: [To be tested]
```

**Test 3: Long Values Truncated**
```
Steps:
1. Query table with VARCHAR(2000) column containing 500-char values
2. Verify values truncated at 100 chars with "…"
3. Verify table layout remains stable
Expected: Long values truncated, table readable
Actual: [To be tested]
```

### Edge Case Tests

**Test 4: Single Wide Column**
```
Scenario: Table with 1 column that is 150 chars wide
Expected: Show full column even if exceeds terminal width
```

**Test 5: Empty Result Set**
```
Scenario: Query returns 0 rows
Expected: Show "(No results)" message, q exits pager
```

**Test 6: Exactly One Page**
```
Scenario: Result set exactly fills one screen (no scrolling needed)
Expected: Pager still active, status bar shows "Rows 1-20 of 20 (100%)"
```

**Test 7: Terminal Resize During Paging**
```
Scenario: User resizes terminal while pager active
Expected: Pager recalculates windows and re-renders smoothly
```

**Test 8: Navigation at Boundaries**
```
Scenario: Press 'k' at first row, press 'j' at last row
Expected: No error, no crash, brief indicator (optional)
```

### UX Validation Tests

**Test 9: Status Bar Clarity**
```
Criteria: User can quickly understand:
- Current column range
- Current row range
- Total rows
- How to navigate
- How to exit
Validation: Show status bar to user, ask for feedback
```

**Test 10: Column Transition Smoothness**
```
Criteria: Shifting columns left/right feels natural
Validation: Navigate through 20 columns, verify smooth experience
```

**Test 11: Exit Confusion Test**
```
Scenario: New user presses 'q' in pager
Expected: User returns to tq> prompt, not confused
Validation: Ask user if behavior was expected
```

### Performance Tests

**Test 12: Large Result Set (10,000 rows)**
```
Scenario: Query returns 10,000 rows
Expected:
- Pager opens in <1s
- Navigation responsive (<100ms per key)
- Memory usage reasonable (<100MB)
```

**Test 13: Very Wide Table (50 columns)**
```
Scenario: Query returns 50 columns
Expected:
- Column window calculated quickly (<200ms)
- Smooth horizontal scrolling
- Status bar shows accurate column counts
```

---

## Summary

This design provides a comprehensive solution to all three critical paging issues:

1. **Safe Exit (Issue 3.1):** 'q' exits pager and returns to REPL, never exits program
2. **Readable Wide Tables (Issue 3.2):** Column windowing shows 4-6 readable columns, left/right navigation for rest
3. **Readable Long Values (Issue 3.3):** Cell truncation at 100 chars prevents layout breaks

The design follows established CLI conventions (`less`, `psql`) while addressing Teradata's unique wide-table and long-value challenges.

**Next Steps:**
1. rust-teradata-architect: Implement this design
2. quality-validator: Execute all test scenarios
3. User: Validate UX meets expectations

---

**Document Status:** Final Design - Approved for Implementation
