# Testing Guidelines for tq

**Version:** 3.0.0
**Last Updated:** 2026-01-21
**Purpose:** Testing methodology, patterns, and best practices for validating the tq CLI tool

This document provides comprehensive testing guidance for the Quality Validator agent and serves as a quick reference for designing and executing quality validation tests.

---

## Core Testing Philosophy

### Test What Users See, Not Just What Code Does

**Key Principle from Sprint 13:** Tests must validate the user experience, not just the implementation mechanics.

**Bad Testing Approach:**
- Test verifies Tab key triggers completion mechanism
- Test checks that output contains some text
- Test confirms function returns without error

**Good Testing Approach:**
- Test verifies Tab after FROM shows database names (not keywords)
- Test checks that output contains semantically correct content
- Test confirms function returns the correct result for the use case

**Why This Matters:**
- Sprint 11 bugs: Tab completion showed "(SQL keyword)" instead of table names - tests passed because mechanism worked, but content was wrong
- Sprint 11 bugs: Table display had broken alignment - tests passed because columns existed, but layout was broken
- **Unit tests validate code logic. Interactive tests validate user experience.**

### The Testing Contract

> "If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate."

This contract ensures:
1. No untested features ship
2. Test failures mean real problems (not false positives)
3. Specifications reflect implementation reality

---

## Test Type Classification

### When to Use Which Test Type

Understanding when to use unit vs integration vs interactive tests is critical for effective validation.

#### Unit Tests

**Purpose:** Validate individual functions, logic, and algorithms in isolation.

**Use For:**
- Pure functions (input → output, no side effects)
- Data transformations and parsing
- Business logic and calculations
- Error handling logic
- Type conversions

**Characteristics:**
- Fast execution (<1ms per test)
- No external dependencies (mock database, file I/O, network)
- Deterministic (same input = same output always)
- Test single function/module in isolation

**Examples:**
```rust
// Good: Unit test for SQL parsing logic
#[test]
fn test_parse_connection_string() {
    let result = parse_connection_string("user:pass@host:1025/db");
    assert_eq!(result.user, "user");
    assert_eq!(result.host, "host");
}

// Good: Unit test for format conversion
#[test]
fn test_format_table_cell() {
    let cell = format_cell("test", 10, Alignment::Left);
    assert_eq!(cell, "test      ");
}
```

**When NOT to Use:**
- Testing REPL interactive features (use interactive tests)
- Testing database queries (use integration tests)
- Testing user-facing behavior (use integration or interactive tests)

#### Integration Tests

**Purpose:** Validate end-to-end workflows with real external dependencies.

**Use For:**
- CLI command execution (full invocation)
- Database query execution with real connections
- File I/O operations
- Pipeline integration (stdin/stdout)
- Output format validation (JSON, CSV, table)
- Exit code correctness

**Characteristics:**
- Slower execution (100ms-1s per test)
- Real external dependencies (database, file system)
- Test entire workflow from command input to output
- May require test fixtures or test database

**Examples:**
```rust
// Good: Integration test for query command
#[test]
fn test_query_command_json_output() {
    let output = Command::new("./target/release/tq")
        .arg("query")
        .arg("SELECT 1 AS test")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json[0]["test"], 1);
}
```

**When NOT to Use:**
- Testing pure logic functions (use unit tests)
- Testing REPL interactive behavior (use interactive tests)
- When test database not available (use unit tests with mocks)

#### Interactive Tests (REPL Features ONLY)

**Purpose:** Validate REPL interactive features as users experience them.

**Use For:**
- Tab completion content and context awareness
- Multi-line editing and line preservation
- Prompt rendering (colors, format)
- History persistence and recall
- Metacommands (output and side effects)
- Table display alignment and truncation
- Syntax highlighting appearance
- Error message display

**Characteristics:**
- Slowest execution (1-5s per test)
- Spawns real tq REPL process
- Simulates keyboard input (Tab, Enter, arrows)
- Captures and parses visual output
- **MANDATORY for all REPL features**

