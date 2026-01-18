# Sprint 8 Regression Test Suite

**Purpose:** Prevent Sprint 8 bugs from recurring in future releases
**Created:** 2026-01-18
**Bugs Covered:** Table Padding, Tab Completion, Result Paging, LIMIT Hint

---

## Overview

This document provides a quick regression test suite that can be run in future sprints to ensure the Sprint 8 bug fixes remain effective. These tests should be executed:
- Before each release
- After any changes to table formatting, completion, or paging code
- As part of automated CI/CD (where possible)

---

## Quick Smoke Test (5 minutes)

Run these minimal tests to verify all 4 bugs remain fixed:

### Smoke Test 1: Table Padding

```sql
tq> SELECT TOP 3 * FROM DBC.DatabasesV;
```

**Check:** Column widths are reasonable (NOT 60+ chars for short data), headers align.

---

### Smoke Test 2: Tab Completion

```sql
tq> SELECT * FROM <TAB>
```

**Check:** Tab shows completion menu with databases and tables.

---

### Smoke Test 3: Result Paging

```sql
tq> SELECT TOP 100 * FROM DBC.TablesV;
```

**Check:** Pager activates, can use j/k to navigate, q to exit.

---

### Smoke Test 4: LIMIT Hint

```sql
tq> SELECT * FROM DBC.TablesV;
```

**Check:** Hint says "Use TOP N or SAMPLE N", NOT "LIMIT".

---

## Detailed Regression Tests (30 minutes)

### Regression Test R1: Table Formatting Regression

**Bug:** Table padding completely broken (excessive whitespace)

**Regression Command:**
```sql
SELECT TOP 5 * FROM DBC.DatabasesV;
```

**Expected Behavior:**
- Column widths are proportional to data (not uniform 60+ chars)
- Total table width ≤ terminal width + 20%
- Headers align with data columns
- Vertical separators form straight lines

**If This Fails:**
- Problem likely in `src/format/table.rs`
- Check `set_width` calculation (should NOT multiply by column count)
- Check `ContentArrangement` setting (should be Dynamic or DynamicFullWidth)

**Commit to Reference:** (fill in after fix)

---

### Regression Test R2: Tab Completion Regression

**Bug:** Tab completion silent failure (pressing Tab did nothing)

**Regression Commands:**
```sql
tq> SELECT * FROM <TAB>
[Should show databases and tables]

tq> SELECT * FROM DBC.<TAB>
[Should show tables in DBC]
```

**Expected Behavior:**
- Tab displays completion menu
- Shows databases and current database tables
- Database-qualified completion (DBC.<TAB>) shows DBC tables
- Loading indicator may appear on first Tab (acceptable)
- Errors are surfaced to user (not silent)

**If This Fails:**
- Problem likely in `src/commands/repl/metadata_completer.rs` or executor integration
- Check that metadata loading errors are displayed
- Check that completion results are passed to reedline
- Check lock acquisition doesn't silently fail

**Commit to Reference:** (fill in after fix)

---

### Regression Test R3: Result Paging Regression

