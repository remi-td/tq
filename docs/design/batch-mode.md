# Batch Mode Technical Design

This document describes the technical architecture for batch mode features in tq, explaining how SQL statement parsing, file output, transaction control, and multi-statement execution are implemented.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [SQL Statement Parser](#sql-statement-parser)
3. [File Output (--output flag)](#file-output---output-flag)
4. [Transaction Control (--atomic flag)](#transaction-control---atomic-flag)
5. [Integration Test Driver Loading](#integration-test-driver-loading)
6. [Code Organization](#code-organization)
7. [Error Handling Patterns](#error-handling-patterns)

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

## SQL Statement Parser

The SQL statement parser lives in `src/sql/parser.rs` and is the entry point for all multi-statement SQL input. Its sole responsibility is splitting raw SQL text into a sequence of `ParsedStatement` values for sequential execution.

### Design Motivation

The original parser used `sql.split(';')`, which treats every semicolon as a statement boundary regardless of context. This produces three categories of failure:

| Bug | Trigger | Effect |
|-----|---------|--------|
| #28 | `WHERE name = 'O''Brien;'` | String literal semicolon splits the statement |
| #29 | Multi-line `SELECT\n  col\nFROM t` | Works, but exposes the root cause clearly |
| #30 | Block comment before next statement | Comment text leaks into the next statement body |

All three share the same root cause: the parser has no awareness of the SQL lexical context around each character it processes.

### Approach: Single-Pass Character Lexer

The replacement parser scans the input one character (Unicode scalar) at a time, maintaining an explicit state machine. This single pass simultaneously:

1. Identifies statement boundaries (`;` in Normal state only)
2. Strips comments (line and block) before assembling statement text
3. Tracks the current line number for error-reporting metadata

A single pass keeps the implementation O(n) in input length and avoids allocating an intermediate token stream.

### State Machine

The lexer uses a four-value state enum:

```rust
/// Lexer state for SQL parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    /// Normal SQL text — semicolons are statement separators here
    Normal,
    /// Inside a single-quoted string literal ('...')
    InSingleQuotedString,
    /// Inside a line comment (-- ... \n)
    InLineComment,
    /// Inside a block comment (/* ... */)
    InBlockComment,
}
```

The state machine has the following transitions:

```
Normal
  ├─ '\''          → InSingleQuotedString  (start string)
  ├─ '-' '-'       → InLineComment         (start line comment)
  ├─ '/' '*'       → InBlockComment        (start block comment)
  └─ ';'           → emit statement, stay Normal

InSingleQuotedString
  ├─ '\'' '\''     → stay InSingleQuotedString  (escaped quote, consume both)
  └─ '\''          → Normal                      (end string)

InLineComment
  └─ '\n'          → Normal   (newline ends line comment)

InBlockComment
  └─ '*' '/'       → Normal   (end block comment)
```

Two-character transitions (`--`, `/*`, `''`, `*/`) require one character of lookahead, implemented by peeking at the next character in the iterator rather than backtracking.

### Comment Handling: Strip Comments

Comments are stripped from the output rather than preserved. This decision is deliberate:

- Bug #30 demonstrates that a block comment between two statements (`stmt1; /* comment */ stmt2;`) causes the comment text to attach to `stmt2`, corrupting the SQL sent to Teradata.
- Teradata handles comments correctly in isolation, but the bug occurs during statement assembly in the parser, not in the Teradata engine.
- Stripping comments at the parser level is safe: the comment's semantic content (documentation) is irrelevant to execution. Teradata receives clean SQL.
- Stripping also prevents multi-line block comments from inflating `start_line` by accident.

Note: `--` comments are stripped but the newline that ends them is preserved, because that newline may contribute to line-number accounting.

### Line Number Tracking

The lexer increments a `current_line: usize` counter on every `\n` character encountered, regardless of lexer state. When a statement boundary is recognised (`;` in Normal state), the line number stored in the `ParsedStatement` is the line number of the first non-whitespace character in the current statement buffer.

This is implemented by recording `statement_start_line` at the moment the first non-whitespace character is appended to the current statement buffer. The counter is reset to `None` at the start of each new statement and set on first content.

### API

The public API is unchanged from the previous implementation:

```rust
/// A parsed SQL statement with metadata for error reporting.
pub struct ParsedStatement {
    /// The SQL statement text (trimmed, comments stripped)
    pub sql: String,
    /// 1-based statement number for user-facing messages
    pub statement_number: usize,
    /// Line number where statement content starts (1-based)
    pub start_line: usize,
}

/// Parse SQL text into individual statements.
///
/// Returns statements in order. Empty and whitespace-only statements
/// (including comment-only segments) are skipped. Comments are stripped
/// from statement text before returning.
pub fn parse_statements(sql: &str) -> Vec<ParsedStatement>

/// Returns true if the SQL contains more than one statement.
pub fn has_multiple_statements(sql: &str) -> bool
```

`ParsedStatement::preview()` is also unchanged — it normalises whitespace in the trimmed SQL, which now never contains comment text.

### Implementation Sketch

```rust
pub fn parse_statements(sql: &str) -> Vec<ParsedStatement> {
    let mut statements: Vec<ParsedStatement> = Vec::new();
    let mut state = LexState::Normal;

    // Buffer for the current statement's content (comments excluded)
    let mut current: String = String::new();
    // Line number of the first content character in `current`
    let mut stmt_start_line: Option<usize> = None;
    let mut current_line: usize = 1;
    let mut statement_number: usize = 0;

    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        // Line tracking applies in every state
        if ch == '\n' {
            current_line += 1;
        }

        match state {
            LexState::Normal => match ch {
                '\'' => {
                    record_content(ch, &mut current, &mut stmt_start_line, current_line);
                    state = LexState::InSingleQuotedString;
                }
                '-' if chars.peek() == Some(&'-') => {
                    chars.next(); // consume second '-'
                    state = LexState::InLineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next(); // consume '*'
                    state = LexState::InBlockComment;
                }
                ';' => {
                    // Statement boundary — emit if non-empty
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        statement_number += 1;
                        statements.push(ParsedStatement::new(
                            trimmed,
                            statement_number,
                            stmt_start_line.unwrap_or(current_line),
                        ));
                    }
                    current.clear();
                    stmt_start_line = None;
                }
                other => {
                    record_content(other, &mut current, &mut stmt_start_line, current_line);
                }
            },

            LexState::InSingleQuotedString => match ch {
                '\'' if chars.peek() == Some(&'\'') => {
                    // Escaped quote — consume both, append both to preserve literal
                    let next = chars.next().unwrap();
                    current.push(ch);
                    current.push(next);
                }
                '\'' => {
                    current.push(ch);
                    state = LexState::Normal;
                }
                other => current.push(other),
            },

            LexState::InLineComment => {
                // Newline was already processed above for line counting;
                // transition back to Normal but do NOT push any comment text.
                if ch == '\n' {
                    state = LexState::Normal;
                }
                // All other characters in a line comment are discarded.
            }

            LexState::InBlockComment => {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    state = LexState::Normal;
                }
                // Block comment content discarded (newlines already counted above).
            }
        }
    }

    // Flush trailing statement (no terminating semicolon)
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statement_number += 1;
        statements.push(ParsedStatement::new(
            trimmed,
            statement_number,
            stmt_start_line.unwrap_or(current_line),
        ));
    }

    statements
}

/// Push `ch` to `buf` and record the start line on first content character.
#[inline]
fn record_content(
    ch: char,
    buf: &mut String,
    start_line: &mut Option<usize>,
    current_line: usize,
) {
    if start_line.is_none() && !ch.is_whitespace() {
        *start_line = Some(current_line);
    }
    buf.push(ch);
}
```

### Handling of Existing Tests

Several existing unit tests in `src/sql/parser.rs` assert that comments are *preserved* in statement output (e.g., `test_parse_preserves_comments`, `test_parse_multiline_comment`, `test_parse_complex_script`). These tests were written against the old "pass comments through" design decision. Because Sprint 42 deliberately reverses that decision (strip comments), those test assertions must be updated:

- `test_parse_preserves_comments` — assert the statement is `"SELECT 1"` (comment stripped)
- `test_parse_multiline_comment` — assert the statement is `"SELECT 1"` (block comment stripped)
- `test_parse_complex_script` — assertions that check `contains("CREATE TABLE")` etc. remain valid; assertions that checked comment text inside statement bodies are removed

The `has_multiple_statements` function and all line-tracking tests are unaffected.

### New Tests to Add

The following test cases must be added to cover the three bugs:

```rust
// Bug #28 — semicolon inside single-quoted string
#[test]
fn test_semicolon_in_string_literal_not_a_boundary() { ... }

// Bug #28 variant — escaped quote inside string
#[test]
fn test_escaped_quote_in_string_literal() { ... }

// Bug #29 — multi-line statement
#[test]
fn test_multi_line_statement_is_single_statement() { ... }

// Bug #30 — block comment between statements
#[test]
fn test_block_comment_between_statements_does_not_contaminate() { ... }

// Bug #30 variant — line comment between statements
#[test]
fn test_line_comment_between_statements_does_not_contaminate() { ... }

// Comment stripping general
#[test]
fn test_comments_are_stripped_from_output() { ... }

// Empty-after-stripping: a comment-only segment is not emitted
#[test]
fn test_comment_only_segment_is_skipped() { ... }
```

### Backwards Compatibility

The `ParsedStatement` struct, `parse_statements` signature, and `has_multiple_statements` signature are all unchanged. Call sites in `src/commands/query.rs` require no modification. The only observable behaviour change is that comment text no longer appears in `ParsedStatement::sql` — which is the correct behaviour per the updated specification.

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
