# Sprint 37 Test Strategy: External Editor Integration

**Created:** 2026-02-13
**Author:** quality-validator
**Sprint:** Sprint 37
**Features:** `/edit` Command (13 ACs), Optional Live-DB Test for `/show indexes` (2 ACs)

---

## Overview

Sprint 37 implements external editor integration for REPL query editing. The primary feature (`/edit` command) launches an external editor to edit the last SQL query, presenting unique testing challenges due to external process interaction. The secondary feature adds database-dependent test coverage for Sprint 36's `/show indexes`.

**Sprint Context:**
- P0: `/edit` command - External editor integration (13 acceptance criteria)
- P1: Optional live-DB test for `/show indexes` (2 acceptance criteria)
- **Total Acceptance Criteria: 15**
- **Test Challenge:** Editor launches external process - requires mock/stub approach
- **Test Complexity:** MEDIUM (external process interaction, temp file management)

---

## Feature-by-Feature Test Strategy

### Feature 1: `/edit` Command - External Editor Integration (13 ACs) - P0

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-37-planning.md` lines 36-58
- Secondary: `docs/specifications/repl.md` (Query Editing section, line 3180)
- Context: Complements `/repeat` (Sprint 36), uses existing `last_sql` field

**Requirements:**
1. "`/edit` opens last SQL query in temporary `.sql` file using $EDITOR (or $VISUAL, fallback to `vi`)" (AC-1, line 41)
2. "On save and exit, the edited SQL is executed automatically" (AC-2, line 42)
3. "On exit without changes (or empty file), no execution occurs" (AC-3, line 43)
4. "Alias `\e` works identically to `/edit`" (AC-4, line 44)
5. "Tab completion includes `/edit` and `\e` in metacommand menu" (AC-5, line 45)
6. "`/help` text includes `/edit` command description" (AC-6, line 46)
7. "Error handling: clear message when no previous query exists ('No previous query to edit')" (AC-7, line 47)
8. "Error handling: clear message when $EDITOR is not set and fallback `vi` not found" (AC-8, line 48)
9. "Temp file uses `.sql` extension for editor syntax highlighting" (AC-9, line 49)
10. "Edited query stored as `last_sql` (enabling `/repeat` after `/edit`)" (AC-10, line 50)
11. "Works in full REPL mode only (not quick REPL), matching `/repeat` behavior" (AC-11, line 51)
12. "Unit tests cover all paths (happy path, no previous query, empty edit, editor error)" (AC-12, line 52)
13. "Integration tests validate CLI behavior" (AC-13, line 53)

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL metacommand with external process)
**Explanation:** `/edit` is a REPL metacommand that launches an external editor process, waits for it to complete, then conditionally executes the edited SQL. This is interactive terminal behavior with external process coordination.

**Observable Behavior:**
- ✅ Visual output in terminal (confirmation messages, error messages, query execution results)
- ✅ File system side effects (creates temp `.sql` file, reads modified content)
- ✅ External process interaction (launches $EDITOR/$VISUAL/vi)
- ✅ Database side effects (executes edited query if modified)
- ✅ State management (reads/writes `ReplState.last_sql`)

**External Dependencies:**
- ✅ Database connection (executes edited queries)
- ✅ Terminal/PTY (REPL metacommand requires interactive session)
- ✅ File system access (creates/reads temp files)
- ✅ External editor process ($EDITOR/$VISUAL/vi)
- ✅ Environment variables ($EDITOR, $VISUAL)

**Validation Challenges:**
- **External editor interaction**: Cannot automate real editor - must mock/stub
- **Editor resolution logic**: Must test $VISUAL → $EDITOR → vi fallback chain
- **Temp file lifecycle**: Must verify file creation, content population, cleanup
- **Content comparison**: Must detect if file was modified (to decide execution)
- **Process exit codes**: Must handle editor failures gracefully
- **Environment variable precedence**: Must test VISUAL vs EDITOR priority

**Critical Behaviors to Validate:**
1. `/edit` reads `ReplState.last_sql` and writes to temp `.sql` file
2. Editor resolution follows precedence: $VISUAL → $EDITOR → `vi` (with clear error if none available)
3. Temp file created with `.sql` extension in system temp directory
4. Process launches editor with `Command::new(editor).arg(temp_path).status()`
5. Content comparison detects modifications (edited != original)
6. Modified content executed automatically (no user prompt)
7. Unmodified or empty content skips execution (with message)
8. Executed query updates `ReplState.last_sql` (enables `/repeat` after `/edit`)
9. Editor error (non-zero exit code) handled gracefully
10. Alias `\e` behaves identically to `/edit`
11. No previous query produces clear error message
12. Tab completion suggests `/edit` and `\e` with description
13. `/help` output includes `/edit` command

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: REPL metacommands require PTY simulation to test user experience

IF "External process interaction" checked:
  → Mock editor approach REQUIRED
  Reason: Cannot automate real editor (vim, nano, etc.) - must stub

IF "File system side effects" + "Process management" checked:
  → Unit tests REQUIRED
  Reason: Editor resolution, file I/O, content comparison must be tested in isolation

IF "Database connection" + "Query execution" checked:
  → Integration tests REQUIRED (with mock editor)
  Reason: End-to-end workflow with real DB validates SQL execution
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Editor resolution logic, temp file creation, content comparison, error messages
- **Approach:** Test individual functions (resolve_editor(), create_temp_file(), compare_content(), handle_edit_command()) with mocks
- **Rationale:** Core logic (editor resolution, file I/O, content comparison) must be validated independently without external processes
- **Gap if missing:** Logic errors in editor resolution, file handling, or content comparison not caught until integration
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (with Mock Editor)**
- **Validates:** Full `/edit` workflow with mock editor script
- **Approach:** Create test script that mimics editor behavior (modifies/doesn't modify file), test via CLI
- **Rationale:** End-to-end workflow must be validated but real editor (vim, nano) cannot be automated - mock editor script provides controllable testing
- **Gap if missing:** Integration issues between components, workflow bugs, process coordination errors
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (with Mock Editor)**
- **Validates:** REPL behavior, user-visible messages, tab completion, help text
- **Approach:** Spawn REPL with mock editor in $EDITOR, execute `/edit`, validate output
- **Rationale:** End-to-end REPL user experience must be validated in real PTY, mock editor enables automation
- **Gap if missing:** REPL integration bugs, output format issues, completion not working
- **Necessity:** ✅ REQUIRED

**Test Type 4: Manual Tests (with Real Editors)**
- **Validates:** Real editor compatibility (vim, nano, emacs, VS Code)
- **Approach:** Human manually tests `/edit` with various real editors
- **Rationale:** Mock editor cannot validate real editor compatibility, quirks, or UX
- **Gap if missing:** Real editor incompatibilities (exit codes, file handling) not discovered
- **Necessity:** ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates core logic (editor resolution, file I/O, comparison) | Logic bugs not caught | MUST IMPLEMENT |
| Integration tests (mock editor) | ✅ REQUIRED | Validates end-to-end workflow with controllable editor | Integration bugs not caught | MUST IMPLEMENT |
| Interactive tests (mock editor) | ✅ REQUIRED | Validates REPL experience with automated editor | REPL bugs, completion missing | MUST IMPLEMENT |
| Manual tests (real editors) | ⚠️ RECOMMENDED | Validates real editor compatibility | Real editor issues not caught | DOCUMENT CHECKLIST |

**Summary:**
- ✅ REQUIRED test types: 3 (Unit, Integration with mock, Interactive with mock)
- ⚠️ RECOMMENDED test types: 1 (Manual with real editors)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | Opens temp `.sql` file using $EDITOR/$VISUAL/vi fallback | sprint-37-planning.md line 41 | Unit + Integration | Unit tests resolution, integration tests launch | TC-037-001 |
| AC-2 | On save and exit, edited SQL executed automatically | sprint-37-planning.md line 42 | Integration + Interactive | Must test with mock editor + real REPL | TC-037-002 |
| AC-3 | On exit without changes, no execution occurs | sprint-37-planning.md line 43 | Unit + Integration | Unit tests comparison, integration tests behavior | TC-037-003 |
| AC-4 | Alias `\e` works identically to `/edit` | sprint-37-planning.md line 44 | Unit + Interactive | Unit tests alias parsing, interactive tests behavior | TC-037-001 |
| AC-5 | Tab completion includes `/edit` and `\e` | sprint-37-planning.md line 45 | Interactive | Must validate PTY completion behavior | TC-037-004 |
| AC-6 | `/help` includes `/edit` command | sprint-37-planning.md line 46 | Unit + Interactive | Unit tests help text, interactive tests display | TC-037-004 |
| AC-7 | Error when no previous query | sprint-37-planning.md line 47 | Unit + Interactive | Unit tests message, interactive tests display | TC-037-005 |
| AC-8 | Error when $EDITOR not set and vi not found | sprint-37-planning.md line 48 | Unit + Integration | Unit tests resolution failure, integration tests error handling | TC-037-005 |
| AC-9 | Temp file uses `.sql` extension | sprint-37-planning.md line 49 | Unit | Must verify file extension for syntax highlighting | TC-037-001 |
| AC-10 | Edited query stored as `last_sql` | sprint-37-planning.md line 50 | Unit + Integration | Unit tests state update, integration tests persistence | TC-037-002 |
| AC-11 | Works in full REPL mode only (not quick REPL) | sprint-37-planning.md line 51 | Integration + Interactive | Must test REPL mode restrictions | TC-037-006 |
| AC-12 | Unit tests cover all paths | sprint-37-planning.md line 52 | Meta-validation | Verify unit tests exist and pass | TC-037-001-005 |
| AC-13 | Integration tests validate CLI behavior | sprint-37-planning.md line 53 | Meta-validation | Verify integration tests exist and pass | TC-037-002-003 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Real Editor Automation (vim, nano, emacs, VS Code)**
- **Reason for omission:** Real editors cannot be automated (require interactive input, different key bindings, complex state)
- **What won't be validated:** Real editor compatibility, exit code quirks, file modification detection edge cases
- **Risk assessment:** MEDIUM - Mock editor covers core logic, but real editors may have quirks
- **Mitigation:** Manual validation checklist with common editors (vim, nano, VS Code)
- **Revisit criteria:** If users report editor-specific bugs or incompatibilities

**Cross-platform Editor Tests (Windows, Linux, macOS)**
- **Reason for omission:** Editor resolution uses standard env vars ($EDITOR, $VISUAL) which are cross-platform
- **What won't be validated:** Platform-specific editor quirks (notepad.exe on Windows)
- **Risk assessment:** LOW - Standard editors (vim, nano) available on all platforms
- **Mitigation:** Document minimum requirements, rely on community testing
- **Revisit criteria:** If users report platform-specific editor resolution failures

**Performance/Benchmark Tests**
- **Reason for omission:** `/edit` performance is dominated by external editor (out of our control)
- **What won't be validated:** Temp file creation speed, content comparison performance
- **Risk assessment:** LOW - File I/O is fast, no performance requirements
- **Mitigation:** Monitor in practice
- **Revisit criteria:** If users report slow `/edit` command (excluding editor launch time)

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/metacommands.rs::tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 10 tests
- **Key scenarios to cover:**
  1. Editor resolution: $VISUAL set → returns $VISUAL value
  2. Editor resolution: $VISUAL not set, $EDITOR set → returns $EDITOR value
  3. Editor resolution: Neither set → returns "vi"
  4. Editor resolution: Neither set, vi not in PATH → error message
  5. Temp file creation with `.sql` extension
  6. Content comparison: modified content → returns true
  7. Content comparison: identical content → returns false
  8. Content comparison: empty file → returns false
  9. `/edit` command parsing
  10. `\e` alias parsing
- **Mocking strategy:** Mock environment variables, mock file system for editor existence checks

**Test Type: Integration Tests (with Mock Editor)**
- **Location:** `tests/integration_edit_command.rs` (new file)
- **Framework:** Built-in Rust integration test support with std::process::Command
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. Create mock editor script (bash script that modifies file)
  2. Set $EDITOR to mock script, test `/edit` modifies and executes query
  3. Create mock editor script that exits without modifying file
  4. Test `/edit` skips execution when file unmodified
  5. Create mock editor script that exits with error code
  6. Test `/edit` handles editor failure gracefully
- **Setup requirements:** Create temp directory with mock editor scripts, set $EDITOR in test process
- **Mock editor scripts:**
  - `mock_editor_modify.sh`: Appends "-- modified" to file, exits 0
  - `mock_editor_no_change.sh`: Exits 0 without modifying file
  - `mock_editor_error.sh`: Exits with code 1

**Test Type: Interactive Tests (with Mock Editor)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. Execute SQL, then `/edit` with mock editor (modifies file), verify query re-executed
  2. Execute SQL, then `/edit` with mock editor (no changes), verify "No changes" message
  3. `/edit` with no previous query, verify error message "No previous query to edit"
  4. Tab completion includes `/edit` and `\e` with description
  5. `/help` output includes `/edit` command
- **Implementation notes:** Requires live database connection (marked with `#[ignore]`), uses mock editor via $EDITOR

