# Sprint 14 Planning: Quality Infrastructure Foundation

**Date:** 2026-01-21
**Type:** Maintenance Sprint (Crisis Response)
**Sprint Duration:** 1 sprint (11-15 hours estimated)
**Sprint Coordinator:** Main Agent

---

## Reality Check Summary

- **Reviewed sprints:** 10, 11, 12
- **Patterns detected:**
  1. **Stuck Issue:** Interactive test framework mentioned across 3 sprints, never implemented
  2. **Build Warnings:** Accumulating clippy warnings (9 warnings identified)
  3. **Documentation Drift:** specifications.md out of sync, Sprint 13 confusion
- **Decision:** Maintenance Sprint (crisis response)
- **Rationale:** Interactive features (tab completion, table display) shipped with bugs that passed unit tests. Test coverage gaps are blocking quality assurance for REPL features.

**Crisis Deliberation:** See `sprint-14-crisis-deliberation.md` for full multi-agent analysis.

---

## Sprint Focus

**Primary Goal:** Establish quality infrastructure for interactive features to prevent UX regressions.

**Key Insight from Crisis Deliberation:**
> "If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate."

This is the contract we broke in Sprint 13. Sprint 14 must restore it.

---

## Objectives

### Objective 1: Clean Build Foundation (P0)

**Goal:** Eliminate all build warnings and enforce quality gates in CI.

**Tasks:**
- Fix all 9 clippy warnings identified:
  - 2 in `src/format/table.rs` (arithmetic style)
  - 1 in `src/commands/repl/highlighter.rs` (dead code path)
  - 5 in `src/commands/repl/metacommands.rs` (needless borrows, `.into()` conversion)
  - 1 in `src/commands/repl/sql_context.rs` (needless borrow)
- Add `#![deny(warnings)]` to lib.rs and main.rs
- Verify `cargo clippy --all-targets --all-features` exits 0
- Verify `cargo test --lib` exits 0

**Acceptance Criteria:**
- [ ] Zero clippy warnings
- [ ] Zero build warnings
- [ ] `#![deny(warnings)]` enforced

**Estimated Effort:** 1-2 hours

---

### Objective 2: Specification Synchronization (P0)

**Goal:** Ensure specifications accurately reflect implemented features.

**Tasks:**
- Update `specifications.md` to resolve Sprint 13 confusion
- Conduct specification audit across all `detailed-specifications/*.md` files
- Mark all features with status: Specified ✓, Implemented ?, Tested ?
- Identify spec/implementation drift from Sprint 13
- Update roadmap with accurate Sprint 13 status

**Acceptance Criteria:**
- [ ] `specifications.md` accurately reflects implementation status
- [ ] All detailed-specifications audited for drift
- [ ] No "Specified but unvalidated" features remain
- [ ] Sprint 13 status documented correctly

**Estimated Effort:** 1-2 hours

---

### Objective 3: Interactive Test Infrastructure (P0)

**Goal:** Build operational test framework for REPL features.

**Tasks:**

**Implementation:**
- Add `expectrl` dependency to Cargo.toml (or equivalent PTY testing framework)
- Create `tests/interactive/` directory structure
- Implement test harness with helper functions
- Create test fixtures for common REPL scenarios
- Set up mock/recorded database responses

**Documentation:**
- Create `tests/README.md` with:
  - Setup instructions for running interactive tests
  - How to write new interactive tests
  - Test fixture format and usage
  - Troubleshooting guide

**Acceptance Criteria:**
- [ ] Interactive test framework operational (expectrl + fixtures)
- [ ] `tests/README.md` documents test infrastructure
- [ ] Helper functions exist for writing interactive tests
- [ ] Test harness can spawn tq REPL and interact with it

**Estimated Effort:** 4 hours

---

### Objective 4: Sprint 13 Validation (P0)

**Goal:** Retroactively validate all Sprint 13 features with interactive tests.

**Tasks:**

