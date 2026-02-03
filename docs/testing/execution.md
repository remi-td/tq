# Test Execution

This document explains how to run tests, interpret results, and debug test failures.

## Quick Reference

### Run All Tests
```bash
cargo test
```

### Run Unit Tests Only
```bash
cargo test --lib
```

### Run Integration Tests
```bash
cargo test --test '*'
```

### Run Interactive Tests
```bash
cargo test --test interactive_tests -- --ignored --test-threads=1
```

### Run Specific Test
```bash
cargo test test_name
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Detailed Test Execution

### Unit Tests

**Command**:
```bash
cargo test --lib
```

**Options**:
- `--lib`: Run only library unit tests (excludes integration tests)
- `-- --nocapture`: Show println! output
- `-- test_name`: Run specific test
- `-- --test-threads=1`: Run tests sequentially (for debugging)

**Example**:
```bash
# Run all unit tests
cargo test --lib

# Run specific test with output
cargo test --lib test_parse_connection_string -- --nocapture

# Run tests sequentially
cargo test --lib -- --test-threads=1
```

### Integration Tests

**Command**:
```bash
cargo test --test integration_test
```

**Prerequisites**:
- Test database configured in `.env`
- `TQ_LOGON` environment variable set

**Example**:
```bash
# Set test connection
export TQ_LOGON="testuser:testpass@testhost:1025/testdb"

# Run integration tests
cargo test --test integration_test

# Run specific integration test
cargo test --test integration_test test_query_command
```

### Interactive Tests

**Command**:
```bash
cargo test --test interactive_tests -- --ignored --test-threads=1
```

**Prerequisites**:
- Test database configured
- Terminal with TTY support
- Visual verification capability

**Why `--ignored`?**
Interactive tests are marked with `#[ignore]` because they:
- Require manual visual verification
- Take longer to execute (1-5s per test)
- Require database connection
- Cannot run in CI without special setup

**Why `--test-threads=1`?**
Interactive tests must run sequentially because they:
- Share database connection
- May affect each other's state
- Capture terminal output

**Example**:
```bash
# Run all interactive tests
cargo test --test interactive_tests -- --ignored --test-threads=1

# Run specific interactive test
cargo test --test interactive_tests test_tab_completion -- --ignored
```

## Coverage Measurement

### Generate Coverage Report

**Install cargo-tarpaulin**:
```bash
cargo install cargo-tarpaulin
```

**Generate HTML coverage report**:
```bash
cargo tarpaulin --out Html --output-dir coverage
open coverage/index.html
```

**Generate multiple format reports**:
```bash
cargo tarpaulin --out Html --out Lcov --output-dir coverage
```

### Understanding Coverage Output

```
|| Tested/Total Lines:
|| src/cli.rs: 45/60 (75.00%)
|| src/db/connection.rs: 120/150 (80.00%)
|| ...
|| 40.07% coverage, 1234/3080 lines covered
```

**Interpreting the numbers**:
- **40.07%**: Overall automated coverage (unit tests only)
- **Does not include**: Interactive tests, integration tests
- **Expected for tq**: REPL features require interactive testing

See `docs/testing/philosophy.md` for coverage philosophy.

## Test Results Interpretation

### Successful Test Run

```
test test_parse_connection_string ... ok
test test_format_cell ... ok
test test_query_execution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

### Failed Test

```
test test_tab_completion ... FAILED

failures:

---- test_tab_completion stdout ----
thread 'test_tab_completion' panicked at 'assertion failed:
  left: "SELECT",
  right: "my_database"'
```

**What to do**:
1. Read the assertion message
2. Check if test expectation is correct
3. If test is correct, fix the code
4. If test is wrong, fix the test

### Flaky Test

If a test fails intermittently:
1. **Check for timing issues**: Add explicit waits
2. **Check for shared state**: Ensure test isolation
3. **Check for external dependencies**: Mock or stabilize
4. **Run with `--test-threads=1`**: Identify race conditions

## Debugging Test Failures

### Strategy 1: Add Debug Output

```rust
#[test]
fn test_feature() {
    let result = do_something();
    eprintln!("Debug: result = {:?}", result); // Use eprintln! not println!
    assert_eq!(result, expected);
}
```

Run with: `cargo test -- --nocapture`

### Strategy 2: Use Test Helpers

```rust
fn assert_contains(output: &str, expected: &str) {
    if !output.contains(expected) {
        eprintln!("Expected to find: {}", expected);
        eprintln!("In output:\n{}", output);
        panic!("Assertion failed");
    }
}
```

### Strategy 3: Isolate the Test

```rust
#[test]
#[ignore] // Temporarily ignore other tests
fn test_specific_issue() {
    // Focused test for debugging
}
```

Run with: `cargo test test_specific_issue -- --ignored`

### Strategy 4: Check Test Data

```bash
# Verify test database state
tq query "SELECT * FROM test_table"

