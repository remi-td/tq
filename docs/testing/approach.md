# Testing Approach

This document defines the testing strategy, test type classification, and design patterns for tq.

## Test Type Classification

tq uses three distinct test types, each serving a specific purpose in the validation strategy.

### Unit Tests

**Purpose**: Validate individual functions, logic, and algorithms in isolation

**Use for**:
- Pure functions (input → output, no side effects)
- Data transformations and parsing
- Business logic and calculations
- Error handling logic
- Type conversions

**Characteristics**:
- Fast execution (<1ms per test)
- No external dependencies (mock database, file I/O, network)
- Deterministic (same input = same output always)
- Test single function/module in isolation

**Location**: Inline with source code (`#[cfg(test)] mod tests`)

**Example**:
```rust
#[test]
fn test_parse_connection_string() {
    let result = parse_connection_string("user:pass@host:1025/db");
    assert_eq!(result.user, "user");
    assert_eq!(result.host, "host");
    assert_eq!(result.port, 1025);
}
```

**When NOT to use**:
- Testing REPL interactive features → Use interactive tests
- Testing database queries → Use integration tests
- Testing user-facing behavior → Use integration or interactive tests

### Integration Tests

**Purpose**: Validate end-to-end workflows with real external dependencies

**Use for**:
- CLI command execution (full invocation)
- Database query execution with real connections
- File I/O operations
- Pipeline integration (stdin/stdout)
- Output format validation (JSON, CSV, table)
- Exit code correctness

**Characteristics**:
- Slower execution (100ms-1s per test)
- Real external dependencies (database, file system)
- Test entire workflow from command input to output
- May require test fixtures or test database

**Location**: `tests/` directory (e.g., `tests/integration_test.rs`)

**Example**:
```rust
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

**When NOT to use**:
- Testing pure logic functions → Use unit tests
- Testing REPL interactive behavior → Use interactive tests
- When test database not available → Use unit tests with mocks

### Interactive Tests (REPL Features ONLY)

**Purpose**: Validate REPL interactive features as users experience them

**Use for**:
- Tab completion content and context awareness
- Multi-line editing and line preservation
- Prompt rendering (colors, format)
- History persistence and recall
- Metacommands (output and side effects)
- Table display alignment and truncation
- Syntax highlighting appearance
- Error message display

**Characteristics**:
- Slowest execution (1-5s per test)
- Spawns real tq REPL process
- Simulates keyboard input (Tab, Enter, arrows)
- Captures and parses visual output
- **MANDATORY for all REPL features**

**Location**: `tests/interactive_tests.rs` (with `#[ignore]` attribute)

**Example**:
```rust
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_tab_completion_after_from_shows_databases() {
    let mut repl = spawn_repl().unwrap();

    repl.send_line("SELECT * FROM ").unwrap();
    repl.send(Key::Tab).unwrap();

    let output = repl.read_until_prompt().unwrap();

    // Verify semantic correctness
    assert!(output.contains("my_database"), "Should show database names");
    assert!(!output.contains("SELECT"), "Should NOT show SQL keywords");
    assert!(!output.contains("(SQL keyword)"), "Should NOT show placeholder");
}
```

**When NOT to use**:
- Testing batch mode commands → Use integration tests
- Testing pure logic → Use unit tests
- When interactive test framework not available → Build it first

## Decision Tree: Which Test Type?

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

## REQUIRED Tests Are Not Optional

When the sprint test strategy classifies a test as **REQUIRED**, that
classification is binding for the sprint. A REQUIRED test that is not authored
by the end of Phase 3 (implementation) **must be reported as a MEDIUM-severity
gap in the quality report**. It is not resolved by code inspection, manual
verification, "structural verification" that greps the source, or a different
test that exercises a related-but-not-identical code path.

Concretely:

- A REQUIRED test is satisfied **only** by authoring a test that asserts on the
  exact code path the acceptance criterion specifies.
- If a REQUIRED test cannot be authored within the sprint, the evidence doc
  labels the AC `skipped for reason: REQUIRED test not authored — <reason>`
  (never `run and passed`, never `skipped for reason: accepted fallback`) and
  rates the gap at **MEDIUM severity minimum**.
- A P2 follow-up item naming the exact test to author goes into the sprint
  review.

This rule prevents phantom coverage: every time it has been relaxed, the next
sprint inherits an AC that the evidence record claims is "passed" or "accepted"
while the code path goes unexercised. The authoritative, detailed statement of
this rule — including the per-AC assertion-citation requirement and worked
examples — lives in [`honest-assessment.md`](honest-assessment.md) under "The
REQUIRED-is-not-optional Rule".

## Testing Strategy by Feature Area

### CLI Argument Parsing
- **Primary**: Unit tests for clap derive validation
- **Secondary**: Integration tests for end-to-end command execution

