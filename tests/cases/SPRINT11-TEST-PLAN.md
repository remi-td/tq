# Sprint 11 Test Plan: Critical Quality Recovery

**Sprint**: 11
**Date**: 2026-01-18
**Commit**: a1c02cd487add7080519760ce07a22383dcbf1e2
**Author**: quality-validator agent

---

## Test Plan Overview

This document indexes all test cases for Sprint 11, which focuses on fixing two critical bugs:

1. **Bug 2 (P0)**: Tab completion showing SQL keywords instead of database objects
2. **Bug 1 (P0)**: Table display broken with excessive padding

**Testing Philosophy:**

Sprint 11 tests emphasize:
- **Semantic validation**: Content must be CORRECT, not just present
- **Visual validation**: Layout must be READABLE, not just rendered
- **Live database testing**: All REPL tests against real Teradata (MANDATORY)
- **Anti-pattern detection**: Explicitly test that bugs DON'T reoccur

---

## Test Categories

### 1. Tab Completion Tests (TC071-TC075)

Testing context-aware completion with live database.

| Test ID | Title | Priority | Focus |
|---------|-------|----------|-------|
| TC071 | Context-Aware Database/Table Completion After FROM | Critical | Core fix - shows objects, not keywords |
| TC072 | Qualified Name Completion (database.table) | High | Teradata-specific qualified naming |
| TC073 | Column Name Completion After SELECT/WHERE | High | Column context detection |
| TC074 | Multi-Line Context Preservation (Regression) | High | Sprint 9 fix still works |
| TC075 | Error Handling When Metadata Unavailable | Medium | Graceful degradation |

**Key Requirements:**
- ALL tests run with live Teradata database
- Verify completions show actual database objects (not keywords)
- Confirm "(SQL keyword)" string never appears
- Validate completions are queryable/usable
- Test context detection with real SQL

**Anti-Pattern Check:**
Each test must explicitly verify NO "(SQL keyword)" repeated output.

---

### 2. Table Display Tests (TC076-TC080)

Testing terminal width detection and column truncation.

| Test ID | Title | Priority | Focus |
|---------|-------|----------|-------|
| TC076 | Terminal Width Detection - 80 cols | Critical | Core fix - simple truncation |
| TC077 | Wide Terminal Handling - 120 cols | High | Dynamic width adjustment |
| TC078 | Very Wide Terminal Handling - 160 cols | Medium | Maximum columns shown |
| TC079 | Ultra-Wide Terminal Handling - 200+ cols | Low | Edge case |
| TC080 | Batch Mode vs TTY Mode Column Visibility | Critical | TTY truncates, batch shows all |

**Key Requirements:**
- Test multiple terminal widths: 80, 120, 160, 200+ cols
- Measure actual character widths (≤ terminal size)
- Verify "(+n cols)" indicator present when truncated
- Confirm "..." indicators in body rows
- Visual inspection MANDATORY (headers align with data)
- TTY mode truncates, batch mode shows all columns

**Visual Validation:**
Each test includes:
- Width measurements
- Alignment checks
- Readability assessment
- Professional appearance verification

---

### 3. Regression Tests (TC081-TC082)

Ensuring Sprint 10 features and core REPL still work.

| Test ID | Title | Priority | Focus |
|---------|-------|----------|-------|
| TC081 | Batch Mode Features Still Work (Sprint 10) | High | No regression in batch mode |
| TC082 | REPL Core Features Still Work | High | Metacommands, multi-line, history |

**Key Requirements:**
- Re-run Sprint 10 test scenarios
- Verify all metacommands functional
- Confirm batch mode complete
- Check core REPL features intact

---

## Test Execution Strategy

### Phase 1: Tab Completion Validation (TC071-TC075)

**Prerequisites:**
- Live Teradata database running
- Multiple databases accessible (3+)
- Each database has tables (5+)
- User has SELECT on DBC.TablesV, DBC.ColumnsV

**Execution Order:**
1. TC071 - Core completion fix (MUST PASS first)
2. TC072 - Qualified names (Teradata-specific)
3. TC073 - Column completion
4. TC074 - Multi-line regression
5. TC075 - Error handling

**Critical Validation:**
- Output contains database/table names (NOT keywords)
- No "(SQL keyword)" strings anywhere
- Completions are queryable

### Phase 2: Table Display Validation (TC076-TC080)