**Examples:**
```rust
// Good: Interactive test for tab completion
#[test]
fn test_tab_completion_after_from_shows_databases() {
    let mut repl = spawn_repl().unwrap();

    repl.send_line("SELECT * FROM ").unwrap();
    repl.send(Key::Tab).unwrap();

    let output = repl.read_until_prompt().unwrap();

    // Verify semantic correctness
    assert!(output.contains("my_database"), "Should show database names");
    assert!(!output.contains("SELECT"), "Should NOT show SQL keywords");
    assert!(!output.contains("(SQL keyword)"), "Should NOT show placeholder text");
}
```

**When NOT to Use:**
- Testing batch mode commands (use integration tests)
- Testing pure logic (use unit tests)
- When interactive test framework not available (build it first)

### Decision Tree: Which Test Type?

```
Is it a REPL interactive feature?
├─ YES → Interactive Test (mandatory)
│         + Integration test for underlying logic
│         + Unit tests for parsing/formatting
│
└─ NO → Does it require database/file I/O?
    ├─ YES → Integration Test
    │         + Unit tests for logic components
    │
    └─ NO → Unit Test
```

---

## Testing Approach

### One-Shot Execution Model

tq follows a simple execution model: one tool call → one connection → close session when done. This simplifies testing:
- Each test is completely independent
- No state carries over between tests
- No need for cleanup between test runs
- Can execute tests in parallel

### Test Environment Setup

**Prerequisites:**
- Compiled release binary: `cargo build --release`
- Live Teradata connection configured in `.env` file
- TQ_LOGON format: `username:password@host:port/database`

**Test Execution:**
- Use release-optimized binary for realistic performance
- Test against live database (not mocks) for real integration validation
- Document test environment details (OS, Rust version, DB version)

## Test Design Patterns

### Functional Test Structure

```bash
# Test ID: TC###
# Category: Functionality
# Description: [What this tests]

# Setup (if needed)
echo "SELECT 1" > /tmp/test.sql

# Execute
./target/release/tq query "SELECT 1 AS test_value"

# Verify
# - Check exit code (should be 0)
# - Check stdout contains expected output
# - Check stderr is empty
# - Verify format is correct

# Cleanup
rm /tmp/test.sql
```

### Error Handling Test Structure

```bash
# Test ID: TC###
# Category: Error-Handling
# Description: [What error condition this tests]

# Execute command that should fail
./target/release/tq query "INVALID SQL" 2>&1

# Verify error handling
# - Check exit code (should be non-zero)
# - Check error message is clear and actionable
# - Check stderr contains error (not stdout)
# - Verify no credential leaks in error output
# - Check for helpful suggestions/troubleshooting steps
```

### Integration Test Structure

```bash
# Test ID: TC###
# Category: Integration
# Description: [What integration this tests]

# Execute end-to-end workflow
echo "SELECT 1" | ./target/release/tq query --format json | jq '.[]'

# Verify
# - Multiple components work together
# - Data flows correctly through pipeline
# - Exit codes propagate correctly
# - Output format is preserved
```

## Test Categories

### 1. Functionality Tests
Test core features work as specified:
- Command execution (query, ping)
- Output formats (table, JSON, CSV)
- Input methods (argument, file, stdin)
- Data type handling (NULL, dates, timestamps, numbers)
- System catalog access (DBC views)

### 2. Error Handling Tests
Verify graceful failure and helpful error messages:
- Connection failures
- SQL syntax errors
- Table/database not found
- Invalid arguments
- Missing required parameters

### 3. Integration Tests
Test how tq works with other tools:
- stdin/stdout pipelines
- File I/O
- Environment variables
- Exit code handling in scripts

### 4. Usability Tests
Validate user experience:
- Help text quality and completeness
- Error message clarity
- Output readability
- Documentation accuracy

## Execution Best Practices

### Exit Code Verification

Always check exit codes - they're critical for scripting:
```bash
./target/release/tq query "SELECT 1"
echo "Exit code: $?"
# Should be 0 for success
```

Expected exit codes:
- `0` = Success
- `1` = Runtime error (connection, SQL, etc.)
- `2` = Usage error (invalid arguments)

### Stream Separation

Verify stdout and stderr are used correctly:
```bash
# Capture stdout only (should contain data)
./target/release/tq query "SELECT 1" > output.txt

# Capture stderr only (should be empty on success)
./target/release/tq query "SELECT 1" 2> errors.txt

# Capture both separately
./target/release/tq query "INVALID" > out.txt 2> err.txt
```

