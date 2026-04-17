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

### Error Handling: Result Return Type (Sprint 43)

Sprint 43 changes `parse_statements()` to return `Result<Vec<ParsedStatement>, ParseError>` instead of `Vec<ParsedStatement>`. This enables callers to receive a structured, actionable error when the input contains unterminated constructs rather than silently producing a partial or malformed result.

#### ParseError Type

```rust
/// Error from the SQL statement parser
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable description of the error
    pub message: String,
    /// 1-based line number where the error was detected
    pub line: usize,
    /// 1-based column number where the error was detected
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}, column {}", self.message, self.line, self.col)
    }
}

impl std::error::Error for ParseError {}
```

#### Error Conditions

| Condition | Error message |
|-----------|--------------|
| Input ends while inside a single-quoted string | `"Unterminated string literal"` |
| Input ends while inside a block comment | `"Unterminated block comment"` |

Line and column are the position of the opening delimiter (`'` or `/*`) that was never closed.

**Unterminated line comments are not an error**: a line comment with no trailing newline is treated as if the newline is present. The comment is stripped and the input ends cleanly.

#### Updated API

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
///
/// # Errors
///
/// Returns `Err(ParseError)` if the input contains an unterminated
/// single-quoted string literal or an unterminated block comment.
pub fn parse_statements(sql: &str) -> Result<Vec<ParsedStatement>, ParseError>

/// Returns true if the SQL contains more than one statement.
///
/// Returns false if parsing fails (defensive: errors are reported by
/// `parse_statements` at the call sites that need them).
pub fn has_multiple_statements(sql: &str) -> bool
```

`has_multiple_statements` continues to return a plain `bool` by internally calling `parse_statements` and treating errors as `false`. This keeps the call sites that only need the boolean unaffected.

#### Call Site Updates

Every call site of `parse_statements` must handle the `Result`:

**`src/commands/query.rs`** (batch execution path):
```rust
// Before (Sprint 42):
let statements = parse_statements(sql);

// After (Sprint 43):
let statements = parse_statements(sql)
    .map_err(|e| TqError::SqlParseError(e.to_string()))?;
```

`TqError::SqlParseError` surfaces the line/column from `ParseError` in the user-facing error message:

```
Error: SQL parse error at line 3, column 12

  Unterminated string literal

  Check that all single-quoted strings are properly closed.
```

#### Tracking Opening Delimiters for Error Reporting

To report the position of the opening delimiter, the parser records the line and column of:
- The `'` that starts a single-quoted string
- The `/` of `/*` that starts a block comment

These are stored as `Option<(usize, usize)>` (line, col) fields in the parser local state. Column tracking is added to the parser loop: a `current_col` counter increments on each character and resets to 1 on each `\n`.

#### Space Injection Behavior (Documentation)

The parser injects a single space character at two transition points to prevent accidental token merging:

1. **End of line comment** (`\n` while in `InLineComment`): A space is pushed to `current` before returning to `Normal` state. This prevents `keyword--comment\nidentifier` from producing `keywordidentifier`.

2. **End of block comment** (`*/`): A space is pushed to `current`. This prevents `keyword/*comment*/identifier` from producing `keywordidentifier`.

These injected spaces are harmless; SQL parsers in Teradata treat runs of whitespace identically to a single space.

**Note on `unwrap()` at parser.rs line 178**: The line:
```rust
let next = chars.next().unwrap();
```
is inside the `InSingleQuotedString` branch handling `'\'' if chars.peek() == Some(&'\'')`  — that is, it only executes after `peek()` has already confirmed the next character exists and equals `'`. The `unwrap()` is therefore unreachable as `None` and safe by invariant. It cannot be replaced with `?` because the surrounding function currently returns `Vec<T>` (before Sprint 43) or `Result<Vec<T>, ParseError>` (after Sprint 43, where only `ParseError` is a valid error variant, not an iterator exhaustion case).

### API Summary

```rust
pub fn parse_statements(sql: &str) -> Result<Vec<ParsedStatement>, ParseError>
pub fn has_multiple_statements(sql: &str) -> bool
```