**Test Type: Manual Validation Checklist**
- **Checklist:**
  - [ ] `/edit` works with vim (Linux, macOS)
  - [ ] `/edit` works with nano (Linux, macOS)
  - [ ] `/edit` works with emacs (if installed)
  - [ ] `/edit` works with VS Code (`code --wait`)
  - [ ] `/edit` works with Sublime Text (`subl --wait`)
  - [ ] $VISUAL takes precedence over $EDITOR
  - [ ] Fallback to `vi` works when neither env var set
  - [ ] Error message clear when no editor available
  - [ ] Syntax highlighting works in editor (`.sql` extension)
  - [ ] Empty file (delete all content) skips execution
  - [ ] Edited query can be repeated with `/repeat` afterward
  - [ ] Works seamlessly in multi-line query workflow
- **Estimated time:** 20 minutes
- **Testing procedure:**
  1. Build release: `cargo build --release`
  2. Start REPL: `./target/release/tq repl --logon ...`
  3. Execute query: `SELECT 1 AS test;`
  4. Test with each editor (set $EDITOR, run `/edit`, modify, save, exit)
  5. Verify query re-executes and result matches edited content

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Editor resolution logic, temp file handling, content comparison, error messages
- Integration tests validate: End-to-end workflow with mock editor, process coordination, error handling
- Interactive tests validate: REPL integration, user-visible messages, tab completion, help text
- Manual validation validates: Real editor compatibility and UX quality
- Combined coverage: **Comprehensive with known gap (real editor compatibility)**