UNIX philosophy: **stdout = data, stderr = diagnostics**

### Format Validation

#### Table Format
```bash
tq query "SELECT 1 AS col1, 2 AS col2"
```
Verify:
- Column headers present
- Separators aligned
- NULL values displayed as `[NULL]`
- Numbers right-aligned, text left-aligned

#### JSON Format
```bash
tq query "SELECT 1 AS col1" --format json
```
Verify:
- Valid JSON (use `jq` to parse)
- Array of objects structure
- Type preservation (numbers as numbers, not strings)
- NULL values as `null` (not string)

#### CSV Format
```bash
tq query "SELECT 'test' AS col1" --format csv
```
Verify:
- RFC 4180 compliance
- Proper quote escaping
- Headers present (unless --no-headers)
- NULL values as empty fields

### Pipeline Testing

Test UNIX pipeline integration:
```bash
# stdin input
echo "SELECT 1" | tq query

# stdout piping
tq query "SELECT 1" | grep "1"

# Full pipeline
echo "SELECT * FROM table" | tq query --format json | jq '.'
```

## Common Pitfalls

### 1. Reserved Keywords

**Issue**: SQL reserved words used as unquoted identifiers cause syntax errors.

**Example**:
```sql
SELECT 1 AS value  -- FAILS: "value" is reserved
SELECT 1 AS "value"  -- OK: quoted
SELECT 1 AS col1  -- OK: not reserved
```

**Testing**: Avoid reserved keywords in test queries unless explicitly testing error handling.

**Teradata Common Reserved Words**: VALUE, USER, DATE, TIME, TIMESTAMP, TABLE, DATABASE, ORDER, BY

### 2. First Statement Only (File Input)

**Issue**: When file contains multiple statements, only first is executed.

**Current Behavior**:
```sql
-- file.sql
SELECT 1;
SELECT 2;  -- This won't execute
```

**Testing**: Use single-statement files or test enhancement requests explicitly.

### 3. Connection Timing

**Issue**: First connection includes TLS handshake overhead (~950ms).

**Testing**: Don't assert on exact timing, use reasonable ranges.

### 4. NULL Handling

**Issue**: NULL values need special display handling.

**Expected Behavior**:
- Table format: `[NULL]`
- JSON format: `null`
- CSV format: empty field

**Testing**: Always include NULL in multi-type tests.

### 5. Credential Leaks

**Critical**: Never leak credentials in error messages or logs.

**Testing Checklist**:
- Error messages don't contain passwords
- Connection string errors mask sensitive parts
- Stack traces don't expose credentials
- Log output (if any) sanitizes secrets

## Test Documentation Standards

### Test Case Format

```markdown
# TC### - [Test Title]

**Category**: Functionality | Error-Handling | Integration | Usability
**Priority**: Critical | High | Medium | Low
**Status**: PASS | FAIL | BLOCKED | SKIPPED

## Objective
[What this test validates]

## Prerequisites
- [Any setup required]

## Test Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Expected Results
- [Expected outcome 1]
- [Expected outcome 2]

## Actual Results
- [What actually happened]

## Exit Code
[Actual exit code and whether it matches expected]

## Notes
[Any observations, issues, or enhancement opportunities]
```

### Test Report Structure

```markdown
# Quality Validation Report

**Date**: YYYY-MM-DD HH:MM:SS
**Commit**: [git hash]
**Test Coverage**: X test cases executed

## Executive Summary
[2-3 paragraphs summarizing overall results and verdict]

## Test Statistics
- Total: X
- Passed: X (X%)
- Failed: X (X%)
- Blocked: X (X%)

## Findings
### Critical Issues
[Issues blocking production]

### Major Issues
[Issues causing significant problems]

### Minor Issues
[Small problems or test issues]

### Enhancement Opportunities
[Nice-to-have improvements]

## Recommendations
### Immediate
[Do before next release]

### Short Term
[Next sprint]

### Long Term
[Backlog items]

## Test Case Summary
[Table with all test cases and results]

## Conclusion
[Final verdict and recommended next steps]
```

## CLI-Specific Testing Considerations

### UNIX Philosophy Compliance

Verify tq follows UNIX principles:

