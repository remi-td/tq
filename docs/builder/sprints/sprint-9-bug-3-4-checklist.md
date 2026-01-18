# Sprint 9: Bug 3 & Bug 4 - Implementation Checklist

**Created:** 2026-01-18
**Purpose:** Quick checklist for implementation and validation

---

## Bug 3: Error Message Formatting

### Implementation Tasks

#### Phase 1: Add Error Parsing Helper
- [ ] Open `src/error.rs`
- [ ] Add `format_teradata_error()` helper method to `impl TqError`
- [ ] Implement session ID extraction: `[Session (\d+)]`
- [ ] Implement error code extraction: `[Error (\d+)]`
- [ ] Implement message extraction (after last `]`, before `at` lines)
- [ ] Implement stack trace filtering (remove lines starting with "at")
- [ ] Handle edge cases: missing session, missing error code, malformed errors

#### Phase 2: Update user_message()
- [ ] Update `TqError::SqlSyntaxError` variant in `user_message()`
- [ ] Update `TqError::QueryExecution` variant in `user_message()`
- [ ] Use new format: "Error: [type]\n\n[message]\n\nError Code: X\nSession ID: Y"
- [ ] Preserve existing behavior for non-SQL errors

#### Phase 3: Add Unit Tests
- [ ] Test parsing with real Teradata error format
- [ ] Test session ID extraction
- [ ] Test error code extraction
- [ ] Test message extraction
- [ ] Test stack trace suppression
- [ ] Test edge case: error without session
- [ ] Test edge case: error without error code
- [ ] Test edge case: malformed error (fallback to raw)

#### Phase 4: Manual Testing
- [ ] Build project: `cargo build`
- [ ] Start REPL: `./target/debug/tq repl`
- [ ] Generate syntax error: `SELCT * FROM table;`
- [ ] Verify: No stack traces visible
- [ ] Verify: Error code displayed
- [ ] Verify: Session ID displayed
- [ ] Generate table not found error: `SELECT * FROM nonexistent;`
- [ ] Generate permission error: `DROP TABLE protected;`
- [ ] Verify: All errors use clean format

---

## Bug 4: LIMIT Hint Message

### Code Changes

#### File: src/cli.rs
- [ ] Open `src/cli.rs`
- [ ] Find line ~271: Comment about LIMIT clause
- [ ] Update comment from "LIMIT clause" to "TOP or SAMPLE"

#### File: src/commands/repl/executor.rs
- [ ] Open `src/commands/repl/executor.rs`
- [ ] Find line ~87-88: Hint message
- [ ] Verify already says: "Use TOP N or SAMPLE N"
- [ ] ✅ Already fixed (confirmed by grep)

### Test Updates

#### File: tests/cases/TC064.md
- [ ] Open `tests/cases/TC064.md`
- [ ] Find lines 64, 69, 126, 136, 181, 192
- [ ] Update expected output to: "Use TOP N or SAMPLE N"
- [ ] Remove any "Add LIMIT clause" references

#### File: tests/cases/TC065.md
- [ ] Open `tests/cases/TC065.md`
- [ ] Review lines 48, 56, 152, 202
- [ ] Verify these test for ABSENCE of LIMIT references
- [ ] Ensure tests still pass with updated messages

### Documentation Updates

#### File: Readme.md
- [ ] Search for "LIMIT" references
- [ ] Replace example queries:
  - `SELECT * FROM table LIMIT 100` → `SELECT TOP 100 * FROM table`
- [ ] Update any mentions of LIMIT clause

#### File: docs/builder/specifications.md
- [ ] Search for "LIMIT" references
- [ ] Update for consistency with Teradata syntax
- [ ] Replace with TOP/SAMPLE examples

#### File: docs/builder/detailed-specifications/batch-mode.md
- [ ] Search for "LIMIT" references
- [ ] Update examples to use TOP/SAMPLE if found

#### File: docs/builder/detailed-specifications/cli-interface.md
- [ ] Search for "LIMIT" references
- [ ] Update examples to use TOP/SAMPLE if found

#### File: docs/builder/detailed-specifications/user-personas.md
- [ ] Search for "LIMIT" references
- [ ] Update query examples to use TOP/SAMPLE if found

### Verification

#### Grep All Files
- [ ] Run: `grep -r "LIMIT" src/ docs/ tests/ Readme.md`
- [ ] Review each result:
  - User-facing message? → MUST fix
  - Code comment? → Should fix
  - Internal logic? → OK to leave
  - Historical doc (BUG-ANALYSIS.md)? → OK to leave
- [ ] Verify no user sees "LIMIT clause" in normal usage

---

## Testing Validation

### Bug 3 Validation

#### Automated Tests
- [ ] Run unit tests: `cargo test`
- [ ] Verify all error parsing tests pass
- [ ] Verify no regressions in error handling

#### Manual REPL Testing
- [ ] Start REPL: `./target/debug/tq repl`
- [ ] Test 1: `SELCT * FROM table;`
  - [ ] Error is clean (no stack traces)
  - [ ] Error code visible
  - [ ] Session ID visible
