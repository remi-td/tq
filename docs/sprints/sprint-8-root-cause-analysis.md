# Sprint 8: Root Cause Analysis Report

**Author:** rust-teradata-architect (Opus)
**Date:** 2026-01-18
**Status:** Investigation Complete

---

## Executive Summary

This document provides detailed root cause analysis for all 4 critical bugs reported in Sprint 8. The investigation reveals a systemic quality failure: **all features were implemented with code and unit tests, but never integrated into the actual execution flow or tested against real Teradata databases.**

The bugs fall into two categories:
1. **Integration gaps** (Bugs 1, 3): Code exists but is not wired into the execution path
2. **Silent failures** (Bug 2): Code executes but fails silently without user feedback
3. **Incorrect assumptions** (Bug 4): Hardcoded SQL syntax assumptions that don't match Teradata

---

## Bug 1: Table Padding Completely Broken

### Symptoms
- Column values don't align with headers
- Table output is unreadable with wide columns
- Works in unit tests but broken with real Teradata data

### Root Cause Analysis

**Primary Cause: Incorrect `set_width` calculation in comfy-table**

Location: `src/format/table.rs`, lines 54-59

```rust
table.set_content_arrangement(ContentArrangement::Dynamic);

// Set maximum column width if specified
if let Some(max_width) = options.max_column_width {
    table.set_width(max_width * result.columns.len() as u16);
}
```

**Problems:**

1. **Misunderstanding of `set_width`**: The `set_width` method sets the **total table width**, not max column width. Multiplying `max_column_width` (80) by column count (16 columns = 1280 width) creates arbitrarily wide tables that overflow terminals.

2. **Dynamic arrangement mismatch**: `ContentArrangement::Dynamic` tells comfy-table to dynamically adjust columns, but then we override with an arbitrary fixed width that doesn't correspond to terminal size.

3. **No terminal width detection**: The code doesn't detect actual terminal width, so it can't properly constrain output.

4. **Teradata's wide columns**: Teradata system tables like `DBC.Databases` have CHAR columns with lengths like 30, 60, or even 128 characters. The current logic creates columns padded to 60+ characters each.

**Why Unit Tests Pass:**
- Unit tests use small, well-behaved test data with known column widths
- Tests don't validate actual visual alignment, only presence of data
- No test uses actual Teradata system table schemas with wide CHAR columns

### Evidence

From the user's bug report in `open-bugs.md`:
```
DatabaseName                                                 ┆ CreatorName
```
The column header "DatabaseName" is padded to 60+ characters, but actual data like "val" is also padded, creating massive whitespace gaps.

### Technical Assessment

**Complexity:** Medium
**Risk:** Low (isolated to formatting layer)

### Proposed Fix

1. Remove the erroneous `set_width` calculation
2. Use `ContentArrangement::DynamicFullWidth` which expands to terminal width
3. Detect terminal width using `crossterm::terminal::size()`
4. Set proper per-column max width constraints using `set_constraint` on each column
5. Consider removing max column width or making it per-column rather than total

**Alternative approach:**
Use `ContentArrangement::Dynamic` without `set_width`, letting comfy-table calculate widths from content. Add `Column::set_constraint(ColumnConstraint::LowerBoundary(...))` for minimum widths.

---

## Bug 2: Tab Completion Doesn't Work At All

### Symptoms
- User presses Tab, nothing happens
- No completions, no error messages, no indication of activity
- Sprint 7's main feature completely non-functional

### Root Cause Analysis

**Primary Cause: Silent failure in metadata loading**

The tab completion system has multiple points of silent failure:

#### Issue 1: Lock contention/failure not reported

Location: `src/commands/repl/metadata_completer.rs`, lines 225-228

```rust
let Ok(mut state) = state.lock() else {
    log::warn!("Failed to acquire lock for table completion");
    return vec![];  // Silent return with no user feedback
};
```

