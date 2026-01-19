# Framework Meta-Fix: Interactive Feature Testing Enforcement

**Date:** 2026-01-19
**Issue:** Tab completion claimed "complete" for 4 sprints despite being broken
**Root Cause:** Agents lacked forcing function to ensure interactive tests exist and pass

---

## Systematic Framework Analysis

### What Actually Happened

**Sprint 7-12 Pattern:**
1. quality-validator designed test cases (TC027.md, TC028.md) ✅
2. quality-validator ran `cargo test` (unit tests only) ✅
3. quality-validator reported "216/216 tests passing (100%)" ✅
4. quality-validator claimed "testing complete" ✅
5. tq-project-manager saw "100% pass rate" ✅
6. tq-project-manager approved sprint closure ✅
7. sprint-coordinator marked sprint complete ✅
8. **Interactive tests: 0 executed, 0 passing** ❌

**User discovered:** Feature completely broken in real usage

### Root Cause: No Forcing Function for Test Type Requirements

**Problem:** Agents can claim "tests passing" without executing the right types of tests.

**Evidence:**
- `tests/cases/TC027.md` exists (designed by quality-validator)
- `tests/interactive_tests.rs` has framework but NO tab completion tests
- quality-validator never wrote expectrl tests
- tq-project-manager never checked if interactive tests exist
- sprint-coordinator never validated test type coverage

**Gap:** No mechanism prevents agents from skipping required test types.

---

## Agent-by-Agent Failure Analysis

### Agent 1: quality-validator (Sonnet)

**Current Instructions (lines 79-90):**
```markdown
3. **Test Execution**
   - Execute each test case systematically
   - Document actual behavior vs. expected behavior
   - Capture error messages, output formatting, and user experience issues
```

**What's Missing:**
- ❌ No classification of feature types (REPL vs batch vs backend)
- ❌ No requirement for interactive tests for REPL features
- ❌ No validation that expectrl tests exist
- ❌ No blocking condition if interactive tests missing

**What Agent Did:**
- Designed TC027.md, TC028.md (manual test cases) ✅
- Ran `cargo test --lib` (unit tests) ✅
- Reported 216/216 passing ✅
- **Never wrote expectrl tests** ❌
- **Never executed manual test cases** ❌

**Why Agent Failed:**
Agent followed instructions literally: "Execute each test case systematically"
- Unit tests: ✅ Executed systematically
- Interactive tests: ❌ No instruction saying they're required

### Agent 2: tq-project-manager (Haiku)

**Current Instructions (lines 36-41):**
```markdown
3. **Read the test report:**
   - File: `tests/results/[latest]/REPORT.md`
   - Verify: 100% test pass rate, comprehensive coverage
```

**What's Missing:**
- ❌ No definition of "comprehensive coverage" by feature type
- ❌ No checklist for required test types
- ❌ No validation that interactive tests exist for REPL features
- ❌ No blocking condition if test types missing

**What Agent Did:**
- Read REPORT.md ✅
- Saw "216/216 tests passing (100%)" ✅
- Verified test pass rate ✅
- Approved sprint closure ✅
- **Never checked if interactive tests exist** ❌

**Why Agent Failed:**
Agent followed instructions literally: "Verify 100% test pass rate"
- Unit tests: ✅ 100% passing
- Interactive tests: ❌ No instruction to verify they exist

### Agent 3: sprint-coordinator (Main Agent)

**Current Instructions (Phase 4, sprint-coordinator/SKILL.md lines 123-146):**
```markdown
### Phase 4: Test Execution Phase (You Coordinate)

**Goal:** Execute all tests and validate quality.

**Your Actions:**

1. **Launch quality-validator:**
   - Expected output: Comprehensive test report with results

2. **Analyze Results:**
   - Review test report
   - Check pass rate (should be 100%)
   - Analyze any failures

3. **Decision Point:**
   - **All tests pass:** Proceed to Phase 5 (Sprint Closure)
   - **Tests fail:** Launch rust-teradata-architect to fix issues, return to start of Phase 4
```

**What's Missing:**
- ❌ No validation that required test types exist
- ❌ No check for feature type vs test type alignment
- ❌ No blocking condition if test coverage inadequate

**What Agent Did:**
- Launched quality-validator ✅
- Received "all tests passing" ✅
- Proceeded to Phase 5 ✅
- **Never verified test types match feature types** ❌

**Why Agent Failed:**
Agent followed instructions literally: "Check pass rate (should be 100%)"
- Pass rate: ✅ 100%
- Test type coverage: ❌ No instruction to verify

