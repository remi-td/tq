# Framework Meta-Fix: Test Strategy Validation (General)

**Date:** 2026-01-19
**Problem:** Agents claim "100% tests passing" without validating test strategy matches feature characteristics
**Root Cause:** No forcing function to ensure test approach validates what specifications actually require

---

## The Core Problem (Abstracted)

### What Happened (General Pattern)

1. Specifications define feature behavior
2. Agents implement feature
3. Agents write SOME tests
4. Tests pass (100%)
5. Agents claim "complete"
6. **Gap:** Tests don't validate what specifications actually require
7. User discovers feature broken in real usage

### Why It Happened

**Agents never asked:**
- "What aspects of the specification need validation?"
- "What type of testing validates those aspects?"
- "Do the tests I wrote actually test those aspects?"
- "Is there a gap between what I tested and what the spec requires?"

**Current framework says:**
- ✅ "Write tests"
- ✅ "Run tests"
- ✅ "Verify 100% pass rate"

**Framework doesn't say:**
- ❌ "Derive test strategy from specification characteristics"
- ❌ "Verify tests validate specification requirements"
- ❌ "Identify gaps between tests and requirements"

---

## The Meta-Fix: Test Strategy Derivation & Validation

### Core Principle

**New Rule:** Agents must explicitly derive test strategy from specification characteristics, then validate tests match both strategy and specifications.

### The Test Strategy Document (New Artifact)

**Purpose:** Force agents to think about WHAT needs testing BEFORE writing tests.

**Created by:** quality-validator during test design phase
**File:** `tests/strategy/sprint-N-test-strategy.md`
**Status:** MANDATORY before test execution

---

## Test Strategy Document Format

```markdown
# Sprint N Test Strategy

**Created:** YYYY-MM-DD
**Author:** quality-validator
**Sprint:** Sprint N
**Features:** [list features being tested]

---

## Feature-by-Feature Test Strategy

### Feature: [Feature Name]

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/[file].md` sections X.Y.Z
- Requirements: [List specific requirements from spec]

**Feature Characteristics:**
- **User Interaction Type:** [Interactive PTY | CLI batch | API | Background process | Library]
- **State Management:** [Stateless | Session state | Persistent state]
- **External Dependencies:** [Database | File system | Network | None]
- **Output Type:** [Visual/terminal | Structured data | Side effects]
- **Timing Sensitivity:** [Real-time | Async | Synchronous]
- **Platform Sensitivity:** [Cross-platform | OS-specific | Terminal-dependent]

**Critical Behaviors to Validate:**
1. [Behavior 1 from specification - be specific]
2. [Behavior 2 from specification - be specific]
3. [Behavior 3 from specification - be specific]

#### 2. Test Strategy Derivation

**From feature characteristics, derive required test types:**

