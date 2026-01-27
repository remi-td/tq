# Sprint 26 Test Strategy - Sessions Command

**Created:** 2026-01-27
**Author:** quality-validator
**Sprint:** Sprint 26
**Features:** `/sessions` metacommand (REPL) and `tq sessions` batch mode

---

## Feature-by-Feature Test Strategy

### Feature: Sessions Command (`/sessions`)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/repl.md` sections REQ-SESS-001 through REQ-SESS-008
- Design: `docs/design/repl.md` Sessions Command section
- Planning: `docs/sprints/sprint-26-planning.md` Acceptance Criteria AC-1 through AC-10

**Requirements:**
1. REQ-SESS-001: "/sessions command available with /s alias" (repl.md lines 1545-1554)
2. REQ-SESS-002: "Data from MonitorSession(-1,'*',0)" (repl.md lines 1556-1579)
3. REQ-SESS-003: "Output formatted as table with 10 columns" (repl.md lines 1581-1593)
4. REQ-SESS-004: "NULL skew for IDLE sessions shown as [--]" (repl.md lines 1595-1605)
5. REQ-SESS-005: "Error handling for privilege errors, connection errors, empty results" (repl.md lines 1607-1671)
6. REQ-SESS-006: "Tab completion suggests /sessions" (repl.md lines 1673-1682)
7. REQ-SESS-007: "Works with CSV, JSON, table formats" (repl.md lines 1684-1725)
8. REQ-SESS-008: "Query execution <1s, loading indicator if >500ms" (repl.md lines 1727-1735)
9. AC-1: "/sessions command available in REPL with /s alias"
10. AC-2: "tq sessions works in batch mode (no SQL file required)"
11. AC-3: "Output displays 10 columns: SessionNo, UserName, LogonTime, PEstate, AMPState, AMPCPUSec, AMPIO, ReqSpool, Amp CPU Skew %, Amp IO Skew %"
12. AC-4: "Skew percentages calculated correctly (NULL for inactive sessions)"
13. AC-5: "Logon times formatted as YYYY/MM/DD HH:MM:SS.ss"
14. AC-6: "Tab completion suggests /sessions command"
15. AC-7: "/help output includes /sessions command description"
16. AC-8: "Error handling for insufficient privileges (DBC access required)"
17. AC-9: "Handles empty result set (no active sessions besides current)"
18. AC-10: "Works with all output formats (--format csv, json, table)"

**Feature Characteristics:**

**User Interaction Type:**
- [x] Interactive PTY (REPL, terminal UI with cursor/colors/rendering)
- [x] CLI Batch (scripted, piped, non-interactive command execution)
- [ ] Web UI (browser-based interface)
- [ ] API (programmatic interface, library usage)
- [ ] Background Process (daemon, service, scheduled task)
- [x] Pure Logic (internal algorithm, no user interaction)

**Explanation:**
This feature has THREE interaction modes:
1. **Interactive PTY**: `/sessions` in REPL requires tab completion, help text integration, terminal table rendering
2. **CLI Batch**: `tq sessions` as standalone command with --format flags
3. **Pure Logic**: Skew calculation algorithm, LogonTime formatting, NULL handling

**Observable Behavior:**
- [x] Visual output in terminal (colors, formatting, layout, cursor position)
- [x] Structured data output (JSON, CSV, XML)
- [ ] File system side effects (files created/modified/deleted)
- [x] Database side effects (records inserted/updated/deleted) - READ ONLY
- [ ] Network interactions (HTTP requests, socket connections)
- [x] Performance characteristics (speed, memory usage, latency) - Query <1s requirement
- [ ] State management (session state, cache, persistence)

**External Dependencies:**
- [x] Database connection (requires live database)
- [ ] File system access (reads/writes files)
- [ ] Network access (API calls, downloads)
- [x] Terminal/PTY (terminal control sequences, cursor positioning)
- [ ] System clipboard (copy/paste operations)
- [ ] Operating system specific features (Windows vs Linux vs macOS)
- [ ] None (pure logic, no external dependencies)