---

## The Meta-Fix: Evidence-Based Validation with Feature Classification

### Core Principle

**New Rule:** Agents cannot claim tests pass without providing evidence of the RIGHT TYPE of tests for the feature type.

### Component 1: Feature Type Classification (specifications.md)

**Change:** Add feature type metadata to specifications.md

**Before:**
```markdown
| Tab completion (tables) | ✅ Implemented | P0 |
```

**After:**
```markdown
| Feature | Status | Type | Required Tests | Priority |
|---------|--------|------|----------------|----------|
| Tab completion (tables) | ✅ Implemented | **Interactive/REPL** | Unit + **Interactive** | P0 |
| Batch mode | ✅ Implemented | **Batch/CLI** | Unit + Integration | P0 |
| SQL parsing | ✅ Implemented | **Backend/Logic** | Unit only | P1 |
```

**Feature Type Definitions:**
- **Interactive/REPL:** Features involving PTY, terminal interaction, tab completion, visual rendering
- **Batch/CLI:** Command-line execution, piped input/output, scripting
- **Backend/Logic:** Internal logic, parsing, algorithms (no user interaction)

**Test Requirements by Type:**
```markdown
## Test Requirements by Feature Type

| Feature Type | Required Tests | Tools |
|--------------|----------------|-------|
| Interactive/REPL | Unit + **Interactive (expectrl)** | `cargo test` + `tests/interactive_tests.rs` |
| Batch/CLI | Unit + Integration | `cargo test` + integration tests |
| Backend/Logic | Unit | `cargo test --lib` |
```

### Component 2: Test Evidence Registry (new file)

**Create:** `tests/test-evidence.md`

**Purpose:** Machine-readable evidence that required tests exist and pass

**Format:**
```markdown
# Test Evidence Registry

## Sprint 12 Features

### Feature: Tab Completion for Tables
**Type:** Interactive/REPL
**Required Tests:**
- [x] Unit tests: `cargo test metadata_completer` - 13/13 passing
- [ ] Interactive tests: `cargo test --test interactive_tests test_table_completion` - **0 tests found** ❌

**Status:** ❌ INCOMPLETE - Interactive tests missing

### Feature: Export to Clipboard
**Type:** Interactive/REPL
**Required Tests:**
- [x] Unit tests: `cargo test metacommands::tests::test_export_to_clipboard` - 5/5 passing
- [ ] Interactive tests: **No expectrl tests exist** ❌

**Status:** ❌ INCOMPLETE - Interactive tests missing

### Feature: Batch Mode
**Type:** Batch/CLI
**Required Tests:**
- [x] Unit tests: `cargo test query` - 12/12 passing
- [x] Integration tests: `cargo test --test integration_tests batch` - 8/8 passing

**Status:** ✅ COMPLETE - All required test types present
```

**Key:** Agents MUST update this file with evidence, not just claims.

### Component 3: Update quality-validator Instructions

**Add to .claude/agents/quality-validator.md after line 90:**

```markdown
## Critical: Feature Type-Based Testing Requirements

**MANDATORY:** You MUST execute the appropriate test types based on feature classification.

### Before Testing: Read Feature Types

1. **Read `docs/builder/specifications.md`** - Check feature type column
2. **Identify required test types** based on feature type:
   - **Interactive/REPL** → Unit tests + Interactive tests (expectrl)
   - **Batch/CLI** → Unit tests + Integration tests
   - **Backend/Logic** → Unit tests only

3. **Create test evidence registry** in `tests/test-evidence.md`

### Test Type Requirements

#### For Interactive/REPL Features

**MANDATORY:** Interactive tests using expectrl MUST exist and pass.

**How to verify:**
```bash
# Check if interactive tests exist for this feature
grep -r "test.*<feature_name>" tests/interactive_tests.rs

