# Sprint 23 Planning

**Date:** 2026-01-23
**Type:** Feature Sprint
**Sprint Coordinator:** Main Claude Agent

---

## Reality Check Summary

**Reviewed Sprints:** 20, 21, 22

**Patterns Detected:**
- ⚠️ Iteration regression: Sprint 21 (1 iteration) → Sprint 22 (2 iterations)
- ⚠️ Documentation quality: Sprint 22 had pattern syntax errors and documented deferred features
- ⚠️ Test implementation gaps: Sprint 22 Iteration 1 missing integration/PTY tests despite clear strategy
- ✅ Test pass rate: Consistently 99-100% across all sprints
- ✅ Technical debt: Zero maintained across all sprints
- ✅ Code quality: Excellent and improving
- ✅ Hybrid testing: Working well, preventing false positives

**Decision:** Feature Sprint

**Rationale:**
- No stuck issues or accumulating debt
- Framework working well (hybrid testing mature)
- Minor process issues are learning opportunities, not crises
- Can be addressed through improved process gates
- Healthy velocity with features shipping at high quality

---

## Sprint 23 Objectives

### Primary Objectives (P0)

1. **Testing Infrastructure Improvements**
   - Create test implementation checklist (prevent Sprint 22 Iteration 1 gaps)
   - Consolidate testing guidelines document
   - Fix integration test driver library conflict (remove `--test-threads=1` workaround)
   - Add documentation review gate to Ship phase

2. **Batch Mode: Output to File**
   - Direct output redirection: `tq query "..." --output results.csv`
   - Better error handling than shell redirection
   - Support all output formats (table, CSV, JSON)
   - Atomic file writing (temp file + rename)

### Secondary Objectives (P1)

3. **Batch Mode: Transaction Control**
   - `--atomic` flag for transaction wrapping
   - Automatic rollback on error
   - Clear transaction status messages
   - Error handling for nested transactions

---

## Acceptance Criteria

### Testing Infrastructure (P0)

- [ ] Test implementation checklist created in `docs/testing/checklist.md`
- [ ] Testing guidelines consolidated in `docs/testing/guidelines.md`
- [ ] Integration test infrastructure fixed (no `--test-threads=1` required)
- [ ] Documentation review step added to Phase 4 process
- [ ] All 297 existing tests pass (no regressions)

### Output to File (P0)

- [ ] `--output <path>` flag implemented for `query` command
- [ ] Supports all formats: table, CSV, JSON
- [ ] Atomic file writing (temp + rename)
- [ ] Clear error messages for write failures
- [ ] File overwrite confirmation (interactive) or `--force` flag
- [ ] Integration tests for file output (all formats)
- [ ] User documentation updated with examples

### Transaction Control (P1)

- [ ] `--atomic` flag implemented for batch mode
- [ ] Automatic BEGIN TRANSACTION before first statement
- [ ] Automatic COMMIT on success, ROLLBACK on error
- [ ] Clear transaction status messages
- [ ] Error handling for transaction failures
- [ ] Integration tests for transaction scenarios
- [ ] User documentation updated with transaction examples

---

## Scope

### In Scope

**Testing Infrastructure:**
- Test implementation verification checklist
- Consolidated testing guidelines (from Sprints 20, 21, 22 learnings)
- Integration test driver loading fix
- Documentation review process improvement

**Batch Mode Output:**
- `--output <path>` flag for file redirection
- All format support (table, CSV, JSON)
- Atomic file operations
- Error handling and user prompts

**Batch Mode Transactions:**
- `--atomic` flag implementation
- BEGIN/COMMIT/ROLLBACK handling
- Transaction error messages
- Basic transaction tests

### Out of Scope

**Explicitly Excluded:**
- Loading indicator for REPL (deferred P1 from Sprint 22 - requires async design)
- Second TAB accepts selection (deferred P1 from Sprint 21 - blocked by reedline Issue #624)
- Variable substitution in SQL (P2 - future sprint)
- Script preprocessing (P2 - future sprint)
- Project config file (P1 - future sprint)
- Additional schema commands (P1 - future sprint)

---

## Dependencies

### External Dependencies

- None blocking - all dependencies are internal implementation decisions

### Internal Dependencies

- Testing infrastructure improvements should complete before other features (inform implementation)
- Output to file feature independent of transaction control
- Transaction control feature builds on existing batch mode foundation

---

## Risk Assessment

### High Risk

**Integration Test Driver Conflict:**
- **Risk:** May not be fixable without library changes
- **Mitigation:** Document investigation thoroughly, accept `--test-threads=1` workaround if necessary
- **Fallback:** Keep workaround, defer fix to future sprint if blocked

### Medium Risk

**Transaction Control Complexity:**
- **Risk:** Edge cases in transaction handling (nested transactions, connection errors)
- **Mitigation:** Start with simple implementation, comprehensive test coverage
- **Fallback:** Defer to Sprint 24 if complexity exceeds sprint capacity

### Low Risk

- Test documentation (low complexity)
- Output to file (well-understood pattern)

---

## Success Metrics

### Minimum Success (Sprint passes if P0 complete)

- Testing infrastructure improvements delivered (checklist, guidelines, process gate)
- Output to file feature delivered and tested
- Zero regressions (all 297 tests pass)
- Zero technical debt introduced

### Stretch Success (Ideal outcome)

- Integration test driver conflict resolved
- Transaction control feature delivered
- All documentation accurate and complete
- User validation confirms features work as expected

---

## Lessons from Previous Sprints

### Sprint 20 Lessons Applied

- Hybrid testing mandatory for user-facing features ✅
- User validation required for bug fixes ✅
- Manual testing PRIMARY for keyboard UX ✅

### Sprint 21 Lessons Applied

- Proactive test strategy prevents false positives ✅
- Test limitations documented upfront ✅
- False positive risk assessment before implementation ✅

### Sprint 22 Lessons Applied

- **NEW:** Test strategy ≠ test implementation - need verification step
- **NEW:** Documentation must match implementation - review before ship
- **NEW:** Deferred features should not be documented until delivered

---

## Phase 2 Preparation

### Agents to Launch (Parallel)

1. **cli-ux-designer** (Sonnet)
   - Update `docs/specifications/batch-mode.md` with output and transaction features
   - Create/update user documentation for new batch mode features
   - Review documentation accuracy (Sprint 22 lesson)

2. **rust-teradata-architect** (Opus)
   - Investigate integration test driver loading issue
   - Assess feasibility of P0 and P1 features
   - Update `docs/design/batch-mode.md` with technical approach

3. **quality-validator** (Sonnet)
   - Create test implementation checklist
   - Consolidate testing guidelines from Sprint 20-22 learnings
   - Design test strategy for Sprint 23 features

### Expected Phase 2 Duration

- Design phase: 1-2 hours (3 agents in parallel)
- Synthesis: 30 minutes (coordinator reviews outputs)

---

## Notes

- Sprint 23 is a hybrid sprint: testing infrastructure improvements + new features
- Testing improvements address Sprint 22 learnings (test verification gap, documentation accuracy)
- Batch mode features are P1 from backlog but fit well with infrastructure focus
- Transaction control is stretch goal - defer if complexity exceeds capacity

