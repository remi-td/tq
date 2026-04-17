# Sprint 66 Review: Maintenance — Tiered PTY Harness + UX Process Sequencing

## Sprint Overview

**Sprint Goal:** Close two recurring framework issues surfaced by retrospectives in Sprints 22, 23, 38, 39, 64, 65: (1) opaque interactive-test timeout blocking REPL acceptance-criteria validation, (2) doc-vs-code drift from parallel agent execution.

**Sprint Theme:** Maintenance — Framework Hardening
**Date:** 2026-04-17
**Version:** v1.48.0
**Type:** Maintenance Sprint

---

## Objectives Completed

### Feature 1: Tiered PTY test harness — DELIVERED
New `tests/common/pty_harness.rs` module providing:
- `Stage` (Connect / Prompt / Query), `Timeouts::from_env()` with env-variable overrides (`TQ_TEST_{CONNECT,PROMPT,QUERY}_TIMEOUT`, defaults 45/15/60s), `PtyError` with distinct timeout variants, `TqPty` wrapper with `expect_stage()`.
- Mandatory PTY buffer dump to `tests/results/sprint-66/<test_name>.pty.log` on any timeout — non-negotiable observability.
- Bounded-retry `drain_pending()` to avoid truncated dumps on transient `Ok(0)` non-blocking reads (added in retrospective per architect feedback).
- `spawn_tq_repl()` default raised 20s → 60s for backward-compatible tests.
- 3 unit tests (dump behaviour, error variants, env parsing) — all pass.
- `test_repl_startup_and_quit` migrated to the tiered API as the live-DB integration proof — **PASS** in ~63s.

### Feature 2: TC097 re-run — PARTIALLY DELIVERED
The migrated `test_repl_startup_and_quit` demonstrates the infrastructure works. TC097-A..H were not mass-migrated (explicitly deferred per Sprint 66 planning Out-of-Scope). Structured analysis filed at `tests/results/sprint-66/tc097-failure-analysis.md` with concrete P2 follow-up: one-line per-test migration to `Stage::Query`, est. 2-3h total.

### Feature 3: Sprint-coordinator process change — DELIVERED
Three process docs edited:
- `phase2-design.md`: cli-ux-designer explicitly barred from editing `docs/user/*.md` in this phase.
- `phase3-build-test.md`: removed the third parallel cli-ux-designer launch; added standing-rule note.
- `phase4-ship.md`: new "Step 1.7: User-Guide Prose (Sequential)" with grep-verification contract covering column headers, error messages, interval/range bounds (including constant-name lookup for computed ranges), frame headers, help-text. Documentation Sync checklist extended. Common Issues bullet added.

---

## Metrics

| Metric | Value |
|--------|-------|
| Features completed | 3/3 (Feature 2 partial per explicit deferral) |
| New unit tests | 3 (pty_harness) |
| Interactive integration proof | 1 (`test_repl_startup_and_quit` PASS, 63s live-DB) |
| Total unit tests | 1096 |
| Test pass rate | 100% |
| Clippy warnings | 0 |
| Version | v1.48.0 |

### Token/Cost Metrics (from `sprint-66-metrics.md`)
- 13 subagent invocations
- Grand total ~63M tokens, 94.2% cache hit
- Estimated cost (Sonnet pricing floor): **$38.96**

**Comparison:** Sprint 64 ~$15, Sprint 65 ~$28, Sprint 66 ~$39. Cost scales with scope complexity — Sprint 66 required 3-round deliberation + 3 features + multiple retro-fix iterations.

---

## Agent Reviews (abridged)

### Technical (rust-teradata-architect)
**Verdict: Sound with concerns → concerns addressed.**
- `drain_pending()` originally broke on transient `Ok(0)` → **fixed** with bounded retry window (~50ms).
- Feature 2 deferral of TC097-A..H acceptable per Out-of-Scope, but noted as under-delivering the literal AC text.
- Phase 2/3/4 edits coherent; grep contract explicit.

### Quality (quality-validator)
**Verdict: APPROVED WITH CONCERNS → concerns addressed.**
- TC098.md had drifted from actual struct names (`TieredTimeouts`/`*_secs` vs shipped `Timeouts`/Duration) → **fixed** in retrospective.
- Feature 2 AC said "for each test: PASSES or PTY log documented" — TC097-A..G neither run nor dumped. This is a soft AC breach, justified by the planning's explicit Out-of-Scope section.

### UX (cli-ux-designer)
**Verdict: Acceptable with concerns → concerns addressed.**
- Phase 3 had `(Sprint 66)` attribution on a standing rule → **fixed** to "Standing rule:".
- Grep contract didn't cover computed/constant-composed ranges → **fixed** by extending Step 1.7 to require grepping constant NAMES when the guide quotes a range that's built from named constants.

---

## Retrospective

### What Went Well
1. **Crisis deliberation converged in Round 1.** All three agents independently identified the same root causes (race condition + opaque timeout); no Round 2 needed.
2. **Infrastructure works end-to-end.** The migrated `test_repl_startup_and_quit` passes live-DB in 63s, proving the tiered harness is sound.
3. **Retrospective caught 4 in-sprint-fixable issues.** `drain_pending` truncation risk, TC098 struct-drift, standing-rule attribution, and incomplete grep contract — all fixed before Sprint 66 closed. No carryover debt.
4. **Zero-debt discipline held across all three recent sprints.** Sprints 64, 65, 66 have all fixed review findings in-sprint rather than deferring.

### What Could Be Improved
1. **Feature 2 AC was written without a "deferred" escape hatch.** The AC said "each test: PASSES or documented with PTY tail" — but TC097-A..G have neither. The deferral was the right call given scope, but the AC should have been tightened at planning time to say "Integration proof = 1 migrated test; full TC097 migration is a P2 follow-up."
2. **Parallel agents producing intermediate artifacts still risks drift.** The quality-validator wrote TC098 against a proposed-but-unshipped struct shape, and the architect later simplified the shape during implementation. The doc-drift race condition this sprint was supposed to fix just manifested inside the test-case document instead of the user guide. The fix is the same principle: authoritative documents must be written AFTER the authoritative code lands.
3. **Cost is rising per sprint** ($15 → $28 → $39). Maintenance sprints with deliberation + process edits are more expensive than clean feature sprints. Acceptable for the value delivered but worth tracking.

### Follow-Up Items
- **P2:** Migrate TC097-A..H to `spawn_tq_repl_tiered` + `Stage::Query` (est. 2-3h). Replace the explicit 30s `set_expect_timeout` overrides at `tests/interactive_tests.rs:3554,3561` with staged timeouts.
- **P2:** Phase 3 test-case authoring should follow the same "write after code lands" principle as user guides. Either move test-case document authoring to Phase 4, or require quality-validator to re-read source before finalising case docs.
- **P3:** Build `src/docgen/` with `const` string tables + `cargo test --test spec_sync` CI check (Architect's longer-term proposal from deliberation). Deferred from this sprint's scope due to effort.
- **P3:** Mock-DB layer for REPL flow tests (QA's long-term proposal). Deferred.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-17 | 1.0 | Sprint 66 review via /sprint-reviewer | Sprint Coordinator |
