# Testing Philosophy

This document establishes the core principles that guide testing decisions for the tq project.

## Fundamental Principle: Test What Users See

**Key insight**: Tests must validate the user experience, not just the implementation mechanics.

### The Problem with Implementation-Focused Testing

**Bad testing approach**:
- Test verifies Tab key triggers completion mechanism ❌
- Test checks that output contains some text ❌
- Test confirms function returns without error ❌

These tests pass when the machinery works, even if the user experience is broken.

**Good testing approach**:
- Test verifies Tab after `FROM` shows database names (not keywords) ✅
- Test checks that output contains semantically correct content ✅
- Test confirms function returns the correct result for the use case ✅

These tests fail when users would encounter problems, even if the code "works".

### Historical Evidence

Real bugs that passed tests because tests focused on mechanics, not experience:

**Sprint 11 - Tab Completion**:
- **Bug**: Tab completion showed "(SQL keyword)" instead of table names
- **Test status**: Passing ✅ (completion mechanism worked)
- **User experience**: Broken ❌ (wrong content displayed)
- **Root cause**: Tests validated that *something* appeared, not *what* appeared

**Sprint 11 - Table Display**:
- **Bug**: Table alignment broken, columns misaligned
- **Test status**: Passing ✅ (columns existed)
- **User experience**: Broken ❌ (unreadable output)
- **Root cause**: Tests validated structure, not visual presentation

### The Solution

**Unit tests validate code logic. Interactive tests validate user experience.**

Both are necessary. Neither is sufficient alone.

## The Testing Contract

> "If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate."

This three-part contract ensures:

1. **No untested features ship**
   - Every specification requirement has corresponding test coverage
   - Features without tests are not considered "done"

2. **Test failures mean real problems**
   - No false positives from flaky tests
   - No tests that pass on broken features
   - Test failures block shipping

3. **Specifications reflect implementation reality**
   - When implementation diverges from spec, one must change
   - Passing tests prove spec and implementation align
   - Tests are living documentation of actual behavior

## Coverage Philosophy

### Two Types of Coverage

**Automated Coverage** (~40% for tq):
- Measured by cargo-tarpaulin
- Includes only `cargo test --lib` (unit tests)
- Does not include interactive or integration tests
- Automatically measured in CI/CD

**Total Coverage** (~85% for tq):
- Includes automated coverage PLUS interactive tests
- Interactive tests cover REPL modules not measurable by unit tests
- Cannot be automatically measured (requires live database)
- Represents true validation coverage

### Why "Low" Automated Coverage is Expected

The 40% automated coverage is **appropriate** for tq's architecture:

1. **REPL modules require interactive testing**
   - `src/commands/repl/` cannot be meaningfully unit tested
   - Tab completion needs real database metadata
   - Syntax highlighting needs visual validation
   - Paging needs terminal interaction

2. **Interactive tests provide real coverage**
   - 20 interactive tests exercise REPL code paths
   - These tests provide user-experience validation
   - Cannot be counted in automated coverage metrics

3. **Quality over quantity**
   - A REPL test validating "Tab after FROM shows databases" provides more value than a unit test mocking the completion mechanism
   - Better to have 20 high-value interactive tests than 200 low-value unit tests

### Coverage Expectations by Module

| Module Type | Primary Test Type | Expected Unit Coverage | Rationale |
|-------------|-------------------|------------------------|-----------|
| Parser | Unit tests | >90% | Pure logic, no I/O |
| Config | Unit tests | >80% | Configuration parsing |
| Format | Unit tests | >80% | Output formatting |
| DB types | Unit tests | >80% | Type conversions |
| CLI | Unit tests | >70% | Argument parsing |
| REPL executor | Interactive tests | <30% unit | Requires live REPL |
| Tab completion | Interactive tests | <20% unit | Requires database |
| Pager | Interactive tests | <20% unit | Requires terminal |
| Syntax highlighting | Interactive tests | <30% unit | Visual validation |

### When to Focus on Automated Coverage

**Improve automated coverage when**:
- Adding new parsing logic (should have unit tests)
- Adding new data transformations (should have unit tests)
- Adding new configuration options (should have unit tests)
- Finding bugs in testable code paths

**Don't worry about low automated coverage for**:
- REPL interaction code (use interactive tests)
- Terminal rendering code (use visual validation)
- Database metadata queries (use integration tests)

## Quality Over Metrics

### Anti-Patterns to Avoid

**Deferring a REQUIRED Test**:
- A test classified REQUIRED in the sprint test strategy that is not authored by
  the end of Phase 3 must be reported as a MEDIUM-severity gap in the quality
  report. It is not resolved by code inspection or manual verification.
- "The harness isn't ready" is only valid if the harness genuinely does not
  exist; if the harness exists, deferring the test is a choice that must be
  recorded as a gap, not hidden behind a `passed`/`accepted` label.
