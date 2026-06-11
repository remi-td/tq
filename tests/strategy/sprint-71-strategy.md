# Sprint 71 Test Strategy — Deterministic Agent-Safe Query Execution (#45)

**Created:** 2026-06-11
**Author:** quality-validator
**Sprint:** Sprint 71
**Features:**
1. Deterministic input-source selection (Feature 1, P0)
2. Structural agent-safe classification (Feature 2, P0)
3. Query timeout + `--max-rows` documentation (Feature 3, P1)
4. Diagnostics & least-privilege documentation (Feature 4, P2)

---

## Tooling Assessment and New-Tool Request

### Can the existing harness exercise a delayed stdin producer?

**Assessment: YES — no new crate required.**

`tests/README.md` already documents the process-level subprocess pattern using
`std::process::Command` + `Stdio`. A delayed producer is exercised by spawning
`tq` with `Stdio::piped()`, sleeping in the test process (or using a separate
thread) before writing bytes to the child's stdin pipe, and then dropping the
pipe. The child sees a non-TTY stdin fd that delivers bytes only after a
wall-clock delay.

Example sketch (no new dependency):

```rust
#[test]
fn positional_arg_ignores_delayed_stdin() {
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
        .args(["query", "SELECT 1", "--format", "json"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn tq");

    // Write to stdin after a delay (simulates slow upstream agent)
    let mut pipe = child.stdin.take().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let _ = pipe.write_all(b"SELECT 2\n");
        // pipe dropped here → EOF
    });

    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    // stdout must contain SELECT 1 result, not SELECT 2
}
```

The above pattern is DB-free only if `tq` decides before reading stdin — which
is the whole point of the new precedence rule. Because the new
`determine_input_source` returns `InputSource::Argument` immediately without
touching stdin, the test subprocess simply exits before stdin data arrives. This
is deterministic.

### Can the existing harness distinguish TTY vs pipe?

**Assessment: YES — subprocess stdio redirection achieves this.**

- **TTY stdin simulation:** The `tq` binary running under `cargo test` never has
  a TTY attached by the test runner. The only way to exercise the TTY branch
  without a live PTY is to check the negative — i.e., spawn with
  `Stdio::inherit()` is NOT available in a test subprocess without a PTY. The
  correct approach is:
  - Unit test: directly test `io::stdin().is_terminal()` is `false` in a test
    subprocess.
  - Integration test: spawn `tq query "SQL"` with no stdin redirection in a
    test subprocess (`Stdio::inherit()`). When run by `cargo test` (which does
    not allocate a TTY), stdin is inherited from the test runner which has a
    pipe. This exercises the "non-TTY but positional arg takes priority" path
    correctly.
  - For the true "positional arg + TTY stdin" case (no piped stdin at all), the
    test must spawn with `Stdio::null()` which ensures `is_terminal() == false`
    AND zero bytes. This passes under the new precedence rule because the
    positional arg is selected first regardless.

**COORDINATOR NOTE — NO NEW TOOLING REQUIRED.** The existing
`std::process::Command` + `Stdio` subprocess pattern covers all input-handling
acceptance criteria. The PTY-based `expectrl` framework is NOT needed for
Feature 1 (these are batch/CLI tests, not REPL tests). No new crates or test
infrastructure need to be built.

---

## Feature 1: Deterministic Input-Source Selection

### 1. Specification Analysis

**AC mapping (from sprint-71-planning.md, Feature 1):**

| AC-ID | Acceptance Criterion |
|-------|----------------------|
| AC1-1 | Positional SQL succeeds with TTY stdin (no pipe) |
| AC1-2 | Positional SQL succeeds with stdin attached to `/dev/null` |
| AC1-3 | Positional SQL succeeds with an empty inherited pipe |
| AC1-4 | Positional SQL succeeds when an immediate producer is also attached; stdin is ignored |
| AC1-5 | Positional SQL succeeds when a delayed producer is also attached; behavior identical to immediate case |
| AC1-6 | `--file` behavior is independent of stdin state |
| AC1-7 | Stdin-only SQL works with an immediate producer |
| AC1-8 | Stdin-only SQL works with a delayed producer |
| AC1-9 | Empty stdin returns `Empty query received from stdin`, not `No query provided` |
| AC1-10 | Unix and Windows follow the same source-precedence contract |
| AC1-11 | No readiness probe is used to infer whether stdin is the chosen source |

**Behavior change:** The old `validate_input_sources` / "Multiple input sources"
error is removed. Tests covering that error message are deleted or replaced.

**Feature Characteristics:**
- **User Interaction Type:** CLI Batch (spawned subprocess, stdin/stdout redirection)
- **Observable Behavior:** Exit code, stdout content, stderr error message
- **External Dependencies:** File system (for `--file` tests); none for in-process unit tests
- **Validation Challenges:**
  - True TTY stdin is not reproducible in the test runner. Covered via subprocess
    with `Stdio::null()` which exercises the "not-TTY, not-pipe" path under the
    new implementation (arg takes precedence regardless).
  - Delayed producer requires thread timing coordination — covered by
    subprocess + writer thread pattern.

### 2. Test Type Decision

- **Unit tests (REQUIRED):** New `determine_input_source` logic is pure (takes
  `QueryArgs`). Mocking stdin state is NOT needed for the arg/file path — those
  branches do not consult stdin. The "stdin is non-TTY, return Stdin" and
  "no query — no TTY — return error" paths require process-level tests.
