# Sprint 76 Test Strategy

**Created:** 2026-08-06
**Author:** quality-validator
**Sprint:** Sprint 76
**Features:**
1. Perm space analysis — `tq space`, `tq dbspace` (Issue #54)
2. Monitoring thresholds & colors — `[monitoring.thresholds]` / `[monitoring.colors]`, severity classification, `refresh_interval` (Issue #23)

---

## Instructions for quality-validator (self-check before implementation)

This strategy is derived from `docs/sprints/sprint-76-planning.md`, GitHub issues #54 and #23, and code research against the current implementation (`src/config.rs`, `src/cli.rs`, `src/commands/skew.rs`, `src/commands/resources.rs`, `src/commands/watch.rs`). No test code is written in this phase.

---

## Feature 1: Space Analysis (`tq space`, `tq dbspace`) — Issue #54

### 1. Specification Analysis

**Specification References:**
- `docs/sprints/sprint-76-planning.md` §Feature 1, Acceptance Criteria (bullets 1–7, 13–14)
- GitHub #54 (SQL source of truth for `DBC.DiskSpaceV` / `DBC.TableSizeV` queries and skew formula)

**Requirements:**
1. `tq space <db>` → one database header row + one row per object directly under it.
2. `tq space <db>.<obj>` → exactly one row for that object.
3. `tq dbspace <db>` → database-level perm/spool/temp only; errors clearly if given `db.obj` or a non-database name.
4. Object level: `CurrentPerm`, `PeakPerm`, perm skew %.
5. Database level: additionally `MaxPerm`, % of MaxPerm used, Spool (current/allocated/peak/skew), Temp (current/allocated/peak/skew).
6. Skew formula: `100 - (AVG(CurrentPerm) / NULLIFZERO(MAX(CurrentPerm)) * 100)`, NULL-safe (no divide-by-zero).
7. All four formats: table, json, csv, markdown.
8. Unknown database/object → standard not-found error, with spelling suggestion where existing helpers provide one.
9. SQL validated against the live Teradata database — no fabricated DBC objects/columns.

**Feature Characteristics:**

**User Interaction Type:** CLI Batch (scripted, non-interactive; also invocable from REPL as a metacommand-style query, consistent with `skew`/`resources`).

**Explanation:** `space`/`dbspace` are one-shot batch commands identical in shape to the existing `skew` command (`src/commands/skew.rs`): parse args → query DBC views → render one of four formats → exit. No PTY/terminal-control behavior is introduced.

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, table, markdown)
- [x] Database side effects — none (read-only), but requires live DB reads
- [ ] Visual output requiring PTY (not applicable — no color/severity concerns tested here; color is covered under Feature 2)
- [ ] File system side effects
- [ ] Network interactions beyond the DB connection itself

**External Dependencies:**
- [x] Database connection (live Teradata via `.env` `TQ_LOGON`) — required to validate real SQL against `DBC.DiskSpaceV` / `DBC.TableSizeV`
- [x] None beyond DB for the pure formatting/parsing/skew-math logic

**Validation Challenges:**
- The skew formula's `NULLIFZERO` divide-by-zero guard must be proven both in the Rust-side fallback (if the Rust code also guards, per the `calculate_skew` pattern in `skew.rs`) and against live NULL results returned by the DB itself.
- Distinguishing "argument is a table, not a database" from "argument does not exist at all" for `dbspace`'s error path requires either two different DBC lookups or one lookup with disambiguation logic — this is a genuine edge case the design must handle and tests must exercise both branches.
- The "spelling suggestion" fuzzy-match helper referenced in the acceptance criteria **does not exist in the codebase today** (confirmed: no `strsim`/levenshtein dependency, no fuzzy-match module). Existing not-found errors give only static guidance. This is new functionality for the architect to build in Sprint 76 — the test strategy assumes it will exist and specifies its expected behavior, but flags this as a design dependency, not an assumption already met by existing code.

**Critical Behaviors to Validate:**
1. Row shape differs correctly across the three invocation forms (`space <db>`, `space <db>.<obj>`, `dbspace <db>`).
2. Skew % is NULL-safe under `MAX(CurrentPerm) = 0` and under empty result sets.
3. `dbspace` rejects qualified names and non-database names with distinguishable, actionable errors.
4. All four `--format` values produce well-formed, parseable output.

### 2. Test Strategy Derivation

**Test Type 1: Unit Tests**
- **Validates:** Skew-percentage math (NULL-safety, boundary cases), row→struct parsing for both DBC views, qualified-name splitting (`db.obj` → (db, obj)), per-format serialization (JSON schema, CSV escaping via existing `escape_csv`, markdown escaping via existing `markdown_escape_pipe`), % of MaxPerm used calculation.
- **Approach:** Pure functions with synthetic `Value` rows, mirroring `SkewInfo::from_row` / `calculate_skew` test patterns already in `src/commands/skew.rs`.
- **Rationale:** These are pure, deterministic calculations — exactly what unit tests are for per `docs/testing/approach.md`. A live DB round-trip should never be the only thing standing between a divide-by-zero panic and production.
- **Gap if missing:** A NULL/zero MaxPerm or CurrentPerm value from a real (but unusual) database state could panic in production; unit tests catch this deterministically without needing to manufacture that DB state live.
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (live database)**
- **Validates:** End-to-end command execution against real `DBC.DiskSpaceV`/`DBC.TableSizeV`, the three invocation-shape behaviors, all four `--format` outputs, and the not-found/wrong-type error paths.
- **Approach:** `std::process::Command` spawning the built `tq` binary (per `tests/README.md` "Process Integration Tests" pattern), using `TQ_LOGON` from `.env`, marked `#[ignore]`.
- **Rationale:** Only a live DB run proves the SQL in #54 is valid against a real DBC catalog (no fabricated columns) and that the CLI's error handling for "wrong object type" is correct — this cannot be mocked without risking the exact Sprint-73/74 "verify against live DB" failure mode this project has hit before (see user's global feedback note on Teradata verification).
- **Gap if missing:** SQL could reference a column that doesn't exist in the target Teradata version, or the skew formula could silently return wrong numbers against real skewed data — undetectable without a live run.
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (expectrl)**
- **Necessity:** ❌ NOT NEEDED — `space`/`dbspace` are batch-shaped commands with no REPL-specific rendering, cursor, or completion behavior distinct from `skew`'s existing (non-interactive-only) treatment. If a follow-up sprint adds REPL metacommand wrapping (e.g., `/space`), interactive tests would be added then.

### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|-----------------|----------|
| Unit tests | ✅ REQUIRED | Skew math, parsing, formatting are pure logic | Divide-by-zero/NULL panics ship undetected | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Only live run proves SQL validity + error paths | Fabricated DBC columns/skew bugs ship undetected | MUST IMPLEMENT |
| Interactive tests | ❌ NOT NEEDED | No REPL-specific behavior introduced | N/A | SKIP |

### 4. Specification Coverage Map

| Requirement | Spec Ref | Test Type | Test Cases |
|---|---|---|---|
| `space <db>` → header + object rows | AC bullet 1 | Integration | TC109-I01 |
| `space <db>.<obj>` → single row | AC bullet 2 | Integration | TC109-I02 |
| `dbspace <db>` → db-level only | AC bullet 3 | Integration | TC109-I03 |
| `dbspace <db>.<obj>` → clear error | AC bullet 4 | Integration | TC109-I04 |
| Unknown db/object → standard not-found + suggestion | AC bullet 5 | Integration | TC109-I06, TC109-I07 |
| Skew formula NULL-safe | AC bullet 6 | Unit + Integration | TC109-U01–U03, TC109-I13 |
| All 4 formats supported | AC bullet 7 | Unit + Integration | TC109-U10–U12, TC109-I08–I11 |
| SQL validated live, no fabricated objects | AC bullet 13 | Integration | TC109-I14 |
| `dbspace` given a real table (not a db) | Issue #54 body | Integration | TC109-I05 |

**Coverage Gaps:** None identified for AC-mapped requirements. One design dependency flagged: the "spelling suggestion" helper is net-new (see §1 Validation Challenges) — if the architect descopes it under time pressure, TC109-I06/I07 degrade to asserting the standard not-found message only (suggestion assertion becomes conditional, documented in evidence, not silently dropped).

### 5. Gap Analysis

**Benchmark/performance tests** — omitted. No performance SLA stated for `space`/`dbspace` in the planning doc. Risk: LOW. Revisit if users report slowness on databases with very large object counts.

### 6. Test Implementation Plan

**Unit Tests**
- **Location:** `src/commands/space.rs` (new module, `#[cfg(test)] mod tests`), mirroring `src/commands/skew.rs`'s existing test module structure.
- **Count:** 13 (TC109-U01–U13)

**Integration Tests**
- **Location:** New file `tests/integration_space.rs` (parallels `tests/integration_fastload.rs`, `tests/integration_search.rs` naming convention), `#[ignore]`, using `TQ_LOGON`.
- **Count:** 14 (TC109-I01–I14)
- **Setup requirements:** A test database/user reachable via `.env` with at least one child object (table/view) to exercise the header+rows case; a known table name to pass to `dbspace` for the "wrong type" error case; a known nonexistent name for not-found cases.

### 7. Coverage Sufficiency Assessment

Unit tests prove the math and formatting never panic and are NULL-safe in isolation. Integration tests prove the SQL is real, the three invocation shapes behave correctly against an actual DBC catalog, and error messages are correct. Combined: sufficient to claim "works as specified." No REPL-specific gap exists because no REPL-specific behavior was added.

---

## Feature 2: Monitoring Thresholds & Colors — Issue #23

### 1. Specification Analysis

**Specification References:**
- `docs/sprints/sprint-76-planning.md` §Feature 2, Acceptance Criteria (bullets 8–12)
- GitHub #23 (user stories US-8.1–US-8.5)

**Requirements:**
1. `[monitoring.thresholds]` and `[monitoring.colors]` parse from config; every key optional; defaults apply per-key (not all-or-nothing).
2. Severity classification (Normal/Warning/Critical) applied to `resources`, `watch`, `skew`, and `space`.
3. Colors honor existing `auto`/`always`/`never` mode, `NO_COLOR`, and piped output — never emit ANSI when suppressed.
4. `refresh_interval` becomes the default for watch-mode `--interval` when the flag is absent; explicit `--interval` still wins.
5. Invalid threshold config (warning > critical, negative, > 100) → descriptive config error, not a panic.

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (threshold parsing, severity classification, precedence resolution) **+ CLI Batch** (color emission observed in actual command output) — a hybrid, so both test types are required, not either/or.

**Explanation:** The classification/precedence/validation logic is pure and mockable. But whether ANSI codes actually appear or don't appear in the real output stream of `resources`/`skew`/`space`/watch-mode commands is only provable by running the actual binary and inspecting bytes on stdout, because color emission today is wired through `main.rs`'s single `use_color: bool` computed from `ColorChoice::should_use_color()` (`src/cli.rs`) and `config::should_use_color()` (`src/config.rs:449`) — and Sprint 76 is precisely the sprint that must stop that function from ignoring `_config`.

**Observable Behavior:**
- [x] Visual output in terminal (ANSI color codes present/absent)
- [x] Structured data output unaffected by color (JSON/CSV must never carry ANSI even if colors are "always")
- [ ] File system side effects
- [x] State management — config precedence (CLI flag > config `refresh_interval` > hardcoded default)

**External Dependencies:**
- [x] File system access (reads `.tq.toml` / `~/.tq/config.toml`)
- [x] Database connection for the integration-level color/severity assertions against real `resources`/`skew`/`space` output
- [ ] None for the pure classification/precedence unit tests

**Validation Challenges:**
- **Testability blocker identified in current code:** `--interval` on `resources`/`locks`/`sessions` is declared with `default_value = "6"` baked directly into `clap` (`src/cli.rs` lines ~742, ~805, ~1284). Clap therefore *always* supplies `6` whether the user typed `--interval 6` or omitted the flag entirely — there is currently no way to distinguish "absent, use config" from "explicitly 6." **This is a design dependency, not a test-writing task**: the architect must change these fields to `Option<u64>` with no baked default (or equivalent), then resolve `cli.interval.unwrap_or(config.monitoring.thresholds.refresh_interval)` in command startup. The test strategy below assumes this resolution point will exist and is written against it; if the architect ships the old baked-default clap field unchanged, TC110-U21/U22 and TC110-I05/I06 will fail immediately and that failure is the correct signal, not a test bug.
- Distinguishing "warning == critical" (an explicit AC boundary) from a validation bug requires a config file, not just an in-memory struct, if the parse-and-validate step happens during `figment` merge (`src/config.rs`) rather than lazily.
- ANSI-presence assertions must scan raw bytes for `\x1b[` sequences, not rely on human eyeballing — no existing helper does this (see §6 tooling gap below).

**Critical Behaviors to Validate:**
1. Per-key partial-config defaults (not "all or nothing").
2. Inclusive boundary classification (`value == warning_threshold` → Warning, not Normal; `value == critical_threshold` → Critical).
3. Zero ANSI bytes under `never`/`NO_COLOR`/piped, non-zero under `always` for a Warning/Critical value.
4. `refresh_interval` precedence: config value used only when `--interval` absent; CLI always wins when present.
5. Validation errors for warning > critical, negative, > 100 — described, not panicking.

### 2. Test Strategy Derivation

**Test Type 1: Unit Tests**
- **Validates:** Default/partial-config resolution, severity classification boundaries, threshold validation errors, color-name→code mapping, ANSI presence/absence as a pure function of a `ColorChoice`+`NO_COLOR` combination (without needing a live process), `refresh_interval` precedence resolution logic.
- **Approach:** Construct `MonitoringSettings`/`ThresholdSettings` structs directly (in-memory) plus a handful of `figment`-backed tests that parse literal TOML strings, mirroring the existing pattern in `src/config.rs`'s `mod tests` (`test_default_config`, `test_load_with_valid_project_config_merges`).
- **Rationale:** Boundary math (inclusive `>=`/`<=`) and validation-error wording are pure and must be pinned exactly — these are exactly the kind of off-by-one bugs that live-DB testing would never surface (a live session's CPU% won't reliably land on the threshold boundary).
- **Gap if missing:** An inclusive/exclusive boundary bug (e.g., warning at exactly 70 misclassified as Normal) ships silently; a negative-threshold panic reaches production.
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (live database + subprocess)**
- **Validates:** That real `resources`/`skew`/`space`/watch-mode command output actually carries or omits ANSI bytes under real conditions (piped, `--color always`, `NO_COLOR=1` env), and that `refresh_interval` precedence holds when the real binary is watching in a loop against a live connection.
- **Approach:** `std::process::Command` subprocess spawn with `Stdio::piped()` (per `tests/README.md` fd-redirection pattern) reading raw stdout bytes; scripted `.tq.toml` project config files (per `tests/integration_project_config_edge_cases.rs`'s `create_user_config` helper) to control `refresh_interval` and thresholds per test.
- **Rationale:** `is_terminal()`/pipe detection is OS-level state that cannot be faked from within the same process (this exact class of test already exists in `tests/README.md` for stdin — the same reasoning applies to stdout color detection). Watch-mode timing can only be proven by timing a real subprocess.
- **Gap if missing:** A regression that leaks ANSI into piped JSON output (breaking every downstream consumer of `tq resources --format json | jq`) would ship undetected by unit tests alone.
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (expectrl)**
- **Necessity:** ⚠️ RECOMMENDED, not required this sprint — a live-terminal color rendering check (does yellow actually look like configured "yellow" in a real PTY) is a manual/visual concern per `docs/testing/approach.md`'s Type 3 "Terminal Output" classification ("Limited Automated Coverage... Mitigation: Integration tests for content + manual visual verification"). Automated PTY tests would only re-prove what the subprocess ANSI-byte tests already prove (presence/absence), not visual correctness of color choice. Documented as a gap, mitigated by one manual verification pass at sprint closure (not a numbered automated test).

### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|-----------------|----------|
| Unit tests | ✅ REQUIRED | Boundary math, validation, precedence are pure logic | Off-by-one severity bugs, panics on bad config ship silently | MUST IMPLEMENT |
| Integration tests (subprocess) | ✅ REQUIRED | ANSI presence is OS/stream-level, not fakeable in-process | ANSI leak into piped/JSON output undetected | MUST IMPLEMENT |
| Interactive tests (expectrl) | ⚠️ RECOMMENDED | Visual color-correctness in real PTY is a UX concern | Subjective "does yellow look right" unverified | DOCUMENT, not gate closure — 1 manual pass |
| Benchmark tests | ❌ NOT NEEDED | No performance requirement stated | N/A | SKIP |

### 4. Specification Coverage Map

| Requirement | Spec Ref | Test Type | Test Cases |
|---|---|---|---|
| Per-key optional defaults | AC bullet 8 | Unit | TC110-U01–U04 |
| Severity classification (Normal/Warning/Critical) | AC bullet 9 | Unit | TC110-U05–U09, TC110-U24 |
| Applied to `resources`, `watch`, `skew`, `space` | AC bullet 9 | Integration | TC110-I01–I04 |
| `NO_COLOR`/`--color never`/piped → zero ANSI | AC bullet 10 | Unit + Integration | TC110-U18, TC110-U20, TC110-I01, TC110-I08 |
| `--color always` → ANSI present | AC bullet 10 (implicit) | Unit + Integration | TC110-U19, TC110-I02 |
| `refresh_interval` default, CLI flag wins | AC bullet 11 | Unit + Integration | TC110-U21–U23, TC110-I05–I06 |
| Misconfigured thresholds → descriptive error | AC bullet 12 | Unit + Integration | TC110-U10–U13, TC110-I07 |

**Coverage Gaps:**
- Visual color-hue correctness in a real terminal is not covered by an automated numbered test (see §2 Test Type 3). Documented as an accepted manual-validation gap, LOW risk (ANSI code correctness is proven programmatically; only human perception of the resulting hue is unverified).
- Invalid *color name* handling (e.g., `normal = "chartreuse"`) is included as TC110-U17 but its exact behavior (hard error vs. silent fallback to default) is not specified in the planning doc — the test is written to assert whatever the architect's design doc specifies; if the design doc is silent, this is flagged back to the architect as a spec gap before TC110-U17 is implemented.

### 5. Gap Analysis

**Interactive/PTY color-rendering tests** — omitted as automated tests, per Test Type 3 analysis above. Risk: LOW. Mitigation: one manual validation pass (`script` capture) at sprint closure showing warning/critical rows in an actual terminal for at least `resources` and `space`. Revisit if users report incorrect/illegible color choices.

### 6. Test Implementation Plan

**Unit Tests**
- **Location:** `src/config.rs` `mod tests` (config parsing/defaults/validation) + `src/commands/monitoring_utils.rs` or a new `src/monitoring/severity.rs` module `mod tests` (classification, color mapping, ANSI presence-as-pure-function, `refresh_interval` precedence) — exact module split depends on where the architect places `MonitoringSettings`/severity logic; strategy assumes it follows the `monitoring_utils.rs` naming already used by `skew.rs`'s helpers (`escape_csv`, `extract_decimal`, etc.).
- **Count:** 24 (TC110-U01–U24)

**Integration Tests**
- **Location:** New file `tests/integration_monitoring.rs`, `#[ignore]` for the live-DB-dependent ones (TC110-I03–I07), not-ignored for the pure subprocess/env ones that don't need a DB round-trip if a no-DB error path suffices (TC110-I08 may run without `--ignored` if it can be triggered pre-connection; otherwise it also requires `#[ignore]`).
- **Count:** 8 (TC110-I01–I08)
- **Setup requirements:** Scripted `.tq.toml` files with controlled `[monitoring.thresholds]`/`[monitoring.colors]` (reusing `create_user_config`-style helper from `tests/integration_project_config_edge_cases.rs`), live `TQ_LOGON`, and a way to force a metric into Warning/Critical territory deterministically — the most reliable way is setting an artificially low threshold (e.g., `cpu_warning = 0`) against a live session/table that has *any* nonzero usage, rather than trying to manufacture a specific real-world CPU/skew value.

### 7. Coverage Sufficiency Assessment

Unit tests pin the classification boundaries, validation rules, and precedence logic exactly. Integration tests prove those rules actually suppress/emit real ANSI bytes in real command output under real OS-level stream conditions. Combined coverage is sufficient to claim "works as specified," with one explicitly accepted manual-validation gap (visual hue correctness) consistent with this project's own `docs/testing/approach.md` guidance on Type 3 terminal-output features.

---

## Numbered Test Plan (Sprint 69 Rule — every planned test enumerated)

### TC109 — Space Analysis (#54)

**Unit (13):**
- TC109-U01: Skew formula returns `None`/NULL when `MAX(CurrentPerm) = 0` (divide-by-zero guard)
- TC109-U02: Skew formula computes correct % for a known avg/max pair
- TC109-U03: Zero-size object (`CurrentPerm=0`, `PeakPerm=0`) does not panic; displays `0%`/N/A appropriately
- TC109-U04: `from_row` parses a `DBC.DiskSpaceV`-shaped row into the database-level struct (happy path)
- TC109-U05: `from_row` parses a `DBC.TableSizeV`-shaped row into the object-level struct (happy path)
- TC109-U06: Qualified name `"db.obj"` splits into `(db, obj)` components correctly
- TC109-U07: Unqualified name `"db"` is identified as database-only (no object component)
- TC109-U08: `dbspace` argument validation rejects a qualified `"db.obj"` string before querying
- TC109-U09: Table-format output distinguishes the database header row from object rows
- TC109-U10: JSON output schema includes all required fields (`CurrentPerm`, `PeakPerm`, `MaxPerm`, skew %, Spool, Temp) with correct nullability
- TC109-U11: CSV output correctly escapes values via the existing `escape_csv` helper
- TC109-U12: Markdown output correctly escapes pipe characters via the existing `markdown_escape_pipe` helper
- TC109-U13: `% of MaxPerm used` calculation is NULL-safe when `MaxPerm = 0`

**Integration — live DB (14):**
- TC109-I01: `tq space <db>` returns one header row + one row per contained object
- TC109-I02: `tq space <db>.<obj>` returns exactly one row
- TC109-I03: `tq dbspace <db>` returns database-level perm/spool/temp metrics only (no object rows)
- TC109-I04: `tq dbspace <db>.<obj>` fails with a clear, actionable error naming the qualified-name problem
- TC109-I05: `tq dbspace <table_name>` (a real object that is not a database) fails with a distinct "not a database" error
- TC109-I06: `tq space <unknown_db>` produces the standard not-found error (with spelling suggestion if the helper exists)
- TC109-I07: `tq space <db>.<unknown_obj>` produces the standard not-found error for the object
- TC109-I08: `--format json` produces valid, parseable JSON
- TC109-I09: `--format csv` produces valid CSV
- TC109-I10: `--format markdown` produces a valid markdown table
- TC109-I11: `--format table` (default) produces human-readable table output
- TC109-I12: A database with zero contained objects returns the header row only, no crash
- TC109-I13: Live skew % values are within `[0, 100]` or explicitly null — never NaN or negative
- TC109-I14: Live execution confirms `DBC.DiskSpaceV`/`DBC.TableSizeV` queries run without column/object errors (validates SQL from #54 against the real catalog)

**TC109 total: 27 tests (13 unit + 14 integration)**

### TC110 — Monitoring Thresholds & Colors (#23)

**Unit (24):**
- TC110-U01: Default thresholds apply when `[monitoring.thresholds]` is absent entirely
- TC110-U02: Partial thresholds section (e.g., only `cpu_warning` given) — other keys fall back to defaults individually
- TC110-U03: Partial colors section (e.g., only `warning` given) — `normal`/`critical` fall back to defaults individually
- TC110-U04: `[monitoring]` section missing entirely → full struct defaults, no panic
- TC110-U05: Value `< warning` → classified `Normal`
- TC110-U06: Value `>= warning and < critical` → classified `Warning`
- TC110-U07: Value `>= critical` → classified `Critical`
- TC110-U08: Value exactly `== warning` → classified `Warning` (inclusive lower boundary)
- TC110-U09: Value exactly `== critical` → classified `Critical` (inclusive lower boundary)
- TC110-U10: `warning == critical` in config → validation error at load time, not silently accepted
- TC110-U11: `warning > critical` in config → descriptive validation error
- TC110-U12: Negative threshold value (e.g., `cpu_warning = -5`) → descriptive validation error
- TC110-U13: Threshold `> 100` (e.g., `cpu_warning = 150`) → descriptive validation error
- TC110-U14: `skew_warning`/`skew_critical` resolve independently of `cpu_*`/`io_*`/`space_*` keys
- TC110-U15: `space_warning`/`space_critical` are used specifically for "% of MaxPerm used" classification
- TC110-U16: Color name strings (`"green"`/`"yellow"`/`"red"`) map to the correct internal color representation
- TC110-U17: Invalid/unrecognized color name in `[monitoring.colors]` produces the behavior specified by the design doc (error or documented fallback — test asserts whichever the architect specifies; flagged back if undocumented)
- TC110-U18: Zero ANSI escape bytes emitted when color choice is `Never`, regardless of severity
- TC110-U19: ANSI escape bytes emitted when color choice is `Always` for a Warning/Critical severity value
- TC110-U20: `NO_COLOR` env var set → zero ANSI bytes even with color choice `Auto` and severity `Critical`
- TC110-U21: `refresh_interval` from config is used as the watch-loop interval when no `--interval` flag is given
- TC110-U22: An explicit `--interval` flag overrides the configured `refresh_interval` (precedence)
- TC110-U23: `refresh_interval` outside `MIN_INTERVAL_SECS`/`MAX_INTERVAL_SECS` bounds in config → validation error
- TC110-U24: A `None`/idle metric value (e.g., idle session with no skew) classifies as `Normal`, never falsely `Critical`

**Integration — subprocess/live DB (8):**
- TC110-I01: `tq resources` (or `skew`/`space`) piped to a non-TTY emits zero ANSI bytes end-to-end
- TC110-I02: `tq resources --color always` piped emits ANSI bytes for a metric forced into Warning/Critical via a low custom threshold
- TC110-I03: `tq skew` output is colorized consistent with configured thresholds against live session data
- TC110-I04: `tq space` output is colorized when `space_warning`/`space_critical` are crossed, against a live object
- TC110-I05: `tq resources --watch --interval 2` overrides a configured `refresh_interval = 6` (observed via timing of refresh cycles)
- TC110-I06: `tq resources --watch` with no `--interval` uses the configured `refresh_interval` from project `.tq.toml` (observed via timing)
- TC110-I07: A misconfigured `.tq.toml` (`warning > critical`) causes the command to exit with a descriptive config error and non-zero exit code, not a panic
- TC110-I08: `NO_COLOR=1` env var set on a real subprocess emits zero ANSI bytes regardless of `--color` flag value

**TC110 total: 32 tests (24 unit + 8 integration)**

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ Feature 1 (space), Feature 2 (monitoring)
- Integration tests (live DB / subprocess): ✅ Feature 1 (space), Feature 2 (monitoring)
- Interactive tests (expectrl): ❌ Feature 1 (not applicable) / ⚠️ Feature 2 (recommended manual-only, not automated this sprint)
- Benchmark tests: ❌ none

**Estimated Test Count:**
- Unit: 37 (13 + 24)
- Integration: 22 (14 + 8)
- **Total: 59 tests planned**

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: the `--interval` clap `default_value = "6"` baked-default issue (§Feature 2, Validation Challenges) blocks TC110-U21/U22 and TC110-I05/I06 unless the architect changes the CLI arg to `Option<u64>`. Flagged to the architect now, before Phase 3, so it isn't discovered mid-implementation.
- LOW risk gaps: visual color-hue correctness (manual validation only); "spelling suggestion" helper for Feature 1 may be descoped under time pressure (test degrades gracefully, documented, not silently dropped).

**Dependencies Required:**
- Live database: Yes (`.env` `TQ_LOGON`) — for all `*-I*` tests and for validating #54's SQL against real `DBC.DiskSpaceV`/`DBC.TableSizeV`
- Network access: No (beyond the DB connection)
- Specific OS: No
- Other: A test database/user with at least one child object (for TC109-I01/I12), a test project directory for scripted `.tq.toml` files (for TC110-I*), ability to force a metric into Warning/Critical territory via artificially low thresholds rather than relying on real-world resource pressure

---

## New Test Tooling Required — flagged to coordinator

**Yes, one piece of new/extended test tooling is requested for Phase 2/3:**

1. **ANSI-detection helper (new).** No existing helper scans raw output for ANSI escape sequences (`\x1b[` / `\u{1b}[`). Every `*-U18/U19/U20` and `*-I01/I02/I08` test needs a shared assertion like:
   ```rust
   pub fn contains_ansi(bytes: &[u8]) -> bool {
       bytes.windows(2).any(|w| w == [0x1b, b'['])
   }
   pub fn assert_no_ansi(bytes: &[u8]) { assert!(!contains_ansi(bytes), "unexpected ANSI escape sequence in output"); }
   pub fn assert_has_ansi(bytes: &[u8]) { assert!(contains_ansi(bytes), "expected ANSI escape sequence, found none"); }
   ```
   Requested location: `tests/helpers/mod.rs` (alongside the existing `pty_harness.rs`/`pager_fixtures.rs` helpers), reusable by both `tests/integration_monitoring.rs` and any unit tests in `src/` that check color output at the string level.

2. **Scripted monitoring-config-file helper (extend existing).** `tests/integration_project_config_edge_cases.rs` already has a `create_user_config(home_dir, toml_content)`-style helper for writing arbitrary TOML to a fake `~/.tq/config.toml`. Requesting it be exposed as a shared helper (moved to `tests/helpers/mod.rs` or re-exported) so `tests/integration_monitoring.rs` can reuse it verbatim for `[monitoring.thresholds]`/`[monitoring.colors]` fixtures instead of duplicating file-writing boilerplate.

No new crates are required for either — both are pure `std`-based helpers, consistent with the "No new crates required" pattern already documented in `tests/README.md` for the fd-redirection and delayed-producer test patterns.

**Non-tooling design flag (not a testing request, but blocks two integration tests and two unit tests if unaddressed):** the `--interval` clap field's baked `default_value = "6"` must become distinguishable from "absent" (e.g., `Option<u64>` with `default_value` removed) for `refresh_interval` precedence to be testable or implementable at all. This is forwarded to `rust-teradata-architect` for Phase 2 design, not something quality-validator can resolve in test code.

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed) — verified against actual code (`src/config.rs`, `src/cli.rs`, `src/commands/skew.rs`, `src/commands/resources.rs`, `src/commands/watch.rs`)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest (fuzzy-match helper absence, clap baked-default issue, manual-validation-only color hue check — all disclosed, not hidden)
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-08-06
**Review Status:** DRAFT
**Submitted for Review:** 2026-08-06

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** —
**Review Comments:** —
