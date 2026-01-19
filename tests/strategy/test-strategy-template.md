# Sprint N Test Strategy Template

**Created:** YYYY-MM-DD
**Author:** quality-validator
**Sprint:** Sprint N
**Features:** [list features being tested]

---

## Instructions for quality-validator

This template guides you through deriving a test strategy from feature specifications. Complete all sections for each feature BEFORE writing any test code.

**Key Principles:**
1. Test strategy derives from feature characteristics (not assumptions)
2. Every test type must be justified by specification requirement
3. Gaps must be explicitly identified and assessed
4. Specifications are the source of truth

---

## Feature-by-Feature Test Strategy

### Feature: [Feature Name]

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/[file].md` sections X.Y.Z
- Secondary: `docs/builder/specifications.md` feature description
- Requirements:
  1. [Quote specific requirement from spec]
  2. [Quote specific requirement from spec]
  3. [Quote specific requirement from spec]

**Feature Characteristics:**

**User Interaction Type:** [Choose one and explain]
- [ ] Interactive PTY (REPL, terminal UI with cursor/colors/rendering)
- [ ] CLI Batch (scripted, piped, non-interactive command execution)
- [ ] Web UI (browser-based interface)
- [ ] API (programmatic interface, library usage)
- [ ] Background Process (daemon, service, scheduled task)
- [ ] Pure Logic (internal algorithm, no user interaction)

**Explanation:** [Why did you classify it this way? What's the primary user interaction?]

**Observable Behavior:** [Check all that apply]
- [ ] Visual output in terminal (colors, formatting, layout, cursor position)
- [ ] Structured data output (JSON, CSV, XML)
- [ ] File system side effects (files created/modified/deleted)
- [ ] Database side effects (records inserted/updated/deleted)
- [ ] Network interactions (HTTP requests, socket connections)
- [ ] Performance characteristics (speed, memory usage, latency)
- [ ] State management (session state, cache, persistence)

**External Dependencies:** [Check all that apply]
- [ ] Database connection (requires live database)
- [ ] File system access (reads/writes files)
- [ ] Network access (API calls, downloads)
- [ ] Terminal/PTY (terminal control sequences, cursor positioning)
- [ ] System clipboard (copy/paste operations)
- [ ] Operating system specific features (Windows vs Linux vs macOS)
- [ ] None (pure logic, no external dependencies)

**Validation Challenges:** [What makes this hard to test?]
- [Challenge 1: e.g., "Visual rendering in terminal requires actual PTY"]
- [Challenge 2: e.g., "Cross-platform clipboard APIs differ"]
- [Challenge 3: e.g., "Async timing behavior is non-deterministic"]

**Critical Behaviors to Validate:** [From specification - be specific]
1. [Behavior 1 - quote from spec with section reference]
2. [Behavior 2 - quote from spec with section reference]
3. [Behavior 3 - quote from spec with section reference]

#### 2. Test Strategy Derivation

**Use the decision tree to derive necessary test types:**

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior, visual rendering

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: End-to-end CLI execution needs validation with real arguments/pipes

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Mocks don't catch SQL syntax errors, query performance issues

IF "Performance characteristics" checked:
  → Benchmark tests (criterion) REQUIRED
  Reason: Functional tests don't measure timing/memory

IF "Operating system specific" checked:
  → Tests on multiple platforms REQUIRED
  Reason: Single-platform tests miss platform-specific bugs
```

**Derived Test Types:**

**Test Type 1: [e.g., Unit Tests]**
- **Validates:** [What specific aspect of behavior - reference spec requirement]
- **Approach:** [How this test type validates it - be specific about technique]
- **Rationale:** [Why this test type is necessary - what does it catch?]
- **Gap if missing:** [What specific bugs/issues would NOT be caught]
- **Necessity:** ✅ REQUIRED | ⚠️ RECOMMENDED | ❌ NOT NEEDED

**Test Type 2: [e.g., Interactive Tests (expectrl)]**
- **Validates:** [What specific aspect of behavior - reference spec requirement]
- **Approach:** [How this test type validates it - be specific about technique]
- **Rationale:** [Why this test type is necessary - what does it catch?]
- **Gap if missing:** [What specific bugs/issues would NOT be caught]
- **Necessity:** ✅ REQUIRED | ⚠️ RECOMMENDED | ❌ NOT NEEDED

**Test Type 3: [e.g., Integration Tests]**
- **Validates:** [What specific aspect of behavior - reference spec requirement]
- **Approach:** [How this test type validates it - be specific about technique]
- **Rationale:** [Why this test type is necessary - what does it catch?]
- **Gap if missing:** [What specific bugs/issues would NOT be caught]
- **Necessity:** ✅ REQUIRED | ⚠️ RECOMMENDED | ❌ NOT NEEDED

[Add more test types as needed]

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates internal logic and algorithms | Logic bugs, edge cases not caught | MUST IMPLEMENT |
| Interactive tests (expectrl) | ✅ REQUIRED | Validates terminal output user sees | Visual bugs, rendering issues, cursor position errors | MUST IMPLEMENT |
| Integration tests | ⚠️ RECOMMENDED | Validates end-to-end with dependencies | Integration issues, configuration bugs | SHOULD IMPLEMENT |
| Manual tests | ⚠️ RECOMMENDED | Human validates subjective UX quality | Usability issues, confusing UX | DOCUMENT TEST CASES |
| Benchmark tests | ❌ NOT NEEDED | Feature has no performance requirements | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: [count] - MUST implement all
- ⚠️ RECOMMENDED test types: [count] - Should implement unless justified
- ❌ NOT NEEDED test types: [count] - Explicitly omitted with rationale

