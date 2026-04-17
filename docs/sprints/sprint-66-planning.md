---
sprint: 66
start_date: 2026-04-17
target_completion: 2026-04-17
status: Planning
---

# Sprint 66 Planning: Maintenance — Test Infra + Doc-Drift Pattern

## Sprint Overview

**Sprint Goal:** Close two recurring framework issues surfaced across recent sprints: (1) interactive-test timeout blocking REPL acceptance-criteria validation, (2) documentation-vs-code drift when parallel agents produce docs against intermediate code states.

**Sprint Theme:** Maintenance Sprint — Framework Hardening
**Type:** Maintenance Sprint
**Date:** 2026-04-17

---

## Crisis Summary

**Patterns Detected:**

1. **Documentation-vs-code drift when parallel agents produce user-facing docs.** The cli-ux-designer agent edits `docs/user/*.md` in parallel with the rust-teradata-architect's implementation. Because the designer reads the codebase at the moment the agent launches (and the architect is still mutating it), the guide frequently documents invented or superseded behaviour. Caught and fixed in retrospective rather than before commit.

2. **Interactive-test timeout prevents execution of REPL acceptance criteria.** The `spawn_tq_repl()` harness uses a 20-second `expect_timeout` that is too short for the live test database's `/sessions` query. All 8 Sprint 65 TC097 interactive tests FAILED with `ExpectTimeout`. Unit tests cover logic, but AC-4 through AC-9 (all PTY-dependent) have zero execution proof.

**Evidence:**

- **Sprint 22** (v1.9.0): User guide documented SQL LIKE `%` pattern syntax; implementation used glob `*`. Caught in retrospective.
- **Sprint 23**: `--force` flag documented in user guide but not implemented. Caught in retrospective.
- **Sprint 38**: Output schema drift (Node Count / PE Count columns differed between guide and implementation).
- **Sprint 39**: Multiple instances — session-history schema mismatch (Session/User/Query Text vs multi-query history), `/q` alias collision, sysconfig help text describing unimplemented features.
- **Sprint 65** (v1.47.0): Interval range drift (guide said "2 to 300", code enforces "1 to 3600"), invented frame-header format, invented error-state border. Caught in retrospective.
- **Sprint 65**: 8/8 TC097 interactive tests FAILED with ExpectTimeout against live Teradata endpoint. AC-4..AC-9 have zero execution proof.

**Impact:**

- Doc drift erodes user trust and creates support churn when docs describe behaviour that doesn't exist.
- Interactive-test timeout means REPL features ship with claim-without-proof, undermining the "tests must be executed, not code reviewed" coordinator rule.
- Both issues reduce the truthfulness of sprint-completion claims, which is the single thing the quality gate is supposed to protect.
- Without a permanent fix, both patterns will keep re-occurring — the coordinator has been catching them in retrospective instead of preventing them.

---

## Objectives

Based on Round 1 deliberation (see `sprint-66-crisis-deliberation.md`), three cheap-but-high-impact actions:

1. **Sequence guide writing after implementation** to eliminate doc-vs-code drift as a race condition.
2. **Make interactive-test timeouts observable and tiered** so AC failures produce actionable diagnostics instead of opaque ExpectTimeout.
3. **Re-run Sprint 65's TC097 tests** with the new harness to close the execution-proof gap for REQ-REPL-SESSIONS-WATCH AC-4..AC-9.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Tiered interactive-test timeouts with PTY buffer dump

**Description:** Refactor `tests/interactive_tests.rs::spawn_tq_repl()` and its usages to support three named timeout tiers. On any timeout, dump the tail of the PTY buffer to a file and include a summary in the test failure message.

