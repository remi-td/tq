# Sprint 9: Bug 3 & Bug 4 Design Document

**Created:** 2026-01-18
**Sprint:** 9
**Author:** cli-ux-designer agent
**Status:** Design Complete

---

## Overview

This document provides detailed UX/UI design specifications for fixing:
- **Bug 3**: Error Message Formatting (Stack Traces)
- **Bug 4**: LIMIT Hint Message (Teradata Syntax)

Both bugs affect user-facing messages and require careful attention to clarity, professionalism, and Teradata-specific correctness.

---

## Bug 3: Error Message Formatting

### Current State

**Problem:** When SQL errors occur, users see full Go stack traces from the Teradata driver library:

```
Error: SQL syntax error: [Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error, expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.
 at gosqldriver/teradatasql.formatError ErrorUtil.go:101
 at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210
 at gosqldriver/teradatasql.(*teradataConnection).makeChainedDatabaseError ErrorUtil.go:226
 at gosqldriver/teradatasql.(*teradataConnection).processErrorParcel TeradataConnection.go:347
 at gosqldriver/teradatasql.(*TeradataRows).processResponseBundle TeradataRows.go:2724
 at gosqldriver/teradatasql.(*TeradataRows).executeSQLRequest TeradataRows.go:1194
 at gosqldriver/teradatasql.newTeradataRows TeradataRows.go:805
 at gosqldriver/teradatasql.(*teradataStatement).QueryContext TeradataStatement.go:122
 at gosqldriver/teradatasql.(*teradataConnection).QueryContext TeradataConnection.go:836
 at database/sql.ctxDriverQuery ctxutil.go:48
 at database/sql.(*DB).queryDC.func1 sql.go:1786
 at database/sql.withLock sql.go:3572
 at database/sql.(*DB).queryDC sql.go:1781
 at database/sql.(*Conn).QueryContext sql.go:2037
 at main.createRows goside.go:1142
 at main.rustgoCreateRows goside.go:999
 at _cgoexp_c43d071e9719_rustgoCreateRows _cgo_gotypes.go:416
 at runtime.cgocallbackg1 cgocall.go:446
 at runtime.cgocallbackg cgocall.go:350
 at runtime.cgocallback asm_arm64.s:1180
 at runtime.goexit asm_arm64.s:1268
```

**User Impact:**
- Unprofessional and confusing
- Obscures the actual SQL error message
- Technical stack trace is irrelevant to end users
- Makes debugging harder, not easier

---

### Design Solution

#### Principles

1. **Show Only Relevant Information**: Display the SQL error message and metadata needed for debugging
2. **Suppress Stack Traces**: Internal Go stack traces serve no purpose for SQL users
3. **Professional Appearance**: Clean, scannable format similar to other CLI tools
4. **Actionable**: Help users understand what went wrong

#### Error Message Format

**Template for SQL Errors:**
```
Error: [Short Error Type]

[Actual SQL Error Message - User-Friendly Text]

Error Code: [Teradata Error Code]
Session ID: [Session Number]
```

**Example - After Fix:**
```
Error: SQL syntax error

Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.

Error Code: 3707
Session ID: 1429
```

#### What to Include

**INCLUDE:**
- ✅ Error type (syntax error, permission error, table not found, etc.)
- ✅ Actual SQL error message (the descriptive text)
- ✅ Teradata error code (for DBA support and documentation lookup)
- ✅ Session ID (for troubleshooting with DBAs)