- **Integration tests / process-level (REQUIRED):** The stdin fd state
  (`is_terminal`, bytes available, delayed producer) can only be controlled at
  the process level via subprocess stdio redirection. These tests exercise the
  full binary with controlled fd configuration.
- **Interactive / PTY (NOT NEEDED):** Feature 1 is batch-mode only. No REPL.

### 3. Numbered Tests — Feature 1

#### Unit tests (in `src/commands/query.rs` `mod tests`)

**TC110-U01** — `determine_input_source` returns `Argument` when query arg is present (no stdin probe)
- Coverage: AC1-1, AC1-11
- Verify: construct `QueryArgs { query: Some("SELECT 1"), file: None, ... }`, call `determine_input_source` → `Ok(InputSource::Argument("SELECT 1"))`
- Note: the unit test cannot control stdin fd; tests the arg-first branch in isolation

**TC110-U02** — `determine_input_source` returns `File` when `--file` is set and no query arg
- Coverage: AC1-6
- Verify: `QueryArgs { query: None, file: Some(path), ... }` → `Ok(InputSource::File(path))`

**TC110-U03** — `read_sql_stdin` returns `EMPTY_STDIN_ERROR` on whitespace-only input
- Coverage: AC1-9 (unit-level; process-level is TC110-I07)
- Verify: call `read_sql_stdin` via a helper that feeds `"\n   \n"` → `Err` containing `"Empty query received from stdin"`

**TC110-U04** — `determine_input_source` does NOT call `stdin_has_data()` when arg is present
- Coverage: AC1-11 (regression guard: the function must not inspect stdin readiness when arg is set)
- Approach: after the refactor, `stdin_has_data()` must not exist or must not be called from the arg path. Assert via code structure (presence of `stdin_has_data` function removed) or by wrapping with a mock. If `stdin_has_data` still exists for other reasons, verify it is not on the call path from `determine_input_source` when `args.query.is_some()`.

**TC110-U05** — Old `validate_input_sources` function removed or no longer exported
- Coverage: AC1-11, behavior change
- Verify: after implementation, `validate_input_sources` does not exist or is no longer called from `main.rs` (checked via compile + grep; if it still compiles that means no call site exists)

#### Integration / process-level tests (in `tests/integration_tests.rs`)

All process-level tests use `env!("CARGO_BIN_EXE_tq")`. They are DB-free (the
binary will fail at connection — but we only check that it does NOT fail at the
input-source determination step; or we pass `--dry-run` if implemented, or we
inspect the error message to distinguish "input error" vs "connection error").

**Strategy note on DB dependency for process tests:** Feature 1 tests only need
to verify that `tq` reached the connection phase (i.e., selected the right input
source and attempted to connect), OR that it returned the right input-phase
error. The binary will fail on connection if no DB is configured. We distinguish:
- `exit 1` with stderr containing `"No query provided"` / `"Empty query"` = input-phase error
- `exit 1` with stderr containing `"connection"` / `"login"` / `"Failed to connect"` = connection-phase error (means input selection succeeded)

This means AC1-1 through AC1-8 can be validated WITHOUT a live DB by checking
which error phase was reached.

**TC110-I01** — Positional arg succeeds past input selection when stdin is `/dev/null`
- Coverage: AC1-2
- Spawn: `tq query "SELECT 1"` with `Stdio::null()`
- Assert: stderr does NOT contain `"No query provided"` and does NOT contain `"Empty query"`
  (any error is connection-phase, not input-phase)

**TC110-I02** — Positional arg succeeds past input selection when stdin is empty pipe (closed immediately)
- Coverage: AC1-3
- Spawn: `tq query "SELECT 1"` with `Stdio::piped()`, drop stdin immediately (no bytes written)
- Assert: same as I01 — no input-phase error

**TC110-I03** — Positional arg ignores immediate stdin producer (stdin is ignored)
- Coverage: AC1-4
- Spawn: `tq query "SELECT 1"` with `Stdio::piped()`, write `"SELECT 2\n"` immediately, close pipe
- Assert: no input-phase error. If DB available (live): stdout reflects `SELECT 1` result, not `SELECT 2`.

**TC110-I04** — Positional arg ignores delayed stdin producer
- Coverage: AC1-5
- Spawn: `tq query "SELECT 1"` with `Stdio::piped()`; writer thread sleeps 300ms, writes `"SELECT 2\n"`, closes
- Assert: no input-phase error and process does not hang

**TC110-I05** — `--file` with stdin piped is independent of stdin
- Coverage: AC1-6
- Setup: create temp file with `"SELECT 1"` content; spawn `tq query --file <path>` with `Stdio::piped()`, write `"SELECT 2\n"` to stdin
- Assert: no input-phase error (selected file, not stdin)

**TC110-I06** — Stdin-only with immediate producer
- Coverage: AC1-7
- Spawn: `tq query` (no positional arg) with `Stdio::piped()`, write `"SELECT 1\n"` immediately, close
- Assert: no input-phase error (process reached connection phase)

**TC110-I07** — Stdin-only with delayed producer
- Coverage: AC1-8
- Spawn: `tq query` with `Stdio::piped()`; writer thread sleeps 300ms, writes `"SELECT 1\n"`, closes
- Assert: no input-phase error and process does not hang

**TC110-I08** — Empty stdin returns the correct error message
- Coverage: AC1-9
- Spawn: `tq query` with `Stdio::piped()`, close pipe immediately (zero bytes)
- Assert: `exit != 0`, stderr contains `"Empty query received from stdin"`, does NOT contain `"No query provided"`

