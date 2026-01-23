# TC-TAB-TABLE-UNIVERSAL - Universal Table Metadata Fetching

**Test ID:** TC-TAB-TABLE-UNIVERSAL
**Category:** Functionality (Interactive)
**Priority:** Critical (P0)
**Sprint:** 21
**Type:** Hybrid Test (Automated + Manual)
**Status:** Pending

---

## Context

**User Issue:** "Some databases objects are not cached/fetched. For example: `tq> | sel * from demo_user.` → NO RECORDS FOUND. I know that there are three tables in this database, but it should be fetched!"

**Root Cause:** Table metadata loading may be limited to specific databases or failing silently for some databases.

**Sprint 21 Feature:** Universal Table Metadata Fetching (P0)

---

## Objective

Verify that table metadata is fetched for ALL databases, not just a subset:
1. Typing `database.` + TAB shows tables in that database
2. `demo_user` database tables appear (NOT "NO RECORDS FOUND")
3. Graceful degradation if permission denied for specific database
4. Error handling provides useful feedback

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env`
- [ ] `demo_user` database exists with at least 1 table
- [ ] Terminal with interactive keyboard support

---

## Test Procedure

### Automated Component

#### Unit Tests (src/commands/repl/metadata_completer.rs)

**Test 1: Qualified Name Parsing**
```rust
#[test]
fn test_qualified_name_parsing() {
    // Input: "demo_user."
    // Expected: database="demo_user", prefix=""
    // Verify completer recognizes qualified name context
}
```

**Test 2: Table Query SQL**
```rust
#[test]
fn test_table_query_sql() {
    // Verify query targets DBC.TablesV WHERE DatabaseName = ?
    // Verify query is parameterized by database name
    // Expected: Correct SQL syntax for per-database table fetch
}
```

#### Integration Tests (tests/integration_tests.rs) - Requires Database

**Test 3: Fetch Tables for demo_user**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_fetch_tables_demo_user() {
    // Execute table fetch for "demo_user" database
    // Verify expected table names returned
    // Expected: tables.len() > 0, specific table names present
    // Document expected tables in test
}
```

**Test 4: Graceful Permission Denied Handling**
```rust
#[test]
#[ignore] // Requires TQ_LOGON with restricted database
fn test_fetch_tables_permission_denied() {
    // Attempt table fetch for restricted database
    // Verify no panic, graceful error
    // Expected: Error or empty list, no crash
}
```

**Test 5: Non-Existent Database**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_fetch_tables_nonexistent_database() {
    // Attempt table fetch for non-existent database
    // Verify empty result (not error)
    // Expected: tables.is_empty() == true, no error message
}
```

#### PTY Tests (tests/interactive_tests.rs) - Requires Database

**Test 6: Tab Completion Shows demo_user Tables**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_tab_completion_shows_demo_user_tables() {
    // Spawn REPL
    // Type "SELECT * FROM demo_user."
    // Send TAB
    // Capture output
    // Verify output contains expected table names
    // Verify output does NOT contain "NO RECORDS FOUND"
    // Expected: output.contains(expected_table_name)
}
```