### SQL Query Execution (Batch Mode)
- **Primary**: Integration tests with real database
- **Secondary**: Unit tests for query parsing/formatting

### REPL Mode
- **Primary**: Interactive tests for user experience
- **Secondary**: Integration tests for command execution
- **Tertiary**: Unit tests for completion logic

### Output Formatting
- **Primary**: Unit tests for formatter logic
- **Secondary**: Integration tests for full pipeline

### Connection Management
- **Primary**: Integration tests with real connections
- **Secondary**: Unit tests for configuration parsing

### Error Handling
- **Primary**: Unit tests for error type construction
- **Secondary**: Integration tests for user-facing error messages

## Test Design Patterns

### Pattern 1: Arrange-Act-Assert

Structure tests clearly with three sections:

```rust
#[test]
fn connection_timeout_returns_clear_error() {
    // Arrange: Set up test conditions
    let config = ConnectionConfig {
        timeout: Duration::from_secs(1),
        ..default_config()
    };

    // Act: Perform the operation
    let result = Connection::connect(&config);

    // Assert: Verify expected outcome
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}
```

### Pattern 2: Test Boundaries and Edge Cases

Always test:
- **Empty input**: `""`, `[]`, `None`
- **Maximum input**: Long strings, large numbers
- **Invalid input**: Malformed data, wrong types
- **NULL values**: Database NULL handling
- **Empty results**: Zero-row query results
- **Connection failures**: Network errors, timeouts

### Pattern 3: Semantic Validation Over Structural

**Bad** (tests structure):
```rust
assert!(output.len() > 0); // Output exists
assert!(output.contains("(")); // Has parentheses
```

**Good** (tests semantics):
```rust
assert!(output.contains("my_database")); // Shows database name
assert!(!output.contains("SELECT")); // Doesn't show keywords
```

### Pattern 4: Descriptive Test Names

Test names should describe behavior from user perspective:

**Bad**:
```rust
fn test_completion() // What about completion?
fn test_tab() // What happens with tab?
```

**Good**:
```rust
fn tab_after_from_shows_database_names()
fn tab_after_select_shows_column_names()
fn invalid_connection_string_returns_error()
```

### Pattern 5: One Assertion Focus Per Test

**Bad** (tests multiple behaviors):
```rust
#[test]
fn test_query() {
    assert_eq!(result.rows.len(), 5);
    assert_eq!(result.format, Format::Table);
    assert_eq!(result.timing, Some(Duration::from_millis(100)));
}
```

**Good** (focused tests):
```rust
#[test]
fn query_returns_correct_row_count() {
    assert_eq!(result.rows.len(), 5);
}

#[test]
fn query_uses_default_table_format() {
    assert_eq!(result.format, Format::Table);
}

#[test]
fn query_tracks_execution_time() {
    assert!(result.timing.is_some());
}
```

### Pattern 6: Test Fixtures for Common Setup

Use fixtures to reduce duplication:

```rust
fn test_connection_config() -> ConnectionConfig {
    ConnectionConfig {
        host: "testhost".to_string(),
        port: 1025,
        database: "testdb".to_string(),
        user: "testuser".to_string(),
        password: Some(Secret::new("testpass".to_string())),
        logmech: LogonMechanism::TD2,
        timeout: Duration::from_secs(30),
    }
}

#[test]
fn connection_succeeds_with_valid_config() {
    let config = test_connection_config();
    let result = Connection::connect(config);
    assert!(result.is_ok());
}
```

## Test Data Management

### Database Test Data

For integration and interactive tests requiring database:

1. **Use test database**: Never test against production
2. **Isolate test data**: Each test creates and cleans up its own data
3. **Use environment variables**: Configure test connection via `.env`
4. **Document requirements**: Test cases specify required database state

### Test Fixtures

For file-based tests:

1. **Store in `tests/fixtures/`**: Separate directory for test data
2. **Keep minimal**: Only include data necessary for test
3. **Version control**: Check fixtures into git
4. **Document format**: Add README explaining fixture purpose

## Test Organization

### File Organization

```
tests/
├── integration_test.rs       # Integration tests
├── interactive_tests.rs      # REPL interactive tests
├── fixtures/                 # Test data files
│   ├── sample_query.sql
│   └── expected_output.csv
├── cases/                    # Test case documentation
│   ├── TC001-ping.md
│   └── TC002-query.md
├── strategy/                 # Per-sprint test strategies
│   └── sprint-15-test-strategy.md
└── results/                  # Test execution results
    └── sprint-15/
```

### Test Case Documentation

