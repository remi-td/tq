# Sprint 68 Review: Test Debt Closure

## Sprint Overview

**Sprint Goal:** Close three consecutive sprints of deferred test infrastructure work: migrate TC097-A..H to the tiered PTY harness, author the three REQUIRED test gaps from Sprint 67 (AC-7, AC-8, PTY pager search), codify the REQUIRED-test rule in testing docs, and pin the CI toolchain to eliminate the `#![deny(warnings)]` / latest-stable drift hazard.

**Sprint Theme:** Maintenance — Test Infrastructure & Framework Hardening
**Date:** 2026-05-29
**Version:** v1.50.0
**Type:** Maintenance Sprint

---

## Objectives Completed

### Objective 1: TC097-A..H Migration to Tiered Harness — DELIVERED

All 8 TC097 interactive watch tests in `tests/interactive_tests.rs` (lines 3408–3628) migrated from legacy `spawn_tq_repl()` + flat 30s `set_expect_timeout` overrides to `spawn_tq_repl_tiered(test_name)` + `Stage::Connect` / `Stage::Query` / `Stage::Prompt` segmented budgets. The two hardcoded `set_expect_timeout` overrides at old lines ~3554/3561 in TC097-H are removed.

**Result of migration:** TC097-A..H still fail at the PTY infrastructure level (reedline emits `[6n` cursor-position queries that the harness does not answer), but the failure mode is now named (`QueryTimeout`) and produces a 4 KB PTY dump per test at `tests/results/sprint-66/<test_name>.pty.log`. The dump content unambiguously shows the root cause. Silent `ExpectTimeout` without evidence is eliminated. AC-OBJ1-2 (pass or produce actionable dump) is satisfied.

Added `spawn_tq_repl_tiered_with_pager(test_name)` helper at `tests/interactive_tests.rs:63` — mirrors `spawn_tq_repl_tiered` but omits `--no-pager` for pager-based PTY tests.

### Objective 2: Sprint 67 REQUIRED Test Gaps — DELIVERED (unit) / UNVERIFIED (PTY)

**AC-7 — `scroll_to_match_snaps_to_rightmost_column` (DELIVERED):** Unit test in `src/commands/repl/pager.rs` (line ~2820). Uses `make_pager_with_data(3, 8)` with `term_width = 55` yielding `visible_column_count = 3`. Plants unique match `val_0_6` in column 6 (off-screen right), calls `submit_search`, asserts `col_offset == 4` (= match_col + 1 - visible_at_start). Exercises the previously-untested `col_offset` right-shift branch at `pager.rs:1176–1183`. **PASSES.**

**AC-8 — `write_value_with_highlights_wraps_match_in_reverse_sgr` (DELIVERED):** Unit test in `pager.rs` (line ~2880). Calls `write_value_with_highlights` with a `Vec<u8>` writer on `"foobarbaz"` with span `(3..6)` = `"bar"`. Asserts `\x1b[7m` precedes `"bar"` bytes and `\x1b[27m` (NoReverse) follows. Uses a `find_subslice` helper. **PASSES.**

**AC-1/AC-11 — PTY pager search test (UNVERIFIED):** `test_pager_search_prompt_shows_match_count` at `tests/interactive_tests.rs:3631` uses `spawn_tq_repl_tiered_with_pager`, runs a 30-row query to activate pager, then attempts to open the search prompt with `/`. Reports `1 passed` in 64s, but the PTY dump confirms the early-return cursor-detection guard fired — reedline `[6n` failures prevented pager activation. The search AC was not exercised. Correctly reclassified as `skipped for reason: PTY cursor detection fired` in test evidence.

### Objective 3: REQUIRED Test Rule Codified — DELIVERED

Three testing docs updated:
- `docs/testing/approach.md`: new "REQUIRED Tests Are Not Optional" section with explicit MEDIUM-severity rule and P2-follow-up requirement
- `docs/testing/philosophy.md`: new "Deferring a REQUIRED Test" anti-pattern entry with cross-references
- `docs/testing/honest-assessment.md`: reconciled (existing detailed rule preserved, new docs cite it)

### Objective 4: Toolchain Pin — DELIVERED

