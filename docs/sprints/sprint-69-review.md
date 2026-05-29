# Sprint 69 Review: PTY Cursor Fix + Pager Search Status Bar

## Sprint Overview

**Sprint Goal:** Fix the PTY cursor-position root cause blocking TC097-A..H and TC104 for four consecutive sprints; add pager search status bar composition showing row context alongside match count.

**Sprint Theme:** Feature Sprint (PTY infrastructure fix + UX enhancement)
**Date:** 2026-05-29
**Version:** v1.51.0
**Type:** Feature Sprint

---

## Objectives Completed

### Objective 1: PTY Cursor-Position Fix — DELIVERED

Root cause confirmed and fixed in `tests/common/pty_harness.rs`:

- Reedline 0.38 calls `crossterm::cursor::position()` on every `read_line` call, emitting `\x1b[6n` (DSR — Device Status Report) and waiting up to 2000 ms for `\x1b[row;colR` (CPR — Cursor Position Report). In the PTY harness, no responder existed, so reedline entered a retry error loop ("cursor position could not be read"), preventing the REPL prompt from ever appearing.

- **Fix approach:** Rewrote `expect_stage()` to poll `try_read()` directly instead of delegating to `session.expect()`. Every chunk read passes through `absorb_chunk()`, which calls `detect_cpr_query(&chunk)` and writes `CPR_REPLY = b"\x1b[1;1R"` back to the PTY session when `\x1b[6n` is detected. This ensures the CPR answer is sent on the hot read path, not in post-timeout diagnostics.

- **Result:** TC097-A..H ran in 10.88s (was timing out at 64s+ with cursor error loop). TC104 pager search test passes with real assertions — `Pattern: DBC ... match` confirmed in status bar.

- Added `pub(crate) fn detect_cpr_query(bytes: &[u8]) -> bool` for unit testability. Unit test `cpr_detection_returns_true_for_dsr_query_bytes` covers bare, embedded, trailing, and negative cases.

### Objective 2: Pager Search Status Bar Composition — DELIVERED

`render_status_bar_to_buffer` updated in `src/commands/repl/pager.rs:885–916`:

- **Wide terminal:** `Pattern: <pat>  (N matches)  |  Rows X-Y of Z`
- **Narrow terminal:** `Pattern: <pat>  (N matches)` (row context dropped when `search_seg.width() + row_seg.width() > term_width - 2`)
- **Very narrow terminal:** `search_seg` truncated to `term_width - 2` budget (retro fix from review)
- **Not-found state:** unchanged `Pattern: <pat>  not found`

REQ-PAGER-SEARCH-009.1–009.7 updated with composed format spec. User guide updated with grep-verified examples.

### Retro Fixes Applied In-Sprint

Five fixes landed before sprint close:

| # | Finding | Fix | File |
|---|---------|-----|------|
| 1 | `search_seg` written without width guard on very narrow terminals | Truncate `search_seg` to `budget` chars before fallback write | `pager.rs:902-915` |
| 2 | TC108: 5 of 7 unit tests not implemented | Add U03 (no `%`), U06 (scroll persistence), U07 (not-found no-context), U05 (narrow truncation) | `pager.rs:2645-2745` |
| 3 | TC097-E assertion too weak (trivially true from banner text) | Require watch-frame markers (`interval`/`refreshing`/`Updated`) + ESC-free output | `interactive_tests.rs:3529-3541` |
| 4 | TC104 pager-activation early-return remains after PTY fix | Convert `if .is_err() { return; }` → hard `.expect()` failure | `interactive_tests.rs:3669` |
| 5 | Clippy `manual_pattern_char_comparison` on `trim_end_matches(|c| c == '\r' \|\| c == '\n')` | Use `trim_end_matches(['\r', '\n'])` | `pager.rs:2723` |

---

## Metrics

| Metric | Value |
|--------|-------|
| Objectives completed | 2/2 (100%) |
| Unit tests before | 1138 |
| Unit tests after | 1142 (+4: TC108 retro additions) |
| Tests passing (`--all-targets`) | 1343 passed, 88 ignored |
| Test pass rate | 100% |
| Clippy warnings | 0 |
| Version | v1.51.0 |
| Tag pushed | `v1.51.0` (release workflow triggered) |
| Retro fixes | 5 |

### Test Execution Verdict

**APPROVED (quality-validator — iteration 2)**