**Gaps in combined coverage:**
- Real editor compatibility testing relies on manual validation (cannot automate vim/nano/emacs)
- Cross-platform editor quirks may exist but are low risk (standard editors consistent)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified" (with manual validation for real editors)
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Real editor automation gap is acceptable because:
  - Mock editor covers all core logic paths
  - Real editors (vim, nano, VS Code) are industry-standard with predictable behavior
  - Manual validation checklist covers common editors
  - Community testing will identify rare editor quirks
  - Risk is MEDIUM but mitigated by thorough mock testing + manual checklist

---

### Feature 2: Optional Live-DB Test for `/show indexes` (2 ACs) - P1

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-37-planning.md` lines 64-74
- Secondary: Sprint 36 Review - Action Items
- Context: Sprint 36 delivered `/show indexes` but deferred optional live-DB test

**Requirements:**
1. "`#[ignore]` test validates `/show indexes` with real Teradata connection" (AC-14, line 68)
2. "Test validates output format and column headers" (AC-15, line 69)

**Feature Characteristics:**

**User Interaction Type:** ✅ Integration Test (database-dependent)
**Explanation:** This is a test case addition, not a feature. It validates Sprint 36's `/show indexes` command against a live database.

**Observable Behavior:**
- ✅ Database side effects (queries DBC.IndicesV)
- ✅ Structured data output (table format validation)

