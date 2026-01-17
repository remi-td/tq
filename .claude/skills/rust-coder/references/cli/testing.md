# Testing Rust CLI Applications

## Core Testing Principle

**"Untested software rarely works."** Start by documenting expected behavior, then write tests to verify it. This aligns with test-driven development (TDD) methodology.

## Built-in Test Framework

Rust includes a test framework using the `#[test]` attribute:

```rust
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn another_test() {
    assert!(true);
}
```

Run with:
```bash
cargo test
```

## Making Code Testable

### Problem: Direct stdout Printing

Functions that print directly are hard to test:

```rust
// Hard to test - output goes to stdout
fn find_matches(content: &str, pattern: &str) {
    for line in content.lines() {
        if line.contains(pattern) {
            println!("{}", line);
        }
    }
}
```

### Solution: Accept Writer Trait

Use `impl std::io::Write` to abstract the output destination:

```rust
use std::io::{self, Write};

fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl Write,
) -> io::Result<()> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

// In production: pass stdout
fn main() {
    let stdout = std::io::stdout();
    find_matches(content, pattern, stdout.lock()).unwrap();
}

// In tests: pass a buffer
#[test]
fn test_find_matches() {
    let mut output = Vec::new();
    find_matches("foo\nbar\nbaz", "ba", &mut output).unwrap();
    assert_eq!(output, b"bar\nbaz\n");
}
```

## Project Structure for Testability

Separate library code from binary code:

```
my-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs       # Core logic (public API)
│   ├── main.rs      # CLI entry point
│   └── ...          # Additional modules
├── tests/
│   ├── integration_test.rs
│   └── ...
├── benches/
│   └── benchmark.rs
└── examples/
    └── usage_example.rs
```

### src/lib.rs - Core Logic

```rust
use std::io::{self, Write};

pub fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl Write,
) -> io::Result<()> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_match() {
        let mut output = Vec::new();
        find_matches("hello\nworld", "world", &mut output).unwrap();
        assert_eq!(output, b"world\n");
    }

    #[test]
    fn test_no_match() {
        let mut output = Vec::new();
        find_matches("hello\nworld", "foo", &mut output).unwrap();
        assert_eq!(output, b"");
    }
}
```

### src/main.rs - CLI Entry Point

```rust
use my_cli::find_matches;
use clap::Parser;
use anyhow::{Context, Result};

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = std::fs::read_to_string(&cli.path)
        .with_context(|| format!("could not read file `{:?}`", cli.path))?;

    let stdout = std::io::stdout();
    find_matches(&content, &cli.pattern, stdout.lock())
        .context("writing output failed")?;

    Ok(())
}
```

## Unit Tests

### Basic Assertions

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
    assert_ne!(2 + 2, 5);
    assert!(2 + 2 == 4);
}

#[test]
fn test_string_contains() {
    let s = "hello world";
    assert!(s.contains("world"));
}
```

### Testing Errors

```rust
#[test]
fn test_error_case() {
    let result = parse_number("not a number");
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "divide by zero")]
fn test_panic() {
    divide(10, 0);
}
```

### Testing with Results

```rust
#[test]
fn test_with_result() -> Result<(), Box<dyn std::error::Error>> {
    let value = std::fs::read_to_string("test_file.txt")?;
    assert!(value.len() > 0);
    Ok(())
}
```

### Ignoring Tests

```rust
#[test]
#[ignore = "expensive test"]
fn test_expensive_operation() {
    // This test is skipped unless --ignored is passed
}
```

Run ignored tests:
```bash
cargo test -- --ignored
```

## Integration Tests

Test the compiled binary as an end-user would.

### Using assert_cmd

Add to `Cargo.toml`:
```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"
```

### Basic Binary Testing

Create `tests/integration_test.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("my-cli")?;

    cmd.arg("pattern")
        .arg("test_file.txt");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("expected output"));

    Ok(())
}

#[test]
fn test_missing_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("my-cli")?;

    cmd.arg("pattern")
        .arg("nonexistent.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent.txt"));

    Ok(())
}
```

### Testing with Temporary Files

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_with_temp_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary file
    let temp = assert_fs::TempDir::new()?;
    let input_file = temp.child("input.txt");
    input_file.write_str("hello\nworld\nfoo")?;

    // Run command
    let mut cmd = Command::cargo_bin("my-cli")?;
    cmd.arg("world")
        .arg(input_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("world"));

    // Temporary file is automatically deleted when temp goes out of scope
    Ok(())
}
```

