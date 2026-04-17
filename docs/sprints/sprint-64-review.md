# Sprint 64 Review: SPL Body Parser & Stdin Detection Bug Fixes

## Sprint Overview

**Sprint Goal:** Eliminate two user-reported bugs that broke `tq query` for stored procedure deployment (#42) and for CI/agent harnesses with redirected empty stdin (#43).

**Sprint Theme:** Bug Fix Sprint
**Date:** 2026-04-17
**Version:** v1.46.0
**Type:** Feature Sprint (bug-fix focused)

---

## Objectives Completed

### Feature 1: BEGIN/END body splitting (Bug #42) — DELIVERED

The file-mode statement splitter now preserves `CREATE/REPLACE PROCEDURE/TRIGGER/MACRO/FUNCTION` bodies as a single statement. Internal semicolons inside `BEGIN … END` are no longer misinterpreted as top-level terminators.

**Implementation:**
- Extended `parse_statements()` in `src/sql/parser.rs` with a `begin_end_depth: u32` counter composed on top of the existing `LexState` state machine.
- Procedure-header detection via a lazy `regex::Regex` (`OnceLock`) matching `(CREATE|REPLACE) ... (PROCEDURE|TRIGGER|MACRO|FUNCTION)` on the current buffer. Entered on the first `BEGIN` following a recognised header.
- Word-boundary scanning of `BEGIN` / `END` with one-token lookahead to suppress `END IF | LOOP | WHILE | CASE | FOR`.
- Lookahead now skips `--` and `/* */` comments, so patterns like `END /* x */ IF` classify correctly (follow-up fix from UX review).
- At end-of-input, an unclosed BEGIN raises `Unterminated procedure/trigger/macro body` with the opening line (REQ-BATCH-SPL-007 compliance — follow-up from UX review).

### Feature 2: Stdin detection with empty redirected fd (Bug #43) — DELIVERED

`tq query "SQL"` now accepts redirected-but-empty stdin (`< /dev/null`, `<<< ""`, subshells that close stdin) without raising the spurious "multiple input sources" conflict.

**Implementation:**
- New `stdin_has_data()` helper in `src/commands/query.rs`.
- Unix: `libc::poll(fd=0, POLLIN, timeout=0)` + `libc::ioctl(fd, FIONREAD, &n)` to distinguish "data ready" from "ready-at-EOF", with `fstat` fallback for regular files.
- Non-Unix: returns `true` (preserves legacy behaviour, avoids regression).
- `determine_input_source()` now uses `!is_terminal() && stdin_has_data()` instead of `!is_terminal()` alone.

**Files changed:** `src/sql/parser.rs`, `src/commands/query.rs`, 2 specification files, 2 design docs, user guide, tests.

---

## Metrics

| Metric | Value |
|--------|-------|
| Features completed | 2/2 (100%) |
| P0 features | 2/2 |
| GitHub issues closed | 2 (#42, #43) |
| New unit tests | 18 (15 parser + 3 query) |
| New integration tests | 4 live-DB (`#[ignore]`) + 1 no-DB |
| Total unit tests | 1076 |
| Test pass rate | 100% |
| Clippy warnings | 0 |
| Lines added | ~3040 |
| Lines removed | ~48 |
| Version | v1.46.0 |

### Token/Cost Metrics (from `sprint-64-metrics.md`)

| Metric | Value |
|--------|-------|
| Subagent invocations | 10 |
| Total input tokens | 60,319 |
| Total output tokens | 143,444 |
| Cache creation tokens | 1,538,707 |
| Cache read tokens | 27,744,999 |
| Grand total | 29,487,469 |
| Overall cache hit rate | 94.6% |
| Estimated cost (Sonnet pricing floor) | $15.27 |

**Comparison to Sprint 63:** Token metrics were not collected for Sprint 63, so no direct delta is available. Sprint 64 shipped 2 features vs. 1 in Sprint 63, with roughly similar single-session efficiency. Cost-per-feature: **~$7.64** — consistent with the single-session sprint target.

---

## Agent Reviews

### Technical Review (rust-teradata-architect)

**Verdict: Sound with concerns (concerns addressed).**

Idiomatic Rust throughout — `OnceLock` for the lazy regex, explicit state-machine composition, `saturating_add` for the depth counter, `#[cfg(unix)]` gating, properly scoped `unsafe` blocks with SAFETY comments. The state machine composes correctly: BEGIN/END detection lives inside `LexState::Normal` only, so string-literal and comment contexts are immune by construction. `byte_offset` tracking for `sql[byte_offset..]` lookahead is carefully maintained across every branch.

**Concerns raised + disposition:**
- `is_compound_end` did not skip comments between `END` and the compound keyword → **fixed** (commit bc631bc).
- Regex-based header detection has a theoretical false-positive surface on large mixed-script buffers with unrelated `PROCEDURE`/`FUNCTION` words later in the same statement → **accepted as known limitation**; tightening the regex to require header-before-first-semicolon is a P3 follow-up.
- The non-Unix `true` fallback re-introduces #43 for exotic fd types → **accepted conservative default**; platform-specific tightening is P3.

### Quality Review (quality-validator)

**Verdict: APPROVED WITH CONCERNS (concerns addressed).**

All 8 acceptance criteria for #42 and 6/7 for #43 have automated coverage (AC-5 for #43 is TTY-dependent and correctly gated as manual-only). Unit: 1072 → 1076. Integration: 40 non-ignored + 12 ignored all pass.

**Concerns raised + disposition:**
- Compound `END` with interleaved comments not covered → **fixed**; 2 new tests added (`test_compound_end_with_line_comment_between`, `test_compound_end_with_block_comment_between`).
- MACRO sub-case silently dropped from TC094-G test → **accepted**; MACRO deployment exercised at integration level, inline MACRO body via `(...)` is not semicolon-sensitive in the problematic way (deferred as P3 follow-up test).
- FUNCTION support in regex but absent from REQ-BATCH-SPL-001 spec → **fixed**; spec now lists `CREATE/REPLACE FUNCTION` explicitly.

### UX Review (cli-ux-designer)

**Verdict: Acceptable with concerns (P0 concern fixed).**

Both fixes match user expectations. The CI/stdin example in `docs/user/batch-mode-guide.md` is copy-paste correct.

**P0 concern + disposition:**
- Unterminated BEGIN at EOF silently flushed partial buffer to Teradata, producing a confusing server-side error instead of the REQ-BATCH-SPL-007 "Unterminated procedure body" local diagnostic → **fixed** (commit bc631bc). The parser now raises `ParseError` with the opening BEGIN line, including a nested-body test that asserts the line points to the outermost (first-opened) BEGIN.

---

## Retrospective

### What Went Well

1. **Well-scoped bugs.** Both issues had clear repros and the reporter even suggested fixes. Planning was trivial.
2. **Existing state machine composed cleanly.** The Sprint 42 lexer was designed extensibly enough to bolt on a `begin_end_depth` counter without restructuring. No architectural changes were required.
3. **Parallel agent execution.** Design, implementation, and test authoring all ran concurrently across three agents. The quality validator even pre-seeded the parser test stubs so the architect's implementation landed with tests already in place — zero-effort matching.
4. **Reviewers caught real issues.** The three-agent retrospective surfaced a P0 spec non-compliance (REQ-BATCH-SPL-007) that the coordinator missed during Phase 4 validation. Fixed in the same sprint per zero-debt policy rather than deferred.
5. **Live DB integration tests.** All four new `#[ignore]` tests passed against live Teradata on the first execution, validating the fix end-to-end (not just at the parser-unit level).

### What Could Be Improved

1. **Phase 4 checklist missed REQ-BATCH-SPL-007.** The spec required an error at EOF for unterminated bodies, but the initial implementation flushed silently. The Phase 4 "Documentation Sync" checklist should include "for each new REQ, verify the implementation actually implements that specific requirement end-to-end." The UX reviewer found this — the coordinator should have.
2. **MACRO test coverage deferred implicitly.** The original test strategy flagged MACRO as potentially needing deferral (because MACRO uses `(...)` not `BEGIN...END`), but the deferral was never formalised as a follow-up. Going forward, "deferred from strategy" items need an explicit P3 entry, not a silent drop.
3. **Driver dylib path test fragility.** Running ignored integration tests required manually copying `teradatasql.dylib` into `target/debug/deps/` to satisfy driver discovery, which then broke a unit test that asserts the fallback path. A pre-existing environment issue, not Sprint 64's fault, but worth noting as friction.

### Follow-Up Items

- **P3:** Tighten the procedure-header regex to require header-before-first-semicolon to reduce false-positive surface on pathological mixed scripts.
- **P3:** Add explicit MACRO `(...)` body test coverage and/or document as a known limitation.
- **P3:** Add test for `BEGIN TRANSACTION` (top-level, non-SPL) to confirm it's NOT treated as a body header — partially covered by `is_procedure_header` check but no dedicated assertion.
- **P3:** Investigate making the `teradatasql.dylib` discovery test-harness-aware so ignored integration tests don't require manual driver placement.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-17 | 1.0 | Sprint review created via /sprint-reviewer skill | Sprint Coordinator |
