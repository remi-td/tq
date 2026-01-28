# Sprint 28 Planning

**Date:** 2026-01-28
**Type:** Feature Sprint - Real Implementation
**Sprint Coordinator:** Main Claude Agent

---

## Reality Check Summary

- **Reviewed sprints:** 27, 26, 25
- **Patterns detected:** None - healthy velocity maintained
- **Decision:** Feature Sprint
- **Rationale:**
  - 3/3 sprints delivered 100% of features
  - Zero technical debt across all 3 sprints
  - Excellent test pass rates (100%)
  - Time to deliver substantial user value

**Team Health:** EXCELLENT - Ready for ambitious feature work

---

## Sprint Context

### Critical Self-Reflection

**Previous sprint pattern:** Delivering small incremental improvements (bug fixes, documentation polish)
**User feedback:** "The value you are delivering in every sprint is little!"
**Sprint 28 response:** Deliver TWO substantial features with real user impact

### GitHub Issues Analysis

**Sprint-Ready Issues:**
- **#11** (priority-high, bug): Cargo warnings pollute REPL startup
  - User experience: Unprofessional, confusing output
  - Impact: Every REPL session startup
  - Fix: Suppress cargo output or fix build.rs

- **#7** (priority-medium, enhancement): Interactive horizontal paging
  - User experience: Cannot explore wide result sets
  - Impact: Data analysts with 20+ column tables
  - Implementation: Full interactive pager with arrow key navigation
  - **CORRECTED ASSESSMENT**: Feature does NOT exist, needs full implementation

### Version Planning

- **Current:** 1.12.1 (Sprint 27 - bug fix patch)
- **Next:** 1.13.0 (Sprint 28 - TWO major features)

---

## Objectives

### P0 - Critical (Must Have)