# Run interactive tests
cargo test --test interactive_tests <feature_name>
```

**If interactive tests don't exist:**
- ❌ You CANNOT claim "testing complete"
- ❌ You MUST create expectrl tests before proceeding
- ❌ Your REPORT.md MUST show "INCOMPLETE - Interactive tests missing"

**Example Interactive Test (expectrl):**
```rust
#[test]
fn test_table_completion_after_from() {
    let mut p = spawn_tq_repl();
    p.send("SELECT * FROM ");
    p.send("\t"); // Press Tab

    // Verify databases appear, not keywords
    p.expect_any(vec!["DBC", "SYSUDTLIB"]).expect("Should show databases");

    // Verify keywords don't appear
    let output = p.exp_string().unwrap();
    assert!(!output.contains("(SQL keyword)"), "Should not show SQL keywords");
}
```

#### For Batch/CLI Features

**MANDATORY:** Integration tests MUST exist and pass.

Integration tests run the binary end-to-end:
```rust
#[test]
fn test_batch_mode_multiple_statements() {
    let output = Command::cargo_bin("tq")
        .arg("query")
        .arg("-l").arg(env::var("TQ_LOGON").unwrap())
        .arg("SELECT 1; SELECT 2;")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    // Verify output contains both results
}
```

#### For Backend/Logic Features

**SUFFICIENT:** Unit tests covering logic paths.

### Test Evidence Documentation

**MANDATORY:** Update `tests/test-evidence.md` with concrete evidence:

```markdown
### Feature: Tab Completion
**Type:** Interactive/REPL
**Required Tests:**
- [x] Unit tests: `cargo test metadata_completer` - 13/13 passing
- [x] Interactive tests: `cargo test --test interactive_tests table_completion` - 3/3 passing ✅

**Evidence:**
- Unit test output: [paste relevant output]
- Interactive test output: [paste relevant output]
- Test files: tests/interactive_tests.rs lines 50-120

**Status:** ✅ COMPLETE - All required test types executed and passing
```

### Quality Gate: Test Type Validation

**Before generating REPORT.md:**

1. **Verify every feature has required test types:**
   - Check test-evidence.md for each feature
   - Ensure all checkboxes marked for required types
   - Verify concrete evidence provided (not just claims)

2. **If ANY required test type missing:**
   - Mark REPORT.md status as "INCOMPLETE"
   - List missing test types explicitly
   - Provide commands to create missing tests
   - **DO NOT claim "testing complete"**

3. **REPORT.md must include test type breakdown:**
   ```markdown
   ## Test Coverage by Type

   | Feature | Type | Unit | Integration | Interactive | Status |
   |---------|------|------|-------------|-------------|--------|
   | Tab completion | Interactive/REPL | ✅ 13/13 | N/A | ❌ 0/? | INCOMPLETE |
   | Batch mode | Batch/CLI | ✅ 12/12 | ✅ 8/8 | N/A | COMPLETE |
   | SQL parsing | Backend/Logic | ✅ 24/24 | N/A | N/A | COMPLETE |
   ```

**Critical:** You cannot approve sprint closure if ANY Interactive/REPL feature lacks interactive tests.
```

### Component 4: Update tq-project-manager Instructions

**Add to .claude/agents/tq-project-manager.md after line 41:**

```markdown
### Step 2a: Validate Test Type Coverage (MANDATORY)

**CRITICAL:** Before validating feature completion, verify correct test types exist.

1. **Read test evidence registry:**
   - File: `tests/test-evidence.md`
   - Verify: Every feature has required test type evidence

2. **Check feature type alignment:**
   ```bash
   # For each Interactive/REPL feature in sprint:
   grep -A 10 "Feature: <feature_name>" tests/test-evidence.md

   # Verify line shows:
   # - [x] Interactive tests: ... passing ✅
   # NOT:
   # - [ ] Interactive tests: ... missing ❌
   ```

3. **Validation Logic:**
   ```
   FOR each feature in sprint:
     READ feature type from specifications.md
     READ test evidence from test-evidence.md

     IF feature type is "Interactive/REPL":
       IF interactive tests not present OR not passing:
         BLOCK sprint closure
         STATUS = ❌ NOT APPROVED
         REASON = "Interactive tests missing for Interactive/REPL feature"
       END IF
     END IF
   END FOR
   ```

**Blocking Condition:**

If ANY Interactive/REPL feature lacks interactive tests:
- ❌ Sprint CANNOT be approved
- ❌ You MUST return "NOT APPROVED" status
- ❌ Your report MUST list missing test types explicitly

**Example Blocking Issue:**
```markdown
## Go/No-Go Decision

**Decision:** ❌ NOT APPROVED

**Blockers:**
1. **Tab Completion (Interactive/REPL)** - No interactive tests exist
   - Required: `tests/interactive_tests.rs::test_table_completion_*`
   - Actual: 0 interactive tests found
   - Evidence: test-evidence.md shows "Interactive tests: 0 tests found ❌"
   - Action: quality-validator must create expectrl tests before approval

2. **Export to Clipboard (Interactive/REPL)** - No interactive tests exist
   - Required: `tests/interactive_tests.rs::test_export_clipboard_*`
   - Actual: 0 interactive tests found
   - Evidence: test-evidence.md shows "Interactive tests: missing ❌"
   - Action: quality-validator must create expectrl tests before approval