**TC110-I09** — No query provided returns the correct error
- Coverage: AC1-1 (negative: no arg, no file, stdin is `/dev/null`)
- Spawn: `tq query` with `Stdio::null()`
- Assert: `exit != 0`, stderr contains `"No query provided"`

**TC110-I10** — Old "Multiple input sources" error is no longer produced (regression guard)
- Coverage: AC1-4, AC1-5, behavior change
- Spawn: `tq query "SELECT 1"` with `Stdio::piped()`, write `"SELECT 2\n"` (immediate producer)
- Assert: stderr does NOT contain `"Multiple input sources"`

**Feature 1 total: 5 unit + 10 integration = 15 tests**

---

## Feature 2: Structural Agent-Safe Classification

### 1. Specification Analysis

**AC mapping (from sprint-71-planning.md, Feature 2):**

| AC-ID | Acceptance Criterion |
|-------|----------------------|
| AC2-1 | A read-only CTE (`WITH x AS (SELECT 1) SELECT * FROM x`) is accepted |
| AC2-2 | Multiple and mixed leading comments accepted before read-only SQL |
| AC2-3 | `LOCKING ... SELECT` is accepted when the effective operation is read-only |
| AC2-4 | `LOCKING ROW FOR WRITE UPDATE/DELETE/INSERT/MERGE` is blocked |
| AC2-5 | `COLLECT STATISTICS` blocked by default, requires `--allow-maintenance` |
| AC2-6 | Unknown syntax fails closed with `AGENT_SAFE_UNCLASSIFIED` error |
| AC2-7 | Unknown syntax is NOT mislabeled as DDL |
| AC2-8 | Errors identify the effective operation and reason for rejection |
| AC2-9 | Regression tests cover Teradata abbreviations `SEL`, `INS`, `UPD`, `DEL` |

**New enum variants required:**
```rust
enum StatementSafety {
    ReadOnly,
    Maintenance,   // NEW — COLLECT STATISTICS
    Dml,
    Ddl,
    Unknown { token: Option<String>, reason: String },  // NEW
}
```

**New error code required:** `AGENT_SAFE_UNCLASSIFIED` (distinct from `AGENT_SAFE_BLOCKED`)

**New CLI flag:** `--allow-maintenance`

**Feature Characteristics:**
- **User Interaction Type:** Pure Logic (classifier function) + CLI Batch (for `--allow-maintenance` flag)
- **Observable Behavior:** Unit — `StatementSafety` variant returned; Integration — exit code + JSON error code
- **External Dependencies:** None (no DB required for classification logic)
- **Validation Challenges:**
  - The CTE path requires parenthesis-balanced scanning — must test nested parens
  - LOCKING modifier consumes arbitrary qualifier tokens — test several forms
  - Fail-closed `Unknown` must be verified to produce `AGENT_SAFE_UNCLASSIFIED`, not `AGENT_SAFE_BLOCKED` as DDL

### 2. Test Type Decision

- **Unit tests (REQUIRED):** Classification is pure logic. All AC2 tests are expressible as unit tests against the new `classify_statement` / `classify_safety` function.
- **Integration tests (REQUIRED for `--allow-maintenance` flag and error codes):** The `AGENT_SAFE_UNCLASSIFIED` error code and `--allow-maintenance` flag must be verified end-to-end via the binary (JSON error output format, exit code).
- **Live DB (OPTIONAL / `--ignored`):** Not strictly required for classification logic but useful to confirm that an accepted `LOCKING ... SELECT` actually runs.

### 3. Numbered Tests — Feature 2

#### Unit tests (in `src/commands/query.rs` `mod tests`)

**Group A: ReadOnly classification**

**TC111-U01** — `SELECT` classified `ReadOnly`
- Coverage: AC2-9 (SELECT baseline), existing behaviour preserved
- Input: `"SELECT * FROM t"`

**TC111-U02** — `SEL` abbreviation classified `ReadOnly`
- Coverage: AC2-9
- Input: `"SEL * FROM t"`

**TC111-U03** — `SHOW` classified `ReadOnly`
- Input: `"SHOW VIEW db.v"`

**TC111-U04** — `EXPLAIN` classified `ReadOnly`
- Input: `"EXPLAIN SELECT 1"`

**TC111-U05** — `HELP` classified `ReadOnly`
- Input: `"HELP TABLE t"`

**TC111-U06** — CTE with read-only final query classified `ReadOnly`
- Coverage: AC2-1
- Input: `"WITH x AS (SELECT 1) SELECT * FROM x"`

**TC111-U07** — Nested CTE with multiple CTEs classified `ReadOnly`
- Coverage: AC2-1 (depth)
- Input: `"WITH a AS (SELECT 1), b AS (SELECT * FROM a) SELECT * FROM b"`

**TC111-U08** — CTE with parentheses inside CTE body classified `ReadOnly`
- Coverage: AC2-1 (nested parens)
- Input: `"WITH x AS (SELECT COALESCE(a, b) FROM t) SELECT * FROM x"`

**TC111-U09** — Single line comment before SELECT classified `ReadOnly`
- Coverage: AC2-2
- Input: `"-- comment\nSELECT 1"`

**TC111-U10** — Block comment before SELECT classified `ReadOnly`
- Coverage: AC2-2
- Input: `"/* block */ SELECT 1"`

