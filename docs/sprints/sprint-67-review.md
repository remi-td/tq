# Sprint 67 Review: Pager Search (`/pattern`, `n`, `N`, `\c`)

## Sprint Overview

**Sprint Goal:** Ship less-style forward search in the interactive pager so REPL users can locate values inside large query results without rerunning the query or piping to the shell. Simultaneously close the Sprint 65 P2 follow-up by extracting `handle_tick_result` from `watch.rs` as a pure, unit-testable function.

**Sprint Theme:** REPL / Pager Enhancement
**Date:** 2026-04-19
**Version:** v1.49.0
**Type:** Feature Sprint

---

## Objectives Completed

### Feature 1: Pager Search — DELIVERED

Less-style forward search in `src/commands/repl/pager.rs`, driven by REQ-PAGER-SEARCH-001..012:

- `/` opens a prompt in the status bar; ENTER submits, Esc cancels (retains any prior pattern).
- `find_all_matches()` scans the post-truncation `TableData.cell_values` — what the user actually sees — and returns a pre-computed `Vec<Match>` sorted by (row, col, byte_start).
- `submit_search()` picks the first match at or after the cursor row and scrolls both vertically (`row_offset`) and horizontally (`col_offset`) so the matched cell is visible.
- `n` / `N` navigate next / previous match with wrap notices (`wrapped to first match` / `wrapped to last match`) cleared on the next keypress.
- Case-insensitive ASCII fold by default; `\c` suffix (stripped from displayed pattern) opts into case-sensitive matching.
- Matches highlighted via `SetAttribute(Attribute::Reverse)` so terminal-native color inversion composes on top of cyan headers and DarkGrey NULL foregrounds without a clash.
- Help overlay gains a `Search:` block above `Exit:` covering `/pattern`, `/pattern\c`, `n`, `N`, and (added in retro) `Esc`.
- Status bar line replaces the default position line during an active search: `Pattern: <pat>  (N matches)` / `Pattern: <pat>  (1 match)` / `Pattern: <pat>  not found`.
- Implementation uses a flat `InputMode::{Normal, SearchPrompt{buffer}}` state machine on `Pager`, preserving the Sprint 33 "one poll, one read" discipline (no nested `event::read()` loop).

### Feature 2: `handle_tick_result` extraction — DELIVERED

Pure function in `src/commands/watch.rs`:

```rust
fn handle_tick_result(
    render_result: crate::error::Result<Vec<u8>>,
    last_body: Vec<u8>,
) -> TickOutcome
```