**Test 7: Negative Test - NO RECORDS FOUND Not Present**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_tab_completion_no_error_message() {
    // Spawn REPL
    // Type "SELECT * FROM demo_user."
    // Send TAB
    // Capture output
    // Verify "NO RECORDS FOUND" does NOT appear
    // Expected: !output.contains("NO RECORDS FOUND")
}
```

### Manual Component (PRIMARY VALIDATION)

**Manual Test Procedure:**

#### Step 1: Start REPL
```bash
./target/release/tq repl
```

Wait for connection and prompt: `tq>`

#### Step 2: Type SQL with Qualified Name
**Action:** Type exactly (DO NOT press Enter):
```
SELECT * FROM demo_user.
```
(note the dot after demo_user)

#### Step 3: Press TAB Key
**Action:** Press TAB once.

#### Step 4: Visual Inspection
**Observe the output.**

**Expected Results:**
- Completion menu appears with table names from `demo_user` database
- Expected tables visible (document expected table names)
- NO "NO RECORDS FOUND" error message
- Completion latency acceptable (<2 seconds)
- Menu displays correctly

**Your Observation:**
- [ ] Table names visible in completion menu
- [ ] Expected tables present (list them: _____________)
- [ ] NO "NO RECORDS FOUND" error
- [ ] Latency acceptable (<2s)
- [ ] Menu displays correctly

#### Step 5: Verify Other Databases Work
**Action:** Try another database:
```
SELECT * FROM modelops.
```
Press TAB. Verify tables appear.

**Your Observation:**
- [ ] Other databases also show tables
- [ ] Consistent behavior across databases

#### Step 6: Capture Screenshots
**Action:** Take screenshots for both tests.

**Screenshot Files:**
- `tests/results/sprint-21/tc-tab-table-demo-user-screenshot.png`
- `tests/results/sprint-21/tc-tab-table-modelops-screenshot.png`

---

## Expected Results

### Automated Component

**Unit Tests:**
- Qualified name parsing correct ("demo_user." → database="demo_user")
- Table query SQL correct (DBC.TablesV WHERE DatabaseName = ?)
- All tests PASS

**Integration Tests:**
- Table fetch for demo_user returns expected tables
- Permission denied handled gracefully (no crash)
- Non-existent database returns empty list (no error)
- All tests PASS

**PTY Tests:**
- Output contains expected table names for demo_user
- Output does NOT contain "NO RECORDS FOUND"
- All tests PASS

### Manual Component

**Visual Validation:**
1. Completion menu appears after `demo_user.` + TAB
2. Tables from demo_user visible in menu
3. NO "NO RECORDS FOUND" error message
4. Completion latency acceptable (<2s)
5. Menu rendering correct
6. Works consistently for other databases

### Anti-Patterns (MUST NOT Occur)

- ❌ "NO RECORDS FOUND" error message
- ❌ Empty completion menu (when tables exist)
- ❌ Completion hang or timeout
- ❌ Crash on permission denied
- ❌ Menu truncated or unreadable

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________
**Database:** _______________
**tq Version:** _______________
**Expected Tables in demo_user:** _______________

### Automated Test Results

**Unit Tests:**
```
cargo test --lib metadata_completer::test_qualified
# [Paste output]
```

**Integration Tests:**
```
cargo test --test integration_tests fetch_tables -- --ignored
# [Paste output]
```

**PTY Tests:**
```
cargo test --test interactive_tests demo_user_tables -- --ignored
# [Paste output]
```

**Automated Pass/Fail:**
- [ ] All automated tests PASSED
- [ ] Some automated tests FAILED (document below)

### Manual Test Results

**1. Did completion menu appear for demo_user?**
- [ ] YES - Menu appeared
- [ ] NO - No menu or error (FAIL)

**2. What tables were visible?**
```
[List tables visible in menu]
```

**3. Was "NO RECORDS FOUND" error present?**
- [ ] NO - Error NOT present (CORRECT)
- [ ] YES - Error present (FAIL)

**4. Completion latency:**
```
[Estimate time from TAB to menu display]
```
- [ ] Acceptable (<2s)
- [ ] Too slow (>2s)

**5. Did other databases work (modelops)?**
- [ ] YES - Consistent behavior
- [ ] NO - Inconsistent

**6. Screenshots captured:**
- [ ] Screenshot: `tests/results/sprint-21/tc-tab-table-demo-user-screenshot.png`
- [ ] Screenshot: `tests/results/sprint-21/tc-tab-table-modelops-screenshot.png`

### Combined Verdict

**Verdict Logic:**
- APPROVED: Automated PASS + Manual PASS ✅
- REJECTED: Automated FAIL OR Manual FAIL ❌
- BLOCKED: Tests cannot execute (database unavailable) ⛔

**Final Verdict:**
- [ ] ✅ APPROVED - All tests pass, tables visible, no error
- [ ] ❌ REJECTED - Tests fail or "NO RECORDS FOUND" error
- [ ] ⛔ BLOCKED - Cannot execute tests

**Failure Details (if REJECTED):**
```
[Describe specific failures]
```

**Blocker Details (if BLOCKED):**
```
[Describe blockers]
```

---

## Coverage Analysis

### Requirements Validated

From `docs/sprints/sprint-21-planning.md` (Feature 2 Acceptance Criteria):
- [x] REQ-F2-1: Metadata fetch attempts to load tables for ALL databases
- [x] REQ-F2-2: `demo_user` database tables appear in completion
- [x] REQ-F2-3: Completion shows tables after typing `database.` + TAB
- [x] REQ-F2-4: Error handling: graceful degradation if permission denied

### Test Types Applied

| Requirement | Unit | Integration | PTY | Manual |
|-------------|------|-------------|-----|--------|
| REQ-F2-1    | ✅   | ✅          | ❌  | ✅     |
| REQ-F2-2    | ❌   | ✅          | ✅  | ✅     |
| REQ-F2-3    | ❌   | ✅          | ✅  | ✅     |
| REQ-F2-4    | ❌   | ✅          | ✅  | ❌     |

**Coverage Level:** Comprehensive (all requirements covered by multiple test types)

---

## Debugging Information

If "NO RECORDS FOUND" appears, collect:

**Manual Table Query:**
```sql
-- Run manually in Teradata:
SELECT TableName FROM DBC.TablesV
WHERE DatabaseName = 'demo_user'
ORDER BY TableName;
-- What tables does this return?
```

**Log Output:**
```bash
RUST_LOG=debug ./target/release/tq repl
# Type "SELECT * FROM demo_user." and press TAB
# [Paste log lines showing table metadata query]
```

**Check Database Access:**
```sql
-- Verify you have access:
SELECT * FROM demo_user.some_table SAMPLE 1;
-- Does this work?
```

---

## Risk Assessment

**False Positive Risk:** MEDIUM

**Rationale:**
- Content-based validation CAN verify table names appear
- BUT negative assertion ("NO RECORDS FOUND" must NOT appear) harder to prove
- Graceful degradation testing (permission denied) complex
- PTY tests cannot validate UX latency

**Mitigation:**
- Manual validation confirms error handling UX
- PTY negative test explicitly searches for error strings
- Document expected tables in test for verification

---

## Related Tests

- **TC-TAB-DB-COMPLETE**: Database completion with dbc
- **TC-TAB-SMART-QUALIFIED**: Smart database.table completion
- **TC050**: Tab Completion - FROM database.TAB Shows Tables (Sprint 8)

---

## References

- Planning: `docs/sprints/sprint-21-planning.md` (Feature 2, lines 70-90)
- Strategy: `tests/strategy/sprint-21-test-strategy.md` (Feature 2 analysis)
- Bug Report: `incoming/bugs-sprint-20.md` (lines 25-34)
- Specification: `docs/specifications/repl.md#table-completion`
- Design: `docs/design/repl.md#table-metadata-fetching`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test case for Sprint 21 Feature 2 | quality-validator |