1. **Do one thing well**: Execute SQL queries, output results
2. **Text streams**: Output is parseable text (not binary)
3. **Composability**: Works in pipelines with other tools
4. **Silent success**: No output on stdout except data
5. **Helpful errors**: Clear diagnostic messages on stderr

### Command-Line Argument Testing

Test all input modes:
```bash
# Argument
tq query "SELECT 1"

# File
tq query --file query.sql

# stdin
echo "SELECT 1" | tq query
cat query.sql | tq query
```

### Help Text Validation

Check help quality:
- `tq --help` (top-level)
- `tq query --help` (subcommand)

Help should include:
- Brief description
- Usage patterns
- All options with descriptions
- Examples
- Environment variables
- Security guidance

### Flag Behavior

Test flag combinations:
```bash
# Format flags
tq query "SELECT 1" --format json
tq query "SELECT 1" --format csv --no-headers

# Connection overrides
tq query "SELECT 1" --connection "user:pass@host:port/db"

# Combined flags
tq query "SELECT 1" --format csv --file query.sql
```

## Database-Specific Testing

### Teradata Driver Integration

Test low-level driver behavior:
- Connection establishment
- Query execution
- Result fetching
- Connection cleanup
- Error propagation

### Data Type Coverage

Test Teradata data types:
- **Numeric**: INTEGER, DECIMAL, FLOAT
- **String**: CHAR, VARCHAR
- **Date/Time**: DATE, TIME, TIMESTAMP
- **Special**: NULL

Future coverage:
- BLOB, CLOB
- ARRAY types
- Complex types

### System Catalog Access

Test DBC views work correctly:
```bash
tq query "SELECT * FROM DBC.DatabasesV WHERE DatabaseName = USER"
tq query "SELECT TableName FROM DBC.TablesV WHERE DatabaseName = DATABASE"
```

Verify:
- Queries execute successfully
- Results format correctly
- No permission issues

## Performance Testing

### Execution Time Patterns

- **First connection**: ~950ms (includes TLS handshake)
- **Query execution**: <50ms overhead
- **Small result formatting**: <10ms
- **Large result streaming**: Test separately

### Memory Usage

- Verify streaming output (not buffering entire result set)
- Check for memory leaks on long-running queries
- Test with large result sets (100K+ rows)

### Concurrent Testing

Test multiple simultaneous executions:
```bash
# Launch multiple tq instances in parallel
for i in {1..10}; do
  tq query "SELECT 1" &
done
wait
```

Verify no race conditions or conflicts.

## Test Result Analysis

### Success Criteria

**Production Ready** requires:
- ✅ All critical functionality tests pass
- ✅ All error handling tests pass
- ✅ No crashes or hangs
- ✅ Exit codes correct
- ✅ stdout/stderr used correctly
- ✅ No credential leaks
- ✅ Help documentation complete
- ✅ ≥95% test pass rate

### Pass Rate Interpretation

- **100%**: Perfect (rare, investigate suspicious)
- **95-99%**: Excellent, production ready
- **90-94%**: Good, address failures before release
- **<90%**: Needs work, block release

### Issue Severity Guidelines

**Critical**: Blocks production deployment
- Crashes or data corruption
- Security vulnerabilities
- Core functionality broken

**Major**: Significant but workarounds exist
- Important features don't work
- Poor error handling
- Performance problems

**Minor**: Small issues or test problems
- Edge cases fail
- Minor UX issues
- Enhancement opportunities

**Enhancement**: Not a defect, but nice to have
- Additional features
- UX improvements
- Documentation additions

## Lessons Learned from tq Testing

### What Worked Well

1. **Testing against live database**: Caught real integration issues mocks would miss
2. **Release binary testing**: Realistic performance, caught optimization-dependent bugs
3. **Comprehensive error testing**: Validated exceptional error message quality
4. **Pipeline integration tests**: Proved UNIX philosophy compliance
5. **Format verification**: Ensured all output modes work correctly

### What Could Be Improved

1. **Test data management**: Consider using a dedicated test database/schema
2. **Automated test execution**: Script all tests for one-command validation
3. **Edge case coverage**: Add tests for very large results, unusual data types
4. **Performance benchmarking**: Establish baselines and track over time
5. **Regression testing**: Keep old test cases to prevent reintroduction of bugs

### Key Insights

