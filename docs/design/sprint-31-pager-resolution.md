# Sprint 31: Pager Resolution Design

**Document Owner:** rust-teradata-architect
**Sprint:** 31 (Maintenance Sprint - Framework Crisis Recovery)
**Status:** Design Phase

---

## Executive Summary

The pager feature has been broken across Sprint 29 and Sprint 30 despite significant development effort (~$81 invested). The feature is currently disabled by default (`pager_enabled: false` in `state.rs`). This document provides detailed technical designs for two resolution approaches:

- **Option A: Fix Pager** - Debug and fix with manual terminal validation (4-hour time-box)
- **Option B: Remove Pager** - Clean removal of all pager code

**Recommendation:** Given the complexity of terminal rendering issues and the two-sprint history of failed fixes, Option B (Remove) may be the pragmatic choice if Option A does not succeed within the 4-hour time-box.

---

## Problem Analysis

### Root Cause Hypothesis

After analyzing the code in `pager.rs` (973 lines), the likely root cause is a **mismatch between `visible_column_count()` calculation and actual rendered width**.

The `visible_column_count()` method (lines 291-327) performs width calculations, but there are several potential issues:

1. **Indicator width accounting inconsistency**: The method reserves space for indicators (`INDICATOR_WIDTH + 3` = 13 chars) only when `hidden_left > 0` or `hidden_right_possible` is true. However, this creates a circular dependency - `hidden_columns_right()` calls `visible_column_count()`, which checks for hidden columns.

2. **Border rendering mismatch**: The `render_border()` method (lines 376-417) builds borders character by character, but the width calculation in `visible_column_count()` uses `col.display_width + 3` per column. These must match exactly.

3. **Header/row rendering inconsistency**: `render_header()` and `render_row()` use `format!(" {:^width$} ", ...)` which may produce different widths than expected when Unicode characters are involved.

### Evidence from Commit History

```
e111c7b: "CRITICAL FIX: Limit column widths to 40 chars"
bf51ea2: "CRITICAL FIX: Truncate cell values to prevent table misalignment"
a105c39: "Add file-based debug logging"
```

These commits suggest the issue is related to column width calculation and cell truncation not properly preventing overflow.

---

## Option A: Fix Pager (4 Hours Max)

### Phase 1: Add Testable Render Method (1 Hour)

**Goal:** Add `Pager::render_to_buffer()` to capture rendered output without terminal interaction.

**Implementation:**

```rust
// In src/commands/repl/pager.rs

impl Pager {
    /// Render the current view to a string buffer for testing/debugging.
    ///
    /// This method renders the table to a buffer instead of stdout,
    /// allowing validation of rendered output width without terminal interaction.
    pub fn render_to_buffer(&self) -> String {
        let mut buffer = Vec::new();

        let visible_cols = self.visible_column_count();
        let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
        let end_row = (self.row_offset + self.page_size).min(self.data.row_count);

        // Render top border
        self.render_border_to_buffer(&mut buffer, BorderType::Top);

        // Render header row
        self.render_header_to_buffer(&mut buffer, self.col_offset, end_col);

        // Render header separator
        self.render_border_to_buffer(&mut buffer, BorderType::Middle);

        // Render data rows
        for row_idx in self.row_offset..end_row {
            self.render_row_to_buffer(&mut buffer, row_idx, self.col_offset, end_col);
        }

        // Render bottom border
        self.render_border_to_buffer(&mut buffer, BorderType::Bottom);

        String::from_utf8_lossy(&buffer).to_string()
    }

    fn render_border_to_buffer(&self, buffer: &mut Vec<u8>, border_type: BorderType) {
        // Same logic as render_border but writes to buffer instead of stdout
        // (Duplicate internal logic without ANSI escapes)
    }

    fn render_header_to_buffer(&self, buffer: &mut Vec<u8>, start_col: usize, end_col: usize) {
        // Same logic as render_header but writes to buffer without ANSI escapes
    }

    fn render_row_to_buffer(&self, buffer: &mut Vec<u8>, row_idx: usize, start_col: usize, end_col: usize) {
        // Same logic as render_row but writes to buffer without ANSI escapes
    }
}
```

**Files Changed:**
- `src/commands/repl/pager.rs`

### Phase 2: Debug Width Mismatch (1.5 Hours)