# Check configuration
cat .env
echo $TQ_LOGON
```

### Strategy 5: Use Debugger

With `rust-lldb` or `gdb`:

```bash
# Build test with debug symbols
cargo test --no-run

# Find test binary
find target/debug/deps -name 'tq-*' -type f

# Run in debugger
rust-lldb target/debug/deps/tq-xxxxx
```

## Common Test Issues

### Issue: Test Database Not Available

**Symptom**:
```
Error: Connection failed
test test_query ... FAILED
```

**Solution**:
1. Check `.env` file exists with `TQ_LOGON`
2. Verify database is running
3. Test connection manually: `tq ping`

### Issue: Tests Pass Locally, Fail in CI

**Causes**:
- Environment variables not set in CI
- Database not available in CI
- Timing differences (CI slower)
- File path differences

**Solution**:
- Check CI logs for specific errors
- Ensure CI has database access
- Add explicit timeouts for CI
- Use relative paths, not absolute

### Issue: Flaky Interactive Tests

**Causes**:
- Race conditions in terminal output
- Timing assumptions
- Shared REPL state

**Solution**:
- Add explicit waits: `repl.wait_for_prompt()`
- Run sequentially: `--test-threads=1`
- Restart REPL between tests

### Issue: Coverage Unexpectedly Low

**Causes**:
- REPL code not counted (expected)
- Code in main.rs not tested
- Dead code not removed

**Solution**:
- Check which modules have low coverage: `cargo tarpaulin --verbose`
- Focus on pure logic, not REPL interaction
- Remove unused code

## Test Performance

### Benchmark Test Execution

```bash
# Time unit tests
time cargo test --lib

# Time integration tests
time cargo test --test integration_test

# Time interactive tests
time cargo test --test interactive_tests -- --ignored --test-threads=1
```

**Expected times**:
- Unit tests: <2 seconds
- Integration tests: <30 seconds
- Interactive tests: <60 seconds (20 tests × 3s each)

### Optimizing Slow Tests

**For unit tests**:
- Remove unnecessary assertions
- Use simpler test data
- Mock expensive operations

**For integration tests**:
- Reuse database connections when safe
- Minimize test data
- Run independent tests in parallel

**For interactive tests**:
- Optimize wait times (not too long, not too short)
- Combine related validations
- Skip visual validation in CI (use unit tests)

## Continuous Integration

### GitHub Actions Workflow

Tests run automatically:

**On Push to Any Branch**:
```yaml
- cargo test --lib  # Unit tests
- cargo test --test integration_test  # Integration tests
```

**On Pull Request**:
```yaml
- cargo test --lib
- cargo test --test integration_test
- cargo test --test interactive_tests -- --ignored --test-threads=1
- cargo tarpaulin --out Html
```

**Nightly**:
```yaml
- Full test suite
- Extended database tests
- Performance benchmarks
```

### Required Checks

Pull requests must pass:
- ✅ All unit tests (100% pass rate)
- ✅ All integration tests (100% pass rate)
- ✅ All interactive tests (100% pass rate)
- ✅ Cargo clippy (no warnings)
- ✅ Cargo fmt (properly formatted)

## Test Reporting

### Sprint Test Execution

During sprints, test results are documented in:
- `tests/strategy/sprint-N-test-strategy.md` - Test approach
- `tests/results/sprint-N/` - Execution evidence and reports

### Test Evidence Format

Test evidence should include:
- **Date/Time**: When tests were run
- **Environment**: System info, Rust version
- **Test Output**: Full cargo test output
- **Pass Rate**: X/Y tests passed
- **Failures**: Details of any failures
- **Resolution**: How failures were fixed

See `tests/README.md` for evidence template.

## Manual Validation Process

### When Manual Validation is Required

Manual validation is **MANDATORY** (not optional) for:

1. **Pager/alternate screen features** - Automated tests cannot capture alternate buffer
2. **Terminal width-dependent rendering** - Must test at actual terminal widths
3. **Visual formatting** - Table alignment, borders, column layout
4. **Interactive navigation** - Real input/output cycles in terminal
5. **Any feature where tests passed but functionality was broken** (see Sprint 29/30)

### Step-by-Step Manual Validation

#### 1. Build the Binary

```bash
cargo build --release
```

#### 2. Set Up Test Environment

```bash
# Load test credentials
source .env