Write interactive tests covering **ALL** Sprint 13 acceptance criteria:
1. **Tab Completion Context Awareness**
   - Test: Tab after `SELECT * FROM ` shows database/table names (NOT keywords)
   - Test: Tab after `SELECT ` shows column names (NOT keywords)
   - Test: Tab in keyword position shows keywords

2. **Multi-line Editing**
   - Test: Multi-line SQL input works (statements continue until `;`)
   - Test: Line breaks preserved in history

3. **Table Display**
   - Test: Wide tables truncate columns appropriately
   - Test: "(+n cols)" indicator shows when columns hidden
   - Test: Batch mode shows all columns (no truncation)

4. **Command History**
   - Test: Arrow keys navigate history correctly
   - Test: History persists across sessions

5. **Error Handling**
   - Test: Invalid SQL shows clear error message
   - Test: REPL doesn't crash on error
   - Test: Can continue after error

6. **Metacommands**
   - Test: `/help` shows help text
   - Test: `/exit` or `/quit` exits cleanly
   - Test: `/clear` clears history

**Additional Critical Scenarios:**
- Prompt rendering (verify Teradata orange color)
- Long result sets (verify paging behavior)
- Session state (verify connection info)

**Validation:**
- Run all interactive tests, fix failures
- Measure test coverage baseline (target: >60%)
- Manual smoke test confirms REPL works end-to-end

**Acceptance Criteria:**
- [ ] Interactive tests exist for ALL Sprint 13 acceptance criteria
- [ ] 100% of interactive tests pass
- [ ] Test coverage measured and documented
- [ ] Manual smoke test performed and documented

**Estimated Effort:** 4-6 hours

---

### Objective 5: Process Updates (P0)

**Goal:** Prevent recurrence through updated processes and quality gates.

**Tasks:**

**Definition of Done Updates:**
- Add requirement: "Interactive tests required for REPL features"
- Add quality gate: "Interactive tests pass" is blocking before Phase 4
- Update criteria for "feature complete"

**Agent Instructions:**
- Update Quality Validator agent instructions:
  - Add check in Phase 2: "Does test infrastructure exist for this feature class?"
  - Add authority: Block Phase 3 if infrastructure inadequate
  - Add validation: Verify interactive tests pass in Phase 4
- Update Sprint Coordinator Phase 0 process:
  - Add specification synchronization check
  - Flag "In Progress" items older than 1 sprint

**Documentation:**
- Update `testing-guidelines.md`:
  - Add "Test What Users See" principle
  - Define when to use unit vs integration vs interactive tests
  - Provide examples of each test type
- Create `testing-checklist.md`:
  - Phase 2: Test infrastructure availability check
  - Phase 3: Test writing requirements
  - Phase 4: Test validation checklist

**Acceptance Criteria:**
- [ ] Definition of Done updated
- [ ] Quality Validator agent instructions updated
- [ ] Sprint Coordinator process updated
- [ ] testing-guidelines.md includes interactive testing section
- [ ] testing-checklist.md created

**Estimated Effort:** 2 hours

---

## Success Criteria

Sprint 14 is successful when **ALL** of these conditions are met:

### Build Quality (Blocking)
- [ ] `cargo clippy --all-targets --all-features` exits 0
- [ ] `cargo build --all-targets` exits 0 with zero warnings
- [ ] `#![deny(warnings)]` enforced in lib.rs and main.rs

### Test Infrastructure (Blocking)
- [ ] Interactive test framework operational
- [ ] `tests/README.md` documents test infrastructure
- [ ] Helper functions exist for writing interactive tests
- [ ] Test fixtures available for common scenarios

### Sprint 13 Validation (Blocking)
- [ ] Interactive tests exist for ALL Sprint 13 acceptance criteria
- [ ] 100% of interactive tests pass
- [ ] Manual smoke test confirms REPL works end-to-end

### Coverage & Metrics (Blocking)
- [ ] Test coverage measured and documented (target: >60%)
- [ ] Coverage baseline recorded for future comparison
- [ ] Test execution time <30s for full suite

