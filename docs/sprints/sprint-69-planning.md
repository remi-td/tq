# Sprint 69 Planning

**Date:** 2026-05-29
**Type:** Feature Sprint (with mandatory P1 infrastructure fix)
**Version Target:** v1.51.0

---

## Reality Check Summary

- **Reviewed sprints:** 66, 67, 68
- **Patterns detected:**
  - **Stuck Issue (P1, 4th consecutive deferral):** PTY cursor-position limitation — reedline emits `[6n` cursor queries on REPL startup that the PTY harness never answers. Blocks TC097-A..H (watch interactive tests) and TC104 (pager search PTY test). Documented since Sprint 65, deferred in 66, 67, 68. The 4-sprint deferral threshold was passed in Sprint 67; this is now overdue.
  - **Healthy velocity otherwise:** Features shipping, 100% unit pass rate, retroactive zero-debt discipline holding across 5 sprints.
  - **Cost efficiency improving:** Sprint 68 maintenance sprint $12.23. Feature sprints $15-31. Good baseline.
- **Decision: Feature Sprint** with mandatory PTY fix + one bounded UX improvement
- **Rationale:** The PTY root cause has been analyzed and documented extensively; the fix is now well-understood. Continuing to defer costs all future REPL-touching sprints their interactive test coverage. The UX improvement (pager search status bar position context) is bounded, has spec and design already in place from Sprint 67, and pairs naturally with the PTY work (both touch pager infrastructure).

---

## Objectives

1. **Fix PTY cursor-position root cause:** Implement a `[6n` → `[1;1R` synthetic cursor-position response in the PTY harness so reedline's startup cursor detection succeeds and TC097-A..H + TC104 can actually execute.
2. **Pager search status bar enhancement:** When a search is active, display both `Pattern: <pat>  (N matches)` AND a compact row/column context (`Rows X-Y of Z`) in the status bar, composing them width-aware so the status doesn't wipe position context on narrow terminals.

---

## Acceptance Criteria

### Objective 1: PTY Cursor-Position Fix
- [ ] The PTY harness (or spawn command) correctly handles reedline's `[6n` cursor-position query so the REPL prompt appears without the "cursor position could not be read" error loop
- [ ] TC097-A..H (`test_sessions_watch_*`) tests pass on live DB when run with `--ignored`
- [ ] TC104 (`test_pager_search_prompt_shows_match_count`) executes its search assertions (no early-return via guard) and passes on live DB
- [ ] No regression in `cargo test --lib` or `cargo test --all-targets`

### Objective 2: Pager Search Status Bar Position Context
- [ ] When a search is active, the status bar shows `Pattern: <pat>  (N matches)  |  Rows X-Y of Z` (or similar composition) instead of replacing the row context entirely
- [ ] On narrow terminals (<80 cols), the status bar gracefully truncates: search status takes priority, row context drops when there is not enough width
- [ ] `n`/`N` wrap notices (`wrapped to first/last match`) and not-found notices still appear correctly alongside the composed status
- [ ] REQ-PAGER-SEARCH-009.* and related specs updated to reflect the new composed status format
- [ ] Unit test for the composed status bar rendering added

---

## Scope

### In Scope
- PTY harness `[6n` cursor-position response mechanism
- TC097-A..H migration validation (execution proof on live DB)
- TC104 real execution validation
- Pager search status bar: compose `Pattern: ...` with compact row context
- Width-aware status bar composition (graceful truncation)
- Spec and design doc updates for new status bar format

### Out of Scope
- PMON graphical features (#21, #22, #23 — complex TUI, P3)
- `viewport.rs` extraction and `PagerAction` enum (P3 architecture refactors)
- `n`/`N` transient not-found feedback (related but separate, save for Sprint 70)
- Keyring integration, config validation (separate features, unrelated scope)

---

## GitHub Issues

### Selected for Sprint
_No open sprint-ready issues align with this sprint's scope._

### Deferred
- #21, #22, #23 — PMON graphical TUI features (complex, P3)

---

## Dependencies

- `tests/common/pty_harness.rs` — harness to be modified for `[6n` response
- `src/commands/repl/pager.rs` — status bar rendering (`render_status_bar_to_buffer`)
- `docs/specifications/repl.md` — REQ-PAGER-SEARCH-009.* to be updated
- Live DB via `TQ_LOGON` for TC097 and TC104 execution proof

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `[6n` response fix doesn't resolve cursor detection failure | Medium | High | Alternative: configure reedline with `no_cursor_read` flag (if available); or implement mock harness that bypasses reedline startup entirely |
| Status bar composition too complex for single session | Low | Medium | Fall back to simplest version: always show row context; drop `(N matches)` count when narrow |
| TC097 still fails after PTY fix (different root cause) | Low | Medium | PTY dump will identify new root cause; at minimum the fix closes the known cursor-detection loop |
