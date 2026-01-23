# Sprint 8 - Exact Bugs Identified (Option C)

**Date:** 2026-01-18
**Analysis:** Direct code examination by Sprint Coordinator

---

## Bug 2: Tab Completion - ROOT CAUSES IDENTIFIED

### Bug 2.1: Wrong Completion Logic - Shows Tables Instead of Databases

**File:** `src/commands/repl/metadata_completer.rs` + `src/db/metadata.rs`

**Problem:** When user types `SELECT * FROM <Tab>`, they should see DATABASE NAMES (DBC, val, TD_SYSAL, etc.) but instead see table names (val.customer, TD_SysXML.UTF_V, etc.).

**Root Cause:**
The completion logic doesn't understand Teradata's two-level naming model (`database.table`).

**Current Flow:**
1. User types `SELECT * FROM <Tab>`
2. `sql_context.rs` detects `TableName` context
3. `metadata_completer.rs` line 500: calls `complete_tables(&prefix)`
4. Line 249: calls `state.cache().find_tables_by_prefix(prefix)`
5. **BUG:** `find_tables_by_prefix` returns ALL cached tables with their qualified names

**What Should Happen:**
1. User types `SELECT * FROM <Tab>`
2. Should show **DATABASE NAMES** (DBC, val, TD_SYSAL, etc.)
3. After user types `FROM DBC.<Tab>`, THEN show tables in DBC database

**Missing Component:**
Need separate `find_databases()` method that returns distinct database names, not table names.

**Fix Required:**
1. Add `MetadataCache::get_databases()` method to return distinct database names
2. Modify `complete_tables()` to detect if we're completing databases vs tables
3. When prefix is empty and we're in FROM context, return databases + tables in current DB
4. When we have `schema.` prefix, return tables in that schema

---

### Bug 2.2: Crash on `dbc.<Tab>` - Schema Parsing Issue

**File:** `src/commands/repl/sql_context.rs` line 250-283

**Problem:** Program crashes or errors when user types `select * from dbc.<Tab>`

**Root Cause:**
The `check_schema_qualified` function has confusing logic that might cause panics or return wrong contexts.

**Code Analysis:**
```rust
// Line 257-269
if text.trim_end().ends_with('.') && !last_word.is_empty() {
    let trimmed = text.trim_end();
    let without_dot = &trimmed[..trimmed.len() - 1];
    let schema = get_last_word(without_dot);

    if !schema.is_empty() {
        return Some(CompletionContext::SchemaQualifiedTable {
            schema: schema.to_string(),
            prefix: String::new(),
        });
    }
}
```

**Issue:** The condition `text.trim_end().ends_with('.') && !last_word.is_empty()` is logically inconsistent. If text ends with '.', then `last_word` (which is extracted by `get_last_word`) should either include the dot or be empty, depending on implementation.

**Additional Issue:**
Line 272-279 splits `last_word` on '.' - for input "dbc.", this creates ["dbc", ""], which has len()==2, so it returns `SchemaQualifiedTable { schema: "dbc", prefix: "" }`. This is then passed to `complete_schema_tables`, which builds query for "dbc." but may fail if the table lookup doesn't handle empty prefixes correctly.

**Fix Required:**
1. Simplify the schema-qualified detection logic
2. Add proper error handling for empty prefixes
3. Test with live database

---

## Bug 3: Result Paging - ROOT CAUSES IDENTIFIED

### Bug 3.1: Garbled Display - Multiple Rendering Bugs

**File:** `src/commands/repl/pager.rs`

**Problem:** Complete chaos - characters scattered randomly across screen, no table structure visible

**Root Cause 1: Missing Leading Border (Line 382-403)**
```rust
fn render_row(...) {
    let mut row_str = String::from("│");  // Line 382 - created but NEVER written!

    for col in &self.data.columns[start_col..end_col] {
        // ... writes padded value directly to stdout
        write!(stdout, "{}", padded)?;
        row_str.clear();  // Line 399 - clears unused buffer
        write!(stdout, "│")?;  // Line 400 - writes separator
    }

    writeln!(stdout)  // Line 403
}
```