**Validation Challenges:**
1. **Skew calculation logic**: Requires understanding Teradata MonitorSession output format and validating NULL handling
2. **Privilege error simulation**: Hard to test permission denied without revoking actual database privileges
3. **Empty result set**: Difficult to simulate "no sessions" on live database (current session always present)
4. **Terminal rendering**: Table formatting varies by terminal width, PTY emulation may not match real terminal
5. **CSV/JSON format validation**: Need to verify all NULL skew values are properly serialized
6. **Tab completion integration**: Requires PTY simulation to verify /sessions appears in completion menu
7. **Performance validation**: >500ms query timing is non-deterministic, hard to test loading indicator

**Critical Behaviors to Validate:**
1. "NULL skew percentages (for IDLE sessions): Display as `[--]` (not `[NULL]` or blank)" - REQ-SESS-004.1
2. "Skew percentage format: `X.XX` (two decimal places, no leading zeros)" - REQ-SESS-004.2
3. "Privilege errors SHALL include helpful explanation and GRANT statement example" - REQ-SESS-005.1
4. "Tab completion: Typing `/s<TAB>` SHALL suggest `/sessions` and `/sample`" - REQ-SESS-006.1
5. "CSV format: Standard CSV with headers, NULL skew as empty string" - REQ-SESS-007.2
6. "JSON format: Array of objects, NULL skew as `null`" - REQ-SESS-007.3
7. "LogonTime format: `YYYY/MM/DD HH:MM:SS.ss`" - REQ-SESS-003.3
8. "Skew calculation: `100 * (1 - (avg / hot))` when hot > 0, else NULL" - design/repl.md lines 1997-2002

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, tab completion, help text display

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: End-to-end CLI execution needs validation with real arguments/pipes

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Mocks don't catch SQL syntax errors, MonitorSession behavior, permission issues

IF "Pure Logic" (skew calculation, NULL handling):
  → Unit tests REQUIRED
  Reason: Skew algorithm needs validation with edge cases independent of database
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Skew calculation algorithm, NULL handling, LogonTime formatting, SessionInfo row parsing
- **Approach:** Test `calculate_skew()`, `format_logon_time()`, `SessionInfo::from_row()` with mock data
- **Rationale:** These are pure functions that should work correctly independent of database queries
- **Gap if missing:** Logic bugs in skew formula (e.g., division by zero, incorrect NULL handling), date format errors
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (Batch Mode)**
- **Validates:** `tq sessions` CLI command execution, --format flag handling, output to stdout/file
- **Approach:** Execute `tq sessions` subprocess with live database, capture output, validate structure
- **Rationale:** Validates batch mode works correctly with real database queries and output formatting
- **Gap if missing:** Batch mode integration bugs, CLI argument parsing errors, output piping issues
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (PTY/expectrl)**
- **Validates:** `/sessions` and `/s` in REPL, tab completion, help text integration, terminal table rendering
- **Approach:** Spawn tq REPL in PTY, send `/sessions`, verify table output, test tab completion
- **Rationale:** Only way to validate interactive REPL behavior and terminal rendering
- **Gap if missing:** REPL integration bugs, tab completion failures, terminal rendering issues
- **Necessity:** ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates:** Visual table quality, error message clarity, skew calculation accuracy with real data
- **Approach:** Human tester runs `/sessions` on live system, verifies output makes sense
- **Rationale:** Automated tests may miss visual formatting issues or unclear error messages
- **Gap if missing:** Usability issues, confusing output format, unhelpful error messages
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 5: Error Simulation Tests**
- **Validates:** Privilege error handling, connection error handling, empty result set handling
- **Approach:** Mock DatabaseClient to return permission errors, connection errors, empty results
- **Rationale:** Cannot reliably test error paths with live database without destructive operations
- **Gap if missing:** Error handling bugs, unclear error messages, crashes on edge cases
- **Necessity:** ✅ REQUIRED