**Cannot approve sprint with Interactive/REPL features lacking interactive tests.**
```

### Updated Validation Checklist

Add to existing validation checklist (line 45):

```markdown
### Functional Validation
- [ ] Feature works as specified in detailed-specifications
- [ ] All acceptance criteria from sprint-N-planning.md are met
- [ ] **Test type matches feature type (Interactive/REPL → interactive tests exist)** ← NEW
- [ ] Edge cases are handled correctly
- [ ] Error handling is robust and user-friendly
```

Add to Code Quality Validation (line 54):

```markdown
### Code Quality Validation
- [ ] Code is clean, readable, and idiomatic Rust
- [ ] No code duplication or unnecessary complexity
- [ ] Follows patterns in rust-architecture.md
- [ ] Unit tests exist and pass (100% pass rate)
- [ ] Integration tests exist and pass (100% pass rate - if required by feature type) ← UPDATED
- [ ] **Interactive tests exist and pass (if feature type is Interactive/REPL)** ← NEW
```
```

### Component 5: Update sprint-coordinator Phase 4

**Update .claude/skills/sprint-coordinator/SKILL.md lines 123-146:**

**Before:**
```markdown
2. **Analyze Results:**
   - Review test report
   - Check pass rate (should be 100%)
   - Analyze any failures

3. **Decision Point:**
   - **All tests pass:** Proceed to Phase 5 (Sprint Closure)
   - **Tests fail:** Launch rust-teradata-architect to fix issues, return to start of Phase 4
```

**After:**
```markdown
2. **Analyze Results:**
   - Review test report (tests/results/[latest]/REPORT.md)
   - Check pass rate (should be 100%)
   - **Validate test-evidence.md shows all required test types present** ← NEW
   - Analyze any failures

3. **Validate Test Type Coverage:** ← NEW STEP
   - Read `tests/test-evidence.md`
   - For each feature, verify required test types are checked:
     - Interactive/REPL features → Interactive tests must be present
     - Batch/CLI features → Integration tests must be present
     - Backend/Logic features → Unit tests sufficient
   - If ANY required test type missing → BLOCK Phase 5

4. **Decision Point:**
   - **All tests pass AND all test types present:** Proceed to Phase 5 (Sprint Closure)
   - **Tests fail OR test types missing:** Launch rust-teradata-architect to fix/add tests, return to start of Phase 4
   - **Quality-validator reports "INCOMPLETE":** Cannot proceed to Phase 5
```

---

## Implementation Plan for Meta-Fix

### Phase 1: Update Specifications (Immediate)

**File:** `docs/builder/specifications.md`

**Changes:**
1. Add "Type" column to feature table
2. Classify all existing features (Interactive/REPL, Batch/CLI, Backend/Logic)
3. Add "Test Requirements by Feature Type" section

**Estimated Effort:** 1 hour

### Phase 2: Create Test Evidence Registry (Immediate)

**File:** `tests/test-evidence.md` (NEW)

**Contents:**
- Template with feature classification
- Evidence checkboxes for each test type
- Status indicators (COMPLETE/INCOMPLETE)

**Estimated Effort:** 30 minutes

### Phase 3: Update Agent Instructions (Critical)

**Files:**
- `.claude/agents/quality-validator.md` (add ~150 lines)
- `.claude/agents/tq-project-manager.md` (add ~80 lines)
- `.claude/skills/sprint-coordinator/SKILL.md` (modify Phase 4)

**Changes:**
- Add feature type-based testing requirements
- Add test evidence validation logic
- Add blocking conditions for missing test types

**Estimated Effort:** 2-3 hours

### Phase 4: Backfill Test Evidence for Sprint 12 (Validation)

**Action:** Document current state honestly in test-evidence.md

**Example:**
```markdown
### Feature: Tab Completion
**Type:** Interactive/REPL
**Required Tests:**
- [x] Unit tests: 13/13 passing
- [ ] Interactive tests: **0 tests exist** ❌

**Status:** ❌ INCOMPLETE - Cannot claim feature "complete" without interactive tests
```

**Estimated Effort:** 15 minutes

### Phase 5: User Approval (Required)

**Present to user:**
- This meta-fix document
- Proposed changes to agent instructions
- Test evidence registry template
- Implementation plan

**Get explicit approval before proceeding.**

---

## Why This Meta-Fix Works

### Forcing Function 1: Feature Type Classification