**BUG:** The leading "│" is created in `row_str` but never written to stdout! So each row is missing its left border, causing misalignment.

**Should be:**
```rust
fn render_row(...) {
    write!(stdout, "│")?;  // Write leading border FIRST

    for col in &self.data.columns[start_col..end_col] {
        write!(stdout, " {:width$} │", value, width = col.display_width)?;
    }

    writeln!(stdout)
}
```

---

**Root Cause 2: Fragile Table Parsing (Line 196-204)**
```rust
fn parse_row_cells(line: &str) -> Vec<String> {
    line.split('│')
        .skip(1) // Skip leading empty
        .take_while(|s| !s.is_empty() || line.ends_with('│'))  // CONFUSING LOGIC
        .filter(|s| !s.trim().is_empty() || s.len() > 0)       // REDUNDANT
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

**BUG:** This fragile parsing of comfy-table output fails on edge cases:
- The `take_while` condition `!s.is_empty() || line.ends_with('│')` is confusing
- Two filters on emptiness are redundant and might filter out empty cells
- If comfy-table format changes slightly, parsing breaks completely

**Fundamental Problem:** The pager is trying to REPARSE and RE-RENDER output that's already been formatted by comfy-table. This is architecturally wrong.

---

**Root Cause 3: Unused row_str Buffer**

The `row_str` variable is created at line 382 but never accumulated into. All writes go directly to stdout, making row_str completely useless. This wastes memory and adds confusion.

---

### Bug 3.2: Double-Rendering Architecture

**Fundamental Design Flaw:**
The pager receives FORMATTED TABLE OUTPUT from comfy-table, then:
1. Parses it back into raw data (fragile)
2. Re-renders it in a custom format (buggy)

This double-rendering is the root cause of all paging issues.

**Better Approach:**
1. Get RAW data from query executor (before comfy-table formatting)
2. Pass raw data to pager
3. Pager renders directly from raw data (no parsing needed)

---

## Summary of Required Fixes

### Bug 2 (Tab Completion)

**Short-term fixes:**
1. Add `MetadataCache::get_databases()` to return distinct database names
2. Modify `complete_tables()` to check if completing databases vs tables:
   - If `prefix.is_empty()` and in FROM context → return databases + current DB tables
   - If `prefix.contains('.')` → parse and return schema-specific tables
3. Add error handling for schema-qualified completion
4. Test with live database

**Implementation Priority:** HIGH - This is blocking users

---

### Bug 3 (Result Paging)

**Option A: Quick Fix (RECOMMENDED)**
1. Fix the missing leading "│" in `render_row` (line 382-403)
2. Simplify `parse_row_cells` logic
3. Add more robust parsing with error handling
4. Test with real queries

**Option B: Proper Fix (More Work)**
1. Change architecture: Pass RAW DATA to pager, not formatted string
2. Modify executor to provide raw result set
3. Pager renders directly from raw data (no parsing)
4. More reliable and maintainable

**Recommendation:** Start with Option A to get something working quickly, plan Option B for Sprint 9.

---

## Testing Requirements

After implementing fixes:

1. **Bug 2 Testing (with live database):**
   ```sql
   tq> SELECT * FROM <Tab>
   Expected: List of databases (DBC, val, TD_SYSAL, etc.)

   tq> SELECT * FROM DBC.<Tab>
   Expected: Tables in DBC database (DatabasesV, TablesV, etc.)

   tq> SELECT * FROM val.<Tab>
   Expected: Tables in val database
   ```

2. **Bug 3 Testing (with live database):**
   ```sql
   tq> SELECT * FROM DBC.DatabasesV;
   Expected: Readable paged table with proper borders and alignment

   tq> Press 'q'
   Expected: Return to tq> prompt (not exit program)
   ```

---

## Implementation Plan

1. **Fix Bug 2 first** - Higher user impact, clearer fix path
2. **Fix Bug 3** - More complex, but bugs are now identified
3. **Test each fix immediately with live database**
4. **User validates before declaring complete**

---

**Next Steps:** Implement fixes based on this analysis.