**TC111-U11** — Multiple mixed comments before SELECT classified `ReadOnly`
- Coverage: AC2-2
- Input: `"/* a */ /* b */ SELECT 1"`

**TC111-U12** — Interleaved line and block comments before SELECT classified `ReadOnly`
- Coverage: AC2-2
- Input: `"-- line\n/* block */\n-- another\nSELECT 1"`

**TC111-U13** — `LOCKING TABLE x FOR ACCESS SELECT *` classified `ReadOnly`
- Coverage: AC2-3
- Input: `"LOCKING TABLE t FOR ACCESS SELECT * FROM t"`

**TC111-U14** — `LOCKING ROW FOR ACCESS SELECT` classified `ReadOnly`
- Coverage: AC2-3
- Input: `"LOCKING ROW OF t FOR ACCESS SELECT * FROM t"`

**TC111-U15** — `LOCK TABLE ... SELECT` classified `ReadOnly` (LOCK synonym)
- Coverage: AC2-3
- Input: `"LOCK TABLE t FOR ACCESS SELECT 1"`

**Group B: Maintenance classification**

**TC111-U16** — `COLLECT STATISTICS` classified `Maintenance`
- Coverage: AC2-5
- Input: `"COLLECT STATISTICS ON t COLUMN (x)"`

**TC111-U17** — `COLLECT STATS` classified `Maintenance` (Teradata abbreviation)
- Coverage: AC2-5
- Input: `"COLLECT STATS ON t"`

**Group C: DML classification**

**TC111-U18** — `INSERT` classified `Dml`
- Coverage: AC2-9
- Input: `"INSERT INTO t VALUES (1)"`

**TC111-U19** — `INS` abbreviation classified `Dml`
- Coverage: AC2-9
- Input: `"INS INTO t VALUES (1)"`

**TC111-U20** — `UPDATE` classified `Dml`
- Coverage: AC2-9
- Input: `"UPDATE t SET x=1"`

**TC111-U21** — `UPD` abbreviation classified `Dml`
- Coverage: AC2-9
- Input: `"UPD t SET x=1"`

**TC111-U22** — `DELETE` classified `Dml`
- Coverage: AC2-9
- Input: `"DELETE FROM t"`

**TC111-U23** — `DEL` abbreviation classified `Dml`
- Coverage: AC2-9
- Input: `"DEL FROM t"`

**TC111-U24** — `MERGE` classified `Dml`
- Input: `"MERGE INTO t USING s ON ..."`

**TC111-U25** — `UPSERT` classified `Dml`
- Input: `"UPSERT INTO t VALUES (1)"`

**TC111-U26** — `LOCKING ROW FOR WRITE UPDATE` classified `Dml`
- Coverage: AC2-4
- Input: `"LOCKING ROW OF t FOR WRITE UPDATE t SET x=1"`

**TC111-U27** — `LOCKING ROW FOR WRITE DELETE` classified `Dml`
- Coverage: AC2-4
- Input: `"LOCKING ROW OF t FOR WRITE DELETE FROM t"`

**TC111-U28** — `LOCKING ROW FOR WRITE INSERT` classified `Dml`
- Coverage: AC2-4
- Input: `"LOCKING ROW OF t FOR WRITE INSERT INTO t VALUES (1)"`

**TC111-U29** — `LOCKING ROW FOR WRITE MERGE` classified `Dml`
- Coverage: AC2-4
- Input: `"LOCKING ROW OF t FOR EXCLUSIVE MERGE INTO t USING s ON t.id = s.id"`

**Group D: DDL classification**

**TC111-U30** — `CREATE` classified `Ddl`
- Input: `"CREATE TABLE t (id INT)"`

**TC111-U31** — `DROP` classified `Ddl`
- Input: `"DROP TABLE t"`

**TC111-U32** — `ALTER` classified `Ddl`
- Input: `"ALTER TABLE t ADD COLUMN x INT"`

**TC111-U33** — `RENAME` classified `Ddl`
- Input: `"RENAME TABLE t TO t2"`

**TC111-U34** — `REPLACE` classified `Ddl`
- Input: `"REPLACE VIEW v AS SELECT 1"`

**TC111-U35** — `GRANT` classified `Ddl`
- Input: `"GRANT SELECT ON t TO u"`

**TC111-U36** — `REVOKE` classified `Ddl`
- Input: `"REVOKE SELECT ON t FROM u"`

**Group E: Unknown (fail-closed)**

**TC111-U37** — Completely unknown keyword returns `Unknown` variant, NOT `Ddl`
- Coverage: AC2-6, AC2-7
- Input: `"FROBNICATE t"` (invented keyword)
- Assert: `classify_statement` returns `StatementSafety::Unknown { .. }`

**TC111-U38** — Empty statement (whitespace only) returns `Unknown`, NOT `Ddl`
- Coverage: AC2-6, AC2-7
- Input: `""`

**TC111-U39** — Statement starting with a number returns `Unknown`
- Coverage: AC2-6, AC2-7
- Input: `"42 THINGS"`

**TC111-U40** — `Unknown` variant contains the offending token
- Coverage: AC2-8 (partial — error identification)
- Input: `"FROBNICATE t"` → `Unknown { token: Some("FROBNICATE"), .. }`

**Group F: `validate_agent_safe` with new variants**

