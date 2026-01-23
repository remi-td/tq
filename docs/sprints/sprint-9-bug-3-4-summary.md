# Sprint 9: Bug 3 & Bug 4 Design Summary

**Created:** 2026-01-18
**For:** rust-teradata-architect implementation
**Status:** Ready for Implementation

---

## Quick Reference

This is a concise summary for the implementation agent. See `sprint-9-bug-3-4-design.md` for full design details.

---

## Bug 3: Error Message Formatting

### Problem
Stack traces from Teradata driver are shown to users, making errors unprofessional and hard to read.

### Solution
Parse and format SQL errors to show only relevant information.

### Error Message Format

**Template:**
```
Error: [Short Error Type]

[Actual SQL Error Message]

Error Code: [Teradata Error Code]
Session ID: [Session Number]
```

**Example:**
```
Error: SQL syntax error

Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.

Error Code: 3707
Session ID: 1429
```

### Implementation

**File: `src/error.rs`**

Add helper method:
```rust
impl TqError {
    /// Format Teradata SQL error by extracting message, error code, session ID
    /// and suppressing stack traces
    fn format_teradata_error(raw_message: &str) -> String {
        // 1. Extract session ID: [Session 1429]
        // 2. Extract error code: [Error 3707]
        // 3. Extract message: text after last ] and before "at" lines
        // 4. Format cleanly without stack traces
    }
}
```

Update `user_message()` for these variants:
- `TqError::SqlSyntaxError`
- `TqError::QueryExecution`

**Key Parsing Rules:**
- Extract `[Session NNNN]` → Session ID
- Extract `[Error NNNN]` → Error Code
- Extract message after last `]` bracket
- **Discard all lines starting with "at"** (stack traces)

---

## Bug 4: LIMIT Hint Message

### Problem
Hint message suggests `LIMIT` syntax, which doesn't work in Teradata.

### Solution
Update all messages to use Teradata syntax: `TOP N` or `SAMPLE N`

### Message Changes

**Before:**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**After:**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

---

## Files Requiring Changes

### Critical (User-Facing Code)

| File | Line(s) | Change | Status |
|------|---------|--------|--------|
| `src/error.rs` | Multiple | Add `format_teradata_error()` helper | 🔴 TODO |
| `src/error.rs` | Multiple | Update `user_message()` for SQL errors | 🔴 TODO |
| `src/cli.rs` | 271 | Update comment: "LIMIT clause" → "TOP or SAMPLE" | 🔴 TODO |
| `src/commands/repl/executor.rs` | 87-88 | Message already fixed | ✅ DONE |

### Critical (Tests)

| File | Change | Status |
|------|--------|--------|
| `tests/cases/TC064.md` | Update expected hint message output | 🔴 TODO |
| `tests/cases/TC065.md` | Verify no LIMIT references | 🔴 TODO |

### Important (Documentation)

| File | Change | Status |
|------|--------|--------|
| `Readme.md` | Replace LIMIT with TOP/SAMPLE in examples | 🔴 TODO |
| `docs/builder/specifications.md` | Consistency check | 🔴 TODO |
| `docs/builder/detailed-specifications/batch-mode.md` | Update examples if present | 🔴 TODO |
| `docs/builder/detailed-specifications/cli-interface.md` | Update examples if present | 🔴 TODO |
| `docs/builder/detailed-specifications/user-personas.md` | Update query examples | 🔴 TODO |
| `docs/builder/detailed-specifications/error-handling.md` | Update examples | 🔴 TODO |

### Informational Only (No Changes Needed)

| File | Reason |
|------|--------|
| `docs/builder/detailed-specifications/repl-mode.md` | ✅ Already updated |
| `docs/builder/rust-architecture.md` | ✅ Already documents fix |
| `BUG-ANALYSIS.md` | Historical document, no changes needed |

---

## Implementation Steps

### Phase 1: Bug 3 (Error Message Formatting)

1. **Add error parsing helper** in `src/error.rs`:
   - `format_teradata_error(raw_message: &str) -> String`
   - Extract session ID, error code, message
   - Suppress stack traces (lines starting with "at")

2. **Update `user_message()` method** in `src/error.rs`:
   - Call `format_teradata_error()` for SQL errors
   - Apply to `SqlSyntaxError` and `QueryExecution` variants