1. **Error messages matter**: Users judge quality by how helpful errors are
2. **Exit codes critical**: Scripts depend on correct exit codes
3. **Stream separation essential**: stdout/stderr must be used correctly
4. **Format compliance matters**: JSON must be valid, CSV must follow RFC 4180
5. **Documentation is feature**: Help text is first user experience

## Sprint 11 Lessons: Testing Visual/Interactive Features

### What We Learned From Sprint 11 Bug Failures

**Bug Context:**
- Tab completion showed "(SQL keyword)" repeated instead of database/table names
- Table display had excessive padding, scattered text (broken AGAIN)
- Both bugs occurred despite Sprint 9/10 tests passing

**Root Cause - Test Coverage Gaps:**

1. **Content Validation Missing**
   - Tests verified *mechanism* works (Tab triggers completion)
   - Tests didn't verify *content* is correct (shows actual tables, not keywords)
   - **Lesson**: Test semantic meaning, not just mechanical function

2. **Visual Layout Not Validated**
   - Tests checked content presence (columns exist in output)
   - Tests didn't check visual layout (headers align with data)
   - **Lesson**: Interactive features need visual inspection, not just content checks

3. **Live Database Testing Inconsistent**
   - Some tests used mocks or assumptions
   - Real Teradata behavior differs from assumptions
   - **Lesson**: MANDATORY live database testing for all REPL features

### New Testing Requirements (Sprint 11+)

**For Tab Completion Tests:**

1. **Content Validation** (not just mechanism)
   ```
   ✗ BAD:  "Verify Tab key triggers completion"
   ✓ GOOD: "Verify Tab after FROM shows database names (not keywords)"
   ```

2. **Semantic Checks**
   - What type of completions shown? (tables vs keywords vs garbage)
   - Are they queryable? (can user use them?)
   - Do they make sense in context? (databases after FROM)

3. **Anti-Pattern Detection**
   - Explicitly test for known failure modes
   - Example: Check for "(SQL keyword)" repeated
   - Document what should NOT happen

4. **Live Database Required**
   - Must test against real Teradata
   - Must have actual databases/tables
   - Must verify metadata queries work

**For Table Display Tests:**

1. **Visual Layout Validation**
   ```
   ✗ BAD:  "Verify table contains columns X, Y, Z"
   ✓ GOOD: "Verify header 'X' aligns with data column X"
   ```

2. **Width Measurements**
   - Measure actual character widths
   - Count columns displayed
   - Verify fits in terminal (80, 120, 160 cols)
   - Test with `tput cols` and measure output

3. **Readability Checks**
   - Can human read it? (subjective but critical)
   - Headers clear? Data clear?
   - Alignment preserved?
   - Professional appearance?

4. **Terminal Width Testing**
   - Test multiple widths: 80, 120, 160, 200+ cols
   - Verify dynamic adjustment
   - Check edge cases (very narrow, very wide)

5. **Visual Comparison**
   - Screenshots recommended
   - Before/after comparisons
   - Document expected appearance

**For REPL Interactive Features (General):**

1. **Test With Live Environment**
   - Real database connection
   - Real terminal (not just stdout capture)
   - Real user interaction patterns

2. **Validate Meaning, Not Just Mechanics**
   - Don't just check "output exists"
   - Check "output is useful and correct"
   - Semantic validation critical

3. **Visual Inspection Required**
   - Some features need human eyes
   - Automated tests can't catch everything
   - Document visual acceptance criteria

4. **Test Failure Modes**
   - What happens when things go wrong?
   - Permission denied, timeout, connection loss
   - Graceful degradation vs crashes

5. **Regression Test Suites**
   - Re-run previous sprint tests
   - Ensure fixes don't break other features
   - Systematic regression checking

### Updated Test Case Template

**For interactive/visual features, test cases must include:**

```markdown
## Anti-Pattern (What Should NOT Happen)

**INCORRECT Output (Bug Behavior):**
[Screenshot or description of broken behavior]
[Specific example of failure mode to watch for]

## Visual Validation

**Layout Checklist:**
- [ ] Visual inspection item 1
- [ ] Visual inspection item 2
- [ ] Specific alignment checks
- [ ] Width measurements

**Semantic Validation:**
- [ ] Content type is correct (tables not keywords)
- [ ] Content is usable/queryable
- [ ] Makes sense in context
```

