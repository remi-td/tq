# Test Case Index for tq (Teradata Query)

**Project:** tq - Teradata Query CLI Tool
**Version:** 1.7.0 (Sprint 17)
**Last Updated:** 2026-01-21
**Base Commit:** [Sprint 17 implementation]

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
