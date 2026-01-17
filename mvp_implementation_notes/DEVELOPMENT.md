# Development Guidelines

This document outlines the code principles, patterns, and best practices used in the `tq` project. Following these guidelines ensures consistency, maintainability, and robustness across the codebase.

## Table of Contents

1. [Core Design Philosophy](#core-design-philosophy)
2. [Code Organization](#code-organization)
3. [Type Safety and API Design](#type-safety-and-api-design)
4. [Error Handling](#error-handling)
5. [Naming Conventions](#naming-conventions)
6. [Testing Strategy](#testing-strategy)
7. [Documentation Standards](#documentation-standards)
8. [Performance Considerations](#performance-considerations)
9. [Security Best Practices](#security-best-practices)
10. [Code Style](#code-style)

---

## Core Design Philosophy

### Library-First Architecture

The project follows a **library-first design** where business logic resides in `src/lib.rs` and related modules, while `src/main.rs` serves as a thin CLI wrapper. This approach provides:

- **Testability**: Core logic can be tested independently of CLI parsing
- **Reusability**: Business logic can be consumed by other interfaces (GUI, web server, etc.)
- **Separation of concerns**: Clear boundary between application logic and user interface

**Example:**
```rust
// ✅ Good: Logic in library
// src/db.rs
pub fn execute_query(&self, query: &str) -> Result<QueryResults> {
    // Business logic here
}

// src/main.rs
fn main() -> Result<()> {
    let client = DatabaseClient::new(config, None)?;
    let results = client.execute_query(&query)?;  // Thin wrapper
}
```

### Expression-Oriented Programming

Prefer returning values from expressions rather than using statement-based approaches with separate assignments.

**Example:**
```rust
// ✅ Good: Expression-oriented
let password_override = cli.password_file
    .as_ref()
    .map(|file| read_password(file))
    .transpose()?;

// ❌ Avoid: Statement-based
let password_override;
if let Some(file) = &cli.password_file {
    password_override = Some(read_password(file)?);
} else {
    password_override = None;
}
```

### Minimal Complexity

Only add the complexity needed for the current requirements. Avoid:
- Over-engineering for hypothetical future needs
- Premature abstractions
- Unnecessary error handling for impossible scenarios
- Feature flags when simple code changes suffice

---

## Code Organization

### Module Structure

```
src/
  lib.rs          # Public library API and module declarations
  main.rs         # CLI entry point (thin wrapper)
  cli.rs          # CLI argument definitions using clap
  connection.rs   # Connection configuration and parsing
  db.rs           # Database client and query execution
  error.rs        # Error types using thiserror
  format.rs       # Output formatters (table, JSON, CSV)
```

### Module Responsibilities

Each module has a **single, well-defined responsibility**:

- **cli.rs**: Command-line interface structure using `clap` with subcommands
- **connection.rs**: Connection string parsing, validation, and configuration
- **db.rs**: Database client, query execution, and result handling
- **error.rs**: Structured error types with granular variants
- **format.rs**: Result formatting logic separated from business logic

### Public API Design

- Use `pub` sparingly; prefer `pub(crate)` for internal types
- Re-export commonly used types in `lib.rs` for convenience
- Keep the public API surface small and focused

**Example:**
```rust
// src/lib.rs
pub use cli::{Command, OutputFormat};
pub use connection::{ConnectionConfig, LogonMechanism};
pub use db::{DatabaseClient, QueryResults, Row};
pub use error::{Result, TqError};
```

---

## Type Safety and API Design

### Newtype Pattern for Domain Modeling

Use newtypes to add semantic meaning and type safety to primitive types.

**Example:**
```rust
// ✅ Good: Type-safe row abstraction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(Vec<String>);

impl Row {
    pub fn new(values: Vec<String>) -> Self { Self(values) }
    pub fn get(&self, index: usize) -> Option<&str> {
        self.0.get(index).map(|s| s.as_str())
    }
}

// ❌ Avoid: Primitive type alias provides no safety
pub type Row = Vec<String>;
```

### Encapsulation

Keep fields private and provide controlled access through methods.

**Example:**
```rust
// ✅ Good: Encapsulated design
pub struct QueryResults {
    rows: Vec<Row>,  // Private field
}

impl QueryResults {
    pub fn new(rows: Vec<Row>) -> Self { Self { rows } }
    pub fn iter(&self) -> std::slice::Iter<Row> { self.rows.iter() }
    pub fn get(&self, index: usize) -> Option<&Row> { self.rows.get(index) }
}

// ❌ Avoid: Public fields break encapsulation
pub struct QueryResults {
    pub rows: Vec<Row>,  // Exposes internal structure
}
```

### Enum-Based Command Structure

Use enums with associated data instead of boolean flags + optional fields for commands.

**Example:**
```rust
// ✅ Good: Type-safe command structure
#[derive(Subcommand, Debug)]
pub enum Command {
    Ping,
    Query {
        query: String,
        format: OutputFormat,
    },
}

// ❌ Avoid: Boolean flags require runtime validation
pub struct Cli {
    pub ping: bool,
    pub query: Option<String>,
}
```

### Iterator Implementations

Implement standard iterator traits for collection-like types.

**Example:**
```rust
// Consuming iterator
impl IntoIterator for QueryResults {
    type Item = Row;
    type IntoIter = std::vec::IntoIter<Row>;
    fn into_iter(self) -> Self::IntoIter { self.rows.into_iter() }
}

// Borrowing iterator
impl<'a> IntoIterator for &'a QueryResults {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;
    fn into_iter(self) -> Self::IntoIter { self.rows.iter() }
}
```

---

## Error Handling

### Structured Error Types

Use `thiserror` for library errors with **granular variants** that distinguish between different failure modes.

**Example:**
```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TqError {
    #[error("Invalid connection string format: {0}")]
    InvalidConnectionString(String),

    #[error("Connection failed to {host}: {message}")]
    Connection { host: String, message: String },

    #[error("Failed to load Teradata driver from '{path}': {message}")]
    DriverLoad { path: String, message: String },

    #[error("Failed to execute query: {0}")]
    QueryExecution(String),

    #[error("Failed to fetch row {row_num}: {message}")]
    RowFetch { row_num: usize, message: String },
}
```

### Error Handling Principles

1. **Use `Result<T>` instead of panicking** - Let callers handle errors
2. **Provide context** - Use `anyhow::Context` in application code
3. **Map errors at boundaries** - Convert external errors to domain errors
4. **Derive `PartialEq` for testability** - Enable error assertions in tests

**Example:**
```rust
// ✅ Good: Granular error with context
teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, query, bind_values)
    .map_err(|e| TqError::QueryExecution(e.to_string()))?;

// ❌ Avoid: Generic error loses information
teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, query, bind_values)
    .map_err(|e| TqError::Database(format!("Something failed: {}", e)))?;
```

### Error Context Propagation

Use `anyhow` in application code for easy context chaining.

**Example:**
```rust
let config = ConnectionConfig::parse(&cli.logon, &cli.logmech, password_override)
    .context("Failed to parse connection string")?;

let client = DatabaseClient::new(config, cli.driver_lib_dir)
    .context("Failed to create database client")?;
```

---

## Naming Conventions

Follow Rust's official naming conventions from RFC 430:

- **`UpperCamelCase`** - Types, traits, enum variants
- **`snake_case`** - Functions, methods, fields, local variables, modules
- **`SCREAMING_SNAKE_CASE`** - Constants and statics

**Examples:**
```rust
// Types and traits
pub struct ConnectionConfig { }
pub trait DatabaseClient { }
pub enum LogonMechanism { TD2, LDAP }

// Functions and methods
pub fn parse_connection_string() { }
fn validate_host(host: &str) -> Result<()> { }

// Constants
const DEFAULT_PORT: u16 = 1025;
static DRIVER_LOADED: OnceLock<()> = OnceLock::new();
```

---

## Testing Strategy

### Test Organization

- **Unit tests**: In-module tests using `#[cfg(test)]` for private API testing
- **Integration tests**: In `tests/` directory for public API testing
- **Documentation tests**: Executable examples in doc comments

### Test Coverage

Aim for comprehensive coverage of:
- Public APIs and their contracts
- Error conditions and edge cases
- Type conversions and data transformations
- Format output correctness

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_creation_and_access() {
        let row = Row::new(vec!["col1".to_string(), "col2".to_string()]);
        assert_eq!(row.len(), 2);
        assert_eq!(row.get(0), Some("col1"));
        assert_eq!(row.get(2), None);
    }

    #[test]
    fn test_query_results_empty() {
        let results = QueryResults::new(vec![]);
        assert!(results.is_empty());
        assert_eq!(results.row_count(), 0);
    }
}
```

### Test Naming

Use descriptive test names that indicate:
1. What is being tested
2. What the expected behavior is

**Pattern:** `test_<component>_<scenario>`

Examples:
- `test_row_creation_and_access`
- `test_connection_config_invalid_port`
- `test_format_csv_with_special_characters`

---

## Documentation Standards

### Public API Documentation

Every public item should have documentation comments (`///`) explaining:
- **Purpose**: What the item does
- **Arguments**: For functions/methods
- **Returns**: What is returned
- **Errors**: When errors occur
- **Examples**: Usage examples where helpful

**Example:**
```rust
/// Execute a SQL query and return the results
///
/// This method establishes a connection, executes the query,
/// fetches all results, and closes the connection.
///
/// # Arguments
/// * `query` - The SQL query to execute
///
/// # Returns
/// - `Ok(QueryResults)` with the query results
/// - `Err(TqError)` if the connection or query fails
///
/// # Example
/// ```no_run
/// # use tq::{ConnectionConfig, DatabaseClient};
/// # let config = ConnectionConfig::parse("user:pass@host:1025/db", "TD2", None)?;
/// let client = DatabaseClient::new(config, None)?;
/// let results = client.execute_query("SELECT * FROM my_table")?;
/// println!("Retrieved {} rows", results.row_count());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn execute_query(&self, query: &str) -> Result<QueryResults> {
    // Implementation
}
```

### Module-Level Documentation

Use inner doc comments (`//!`) at the top of modules to explain:
- Module purpose and responsibilities
- High-level design decisions
- Usage patterns

**Example:**
```rust
//! Output formatting for query results
//!
//! This module provides formatters for displaying query results in different formats:
//! - Table: Human-readable ASCII table
//! - JSON: Machine-parseable JSON array
//! - CSV: Comma-separated values with proper escaping
```

---

## Performance Considerations

### Avoid Unnecessary Allocations

- Reuse computed values instead of recomputing
- Use references (`&str`) instead of owned strings (`String`) where possible
- Leverage iterator chains instead of intermediate collections

**Example:**
```rust
// ✅ Good: Compute separator once
let separator = build_separator(&widths);
output.push_str(&separator);
// ... print rows ...
output.push_str(&separator);

// ❌ Avoid: Recompute separator
output.push_str(&build_separator(&widths));
// ... print rows ...
output.push_str(&build_separator(&widths));
```

### Avoid Unnecessary Clones

Only clone when ownership transfer is genuinely needed.

**Example:**
```rust
// ✅ Good: Move value directly
let client = DatabaseClient::new(config, cli.driver_lib_dir)?;

// ❌ Avoid: Unnecessary clone
let client = DatabaseClient::new(config.clone(), cli.driver_lib_dir)?;
```

### Use Standard Library Synchronization Primitives

Prefer standard library types over external dependencies when available.

**Example:**
```rust
// ✅ Good: Use stdlib (Rust 1.70+)
use std::sync::OnceLock;
static DRIVER_LOADED: OnceLock<()> = OnceLock::new();

// ❌ Avoid: External dependency for stdlib feature
use once_cell::sync::OnceCell;
static DRIVER_LOADED: OnceCell<()> = OnceCell::new();
```

---

## Security Best Practices

### Credential Handling

1. **Never log passwords** - Use `secrecy::Secret<String>` for passwords
2. **Redact in Debug output** - Implement custom `Debug` to hide sensitive data
3. **Never accept passwords as CLI flags** - They leak to `ps` output
4. **Validate file permissions** - Warn on insecure password files (Unix: 0600)

**Example:**
```rust
use secrecy::{Secret, ExposeSecret};

pub struct ConnectionConfig {
    pub user: String,
    pub password: Secret<String>,  // Zeroed on drop
    pub host: String,
    // ...
}

impl std::fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("user", &self.user)
            .field("password", &"[REDACTED]")  // Never log passwords
            .field("host", &self.host)
            .finish()
    }
}
```

### Input Validation

Validate all external input at system boundaries.

**Example:**
```rust
fn validate_identifier(s: &str, name: &str) -> Result<()> {
    if s.is_empty() {
        return Err(TqError::InvalidConnectionString(
            format!("{} cannot be empty", name)
        ));
    }
    if s.contains(['\0', '\n', '\r']) {
        return Err(TqError::InvalidConnectionString(
            format!("{} contains invalid characters", name)
        ));
    }
    Ok(())
}
```

### Build Script Safety

Avoid `unwrap()` in build scripts; use proper error handling.

**Example:**
```rust
// ✅ Good: Propagate errors properly
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let target_dir = PathBuf::from(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or("Failed to determine target directory")?
        .to_path_buf();
    // ...
    Ok(())
}

// ❌ Avoid: Panics during build
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();  // Can panic
}
```

---

## Code Style

### Indentation and Formatting

- **4 spaces** for indentation (never tabs)
- **100 characters** maximum line length
- **Block indentation** over visual alignment
- **Trailing commas** in multi-line constructs
- Run `cargo fmt` before committing

### Pattern Matching

Use pattern matching over `if let` chains when handling multiple variants.

**Example:**
```rust
// ✅ Good: Clear pattern matching
match cli.command {
    Command::Ping => handle_ping(&client)?,
    Command::Query { query, format } => handle_query(&client, &query, format)?,
}

// ❌ Avoid: Nested if-let chains
if let Command::Ping = cli.command {
    handle_ping(&client)?;
} else if let Command::Query { query, format } = cli.command {
    handle_query(&client, &query, format)?;
}
```

### Function Length

Keep functions short and focused. If a function is too complex:
1. Extract helper functions
2. Break into logical steps
3. Consider refactoring into a method

### Import Organization

Organize imports in this order:
1. Standard library (`std`, `core`)
2. External crates (alphabetically)
3. Internal modules (`crate::`, `super::`, `self::`)

**Example:**
```rust
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;

use crate::connection::ConnectionConfig;
use crate::error::TqError;
```

---

## Development Workflow

### Before Committing

Run these commands to ensure code quality:

```bash
# Format code
cargo fmt

# Lint with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test

# Check documentation
cargo doc --no-deps
```

### Continuous Integration

The project should have CI checks for:
- `cargo fmt --check` - Formatting compliance
- `cargo clippy` - Linting with warnings as errors
- `cargo test` - All tests passing
- `cargo build --release` - Release build succeeds

---

## Summary of Key Principles

1. **Library-first design** - Separate business logic from CLI
2. **Type safety** - Use newtypes and enums for domain modeling
3. **Encapsulation** - Private fields with controlled access
4. **Granular errors** - Specific error variants for different failures
5. **Expression-oriented** - Prefer expressions over statements
6. **Comprehensive testing** - Unit, integration, and doc tests
7. **Clear documentation** - Document all public APIs with examples
8. **Security-conscious** - Protect credentials, validate input
9. **Performance-aware** - Avoid unnecessary allocations and clones
10. **Consistent style** - Follow Rust conventions and run `cargo fmt`

By following these guidelines, we maintain a codebase that is robust, maintainable, secure, and idiomatic Rust.
