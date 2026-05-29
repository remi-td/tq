# Sprint 70 — Technical Quality & Architecture Review

**Date:** 2026-05-29
**Type:** Cross-cutting technical review (not a feature sprint)
**Scope:** Rust code quality, database-layer robustness, security, CLI/UX, and QA/test architecture.
**Baseline:** v1.51.0 — ~42.6k LOC Rust, 1,331 tests (`#[test]`/`#[tokio::test]`), clippy clean.

This sprint audited the whole codebase from five angles (one specialist reviewer each), verified the
highest-priority findings against the live trial Teradata system, and implemented the most urgent
low-risk fixes. The remainder is captured below as a prioritized PR roadmap.

---

## Executive summary

`tq` is a **healthy, disciplined codebase**. Error handling is exemplary (fully-typed `thiserror`
errors with exit codes and dual human/JSON rendering, no panic-on-bad-input in production paths),
there is essentially no TODO/FIXME debt, SQL generation is consistently injection-safe via centralized
quoting helpers, and — verified against the live DB — **there is no DECIMAL/NUMBER/BIGINT precision
loss** (the driver delivers those types as JSON strings that `tq` preserves verbatim; `DECIMAL(38,0)
9999999999999999999` round-trips byte-exact).

The material gaps are **operational and process** rather than correctness:

1. **CI was not running the integration tests** (`cargo test --lib` only) — ~101 DB-free, full-binary
   tests never executed in CI. *(fixed this sprint)*
2. **Connection `--timeout` was parsed but never sent to the driver** — dead config. *(fixed this sprint)*
3. **`atty` dependency** carries RUSTSEC-2021-0145 and would fail the `cargo audit` CI job. *(fixed this sprint)*
4. Agent-friendliness gaps (no `--password-stdin`, JSON-error path doesn't cover all commands,
   `ping` has no `--format`), discoverability gaps (no shell completions / man page), and a dead
   `--quiet` flag — captured as roadmap PRs below.

---

## Implemented this sprint (branch `sprint-70-tech-review`)

| # | Change | Severity | Files | Verification |
|---|--------|----------|-------|--------------|
| 1 | **CI now runs the full test suite** (`cargo test` instead of `cargo test --lib`) and GitHub clippy now lints `--all-targets` (matches `ci-check.sh`) | P0 | `.github/workflows/ci.yml`, `scripts/ci-check.sh` | `cargo test`: 1347 pass, 88 ignored, 0 fail |
| 2 | **Removed unmaintained `atty`** → `std::io::IsTerminal` (already used elsewhere); drops RUSTSEC-2021-0145 and a dependency | P1 | `Cargo.toml`, `src/commands/repl/state.rs` | clippy clean, builds |
| 3 | **Wired connection `timeout` → driver `connect_timeout`** (previously parsed but never sent); whole-second, 1s floor | P1 | `src/db/connection.rs` | Live DB: connects with param accepted, no regression; 2 new unit tests |
| 4 | **Hardened `inspect` SHOW VIEW/MACRO quoting** to use centralized `quote_qualified_name` (doubles embedded quotes) | P2 | `src/commands/inspect.rs` | Live DB: `inspect dbc.tablesv` definition unchanged; new quote-doubling test |

All changes pass the full `scripts/ci-check.sh` gate (clippy `--all-targets -D warnings` + `cargo test`).

### Note on `connect_timeout` enforcement
The driver **accepts** the parameter (verified: the demo DB connects successfully with it present) and
connectivity is not regressed. Strict timeout *enforcement* could not be proven against synthetic
hosts in the sandbox because Teradata COP hostname discovery short-circuits unreachable IPs before the
TCP timeout applies. The change is strictly better than the prior behavior (timeout fully ignored) and
uses a documented teradatasql connection parameter.

---

## Roadmap (proposed PRs, prioritized)

### P0 / P1 — High value

- **PR: Agent-friendly credential input — `--password-stdin`.** Add a flag that reads one line from
  stdin (the `mysql`/`docker login` pattern). The password-on-`--logon` path is visible via `ps`/`/proc`
  and shell history; today the only non-file alternative is an env var. Also: the help/error text
  advertises an **interactive prompt that does not exist** (`resolve_password` returns `MissingPassword`
  instead of prompting) — either implement it (`rpassword`) or remove the claim. *(Security P1)*

- **PR: Structured JSON errors for all failure paths.** `main.rs` derives `is_json_format` from the
  per-command `--format`, so `ping`, `profiles`, `profile`, and all clap parse errors never emit JSON
  even under `--format json`. Make the error envelope honor a global format/`TQ_FORMAT`, and use
  `Cli::try_parse()` to JSON-ify usage errors. Undercuts the "agent-friendly" promise where it matters
  most. *(UX P0)*

- **PR: `ping --format`/`--output` + JSON envelope.** The canonical agent health-check has no machine
  output, breaking the otherwise-universal pattern. Add `ok`/latency/attempts JSON. *(UX P0)*

