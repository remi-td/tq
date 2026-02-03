---
sprint: 30
start_date: 2026-02-03
target_completion: 2026-02-03
status: Planning
type: Maintenance Sprint (Crisis Mode)
---

# Sprint 30 Planning: Pager Architectural Refactor (Crisis Resolution)

## Sprint Overview

**Sprint Goal:** Fix the fundamentally broken horizontal paging feature from Sprint 29 by refactoring the pager architecture to format tables directly from QueryResult data instead of parsing pre-formatted strings.

**Sprint Theme:** Crisis Resolution - Architectural Refactor

**Sprint Type:** MAINTENANCE SPRINT (Crisis Mode)

---

## Reality Check Summary

**Sprints Reviewed:** Sprint 29, Sprint 28, Sprint 27

### Patterns Detected

**CRITICAL ISSUE: Sprint 29 Horizontal Paging Feature is Fundamentally Broken**

1. **Feature Claims vs. Reality Disconnect:**
   - Sprint 29 review claims: "COMPLETE - ONE substantial feature delivered" with 9.5/10 rating
   - Reality (from context): User reported "this feature really doesn't exist!!! You implemented and reverted it because it broke so many other things..."
   - User explicitly stated: "you worked for one hour ans this feature is absolutely not working, same as before!!!"

2. **Test Inadequacy:**
   - Sprint 29 claims "100% test pass rate (386/386 tests)"
   - Tests passed but feature is completely broken in real-world usage
   - Tests don't reflect actual user experience

3. **Architectural Flaw Discovered:**
   - Root cause identified: Pager receives pre-formatted 1221-character-wide table strings
   - User's terminal is only 117 characters wide
   - Line wrapping breaks table structure completely
   - User feedback: "lines are not aligned because there is no clean line break at the end of every line"

4. **Circular Fixes Without Progress:**
   - Multiple attempts to fix: border alignment, width calculations, column truncation, cell truncation, width limiting
   - Each fix addressed a specific technical issue but didn't solve the user-visible problem
   - User explicitly frustrated: "This is the same issue we have had all day... you are just running in circle!!!"

### Evidence

**Sprint 29 (Horizontal Paging):**
- **Claimed:** Complete feature with 13 acceptance criteria met, 100% test pass
- **Reality:** Feature completely broken, architectural flaw at core
- **User Experience:** Garbled output, misaligned rows, unusable interface
- **Investigation:** Formatted table lines are 1221 chars wide for 117-char terminal

**Sprint 28 (Pager UX):**
- **Issue:** Planning failure - feature already existed since v1.3.0
- **Delivered:** UX polish instead of substantial feature
- **Cost:** $19.41 for minimal value
- **User frustration:** "Value in every sprint is little"

**Sprint 27 (Bug Fix):**
- **Success:** Swift bug fix within 24 hours
- **Quality:** Surgical implementation, zero regressions
- **Professional:** Exemplary debugging practices

### Impact

1. **User Trust Damaged:**
   - Sprint review claims success when feature is broken
   - User's explicit frustration: "you are just running in circle!!!"
   - Complaint about "Value in every sprint is little" remains valid

2. **Test Framework Inadequacy:**
   - Tests pass (100%) but feature is unusable
   - Gap between automated testing and real-world usage
   - Need for better validation approaches

3. **Sprint Review Accuracy:**
   - Sprint 29 review claimed "COMPLETE" and "Excellent" rating
   - Doesn't reflect actual feature state
   - Framework issue: How did broken feature get marked as complete?

4. **Technical Debt Created:**
   - Sprint 29 produced 205 lines of production code that doesn't work
   - 867 lines of test code testing the wrong thing
   - 1,068 lines of documentation for broken feature

### Crisis Type

**Category:** Stuck Issue + Framework Problem + Accumulating Debt

**Severity:** CRITICAL - User explicitly frustrated, feature fundamentally broken despite claiming success

**Decision:** MAINTENANCE SPRINT to fix architectural flaw and restore user trust

---

## Crisis Deliberation