`ParsedStatement::preview()` is unchanged — it normalises whitespace in the trimmed SQL, which never contains comment text.

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

The following test cases must be added to cover the three Sprint 42 bugs and the Sprint 43 remediation items:

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

// Sprint 43 AC-17: comment marker (--) inside a string literal is not treated as a comment
#[test]
fn test_comment_marker_inside_string_is_not_comment() {
    // The -- inside the string must NOT start a line comment
    let sql = "SELECT 'hello -- world' FROM t;";
    let statements = parse_statements(sql).unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].sql, "SELECT 'hello -- world' FROM t");
}

// Sprint 43 AC-13: unterminated string returns ParseError with line/col
#[test]
fn test_unterminated_string_returns_parse_error() {
    let result = parse_statements("SELECT 'unterminated");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Unterminated string"));
    assert_eq!(err.line, 1);
    assert_eq!(err.col, 8); // position of opening '
}

// Sprint 43 AC-14: unterminated block comment returns ParseError with line/col
#[test]
fn test_unterminated_block_comment_returns_parse_error() {
    let result = parse_statements("SELECT 1; /* unterminated");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Unterminated block comment"));
}
```

### Backwards Compatibility

The `ParsedStatement` struct, `parse_statements` signature, and `has_multiple_statements` signature are all unchanged. Call sites in `src/commands/query.rs` require no modification. The only observable behaviour change is that comment text no longer appears in `ParsedStatement::sql` — which is the correct behaviour per the updated specification.

### BEGIN/END Block Tracking (Stored Procedure Bodies)

Stored procedures, triggers, and macro bodies routinely contain internal `;` characters that terminate statements *inside the procedure body* but MUST NOT be treated as top-level statement boundaries by the file-mode splitter. The canonical example:

```sql
REPLACE PROCEDURE demo.sp()
BEGIN
    DECLARE v INTEGER;     -- internal ;
    SET v = 1;             -- internal ;
    IF v = 1 THEN
        SET v = 2;         -- internal ;
    END IF;                -- internal ;
END;                       -- top-level ; (end of procedure)
```

The Sprint 42 state machine only tracks string/comment states, so the first internal `;` is interpreted as a top-level boundary and a truncated fragment is sent to Teradata.

#### Approach: BEGIN/END Depth Counter + Header Gate

Extend the existing state machine with a `begin_end_depth: u32` counter and a boolean latch `in_procedure_header: bool`. Top-level `;` remains a statement terminator only when BOTH:

- `state == LexState::Normal`, AND
- `begin_end_depth == 0`

This is a pure composition with the existing string/comment states — the three existing states already inhibit `;` matching, and the depth counter adds a fourth inhibition condition.

#### Procedure-Header Detection

The naive approach (any `BEGIN` anywhere bumps the depth) is wrong: a `BEGIN` inside a regular `SELECT ... FROM t BEGIN_DATE` column name, or a free-standing `BEGIN TRANSACTION` statement, must not be treated as an SPL block opener.

Detection gate: the parser arms a `procedure_header_seen` flag when it recognises the pattern `(CREATE | REPLACE) ... (PROCEDURE | TRIGGER | MACRO | FUNCTION)` in the current statement buffer. Only when this flag is armed does a subsequent top-level `BEGIN` increment `begin_end_depth` (and disarm the flag — the flag only matters for the first BEGIN that opens the body).

To avoid a full SQL parser, the gate is implemented as a lightweight keyword-sequence matcher operating on the token stream already produced by the lexer. A minimal approach:

- Maintain a rolling view of the last N tokens (N=8 is sufficient) of the current statement buffer, split on whitespace, uppercased for comparison.
- At each `;` emission OR at the start of the buffer, reset the sequence tracker.
- On each transition to `Normal` from a non-content state, scan the buffer tail for the pattern:
  `(CREATE|REPLACE) (OR REPLACE)? [non-keyword-tokens]* (PROCEDURE|TRIGGER|MACRO|FUNCTION)`

Practical implementation: check `procedure_header_seen` lazily right before a `BEGIN` match by scanning the current uppercased buffer for `/\b(CREATE|REPLACE)\b.*\b(PROCEDURE|TRIGGER|MACRO|FUNCTION)\b/` using a one-shot `str::contains`-style check on the buffer so far. The `regex` crate is already a dependency; a lazy-compiled `Regex` is the cleanest implementation.

```rust
use regex::Regex;
use std::sync::OnceLock;