**SUPPRESS:**
- ❌ Stack traces (all lines starting with "at")
- ❌ Version information (redundant, not helpful for SQL errors)
- ❌ Internal function names (gosqldriver/*, runtime.*, etc.)
- ❌ File paths and line numbers from driver internals

#### Parsing Strategy

The current error format from Teradata driver:
```
[Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] <message>
 at <stack trace lines...>
```

**Extraction Logic:**
1. **Session ID**: Extract from `[Session NNNN]` pattern
2. **Error Code**: Extract from `[Error NNNN]` pattern
3. **Message**: Everything after the last `]` before the first `at` line
4. **Stack Trace**: Discard all lines starting with `at` (preceded by whitespace)

**Pseudocode:**
```rust
fn format_sql_error(raw_error: &str) -> String {
    // Extract session ID: [Session 1429]
    let session = extract_pattern(raw_error, r"\[Session (\d+)\]");

    // Extract error code: [Error 3707]
    let error_code = extract_pattern(raw_error, r"\[Error (\d+)\]");

    // Extract message: everything after last ] and before "at"
    let message = extract_message(raw_error);

    // Build clean error output
    format!(
        "Error: SQL syntax error\n\n{}\n\nError Code: {}\nSession ID: {}",
        message, error_code, session
    )
}

fn extract_message(raw_error: &str) -> String {
    // Find the actual error message text
    // Skip version, session, database, error code metadata
    // Take text until first " at " line
    let lines: Vec<&str> = raw_error.lines().collect();

    // First line contains metadata + message
    let first_line = lines[0];

    // Find position after last ']' in metadata section
    if let Some(pos) = first_line.rfind(']') {
        let message = &first_line[pos+1..].trim();
        return message.to_string();
    }

    // Fallback: return full first line
    lines[0].to_string()
}
```

#### Edge Cases

**Case 1: Error without stack trace**
- Some errors may not include stack traces
- Still apply formatting, just show the message cleanly

**Case 2: Multi-line error messages**
- Some SQL errors span multiple lines
- Preserve line breaks within the message text
- Stop at first `at` line

**Case 3: Non-SQL errors**
- Connection errors, authentication errors should use their existing formats
- Only apply this formatting to `TqError::SqlSyntaxError` and `TqError::QueryExecution`

**Case 4: Unknown error format**
- If parsing fails, show the original error
- Better to show raw error than hide information

---

### Implementation Locations

**File: `src/error.rs`**

1. **Add helper function** to parse and format Teradata error messages:
   ```rust
   impl TqError {
       /// Format Teradata SQL error message for user display
       /// Extracts error code, session ID, and message while suppressing stack traces
       fn format_teradata_error(raw_message: &str) -> String {
           // Implementation as designed above
       }
   }
   ```

2. **Update `user_message()` method** for SQL error variants:
   ```rust
   TqError::SqlSyntaxError { message, query } => {
       let formatted = Self::format_teradata_error(message);
       // Build user-friendly output
   }

   TqError::QueryExecution(message) => {
       let formatted = Self::format_teradata_error(message);
       // Build user-friendly output
   }
   ```

**File: `src/commands/repl/executor.rs`**

- No changes needed - errors are already passed through `TqError::user_message()`
- The fix in `error.rs` will automatically improve REPL error display

**Testing Locations:**
- `tests/cases/TC064.md` - Test case for error message validation
- `tests/cases/TC065.md` - Test case for user-facing message correctness
- Add unit tests in `src/error.rs` for message parsing

---

## Bug 4: LIMIT Hint Message

### Current State

**Problem:** When displaying large result sets, tq shows a hint message suggesting SQL syntax:

```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Issue:** Teradata does **not** support `LIMIT` syntax. The correct Teradata syntax is:
- `SELECT TOP N ...` (limit to N rows)
- `SELECT ... SAMPLE N` (sample N rows)

**User Impact:**
- Confuses users who try to use `LIMIT` in Teradata
- Suggests invalid syntax
- Indicates lack of Teradata expertise

---

### Design Solution

#### Principles

1. **Use Correct Teradata Syntax**: Only reference syntax that actually works in Teradata
2. **Be Helpful**: Provide clear guidance on how to control result size
3. **Consistency**: Update ALL user-facing messages

#### Updated Hint Message

**Current (Incorrect):**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Fixed (Correct):**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**Examples in Help Text (Before):**
```
SELECT * FROM large_table LIMIT 100
```

**Examples in Help Text (After):**
```
SELECT TOP 100 * FROM large_table
SELECT * FROM large_table SAMPLE 100
```

---

### All Files Requiring Updates

Based on grep search for "LIMIT", the following files contain references:

#### 1. Source Code Files (CRITICAL - User-Facing)

**File: `src/commands/repl/executor.rs`**
- **Line 87-88**: Hint message after applying row limit
  ```rust
  // BEFORE:
  "Showing first {} rows. Add LIMIT clause for different results."

  // AFTER:
  "Showing first {} rows. Use TOP N or SAMPLE N for different results."
  ```
- **Status**: ✅ ALREADY FIXED in current code (verified by grep output)

**File: `src/cli.rs`**
- **Line 271**: Comment about LIMIT clause behavior
  ```rust
  // BEFORE:
  /// In REPL mode, SELECT queries without an explicit LIMIT clause will

  // AFTER:
  /// In REPL mode, SELECT queries without TOP or SAMPLE will
  ```
- **Status**: ⚠️ NEEDS UPDATE

#### 2. Documentation Files (Important for Consistency)

**File: `docs/builder/detailed-specifications/repl-mode.md`**
- **Lines 1486, 1493**: Hint message examples
- Update to use "TOP N or SAMPLE N"
- **Status**: ✅ ALREADY UPDATED (verified by grep output)

**File: `docs/builder/rust-architecture.md`**
- **Line 13**: LIMIT hint message mention
- Likely already documents the fix
- **Status**: ✅ ALREADY DOCUMENTED (mentions the change)

#### 3. Test Cases (Critical for Validation)

**File: `tests/cases/TC064.md`**
- **Lines 64, 69, 126, 136, 181, 192**: Test assertions for hint message
- Update expected output to match new message
- **Status**: ⚠️ NEEDS UPDATE

**File: `tests/cases/TC065.md`**
- **Lines 48, 56, 152, 202**: Negative test for LIMIT references
- Tests that verify LIMIT is NOT mentioned
- **Status**: ⚠️ REVIEW (likely already correct, verify)

#### 4. Documentation Files (Reference Only)

**File: `Readme.md`**
- General documentation may reference LIMIT
- **Status**: ⚠️ REVIEW and UPDATE if needed

**File: `BUG-ANALYSIS.md`**
- Sprint 8 analysis document (historical)
- **Status**: ℹ️ INFORMATIONAL ONLY - documents the bug, no action needed

**File: `docs/builder/specifications.md`**
- Main specifications dashboard
- **Status**: ⚠️ REVIEW - may need consistency updates

**File: `docs/builder/user/roadmap.md`**
- User-facing roadmap
- **Status**: ⚠️ REVIEW - may reference LIMIT in examples

**File: `docs/builder/detailed-specifications/batch-mode.md`**
- Batch mode specification
- **Status**: ⚠️ REVIEW - may contain LIMIT examples

**File: `docs/builder/detailed-specifications/cli-interface.md`**
- CLI interface specification
- **Status**: ⚠️ REVIEW - may contain LIMIT examples

**File: `docs/builder/detailed-specifications/user-personas.md`**
- User personas and use cases
- **Status**: ⚠️ REVIEW - may contain LIMIT in example queries

**File: `docs/builder/detailed-specifications/error-handling.md`**
- Error handling specification
- **Status**: ⚠️ REVIEW - may contain LIMIT in examples

#### 5. Non-User-Facing Files (Low Priority)

**Files with LIMIT in code comments or internal logic:**
- `src/commands/repl/metadata_completer.rs` - Internal completion logic
- `src/commands/repl/sql_context.rs` - SQL parsing logic
- `src/commands/repl/highlighter.rs` - Syntax highlighting
- `src/commands/repl/completer.rs` - Tab completion
- `tests/cases/INDEX.md` - Test index

**Status**: ℹ️ REVIEW - Only update if they affect user-facing behavior

---

### Implementation Summary

#### Files Requiring Code Changes

| File | Lines | Change Type | Priority | Status |
|------|-------|-------------|----------|--------|
| `src/cli.rs` | 271 | Comment update | Medium | ⚠️ TODO |
| `tests/cases/TC064.md` | Multiple | Test assertion update | High | ⚠️ TODO |
| `tests/cases/TC065.md` | Multiple | Verify test assertions | High | ⚠️ TODO |

#### Files Requiring Documentation Updates

| File | Change Type | Priority | Status |
|------|-------------|----------|--------|
| `Readme.md` | Remove LIMIT references | Medium | ⚠️ TODO |
| `docs/builder/specifications.md` | Consistency check | Medium | ⚠️ TODO |
| `docs/builder/user/roadmap.md` | Example updates | Low | ⚠️ TODO |
| `docs/builder/detailed-specifications/*.md` | Example updates | Medium | ⚠️ TODO |

#### Already Fixed

| File | Status |
|------|--------|
| `src/commands/repl/executor.rs` | ✅ COMPLETE |
| `docs/builder/detailed-specifications/repl-mode.md` | ✅ COMPLETE |
| `docs/builder/rust-architecture.md` | ✅ DOCUMENTED |

---

## Testing Strategy

### Bug 3: Error Message Formatting

**Test Plan:**

1. **Generate SQL syntax errors** with various error codes:
   ```sql
   SELCT * FROM table;           -- Misspelled keyword
   SELECT * FROM nonexistent;     -- Table not found
   SELECT * FROM db.;             -- Incomplete table reference
   DROP TABLE protected_table;    -- Permission error
   ```

2. **Verify error output**:
   - ✅ No stack traces visible (no "at gosqldriver/..." lines)
   - ✅ Error code displayed
   - ✅ Session ID displayed
   - ✅ Clear, readable error message
   - ✅ Professional formatting

3. **Edge case testing**:
   - Multi-line error messages
   - Errors without error codes
   - Non-SQL errors (connection, auth) still formatted correctly

4. **Visual inspection**:
   - User validates error messages are professional and helpful
   - Messages are actionable (user understands what to fix)

### Bug 4: LIMIT Hint Message

**Test Plan:**

1. **Run queries triggering row limit**:
   ```sql
   SELECT * FROM large_table;  -- Returns 100+ rows
   ```

2. **Verify hint message**:
   - ✅ Message says "Use TOP N or SAMPLE N"
   - ✅ NO mention of "LIMIT clause"
   - ✅ Clear and accurate guidance

3. **Grep verification**:
   ```bash
   # Search ALL source files for LIMIT references
   grep -r "LIMIT" src/ docs/ tests/ Readme.md

   # Verify no user-facing LIMIT references remain
   # Exceptions: internal code comments OK, BUG-ANALYSIS.md OK (historical)
   ```

4. **Documentation review**:
   - All examples use Teradata syntax (TOP N or SAMPLE N)
   - Help text uses correct syntax
   - No MySQL/PostgreSQL syntax suggestions

---

## Acceptance Criteria

### Bug 3: Error Message Formatting

- [ ] SQL errors display **clean, professional messages** without stack traces
- [ ] Error messages include **error code** and **session ID**
- [ ] Error messages are **actionable** and easy to understand
- [ ] Stack traces are **completely suppressed** for SQL errors
- [ ] Non-SQL errors (connection, auth) still display appropriately
- [ ] User validates error messages are **professional and helpful**

### Bug 4: LIMIT Hint Message

- [ ] Row limit hint message uses **"TOP N or SAMPLE N"** syntax
- [ ] NO references to "LIMIT clause" in **user-facing messages**
- [ ] Code comments updated for consistency (low priority)
- [ ] Test cases updated with correct expected messages
- [ ] Documentation updated with Teradata syntax examples
- [ ] Grep search confirms no user-facing LIMIT references remain

---

## Design Review Checklist

Before implementation:

- [x] Design follows CLI UX best practices
- [x] Error messages are professional and actionable
- [x] Teradata syntax is correct and consistent
- [x] All user-facing messages identified
- [x] Test strategy is comprehensive
- [x] Edge cases considered
- [x] Acceptance criteria are clear and measurable

---

## Next Steps

1. **User Review**: Get approval on error message format and hint message text
2. **Implementation**: rust-teradata-architect implements the changes
3. **Testing**: quality-validator executes test plan
4. **User Validation**: User tests error messages in real REPL session
5. **Documentation**: Update specifications with final implementation

---

## Appendix: Error Message Examples

### Example 1: Syntax Error (After Fix)

```
Error: SQL syntax error

Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.

Error Code: 3707
Session ID: 1429
```

### Example 2: Table Not Found (After Fix)

```
Error: Table does not exist

Object 'nonexistent_table' does not exist.

Error Code: 3807
Session ID: 1429
```

### Example 3: Permission Denied (After Fix)

```
Error: Permission denied

User 'alice' does not have DROP privilege on table 'important_data'.

Error Code: 3523
Session ID: 1429
```

### Example 4: Row Limit Hint (After Fix)

```
┌────┬────────┬────────┐
│ id │ name   │ value  │
├────┼────────┼────────┤
│ 1  │ Alice  │ 100    │
│ 2  │ Bob    │ 200    │
... (98 more rows)
└────┴────────┴────────┘

Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

---

## References

- Sprint 9 Planning: `/Users/remi.turpaud/Code/genAI/tq/docs/builder/sprints/sprint-9-planning.md`
- Error Handling Spec: `/Users/remi.turpaud/Code/genAI/tq/docs/builder/detailed-specifications/error-handling.md`
- Current Implementation: `/Users/remi.turpaud/Code/genAI/tq/src/error.rs`
- REPL Executor: `/Users/remi.turpaud/Code/genAI/tq/src/commands/repl/executor.rs`