**Goal:** Identify and fix the specific mismatch between calculated and actual width.

**Debug Strategy:**

1. **Add width assertion in render methods:**

```rust
fn render_row_to_buffer(&self, buffer: &mut Vec<u8>, row_idx: usize, start_col: usize, end_col: usize) {
    let mut line = String::new();
    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();

    line.push('|');

    // Left indicator cell
    if hidden_left > 0 {
        let indicator = "    <--   ";
        line.push_str(&format!(" {} |", indicator));
    }

    for (vis_idx, col) in self.data.columns[start_col..end_col].iter().enumerate() {
        let col_idx = start_col + vis_idx;
        let value = self.data.get_cell(row_idx, col_idx);

        // Format with alignment
        let padded = match col.alignment {
            Alignment::Right => format!(" {:>width$} ", value, width = col.display_width),
            Alignment::Center => format!(" {:^width$} ", value, width = col.display_width),
            Alignment::Left => format!(" {:width$} ", value, width = col.display_width),
        };
        line.push_str(&padded);
        line.push('|');
    }

    // Right indicator cell
    if hidden_right > 0 {
        let indicator = "   -->    ";
        line.push_str(&format!(" {} |", indicator));
    }

    // WIDTH ASSERTION
    let actual_width = unicode_width::UnicodeWidthStr::width(line.as_str());
    let expected_width = self.calculate_expected_line_width();

    log::debug!(
        "Row {}: actual_width={}, expected_width={}, term_width={}",
        row_idx, actual_width, expected_width, self.term_width
    );

    if actual_width > self.term_width {
        log::error!(
            "WIDTH OVERFLOW: Row {} is {} chars, terminal is {} chars",
            row_idx, actual_width, self.term_width
        );
    }

    line.push('\n');
    buffer.extend(line.as_bytes());
}

fn calculate_expected_line_width(&self) -> usize {
    let visible_cols = self.visible_column_count();
    let end_col = (self.col_offset + visible_cols).min(self.data.columns.len());
    let hidden_left = self.hidden_columns_left();
    let hidden_right = self.hidden_columns_right();

    let mut width = 1; // Leading border

    if hidden_left > 0 {
        width += INDICATOR_WIDTH + 3; // " " + indicator + " " + "|"
    }

    for col in &self.data.columns[self.col_offset..end_col] {
        width += col.display_width + 3; // " " + content + " " + "|"
    }

    if hidden_right > 0 {
        width += INDICATOR_WIDTH + 3;
    }

    width
}
```

2. **Check for off-by-one in visible_column_count():**

The current implementation has a subtle issue - it checks `total_width + col_width > available_width` but this means it allows the column that exactly reaches the limit, which may cause overflow if the right indicator is then added.

**Potential Fix:**

```rust
fn visible_column_count(&self) -> usize {
    let hidden_left = self.hidden_columns_left();

    // Always reserve space for potential right indicator
    // This prevents adding a column that exactly fits, then adding an indicator that overflows
    let right_indicator_width = INDICATOR_WIDTH + 3;
    let left_indicator_width = if hidden_left > 0 { INDICATOR_WIDTH + 3 } else { 0 };

    let mut total_width = 1 + left_indicator_width; // Leading border + left indicator
    let mut count = 0;

    for (i, col) in self.data.columns.iter().skip(self.col_offset).enumerate() {
        let col_width = col.display_width + 3;

        // Check if adding this column would leave room for right indicator
        // (if there would be more columns after this one)
        let more_cols_after = self.data.columns.len() > self.col_offset + i + 1;
        let required_right_space = if more_cols_after { right_indicator_width } else { 0 };

        if total_width + col_width + required_right_space > self.term_width && count > 0 {
            break;
        }

        total_width += col_width;
        count += 1;
    }

    count.max(1)
}
```

### Phase 3: Manual Terminal Validation (1 Hour)

**Goal:** Validate fix works at multiple terminal widths with real data.

**Validation Strategy:**

1. **Create test script:**

