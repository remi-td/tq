---
sprint: 71
start_date: 2026-06-11
target_completion: 2026-06-11
status: Complete
---

# Sprint 71 Planning: Deterministic Agent-Safe Query Execution (#45)

## Reality Check Summary

- **Reviewed sprints:** 70 (technical review), 69 (PTY cursor fix — feature), 68 (test debt closure — maintenance)
- **Patterns detected:** None. Healthy velocity — zero TODO/FIXME debt for 6+ consecutive sprints, 100% test pass rate, clippy clean, 1,347 tests. Sprint 70's audit explicitly rated the codebase "healthy and disciplined."
- **Decision:** **Feature / Bug-fix Sprint**
- **Rationale:** No crisis. A single, well-specified, high-value GitHub issue (#45) targets the core agent-friendliness contract of `tq`. The work is cohesive (input selection + agent-safe classification + runtime limits) and bounded. Notably, Sprint 70's roadmap already flagged the request/query-timeout follow-up (review line 80), so this sprint also discharges a carried-forward item.

---

## Sprint Overview

**Sprint Goal:** Make `tq query` behave deterministically in agent harnesses and harden `--agent-safe`, so autonomous agents can rely on `tq` without per-agent wrapper scripts.

**Sprint Theme:** Agent-friendliness — deterministic input selection, structural SQL safety classification, and real runtime guardrails.

**Issue:** [#45](https://github.com/remi-td/tq/issues/45) — `[BUG] Make tq query deterministic in agent harnesses and harden --agent-safe`

**Target Version:** v1.52.0

---

## Objectives

1. **Deterministic input-source selection** — select the SQL source from explicit syntax (argument > file > non-TTY stdin), never from transient file-descriptor readiness. Remove the `poll`/`FIONREAD`/`fstat` probe and the Unix/non-Unix semantic split.
2. **Structural agent-safe classification** — replace first-keyword classification with a tokenizer-based classifier that handles leading comments, `WITH` CTEs, and `LOCKING` modifiers; introduces a `Maintenance` category (`COLLECT STATISTICS`) and fails closed on unknown syntax (`Unknown`, not `Ddl`).
3. **Runtime guardrails** — add a query/request timeout distinct from the connection timeout, with a conservative agent-safe default and a structured `QUERY_TIMEOUT` error; clarify `--max-rows` as a client fetch/output cap in help and docs.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Deterministic input-source selection

**Description:** Rewrite `determine_input_source` to use the precedence contract from the issue (Recommended Solution A). If a positional query is present, use it and never inspect stdin. Else if `--file` is present, use it and never inspect stdin. Else if stdin is non-TTY, select stdin and perform a normal blocking read to EOF. Else return `No query provided`. Empty stdin returns `Empty query received from stdin` (distinct from `No query provided`). Delete `stdin_has_data()` and the `libc` poll/FIONREAD/fstat machinery; unify Unix and non-Unix behavior.

**Acceptance Criteria (from issue — Input handling):**
- [ ] Positional SQL succeeds with TTY stdin.
- [ ] Positional SQL succeeds with stdin attached to `/dev/null`.
- [ ] Positional SQL succeeds with an empty inherited pipe.
- [ ] Positional SQL succeeds when an immediate producer is also attached; stdin is ignored.
- [ ] Positional SQL succeeds when a delayed producer is also attached; behavior identical to the immediate case.
- [ ] `--file` behavior is independent of stdin state.
- [ ] Stdin-only SQL works with an immediate producer.
- [ ] Stdin-only SQL works with a delayed producer.
- [ ] Empty stdin returns `Empty query received from stdin`, not `No query provided`.
- [ ] Unix and Windows follow the same source-precedence contract.
- [ ] No readiness probe is used to infer whether stdin is the chosen source.

**Behavior change to document:** `echo "ignored" | tq query "SELECT 1"` now executes the explicit query (stdin ignored) instead of returning `Multiple input sources`. This matches `psql -c`/`-f`. The existing `validate_input_sources` conflict check and its "Multiple input sources" error are **removed** (no longer reachable under the new precedence). The `Multiple input sources` error-content test is removed/replaced accordingly.

**Reference:** Issue #45 §Recommended Solution A; `src/commands/query.rs:120-303`

**Estimated Complexity:** Medium (net code reduction)

---

#### Feature 2: Structural agent-safe classification

**Description:** Replace `classify_statement` (which reuses the display helper `get_statement_type` + first-token allowlist mapping unknowns to DDL) with a dedicated structural classifier that reuses the existing SQL lexical state machine in `src/sql/parser.rs` to skip any mixture of whitespace, line comments, and block comments, then classify the effective top-level operation.

New result type:
```rust
enum StatementSafety {
    ReadOnly,
    Maintenance,
    Dml,
    Ddl,
    Unknown { token: Option<String>, reason: String },
}
```

Required behavior:
1. Skip arbitrary interleaved whitespace / line / block comments before the first significant token.
2. For `WITH`, identify the final top-level operation after all CTE definitions (parenthesis-aware).
3. For `LOCKING`/`LOCK`, consume the request modifier and classify the operation it modifies.
4. Allow `SELECT`/`SEL`, `SHOW`, `HELP`, `EXPLAIN` as `ReadOnly`.
5. Classify `COLLECT STATISTICS` (and `COLLECT STATS`) as `Maintenance`, blocked by default.
6. Classify `INSERT`/`INS`, `UPDATE`/`UPD`, `DELETE`/`DEL`, `MERGE`, `UPSERT` as `Dml`.
7. Classify known DDL/DCL (`CREATE`, `DROP`, `ALTER`, `RENAME`, `REPLACE`, `GRANT`, `REVOKE`, …) explicitly as `Ddl`.
8. Fail closed on unknown syntax → `Unknown`, surfaced as a distinct `AGENT_SAFE_UNCLASSIFIED` error, **not** mislabeled DDL.

Add `--allow-maintenance` flag to `QueryArgs` to opt into `Maintenance` statements. Error messages must identify the effective operation and the rejection reason.

**Note on `sqlparser-rs`:** The issue suggests evaluating the upstream `TeradataDialect`. **Executive decision: build the classifier on the existing in-tree lexer (`src/sql/parser.rs`) rather than adding the `sqlparser-rs` dependency.** Rationale: (a) zero new dependency / attack surface (consistent with Sprint 70's `atty` removal); (b) the lexer already correctly handles quotes/comments/parens and is used by `parse_statements`; (c) Teradata `LOCKING` request modifiers need tq-side handling regardless; (d) fail-closed `Unknown` classification is trivial to guarantee in-tree. The architect may revisit only if in-tree classification proves infeasible for the required cases.

**Acceptance Criteria (from issue — Agent-safe classification):**
- [ ] A read-only CTE (`WITH x AS (SELECT 1) SELECT * FROM x`) is accepted.
- [ ] Multiple and mixed leading comments are accepted before read-only SQL (`/* a */ /* b */ SELECT 1`, and interleaved `--`/`/* */`).
- [ ] `LOCKING ... SELECT` is accepted when the effective operation is read-only.
- [ ] `LOCKING ROW FOR WRITE UPDATE/DELETE/INSERT/MERGE` is blocked without the corresponding write opt-in.
- [ ] `COLLECT STATISTICS` is blocked by default and requires `--allow-maintenance`.
- [ ] Unknown syntax fails closed with an `AGENT_SAFE_UNCLASSIFIED` (or equivalent) error.
- [ ] Unknown syntax is not mislabeled as DDL.
- [ ] Errors identify the effective operation and the reason for rejection.
- [ ] Regression tests cover Teradata abbreviations `SEL`, `INS`, `UPD`, `DEL`.

**Reference:** Issue #45 §Recommended Solution B; `src/commands/query.rs:850-912`, `src/sql/parser.rs`

**Estimated Complexity:** High

---

### P1 - High Priority (Should Have)

#### Feature 3: Query timeout + `--max-rows` documentation

**Description:** Add a `--query-timeout` option distinct from the connection `--timeout`:
- `--timeout`: connection establishment only (unchanged).
- `--query-timeout <DURATION>`: execution/fetch deadline.
- Agent-safe mode applies a conservative finite default query timeout when none is given explicitly.
- Timeout produces a structured `QUERY_TIMEOUT` JSON error with documented retryability.
- Attempt to cancel/abort the active request when the driver supports it; otherwise abandon local output and return the structured error.

Also clarify in `--help` and JSON metadata/docs that `--max-rows` is a **client fetch/output cap** (`tq` fetches at most `max_rows + 1`), not a database workload limit, and that maintenance operations require explicit opt-in.

**Implementation note:** The architect will determine in Phase 2 whether the `teradatasql` driver exposes a native request timeout parameter. **Feasibility fallback (scope guard):** if driver-native cancellation is not provable against the live DB in this session, deliver `--query-timeout` enforcement via a client-side execution deadline (worker thread + deadline) that returns the structured `QUERY_TIMEOUT` error and closes the session, and document the cancellation limitation honestly. Flag plumbing, default wiring, structured error, and `--max-rows` doc clarification are **not** subject to the fallback — they ship regardless.

**Acceptance Criteria (from issue — Runtime limits):**
- [ ] Connection timeout and query timeout are separate, documented controls.
- [ ] Agent-safe mode has a finite query timeout by default, or requires one explicitly.
- [ ] Query timeout produces a structured JSON error (`QUERY_TIMEOUT`).
- [ ] `--max-rows` documentation states it is a client fetch/output cap.
- [ ] Timeout attempts to cancel/abort the active request rather than only abandoning local output (or the limitation is documented honestly if the driver cannot).

**Reference:** Issue #45 §Recommended Solution C; Sprint 70 review line 80; `src/cli.rs`, `src/db/connection.rs`, `src/db/client.rs`, `src/error.rs`

**Estimated Complexity:** High

---

### P2 - Medium Priority (Nice to Have)

#### Feature 4: Diagnostics & least-privilege documentation

**Description:** Document `--agent-safe` as defense-in-depth, not a security boundary; point operators to Teradata database-side least privilege (dedicated agent user, `SELECT`/metadata-only grants, roles). Improve rejection diagnostics where cheap.

**Acceptance Criteria:**
- [ ] `docs/specifications/security.md` documents `--agent-safe` as defense-in-depth and recommends DB-side least privilege.
- [ ] `--agent-safe` help text references the least-privilege guidance.

**Reference:** Issue #45 §B (defense in depth), Non-Goals

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Agent-specific wrapper scripts or per-harness invocation rules (issue Non-Goals).
- An explicit `--stdin` flag or `--file -` selector (issue notes these can be added later; not required to fix current behavior).
- Replacing client-side classification with a full SQL parser dependency (`sqlparser-rs`) — see Feature 2 executive decision.
- Treating client-side classification as a substitute for database privileges.

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working as specified.
- [ ] All P1 features implemented and tested (or explicitly deferred with rationale).
- [ ] 100% test pass rate (unit + integration); interactive/live tests executed with `--ignored` where applicable.
- [ ] All acceptance criteria met for delivered features.
- [ ] Specifications + design + user docs synchronized with behavior.
- [ ] Zero technical debt introduced; `scripts/ci-check.sh` green (clippy `--all-targets -D warnings` + full `cargo test`).
- [ ] Validated by quality-validator and tq-project-manager.

---

## Dependencies

### External
- `teradatasql` driver behavior for request/query timeout and cancellation (Feature 3) — to be probed against the live trial DB in Phase 2.
- Live trial Teradata system (`.env` `TQ_LOGON`) for behavioral verification of agent-safe classification and timeout.

### Prerequisite Work
- None blocking. Builds on Sprint 70's `connect_timeout` wiring (review line 41) and existing `src/sql/parser.rs` lexer.

### Blockers
- None known. Driver cancellation uncertainty for Feature 3 is mitigated by the documented client-side-deadline fallback.

---

## Risks & Mitigation

### Risk 1: Driver does not support query cancellation
- **Probability:** Medium · **Impact:** Medium
- **Mitigation:** Client-side execution-deadline fallback (worker thread + deadline + session close), structured `QUERY_TIMEOUT` error, honest documentation of the cancellation limitation. Flag/default/error/doc work ships regardless.

### Risk 2: Removing the input-source conflict changes observable behavior
- **Probability:** High (intentional) · **Impact:** Low
- **Mitigation:** Documented as an explicit, deliberate contract change (matches `psql`). Update/replace the affected unit tests and the spec; call it out in the sprint review and the issue closure comment.

### Risk 3: CTE/LOCKING tokenization edge cases (nested parens, comments inside modifiers)
- **Probability:** Medium · **Impact:** Medium
- **Mitigation:** Reuse the proven in-tree lexer state machine; fail closed to `Unknown` on anything not provably classified; comprehensive unit tests including the issue's example table.

### Risk 4: Single-session scope overrun (3 substantial features)
- **Probability:** Medium · **Impact:** Medium
- **Mitigation:** P0 (input + classifier) is the correctness/safety core and ships first. P1 timeout has a scoped-down fallback. P2 is docs-only and droppable. Quality gate never compromised.

---

## Action Items from Previous Sprint

From `sprint-70-tech-review.md` roadmap (only those relevant here):
- [ ] Make `connect_timeout`/`request_timeout` first-class; consider a separate query/request timeout knob and surface semantics in `--help` (review line 80) → **addressed by Feature 3.**

Other Sprint 70 roadmap items (`--password-stdin`, JSON errors for all paths, completions/man page, module decomposition) are **not** in this sprint's scope — they are independent of #45 and remain on the backlog.

---

## GitHub Issues

### Selected for Sprint
- #45: Make `tq query` deterministic in agent harnesses and harden `--agent-safe` (bug, priority-high)

### Deferred
- None.

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update `docs/specifications/cli-interface.md` (input precedence contract, `--query-timeout`, `--allow-maintenance`, `--max-rows` semantics).
- Update `docs/specifications/security.md` (agent-safe defense-in-depth, DB least privilege).
- Update `docs/specifications/error-handling.md` (`QUERY_TIMEOUT`, `AGENT_SAFE_UNCLASSIFIED`, removal of `Multiple input sources`).
- Update relevant user guide(s) with grep-verified examples.

### rust-teradata-architect (Opus)
- Implement Features 1–3; reuse `src/sql/parser.rs` lexer for the classifier.
- Probe `teradatasql` timeout/cancellation feasibility against the live DB early in Phase 2.
- Update `docs/design/` (input selection, agent-safe classification, connection/timeout) as patterns change.
- Unit tests for all new code; zero debt.

### quality-validator (Sonnet)
- Author test cases covering every acceptance-criterion checkbox above, including the issue's classification example table and Teradata abbreviations.
- Execute full suite + interactive/live (`--ignored`) tests; provide execution proof.
- Validate the delayed-producer and `/dev/null`/empty-pipe input scenarios specifically.

### tq-project-manager (Haiku)
- Validate completion against Definition of Done; commit, push, tag `v1.52.0`; close #45.

---

## Files Involved

### Objective 1: Deterministic input selection
- `src/commands/query.rs` (`determine_input_source`, remove `stdin_has_data`, `validate_input_sources`)
- `src/main.rs` (remove the pre-connection `validate_input_sources` call if present)
- `Cargo.toml` (drop `libc` if no longer used elsewhere — verify first)

### Objective 2: Agent-safe classification
- `src/commands/query.rs` (`StatementSafety`, `classify_statement`, `validate_agent_safe`)
- `src/sql/parser.rs` / `src/sql/mod.rs` (expose tokenizer pieces if needed)
- `src/cli.rs` (`--allow-maintenance`)
- `src/error.rs` (`AGENT_SAFE_UNCLASSIFIED`, maintenance-blocked variants)

### Objective 3: Query timeout
- `src/cli.rs` (`--query-timeout`, `--max-rows` help text)
- `src/db/connection.rs`, `src/db/client.rs` (timeout wiring/enforcement)
- `src/error.rs` (`QUERY_TIMEOUT`)
- `src/main.rs` (plumbing)

### Documentation
- `docs/specifications/{cli-interface,security,error-handling}.md`
- `docs/design/{connection-management,cli-interface}.md`
- User guide(s) under `docs/`

---

## Notes

- This is a single-session sprint. P0 first, then P1, then P2 (droppable).
- Behavior change (stdin ignored when explicit source present) is intentional and must be called out in the issue-closure comment.
- `--agent-safe` is defense-in-depth, not a security boundary — keep that framing in all docs.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-06-11 | 1.0 | Initial Sprint 71 plan for #45 | Sprint Coordinator |
