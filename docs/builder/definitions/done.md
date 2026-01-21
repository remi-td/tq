# Definition of Done

**Version:** 1.0.0
**Last Updated:** 2026-01-21
**Purpose:** Sprint completion criteria for tq project
**Applies to:** All feature sprints and maintenance sprints

---

## Overview

A sprint is "done" when all items in this definition are complete. The Sprint Coordinator must validate each item before moving to Phase 4 (Ship).

**Key Principle:** Zero compromise on quality. If any item is incomplete, the sprint is not done.

---

## 1. Feature Complete

### 1.1 All Acceptance Criteria Met

- [ ] Every acceptance criterion in sprint planning has been implemented
- [ ] No partial implementations (feature either complete or not in scope)
- [ ] All specified behavior matches actual implementation
- [ ] No known bugs in new features

### 1.2 Specifications Updated

- [ ] `specifications.md` updated with feature status
- [ ] Relevant `detailed-specifications/*.md` files updated
- [ ] Specification accurately describes actual behavior (not aspirational)
- [ ] Known limitations documented

### 1.3 No Regressions

- [ ] Existing features still work (regression testing passed)
- [ ] No new bugs introduced in unrelated code
- [ ] Performance not degraded
- [ ] User workflows not broken

---

## 2. Quality Gates (BLOCKING)

### 2.1 Test Coverage

**CRITICAL: TESTS MUST BE EXECUTED, NOT CODE REVIEWED**
- [ ] **ALL tests EXECUTED and produced output** - Code review is NOT execution
- [ ] **Unit tests:** 100% execution + 100% pass rate required
- [ ] **Integration tests:** 100% execution + 100% pass rate required
- [ ] **Interactive tests:** 100% execution + 100% pass rate required (if REPL features)
- [ ] **Tests with `#[ignore]` flag:** MUST be run with `cargo test -- --ignored`
- [ ] **Test execution proof included in quality report** - Actual cargo test output required
- [ ] Every acceptance criterion has at least one test
- [ ] Edge cases tested
- [ ] Error conditions tested

**BLOCKING: If tests cannot be executed (no database, no credentials):**
- [ ] Sprint MUST be marked as BLOCKED
- [ ] Cannot ship based on "tests look correct in code review"
- [ ] Must fix environment/setup before proceeding

**CRITICAL REQUIREMENT FOR REPL FEATURES:**
- [ ] **Interactive tests MANDATORY** - Unit tests alone are insufficient for REPL features
- [ ] **Interactive tests MUST BE EXECUTED** - Not code reviewed
- [ ] Interactive tests verify semantic correctness (not just mechanics)
- [ ] Interactive tests use live database (where applicable)
- [ ] Interactive tests validate visual layout (where applicable)
- [ ] Run with: `cargo test --test interactive_tests -- --ignored`

### 2.2 Build Quality

- [ ] **Zero compiler warnings:** `cargo build --all-targets` produces no warnings
- [ ] **Zero clippy warnings:** `cargo clippy --all-targets --all-features` produces no warnings
- [ ] **Code formatted:** `cargo fmt -- --check` passes
- [ ] **Deny warnings enforced:** `#![deny(warnings)]` in lib.rs and main.rs (if sprint requires it)

### 2.3 Manual Validation

- [ ] **Smoke test performed:** Manual testing confirms feature works as user would use it
- [ ] **Visual inspection:** REPL features visually inspected for layout, colors, alignment
- [ ] **Performance acceptable:** No noticeable lag or delays
- [ ] **Error handling verified:** Graceful failure with helpful messages

---

## 3. Documentation Complete

### 3.1 User-Facing Documentation

- [ ] Help text updated: `tq --help` and subcommand help
- [ ] Help examples work when copy-pasted
- [ ] Error messages are clear and actionable
- [ ] Known limitations communicated to user

### 3.2 Developer Documentation

- [ ] Code comments explain non-obvious logic
- [ ] Public API documented with doc comments
- [ ] Architecture documents updated (if structure changed)
- [ ] Test documentation explains test approach

### 3.3 Process Documentation

- [ ] Sprint review document created
- [ ] Lessons learned captured
- [ ] Known issues documented
- [ ] Migration notes (if breaking changes)

---

## 4. Process Compliance

### 4.1 Sprint Workflow Followed

- [ ] Phase 0: Reality Check completed (for all sprints)
- [ ] Phase 1: Sprint planning document created
- [ ] Phase 2: Design specifications created/updated by CLI UX Designer
- [ ] Phase 2: Architecture assessment completed by Rust Architect
- [ ] Phase 3: Implementation by Rust Architect
- [ ] Phase 3: Testing by Quality Validator
- [ ] Phase 4: Validation against this Definition of Done