### Testing Multiple Scenarios

```rust
use assert_cmd::Command;

#[test]
fn test_various_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Test --help
    Command::cargo_bin("my-cli")?
        .arg("--help")
        .assert()
        .success();

    // Test --version
    Command::cargo_bin("my-cli")?
        .arg("--version")
        .assert()
        .success();

    // Test invalid arguments
    Command::cargo_bin("my-cli")?
        .assert()
        .failure();

    Ok(())
}
```

### Predicates for Complex Assertions

```rust
use predicates::prelude::*;

#[test]
fn test_with_predicates() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("my-cli")?;

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("line 1")
                .and(predicate::str::contains("line 2"))
                .and(predicate::str::is_match(r"\d{3}")?)  // Regex
        );

    Ok(())
}
```

## Testing Strategy

### What to Test

**DO test:**
- Core business logic (unit tests)
- Edge cases and boundary conditions
- Error handling paths
- Integration with the file system (using temp files)
- Command-line argument parsing
- Exit codes for different scenarios

**DON'T test:**
- Auto-generated help text formatting
- Standard library functionality
- Third-party library internals
- Obvious pass-throughs

### Test Organization

```
tests/
├── integration_test.rs          # Main integration tests
├── cli_arguments_test.rs        # Argument parsing tests
├── error_handling_test.rs       # Error case tests
└── fixtures/
    ├── sample_input.txt         # Test data
    └── expected_output.txt
```

## Property-Based Testing

For complex logic, use `proptest`:

```toml
[dev-dependencies]
proptest = "1.0"
```

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_reversing_twice_is_identity(s in ".*") {
        let reversed: String = s.chars().rev().collect();
        let double_reversed: String = reversed.chars().rev().collect();
        assert_eq!(s, double_reversed);
    }

    #[test]
    fn test_parse_valid_numbers(n in 0i32..1000) {
        let s = n.to_string();
        assert_eq!(s.parse::<i32>().unwrap(), n);
    }
}
```

## Benchmarking

Create `benches/benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_cli::find_matches;

fn benchmark_find_matches(c: &mut Criterion) {
    let content = "line 1\nline 2\nline 3\n".repeat(1000);

    c.bench_function("find_matches", |b| {
        b.iter(|| {
            let mut output = Vec::new();
            find_matches(
                black_box(&content),
                black_box("line"),
                &mut output
            ).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_find_matches);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmark"
harness = false
```

Run benchmarks:
```bash
cargo bench
```

## Test Coverage

### Using cargo-tarpaulin (Linux)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Using cargo-llvm-cov

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

## Continuous Integration

### GitHub Actions Example

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## Debugging Tests

### Show println! Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_name
```

### Run Tests Matching Pattern

```bash
cargo test pattern_
```

### Show Test Names Without Running

```bash
cargo test -- --list
```

## Best Practices

1. **Write tests first** (TDD)
   - Define expected behavior
   - Write failing test
   - Implement until test passes

2. **Keep tests independent**
   - Each test should work in isolation
   - Use temporary files for file system tests
   - Clean up resources

3. **Test error cases**
   - Missing files
   - Invalid input
   - Permission errors
   - Edge cases

4. **Make code testable**
   - Accept `impl Write` instead of printing
   - Return `Result` instead of panicking
   - Separate logic from I/O

5. **Use descriptive test names**
   ```rust
   #[test]
   fn test_returns_error_when_file_not_found() { }

   #[test]
   fn test_parses_valid_config_successfully() { }
   ```

6. **Group related tests in modules**
   ```rust
   #[cfg(test)]
   mod parse_tests {
       #[test]
       fn test_parse_valid() { }

       #[test]
       fn test_parse_invalid() { }
   }
   ```

7. **Use test helpers**
   ```rust
   #[cfg(test)]
   mod tests {
       fn setup() -> TestContext {
           // Common setup code
       }

       #[test]
       fn test_something() {
           let ctx = setup();
           // Test code
       }
   }
   ```

## Summary

- Use `#[test]` for unit tests in `src/lib.rs`
- Use `assert_cmd` for integration tests in `tests/`
- Make code testable by accepting `impl Write`
- Use temporary files with `assert_fs`
- Test error cases and edge conditions
- Run `cargo test` frequently during development
- Aim for high test coverage of critical paths
- Keep tests fast and independent