**TC111-U41** — Agent-safe mode blocks `Maintenance` by default
- Coverage: AC2-5
- Arrange: `QueryArgs { agent_safe: true, allow_maintenance: false, allow_dml: false, ... }`
- Input: `"COLLECT STATISTICS ON t"`
- Assert: `validate_agent_safe` returns `Err`

**TC111-U42** — Agent-safe mode permits `Maintenance` with `--allow-maintenance`
- Coverage: AC2-5
- Arrange: `QueryArgs { agent_safe: true, allow_maintenance: true, ... }`
- Input: `"COLLECT STATISTICS ON t"`
- Assert: `validate_agent_safe` returns `Ok(())`

**TC111-U43** — Agent-safe mode blocks `Unknown` with `AGENT_SAFE_UNCLASSIFIED` error code
- Coverage: AC2-6, AC2-7, AC2-8
- Arrange: `QueryArgs { agent_safe: true, ... }`
- Input: `"FROBNICATE t"`
- Assert: `Err(TqError::AgentSafeUnclassified { .. })` (new variant) OR error code is `"AGENT_SAFE_UNCLASSIFIED"`, NOT `"AGENT_SAFE_BLOCKED"` mislabeled as DDL

**TC111-U44** — Error message for blocked `Maintenance` identifies operation
- Coverage: AC2-8
- Assert: the error string from `validate_agent_safe` on `"COLLECT STATISTICS"` contains `"COLLECT"` and a reason string (e.g. `"maintenance"` or `"--allow-maintenance"`)

**TC111-U45** — Error message for blocked `Unknown` identifies the token
- Coverage: AC2-8
- Assert: the error string from `validate_agent_safe` on `"FROBNICATE t"` contains `"FROBNICATE"` or `"unclassified"` or similar identifying text

**Feature 2 Unit total: 45 tests**

#### Integration / process-level tests (in `tests/integration_tests.rs`)

**TC111-I01** — Binary exits with `AGENT_SAFE_UNCLASSIFIED` error code for unknown syntax
- Coverage: AC2-6, AC2-7
- Spawn: `tq query --agent-safe "FROBNICATE t" --format json`
- Assert: `exit != 0`, stdout (or stderr) JSON contains `"code": "AGENT_SAFE_UNCLASSIFIED"`, NOT `"AGENT_SAFE_BLOCKED"` with `statement_type: "FROBNICATE"` mislabeled as DDL

**TC111-I02** — Binary exits with error for `COLLECT STATISTICS` without `--allow-maintenance`
- Coverage: AC2-5
- Spawn: `tq query --agent-safe "COLLECT STATISTICS ON t" --format json`
- Assert: `exit != 0`, error body contains `"COLLECT"` or maintenance-related reason

**TC111-I03** — `--allow-maintenance` flag is accepted by the binary (parse test)
- Coverage: AC2-5 (flag plumbing)
- Spawn: `tq query --agent-safe --allow-maintenance "COLLECT STATISTICS ON t"`
- Assert: error is connection-phase (not argument-parse error), confirming the flag is wired

**TC111-I04** — JSON error for `AGENT_SAFE_UNCLASSIFIED` contains retryability field
- Coverage: AC2-6, AC2-8
- Spawn: `tq query --agent-safe "FROBNICATE t" --format json`
- Assert: JSON error has `"retryable": false` (or equivalent field per error schema)

**Feature 2 Integration total: 4 tests**

**Feature 2 total: 45 unit + 4 integration = 49 tests**

---

## Feature 3: Query Timeout + `--max-rows` Documentation

### 1. Specification Analysis

**AC mapping (from sprint-71-planning.md, Feature 3):**

| AC-ID | Acceptance Criterion |
|-------|----------------------|
| AC3-1 | Connection timeout and query timeout are separate, documented controls |
| AC3-2 | Agent-safe mode has a finite query timeout by default, or requires one explicitly |
| AC3-3 | Query timeout produces a structured JSON error (`QUERY_TIMEOUT`) |
| AC3-4 | `--max-rows` documentation states it is a client fetch/output cap |
| AC3-5 | Timeout attempts to cancel/abort (or limitation documented if driver cannot) |

**Feature Characteristics:**
- **User Interaction Type:** CLI Batch (flag parsing + error output)
- **Observable Behavior:** Exit code, JSON error code, `--help` text, structured error fields
- **External Dependencies:** Live DB for AC3-5 (cancel/abort behavior); unit tests cover AC3-1, AC3-2, AC3-3, AC3-4
- **Validation Challenges:**
  - AC3-5 genuinely requires a live DB with a slow/hanging query to trigger a timeout.
  - AC3-3 can be partially tested at unit level (error type construction) + binary level (JSON format).
  - AC3-4 is documentation — validated by inspecting `--help` output.

### 2. Test Type Decision

- **Unit tests (REQUIRED):** Error type construction, timeout flag parsing, agent-safe default timeout logic.
- **Integration tests — DB-free (REQUIRED):** `--help` output for `--query-timeout` and `--max-rows`, flag parsing acceptance.
- **Live DB tests (`--ignored`) (REQUIRED for AC3-3 and AC3-5):** Actually triggering the timeout and verifying the structured `QUERY_TIMEOUT` JSON error and cancellation attempt.

### 3. Numbered Tests — Feature 3

#### Unit tests

**TC112-U01** — `TqError::QueryTimeout` variant exists and produces error code `"QUERY_TIMEOUT"`
- Coverage: AC3-3
- Assert: `TqError::QueryTimeout { .. }.error_code() == "QUERY_TIMEOUT"`

