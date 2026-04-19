---
sprint: 67
start_date: 2026-04-19
target_completion: 2026-04-19
status: Planning
---

# Sprint 67 Planning: Search in Pager

## Sprint Overview

**Sprint Goal:** Ship less-style forward search in the interactive pager so REPL users can locate values inside large query results without rerunning the query or piping to the shell.

**Sprint Theme:** REPL / Pager Enhancement
**Type:** Feature Sprint

---

## Reality Check Summary

- **Reviewed sprints:** 64, 65, 66.
- **Patterns detected:** None meeting the "crisis" threshold.
  - Sprints 64/65 surfaced two recurring framework issues (interactive-test timeout, doc/code drift); Sprint 66 addressed both via the tiered PTY harness and the Phase 4 Step 1.7 sequential-user-guide rule. Both closures held through Sprint 66's own retrospective.
  - CI had been red on master since Sprint 64 due to `rust 1.95.0` clippy lints promoted by `#![deny(warnings)]`; fixed outside the sprint cycle in commit `4b39973` with a new `scripts/ci-check.sh` pre-push gate wired into Phase 4.
  - P2 follow-ups are tracked in each review and not repeating — healthy backlog hygiene, not accumulating debt.
- **Decision:** Feature Sprint.
- **Rationale:** Framework is healthy, zero-debt discipline holding, and the top backlog item that fits a single-session budget (Search in Pager) builds directly on Sprint 63's pager infrastructure. This sprint also validates two Sprint 66 process investments in a real flow: the Step 1.7 sequential user-guide rule and the new pre-push CI gate.

---

## Objectives

1. Add less-style forward search to the interactive pager: prompt for `/pattern`, highlight matches, jump to next/previous match with `n`/`N`.
2. Update `docs/specifications/repl.md` with a new `#pager-search` section containing REQ-PAGER-SEARCH-* requirements — the backlog currently references this anchor but no spec exists.
3. Exercise the Sprint 66 process investments end-to-end: sequential user-guide authoring (Step 1.7) and `scripts/ci-check.sh` as a blocking pre-push gate.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Forward search in pager

**Description:** In the pager (the interactive viewer shown for query results when paging is enabled, see `src/commands/repl/pager.rs`), pressing `/` opens a prompt in the status bar. The user types a pattern and presses ENTER. The pager:

1. Scans the in-memory result set from the current row forward for the first row whose displayed cell text contains the pattern (case-insensitive by default).
2. Scrolls vertically so that row is visible (preferring top-of-view) and, if the match falls outside the currently visible horizontal column window, scrolls horizontally so the matched cell is in view.
3. Highlights the matched substring inside the cell (reversed colors) and every other match visible in the current viewport.
4. Leaves a status line indicating `Pattern: <pat>  (M matches)` or `Pattern: <pat>  not found`.

Subsequent navigation:
- `n` — jump to the next match (wraps to first match on end-of-results, with a status line wrap notice).
- `N` — jump to the previous match (wraps to last match on start-of-results, with a status line wrap notice).
- `Esc` while the search prompt is active cancels the prompt and returns to normal pager navigation without clearing the previous search.
- Any existing pager key (`q`, `j`, `k`, arrows, `g`, `G`, `h`, `l`, `H`, `L`, `?`) still works while a search is active.

**Acceptance Criteria:**
- [ ] **AC-1 (Prompt):** Pressing `/` in the pager shows a prompt in the status bar starting with `/` and accepts typed characters; backspace deletes; ENTER submits; Esc cancels (returning to the prior view with the prior search, if any, retained).
- [ ] **AC-2 (Forward search):** After submitting `/pattern` with a pattern that matches at least one cell, the pager scrolls to the first match at or after the current row and highlights the matched substring inside that cell.
- [ ] **AC-3 (No match):** After submitting `/pattern` with a pattern that matches nothing, the status bar shows `Pattern: <pat>  not found` and the pager does not scroll.
- [ ] **AC-4 (`n` next):** After a successful search, pressing `n` scrolls to the next match after the current cursor row; if no later match exists, it wraps to the first match and shows `wrapped to first match` on the status bar for one frame.
- [ ] **AC-5 (`N` previous):** After a successful search, pressing `N` scrolls to the previous match before the current cursor row; if no earlier match exists, it wraps to the last match and shows `wrapped to last match` for one frame.
- [ ] **AC-6 (Case sensitivity):** Search is case-insensitive by default (`/foo` matches `Foo`, `FOO`). Appending `\c` to the pattern (e.g. `/Foo\c`) makes the search case-sensitive.
- [ ] **AC-7 (Column scrolling):** When a match falls in a column currently outside the horizontal viewport, the pager scrolls horizontally so the matched cell is in view.
- [ ] **AC-8 (Highlighted rendering):** The matched substring inside each visible cell is rendered with reversed foreground/background. Other (non-search) cell content renders unchanged.
- [ ] **AC-9 (Status bar match count):** After a successful search, the status bar shows `Pattern: <pat>  (M matches)` where M is the total match count across all rows.
- [ ] **AC-10 (Interaction with paging):** Search works correctly when the result set spans multiple pages (more rows than `page_size`), matches in rows beyond the initial viewport are found by `/` and `n`.
- [ ] **AC-11 (Pager exit does not crash):** `q`/`Esc` to exit the pager while a search is active behaves identically to exiting without a search — no terminal-state corruption (RawModeGuard-style cleanup holds).
- [ ] **AC-12 (Help text):** Pressing `?` in the pager shows the help overlay with `/pattern`, `n`, `N` documented alongside existing navigation keys.