- **PR: Shell completions + man page.** Add `clap_complete` (`tq completions <shell>`) and
  `clap_mangen`. Near-free with clap; baseline discoverability expectation. *(UX P0)*

- **PR: Wire or remove the dead `--quiet` flag** (`cli.rs:160`, never read). Advertised flag that does
  nothing. *(UX P1)*

- **PR: Make `connect_timeout`/`request_timeout` first-class.** Follow-up to this sprint's fix:
  consider a separate query/request timeout knob distinct from connect timeout, and surface the
  semantics in `--help`. *(DB P1)*

### P2 — Robustness & maintainability

- **PR: Type-aware value conversion in `db/client.rs`.** `convert_value` ignores `col.data_type` and
  uses content heuristics (`looks_like_date`/`looks_like_timestamp`) that can misclassify legitimate
  VARCHARs (e.g. `"1234-56-78"` → Date). Drive conversion from the known column type; delete the
  heuristics. Add a regression test asserting the DECIMAL(38,x)/NUMBER/BIGINT string contract so a
  future driver bump that breaks it is caught. *(DB P2 — latent)*

- **PR: RAII connection guard.** Cleanup currently relies on straight-line code reaching
  `go_close_connection_wrapper`. Wrap `(u_log, conn_handle)` in a `Drop` guard so server-side sessions
  close on all early returns (and to be safe if `panic = abort` is ever changed to unwind). *(DB P2)*

- **PR: Reduce `main.rs` boilerplate.** ~13 commands repeat the identical output-file-vs-stdout block
  (and a watch/output/stdout triple). Introduce a `with_output_writer(Option<&Path>, impl FnOnce(&mut
  dyn Write))` helper. ~150 lines removed. *(Rust P1)*

- **PR: Consolidate monitoring-command formatting.** ~18 commands hand-roll `match format { Table | Json
  | Csv | Markdown }` with manual escaping; route flat row sets through the existing `QueryResult`
  formatters / `monitoring_utils`. Largest single LOC-reduction opportunity. *(Rust P1)*

- **PR: Decompose oversized modules.** `metacommands.rs` (3476), `pager.rs` (3091), `cli.rs` (2284),
  `search.rs` (1933). Split into submodules; move per-command `Args` structs next to their commands.
  Pure refactor, no behavior change. *(Rust P2)*

- **PR: proptest for `sql/identifiers.rs` and `sql/parser.rs`.** Replace ~25 near-duplicate example
  tests with invariant properties ("a quoted identifier never breaks out of its quotes for any input";
  "escape is idempotent"). Highest-ROI hardening of the injection-prevention surface. *(QA P1)*

- **PR: Non-blocking live-DB + PTY CI job.** Add a scheduled (nightly) job that injects `TQ_LOGON` from
  a secret and runs `cargo test -- --ignored`, keeping the ~88 live/PTY tests out of the PR-blocking
  path but actually executed. *(QA P0 for the live suite)*

- **PR: Coverage measurement.** Add `cargo-llvm-cov` to CI (informational threshold first, then
  ratchet). True coverage of the shipped path is currently unmeasured. *(QA P1)*

- **PR: CI-reliability for `cargo audit`.** Pin `cargo-audit`, add `audit.toml` with a documented
  allow-list, consider making advisories scheduled rather than gating every PR. *(QA P2)*

### P3 — Polish

- Centralize the three near-duplicate 0600 permission checks; close the stat-then-open TOCTOU by
  `fstat`-ing the open handle; document the Windows permission-check gap. *(Security P2)*
- Zeroize the transient password `String` produced by `to_json_string()` (use `Zeroizing`/`SecretString`
  up to the FFI call). *(Security P2)*
- Use `escape_sql_like` for the `search` keyword so `%`/`_` aren't unintended wildcards; replace ad-hoc
  `.replace('\'', ...)` in `abort.rs`/`history.rs` with `escape_sql_string`. *(consistency)*
- Make `--password-file`/`--logmech` `global = true` (drop the "place before subcommand" caveats);
  expose `--format jsonl` (code already exists); honor `TERM=dumb` in color detection. *(UX P2)*
- Dependency bumps: `secrecy 0.8→0.10`, `thiserror 1→2`, `directories 5→6`; remove unused `anyhow`. *(Rust P2)*
- Error mapping: key on Teradata numeric error codes (3807/3523/3706/2631) rather than English
  substring matching, for robustness against localized/reworded driver messages. *(DB P3)*

---

## Verification artifacts
- Full suite green without a DB: `cargo test` → **1347 passed, 88 ignored, 0 failed**.
- `scripts/ci-check.sh` → **OK** (clippy `--all-targets -D warnings` clean, all unit/integration/doc tests pass).
- Live DB (`demo-…trial.teradata.com`): `ping` OK; `DECIMAL(38,0) 9999999999999999999` round-trips
  exact (no precision loss); `inspect dbc.tablesv` definition unchanged after quoting refactor;
  `connect_timeout` accepted by driver without regression.
