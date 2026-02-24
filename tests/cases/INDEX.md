# Test Case Index for tq (Teradata Query)

**Project:** tq - Teradata Query CLI Tool
**Version:** 1.8.0 (Sprint 33 - Pager Bug Fix + Data Sampling)
**Last Updated:** 2026-02-03
**Base Commit:** [Sprint 33 - In Progress]

## Overview

This directory contains comprehensive test case definitions for the tq CLI tool. These test cases cover all implemented MVP features (FR-001 through FR-010) and provide detailed procedures for validating functionality, usability, error handling, and security.

## Test Case Categories

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
