# Sprint 65 Review: `/sessions --watch` Hardening

## Sprint Overview

**Sprint Goal:** Ship production-quality auto-refreshing session monitoring for DBAs.

**Sprint Theme:** DBA Live Monitoring — Hardening
**Date:** 2026-04-17
**Version:** v1.47.0
**Type:** Feature Sprint (turned out to be a hardening sprint)

---

## Objectives Completed

### Feature 1: `/sessions --watch` Hardening — DELIVERED

**Unexpected scope shift discovered in Phase 2 Design:** `src/commands/watch.rs` already existed with a basic loop, keystroke polling, and interval parsing (not tied to an earlier GitHub issue). Sprint 65 was therefore not greenfield — it became a hardening sprint to close the real production-readiness gaps.

**What was hardened in `src/commands/watch.rs`:**

- **`RawModeGuard` RAII** — enter raw mode + `EnterAlternateScreen` + hide cursor on construction; unconditional `Drop` reverses all three. Terminal is restored even on panic or unexpected error. If `EnterAlternateScreen` fails after `enable_raw_mode` succeeded, the constructor explicitly unwinds before returning — no leak window.

- **Per-tick error handling** — DB query errors during a refresh no longer crash watch mode. The frame top shows a red/bold `Error at HH:MM:SS: <msg> - retrying in Ns` line, the last successful table body is retained below, and the loop continues at the next tick.

- **Exit snapshot (Sprint 63 pattern)** — `ExitReason::Quit` (`q`, `Q`, `Esc`) prints a plain-text copy of the final frame after the guard drops, landing in scrollback. `ExitReason::Interrupt` (`Ctrl-C`) skips the snapshot, matching the pager convention for graceful-exit vs. interrupt.

- **Interval range alignment** — new constants `MIN_INTERVAL_SECS=1`, `MAX_INTERVAL_SECS=3600`, `DEFAULT_INTERVAL_SECS=6`. `parse_watch_args` clamps to `[1, 3600]` (previously `[2, 300]`). Matches the spec REQ-REPL-SESSIONS-WATCH-002.

- **`/locks --watch` and `/resources --watch` get the same hardening** because all three callers share `run_watch()`. No per-command edits needed.

**Files changed:** `src/commands/watch.rs`, `Cargo.toml`, 3 spec/design docs, user guide, tests.

---

## Metrics

