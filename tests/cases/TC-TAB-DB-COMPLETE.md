# TC-TAB-DB-COMPLETE - Database Completion with `dbc`

**Test ID:** TC-TAB-DB-COMPLETE
**Category:** Functionality (Interactive)
**Priority:** Critical (P0)
**Sprint:** 21
**Type:** Hybrid Test (Automated + Manual)
**Status:** Pending

---

## Context

**User Issue:** "If I do `sel * from `+TAB I get a list of many databases, it should contain all databases on the system, but I noticed that I am using the dbc one!!! Make sure all databases are included"

**Root Cause:** Metadata query may be filtering system databases or using incomplete system catalog view.

**Sprint 21 Feature:** Complete Database Metadata Fetching (P0)

---

## Objective

Verify that pressing TAB after "FROM " in the REPL:
1. Shows ALL databases on the Teradata system, including system database `dbc`
2. Database metadata query fetches complete system catalog
3. No filtering by user permissions during fetch (graceful degradation handled separately)

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env` with test credentials
- [ ] Test database has `dbc` system database accessible
- [ ] Terminal with interactive keyboard support

---

## Test Procedure

### Automated Component

#### Unit Tests (src/db/metadata.rs)

**Test 1: Database Query Uses Complete System Catalog**
```rust
#[test]
fn test_database_query_uses_complete_catalog() {
    // Verify query targets DBC.DatabasesV or equivalent
    // Verify query does NOT filter by permissions
    // Expected: Query text includes "DBC.DatabasesV" or "DBC.Databases"
}
```

**Test 2: Query Does Not Filter System Databases**
```rust
#[test]
fn test_database_query_no_system_filter() {
    // Verify query does not have WHERE clause filtering system databases
    // Expected: No filters like "WHERE DatabaseName NOT IN ('dbc', 'sys')"
}
```

#### Integration Tests (tests/integration_tests.rs) - Requires Database

**Test 3: Database Fetch Returns `dbc`**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_database_fetch_includes_dbc() {
    // Execute metadata fetch
    // Verify `dbc` in returned database list
    // Expected: databases.contains("dbc") == true
}
```

**Test 4: Database Fetch Returns Expected Count**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_database_fetch_count_complete() {
    // Execute metadata fetch
    // Verify count matches expected total (document expected count)
    // Expected: databases.len() >= expected_count
}
```

#### PTY Tests (tests/interactive_tests.rs) - Requires Database

**Test 5: Tab Completion Shows `dbc` in Output**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_tab_completion_shows_dbc() {
    // Spawn REPL
    // Type "SELECT * FROM "
    // Send TAB
    // Capture output
    // Verify output contains "dbc"
    // Expected: output.contains("dbc")
}
```

### Manual Component (PRIMARY VALIDATION)

**Manual Test Procedure:**

#### Step 1: Start REPL
```bash
./target/release/tq repl
```

Wait for connection and prompt: `tq>`

#### Step 2: Type SQL Fragment
**Action:** Type exactly (DO NOT press Enter):
```
SELECT * FROM
```
(note the space after FROM)

#### Step 3: Press TAB Key
**Action:** Press TAB once.

#### Step 4: Visual Inspection
**Observe the completion menu.**

**Expected Results:**
- Completion menu appears with database names
- `dbc` is VISIBLE in the list
- Menu displays correctly (not truncated, readable columns)
- NO error messages
- NO pager output

**Your Observation:**
- [ ] `dbc` visible in completion menu
- [ ] Menu displays correctly (columns aligned, readable)
- [ ] Other databases visible (demo_user, DemoNow_Monitor, etc.)
- [ ] No error messages
- [ ] No pager output

#### Step 5: Capture Screenshot
**Action:** Take screenshot showing completion menu with `dbc` visible.

**Screenshot Requirements:**
- Show prompt line with "SELECT * FROM "
- Show completion menu with database names
- Show `dbc` clearly visible
- Show entire visible terminal output

**Screenshot File:** Save as `tests/results/sprint-21/tc-tab-db-complete-screenshot.png`

---

## Expected Results

### Automated Component

**Unit Tests:**
- Database query uses `DBC.DatabasesV` or equivalent system view
- Query has no filters excluding system databases
- All tests PASS

**Integration Tests:**
- Database fetch returns `dbc` in list
- Database count matches expected total (>= 5 databases typical)
- All tests PASS

**PTY Tests:**
- Output text contains "dbc" string
- Output contains expected database names
- All tests PASS

### Manual Component