**TC112-U02** — `QueryTimeout` error serializes to JSON with `"retryable": true` (or documented value)
- Coverage: AC3-3
- Assert: JSON representation of `QueryTimeout` error has the correct retryability field

**TC112-U03** — `QueryArgs` has a `query_timeout` field (flag plumbing)
- Coverage: AC3-1
- Assert: `QueryArgs { query_timeout: Some(Duration::from_secs(30)), ... }` compiles

**TC112-U04** — `QueryArgs` `--timeout` and `--query-timeout` are distinct fields
- Coverage: AC3-1
- Assert: both `timeout` and `query_timeout` fields exist and are independently settable

**TC112-U05** — Agent-safe mode applies a finite default query timeout when none given
- Coverage: AC3-2
- Assert: function that resolves effective timeout returns `Some(duration)` when `agent_safe = true` and `query_timeout = None`

**TC112-U06** — Non-agent-safe mode with no `--query-timeout` applies no default timeout
- Coverage: AC3-2 (negative)
- Assert: effective timeout is `None` when `agent_safe = false` and `query_timeout = None`

#### Integration tests — DB-free

**TC112-I01** — `--query-timeout` flag is accepted by the binary (parse test)
- Coverage: AC3-1
- Spawn: `tq query --query-timeout 30s "SELECT 1"`
- Assert: error is connection-phase, NOT argument-parse error

**TC112-I02** — `--help` output for `query` subcommand mentions `--query-timeout` and `--timeout` as separate controls
- Coverage: AC3-1
- Spawn: `tq query --help`
- Assert: stdout contains `--query-timeout` and `--timeout` with distinct descriptions

**TC112-I03** — `--help` output states `--max-rows` is a client fetch/output cap
- Coverage: AC3-4
- Spawn: `tq query --help`
- Assert: stdout contains text indicating `--max-rows` is a client-side cap (e.g., `"client"` or `"fetch"` or `"output cap"` near `--max-rows`)

**TC112-I04** — `QUERY_TIMEOUT` error is machine-readable JSON
- Coverage: AC3-3
- This is a schema test: construct a `TqError::QueryTimeout` and serialize it; assert the JSON has `"code": "QUERY_TIMEOUT"`. (Unit-level; repeated here as integration confirmation via the error serialization path used in `main.rs`.)

#### Live DB tests (`#[ignore]`)

**TC112-L01** — Query timeout triggers `QUERY_TIMEOUT` JSON error on a slow query
- Coverage: AC3-3, AC3-5
- Setup: live DB, query that takes longer than the given timeout (e.g., `SELECT ... WITH RETRY` or a sleep UDF if available, or a very large table scan with 1s timeout)
- Spawn: `tq query --query-timeout 1s "SELECT ... heavy scan" --format json`
- Assert: `exit != 0`, stdout JSON contains `"code": "QUERY_TIMEOUT"`

**TC112-L02** — Agent-safe default timeout fires on a slow query (no `--query-timeout` flag)
- Coverage: AC3-2, AC3-3
- Spawn: `tq query --agent-safe "SELECT ... heavy" --format json`
- Assert: `exit != 0`, JSON error contains `QUERY_TIMEOUT` (proves the default timeout is wired)

**Feature 3 total: 6 unit + 4 integration + 2 live = 12 tests**

---

## Feature 4: Diagnostics & Least-Privilege Documentation

### 1. Specification Analysis

**AC mapping (from sprint-71-planning.md, Feature 4, P2):**

| AC-ID | Acceptance Criterion |
|-------|----------------------|
| AC4-1 | `docs/specifications/security.md` documents `--agent-safe` as defense-in-depth and recommends DB-side least privilege |
| AC4-2 | `--agent-safe` help text references the least-privilege guidance |

**Feature Characteristics:**
- **User Interaction Type:** Documentation + help text
- **Observable Behavior:** File content (`docs/specifications/security.md`), `--help` output
- **External Dependencies:** None
- **Validation Challenges:** Documentation tests are simple content assertions.

### 2. Test Type Decision

- **Integration tests (REQUIRED):** `--help` output, file content.
- **Unit tests (NOT NEEDED):** No logic to unit test.

### 3. Numbered Tests — Feature 4

**TC113-I01** — `tq query --help` mentions defense-in-depth or least-privilege near `--agent-safe`
- Coverage: AC4-2
- Spawn: `tq query --help`
- Assert: stdout contains `"least-privilege"` or `"defense"` or similar near `--agent-safe` description

**TC113-I02** — `docs/specifications/security.md` contains defense-in-depth framing
- Coverage: AC4-1
- Read file: assert it contains `"defense-in-depth"` or `"defence-in-depth"`

**TC113-I03** — `docs/specifications/security.md` recommends DB-side least privilege
- Coverage: AC4-1
- Read file: assert it contains `"least privilege"` or `"dedicated"` user and `"GRANT"` or `"grants"` near that section

**Feature 4 total: 3 integration tests**

---

## Obsolete Tests to Remove or Update

The following existing tests in `src/commands/query.rs::tests` cover behavior that is intentionally removed in Sprint 71:

1. **`test_multiple_input_sources_error_message_content`** — tests the "Multiple input sources" error string which is explicitly removed. This test MUST be deleted or replaced with TC110-I10 (regression guard that the error no longer appears).

2. **`test_stdin_has_data_does_not_panic`** — tests `stdin_has_data()` which is deleted by Feature 1. This test MUST be removed.