- See [`approach.md`](approach.md#required-tests-are-not-optional) and the
  authoritative rule in [`honest-assessment.md`](honest-assessment.md).

**Test to Coverage Ratio**:
- Don't write tests just to increase coverage percentage
- Coverage is a byproduct of good testing, not the goal

**Mock-Heavy Unit Tests**:
- If 80% of test code is mocking, consider integration testing
- Mocks test your understanding of interfaces, not actual behavior

**Tests That Mirror Implementation**:
- Tests should validate behavior from user perspective
- Implementation changes shouldn't require test rewrites

### Patterns to Embrace

**Test User Scenarios**:
```rust
#[test]
fn user_can_complete_table_names_after_from() {
    // Simulates actual user workflow
    let input = "SELECT * FROM cu";
    let completions = get_completions(input, db);
    assert!(completions.contains("customer"));
    assert!(!completions.contains("SELECT")); // No keywords!
}
```

**Integration Over Unit for Complex Flows**:
```rust
#[test]
fn query_execution_end_to_end() {
    let result = execute_query_string("SELECT 1", &config);
    assert_eq!(result.rows[0][0], "1");
}
```

**Interactive Tests for User Experience**:
```rust
#[test]
#[ignore] // Requires manual verification
fn tab_completion_shows_correct_suggestions() {
    // Test script that developer runs and validates visually
}
```

## Test Design Philosophy

### Start with "What Should Happen"

1. Read the specification
2. Write test name describing expected behavior
3. Implement test validating that behavior
4. Implement feature to pass the test

### Focus on Boundaries and Edge Cases

**Boundaries**:
- Empty input
- Maximum input
- Invalid input
- Missing configuration

**Edge Cases**:
- NULL values
- Empty result sets
- Connection failures
- Timeout scenarios

### Make Tests Readable

Tests are documentation. Other developers (and AI agents) should understand:
- What behavior is being tested
- Why this test exists
- What failure indicates

```rust
#[test]
fn connection_timeout_returns_clear_error() {
    // Given: A database connection with 1s timeout
    let config = ConnectionConfig {
        timeout: Duration::from_secs(1),
        ..default_config()
    };

    // When: Connecting to unreachable host
    let result = Connection::connect(&config);

    // Then: Should fail with timeout error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}
```

## Testing Limitations and Manual Validation

### What Automated Tests Cannot Validate

**Critical Insight from Sprint 29/30:** Automated tests with 100% pass rate delivered completely broken features. This section documents the fundamental limitations of automated testing.

**Visual/Rendering Features:**
- Actual rendered output in a terminal
- Column alignment and table formatting
- Text wrapping and truncation behavior
- Color display correctness
- Alternate screen buffer content (pager, full-screen modes)

**Interactive Features:**
- Real terminal input/output timing
- PTY behavior differences from unit test environment
- User perception of responsiveness
- Navigation flow across alternate screens

**Terminal-Dependent Behavior:**
- Output at specific terminal widths (80, 117, 120, 160 chars)
- Resize handling
- Scrollback buffer behavior
- ANSI escape sequence rendering

### The Sprint 29/30 Lesson

**Pattern Observed:**
- Sprint 29: 386/386 tests passed (100%) - Feature completely broken
- Sprint 30: 449/449 tests passed (100%) - Feature still broken
- Total investment: $81 for zero working functionality

**Root Cause:** Tests validated code structure and internal logic, not actual user-visible output. The pager wrote to alternate screen buffer which was invisible to test framework.

**Key Learning:** A test pass rate of 100% means nothing if tests do not validate what users actually see and experience.

### When Manual Validation is Mandatory

Manual validation is **REQUIRED** (not optional) for:

1. **Pager/alternate screen features** - Cannot capture alternate buffer content
2. **Terminal width-dependent rendering** - Must test at multiple actual widths
3. **Visual formatting** - Table alignment, column widths, borders
4. **Interactive navigation** - Real terminal input/response cycles
5. **Any feature where automated tests cannot capture actual output**

### Manual Validation Process

1. **Build release binary**: `cargo build --release`
2. **Test at multiple terminal widths**: 80, 117, 120, 160 characters
3. **Use real database connection**: Not mocks
4. **Capture evidence**: Use `script` command or screenshots
5. **Human verification**: Developer or coordinator confirms correct behavior

### Test Type Selection for Visual Features

| Feature Type | Unit Tests | Integration Tests | Interactive Tests | Manual Validation |
|--------------|------------|-------------------|-------------------|-------------------|
| Pure logic | Primary | Secondary | - | - |
| Output formatting | Primary | Secondary | - | Verify |
| REPL interaction | Tertiary | Secondary | Primary | **Required** |
| Pager rendering | Tertiary | - | Primary | **Required** |
| Alternate screen | - | - | Limited | **Required** |

### Sprint Quality Gate Update

**Sprint Closure Requirements:**
1. All automated tests pass (necessary but not sufficient)
2. Manual validation completed for visual/interactive features
3. Manual validation evidence documented in test results
4. Coordinator approval based on actual functionality, not test metrics

**quality-validator verdict is ADVISORY for visual features.** The sprint coordinator must manually verify before approval.

## Conclusion

Effective testing validates user experience, not just code mechanics. The goal is confidence that features work as specified, not arbitrary coverage percentages.

**Critical Update:** For visual and interactive features, automated test pass rates are necessary but NOT sufficient. Manual validation by a human is REQUIRED before claiming a feature works.

When in doubt, ask: "If this test passes, can I be confident the user experience is correct?" For visual features, the answer is often "No - I need to see it with my own eyes."