- [ ] Test 2: `SELECT * FROM nonexistent;`
  - [ ] Error is clean
  - [ ] Correct error type shown
- [ ] Test 3: `SELECT * FROM DBC.;`
  - [ ] Error is clean
  - [ ] Message is clear
- [ ] Test 4: `DROP TABLE protected_table;`
  - [ ] Permission error is clean
  - [ ] Error code and session shown

#### User Validation
- [ ] User reviews error messages in real REPL
- [ ] User confirms messages are professional
- [ ] User confirms messages are helpful
- [ ] User confirms NO stack traces visible
- [ ] User approves Bug 3 fix

### Bug 4 Validation

#### Automated Tests
- [ ] Run tests: `cargo test`
- [ ] Verify TC064 passes with new expected output
- [ ] Verify TC065 passes (no LIMIT references)

#### Manual REPL Testing
- [ ] Start REPL: `./target/debug/tq repl`
- [ ] Run: `SELECT * FROM large_table;` (100+ rows)
- [ ] Verify hint says: "Use TOP N or SAMPLE N"
- [ ] Verify NO mention of "LIMIT clause"

#### Grep Verification
- [ ] Run: `grep -r "LIMIT" src/ docs/ tests/ Readme.md`
- [ ] Verify results:
  - [ ] NO user-facing messages with "LIMIT clause"
  - [ ] Examples use TOP/SAMPLE
  - [ ] Only internal/historical references remain

#### User Validation
- [ ] User reviews hint message
- [ ] User confirms syntax is correct for Teradata
- [ ] User confirms message is clear
- [ ] User approves Bug 4 fix

---

## Completion Criteria

### Bug 3: Error Message Formatting
- [ ] Code changes complete in `src/error.rs`
- [ ] Unit tests pass (100%)
- [ ] Manual testing complete
- [ ] NO stack traces in error output
- [ ] Error code and session ID displayed
- [ ] User validation: APPROVED

### Bug 4: LIMIT Hint Message
- [ ] Code changes complete (minimal)
- [ ] Test cases updated (TC064, TC065)
- [ ] Documentation updated
- [ ] Grep verification: PASSED
- [ ] Manual testing complete
- [ ] User validation: APPROVED

---

## Final Checks

### Code Quality
- [ ] Run: `cargo build` → Zero warnings
- [ ] Run: `cargo test` → 100% pass rate
- [ ] Run: `cargo clippy` → No issues

### Documentation
- [ ] Error handling spec updated (`docs/builder/detailed-specifications/error-handling.md`)
- [ ] Design document complete (`sprint-9-bug-3-4-design.md`)
- [ ] Implementation summary available (`sprint-9-bug-3-4-summary.md`)

### User Approval
- [ ] User tested Bug 3 fix in REPL
- [ ] User tested Bug 4 fix in REPL
- [ ] User explicitly approves both fixes
- [ ] No regressions detected
- [ ] Ready to move to next bug

---

## Status Tracking

| Task | Status | Notes |
|------|--------|-------|
| Bug 3: Design | ✅ COMPLETE | This document |
| Bug 3: Implementation | ⏳ PENDING | Waiting for rust-teradata-architect |
| Bug 3: Testing | ⏳ PENDING | After implementation |
| Bug 3: User Validation | ⏳ PENDING | After testing |
| Bug 4: Design | ✅ COMPLETE | This document |
| Bug 4: Implementation | ⏳ PENDING | Waiting for rust-teradata-architect |
| Bug 4: Testing | ⏳ PENDING | After implementation |
| Bug 4: User Validation | ⏳ PENDING | After testing |

---

## Notes for Implementer

**Key Points:**
1. Bug 3 is about FORMATTING, not changing error detection
2. Bug 4 is about MESSAGING, not changing functionality
3. Both are low-risk text/display fixes
4. Focus on clean, professional output
5. Teradata syntax correctness is critical for Bug 4

**Testing Strategy:**
- Bug 3: Generate various SQL errors, verify clean output
- Bug 4: Visual inspection + grep verification

**User Validation:**
- Critical for both bugs
- User must test in real REPL with live database
- Explicit approval required before moving to next bug

---

## Quick Reference

**Error Message Format (Bug 3):**
```
Error: [type]

[message]

Error Code: XXXX
Session ID: YYYY
```

**Hint Message (Bug 4):**
```
Showing first N rows. Use TOP N or SAMPLE N for different results.
```

**Stack Trace Suppression (Bug 3):**
- Remove ALL lines starting with "at gosqldriver/..."
- Remove ALL lines starting with "at database/sql..."
- Remove ALL lines starting with "at runtime..."
- Keep only: error type, message, error code, session ID

**Teradata Syntax (Bug 4):**
- ✅ TOP N: `SELECT TOP 100 * FROM table`
- ✅ SAMPLE N: `SELECT * FROM table SAMPLE 100`
- ❌ LIMIT: `SELECT * FROM table LIMIT 100` (PostgreSQL/MySQL, NOT Teradata)
