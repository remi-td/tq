# Test Case Index for tq (Teradata Query)

**Project:** tq - Teradata Query CLI Tool
**Version:** 1.51.0 (Sprint 69 - PTY Cursor-Position Fix + Status Bar Composition)
**Last Updated:** 2026-05-29
**Base Commit:** [Sprint 69 - In Progress]

## Overview

This directory contains comprehensive test case definitions for the tq CLI tool. These test cases cover all implemented MVP features (FR-001 through FR-010) and provide detailed procedures for validating functionality, usability, error handling, and security.

## Test Case Categories

### Sprint 76: Feature — Perm Space Analysis (#54) + Monitoring Thresholds & Colors (#23)

- **TC109**: Space Analysis (`tq space`, `tq dbspace`) — 13 unit tests (TC109-U01..U13, colocated in
  `src/commands/space.rs` `#[cfg(test)]`, owned by `rust-teradata-architect` per the project's
  unit-test-ownership convention) plus 14 live-DB integration tests (TC109-I01..I14) in
  `tests/integration_space.rs`, `#[ignore]`. Covers the three invocation shapes (`space <db>`,
  `space <db>.<obj>`, `dbspace <db>`), all four output formats, the NULL-safe skew formula, and the
  descoped (no fuzzy-match helper) not-found error path. — `TC109.md`