**External Dependencies:**
- ✅ Database connection (requires live Teradata with TQ_LOGON env var)

**Validation Challenges:**
- **Database availability**: Test marked `#[ignore]`, runs only when database available
- **Test data dependency**: Requires table with indexes for validation
- **Output format validation**: Must verify column headers and formatting

**Critical Behaviors to Validate:**
1. `/show indexes <table>` queries DBC.IndicesV successfully
2. Output format matches specification (IndexName, IndexType, ColumnName, ColumnPosition)
3. Column headers displayed correctly
4. Table exists with indexes (test data setup)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Database connection" + "System catalog query" checked:
  → Integration test with #[ignore] REQUIRED
  Reason: Live database validation, runs optionally when TQ_LOGON available
```

**Derived Test Types:**

**Test Type 1: Integration Test (Live Database, #[ignore])**
- **Validates:** `/show indexes` output format and correctness with real Teradata connection
- **Approach:** Create `#[ignore]` test that queries known table with indexes, validates output structure
- **Rationale:** Provides optional live-database validation when TQ_LOGON available
- **Gap if missing:** Real database behavior not validated (Sprint 36 already has unit tests)
- **Necessity:** ✅ REQUIRED (P1 acceptance criteria)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Integration test (#[ignore]) | ✅ REQUIRED | P1 acceptance criteria from Sprint 36 review | Live DB validation missing | MUST IMPLEMENT |
| Unit tests | ❌ NOT NEEDED | Sprint 36 already has unit tests | N/A | SKIP |
| Interactive tests | ❌ NOT NEEDED | Sprint 36 already has interactive tests | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 1 (Integration with #[ignore])
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 2 (Unit, Interactive - already exist)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-14 | `#[ignore]` test with real Teradata connection | sprint-37-planning.md line 68 | Integration (#[ignore]) | Optional live-DB validation | TC-037-007 |
| AC-15 | Validates output format and column headers | sprint-37-planning.md line 69 | Integration (#[ignore]) | Must verify table structure | TC-037-007 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ All test types justified by requirements
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Unit Tests**
- **Reason for omission:** Sprint 36 already has comprehensive unit tests for `/show indexes`
- **What won't be validated:** Nothing - unit tests already exist
- **Risk assessment:** NONE - existing tests sufficient
- **Mitigation:** N/A
- **Revisit criteria:** Never - Sprint 36 coverage is complete

**Interactive Tests**
- **Reason for omission:** Sprint 36 already has interactive tests for `/show indexes`
- **What won't be validated:** Nothing - interactive tests already exist
- **Risk assessment:** NONE - existing tests sufficient
- **Mitigation:** N/A
- **Revisit criteria:** Never - Sprint 36 coverage is complete

#### 6. Test Implementation Plan

**Test Type: Integration Test (Live Database, #[ignore])**
- **Location:** `tests/integration_show_indexes.rs` (new file or append to existing)
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 1 test
- **Key scenario to cover:**
  1. Connect to live database via TQ_LOGON
  2. Execute `/show indexes` on known table with indexes (e.g., system catalog table)
  3. Validate output contains expected columns: IndexName, IndexType, ColumnName, ColumnPosition
  4. Validate column headers present
  5. Validate table formatting (alignment, separators)
- **Setup requirements:** Live Teradata connection with TQ_LOGON env var, test uses system catalog table (guaranteed indexes)
- **Test attributes:** `#[test] #[ignore]` - Runs only with `cargo test -- --ignored`

**Implementation notes:**
```rust
#[test]
#[ignore] // Requires live database: cargo test -- --ignored
fn test_show_indexes_live_database_output_format() {
    // Load TQ_LOGON from environment
    let logon = env::var("TQ_LOGON").expect("TQ_LOGON not set");

    // Use system catalog table that always has indexes (e.g., DBC.Tables)
    let output = Command::new("./target/release/tq")
        .arg("repl")
        .arg("--logon")
        .arg(&logon)
        .arg("--command")
        .arg("/show indexes DBC.Tables")
        .output()
        .expect("Failed to execute tq");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Validate column headers
    assert!(stdout.contains("IndexName"));
    assert!(stdout.contains("IndexType"));
    assert!(stdout.contains("ColumnName"));
    assert!(stdout.contains("ColumnPosition"));

    // Validate table formatting (at least one index should exist)
    assert!(stdout.contains("---")); // Table separator

    // Exit status should be 0
    assert_eq!(output.status.code(), Some(0));
}
```

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Integration test validates: Real database query, output format, column headers
- Sprint 36 tests validate: Unit logic, interactive REPL behavior
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - integration test completes Sprint 36 validation

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:** N/A - no gaps

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ [Feature 1 only] (required)
- Integration tests: ✅ [Feature 1 (mock editor), Feature 2 (#[ignore])] (required)
- Interactive tests: ✅ [Feature 1 only] (required)
- Manual validation: ⚠️ [Feature 1 only] (recommended)

**Estimated Test Count:**
- Unit: 10 (Feature 1) + 0 (Feature 2) = **10 tests**
- Integration: 6 (Feature 1 mock) + 1 (Feature 2 live-DB) = **7 tests**
- Interactive: 5 (Feature 1 mock) = **5 tests**
- **Total new automated tests: 22**
- **Baseline: ~674 tests → Target: ~696 tests**

**Test Cases to Document:**
- TC-037-001: `/edit` editor resolution and temp file creation (unit)
- TC-037-002: `/edit` modified content execution (integration + interactive)
- TC-037-003: `/edit` unmodified content skips execution (unit + integration)
- TC-037-004: `/edit` tab completion and help text (interactive)
- TC-037-005: `/edit` error handling (unit + interactive)
- TC-037-006: `/edit` REPL mode restrictions (integration + interactive)
- TC-037-007: `/show indexes` live-DB validation (integration #[ignore])

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:**
  - Real editor compatibility (mitigated by manual validation checklist)
- **LOW risk gaps:**
  - Cross-platform editor quirks (mitigated by standard env vars)
  - Performance testing deferred (no requirements)

**Dependencies Required:**
- Live database: ✅ YES (Feature 1 interactive tests, Feature 2 integration test)
- Network access: ❌ NO
- Specific OS: ❌ NO (cross-platform env vars)
- File system: ✅ YES (Feature 1 - temp file creation)
- External editor: ✅ YES (Feature 1 - mock editor for automation, real editors for manual validation)

**Sprint 37 Specific Notes:**
- **Feature 1 (`/edit`):** Complex due to external process interaction - requires mock editor approach
- **Feature 2 (live-DB test):** Simple addition, completes Sprint 36 validation
- **Test complexity:** MEDIUM (mock editor strategy, process coordination)
- **Test infrastructure needs:** NEW TOOL REQUIRED - Mock editor scripts for automation

---

## Tool Requirements Assessment

### Current Testing Tools

**Available:**
- ✅ Unit test framework (built-in Rust)
- ✅ Integration test framework (std::process::Command)
- ✅ Interactive test framework (expectrl + PTY)
- ✅ Tempfile crate for test fixtures
- ✅ Environment variable management (std::env)

**Needed for Sprint 37:**
- ✅ tempfile - Already available (for temp `.sql` file testing)
- ✅ std::env - Built-in (for $EDITOR/$VISUAL testing)
- ✅ std::process::Command - Built-in (for editor process launch)
- ✅ std::fs - Built-in (for file I/O and content comparison)
- ❌ **Mock editor scripts** - NOT AVAILABLE - Must create

### New Tools Required: Mock Editor Scripts

**Why needed:** Real editors (vim, nano, VS Code) cannot be automated in tests. They require interactive input, have complex state machines, and different key bindings. Mock editors enable automated testing of `/edit` workflow.

**What they do:**
1. **mock_editor_modify.sh**: Simulates editor that modifies file
   - Reads file path from $1
   - Appends "-- modified by test editor\n" to file
   - Exits with code 0 (success)
   - Use case: Test happy path (edit → execute)

2. **mock_editor_no_change.sh**: Simulates editor that exits without modifying file
   - Reads file path from $1
   - Does nothing to file
   - Exits with code 0 (success)
   - Use case: Test no-change path (edit → skip execution)

3. **mock_editor_error.sh**: Simulates editor failure
   - Reads file path from $1
   - Does nothing to file
   - Exits with code 1 (failure)
   - Use case: Test error handling (editor crash)

4. **mock_editor_empty.sh**: Simulates editor that empties file
   - Reads file path from $1
   - Truncates file to empty
   - Exits with code 0 (success)
   - Use case: Test empty file handling (delete all content)

**Location:** `tests/fixtures/mock_editors/`

**Implementation:**
```bash
#!/bin/bash
# mock_editor_modify.sh
echo "-- modified by test editor" >> "$1"
exit 0
```

```bash
#!/bin/bash
# mock_editor_no_change.sh
# Exit without modifying file
exit 0
```

```bash
#!/bin/bash
# mock_editor_error.sh
# Simulate editor failure
exit 1
```

```bash
#!/bin/bash
# mock_editor_empty.sh
# Empty the file
> "$1"
exit 0
```

**Usage in tests:**
```rust
#[test]
fn test_edit_with_mock_editor_modify() {
    let mock_editor = PathBuf::from("tests/fixtures/mock_editors/mock_editor_modify.sh");
    std::env::set_var("EDITOR", mock_editor.to_str().unwrap());

    // Test /edit command
    let result = execute_edit_command(&mut state, &client);

    // Verify query was re-executed
    assert!(result.is_ok());
    // Verify output contains modified query result
}
```

**Platform compatibility:**
- Linux: Bash scripts work natively
- macOS: Bash scripts work natively
- Windows: Requires Git Bash or WSL (acceptable for test environment)

### Tool Assessment Summary

**Can current tools test all Sprint 37 features?** ⚠️ MOSTLY - Need mock editor scripts

**New tools needed:**
- ✅ Mock editor scripts (4 simple bash scripts) - MUST CREATE
- Location: `tests/fixtures/mock_editors/`
- Complexity: LOW (simple bash scripts)
- Development time: 15 minutes

**Recommendation:**
1. Create `tests/fixtures/mock_editors/` directory
2. Implement 4 mock editor scripts (5 lines each)
3. Make scripts executable (`chmod +x`)
4. Use in integration and interactive tests via $EDITOR env var
5. Document in `tests/README.md` - Mock Editor Testing section

**Note to coordinator:** Please ensure mock editor scripts are created before rust-teradata-architect begins implementation. These are testing infrastructure required for TDD approach.

---

## Test Execution Strategy

### Phase 1: Mock Editor Setup (15 minutes)
**Priority:** Critical (blocks all other test implementation)

**Sequence:**
1. Create `tests/fixtures/mock_editors/` directory
2. Write 4 mock editor bash scripts (mock_editor_modify.sh, mock_editor_no_change.sh, mock_editor_error.sh, mock_editor_empty.sh)
3. Make scripts executable: `chmod +x tests/fixtures/mock_editors/*.sh`
4. Verify scripts work: `bash tests/fixtures/mock_editors/mock_editor_modify.sh /tmp/test.sql`
5. Document in `tests/README.md`

**Expected results:** 4 executable mock editor scripts ready for use in tests

### Phase 2: Unit Tests (1-2 hours)
**Priority:** High (validates core logic)

**Sequence:**
1. Feature 1 unit tests (editor resolution, temp file, content comparison) - 60 min
2. Run: `cargo test --lib metacommands::tests::test_edit_*` - verify all pass
3. Expected: 10 new unit tests passing

**Expected results:** 10 new unit tests passing

### Phase 3: Integration Tests (1-2 hours)
**Priority:** High (validates workflow with mock editor)

**Prerequisites:** Mock editor scripts from Phase 1

**Sequence:**
1. Feature 1 integration tests (CLI with mock editor) - 60 min
2. Feature 2 integration test (live-DB #[ignore] test) - 15 min
3. Run: `cargo test --test integration_edit_command`
4. Run (with database): `cargo test --test integration_show_indexes -- --ignored`

**Expected results:** 6 integration tests passing (Feature 1), 1 integration test passing (Feature 2, if database available)

### Phase 4: Interactive Tests (1-2 hours)
**Priority:** High (validates REPL user experience)

**Prerequisites:** Live database connection, mock editor scripts

**Sequence:**
1. Feature 1 interactive tests (`/edit` with mock editor in REPL) - 60 min
2. Run: `cargo test --test interactive_tests::test_edit_* -- --ignored --test-threads=1`

**Expected results:** 5 new interactive tests passing

### Phase 5: Full Regression (15-30 minutes)
**Priority:** Critical (ensure zero regressions)

**Sequence:**
```bash
# Run all unit tests
cargo test --lib

# Run all integration tests
cargo test --test integration_*

# Run all interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: ~696 tests passing (674 baseline + 22 new)
```

### Phase 6: Manual Validation (20 minutes)
**Priority:** Medium (validates real editor compatibility)

**Prerequisites:** Build release binary, real editors installed

**Sequence:**
1. Test with vim: Set EDITOR=vim, run `/edit`, modify query, verify execution - 5 min
2. Test with nano: Set EDITOR=nano, run `/edit`, modify query, verify execution - 5 min
3. Test with VS Code: Set EDITOR="code --wait", run `/edit`, verify execution - 5 min
4. Test editor fallback: Unset EDITOR/VISUAL, verify `vi` used - 2 min
5. Test error case: Set EDITOR=nonexistent, verify error message - 2 min
6. Test `/repeat` after `/edit`: Verify edited query repeatable - 1 min

### Phase 7: Test Report Generation
**Priority:** Critical (documents results)

Create `tests/results/sprint-37/REPORT.md` with:
- Test execution proof (cargo output)
- Pass rate summary (X/Y tests passed)
- Manual validation results (checklist completion)
- Coverage assessment (all ACs validated)
- Verdict: APPROVED / REJECTED / BLOCKED

---

## Coverage Sufficiency Assessment

### Overall Coverage Analysis

**Feature 1 (`/edit` Command):**
- Unit tests validate: Editor resolution logic, temp file creation, content comparison, error messages
- Integration tests validate: End-to-end workflow with mock editor, process coordination, error handling
- Interactive tests validate: REPL integration, user-visible messages, tab completion, help text
- Manual validation validates: Real editor compatibility (vim, nano, VS Code, emacs)
- Coverage: **Comprehensive with MEDIUM-risk gap (real editor compatibility)**
- Gap mitigation: Manual validation checklist covers common editors, community testing will find rare quirks

**Feature 2 (`/show indexes` Live-DB Test):**
- Integration test validates: Real database query, output format, column headers
- Sprint 36 tests validate: Unit logic, interactive REPL behavior (already exist)
- Coverage: **Comprehensive** (completes Sprint 36 validation)

**Combined Sprint Coverage:**
- All 15 acceptance criteria have automated tests
- All critical behaviors validated across 3 test types
- One MEDIUM-risk gap (real editor compatibility) mitigated by manual validation
- **Overall: Comprehensive coverage for Sprint 37 deliverables**

---

## Success Criteria

Sprint 37 test strategy is successful if:

1. **Test Coverage Complete:**
   - ✅ Both features have test strategy defined
   - ✅ All 15 acceptance criteria mapped to tests
   - ✅ Test types derived from feature characteristics

2. **Test Implementation Achievable:**
   - ✅ 22 automated tests (clear scope, well-defined)
   - ⚠️ Mock editor scripts required (NEW TOOL - must create first)
   - ✅ Test setup straightforward (tempfile + mock editors + expectrl)

3. **Quality Assurance Robust:**
   - ✅ Feature 1 has unit + integration + interactive coverage
   - ✅ Feature 2 has integration coverage (complements Sprint 36)
   - ✅ Manual validation covers real editor compatibility gap

4. **Gaps Identified and Accepted:**
   - ✅ Real editor compatibility gap (MEDIUM risk, mitigated by manual validation)
   - ✅ Cross-platform editor quirks (LOW risk, standard env vars)
   - ✅ Performance tests deferred (LOW risk, no requirements)
   - ✅ All gaps have risk assessment

5. **Execution Plan Clear:**
   - ✅ Test phases defined with dependencies
   - ✅ Priority order: Mock editor setup → Unit → Integration → Interactive
   - ✅ Success criteria for each phase

---

## Strategy Validation Checklist

**Before submitting for review:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (not guessed)
- ✅ Every test type has clear rationale
- ✅ Gap analysis is complete and honest
- ✅ Specification coverage map includes all requirements
- ✅ Every requirement maps to at least one test type
- ✅ Test implementation plan is detailed and actionable
- ✅ Coverage sufficiency is assessed
- ✅ No hand-waving or vague justifications
- ✅ New tool requirements identified (mock editor scripts)

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-13
**Review Status:** DRAFT
**Sprint:** 37 - External Editor Integration
**Submitted for Review:** 2026-02-13

**Reviewer:** tq-project-manager (pending)
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Mock editor tool requirement identified and justified
- ✅ Ready to proceed with mock editor creation + test implementation