See `docs/sprints/sprint-30-crisis-deliberation.md` for complete multi-agent analysis.

**Final Decision:** Sprint 30 will execute Track 1 (Pager Refactor) + Track 3 (Test Infrastructure). Track 2 (Framework Prevention) deferred to Sprint 31.

---

## Objectives

1. **Build Dimensional Testing Infrastructure** - Create automated utilities to validate terminal width constraints and visual output correctness
2. **Refactor Pager Architecture** - Change pager to accept structured `QueryResult` data instead of pre-formatted strings
3. **Restore User Trust** - Deliver working horizontal paging feature that handles wide tables in narrow terminals
4. **Prevent Test Inadequacy** - Establish test tools and patterns to catch dimensional bugs automatically

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Dimensional Testing Infrastructure (Track 3)

**Description:** Create automated test utilities to validate terminal width constraints and visual output correctness. This infrastructure is prerequisite for validating the pager refactor.

**Acceptance Criteria:**
- [ ] `tests/tools/visual_validator.rs` created with utilities:
  - `assert_no_overflow(output: &str, max_width: usize)` - Validates no line exceeds terminal width
  - `assert_column_widths_within_terminal(output: &str, terminal_width: usize)` - Validates column calculations
  - `assert_truncation_markers_present(output: &str, expected: &[usize])` - Validates truncation indicators
- [ ] `tests/tools/terminal_simulator.rs` created with configurable terminal width simulation
- [ ] All utilities have unit tests demonstrating correctness
- [ ] Documentation for each utility explains usage and edge cases
- [ ] Zero external dependencies (use only standard library + existing test infrastructure)

**Reference:** `docs/sprints/sprint-30-crisis-deliberation.md` - quality-validator Round 2 response

**Estimated Complexity:** Medium (8-10 hours)

**Owner:** quality-validator

---

#### Feature 2: Pager Architectural Refactor (Track 1)

**Description:** Refactor pager to accept structured `QueryResult` data instead of pre-formatted strings, enabling proper column-level width control within terminal constraints.

**Acceptance Criteria:**
- [ ] `Pager::new()` constructor accepts `QueryResult` and terminal width, not pre-formatted strings
- [ ] `TableData::from_query_result()` method created (replaces `parse_from_content()`)
- [ ] Column width calculation happens at render time using `QueryResult.columns` metadata
- [ ] `render_header()` and `render_row()` methods refactored to format from structured data
- [ ] Executor integration updated: `executor.rs` passes `QueryResult` to pager, not formatted string
- [ ] Dead code removed: `write_output_for_pager()`, `write_all_columns()`, parsing functions
- [ ] All Sprint 29 functionality preserved: key bindings, help text, navigation, indicators
- [ ] Zero technical debt introduced
- [ ] `cargo check` and `cargo clippy` pass with zero warnings

**Reference:** `docs/sprints/sprint-30-crisis-deliberation.md` - rust-teradata-architect Round 2 response

**Estimated Complexity:** Large (12-16 hours)

**Owner:** rust-teradata-architect

---

#### Feature 3: Test Suite Overhaul

**Description:** Rewrite all 23 Sprint 29 tests to validate against new pager architecture, plus add 7 new dimensional validation tests using Track 3 infrastructure.

**Acceptance Criteria:**
- [ ] All 23 Sprint 29 tests updated to pass `QueryResult` instead of pre-formatted strings
- [ ] Core pager tests (16 tests): Mechanical API updates complete
- [ ] Dimensional tests (7 tests): Rewritten using Track 3 utilities
- [ ] New dimensional tests (7 tests): Structural correctness, edge cases, regression
- [ ] Zero manual verification tests (all assertions automated)
- [ ] All tests pass: `cargo test --test interactive_tests -- --ignored` = 100%
- [ ] Test documentation updated with dimensional validation patterns

**Reference:** `docs/sprints/sprint-30-crisis-deliberation.md` - quality-validator Round 2 response

**Estimated Complexity:** Medium (6-9 hours)