### Specification Integrity (Blocking)
- [ ] `specifications.md` accurately reflects implementation status
- [ ] All `detailed-specifications/*.md` files audited
- [ ] No "Specified but unvalidated" features remain

### Process Updates (Blocking)
- [ ] Definition of Done includes interactive testing requirements
- [ ] Quality Validator agent instructions updated
- [ ] Sprint Coordinator process includes spec sync check
- [ ] testing-guidelines.md updated
- [ ] testing-checklist.md created

---

## Effort Breakdown

| Objective | Effort | Priority |
|-----------|--------|----------|
| Clean Build Foundation | 1-2 hours | P0 |
| Specification Synchronization | 1-2 hours | P0 |
| Interactive Test Infrastructure | 4 hours | P0 |
| Sprint 13 Validation | 4-6 hours | P0 |
| Process Updates | 2 hours | P0 |
| **Total** | **12-16 hours** | **P0** |

**Contingency:** 2-4 hours for unexpected issues

**Feasibility:** Sprint can be completed in 1 maintenance sprint (1-2 days of focused work)

---

## Out of Scope (Deferred to Sprint 15)

The following work is explicitly **NOT** in Sprint 14:

- Architectural refactoring (trait abstractions for LineEditor, Completer)
- Full mock framework for deterministic testing without database
- CI integration for interactive tests (if requires Teradata container setup)
- Performance benchmarking of REPL operations
- Full retroactive coverage of Sprints 1-12 (only Sprint 13 validated)

**Rationale:** Pragmatic-first approach. Sprint 14 establishes working infrastructure. Sprint 15 can evaluate need for architectural improvements based on empirical data from Sprint 14.

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Interactive tests require live database | Medium | High | Use mock/fixture approach; record real sessions |
| expectrl complexity | Low | Medium | Fallback to rexpect or custom PTY wrapper |
| Coverage target not met | Low | Low | Document actual coverage, adjust target if needed |
| Tests flaky in CI environment | Medium | High | Run locally first; document CI requirements |
| Specification audit reveals major drift | Low | Medium | Document drift, prioritize fixes in Sprint 15 |

---

## Dependencies

**External:**
- None (all work is internal to project)

**Internal:**
- Cargo.toml (add expectrl dependency)
- Existing test infrastructure (build on current patterns)

**Blocking:**
- None (this is maintenance work to unblock future development)

---

## Deliverables

At the end of Sprint 14, the following artifacts will exist:

### Code
- Zero clippy warnings (9 warnings fixed)
- `#![deny(warnings)]` in lib.rs and main.rs
- `tests/interactive/` directory with working test harness
- Interactive tests for all Sprint 13 features

### Documentation
- `tests/README.md` (test infrastructure guide)
- `testing-checklist.md` (validation checklist)
- Updated `testing-guidelines.md` (interactive testing section)
- Updated `specifications.md` (synchronized with reality)
- Updated Definition of Done (interactive testing requirements)

### Process
- Updated Quality Validator agent instructions
- Updated Sprint Coordinator Phase 0 process
- Coverage baseline metrics documented

---

## Next Steps After Sprint 14

1. **Sprint 14 Review:** Document lessons learned and metrics
2. **Consider `/optimize-agents`:** Analyze Sprint 13 failure patterns
3. **Sprint 15 Planning:** Decide on architectural refactoring based on Sprint 14 data
4. **Return to Feature Development:** With confidence in REPL quality infrastructure

---

## Notes from Crisis Deliberation

**Key Quotes:**

cli-ux-designer:
> "The crisis is resolved when: 'If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate.' This is the contract we broke in Sprint 13."

rust-teradata-architect:
> "Shipping quality improvements faster is better than perfect architecture later."

quality-validator:
> "This is the minimum viable solution to resolve the crisis."

**Consensus:** All three agents agreed on the pragmatic-first approach. Architect shifted from advocating 2-sprint refactoring to supporting pragmatic testing now, with optional refactoring in Sprint 15 based on data.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0 | Sprint 14 Planning - Maintenance Sprint (Quality Infrastructure) | Sprint Coordinator |