### Testing Philosophy Changes

**OLD Approach (Insufficient):**
- ✓ Feature exists
- ✓ Mechanism works
- ✓ Content present

**NEW Approach (Sprint 11+):**
- ✓ Feature exists
- ✓ Mechanism works
- ✓ Content present
- ✓ **Content is semantically correct** (NEW)
- ✓ **Layout is visually correct** (NEW)
- ✓ **Tested with live database** (NEW)
- ✓ **Known failure modes don't occur** (NEW)

### Specific Test Gaps That Allowed Sprint 11 Bugs

**Tab Completion Gaps:**

What we tested:
- ✓ Tab key triggers completion
- ✓ Completion mechanism works
- ✓ Some output appears

What we DIDN'T test:
- ✗ Output contains actual database/table names
- ✗ Output does NOT contain generic keywords
- ✗ Context detection works with real SQL
- ✗ Completions are queryable/useful

**Table Display Gaps:**

What we tested:
- ✓ Table output generated
- ✓ Columns present in output
- ✓ Data values appear

What we DIDN'T test:
- ✗ Headers align with data visually
- ✗ Column widths reasonable (not excessive)
- ✗ Table fits in terminal width
- ✗ Layout is readable by human

### Mandatory Checklist for REPL Feature Tests

Before marking test complete:

- [ ] Tested with live Teradata database
- [ ] Validated content semantically (not just presence)
- [ ] Checked visual layout/alignment (if applicable)
- [ ] Measured widths/counts (quantitative validation)
- [ ] Tested known failure modes explicitly
- [ ] Human visual inspection performed
- [ ] Can reproduce bug if test fails
- [ ] Anti-patterns documented

### Tools and Techniques

**For Visual Testing:**

1. **Terminal Width Control:**
   ```bash
   # Set terminal to specific width
   # iTerm2: Profiles > Window > Columns
   # Terminal.app: Window Settings
   tput cols  # Verify width
   ```

2. **Width Measurement:**
   ```bash
   # Count characters in output line
   output | head -1 | wc -c

   # Visual ruler (iTerm2)
   # View > Show Ruler
   ```

3. **Screenshot Capture:**
   - Document expected appearance
   - Compare before/after
   - Include in test results

4. **Automated REPL Testing:**
   - Use `expectrl` crate (see tests/interactive_tests.rs)
   - Simulate Tab key presses
   - Capture and validate output

**For Semantic Testing:**

1. **Live Database Queries:**
   ```rust
   // Verify completion results are queryable
   let completion = get_completion_after_from();
   let query = format!("SELECT * FROM {} LIMIT 1", completion);
   assert!(execute_query(query).is_ok());  // Must work!
   ```

2. **Content Type Validation:**
   ```rust
   let completions = get_completions();
   assert!(completions.iter().all(|c| is_database_object(c)));
   assert!(!completions.iter().any(|c| is_sql_keyword(c)));
   ```

3. **Context Verification:**
   ```rust
   // After FROM, should show databases/tables
   let context = CompletionContext::from("SELECT * FROM ");
   assert_eq!(context.expected_type, CompletionType::Table);
   ```

### Prevention: How to Avoid Sprint 11-Style Regressions

1. **Mandatory Live Database Testing**
   - Every REPL feature test runs against real Teradata
   - No mocks for integration tests
   - CI/CD must have test database available

2. **Visual Acceptance Tests**
   - Manual visual inspection required
   - QA agent performs human validation
   - Screenshots in test results

3. **Semantic Assertions**
   - Test MEANING of output, not just presence
   - Validate completions are usable
   - Check context awareness works

4. **Regression Test Suites**
   - Re-run all previous tests after changes
   - Automated regression detection
   - Block merge if regressions found

5. **Known-Failure Testing**
   - Explicitly test for past bug patterns
   - Example: "Does NOT show '(SQL keyword)' repeated"
   - Document anti-patterns in tests

### Success Metrics

**Quality-Validator Agent Checklist:**

When validating a sprint, verify:

1. **Test Coverage:**
   - [ ] All features have test cases
   - [ ] Tests include semantic validation
   - [ ] Visual features have layout tests
   - [ ] Anti-patterns explicitly checked