3. **`test_classify_select_is_readonly`** (existing) — currently includes `COLLECT STATISTICS ON t` as `ReadOnly`. Under Feature 2, `COLLECT STATISTICS` becomes `Maintenance`. This test MUST be updated to remove that assertion (covered by TC111-U16/U17 instead).

4. **`test_classify_with_comments`** — existing test; verify it still holds after the new classifier is in place. Keep but mark as regression guard.

---

## Acceptance Criterion → Test Coverage Matrix

### Feature 1

| AC-ID | Criterion Summary | Test IDs | Tier |
|-------|-------------------|----------|------|
| AC1-1 | Positional SQL + TTY stdin | TC110-U01, TC110-I09 (negative) | Unit + Integration |
| AC1-2 | Positional SQL + `/dev/null` | TC110-I01 | Integration |
| AC1-3 | Positional SQL + empty pipe | TC110-I02 | Integration |
| AC1-4 | Positional SQL + immediate producer (stdin ignored) | TC110-I03, TC110-I10 | Integration |
| AC1-5 | Positional SQL + delayed producer | TC110-I04 | Integration |
| AC1-6 | `--file` independent of stdin | TC110-U02, TC110-I05 | Unit + Integration |
| AC1-7 | Stdin-only + immediate producer | TC110-I06 | Integration |
| AC1-8 | Stdin-only + delayed producer | TC110-I07 | Integration |
| AC1-9 | Empty stdin → distinct error | TC110-U03, TC110-I08 | Unit + Integration |
| AC1-10 | Same contract on Unix/Windows | TC110-U01..U05 (logic purity) | Unit (portability note below) |
| AC1-11 | No readiness probe used | TC110-U04, TC110-U05 | Unit |

**Note on AC1-10:** True Windows cross-platform validation is not executable in this environment (Darwin CI). The test strategy covers this via: (a) unit tests verify the logic is purely `is_terminal()` + blocking read, with no `libc` probe involved, and (b) the code deletion of `stdin_has_data()` is the structural proof. Risk: LOW (documented limitation).

### Feature 2

| AC-ID | Criterion Summary | Test IDs | Tier |
|-------|-------------------|----------|------|
| AC2-1 | Read-only CTE accepted | TC111-U06, U07, U08 | Unit |
| AC2-2 | Multiple/mixed leading comments accepted | TC111-U09, U10, U11, U12 | Unit |
| AC2-3 | `LOCKING ... SELECT` accepted | TC111-U13, U14, U15 | Unit |
| AC2-4 | `LOCKING ... DML` blocked | TC111-U26, U27, U28, U29 | Unit |
| AC2-5 | `COLLECT STATISTICS` blocked by default; `--allow-maintenance` permits | TC111-U16, U17, U41, U42, TC111-I02, I03 | Unit + Integration |
| AC2-6 | Unknown syntax → `AGENT_SAFE_UNCLASSIFIED` | TC111-U37..U40, U43, TC111-I01, I04 | Unit + Integration |
| AC2-7 | Unknown NOT mislabeled DDL | TC111-U37, U38, U39, U43, TC111-I01 | Unit + Integration |
| AC2-8 | Errors identify operation + reason | TC111-U44, U45, TC111-I04 | Unit + Integration |
| AC2-9 | Teradata abbreviations SEL/INS/UPD/DEL | TC111-U02, U19, U21, U23 | Unit |

