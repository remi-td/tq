# TC-TAB-SMART-QUALIFIED - Smart Database-Dot-TAB Completion

**Test ID:** TC-TAB-SMART-QUALIFIED
**Category:** Functionality (Interactive)
**Priority:** High (P1)
**Sprint:** 21
**Type:** Hybrid Test (Automated + Manual)
**Status:** Pending

---

## Context

**User Issue:** "When I hit tab on a database after a FROM/JOIN, I would expect to complete the database name, add a '.' and prompt the list of tables in this database directly."

**Expected Behavior:** Typing partial database name (`dem`) + TAB should complete to `demo_user.` AND immediately show tables in that database.

**Sprint 21 Feature:** Smart Database-Dot-TAB Completion (P1)

---

## Objective

Verify multi-stage completion logic:
1. Typing unambiguous database prefix (`dem`) + TAB completes to `demo_user.` (with dot)
2. After completing database name, immediately shows tables in that database
3. If ambiguous (multiple matches like `de`), shows database list first
4. Works after FROM and JOIN keywords
5. UX feels smooth and immediate

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env`
- [ ] `demo_user` database exists (unambiguous when typing `dem`)
- [ ] Another database starting with `de` exists for ambiguity test
- [ ] Terminal with interactive keyboard support

---

## Test Procedure

### Automated Component

#### Unit Tests (src/commands/repl/metadata_completer.rs)

**Test 1: Unambiguous Prefix Completes with Dot**
```rust
#[test]
fn test_unambiguous_database_completion() {
    // Input: "dem" (matches only "demo_user")
    // Expected: Returns "demo_user." with dot appended
    // State: Triggers table fetch for demo_user
}
```

**Test 2: Ambiguous Prefix Shows Database List**
```rust
#[test]
fn test_ambiguous_database_completion() {
    // Input: "de" (matches "demo_user", "demo_admin")
    // Expected: Returns database list only (no tables)
    // State: Waits for user to select specific database
}
```

**Test 3: FROM Context Detection**
```rust
#[test]
fn test_from_context_enables_database_completion() {
    // Input: "SELECT * FROM dem"
    // Expected: Context = FROM, database completion enabled
    // Verify: Completer recognizes FROM keyword
}
```

**Test 4: JOIN Context Detection**
```rust
#[test]
fn test_join_context_enables_database_completion() {
    // Input: "SELECT * FROM t1 JOIN dem"
    // Expected: Context = JOIN, database completion enabled
    // Verify: Completer recognizes JOIN keyword
}
```

#### Integration Tests (tests/integration_tests.rs) - Requires Database

**Test 5: Complete Unambiguous Database and Fetch Tables**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_smart_completion_demo_user() {
    // Execute completion for "dem"
    // Verify returns "demo_user." + table list
    // Expected: completion.text == "demo_user.", completion.suggestions.len() > 0
}
```

**Test 6: Ambiguous Database Returns List Only**
```rust
#[test]
#[ignore] // Requires TQ_LOGON
fn test_ambiguous_completion_de() {
    // Execute completion for "de"
    // Verify returns database list only (no tables)
    // Expected: completion.suggestions = [databases], no table names
}
```

#### PTY Tests (tests/interactive_tests.rs) - Requires Database

**Test 7: Smart Completion After FROM**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_smart_completion_from_dem() {
    // Spawn REPL
    // Type "SELECT * FROM dem"
    // Send TAB
    // Capture output
    // Verify output contains "demo_user." AND table names
    // Expected: output.contains("demo_user.") && output.contains(expected_table)
}
```

**Test 8: Smart Completion After JOIN**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_smart_completion_join_dem() {
    // Spawn REPL
    // Type "SELECT * FROM t1 JOIN dem"
    // Send TAB
    // Capture output
    // Verify output contains "demo_user." AND table names
    // Expected: output.contains("demo_user.") && output.contains(expected_table)
}
```