Each test case in `tests/cases/` should document:
- **ID**: Unique identifier (TC###)
- **Feature**: What feature is being tested
- **Objective**: What behavior is validated
- **Prerequisites**: Required setup (database, config, etc.)
- **Steps**: How to execute the test
- **Expected Results**: What should happen
- **Acceptance Criteria**: How to judge pass/fail

See `tests/README.md` for test case template.

## Test Maintenance

### Keeping Tests Green

1. **Fix broken tests immediately**: Don't let tests stay red
2. **Update tests when specs change**: Tests must match current requirements
3. **Remove obsolete tests**: Delete tests for removed features
4. **Refactor duplicate logic**: Extract common patterns to helpers

### Test Quality Metrics

Evaluate test quality by:
- **Pass rate**: Should be 100% on main branch
- **Flakiness**: Tests should not randomly fail
- **Execution time**: Keep unit tests fast (<1ms)
- **Clarity**: Tests should be readable without comments
- **Coverage**: Validate user-critical paths

## Testing Limitations by Feature Type

### The Fundamental Gap: Sprint 29/30 Post-Mortem

Two consecutive sprints achieved 100% test pass rates while delivering completely broken pager functionality:

| Sprint | Tests Passed | Feature Status | User Assessment |
|--------|--------------|----------------|-----------------|
| Sprint 29 | 386/386 (100%) | Broken | "absolutely not working" |
| Sprint 30 | 449/449 (100%) | Still broken | "exact same issue" |

**Root Cause Analysis:**

Tests validated:
- API contracts (compile-time verification)
- Configuration management
- Logic flow and branching
- Data structure integrity

Tests did NOT validate:
- Actual rendered output width
- Visual alignment in real terminal
- Alternate screen buffer content
- User-perceived functionality

### Feature Types and Their Testing Limitations

#### Type 1: Pure Logic (Full Automated Coverage Possible)

Features: Parsing, calculations, data transformations, business rules

**Limitations:** None - automated tests can fully validate

**Example:** Connection string parsing, SQL formatting

#### Type 2: I/O-Based (Partial Automated Coverage)

Features: File operations, network calls, database queries

**Limitations:**
- Environment-dependent behavior
- Timing and timeout variations
- Network/disk failure scenarios

**Mitigation:** Integration tests with real dependencies + edge case mocking

#### Type 3: Terminal Output (Limited Automated Coverage)

Features: Table formatting, colored output, progress indicators

**Limitations:**
- ANSI escape sequences not captured in test output
- Terminal width effects not reproducible
- Visual alignment not verifiable

**Mitigation:** Integration tests for content + **manual visual verification**

#### Type 4: Interactive/Alternate Screen (Minimal Automated Coverage)

Features: Pager, full-screen modes, interactive navigation

**Limitations:**
- Alternate screen buffer invisible to test framework
- PTY timing differs from real terminal
- User interaction sequences not reproducible
- Terminal resize behavior untestable

**Mitigation:** Limited PTY tests for state changes + **MANDATORY manual validation**

### Testing Strategy by Limitation Type

#### For Type 4 Features (Interactive/Alternate Screen)

**DO NOT:**
- Trust 100% test pass rate as proof of functionality
- Build more test infrastructure when tests cannot capture output
- Claim feature works without manual terminal verification

**DO:**
- Use automated tests for what they CAN validate (state changes, API contracts)
- Require manual validation at multiple terminal widths (80, 117, 120, 160)
- Capture manual test evidence with `script` command
- Block sprint closure until manual validation passes

#### Validation Checklist for Type 4 Features

Before claiming a Type 4 feature is complete:

- [ ] Automated tests pass (necessary but not sufficient)
- [ ] Manual test at 80 char terminal width
- [ ] Manual test at 117 char terminal width (user-reported problematic width)
- [ ] Manual test at 120 char terminal width
- [ ] Manual test at 160 char terminal width
- [ ] Evidence captured (script output or screenshots)
- [ ] Human verification that output is correct

### Test Infrastructure Reality Check

**Sprint 30 Track 3 Infrastructure:**
- 1,552 lines of test utilities (visual_validator.rs, terminal_simulator.rs)
- 92 utility tests (all passing)
- 28 dimensional tests (all passing)
- **Bugs caught:** Zero
- **Rendering issues detected:** Zero

**Lesson:** More test infrastructure does not equal better validation when the fundamental problem is that tests cannot capture the output being validated.

### The "Can We Actually Test This?" Question

Before building test infrastructure, ask:

1. **Can automated tests capture the actual user-visible output?**
   - If NO: Manual validation is required, not more test code

2. **Would a passing test prove the feature works from user perspective?**
   - If NO: The test validates the wrong thing

3. **Has this type of feature failed in the past despite tests passing?**
   - If YES: Previous approach was wrong, need different strategy

## Continuous Integration

Tests run automatically:
- **On every commit**: Unit tests and fast integration tests
- **On PR**: Full test suite including interactive tests
- **Nightly**: Extended integration tests with real database

**Note:** CI test pass rate is necessary but NOT sufficient for visual/interactive features. Manual validation gates must be passed before sprint closure.

See `.github/workflows/` for CI configuration.
