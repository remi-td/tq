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

## Continuous Integration

The test suite is designed to work in CI environments:

1. **Fast tests** (`cargo test`): Run on every commit, no external dependencies
2. **Live tests** (`cargo test -- --ignored`): Run with database access (optional)
3. **Clippy** (`cargo clippy --all-targets --all-features`): Enforced with zero warnings

### CI Commands

```bash
# Required checks
cargo clippy --all-targets --all-features
cargo test --lib
cargo test --test integration_tests

# Optional (with database)
cargo test -- --ignored
```