2. **Test Execution:**
   - [ ] All tests run against live database
   - [ ] Visual inspection performed
   - [ ] Measurements taken (widths, counts)
   - [ ] Screenshots captured

3. **Regression Prevention:**
   - [ ] Previous sprint tests re-run
   - [ ] No new failures introduced
   - [ ] Known bugs don't reappear

4. **Documentation:**
   - [ ] Test cases document visual expectations
   - [ ] Anti-patterns described
   - [ ] Failure modes covered
   - [ ] Lessons learned updated

## Quick Reference Checklist

### Before Testing

- [ ] Build release binary: `cargo build --release`
- [ ] Configure test environment in `.env`
- [ ] Verify database connectivity: `tq ping`
- [ ] Document test environment (OS, versions, etc.)

### During Testing

- [ ] Execute each test case individually
- [ ] Capture stdout and stderr separately
- [ ] Record exit codes
- [ ] Save output samples
- [ ] Note any unexpected behavior

### After Testing

- [ ] Calculate pass/fail statistics
- [ ] Categorize all issues by severity
- [ ] Document enhancement opportunities
- [ ] Generate comprehensive report
- [ ] Recommend next steps

### Test Coverage Checklist

- [ ] Version flag
- [ ] Help text (top-level and subcommand)
- [ ] Ping command
- [ ] Query execution (all formats)
- [ ] stdin input
- [ ] File input
- [ ] NULL handling
- [ ] Multi-column queries
- [ ] Date/time types
- [ ] System catalog queries
- [ ] Connection failure
- [ ] SQL syntax error
- [ ] Table not found
- [ ] Invalid arguments
- [ ] Pipeline integration

## Templates

### Quick Test Script Template

```bash
#!/bin/bash
# Test: [Description]

TQ="./target/release/tq"

echo "Test: [TC###] - [Title]"

# Execute
$TQ query "SELECT 1 AS test" > /tmp/out.txt 2> /tmp/err.txt
EXIT_CODE=$?

# Verify
if [ $EXIT_CODE -eq 0 ] && grep -q "test" /tmp/out.txt; then
    echo "✓ PASS"
else
    echo "✗ FAIL (exit code: $EXIT_CODE)"
    cat /tmp/err.txt
fi

# Cleanup
rm -f /tmp/out.txt /tmp/err.txt
```

### Test Case Template (Markdown)

```markdown
# TC### - [Test Title]

**Category**: Functionality
**Priority**: High
**Status**: PASS

## Objective
Verify that [specific behavior] works correctly.

## Prerequisites
- tq binary built in release mode
- Valid TQ_LOGON configured
- [Any other requirements]

## Test Steps
1. Execute: `tq query "SELECT 1 AS col1"`
2. Check exit code
3. Verify output format

## Expected Results
- Exit code: 0
- stdout contains: formatted table with value 1
- stderr: empty

## Actual Results
✓ Exit code: 0
✓ Output correct
✓ No errors

## Notes
Test passed successfully. Output formatting is excellent.
```

## Reference

### Key Commands

```bash
# Build for testing
cargo build --release

# Run single test
./target/release/tq query "SELECT 1"

# Test with format
./target/release/tq query "SELECT 1" --format json

# Test stdin
echo "SELECT 1" | ./target/release/tq query

# Test file
./target/release/tq query --file test.sql

# Test help
./target/release/tq --help
./target/release/tq query --help

# Test ping
./target/release/tq ping
```

### Useful Verification Tools

- `jq` - Validate and format JSON
- `csvlint` - Validate CSV format
- `wc -l` - Count result rows
- `grep` - Search output
- `diff` - Compare outputs
- `xxd` - Inspect binary/encoding

---

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 3.0.0 | Added "Test What Users See" philosophy, test type classification decision tree, interactive testing requirements clarified | CLI UX Designer |
| 2026-01-18 | 2.0.0 | Sprint 11 lessons added - visual/interactive feature testing requirements | Quality Validator |
| 2026-01-17 | 1.0.0 | Initial version from Sprint 10 testing | Quality Validator |

---

**See Also:**
- [Testing Checklist](testing-checklist.md) - Phase-specific testing requirements
- [Definition of Done](definitions/done.md) - Sprint completion criteria
- Interactive test examples: `tests/interactive_tests.rs`