**Test 9: Ambiguous Prefix Shows Databases First**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_ambiguous_completion_de() {
    // Spawn REPL
    // Type "SELECT * FROM de"
    // Send TAB
    // Capture output
    // Verify output contains database names (demo_user, demo_admin)
    // Verify output does NOT contain table names yet
    // Expected: output.contains("demo_user") && !output.contains(known_table_name)
}
```

**Test 10: No Error Messages**
```rust
#[test]
#[ignore] // Requires TQ_LOGON and PTY
fn test_smart_completion_no_errors() {
    // Spawn REPL
    // Type "SELECT * FROM dem"
    // Send TAB
    // Capture output
    // Verify no error messages
    // Expected: !output.contains("ERROR") && !output.contains("NO RECORDS")
}
```

### Manual Component (UX VALIDATION)

**Manual Test Procedure:**

#### Test 1: Unambiguous Database Completion (FROM context)

**Step 1:** Start REPL
```bash
./target/release/tq repl
```

**Step 2:** Type partial database name after FROM
```
SELECT * FROM dem
```
(DO NOT press Enter)

**Step 3:** Press TAB once

**Step 4:** Observe behavior

**Expected:**
- Text completes to `SELECT * FROM demo_user.`
- Table list appears immediately after dot
- Transition feels smooth and immediate (<500ms perceived)
- No error messages

**Your Observation:**
- [ ] Text completed to `demo_user.` (with dot)
- [ ] Tables appeared immediately
- [ ] Transition smooth (<500ms perceived)
- [ ] No error messages
- [ ] Latency acceptable

**Screenshot:** `tests/results/sprint-21/tc-smart-from-dem-screenshot.png`

---

#### Test 2: Unambiguous Database Completion (JOIN context)

**Step 1:** Clear line (Ctrl-U)

**Step 2:** Type
```
SELECT * FROM t1 JOIN dem
```

**Step 3:** Press TAB

**Step 4:** Observe behavior

**Expected:**
- Text completes to `demo_user.`
- Tables appear
- Works same as FROM context

**Your Observation:**
- [ ] Completes to `demo_user.` with tables
- [ ] Consistent with FROM behavior

---

#### Test 3: Ambiguous Database Prefix

**Step 1:** Clear line (Ctrl-U)

**Step 2:** Type
```
SELECT * FROM de
```
(Assuming `de` matches multiple databases like `demo_user`, `demo_admin`)

**Step 3:** Press TAB

**Step 4:** Observe behavior

**Expected:**
- Shows database list (demo_user, demo_admin, etc.)
- Does NOT show tables yet
- User can select database

**Step 5:** Select `demo_user` from list (use arrows + Enter or TAB)

**Step 6:** Press TAB again (or completes automatically to `demo_user.`)

**Step 7:** Observe tables appear

**Your Observation:**
- [ ] First TAB showed database list only
- [ ] Selected database, then saw tables
- [ ] Two-stage process worked correctly

**Screenshot:** `tests/results/sprint-21/tc-smart-ambiguous-de-screenshot.png`

---

#### Test 4: UX Smoothness Assessment

**Subjective Evaluation:**

Rate the following on scale 1-5 (1=Poor, 5=Excellent):

**Immediacy (tables appear quickly):**
- [ ] 1 - Very slow (>2s)
- [ ] 2 - Slow (1-2s)
- [ ] 3 - Acceptable (500ms-1s)
- [ ] 4 - Fast (200-500ms)
- [ ] 5 - Immediate (<200ms)

**Intuitiveness (behavior matches expectation):**
- [ ] 1 - Confusing, unexpected
- [ ] 2 - Somewhat unintuitive
- [ ] 3 - Acceptable
- [ ] 4 - Intuitive
- [ ] 5 - Perfectly intuitive

**Smoothness (no jank, flicker, or delay):**
- [ ] 1 - Janky, visible rendering issues
- [ ] 2 - Some flicker or delay
- [ ] 3 - Acceptable
- [ ] 4 - Smooth
- [ ] 5 - Perfectly smooth

**Overall UX:**
```
[Describe your overall impression of the smart completion UX]
```

---

## Expected Results

### Automated Component

**Unit Tests:**
- Unambiguous prefix returns database + dot
- Ambiguous prefix returns database list only
- FROM and JOIN context detection works
- All tests PASS

**Integration Tests:**
- `dem` completion returns `demo_user.` + tables
- `de` completion returns database list only
- All tests PASS

**PTY Tests:**
- Output contains `demo_user.` + table names for `dem`
- Works after FROM and JOIN
- Ambiguous prefix shows databases only
- No error messages
- All tests PASS

### Manual Component

**UX Validation:**
1. Unambiguous prefix completes database + shows tables in one TAB
2. Works after FROM and JOIN keywords
3. Ambiguous prefix shows database list first, then tables after selection
4. Latency acceptable (<500ms perceived)
5. UX smooth and intuitive (user ratings 4-5)

### Anti-Patterns (MUST NOT Occur)

- ❌ Database completes but tables don't appear
- ❌ Long delay between completion and table display (>2s)
- ❌ Choppy or janky rendering
- ❌ Error messages during completion
- ❌ Database doesn't complete (just shows list)
- ❌ Context detection fails (doesn't work after JOIN)

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________
**Database:** _______________
**tq Version:** _______________
**Databases starting with `de`:** _______________

### Automated Test Results

**Unit Tests:**
```
cargo test --lib metadata_completer::test_smart
# [Paste output]
```

**Integration Tests:**
```
cargo test --test integration_tests smart_completion -- --ignored
# [Paste output]
```

**PTY Tests:**
```
cargo test --test interactive_tests smart_completion -- --ignored
# [Paste output]
```

**Automated Pass/Fail:**
- [ ] All automated tests PASSED
- [ ] Some automated tests FAILED (document below)

### Manual Test Results

**Test 1: Unambiguous FROM completion**
- [ ] PASS - Completes to `demo_user.` with tables
- [ ] FAIL - Issues: _______________

**Test 2: Unambiguous JOIN completion**
- [ ] PASS - Works like FROM
- [ ] FAIL - Issues: _______________

**Test 3: Ambiguous prefix handling**
- [ ] PASS - Shows databases first, then tables
- [ ] FAIL - Issues: _______________

**Test 4: UX Quality**
- Immediacy: ___/5
- Intuitiveness: ___/5
- Smoothness: ___/5
- Overall: _______________

**Screenshots captured:**
- [ ] `tests/results/sprint-21/tc-smart-from-dem-screenshot.png`
- [ ] `tests/results/sprint-21/tc-smart-ambiguous-de-screenshot.png`

### Combined Verdict

**Verdict Logic:**
- APPROVED: Automated PASS + Manual PASS (UX ratings 4-5) ✅
- REJECTED: Automated FAIL OR Manual FAIL (UX ratings 1-2) ❌
- BLOCKED: Tests cannot execute ⛔

**Final Verdict:**
- [ ] ✅ APPROVED - All tests pass, UX smooth and intuitive
- [ ] ❌ REJECTED - Tests fail or UX poor
- [ ] ⛔ BLOCKED - Cannot execute tests

**Failure Details (if REJECTED):**
```
[Describe specific failures]
```

---

## Coverage Analysis

### Requirements Validated

From `docs/sprints/sprint-21-planning.md` (Feature 4 Acceptance Criteria):
- [x] REQ-F4-1: Typing `dem` + TAB completes to `demo_user.` (if unambiguous)
- [x] REQ-F4-2: After completing, immediately show tables
- [x] REQ-F4-3: If ambiguous, show database list first
- [x] REQ-F4-4: Works after FROM keyword
- [x] REQ-F4-5: Works after JOIN keyword

### Test Types Applied

| Requirement | Unit | Integration | PTY | Manual |
|-------------|------|-------------|-----|--------|
| REQ-F4-1    | ✅   | ✅          | ✅  | ✅     |
| REQ-F4-2    | ❌   | ✅          | ✅  | ✅     |
| REQ-F4-3    | ✅   | ✅          | ✅  | ✅     |
| REQ-F4-4    | ✅   | ❌          | ✅  | ✅     |
| REQ-F4-5    | ✅   | ❌          | ✅  | ✅     |

**Coverage Level:** Comprehensive (all requirements covered by multiple test types)

---

## Debugging Information

If smart completion doesn't work:

**Check Database Name:**
```sql
-- Verify demo_user exists:
SELECT DatabaseName FROM DBC.DatabasesV
WHERE DatabaseName LIKE 'dem%';
```

**Check Ambiguity:**
```sql
-- How many databases start with 'de'?
SELECT COUNT(*) FROM DBC.DatabasesV
WHERE DatabaseName LIKE 'de%';
```

**Log Output:**
```bash
RUST_LOG=debug ./target/release/tq repl
# Type "SELECT * FROM dem" and press TAB
# [Paste log lines showing completion logic]
```

---

## Risk Assessment

**False Positive Risk:** MEDIUM

**Rationale:**
- Content-based validation CAN verify database + tables appear
- BUT UX smoothness ("immediate") NOT testable with automation
- Multi-stage completion state management complex
- Automated tests validate logic, not UX quality

**Mitigation:**
- Manual validation confirms UX smoothness and latency
- Unit tests validate logic, reducing integration bug risk
- User ratings provide subjective UX assessment

---

## Related Tests

- **TC-TAB-DB-COMPLETE**: Database completion with dbc
- **TC-TAB-TABLE-UNIVERSAL**: Universal table metadata fetching
- **TC050**: Tab Completion - FROM database.TAB Shows Tables (Sprint 8)

---

## References

- Planning: `docs/sprints/sprint-21-planning.md` (Feature 4, lines 119-141)
- Strategy: `tests/strategy/sprint-21-test-strategy.md` (Feature 4 analysis)
- Bug Report: `incoming/bugs-sprint-20.md` (lines 20-23)
- Specification: `docs/specifications/repl.md#qualified-name-completion`
- Design: `docs/design/repl.md#multi-stage-completion`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test case for Sprint 21 Feature 4 | quality-validator |