fn procedure_header_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(CREATE|REPLACE)\b[\s\S]*?\b(PROCEDURE|TRIGGER|MACRO|FUNCTION)\b")
            .expect("valid procedure-header regex")
    })
}
```

The `[\s\S]*?` is non-greedy and bounded in practice by the current statement buffer, which is cleared at each top-level `;`. Case-insensitive matching is enabled via the `(?i)` inline flag.

#### BEGIN/END Keyword Matching

Inside the `LexState::Normal` branch of the main loop, after the existing `;` / `'` / `--` / `/*` checks, add keyword boundary detection:

1. On encountering an ASCII alphabetic character in `Normal` state, extend an in-place word scan via `chars.peek()` lookahead to identify the next whole word (ASCII-letter run).
2. Match the word against `BEGIN` / `END` case-insensitively using `eq_ignore_ascii_case`.
3. For `BEGIN`: if `begin_end_depth > 0` OR the current statement buffer matches the procedure-header regex, increment `begin_end_depth`. Push the word to the buffer as usual.
4. For `END`: if `begin_end_depth > 0`, decrement `begin_end_depth`. Push the word to the buffer.

Important edge cases:

- **`END IF`, `END LOOP`, `END WHILE`, `END CASE`**: These are composite SPL keywords that logically close an inner block, not the procedure. They ARE correctly handled by a pure depth counter because each is paired with a matching `IF`/`LOOP`/`WHILE`/`CASE` that increments the depth. **Correction: these DO NOT increment depth** — only `BEGIN` does. We must NOT decrement for `END IF` / `END LOOP` / `END WHILE` / `END CASE` / `END FOR`. The implementation therefore peeks ahead after matching `END`: if the next non-whitespace token is one of `IF | LOOP | WHILE | CASE | FOR`, treat the `END` as a non-counting keyword (do not decrement). Only a bare `END` (followed by `;` or whitespace-then-`;` or EOF) decrements the depth.
- **Identifiers containing `BEGIN` / `END` substrings** (e.g., `BEGIN_DATE`, `APPEND`): rejected by the word-boundary scan — a trailing `_` or alphanumeric character means the word is not `BEGIN`/`END`. Use `char::is_ascii_alphanumeric` or `_` as continuation characters.
- **`BEGIN` / `END` inside string literals**: The existing `LexState::InSingleQuotedString` already suppresses all keyword matching. This composes correctly.
- **`BEGIN` / `END` inside comments**: Same — the `InLineComment` / `InBlockComment` states suppress keyword matching.
- **`BEGIN TRANSACTION` at top level**: If `procedure_header_seen == false`, the `BEGIN` does not increment depth. The matching `COMMIT` / `ROLLBACK` / `END TRANSACTION` / `ET;` is never counted as a decrement either.
- **Case sensitivity**: `Begin`, `BEGIN`, `begin` all match. Use `eq_ignore_ascii_case`.
- **Nested `BEGIN ... END` blocks inside SPL**: `begin_end_depth` correctly tracks nesting — depth 2, 3, … are handled uniformly.
- **Unterminated SPL body (no matching `END`)**: Input ends with `begin_end_depth > 0`. The final flush still emits the accumulated content as one statement (the user's SQL is syntactically invalid and Teradata will reject it with its own error). We do NOT return a parser error — the parser's job is splitting, not SPL validation.

#### State Machine Update

The state machine gains one new piece of mutable state but no new `LexState` variant. The four existing states remain sufficient; the depth counter is an orthogonal inhibition mechanism:

```
Top-level statement boundary fires when:
  state == Normal
  AND begin_end_depth == 0
  AND ch == ';'
```

Alternative rejected: adding an `InProcedureBody` state variant. This was considered but adds coupling between SPL-awareness and the core string/comment lexer, and does not gracefully handle nested `BEGIN ... END` depth. A depth counter is the minimal and composable change.

#### Files Modified

- **`src/sql/parser.rs`** — Primary change. Extend `parse_statements()` with:
  - A `begin_end_depth: u32` local variable (initialised to 0).
  - A helper `fn consume_word(ch: char, chars: &mut Peekable<Chars>) -> String` that returns the uppercase ASCII word starting at `ch`, consuming alphanumeric/underscore continuation characters from the iterator.
  - A helper `fn is_procedure_header(buf: &str) -> bool` using the lazy-initialised regex described above.
  - A helper `fn peek_end_is_inner_keyword(chars: &Peekable<Chars>) -> bool` that looks ahead (without consuming) for whitespace-then-`IF|LOOP|WHILE|CASE|FOR`.
  - Logic at the start of the `Normal` match arm: if the current char is ASCII alphabetic, peek-consume to build the word, match `BEGIN` / `END` case-insensitively, adjust `begin_end_depth`, and push the consumed chars to the buffer.
  - Gate the existing `';'` emission on `begin_end_depth == 0`.
- **No changes** to `src/commands/query.rs`, `src/sql/mod.rs`, or any other file. The parser API (`parse_statements`, `has_multiple_statements`, `ParsedStatement`) is unchanged.

#### New Tests (in `src/sql/parser.rs`)

```rust
#[test] fn test_create_procedure_body_is_single_statement() { ... }
#[test] fn test_replace_procedure_body_is_single_statement() { ... }
#[test] fn test_nested_begin_end_blocks_handled() { ... }  // BEGIN ... BEGIN ... END; END;
#[test] fn test_end_if_does_not_decrement_depth() { ... }
#[test] fn test_end_loop_does_not_decrement_depth() { ... }
#[test] fn test_end_case_does_not_decrement_depth() { ... }
#[test] fn test_begin_inside_string_does_not_open_body() { ... }
#[test] fn test_end_inside_string_does_not_close_body() { ... }
#[test] fn test_begin_end_in_line_comment_ignored() { ... }
#[test] fn test_begin_end_in_block_comment_ignored() { ... }
#[test] fn test_begin_transaction_at_top_level_not_tracked() { ... }
#[test] fn test_multi_procedure_script_splits_correctly() { ... }
#[test] fn test_mixed_spl_and_regular_statements() { ... }
#[test] fn test_identifier_beginning_with_begin_not_a_keyword() { ... }  // BEGIN_DATE column
#[test] fn test_create_trigger_body_is_single_statement() { ... }
#[test] fn test_create_macro_body_is_single_statement() { ... }
#[test] fn test_case_insensitive_begin_end() { ... }  // lowercase begin/end
```

#### Concerns / Risks

- **Teradata DDL flavours**: Some `CREATE FUNCTION` variants (e.g., SQL scalar functions) use `RETURNS ... RETURN expr;` syntax rather than `BEGIN ... END`. The regex arms on FUNCTION but `begin_end_depth` only increments on an actual `BEGIN`, so these are unaffected. Table functions with `BEGIN ... END` bodies ARE covered correctly.
- **`CREATE TABLE ... AS ... END_OF_STATEMENT` false positives**: The regex requires both `(CREATE|REPLACE)` and one of the SPL object keywords. `CREATE TABLE` does not match. Safe.
- **`BEGIN` keyword inside a `CASE` expression's `WHEN` clause** (`CASE WHEN col = 'BEGIN' THEN ...`): String literal, suppressed. Safe.
- **`COMMIT` / `ROLLBACK` inside SPL body**: These do NOT affect `begin_end_depth` because we only count `BEGIN` / bare `END`. Safe.
- **Windows line endings in procedure bodies**: `\r\n` is handled by the existing newline logic; the depth counter is unaffected. Safe.

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