- `rust-toolchain.toml` created at project root: `channel = "1.94.0"`, `components = ["clippy", "rustfmt"]`
- `.github/workflows/ci.yml` updated: `dtolnay/rust-toolchain@stable` → `@master` with explicit `toolchain: "1.94.0"`
- `src/lib.rs:1` `#![deny(warnings)]` retained and documented: now safe with the pin; requires deliberate bump commit before toolchain version changes
- Decision: use hard version pin (`"1.94.0"`) rather than bare `"stable"` — eliminates the "new lint on next stable release" class of CI failure entirely rather than just making it less likely

### Retro Fixes (In-Sprint)

Four fixes applied during Phase 5 per review findings:

| # | Finding | Fix | File |
|---|---------|-----|------|
| 1 | REQ-PAGER-SEARCH-007.2 missing left-scroll case and no-op guard | Rewrote requirement with scroll-right, scroll-left, and no-op guard sub-points | `docs/specifications/repl.md:1727` |
| 2 | REQ-PAGER-SEARCH-008.1 ambiguous "SGR reset or equivalent" | Specified `\x1b[27m` (NoReverse, SGR 27) as the required terminator with rationale | `docs/specifications/repl.md:1748` |
| 3 | TC104 evidence mislabeled "PASSED" when early-return guard fired | Reclassified to "skipped for reason: PTY cursor detection fired" with PTY dump reference | `tests/results/sprint-68/test-evidence-1.md` |
| 4 | TC101.md function name mismatch (TC097-H) | Fixed `test_sessions_watch_non_watch_regression` → `test_sessions_no_watch_regression` | `tests/cases/TC101.md:119` |

---

## Metrics

| Metric | Value |
|--------|-------|
| Objectives completed | 4/4 (Objective 2 PTY portion unverified — pre-existing infrastructure limitation) |
| Unit tests before | 1134 |
| Unit tests after | 1136 (+2: AC-7 and AC-8) |
| Tests passing (`--all-targets`) | 1337 passed, 88 ignored |
| Test pass rate | 100% (unit + non-ignored integration) |
| Clippy warnings | 0 |
| Version | v1.50.0 |
| Tag pushed | `v1.50.0` (release workflow triggered) |
| Retro fixes | 4 |

### Test Execution Verdict

**APPROVED WITH CONCERNS (quality-validator)**

| AC | Test | Status |
|----|------|--------|
| OBJ1-1: TC097 uses `spawn_tq_repl_tiered` | TC101 structural grep | EXECUTED — PASSED |
| OBJ1-2: TC097 produces PTY dumps on failure | TC101 A..H interactive | PRE-EXISTING FAILURE — PTY dumps produced |
| OBJ1-3: `set_expect_timeout` removed | TC101 structural grep | EXECUTED — PASSED |
| OBJ2-1: AC-7 horizontal scroll unit test | `cargo test --lib scroll_to_match` | EXECUTED — PASSED |
| OBJ2-2: AC-8 ANSI byte unit test | `cargo test --lib write_value_with` | EXECUTED — PASSED |
| OBJ2-3: AC-1/AC-11 PTY pager search | TC104 interactive | SKIPPED — PTY cursor detection (MEDIUM severity) |
| OBJ3: REQUIRED rule in testing docs | TC106 doc grep | EXECUTED — PASSED |
| OBJ4: `rust-toolchain.toml` + CI | TC105 structural + `ci-check.sh` | EXECUTED — PASSED |

### Token / Cost Metrics

From `docs/sprints/sprint-68-metrics.md`:

| Metric | Value |
|--------|-------|
| Subagent invocations | 7 (Phase 2 ×3, Phase 3 ×2, Phase 4 QV ×1, metrics ×1) |
| Total input tokens | 25,027 |
| Total output tokens | 153,563 |
| Cache creation tokens | 1,104,392 |
| Cache reads | 21,795,972 |
| Grand total | 23,078,954 |
| Overall cache hit rate | 95.1% |
| Estimated cost (Sonnet pricing floor) | **$12.23** |

**Comparison:** Sprint 65 ~$28, Sprint 66 ~$39, Sprint 67 ~$31, **Sprint 68 ~$12**. Maintenance sprints with no new feature code and minimal design work are significantly cheaper than feature sprints. The 95.1% cache hit rate confirms tight context reuse across phases.

