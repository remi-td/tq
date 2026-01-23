# Testing Tools and Infrastructure

This document describes the testing tools, utilities, and infrastructure used for validating tq.

## Testing Dependencies

### Core Testing Framework

**cargo test** - Rust's built-in test framework
- No additional dependencies required for basic testing
- Supports unit tests, integration tests, doc tests
- Parallel execution by default
- Built-in assertions and test organization

### Test Utilities

Listed in `Cargo.toml` `[dev-dependencies]`:

```toml
[dev-dependencies]
assert_cmd = "2.0"      # CLI testing
predicates = "3.0"      # Flexible assertions
tempfile = "3.8"        # Temporary files/directories
serde_json = "1.0"      # JSON parsing for tests
```

**assert_cmd**: Command-line testing
```rust
use assert_cmd::Command;

#[test]
fn test_query_command() {
    Command::cargo_bin("tq")
        .unwrap()
        .arg("query")
        .arg("SELECT 1")
        .assert()
        .success();
}
```

**predicates**: Flexible assertions
```rust
use predicates::prelude::*;

#[test]
fn output_contains_expected_text() {
    Command::cargo_bin("tq")
        .unwrap()
        .arg("query")
        .arg("SELECT 1")
        .assert()
        .stdout(predicate::str::contains("1"));
}
```

**tempfile**: Temporary test files
```rust
use tempfile::TempDir;

#[test]
fn test_config_file() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");
    // Test creates files in isolated directory
}
```

## Coverage Tools

### cargo-tarpaulin

**Purpose**: Measure code coverage for Rust projects

**Installation**:
```bash
cargo install cargo-tarpaulin
```

**Basic usage**:
```bash
# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Generate Lcov format (for editors)
cargo tarpaulin --out Lcov --output-dir coverage

# Generate multiple formats
cargo tarpaulin --out Html --out Lcov --out Xml
```

**Configuration** (`.tarpaulin.toml`):
```toml
[default]
# Exclude integration and interactive tests from coverage
exclude-files = [
    "tests/*",
]
```

**Limitations**:
- Only measures unit test coverage
- Does not include interactive test coverage
- May report inaccurate results for async code

### Alternative: cargo-llvm-cov

**Purpose**: Alternative coverage tool using LLVM

**Installation**:
```bash
cargo install cargo-llvm-cov
```

**Usage**:
```bash
cargo llvm-cov --html
```

## Interactive Test Framework

### REPL Test Harness

**Location**: `tests/interactive_tests.rs`

**Components**:

1. **ReplProcess**: Manages REPL subprocess
```rust
struct ReplProcess {
    child: std::process::Child,
    stdin: ChildStdin,
    output_reader: BufReader<ChildStdout>,
}

impl ReplProcess {
    fn new() -> Result<Self> { /* spawn tq repl */ }
    fn send_line(&mut self, line: &str) -> Result<()> { /* send input */ }
    fn send_key(&mut self, key: Key) -> Result<()> { /* send tab, enter, etc */ }
    fn read_until_prompt(&mut self) -> Result<String> { /* read output */ }
    fn wait_for(&mut self, text: &str, timeout: Duration) -> Result<()> { /* wait */ }
}
```

2. **Output Parsing**: Extract meaningful data from REPL output
```rust
fn extract_completions(output: &str) -> Vec<String> {
    // Parse tab completion suggestions from output
}

fn extract_table_rows(output: &str) -> Vec<Vec<String>> {
    // Parse table display into structured data
}
```

3. **Keyboard Simulation**:
```rust
enum Key {
    Tab,
    Enter,
    Up,
    Down,
    Left,
    Right,
    CtrlC,
    CtrlD,
}
```

**Example test**:
```rust
#[test]
#[ignore]
fn test_tab_completion_context_aware() {
    let mut repl = ReplProcess::new().unwrap();

    // Type partial query
    repl.send_line("SELECT * FROM cu").unwrap();

    // Press Tab
    repl.send_key(Key::Tab).unwrap();

    // Read suggestions
    let output = repl.read_until_prompt().unwrap();
    let completions = extract_completions(&output);

    // Verify semantic correctness
    assert!(completions.contains(&"customer".to_string()));
    assert!(!completions.iter().any(|c| c.contains("SELECT")));
}
```

### Test Database Setup

**Purpose**: Provide consistent test database environment

**Location**: `tests/fixtures/test_db.sql`

**Schema**:
```sql
-- Test database setup
CREATE DATABASE test_tq;

CREATE TABLE test_tq.sample_data (
    id INTEGER,
    name VARCHAR(100),
    value DECIMAL(10,2)
);

INSERT INTO test_tq.sample_data VALUES
    (1, 'Test Item 1', 100.50),
    (2, 'Test Item 2', 200.75),
    (3, 'Test Item 3', 300.00);
```