**Reference:** New spec section to be authored in Phase 2: `docs/specifications/repl.md#pager-search` (REQ-PAGER-SEARCH-001..REQ-PAGER-SEARCH-012).

**Estimated Complexity:** Medium.

---

### P1 - High Priority (Should Have)

#### Feature 2: `handle_tick_result` extraction + unit test (Sprint 65 P2 follow-up)

**Description:** Extract the watch-loop tick result handling from `src/commands/watch.rs` (the `match render_result { Ok(body) => ..., Err(e) => ... }` block inside `run_watch`) into a pure function `fn handle_tick_result(render_result: RenderResult, last_body: String) -> TickOutcome`. Add a unit test that feeds `Err(...)` directly and asserts the retained body is unchanged and a formatted error-line is returned — closes the AC-8 unit gap flagged in the Sprint 65 review (`docs/sprints/sprint-65-review.md` → Follow-Up Items → P2).

**Acceptance Criteria:**
- [ ] **AC-1:** `handle_tick_result` is a pure (no I/O, no global state) function that returns both what to display and what to retain.
- [ ] **AC-2:** Existing `/sessions --watch`, `/locks --watch`, and `/resources --watch` behaviour is byte-identical to pre-extraction: error frame renders with the red header, retained body shows below, on success the new body replaces the last.
- [ ] **AC-3:** Unit test `test_handle_tick_result_error_retains_last_body` feeds `RenderResult::Err` and asserts the retained body is exactly `last_body` and the formatted error line contains the error message.
- [ ] **AC-4:** Unit test `test_handle_tick_result_success_replaces_body` feeds `RenderResult::Ok(new)` and asserts the retained body becomes `new`.

**Reference:** `docs/sprints/sprint-65-review.md` → Follow-Up Items → second P2 item.

**Estimated Complexity:** Low.

---

### Explicitly Out of Scope

