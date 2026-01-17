# Test Case Index for tq (Teradata Query)

**Project:** tq - Teradata Query CLI Tool
**Version:** 0.1.0
**Last Updated:** 2026-01-17
**Base Commit:** 369af18

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

### Error-Handling
- **TC002**: Ping Command - Connection Failure
- **TC007**: Connection String Parsing - Invalid Formats
- **TC021**: Query Command - SQL Syntax Errors

### Usability
- **TC013**: CLI Help and Version Information
- **TC017**: Verbose and Quiet Output Modes
- **TC018**: Color Output Control

### Integration
- **TC014**: Exit Codes - Comprehensive Validation
- **TC019**: Environment Variable Configuration
- **TC023**: CSV Format - Special Character Escaping

### Security
- **TC022**: Security - No Password Exposure

## Test Priority Matrix

### Critical Priority (Must Pass for MVP)
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

### Medium Priority (Quality of Life)
| Test ID | Feature | Category |
|---------|---------|----------|
| TC015 | NULL Handling | Functionality |
| TC017 | Verbose/Quiet | Usability |
| TC018 | Color Control | Usability |
| TC020 | Large Results | Functionality |
| TC024 | Multiple Pings | Functionality |
| TC025 | Query Timing | Functionality |

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

## Test Execution Guidelines

### Prerequisites for All Tests
1. tq binary built and available (`cargo build --release`)
2. Test Teradata database accessible (or mock for some tests)
3. Valid test credentials configured in `.env` file (recommended) or via environment variables
4. Required tools installed: jq (for JSON tests), ps (for security tests)

### Test Execution Order
**Recommended order for initial validation:**

1. **Smoke Tests** (verify basic functionality):
   - TC001: Basic ping
   - TC003: Basic query
   - TC013: Help/version

2. **Core Functionality**:
   - TC004, TC005: Output formats
   - TC006: Connection string parsing
   - TC008: Authentication mechanisms
   - TC009: Password files

3. **Error Handling**:
   - TC002: Connection failures
   - TC007: Invalid connection strings
   - TC021: SQL errors

4. **Integration**:
   - TC010, TC011, TC012: Input/output methods
   - TC014: Exit codes
   - TC019: Environment variables

5. **Security**:
   - TC022: Password exposure

6. **Quality**:
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