**Visual Validation:**
1. Completion menu appears after TAB
2. `dbc` database visible in menu
3. Menu rendering correct (columns, alignment, readability)
4. No error messages or pager output
5. Can navigate and select databases

### Anti-Patterns (MUST NOT Occur)

- ❌ `dbc` missing from completion menu
- ❌ Completion menu empty or incomplete
- ❌ Error messages during completion
- ❌ Pager output appears
- ❌ Menu truncated or unreadable

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________
**Database:** _______________
**tq Version:** _______________

### Automated Test Results

**Unit Tests:**
```
cargo test --lib metadata::test_database
# [Paste output]
```

**Integration Tests:**
```
cargo test --test integration_tests database_fetch -- --ignored
# [Paste output]
```

**PTY Tests:**
```
cargo test --test interactive_tests tab_completion_shows_dbc -- --ignored
# [Paste output]
```

**Automated Pass/Fail:**
- [ ] All automated tests PASSED
- [ ] Some automated tests FAILED (document below)

### Manual Test Results

**1. Did completion menu appear?**
- [ ] YES - Menu appeared
- [ ] NO - No menu (FAIL)

**2. Was `dbc` visible in menu?**
- [ ] YES - `dbc` visible (CORRECT)
- [ ] NO - `dbc` missing (FAIL)

**3. What databases were visible?**
```
[List databases visible in menu]
```

**4. Menu rendering quality:**
- [ ] Correct - readable, aligned, complete
- [ ] Issues - truncated, misaligned, confusing

**5. Screenshot captured:**
- [ ] Screenshot: `tests/results/sprint-21/tc-tab-db-complete-screenshot.png`

### Combined Verdict

**Verdict Logic:**
- APPROVED: Automated PASS + Manual PASS ✅
- REJECTED: Automated FAIL OR Manual FAIL ❌
- BLOCKED: Tests cannot execute (database unavailable) ⛔

**Final Verdict:**
- [ ] ✅ APPROVED - All tests pass, `dbc` visible in menu
- [ ] ❌ REJECTED - Tests fail or `dbc` missing
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

From `docs/sprints/sprint-21-planning.md` (Feature 1 Acceptance Criteria):
- [x] REQ-F1-1: System database `dbc` appears in database completion list
- [x] REQ-F1-2: ALL databases on Teradata system are fetched
- [x] REQ-F1-3: Query used to fetch databases returns complete system catalog

### Test Types Applied

| Requirement | Unit | Integration | PTY | Manual |
|-------------|------|-------------|-----|--------|
| REQ-F1-1    | ✅   | ✅          | ✅  | ✅     |
| REQ-F1-2    | ✅   | ✅          | ❌  | ✅     |
| REQ-F1-3    | ✅   | ❌          | ❌  | ❌     |

**Coverage Level:** Comprehensive (all requirements covered by multiple test types)

---

## Debugging Information

If `dbc` is missing, collect this information:

**Database Query Output:**
```sql
-- Run manually in Teradata:
SELECT DatabaseName FROM DBC.DatabasesV ORDER BY DatabaseName;
-- Does this return 'dbc'?
```

**Log Output:**
```bash
RUST_LOG=debug ./target/release/tq repl
# Type "SELECT * FROM " and press TAB
# [Paste log lines showing metadata query]
```

**Metadata Cache Contents:**
```bash
# In debug build, add logging to show cached databases
# [Paste cached database list]
```

---

## Risk Assessment

**False Positive Risk:** LOW

**Rationale:**
- Content-based validation (database names in text)
- PTY tests can reliably verify "dbc" string presence
- Integration tests prove database query works
- Manual validation confirms visual rendering

**Mitigation:**
- Manual validation REQUIRED to confirm menu displays correctly
- Screenshot evidence mandatory

---

## Related Tests

- **TC-TAB-TABLE-UNIVERSAL**: Universal table metadata fetching
- **TC-TAB-SMART-QUALIFIED**: Smart database.table completion
- **TC049**: Tab Completion - FROM Shows Databases (Sprint 8)

---

## References

- Planning: `docs/sprints/sprint-21-planning.md` (Feature 1, lines 48-68)
- Strategy: `tests/strategy/sprint-21-test-strategy.md` (Feature 1 analysis)
- Bug Report: `incoming/bugs-sprint-20.md` (lines 9-18)
- Specification: `docs/specifications/repl.md#database-completion`
- Design: `docs/design/repl.md#metadata-fetching`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test case for Sprint 21 Feature 1 | quality-validator |