**Setup script** (`tests/tools/setup_test_db.sh`):
```bash
#!/bin/bash
# Setup test database for tq testing

# Load test connection
source .env

# Create test database
tq query "CREATE DATABASE test_tq"

# Load schema
tq query < tests/fixtures/test_db.sql

echo "Test database ready"
```

## Test Fixtures

### File Structure

```
tests/fixtures/
├── test_db.sql           # Database schema
├── sample_queries/
│   ├── simple.sql        # SELECT 1
│   ├── complex.sql       # Multi-table join
│   └── error.sql         # Invalid SQL
└── expected_outputs/
    ├── simple.json       # Expected JSON output
    ├── simple.csv        # Expected CSV output
    └── simple.txt        # Expected table output
```

### Using Fixtures in Tests

```rust
#[test]
fn test_query_from_file() {
    let query = include_str!("../fixtures/sample_queries/simple.sql");
    let expected = include_str!("../fixtures/expected_outputs/simple.json");

    let output = execute_query(query);
    assert_eq!(output.trim(), expected.trim());
}
```

## Test Utilities

### Custom Test Helpers

**Location**: `tests/common/mod.rs`

```rust
// Common test helpers
pub mod common {
    use std::process::Command;

    pub fn run_tq(args: &[&str]) -> (String, String, i32) {
        let output = Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(args)
            .output()
            .expect("Failed to run tq");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        (stdout, stderr, code)
    }

    pub fn assert_successful_query(query: &str) {
        let (stdout, stderr, code) = run_tq(&["query", query]);
        assert_eq!(code, 0, "Query failed: {}", stderr);
        assert!(!stdout.is_empty(), "No output produced");
    }

    pub fn test_connection_config() -> ConnectionConfig {
        // Returns standard test configuration
        ConnectionConfig {
            host: env::var("TQ_TEST_HOST").unwrap_or("localhost".into()),
            port: 1025,
            database: "test_tq".into(),
            // ... other fields
        }
    }
}
```

**Usage in tests**:
```rust
mod common;
use common::*;

#[test]
fn test_simple_query() {
    assert_successful_query("SELECT 1");
}
```

## Continuous Integration Tools

### GitHub Actions

**Configuration**: `.github/workflows/test.yml`

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v2

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable

    - name: Cache cargo
      uses: actions/cache@v2
      with:
        path: ~/.cargo
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run unit tests
      run: cargo test --lib

    - name: Run integration tests
      run: cargo test --test integration_test
      env:
        TQ_LOGON: ${{ secrets.TEST_DB_CONNECTION }}

    - name: Check coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml

    - name: Upload coverage
      uses: codecov/codecov-action@v2
```

### Pre-commit Hooks

**Installation**:
```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks
pre-commit install
```

**Configuration** (`.pre-commit-config.yaml`):
```yaml
repos:
- repo: local
  hooks:
  - id: cargo-test
    name: cargo test
    entry: cargo test --lib
    language: system
    pass_filenames: false

  - id: cargo-fmt
    name: cargo fmt
    entry: cargo fmt -- --check
    language: system
    pass_filenames: false

  - id: cargo-clippy
    name: cargo clippy
    entry: cargo clippy -- -D warnings
    language: system
    pass_filenames: false
```

## Performance Testing Tools

### Benchmarking

**Using criterion** (for micro-benchmarks):

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "query_parsing"
harness = false
```

```rust
// benches/query_parsing.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_connection_string(c: &mut Criterion) {
    c.bench_function("parse connection string", |b| {
        b.iter(|| {
            parse_connection_string(black_box("user:pass@host:1025/db"))
        });
    });
}

criterion_group!(benches, bench_parse_connection_string);
criterion_main!(benches);
```

**Run benchmarks**:
```bash
cargo bench
```

## Testing Best Practices

### Test Data Management

1. **Use realistic data**: Test with data similar to production
2. **Keep data minimal**: Only what's needed for the test
3. **Clean up after tests**: Use `Drop` trait or defer cleanup
4. **Version control fixtures**: Check test data into git

### Test Organization

1. **Group related tests**: Use `mod tests` for organization
2. **Name tests descriptively**: `test_feature_behavior_scenario`
3. **One test per behavior**: Don't combine unrelated assertions
4. **Document complex tests**: Add comments explaining "why"

### CI/CD Integration

1. **Fast feedback**: Run unit tests first (fastest)
2. **Parallel execution**: Run independent tests concurrently
3. **Fail fast**: Stop on first failure in CI
4. **Cache dependencies**: Cache cargo build artifacts

## Future Tools

Potential tools to add:

- **Mutation testing**: `cargo-mutants` to validate test effectiveness
- **Property testing**: `proptest` for generative testing
- **Fuzzing**: `cargo-fuzz` for input fuzzing
- **Visual regression**: Screenshot comparison for REPL output
- **Load testing**: Database query performance under load