---

## Agent Reviews (abridged)

### Technical (rust-teradata-architect)
**Verdict: Sound with concerns — P2s fixed in retro; P3s captured for backlog.**

- **C1 (P2, fixed in retro):** REQ-PAGER-SEARCH-007.2 omitted the left-scroll case — `scroll_to_match_index` has both a right-shift and a left-shift branch, but the spec only documented rightmost snap. Fixed by splitting the requirement into scroll-right, scroll-left, and no-op guard sub-points.
- **C2 (P2, fixed in retro):** REQ-PAGER-SEARCH-008.1 listed `\x1b[0m]` as the primary terminator with "or equivalent." Implementation deliberately uses `\x1b[27m]` (NoReverse) to avoid stripping foreground colors on styled cells. Fixed by specifying SGR 27 as the required terminator with rationale.
- **C3 (P3):** Toolchain version duplicated in `rust-toolchain.toml` and `ci.yml` — manual two-file sync hazard. Cleaner to let `@master` read the toml. Follow-up.
- **C4 (P3):** CI clippy runs without `--all-targets`; new test code is only clippy-gated locally. Pre-existing.
- **C5 (P3):** PTY dump path hardcoded to `tests/results/sprint-66/`. Operational annoyance, follow-up.
- **C6 (P3):** AC-7 conservative assertion (`match_col >= col_offset`, not strict rightmost). Intentional per documented "settle on next render" behavior; acceptable.

### Quality (quality-validator)
**Verdict: APPROVED WITH CONCERNS — P1 fixed in retro evidence correction; P2/P3 captured.**

- **P1 (fixed in retro):** TC104 reported "PASSED" when the early-return cursor-detection guard had fired without exercising any search assertions. PTY dump confirmed phantom pass. Evidence corrected to "skipped for reason."
- **P2:** TC101 evidence prose said "9 tests" when there are exactly 8 (TC097-H is `test_sessions_no_watch_regression`, not a separate 9th test). Fixed in TC101.md, noted in review.
- **P2:** Hard pin `"1.94.0"` diverges slightly from planning doc language ("channel = 'stable'"). The hard pin is arguably better (completely eliminates rustc-drift) but requires a deliberate toolchain-bump commit. Design decision documented.
- **P3:** PTY dump path hardcoded to sprint-66 directory (shared concern with Architect).
- **P3:** TC101.md function name for TC097-H was wrong. Fixed.
- **Root cause pattern:** AC-OBJ2-3 (PTY pager search) and TC097-A..H share the same blocker: reedline emits `[6n` cursor-position queries that the harness never answers. This single PTY infrastructure gap blocks both the watch tests and the pager search PTY test. **The shared root cause should be the Sprint 69 P1 candidate.**

### UX (cli-ux-designer)
**Verdict: Acceptable with concerns — P1 fixed in retro; P2/P3 captured.**

- **P1 (fixed in retro):** REQ-PAGER-SEARCH-008.1 ambiguity between SGR 0 and SGR 27 resolved by specifying `\x1b[27m` with rationale.
- **P2:** REQ-PAGER-SEARCH-007.2 no-op guard wording: now clearer as a separate labelled sub-point, but a worked no-op example would strengthen testability.
- **P3:** REQ-PAGER-SEARCH-001.7 sub-sentence prescribing internal byte layout belongs in `docs/design/repl.md` rather than a specification. Low priority.
- **P3:** `docs/testing/approach.md` REQUIRED-test paragraph ends with historical voice; suggest normative rewrite.
- **User guide drift: None detected.** No `docs/user/*.md` changes in Sprint 68. Clean.

---

## Retrospective

### What Went Well

1. **The TC097 migration landed cleanly.** The mechanical `spawn_tq_repl()` → `spawn_tq_repl_tiered()` + `Stage`-based waits migration is complete across all 8 tests. The harness's on-timeout PTY dump behavior now gives actionable failure evidence instead of opaque `ExpectTimeout`. This closes a two-sprint-old structural obligation.

