# Sprint 66 Crisis Deliberation

## Round 1 Summary

### Problem Statement

1. **Doc drift**: cli-ux-designer running in parallel with rust-teradata-architect reads the codebase mid-mutation and produces user docs that describe invented/stale behavior. Recurrent since Sprint 22, always caught in retrospective, never prevented.
2. **Interactive-test timeout**: `spawn_tq_repl()` uses 20s `expect_timeout`. Live `/sessions` query against the test endpoint exceeds this. Sprint 65's 8 TC097 tests FAILED with ExpectTimeout, leaving AC-4..AC-9 without execution proof.

### Agent Perspectives

#### cli-ux-designer
- **Root cause:** A sequencing assumption the framework never enforces. Parallelism is correct for efficiency, but guide-writing describes concrete output strings that only stabilize AFTER implementation commits.
- **Proposed solution:** (1) Split UX work into two phases — specs/ACs in parallel with architect (Phase 2), user-guide prose AFTER merge (Phase 4). (2) Phase 4 checklist: UX verifies every example string against source literals via grep.
- **Effort:** Small.

#### rust-teradata-architect
- **Root cause:** Doc drift = no machine check of spec↔code; timeout = wrong budget for cold Teradata endpoint (TLS + auth + catalog warm-up eats 8-15s before query even runs). Harness swallows partial output on timeout, making failures opaque.
- **Proposed solution:** (1) Doc drift: sequence UX after architect + `src/docgen/` with `const` string tables + `tests/spec_sync.rs` CI test diffing generated vs committed fragments. (2) Timeout: three-tier timeouts (connect=45s, prompt=15s, query=30-60s); configurable via `TQ_TEST_*_TIMEOUT` env vars; **dump last 4KB of PTY buffer to `tests/results/sprint-N/TCxxx.pty.log` on any ExpectTimeout** — non-negotiable for observability.
- **Effort:** Doc drift Medium, timeout Small.

#### quality-validator
- **Root cause:** Doc drift is a race condition, not a doc problem. Timeout is symptom — real issue is that `spawn_tq_repl()` couples PTY lifecycle + query latency + assertion logic. All ACs untestable when slowest layer times out.
- **Proposed solution:** (1) Pre-commit string-consistency check: extract user-visible strings into canonical `strings.toml`, fail commit if spec/guide references a string not in source. (2) Three-layer refactor: pure rendering functions (unit-testable), step-wise PTY timeouts (prompt/dispatch/receipt), and a mock-DB layer for REPL flow tests.
- **Effort:** Doc-check Small, test refactor Large.

### Synthesis

**Areas of agreement (unanimous):**
- Doc drift is a race condition caused by parallel execution. The fix is sequencing, not better prompts.
- Interactive-test timeout is a symptom of an undifferentiated, opaque harness. Observability (PTY buffer dump) and tiered timeouts are orthogonal to "raise the number."
- Pure rendering functions unit-testable without DB are the right long-term architecture for REPL features.

**Areas of disagreement:**
- UX wants pure process change (small effort). Architect wants tooling (specgen + CI test, medium effort). QA wants a pre-commit hook (small effort). These are complementary, not competing — pick the cheapest that closes the loop.
- Architect and QA both endorse the pure-function extraction path; UX endorses it secondarily. Disagreement is on scope: QA says "Large," Architect and UX see it as a longer-term direction, not a Sprint 66 item.

**Emerging consensus:**
1. Sequence cli-ux-designer AFTER architect for user-guide prose. Keep parallelism for spec design in Phase 2.
2. Dump PTY buffer on ExpectTimeout — unanimous, trivial effort, huge debugging win.
3. Tier the interactive timeouts — trivial effort, makes failures localisable.
4. Defer specgen + mock-DB + pure-function extraction to a future sprint — too large for this one.

---

## Final Decision

**Sprint Focus:** Cheap, high-impact process + observability hardening. No large refactors.

**Rationale:** Round 1 reached strong convergence on root causes and the cheapest effective fixes. Specgen, mock-DB, and pure-function extraction are real investments but would consume the entire sprint and prevent the small wins. Sprint 66 closes the bleed; those larger items stay in the backlog for dedicated future sprints.

**Acceptance Criteria:**
- [ ] User-guide edits for Sprint 66 onwards are produced AFTER the architect's implementation lands — encoded in Phase 3/4 of the sprint-coordinator skill.
- [ ] The sprint-coordinator Phase 4 checklist includes "UX verified all example output strings against source code literals (grep-based)."
- [ ] `spawn_tq_repl()` in `tests/interactive_tests.rs` is split into tiered timeouts: connect (45s), prompt-ready (15s), query-result (60s). Each tier fails with a specific, differentiated error.
- [ ] On any `ExpectTimeout`, the last 4KB of the PTY output buffer is dumped to `tests/results/sprint-66/TCxxx.pty.log` and included in the failure message.
- [ ] Sprint 65's TC097 tests are re-run with the new harness. Any that still fail are documented with concrete root cause from the dumped PTY log (not just "ExpectTimeout").
- [ ] Skill and process docs updated to reflect the new sequencing and checklist rule.