**Issue example-table coverage (from #45 issue):** The strategy's tests cover all rows of the classification table referenced in the issue. Specifically: CTEs (U06-U08), comments (U09-U12), LOCKING read (U13-U15), LOCKING write (U26-U29), COLLECT STATISTICS (U16-U17), abbreviations (U02, U19, U21, U23), unknown (U37-U40). The full mapping will be verified when authoring TC111.md.

### Feature 3

| AC-ID | Criterion Summary | Test IDs | Tier |
|-------|-------------------|----------|------|
| AC3-1 | Connection and query timeouts are separate | TC112-U03, U04, TC112-I01, I02 | Unit + Integration |
| AC3-2 | Agent-safe has finite default query timeout | TC112-U05, U06, TC112-L02 | Unit + Live |
| AC3-3 | `QUERY_TIMEOUT` structured JSON error | TC112-U01, U02, TC112-I04, TC112-L01 | Unit + Integration + Live |
| AC3-4 | `--max-rows` docs say client cap | TC112-I03 | Integration |
| AC3-5 | Cancel/abort attempted (or documented limitation) | TC112-L01, TC112-L02 | Live |

### Feature 4

| AC-ID | Criterion Summary | Test IDs | Tier |
|-------|-------------------|----------|------|
| AC4-1 | security.md: defense-in-depth + DB least privilege | TC113-I02, TC113-I03 | Integration |
| AC4-2 | `--help` references least-privilege guidance | TC113-I01 | Integration |

---

## Test Tier Summary

### Tier Classification

| Tier | Tag | Run command | DB required |
|------|-----|-------------|-------------|
| Unit | `#[test]` in `src/` | `cargo test --lib` | No |
| Integration (DB-free) | `#[test]` in `tests/integration_tests.rs` | `cargo test --test integration_tests` | No |
| Live (DB) | `#[test] #[ignore]` in `tests/integration_tests.rs` | `cargo test --test integration_tests -- --ignored` | Yes |
| Interactive (PTY) | `#[test] #[ignore]` in `tests/interactive_tests.rs` | `cargo test --test interactive_tests -- --ignored` | Yes |

**Note:** No new interactive (PTY) tests are required for Sprint 71. All new features are batch-mode CLI, not REPL.

---

## Total Test Count

| Feature | Unit | Integration (DB-free) | Live (DB) | Total |
|---------|------|----------------------|-----------|-------|
| Feature 1 — Input selection | 5 | 10 | 0 | 15 |
| Feature 2 — Agent-safe classification | 45 | 4 | 0 | 49 |
| Feature 3 — Query timeout | 6 | 4 | 2 | 12 |
| Feature 4 — Documentation | 0 | 3 | 0 | 3 |
| **Total** | **56** | **21** | **2** | **79** |

**Grand total planned: 79 tests**

---

## Gap Analysis

### Gap 1: AC1-10 — Windows cross-platform validation

- **Reason for omission:** CI runs on macOS/Linux; no Windows runner available.
- **What won't be validated:** Actual behavior on Windows with named pipe or console handles.
- **Risk:** LOW — the fix removes platform-diverging code (`stdin_has_data`). The new path uses only `std::io::stdin().is_terminal()` which is cross-platform. The behavior change is a code deletion, not a new branch.
- **Mitigation:** Document in sprint review. The struct-level proof (no `libc` in the new path, no `#[cfg(unix)]` branch) is the primary evidence.

### Gap 2: AC3-5 — Live cancellation behavior

- **Reason for partial coverage:** Driver cancellation feasibility is TBD (architect probes in Phase 2). If driver does not support it, the fallback is a client-side deadline with `QUERY_TIMEOUT` error and session close.
- **What won't be validated in DB-free tests:** Whether the active request is actually cancelled vs. session closed.
- **Risk:** MEDIUM — the structured error and default timeout ship regardless (per planning doc scope guard). The AC itself has a documented fallback: "cancel/abort OR limitation documented honestly."
- **Mitigation:** TC112-L01 and TC112-L02 exercise the live path. If the DB trial is unavailable, these are BLOCKED (not APPROVED). The cancellation-vs-close distinction is validated by log/diagnostic output in the live test.

### Gap 3: Delayed-producer test timing on slow CI

- **Reason:** TC110-I04 and TC110-I07 use a 300ms thread sleep. On very slow CI, this could be flaky.
- **Risk:** LOW — 300ms is conservative. The test could fail only if the host is so loaded that the child exits before the writer thread even starts. This is not a logic test; the exit condition is observed.
- **Mitigation:** Use 500ms sleep in the test. The test process exits quickly (before the sleep fires) under the new input-precedence implementation; if it times out waiting for the process, the test fails explicitly.

---

## Implementation Plan

### File locations for new tests

| Test type | File |
|-----------|------|
| Unit tests | `src/commands/query.rs` — inline `mod tests` |
| Unit tests (error) | `src/error.rs` — inline `mod tests` |
| Unit tests (CLI) | `src/cli.rs` — inline `mod tests` |
| Integration tests (DB-free + Live) | `tests/integration_tests.rs` |

### Test case documentation files

Each numbered test group will be documented in:
- `tests/cases/TC110.md` — Feature 1: Input selection
- `tests/cases/TC111.md` — Feature 2: Agent-safe classification
- `tests/cases/TC112.md` — Feature 3: Query timeout
- `tests/cases/TC113.md` — Feature 4: Documentation

### Tests to delete

- `test_multiple_input_sources_error_message_content` (line 1166 in query.rs) — behavior removed
- `test_stdin_has_data_does_not_panic` (line 1190 in query.rs) — function removed
- The `COLLECT STATISTICS ON t` → `ReadOnly` assertion inside `test_classify_select_is_readonly` (line 1062) — now `Maintenance`

---

## Coverage Sufficiency Assessment

If all 79 tests are implemented and passing:

- **Feature 1 (input selection):** Full coverage. Process-level tests directly exercise the fd-state paths that motivated the bug. Unit tests confirm the logic purity. The delayed-producer path is exercised deterministically.
- **Feature 2 (classification):** Full coverage. Every AC has multiple unit tests. The fail-closed `Unknown` variant is tested at both unit and integration levels.
- **Feature 3 (timeout):** Coverage conditional on live DB for AC3-3 and AC3-5. DB-free tests cover flag plumbing, error code construction, and help text. If DB unavailable, verdict is BLOCKED (not APPROVED) for AC3-2, AC3-3, AC3-5.
- **Feature 4 (docs):** Full coverage. Simple content assertions.

**Combined verdict criteria:**
- APPROVED: All 77 DB-free tests pass + 2 live tests pass (or are explicitly BLOCKED with documented mitigation for AC3-5 fallback)
- REJECTED: Any DB-free test fails
- BLOCKED: DB unavailable and any live test is required for APPROVED verdict

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest (3 documented gaps, all LOW or MEDIUM with mitigations)
- [x] Specification coverage map includes all AC checkboxes
- [x] Every AC maps to at least one numbered test
- [x] Test implementation plan is detailed and actionable (file locations, deletion list)
- [x] Coverage sufficiency is assessed
- [x] Sprint 69 rule satisfied: every test has a unique number and the total count is stated

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-06-11
**Review Status:** DRAFT
**Total planned tests:** 79 (56 unit, 21 integration DB-free, 2 live)

**Reviewer:** tq-project-manager
**Review Status:** PENDING