| AC | Test | Status |
|----|------|--------|
| OBJ1-1: TC097-A..H pass on live DB | TC097-A..H interactive | EXECUTED — 8/8 PASSED (10.88s) |
| OBJ1-2: TC104 validates search assertions | TC104 interactive | EXECUTED — PASSED (63.75s) |
| OBJ1-3: `detect_cpr_query` unit test | TC107-U01 | EXECUTED — PASSED |
| OBJ1-4: cursor guard removed from TC104 | TC107-structural grep | EXECUTED — PASSED |
| OBJ2-1: wide terminal composed format | TC108-U01 | EXECUTED — PASSED |
| OBJ2-2: narrow terminal drops row context | TC108-U04 | EXECUTED — PASSED |
| OBJ2-3: row context no `%`, separator exact | TC108-U03 | EXECUTED — PASSED (retro) |
| OBJ2-4: scroll persistence | TC108-U06 | EXECUTED — PASSED (retro) |
| OBJ2-5: not-found no row context | TC108-U07 | EXECUTED — PASSED (retro) |
| OBJ2-6: very narrow truncation | TC108-U05 | EXECUTED — PASSED (retro) |

### Token / Cost Metrics

From `docs/sprints/sprint-69-metrics.md`:

**Note:** Metrics are session-cumulative. The session `c68a1c89` also ran Sprint 68's review phase ($12.23 shown in Sprint 68 metrics). Sprint 69 work accounts for the increment above that baseline.

| Metric | Value (session cumulative) |
|--------|---------------------------|
| Subagent invocations (Sprint 69) | ~9 (Phase 2 ×3, Phase 3 ×3, Phase 4 ×2 + fix, Phase 5 ×3) |
| Grand total (cumulative session) | 63,653,746 tokens |
| Sprint 68 baseline | 23,078,954 tokens |
| Sprint 69 delta | ~40,574,792 tokens |
| Overall cache hit rate | 94.9% |
| Estimated cost (session cumulative) | $33.18 |
| Sprint 69 estimated delta | ~$20.95 |

**Comparison:** Sprint 68 maintenance ~$12, Sprint 67 feature ~$31, **Sprint 69 feature ~$21**. The PTY fix involved a rewrite cycle (initial CPR-in-drain-pending approach failed, required a second implementation pass) which accounts for the higher cost vs. a clean feature sprint.

---

## Agent Reviews (abridged)

### Technical (rust-teradata-architect)
**Verdict: Sound with concerns — P3s captured for backlog.**

- **C1 (Medium, follow-up):** `absorb_chunk` detects CPR query per-chunk only. If `\x1b` arrives in one chunk and `[6n` in the next, the 4-byte sequence is split and the reply is never sent. Low probability (kernel pipe delivers a single `write()` atomically into a 4096-byte read), but is a latent reintroduction of the P1 bug. Fix: scan an overlap window of the accumulated tail. Schedule for Sprint 70.
- **C2 (Low, follow-up):** Unit test only validates single-slice detection; no cross-chunk absorption test exists.
- **C3 (Low, follow-up):** `Ok(0)` (genuine child EOF) treated same as `WouldBlock` — exhausts stage budget before failing with accurate "process exited" diagnostic.
- **C4 (Low, follow-up):** `total_rows` (search segment denominator) vs `data.row_count` (default segment denominator) — equal in current in-memory model, could diverge in a future streaming mode.
- Status-bar width math confirmed correct: `row_seg` carries its own `  |  ` prefix, `\r\n` excluded, UnicodeWidthStr handles wide chars. Sound.

### Quality (quality-validator)
**Verdict: APPROVED WITH CONCERNS — all concerns addressed in retro.**