**Prerequisites:**
- Terminal width control available
- Can resize terminal to 80, 120, 160, 200+ cols
- Test query with 10+ columns ready (use DBC.TablesV)
- Visual inspection capability

**Execution Order:**
1. TC076 - 80 cols (MUST PASS first - core fix)
2. TC077 - 120 cols (common width)
3. TC078 - 160 cols (very wide)
4. TC079 - 200+ cols (edge case)
5. TC080 - TTY vs batch mode (CRITICAL - data completeness)

**Critical Validation:**
- Table fits in terminal width
- Headers align with data
- "(+n cols)" indicator when truncated
- Batch mode shows ALL columns

### Phase 3: Regression Validation (TC081-TC082)

**Prerequisites:**
- Sprint 10 test documentation available
- Core REPL features known working baseline

**Execution Order:**
1. TC082 - REPL core features (metacommands, history)
2. TC081 - Batch mode features (stdin, file, formats)

**Critical Validation:**
- No new failures vs Sprint 10
- All previous features work
- No state corruption

---

## Pass/Fail Criteria

### Sprint 11 Success Requires:

**Tab Completion:**
- ✅ TC071 PASS - Core completion shows objects, not keywords
- ✅ TC072 PASS - Qualified names work
- ✅ TC073 PASS - Column completion works
- ✅ TC074 PASS - Multi-line preserved (no regression)
- ⚠️ TC075 - Error handling (nice to have)

**Table Display:**
- ✅ TC076 PASS - 80-col terminal displays correctly
- ✅ TC077 PASS - 120-col terminal uses width
- ⚠️ TC078-TC079 - Wider terminals (nice to have)
- ✅ TC080 PASS - TTY vs batch mode (CRITICAL for data completeness)

**Regressions:**
- ✅ TC081 PASS - Batch mode intact
- ✅ TC082 PASS - REPL core intact

**Minimum Passing:**
- TC071, TC073, TC076, TC080, TC081, TC082 = MUST PASS
- TC072, TC074, TC077 = SHOULD PASS (high priority)
- TC075, TC078-TC079 = NICE TO PASS (lower priority)

---

## Testing Environment

### Required Setup:

**Database:**
- Live Teradata database (NOT mock)
- Multiple databases (production, staging, development recommended)
- Each database with 5+ tables
- User with SELECT on DBC views

**Terminal:**
- Ability to set terminal width
- iTerm2 or Terminal.app on macOS (can set columns)
- `tput cols` command available
- Visual ruler helpful (iTerm2: View > Show Ruler)

**Binary:**
- Release build: `cargo build --release`
- Sprint 11 fixes applied
- Commit: a1c02cd487add7080519760ce07a22383dcbf1e2

**Environment:**
- `.env` file with TQ_LOGON configured
- Connection verified: `./target/release/tq ping`

---

## New Testing Requirements (Sprint 11+)

Based on lessons learned from Sprint 11 bug failures:

### 1. Semantic Validation (NEW)

**OLD approach (insufficient):**
```
✗ "Verify completion triggered"
✗ "Verify output present"
```

**NEW approach (required):**
```
✓ "Verify completion shows database names (not keywords)"
✓ "Verify completions are queryable"
✓ "Verify output makes sense in context"
```

### 2. Visual Validation (NEW)

**OLD approach (insufficient):**
```
✗ "Verify table contains columns X, Y, Z"
✗ "Verify data present"
```

**NEW approach (required):**
```
✓ "Verify header X aligns with data column X"
✓ "Verify table width ≤ terminal width"
✓ "Verify layout readable by human"
✓ "Measure and document widths"
```

### 3. Anti-Pattern Detection (NEW)

Every test must include "Anti-Pattern" section:

```markdown
## Anti-Pattern (What Should NOT Happen)

**INCORRECT Output (Bug Behavior):**
[Specific example of failure mode]
[Screenshot or description of broken behavior]
```

### 4. Live Database Testing (MANDATORY)

- ALL tab completion tests against live Teradata
- NO mocks or simulations for integration tests
- Verify metadata queries work in practice
- Test with real database names/tables

### 5. Visual Inspection (MANDATORY for UI)

- Human visual inspection required
- Screenshots recommended
- Width measurements taken
- Alignment verified manually

---

## Test Execution Checklist

### Before Testing:

- [ ] Build release binary
- [ ] Start test database
- [ ] Verify connectivity (`tq ping`)
- [ ] Set terminal to 80 cols for TC076
- [ ] Have `tput cols` command ready
- [ ] Screenshot capability available