1. **Interactive Horizontal Paging (#7) - MAJOR FEATURE**
   - Implement full interactive pager mode for wide result sets
   - Arrow key navigation: ← → to scroll columns left/right
   - Column position indicators: `(+N cols)` on borders
   - Exit pager: `q` or `Esc` returns to prompt
   - Status bar: Show current column window and navigation hints
   - Integration: Automatic pager activation for wide tables

2. **Fix REPL Startup Warnings (#11) - HIGH PRIORITY BUG**
   - Suppress cargo build warnings during `cargo run`
   - Clean REPL startup experience
   - Professional appearance for first-time users

---

## Acceptance Criteria

### Feature #7: Interactive Horizontal Paging

- [ ] Pager activates automatically for result sets wider than terminal
- [ ] Arrow keys (← →) scroll columns left/right by 1 column
- [ ] Column position indicators show `(+N cols)` on truncated sides
- [ ] Status bar displays: `Columns X-Y of Z | ← →: scroll | q: exit`
- [ ] `q` or `Esc` exits pager and returns to REPL prompt
- [ ] Vertical scrolling still works (↑ ↓, j k, Space, b, g, G)
- [ ] Combined horizontal + vertical navigation works smoothly
- [ ] `/pager off` disables pager (shows truncated single-page output)
- [ ] Works with all output formats (table only, CSV/JSON are single-line)
- [ ] 100% existing tests pass (no regressions)
- [ ] 10+ new interactive tests validate paging behavior

### Feature #11: Clean REPL Startup

- [ ] No cargo warnings visible during `cargo run -- repl`
- [ ] No "Finished" or "Running" messages visible
- [ ] Only tq logo and connection info displayed
- [ ] Solution works in both dev (cargo run) and release builds
- [ ] Documentation updated if workaround required
- [ ] 100% existing tests pass

---

## Scope

### In Scope - Feature #7 (Interactive Horizontal Paging)

**Core Paging Functionality:**
- Detect wide result sets (total width > terminal width)
- Automatic pager activation for wide tables
- Arrow key handling for horizontal scrolling
- Column windowing logic (show N columns at a time)
- Column position tracking and indicators
- Status bar with navigation hints

**Pager Integration:**
- Extend existing pager (`src/commands/repl/pager.rs`)
- Combine horizontal + vertical paging
- Maintain existing vertical paging behavior
- Exit handling (`q`, `Esc`)

**User Experience:**
- Clear visual indicators of hidden columns
- Smooth scrolling (no flicker or artifacts)
- Intuitive navigation (matches less/vim conventions)

### In Scope - Feature #11 (Startup Warnings)

**Investigation:**
- Identify source of cargo output (build.rs vs cargo itself)
- Determine if dev-only issue or affects release builds

**Fix Options:**
- Option 1: Suppress cargo warnings in build.rs
- Option 2: Redirect stderr during REPL startup
- Option 3: Add cargo flags to documentation
- Option 4: Create run script that suppresses output

**Implementation:**
- Implement best fix option
- Test in development mode
- Verify release builds unaffected
- Update developer documentation

### Out of Scope

- Search within pager (future enhancement)
- Column filtering or hiding (future enhancement)
- Horizontal page jumps (Ctrl-← Ctrl-→) - MVP: single column scroll
- Configurable scroll speed (MVP: 1 column at a time)
- Mouse support (keyboard-first tool)
- Alternative pager libraries (use existing crossterm implementation)

---

## GitHub Issues

### Selected for Sprint

- **#7**: [FEATURE] Horizontal paging of resultsets (priority-medium, enhancement)
  - Status: Sprint-ready
  - Deliverable: Full interactive pager implementation
  - User Impact: HIGH - unlocks wide table exploration

- **#11**: [BUG] Warning and info messages on startup (priority-high, bug)
  - Status: Sprint-ready
  - Deliverable: Clean REPL startup
  - User Impact: HIGH - professional first impression

### Deferred

No other sprint-ready issues at this time.

---

## Dependencies

**Feature #7 Dependencies:**
- Existing pager code (`src/commands/repl/pager.rs`)
- Terminal width detection (already implemented)
- Crossterm for key event handling (already in use)
- Column width calculation logic (exists in table formatter)

**Feature #11 Dependencies:**
- Understanding of build.rs (already present)
- Cargo build process knowledge
- REPL startup sequence

**No external blockers** - All dependencies already in codebase.

---

## Technical Approach

### Feature #7: Interactive Horizontal Paging

**Phase 1: Investigation (30 min)**
- Read existing pager implementation (`src/commands/repl/pager.rs`)
- Understand current column windowing (if any)
- Identify where to add horizontal navigation
- Review table rendering logic

**Phase 2: Core Implementation (4-6 hours)**
- Extend `PagerState` to track horizontal offset
- Implement column windowing logic:
  - Calculate visible column subset based on offset + terminal width
  - Render only visible columns
  - Add `(+N cols)` indicators on truncated sides
- Add arrow key handlers:
  - `KeyCode::Left`: Decrement column offset
  - `KeyCode::Right`: Increment column offset
  - Handle bounds (don't scroll past first/last column)
- Update status bar with column position
- Test with wide result sets (20+ columns)

**Phase 3: Polish (1-2 hours)**
- Status bar improvements (clear navigation hints)
- Edge case handling (single column, exact fit)
- Performance optimization (if needed)

**Phase 4: Testing (2-3 hours)**
- Interactive tests for horizontal navigation
- Regression tests for vertical paging
- Combined navigation tests
- Wide table test cases

### Feature #11: Startup Warnings

**Phase 1: Investigation (30 min)**
- Reproduce issue: `cargo run -- repl`
- Identify source of warnings:
  - Check build.rs output
  - Check cargo's stdout/stderr
- Test release build: `cargo build --release && ./target/release/tq repl`
- Determine if dev-only or affects release

**Phase 2: Fix Implementation (1-2 hours)**
- If build.rs: Suppress warnings in build script
- If cargo: Add `.cargo/config.toml` or run script
- If stderr: Redirect during REPL initialization
- Test fix in development mode
- Verify release build unaffected

**Phase 3: Documentation (30 min)**
- Update CONTRIBUTING.md or developer docs
- Document expected behavior
- Add run commands if workaround needed

---

## Success Metrics

- [ ] 100% P0 objectives delivered (2 features)
- [ ] Zero new technical debt
- [ ] All existing tests pass (386/386)
- [ ] New tests pass (target: 10-15 new tests)
- [ ] User-visible value: Users can explore wide tables interactively
- [ ] Professional startup: Clean REPL experience
- [ ] GitHub issues #7 and #11 closed with implementation details

---

## Risk Assessment

**MEDIUM RISK SPRINT:**

**Risks - Feature #7:**
- **Complexity:** Interactive paging with terminal state management
  - Mitigation: Extend existing pager, don't rewrite
- **Edge cases:** Many terminal sizes and result set dimensions
  - Mitigation: Comprehensive test cases, careful bounds checking
- **Performance:** Rendering wide tables might be slow
  - Mitigation: Profile and optimize if needed

**Risks - Feature #11:**
- **Root cause unclear:** May not be simple fix
  - Mitigation: Investigation phase first, adjust scope if needed
- **Platform-specific:** May behave differently on Mac/Linux/Windows
  - Mitigation: Test on primary development platform, document differences

**Confidence Level:** MEDIUM-HIGH - Substantial features but clear scope

---

## Estimated Effort

**Phase 2 (Design):** 3-4 hours
- cli-ux-designer: Pager UX design, status bar improvements
- rust-teradata-architect: Technical design, architecture assessment
- quality-validator: Test strategy for interactive paging

**Phase 3 (Build & Test):** 12-16 hours
- Horizontal paging implementation: 8-10 hours
- Startup warnings fix: 2-3 hours
- Testing (both features): 4-6 hours
- Documentation updates: 2 hours

**Total:** 15-20 hours (substantial sprint, real value delivery)

---

## Value Proposition

**Why Sprint 28 is different:**

1. **Real User Pain Points**: Both issues address actual user frustration
2. **Substantial Features**: Interactive paging is a major capability unlock
3. **Professional Quality**: Clean startup shows attention to detail
4. **Complete Solutions**: Not incremental polish, but full implementations

**Expected User Impact:**
- DBAs can explore 30+ column tables without scrolling limitations
- First-time users see professional, clean tool startup
- Data analysts can investigate wide result sets interactively
- Tool feels polished and production-ready

---

## Notes

- Sprint 28 is a **value-driven sprint** responding to user feedback
- Interactive horizontal paging is a **major feature** (not minor UX polish)
- Both features deliver **immediate, visible user value**
- Estimated 15-20 hours is appropriate for two substantial features
- This sprint demonstrates commitment to delivering meaningful improvements

---

## Related Documents

- **Specifications:** `docs/specifications/repl.md` (to be updated with paging requirements)
- **Design:** `docs/design/repl.md` (pager architecture)
- **Existing Code:** `src/commands/repl/pager.rs`
- **GitHub Issues:** #7 (paging), #11 (startup)