- **TC097-E assertion strengthened (retro fix #3):** Original assertion checked any alphanumeric char in full capture buffer — trivially satisfied by banner. Fixed to require watch-frame markers + ESC-free output.
- **TC104 early-return converted (retro fix #4):** Residual pager-activation `if .is_err() { return; }` guard converted to hard `.expect()` failure.
- **TC108 missing tests added (retro fix #2):** All 7 TC108 tests now implemented.
- **Split-chunk CPR gap:** Latent but low-probability; carries forward to Sprint 70 as P2.
- Honest-assessment rules correctly applied in evidence; PTY guard rule from Sprint 68 framework optimization correctly prevented TC104 from being labeled "PASSED" when the guard fired in iteration 1.

### UX (cli-ux-designer)
**Verdict: Acceptable with concerns — P2 addressed in retro.**

- **`search_seg` overflow (Medium, fixed in retro):** `pager.rs:902` wrote `search_seg` without a width guard. Spec 009.4 requires truncation. Fixed by adding the truncation path.
- **Spec 009.4 rationale (Low, follow-up):** "2-column right margin" rule lacks a rationale note. Add one sentence for future implementors.
- **Spec 009 cross-reference to 003 (Low, follow-up):** Not-found exclusion from composed format should cross-reference REQ-PAGER-SEARCH-003. Minor clarity improvement.
- **User guide drift: None.** All guide examples verified against `pager.rs` source literals (`pager.rs:890`, `pager.rs:895`, `pager.rs:907`). Separator `  |  ` and double-space conventions confirmed.

---

## Retrospective

### What Went Well

1. **The PTY cursor fix shipped and immediately delivered 4 sprints of blocked test coverage.** TC097-A..H now pass in ~10s; TC104 executes its real assertions. The investigation in Phase 2 correctly identified all three options, proved option 1 (CPR injection) with a live-DB probe, and the team committed to the proven approach. Even though the first implementation attempt (CPR in `drain_pending`) failed, the rewrite to poll `try_read` directly was discovered and implemented within the same phase.

2. **Retro review quality was high.** The three Phase 5 reviews caught: `search_seg` truncation gap (spec/code divergence), 5 missing unit tests, TC097-E weak assertion, TC104 residual skip guard — all fixed before close. Five retro fixes in-sprint. Zero-debt discipline held for the 6th consecutive sprint.

3. **Framework investment paid off.** The Sprint 68 PTY early-return guard waste pattern (Pattern 9b) was correctly applied in the Phase 3 iteration 1 QV report: TC104 was labeled "skipped for reason" rather than "PASSED" when the guard fired, which forced iteration 2 to properly validate the AC. This is exactly what the pattern codification was designed to catch.

4. **Cost efficiency improving.** Sprint 69 at ~$21 is cheaper than Sprint 67's $31 despite being a more technically complex sprint (PTY rewrite cycle). The 94.9% cache hit rate and 9 focused subagents contributed.

### What Could Be Improved

1. **Split-chunk CPR detection is a latent reintroduction of the P1 bug.** The `absorb_chunk` function scans each individual chunk for the `[6n` sequence. A 4-byte sequence split across two `try_read` returns would go undetected. Probability is low (the kernel pipe usually delivers a single `write()` atomically), but it's not zero — especially on heavily loaded systems. Sprint 70 should address this by scanning the overlap window of accumulated tail + new chunk.

2. **The first TC104 `--ignored` run produced a "PASSED" result that needed correction.** Even with the Sprint 68 guard rule in place, the iteration 1 QV initially labeled TC104 as "PASSED" before deeper investigation found the early-return. The guard detection heuristic (grep for `cursor position` text) wasn't broad enough to catch the pager-activation guard. The fix — hard `.expect()` — is better than any heuristic: make silent bypasses impossible.

3. **Metrics are session-cumulative and harder to read for individual sprints.** When 3 sprints run in a single session, the collect-metrics script sums all agents. The Sprint 69 cost had to be derived as a delta from Sprint 68. Consider adding a sprint-boundary marker to the session or allowing collect-metrics to take a time range argument.

### Follow-Up Items

- **P1 (Sprint 70 candidate):** Fix split-chunk CPR detection — scan `captured[tail] + chunk` overlap window in `absorb_chunk`; add cross-chunk absorption unit test
- **P2:** Convert TC104 pager search PTY test to validate AC-11 (clean REPL exit) with an assertion, not just a no-crash sleep
- **P2:** TC097-E: the strengthened assertion now checks for watch-frame markers but still scans the full accumulated buffer; consider asserting specifically on the last N bytes of the capture (the exit snapshot region)
- **P3:** `Ok(0)` fast-fail — detect genuine child EOF in `expect_stage` to avoid burning stage budget on a crashed process
- **P3:** `total_rows` vs `data.row_count` denominator — unify to single source in `render_status_bar_to_buffer`
- **P3:** Spec 009.4 — add rationale note for 2-column right margin
- **P3:** Spec 009 — cross-reference REQ-PAGER-SEARCH-003 for not-found exclusion
- **P3:** Pager UX carry-forwards from Sprint 67: `n`/`N` transient not-found feedback
- **P3:** `viewport.rs` extraction, `PagerAction` enum (Sprint 67 architect recommendations)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-05-29 | 1.0 | Sprint 69 review via /sprint-reviewer | Sprint Coordinator |