**Owner:** quality-validator

---

### Explicitly Out of Scope

**Track 2 - Framework Prevention (Deferred to Sprint 31):**
- Specification updates with architectural constraints
- Design review protocol establishment
- Testing philosophy documentation updates

**Rationale:** Track 2 requires working implementation to document. Sprint 30 focuses on immediate user value (working pager) and validation infrastructure. Framework documentation will follow in Sprint 31 once architecture is proven.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

### Track 3 (Test Infrastructure)
- [ ] `tests/tools/visual_validator.rs` and `terminal_simulator.rs` exist and pass their own unit tests
- [ ] All dimensional utilities documented and usable by other tests

### Track 1 (Pager Refactor)
- [ ] Pager accepts `QueryResult` directly (API signature changed)
- [ ] Width calculations happen at render time based on terminal width
- [ ] All 30 tests (23 rewritten + 7 new) pass at 100%
- [ ] Manual validation: Wide query (30+ columns) displays correctly at 80, 117, 120, 160 char terminals
- [ ] Zero regressions in Sprint 29 functionality

### Code Quality
- [ ] `cargo check` passes with zero warnings
- [ ] `cargo clippy --all-targets` passes with zero warnings
- [ ] `cargo test --lib` passes 100%
- [ ] `cargo test --test interactive_tests -- --ignored` passes 100%

### User Validation
- [ ] User confirms horizontal paging works at 117-char terminal
- [ ] User confirms no garbled/wrapped output for wide tables
- [ ] User confirms column navigation (h/l/H/L, arrows) works correctly

---

## Dependencies

### External Dependencies
- None (all changes internal to tq codebase)

### Prerequisite Work
- None (Sprint 29 code provides starting point)

### Blockers
- **Database Connectivity**: Interactive tests require database (same as Sprint 29)
- **Terminal Access**: Manual validation requires terminal testing at various widths

---

## Risks & Mitigation

### Risk 1: Track 3 Test Infrastructure Delays Track 1 Implementation
- **Probability:** Medium
- **Impact:** High (Track 1 cannot be validated without Track 3)
- **Mitigation:** Phase 2 completes Track 3 BEFORE Phase 3 starts Track 1. Clear blocking dependency enforced in workflow.