**Acceptance Criteria:**
- [ ] `spawn_tq_repl()` accepts timeouts for: connect/auth (default 45s), prompt-ready (default 15s), query-result (default 60s)
- [ ] Each tier fails with a distinct, named error (e.g., `PtyError::ConnectTimeout`, `::PromptTimeout`, `::QueryTimeout`) so test failures identify the stage
- [ ] On ANY expect timeout, the last 4096 bytes of the PTY read buffer are written to `tests/results/sprint-66/<test_name>.pty.log` and the failure message references that file
- [ ] Environment overrides: `TQ_TEST_CONNECT_TIMEOUT`, `TQ_TEST_PROMPT_TIMEOUT`, `TQ_TEST_QUERY_TIMEOUT` (all u64 seconds)
- [ ] At least one existing interactive test (any passing one) is updated to use the new tiered API to prove it integrates correctly
- [ ] Unit test for the buffer-dump path (can be done without a PTY by calling the dump function directly with synthesised bytes)

**Reference:** Sprint 66 crisis deliberation — Architect + QA convergent recommendation
**Estimated Complexity:** Small/Medium

---

#### Feature 2: Sprint 65 TC097 re-execution with new harness

**Description:** Re-run Sprint 65's 8 TC097 interactive tests with the new tiered timeouts. Capture concrete root cause for any that still fail using the dumped PTY log.

**Acceptance Criteria:**
- [ ] TC097-A..H re-executed with new harness
- [ ] For each test: either it now PASSES (AC-4..AC-9 get real execution proof), or the failure is documented in `tests/results/sprint-66/tc097-failure-analysis.md` with the specific stage that timed out (connect vs prompt vs query) and the PTY tail
- [ ] If any fail, a follow-up item is created with a concrete next action (e.g., "connect takes 38s cold — env needs warmer DB, not a code fix")

**Reference:** Sprint 65 review — P2 follow-up
**Estimated Complexity:** Small (depends on Feature 1)

---

#### Feature 3: Sequence user-guide writing after implementation

**Description:** Update the sprint-coordinator skill to sequence cli-ux-designer's user-guide edits AFTER the rust-teradata-architect's implementation lands. Spec work stays in Phase 2 (parallel); user-guide prose moves to Phase 4.

**Acceptance Criteria:**
- [ ] `.claude/skills/sprint-coordinator/process/phase2-design.md` updated: cli-ux-designer produces specs only, no user-guide prose
- [ ] `.claude/skills/sprint-coordinator/process/phase3-build-test.md` and `phase4-ship.md` updated: user-guide prose is an EXPLICIT Phase 4 step, run AFTER code is merged, with a grep-verify instruction for every example output string
- [ ] Phase 4 "Documentation Sync" checklist gains the line: "UX verified all user-guide example output strings exist verbatim in source literals (grep check)"
- [ ] No impact on existing specs/user-guide content — only the PROCESS rule is changing

**Reference:** Sprint 66 crisis deliberation — unanimous agent agreement
**Estimated Complexity:** Small

---

### Out of Scope (Explicit)

- `src/docgen/` module with const string tables and `cargo spec-sync` CI test (Architect proposal) — **Medium/Large effort, deferred to future sprint** as a dedicated framework-tooling investment
- Mock-DB layer for REPL flow tests (QA proposal) — **Large effort, deferred**
- Pure-function extraction of watch/render logic (all 3 agents endorsed as long-term direction) — **Large, deferred**
- New feature work (PMON Alerting #23 etc.) — this is a Maintenance Sprint

---

## GitHub Issues

No new GitHub issues addressed. This sprint targets internal framework gaps surfaced by Sprint 64 and Sprint 65 retrospectives.

---

## Dependencies

- None. All changes are internal to `tests/interactive_tests.rs` and `.claude/skills/sprint-coordinator/`.

---

## Definition of Done

- [ ] All three features implemented per acceptance criteria
- [ ] 100% unit test pass rate
- [ ] No new clippy warnings
- [ ] Sprint 66 TC097 re-run documented (pass or failure-with-root-cause)
- [ ] Version bumped to v1.48.0
- [ ] Git tag + release published
- [ ] Process docs updated and committed