If lock acquisition fails, the user sees nothing. The `log::warn!` only goes to log files (which aren't visible in normal REPL operation).

#### Issue 2: Metadata loading failure not reported to user

Location: `src/commands/repl/metadata_completer.rs`, lines 230-233

```rust
if !state.ensure_tables_loaded() {
    return vec![];  // Silent return - no feedback that loading failed
}
```

If `ensure_tables_loaded()` returns false (query failed, timeout, etc.), the user gets no feedback.

#### Issue 3: Aggressive timeout on metadata queries

Location: `src/db/metadata.rs`, lines 16-20

```rust
pub const TABLE_QUERY_TIMEOUT: Duration = Duration::from_millis(500);
pub const COLUMN_QUERY_TIMEOUT: Duration = Duration::from_millis(300);
```

These timeouts are defined but **never actually used**. The metadata queries use the same connection mechanism as regular queries with no special timeout handling. If the metadata query takes >500ms (very likely on large databases), it may simply complete slowly rather than timeout - but if it does timeout, there's no feedback.

#### Issue 4: No "loading" indicator

The user receives no visual feedback that:
- Metadata is being loaded
- A query is in progress
- An error occurred

The Sprint 7 specification mentioned "Visual feedback when loading metadata" but this was never implemented.

#### Issue 5: Query may be failing silently

Location: `src/db/metadata.rs`, lines 237-268

```rust
match client.execute(sql) {
    Ok(result) => {
        // ... process results
    }
    Err(e) => {
        let error_msg = format!("Failed to load table metadata: {}", e);
        log::warn!("{}", error_msg);  // Only logged, not shown to user
        self.last_error = Some(error_msg);
        self.loading_tables = false;
        false
    }
}
```

Errors are logged and stored in `last_error`, but `last_error` is never displayed to the user during completion.

**Why Unit Tests Pass:**
- Unit tests test individual components in isolation
- `MetadataCompleter::keywords_only()` tests don't require database connection
- Tests don't verify actual reedline integration
- No tests execute actual Tab key press against real database

### Evidence

From the user's bug report:
> "Tab completion doesn't work at all, for either databases, tables or columns... And I don't see any indication as to what's happening."

### Technical Assessment

**Complexity:** Medium-High
**Risk:** Medium (needs architectural changes to feedback mechanism)

### Proposed Fix

1. **Add visual feedback during metadata loading:**
   - Show a spinner or "Loading metadata..." message when Tab is first pressed
   - This requires integration with reedline's hint/menu system

2. **Surface errors to user:**
   - When metadata loading fails, display error message in completion dropdown
   - Create a "Error: [message]" pseudo-suggestion

3. **Improve timeout handling:**
   - Actually implement the timeouts that are defined
   - Consider async loading with cancellation

4. **Add logging that's visible to users:**
   - When `--verbose` or a debug mode is enabled, show metadata loading activity

5. **Validate SQL queries against real Teradata:**
   - The metadata SQL in `load_tables` uses `SAMPLE 10000` which is valid
   - But validate the excluded database names list is comprehensive

---

## Bug 3: Result Paging Doesn't Work

### Symptoms
- Arrow keys don't work
- No paging interface appears
- Sprint 5's main feature completely broken

### Root Cause Analysis

**Primary Cause: Pager module exists but is NEVER INTEGRATED into the execution flow**

This is the most clear-cut root cause: the pager code exists (`src/commands/repl/pager.rs`) but is **never called** from anywhere in the actual execution path.

#### Evidence of Non-Integration

1. **Exports exist but are unused:**

   Location: `src/commands/repl/mod.rs`, line 36
   ```rust
   pub use pager::{PagedOutput, PagerConfig};
   ```

   These are exported but never imported anywhere else.

2. **Execution path bypasses pager:**

   Location: `src/commands/repl/executor.rs`

   The `execute_sql_with_state` function directly calls:
   ```rust
   write_output_with_timing(
       &result_clone,
       writer,
       OutputFormat::Table,
       &format_options,
       true,
   )?;
   ```

   There is NO check for `state.is_pager_enabled()` and NO call to `PagedOutput::new()`.

3. **State tracks pager setting but never uses it:**

   Location: `src/commands/repl/state.rs`, lines 155-162
   ```rust
   pub fn set_pager(&mut self, enabled: bool) {
       self.pager_enabled = enabled;
   }

   pub fn is_pager_enabled(&self) -> bool {
       self.pager_enabled
   }
   ```

   The `/pager on|off` metacommand sets this value, but nothing ever reads it to conditionally enable paging.

4. **Pager module is complete but orphaned:**

   The `pager.rs` file contains a complete implementation:
   - `PagerConfig` with all settings
   - `PagedOutput` with scroll state
   - Methods for navigation: `scroll_down()`, `scroll_up()`, `page_down()`, etc.
   - Status line generation
   - Horizontal scrolling support

   But none of this code is ever executed.

**Why Unit Tests Pass:**
- The pager module has unit tests that test its internal logic
- These tests pass because the module works correctly in isolation
- There are no integration tests that verify paging activates in the REPL

### Technical Assessment

**Complexity:** High
**Risk:** Medium (requires terminal mode switching, keyboard event handling)

### Proposed Fix

The pager needs to be integrated into the execution flow. This requires:

1. **Modify `execute_sql_with_state` to check pager setting:**
   ```rust
   // After getting result
   let formatted_output = format_to_string(&result, ...);

   if state.is_pager_enabled() {
       let paged = PagedOutput::new(formatted_output, PagerConfig::default());
       if paged.needs_paging() {
           run_interactive_pager(&mut paged)?;
       } else {
           // Write directly
           write!(writer, "{}", formatted_output)?;
       }
   } else {
       write!(writer, "{}", formatted_output)?;
   }
   ```

2. **Implement `run_interactive_pager` function:**
   - Switch terminal to raw mode using crossterm
   - Set up keyboard event loop
   - Handle navigation keys (j/k, arrows, Page Up/Down)
   - Handle exit key (q, Esc)
   - Render current page with status line
   - Restore terminal on exit

3. **Handle terminal state properly:**
   - Save terminal state before paging
   - Restore state after paging (including on error/panic)
   - Coordinate with reedline (which also manages terminal state)

**Alternative: Use `minus` crate (already in Cargo.toml)**

The project already has `minus = { version = "5.6", features = ["search"] }` as a dependency. `minus` provides a less-like pager. We could use it instead of the custom implementation.

---

## Bug 4: Incorrect LIMIT Hint

### Symptoms
- Message says "Add LIMIT clause" but Teradata doesn't support LIMIT
- Teradata uses TOP or SAMPLE instead
- Confuses users with invalid syntax suggestion

### Root Cause Analysis

**Primary Cause: Hardcoded MySQL/PostgreSQL syntax in hint message**

Location: `src/commands/repl/executor.rs`, lines 83-89 and 169-175

```rust
if limited {
    writeln!(writer)?;
    writeln!(
        writer,
        "Showing first {} rows. Add LIMIT clause for different results.",
        default_limit
    )?;
}
```

This message appears twice in `executor.rs`:
1. In `execute_sql` (line 87)
2. In `execute_sql_with_state` (line 173)

The message assumes standard SQL LIMIT syntax, but Teradata uses:
- `SELECT TOP N * FROM table` - First N rows
- `SELECT * FROM table SAMPLE N` - Random N rows

**Why Unit Tests Pass:**
- Unit tests don't validate message content for correctness
- No Teradata-specific syntax validation in tests

### Evidence

From user bug report:
> "LIMIT isn't a valid Teradata SQL keyword (TOP is the equivalent keyword, SAMPLE is an alternative)"

### Technical Assessment

**Complexity:** Low
**Risk:** Very Low (simple text change)

### Proposed Fix

Change the hint message in both locations to:

```rust
writeln!(
    writer,
    "Showing first {} rows. Use TOP N or SAMPLE N for different results.",
    default_limit
)?;
```

Additionally, scan for other places that might assume standard SQL syntax and update them:
- Help text
- Error messages
- Documentation

---

## Systemic Issues Identified

### Issue 1: Unit Tests Don't Validate Integration

All four bugs share a common pattern: unit tests pass but features don't work. This reveals:

1. **Component tests vs. integration tests**: We test components in isolation but don't test them working together
2. **Mock vs. real database**: We test with mock data but not against real Teradata
3. **Code coverage vs. feature coverage**: We have code coverage but not feature coverage

### Issue 2: Silent Failures Are Undetectable

Multiple features fail silently:
- Tab completion fails with no feedback
- Pager setting is ignored with no warning
- Metadata loading errors go to log files only

### Issue 3: Incomplete Implementation Marked as Complete

Sprint reviews marked features "complete" when:
- Code existed
- Unit tests passed
- But integration was never completed
- And manual testing was never performed

### Issue 4: Documentation-Code Mismatch

- REPL banner says "Result paging: enabled" even though paging doesn't work
- `/help` documents `/pager on|off` but it has no effect
- Architecture docs describe paging features that aren't implemented

---

## Technical Feasibility Assessment

| Bug | Complexity | Effort | Risk | Architectural Changes Needed |
|-----|------------|--------|------|------------------------------|
| Bug 1: Table Padding | Medium | 2-4 hours | Low | No - fix formatting logic |
| Bug 2: Tab Completion | Medium-High | 4-8 hours | Medium | Minor - add feedback UI |
| Bug 3: Result Paging | High | 8-16 hours | Medium | Yes - implement pager integration |
| Bug 4: LIMIT Hint | Low | 30 min | Very Low | No - text change only |

### Can All Bugs Be Fixed in One Sprint?

**Yes, with caveats:**

- Bug 4 can be fixed immediately (30 minutes)
- Bug 1 can be fixed in one session (2-4 hours)
- Bug 2 requires more investigation of reedline's completion feedback mechanisms
- Bug 3 is the most complex - requires terminal mode handling and keyboard event loops

**Recommended priority order:**
1. Bug 4 (LIMIT hint) - Quick win, immediate user impact
2. Bug 1 (Table padding) - Core functionality, visible improvement
3. Bug 2 (Tab completion) - Important feature, medium complexity
4. Bug 3 (Paging) - Complex, may need to be scoped down

---

## Files Requiring Modification

### Bug 1 (Table Padding)
- `src/format/table.rs` - Fix width calculation

### Bug 2 (Tab Completion)
- `src/commands/repl/metadata_completer.rs` - Add error feedback
- `src/db/metadata.rs` - Improve error handling

### Bug 3 (Paging)
- `src/commands/repl/executor.rs` - Integrate pager
- `src/commands/repl/pager.rs` - May need enhancements
- `src/commands/repl/mod.rs` - Wire up pager

### Bug 4 (LIMIT Hint)
- `src/commands/repl/executor.rs` - Change hint text (2 locations)

---

## Implementation Plan

### Phase 1: Quick Wins (1-2 hours)
1. Fix Bug 4 (LIMIT hint) - Simple text change
2. Verify change with live database

### Phase 2: Table Formatting (2-4 hours)
1. Remove erroneous `set_width` calculation
2. Implement proper terminal width detection
3. Test with real Teradata system tables
4. Verify alignment with various column widths

### Phase 3: Tab Completion Feedback (4-8 hours)
1. Research reedline's feedback mechanisms
2. Implement loading indicator (if possible)
3. Surface errors as pseudo-suggestions
4. Add verbose logging option
5. Test against real Teradata with large schemas

### Phase 4: Pager Integration (8-16 hours)
1. Design pager integration architecture
2. Implement terminal mode switching
3. Implement keyboard event loop
4. Integrate with executor
5. Handle edge cases (Ctrl-C, terminal resize)
6. Test with large result sets

### Testing Strategy

For each fix:
1. Verify existing unit tests still pass
2. Add new unit tests for the fix
3. **Mandatory:** Test against live Teradata database
4. **Mandatory:** Document test results with screenshots/logs

---

## Conclusion

The four bugs stem from a common root cause: **incomplete integration and lack of real-world testing**. The code exists and is well-structured, but it was never wired into the actual execution paths and never validated against real Teradata databases.

The fixes are all technically feasible within Sprint 8, with Bug 3 (paging) being the most complex. The recommended approach is to prioritize by user impact and complexity, starting with the quick wins (Bug 4, Bug 1) before tackling the more complex issues (Bug 2, Bug 3).

Most importantly, this sprint must establish the practice of **mandatory live database testing** before any feature is marked complete.