```bash
#!/bin/bash
# tests/manual/pager_validation.sh

# Test widths: 80, 117, 120, 160
for WIDTH in 80 117 120 160; do
    echo "=== Testing at width $WIDTH ==="

    # Resize terminal (macOS)
    printf '\e[8;40;'$WIDTH't'
    sleep 0.5

    # Run test query with pager enabled
    echo "SELECT * FROM test_table SAMPLE 50;" | \
        script -q /tmp/pager_test_${WIDTH}.txt \
        cargo run -- repl --logon "$TQ_LOGON"

    # Check for overflow
    MAX_LINE=$(awk '{ if (length > max) max = length } END { print max }' /tmp/pager_test_${WIDTH}.txt)
    echo "Max line width at terminal $WIDTH: $MAX_LINE"

    if [ "$MAX_LINE" -gt "$WIDTH" ]; then
        echo "FAIL: Overflow detected at width $WIDTH"
    else
        echo "PASS: No overflow at width $WIDTH"
    fi
done
```

2. **Manual validation checklist:**

| Terminal Width | Expected Behavior | Pass/Fail |
|---------------|-------------------|-----------|
| 80 chars | Table fits, indicators show hidden columns | |
| 117 chars | Table fits (user-reported problematic width) | |
| 120 chars | Table fits | |
| 160 chars | More columns visible | |

3. **Evidence capture with `script` command:**

```bash
# Capture actual terminal output
script -q evidence_117.txt
# ... run pager test ...
exit

# Analyze
wc -L evidence_117.txt  # Max line width
```

### Phase 4: Connect Track 3 Utilities (30 Minutes)

**Goal:** Use visual_validator.rs to validate rendered output.

**Implementation:**

```rust
// tests/pager_width_validation.rs

#[cfg(test)]
mod tests {
    use crate::commands::repl::pager::{Pager, PagerConfig};
    use crate::db::{ColumnMetadata, QueryResult, TeradataType, Value};
    use std::time::Duration;

    // Import Track 3 utilities
    mod tools {
        include!("../tests/tools/visual_validator.rs");
        include!("../tests/tools/terminal_simulator.rs");
    }

    fn create_wide_result(num_cols: usize, num_rows: usize) -> QueryResult {
        let columns: Vec<ColumnMetadata> = (0..num_cols)
            .map(|i| ColumnMetadata::new(
                format!("column_name_{}", i),
                TeradataType::Varchar,
                true
            ))
            .collect();

        let rows: Vec<Vec<Value>> = (0..num_rows)
            .map(|r| {
                (0..num_cols)
                    .map(|c| Value::String(format!("value_row{}_col{}", r, c)))
                    .collect()
            })
            .collect();

        QueryResult::new(columns, rows, Duration::from_millis(100))
    }

    #[test]
    fn test_pager_render_width_80() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);

        // Override terminal width for testing
        pager.term_width = 80;

        let rendered = pager.render_to_buffer();

        // Use Track 3 validator
        tools::assert_no_overflow(&rendered, 80);
    }

    #[test]
    fn test_pager_render_width_117() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);

        pager.term_width = 117;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 117);
    }

    #[test]
    fn test_pager_render_width_120() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);

        pager.term_width = 120;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 120);
    }

    #[test]
    fn test_pager_render_width_160() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);

        pager.term_width = 160;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 160);
    }
}
```

### Option A Success Criteria

1. All dimensional tests pass at 80, 117, 120, 160 char widths
2. Manual terminal validation confirms no garbled output
3. `cargo test --lib` passes
4. `cargo clippy` passes
5. Pager enabled by default (`pager_enabled: true`)

---

## Option B: Remove Pager (Clean Removal)

If Option A does not succeed within the 4-hour time-box, Option B provides a clean removal path.

### Phase 1: Stub Pager Module (30 Minutes)

**Goal:** Replace pager.rs with a stub that documents removal.

**Implementation:**