**Problem:** Agents didn't know interactive tests were required
**Solution:** Specifications.md explicitly states test requirements by feature type
**Enforcement:** Agents read specifications before testing (already required)

### Forcing Function 2: Test Evidence Registry

**Problem:** Agents could claim "tests pass" without proof
**Solution:** Machine-readable evidence file with checkboxes
**Enforcement:** Agents must update with concrete evidence, not claims

### Forcing Function 3: quality-validator Validation Logic

**Problem:** quality-validator skipped interactive tests
**Solution:** Instructions explicitly require interactive tests for Interactive/REPL features
**Enforcement:** Cannot generate REPORT.md with "COMPLETE" status if tests missing

### Forcing Function 4: tq-project-manager Blocking Condition

**Problem:** tq-project-manager approved without checking test types
**Solution:** Validation checklist includes test type verification
**Enforcement:** "NOT APPROVED" status if required test types missing

### Forcing Function 5: sprint-coordinator Gate

**Problem:** sprint-coordinator proceeded to Phase 5 without validating test coverage
**Solution:** Phase 4 includes test evidence validation step
**Enforcement:** Cannot proceed to Phase 5 if test-evidence.md shows INCOMPLETE

### Defense in Depth

**Multiple checkpoints:**
1. quality-validator must create evidence → blocks at test design
2. tq-project-manager must verify evidence → blocks at validation
3. sprint-coordinator must check evidence → blocks at phase transition

**Any agent failing = sprint blocked = issue surfaced immediately**

---

## Expected Outcomes

### Short Term (Sprint 13)

1. **Interactive tests created for tab completion** - Cannot proceed without them
2. **Test evidence documented explicitly** - No more claims without proof
3. **Sprint closure blocked if tests missing** - Forcing function prevents repeat

### Medium Term (Future Sprints)

1. **Agents automatically classify features** - Read type from specifications
2. **Test design includes correct types** - quality-validator knows requirements
3. **Validation catches gaps immediately** - No more "complete but broken"

### Long Term (Framework Evolution)

1. **Feature type taxonomy expands** - Add types as needed (API, Config, etc.)
2. **Test requirements evolve** - Add new test types (performance, security, etc.)
3. **Framework self-corrects** - Agents enforce quality standards automatically

---

## Validation of Meta-Fix

### Test the Meta-Fix Implementation

**Scenario 1: Interactive Feature Without Interactive Tests**
1. Add Interactive/REPL feature to sprint
2. quality-validator designs unit tests only
3. test-evidence.md shows "Interactive tests: missing ❌"
4. tq-project-manager reads evidence → "NOT APPROVED"
5. sprint-coordinator blocks Phase 5
6. **Expected:** Sprint cannot close until interactive tests exist

**Scenario 2: Backend Feature (No Interactive Tests Needed)**
1. Add Backend/Logic feature to sprint
2. quality-validator designs unit tests
3. test-evidence.md shows "Unit tests: passing ✅"
4. tq-project-manager reads evidence → "APPROVED"
5. sprint-coordinator proceeds to Phase 5
6. **Expected:** Sprint closes normally (no interactive tests required)

**Scenario 3: Batch Feature Without Integration Tests**
1. Add Batch/CLI feature to sprint
2. quality-validator designs unit tests only
3. test-evidence.md shows "Integration tests: missing ❌"
4. tq-project-manager reads evidence → "NOT APPROVED"
5. **Expected:** Sprint blocked until integration tests exist

**If all scenarios behave as expected:** Meta-fix is working correctly.

---

## Commitment

With this meta-fix implemented, the tab completion failure pattern CANNOT repeat:

1. **specifications.md will classify tab completion as "Interactive/REPL"** ✅
2. **quality-validator will be required to create expectrl tests** ✅
3. **test-evidence.md will show "Interactive tests: 0 tests" explicitly** ✅
4. **tq-project-manager will block approval with clear reason** ✅
5. **sprint-coordinator will not proceed to Phase 5** ✅
6. **User will see "Sprint blocked: Interactive tests missing"** ✅

**Result:** Issue surfaced immediately, not after 4 sprints of "complete but broken."

---

## Next Steps

1. **User reviews this meta-fix document**
2. **User approves implementation plan**
3. **Implement Phase 1-3 (update specifications, create registry, update agents)**
4. **Validate meta-fix with test scenarios**
5. **Apply to Sprint 13 planning**

---

**Document Status:** PROPOSED - Awaiting user approval
**Implementation:** Blocked pending user sign-off
