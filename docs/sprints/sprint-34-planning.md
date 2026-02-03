# Sprint 34 Planning

**Date:** 2026-02-03
**Type:** MAINTENANCE SPRINT (Technical Debt Cleanup)

---

## Reality Check Summary

**Sprints Reviewed:** 31, 32, 33

**Patterns Detected:**
- ✅ **Healthy Velocity** - Three consecutive excellent sprints (8.5, 9.5, 9.0 ratings)
- ✅ **Sustained Quality** - 100% test pass rate maintained across all sprints
- ✅ **Framework Maturity** - Sprint 31 recovery lessons fully integrated into workflow
- ✅ **User Value Delivery** - Consistent HIGH/EXCEPTIONAL value delivery
- ✅ **Cost Efficiency** - $10-20 per sprint (reasonable and predictable)
- ✅ **Honest Assessment** - Transparency maintained without forcing

**Sprint Type Decision: MAINTENANCE SPRINT (Cleanup)**

**Rationale:**
- No crisis detected - framework healthy and mature
- Sprint 33 identified minor technical debt (code duplication, security gaps, doc mismatches)
- Clean foundation before tackling larger P1 features (Config Management, REPL enhancements)
- Small, focused sprint addressing deferred improvements from Sprint 33

**Sprint Health Assessment:** EXCELLENT - Ready for focused technical debt cleanup

---

## Objectives

### 1. Code Quality Improvements
Eliminate code duplication and improve maintainability identified in Sprint 33 technical review.

### 2. Security Hardening
Add SQL identifier quoting to prevent potential injection issues in data sampling commands.

### 3. Documentation Synchronization
Resolve specification/implementation discrepancies for `/peek` command and pager status.

---

## Acceptance Criteria

### Code Quality (Objective 1)
- [ ] AC-1: `format_column_type()` extracted to shared module (`src/utils/teradata_types.rs`)
- [ ] AC-2: Both `sample.rs` and `metacommands.rs` use shared implementation
- [ ] AC-3: Unit tests pass for shared utility module
- [ ] AC-4: No code duplication detected in technical review
- [ ] AC-5: Zero regressions (all 471 tests continue to pass)

### Security (Objective 2)
- [ ] AC-6: SQL identifiers quoted in `/sample` command (`"database"."table"`)
- [ ] AC-7: SQL identifiers quoted in `/peek` command
- [ ] AC-8: SQL identifiers quoted in batch mode (`tq sample`, `tq peek`)
- [ ] AC-9: Unit tests validate quoted identifier generation
- [ ] AC-10: Regression tests verify functionality with special characters in table names

### Documentation (Objective 3)
- [ ] AC-11: `/peek` specification updated to allow `[N]` parameter (REQ-SAMPLE-004.1)
- [ ] AC-12: Pager status badges added to `docs/specifications/repl.md` section headers
- [ ] AC-13: Specification matches implementation behavior
- [ ] AC-14: User documentation reflects accurate `/peek` syntax
- [ ] AC-15: No specification/implementation discrepancies remain

---

## Scope

### In Scope

**Track 1: Code Quality Cleanup**
- Extract `format_column_type()` function to shared module
- Create `src/utils/teradata_types.rs` with proper tests
- Update `src/commands/sample.rs` to use shared utility
- Update `src/commands/repl/metacommands.rs` to use shared utility
- Remove duplicate implementations

**Track 2: Security Hardening**
- Add `quote_identifier()` function to shared utilities
- Update SQL generation in `/sample` command (REPL + batch)
- Update SQL generation in `/peek` command (REPL + batch)
- Add tests for identifier quoting with edge cases (spaces, quotes, special chars)

**Track 3: Documentation Synchronization**
- Update REQ-SAMPLE-004.1 in `docs/specifications/repl.md` to allow optional `[N]` parameter
- Add status badges to pager sections in `docs/specifications/repl.md`
- Update user guide examples to show `/peek [N]` option
- Verify all documentation matches current implementation

### Out of Scope
- New features from P1 backlog (deferred to Sprint 35+)
- Performance benchmarks (criterion integration - deferred)
- Pager manual validation (requires human testing - deferred)
- Track 3 test utilities assessment (visual_validator, terminal_simulator - deferred)
- Large refactoring or architectural changes

---

## GitHub Issues

### Selected for Sprint
No GitHub issues selected - this sprint addresses Sprint 33 follow-up items documented in sprint review.

### Deferred
No open GitHub issues remain (all issues triaged and closed as of Sprint 33).

---

## Dependencies

**None** - All changes are isolated improvements with no external dependencies.

**Build Dependencies:**
- Existing Rust toolchain
- No new crates required

**Testing Dependencies:**
- Existing test infrastructure sufficient
- No new test frameworks needed

---

## Sprint 33 Follow-up Items

This sprint directly addresses recommendations from Sprint 33 Review:

1. ✅ **Extract `format_column_type()` to shared module** (Sprint 33 Technical Review recommendation)
   - Priority: HIGH
   - Impact: Reduces maintenance burden, prevents code drift
   - Effort: 1-2 hours

2. ✅ **Add SQL identifier quoting** (Sprint 33 Technical Review recommendation)
   - Priority: MEDIUM (low risk but good practice)
   - Impact: Security improvement, handles edge cases
   - Effort: 1 hour

3. ✅ **Update `/peek [N]` specification** (Sprint 33 UX Review recommendation)
   - Priority: MEDIUM (documentation accuracy)
   - Impact: Specification matches implementation
   - Effort: 30 minutes

4. ✅ **Add pager status badges to specifications** (Sprint 33 UX Review recommendation)
   - Priority: LOW (documentation clarity)
   - Impact: Improved visibility of experimental status
   - Effort: 15 minutes

**Total Estimated Effort:** 3-4 hours (small, focused sprint)

---

## Success Criteria

**Must Achieve:**
1. Zero code duplication in Teradata type formatting
2. SQL identifiers properly quoted in all data sampling commands
3. Specifications synchronized with implementation
4. 100% test pass rate maintained (471/471 tests)
5. Zero regressions introduced

**Quality Standards:**
- Clean code review (9/10+ technical rating)
- Comprehensive test coverage (existing + new edge cases)
- Complete documentation updates
- Honest assessment of any gaps

**Sprint Health Indicators:**
- Small, focused scope executed efficiently
- Technical debt reduced (not increased)
- Foundation clean for Sprint 35+ feature work
- Framework maturity maintained

---

## Risk Assessment

**Low Risk Sprint** - All changes are isolated improvements with strong test coverage.

**Potential Risks:**
1. **Risk:** Shared module import conflicts
   - **Mitigation:** Careful module structure, verify all imports
   - **Likelihood:** Low
   - **Impact:** Low (easy to fix)

2. **Risk:** SQL quoting breaks existing functionality
   - **Mitigation:** Comprehensive regression tests, test with special characters
   - **Likelihood:** Low
   - **Impact:** Medium (would require fix)

3. **Risk:** Specification updates miss edge cases
   - **Mitigation:** Code review validates spec matches implementation
   - **Likelihood:** Low
   - **Impact:** Low (documentation only)

**Overall Risk:** LOW - Straightforward cleanup sprint

---

## Related Documents

- **Sprint 33 Review:** `docs/sprints/sprint-33-review.md` (source of follow-up items)
- **Current Status:** `docs/roadmap/status.md` (v1.15.0)
- **Backlog:** `docs/roadmap/backlog.md` (P1 features deferred to Sprint 35+)
- **Specifications:** `docs/specifications/repl.md` (data sampling, pager sections)
- **Design:** `docs/design/repl.md` (technical implementation details)