**Test Type 6: Format Compatibility Tests**
- **Validates:** CSV output format, JSON output format, NULL serialization
- **Approach:** Execute with --format csv/json, parse output, validate structure and NULL handling
- **Rationale:** Each format has specific serialization requirements (CSV empty string vs JSON null)
- **Gap if missing:** Format-specific bugs, invalid JSON, malformed CSV
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates skew calculation, NULL handling, date formatting | Logic bugs, edge cases not caught | MUST IMPLEMENT |
| Integration tests (batch) | ✅ REQUIRED | Validates `tq sessions` CLI with live database | Batch mode bugs, CLI integration issues | MUST IMPLEMENT |
| Interactive tests (PTY) | ✅ REQUIRED | Validates `/sessions` in REPL, tab completion, help text | REPL integration bugs, terminal rendering issues | MUST IMPLEMENT |
| Error simulation tests | ✅ REQUIRED | Validates privilege errors, connection errors, empty results | Error handling bugs, unclear messages | MUST IMPLEMENT |
| Format compatibility tests | ✅ REQUIRED | Validates CSV, JSON, table output formats | Format-specific serialization bugs | MUST IMPLEMENT |
| Manual validation | ⚠️ RECOMMENDED | Human validates visual quality and usability | Usability issues, confusing UX | DOCUMENT TEST CASES |
| Performance tests | ❌ NOT NEEDED | No hard performance SLA beyond <1s guideline | Performance regression | DEFER (monitor in production) |

**Summary:**
- ✅ REQUIRED test types: 5 (unit, integration batch, PTY, error sim, format compat)
- ⚠️ RECOMMENDED test types: 1 (manual validation)
- ❌ NOT NEEDED test types: 1 (performance benchmarks)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| REQ-SESS-001.1 | "Primary command: `/sessions`" | repl.md:1549 | PTY | Only PTY can execute REPL metacommands | TC-SESS-001 |
| REQ-SESS-001.2 | "Short alias: `/s`" | repl.md:1550 | PTY | Only PTY can test alias execution | TC-SESS-009 |
| AC-2 | "`tq sessions` flag works in batch mode" | planning.md:107 | Integration | Batch mode requires subprocess execution | TC-SESS-002 |
| REQ-SESS-002.1 | "Data source: `MonitorSession(-1,'*',0)`" | repl.md:1560 | Integration + Unit | Integration validates query, unit validates row parsing | TC-SESS-001, TC-SESS-003 |
| AC-3 | "Output displays 10 columns" | planning.md:108 | Integration + PTY | Both modes must display all columns | TC-SESS-001, TC-SESS-002 |
| AC-4 | "Skew percentages calculated correctly" | planning.md:109 | Unit + Integration | Unit validates formula, integration validates real data | TC-SESS-003 |
| REQ-SESS-004.1 | "NULL skew: Display as `[--]`" | repl.md:1599 | Unit + Integration | Unit tests NULL handling, integration validates display | TC-SESS-003, TC-SESS-007 |
| REQ-SESS-004.2 | "Skew format: `X.XX`" | repl.md:1600 | Unit | Pure formatting logic | TC-SESS-003 |
| AC-5 | "Logon times formatted as `YYYY/MM/DD HH:MM:SS.ss`" | planning.md:110 | Unit + Integration | Unit tests format function, integration validates display | TC-SESS-003 |
| AC-6 | "Tab completion suggests `/sessions`" | planning.md:111 | PTY | Only PTY can test tab completion | TC-SESS-004 |
| AC-7 | "`/help` includes `/sessions`" | planning.md:112 | PTY | Only PTY can test help output | TC-SESS-005 |
| REQ-SESS-005.1 | "Privilege error with GRANT example" | repl.md:1611-1619 | Error Sim | Cannot reliably trigger privilege error on live DB | TC-SESS-006 |
| AC-9 | "Handles empty result set" | planning.md:114 | Error Sim | Cannot reliably create empty sessions on live DB | TC-SESS-007 |
| REQ-SESS-007.2 | "CSV format: NULL skew as empty string" | repl.md:1689-1693 | Format Compat | Format-specific serialization | TC-SESS-008 |
| REQ-SESS-007.3 | "JSON format: NULL skew as `null`" | repl.md:1695-1723 | Format Compat | Format-specific serialization | TC-SESS-008 |
| AC-10 | "Works with all output formats" | planning.md:115 | Format Compat | Need to test table, CSV, JSON modes | TC-SESS-008 |
| REQ-SESS-006.1 | "Tab completion: `/s<TAB>` suggests `/sessions`" | repl.md:1676 | PTY | Tab completion only in PTY | TC-SESS-004 |
| REQ-SESS-008.1 | "Target execution time: <1 second" | repl.md:1731 | Manual | Non-deterministic, varies by system load | Manual Test |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- **Performance validation (REQ-SESS-008)**: <1s execution time is non-deterministic and varies by database load. We will document expected timing in manual validation but not enforce in automated tests.
- **Loading indicator (REQ-SESS-008.2)**: >500ms delay is hard to test reliably. Will validate logic exists but not enforce timing.

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** No hard SLA defined beyond "<1 second for <1000 sessions" guideline
- **What won't be validated:** Query execution speed, memory usage, loading indicator timing
- **Risk assessment:** LOW
- **Mitigation:** Manual validation will observe query timing. Production monitoring will catch performance regressions.
- **Revisit criteria:** If users report slow queries or if performance requirements are added to specification

