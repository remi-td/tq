# Testing Guidelines

This document consolidates testing best practices learned from Sprints 20, 21, and 22, providing comprehensive guidelines for test strategy creation, implementation, and execution.

## Purpose

**Context**: tq is a database CLI tool with two primary modes:
- **Batch mode**: Non-interactive command execution (scripts, pipelines)
- **REPL mode**: Interactive terminal with tab completion, syntax highlighting, multi-line editing

**Challenge**: Hybrid testing required - automated tests validate logic, manual tests validate user experience.

**Solution**: This guide documents the proven hybrid testing approach that evolved through:
- Sprint 20: Crisis learning (3 iterations to find correct root cause)
- Sprint 21: Proactive excellence (comprehensive test strategy prevented false positives)
- Sprint 22: Process maturation (test implementation checklist added)

---

## Table of Contents

1. [Core Testing Principles](#core-testing-principles)
2. [Hybrid Testing Pattern](#hybrid-testing-pattern)
3. [Test Strategy Creation](#test-strategy-creation)
4. [Test Types by Feature Type](#test-types-by-feature-type)
5. [Automation Capabilities & Limitations](#automation-capabilities--limitations)
6. [False Positive Prevention](#false-positive-prevention)
7. [Verdict Criteria](#verdict-criteria)
8. [Test Implementation Process](#test-implementation-process)
9. [Manual Testing Guidelines](#manual-testing-guidelines)
10. [Common Pitfalls & Solutions](#common-pitfalls--solutions)

---

## Core Testing Principles

### Principle 1: Test What Users See (Not What Code Does)

**Bad**: Test verifies function returns without error
**Good**: Test verifies function produces correct user-visible result

**Sprint 20 Example**:
- **Iteration 1-2**: Automated tests passed (completion logic worked)
- **User validation**: FAILED (pager banner still appeared)
- **Root cause**: Tests validated data layer, not presentation layer

**Lesson**: Automated tests validate CODE behavior, manual tests validate USER experience. Both required.

### Principle 2: Specifications Are Source of Truth

**Process**:
1. Read specification to understand expected behavior
2. Design tests that validate specification requirements
3. Implement feature to pass tests
4. If test fails, either fix implementation OR update specification (never ignore failures)

**Sprint 22 Example**:
- **Issue**: User guide described SQL LIKE patterns (`%`, `_`), code used glob (`*`, `?`)
- **Impact**: Documentation-implementation mismatch created false user expectations
- **Lesson**: Update documentation AFTER implementation confirmed, verify before ship

### Principle 3: Hybrid Testing for Interactive Features

**Pattern**: Automated tests + Manual validation = Confidence

| Test Component | Purpose | What It Validates | What It Misses |
|----------------|---------|-------------------|----------------|
| **Automated** | Fast feedback, regression detection, CI/CD | Logic correctness, data accuracy, error handling | Visual rendering, keyboard UX, cursor position |
| **Manual** | User experience validation, false positive detection | Visual output, interaction flow, subjective quality | Cannot run in CI/CD, requires human |

**Sprint 21 Success**: Made manual validation PRIMARY (not secondary) for Feature 3 (keyboard UX), preventing false positives.

### Principle 4: Test Limitations Must Be Explicit

**Sprint 21 Innovation**: 15,461-line test strategy documented automation limitations BEFORE implementation:

**PTY Tests CANNOT Validate**:
- TAB vs ENTER vs DOWN arrow behavior (keyboard interaction)
- Visual menu rendering (columns, alignment, colors)
- Cursor position after completion
- Negative assertions ("no pager output appears")

**Result**: Team knew Feature 3 had EXTREMELY HIGH false positive risk, made manual testing PRIMARY.

### Principle 5: Test Strategy ≠ Test Implementation

**Sprint 22 Lesson**: Clear test strategy doesn't guarantee tests are implemented.

**Problem**: Iteration 1 missing Feature 2 integration/PTY tests despite strategy specifying them.

**Solution**: Test implementation checklist (see `docs/testing/checklist.md`) verifies strategy requirements met before quality review.

---

## Hybrid Testing Pattern

### When to Use Hybrid Testing

**MANDATORY for**:
- User-facing bug fixes (Sprint 20 lesson)
- Interactive REPL features (Sprint 21 lesson)
- Visual output changes (logo, formatting, colors)
- Keyboard interaction features (tab completion, shortcuts)
- Features with false positive history

**OPTIONAL for**:
- Internal refactoring (no user-facing changes)
- Pure logic/algorithms (no visual output)
- Batch mode commands (CLI, no interaction)

### Hybrid Testing Workflow

```
Phase 1: Automated Testing
├─ Unit tests validate logic correctness
├─ Integration tests validate end-to-end with database
├─ PTY tests validate terminal output content
└─ Result: PASS or FAIL

IF Automated PASS:
  Phase 2: Manual Validation
  ├─ Human validates visual output
  ├─ Human tests keyboard interaction
  ├─ Human verifies subjective quality
  └─ Result: PASS or FAIL

Verdict Logic:
  APPROVED: Automated PASS + Manual PASS ✅
  REJECTED: Either component FAIL ❌
  BLOCKED: Tests cannot execute ⛔
```

### Sprint 20 Example: Hybrid Testing Success

| Iteration | Unit | Integration | PTY | Manual | User | Verdict |
|-----------|------|-------------|-----|--------|------|---------|
| 1 | ✅ 234/234 | ✅ 37/37 | ✅ 19/19 | ❌ FAIL | "Still same" | REJECTED |
| 2 | ✅ 234/234 | ✅ 37/37 | ✅ 19/19 | ❌ FAIL | "Still same" | REJECTED |
| 3 | ✅ 234/234 | ✅ 37/37 | ✅ 19/19 | ✅ PASS | "Bravo!!!" | APPROVED ✅ |

**Key Insight**: Automated tests passed in ALL 3 iterations, only manual validation detected false positives.

---

## Test Strategy Creation

### Step 1: Analyze Feature Characteristics

For each feature, answer:

1. **What is the primary user interaction?**
   - Interactive PTY (REPL, terminal UI)
   - CLI Batch (scripted, piped, non-interactive)
   - Pure logic (internal algorithm)

2. **What is the observable behavior?**
   - Visual output (colors, formatting, layout)
   - Structured data (JSON, CSV)
   - File system changes
   - Database changes

3. **What are the external dependencies?**
   - Database connection
   - Terminal/PTY
   - File system
   - None (pure logic)

4. **What makes this hard to test?**
   - Visual rendering requires actual PTY
   - Keyboard UX requires real interaction
   - Async timing is non-deterministic

5. **What are critical behaviors from specification?**
   - Quote specific requirements with section references
   - Identify must-have vs nice-to-have validations

### Step 2: Derive Required Test Types

Use decision tree:

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: End-to-end CLI execution needs validation

IF "Visual output in terminal" checked:
  → Interactive tests OR manual validation REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Mocks don't catch SQL syntax errors, query performance issues

IF "Keyboard interaction" checked:
  → Manual validation PRIMARY (automated tests secondary)
  Reason: PTY tests cannot distinguish TAB vs ENTER vs arrow keys
```

### Step 3: Assess False Positive Risk

For each feature, rate false positive risk:

**LOW Risk** - Automated tests sufficient:
- Pure logic/algorithms
- Data transformations
- Configuration parsing
- Simple database queries

**MEDIUM Risk** - Automated + spot-check manual:
- Complex database operations
- Multi-stage workflows
- Output formatting
- Error message display

**EXTREMELY HIGH Risk** - Manual validation PRIMARY:
- Keyboard interaction (TAB, ENTER, arrows)
- Visual rendering (colors, alignment, cursor)
- Timing-based behavior
- Subjective UX quality

**Sprint 21 Example**: Feature 3 (second TAB accepts) identified as EXTREMELY HIGH risk upfront, manual made PRIMARY.

### Step 4: Document Automation Limitations

Explicitly state what automated tests CANNOT validate:

**Example from Sprint 21**:
```markdown
## Automation Limitations

PTY tests CANNOT validate:
- TAB vs ENTER vs DOWN arrow behavior (keyboard events indistinguishable)
- Visual menu rendering (colors, alignment, width)
- Cursor position after completion (reedline limitation)
- Negative assertions ("no banner appears" - false positives likely)

Therefore: Manual validation is PRIMARY for Feature 3.
```

### Step 5: Define Verdict Criteria

Be explicit about approval requirements:

**Sprint 21 Verdict Logic**:
```
APPROVED:
  - Feature 1: Automated PASS + Manual PASS
  - Feature 2: Automated PASS + Manual PASS
  - Feature 3: DEFERRED (not implemented)
  - Feature 4: Automated PASS + Manual PASS

REJECTED:
  - Manual validation NOT PERFORMED for any P0 feature
  - Automated tests FAIL
  - Test types from strategy missing

BLOCKED:
  - Database unavailable
  - Credentials missing
  - Driver initialization error
```

---

## Test Types by Feature Type

### Batch Mode Features

**Examples**: `--output file.csv`, `--atomic`, file input

**Test Strategy**:
- **Unit tests**: Argument parsing, option validation, file path handling
- **Integration tests**: Full CLI invocation with real database, verify output files created
- **PTY tests**: NOT NEEDED (non-interactive)
- **Manual tests**: NOT NEEDED (automated sufficient)

**Verdict**: Automated PASS = APPROVED (no manual needed)

### REPL Keyboard UX Features

**Examples**: Tab completion, keyboard shortcuts, multi-line editing

**Test Strategy**:
- **Unit tests**: Completion logic, state management (validates mechanics)
- **Integration tests**: OPTIONAL (may not add value over PTY)
- **PTY tests**: OPTIONAL (often false positives for keyboard events)
- **Manual tests**: PRIMARY (only way to validate keyboard UX)

**Verdict**: Manual PASS required (automated tests secondary)

**Sprint 21 Example**: Feature 3 - Made manual PRIMARY, prevented false positive.

### REPL Visual Output Features

**Examples**: Table formatting, syntax highlighting, prompt styling

**Test Strategy**:
- **Unit tests**: Formatting logic (string generation)
- **Integration tests**: NOT NEEDED
- **PTY tests**: Validate output content (not pixel-perfect rendering)
- **Manual tests**: RECOMMENDED (verify visual quality)

**Verdict**: Automated PASS + spot-check manual = APPROVED

### Database Commands

**Examples**: `/list databases`, `SELECT * FROM table`, metacommands

**Test Strategy**:
- **Unit tests**: SQL generation, result parsing
- **Integration tests**: Execute against live database, verify results
- **PTY tests**: If REPL mode, verify output display
- **Manual tests**: NOT NEEDED (automated sufficient)

**Verdict**: Automated PASS = APPROVED

**Sprint 22 Example**: Feature 2 (schema commands) - Integration + PTY required, verified in Iteration 2.

### Pure Logic/Algorithms

**Examples**: Glob pattern matching, string parsing, calculations

**Test Strategy**:
- **Unit tests**: ONLY test type needed
- **Integration tests**: NOT NEEDED
- **PTY tests**: NOT NEEDED
- **Manual tests**: NOT NEEDED

**Verdict**: Unit tests PASS = APPROVED

---

## Automation Capabilities & Limitations

### What Automated Tests CAN Validate

| Test Type | Capabilities |
|-----------|--------------|
| **Unit** | Logic correctness, data transformations, error handling, edge cases, boundary conditions |
| **Integration** | End-to-end workflows, database query results, file I/O, exit codes, structured output (JSON/CSV) |
| **PTY** | Output content presence, text structure, multi-line behavior, basic REPL flow |

### What Automated Tests CANNOT Validate

| Test Type | Limitations |
|-----------|-------------|
| **Unit** | Visual rendering, user interaction, database-specific SQL dialects, actual terminal behavior |
| **Integration** | Subjective UX quality, visual alignment, color rendering, keyboard shortcuts |
| **PTY** | **Exact cursor position**, TAB vs ENTER vs arrow keys, **visual menu rendering**, negative assertions ("banner does NOT appear"), **color accuracy** |

### Critical PTY Test Limitations (Sprint 20-21 Learnings)

**Problem**: PTY tests give false positives for presentation-layer bugs.

**Examples**:
1. **Sprint 20 Iter 1-2**: PTY tests passed, pager banner still appeared
   - **Why**: Tests checked data (completions exist), not UI (menu component)

2. **Sprint 21 Feature 3**: PTY tests cannot distinguish keyboard events
   - **Why**: reedline library sends same event for TAB and ENTER in PTY environment

**Mitigation**: Document limitations upfront, make manual validation PRIMARY for affected features.

### When PTY Tests Are Sufficient

**Good use cases**:
- Output content validation ("database names appear")
- Multi-line behavior ("prompt preserves across lines")
- Basic interaction flow ("query executes successfully")
- Error message display ("error shown to user")

**Bad use cases** (require manual):
- Cursor position validation
- Keyboard event disambiguation
- Visual rendering quality
- Color/alignment accuracy
- Negative assertions

---

## False Positive Prevention

### Definition

**False Positive**: Automated tests pass, but feature doesn't work correctly for users.

**Sprint 20 Example**: All 290 tests passed in Iterations 1-2, but pager banner still appeared.

### Root Causes

1. **Testing Wrong Layer**
   - Tests validate data layer (completion logic works)
   - Bug exists in presentation layer (menu displays banner)
   - Solution: Test the layer where bug could exist

2. **Incomplete Validation**
   - Tests check output contains "database names"
   - Bug: Output also contains "(SQL keyword)" garbage
   - Solution: Test for absence of wrong content, not just presence of right content

3. **Mock-Heavy Tests**
   - Tests mock database driver
   - Bug: Real driver behaves differently than mock
   - Solution: Use real dependencies for integration tests

4. **PTY Limitations**
   - PTY tests cannot validate keyboard events accurately
   - Bug: Second TAB moves cursor down instead of accepting
   - Solution: Manual validation for keyboard UX

### Prevention Strategies

#### Strategy 1: Proactive Risk Assessment

Before implementation, identify false positive risk:

**Sprint 21 Approach**:
```markdown
## False Positive Risk Assessment

Feature 1 (Database metadata): LOW
- Pure data query, unit tests sufficient

Feature 2 (On-demand loading): MEDIUM
- Complex workflow, integration tests required

Feature 3 (Second TAB accepts): EXTREMELY HIGH
- Keyboard UX, PTY tests unreliable
- Solution: Manual validation PRIMARY
```

#### Strategy 2: Negative Assertions

Test that wrong things DON'T happen:

**Good**:
```rust
#[test]
fn tab_after_from_shows_databases_not_keywords() {
    let completions = get_completions("SELECT * FROM ", db);
    assert!(completions.contains("my_database"));
    assert!(!completions.contains("SELECT")); // Negative assertion
    assert!(!completions.contains("(SQL keyword)")); // Negative assertion
}
```

**Bad**:
```rust
#[test]
fn tab_after_from_shows_completions() {
    let completions = get_completions("SELECT * FROM ", db);
    assert!(completions.len() > 0); // Could be wrong completions!
}
```

#### Strategy 3: Test Multiple Layers

For critical features, test all layers:

**Example: Tab Completion**
- **Unit tests**: Completion logic returns correct databases
- **Integration tests**: REPL accepts Tab key and queries database
- **PTY tests**: Output contains database names, not keywords
- **Manual tests**: Human verifies menu displays correctly, Tab accepts selection

#### Strategy 4: User Validation for Bug Fixes

**Sprint 20 Lesson**: For user-reported bugs, user validation is MANDATORY.

**Process**:
1. User reports bug with reproduction steps
2. Implement fix and automated tests
3. User validates fix in their environment
4. Only approve if user confirms "works for me"

**Why**: Users encounter bugs in real-world scenarios automated tests miss.

#### Strategy 5: Manual Validation PRIMARY for High-Risk Features

**Sprint 21 Innovation**: Don't treat manual testing as "nice to have" backup.

**When manual is PRIMARY**:
- Keyboard interaction (TAB, ENTER, shortcuts)
- Visual quality (alignment, colors, layout)
- Subjective UX (smooth, intuitive, responsive)
- Timing-based behavior (loading indicators, spinners)

**Verdict logic**: Manual PASS required for APPROVED (automated tests are secondary validation).

---

## Verdict Criteria

### Verdict Options

**APPROVED ✅** - Sprint can ship
- All P0 features delivered
- All automated tests pass (100% or documented exceptions)
- All manual validations pass (for features requiring manual)
- Zero regressions (existing tests still pass)
- Documentation matches implementation

**REJECTED ❌** - Sprint cannot ship (iterate required)
- Any P0 feature fails tests
- Manual validation not performed (when required)
- Test implementation gaps (missing test types from strategy)
- Regressions detected (existing tests broken)
- Documentation-implementation mismatches

**BLOCKED ⛔** - Sprint cannot proceed (external dependency issue)
- Database unavailable (no credentials, network down)
- Test infrastructure broken (driver conflicts, environment issues)
- Upstream library bug (blocking P0 feature)

### Feature Priority Levels

**P0 (Must Have)**: Blocking for sprint approval
- Sprint fails if ANY P0 feature rejected
- Both automated AND manual tests must pass
- No P0 deferrals allowed

**P1 (Nice to Have)**: Desirable but not blocking
- Sprint can approve with P1 features deferred
- Deferral requires clear justification (technical limitation, complexity)
- User communication plan required for deferred P1

**P2 (Stretch Goals)**: Bonus if delivered
- Completely optional
- No impact on sprint verdict if skipped

### Verdict Examples from Past Sprints

**Sprint 20 Iteration 3: APPROVED**
```
Automated: 290/290 PASS ✅
Manual: 2/2 PASS ✅
User Validation: "Bravo!!!" ✅
Verdict: APPROVED ✅
```

**Sprint 20 Iteration 1: REJECTED**
```
Automated: 290/290 PASS ✅
Manual: 0/2 PASS ❌ (logo wrong, banner present)
User Validation: "Still same issue" ❌
Verdict: REJECTED ❌
Reason: False positive - tests validated wrong layer
```

**Sprint 22 Iteration 1: REJECTED**
```
Automated: 266/266 unit tests PASS ✅
Integration: 0 tests (strategy required 6) ❌
PTY: 19/19 PASS ✅
Verdict: REJECTED ❌
Reason: Test implementation gap - missing integration tests
```

**Sprint 21 Feature 3: DEFERRED**
```
Investigation: Complete ✅
Technical Limitation: reedline Issue #624 (no MenuAccept event) ⛔
Workaround: Press ENTER (acceptable) ✅
User Communication: Templates prepared ✅
Verdict: P1 feature DEFERRED (not blocking) ✅
```

---

## Test Implementation Process

### Phase 1: Test Strategy Creation (quality-validator)

**Input**: Sprint planning document, specifications, design docs

**Output**: `tests/strategy/sprint-N-test-strategy.md`

**Activities**:
1. Analyze each feature's characteristics (interaction type, dependencies, risks)
2. Derive required test types using decision tree
3. Assess false positive risk (LOW/MEDIUM/EXTREMELY HIGH)
4. Document automation limitations explicitly
5. Define verdict criteria (what does APPROVED require?)
6. Estimate test counts (unit/integration/PTY/manual)

**Quality Gate**: Strategy covers ALL features, no hand-waving about test types.

### Phase 2: Test Implementation (rust-teradata-architect)

**Input**: Test strategy document

**Output**: Implemented tests in `src/`, `tests/`

**Activities**:
1. Read test strategy completely
2. For each feature, implement ALL required test types
3. Run tests locally and verify 100% pass rate
4. Complete test implementation checklist
5. Request quality-validator review

**Quality Gate**: Checklist complete (see `docs/testing/checklist.md`).

### Phase 3: Test Execution & Review (quality-validator)

**Input**: Implemented tests, test strategy

**Output**: Test report, verdict (APPROVED/REJECTED/BLOCKED)

**Activities**:
1. Verify test implementation matches strategy (all test types present)
2. Execute all automated tests (unit, integration, PTY)
3. Review manual test procedures (if applicable)
4. Document test results with execution proof
5. Issue verdict with justification

**Quality Gate**: All tests executed (not just code reviewed), evidence captured.

### Phase 4: Manual Validation (user or coordinator)

**Input**: Manual test procedures from `tests/cases/MANUAL-*.md`

**Output**: Evidence (screenshots, command output), pass/fail verdict

**Activities**:
1. Follow manual test procedures step-by-step
2. Capture evidence (screenshots for visual, command output for behavior)
3. Compare actual results to expected results
4. Issue pass/fail for each manual test
5. Provide final verdict (APPROVED if all pass)

**Quality Gate**: Evidence captured for audit trail.

---

## Manual Testing Guidelines

### When Manual Testing Required

**MANDATORY for**:
- User-facing bug fixes (Sprint 20 lesson)
- Keyboard interaction features (Sprint 21 lesson)
- Visual output quality (logo, formatting, colors)
- Subjective UX (smooth, intuitive, responsive)

**OPTIONAL for**:
- Batch mode commands (automated sufficient)
- Pure logic/algorithms (automated sufficient)
- Database queries (automated sufficient)

### Manual Test Documentation Format

**Location**: `tests/cases/MANUAL-<feature>.md`

**Required sections**:
1. **Test ID**: Unique identifier (MANUAL-F1, MANUAL-F2)
2. **Feature**: What feature is being validated
3. **Objective**: What behavior is being checked
4. **Prerequisites**: Required setup (database connection, test data)
5. **Steps**: Numbered step-by-step instructions
6. **Expected Results**: Clear description of correct behavior
7. **Actual Results**: Space for tester to record observations
8. **Evidence**: Required screenshots or command output
9. **Pass/Fail**: Explicit verdict

**Example**:
```markdown
# MANUAL-F1: Verify dbc Database in Completion

## Objective
Verify that the `dbc` system database appears in tab completion after FROM/JOIN keywords.

## Prerequisites
- Database connection configured
- `dbc` database exists and is accessible

## Steps
1. Start REPL: `tq repl`
2. Type: `SELECT * FROM dbc.`
3. Press TAB key
4. Observe completion menu

## Expected Results
- Completion menu appears
- `dbc` database is listed
- Tables from `dbc` database shown (e.g., `DatabasesV`, `TablesV`)

## Actual Results
[Tester fills in]

## Evidence
[Screenshot of completion menu]

## Verdict
[ ] PASS - `dbc` appears in completion
[ ] FAIL - `dbc` missing or error
```

### Evidence Requirements

**For visual tests**:
- Screenshot showing full terminal window
- Annotations highlighting relevant areas
- File naming: `manual-f1-completion-menu.png`

**For behavior tests**:
- Command transcript (copy-paste from terminal)
- Full output (not truncated)
- Timestamps if timing-sensitive

**For keyboard tests**:
- Screencast/video (if available)
- Step-by-step screenshots
- Description of keyboard events

### Manual Test Execution

**Best practices**:
1. **Fresh environment**: Clear caches, restart REPL before each test
2. **Follow procedures exactly**: Don't improvise or skip steps
3. **Capture evidence immediately**: Don't test first, then try to reproduce for screenshot
4. **Test edge cases**: Try reasonable variations (e.g., uppercase/lowercase)
5. **Document failures clearly**: Describe what happened vs what should happen

**Time estimation**:
- Simple manual test: 5-10 minutes (single screenshot)
- Complex manual test: 15-30 minutes (multiple scenarios)
- Full manual suite: Plan 1-2 hours for 4-6 tests

---

## Common Pitfalls & Solutions

### Pitfall 1: Test Strategy ≠ Test Implementation

**Problem**: Strategy documents required tests, but implementation is missing them.

**Sprint 22 Example**: Iteration 1 missing integration/PTY tests despite strategy.

**Solution**: Use test implementation checklist (`docs/testing/checklist.md`) before requesting review.

**Prevention**:
- Count tests implemented vs strategy estimates
- Verify each test type present (unit/integration/PTY/manual)
- Run all test types locally before submitting

### Pitfall 2: Testing Wrong Layer

**Problem**: Tests validate data layer, bug exists in presentation layer.

**Sprint 20 Example**: Completion logic tests passed, but menu widget displayed banner.

**Solution**: Test the layer where the bug could exist.

**Prevention**:
- For REPL features, ALWAYS implement PTY tests (not just unit)
- For visual output, include manual validation
- For user-reported bugs, replicate their exact scenario

### Pitfall 3: False Confidence from PTY Tests

**Problem**: PTY tests pass but feature broken for keyboard interaction.

**Sprint 21 Example**: Feature 3 - PTY tests can't distinguish TAB from ENTER.

**Solution**: Document PTY limitations upfront, make manual PRIMARY for keyboard features.

**Prevention**:
- Read PTY limitations section in this guide
- Assess false positive risk BEFORE implementation
- Make manual PRIMARY (not backup) for high-risk features

### Pitfall 4: Deferred Features Documented

**Problem**: User guide describes features that were deferred.

**Sprint 22 Example**: Loading indicator documented but not implemented.

**Solution**: Review user documentation before ship phase, verify only delivered features documented.

**Prevention**:
- Add documentation review to Ship phase
- Cross-check deliverables list against documentation
- Mark deferred features clearly in specifications

### Pitfall 5: Unit Tests Only for Database Features

**Problem**: Database queries only tested with unit tests (mocks).

**Sprint 22 Iteration 1**: Only unit tests for schema commands, missed SQL integration issues.

**Solution**: Database features REQUIRE integration tests with live database.

**Prevention**:
- Check feature type: Does it query database? → Integration tests required
- Don't rely on mocks for SQL validation
- Verify SQL syntax with real Teradata instance

### Pitfall 6: Automated Tests as Only Validation

**Problem**: Rely solely on automated tests for user-facing features.

**Sprint 18 Example**: 286/286 tests passed, bugs shipped to user.

**Solution**: Hybrid testing - automated + manual for user-facing features.

**Prevention**:
- Check feature type: Is it user-facing? → Manual validation required
- User-reported bugs MUST have user validation before closure
- Don't trust automation alone for visual/UX features

---

## Quick Reference

### Test Type Selection

| Feature Type | Unit | Integration | PTY | Manual |
|--------------|------|-------------|-----|--------|
| Batch mode CLI | ✅ | ✅ | ❌ | ❌ |
| REPL keyboard UX | ✅ logic | ⚠️ optional | ⚠️ optional | ✅ PRIMARY |
| REPL visual output | ✅ logic | ❌ | ✅ | ⚠️ recommended |
| Database commands | ✅ | ✅ | ⚠️ if REPL | ❌ |
| Pure logic | ✅ | ❌ | ❌ | ❌ |

### Verdict Decision Tree

```
Are all P0 features delivered?
├─ NO → REJECTED ❌
└─ YES → Do all automated tests pass?
    ├─ NO → REJECTED ❌
    └─ YES → Can tests execute?
        ├─ NO → BLOCKED ⛔
        └─ YES → Does any feature require manual validation?
            ├─ NO → APPROVED ✅
            └─ YES → Did manual validation pass?
                ├─ NO → REJECTED ❌
                ├─ NOT PERFORMED → REJECTED ❌
                └─ YES → APPROVED ✅
```

### Sprint-by-Sprint Evolution

| Sprint | Approach | Iterations | Key Lesson |
|--------|----------|------------|------------|
| 18 | Automated only | 1 (shipped bugs) | Automated insufficient for UX |
| 19 | Manual only | Blocked | Need both automated + manual |
| 20 | Hybrid (reactive) | 3 | Manual validation MANDATORY for bug fixes |
| 21 | Hybrid (proactive) | 1 | False positive risk assessment prevents iterations |
| 22 | Hybrid (mature) | 2 | Test implementation verification needed |
| 23 | Hybrid + checklist | TBD | Prevent test implementation gaps |

---

## References

- Test Implementation Checklist: `docs/testing/checklist.md`
- Test Strategy Template: `tests/strategy/test-strategy-template.md`
- Testing Philosophy: `docs/testing/philosophy.md`
- Testing Approach: `docs/testing/approach.md`
- Sprint 20 Review: `docs/sprints/sprint-20-review.md` (3-iteration crisis learning)
- Sprint 21 Review: `docs/sprints/sprint-21-review.md` (proactive excellence)
- Sprint 22 Review: `docs/sprints/sprint-22-review.md` (test strategy ≠ implementation)