3. **Add unit tests** in `src/error.rs`:
   - Test error parsing with sample Teradata error messages
   - Test stack trace suppression
   - Test edge cases (missing session, missing error code)

4. **Manual testing**:
   - Generate SQL syntax errors in REPL
   - Verify clean error output (no stack traces)
   - Verify error code and session ID are displayed

### Phase 2: Bug 4 (LIMIT Hint Message)

1. **Update code comment** in `src/cli.rs` line 271:
   - Change comment from "LIMIT clause" to "TOP or SAMPLE"

2. **Update test cases**:
   - `tests/cases/TC064.md`: Update expected hint message
   - `tests/cases/TC065.md`: Verify assertions still pass

3. **Update documentation**:
   - Search and replace LIMIT examples with TOP/SAMPLE
   - Focus on user-facing docs (Readme, specifications, CLI interface)

4. **Grep verification**:
   ```bash
   grep -r "LIMIT" src/ docs/ tests/ Readme.md
   ```
   - Verify no user-facing LIMIT references remain
   - Internal comments OK, historical docs OK

---

## Testing Checklist

### Bug 3: Error Message Formatting

- [ ] Generate syntax error: `SELCT * FROM table;`
- [ ] Verify: No stack traces visible
- [ ] Verify: Error code displayed
- [ ] Verify: Session ID displayed
- [ ] Verify: Message is clear and professional
- [ ] Test: Multi-line error messages
- [ ] Test: Errors without error codes
- [ ] Test: Non-SQL errors still work correctly

### Bug 4: LIMIT Hint Message

- [ ] Run query: `SELECT * FROM large_table;` (100+ rows)
- [ ] Verify: Hint says "Use TOP N or SAMPLE N"
- [ ] Verify: NO mention of "LIMIT clause"
- [ ] Run grep: No user-facing LIMIT references
- [ ] Visual inspection: All examples use Teradata syntax

---

## Acceptance Criteria

### Bug 3
- [x] Error messages are clean and professional (no stack traces)
- [x] Error code and session ID are displayed
- [x] Messages are actionable and easy to understand
- [x] User validates errors are helpful

### Bug 4
- [x] Hint message uses "TOP N or SAMPLE N"
- [x] No user-facing references to "LIMIT clause"
- [x] Tests updated with correct expected output
- [x] Documentation uses Teradata syntax

---

## Error Message Parsing Examples

### Input (Raw Teradata Error)
```
[Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error, expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.
 at gosqldriver/teradatasql.formatError ErrorUtil.go:101
 at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210
 ... (more stack trace lines)
```

### Output (Formatted for User)
```
Error: SQL syntax error

Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.

Error Code: 3707
Session ID: 1429
```

### Parsing Logic
1. Extract session: `[Session 1429]` → `1429`
2. Extract error code: `[Error 3707]` → `3707`
3. Extract message: Text after last `]`, before `at` lines
4. Discard: All lines starting with `at` (preceded by whitespace)

---

## Search/Replace Patterns for Bug 4

**Pattern 1: Hint Messages**
```
BEFORE: "Add LIMIT clause"
AFTER:  "Use TOP N or SAMPLE N"
```

**Pattern 2: Example Queries**
```
BEFORE: SELECT * FROM table LIMIT 100
AFTER:  SELECT TOP 100 * FROM table
```

**Pattern 3: Comments**
```
BEFORE: "queries without an explicit LIMIT clause"
AFTER:  "queries without TOP or SAMPLE"
```

---

## Questions for User

Before implementation:

1. ✅ Is the error message format acceptable? (Simple, clean, professional)
2. ✅ Should we include database name in error output? (Not in current design)
3. ✅ Are error code and session ID sufficient for debugging?
4. ✅ Should verbose mode show the full stack trace? (Suggest: no, keep it clean)

---

## Resources

- **Full Design**: `sprint-9-bug-3-4-design.md`
- **Sprint Plan**: `sprint-9-planning.md`
- **Error Handling Spec**: `docs/builder/detailed-specifications/error-handling.md`
- **Current Code**: `src/error.rs`, `src/commands/repl/executor.rs`
