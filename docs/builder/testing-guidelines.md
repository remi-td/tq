# Testing Guidelines for tq

This document provides testing methodology, patterns, and best practices for validating the tq CLI tool. It serves as a quick reference for designing and executing quality validation tests.

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

**Document Version**: 1.0
**Last Updated**: 2026-01-17
**Based on**: tq v1.0.0 testing (commit 369af18)
**Author**: quality-validator agent