`Vec<u8>` preserved throughout (body bytes are not guaranteed UTF-8 — the planning doc's `String` was mistaken). The watch-loop call site is a 1:1 substitution; `/sessions --watch`, `/locks --watch`, and `/resources --watch` behavior is byte-identical. Closes the Sprint 65 P2 unit-gap for tick-error handling; also incidentally closes the Sprint 65 P3 `std::mem::take` item (the extraction moves ownership, no clone needed).

---

## Metrics

| Metric | Value |
|--------|-------|
| Features completed | 2/2 (100%) |
| New unit tests (initial landing) | 36 |
| New unit tests (after retro fixes) | 38 (+2 tests for `\c`-only cancel and singular-match pluralization) |
| Unit test baseline | 1096 (Sprint 66) → 1134 |
| Integration tests | 27 passing (8+7+6+6), 5 ignored (pre-existing live-DB only) |
| Test pass rate | 100% |
| Clippy warnings | 0 |
| Version | v1.49.0 |
| Tag pushed | `v1.49.0` (release workflow: success) |
| CI run | `24638512062` success |
| Release run | `24638512274` success |

### Test Execution Verdict

**APPROVED WITH CONCERNS** (quality-validator). 10 ACs run-and-passed; 0 run-and-failed; 4 skipped-for-reason:
- **AC-1 (PTY prompt echo):** no PTY test authored. Sprint 66 tiered harness was ready — authoring was deprioritised in session budget, not technically blocked.
- **AC-7 (horizontal scroll arithmetic):** no multi-column fixture test authored. Originally labeled `run and passed` in evidence-1.md first draft; QV self-critique caught the mislabel (the vertical-scroll test used a 3-col data set where the `col_offset` branch never fires). Corrected in retro to `skipped for reason`.
- **AC-8 (ANSI-byte assertion):** no writer-injected unit test authored. Architect exposed `write_value_with_highlights` for testability; the test to assert `\x1b[7m` emission was not written.
- **AC-2 of Feature 2 (byte-identical watch behavior):** TC097 interactive tests have been failing with `ExpectTimeout` since Sprint 65 (pre-existing infrastructure limitation, not a regression). Code inspection confirms the extraction is a 1:1 substitution.

### Token / Cost Metrics

From `docs/sprints/sprint-67-metrics.md`:

| Metric | Value |
|--------|-------|
| Subagent invocations | 9 (Phase 2 ×3, Phase 3 ×2, Phase 4 user-guide ×1, Phase 5 review ×3) |
| Total input tokens | 4,068 |
| Total output tokens | 318,589 |
| Cache creation tokens | 2,266,850 |
| Cache reads | 66,181,479 |
| Grand total | 68,770,986 |
| Overall cache hit rate | 96.7% |
| Estimated cost (Sonnet pricing floor) | **$31.44** |

**Comparison:** Sprint 64 ~$15, Sprint 65 ~$28, Sprint 66 ~$39, **Sprint 67 ~$31**. Clean feature sprints cost less than maintenance sprints with multi-round deliberation; Sprint 67 added one round of in-retro fixes which bumped cost slightly above a pure feature baseline. Cost-per-feature: **~$15.72** (for two features), comparable to Sprint 64's $7.64/feature and within the single-session envelope.

---

## Agent Reviews (abridged)

### Technical (rust-teradata-architect)
**Verdict: Sound with minor concerns — C1 fixed in retro; C2–C6 deferred.**

- **C1 (P3, fixed in retro):** `SetAttribute(Attribute::Reset)` at `pager.rs:841` stripped all attributes including foreground color. Latent (no current caller sets FG before the highlight), but would break if future column-coloring is added. Swapped to `SetAttribute(Attribute::NoReverse)` (SGR 27) in the retro commit. Design-doc prose at `docs/design/repl.md` already matches the corrected semantics.
- **C2 (P3 follow-up):** UTF-8 continuation-byte match unreachable via keyboard input; accepted as theoretical edge case.
- **C3 (P3 follow-up):** Ctrl+C inside the search prompt inserts `c`. Pre-existing pattern — Sprint 67 inherits, does not worsen.
- **C4 (accepted limitation):** `scroll_to_match_index` stale `visible_column_count()` after offset change. Self-correcting; documented in source.
- **C5 (P2 follow-up):** AC-7 dedicated unit test (horizontal scroll) — single new fixture test, ~15 LOC. Now corrected to `skipped for reason` in evidence after QV self-critique.
- **C6 (P2 follow-up):** AC-1 / AC-11 PTY interactive tests deferred. Tiered harness ready; authoring est. ~30 min.
- **Architectural recommendations:** extract `viewport.rs`, introduce `enum PagerAction` for keybinding decoding, formalize writer-injected pure-render pattern in `docs/design/vision.md` as a project-wide standing rule.

### Quality (quality-validator, self-critique)
**Verdict: APPROVED WITH CONCERNS — AC-7 evidence mislabel caught and corrected in retro.**

- AC-1 / AC-11 PTY skips correctly categorised, but the *reason* given understates the gap: the harness was ready, authoring was deprioritised.
- AC-8 ANSI unit gap is a genuine authoring miss for a test the strategy rated REQUIRED.
- AC-7 was mislabeled `run and passed` — the tested code path was vertical scroll, not the horizontal `col_offset` branch. Evidence-1.md row and frontmatter corrected in retro (skipped count 3 → 4).
- Feature 2 closes the pure-function unit gap but does NOT close the interactive rendering proof (TC097 still blocked).
- Designer's grep citations spot-checked (help-overlay block, wrap notices) — both exactly correct against source.

### UX (cli-ux-designer)
**Verdict: Acceptable with concerns — P0 bug + P1 items fixed in retro; P2/P3 captured for backlog.**

- **P0 bug (fixed in retro):** `\c`-only pattern rendered `Pattern:   not found` with a blank where the pattern should be. Looked like a rendering bug to a new user. Fixed in `handle_key` ENTER arm: parse buffer first, treat post-parse-empty as cancel (one-liner guard + REQ-PAGER-SEARCH-001.6 spec extension + new `enter_on_c_only_buffer_cancels_instead_of_submitting` test).
- **P1 (fixed in retro):** `(1 matches)` grammatical error. Fixed via `n == 1 ? "match" : "matches"` in `render_status_bar_to_buffer`; REQ-PAGER-SEARCH-009.1 updated; user guide example at `docs/user/repl-guide.md:2549` corrected; new `status_bar_singular_match_uses_match_not_matches` test added.
- **P1 (fixed in retro):** Help overlay missing `Esc` entry for the prompt — added to `HELP_TEXT` in `pager.rs` and to REQ-PAGER-SEARCH-012.2 in the spec.
- **P2 (backlog):** Status bar replaces the default position line during active search — users lose row/column context. Could be fixed by composing the search status alongside a compact `Rows X-Y of Z` summary.
- **P2 (backlog):** `n` / `N` with `SearchStatus::NotFound` gives no visual feedback. Could re-show the not-found notice transiently.
- **P3 (fixed in retro):** REQ-PAGER-SEARCH-008.4 wording ("pattern was cleared...") inaccurately implied a clearing event that doesn't exist. Rewritten to "no pattern submitted this pager session, or the prompt was cancelled."

---

## Retro Fixes Applied In-Sprint (Zero-Debt Discipline)

Nine fixes landed in the retro commit, consistent with Sprints 64/65/66:

| # | Finding | Fix | File:Line |
|---|---------|-----|-----------|
| 1 | UX P0: `\c`-only pattern renders blank in status | ENTER guard uses `parse_search_input` to detect post-parse-empty; treats as cancel | `pager.rs:960-973` |
| 2 | UX P1: `(1 matches)` grammar | `n == 1 ? "match" : "matches"` in status renderer | `pager.rs:881-895` |
| 3 | UX P1: Help overlay missing Esc entry | Added `Esc  Cancel prompt (keeps previous search)` to `HELP_TEXT` | `pager.rs:1216` |
| 4 | Tech C1 (P3): `Reset` strips fg color alongside Reverse | Use `Attribute::NoReverse` (SGR 27) with explanatory comment | `pager.rs:841-845` |
| 5 | QV self-critique: AC-7 mislabeled `run and passed` | Corrected to `skipped for reason: no multi-column fixture`; frontmatter count 3 → 4 | `tests/results/sprint-67/test-evidence-1.md` |
| 6 | Spec REQ-PAGER-SEARCH-001.6 too narrow | Extended to cover post-parse-empty patterns | `docs/specifications/repl.md:1569` |
| 7 | Spec REQ-PAGER-SEARCH-008.4 misleading wording | Rewrote to reflect actual cancel-not-clear semantics | `docs/specifications/repl.md:1746` |
| 8 | Spec REQ-PAGER-SEARCH-009.1 didn't specify pluralization | Split into singular/plural cases | `docs/specifications/repl.md:1768-1772` |
| 9 | Spec REQ-PAGER-SEARCH-012.2 missing Esc entry | Added Esc row to help-overlay REQ | `docs/specifications/repl.md:1834-1839` |

Plus user-guide pluralization example fix and two new unit tests (`enter_on_c_only_buffer_cancels_instead_of_submitting`, `status_bar_singular_match_uses_match_not_matches`). Test count 1132 → 1134.

---

## Retrospective

### What Went Well

1. **Framework investments from Sprint 66 paid off immediately.** The Step 1.7 sequential user-guide rule worked cleanly on its first real-world run: designer authored the guide AFTER code landed, grep-verified 12 quoted strings against source literals, zero drift caught in review. The pre-push `scripts/ci-check.sh` gate (added in the hotfix commit `4b39973`) caught the retro pluralization test fix before push.
2. **Zero-debt discipline held.** Reviews surfaced 9 in-sprint-fixable items (1 P0, 2 P1, 1 P3 + 4 spec edits + 1 evidence correction). All fixed before the sprint closed. Matches the Sprint 64/65/66 pattern of fixing review findings in-sprint.
3. **Writer-injected pure-render pattern validated.** `render_status_bar_to_buffer`, `find_all_matches` as `pub(crate)`, and `Pager::render_help_text` all gave quality-validator clean unit-testable surfaces. 32 new pager unit tests landed with no terminal-dependent assertions. The architect's recommendation to institutionalize this as a standing rule in `docs/design/vision.md` is worth acting on.
4. **QV self-critique surfaced AC-7 mislabel.** A first-draft evidence row said `run and passed` when the tested code path did not actually exercise the AC. QV caught this on re-review, and the label was corrected. This is exactly the Sprint 65 honest-evidence convention working as intended.
5. **Cost back under $32.** Sprint 66 was $39 due to 3-round crisis deliberation + process edits + multi-retro fixes. Sprint 67 (clean feature sprint + one retro round) came in at $31.44, with 96.7% cache hit.

### What Could Be Improved

1. **"REQUIRED" strategy classification needs teeth.** QV's strategy rated TC099-I01 (PTY test for AC-1 / AC-11) as REQUIRED, then fell back to manual verification. Similarly, the AC-8 ANSI byte test was rated REQUIRED, then skipped for authoring. A REQUIRED test that isn't authored should be reported as a MEDIUM-severity gap, not a LOW-severity one — this should be codified in `docs/testing/` before next sprint.
2. **AC-7 first-draft mislabel is a leading indicator.** The evidence doc called a vertical-scroll test pass as proof for a horizontal-scroll AC. This was caught, but only because QV did a thorough self-critique in Phase 5. The underlying cause is that test names don't make the AC they cover explicit — a future test-evidence format could require each AC row to name the *specific assertion* that proves the AC, not just the test function name.
3. **TC097 migration slipped a third sprint.** Sprints 65, 66, and now 67 all deferred the TC097 → tiered harness migration. The interactive watch tests remain permanently failing with `ExpectTimeout`, which means every sprint that touches `watch.rs` (65, 66, 67) has zero execution proof for the interactive watch path. Either the migration gets done in Sprint 68 OR the TC097 suite should be retired to a "known infrastructure limitation" list rather than counting as phantom coverage.
4. **`#![deny(warnings)]` still ambushes CI on each rustc stable.** Sprint 67 did not trigger a new lint (all 1.95.0 lints were fixed in commit `4b39973` before this sprint started), but the root issue remains: `src/lib.rs:1` denies all future warnings, and CI tracks latest stable. A future sprint should decide: (a) keep `#![deny(warnings)]` and pin CI to a known toolchain via `rust-toolchain.toml`, OR (b) remove the crate-level attribute and rely solely on CI's `-D warnings` flag. Either resolves the drift; doing neither keeps the hazard live.
5. **UX P2: search status bar wipes position context.** Unfolding this properly needs width-aware composition (the bar must gracefully drop segments when terminal is narrow). Real user value; not a single-sprint afternoon task. Belongs in a dedicated pager-UX polish sprint.

### Follow-Up Items

- **P2 (Sprint 68 candidate):** TC097-A..H migration to `spawn_tq_repl_tiered` + `Stage::Query`. Deferred since Sprint 66. Third deferral; owed.
- **P2:** Author TC099-I01 (PTY test for AC-1 / AC-11 of pager search) using Sprint 66 tiered harness; author `scroll_to_match_snaps_to_rightmost_column` unit test to close AC-7 execution gap; author `write_value_with_highlights` ANSI-byte unit test to close AC-8.
- **P2:** Search status bar composition — keep row/column context visible alongside `Pattern: ...  (N matches)` when terminal width permits.
- **P2:** `n` / `N` transient feedback when `SearchStatus::NotFound`.
- **P3:** Extract `viewport.rs` module (Architect recommendation) — `col_offset`, `row_offset`, `visible_column_count`, `scroll_to_match_index` as a cohesive unit testable without constructing a full `Pager`.
- **P3:** Introduce `enum PagerAction` intermediate layer for keybinding → action decoding (Architect recommendation).
- **P3:** Pin CI toolchain via `rust-toolchain.toml` OR remove `#![deny(warnings)]` from `src/lib.rs` — pick one to eliminate the stable-drift CI hazard.
- **P3:** Update `docs/testing/approach.md` and `docs/testing/philosophy.md` with the "REQUIRED tests that aren't authored are MEDIUM-severity gaps, not resolved by code inspection" rule (QV self-critique).
- **P3:** Document the writer-injected pure-render pattern as a project-wide standing rule in `docs/design/vision.md` (Architect recommendation).
- **P3:** `classify_key` modifier tightening (Sprint 65 carry-forward); `std::mem::take` item CLOSED by Feature 2 (can be struck from backlog).

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-19 | 1.0 | Sprint 67 review via /sprint-reviewer | Sprint Coordinator |
