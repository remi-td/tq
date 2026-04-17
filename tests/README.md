# tq Test Infrastructure

This document describes the testing infrastructure for the `tq` (Teradata Query) CLI tool.

## Test Types

### 1. Unit Tests (`cargo test --lib`)

Unit tests are embedded in the source code using Rust's `#[cfg(test)]` modules. They test individual functions and modules in isolation.

**Location:** `src/**/*.rs` (inline `mod tests` blocks)

**Run:**
```bash
cargo test --lib
```

**What they cover:**
- CLI argument parsing
- SQL parsing and tokenization
- Output formatting (table, JSON, CSV)
- Value type conversions
- Error handling
- Connection string parsing
- Metadata caching logic
- SQL context analysis for completion

**Example test modules:**
- `src/cli.rs::tests` - CLI parsing tests
- `src/format/table.rs::tests` - Table formatting tests
- `src/db/types.rs::tests` - Value type tests
- `src/commands/repl/sql_context.rs::tests` - SQL context analysis tests

### 2. Integration Tests (`cargo test --test integration_tests`)

Integration tests verify the library's public API works correctly without requiring a live database.

**Location:** `tests/integration_tests.rs`

**Run:**
```bash
cargo test --test integration_tests
```

**What they cover:**
- Connection configuration creation
- Duration parsing
- Value display and conversion
- Output format generation
- Error message formatting
- CLI option parsing

**Live Database Tests (ignored by default):**
Some integration tests require a live Teradata database. These are marked with `#[ignore]` and can be run with:
```bash
# Set connection info
export TQ_LOGON="user:password@host:1025/database"

# Run ignored tests
cargo test --test integration_tests -- --ignored
```

### 3. Interactive Tests (`cargo test --test interactive_tests`)

Interactive tests use `expectrl` (PTY testing) to test the REPL in a realistic terminal environment.

**Location:** `tests/interactive_tests.rs`

**Run:**
```bash
# All interactive tests require a live database
cargo test --test interactive_tests -- --ignored
```

**What they cover:**
- REPL startup and shutdown
- Tab completion behavior
- Table display with column truncation
- Multi-line input handling
- Metacommand execution

**Note:** Interactive tests may have limitations in certain PTY environments due to cursor position detection issues with reedline.

## Running Tests

### All Tests (Excluding Live Database)
```bash
cargo test
```

### Library Tests Only
```bash
cargo test --lib
```

### With Verbose Output
```bash
cargo test -- --nocapture
```

### Single Test
```bash
cargo test test_name -- --exact
```

### Tests Requiring Database
```bash
# Set up connection
export TQ_LOGON="user:password@host:1025/database"

# Run all ignored tests
cargo test -- --ignored
```

## Test Prerequisites

### For Unit and Integration Tests
- Rust toolchain (stable)
- No external dependencies

### For Interactive Tests
- Live Teradata database access
- `TQ_LOGON` environment variable or `.env` file
- Terminal with PTY support

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TQ_LOGON` | Connection string for live tests | `user:pass@host:1025/db` |
| `TQ_HOST` | Database host (alternative to TQ_LOGON) | `myhost.com` |
| `TQ_USER` | Database user | `myuser` |
| `TQ_PASSWORD` | Database password | `secret` |
| `TQ_DATABASE` | Default database | `mydb` |

## Writing New Tests

### Unit Tests

Add tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Add tests in `tests/integration_tests.rs`:

```rust
#[test]
fn test_api_feature() {
    let result = tq::some_public_api();
    assert_eq!(result, expected);
}