```rust
// src/commands/repl/pager.rs (REPLACEMENT - 50 lines instead of 973)

//! Pager Module - Feature Removed
//!
//! Sprint 31: Pager feature removed after two sprints of unsuccessful fixes.
//! The pager was disabled by default since Sprint 30 due to terminal width
//! calculation issues causing garbled output.
//!
//! ## Rationale
//!
//! - Sprint 29: Initial implementation had fundamental architecture flaw
//!   (pre-formatted strings exceeded terminal width)
//! - Sprint 30: Architectural refactor did not resolve rendering issues
//! - Sprint 31: Time-boxed fix attempt unsuccessful, clean removal preferred
//!   over indefinite disabled code
//!
//! ## Alternative Approaches for Large Results
//!
//! - Use `/export` to save results to file
//! - Use `SAMPLE N` or `TOP N` to limit result size
//! - Pipe output to external pager: `tq repl | less -S`

use crate::db::QueryResult;

/// Pager configuration (stub - paging not supported)
#[derive(Debug, Clone, Default)]
pub struct PagerConfig {
    _private: (),
}

impl PagerConfig {
    /// Create a disabled pager config
    pub fn disabled() -> Self {
        Self { _private: () }
    }
}

/// Check if content should be paged (always returns false - paging removed)
pub fn should_page(_result: &QueryResult, _config: &PagerConfig) -> bool {
    false
}

/// Display with pager (stub - always returns Ok(false))
///
/// Pager feature has been removed. This function exists for API compatibility
/// and always returns `Ok(false)` to indicate paging was not used.
pub fn display_with_pager(
    _result: &QueryResult,
    _config: &PagerConfig,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(false)
}
```

**Files Changed:**
- `src/commands/repl/pager.rs` (replace 973 lines with 50 lines)

### Phase 2: Update Executor (15 Minutes)

**Goal:** Simplify executor.rs to remove pager integration.

**Changes to `src/commands/repl/executor.rs`:**

```rust
// Remove or simplify pager imports and integration

// BEFORE (lines 18-19):
use super::pager::{display_with_pager, PagerConfig};

// AFTER:
// Pager import removed - feature not supported

// BEFORE (lines 176-219): Pager integration block
// AFTER: Remove entire pager block, always use direct output

/// Execute SQL with state management
pub fn execute_sql_with_state<W: Write>(
    client: &DatabaseClient,
    state: &mut ReplState,
    sql: &str,
    writer: &mut W,
    default_limit: usize,
) -> Result<usize> {
    // ... existing query execution code ...

    // SIMPLIFIED: Always format and write output directly
    // Pager feature removed in Sprint 31
    let format_options = FormatOptions::default()
        .with_header(true)
        .with_color(state.are_colors_enabled());

    write_output_with_timing(
        &result_clone,
        writer,
        OutputFormat::Table,
        &format_options,
        true,
    )?;

    // ... existing limit message code ...

    Ok(row_count)
}
```

### Phase 3: Update State (15 Minutes)

**Goal:** Remove pager_enabled field from ReplState.

**Changes to `src/commands/repl/state.rs`:**

```rust
// REMOVE these fields and methods:

// Field (line 41):
// pager_enabled: bool,

// In new() (line 66):
// pager_enabled: false,

// Methods (lines 182-189):
// pub fn set_pager(&mut self, enabled: bool)
// pub fn is_pager_enabled(&self) -> bool

// UPDATE documentation to note pager removal
```

**Note:** Removing `pager_enabled` will require updating any code that references it. Search for usages:

```bash
grep -r "pager_enabled\|is_pager_enabled\|set_pager" src/
```

Likely locations:
- `src/commands/repl/executor.rs` (already handled)
- `src/commands/repl/metacommands.rs` (handle `/pager` command)

### Phase 4: Update Metacommands (15 Minutes)

**Goal:** Remove or update `/pager` metacommand.

**Changes to metacommands handling:**

```rust
// Option 1: Remove /pager command entirely
// Option 2: Keep /pager but make it print deprecation message

// Preferred: Option 2 (less breaking)
fn handle_pager_command(args: &str, _state: &mut ReplState, writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "Pager feature is not currently supported.")?;
    writeln!(writer, "Alternative approaches:")?;
    writeln!(writer, "  - Use /export to save results to file")?;
    writeln!(writer, "  - Use SAMPLE N or TOP N to limit results")?;
    writeln!(writer, "  - Pipe output: tq repl | less -S")?;
    Ok(())
}
```

### Phase 5: Update Documentation (30 Minutes)

**Goal:** Update documentation to reflect pager removal.

**Files to Update:**

1. **`docs/design/repl.md`** - Remove/update pager sections (major update)
2. **`docs/specifications/repl.md`** - Update to note pager not supported
3. **`docs/roadmap/status.md`** - Update pager feature status

**Example update for `docs/specifications/repl.md`:**