### During Testing:

- [ ] Execute tests in order (critical first)
- [ ] Capture screenshots of visual tests
- [ ] Measure widths for table tests
- [ ] Verify semantic meaning of completions
- [ ] Check anti-patterns don't occur
- [ ] Document actual vs expected

### After Testing:

- [ ] Calculate pass/fail statistics
- [ ] Categorize issues by severity
- [ ] Generate test report
- [ ] Update testing-guidelines.md with findings
- [ ] Provide go/no-go recommendation

---

## Known Issues to Watch For

### Tab Completion Failure Modes:

1. **"(SQL keyword)" repeated** - Primary bug, must be fixed
2. **Empty completion** - Context detection broken
3. **Wrong context** - Shows keywords in table position
4. **Slow/timeout** - Metadata query problems
5. **Crash on Tab** - Error handling broken

### Table Display Failure Modes:

1. **Excessive padding** - Primary bug, must be fixed
2. **Headers misaligned** - Layout calculation broken
3. **Exceeds terminal width** - Width detection broken
4. **All columns crammed** - No truncation applied
5. **Missing indicator** - "(+n cols)" not shown

---

## Test Automation

### Automated REPL Testing:

Use `expectrl` for REPL interaction tests (see `tests/interactive_tests.rs`):

```rust
use expectrl::spawn;

let mut p = spawn("tq repl")?;
p.expect("tq>")?;
p.send_line("SELECT * FROM ")?;
p.send(b"\t")?;  // Tab key
let output = p.expect_regex(r"Loading tables|Databases")?;
// Validate output contains database names
assert!(!output.contains("(SQL keyword)"));
```

### Manual Testing Required:

The following CANNOT be fully automated:
- Visual layout quality
- Readability assessment
- Terminal width testing (requires resizing)
- Human judgment of "looks good"

Automated tests can validate:
- Width measurements
- Content presence/absence
- Semantic types (objects vs keywords)
- Exit codes, errors

---

## Documentation

Each test case includes:

- **Purpose**: What this tests
- **Scope**: What is/isn't tested
- **Prerequisites**: Setup needed
- **Test Procedure**: Step-by-step execution
- **Expected Results**: Detailed expected output
- **Anti-Pattern**: What should NOT happen
- **Pass/Fail Criteria**: Clear success definition
- **Notes**: Context, implementation details, why this matters

Test results will be documented in:
- `tests/results/YYYYMMDD-HHMMSS/TC###.md` - Individual results
- `tests/results/YYYYMMDD-HHMMSS/REPORT.md` - Comprehensive report

---

## Sprint 11 Quality Gate

Sprint 11 can close when:

1. **Critical Tests Pass:**
   - TC071: Tab completion shows objects (not keywords) ✅
   - TC076: Table fits in 80-col terminal ✅
   - TC080: TTY truncates, batch shows all ✅
   - TC081: Batch mode works ✅
   - TC082: REPL core works ✅

2. **Bug Verification:**
   - Bug 2 (tab completion) FIXED and verified
   - Bug 1 (table display) FIXED and verified
   - No "(SQL keyword)" garbage
   - No excessive padding

3. **Regression Check:**
   - Sprint 9 multi-line completion still works
   - Sprint 10 batch mode still works
   - Core REPL features intact

4. **Documentation:**
   - Test report generated
   - Lessons learned documented
   - Testing guidelines updated
   - Anti-patterns documented for future

5. **User Acceptance:**
   - User verifies fixes work
   - User satisfied with quality
   - Trust restored in tool

---

## References

- Sprint 11 planning: `docs/builder/sprints/sprint-11-planning.md`
- Bug report: `docs/builder/incoming/open-bugs.md`
- Screenshots: `docs/builder/incoming/completion.png`, `table display-bug.png`
- Testing guidelines: `docs/builder/testing-guidelines.md` (v2.0 - updated with Sprint 11 lessons)
- REPL specifications: `docs/builder/detailed-specifications/repl-mode.md`
- Output specifications: `docs/builder/detailed-specifications/output-formats.md`
- Batch mode specifications: `docs/builder/detailed-specifications/batch-mode.md`

---

**Next Steps:**

1. Review this test plan with user
2. Verify test database available
3. Begin Phase 1: Tab completion tests
4. Execute tests systematically
5. Document results thoroughly
6. Generate comprehensive report
7. Provide quality gate assessment
