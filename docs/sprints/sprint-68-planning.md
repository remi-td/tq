# Sprint 68 Planning

**Date:** 2026-05-29
**Type:** Maintenance Sprint
**Version Target:** v1.50.0

---

## Reality Check Summary

- **Reviewed sprints:** 65, 66, 67
- **Patterns detected:**
  - **Stuck Issue (P2):** TC097 interactive watch tests deferred in Sprints 65, 66, and 67 (3 consecutive). Sprint 66 built the tiered harness specifically to unblock this migration; TC097-A..H migration still has not occurred.
  - **Accumulating Test Debt:** Sprint 67 marked 3 acceptance criteria as REQUIRED but unauthoured: AC-7 (horizontal scroll unit test), AC-8 (ANSI byte assertion for `write_value_with_highlights`), AC-1/AC-11 (PTY pager search tests). "REQUIRED tests that aren't authored" rule not yet codified in `docs/testing/`.
  - **CI Hazard:** `src/lib.rs:1` contains `#![deny(warnings)]` while CI tracks latest stable Rust. Sprint 67 narrowly avoided a CI break (lints were pre-fixed); the root drift remains.
- **Decision: Maintenance Sprint**
- **Rationale:** TC097 has crossed the 2+ sprint recurrence threshold for stuck issues. The tiered harness exists; the only remaining work is mechanical migration. Running another feature sprint with TC097 still in limbo means every watch.rs change has zero interactive execution proof. The test debt and CI hazard compound this concern.

---

## Objectives

1. **Close TC097 for good:** Migrate all 8 TC097 interactive watch tests (A..H) to `spawn_tq_repl_tiered` + `Stage::Query` so they have a realistic chance of passing in CI.
2. **Close Sprint 67 REQUIRED test gaps:** Author the three AC-classified tests that were skipped: AC-7 (horizontal scroll), AC-8 (ANSI byte highlighting), and at least one PTY pager search test (AC-1/AC-11).
3. **Codify the REQUIRED test rule:** Update `docs/testing/approach.md` and `docs/testing/philosophy.md` to make "REQUIRED tests that aren't authored = MEDIUM severity gap" explicit and actionable.
4. **Resolve the `#![deny(warnings)]` CI hazard:** Implement `rust-toolchain.toml` to pin CI to a known stable toolchain version and eliminate the rustc-drift ambush pattern.

---

## Acceptance Criteria

### Objective 1: TC097 Migration
- [ ] TC097-A..H each use `spawn_tq_repl_tiered` + `Stage::Query` (replacing hardcoded 30s `set_expect_timeout`)
- [ ] TC097-A..H tests pass or produce a PTY dump explaining the failure (no silent `ExpectTimeout` without evidence)
- [ ] Old `set_expect_timeout` overrides at `tests/interactive_tests.rs:3554,3561` removed

### Objective 2: Sprint 67 Test Gaps
- [ ] Unit test `scroll_to_match_snaps_to_rightmost_column` (or equivalent) added to `pager.rs` tests — exercises the `col_offset` branch for AC-7
- [ ] Unit test for `write_value_with_highlights` asserting `\x1b[7m` byte emission — closes AC-8 ANSI assertion gap
- [ ] At least one PTY pager search test using the tiered harness (AC-1 or AC-11: opens search prompt, enters pattern, sees match highlighted)

### Objective 3: REQUIRED Test Rule
- [ ] `docs/testing/approach.md` contains explicit statement: "A test classified REQUIRED in the test strategy that is not authored must be reported as a MEDIUM-severity gap in the quality report — it is not resolved by code inspection or manual verification."
- [ ] `docs/testing/philosophy.md` updated to align with the above
- [ ] `honest-assessment.md` entry added or updated if applicable

### Objective 4: Toolchain Pin
- [ ] `rust-toolchain.toml` added to project root with `channel = "stable"` pinned to current known-good version
- [ ] CI passes with pinned toolchain
- [ ] `#![deny(warnings)]` decision documented: either keep (now safe with pin) or remove crate-level attr in favor of CI `-D warnings` flag alone

---

## Scope

### In Scope
- TC097-A..H migration (`tests/interactive_tests.rs`)
- Three Sprint 67 test gap closures (pager unit tests + one PTY pager test)
- Testing documentation updates (`docs/testing/approach.md`, `docs/testing/philosophy.md`)
- `rust-toolchain.toml` addition + CI validation

### Out of Scope
- New user-facing features (this is a pure maintenance sprint)
- PMON graphical features (#21, #22, #23 — complex TUI work, deferred)
- `viewport.rs` extraction and `PagerAction` enum (P3 architecture, deferred)
- Search status bar composition (P2 UX improvement, deferred)
- `n`/`N` transient feedback for not-found (P2, deferred)

---

## GitHub Issues

### Selected for Sprint
_None of the open sprint-ready issues (#21, #22, #23) align with maintenance work. This sprint is driven entirely by carry-forward items from Sprint 67 retrospective._

### Deferred
- #21 PMON Graphical Resource Displays — complex TUI, P3
- #22 PMON Graphical Session Displays — complex TUI, P3
- #23 PMON Alerting and Threshold Configuration — complex TUI, P3

---

## Dependencies

- `tests/common/pty_harness.rs` tiered harness (shipped in Sprint 66) — already available
- Live database connection via `TQ_LOGON` for TC097 execution proof
- Current toolchain: `rustc 1.95.0` (need to check actual version for pinning)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| TC097 still times out with tiered harness | Medium | Medium | PTY dump evidence required; acceptable if dump explains root cause |
| PTY pager search test flaky in CI | Medium | Low | Use staged timeouts; acceptable to mark `#[ignore]` if genuinely infrastructure-limited |
| Toolchain pin breaks CI | Low | Medium | Test locally before push; use `stable` channel not a hard version string |