**Bug:** Paging not integrated (j/k/arrow keys didn't work)

**Regression Commands:**
```sql
tq> SELECT TOP 100 * FROM DBC.TablesV;
[Pager should activate]
[Press 'j' to scroll down]
[Press 'k' to scroll up]
[Press 'q' to exit]
```

**Expected Behavior:**
- Pager activates for results > terminal height
- j/k keys scroll vertically
- h/l or arrows scroll horizontally (if table is wide)
- Status line shows position
- q or Esc exits cleanly
- `/pager off` disables paging

**If This Fails:**
- Problem likely in `src/commands/repl/executor.rs` - pager not called
- Check `state.is_pager_enabled()` is checked
- Check `PagedOutput::new()` is called for large results
- Check terminal mode switching (raw mode on/off)

**Commit to Reference:** (fill in after fix)

---

### Regression Test R4: LIMIT Hint Regression

**Bug:** Hint message said "LIMIT" (invalid Teradata syntax)

**Regression Command:**
```sql
SELECT * FROM DBC.TablesV;
```

**Expected Behavior:**
- Results are limited to 100 rows (default)
- Hint message says: "Use TOP N or SAMPLE N"
- NO mention of "LIMIT clause"
- Help text (`/help`) also uses Teradata syntax

**If This Fails:**
- Problem likely in `src/commands/repl/executor.rs` hint text
- Check TWO locations: `execute_sql` and `execute_sql_with_state`
- Check help text hasn't reverted to LIMIT

**Commit to Reference:** (fill in after fix)

---

## Automated Regression Tests

These tests can be scripted for CI/CD:

### Script: test-table-formatting.sh

```bash
#!/bin/bash
# Test table formatting regression

TQ="./target/release/tq"

# Execute query and capture output
OUTPUT=$($TQ query "SELECT TOP 3 * FROM DBC.DatabasesV")

# Check for excessive padding (column width > 100 chars indicates bug)
if echo "$OUTPUT" | grep -q "│.\{100,\}"; then
    echo "FAIL: Excessive column padding detected"
    exit 1
fi

# Check headers are present
if ! echo "$OUTPUT" | grep -q "DatabaseName"; then
    echo "FAIL: Headers missing or malformed"
    exit 1
fi

echo "PASS: Table formatting OK"
```

---

### Script: test-limit-hint.sh

```bash
#!/bin/bash
# Test LIMIT hint regression

TQ="./target/release/tq"

# Query with >100 rows (triggers hint)
OUTPUT=$($TQ query "SELECT * FROM DBC.TablesV" 2>&1)

# Check for correct Teradata syntax
if echo "$OUTPUT" | grep -iq "TOP N or SAMPLE N"; then
    echo "PASS: Hint uses correct Teradata syntax"
elif echo "$OUTPUT" | grep -iq "LIMIT"; then
    echo "FAIL: Hint still says LIMIT (bug regression!)"
    exit 1
else
    echo "WARN: Hint message not found (query may have < 100 rows)"
fi
```

---

### Script: test-pager-integration.sh

```bash
#!/bin/bash
# Test pager integration (note: hard to automate fully)

TQ="./target/release/tq"

# This test verifies pager state is tracked
# Full interactive testing requires manual verification

# Check that /pager command exists
if $TQ query "/help" 2>&1 | grep -q "pager"; then
    echo "PASS: Pager command documented in help"
else
    echo "WARN: Pager command not in help"
fi

# Note: Full pager testing (j/k keys) requires interactive session
echo "Note: Full pager testing requires manual execution (see manual-testing-guide)"
```

---

## Regression Test Schedule

**Before Each Release:**
- Run Quick Smoke Test (5 min) - MANDATORY
- Run Detailed Regression Tests (30 min) - RECOMMENDED

**After Code Changes in These Areas:**
- Table formatting (`src/format/table.rs`) → Run R1
- Tab completion (`src/commands/repl/metadata_completer.rs`) → Run R2
- Paging (`src/commands/repl/pager.rs`, executor) → Run R3
- Hint messages or help text → Run R4

**Monthly:**
- Full manual test suite execution (use manual-testing-guide-sprint-8.md)

---

## Regression Indicators

**Signs That Bug 1 Has Regressed:**
- Table output is very wide (>500 chars)
- Headers don't align with data
- Complaints about "unreadable tables"

**Signs That Bug 2 Has Regressed:**
- Users report "Tab doesn't work"
- No completion suggestions appear
- Metadata errors are silent

**Signs That Bug 3 Has Regressed:**
- All results dump to terminal (no paging)
- Users report "j/k keys don't work"
- `/pager off` mentioned but has no effect

**Signs That Bug 4 Has Regressed:**
- Users try `LIMIT` and get syntax errors
- Hint message mentions "LIMIT clause"
- Documentation uses MySQL/PostgreSQL syntax

---

## Bug Reproduction (For Debugging)

If a regression is suspected, reproduce the ORIGINAL bug to confirm:

### Reproduce Original Bug 1 (Table Padding)

Check out pre-fix commit, build, and run:
```sql
SELECT TOP 3 * FROM DBC.DatabasesV;
```

Original bug: Columns are 60+ characters wide, total table width 960+ chars.

---

### Reproduce Original Bug 2 (Tab Completion)

Check out pre-fix commit, build, and run:
```sql
tq> SELECT * FROM <TAB>
```

Original bug: Nothing happens (no completion menu, no feedback).

---

### Reproduce Original Bug 3 (Paging)

Check out pre-fix commit, build, and run:
```sql
tq> SELECT TOP 100 * FROM DBC.TablesV;
```

Original bug: All 100 rows dump immediately, j/k keys don't work.

---

### Reproduce Original Bug 4 (LIMIT Hint)

Check out pre-fix commit, build, and run:
```sql
tq> SELECT * FROM DBC.TablesV;
```

Original bug: Hint says "Add LIMIT clause" (invalid Teradata syntax).

---

## Performance Benchmarks

These performance targets should be maintained post-fix:

| Operation | Target | Notes |
|-----------|--------|-------|
| Table formatting (16 cols) | <2s | Should not hang or delay |
| Tab completion (cached) | <50ms | Instant response |
| Tab completion (first load) | <500ms | With loading indicator |
| Pager activation | <200ms | Enter interactive mode quickly |
| Pager navigation (j/k) | <100ms | Responsive scrolling |

If any operation exceeds 2x target, investigate for regression.

---

## Test Data Requirements

For consistent regression testing, ensure test environment has:

**Required:**
- DBC.DatabasesV access (for wide table testing)
- DBC.TablesV access (for paging and hint testing)
- At least 3 databases in system (for tab completion)
- Current database has 5+ tables (for tab completion)

**Optional:**
- Permission to CREATE/DROP tables (for cache invalidation tests)
- Multiple user databases (for database switching tests)

---

## Integration with CI/CD

**Automated Tests (CI):**
- Run `test-table-formatting.sh`
- Run `test-limit-hint.sh`
- Run unit tests: `cargo test`

**Manual Tests (Pre-Release):**
- Run full manual-testing-guide-sprint-8.md
- Verify all 22 test cases pass
- Sign off on Sprint 8 regression testing

---

## Reporting Regressions

If a Sprint 8 bug regresses:

1. **Identify:** Which of the 4 bugs has returned?
2. **Reproduce:** Confirm bug using reproduction steps above
3. **Git Bisect:** Find commit that reintroduced bug
4. **Document:** Create new bug report referencing Sprint 8 fix
5. **Fix:** Apply fix and re-run full Sprint 8 test suite
6. **Prevent:** Update regression tests to catch this scenario

---

## References

- **Test Cases:** `tests/cases/TC044-TC065.md`
- **Manual Guide:** `tests/manual-testing-guide-sprint-8.md`
- **Root Cause Analysis:** `docs/builder/sprints/sprint-8-root-cause-analysis.md`
- **UX Design:** `docs/builder/sprints/sprint-8-ux-design.md`
- **Sprint Review:** `docs/builder/sprints/sprint-8-review.md` (to be created)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-18
**Maintained By:** quality-validator agent