#### 4. Specification Coverage Map

**Map each specification requirement to test type(s) that validate it:**

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| REQ-1 | "Tab completion shows databases after FROM" | repl-mode.md §5.6.2 | Interactive (expectrl) | Only interactive test can observe terminal output | IC001, IC002 |
| REQ-2 | "No keyword fallback in table context" | repl-mode.md §5.6.2 | Unit + Interactive | Unit validates logic, interactive validates what user sees | TC011, IC001 |
| REQ-3 | "Context preserved across line breaks" | repl-mode.md §5.6.2 | Interactive (expectrl) | Multi-line PTY state only in interactive test | IC003 |
| REQ-4 | "Completion inserts at cursor position" | repl-mode.md §5.6.2 | Interactive (expectrl) | Cursor position only testable in PTY | IC004 |
| REQ-5 | "Cache management logic correct" | Internal architecture | Unit | Pure logic, no user-observable behavior | TC012-TC015 |

**Coverage Validation:**
- [ ] Every specification requirement appears in table
- [ ] Every requirement maps to at least one test type
- [ ] Every test type is justified by requirement
- [ ] No orphaned requirements (missing test coverage)
- [ ] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- [List any requirements not fully covered with explanation]
- [List any gray areas where coverage is uncertain]

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**[Test Type]** - [e.g., "Performance/Benchmark Tests"]
- **Reason for omission:** [Why not implementing this type]
- **What won't be validated:** [Specific aspects not covered]
- **Risk assessment:** [HIGH | MEDIUM | LOW]
- **Mitigation:** [If low/medium risk, how do we mitigate?]
- **Revisit criteria:** [Under what conditions would we add this test type?]

**Example:**
```
**Performance/Benchmark Tests**
- **Reason:** Specification has no performance requirements (<Xms timing)
- **What won't be validated:** Query execution speed, memory usage patterns
- **Risk:** LOW - Feature is not performance-critical, no SLA defined
- **Mitigation:** Monitor in production, add benchmarks if performance issues reported
- **Revisit:** If users report slowness or performance requirements added to spec
```

#### 6. Test Implementation Plan

**For each REQUIRED test type, document implementation approach:**

**Test Type: Unit Tests**
- **Location:** `src/[module]/[file].rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** [number] tests
- **Key scenarios to cover:**
  1. [Scenario 1]
  2. [Scenario 2]
  3. [Scenario 3]
- **Mocking strategy:** [What gets mocked, what doesn't]

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** [number] tests
- **Key scenarios to cover:**
  1. [Scenario 1 - be specific about user actions and expected output]
  2. [Scenario 2 - be specific about user actions and expected output]
  3. [Scenario 3 - be specific about user actions and expected output]
- **Implementation notes:** [Any tricky aspects, timing considerations, etc.]

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or `tests/[feature]_integration.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** [number] tests
- **Key scenarios to cover:**
  1. [Scenario 1]
  2. [Scenario 2]
- **Setup requirements:** [Database connection, test fixtures, etc.]

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: [what aspects]
- Interactive tests validate: [what aspects]
- Integration tests validate: [what aspects]
- Combined coverage: [comprehensive | adequate | has gaps]

**Gaps in combined coverage:**
- [Gap 1: e.g., "Cross-browser compatibility not tested (no browser in scope)"]
- [Gap 2: e.g., "Long-running stability not tested (no soak tests)"]

**Acceptance criteria:**
- [ ] All specification requirements have test coverage
- [ ] All test types justified by requirements
- [ ] Combined coverage is sufficient to claim "works as specified"
- [ ] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- [Gap 1 is acceptable because: [reason]]
- [Gap 2 is acceptable because: [reason]]

---

## Strategy Summary

**Total Features Analyzed:** [number]

**Test Types Required:**
- Unit tests: ✅ [feature1, feature2, ...]
- Interactive tests: ✅ [feature1, feature3, ...]
- Integration tests: ⚠️ [feature2, ...]
- Benchmark tests: ❌ [none]

**Estimated Test Count:**
- Unit: [number] tests
- Interactive: [number] tests
- Integration: [number] tests
- Total: [number] tests

**Risk Assessment:**
- HIGH risk gaps: [list or "none"]
- MEDIUM risk gaps: [list or "none"]
- LOW risk gaps: [list or "none"]

**Dependencies Required:**
- Live database: [Yes/No]
- Network access: [Yes/No]
- Specific OS: [Yes/No/Details]
- Other: [list]

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- [ ] Every feature has complete specification analysis section
- [ ] Feature characteristics are classified (not assumed)
- [ ] Test strategy is derived from characteristics (not guessed)
- [ ] Every test type has clear rationale
- [ ] Gap analysis is complete and honest
- [ ] Specification coverage map includes all requirements
- [ ] Every requirement maps to at least one test type
- [ ] Test implementation plan is detailed and actionable
- [ ] Coverage sufficiency is assessed
- [ ] No hand-waving or vague justifications

**If ANY checkbox unchecked:** Strategy is incomplete, do not submit.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** YYYY-MM-DD
**Review Status:** DRAFT
**Submitted for Review:** [Date when sent to tq-project-manager]

**Reviewer:** tq-project-manager
**Review Status:** [PENDING | APPROVED | REJECTED]
**Review Date:** [Date]
**Review Comments:** [tq-project-manager's feedback]

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation

**Approval signature:** [tq-project-manager agent ID and timestamp]