#[test]
#[ignore] // Requires live database
fn test_live_database_feature() {
    // Load .env
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    // ... test with live connection
}
```

### Process Integration Tests (fd Redirection)

Some features depend on OS-level file descriptor state (`isatty()`, bytes available on stdin pipe) that cannot be controlled from within the test process. These tests spawn the `tq` binary as a subprocess with explicit `Stdio` configuration.

Add these tests in `tests/integration_tests.rs`. Use `env!("CARGO_BIN_EXE_tq")` to resolve the binary path:

```rust
#[test]
fn test_feature_with_null_stdin() {
    // Simulates: tq query "SQL" < /dev/null
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
        .args(["query", "SELECT 1"])
        .stdin(std::process::Stdio::null())    // < /dev/null
        .output()
        .expect("failed to spawn tq");
    assert!(output.status.success());
}

#[test]
fn test_feature_with_empty_pipe() {
    // Simulates: tq query "SQL" <<< "" (empty heredoc / closed pipe)
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
        .args(["query", "SELECT 1"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn tq");
    drop(child.stdin.take()); // close write end without writing — empty pipe
    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
}

#[test]
fn test_feature_with_data_on_stdin() {
    // Simulates: echo "SELECT 1" | tq query
    use std::io::Write;
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
        .args(["query"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn tq");
    if let Some(mut stdin_pipe) = child.stdin.take() {
        stdin_pipe.write_all(b"SELECT 1\n").expect("write");
    }
    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
}
```

**Key differences from `Stdio::null()` vs empty `Stdio::piped()`:**
- `Stdio::null()` opens `/dev/null` as an actual device fd (`is_terminal() == false`, `available_bytes == 0`, fd type = character device)
- `Stdio::piped()` with writer dropped creates a pipe fd (`is_terminal() == false`, `available_bytes == 0`, fd type = pipe)
- Both must be handled by the stdin detection logic; they arrive via different fd types at the OS level

**No new crates required:** `std::process::Command` and `std::process::Stdio` are in the Rust standard library.

### Interactive Tests

Add tests in `tests/interactive_tests.rs`:

```rust
#[test]
#[ignore] // Requires live database
fn test_repl_feature() {
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Send commands
    p.send_line("/help").expect("Failed to send");

    // Verify output
    let output = read_available_output(&mut p);
    assert!(output.contains("Commands"));

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}
```

## Test Fixtures

### Mock Data

For tests that don't require a live database, use helper functions to create test data:

```rust
fn make_test_result() -> QueryResult {
    let columns = vec![
        ColumnMetadata::new("id", TeradataType::Integer, false),
        ColumnMetadata::new("name", TeradataType::Varchar, true),
    ];
    let rows = vec![
        vec![Value::Integer(1), Value::String("Alice".into())],
        vec![Value::Integer(2), Value::String("Bob".into())],
    ];
    QueryResult::new(columns, rows, Duration::from_millis(100))
}
```

### Test Configuration

Create test connection configs without connecting:

```rust
fn create_test_config() -> ConnectionConfig {
    ConnectionConfig {
        host: "testhost".to_string(),
        port: 1025,
        database: "testdb".to_string(),
        user: "testuser".to_string(),
        password: None,
        logmech: LogonMechanism::Td2,
        timeout: Duration::from_secs(30),
    }
}
```

## Troubleshooting

### Tests Fail with "TQ_LOGON must be set"

These tests require a live database. Either:
1. Skip them: `cargo test` (default excludes `#[ignore]` tests)
2. Set up connection: `export TQ_LOGON="user:pass@host:1025/db"`

### Interactive Tests Show "cursor position" Errors

This is a known limitation when running reedline in expectrl's pseudo-terminal. The tests handle this gracefully and still validate core functionality.

### Tests Pass Locally but Fail in CI

Check if CI has:
- Database access (for `--ignored` tests)
- PTY support (for interactive tests)
- Correct environment variables

## Code Coverage

To measure test coverage:

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Performance Benchmarking

Performance benchmarks use the criterion crate to measure and track performance over time.

### Running Benchmarks

```bash
# Install criterion (if not already in dev-dependencies)
# Already included in Cargo.toml

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench table_formatting

# Generate HTML report
cargo bench -- --save-baseline my_baseline
```

### Benchmark Organization

Benchmarks are located in `benches/` directory:
- `benches/table_formatting.rs` - Table rendering and column width calculation benchmarks

### Sprint 32 Benchmarks

Sprint 32 added benchmarks for content-based column width calculation:
- Baseline: Schema-based width (legacy)
- Content-based: New width calculation from actual content
- Performance requirement: <10% regression or <1ms absolute for typical tables

## Continuous Integration

The test suite is designed to work in CI environments:

1. **Fast tests** (`cargo test`): Run on every commit, no external dependencies
2. **Live tests** (`cargo test -- --ignored`): Run with database access (optional)
3. **Clippy** (`cargo clippy --all-targets --all-features`): Enforced with zero warnings
4. **Benchmarks** (`cargo bench`): Optional, tracks performance over time

### CI Commands

```bash
# Required checks
cargo clippy --all-targets --all-features
cargo test --lib
cargo test --test integration_tests

# Optional (with database)
cargo test -- --ignored

# Optional (performance tracking)
cargo bench
```

## Sprint 15: Sprint 13 Validation Tests

Sprint 15 added 5 new interactive tests to achieve 100% validation of Sprint 13 features.

### New Tests Added

| Test Name | Feature Validated | Description |
|-----------|-------------------|-------------|
| `test_help_metacommand_shows_all_commands` | /help | Validates all 9 metacommands are documented |
| `test_history_persistence` | History | Validates SQL commands saved to ~/.tq_history |
| `test_multiline_sql_preserved_in_history` | Multi-line History | Validates multi-line SQL preserved as single entry |
| `test_sql_error_format_clear_and_actionable` | Error UX | Validates error messages are clear |
| `test_column_completion_after_select` | Column Completion | Validates tab shows columns in WHERE clause |

### Helper Functions Added

**`spawn_tq_repl_with_history(history_path: &Path)`** - Spawns tq REPL with custom history file for testing history persistence without affecting user's history.

### Running Sprint 15 Tests

```bash
# Run all Sprint 15 tests (requires live database)
cargo test --test interactive_tests test_help_metacommand -- --ignored
cargo test --test interactive_tests test_history_persistence -- --ignored
cargo test --test interactive_tests test_multiline_sql -- --ignored
cargo test --test interactive_tests test_sql_error_format -- --ignored
cargo test --test interactive_tests test_column_completion_after_select -- --ignored
```

## Code Coverage Baseline (Sprint 15)

**Baseline Coverage: 40.07%** (1384/3454 lines covered)

Coverage by module:

| Module | Coverage | Notes |
|--------|----------|-------|
| src/sql/parser.rs | 100% (30/30) | Full coverage |
| src/format/table.rs | 93.7% (163/174) | Well tested |
| src/format/json.rs | 98.4% (61/62) | Well tested |
| src/commands/repl/sql_context.rs | 80.4% (213/265) | Context analysis |
| src/commands/repl/state.rs | 80.5% (62/77) | REPL state |
| src/commands/repl/completer.rs | 92.6% (25/27) | Basic completer |
| src/commands/repl/mod.rs | 0% (0/160) | Main REPL loop (needs PTY) |
| src/commands/ping.rs | 0% (0/55) | Needs live DB |

**Note:** Many REPL modules have low coverage because they require a live database connection and PTY environment. The interactive tests (`--ignored`) provide validation for these modules.

### Generating Coverage Report

```bash
# Install cargo-tarpaulin (with locked dependencies for compatibility)
cargo install cargo-tarpaulin --locked

# Generate HTML report
cargo tarpaulin --lib --skip-clean --out Html

# View report
open tarpaulin-report.html
```

### Coverage Improvement Targets

For future sprints:
- **Target: 60%** - Add more unit tests for database-independent code paths
- **REPL modules**: Focus on extracting testable functions from main loop
- **Error paths**: Add tests for error handling branches