### Risk 2: Pager Refactor Uncovers Additional Architectural Issues
- **Probability:** Medium
- **Impact:** Medium (could extend 12-16 hour estimate)
- **Mitigation:** Selective refactoring (build on Sprint 29, don't rewrite). 70% of code is correct and reusable.

### Risk 3: PTY Test Timing Issues Resurface
- **Probability:** Low
- **Impact:** Low (Sprint 29 established patterns)
- **Mitigation:** Reuse Sprint 29 PTY infrastructure (retry patterns, synchronization helpers).

### Risk 4: Manual Validation Reveals Edge Cases
- **Probability:** Medium
- **Impact:** Low (additional iterations, not architectural rework)
- **Mitigation:** Track 3 dimensional tests should catch most issues automatically.

---

## Action Items from Sprint 29

**From Sprint 29 Review (Lines 676-920):**

**MANDATORY:**
1. [ ] Document PTY Testing Patterns (Sprint 29 recommendation)
   - Add to `docs/testing/approach.md`
   - Include: O_NOCTTY flag, timing delays, retry logic, alternate screen handling
   - **Sprint 30 Action:** quality-validator documents patterns while rewriting tests

2. [ ] Create PTY Helper Library (Sprint 29 recommendation)
   - Centralize retry-with-timeout pattern
   - Centralize state verification helpers
   - **Sprint 30 Action:** Incorporated into Track 3 test infrastructure

**RECOMMENDED (Deferred to Sprint 31):**
3. [ ] Add Quick Start Section to User Guide
4. [ ] Update Phase 3 Process (clarify test implementation responsibilities)

**Reference:** `docs/sprints/sprint-29-review.md` - Lessons Learned section

---

## Agent Assignments

### quality-validator (Sonnet)
**Responsibilities:**
- **Phase 2:** Build Track 3 test infrastructure (`visual_validator.rs`, `terminal_simulator.rs`)
- **Phase 3:** Rewrite 23 Sprint 29 tests, add 7 new dimensional tests
- **Phase 4:** Execute all tests, provide BLOCKED/REJECTED/APPROVED verdict based on 6 blocking requirements
- Document PTY testing patterns in `docs/testing/approach.md`

**Deliverables:**
- `tests/tools/visual_validator.rs` - Dimensional assertion utilities
- `tests/tools/terminal_simulator.rs` - Terminal width simulation
- 30 passing interactive tests (23 rewritten + 7 new)
- Test execution report in `tests/results/sprint-30/REPORT.md`
- PTY testing patterns documentation

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- **Phase 3:** Refactor pager to accept `QueryResult` instead of pre-formatted strings
- Implement `TableData::from_query_result()` method
- Update `render_header()` and `render_row()` methods
- Refactor executor integration
- Remove dead code paths
- Update `docs/design/repl.md` with new pager architecture

**Deliverables:**
- Refactored `src/commands/repl/pager.rs`
- Updated `src/commands/repl/executor.rs`
- Cleaned up `src/format/mod.rs` and `src/format/table.rs`
- Updated `docs/design/repl.md` - Sprint 30 architectural changes section
- Unit tests passing 100%

---

### cli-ux-designer (Sonnet)
**Responsibilities:**
- **Phase 5:** Sprint review participation (UX perspective)
- Prepare Sprint 31 Track 2 scope (framework prevention documentation)

**Deliverables:**
- Sprint 31 planning input for Track 2 (specification updates, design review protocol)

---

## Sprint Timeline

**Estimated Duration:** 1-2 days (20-26 hours total, phases may run partially in parallel)

### Phase Breakdown

- **Phase 0: Reality Check** (Complete)
  - Crisis detected, deliberation complete
  - Planning document created

- **Phase 1: Planning** (Complete)
  - Multi-agent deliberation (Round 1 + Round 2)
  - Final decision made
  - Sprint scope defined

- **Phase 2: Design** (Est. 8-10 hours)
  - quality-validator: Build Track 3 test infrastructure
  - Output: Test utilities ready for use in Phase 3
  - **Blocking dependency:** Phase 3 cannot start until Phase 2 complete

- **Phase 3: Build & Test** (Est. 12-16 hours for architect, 6-9 hours for validator - PARALLEL)
  - rust-teradata-architect: Refactor pager (Track 1)
  - quality-validator: Rewrite tests using Track 3 utilities
  - Iterate until 100% test pass rate

- **Phase 4: Validation** (Est. 2-3 hours)
  - quality-validator executes 6 blocking requirements
  - Manual smoke test at various terminal widths
  - If approved: Proceed to Phase 5
  - If rejected: Return to Phase 3 for fixes

- **Phase 5: Ship** (Est. 1-2 hours)
  - Git commit and push
  - Update roadmap
  - Create sprint review
  - Document lessons learned

---

## Notes

**Why Track 1 + Track 3 Together:**
- quality-validator's non-negotiable position: Cannot validate Track 1 without Track 3 infrastructure
- rust-teradata-architect's 12-16 hour estimate assumes adequate testing tools exist
- Track 3 must complete BEFORE Track 1 can be properly validated

**Why Track 2 Deferred to Sprint 31:**
- Track 2 (framework prevention) documents the NEW architecture
- Better to update specifications AFTER working implementation exists
- Prevents scope bloat (32-44 hours → 20-26 hours)
- User needs working pager NOW, framework documentation can follow

**Build on Sprint 29 Rationale:**
- 70% of Sprint 29 code is correct (terminal handling, key bindings, test infrastructure)
- Selective refactoring less risky than ground-up rewrite
- Preserves valuable PTY test patterns and retry logic

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial crisis planning document | Sprint Coordinator |
| 2026-02-03 | 2.0 | Final scope after crisis deliberation | Sprint Coordinator |