**Cross-platform Tests**
- **Reason for omission:** Feature is database-focused, not OS-specific
- **What won't be validated:** Platform-specific differences (Windows vs macOS vs Linux)
- **Risk assessment:** LOW
- **Mitigation:** CI runs on Linux. Feature uses portable Rust stdlib and reedline (cross-platform)
- **Revisit criteria:** If platform-specific bugs are reported

**Teradata Version Compatibility Tests**
- **Reason for omission:** Requires multiple Teradata instances (14.10+, 13.x, etc.)
- **What won't be validated:** Error message on Teradata <14.10 (MonitorSession not available)
- **Risk assessment:** MEDIUM
- **Mitigation:** Document Teradata 14.10+ requirement in error message and user documentation
- **Revisit criteria:** If users on older Teradata versions request feature

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/sessions.rs` test module (if implemented), or `src/commands/repl/metacommands.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8 tests
- **Key scenarios to cover:**
  1. `test_calculate_skew_active_session` - hot > 0, avg > 0 → returns percentage
  2. `test_calculate_skew_idle_session` - hot = 0 → returns None
  3. `test_calculate_skew_perfect_balance` - avg == hot → returns 0.0%
  4. `test_calculate_skew_extreme_skew` - avg << hot → returns ~100%
  5. `test_format_logon_time` - "2026-01-27 15:33:26.00" → "2026/01/27 15:33:26.00"
  6. `test_session_info_from_row_complete` - Full row with all fields → SessionInfo
  7. `test_session_info_from_row_idle` - IDLE session with NULL skew → SessionInfo with None skew
  8. `test_session_info_from_row_nulls` - NULL handling for optional fields
- **Mocking strategy:** Create mock `Value` arrays representing database rows. NO database connection needed.

**Test Type: Integration Tests (Batch Mode)**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. `test_sessions_batch_mode_table_format` - Execute `tq sessions`, verify table output structure
  2. `test_sessions_batch_mode_csv_format` - Execute `tq sessions --format csv`, parse CSV, verify columns
  3. `test_sessions_batch_mode_json_format` - Execute `tq sessions --format json`, parse JSON, verify schema
  4. `test_sessions_batch_mode_output_to_file` - Execute `tq sessions -o output.txt`, verify file written
- **Setup requirements:** Live database connection via TQ_LOGON. Mark tests with `#[ignore]`.

**Test Type: Interactive Tests (PTY)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `test_sessions_repl_command` - Send `/sessions`, verify table output appears
  2. `test_sessions_repl_alias` - Send `/s`, verify same output as `/sessions`
  3. `test_sessions_tab_completion` - Type `/s<TAB>`, verify `/sessions` appears in suggestions
  4. `test_sessions_help_integration` - Send `/help`, verify `/sessions` listed with description
  5. `test_sessions_repl_multi_format` - Use `/set format csv`, then `/sessions`, verify CSV output