- **Backward search (`?pattern`) as a *prompt prefix*.** `N` already navigates backward through matches of a forward-initiated search; starting a search in reverse is deferred to a future sprint (would add a second prompt path with small additional value).
- **Regex syntax in search patterns.** Sprint 67 ships literal-substring search only. Regex is a potential follow-up if users request it.
- **Search across non-string column types with custom formatting.** Search operates on the displayed cell text (what the user sees), not on the underlying typed value. That is the correct and predictable behavior, and no special handling is required.
- **TC097-A..H full migration to tiered harness.** Sprint 66 explicitly deferred this as a P2 item estimated at 2-3h. Keeping it out of Sprint 67 to preserve single-session budget for the P0 feature; will be picked up in a dedicated follow-up sprint.
- **PMON graphical features (Issues #21, #22, #23).** Require `ratatui` infrastructure; out of single-session budget.

---

## Success Criteria

- [ ] Feature 1 (P0) implemented, all 12 acceptance criteria pass.
- [ ] Feature 2 (P1) implemented if session budget allows, all 4 ACs pass. If deferred, explicitly documented in the review as a clean deferral with no blocking dependency.
- [ ] `scripts/ci-check.sh` passes locally before push (blocking gate per Phase 4 Step 3).
- [ ] CI goes green on push (ci.yml).
- [ ] 100% test pass rate (unit + `--ignored` integration where applicable).
- [ ] Zero clippy warnings under the current CI stable toolchain.
- [ ] `docs/specifications/repl.md` has a new `#pager-search` section with REQ-PAGER-SEARCH-* numbered requirements.
- [ ] `docs/user/repl-guide.md` (or the equivalent user guide for pager) describes `/`, `n`, `N`, `\c` case-sensitivity suffix, and the status-bar formats — written AFTER implementation lands per Phase 4 Step 1.7.
- [ ] Every quoted example string in the user-guide update is `grep`-verified against `src/` source literals (Step 1.7 contract).
- [ ] `docs/roadmap/status.md` updated to mark Pager Search ✅ v1.49.0 (Sprint 67); `docs/roadmap/backlog.md` entry for "Search in Pager" removed.
- [ ] Release tag `v1.49.0` pushed, release workflow succeeds.

---

## Action Items from Previous Sprint

Carried forward from `docs/sprints/sprint-66-review.md` → Follow-Up Items:

- [ ] **P2 (Sprint 65) — tackled here as Feature 2:** Extract `handle_tick_result` as a pure function, add unit test feeding `Err(...)` to close AC-8 unit gap.
- [ ] **P2 (Sprint 66) — deferred:** TC097-A..H migration to `Stage::Query` (est. 2-3h). Not bundled in Sprint 67; slated for a dedicated migration sprint.
- [ ] **P2 (Sprint 66) — applied implicitly:** Test-case documents should be written after code lands, mirroring the Phase 4 Step 1.7 user-guide sequencing rule. Sprint 67 will have quality-validator author TC099 *after* Feature 1 code lands, not in parallel.
- [ ] **P3 (Sprint 65):** Replace `last_body = fresh_body.clone()` with `std::mem::take` — keep on backlog, not in scope for Sprint 67.
- [ ] **P3 (Sprint 65):** Tighten `classify_key` to `modifiers == CONTROL` — keep on backlog, not in scope.

**Reference:** `docs/sprints/sprint-65-review.md`, `docs/sprints/sprint-66-review.md`.

---

## Dependencies

### External Dependencies
- None. The feature is pure REPL / in-memory. No new crates anticipated; the existing `crossterm` event loop, SQL result set in memory, and the already-built column-windowing logic in `pager.rs` are sufficient.

### Prerequisite Work
- Sprint 63 (horizontal column scrolling in pager) — complete.
- Sprint 66 tiered PTY harness — usable if any interactive test of the search flow is authored.

### Blockers
- None identified.

---

## Risks & Mitigation

### Risk 1: Pager search spec has to be authored from scratch in Phase 2, which could burn more design time than an existing-spec feature
- **Probability:** Medium
- **Impact:** Low (<20 min extra design time)
- **Mitigation:** Hand the cli-ux-designer concrete input in the Phase 2 launch prompt: the 12 acceptance criteria above are the functional envelope, the designer's job is to translate them into REQ-PAGER-SEARCH-* numbered requirements with precise wording. No open-ended UX exploration.

### Risk 2: Match-iteration across large result sets could be slow if pre-computed on every `n`/`N`
- **Probability:** Low
- **Impact:** Low-Medium (user-perceptible lag on 10k+ row results)
- **Mitigation:** Pre-compute a `Vec<(row_idx, col_idx, char_range)>` list of matches once per `/pattern` submission and index into it for `n`/`N`. Recompute only when pattern changes. This is the architect's call to confirm during Phase 2 design.

### Risk 3: Reversed-color highlighting could clash with SQL-syntax colors or NULL colors if any exist in cell rendering
- **Probability:** Low
- **Impact:** Low (visual only, not correctness)
- **Mitigation:** Highlight is applied as a final override on the rendered cell substring. Architect confirms during design by reading `pager.rs` render path.

### Risk 4: Interactive test coverage for search will need PTY harness, and the Sprint 66 proof migration only covered `test_repl_startup_and_quit`
- **Probability:** Medium (for authoring new interactive tests)
- **Impact:** Low (unit-level coverage can close most ACs; interactive coverage is the gold but not the only signal)
- **Mitigation:** Quality-validator prioritizes unit tests against a pure `search_result_set` function for AC-2/3/4/5/6/9/10. Interactive coverage via the tiered harness covers AC-1 (prompt) and AC-11 (exit-doesn't-crash). If the interactive test is flaky, we accept unit-level proof for the mechanical ACs and document AC-1 / AC-11 as manually verified, using the Sprint 65 honest-evidence convention.

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Phase 2 — specs only. Do NOT edit `docs/user/*.md` in Phase 2 or 3 (standing rule from Sprint 66 Phase 2/3 process change).**

**Responsibilities:**
- Author new section `docs/specifications/repl.md#pager-search` with REQ-PAGER-SEARCH-001..012 mapping 1:1 to the 12 acceptance criteria above.
- Confirm status-bar format strings (`Pattern: <pat>  (M matches)`, `Pattern: <pat>  not found`, `wrapped to first match`, `wrapped to last match`) are consistent with existing pager status conventions.
- Do NOT touch user-facing docs yet.

**Phase 4 — user-guide only, AFTER implementation lands, per Phase 4 Step 1.7 sequential rule.**

**Deliverables (Phase 2):**
- New spec section with numbered REQ entries.
- Brief UX rationale (2-3 sentences) for case-insensitive-by-default with `\c` opt-in.

**Deliverables (Phase 4):**
- User-guide update with every quoted example string grep-verified against `src/` source literals.

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement Feature 1 (Pager Search) in `src/commands/repl/pager.rs`. Match pre-computation data structure, highlight rendering, prompt input loop.
- Implement Feature 2 (if time permits) — pure-function extraction of `handle_tick_result` in `src/commands/watch.rs`.
- Update `docs/design/repl.md` (or the equivalent design doc for the pager, currently `docs/design/vision.md` has the architecture note) if new internal patterns are introduced.
- Unit-test coverage for every pure function added.

**Deliverables:**
- Working implementation of Feature 1; Feature 2 if in session budget.
- Unit tests co-located with source (`#[cfg(test)]` modules).
- Design doc updates where new patterns are introduced.

---

### quality-validator (Sonnet)
**Phase 3 — test design + execution.**

**Authoring rule (Sprint 66 P2 follow-up applied in Sprint 67):** write test-case documents (`tests/cases/TC099-*`) AFTER the architect's code has landed in a working state, not in parallel. This mirrors the Phase 4 Step 1.7 sequential user-guide rule applied to test-case prose.

**Responsibilities:**
- Design test cases for all 12 Feature 1 ACs and all 4 Feature 2 ACs.
- Execute unit tests: `cargo test --lib`. Execute integration tests: `cargo test --test <...>` and `--ignored` where the test needs the PTY harness.
- Produce test report in `tests/results/sprint-67/REPORT.md` with actual `cargo test` output — not code review.
- Be honest: if an interactive test times out or fails, document it using the Sprint 65 convention (`run and failed` vs `not run` vs `skipped for reason X`).

**Deliverables:**
- `tests/cases/TC099-pager-search.md`.
- Optional `tests/cases/TC100-handle-tick-result.md` if Feature 2 ships.
- `tests/results/sprint-67/REPORT.md` with actual execution output.

---

## Files Involved

### Objective 1: Pager Search (P0)
**Source Files:**
- `src/commands/repl/pager.rs` — add search state, prompt input, match scanning, highlight rendering, `n`/`N` navigation.

**Test Files:**
- Unit tests in `src/commands/repl/pager.rs` `#[cfg(test)]` module (pure search function).
- Interactive test(s) in `tests/interactive_tests.rs` using the tiered PTY harness (`Stage::Prompt`/`Stage::Query`) for AC-1 / AC-11.

**Documentation:**
- `docs/specifications/repl.md` — new `#pager-search` section with REQ-PAGER-SEARCH-*.
- `docs/design/vision.md` or a new `docs/design/repl.md` if patterns warrant — architect's call.
- `docs/user/repl-guide.md` — Phase 4 only, after code lands.

### Objective 2: `handle_tick_result` extraction (P1)
**Source Files:**
- `src/commands/watch.rs` — extract the tick result match arm into a pure `handle_tick_result` function.

**Test Files:**
- Unit tests in `src/commands/watch.rs` `#[cfg(test)]` module feeding `Err(...)` and `Ok(...)` directly.

**Documentation:**
- No spec/user-guide change. Pure internal refactor with test coverage.

---

## Sprint Timeline

**Estimated Duration:** Single session.

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
- **Phase 1: Planning** (This document)
- **Phase 2: Design** (~20-30 min)
  - Parallel: cli-ux-designer writes REQ-PAGER-SEARCH-* spec + architect drafts technical design and assesses feasibility.
- **Phase 3: Implementation** (~60-90 min)
  - Sequential for quality docs: architect ships Feature 1 code first; quality-validator then designs TC099 against the landed code; Feature 2 extraction if time permits; quality-validator executes full test suite.
- **Phase 4: Ship** (~15-20 min)
  - `scripts/ci-check.sh` blocking gate → commit → push → tag v1.49.0 → release workflow.
  - Sequential cli-ux-designer user-guide prose with grep-verified citations.
- **Phase 5: Retrospective** — `/sprint-reviewer` skill.
- **Phase 6: Framework Optimization** — review retro for improvements.

---

## Notes

- Version bump: v1.48.0 → **v1.49.0**.
- This is the first sprint under the new pre-push CI gate (`scripts/ci-check.sh`, added in commit `4b39973`). Phase 4 must invoke it before `git push`, as mandated by `.claude/skills/sprint-coordinator/process/phase4-ship.md` Step 3.
- The `cli-ux-designer` must respect the Sprint 66 rule: no `docs/user/*.md` edits in Phases 2 or 3. User-guide authoring is Phase 4 only.
- Cost target: Sprint 64 ran ~$15, Sprint 65 ~$28, Sprint 66 ~$39. Sprint 67 is a cleaner feature sprint (no crisis deliberation), targeting the $15-25 range.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-19 | 1.0 | Initial Sprint 67 plan | Sprint Coordinator |
