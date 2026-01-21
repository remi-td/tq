# Sprint 18 Planning: CRITICAL PRODUCTION BUGS - Logo & Tab Completion

**Date:** 2026-01-21
**Type:** Maintenance Sprint (CRISIS)
**Priority:** P0 - BLOCKING PRODUCTION USE

---

## Reality Check Summary

**Reviewed Sprints:** 17, 16, 15

**Patterns Detected:**
- **CRITICAL BUG**: ASCII art logo uses uppercase letters instead of lowercase "tq" (Sprint 17 color fix was incomplete)
- **CRITICAL BUG**: Tab completion completely broken - debug traces removed but core functionality still doesn't work
- **REPEATING ISSUE**: Tab completion has been problematic across multiple sprints (Sprints 7, 8, 9, 11, 13) but keeps breaking

**Decision:** MAINTENANCE SPRINT (CRISIS)

**Rationale:** User reports TWO BLOCKING bugs preventing productive use:
1. Logo branding completely wrong (not lowercase, missing subtitle)
2. Tab completion non-functional (text inserted at wrong position, keyword completion interfering)

These are not new features - they are critical failures in existing functionality that worked in earlier sprints.

---

## Sprint Objectives

### P0 - CRITICAL (BLOCKING)

#### 1. Fix ASCII Art Logo
**Problem:** Logo is uppercase ASCII art when it should be lowercase "tq" text with subtitle

**Current (WRONG):**
```
 ████████   ████
    ██      ██  ██
    ██      ██  ██
    ██      ██ ▄██
    ██       ████

 tq  v1.7.0
```

**Expected (CORRECT):**
```
tq  (in Teradata orange - xterm-256 color 202)
Teradata Query tool v1.7.0
```

**Acceptance Criteria:**
- [ ] Logo shows lowercase "tq" text (NOT ASCII block art)
- [ ] Logo includes subtitle "Teradata Query tool v1.7.0"
- [ ] "tq" is displayed in Teradata orange (xterm-256 color 202)
- [ ] Text is simple and clean (no fancy ASCII art)
- [ ] Banner matches branding guidelines

**Files to Modify:**
- `src/commands/repl/mod.rs` (print_banner function)

**Effort Estimate:** 30 minutes

---

#### 2. Rebuild Tab Completion from Scratch
**Problem:** Tab completion is completely broken - inserts text at wrong position, keyword completion interferes

**User Report:**
- "Press tab and then start typing keyword, text appears. Press enter, it's inserted at beginning of line"
- "Drop the reserved keywords completion and FOCUS ON database and tablenames after FROM/JOIN"
- "This worked like 10 sprints ago"

**Root Cause Analysis Needed:**
- Span calculation is wrong (text inserted at cursor position 0 instead of actual position)
- Keyword completion interfering with contextual completion
- Context detection may be broken

**Acceptance Criteria:**
- [ ] Tab completion for database names after FROM/JOIN works correctly
- [ ] Tab completion for table names after FROM/JOIN works correctly
- [ ] Tab completion for column names in SELECT/WHERE works correctly
- [ ] NO keyword completion (dropped completely for now)
- [ ] Text inserted at CORRECT cursor position (not beginning of line)
- [ ] Span calculation fixed and tested
- [ ] All 3 completion contexts work in isolation

**Approach:**
1. Read tab completion code from Sprint 7/8 when it worked
2. Compare with current broken implementation
3. Identify what broke (likely span calculation)
4. Simplify: Remove ALL keyword completion
5. Focus on metadata completion only (databases, tables, columns)
6. Test each context type independently
7. Validate with live REPL testing

**Files to Modify:**
- `src/commands/repl/metadata_completer.rs` (main completer logic)
- `src/commands/repl/sql_context.rs` (context analysis)

**Effort Estimate:** 4-6 hours

---

## P1 - High Priority

#### 3. Test Tab Completion with Live Database
**Problem:** Need to validate all 3 completion contexts work in production