- **Setup requirements:** Live database connection via TQ_LOGON. Mark tests with `#[ignore]`.
- **Implementation notes:** expectrl may have cursor position issues with reedline. Use `read_available_output()` helper and allow timing delays.

**Test Type: Error Simulation Tests**
- **Location:** `tests/integration_tests.rs` or `src/commands/sessions.rs` test module
- **Framework:** Built-in Rust test framework with mock DatabaseClient
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `test_sessions_privilege_error` - Mock DatabaseClient returns "permission denied" error → verify helpful message with GRANT example
  2. `test_sessions_connection_error` - Mock DatabaseClient returns "connection lost" error → verify reconnect suggestion
  3. `test_sessions_empty_result` - Mock DatabaseClient returns 0 rows → verify "0 sessions found" message
- **Mocking strategy:** Create trait for DatabaseClient, implement mock that returns errors/empty results

**Test Type: Format Compatibility Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `test_sessions_csv_null_handling` - Execute with IDLE session → verify NULL skew as empty CSV field
  2. `test_sessions_json_null_handling` - Execute with IDLE session → verify NULL skew as JSON `null`
  3. `test_sessions_table_null_handling` - Execute with IDLE session → verify NULL skew displayed as `[--]`
- **Setup requirements:** Live database with IDLE sessions. Mark tests with `#[ignore]`.

**Test Type: Manual Validation**
- **Location:** `tests/cases/TC-SESS-010.md` (manual test case)
- **Framework:** Human tester
- **Test count estimate:** 1 manual test case
- **Key scenarios to cover:**
  1. Visual table quality (alignment, readability, column widths)
  2. Skew calculation accuracy (compare to known DBA tool output)
  3. Error message clarity (does privilege error make sense to user?)
  4. Query timing (<1 second on typical system)

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Skew calculation logic, NULL handling, date formatting, row parsing
- Integration tests validate: Batch mode CLI execution, output formats, real database queries, file output
- PTY tests validate: REPL integration, tab completion, help text, alias execution
- Error simulation tests validate: Privilege errors, connection errors, empty results
- Format compatibility tests validate: CSV/JSON/table NULL serialization
- Manual tests validate: Visual quality, usability, real-world accuracy
- Combined coverage: COMPREHENSIVE

**Gaps in combined coverage:**
- Performance validation is manual-only (non-deterministic timing)
- Loading indicator >500ms timing not enforced (implementation verified, timing not)
- Teradata <14.10 compatibility error not tested (requires old Teradata instance)

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Performance gap acceptable because: No hard SLA defined, timing is non-deterministic, manual validation provides sanity check
- Loading indicator timing gap acceptable because: Implementation logic is unit-tested, >500ms timing is rare with modern databases
- Teradata version gap acceptable because: Error message documents requirement, test environment is Teradata 14.10+, version check is straightforward

---

## Strategy Summary

**Total Features Analyzed:** 1 (`/sessions` command)

**Test Types Required:**
- Unit tests: ✅ (skew calculation, date formatting, row parsing)
- Integration tests (batch): ✅ (CLI execution, output formats)
- Interactive tests (PTY): ✅ (REPL integration, tab completion, help)
- Error simulation tests: ✅ (privilege, connection, empty result errors)
- Format compatibility tests: ✅ (CSV, JSON, table NULL handling)
- Manual validation: ⚠️ (visual quality, usability)

**Estimated Test Count:**
- Unit: 8 tests
- Integration (batch): 4 tests
- Interactive (PTY): 5 tests
- Error simulation: 3 tests
- Format compatibility: 3 tests
- Manual: 1 test case
- **Total: 24 automated tests + 1 manual test**

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: Teradata version compatibility
- LOW risk gaps: Performance validation, loading indicator timing

**Dependencies Required:**
- Live database: Yes (integration, PTY, format tests marked with `#[ignore]`)
- Network access: Yes (database connection)
- Specific OS: No (portable Rust code)
- Other: Teradata 14.10+ for MonitorSession table function

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-27
**Review Status:** DRAFT
**Submitted for Review:** [Awaiting coordinator approval]

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** [Pending]
**Review Comments:** [Awaiting review]
