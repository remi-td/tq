---
title: Sprint 10 Test Plan - Batch Mode Foundation
sprint: 10
version: 1.0
created: 2026-01-18
status: Ready for Execution
---

# Sprint 10 Test Plan: Batch Mode Foundation

## Executive Summary

This test plan provides comprehensive validation coverage for Sprint 10's batch mode features: stdin input, file input, and multiple statement execution. The plan includes 5 primary test cases (TC066-TC070), unit test requirements, integration test specifications, manual testing procedures, edge case analysis, regression validation, and test automation recommendations.

**Coverage:**
- stdin input (echo, cat, heredoc)
- File input (--file flag, absolute/relative paths)
- Multiple statement execution (semicolon separation, fail-fast)
- Error handling (file I/O, SQL errors, input conflicts)
- All output formats (table, JSON, CSV)

**Test Strategy:**
- Unit tests: Fast, no database required
- Integration tests: CLI + mock/real database
- Manual tests: Real Teradata validation
- Regression tests: Ensure existing features still work

---

## Table of Contents

1. [Test Case Overview](#1-test-case-overview)
2. [Unit Test Requirements](#2-unit-test-requirements)
3. [Integration Test Requirements](#3-integration-test-requirements)
4. [Manual Testing Checklist](#4-manual-testing-checklist)
5. [Edge Cases and Negative Tests](#5-edge-cases-and-negative-tests)
6. [Regression Test Plan](#6-regression-test-plan)
7. [Test Automation Strategy](#7-test-automation-strategy)
8. [Performance Validation](#8-performance-validation)
9. [Security Considerations](#9-security-considerations)
10. [Test Execution Schedule](#10-test-execution-schedule)

---

## 1. Test Case Overview

### Primary Test Cases (TC066-TC070)

| Test ID | Title | Category | Priority | Focus |
|---------|-------|----------|----------|-------|
| TC066 | stdin Input with echo | Functionality | Critical | Pipe from echo command |
| TC067 | File Input with --file Flag | Functionality | Critical | File reading, path resolution |
| TC068 | stdin Input with Heredoc | Functionality | High | Heredoc syntax, shell scripts |
| TC069 | Multiple Statement Execution | Functionality | Critical | Semicolon separation, fail-fast |
| TC070 | Comprehensive Error Handling | Error-Handling | Critical | All error scenarios |

### Test Coverage Matrix

| Feature | Unit Tests | Integration Tests | Manual Tests |
|---------|-----------|-------------------|--------------|
| stdin detection | ✓ | ✓ | ✓ |
| File reading | ✓ | ✓ | ✓ |
| Statement parsing | ✓ | ✓ | ✓ |
| Multiple statements | ✓ | ✓ | ✓ |
| Fail-fast behavior | ✓ | ✓ | ✓ |
| Error handling | ✓ | ✓ | ✓ |
| Output formats | ✓ | ✓ | ✓ |
| Exit codes | ✓ | ✓ | ✓ |

---

## 2. Unit Test Requirements

### 2.1 Statement Parser Module

**Location:** `src/sql/parser.rs` (new module)

**Test Suite:** `tests/unit/sql_parser_tests.rs`

#### Test: Parse Single Statement

```rust
#[test]
fn test_parse_single_statement() {
    let sql = "SELECT 1 AS col1;";
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 1);
    assert_eq!(stmts[0], "SELECT 1 AS col1");
}
```

#### Test: Parse Multiple Statements

```rust
#[test]
fn test_parse_multiple_statements() {
    let sql = "SELECT 1; SELECT 2; SELECT 3;";
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 3);
    assert_eq!(stmts[0], "SELECT 1");
    assert_eq!(stmts[1], "SELECT 2");
    assert_eq!(stmts[2], "SELECT 3");
}
```

#### Test: Skip Empty Statements

```rust
#[test]
fn test_skip_empty_statements() {
    let sql = "SELECT 1;; ; SELECT 2;;;";
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 2);
    assert_eq!(stmts[0], "SELECT 1");
    assert_eq!(stmts[1], "SELECT 2");
}
```

#### Test: Trim Whitespace

```rust
#[test]
fn test_trim_whitespace() {
    let sql = "  SELECT 1  ;  SELECT 2  ;";
    let stmts = parse_statements(sql);
    assert_eq!(stmts[0], "SELECT 1");
    assert_eq!(stmts[1], "SELECT 2");
}
```

#### Test: Multi-line Statements

```rust
#[test]
fn test_multiline_statements() {
    let sql = r#"
        SELECT employee_id,
               first_name,
               last_name
        FROM employees
        WHERE active = 1;
    "#;
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 1);
    assert!(stmts[0].contains("SELECT"));
    assert!(stmts[0].contains("FROM employees"));
}
```

#### Test: Preserve Comments

```rust
#[test]
fn test_preserve_comments() {
    let sql = r#"
        -- This is a comment
        SELECT 1;
        /* Multi-line
           comment */
        SELECT 2;
    "#;
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 2);
    // Comments preserved (Teradata handles them)
}
```

#### Test: No Trailing Semicolon

```rust
#[test]
fn test_no_trailing_semicolon() {
    let sql = "SELECT 1";
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 1);
    assert_eq!(stmts[0], "SELECT 1");
}
```

#### Test: Empty Input

```rust
#[test]
fn test_empty_input() {
    let sql = "";
    let stmts = parse_statements(sql);
    assert_eq!(stmts.len(), 0);
}
```

### 2.2 Input Source Detection Module

**Location:** `src/commands/query.rs`

**Test Suite:** `tests/unit/input_source_tests.rs`

#### Test: Detect Argument Input

```rust
#[test]
fn test_detect_argument_input() {
    let args = QueryArgs {
        sql: Some("SELECT 1".to_string()),
        file: None,
    };
    let source = determine_input_source(&args, false)?;
    assert!(matches!(source, InputSource::Argument(_)));
}
```

#### Test: Detect File Input

```rust
#[test]
fn test_detect_file_input() {
    let args = QueryArgs {
        sql: None,
        file: Some(PathBuf::from("test.sql")),
    };
    let source = determine_input_source(&args, false)?;
    assert!(matches!(source, InputSource::File(_)));
}
```

#### Test: Detect stdin Input

```rust
#[test]
fn test_detect_stdin_input() {
    let args = QueryArgs {
        sql: None,
        file: None,
    };
    // is_stdin_piped = true (simulated)
    let source = determine_input_source(&args, true)?;
    assert!(matches!(source, InputSource::Stdin));
}
```

#### Test: Reject Multiple Sources (Argument + File)

```rust
#[test]
fn test_reject_argument_and_file() {
    let args = QueryArgs {
        sql: Some("SELECT 1".to_string()),
        file: Some(PathBuf::from("test.sql")),
    };
    let result = determine_input_source(&args, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Multiple input sources"));
}
```

#### Test: Reject Multiple Sources (Argument + stdin)

```rust
#[test]
fn test_reject_argument_and_stdin() {
    let args = QueryArgs {
        sql: Some("SELECT 1".to_string()),
        file: None,
    };
    // is_stdin_piped = true
    let result = determine_input_source(&args, true);
    assert!(result.is_err());
}
```

#### Test: Require At Least One Source (TTY)

```rust
#[test]
fn test_require_input_source_tty() {
    let args = QueryArgs {
        sql: None,
        file: None,
    };
    // is_stdin_piped = false (TTY)
    let result = determine_input_source(&args, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No query provided"));
}
```

### 2.3 File I/O Error Handling

**Test Suite:** `tests/unit/file_io_tests.rs`

#### Test: File Not Found

```rust
#[test]
fn test_file_not_found() {
    let result = read_sql_file("/tmp/nonexistent_file_xyz.sql");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("File not found"));
    assert!(err.contains("nonexistent_file_xyz.sql"));
}
```

#### Test: Empty File

```rust
#[test]
fn test_empty_file() {
    let temp_file = "/tmp/tq_test_empty.sql";
    std::fs::write(temp_file, "").unwrap();

    let result = read_sql_file(temp_file);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty"));

    std::fs::remove_file(temp_file).unwrap();
}
```

#### Test: Whitespace-Only File

```rust
#[test]
fn test_whitespace_only_file() {
    let temp_file = "/tmp/tq_test_whitespace.sql";
    std::fs::write(temp_file, "   \n\n   \n").unwrap();

    let result = read_sql_file(temp_file);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty"));

    std::fs::remove_file(temp_file).unwrap();
}
```

### 2.4 Exit Code Mapping

**Test Suite:** `tests/unit/exit_code_tests.rs`

#### Test: Success Exit Code

```rust
#[test]
fn test_success_exit_code() {
    let result = Ok(());
    let exit_code = map_result_to_exit_code(result);
    assert_eq!(exit_code, 0);
}
```

#### Test: SQL Error Exit Code

```rust
#[test]
fn test_sql_error_exit_code() {
    let result = Err(Error::SqlError("Syntax error".to_string()));
    let exit_code = map_result_to_exit_code(result);
    assert_eq!(exit_code, 1);
}
```

#### Test: Usage Error Exit Code

```rust
#[test]
fn test_usage_error_exit_code() {
    let result = Err(Error::UsageError("Missing argument".to_string()));
    let exit_code = map_result_to_exit_code(result);
    assert_eq!(exit_code, 2);
}
```

---

## 3. Integration Test Requirements

### 3.1 CLI Argument Parsing Integration

**Test Suite:** `tests/integration/cli_parsing_tests.rs`

#### Test: Parse --file Flag

```bash
tq query --file test.sql
# Verify flag parsed and file path extracted
```

#### Test: Parse Format Flag with File

```bash
tq query --file test.sql --format json
# Verify both flags parsed correctly
```

#### Test: Parse Connection Override with File

```bash
tq query --file test.sql -l user:pass@host:1025/db
# Verify connection override works with file input
```

### 3.2 stdin Detection Integration

**Test Suite:** `tests/integration/stdin_detection_tests.rs`

#### Test: Detect Piped Input

```bash
echo "SELECT 1" | tq query
# Verify stdin detected, not treated as TTY
```

#### Test: Detect TTY (No Pipe)

```bash
tq query
# (TTY mode, no input)
# Verify error about missing query
```

### 3.3 Format Output Verification

**Test Suite:** `tests/integration/format_output_tests.rs`

#### Test: Table Format from stdin

```bash
echo "SELECT 1 AS col1" | tq query --format table
# Verify table rendered correctly
```

#### Test: JSON Format from File

```bash
echo "SELECT 1 AS col1" > /tmp/test.sql
tq query --file /tmp/test.sql --format json | jq .
# Verify valid JSON output
```

#### Test: CSV Format from Heredoc

```bash
tq query --format csv <<EOF
SELECT 1 AS id, 'test' AS name;
EOF
# Verify CSV with headers
```

### 3.4 Exit Code Propagation

**Test Suite:** `tests/integration/exit_code_tests.rs`

#### Test: Success Exit Code

```bash
echo "SELECT 1" | tq query
echo $?
# Verify exit code is 0
```

#### Test: SQL Error Exit Code

```bash
echo "INVALID SQL" | tq query
echo $?
# Verify exit code is 1
```

#### Test: Usage Error Exit Code

```bash
tq query --file /tmp/nonexistent.sql
echo $?
# Verify exit code is 2
```

---

## 4. Manual Testing Checklist

### 4.1 stdin Input Tests

- [ ] **Echo Simple Query**
  ```bash
  echo "SELECT 1" | tq query
  ```
  - Result displayed correctly
  - Exit code 0

- [ ] **Cat File to stdin**
  ```bash
  cat query.sql | tq query
  ```
  - File content executed
  - Results correct

- [ ] **Heredoc Multi-line**
  ```bash
  tq query <<EOF
  SELECT CURRENT_DATE AS today,
         CURRENT_TIME AS now;
  EOF
  ```
  - Multi-line SQL works
  - Results formatted correctly

- [ ] **Process Substitution**
  ```bash
  tq query < <(echo "SELECT 1")
  ```
  - Query executes
  - Exit code 0

- [ ] **Empty stdin**
  ```bash
  echo "" | tq query
  ```
  - Error message clear
  - Exit code 2

### 4.2 File Input Tests

- [ ] **Absolute Path**
  ```bash
  tq query --file /tmp/test.sql
  ```
  - File read correctly
  - Query executes

- [ ] **Relative Path**
  ```bash
  cd /tmp
  tq query --file test.sql
  ```
  - Relative path resolved
  - Query executes

- [ ] **File Not Found**
  ```bash
  tq query --file /tmp/nonexistent.sql
  ```
  - Error message helpful
  - Current directory shown
  - Exit code 2

- [ ] **Permission Denied**
  ```bash
  chmod 000 test.sql
  tq query --file test.sql
  ```
  - Permission error clear
  - Exit code 2

- [ ] **File with Comments**
  ```sql
  -- Comment line
  SELECT 1;
  /* Block comment */
  SELECT 2;
  ```
  - Comments preserved
  - Both statements execute

### 4.3 Multiple Statement Tests

- [ ] **Multiple SELECT Statements**
  ```sql
  SELECT 1;
  SELECT 2;
  SELECT 3;
  ```
  - All execute in order
  - Results for each displayed
  - Statement numbers shown

- [ ] **DDL + DML + Query**
  ```sql
  CREATE TABLE test (id INT);
  INSERT INTO test VALUES (1);
  SELECT * FROM test;
  DROP TABLE test;
  ```
  - All statements execute
  - CREATE, INSERT, DROP show status
  - SELECT shows results

- [ ] **Fail-Fast on Error**
  ```sql
  SELECT 1;
  INVALID SQL;
  SELECT 3;
  ```
  - Statement 1 succeeds
  - Statement 2 fails with error
  - Statement 3 NOT executed
  - Error shows statement number
  - Exit code 1

### 4.4 Output Format Tests

- [ ] **Table Format from File**
  ```bash
  tq query --file test.sql --format table
  ```
  - Table borders correct
  - Alignment correct

- [ ] **JSON Format from stdin**
  ```bash
  echo "SELECT 1 AS col1" | tq query --format json
  ```
  - Valid JSON array
  - Parseable with jq

- [ ] **CSV Format with Multiple Statements**
  ```bash
  cat > multi.sql <<EOF
  SELECT 1 AS id;
  SELECT 2 AS id;
  EOF
  tq query --file multi.sql --format csv
  ```
  - Each statement produces CSV
  - Headers included

### 4.5 Shell Integration Tests

- [ ] **Exit Code in Script**
  ```bash
  #!/bin/bash
  if tq query --file migration.sql; then
      echo "Migration succeeded"
  else
      echo "Migration failed"
      exit 1
  fi
  ```
  - Exit code propagates correctly
  - Script conditional works

- [ ] **Pipeline with jq**
  ```bash
  echo "SELECT * FROM DBC.DatabasesV SAMPLE 5" | \
    tq query --format json | \
    jq '.[] | .DatabaseName'
  ```
  - Pipeline works
  - jq processes JSON correctly

- [ ] **Output Redirection**
  ```bash
  tq query --file report.sql > output.csv 2> errors.log
  ```
  - stdout to output.csv
  - stderr to errors.log
  - Streams separated correctly

### 4.6 Error Handling Tests

- [ ] **SQL Syntax Error**
  - Error message clear
  - Teradata error code shown
  - Exit code 1

- [ ] **Connection Failure**
  - Connection error clear
  - Troubleshooting steps provided
  - Exit code 1

- [ ] **Multiple Input Sources**
  ```bash
  echo "SELECT 1" | tq query "SELECT 2"
  ```
  - Error about conflict
  - Suggestions provided
  - Exit code 2

---

## 5. Edge Cases and Negative Tests

### 5.1 Input Edge Cases

| Edge Case | Test Command | Expected Behavior |
|-----------|--------------|-------------------|
| Very long statement (10KB+) | `echo "<10KB SQL>" \| tq query` | Executes successfully |
| Unicode in SQL | `echo "SELECT 'こんにちは'" \| tq query` | Unicode preserved |
| Special chars in path | `tq query --file "test (1).sql"` | Path with spaces works |
| Symlink to file | `ln -s real.sql link.sql; tq --file link.sql` | Follows symlink |
| File > 100MB | `tq query --file huge.sql` | Loads into memory (may be slow) |
| 1000+ statements | File with 1000 SELECTs | All execute, may be slow |
| Statement with no semicolon | `echo "SELECT 1" \| tq query` | Executes (last stmt doesn't need `;`) |

### 5.2 Negative Test Cases

| Negative Case | Expected Error | Exit Code |
|---------------|----------------|-----------|
| Binary file as input | "Cannot read file" or SQL error | 2 or 1 |
| Circular symlink | "File not found" or "Too many links" | 2 |
| File locked by another process | "Permission denied" or "Resource busy" | 2 |
| stdin closed unexpectedly | Error or empty input handling | 2 |
| Null bytes in SQL | SQL error or parse error | 1 |

### 5.3 Boundary Conditions

- **File size:** Test with 0 bytes, 1 byte, 100MB, 1GB (if feasible)
- **Statement count:** 0, 1, 10, 100, 1000 statements
- **Line length:** 1 char, 80 chars, 1KB, 10KB per line
- **Whitespace:** Leading, trailing, mixed tabs/spaces
- **Path length:** Very long file paths (OS limit)

---

## 6. Regression Test Plan

### 6.1 Existing Features to Validate

#### Core Functionality (Must Still Work)

- [ ] **Single Query Execution (Direct Argument)**
  ```bash
  tq query "SELECT 1"
  ```
  - Still works as before
  - No behavioral changes

- [ ] **REPL Mode Unaffected**
  ```bash
  tq repl
  ```
  - REPL starts correctly
  - Tab completion works
  - Metacommands work
  - No batch mode features leak into REPL

- [ ] **Ping Command**
  ```bash
  tq ping
  ```
  - Still works
  - Output unchanged

- [ ] **All Output Formats**
  ```bash
  tq query "SELECT 1" --format table
  tq query "SELECT 1" --format json
  tq query "SELECT 1" --format csv
  ```
  - All formats work
  - Output quality unchanged

#### Authentication Methods

- [ ] **TD2 Authentication**
  ```bash
  tq query "SELECT 1" --logmech TD2
  ```

- [ ] **LDAP Authentication**
  ```bash
  tq query "SELECT 1" --logmech LDAP
  ```

- [ ] **Kerberos Authentication**
  ```bash
  tq query "SELECT 1" --logmech KRB5
  ```

#### Configuration

- [ ] **Environment Variable (TQ_LOGON)**
  ```bash
  export TQ_LOGON="user:pass@host:port/db"
  tq query "SELECT 1"
  ```

- [ ] **Password File**
  ```bash
  tq query "SELECT 1" --password-file ~/.tq_password
  ```

### 6.2 Regression Test Checklist

**Pre-Sprint 10 Features:**
- [ ] TC001: Ping - Basic connectivity
- [ ] TC003: Query - Table output
- [ ] TC004: Query - JSON output
- [ ] TC005: Query - CSV output
- [ ] TC006: Connection string parsing
- [ ] TC008: Authentication mechanisms
- [ ] TC009: Password file support
- [ ] TC014: Exit codes
- [ ] TC022: Password security

**REPL Mode (Ensure Unaffected):**
- [ ] REPL starts and runs
- [ ] Tab completion works
- [ ] Syntax highlighting works
- [ ] Result paging works
- [ ] /logon metacommand works
- [ ] History navigation works

### 6.3 Backward Compatibility

**Changes That Must NOT Break Existing Usage:**

1. **Query argument still works:**
   ```bash
   tq query "SELECT 1"  # Must still work exactly as before
   ```

2. **No new required flags:**
   - All new flags (--file) are optional
   - Old invocations still valid

3. **Output format unchanged:**
   - Table format looks the same
   - JSON structure unchanged
   - CSV format identical

4. **Exit codes consistent:**
   - 0 = success
   - 1 = runtime error
   - 2 = usage error
   - (No new exit codes introduced)

---

## 7. Test Automation Strategy

### 7.1 Automation Classification

#### Unit Tests (Automated - Fast)

**Run on every commit:**
- Statement parser tests
- Input source detection tests
- File I/O error tests
- Exit code mapping tests

**Characteristics:**
- No database required
- Run in <5 seconds
- Part of `cargo test`

**Command:**
```bash
cargo test --lib
```

#### Integration Tests (Automated - Moderate)

**Run before merge to main:**
- CLI argument parsing
- stdin detection
- Format output verification
- Exit code propagation

**Characteristics:**
- May use mock database
- Run in <30 seconds
- Part of `cargo test --test`

**Command:**
```bash
cargo test --test integration_tests
```

#### Manual Tests (Manual - Slow)

**Run before release:**
- Real database validation
- Interactive scenarios
- Error message quality review
- Shell integration patterns

**Characteristics:**
- Requires live Teradata
- Human judgment needed
- Run in ~30 minutes
- Documented in checklist

**Execution:**
- Follow Manual Testing Checklist (Section 4)
- Document results in test results file

### 7.2 Continuous Integration (CI) Setup

**Recommended CI Pipeline:**

```yaml
# .github/workflows/test.yml (example)
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test integration_tests

      - name: Check for warnings
        run: cargo build --release 2>&1 | tee build.log && ! grep -i warning build.log
```

### 7.3 Test Automation Priorities

| Priority | Test Type | Automation Level | Effort |
|----------|-----------|------------------|--------|
| P0 | Unit tests | Fully automated | Low |
| P0 | Integration tests (mock DB) | Fully automated | Medium |
| P1 | Integration tests (real DB) | Semi-automated | High |
| P1 | Manual validation | Manual | High |
| P2 | Performance tests | Automated (separate) | High |

### 7.4 Test Data Management

**Automated Test Files:**
Create fixture files in `tests/fixtures/`:

```
tests/
└── fixtures/
    ├── simple.sql           # Single statement
    ├── multi_stmt.sql       # Multiple statements
    ├── with_comments.sql    # SQL with comments
    ├── error_stmt.sql       # Statement that fails
    └── large.sql            # Large file (performance)
```

**Test Database Setup:**
For integration tests requiring a database:
- Use Docker container with Teradata Express (if available)
- Or mock database interface
- Or skip tests if database not available (CI)

---

## 8. Performance Validation

### 8.1 Performance Benchmarks

| Operation | Target | Measurement |
|-----------|--------|-------------|
| File I/O overhead | <50ms | Time to read 10MB file |
| Statement parsing | <10ms | Parse 100 statements |
| stdin reading | <10ms | Read 10KB from pipe |
| Connection overhead | 100-200ms | First connection TLS handshake |
| Query execution | Baseline | No additional overhead |

### 8.2 Performance Test Cases

#### Test: File Input Overhead

```bash
# Create 10MB SQL file
yes "SELECT 1;" | head -100000 > /tmp/large.sql

# Measure execution time
time tq query --file /tmp/large.sql
```

**Expected:**
- File I/O: <50ms
- Parsing: <100ms
- Total overhead: <150ms (excluding query execution)

#### Test: Statement Parsing Performance

```bash
# File with 1000 statements
for i in {1..1000}; do echo "SELECT $i;"; done > /tmp/1000_stmts.sql

# Measure execution time
time tq query --file /tmp/1000_stmts.sql
```

**Expected:**
- Parsing 1000 statements: <50ms
- Most time spent in query execution, not parsing

#### Test: stdin Throughput

```bash
# Stream large SQL to stdin
cat large.sql | time tq query
```

**Expected:**
- stdin reading negligible
- No buffering issues

---

## 9. Security Considerations

### 9.1 Security Test Cases

#### Test: No Password in Error Messages

```bash
echo "INVALID SQL" | tq query -l user:SECRET@host:1025/db 2>&1 | grep -i secret
```

**Expected:**
- "SECRET" NOT found in error output
- Connection string sanitized in errors

#### Test: No Password in Process Listing

```bash
# While tq is running
echo "SELECT SLEEP(5)" | tq query -l user:SECRET@host:1025/db &
ps aux | grep tq | grep -i secret
```

**Expected:**
- "SECRET" NOT in process arguments
- Password not visible in `ps` output

#### Test: File Permissions Respected

```bash
# Create file readable only by owner
echo "SELECT 1" > /tmp/restricted.sql
chmod 600 /tmp/restricted.sql

# Try to read as different user (if possible)
sudo -u otheruser tq query --file /tmp/restricted.sql
```

**Expected:**
- Permission denied error
- File not read without proper permissions

#### Test: No SQL Injection via File Path

```bash
# Attempt SQL injection in file path
tq query --file "'; DROP TABLE users; --"
```

**Expected:**
- File not found error (path treated as literal)
- No SQL injection possible

### 9.2 Security Checklist

- [ ] Passwords never in error messages
- [ ] Passwords never in logs
- [ ] Passwords never in process listings
- [ ] File permissions respected
- [ ] No arbitrary code execution via file paths
- [ ] No SQL injection possible via inputs
- [ ] Connection strings sanitized in all output
- [ ] Temporary files (if any) created securely

---

## 10. Test Execution Schedule

### 10.1 Development Phase Testing

**During Implementation:**
- [ ] Write unit tests alongside code
- [ ] Run unit tests on every save (`cargo watch`)
- [ ] Integration tests after each feature complete

### 10.2 Pre-Review Testing

**Before Code Review:**
- [ ] All unit tests pass (100%)
- [ ] All integration tests pass (100%)
- [ ] No build warnings (`cargo build --release`)
- [ ] Manual smoke tests (TC066, TC067, TC069)

### 10.3 Pre-Merge Testing

**Before Merging to Main:**
- [ ] Full unit test suite (100% pass)
- [ ] Full integration test suite (100% pass)
- [ ] Manual testing checklist (Section 4) complete
- [ ] Regression tests (Section 6) complete
- [ ] Performance validation (Section 8) acceptable
- [ ] Security tests (Section 9) pass

### 10.4 Pre-Release Testing

**Before Tagging Release:**
- [ ] All automated tests pass
- [ ] All manual tests complete
- [ ] Regression suite clean
- [ ] Performance benchmarks met
- [ ] Security review complete
- [ ] Documentation updated
- [ ] Test results documented

---

## Summary

This test plan provides comprehensive coverage for Sprint 10 batch mode features:

**Test Cases:** 5 primary test cases (TC066-TC070)
**Unit Tests:** 25+ unit test scenarios
**Integration Tests:** 15+ integration scenarios
**Manual Tests:** 30+ manual validation steps
**Edge Cases:** 15+ edge case scenarios
**Regression:** Full backward compatibility validation
**Automation:** Unit and integration tests automated

**Success Criteria:**
- 100% unit test pass rate
- 100% integration test pass rate
- All manual tests complete with PASS
- Zero regressions
- Performance targets met
- Zero security issues

**Estimated Execution Time:**
- Automated tests: 2 minutes
- Manual validation: 30 minutes
- Total: ~35 minutes for full validation

---

**Document Version:** 1.0
**Last Updated:** 2026-01-18
**Sprint:** 10
**Author:** quality-validator