# Verify connection works
./target/release/tq ping
```

#### 3. Test at Multiple Terminal Widths

For each width (80, 117, 120, 160 chars):

```bash
# macOS: Resize terminal
printf '\e[8;40;80t'    # 80 columns
sleep 0.5

# Verify width
tput cols

# Test feature
./target/release/tq repl --logon "$TQ_LOGON"
```

#### 4. Capture Evidence with Script Command

```bash
# Start recording
script -q /tmp/manual_test_80_chars.txt

# Run tests manually
./target/release/tq repl --logon "$TQ_LOGON"
# Execute query, observe output, test navigation

# End recording
exit

# Analyze for width overflow
wc -L /tmp/manual_test_80_chars.txt  # Max line length
awk '{ if (length > 80) print NR": "length }' /tmp/manual_test_80_chars.txt
```

#### 5. Visual Verification Checklist

For pager/table features, verify:

- [ ] Table borders align correctly
- [ ] Column headers centered properly
- [ ] Data cells aligned to column
- [ ] No line wrapping or overflow
- [ ] Navigation keys work (arrows, h/j/k/l)
- [ ] Status bar displays correct position
- [ ] Exit returns to REPL cleanly

#### 6. Document Evidence

Create evidence file in `tests/results/sprint-N/`:

```markdown
## Manual Validation Evidence

**Date:** YYYY-MM-DD
**Tester:** [name or role]
**Feature:** [feature name]

### Terminal Widths Tested

| Width | Result | Notes |
|-------|--------|-------|
| 80    | PASS/FAIL | [observations] |
| 117   | PASS/FAIL | [observations] |
| 120   | PASS/FAIL | [observations] |
| 160   | PASS/FAIL | [observations] |

### Evidence Files

- `/tmp/manual_test_80_chars.txt`
- [screenshots if applicable]

### Verdict

[ ] Feature works correctly at all tested widths
[ ] Feature has issues: [describe]
```

### Sprint Closure Gate

**For features requiring manual validation:**

1. Automated tests pass: YES/NO
2. Manual validation completed: YES/NO
3. Manual validation passed: YES/NO
4. Evidence documented: YES/NO

**All four must be YES for sprint closure.**

If manual validation fails, sprint is BLOCKED. Return to implementation.

### Common Manual Validation Failures

#### Failure: Line Overflow

**Symptom:** Lines wrap unexpectedly in terminal

**Debug:**
```bash
# Find overflowing lines
awk -v w=80 '{ if (length > w) print NR": "length" chars" }' output.txt
```

**Root Cause:** Width calculation doesn't match rendered width

#### Failure: Alignment Issues

**Symptom:** Columns misaligned, borders don't line up

**Debug:**
- Check ANSI escape sequences are excluded from width calculation
- Verify unicode_width is used for wide characters

#### Failure: Navigation Broken

**Symptom:** Keys don't respond or cause unexpected behavior

**Debug:**
- Test in raw mode terminal
- Verify crossterm events are being captured

### The Sprint 29/30 Warning

**Context:** Two consecutive sprints shipped with 100% test pass rate but completely broken pager functionality.

**Lesson:** For visual/interactive features:
- Test pass rate is NECESSARY but NOT SUFFICIENT
- If you cannot manually verify the feature works, it does not work
- "Tests pass" does not mean "feature works"

**New Standard:** Sprint closure requires:
1. Automated tests pass (proves code structure is correct)
2. Manual validation passes (proves user experience is correct)

Neither alone is sufficient. Both are required.
