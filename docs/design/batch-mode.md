# Batch Mode Technical Design

This document describes the technical architecture for batch mode features in tq, explaining how file output, transaction control, and multi-statement execution are implemented.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [File Output (--output flag)](#file-output---output-flag)
3. [Transaction Control (--atomic flag)](#transaction-control---atomic-flag)
4. [Integration Test Driver Loading](#integration-test-driver-loading)
5. [Code Organization](#code-organization)
6. [Error Handling Patterns](#error-handling-patterns)

---

## Architecture Overview

Batch mode builds on tq's one-shot execution model, extending it to handle multiple statements and file operations while maintaining the same connection lifecycle: connect, execute, disconnect.

### Design Principles

1. **Fail-Fast**: Stop on first error, report context
2. **Atomic File Writes**: Use temp file + rename pattern
3. **Stream Results**: Never buffer entire result sets in memory
4. **Clear Ownership**: File handles owned by caller, closed on drop

### Data Flow

```
SQL Input → Statement Parser → Sequential Executor → Result Formatter → Output Destination
     │                              │                       │                │
     │                              │                       │                ├─ stdout (default)
     │                              │                       │                └─ file (--output)
     │                              │                       │
     │                              │                       └─ table/csv/json
     │                              │
     │                              └─ Optional transaction wrapper (--atomic)
     │
     └─ argument / file / stdin
```

---

## File Output (--output flag)

### Implementation Approach

The `--output` flag redirects query results to a file with better error handling and status reporting compared to shell redirection.

#### CLI Extension

```rust
// src/cli.rs - QueryArgs extension
#[derive(Parser, Debug)]
pub struct QueryArgs {
    // ... existing fields ...

    /// Write output to file instead of stdout
    ///
    /// Uses atomic file writing (temp file + rename) to prevent
    /// partial writes on error. If the file exists, it will be
    /// overwritten.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

#### Atomic File Writing Pattern

The implementation uses a temp-file-then-rename pattern for atomic writes:

```rust
// src/commands/query.rs - File output implementation
use std::fs::{File, rename};
use std::io::{BufWriter, Write};
use tempfile::NamedTempFile;

/// Execute query and write to file atomically
pub fn execute_to_file<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    status_writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    let output_path = args.output.as_ref().ok_or_else(|| {
        TqError::InternalError("execute_to_file called without output path".to_string())
    })?;

    // Create temp file in same directory (ensures same filesystem for rename)
    let parent_dir = output_path.parent().unwrap_or(Path::new("."));
    let temp_file = NamedTempFile::new_in(parent_dir)
        .map_err(|e| TqError::FileWriteError {
            path: output_path.clone(),
            source: e,
        })?;

    // Execute query and write to temp file
    let mut writer = BufWriter::new(temp_file.as_file());
    execute_query_to_writer(client, args, &mut writer, use_color, verbose)?;
    writer.flush()?;

    // Atomic rename to final destination
    temp_file.persist(output_path)
        .map_err(|e| TqError::FileWriteError {
            path: output_path.clone(),
            source: e.error,
        })?;

    // Report success to status writer (stderr)
    writeln!(status_writer, "Wrote {} rows to {}", row_count, output_path.display())?;
    Ok(())
}
```

#### Current Implementation Status

The current implementation in `src/commands/query.rs` already has `execute_to_file` but uses direct file creation rather than atomic writes. The improvement needed is:

1. Add `tempfile` dependency to `Cargo.toml`
2. Replace direct `File::create` with `NamedTempFile::new_in`
3. Use `persist()` for atomic rename

#### Error Handling

File output errors are mapped to structured error types:

| Scenario | Error Type | User Message |
|----------|-----------|--------------|
| Cannot create temp file | `FileWriteError` | "Cannot write to directory..." |
| Write fails mid-stream | `IoError` | "Write failed: ..." |
| Rename fails | `FileWriteError` | "Cannot complete file write..." |
| Disk full | `IoError` | "No space left on device" |

---

## Transaction Control (--atomic flag)

### Implementation Approach

The `--atomic` flag wraps multi-statement execution in a transaction, providing automatic rollback on error.

#### CLI Extension

```rust
// src/cli.rs - QueryArgs extension
#[derive(Parser, Debug)]
pub struct QueryArgs {
    // ... existing fields ...

    /// Wrap statements in a transaction (batch mode only)
    ///
    /// Executes BEGIN TRANSACTION before the first statement and
    /// COMMIT on success. If any statement fails, automatically
    /// executes ROLLBACK before reporting the error.
    ///
    /// Note: Only applies to multi-statement execution from
    /// --file or stdin. Single statement queries are unaffected.
    #[arg(long)]
    pub atomic: bool,
}
```

#### Transaction Wrapper Implementation

```rust
// src/commands/query.rs - Transaction control

/// Execute batch with optional transaction wrapper
fn execute_batch<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    args: &QueryArgs,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    let statements = parse_statements(sql);
    let total_count = statements.len();

    // Begin transaction if atomic mode requested
    if args.atomic && total_count > 1 {
        if verbose {
            eprintln!("BEGIN TRANSACTION (--atomic mode)");
        }
        client.execute("BEGIN TRANSACTION")?;
    }

    // Execute statements with fail-fast behavior
    let result = execute_statements_sequentially(
        client, &statements, args, writer, use_color, verbose
    );

    // Handle transaction completion
    if args.atomic && total_count > 1 {
        match &result {
            Ok(_) => {
                if verbose {
                    eprintln!("COMMIT (all statements succeeded)");
                }
                client.execute("COMMIT")?;
            }
            Err(_) => {
                if verbose {
                    eprintln!("ROLLBACK (statement failed)");
                }
                // Best effort rollback - don't mask original error
                if let Err(rollback_err) = client.execute("ROLLBACK") {
                    log::warn!("Rollback failed: {}", rollback_err);
                }
            }
        }
    }

    result
}
```

#### Teradata Transaction Semantics

Teradata transaction behavior considerations:

1. **ANSI Mode vs BTET Mode**: Teradata supports both modes
   - ANSI: Auto-commit after each statement
   - BTET (Begin Transaction/End Transaction): Explicit transactions

2. **Nested Transactions**: Teradata does not support nested transactions
   - If user's SQL contains explicit `BEGIN TRANSACTION`, detect and warn

3. **DDL Behavior**: Some DDL auto-commits in Teradata
   - `CREATE TABLE` may force commit
   - Document this limitation

#### Transaction State Tracking

```rust
/// Transaction state for batch execution
#[derive(Debug, Clone, Copy, PartialEq)]
enum TransactionState {
    /// No transaction active
    None,
    /// Transaction started by --atomic flag
    AutoStarted,
    /// Transaction detected in user SQL (don't interfere)
    UserManaged,
}

/// Detect if user SQL contains transaction control
fn detect_user_transaction(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();
    sql_upper.contains("BEGIN TRANSACTION")
        || sql_upper.contains("BT;")
        || sql_upper.contains("BEGIN TRAN")
}
```

#### Error Messages

Transaction-specific error messages:

```rust
// When --atomic fails to begin transaction
TqError::TransactionError {
    operation: "BEGIN",
    message: "Failed to start transaction",
    source: Some(e),
}

// When --atomic conflicts with user transaction
TqError::InvalidConfig(
    "Cannot use --atomic with SQL containing explicit BEGIN TRANSACTION.\n\
     Either remove --atomic or remove BEGIN/COMMIT from your SQL."
)

// When commit fails
TqError::TransactionError {
    operation: "COMMIT",
    message: "All statements succeeded but COMMIT failed",
    source: Some(e),
}
```

---

## Integration Test Driver Loading

### Problem Analysis

The `teradatarustapi` library uses global state for driver loading via `load_driver()`. When multiple integration tests run in parallel:

1. Thread A calls `load_driver("/path/to/lib")`
2. Thread B calls `load_driver("/path/to/lib")` simultaneously
3. The Go-based driver has internal state that gets corrupted

Current workaround: `--test-threads=1` forces sequential execution.

### Root Cause Investigation

The driver loading issue stems from the `teradatarustapi` crate's design:

```rust
// Current code in src/db/client.rs
static DRIVER_LOADED: OnceLock<()> = OnceLock::new();

fn ensure_driver_loaded(&self) -> Result<()> {
    if DRIVER_LOADED.get().is_some() {
        return Ok(());
    }

    teradatarustapi::load_driver(&self.driver_lib_dir)?;
    let _ = DRIVER_LOADED.set(());
    Ok(())
}
```

The `OnceLock` protects against multiple loads in the same process, but:
- Integration tests run as separate test threads with shared memory
- The underlying Go library may have thread-safety issues
- The `load_driver` call may not be thread-safe at the FFI boundary

### Potential Solutions

#### Solution A: Test-Level Synchronization (Recommended)

Add a global mutex specifically for tests that require driver access:

```rust
// tests/common/mod.rs
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global lock for tests that use the Teradata driver
/// This serializes driver initialization across test threads
pub static DRIVER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Initialize driver within locked context
pub fn with_driver<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = DRIVER_LOCK.lock().expect("Driver lock poisoned");
    f()
}
```

Usage in tests:
```rust
#[test]
#[ignore]
fn test_live_query() {
    common::with_driver(|| {
        let client = create_test_client();
        // ... test code ...
    });
}
```

**Pros:**
- No changes to production code
- Tests can still run in parallel (non-driver tests unaffected)
- Explicit synchronization makes concurrency boundary clear

**Cons:**
- Requires modifying all driver-using tests
- Still serializes driver tests

#### Solution B: Process-Level Isolation

Run each driver test in a separate process using `cargo-nextest`:

```bash
cargo nextest run --test integration_tests
```

The `nextest` runner runs each test in its own process, eliminating shared state issues.

**Pros:**
- True isolation
- No code changes required
- Better failure isolation

**Cons:**
- Requires additional tooling
- Slower startup per test
- More complex CI setup

#### Solution C: Driver Lazy Initialization with Mutex

Enhance the production code to use a mutex during initialization:

```rust
// src/db/client.rs
use std::sync::Mutex;

static DRIVER_INIT_MUTEX: Mutex<bool> = Mutex::new(false);

fn ensure_driver_loaded(&self) -> Result<()> {
    let mut initialized = DRIVER_INIT_MUTEX.lock()
        .map_err(|_| TqError::InternalError("Driver mutex poisoned".into()))?;

    if *initialized {
        return Ok(());
    }

    teradatarustapi::load_driver(&self.driver_lib_dir)?;
    *initialized = true;

    Ok(())
}
```

**Pros:**
- Fixes issue in production code
- Works for all test scenarios
- No test modification needed

**Cons:**
- Adds synchronization to hot path (minor overhead)
- Mutex in production code for test issue

### Recommended Approach

Given the constraints, **Solution A (Test-Level Synchronization)** is recommended:

1. It isolates the fix to test infrastructure
2. Production code remains unchanged
3. The issue is fundamentally a test concurrency problem

Implementation steps:
1. Create `tests/common/mod.rs` with driver lock
2. Update `tests/integration_tests.rs` to use `with_driver`
3. Document pattern in `docs/testing/execution.md`
4. Remove `--test-threads=1` requirement from documentation

### Fallback Position

If investigation reveals the issue is in the `teradatarustapi` crate itself:
1. Document the limitation
2. Keep `--test-threads=1` workaround
3. Consider opening an issue with the upstream library

---

## Code Organization

### Module Structure

```
src/
├── commands/
│   └── query.rs          # Query execution (single + batch)
├── cli.rs                # CLI definitions (--output, --atomic flags)
├── db/
│   └── client.rs         # Database client, driver loading
└── error.rs              # Error types (FileWriteError, TransactionError)
```

### Key Types

```rust
// Input source enumeration (existing)
pub enum InputSource {
    Argument(String),
    File(PathBuf),
    Stdin,
}

// Batch execution result (existing)
pub struct BatchExecutionResult {
    pub successful_count: usize,
    pub total_count: usize,
}

// New: Transaction state tracking
pub enum TransactionState {
    None,
    AutoStarted,
    UserManaged,
}
```

---

## Error Handling Patterns

### File Operation Errors

```rust
// Pattern: Map I/O errors to structured types with context
File::create(path).map_err(|e| TqError::FileWriteError {
    path: path.to_path_buf(),
    source: e,
})?;
```

### Transaction Errors

```rust
// Pattern: New error type for transaction operations
#[derive(Error, Debug)]
pub enum TqError {
    // ... existing variants ...

    /// Transaction operation failed
    #[error("Transaction {operation} failed: {message}")]
    TransactionError {
        operation: String,      // "BEGIN", "COMMIT", "ROLLBACK"
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
```

### Session Mode Transaction Errors (Sprint 24)

Teradata has different session modes that affect transaction control support:

| Session Mode | Transaction Support | Common Usage |
|--------------|---------------------|--------------|
| ANSI | Auto-commit by default, explicit BEGIN required | Standard SQL |
| Teradata | Implicit transactions, COMMIT/ROLLBACK supported | Traditional Teradata |
| DBC/SQL (ODBC/JDBC) | May restrict transaction control statements | Driver connections |

When transaction control fails due to session mode limitations, tq provides enhanced error messages:

```rust
// src/error.rs - SessionModeTransactionError variant
#[error("Transaction control not supported in current session mode")]
SessionModeTransactionError {
    /// The attempted operation (e.g., "COMMIT", "BEGIN TRANSACTION")
    operation: String,
    /// Original error code if available (e.g., 3706)
    error_code: Option<u32>,
    /// Original error message from database
    original_message: String,
}

// src/db/client.rs - Detection logic
fn is_transaction_session_error(error_lower: &str, sql: &str) -> bool {
    // Detect if SQL is a transaction control statement
    let is_transaction_sql = sql contains COMMIT/ROLLBACK/BEGIN TRANSACTION/BT/ET

    // Check for session mode restriction patterns
    error_lower contains "not allowed" OR "not supported" OR "3706" etc
}
```

**Error Message Example:**
```
Error: Transaction control not supported [Error 3706]

COMMIT is not allowed for DBC/SQL session

Operation attempted: COMMIT

This error typically occurs when the session mode does not support
explicit transaction control (e.g., DBC/SQL sessions via ODBC/JDBC).

Troubleshooting:
  - Verify the connection session mode supports transactions
  - If using --atomic, try without it and manage transactions manually
  - For ANSI mode databases, transactions are auto-committed by default
  - Contact your DBA to verify session configuration

Technical details:
  Teradata has different session modes:
  - ANSI mode: Auto-commit by default, explicit BEGIN required
  - Teradata mode: Implicit transactions, COMMIT/ROLLBACK supported
  - DBC/SQL (ODBC/JDBC): May restrict transaction control statements
```

**Implementation Files (Sprint 24):**
- `src/error.rs` - `SessionModeTransactionError` variant and `user_message()` implementation
- `src/db/client.rs` - `is_transaction_session_error()`, `extract_transaction_operation()`, `extract_error_code()` functions

### User-Friendly Messages

All errors provide actionable guidance:

```rust
impl TqError {
    pub fn user_message(&self) -> String {
        match self {
            TqError::FileWriteError { path, source } => {
                format!(
                    "Error: Cannot write to '{}'\n\n\
                     Cause: {}\n\n\
                     Suggestions:\n  \
                     - Check directory exists and is writable\n  \
                     - Verify disk space available\n  \
                     - Check file permissions",
                    path.display(), source
                )
            }

            TqError::TransactionError { operation, message, .. } => {
                format!(
                    "Error: Transaction {} failed\n\n\
                     {}\n\n\
                     Note: When using --atomic, all changes are rolled back on error.\n\
                     Previous statements in this batch may have been undone.",
                    operation, message
                )
            }

            // ... other cases ...
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

- Statement parsing edge cases
- Transaction detection in SQL
- File path handling

### Integration Tests

- File output with all formats (table, CSV, JSON)
- Atomic file writes (verify no partial files on error)
- Transaction commit/rollback scenarios
- Error message formatting

### Manual Validation

- Large file writes (verify streaming, not buffering)
- Disk full scenarios
- Permission denied scenarios