2. **AC-7 and AC-8 unit tests are genuinely exercising the target branches.** The architect verified that the AC-7 test reaches `pager.rs:1176–1183` (the right-shift branch that no prior test covered) and the AC-8 test verifies exact byte emission via a `Vec<u8>` writer without any terminal dependency. These are not phantom coverage.

3. **Cost dropped sharply.** $12.23 vs $31 (Sprint 67) vs $39 (Sprint 66). Maintenance-only sprints with no new feature design or user guide work are significantly cheaper. The 95.1% cache hit rate confirms the session budget management was effective.

4. **Three spec contradictions found and fixed.** The Phase 5 reviews caught three spec-vs-code divergences (REQ-PAGER-SEARCH-007.2 missing left-scroll case, REQ-PAGER-SEARCH-008.1 wrong terminator, TC101.md wrong function name) and all three were fixed before the sprint closed. Zero-debt discipline held for the 5th consecutive sprint.

5. **REQUIRED-test rule is now cross-linked across three documents.** `approach.md` / `philosophy.md` / `honest-assessment.md` form a coherent three-layer system: imperative rule, anti-pattern label, detailed enforcement. The QV review confirmed the cross-links are complete and consistent.

### What Could Be Improved

1. **The PTY cursor-position root cause blocks two feature areas and is not getting closer to fixed.** TC097-A..H and TC104 both fail because reedline emits `[6n` cursor-position queries on REPL startup that the `expect_rl::Session` never answers. This has been documented since Sprint 65 (4 sprints). The fix likely requires either: (a) configuring reedline to skip cursor detection in non-TTY contexts, (b) making the PTY harness answer `[6n` queries with a synthetic `[1;1R` response, or (c) using a different test mechanism that doesn't go through reedline startup at all (e.g., `spawn_tq_repl_with_pager` pattern that passes `--no-syntax-highlight` and a simplified reedline config). Sprint 69 should own this.

2. **TC104 phantom pass would have shipped undetected without Phase 5.** The initial test-execution run reported `1 passed` and the QV labeled it "PASSED." Only the Phase 5 technical review's inspection of the PTY dump found the early-return. This is exactly the scenario the REQUIRED-test rule was written to prevent — but the rule kicked in at Phase 5, not Phase 3. The test strategy should specify that PTY tests with early-return guards must be marked `skipped for reason` in evidence, not `passed`, if the guard fires. This could be codified as a `guard_fired: bool` assertion at the guard site.

3. **CI toolchain pin is duplicated.** `rust-toolchain.toml` and `.github/workflows/ci.yml` both state `"1.94.0"`. The intent of `rust-toolchain.toml` is that `dtolnay/rust-toolchain@master` reads the file — eliminating the duplication. The CI workflow should be updated to drop the explicit `toolchain:` key and rely on the file. Sprint 69 P3 cleanup.

### Follow-Up Items

- **P1 (Sprint 69 candidate):** Fix PTY cursor-detection limitation — reedline `[6n` queries go unanswered by the harness, blocking TC097-A..H and TC104. Options: answer `[6n` in harness, configure reedline to skip cursor detection, or redesign PTY test entry point. This is the single item that, if fixed, unlocks interactive watch tests and pager search PTY tests for all future sprints.
- **P2:** Add a worked no-op example to REQ-PAGER-SEARCH-007.2 (match already in viewport, no scroll).
- **P3:** Remove toolchain duplication: drop `toolchain:` key from `.github/workflows/ci.yml` and let `@master` read `rust-toolchain.toml`.
- **P3:** Update `scripts/ci-check.sh` (and CI) to use `--all-targets` for clippy so test-target code is CI-gated.
- **P3:** Parameterize PTY dump path in `tests/common/pty_harness.rs:294` — replace hardcoded `sprint-66` with per-sprint or per-run directory.
- **P3:** Move REQ-PAGER-SEARCH-001.7 byte-layout sub-sentence to `docs/design/repl.md`.
- **P3:** Normalise `approach.md` historical-voice sentence to imperative/normative tone.
- **P3:** Pager UX P2 carry-forwards from Sprint 67: search status bar position context, `n`/`N` transient not-found feedback.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-05-29 | 1.0 | Sprint 68 review via /sprint-reviewer | Sprint Coordinator |