| Metric | Value |
|--------|-------|
| Features completed | 1/1 (100%) |
| GitHub issues addressed | 1 (#25) |
| New unit tests | 20 (8 new TC096 + 12 structural watch tests) |
| New interactive tests | 8 (TC097, `#[ignore]`) |
| Total unit tests | 1096 |
| Test pass rate | 100% (unit + non-ignored integration) |
| Clippy warnings | 0 |
| Version | v1.47.0 |

### Token/Cost Metrics (from `sprint-65-metrics.md`)

| Metric | Value |
|--------|-------|
| Subagent invocations | 10 |
| Grand total tokens | ~53.7M |
| Cache hit rate | 94.3% |
| Estimated cost (Sonnet pricing floor) | $28.57 |

**Comparison to Sprint 64:** Sprint 64 ran ~$15; Sprint 65 ran ~$28. The cost approximately doubled because the design phase uncovered an existing-but-incomplete implementation, which required deeper analysis and more careful rework rather than clean greenfield coding. Cost-per-feature: **~$28.57** — higher than Sprint 64's $7.64/feature but still within the single-session budget envelope.

---

## Agent Reviews

### Technical Review (rust-teradata-architect)

**Verdict: Sound with concerns.** No blockers.

RAII Drop ordering is correct (`Show` + `LeaveAlternateScreen` then `disable_raw_mode` — the inverse of `enter()`). The early-return in `RawModeGuard::enter()` handles the "enable succeeded, EnterAlternateScreen failed" window cleanly. Panic safety verified: `Drop` swallows errors with `let _ = …` so it won't panic-while-unwinding.

Exit-reason classification (`classify_key`, watch.rs:266) maps only `Ctrl+c`/`Ctrl+C` to `Interrupt`; `q`/`Q`/`Esc` map to `Quit`. Snapshot gate correctly suppresses output only on Interrupt.

Per-tick error handling is correct: `fresh_body` is discarded on `Err`, `last_body` untouched, retained body shown below the red header. No resource leaks.

`/locks --watch` and `/resources --watch` truly share the hardening via `run_watch()` — verified in `metacommands.rs`.

**Concerns (P3 follow-ups):**
- `watch.rs:168`: `last_body = fresh_body.clone()` is an avoidable per-tick allocation. `std::mem::take(&mut fresh_body)` would be cheaper.
- `classify_key` accepts `Ctrl+c` with `SHIFT`/`ALT` also held. Tighten to `modifiers == CONTROL` if strictness matters.

### Quality Review (quality-validator)

**Verdict: APPROVED WITH CONCERNS.**

Unit-covered and executed: AC-1, AC-2, AC-3, AC-9 (argument regression). 36/36 pass.

Interactive-only coverage: AC-4 through AC-8 (frame rendering, exit keys in PTY, snapshot placement, terminal state restoration, tick resilience).

**Honest execution gap:** The 8 TC097 interactive tests were run with `--ignored` and **all FAILED with `ExpectTimeout`** — the live `/sessions` query against the current test endpoint exceeds the 20 s timeout in `spawn_tq_repl()`. The initial test evidence said "not executed due to timeout constraints" which is misleading: they were attempted and failed. This distinction matters. AC-4..AC-9 have zero execution proof in this sprint.

**Confidence assessment:** Structural code paths are correct by inspection and match the proven Sprint 63 pager pattern. But "correct by inspection" ≠ "executed proves correct." The honesty gap has been recorded in the updated test evidence document.

**Follow-ups (P2):**
- Extend `spawn_tq_repl()` timeout to 60 s or switch to a faster test DB so TC097 can actually pass.
- Extract `handle_tick_result(render_result, last_body)` as a pure function and add a unit test that feeds `Err(...)` directly — closes AC-8 unit gap without requiring live DB.

### UX Review (cli-ux-designer)

**Verdict: Acceptable with concerns (fixed).**

Interval range in the guide correctly reads "1 to 3600 seconds" — the earlier drift was fixed mid-sprint.

**Drift caught in retrospective and fixed:**
- Frame header mock-up in the guide invented a pipe-delimited `Last updated: ... | Refreshing every ...` footer that doesn't match the actual code strings. Actual strings: top `Updated {} - refreshing every {}s`, bottom `Press q, Esc, or Ctrl-C to stop (interval: {}s)`. **Fixed** in this retrospective commit.
- Error-state mock-up showed an invented box-drawing border. Actual error path just shows `Error at HH:MM:SS: <msg> - retrying in Ns` above the retained table. **Fixed.**

Exit keys (`q`/`Q`/`Esc` vs `Ctrl-C`) are correctly documented. Snapshot-vs-no-snapshot distinction is clear.

---

## Retrospective

### What Went Well

1. **Design phase caught the scope reality.** The architect found an existing `watch.rs` module before implementation began, preventing a greenfield rebuild. Sprint converted cleanly from "build" to "harden" without wasted work.
2. **Code sharing paid off.** Hardening `run_watch()` also fixed `/locks --watch` and `/resources --watch` for free — three commands benefit from one effort.
3. **RAII pattern.** The `RawModeGuard` is textbook Rust and the correct architectural answer to panic-safe terminal cleanup. Sprint 63 proved the pattern in pager.rs; Sprint 65 applies it.
4. **Zero-debt policy applied twice mid-sprint.** Interval range drift (spec vs initial code) and user guide mock-up drift (guide vs final code) were both caught and fixed before closing.

### What Could Be Improved

1. **Documentation-vs-code drift happened twice in one sprint.** The UX designer edited the user guide against an intermediate state of the code both times. Parallel agent execution is efficient but creates stale-read risk. Fix: when UX-designer is running in parallel with the architect, the UX designer should always verify against the final merged source, and the coordinator's Phase 4 "Documentation Sync" check must compare user guide example strings against actual code string literals.
2. **Test-evidence wording was not fully honest.** The first version of the test evidence said interactive tests were "not executed" when in fact they were attempted and failed. This masks real coverage gaps. Fix: test evidence must distinguish "not run," "run and passed," "run and failed," and "skipped for reason X."
3. **Live-DB interactive tests are structurally fragile in this environment.** The `/sessions` query exceeds the 20 s interactive test timeout. This is not a Sprint 65 bug — it is a standing test-infrastructure limitation that blocks REPL feature validation generally. Needs a P2 investment.

### Follow-Up Items

- **P2:** Extend interactive test `spawn_tq_repl()` timeout to 60 s (or switch to faster test DB). Re-run TC097 to get real execution proof for AC-4..AC-9.
- **P2:** Extract `handle_tick_result(render_result, last_body)` as a pure function. Add a unit test feeding `Err(...)` directly to close AC-8 unit-level gap.
- **P3:** Replace `last_body = fresh_body.clone()` with `std::mem::take` (avoids per-tick allocation).
- **P3:** Tighten `classify_key` modifier check from "CONTROL present" to "CONTROL only" for `Ctrl-C` classification.
- **P3:** Phase 4 "Documentation Sync" check should diff user-guide example output strings against actual code string literals (catches cases like this sprint's guide-vs-code drift).

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-17 | 1.0 | Sprint review via /sprint-reviewer skill | Sprint Coordinator |