```markdown
### Result Display

Results are displayed in formatted tables with:
- Colored headers (when colors enabled)
- Aligned columns
- Row counts and timing information

**Note:** Built-in result paging is not currently supported. For large results:
- Use `SAMPLE N` or `TOP N` in queries to limit rows
- Use `/export` to save results to file
- Pipe output to external pager: `tq repl | less -S`
```

### Phase 6: Track 3 Utilities Decision (15 Minutes)

**Goal:** Decide whether to keep or remove visual_validator.rs and terminal_simulator.rs.

**Recommendation: KEEP**

Rationale:
- Utilities are well-designed and documented (766 + 788 lines)
- May be useful for future feature development
- No runtime cost (test-only code)
- Document as "available for future use"

**Action:** Add documentation note in `tests/tools/mod.rs`:

```rust
//! Test Utilities Module
//!
//! ## Terminal Validation Utilities
//!
//! Sprint 30 developed dimensional validation utilities that are retained
//! for potential future use:
//!
//! - `visual_validator.rs` - Assertions for terminal width validation
//! - `terminal_simulator.rs` - Simulated terminal for testing
//!
//! These utilities were developed for pager testing but can be applied to
//! any output that needs terminal dimension validation.
```

### Option B File Changes Summary

| File | Action | Lines Changed |
|------|--------|---------------|
| `src/commands/repl/pager.rs` | Replace | 973 -> 50 |
| `src/commands/repl/executor.rs` | Simplify | ~40 lines removed |
| `src/commands/repl/state.rs` | Remove pager fields | ~15 lines removed |
| `src/commands/repl/metacommands.rs` | Update /pager handler | ~10 lines |
| `docs/design/repl.md` | Update | Major sections |
| `docs/specifications/repl.md` | Update | Note pager not supported |
| `docs/roadmap/status.md` | Update | Feature status |
| `tests/tools/mod.rs` | Add documentation | ~10 lines |

### Option B Success Criteria

1. `cargo check` passes
2. `cargo clippy` passes
3. `cargo test --lib` passes
4. No references to removed code remain
5. Documentation accurately reflects removal
6. `/pager` command provides helpful message

---

## Decision Framework

### Choose Option A If:
- Initial debugging reveals simple, fixable issue
- Width mismatch is isolated to one or two functions
- Fix can be validated within 2 hours, leaving 2 hours for testing
- Manual terminal tests pass at all required widths

### Choose Option B If:
- Debugging reveals fundamental architecture issues
- Fix requires changes across multiple interdependent functions
- Option A time-box (4 hours) expires without working solution
- Manual terminal tests continue to fail after fix attempts

### Time-Box Enforcement

```
Hour 0-1: Phase 1 (render_to_buffer) + initial debugging
Hour 1-2.5: Phase 2 (fix implementation)
Hour 2.5-3.5: Phase 3 (manual validation)
Hour 3.5-4: Phase 4 (Track 3 integration) OR decision point

If at hour 4 the feature is not working:
-> Immediately switch to Option B
-> Option B takes ~2 hours
```

---

## Appendix: Code References

### Current Pager Structure (pager.rs)

```
Lines 1-30: Module documentation
Lines 31-44: Imports
Lines 46-56: Constants (MAX_CELL_LENGTH, MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH, INDICATOR_WIDTH)
Lines 58-116: PagerConfig struct and impl
Lines 118-207: ColumnInfo and TableData structs
Lines 209-229: truncate_cell() helper
Lines 231-249: Pager struct
Lines 251-286: Pager::new()
Lines 291-327: visible_column_count() - KEY FUNCTION FOR FIX
Lines 329-339: hidden_columns_left(), hidden_columns_right()
Lines 341-373: render() - main render entry
Lines 375-417: render_border()
Lines 419-463: render_header()
Lines 465-521: render_row()
Lines 523-581: render_status_bar()
Lines 583-641: handle_key()
Lines 643-697: show_help()
Lines 699-731: run() - event loop
Lines 733-801: BorderType, should_page(), display_with_pager()
Lines 803-973: Unit tests
```

### Key Width Calculation (visible_column_count)

Current implementation at lines 291-327 is the primary suspect for width overflow issues. The calculation involves:

1. Checking hidden_left via col_offset
2. Conditionally reserving indicator space
3. Iterating columns and summing widths
4. Breaking when total exceeds available width

The issue is likely in step 2-3 coordination - the indicator space reservation and column width summation may not account for all border characters correctly.