**Test Type 1: [e.g., Unit Tests]**
- **Validates:** [What aspect of behavior]
- **Approach:** [How this test type validates it]
- **Rationale:** [Why this test type is necessary]
- **Gap if missing:** [What wouldn't be validated]

**Test Type 2: [e.g., Interactive Tests]**
- **Validates:** [What aspect of behavior]
- **Approach:** [How this test type validates it]
- **Rationale:** [Why this test type is necessary]
- **Gap if missing:** [What wouldn't be validated]

**Test Type N: [e.g., Integration Tests]**
- **Validates:** [What aspect of behavior]
- **Approach:** [How this test type validates it]
- **Rationale:** [Why this test type is necessary]
- **Gap if missing:** [What wouldn't be validated]

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted |
|-----------|------------|-----------|----------------|
| Unit tests | ✅ Yes | Validates internal logic | Logic bugs not caught |
| Interactive tests (PTY) | ✅ Yes | Validates user sees correct output in terminal | Visual bugs, rendering issues, context detection failures |
| Integration tests | ⚠️ If database used | Validates end-to-end with real DB | Database interaction issues |
| Manual tests | ⚠️ For UX validation | Human validates usability | UX issues not caught |
| Performance tests | ❌ No | Not a performance-critical feature | N/A |

#### 4. Specification Coverage Map

**Map each specification requirement to test type(s):**

| Requirement | Source | Test Type(s) | Test Cases |
|-------------|--------|--------------|------------|
| "Tab completion shows databases after FROM" | repl-mode.md §5.6.2 | Interactive (PTY) | TC027, IC001 |
| "No keyword fallback in table context" | repl-mode.md §5.6.2 | Unit + Interactive | TC011, IC001 |
| "Multi-line context preserved" | repl-mode.md §5.6.2 | Interactive (PTY) | IC002 |
| "Completion inserts at cursor position" | repl-mode.md §5.6.2 | Interactive (PTY) | IC003 |

**Coverage Validation:**
- [ ] Every specification requirement maps to at least one test type
- [ ] Every test type is justified by specification requirement
- [ ] No specification requirement lacking test coverage
- [ ] No test type included without specification rationale

#### 5. Test Implementation Plan

**Unit Tests (src/commands/repl/metadata_completer.rs):**
- Test logic: Context detection, cache management, query handling
- Location: Existing test module
- Expected: 13+ tests

**Interactive Tests (tests/interactive_tests.rs):**
- Test visual output: What user sees in terminal when pressing Tab
- Test cursor behavior: Where completion inserts text
- Test multi-line state: Context across line breaks
- Location: New test module to create
- Expected: 3-5 tests covering key scenarios

**Coverage Gap Analysis:**
- Unit tests alone: ❌ INSUFFICIENT - Cannot validate terminal output, cursor position
- Interactive tests alone: ❌ INSUFFICIENT - Cannot validate internal logic
- Both combined: ✅ SUFFICIENT - Full specification coverage

---

## Feature: [Next Feature]

[Repeat same structure for each feature]

---

## Strategy Validation Checklist

Before proceeding to test implementation:

- [ ] Every feature has specification analysis section
- [ ] Every feature lists specific specification requirements
- [ ] Every feature derives test strategy from characteristics
- [ ] Every feature has test type necessity matrix
- [ ] Every feature has specification coverage map
- [ ] Every specification requirement is mapped to test type(s)
- [ ] Every test type is justified by specification requirement
- [ ] No obvious gaps between requirements and test coverage

**If ANY checkbox unchecked:** Test strategy is incomplete. Cannot proceed to test implementation.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Review Status:** [DRAFT | REVIEWED | APPROVED]
**Reviewer:** tq-project-manager
**Approval Date:** YYYY-MM-DD

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with rationale
- ✅ Coverage gaps explicitly identified
- ✅ Implementation plan is clear and complete
```

---

## Updated Agent Responsibilities

### quality-validator: Test Strategy Author

**New Phase: Test Strategy Design (BEFORE test implementation)**

Add to `.claude/agents/quality-validator.md` after line 64:

```markdown
## Phase 1: Test Strategy Design (MANDATORY FIRST STEP)

**CRITICAL:** Before writing ANY test code, you MUST create a test strategy document.

### Step 1: Analyze Feature Characteristics

For each feature in the sprint:

1. **Read the specification in detail:**
   - Location: `docs/builder/detailed-specifications/[relevant].md`
   - Identify: Specific behaviors, requirements, acceptance criteria
   - Extract: Exact specification text (quote it)

2. **Classify feature characteristics:**
   - **User Interaction Type:** How does user interact with this feature?
     * Interactive PTY (REPL, terminal UI)
     * CLI batch (piped, scripted)
     * API (programmatic)
     * Background (daemon, service)
     * Library (code-only, no user interaction)

   - **Observable Behavior:** What does user observe?
     * Visual output in terminal (colors, formatting, layout)
     * Structured data output (JSON, CSV)
     * Side effects (files created, database modified)
     * Performance characteristics (speed, resource usage)

   - **Validation Challenge:** What's hard to test?
     * Visual rendering
     * Cursor position
     * Timing/async behavior
     * Cross-platform differences
     * User experience (subjective)

3. **Identify validation requirements:**
   - What MUST be validated to claim specification is met?
   - What can go wrong that tests should catch?
   - What did previous bugs teach us about this feature type?

### Step 2: Derive Test Strategy

**For each feature characteristic, derive necessary test type(s):**

**Decision Tree:**

```
IF feature has "Interactive PTY" characteristic:
  THEN "Interactive tests (expectrl)" are NECESSARY
  RATIONALE: Unit tests cannot validate terminal output, cursor behavior
  GAP IF OMITTED: Visual bugs, rendering issues, context detection failures

IF feature has "CLI batch" characteristic:
  THEN "Integration tests" are NECESSARY
  RATIONALE: Unit tests cannot validate end-to-end CLI execution
  GAP IF OMITTED: Argument parsing bugs, piping issues, exit code errors

IF feature has "Visual output" characteristic:
  THEN "Interactive tests" OR "Integration tests with output validation" are NECESSARY
  RATIONALE: Unit tests cannot validate formatting, colors, layout
  GAP IF OMITTED: Formatting bugs, layout issues, color rendering problems

IF feature has "Database dependency" characteristic:
  THEN "Integration tests with live database" are NECESSARY
  RATIONALE: Unit tests with mocks cannot validate real database behavior
  GAP IF OMITTED: SQL syntax errors, query performance issues, transaction handling bugs

IF feature has "Performance requirement" characteristic:
  THEN "Performance/benchmark tests" are NECESSARY
  RATIONALE: Functional tests don't measure performance
  GAP IF OMITTED: Performance regressions, resource leaks

IF feature has "Cross-platform requirement" characteristic:
  THEN "Tests on multiple platforms" are NECESSARY
  RATIONALE: Single-platform tests don't catch platform-specific issues
  GAP IF OMITTED: Platform-specific bugs, path separator issues, terminal differences
```

**Key Principle:** Each test type MUST be justified by feature characteristic and specification requirement.

### Step 3: Map Requirements to Test Types

**Create specification coverage map:**

For each requirement in the specification:
1. List the exact requirement (quote from spec)
2. Identify test type(s) that can validate it
3. Justify why that test type is necessary
4. Identify gap if test type is omitted

**Example (Tab Completion):**

| Requirement | Source | Can Unit Test Validate? | Can Interactive Test Validate? | Which is Necessary? |
|-------------|--------|-------------------------|-------------------------------|---------------------|
| "Shows database names after FROM" | repl-mode.md | ❌ No (mocks return data, but user sees nothing) | ✅ Yes (observes terminal output) | Interactive |
| "No keyword fallback in table context" | repl-mode.md | ⚠️ Partial (logic correct, but rendering?) | ✅ Yes (observes what user sees) | Both |
| "Context preserved across lines" | repl-mode.md | ❌ No (reedline state not in unit tests) | ✅ Yes (simulates multi-line in PTY) | Interactive |
| "Cache management logic" | Internal | ✅ Yes (pure logic) | ❌ No (internal implementation) | Unit |

### Step 4: Identify and Document Gaps

**If test strategy omits a test type:**
- Document WHY it's omitted
- Identify WHAT won't be validated
- Assess RISK of omission

**Example:**
```markdown
**Test Type Omitted:** Interactive tests
**Reason:** [None - this is NOT acceptable for Interactive PTY features]
**Gap:** Terminal output, cursor position, multi-line state not validated
**Risk:** HIGH - User experience completely untested, bugs will ship
**Decision:** ❌ CANNOT OMIT - Interactive tests are MANDATORY for this feature
```

### Step 5: Create Test Strategy Document

**File:** `tests/strategy/sprint-N-test-strategy.md`
**Format:** Use template above
**Content:** Complete analysis for every feature

**Quality Gates:**
- [ ] Every specification requirement mapped to test type(s)
- [ ] Every test type justified by requirement or characteristic
- [ ] Gaps explicitly identified and assessed
- [ ] Coverage is demonstrably complete

**Submit for review:** Send to tq-project-manager for strategy validation BEFORE writing tests.

---

## Phase 2: Test Implementation (AFTER strategy approved)

Only proceed after test strategy is approved by tq-project-manager.

[Rest of existing instructions continue...]
```

### tq-project-manager: Test Strategy Validator

**New Responsibility: Validate test strategy BEFORE test implementation**

Add to `.claude/agents/tq-project-manager.md` after line 23:

```markdown
## Pre-Implementation Validation: Test Strategy Review

**When:** After quality-validator creates test strategy document, BEFORE test implementation
**File:** `tests/strategy/sprint-N-test-strategy.md`

### Your Validation Responsibility

You are the **test strategy gatekeeper**. Your job is to ensure quality-validator has thought through the testing approach completely.

### Validation Checklist

#### 1. Specification Analysis Validation

For each feature:

- [ ] Specification references are specific (not vague)
  - ❌ "Uses tab completion"
  - ✅ "repl-mode.md §5.6.2 lines 45-67: Tab completion shows databases after FROM"

- [ ] Feature characteristics are analyzed (not assumed)
  - Must include: User interaction type, output type, dependencies
  - Must classify: Interactive PTY, CLI batch, API, etc.

- [ ] Critical behaviors are extracted from specifications (not invented)
  - Each behavior must reference specific specification section
  - Each behavior must be testable (not vague like "works well")

#### 2. Test Strategy Derivation Validation

For each test type proposed:

- [ ] Test type is **derived from feature characteristic** (not guessed)
  - ❌ "We should test this"
  - ✅ "Feature is Interactive PTY → Interactive tests necessary because unit tests cannot validate terminal output"

- [ ] Rationale explicitly states what this test type validates
  - Must answer: "What aspect of specification does this validate?"
  - Must answer: "Why can't another test type validate this?"

- [ ] Gap analysis shows what's NOT validated if test type omitted
  - Must explicitly state: "If we omit interactive tests, we won't validate [specific behaviors]"
  - Must assess risk: HIGH/MEDIUM/LOW

#### 3. Coverage Completeness Validation

- [ ] Every specification requirement mapped to at least one test type
  - Check specification coverage map
  - Ensure no requirement is "orphaned" (no test type assigned)

- [ ] Every test type is justified by specification requirement
  - No test type included "just because"
  - Each type must trace back to specific requirement

- [ ] Gaps are explicitly acknowledged
  - If test type omitted, reason documented
  - Risk assessment included

#### 4. Logic Consistency Validation

**Ask these questions:**

1. **For Interactive PTY features:**
   - "Does test strategy include interactive tests?"
   - If NO → ❌ REJECT STRATEGY
   - Reason: Interactive PTY features CANNOT be validated without interactive tests

2. **For CLI batch features:**
   - "Does test strategy include integration tests?"
   - If NO → ⚠️ CHALLENGE STRATEGY
   - Reason: End-to-end CLI behavior needs integration tests

3. **For features with visual output:**
   - "Does test strategy include tests that validate what user sees?"
   - If NO → ❌ REJECT STRATEGY
   - Reason: Visual bugs won't be caught

4. **For features with database dependency:**
   - "Does test strategy include tests with live database?"
   - If NO → ⚠️ CHALLENGE STRATEGY
   - Reason: Mock tests don't catch real database issues

### Validation Outcomes

#### APPROVED
Strategy is complete and logical:
- All requirements covered
- Test types justified
- Gaps acknowledged
- Implementation plan clear

**Action:** Allow quality-validator to proceed with test implementation

#### REJECTED
Strategy has critical gaps:
- Requirements not covered
- Test types not justified
- Gaps not acknowledged
- Logic inconsistent with feature characteristics

**Action:** Return to quality-validator with specific issues to address

**Example Rejection:**
```markdown
**Status:** ❌ REJECTED

**Issues:**

1. **Tab Completion (Interactive PTY feature):**
   - Test strategy proposes: Unit tests only
   - Missing: Interactive tests
   - Gap: Terminal output, cursor position, multi-line state not validated
   - **Blocker:** Interactive PTY features MUST have interactive tests
   - **Required:** Add interactive test strategy section with expectrl tests

2. **Export to Clipboard:**
   - Specification requirement: "Copies result to system clipboard"
   - Test coverage map: Shows unit test only
   - Gap: Actual clipboard interaction not tested
   - **Challenge:** How will you validate clipboard works across platforms?
   - **Required:** Justify why unit test with mock clipboard is sufficient, or add integration test

**Cannot approve test implementation until these gaps addressed.**
```

### Sign-off Authority

You have authority to:
- ✅ APPROVE strategy → quality-validator proceeds with implementation
- ❌ REJECT strategy → quality-validator must revise
- ⚠️ CONDITIONAL APPROVAL → quality-validator addresses specific concerns

**You must not:**
- Approve incomplete strategies to "move things along"
- Accept vague rationales like "it's probably fine"
- Skip validation "because we're behind schedule"

---

## Post-Implementation Validation: Test Evidence Review

[This is your existing responsibility - lines 24-119 continue...]

**But now add this validation:**

### Validate Tests Match Strategy

1. **Read test strategy:** `tests/strategy/sprint-N-test-strategy.md`
2. **Read test evidence:** `tests/test-evidence.md`
3. **Compare:** Do implemented tests match strategy?

**For each test type in strategy:**
- Verify tests of that type exist
- Verify tests validate what strategy claimed they would
- Verify coverage matches strategy's coverage map

**If tests don't match strategy:**
- ❌ NOT APPROVED
- Reason: "Tests don't match approved strategy"
- Action: quality-validator must align implementation with strategy
```

### sprint-coordinator: Process Enforcer

**Update Phase 4** in `.claude/skills/sprint-coordinator/SKILL.md`:

**Replace existing Phase 4 (lines 123-146) with:**

```markdown
### Phase 4: Test Execution Phase (You Coordinate)

**Goal:** Validate test strategy, execute tests, verify quality.

**Critical:** Testing now has TWO stages: Strategy → Implementation

#### Stage 1: Test Strategy Validation (NEW)

1. **Launch quality-validator for strategy design:**

```
Task: quality-validator
- Prompt: "Create test strategy document for sprint-N features. Analyze specifications, derive required test types, map requirements to tests, identify gaps. File: tests/strategy/sprint-N-test-strategy.md"
- Expected output: Complete test strategy document
```

2. **Launch tq-project-manager for strategy review:**

```
Task: tq-project-manager
- Prompt: "Review test strategy in tests/strategy/sprint-N-test-strategy.md. Validate test types are derived from feature characteristics, all requirements covered, gaps acknowledged. APPROVE or REJECT with specific reasons."
- Expected output: Strategy validation report (APPROVED/REJECTED)
```

3. **Decision Point:**
   - **Strategy APPROVED:** Proceed to Stage 2 (Test Implementation)
   - **Strategy REJECTED:** Return to quality-validator with issues, repeat Stage 1

**CRITICAL:** Cannot proceed to test implementation without approved strategy.

#### Stage 2: Test Implementation & Execution

1. **Launch quality-validator for test implementation:**

```
Task: quality-validator
- Prompt: "Implement tests according to approved strategy in tests/strategy/sprint-N-test-strategy.md. Create all test types specified. Execute tests. Generate results."
- Expected output: Test results with evidence
```

2. **Validate implementation matches strategy:**
   - Read: `tests/strategy/sprint-N-test-strategy.md` (approved strategy)
   - Read: `tests/test-evidence.md` (what was actually implemented)
   - Compare: Do implemented tests match strategy?

3. **Decision Point:**
   - **Tests match strategy AND all pass:** Proceed to Phase 5 (Sprint Closure)
   - **Tests don't match strategy:** Return to quality-validator to align with strategy
   - **Tests fail:** Launch rust-teradata-architect to fix, return to Stage 2
   - **Strategy was wrong:** Return to Stage 1 to revise strategy
```

---

## Why This Works (Abstracted)

### Forcing Function 1: Explicit Strategy Derivation

**Problem:** Agents skipped thinking about test approach
**Solution:** Test strategy document MUST be created before test code
**Enforcement:** tq-project-manager gates test implementation on strategy approval

**Works for any feature type:**
- REPL feature → Strategy derives interactive tests from "Interactive PTY" characteristic
- Web UI feature → Strategy would derive "browser automation tests" from "Web UI" characteristic
- API feature → Strategy would derive "API integration tests" from "API" characteristic
- Performance feature → Strategy would derive "benchmark tests" from "performance requirement" characteristic

### Forcing Function 2: Specification Traceability

**Problem:** Tests validated "something" but not "specification requirements"
**Solution:** Coverage map links every requirement to test type
**Enforcement:** tq-project-manager verifies every requirement has test coverage

**Universal application:**
- Any feature type
- Any specification format
- Any test approach

### Forcing Function 3: Gap Analysis Requirement

**Problem:** Agents didn't realize tests were missing
**Solution:** Strategy MUST identify what's NOT validated
**Enforcement:** "Test type omitted" requires explicit rationale and risk assessment

**Catches any test gap:**
- Missing interactive tests
- Missing performance tests
- Missing security tests
- Missing any test type

### Forcing Function 4: Evidence vs Strategy Validation

**Problem:** Agents wrote different tests than strategy said
**Solution:** tq-project-manager compares implementation to strategy
**Enforcement:** Mismatch blocks sprint closure

**Prevents:**
- Writing easy tests instead of necessary tests
- Claiming "done" without implementing strategy
- Shortcuts that skip hard test types

### Forcing Function 5: Two-Stage Gating

**Problem:** Agents proceeded to implementation without thinking
**Solution:** Stage 1 (strategy) → gate → Stage 2 (implementation) → gate → Phase 5
**Enforcement:** sprint-coordinator cannot skip stages

**Forces:**
- Thinking before coding
- Validation before proceeding
- Evidence at each gate

---

## How This Solves the REPL Problem (Specifically)

### Sprint 7-12: What Would Have Happened

**Phase 4, Stage 1: Test Strategy**

1. quality-validator analyzes tab completion specification
2. Identifies characteristic: "Interactive PTY" (user presses Tab, sees output in terminal)
3. Derives test strategy:
   ```markdown
   **Test Type: Interactive Tests (expectrl)**
   - **Validates:** What user sees when pressing Tab
   - **Rationale:** Unit tests mock the database but don't show terminal output
   - **Gap if omitted:** Visual bugs (showing keywords instead of databases) not caught
   ```
4. tq-project-manager reviews strategy
5. Sees: "Interactive PTY feature" + "No interactive tests planned" → ❌ **REJECTS**
6. Reason: "Interactive PTY features require interactive tests to validate terminal output"
7. quality-validator revises strategy to include expectrl tests
8. tq-project-manager approves revised strategy

**Phase 4, Stage 2: Test Implementation**

9. quality-validator implements expectrl tests per strategy
10. Runs interactive tests
11. **Tests FAIL:** Terminal shows "(SQL keyword)" instead of databases
12. quality-validator reports: "Interactive tests failing"
13. sprint-coordinator launches rust-teradata-architect to fix
14. Iterate until tests pass

**Result:** Bug caught in Sprint 7, not Sprint 12.

---

## How This Solves Future Problems (Generally)

### Hypothetical: Web UI Feature

**Sprint N: Add web-based query builder**

**Phase 4, Stage 1:**

1. quality-validator analyzes specification
2. Identifies characteristics:
   - User interaction type: Web UI (browser-based)
   - Observable behavior: Visual layout, button clicks, form validation
   - Validation challenge: JavaScript rendering, cross-browser compatibility
3. Derives test strategy:
   ```markdown
   **Test Type: Browser Automation Tests (Selenium/Playwright)**
   - **Validates:** UI renders correctly, buttons work, form validation
   - **Rationale:** Unit tests of React components don't validate actual browser rendering
   - **Gap if omitted:** Layout bugs, browser-specific issues, JavaScript errors
   ```
4. tq-project-manager reviews: "Web UI feature without browser tests?" → ❌ **REJECTS**

**Result:** Web UI tested with browser automation, not just unit tests.

### Hypothetical: Performance-Critical Feature

**Sprint N: Add query caching**

**Phase 4, Stage 1:**

1. quality-validator analyzes specification
2. Sees requirement: "Cache must return results in <10ms"
3. Identifies characteristic: Performance requirement (timing-sensitive)
4. Derives test strategy:
   ```markdown
   **Test Type: Benchmark Tests (criterion)**
   - **Validates:** Cache performance meets <10ms requirement
   - **Rationale:** Functional tests don't measure timing
   - **Gap if omitted:** Performance regressions not caught
   ```
5. tq-project-manager reviews: "Performance requirement without benchmark tests?" → ❌ **REJECTS**

**Result:** Performance tested with benchmarks, not just functional tests.

---

## Implementation Plan (General Framework)

### Phase 1: Create Test Strategy Template (1 hour)

**File:** `tests/strategy/test-strategy-template.md`
**Contents:** Template from this document
**Purpose:** Standardize strategy format

### Phase 2: Update quality-validator Instructions (2 hours)

**File:** `.claude/agents/quality-validator.md`
**Changes:**
- Add "Phase 1: Test Strategy Design" (before implementation)
- Add decision tree for deriving test types
- Add coverage map requirements
- Add gap analysis requirements

### Phase 3: Update tq-project-manager Instructions (2 hours)

**File:** `.claude/agents/tq-project-manager.md`
**Changes:**
- Add "Pre-Implementation: Test Strategy Review"
- Add strategy validation checklist
- Add approval/rejection criteria
- Add post-implementation strategy alignment check

### Phase 4: Update sprint-coordinator Workflow (1 hour)

**File:** `.claude/skills/sprint-coordinator/SKILL.md`
**Changes:**
- Update Phase 4 to two-stage process
- Add Stage 1: Strategy validation gate
- Add Stage 2: Implementation validation gate

### Phase 5: Backfill Sprint 12 Strategy (Validation - 2 hours)

**Action:** Create `tests/strategy/sprint-12-test-strategy.md` retrospectively
**Purpose:** Validate framework would have caught the issue
**Contents:**
- Analyze tab completion from specifications
- Derive that interactive tests were necessary
- Show that unit tests alone were insufficient
- Demonstrate gap analysis would have surfaced issue

### Phase 6: Apply to Sprint 13 (Operational - 4 hours)

**Action:** Run Sprint 13 with new framework
**Monitor:**
- Does quality-validator create strategy first?
- Does tq-project-manager catch gaps?
- Does framework prevent issues?

---

## Validation Criteria (Meta-Meta-Fix)

**This framework is successful if:**

1. **For REPL features:**
   - Strategy derives interactive tests from "Interactive PTY" characteristic
   - tq-project-manager rejects strategies without interactive tests
   - Bugs caught before user testing

2. **For hypothetical Web UI features:**
   - Strategy would derive browser automation tests
   - tq-project-manager would reject strategies without browser tests
   - Same framework, different test type

3. **For any feature type:**
   - Agents explicitly think about test approach
   - Test types are justified by specifications
   - Gaps are identified and addressed
   - Framework is domain-agnostic

**The test:** If we add a completely new feature type tomorrow, does the framework guide agents to the right test approach?

---

## Commitment

This framework ensures:

1. **Agents recognize gaps** - Strategy document forces explicit thinking
2. **Agents identify what to do** - Decision tree derives test types from characteristics
3. **Agents validate it was done** - Evidence checked against strategy
4. **Tests validate specifications** - Coverage map ensures traceability

**For REPL specifically:**
- Tab completion classified as "Interactive PTY"
- Strategy derives "interactive tests necessary"
- Gap analysis shows "visual bugs not caught without them"
- tq-project-manager rejects strategy without them
- Tests validate what specifications actually require

**For any feature generally:**
- Same process
- Different characteristics
- Different test types
- Same validation rigor

---

**Document Status:** PROPOSED - Awaiting user approval
**Implementation:** 8-10 hours total
**Applicability:** Universal (all feature types)