**Acceptance Criteria:**
- [ ] Test completion after "FROM " → shows databases + tables
- [ ] Test completion after "JOIN " → shows databases + tables
- [ ] Test completion after "SELECT " → shows columns (when table context known)
- [ ] Test completion after "WHERE " → shows columns (when table context known)
- [ ] Test qualified names "DBC." → shows tables in DBC database
- [ ] NO keyword suggestions in any context
- [ ] All completions insert at correct position

**Approach:**
- Manual interactive testing in live REPL
- Test with real Teradata database
- Document test cases

**Effort Estimate:** 1 hour

---

## Sprint Success Criteria

### Definition of Done
- [ ] Logo shows lowercase "tq" with subtitle (no ASCII art blocks)
- [ ] Tab completion works for databases, tables, columns
- [ ] Tab completion inserts text at CORRECT position
- [ ] NO keyword completion
- [ ] All manual tests pass
- [ ] Zero regressions in existing functionality
- [ ] User validates fixes resolve blocking issues

### Quality Gates
- [ ] 100% test pass rate (existing tests)
- [ ] Zero build warnings
- [ ] Zero clippy warnings
- [ ] Manual validation with live database

---

## Sprint Risks

| Risk | Impact | Mitigation |
|------|--------|----------|
| Tab completion span calculation complex | High | Review Sprint 7/8 code when it worked |
| Reedline library limitations | High | Test with simple cases first |
| Multiple failure modes in completion | High | Fix one context at a time |

---

## Sprint Scope: MAINTENANCE ONLY

This is a **Maintenance Sprint** focused on fixing critical bugs. NO new features.

**In Scope:**
- Fix logo display (lowercase text, subtitle)
- Rebuild tab completion (metadata only)
- Test and validate fixes

**Out of Scope:**
- New features
- Documentation updates (unless fixing errors)
- Architecture changes
- Performance optimizations

---

## Phase Execution Plan

### Phase 0: Reality Check ✅ COMPLETE
- Reviewed sprints 15, 16, 17
- Identified repeating tab completion issues
- Detected critical logo bug
- **Decision:** Maintenance Sprint

### Phase 1: Planning ✅ COMPLETE
- Created sprint-18-planning.md
- Defined 2 P0 objectives
- Documented acceptance criteria

### Phase 2: Design (Skip for Emergency Fixes)
- These are bug fixes, not new features
- No design phase needed
- Proceed directly to Phase 3 implementation

### Phase 3: Implementation & Testing
- rust-teradata-architect: Fix logo + rebuild tab completion
- quality-validator: Test with live database
- Iterate until both P0 objectives complete

### Phase 4: Ship
- Validate against Definition of Done
- Git commit with detailed bug fix description
- Push to master

### Phase 5: Retrospective
- Use `/sprint-reviewer` skill
- Document root causes
- Identify process improvements to prevent recurrence

---

## Historical Context

### Tab Completion Timeline
- **Sprint 7**: Tab completion initially implemented
- **Sprint 8**: Bug fixes for database.table qualified names
- **Sprint 9**: Multi-line context support added
- **Sprint 11**: Bug fixes for empty prefix handling
- **Sprint 13**: Debug logging added (now removed)
- **Sprint 17**: Debug traces removed but core still broken

### Root Cause Hypothesis
The completion system has been patched multiple times but never refactored. Span calculation appears to be fundamentally broken. Need to rebuild with correct architecture.

---

## Success Metrics

### Before Sprint 18 (Current State)
- Logo: Uppercase ASCII art blocks ❌
- Tab completion: Text inserted at position 0 ❌
- User productivity: BLOCKED ❌

### After Sprint 18 (Target State)
- Logo: Lowercase "tq" with subtitle ✅
- Tab completion: Databases/tables/columns work ✅
- User productivity: UNBLOCKED ✅

---

## Notes

**CRITICAL:** This sprint is blocking user's productive work. Both bugs must be fixed completely, not partially.

**User Expectation:** "You had this working like 10 sprints ago" - we need to restore that working state.

**Quality Bar:** User must validate that fixes actually work in their workflow. No "code looks correct" - must test with real usage.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0 | Sprint 18 planning - Critical production bug fixes | Sprint Coordinator |