### 4.2 Artifacts Created

- [ ] Sprint planning document: `docs/builder/sprints/sprint-N-planning.md`
- [ ] Sprint review document: `docs/builder/sprints/sprint-N-review.md`
- [ ] Test report: Test results documented in sprint review
- [ ] All artifacts committed to repository

### 4.3 Quality Validator Verdict

- [ ] Quality Validator has issued **APPROVED** verdict
- [ ] All test reports show 100% pass rate
- [ ] No blocking issues remain
- [ ] Manual validation completed

---

## 5. Technical Debt Management

### 5.1 Zero Technical Debt Policy

- [ ] No "TODO" comments added without issue tracking
- [ ] No commented-out code committed
- [ ] No temporary workarounds left undocumented
- [ ] All identified issues either fixed or explicitly deferred with justification

### 5.2 Debt Documentation

If technical debt is intentionally deferred:

- [ ] Issue documented in `docs/technical-debt.md` (if exists) or sprint review
- [ ] Reason for deferral documented
- [ ] Impact assessed
- [ ] Plan for resolution documented
- [ ] Prioritized in backlog

---

## 6. Version Control & Deployment

### 6.1 Code Committed

- [ ] All code changes committed to git
- [ ] Commit message follows format:
  ```
  <Type>: <Summary>

  <Body explaining what and why>

  Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
  ```
- [ ] No uncommitted changes remain
- [ ] No large binary files committed

### 6.2 Version Management

- [ ] Version number updated in `Cargo.toml` (if release)
- [ ] CHANGELOG updated (if exists)
- [ ] Git tag created for release (if applicable)

### 6.3 Deployment Ready

- [ ] Binary builds successfully: `cargo build --release`
- [ ] Binary tested on target platform(s)
- [ ] No platform-specific issues
- [ ] Deployment documentation updated (if process changed)

---

## Special Requirements by Feature Type

### For REPL Features

**BLOCKING REQUIREMENT:** Interactive tests are mandatory and must pass.

- [ ] Interactive test framework operational
- [ ] Interactive tests written for all acceptance criteria
- [ ] Interactive tests verify semantic correctness (not just mechanics)
- [ ] Interactive tests use live database (where applicable)
- [ ] Manual visual inspection performed
- [ ] Layout and formatting validated
- [ ] Colors and styling verified

**Rationale:** Unit tests cannot validate user experience. Sprint 11 and earlier sprints shipped REPL bugs that passed unit tests but failed in actual use.

### For Batch/CLI Features

- [ ] Exit codes validated (0 = success, 1 = error, 2 = usage)
- [ ] stdout/stderr separation correct (data on stdout, diagnostics on stderr)
- [ ] Pipeline integration tested (stdin input, stdout output)
- [ ] Script-friendly behavior (silent success, verbose errors)

### For Database Features

- [ ] Tested against live Teradata database
- [ ] SQL execution validated
- [ ] Data type handling verified (including NULL)
- [ ] Error conditions tested (connection loss, permission denied, syntax error)
- [ ] Teradata-specific behavior validated (not assumptions from other databases)

---

## Checklist Quick Reference

### Must Have (Blocking)
1. All acceptance criteria implemented
2. 100% test pass rate (unit + integration + interactive for REPL)
3. Interactive tests pass (REPL features - BLOCKING)
4. Zero build warnings (if sprint requires enforcement)
5. Manual smoke test performed
6. Quality Validator APPROVED verdict
7. Sprint review document created
8. All code committed

### Should Have (High Priority)
1. Specifications updated
2. Help text updated
3. Code formatted
4. No technical debt
5. Performance acceptable

### Nice to Have
1. Coverage metrics documented
2. Screenshots for visual features
3. Performance benchmarks

---

## Definition Updates

This definition may be updated during the project. Updates require:
1. Sprint Coordinator approval
2. All agents notified of changes
3. Version number incremented
4. Change documented in history below

---

## Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0.0 | Initial Definition of Done created for Sprint 14. Key addition: Interactive tests mandatory for REPL features (blocking requirement). | CLI UX Designer |

---

## See Also

- [Testing Guidelines](../testing-guidelines.md) - How to design and execute tests
- [Testing Checklist](../testing-checklist.md) - Phase-specific testing requirements
- Sprint Coordinator process: `.claude/skills/sprint-coordinator/process/phase4-ship.md`
- Quality Validator agent: `.claude/agents/quality-validator.md`