- **TC110**: Monitoring Thresholds & Colors — 24 unit tests (TC110-U01..U24, colocated in
  `src/config.rs`/`src/commands/severity.rs` `#[cfg(test)]`) plus 8 integration tests
  (TC110-I01..I08) in `tests/integration_monitoring.rs`, `#[ignore]`. Covers per-key partial-config
  defaults, inclusive severity-boundary classification, fatal threshold validation
  (`config.monitoring.validate()?` per coordinator ruling #3), ANSI presence/absence under
  `Auto`/`Always`/`Never`/`NO_COLOR`, and `refresh_interval` CLI-flag-over-config precedence
  (verified via a PTY since `--watch` requires `enable_raw_mode()`). — `TC110.md`

**Sprint 76 test case summary:**
- Unit tests (colocated in `src/`, owned by `rust-teradata-architect`): TC109-U01..U13 (13) +
  TC110-U01..U24 (24) = 37 new unit tests
- Integration tests (live DB / PTY, `#[ignore]`, owned by `quality-validator`):
  TC109-I01..I14 (14) + TC110-I01..I08 (8) = 22 new `#[ignore]` tests
- New shared test tooling (`tests/helpers/mod.rs`): `contains_ansi`/`assert_no_ansi`/
  `assert_has_ansi` (pure `std` byte scan); `create_user_config`/`create_project_config` promoted
  out of `tests/integration_project_config_edge_cases.rs`.
- **Total planned: 59 tests** (37 unit + 22 integration), per
  `tests/strategy/sprint-76-strategy.md`.

---

### Sprint 69: Feature — PTY Cursor-Position Fix + Pager Search Status Bar Composition

- **TC107**: PTY Cursor-Position Fix Validation — (a) Unit test for the `[6n` → `[1;1R` CPR
  responder mechanism (TC107-U01, no DB); (b) structural grep confirming the early-return cursor
  guard is removed from `test_pager_search_prompt_shows_match_count` (TC107-structural); (c) 8
  interactive `#[ignore]` watch tests TC107-A..H (= TC097-A..H on live DB) now expected to PASS
  outright — "PTY dump = acceptable" bar from Sprint 68 is retired; (d) TC107-TC104 = live run of
  `test_pager_search_prompt_shows_match_count` without early-return, must reach `Pattern: DBC`
  assertion. Location: TC107-U01 in `tests/common/pty_harness.rs`; TC107-A..H + TC107-TC104 in
  `tests/interactive_tests.rs`. — `TC107.md`

- **TC108**: Composed Search Status Bar Unit Tests — 7 new unit tests
  (TC108-U01..U07, no DB, no PTY) in `src/commands/repl/pager.rs` `#[cfg(test)]` covering:
  full composed format on wide terminal; separator `  |  ` exact; compact `Rows X-Y of Z` (no `%`);
  row context dropped when narrow; search segment truncated when very narrow; row numbers update
  after scroll; not-found path unchanged. Also validates existing pre-Sprint-69 status bar tests
  still pass after the implementation change. — `TC108.md`

**Sprint 69 test case summary:**
- Unit tests (no DB, no PTY): TC107-U01 (1 CPR mechanism test) + TC108-U01..U07 (7 status bar tests) = 8 new unit tests
- Interactive tests (live DB + PTY, `#[ignore]`): TC107-A..H (8 watch tests, now must PASS) + TC107-TC104 (1 pager search, no early-return) = 9 `#[ignore]` tests
- Structural checks: TC107-structural (1 grep for cursor guard removal) = 1 structural verification
- Sprint 69 new unit test baseline: 1134 (Sprint 68) + 8 = 1142 minimum

---

### Sprint 68: Maintenance — TC097 Migration + Sprint 67 Test Gaps + Toolchain Pin

- **TC101**: TC097-A..H Migration Validation — structural grep check (AC-OBJ1-1, AC-OBJ1-3) + 8 interactive `#[ignore]` watch tests (AC-OBJ1-2): each must pass or produce a PTY dump at `tests/results/sprint-66/<name>.pty.log`. Location: structural check via grep on `tests/interactive_tests.rs`; execution via `cargo test --test interactive_tests watch -- --ignored`. — `TC101.md`

- **TC102**: `scroll_to_match_snaps_to_rightmost_column` Unit Test — 1 unit test (no DB, no PTY): 8-column fixture with `term_width` calibrated for 3 visible columns; asserts `col_offset > 0` after `submit_search("TARGET")` where match is at col 6. Closes Sprint 67 AC-7 REQUIRED gap. Location: `src/commands/repl/pager.rs` `#[cfg(test)]`. — `TC102.md`

- **TC103**: `write_value_with_highlights_emits_reverse_video` Unit Test — 1 unit test (no DB, no PTY): `Vec<u8>` writer fixture; asserts `\x1b[7m` before matched substring and `\x1b[27m` after, with prefix/suffix written unmodified. Closes Sprint 67 AC-8 REQUIRED gap. Location: `src/commands/repl/pager.rs` `#[cfg(test)]`. — `TC103.md`

- **TC104**: `test_pager_search_prompt_and_exit` PTY Test — 1 interactive `#[ignore]` test (live DB + PTY): uses `spawn_tq_repl_tiered_with_pager` (omits `--no-pager`); queries DBC.ColumnsV to trigger pager; sends `/`, enters `ColumnName\n`, asserts `Pattern:` in PTY; exits with `q`, asserts `tq>` returns. Closes Sprint 67 AC-1/AC-11 REQUIRED gap. Location: `tests/interactive_tests.rs`. — `TC104.md`

- **TC105**: `rust-toolchain.toml` Validation — 4 checks: file exists with valid `[toolchain]` section (AC-OBJ4-1); `scripts/ci-check.sh` exits 0 locally (AC-OBJ4-2 local proxy); GitHub Actions CI green on sprint tag (AC-OBJ4-2 remote, deferred to Phase 5); `#![deny(warnings)]` decision documented (AC-OBJ4-3). — `TC105.md`

- **TC106**: REQUIRED-Test Rule in Testing Docs — 3 checks: `docs/testing/approach.md` contains `MEDIUM-severity` gap statement (AC-OBJ3-1); `docs/testing/philosophy.md` aligned (AC-OBJ3-2); `docs/testing/honest-assessment.md` preserved from Sprint 67 (AC-OBJ3-3, RECOMMENDED). All verified by grep. — `TC106.md`

**Sprint 68 test case summary:**
- Unit tests (no DB, no PTY): TC102 (1 test in `src/commands/repl/pager.rs`) + TC103 (1 test in `src/commands/repl/pager.rs`) = 2 new unit tests
- Interactive tests (live DB + PTY, `#[ignore]`): TC101-A..H (8 migrated tests) + TC104-I01 (1 new pager search test) = 9 `#[ignore]` tests
- Structural checks (grep, no DB): TC101-code-check (3 greps) + TC105-file/local/doc (3 checks) + TC106-approach/philosophy/honest (3 greps) = 9 structural verifications
- Remote CI: TC105-ci (1 observation, Phase 5)
- Sprint 68 new unit test baseline: 1132 (Sprint 67) + 2 = 1134 minimum

---

### Sprint 67: Feature — Search in Pager + `handle_tick_result` extraction

- **TC099**: Pager Search (Feature 1) — 32 unit tests (no DB, no PTY): `parse_search_input` (AC-6 case suffix), `find_all_matches` (AC-2/3/6/8/9/10), `pick_initial_match` (AC-2/4), status-bar writer-injected rendering (AC-1/3/4/5/9), `submit_search` integration (AC-2/3/10), `jump_match` navigation (AC-4/5), help-text writer-injected rendering (AC-12). Location: `src/commands/repl/pager.rs` `#[cfg(test)]`. AC-1/AC-11 interactive PTY: skipped for reason — fallback to manual per sprint-67-strategy.md. AC-8 ANSI unit: skipped for reason — no writer-injected highlight path; ANSI bytes confirmed by source grep. — `TC099.md`

- **TC100**: `handle_tick_result` extraction (Feature 2, Sprint 65 P2 follow-up) — 4 unit tests (no DB, no PTY): error path retains `last_body` (AC-3), success path replaces `last_body` (AC-4), empty-last_body edge case (AC-3 corollary), ownership transfer (AC-1 purity proof). Location: `src/commands/watch.rs` `#[cfg(test)]`. AC-2 behavioral identity: skipped for reason — TC097 pre-existing timeout; code inspection confirms behavioral identity. — `TC100.md`

**Sprint 67 test case summary:**
- Unit tests (no DB, no PTY): TC099 (32 tests in `src/commands/repl/pager.rs`) + TC100 (4 tests in `src/commands/watch.rs`) = 36 unit tests
- Interactive tests (live DB + PTY, `#[ignore]`): none authored in Sprint 67 (AC-1/AC-11 skipped with fallback to manual)
- Sprint 67 new unit test count: baseline 1096 → 1132 (+36 new)
- Evidence: `tests/results/sprint-67/test-evidence-1.md`

---

### Sprint 66: Maintenance — Test Infra Hardening (Tiered PTY Timeouts + PTY Buffer Dump)

- **TC098**: Tiered Interactive-Test Timeouts + PTY Buffer Dump — 4 unit tests (no DB, no PTY): `TieredTimeouts` default values (F1-AC-1), `PtyError` variant Display names (F1-AC-2), `dump_pty_buffer()` writes last 4096 bytes (F1-AC-3), env overrides parsed correctly (F1-AC-4). Plus 1 `#[ignore]` interactive regression test (live DB + PTY): `test_repl_startup_and_quit` migrated to tiered API (F1-AC-5). Location: unit tests in `tests/common/pty_harness.rs`, interactive test in `tests/interactive_tests.rs` — `TC098.md`

**Sprint 66 test case summary:**
- Unit tests (no DB, no PTY): TC098 (4 tests in `tests/common/pty_harness.rs`) = 4 unit tests
- Interactive tests (live DB + PTY, `#[ignore]`): TC098-I01 (1 test in `tests/interactive_tests.rs`) = 1 `#[ignore]` test
- Re-executed from Sprint 65: TC097-A..H (8 tests) — outcome in `tests/results/sprint-66/tc097-failure-analysis.md`
- Total: 4 unit tests + 1 `#[ignore]` regression test

---

### Sprint 65: Feature — `/sessions --watch` Dynamic Session Monitoring

- **TC096**: `/sessions --watch` Interval Parsing Unit Tests — 8 unit tests (no DB, no PTY): no `--watch` flag returns `None` (AC-9), default interval 6 s (AC-1), `--interval 10` (AC-2), minimum boundary 1 s accepted (AC-3), interval 0 clamped to minimum (AC-3), maximum boundary 3600 s accepted (AC-3), above-max 3601 clamped to 3600 (AC-3), `--interval` alone without `--watch` returns `None` (AC-9). Location: `src/commands/watch.rs` `#[cfg(test)]` — `TC096.md`

- **TC097**: `/sessions --watch` Interactive PTY Tests — 8 `#[ignore]` interactive tests (live DB + PTY, expectrl): `q` exits watch mode and REPL prompt returns (AC-5), frame header contains interval indicator (AC-4), `Esc` exits watch mode (AC-5), `Ctrl-C` exits watch but not REPL (AC-5), exit snapshot is readable plain text (AC-6), terminal state restored after exit (AC-7), watch loop survives 3 ticks without crash (AC-8), non-watch `/sessions` regression guard (AC-9). Helper `strip_ansi()` added to `tests/interactive_tests.rs`. Location: `tests/interactive_tests.rs` — `TC097.md`

**Sprint 65 test case summary:**
- Unit tests (no DB): TC096 (8 tests in `src/commands/watch.rs`) = 8 unit tests
- Interactive tests (live DB + PTY, `#[ignore]`): TC097 (8 tests in `tests/interactive_tests.rs`) = 8 `#[ignore]` tests
- Manual smoke test: AC-7 panic-path `Drop` guard (not automatable)
- Total: 8 unit tests + 8 `#[ignore]` interactive tests

---

### Sprint 64: Bug Fixes — File Mode Parser & Stdin Detection

- **TC094**: BEGIN/END Depth Tracking in Statement Splitter (Bug #42) — 9 unit tests covering: single procedure (exact #42 repro), nested BEGIN/END blocks, BEGIN inside string literal, BEGIN/END inside line/block comments, multi-procedure script, mixed SPL + regular statements, case-insensitive headers (PROCEDURE/TRIGGER/MACRO), CREATE vs REPLACE variant, plain multi-statement regression guard. Plus 1 `#[ignore]` integration test: end-to-end `tq query --file repro_sp.sql` against live DB. Location: unit tests in `src/sql/parser.rs`, integration test in `tests/integration_tests.rs` — `TC094.md`

- **TC095**: Stdin Detection with Redirected Empty Stdin (Bug #43) — 3 `#[ignore]` process integration tests (live DB): `Stdio::null()` redirect (simulates `< /dev/null`), empty pipe with writer dropped (simulates `<<< ""`), stdin-only regression guard (`echo SQL | tq query`). 1 process integration test (no DB, no `#[ignore]`): real conflict still rejected (`echo SQL | tq query "SQL"`). 1 unit test: error message content verification. Manual-only: AC-5 TTY stdin path. Location: process tests in `tests/integration_tests.rs`, unit test in `src/commands/query.rs` — `TC095.md`

**Sprint 64 test case summary:**
- Unit tests (no DB): TC094 (9 tests in `src/sql/parser.rs`) + TC095 (1 test in `src/commands/query.rs`) = 10 unit tests
- Process integration tests (no DB, runs in `cargo test`): TC095-D (1 test) = 1 test
- Integration tests (live DB, `#[ignore]`): TC094 (1 test) + TC095 (3 tests) = 4 `#[ignore]` tests
- Total: 11 unit/no-DB tests + 4 `#[ignore]` tests

---

### Sprint 63: Pager Exit Snapshot

- **TC-063-001**: Pager Exit Snapshot Unit Tests — `render_exit_snapshot(&self, writer: &mut impl Write)` on `Pager`: viewport selection (row_offset/col_offset/page_size), box-drawing borders, header and data rows, hidden-columns footer format ("N columns hidden: col1, col2..."), --format csv/json hint, row-count/timing footer ("N row(s) in set (X.XXXs)"), no ANSI codes, `\n` line endings, CJK/Unicode-safe padding (Unit writer-injection `Vec<u8>`, no DB) — `TC-063-001.md`

**Sprint 63 test case summary:**
- Unit tests (no DB): TC-063-001 (10 tests)
- Structural grep checks: 2 checks (render_exit_snapshot exists, no `\r\n` in method)
- Interactive tests (recommended, DB required): 1 scenario (q/Esc trigger + terminal state)
- Total: 10 unit tests + 2 structural checks

---

### Sprint 56: Result Pagination & Sprint 55 Cleanup

- **TC-056-001**: Query Result Pagination Unit Tests — JSON envelope pagination metadata (page/page_size/has_more/total_rows), slicing correctness (page 1, page 2, last page, partial last page, beyond last), backward compatibility (no pagination fields when flag absent), table/CSV/markdown format footer, error on --page without --page-size, error on --page-size with --limit (Unit writer-injection, no DB) — `TC-056-001.md`
- **TC-056-002**: Query Result Pagination Integration Tests — end-to-end CLI flag wiring: `--page-size`, `--page`, `--agent-safe` mode, backward compat (Integration live DB required, `#[ignore]`) — `TC-056-002.md`
- **TC-056-003**: Search and List Pagination — `tq search tables/columns --page-size N` and `tq list --page-size N` pagination, JSON envelope metadata, --limit/--page-size mutual exclusion (Unit no DB + Integration `#[ignore]`) — `TC-056-003.md`
- **TC-056-004**: Sprint 55 Tech Debt Cleanup — `markdown_escape_pipe()` unit tests, REPL `/search` dispatch alias routing tests, structural grep verifying `esc()` consolidated into format_helpers.rs and `_use_color` addressed (Unit + Structural grep, no DB) — `TC-056-004.md`

**Sprint 56 test case summary:**
- Unit tests (no DB): TC-056-001 (~14 tests), TC-056-003 (~8 tests), TC-056-004 (~11 tests) = ~33 unit tests
- Integration tests (DB required, `#[ignore]`): TC-056-002 (4 tests), TC-056-003 (2 tests) = 6 tests
- Structural grep: TC-056-004 (4 checks)
- Total: ~39 tests + 4 structural checks

---

### Sprint 55: Search/Discovery Commands

- **TC-055-001**: `tq search tables <keyword>` — struct construction, all four output formats (table/JSON/CSV/markdown), JSON envelope, `--database` scoping, no-results handling (Unit writer-injection + Integration DB required `#[ignore]`) — `TC-055-001.md`
- **TC-055-002**: `tq search columns <keyword>` — struct construction, all four output formats, JSON boolean nullable field, `--database` scoping, no-results handling (Unit writer-injection + Integration DB required `#[ignore]`) — `TC-055-002.md`
- **TC-055-003**: REPL `/search` metacommand — `/search tables`, `/search columns`, help text on bare `/search`, tab completion (Interactive expectrl DB required `#[ignore]` + Unit structural check no-DB) — `TC-055-003.md`

**Sprint 55 test case summary:**
- Unit tests (no DB): TC-055-001 (8 tests), TC-055-002 (8 tests), TC-055-003 (2 unit checks) = 18 unit tests
- Integration tests (DB required, `#[ignore]`): TC-055-001 (4 tests), TC-055-002 (4 tests) = 8 tests
- Interactive tests (DB required, `#[ignore]`): TC-055-003 (4 tests) = 4 tests
- Structural grep: TC-055-003 Part E (1 check)
- Total: ~31 test cases across 3 files

---

### Sprint 48: Query Layer Consolidation & Spec Alignment

- **TC-048-001**: Shared Query Layer Extraction — structural grep verifies each shared function/type defined once, regression run (Structural grep + cargo test --lib, no-DB) — `TC-048-001.md`
- **TC-048-002**: JSON API Type Fixes — boolean nullable, null default, integer rows/size, "database" key (Unit writer-injection + Integration DB required `#[ignore]`) — `TC-048-002.md`
- **TC-048-003**: Bug Fixes — summarize_error UTF-8 safety, TABLE→OBJECT structural grep, System/User type labels, Error: prefix, DescribeArgs.table→.object (Unit + Structural grep, no-DB) — `TC-048-003.md`
- **TC-048-004**: Edge Case Messages — "No indexes defined.", "No Primary Index (NoPI)", "No secondary indexes." (Unit + Integration DB required `#[ignore]`) — `TC-048-004.md`
- **TC-048-005**: Missing Unit Tests — 6 DDL tests from TC-047-001, writer-injection for describe/show_indexes/list_databases, column_type_case_sql 21-branch completeness (Unit, no-DB) — `TC-048-005.md`

**Sprint 48 test case summary:**
- Structural grep (no DB): TC-048-001 (6 checks), TC-048-003 (6 checks) = 12 structural checks
- Unit tests (no DB): TC-048-002 (7 tests), TC-048-003 (9 tests), TC-048-004 (7 tests), TC-048-005 (19 tests) = 42 unit tests
- Regression run: TC-048-001 (all ~130 lib tests must still pass)
- Integration tests (DB required, `#[ignore]`): TC-048-002 (3 tests), TC-048-004 (2 tests) = 5 tests
- Total: ~59 test cases across 5 files (plus regression baseline)

---

### Sprint 47: Tech Debt Elimination & Command Enrichment

- **TC-047-001**: Bug #36 — /inspect DDL & Column Type Fix for Views (Unit + Integration, DB required for integration) — `TC-047-001.md`
- **TC-047-002**: Shared Helpers Extraction — `json_escape`, `csv_escape`, `parse_table_name`, `truncate_str` with UTF-8 safety (Unit + Structural grep, no-DB) — `TC-047-002.md`
- **TC-047-003**: REPL Delegation — `/describe` and `/list` delegate to batch modules (Interactive, DB required, `#[ignore]`) — `TC-047-003.md`
- **TC-047-004**: Enrich `tq describe` — object header, Comments column, Indexes section, structured JSON (Unit + Integration, DB required for integration) — `TC-047-004.md`
- **TC-047-005**: Enrich `tq list` — Owner/Type for databases, Rows/Size for tables, structured JSON (Unit + Integration, DB required for integration) — `TC-047-005.md`
- **TC-047-006**: Enrich `tq show-indexes` — Primary/Secondary sections, UPI/NUPI/USI/NUSI labels, structured JSON (Unit + Integration, DB required for integration) — `TC-047-006.md`

**Sprint 47 test case summary:**
- Unit tests (no DB): TC-047-001 (6 tests), TC-047-002 (20 tests + 1 structural grep), TC-047-004 (18 tests), TC-047-005 (16 tests), TC-047-006 (15 tests) = 75 unit tests
- Interactive (DB required, `#[ignore]`): TC-047-003 (5 tests) = 5 tests
- Integration (DB required, `#[ignore]`): TC-047-001 (4 tests), TC-047-004 (5 tests), TC-047-005 (6 tests), TC-047-006 (4 tests) = 19 tests
- Total: ~99 test cases across 6 files

---

### Sprint 46: Bug Fixes & /inspect Polish

- **TC-046-001**: Bug #35 — `quote_identifier()` uppercase behavior (Unit, no-DB) — `TC-046-001.md`
- **TC-046-002**: Bug #35 — `extract_table_name()` word boundary matching (Unit, no-DB) — `TC-046-002.md`
- **TC-046-003**: Bug #35 — End-to-end sample/peek with identifier fix (Integration, DB required, `#[ignore]`) — `TC-046-003.md`
- **TC-046-004**: Bug #34 — Clap argument parsing for describe/list/show-indexes (Unit, no-DB) — `TC-046-004.md`
- **TC-046-005**: Bug #34 — CLI help text and error messages (CLI integration, no-DB) — `TC-046-005.md`
- **TC-046-006**: Bug #34 — End-to-end describe/list/show-indexes (Integration, DB required, `#[ignore]`) — `TC-046-006.md`
- **TC-046-007**: /inspect formatting compliance — all 8 ACs (Unit, no-DB) — `TC-046-007.md`
- **TC-046-INSPECT-INTEGRATION**: /inspect end-to-end formatting (Integration, DB required, `#[ignore]`) — `TC-046-INSPECT-INTEGRATION.md`

**Sprint 46 test case summary:**
- Unit tests (no DB): TC-046-001 (10 tests), TC-046-002 (8 tests), TC-046-004 (10 tests), TC-046-007 (15 tests) = 43 unit tests
- CLI integration (no DB, binary spawn): TC-046-005 (6 tests) = 6 tests
- Integration (DB required, `#[ignore]`): TC-046-003 (4 tests), TC-046-006 (5 tests), TC-046-INSPECT-INTEGRATION (3 tests) = 12 tests
- Total: 61 test cases across 8 files

### Sprint 45: Helper Bug Fix & Object Inspection

- **TC-045-001**: Semicolon Stripped from /describe Argument (Unit, no-DB) — `TC-045-001.md`
- **TC-045-BUG32-SEMICOLON**: Bug #32 Full Metacommand Semicolon Stripping Suite (Unit, no-DB) — covers TC-045-001 through TC-045-006 — `TC-045-BUG32-SEMICOLON.md`
- **TC-045-INSPECT-UNIT**: /inspect Command — Unit Tests for Pure Logic Helpers (Unit, no-DB) — covers TC-045-008 through TC-045-013 — `TC-045-INSPECT-UNIT.md`
- **TC-045-INSPECT-INTEGRATION**: /inspect Command — Integration Tests (DB required, #[ignore]) — covers TC-045-007, TC-045-014 through TC-045-018 — `TC-045-INSPECT-INTEGRATION.md`
- **TC-045-DEFERRED-S44**: Sprint 44 Deferred Items (Unit + Code Inspection, no-DB) — covers TC-045-020 through TC-045-023 — `TC-045-DEFERRED-S44.md`

**Sprint 45 test case summary:**
- Unit tests (no DB): TC-045-001 through TC-045-006 (Bug #32), TC-045-008 through TC-045-013 (/inspect logic), TC-045-021 through TC-045-023 (deferred items) = 23 unit tests
- Integration/interactive (DB required, ignored): TC-045-007, TC-045-014 through TC-045-019 = 9 tests
- Manual code inspection: TC-045-020 = 1 procedure
- Total: 33 test cases

### Sprint 44: Driver Distribution Fix & Profile Polish

- **TC-044-001**: Driver Path Resolution - Fallback Chain Order (Unit, no-DB, no-binary)
- **TC-044-002**: Driver Resolution Error Message Lists All Searched Paths (Unit, no-DB, no-binary)
- **TC-044-003**: build.rs Does Not Hardcode Absolute Build Path (Unit/Code Inspection, no-DB)
- **TC-044-004**: Install Script License Gate - shellcheck + Behavioral Tests (Shell, no-DB)
- **TC-044-005**: License File Stored in Repository, Not Network-Fetched (File Existence + Code Inspection)
- **TC-044-006**: Profile Flag Naming - --logmech and --password-file Work on Profile Subcommands (CLI Integration + Unit, no-DB)
- **TC-044-007**: Profile Delete Confirmation - --force Bypass and Non-TTY Behavior (Unit + Injection/expectrl, no-DB)
- **TC-044-008**: SqlParseError Struct Variant with Line and Column (Unit, no-DB)
- **TC-044-009**: display_profiles() Helper Produces Correct Output (Unit, no-DB)

### Sprint 43: Profile Management & Parser Polish

- **TC-043-001**: Profile Add - Happy Path (Integration + Unit, no-DB)
- **TC-043-002**: Profile Add - Error Cases (Integration + Unit, no-DB)
- **TC-043-003**: Profile Edit - Happy Path and Errors (Integration + Unit, no-DB)
- **TC-043-004**: Profile Delete - Happy Path and Errors (Integration + Code Inspection, no-DB)
- **TC-043-005**: Parser Remediation - Result Return Type and Error Location (Unit + Integration, no-DB)
- **TC-043-006**: Config Preservation - Add/Edit/Delete Preserve Unrelated Content (Integration + Unit, no-DB)

### Sprint 42: SQL Parser Hardening (Bugs #28, #29, #30)

- **TC-042-001**: Semicolons Inside Quoted Strings - Bug #28 (Unit, no-DB)
- **TC-042-002**: Multi-Line SQL Statements - Bug #29 (Unit, no-DB)
- **TC-042-003**: Comment Blocks Between Statements - Bug #30 (Unit, no-DB)
- **TC-042-004**: Combined Scenarios - All 3 Bugs Together (Unit + Integration)
- **TC-042-005**: Sprint 41 Remediation & Regression Suite (Regression + Code Inspection)

### Functionality (Core Features)
- **TC001**: Ping Command - Basic Connectivity Test
- **TC003**: Query Command - Basic Execution with Table Output
- **TC004**: Query Command - JSON Output Format
- **TC005**: Query Command - CSV Output Format
- **TC006**: Connection String Parsing - Valid Formats
- **TC008**: Authentication Mechanisms - TD2, LDAP, Kerberos
- **TC009**: Password File Support - Secure Credential Handling
- **TC010**: Query Command - Read from stdin
- **TC011**: Query Command - Read from File
- **TC012**: Query Command - Output to File
- **TC015**: Query Command - NULL Value Handling
- **TC016**: Query Command - Type Preservation in JSON
- **TC020**: Query Command - Large Result Sets
- **TC024**: Ping Command - Multiple Attempts
- **TC025**: Query Timing Information
- **TC026**: REPL Tab Completion - Table Names After FROM Keyword
- **TC027**: REPL Tab Completion - Table Names After JOIN Keywords
- **TC028**: REPL Tab Completion - Table Names After UPDATE Keyword
- **TC030**: REPL Tab Completion - Table Cache Invalidation
- **TC031**: REPL Tab Completion - Column Names After SELECT Keyword
- **TC032**: REPL Tab Completion - Column Names in JOIN Queries
- **TC033**: REPL Tab Completion - Column Names in ORDER BY and GROUP BY
- **TC035**: REPL Tab Completion - Column Cache Management
- **TC036**: REPL /logon Metacommand - Show Current Connection
- **TC037**: REPL /logon Metacommand - Successful Connection Switch
- **TC039**: REPL /logon Metacommand - State Preservation
- **TC040**: REPL /logon Metacommand - Authentication Mechanisms
- **TC041**: REPL /logon Metacommand - Performance and Timeout
- **TC-HELP-001**: Help Config Subcommand - Display Configuration Documentation (Sprint 17)
- **TC-HELP-002**: Help Credentials Subcommand - Display Password Management Guide (Sprint 17)
- **TC-PROFILES-001**: List Profiles from Config File (Sprint 17)

### Error-Handling
- **TC002**: Ping Command - Connection Failure
- **TC007**: Connection String Parsing - Invalid Formats
- **TC021**: Query Command - SQL Syntax Errors
- **TC029**: REPL Tab Completion - Table Metadata Error Handling
- **TC034**: REPL Tab Completion - Column Metadata Error Handling
- **TC038**: REPL /logon Metacommand - Connection Failure Handling
- **TC-HELP-003**: Help Unknown Topic - Error Handling (Sprint 17)
- **TC-PROFILES-002**: No Config File - Error Handling (Sprint 17)
- **TC-PROFILES-003**: Config Exists But No Profiles - Error Handling (Sprint 17)

### Usability
- **TC013**: CLI Help and Version Information
- **TC017**: Verbose and Quiet Output Modes
- **TC018**: Color Output Control

### Integration
- **TC014**: Exit Codes - Comprehensive Validation
- **TC019**: Environment Variable Configuration
- **TC023**: CSV Format - Special Character Escaping
- **TC042**: REPL Performance - Table Completion Benchmark
- **TC043**: REPL Performance - Column Completion Benchmark
- **TC044**: Table Formatting - Basic 5 Column Layout (Sprint 8)
- **TC045**: Table Formatting - Wide Table with 16+ Columns (Sprint 8)
- **TC046**: Table Formatting - NULL Values and Proper Alignment (Sprint 8)
- **TC047**: Table Formatting - Very Long Values and Truncation (Sprint 8)
- **TC048**: Table Formatting - Mixed Data Types Alignment (Sprint 8)
- **TC049**: Tab Completion - FROM Shows Databases and Current DB Tables (Sprint 8)
- **TC050**: Tab Completion - FROM database.TAB Shows Tables in That Database (Sprint 8)
- **TC051**: Tab Completion - Loading Indicator for Slow Metadata Queries (Sprint 8)
- **TC053**: Tab Completion - Cache Cleared After CREATE TABLE DDL (Sprint 8)
- **TC054**: Tab Completion - Cache Cleared After DROP TABLE DDL (Sprint 8)
- **TC055**: Tab Completion - Works With Alias Context (Sprint 8)
- **TC056**: Tab Completion - Handles Multiple Databases Gracefully (Sprint 8)
- **TC057**: Result Paging - Vertical Paging with j/k Keys (Sprint 8)
- **TC058**: Result Paging - Vertical Paging with PageUp/PageDown Keys (Sprint 8)
- **TC059**: Result Paging - Horizontal Paging with h/l Keys (Sprint 8)
- **TC060**: Result Paging - Horizontal Paging with Arrow Keys (Sprint 8)
- **TC061**: Result Paging - Pager Shows Position Indicator (Sprint 8)
- **TC062**: Result Paging - Exit Pager with q or Esc (Sprint 8)
- **TC063**: Result Paging - /pager on and /pager off Metacommands (Sprint 8)
- **TC064**: LIMIT Hint - Query with 100+ Rows Shows Correct Teradata Syntax (Sprint 8)
- **TC065**: LIMIT Hint - Help Text Uses Teradata Syntax (Sprint 8)

### Error-Handling (Sprint 8 Additions)
- **TC052**: Tab Completion - Error Messages When Metadata Query Fails

### Sprint 17: Configuration UX Completion

**Sprint 17 Test Cases (9 total):**

#### Help Subcommands (P0)
- **TC-HELP-001**: Help Config - Displays configuration documentation
- **TC-HELP-002**: Help Credentials - Displays password management guide
- **TC-HELP-003**: Help Unknown Topic - Error handling with available topics list

#### Profile Listing Command (P1)
- **TC-PROFILES-001**: List profiles from config file
- **TC-PROFILES-002**: No config file - Error handling with setup instructions
- **TC-PROFILES-003**: Config exists but no profiles - Error handling with add instructions

#### Security Enhancements (P0/P1)
- **TC-SECURITY-001**: Password file 0644 permissions **REJECTED** (enforcement)
- **TC-SECURITY-002**: Config file 0644 permissions **WARNED** (different policy)
- **TC-SECURITY-003**: Security check ordering - Permission check before file read

**Sprint 17 Test Strategy:**
- **Test Count**: 9 integration tests (all required)
- **No Database Needed**: All Sprint 17 features are CLI-only
- **No Interactive Tests**: All features are batch mode commands
- **Security Focus**: Multiple tests validate password protection and enforcement
- **Regression**: Full test suite must pass (280+ tests from Sprint 16)

### Sprint 18: Critical Bug Fixes (Maintenance Sprint)

**Sprint 18 Test Cases (6 total):**

#### Logo Fix (P0 - CRITICAL)
- **TC-LOGO-001**: Logo Display - Lowercase "tq" with Subtitle

#### Tab Completion Rebuild (P0 - CRITICAL)
- **TC-COMPLETION-001**: Tab Completion - Database Names After FROM
- **TC-COMPLETION-002**: Tab Completion - Table Names After FROM
- **TC-COMPLETION-003**: Tab Completion - Column Names in SELECT and WHERE
- **TC-COMPLETION-004**: Tab Completion - Qualified Name Completion (database.table)
- **TC-COMPLETION-005**: Tab Completion - Verify NO Keyword Completion

**Sprint 18 Test Strategy:**
- **Test Count**: 6 manual test cases (all critical)
- **Type**: Maintenance Sprint (CRISIS) - fixing blocking production bugs
- **Database Required**: Yes (for tab completion metadata queries)
- **Interactive Tests Required**: Yes (all features are REPL-based)
- **Focus**: Logo branding fix + tab completion rebuild from scratch
- **Acceptance**: Both P0 bugs must be 100% fixed, no regressions
- **Outcome**: APPROVED but user reported bugs still present (false positive)

### Sprint 19: CRITICAL BUG FIXES - RETRY (Sprint 18 Failed)

**Sprint 19 Context:** Sprint 18 was APPROVED but user reports SAME bugs still present.

**Sprint 19 Test Cases (3 total - manual visual tests only):**

#### Logo Fix - RETRY (P0 - CRITICAL)
- **TC-LOGO-002**: Logo ASCII Art with Info on Right (Manual visual test)

#### Tab Completion Fix - RETRY (P0 - CRITICAL)
- **TC-TAB-COMPLETION-001**: Tab Completion After FROM (No Pager Output) (Manual test)
- **TC-TAB-COMPLETION-002**: Tab Completion After Qualified Name (No Pager Output) (Manual test)

**Sprint 19 Test Strategy:**
- **Test Count**: 3 manual visual test cases (all critical)
- **Type**: Maintenance Sprint (CRISIS - RETRY)
- **Why Retry**: Sprint 18 tests gave FALSE POSITIVES - tests passed but bugs not fixed
- **Database Required**: Yes (for tab completion)
- **Real Terminal Required**: YES - PTY automation missed bugs in Sprint 18
- **Manual Testing**: MANDATORY - No automated tests, human visual validation only
- **Screenshot Evidence**: REQUIRED for all tests
- **Focus**: Verify ACTUAL user experience, not code behavior
- **Key Difference**: Tests what USER SEES, not what code returns
- **Acceptance**: User's exact bug reports must be proven fixed with visual evidence

### Sprint 20: CRITICAL BUG FIXES - HYBRID TESTING (Sprint 18/19 Failed)

**Sprint 20 Context:** Sprint 18 and 19 both failed to fix two critical bugs. Sprint 20 implements hybrid testing strategy.

**Sprint 20 Test Cases (2 total - hybrid: automated + manual):**

#### Logo Fix - 9-Line ASCII Art (P0 - CRITICAL)
- **TC-LOGO-003**: Logo Display Verification - 9-Line ASCII Art (Hybrid: Interactive automated + manual visual)

#### Tab Completion Fix - No Pager Output (P0 - CRITICAL)
- **TC-TAB-COMPLETION-003**: Tab Completion Without Pager Output (Hybrid: Interactive automated + manual visual)

**Sprint 20 Test Strategy:**
- **Test Count**: 2 hybrid test cases + 8-10 automated tests + 2 screenshots
- **Type**: Maintenance Sprint (CRISIS - FINAL ATTEMPT)
- **Why Hybrid**: Prevent Sprint 18 false positives AND Sprint 19 execution blockers
- **Database Required**: Yes (for tab completion tests)
- **Automated Component**: PTY tests with negative assertions (NO pager text) for regression detection
- **Manual Component**: Human visual validation with screenshot evidence for correctness
- **Unit Tests**: OutputSuppressor mechanism, logo data structures
- **Interactive Tests**: Tab completion, logo rendering with expectrl
- **Screenshot Evidence**: MANDATORY for both tests
- **Focus**: Test what users SEE (manual) AND prevent regressions (automated)
- **Key Innovation**: BOTH automated and manual must pass for APPROVED verdict
- **Acceptance**: User confirms bugs fixed + automated tests pass (100%)

---

### Sprint 23: Batch Mode File Output & Transaction Control

**Sprint 23 Context:** Feature sprint implementing batch mode improvements with testing infrastructure enhancements.

**Sprint 23 Test Cases (17 total):**

#### Feature 1: Batch Mode Output to File (P0) - 9 tests
- **TC077**: Output to File - Table Format (basic functionality)
- **TC078**: Output to File - CSV Format (RFC 4180 compliance)
- **TC079**: Output to File - JSON Format (type preservation)
- **TC080**: Atomic File Writing (temp + rename pattern)
- **TC081**: File Output Error - Permission Denied
- **TC082**: File Output Error - Invalid Path
- **TC083**: File Overwrite - Existing File
- **TC084**: Large Result Sets - Streaming to File
- **TC085**: Empty Result Set to File

#### Feature 2: Batch Mode Transaction Control (P1) - 6 tests
- **TC086**: Transaction Control - Basic Success (--atomic)
- **TC087**: Transaction Control - Rollback on Error
- **TC088**: Transaction Status Messages
- **TC089**: Nested Transaction Detection
- **TC090**: Single Statement - No Transaction
- **TC091**: Large Transaction - Many Statements

#### Integration Tests - 2 tests
- **TC092**: Combined Feature - File Output with Atomic Transaction
- **TC093**: Transaction with Different Output Formats

**Sprint 23 Test Strategy:**
- **Test Count**: 17 integration tests (15 required, 2 integration)
- **Type**: Feature Sprint (hybrid - testing infrastructure + new features)
- **Database Required**: Yes (batch mode features require live database)
- **Test Types**: Unit tests (8-10) + Integration tests (22-27) per strategy
- **Critical Success Factor**: Apply checklist before quality review
- **Test Implementation**: Both unit AND integration tests required (Sprint 22 lesson)
- **Documentation**: Test only delivered features, no deferred features documented
- **Acceptance**: 100% test pass rate for P0 features, zero regressions

### Sprint 27: Bug Fix and Documentation (Bug Fix + LICENSE + README)

**Sprint 27 Context:** Critical bug fix for /sessions command + legal compliance + user-facing documentation improvements.

**Sprint 27 Test Cases (15 total: 11 automated + 3 manual + 1 regression):**

#### Feature 1: Bug Fix - /sessions Command (#10) - 4 tests
- **TC-SESS-BUG-001**: Bug Fix - All Sessions Displayed (Row Count Match)
- **TC-SESS-BUG-002**: Bug Fix - Session State Coverage (All States Displayed)
- **TC-SESS-BUG-003**: Bug Fix - Regression Test (Sprint 26 Tests Still Pass)
- **TC-SESS-BUG-004-MANUAL**: Bug Fix - Manual Verification with User Scenario

#### Feature 2: LICENSE File Validation (#8) - 5 tests
- **TC-LICENSE-001**: LICENSE File Existence and Completeness
- **TC-LICENSE-002**: LICENSE Attribution Validation (MIT + BSD + Go)
- **TC-LICENSE-003**: NOTICE or THIRD-PARTY-LICENSES File Check
- **TC-LICENSE-004**: README Licensing Section
- **TC-LICENSE-MANUAL**: Legal Compliance Manual Review (BLOCKING)

#### Feature 3: README Validation (#9) - 6 tests
- **TC-README-001**: README Structure and TLDR Section
- **TC-README-002**: README AI Development Story
- **TC-README-003**: README Screenshot Validation
- **TC-README-004**: README Installation Instructions
- **TC-README-005**: README Documentation Links
- **TC-README-006**: README GitHub Configuration Section Moved
- **TC-README-MANUAL**: README Tone and Quality Manual Review (BLOCKING)

**Sprint 27 Test Strategy:**
- **Test Count**: 15 test cases (11 automated + 3 manual + 1 regression suite)
- **Type**: Bug Fix + Documentation Sprint
- **Database Required**: Yes (bug fix tests only)
- **Test Types**: Integration tests (row count, state coverage, file validation) + Manual reviews (2 BLOCKING)
- **Critical Focus**: Bug fix must not regress Sprint 26 functionality
- **Manual Reviews**: LICENSE legal review and README quality review are BLOCKING for release
- **Acceptance**: 100% bug fix validation + legal compliance + professional README

### Sprint 33: Pager Bug Fix + Data Sampling Commands

**Sprint 33 Context:** Fix pager rendering bug from Issue #14 (disable by default) + deliver data exploration feature with `/sample` and `/peek` commands.

**Sprint 33 Test Cases (10 total: 9 automated + 1 manual documented):**

#### Feature 1: Pager Bug Fix (Issue #14) - 1 test + verification
- **TC-033-001**: Pager Disabled by Default - Unit test for AC-3 (pager_enabled: false)
- **TC-033-PAGER-MANUAL**: Manual Visual Validation - Documented test case for pager rendering at terminal width 117 (NOT EXECUTABLE - no human tester)
- **Existing Tests Verification**: 27 pager unit tests + 48 interactive tests must pass (AC-4, AC-5, AC-10)

#### Feature 2: Data Sampling Commands - 8 tests
- **TC-033-002**: Unit Tests - /sample Command (parsing, SQL generation, validation)
- **TC-033-003**: Unit Tests - /peek Command (parsing, metadata query generation)
- **TC-033-004**: Integration Tests - /sample Command (live database execution)
- **TC-033-005**: Integration Tests - /peek Command (metadata + data retrieval)
- **TC-033-006**: Interactive Tests - /sample in REPL (PTY, tab completion, help)
- **TC-033-007**: Interactive Tests - /peek in REPL (PTY, tab completion, help)
- **TC-033-008**: Batch Mode Tests - tq sample CLI command
- **TC-033-009**: Batch Mode Tests - tq peek CLI command

#### Test Coverage Summary
- **TC-033-COVERAGE**: Comprehensive test coverage matrix mapping all 25 acceptance criteria to test cases

**Sprint 33 Test Strategy:**
- **Test Count**: 10 test case documents (9 automated + 1 manual documented)
- **Estimated Test Functions**: 61-66 automated tests
- **Type**: Mixed Sprint (Bug Fix + Feature)
- **Database Required**: Yes (for data sampling integration/interactive/batch tests)
- **PTY Required**: Yes (for interactive tests)
- **Test Types**: Unit (3 docs, ~18 tests) + Integration (2 docs, ~18 tests) + Interactive (2 docs, ~13 tests) + Batch (2 docs, ~15 tests) + Manual (1 doc, documented only)
- **Critical Success**: 100% automated test pass rate + pager disabled by default
- **Pager Manual Validation**: Documented but NOT EXECUTED (no human tester) - pager disabled by default for user protection
- **Acceptance**: All automated tests pass + all 25 ACs covered + zero regressions

### Sprint 34: Technical Debt Cleanup (Maintenance Sprint)

**Sprint 34 Context:** Maintenance sprint addressing technical debt from Sprint 33 review - code duplication, security hardening, and documentation synchronization.

**Sprint 34 Test Cases (3 test documents covering 15 acceptance criteria):**

#### Track 1: Code Quality - Extract Duplicate Code
- **TC-034-CODE-QUALITY-001**: Extract format_column_type() to Shared Module (AC-1 to AC-5)
  - Unit tests for shared type formatting utility (12 tests)
  - Code review verification (module structure, no duplicates, imports)
  - Regression suite validation (471 tests must pass)

#### Track 2: Security Hardening - SQL Identifier Quoting
- **TC-034-SECURITY-001**: SQL Identifier Quoting for Security Hardening (AC-6 to AC-10)
  - Unit tests for quote_identifier() function (7 tests)
  - Unit tests for quote_qualified_name() function (5 tests)
  - SQL generation tests with quoting (5 tests)
  - Integration tests with special character table names (2 tests, database-dependent)
  - Regression suite validation (471 tests must pass)

#### Track 3: Documentation Synchronization
- **TC-034-DOCUMENTATION-001**: Documentation Synchronization (AC-11 to AC-15)
  - Manual review of /peek specification update (REQ-SAMPLE-004.1)
  - Manual review of pager status badges
  - Code review for spec/impl alignment
  - Regression tests to confirm no code changes

#### Test Summary
- **TC-034-SUMMARY**: Sprint 34 test execution plan and coverage matrix

**Sprint 34 Test Strategy:**
- **Test Count**: 3 test case documents + 1 summary document
- **Estimated Test Functions**: 29 new automated tests + 471 regression tests = 500 total
- **Type**: Maintenance Sprint (Technical Debt Cleanup)
- **Database Required**: Optional (only for Track 2 integration tests - can skip with BLOCKED verdict)
- **Test Types**: Unit (29 new tests) + Code Review (6 verifications) + Manual Review (5 documentation reviews) + Regression (471 existing tests)
- **Critical Success**: 100% test pass rate (500/500) + zero regressions + all 15 ACs satisfied
- **Track 1 Focus**: Code quality - extract duplicates, shared utilities
- **Track 2 Focus**: Security - SQL identifier quoting for injection prevention
- **Track 3 Focus**: Documentation - synchronize specs with implementation
- **Acceptance**: All automated tests pass + code review clean + documentation aligned + zero regressions

### Sprint 38: PMON Foundation - System Config & Lock Monitoring

**Sprint 38 Context:** First two PMON (Performance Monitor) commands for DBA observability: `/sysconfig` displays system topology (version, AMP count, nodes) and `/locks` displays current lock contention. Both follow the established `sessions.rs` pattern.

**Sprint 38 Test Cases (10 test documents covering 18 acceptance criteria):**

#### Feature 1: `/sysconfig` Command - System Configuration Summary (P0) - 5 test documents
- **TC-038-001**: SysconfigInfo SQL Constants, Struct Parsing, and Formatting Unit Tests (AC-1, AC-2, AC-3, AC-8, AC-9)
  - SQL validates DBC.DBCInfoV and HASHAMP()+1
  - Struct parsing from mock rows (valid, insufficient columns, NULLs)
  - Table/CSV/JSON formatter tests
  - Privilege error message validation
  - REPL summary content (AMP count, version) (12 unit tests)

- **TC-038-002**: Sysconfig Batch Mode CLI Integration Tests (AC-4)
  - CLI wiring validation (3 no-DB tests)
  - Live-DB format flag tests - table/csv/json (2 `#[ignore]` tests)

- **TC-038-003**: Sysconfig REPL Tab Completion and Help Text (AC-5, AC-6)
  - Tab completion includes `/sysconfig` (3 `#[ignore]` interactive tests)

- **TC-038-004**: Sysconfig REPL Command Execution and Alias (AC-1, AC-2, AC-3, AC-9)
  - `/sysconfig` executes and shows AMP count + version
  - `/sc` alias works (3 `#[ignore]` interactive tests)

- **TC-038-005**: Sysconfig Error Handling (AC-7)
  - Privilege error detection and message generation
  - Actionable guidance in error messages (4 unit + 1 `#[ignore]` interactive)

#### Feature 2: `/locks` Command - Session Blocking & Lock Information (P0) - 5 test documents
- **TC-038-006**: LockInfo SQL, Parsing, Lock Type Mapping Unit Tests (AC-1, AC-2, AC-3, AC-8, AC-9)
  - SQL validates DBC.LockInfoV
  - Struct parsing for all lock types (READ, WRITE, EXCLUSIVE, SHARE)
  - Lock type mapping: RD→READ, WR→WRITE, EX→EXCLUSIVE, SR→SHARE
  - Empty lock list message
  - Table/CSV/JSON formatter tests
  - `/lk` alias validation (15 unit tests)

- **TC-038-007**: Locks Batch Mode CLI Integration Tests (AC-4)
  - CLI wiring validation (3 no-DB tests)
  - Live-DB tests handle both empty locks and data (2 `#[ignore]` tests)

- **TC-038-008**: Locks REPL Tab Completion and Help Text (AC-5, AC-6)
  - Tab completion includes `/locks` and `/lk` (3 `#[ignore]` interactive tests)

- **TC-038-009**: Locks REPL Command Execution and Alias (AC-1, AC-2, AC-3, AC-9)
  - `/locks` executes without hang (no-locks case expected in CI)
  - `/lk` alias works (3 `#[ignore]` interactive tests)

- **TC-038-010**: Locks Error Handling (AC-7)
  - Privilege error detection, message generation
  - View-not-found error handling (DBC.LockInfoV availability)
  - Actionable guidance (5 unit + 1 `#[ignore]` interactive)

#### Test Summary
- **TC-038-SUMMARY**: Sprint 38 test execution plan and coverage matrix

**Sprint 38 Test Strategy:**
- **Test Count**: 10 test case documents + 1 summary document
- **Estimated Test Functions**: 60 new automated tests + ~721 regression tests = ~781 total
- **Type**: Feature Sprint (PMON Foundation - DBA Monitoring)
- **Database Required**: Yes (for interactive tests - 14/60 tests, and live-DB integration - 4/60 tests)
- **Test Types**: Unit (36 tests) + Integration CLI (6 no-DB + 4 live-DB) + Interactive (14 tests)
- **Critical Success**: 100% test pass rate (~781/~781) + zero regressions + all 18 ACs satisfied
- **Feature 1 Focus**: `/sysconfig` - AMP count, version, topology display; follows sessions.rs pattern
- **Feature 2 Focus**: `/locks` - Lock type mapping (RD/WR/EX/SR), empty lock state handling, blocking chain
- **No New Infrastructure**: All testing tools already available (expectrl, Value::*, DatabaseClient::mock())
- **Acceptance**: All automated tests pass + all 18 ACs covered + zero regressions

---

### Sprint 37: External Editor Integration

**Sprint 37 Context:** Implement `/edit` command to open last SQL query in external editor ($EDITOR/$VISUAL), completing query editing feature set alongside `/repeat` (Sprint 36). Also add optional live-DB test for `/show indexes` from Sprint 36.

**Sprint 37 Test Cases (7 test documents covering 15 acceptance criteria):**

#### Feature 1: `/edit` Command - External Editor Integration (P0) - 6 test documents
- **TC-037-001**: Editor Resolution and Temp File Creation (AC-1, AC-4, AC-9)
  - Unit tests for editor resolution logic ($VISUAL → $EDITOR → vi)
  - Unit tests for temp file creation with `.sql` extension
  - Command parsing tests for `/edit` and `\e` alias (8 unit tests)

- **TC-037-002**: Edit Modified Content Execution (AC-2, AC-10)
  - Integration tests with mock editor (modified content auto-executes)
  - Interactive tests validating `/repeat` after `/edit` (2 integration + 2 interactive)

- **TC-037-003**: Edit Without Changes Skips Execution (AC-3)
  - Unit tests for content comparison logic
  - Integration tests with mock editor (no changes = no execution)
  - Interactive tests for empty file handling (4 unit + 2 integration + 2 interactive)

- **TC-037-004**: Edit Tab Completion and Help Text (AC-5, AC-6)
  - Interactive tests for tab completion (includes `/edit` and `\e`)
  - Help text validation (`/help` includes `/edit` description) (9 interactive)

- **TC-037-005**: Edit Error Handling (AC-7, AC-8)
  - Unit tests for error messages (no previous query, no editor available)
  - Interactive tests for graceful error handling (3 unit + 5 interactive)

- **TC-037-006**: Edit Full REPL Mode Only (AC-11)
  - Integration tests for mode detection (works in full REPL, not quick REPL)
  - Interactive tests validating consistency with `/repeat` (3 integration + 3 interactive)

#### Feature 2: `/show indexes` Live-DB Test (P1) - 1 test document
- **TC-037-007**: Show Indexes Live Database Test (AC-14, AC-15)
  - Integration tests with real Teradata connection (#[ignore])
  - Output format validation (IndexName, IndexType, ColumnName, ColumnPosition) (4 integration #[ignore])

#### Test Summary
- **TC-037-SUMMARY**: Sprint 37 test execution plan and coverage matrix

**Sprint 37 Test Strategy:**
- **Test Count**: 7 test case documents + 1 summary document
- **Estimated Test Functions**: 47 new automated tests + 674 regression tests = 721 total
- **Type**: Feature Sprint (External Editor Integration)
- **Database Required**: Yes (for interactive tests - 21/47 tests)
- **Test Types**: Unit (15 tests) + Integration (11 tests, 7 mock + 4 live-DB #[ignore]) + Interactive (21 tests)
- **Mock Editor Approach**: 4 bash scripts in `tests/fixtures/mock_editors/` enable automated testing without real editor interaction
- **Critical Success**: 100% test pass rate (721/721) + zero regressions + all 15 ACs satisfied
- **Feature 1 Focus**: `/edit` command - external editor workflow, error handling, REPL integration
- **Feature 2 Focus**: Optional live-DB validation for Sprint 36's `/show indexes`
- **Manual Validation**: Real editor compatibility checklist (vim, nano, VS Code) recommended but not required
- **Acceptance**: All automated tests pass + mock editors functional + manual validation documented + zero regressions

### Sprint 39: PMON Hardening & Query Inspection

**Sprint 39 Context:** Monitoring utilities extraction (shared `monitoring_utils.rs`), Sprint 38 bug fixes (CSV no-waiter, error handling tests), and new `/query-inspect` command showing SQL text from DBC.QryLogV.

- **TC-039-001**: Monitoring Utils Shared Module Unit Tests
- **TC-039-002**: Sprint 38 Bug Fixes - CSV Output and Error Handling
- **TC-039-003**: QueryInspectInfo SQL, Parsing, Truncation, Error Unit Tests
- **TC-039-004**: Query Inspect Batch Mode CLI Integration Tests
- **TC-039-005**: Query Inspect REPL Tab Completion and Help
- **TC-039-006**: Query Inspect REPL Command Execution and Alias

---

### Sprint 41: GitHub Releases & Binary Distribution

**Sprint 41 Context:** DevOps/CI sprint delivering GitHub Actions release workflow, cross-compilation build.rs fix, POSIX install script, and Sprint 40 code quality remediation. Test strategy differs from feature sprints: locally-executable tests focus on build verification, regression, code inspection, and static analysis. GitHub Actions runtime execution and end-to-end install are NOT locally testable.

**Sprint 41 Test Cases (5 test case documents):**

- **TC-041-001**: Build Verification and Regression Suite (AC-14, AC-15, AC-26) - `cargo build` + `cargo test` 855+ tests pass
- **TC-041-002**: Build.rs Cross-Compilation Code Inspection (AC-12, AC-13) - CARGO_CFG_TARGET_OS/ARCH usage verification
- **TC-041-003**: Release Workflow YAML Structural Validation (AC-1 to AC-11) - actionlint + structure review (runtime deferred)
- **TC-041-004**: Install Script Static Analysis and Structure Review (AC-16 to AC-22) - shellcheck (BLOCKED) + sh -n + review
- **TC-041-005**: Sprint 40 Remediation Verification (AC-23 to AC-25) - /p alias, execute deduplication, LazyLock

**Sprint 41 Test Strategy:**
- **Test Count**: 5 test case documents covering 26 acceptance criteria
- **Type**: DevOps/CI Sprint
- **Database Required**: NO (all locally-executable tests are no-DB)
- **Test Types**: Build verification (2 commands) + Code inspection (13 checks) + Static analysis (shellcheck/actionlint)
- **Locally Testable ACs**: AC-12 to AC-16, AC-19 to AC-26 (build.rs, remediation, install script structure)
- **NOT Locally Testable ACs**: AC-1 to AC-11 (workflow runtime), AC-17/AC-18 (download/checksum)
- **Blocker**: shellcheck not installed (AC-21 test BLOCKED); actionlint not installed (AC workflow syntax limited)
- **Acceptance**: cargo build passes + 855+ tests pass + code inspection clean + install script syntax valid

---

### Sprint 40: Variable Substitution

**Sprint 40 Context:** YAML-based variable substitution engine for SQL templates using `{{variable}}` markers, `{{$ENV.VAR_NAME}}` for environment variables, multi-file parameter merging, CLI `--params`/`-p` flag, and REPL `/params` metacommand. Also Sprint 39 remediation: remove 31 redundant utility tests from consumer modules.

**Sprint 40 Test Cases (4 test case documents):**

#### Feature 1: Variable Substitution Engine (P0) - 3 test documents
- **TC-040-001**: Variable Substitution Engine - Unit Tests (AC-2, AC-3, AC-4, AC-5, AC-8, AC-11)
  - YAML parsing: flat, nested 2-level, nested 3-level, type coercion, empty, invalid, special chars (9 tests)
  - Variable resolution: simple, nested path, multiple markers, same variable twice, env var, missing env var, undefined var error with name/available list, passthrough (no markers) (11 tests)
  - Multi-file merge: non-overlapping, override (later wins), three-file priority, nested override (4 tests)
  - Edge cases: empty SQL, single `{`, unclosed `{{`, value with curly braces, YAML null value, `$env` case sensitivity, markers at boundaries (7 tests)
  - **Total: 30 unit tests**

- **TC-040-002**: Variable Substitution - CLI Batch Integration Tests (AC-1, AC-6, AC-8, AC-9)
  - Flag acceptance: `--params` long flag, `-p` short flag (2 no-DB tests)
  - Error cases: file not found, invalid YAML, undefined variable, multiple -p flags (4 no-DB tests)
  - Help: `tq help params` topic exists and contains syntax (1 no-DB test)
  - Passthrough and nested path substitution via CLI (2 no-DB tests)
  - Live-DB: inline SQL, file SQL, stdin SQL with substitution (3 `#[ignore]` tests)
  - **Total: 12 integration tests (9 no-DB + 3 `#[ignore]`)**

- **TC-040-003**: Variable Substitution - REPL Metacommand Interactive Tests (AC-7, AC-10)
  - Tab completion shows `/params` (1 test)
  - `/params load` confirmation (1 test)
  - `/params show` displays variables (1 test)
  - Parameters used in subsequent query (1 test)
  - `/params unload` clears state (1 test)
  - `/params load` non-existent file - error without REPL crash (1 test)
  - **Total: 6 interactive tests (all `#[ignore]`, all require live database)**

#### Feature 2: Sprint 39 Remediation - Redundant Test Removal (P0) - 1 test document
- **TC-040-004**: Sprint 39 Remediation - Redundant Test Removal (AC-13)
  - Identifies 31 redundant tests across sessions.rs (9), sysconfig.rs (11), locks.rs (7), sample.rs (4)
  - Validates removal via regression suite + clippy check
  - Verifies monitoring_utils.rs still provides equivalent coverage (27 authoritative tests)

**Sprint 40 Test Strategy:**
- **Test Count**: 4 test case documents
- **New Tests Added**: 48 tests (30 unit + 12 integration + 6 interactive)
- **Tests Removed (redundant)**: 31 tests from consumer modules
- **Net Change**: +17 tests from Sprint 39 baseline (~790 → ~807)
- **Type**: Feature Sprint (Variable Substitution) + Remediation Sprint
- **Database Required**: Yes (for 6 interactive + 3 live-DB integration tests; all marked `#[ignore]`)
- **Test Types**: Unit (30) + Integration CLI (9 no-DB + 3 live-DB) + Interactive REPL (6) + Regression (after removal)
- **Fixture Files Required**: `tests/fixtures/params/` directory with 6 YAML fixture files
- **Critical Success**: 100% test pass rate + ~807 tests total + all 11 ACs (AC-1 to AC-11) + remediation complete
- **No New Testing Tools Required**: serde_yaml sufficient for unit tests; existing expectrl for interactive tests
- **Acceptance**: All automated tests pass + all AC covered + 31 redundant tests removed + zero regressions

---

### Security
- **TC022**: Security - No Password Exposure
- **TC-SECURITY-001**: Password File Permission Enforcement - 0644 Rejected (Sprint 17)
- **TC-SECURITY-002**: Config File Permission Warning - 0644 Allowed (Sprint 17)
- **TC-SECURITY-003**: Security Check Ordering - Permission Check Before File Read (Sprint 17)

## Test Priority Matrix

### Critical Priority (Must Pass for Release)
| Test ID | Feature | Category |
|---------|---------|----------|
| TC001 | Ping - Basic | Functionality |
| TC002 | Ping - Failure | Error-Handling |
| TC003 | Query - Table Output | Functionality |
| TC004 | Query - JSON Output | Functionality |
| TC005 | Query - CSV Output | Functionality |
| TC006 | Connection String - Valid | Functionality |
| TC008 | Authentication | Functionality |
| TC009 | Password Files | Functionality |
| TC022 | Password Security | Security |
| TC026 | Table Completion - FROM | Functionality |
| TC027 | Table Completion - JOIN | Functionality |
| TC028 | Table Completion - UPDATE | Functionality |
| TC037 | /logon - Connection Switch | Functionality |
| TC038 | /logon - Failure Handling | Error-Handling |
| TC044 | Table Formatting - 5 Columns | Functionality (Sprint 8) |
| TC045 | Table Formatting - 16+ Columns | Functionality (Sprint 8) |
| TC049 | Tab Completion - FROM | Functionality (Sprint 8) |
| TC050 | Tab Completion - Database.Table | Functionality (Sprint 8) |
| TC052 | Tab Completion - Error Handling | Error-Handling (Sprint 8) |
| TC057 | Paging - j/k Keys | Functionality (Sprint 8) |
| TC062 | Paging - Exit with q/Esc | Functionality (Sprint 8) |
| TC-HELP-001 | Help Config Subcommand | Functionality (Sprint 17) |
| TC-HELP-002 | Help Credentials Subcommand | Functionality (Sprint 17) |
| TC-SECURITY-001 | Password File 0644 Rejected | Security (Sprint 17) |
| TC-SECURITY-003 | Security Check Ordering | Security (Sprint 17) |
| TC077 | Output to File - Table Format | Functionality (Sprint 23) |
| TC078 | Output to File - CSV Format | Functionality (Sprint 23) |
| TC079 | Output to File - JSON Format | Functionality (Sprint 23) |
| TC080 | Atomic File Writing | Functionality (Sprint 23) |
| TC086 | Transaction Control - Basic Success | Functionality (Sprint 23) |
| TC087 | Transaction Rollback on Error | Functionality (Sprint 23) |

### High Priority (Important Features)
| Test ID | Feature | Category |
|---------|---------|----------|
| TC007 | Connection String - Invalid | Error-Handling |
| TC010 | stdin Input | Functionality |
| TC011 | File Input | Functionality |
| TC012 | File Output | Functionality |
| TC013 | Help/Version | Usability |
| TC014 | Exit Codes | Integration |
| TC016 | Type Preservation | Functionality |
| TC019 | Environment Variables | Integration |
| TC021 | SQL Errors | Error-Handling |
| TC023 | CSV Escaping | Integration |
| TC029 | Table Metadata Errors | Error-Handling |
| TC030 | Table Cache Invalidation | Functionality |
| TC031 | Column Completion - SELECT | Functionality |
| TC032 | Column Completion - JOIN | Functionality |
| TC033 | Column Completion - ORDER BY | Functionality |
| TC034 | Column Metadata Errors | Error-Handling |
| TC036 | /logon - Show Connection | Functionality |
| TC039 | /logon - State Preservation | Functionality |
| TC042 | Table Completion Performance | Integration |
| TC043 | Column Completion Performance | Integration |
| TC046 | Table Formatting - NULLs | Functionality (Sprint 8) |
| TC048 | Table Formatting - Mixed Types | Functionality (Sprint 8) |
| TC051 | Tab Completion - Loading Indicator | Usability (Sprint 8) |
| TC053 | Tab Completion - CREATE TABLE Cache | Functionality (Sprint 8) |
| TC054 | Tab Completion - DROP TABLE Cache | Functionality (Sprint 8) |
| TC058 | Paging - PageUp/PageDown | Functionality (Sprint 8) |
| TC059 | Paging - h/l Keys | Functionality (Sprint 8) |
| TC061 | Paging - Position Indicator | Usability (Sprint 8) |
| TC063 | Paging - /pager on/off | Functionality (Sprint 8) |
| TC064 | LIMIT Hint - Correct Syntax | Usability (Sprint 8) |
| TC-HELP-003 | Help Unknown Topic Error | Error-Handling (Sprint 17) |
| TC-PROFILES-001 | List Profiles | Functionality (Sprint 17) |
| TC-PROFILES-002 | No Config File Error | Error-Handling (Sprint 17) |
| TC-PROFILES-003 | No Profiles Error | Error-Handling (Sprint 17) |
| TC-SECURITY-002 | Config File 0644 Warning | Security (Sprint 17) |
| TC-LOGO-001 | Logo Display - Lowercase "tq" | Functionality (Sprint 18) |
| TC-COMPLETION-001 | Database Completion After FROM | Functionality (Sprint 18) |
| TC-COMPLETION-002 | Table Completion After FROM | Functionality (Sprint 18) |
| TC-COMPLETION-003 | Column Completion in SELECT/WHERE | Functionality (Sprint 18) |
| TC-COMPLETION-004 | Qualified Name Completion | Functionality (Sprint 18) |
| TC-COMPLETION-005 | NO Keyword Completion | Functionality (Sprint 18) |
| TC081 | File Output Error - Permission Denied | Error-Handling (Sprint 23) |
| TC082 | File Output Error - Invalid Path | Error-Handling (Sprint 23) |
| TC088 | Transaction Status Messages | Usability (Sprint 23) |
| TC089 | Nested Transaction Detection | Error-Handling (Sprint 23) |
| TC092 | File Output + Atomic Transaction | Integration (Sprint 23) |
| TC093 | Transaction with Output Formats | Integration (Sprint 23) |

### Medium Priority (Quality of Life)
| Test ID | Feature | Category |
|---------|---------|----------|
| TC015 | NULL Handling | Functionality |
| TC017 | Verbose/Quiet | Usability |
| TC018 | Color Control | Usability |
| TC020 | Large Results | Functionality |
| TC024 | Multiple Pings | Functionality |
| TC025 | Query Timing | Functionality |
| TC035 | Column Cache Management | Functionality |
| TC040 | /logon - Auth Mechanisms | Functionality |
| TC041 | /logon - Performance | Functionality |
| TC047 | Table Formatting - Long Values | Functionality (Sprint 8) |
| TC055 | Tab Completion - Alias Context | Functionality (Sprint 8) |
| TC056 | Tab Completion - Multiple Databases | Functionality (Sprint 8) |
| TC060 | Paging - Arrow Keys | Functionality (Sprint 8) |
| TC065 | LIMIT Hint - Help Text | Usability (Sprint 8) |
| TC083 | File Overwrite - Existing File | Functionality (Sprint 23) |
| TC084 | Large Result Sets - Streaming | Functionality (Sprint 23) |
| TC085 | Empty Result Set to File | Functionality (Sprint 23) |
| TC090 | Single Statement - No Transaction | Functionality (Sprint 23) |
| TC091 | Large Transaction - Many Statements | Functionality (Sprint 23) |

## Feature Coverage Matrix

### Functional Requirements Coverage

| FR ID | Requirement | Test Cases |
|-------|-------------|------------|
| FR-001 | Execute single SQL query | TC003, TC004, TC005, TC010, TC011, TC012, TC015, TC016, TC020, TC021, TC025 |
| FR-002 | Ping database connectivity | TC001, TC002, TC024 |
| FR-003 | Multiple output formats | TC003, TC004, TC005, TC015, TC016, TC023 |
| FR-004 | TD2 authentication | TC008 |
| FR-005 | LDAP authentication | TC008 |
| FR-006 | Kerberos authentication | TC008 |
| FR-007 | Connection string parsing | TC006, TC007 |
| FR-008 | TQ_LOGON environment variable | TC019 |
| FR-009 | Password file support | TC009 |
| FR-010 | Secure credential handling | TC009, TC022 |
| FR-116 | Table name tab completion | TC026, TC027, TC028, TC029, TC030, TC042 |
| FR-117 | Column name tab completion | TC031, TC032, TC033, TC034, TC035, TC043 |
| FR-118 | /logon metacommand | TC036, TC037, TC038, TC039, TC040, TC041 |
| FR-119 | Batch mode file output | TC077, TC078, TC079, TC080, TC081, TC082, TC083, TC084, TC085 |
| FR-120 | Batch mode transaction control | TC086, TC087, TC088, TC089, TC090, TC091 |

### Specifications Coverage

| Section | Topic | Test Cases |
|---------|-------|------------|
| 3.1 | Core Features (MVP) | All test cases |
| 4.3 | Global Options | TC013, TC017, TC018, TC019 |
| 4.4.1 | Ping Command | TC001, TC002, TC024 |
| 4.4.2 | Query Command | TC003-TC012, TC015, TC016, TC020, TC021, TC025 |
| 4.5.3 | Exit Code Standards | TC014 |
| 8 | Output Format Specifications | TC003, TC004, TC005, TC015, TC016, TC023 |
| 9 | Error Handling | TC002, TC007, TC021 |
| 10 | Security Requirements | TC009, TC022 |
| Appendix A | CLI Design Checklist | TC013, TC014, TC018 |
| 5.6.2 | Table Name Completion | TC026, TC027, TC028, TC029, TC030, TC042 |
| 5.6.3 | Column Name Completion | TC031, TC032, TC033, TC034, TC035, TC043 |
| 5.8.1 | /logon Metacommand | TC036, TC037, TC038, TC039, TC040, TC041 |
| batch-mode.md §4 | Output Destinations (--output flag) | TC077, TC078, TC079, TC080, TC081, TC082, TC083, TC084, TC085 |
| batch-mode.md §8 | Transaction Control (--atomic flag) | TC086, TC087, TC088, TC089, TC090, TC091 |
| batch-mode.md | Integration (File Output + Transactions) | TC092, TC093 |

## Test Execution Guidelines

### Prerequisites for All Tests
1. tq binary built and available (`cargo build --release`)
2. Test Teradata database accessible (or mock for some tests)
3. Valid test credentials configured in `.env` file (recommended) or via environment variables
4. Required tools installed: jq (for JSON tests), ps (for security tests)

### Test Execution Order
**Recommended order for Sprint 7 validation:**

1. **Smoke Tests** (verify basic functionality):
   - TC001: Basic ping
   - TC003: Basic query
   - TC013: Help/version

2. **Core Functionality** (existing features):
   - TC004, TC005: Output formats
   - TC006: Connection string parsing
   - TC008: Authentication mechanisms
   - TC009: Password files

3. **Sprint 7 - Table Completion**:
   - TC026: Table completion - FROM
   - TC027: Table completion - JOIN
   - TC028: Table completion - UPDATE
   - TC029: Table metadata errors
   - TC030: Table cache invalidation

4. **Sprint 7 - Column Completion**:
   - TC031: Column completion - SELECT/WHERE
   - TC032: Column completion - JOIN queries
   - TC033: Column completion - ORDER BY/GROUP BY
   - TC034: Column metadata errors
   - TC035: Column cache management

5. **Sprint 7 - /logon Metacommand**:
   - TC036: Show current connection
   - TC037: Successful connection switch
   - TC038: Connection failure handling
   - TC039: State preservation
   - TC040: Authentication mechanisms
   - TC041: Performance and timeout

6. **Sprint 7 - Performance Validation**:
   - TC042: Table completion performance
   - TC043: Column completion performance

7. **Error Handling**:
   - TC002: Connection failures
   - TC007: Invalid connection strings
   - TC021: SQL errors
   - TC029, TC034, TC038: Sprint 7 error handling

8. **Integration**:
   - TC010, TC011, TC012: Input/output methods
   - TC014: Exit codes
   - TC019: Environment variables

9. **Security**:
   - TC022: Password exposure

10. **Quality**:
    - TC015, TC016: Data type handling
    - TC017, TC018: Output control
    - TC020: Large results
    - TC023: CSV compliance
    - TC024, TC025: Advanced features

### Environment Setup
```bash
# Build the binary
cargo build --release

# Set up .env file with test credentials (recommended approach)
cp .env.example .env
# Edit .env to set: TQ_LOGON=testuser:testpass@testhost:1025/testdb

# Alternative: Set test credentials via environment variable
# export TQ_LOGON="testuser:testpass@testhost:1025/testdb"

# Optional: Set log level for debugging
# export RUST_LOG=debug

# Make binary easily accessible
export PATH="$PWD/target/release:$PATH"
```

**Note**: The `.env` file approach is recommended for development and testing as it:
- Keeps credentials in a secure file (not shell history)
- Automatically loads on each tq command
- Is already excluded from git via .gitignore
- Avoids exposing credentials in process listings

### Test Execution Template
```bash
# For each test case:
# 1. Read the test case markdown file
# 2. Follow the test procedure step by step
# 3. Compare actual results with expected results
# 4. Document pass/fail in the "Actual Results" section
# 5. Note any deviations or issues
```

## Test Results Tracking

Create a test results summary file after execution:

```markdown
# Test Results Summary - [Date]

## Environment
- OS: [Linux/macOS/Windows]
- tq version: [version]
- Commit: [commit hash]
- Teradata version: [version]

## Results
| Test ID | Status | Notes |
|---------|--------|-------|
| TC001   | PASS   |       |
| TC002   | PASS   |       |
| ...     | ...    | ...   |

## Issues Found
1. [Issue description]
2. [Issue description]

## Overall Assessment
- Pass: X/25
- Fail: Y/25
- Skip: Z/25
```

## Known Limitations

### Test Environment Dependencies
- Some tests require actual Teradata connectivity (can't be fully mocked)
- Security tests (TC022) may behave differently on Windows
- Large result set tests (TC020) depend on available test data

### Platform-Specific Considerations
- **Linux**: All tests should work
- **macOS**: All tests should work
- **Windows**: File permission tests may need adjustment

### Test Data Requirements
Tests may need adjustment based on available test database:
- TC020: Requires table with substantial data
- TC021: Requires appropriate permissions for various SQL errors

## Future Test Cases

Additional test cases to consider for future releases:

### REPL Mode (Phase 2)
- Interactive prompt
- Multi-line input
- Command history
- Tab completion
- Metacommands

### Batch Mode (Phase 3)
- Multiple statement execution
- Transaction control
- Variable substitution

### Configuration (Phase 4)
- Configuration file loading
- Connection profiles
- Keyring integration

## Contributing Test Cases

When adding new test cases:

1. **Naming**: Use sequential numbering (TC026, TC027, etc.)
2. **Format**: Follow the established template
3. **Metadata**: Include all required fields
4. **Coverage**: Reference specific FR or section
5. **Priority**: Assign appropriate priority
6. **Index**: Update this INDEX.md file

### Test Case Template
See any existing TC file for the complete template structure.

## References

- Specifications: `docs/builder/specifications.md`
- CLI Design Guide: `docs/builder/rust-cli-design-general.md`
- Rust Architecture: `docs/builder/rust-architecture.md`
- Project Overview: `CLAUDE.md`
- README: `Readme.md`

---

**Note**: This is a living document. Update as test cases are added, modified, or executed.
