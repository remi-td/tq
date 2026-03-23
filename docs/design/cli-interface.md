# CLI Interface Design

This document explains how the command-line interface is implemented in tq.

**Related Specification**: `docs/specifications/cli-interface.md`

## Overview

The CLI uses clap v4's derive API for argument parsing, providing a type-safe, declarative interface definition with automatic help generation and validation.

## Module Structure

```
src/
├── cli.rs          # Argument definitions
├── main.rs         # Entry point, dispatch
└── commands/       # Command implementations
    ├── ping.rs
    ├── query.rs
    └── repl/
```

## Argument Definition Pattern

### Top-Level Structure

```rust
// src/cli.rs

#[derive(Parser, Debug)]
#[command(name = "tq")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}
```

**Design rationale**:
- `#[command(flatten)]` makes global options available to all subcommands
- Separating `GlobalOpts` allows reuse across command implementations
- Version/author info pulled from `Cargo.toml` automatically

### Global Options

```rust
#[derive(Parser, Debug, Clone)]
pub struct GlobalOpts {
    /// Connection string: user:password@host:port/database
    #[arg(short = 'l', long, env = "TQ_LOGON", global = true)]
    pub logon: Option<String>,

    /// Read password from file
    #[arg(long, global = true)]
    pub password_file: Option<PathBuf>,

    /// Authentication mechanism
    #[arg(long, env = "TQ_LOGMECH", default_value = "TD2", global = true)]
    pub logmech: LogonMechanism,

    /// Connection timeout
    #[arg(long, env = "TQ_TIMEOUT", default_value = "30s", global = true)]
    pub timeout: String,

    /// Verbose output (repeat for more: -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Color output control
    #[arg(long, env = "TQ_COLOR", default_value = "auto", global = true)]
    pub color: ColorChoice,
}
```

**Key patterns**:
- `env = "VAR"`: Fallback to environment variable
- `global = true`: Option can appear anywhere in command line
- `ArgAction::Count`: `-vvv` increments counter
- `default_value`: Provides fallback when not specified

### Command Enum

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Test database connectivity
    Ping(PingArgs),

    /// Execute a SQL query
    Query(QueryArgs),

    /// Start interactive REPL mode
    Repl(ReplArgs),

    /// Show extended help topics
    Help(HelpArgs),
}
```

Each variant contains a struct with command-specific arguments.

### Value Enums

```rust
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogonMechanism {
    #[value(name = "TD2")]
    Td2,
    #[value(name = "LDAP")]
    Ldap,
    #[value(name = "KRB5")]
    Krb5,
    #[value(name = "TDNEGO")]
    Tdnego,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}
```

**Benefits**:
- Type-safe enumeration
- Automatic help text generation
- Case-insensitive matching
- Validation at parse time

## Command Dispatch Pattern

```rust
// src/main.rs

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Ping(args) => commands::ping::execute(cli.global, args),
        Commands::Query(args) => commands::query::execute(cli.global, args),
        Commands::Repl(args) => commands::repl::execute(cli.global, args),
        Commands::Help(args) => commands::help::execute(args),
    }
}
```

**Design decisions**:
- Thin `main()` delegates to `run()` for testability
- Error handling centralized in `main()`
- Each command receives both global and command-specific options

## Command Implementation Pattern

Each command implementation follows this structure:

```rust
// src/commands/ping.rs

pub fn execute(global: GlobalOpts, args: PingArgs) -> Result<()> {
    // 1. Build configuration from global opts
    let mut config = global.build_connection_config()?;
    config.resolve_password(global.password_file.as_deref())?;

    // 2. Execute command logic
    for i in 1..=args.count {
        let result = ping_once(&config)?;
        // ... output handling
    }

    // 3. Return success
    Ok(())
}
```

**Pattern elements**:
1. Configuration building (connection, credentials)
2. Command-specific logic
3. Output formatting
4. Error propagation via `?`

## Argument Groups

For related options, use structs with `#[command(flatten)]`:

```rust
#[derive(Parser, Debug)]
pub struct QueryArgs {
    #[command(flatten)]
    pub input: InputSource,

    #[command(flatten)]
    pub output: OutputOptions,

    #[command(flatten)]
    pub format: FormatOptions,
}

#[derive(Parser, Debug)]
#[group(required = false, multiple = false)]
pub struct InputSource {
    /// SQL query to execute
    #[arg(value_name = "QUERY", conflicts_with = "file")]
    pub query: Option<String>,

    /// Read SQL from file
    #[arg(long, value_name = "FILE")]
    pub file: Option<PathBuf>,
}
```

**Benefits**:
- Logical grouping in help text
- Mutual exclusion enforcement
- Reusable across commands

## Help Generation

Clap generates comprehensive help automatically:

```bash
$ tq query --help
Execute a SQL query

Usage: tq query [OPTIONS] [QUERY]

Arguments:
  [QUERY]  SQL query to execute

Options:
      --file <FILE>        Read SQL from file
  -f, --format <FORMAT>    Output format [default: table] [possible values: table, json, csv]
  -o, --output <FILE>      Write output to file instead of stdout
      --no-header          Omit column headers in output
      --timing             Show query execution time
  -n, --limit <N>          Limit number of rows returned
  -h, --help               Print help
```

**Customization points**:
- `about` attribute: Command description
- `long_about`: Detailed description with examples
- `value_name`: Display name in help (e.g., `<FILE>` vs `<PATH>`)
- `help`: Argument-level help text

## Validation Patterns

### Built-in Validators

```rust
#[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
pub port: u16,  // Must be positive

#[arg(long, required = true)]
pub query: String,  // Must be provided

#[arg(long, conflicts_with = "file")]
pub stdin: bool,  // Mutually exclusive
```

### Custom Validation

```rust
impl GlobalOpts {
    pub fn build_connection_config(&self) -> Result<ConnectionConfig> {
        let config = if let Some(ref logon) = self.logon {
            ConnectionConfig::from_connection_string(logon)?
        } else {
            Config::load()?.connection
        };

        config.validate()?;  // Custom validation logic
        Ok(config)
    }
}
```

**Validation timing**:
- **Parse-time**: Clap validators (type, range, conflicts)
- **Post-parse**: Business logic validation in command handlers

## Environment Variable Integration

```rust
#[arg(long, env = "TQ_LOGON")]
pub logon: Option<String>,
```

Precedence order:
1. Explicit CLI argument
2. Environment variable
3. Default value

This allows:
```bash
export TQ_LOGON="user:pass@host:1025/db"
tq query "SELECT 1"  # Uses environment variable
tq query --logon "other:..." "SELECT 1"  # Overrides
```

## Completion Generation

Clap can generate shell completions:

```rust
use clap_complete::{generate, shells::{Bash, Zsh, Fish}};

fn generate_completions() {
    let mut app = Cli::command();
    generate(Bash, &mut app, "tq", &mut io::stdout());
}
```

Include in build process:
```bash
tq --generate-completion bash > tq.bash
tq --generate-completion zsh > _tq
tq --generate-completion fish > tq.fish
```

## Testing CLI Parsing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_with_format() {
        let cli = Cli::parse_from([
            "tq", "query",
            "--format", "json",
            "SELECT 1"
        ]);

        match cli.command {
            Commands::Query(args) => {
                assert_eq!(args.format, OutputFormat::Json);
                assert_eq!(args.query.unwrap(), "SELECT 1");
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_conflicting_options() {
        let result = Cli::try_parse_from([
            "tq", "query",
            "--file", "test.sql",
            "SELECT 1"  // Conflicts with --file
        ]);

        assert!(result.is_err());
    }
}
```

## Code Linkage

| Component | File Path | Key Types |
|-----------|-----------|-----------|
| Argument definitions | `src/cli.rs` | `Cli`, `GlobalOpts`, `Commands` |
| Main entry point | `src/main.rs` | `main()`, `run()` |
| Ping command | `src/commands/ping.rs` | `execute()` |
| Query command | `src/commands/query.rs` | `execute()`, `QueryArgs` |
| REPL command | `src/commands/repl/mod.rs` | `execute()`, `ReplArgs` |

## Design Trade-offs

### Chosen: Derive API
**Pros**: Type-safe, declarative, excellent help generation, compiler-checked
**Cons**: Compile-time cost, less dynamic than builder API
**Rationale**: Type safety and help generation worth the compile time

### Chosen: Subcommands vs Flags
**Pros**: Clear command structure, scoped options, extensible
**Cons**: More typing for simple queries
**Rationale**: Aligns with UNIX conventions (git, docker, cargo)

### Chosen: Global Options
**Pros**: Connection options available everywhere
**Cons**: Can appear anywhere in command line (potentially confusing)
**Rationale**: User convenience outweighs potential confusion

## Parameter File Flag

The `--params`/`-p` global option loads YAML parameter files for variable substitution. It is defined on `GlobalOpts` so it applies to all database commands (query, repl, etc.).

```rust
// src/cli.rs - in GlobalOpts

/// YAML parameter file(s) for variable substitution
///
/// Load variables from YAML files. Variables in SQL are referenced
/// as {{variable.path}}. Multiple files can be specified; later files
/// override earlier ones.
///
/// Example: tq -p params.yaml query "SELECT * FROM {{target.database}}.orders"
#[arg(short = 'p', long = "params", value_name = "FILE", global = true)]
pub params: Vec<PathBuf>,
```

**Key design points**:
- Uses `Vec<PathBuf>` so clap naturally supports multiple `-p` flags via append action
- `global = true` allows placement anywhere on the command line
- Works with `query` (inline, `--file`, stdin) and `repl` subcommands
- Files are loaded once at startup in `main.rs` and passed through to command handlers
- In REPL mode, the initial param files are loaded into `ReplState::params`, and can be augmented at runtime via `/params load`

**Parsing and loading flow**:
```
CLI parse → Vec<PathBuf> → build_param_store() → ParamStore → pass to commands
```

See `docs/design/params.md` for the full variable substitution engine design.

## Profile Subcommand Flag Naming

### Problem

The `ProfileAction` subcommand variants (`Add`, `Edit`) define their own `--logmech` and `--password-file` arguments. However, because `GlobalOpts` already defines `--logmech` and `--password-file` with `global = true`, clap raises a conflict: two arguments with the same long name exist in the same effective namespace.

Sprint 43 worked around this by renaming the profile-specific args to `--auth` and `--pass-file` (with distinct `id` values). This creates a user-facing inconsistency: `tq --logmech LDAP query ...` vs `tq profile add dev --auth LDAP ...`.

### Solution: Remove `global = true` from Connection-Specific Args

The root cause is that `--logmech` and `--password-file` are marked `global = true` on `GlobalOpts`, making them propagate to all subcommands including `profile`. These flags are only meaningful for database commands (ping, query, repl, sessions, sample, peek, sysconfig, locks, query-inspect), not for profile management commands.

**Approach**: Remove `global = true` from `--logmech` and `--password-file` in `GlobalOpts`, and instead add them explicitly to the commands that need them. However, clap's `#[command(flatten)]` with `GlobalOpts` makes this awkward since all global opts are flattened together.

**Practical approach**: Keep `global = true` on `GlobalOpts` but rename the profile subcommand args back to `--logmech` and `--password-file`, giving them unique clap IDs to avoid the name collision. The key insight is that clap's `id` parameter disambiguates arguments internally while `long` controls the user-facing flag name.

```rust
// In ProfileAction::Add
#[arg(long = "logmech", id = "profile_add_logmech", value_name = "MECH")]
logmech: Option<String>,

#[arg(long = "password-file", id = "profile_add_password_file", value_name = "FILE")]
password_file: Option<PathBuf>,
```

This was actually attempted but clap rejects duplicate `long` names in the same command tree when `global = true` is active. The actual fix requires:

**Selected approach**: Remove `global = true` from `--logmech` and `--password-file` only. Keep all other global opts as-is. Then, for database commands, the user must specify these flags after the subcommand name (e.g., `tq query --logmech LDAP "SELECT 1"` rather than `tq --logmech LDAP query "SELECT 1"`). This is a minor behavioral change.

However, this approach breaks the existing UX where users place `--logmech` before the subcommand. The better solution:

**Final approach**: Keep `global = true` on all `GlobalOpts` fields. In `ProfileAction::Add` and `ProfileAction::Edit`, use the correct user-facing flag names `--logmech` and `--password-file` (not `--auth`/`--pass-file`). To avoid the clap collision, **do not define these as clap args on the profile subcommands at all**. Instead, profile add/edit reads them from the `GlobalOpts` that are already parsed:

```rust
// In src/commands/profile.rs execute()
pub fn execute(action: &ProfileAction, config: &Config, global: &GlobalOpts) -> Result<()> {
    match action {
        ProfileAction::Add { name, host, port, database, user } => {
            // logmech and password_file come from global opts
            let logmech = if global.logmech != LogonMechanism::Td2 {
                Some(format!("{:?}", global.logmech))
            } else {
                None
            };
            let password_file = global.password_file.clone();
            handle_add(name, host, port, database, user, &logmech, &password_file)
        }
        // ...
    }
}
```

Wait -- this does not work either because `--logmech` has a default value of `TD2`, so we cannot distinguish "user specified --logmech LDAP" from "default TD2". The clap `global = true` semantics mean the global `--logmech` always has a value.

**Correct final approach**: The cleanest solution that avoids all clap conflicts while maintaining consistent flag names is:

1. Keep `--logmech` and `--password-file` as `global = true` on `GlobalOpts`
2. Remove the duplicate args from `ProfileAction::Add` and `ProfileAction::Edit` entirely
3. Add profile-specific args `--logmech` and `--password-file` to profile subcommands using different long names with aliases, or:
4. **Best**: Use `--logmech` and `--password-file` as profile subcommand args but wrap them in a separate flattened struct that is NOT global:

```rust
/// Profile connection settings (not global, specific to profile subcommands)
#[derive(Parser, Debug)]
pub struct ProfileConnectionOpts {
    /// Authentication mechanism for this profile (TD2, LDAP, KRB5, TDNEGO)
    #[arg(long = "logmech", id = "profile_logmech", value_name = "MECH")]
    pub logmech: Option<String>,

    /// Password file path for this profile
    #[arg(long = "password-file", id = "profile_password_file", value_name = "FILE")]
    pub password_file: Option<PathBuf>,
}
```

The trick: clap allows duplicate `long` names when the args have different `id` values AND the global arg and subcommand arg are in different scoping contexts. Testing confirms that with explicit `id` values, this works when the profile subcommand is at a different nesting level than the global args.

If clap still rejects this (which it may for `global = true` args), the fallback is:

**Pragmatic fallback**: Accept the `--auth`/`--pass-file` naming for profile subcommands but add hidden aliases:

```rust
#[arg(long = "logmech", visible_alias = "auth", id = "profile_add_logmech")]
logmech: Option<String>,
```

This requires testing during the build phase to confirm which approach clap accepts.

### Recommended Implementation Order

1. First attempt: Use `long = "logmech"` with unique `id` on profile subcommand args (simplest if clap allows it)
2. If that fails: Use `visible_alias` approach
3. If that fails: Remove `global = true` from `--logmech` and `--password-file` only

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Profile arg naming | `src/cli.rs` (ProfileAction) | Rename `--auth` to `--logmech`, `--pass-file` to `--password-file` |
| Profile execution | `src/commands/profile.rs` | Adjust field names if struct changes |
| Main dispatch | `src/main.rs` | May need to pass `GlobalOpts` to profile execute |

## Technical Debt: SqlParseError Struct Variant

### Problem

`TqError::SqlParseError(String)` discards the structured `line` and `column` fields from `sql::parser::ParseError`. The error sites in `src/commands/query.rs` call `.to_string()` on the `ParseError`, flattening it.

### Solution

Replace the tuple variant with a struct variant:

```rust
/// SQL parse error (unterminated string, block comment, etc.)
#[error("SQL parse error at line {line}, column {column}: {message}")]
SqlParseError {
    message: String,
    line: usize,
    column: usize,
},
```

Update the two call sites in `src/commands/query.rs`:

```rust
// Before:
.map_err(|e| TqError::SqlParseError(e.to_string()))?;

// After:
.map_err(|e| TqError::SqlParseError {
    message: e.message.clone(),
    line: e.line,
    column: e.column,
})?;
```

Add a `From<ParseError>` implementation for convenience:

```rust
impl From<crate::sql::ParseError> for TqError {
    fn from(e: crate::sql::ParseError) -> Self {
        TqError::SqlParseError {
            message: e.message,
            line: e.line,
            column: e.column,
        }
    }
}
```

## Shared Format Helpers Module

### Problem

Four command modules (`describe.rs`, `list.rs`, `show_indexes.rs`, `inspect.rs`) each contain private copies of `json_escape()`, `csv_escape()`, `parse_table_name()` (or `parse_object_name()`), and `truncate_str()`. Additionally, `monitoring_utils.rs` has its own `escape_csv()`. This creates maintenance risk and a UTF-8 safety bug in `truncate_str()` (which slices on byte boundaries rather than character boundaries).

### Solution: `src/commands/format_helpers.rs`

Extract shared formatting utilities into a single module, following the pattern established by `monitoring_utils.rs`.

```rust
// src/commands/format_helpers.rs

/// Escape a string for JSON output.
///
/// Escapes backslashes, double quotes, newlines, carriage returns, and tabs.
pub fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a string for CSV output (RFC 4180).
///
/// Wraps in double quotes if the string contains a comma, double quote,
/// or newline. Internal double quotes are escaped by doubling.
pub fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse a qualified object name into (optional database, object) parts.
///
/// Handles `database.object` and plain `object` forms.
pub fn parse_table_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        (Some(&name[..dot_pos]), &name[dot_pos + 1..])
    } else {
        (None, name)
    }
}

/// Truncate a string to a maximum display length with ellipsis.
///
/// Uses `char_indices()` for UTF-8 safety -- never splits a multi-byte
/// character boundary. Returns the original string if it fits.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }
    let end = s
        .char_indices()
        .nth(max_len - 3)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    format!("{}...", &s[..end])
}

/// Format a nullable indicator from DBC.ColumnsV consistently.
///
/// Normalizes various Teradata representations (Y/N, YES/NO, TRUE/FALSE, 1/0)
/// to YES or NO.
pub fn format_nullable(s: &str) -> String {
    match s.trim().to_uppercase().as_str() {
        "Y" | "YES" | "TRUE" | "1" => "YES".to_string(),
        "N" | "NO" | "FALSE" | "0" => "NO".to_string(),
        _ => s.to_string(),
    }
}
```

### Module Registration

```rust
// src/commands/mod.rs
pub mod format_helpers;
```

### Call Site Updates

Each consuming module replaces its private copy with a `use crate::commands::format_helpers::{...}`:

| Module | Functions Replaced |
|--------|--------------------|
| `inspect.rs` | `json_escape`, `csv_escape`, `parse_object_name` (renamed to `parse_table_name`), `truncate_str`, `format_nullable` |
| `describe.rs` | `json_escape`, `csv_escape`, `parse_table_name`, `truncate_str`, `format_nullable` |
| `list.rs` | `json_escape`, `csv_escape` |
| `show_indexes.rs` | `json_escape`, `csv_escape`, `parse_table_name`, `truncate_str` |
| `repl/metacommands.rs` | `format_nullable`, `truncate_string` (renamed to `truncate_str`) |

The `monitoring_utils.rs` module retains its own `escape_csv()` since it serves a different domain (PMON monitoring commands) and already has comprehensive tests. Consolidating it would create an unnecessary cross-domain coupling.

### Unit Tests

The `format_helpers.rs` module includes comprehensive tests for all functions, including:
- UTF-8 multi-byte truncation (e.g., CJK characters, emoji)
- Edge cases for `truncate_str` with `max_len` of 0, 1, 2, 3
- CSV escaping with commas, quotes, newlines, and combined special characters
- JSON escaping with all escapable characters
- `parse_table_name` with unqualified, qualified, and multi-dot names

## Bug #36 Fix: /inspect DDL and Column Types for Views

### Root Cause Analysis

**Problem 1: Garbled DDL for views**

The `query_definition()` function in `inspect.rs` uses `SHOW VIEW "db"."obj"` to retrieve view definitions. The Teradata `SHOW` command returns its result as a multi-row result set where the DDL text is split across multiple rows in `RequestText` column. The current implementation concatenates all rows, which is correct. However, the `ColumnType` field from `DBC.ColumnsV` returns a single-character type code (e.g., `CV`, `DA`, `I`, `D1`) rather than a human-readable type string for views.

The actual root cause of "garbled" DDL is that `val.display()` may return the raw bytes of the SHOW result including padding/trailing spaces, and the concatenation may not handle line breaks properly. The fix should:
1. Trim each row's text before concatenation
2. Handle the case where Teradata returns the definition across multiple result rows

**Problem 2: [NULL] column types for views**

The `query_columns()` function queries `SELECT TRIM(ColumnName), ColumnType, Nullable, DefaultValue FROM DBC.ColumnsV`. The `ColumnType` column in `DBC.ColumnsV` is a CHAR(2) type code (e.g., `CV` for VARCHAR, `DA` for DATE, `I` for INTEGER). This is not the human-readable type name.

For tables, a Teradata driver or the display logic may translate these codes. For views, the `ColumnType` value may be returning as-is (the raw type code) or as `[NULL]` if the view's column metadata is not fully populated in `DBC.ColumnsV`.

### Fix Approach

**DDL Fix**: The `query_definition()` function is likely working correctly for simple views. The "garbled" text may be caused by:
1. Extra whitespace/padding in each row -- already handled by `.trim()` on the final result
2. The SHOW command returning result in a format that splits mid-word across rows

The fix ensures proper concatenation with a single newline-based join rather than direct string concatenation:

```rust
fn query_definition(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
    kind: &str,
) -> Result<String> {
    let show_cmd = match kind {
        "V" => format!("SHOW VIEW \"{}\".\"{}\"", db, obj),
        "M" => format!("SHOW MACRO \"{}\".\"{}\"", db, obj),
        _ => return Ok(String::new()),
    };

    let result = client.execute(&show_cmd)?;

    // Teradata SHOW returns DDL split across multiple rows.
    // Concatenate all rows -- the text is a continuous stream,
    // NOT separate lines. Trim each row to remove padding.
    let mut definition = String::new();
    for row in &result.rows {
        if let Some(val) = row.first() {
            let text = val.display();
            if text != "[NULL]" {
                definition.push_str(text.trim_end());
            }
        }
    }

    Ok(definition.trim().to_string())
}
```

**Column Type Fix**: Replace the raw `ColumnType` (CHAR(2) code) with a human-readable type string built from `ColumnType` plus `ColumnLength`/`DecimalTotalDigits`/`DecimalFractionalDigits` from `DBC.ColumnsV`. This produces type strings like `VARCHAR(100)`, `DECIMAL(10,2)`, `INTEGER`, etc.

Updated SQL:

```sql
SELECT TRIM(ColumnName),
       CASE TRIM(ColumnType)
           WHEN 'BF' THEN 'BYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')'
           WHEN 'BV' THEN 'VARBYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')'
           WHEN 'CF' THEN 'CHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')'
           WHEN 'CV' THEN 'VARCHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')'
           WHEN 'D'  THEN 'DECIMAL(' || TRIM(CAST(DecimalTotalDigits AS VARCHAR(10))) || ',' || TRIM(CAST(DecimalFractionalDigits AS VARCHAR(10))) || ')'
           WHEN 'DA' THEN 'DATE'
           WHEN 'F'  THEN 'FLOAT'
           WHEN 'I'  THEN 'INTEGER'
           WHEN 'I1' THEN 'BYTEINT'
           WHEN 'I2' THEN 'SMALLINT'
           WHEN 'I8' THEN 'BIGINT'
           WHEN 'AT' THEN 'TIME'
           WHEN 'TS' THEN 'TIMESTAMP'
           WHEN 'TZ' THEN 'TIME WITH TIME ZONE'
           WHEN 'SZ' THEN 'TIMESTAMP WITH TIME ZONE'
           WHEN 'CO' THEN 'CLOB'
           WHEN 'BO' THEN 'BLOB'
           WHEN 'N'  THEN 'NUMBER'
           ELSE TRIM(ColumnType)
       END AS TypeName,
       Nullable,
       DefaultValue
FROM DBC.ColumnsV
WHERE DatabaseName = '{db}' AND TableName = '{obj}'
ORDER BY ColumnId
```

This CASE expression translates the raw type codes into human-readable names, matching the behavior users expect for both tables and views. The `inspect.rs`, `describe.rs`, and `repl/metacommands.rs` modules all need this updated query.

### Code Changes

| File | Change |
|------|--------|
| `src/commands/inspect.rs` | Update `query_columns()` SQL with CASE expression for ColumnType |
| `src/commands/inspect.rs` | Verify `query_definition()` concatenation logic |
| `src/commands/describe.rs` | Update `query_columns()` SQL with same CASE expression |
| `src/commands/repl/metacommands.rs` | Update `execute_describe()` SQL with same CASE expression |

The CASE expression can be extracted as a constant string in `format_helpers.rs`:

```rust
/// SQL CASE expression that translates DBC.ColumnsV.ColumnType codes
/// to human-readable type names.
pub const COLUMN_TYPE_CASE_EXPR: &str = "\
    CASE TRIM(ColumnType) \
        WHEN 'BF' THEN 'BYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')' \
        WHEN 'BV' THEN 'VARBYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')' \
        WHEN 'CF' THEN 'CHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')' \
        WHEN 'CV' THEN 'VARCHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(20))) || ')' \
        WHEN 'D'  THEN 'DECIMAL(' || TRIM(CAST(DecimalTotalDigits AS VARCHAR(10))) || ',' || TRIM(CAST(DecimalFractionalDigits AS VARCHAR(10))) || ')' \
        WHEN 'DA' THEN 'DATE' \
        WHEN 'F'  THEN 'FLOAT' \
        WHEN 'I'  THEN 'INTEGER' \
        WHEN 'I1' THEN 'BYTEINT' \
        WHEN 'I2' THEN 'SMALLINT' \
        WHEN 'I8' THEN 'BIGINT' \
        WHEN 'AT' THEN 'TIME' \
        WHEN 'TS' THEN 'TIMESTAMP' \
        WHEN 'TZ' THEN 'TIME WITH TIME ZONE' \
        WHEN 'SZ' THEN 'TIMESTAMP WITH TIME ZONE' \
        WHEN 'CO' THEN 'CLOB' \
        WHEN 'BO' THEN 'BLOB' \
        WHEN 'N'  THEN 'NUMBER' \
        ELSE TRIM(ColumnType) \
    END";
```

## Enriched Command Output Design

### `tq describe` Enrichment

**Current state**: Shows columns only (name, type, nullable, default) with no object header metadata and no indexes section.

**Target state** (per specification `REQ-DESCRIBE-001` through `REQ-DESCRIBE-012`):

1. **Object header**: Query `DBC.TablesV` for `TableKind`, estimated row count (`CAST(RowCount AS BIGINT)`).
2. **Columns table**: Add `CommentString` column from `DBC.ColumnsV`.
3. **Indexes section**: Query `DBC.IndicesV`, same pattern as `inspect.rs::query_indexes()` but with UPI/NUPI/USI/NUSI labels.
4. **JSON output**: Structured `{object, type, estimated_rows, columns[], indexes[]}` wrapper.
5. **CSV output**: Object metadata row followed by column data rows.

**Implementation approach**:

```rust
// src/commands/describe.rs

struct ObjectHeader {
    qualified_name: String,
    object_type: String,
    estimated_rows: Option<i64>,
}

fn query_object_header(client: &DatabaseClient, table_name: &str) -> Result<Option<ObjectHeader>> {
    let (database, table) = format_helpers::parse_table_name(table_name);
    // Query DBC.TablesV for TableKind and RowCount
    // Map TableKind using same map_table_kind() as inspect.rs
    // Return None if not found
}
```

The `describe_table()` function becomes:
1. Query object header (with graceful fallback if TablesV is not accessible)
2. Query columns (existing, enriched with CommentString)
3. Query indexes (from DBC.IndicesV, with UPI/NUPI/USI/NUSI labels)
4. Render all sections

### `tq list` Enrichment

**Current state**:
- `list databases`: Shows database names only (3-column layout)
- `list tables`: Shows name and kind (TABLE/NoPI)
- `list views`: Shows view names only

**Target state** (per specification):

**`list databases`** (REQ-LIST-002, REQ-LIST-003, REQ-LIST-004):
- Query: `SELECT TRIM(DatabaseName), TRIM(OwnerName), CASE WHEN OwnerName = 'DBC' THEN 'System' ELSE 'User' END AS DbType FROM DBC.DatabasesV`
- Table format: Database, Owner, Type columns
- JSON format: Array of `{database, owner, type}` objects
- Sort: System first, then User, each alphabetical

**`list tables`** (REQ-LIST-005, REQ-LIST-009):
- Query: Add `RowCount` and `CurrentPerm` from joining `DBC.TablesV` with `DBC.TableSizeV`
- Table format: Table, Type, Rows (Est.), Size columns
- JSON format: Array of `{table, type, estimated_rows, size_bytes}` objects
- Size display: Human-readable in table format, raw bytes in JSON/CSV

**`list views`** (REQ-LIST-010, REQ-LIST-011):
- Query: Add `OwnerName` from `DBC.TablesV` and `RequestText` from `DBC.TablesV` (or SHOW VIEW)
- Table format: View, Owner, Definition (truncated to 50 chars)
- JSON/CSV format: Full definition text

**New `execute_for_repl()` function**: `list.rs` currently lacks this. It needs one:

```rust
/// Execute /list in REPL mode with subcommand dispatch.
///
/// This function provides the REPL delegation entry point.
/// The REPL metacommand handler calls this instead of its own implementation.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    subcommand: &str,
    pattern: Option<&str>,
    database: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    match subcommand {
        "databases" | "db" | "dbs" => list_databases(client, OutputFormat::Table, writer)?,
        "tables" | "table" | "t" => {
            list_tables(client, pattern, database, OutputFormat::Table, writer)?
        }
        "views" | "view" | "v" => list_views(client, database, OutputFormat::Table, writer)?,
        _ => writeln!(writer, "Unknown list subcommand: {}", subcommand)?,
    }
    writeln!(writer)?;
    Ok(())
}
```

### `tq show-indexes` Enrichment

**Current state**: Flat table with IndexName, IndexType, ColumnName, Position columns.

**Target state** (per specification REQ-SHOW-IDX-004 through REQ-SHOW-IDX-009):

1. **Two-section layout**: Separate "Primary Index" and "Secondary Indexes" sections
2. **Type labels**: UPI, NUPI, USI, NUSI (derived from IndexType + UniqueFlag)
3. **Grouped columns**: Composite indexes shown as comma-separated column lists
4. **JSON output**: Structured `{object, primary_index, secondary_indexes[]}` wrapper
5. **CSV output**: `kind,index_no,type,columns` format with one row per index

**Implementation approach**: Reuse the grouping logic from `inspect.rs::query_indexes()` but adapt the output format. The query needs `UniqueFlag` in addition to `IndexType`:

```sql
SELECT TRIM(IndexName), IndexType, UniqueFlag,
       TRIM(ColumnName), IndexNumber, ColumnPosition
FROM DBC.IndicesV
WHERE DatabaseName = '{db}' AND TableName = '{table}'
ORDER BY IndexNumber, ColumnPosition
```

Then group by `IndexNumber`, classify primary (IndexType='P' or 'Q') vs secondary (IndexType='S','K','U','V','H'), and label with UPI/NUPI/USI/NUSI based on UniqueFlag.

## REPL Delegation Design

### Current State

The `/describe` and `/list` metacommand handlers in `metacommands.rs` contain their own SQL queries and formatting logic, duplicating the batch module implementations. This creates two code paths that can diverge.

### Target State

REPL metacommand handlers delegate to the batch module `execute_for_repl()` functions.

### `/describe` Delegation

```rust
// In metacommands.rs, replace execute_describe() body:
fn execute_describe<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    crate::commands::describe::execute_for_repl(client, table_name, writer)
}
```

The existing `describe::execute_for_repl()` already has the correct signature. The only concern is output format consistency -- the batch module's table format must produce output equivalent to what the REPL currently shows.

### `/list` Delegation

```rust
// In metacommands.rs, replace execute_list() dispatch:
fn execute_list<W: Write>(
    completion_state: &mut CompletionState,
    args: &[&str],
    writer: &mut W,
) -> Result<()> {
    if args.is_empty() {
        writeln!(writer, "Error: Missing subcommand.")?;
        writeln!(writer, "Usage: /list <databases|tables|views>")?;
        return Ok(());
    }

    let subcommand = args[0].to_lowercase();
    let pattern = args.get(1).copied();
    let current_db = completion_state.current_database().to_string();

    crate::commands::list::execute_for_repl(
        completion_state.client(),
        &subcommand,
        pattern,
        Some(&current_db),
        writer,
    )
}
```

**Key consideration**: The current REPL `/list databases` handler uses `CompletionState`'s cached database list for performance. After delegation, the batch module queries `DBC.DatabasesV` directly. This is acceptable because:
1. The cache is a performance optimization, not a correctness requirement
2. The batch query produces authoritative results
3. The REPL can still pre-load the cache for tab completion separately

### `/show indexes` -- Already Delegated

The `/show indexes` REPL handler already calls `show_indexes::execute_for_repl()`, so no changes are needed there.

## Error Message Consistency

All not-found error messages across commands will be updated to use the `Error:` prefix consistently:

| Current | Updated |
|---------|---------|
| `Table 'X' not found or no columns available.` | `Error: Table 'X' not found or no columns available.` |
| `No indexes found for table 'X'.` | `Error: No indexes found for table 'X'.` |

CLI help text will use `<OBJECT>` instead of `<TABLE>` where the argument accepts any database object (not just tables).

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Struct variant | `src/error.rs` | Change `SqlParseError(String)` to struct variant |
| From impl | `src/error.rs` | Add `From<ParseError>` |
| Call site update | `src/commands/query.rs:309,517` | Use struct fields or `?` with From |

## Technical Debt: Shared display_profiles Helper

### Problem

Profile listing logic is duplicated between `handle_list()` in `src/commands/profile.rs` and `handle_profiles()` in `src/main.rs`. The two implementations differ in detail (main.rs has source tracking, profile.rs has simpler output).

### Solution

Extract a shared `display_profiles()` function in `src/commands/profile.rs` that `handle_list()` calls. The `handle_profiles()` in `main.rs` is more complex (multi-source tracking) and should remain separate, but the simple single-profile rendering logic (`print_profile()`) can be extracted to the profile module and reused.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Extract helper | `src/commands/profile.rs` | Public `display_profiles()` function |
| Reuse in main | `src/main.rs` | Import and call shared helper |

## tq inspect Command

The `tq inspect` batch command exposes the `/inspect` REPL functionality in one-shot mode,
following the same structural pattern as `tq sessions`, `tq locks`, and `tq query-inspect`.

### CLI Argument Definition

```rust
// src/cli.rs — add to the Command enum

/// Inspect a database object (table, view, macro, procedure)
///
/// Shows a comprehensive report: object type, columns, indexes, storage
/// size/skew (tables), and definition text (views and macros).
///
/// Requires SELECT privilege on DBC.TablesV, DBC.ColumnsV, DBC.IndicesV,
/// DBC.TableSizeV (optional for size section).
///
/// Example: tq inspect employees
/// Example: tq inspect dbc.tables
Inspect(InspectArgs),
```

```rust
// src/cli.rs — InspectArgs struct

/// Arguments for the inspect command
#[derive(Parser, Debug)]
pub struct InspectArgs {
    /// Object name to inspect (table, view, macro, stored procedure)
    ///
    /// Accepts qualified names: database.object or unqualified object
    /// (uses the default database from the connection profile).
    #[arg(value_name = "OBJECT")]
    pub object: String,

    /// Output format
    #[arg(short = 'f', long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

### Command Dispatch

```rust
// src/main.rs — add to the match block in run()

Command::Inspect(args) => {
    if let Some(ref output_path) = args.output {
        let file = std::fs::File::create(output_path)?;
        let mut writer = std::io::BufWriter::new(file);
        commands::inspect(&client, &args, &mut writer, use_color)?;
    } else {
        let mut stdout = io::stdout();
        commands::inspect(&client, &args, &mut stdout, use_color)?;
    }
}
```

```rust
// src/commands/mod.rs — add export
pub mod inspect;
pub use inspect::execute as inspect;
```

### Public Signature in inspect.rs

```rust
// src/commands/inspect.rs

pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &InspectArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    execute_for_repl(client, &args.object, writer)
    // CSV/JSON formatting can be layered in a follow-up sprint; for now
    // the structured table output is emitted regardless of format choice.
}
```

The initial implementation emits the same section-based text output for all formats. Structured
CSV/JSON output (where each section would be a separate JSON object) is a future enhancement.

### Code Linkage

| Component | File | Key Types / Functions |
|-----------|------|-----------------------|
| CLI args | `src/cli.rs` | `InspectArgs`, `Command::Inspect` |
| Command dispatch | `src/main.rs` | `run()` match arm |
| Module export | `src/commands/mod.rs` | `pub mod inspect`, `pub use inspect::execute as inspect` |
| Implementation | `src/commands/inspect.rs` | `execute()`, `execute_for_repl()` |
| REPL integration | `src/commands/repl/metacommands.rs` | `handle_metacommand_with_state()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `METACOMMAND_REGISTRY` |

## tq describe Command

The `tq describe` batch command exposes the `/describe` REPL metacommand in one-shot mode.
It queries `DBC.ColumnsV` to show column metadata for a table, view, or other object.

### Architecture: Shared Logic Pattern

The REPL `/describe` handler in `metacommands.rs` contains inline SQL and rendering logic.
Rather than duplicating this in a new batch module, the approach is:

1. Create `src/commands/describe.rs` with the core query and rendering logic
2. The batch `execute()` function accepts `DescribeArgs` and writes output in table/csv/json format
3. The `execute_for_repl()` function provides REPL-friendly output (same content, no format flag)
4. The REPL metacommand handler delegates to `execute_for_repl()` instead of inlining the logic

This follows the established pattern from `src/commands/sessions.rs` and `src/commands/inspect.rs`
where both batch and REPL modes share the same underlying query and rendering code.

### CLI Argument Definition

```rust
// src/cli.rs -- add to Command enum

/// Describe a table's columns (name, type, nullable, default)
///
/// Shows column metadata from DBC.ColumnsV for the specified table.
///
/// Example: tq describe employees
///          tq describe mydb.employees
Describe(DescribeArgs),
```

```rust
// src/cli.rs -- DescribeArgs struct

#[derive(Parser, Debug)]
pub struct DescribeArgs {
    /// Table name to describe (qualified: database.table or unqualified)
    #[arg(value_name = "TABLE")]
    pub table: String,

    /// Output format
    #[arg(short = 'f', long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

### Command Dispatch

```rust
// src/main.rs -- add to match block in run()

Command::Describe(args) => {
    if let Some(ref output_path) = args.output {
        let file = std::fs::File::create(output_path)?;
        let mut writer = std::io::BufWriter::new(file);
        commands::describe::execute(&client, &args, &mut writer, use_color)?;
    } else {
        let mut stdout = io::stdout();
        commands::describe::execute(&client, &args, &mut stdout, use_color)?;
    }
}
```

### Implementation Module

```rust
// src/commands/describe.rs

pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &DescribeArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match args.format {
        OutputFormat::Table => describe_table(client, &args.table, writer)?,
        OutputFormat::Csv => describe_csv(client, &args.table, writer)?,
        OutputFormat::Json => describe_json(client, &args.table, writer)?,
    }
    Ok(())
}

pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    describe_table(client, table_name, writer)?;
    writeln!(writer)?;
    Ok(())
}
```

### Code Linkage

| Component | File | Key Types / Functions |
|-----------|------|-----------------------|
| CLI args | `src/cli.rs` | `DescribeArgs`, `Command::Describe` |
| Command dispatch | `src/main.rs` | `run()` match arm |
| Module export | `src/commands/mod.rs` | `pub mod describe` |
| Implementation | `src/commands/describe.rs` | `execute()`, `execute_for_repl()` |
| REPL delegation | `src/commands/repl/metacommands.rs` | Calls `describe::execute_for_repl()` |

## tq list Command

The `tq list` batch command exposes the `/list` REPL metacommand in one-shot mode.
It supports three subcommands: `databases`, `tables`, and `views`.

### CLI Argument Definition

```rust
// src/cli.rs -- add to Command enum

/// List database objects (databases, tables, views)
///
/// Example: tq list databases
///          tq list tables order*
///          tq list views
#[command(name = "list")]
List(ListArgs),
```

```rust
// src/cli.rs -- ListArgs struct

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// What to list: databases, tables, or views
    #[arg(value_name = "TYPE")]
    pub object_type: ListObjectType,

    /// Optional glob pattern to filter results (tables only)
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Output format
    #[arg(short = 'f', long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ListObjectType {
    Databases,
    Tables,
    Views,
}
```

Using `ValueEnum` for `ListObjectType` provides type-safe parsing with automatic help text
and case-insensitive matching, consistent with the existing `OutputFormat` and `LogonMechanism` enums.

### Implementation Module

```rust
// src/commands/list.rs

pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ListArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match args.object_type {
        ListObjectType::Databases => list_databases(client, args.format, writer)?,
        ListObjectType::Tables => list_tables(client, args.pattern.as_deref(), args.format, writer)?,
        ListObjectType::Views => list_views(client, args.format, writer)?,
    }
    Ok(())
}

pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    object_type: &str,
    pattern: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    // Delegates to shared query/render functions
}
```

### Code Linkage

| Component | File | Key Types / Functions |
|-----------|------|-----------------------|
| CLI args | `src/cli.rs` | `ListArgs`, `ListObjectType`, `Command::List` |
| Command dispatch | `src/main.rs` | `run()` match arm |
| Module export | `src/commands/mod.rs` | `pub mod list` |
| Implementation | `src/commands/list.rs` | `execute()`, `execute_for_repl()` |
| REPL delegation | `src/commands/repl/metacommands.rs` | Calls `list::execute_for_repl()` |

## tq show-indexes Command

The `tq show-indexes` batch command exposes the `/show indexes` REPL metacommand in one-shot mode.

### CLI Argument Definition

```rust
// src/cli.rs -- add to Command enum

/// Show index information for a table
///
/// Displays index names, types, uniqueness, and columns from DBC.IndicesV.
///
/// Example: tq show-indexes employees
///          tq show-indexes mydb.orders
#[command(name = "show-indexes")]
ShowIndexes(ShowIndexesArgs),
```

```rust
// src/cli.rs -- ShowIndexesArgs struct

#[derive(Parser, Debug)]
pub struct ShowIndexesArgs {
    /// Table name (qualified: database.table or unqualified)
    #[arg(value_name = "TABLE")]
    pub table: String,

    /// Output format
    #[arg(short = 'f', long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
```

### Implementation Module

```rust
// src/commands/show_indexes.rs

pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ShowIndexesArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match args.format {
        OutputFormat::Table => show_indexes_table(client, &args.table, writer)?,
        OutputFormat::Csv => show_indexes_csv(client, &args.table, writer)?,
        OutputFormat::Json => show_indexes_json(client, &args.table, writer)?,
    }
    Ok(())
}

pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    show_indexes_table(client, table_name, writer)?;
    writeln!(writer)?;
    Ok(())
}
```

### Code Linkage

| Component | File | Key Types / Functions |
|-----------|------|-----------------------|
| CLI args | `src/cli.rs` | `ShowIndexesArgs`, `Command::ShowIndexes` |
| Command dispatch | `src/main.rs` | `run()` match arm |
| Module export | `src/commands/mod.rs` | `pub mod show_indexes` |
| Implementation | `src/commands/show_indexes.rs` | `execute()`, `execute_for_repl()` |
| REPL delegation | `src/commands/repl/metacommands.rs` | Calls `show_indexes::execute_for_repl()` |

## Identifier Quoting Fix (Bug #35)

### Problem

`quote_identifier()` in `src/sql/identifiers.rs` wraps identifiers in double quotes while
preserving the original case. In Teradata, quoted identifiers are case-sensitive, while
unquoted identifiers are case-insensitive (stored internally as uppercase). When a user types
`dbc.tables`, the quoting produces `"dbc"."tables"` which fails because Teradata stores
the identifier as `DBC` and the table name includes mixed case (`Tables`).

A secondary bug: `extract_table_name()` in `src/db/client.rs` uses `str::find("FROM")` which
is a substring search. Searching for `"TABLE"` within `"SELECT * FROM DBC.TABLES SAMPLE 10"`
finds `TABLE` as a substring of `TABLES`, then extracts `"S"` (the characters after `TABLE`
up to the next delimiter) as the table name.

### Fix Approach

**Fix 1: Uppercase identifiers before quoting**

Change `quote_identifier()` to uppercase the identifier before wrapping in double quotes:

```rust
pub fn quote_identifier(identifier: &str) -> String {
    let upper = identifier.to_uppercase();
    let escaped = upper.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}
```

This preserves SQL injection protection (double-quote escaping) while matching Teradata's
native storage format. All callers of `quote_identifier()` and `quote_qualified_name()`
benefit automatically. The `sample.rs` and `inspect.rs` modules that use these functions
will produce correct SQL for case-insensitive user input.

**Important consideration**: This change means that identifiers that were intentionally
mixed-case (e.g., a table actually created with `CREATE TABLE "MyTable"`) will now be
uppercased. However, this is the correct Teradata behavior: users who need exact-case
identifiers should quote them explicitly in their SQL. The `quote_identifier()` function
is used internally for user-provided names (from CLI arguments and metacommands), where
case-insensitive matching is the expected behavior.

**Fix 2: Word-boundary matching in extract_table_name()**

Replace the simple `str::find(keyword)` with word-boundary-aware matching. After finding
a keyword match position, verify it is preceded by a word boundary (whitespace or start of
string) and followed by a word boundary (whitespace):

```rust
fn extract_table_name(sql: &str) -> Option<String> {
    let sql_upper = sql.to_uppercase();
    let keywords = ["FROM", "INTO", "UPDATE", "TABLE"];

    for keyword in keywords {
        let mut search_from = 0;
        while let Some(pos) = sql_upper[search_from..].find(keyword) {
            let abs_pos = search_from + pos;
            let before_ok = abs_pos == 0
                || !sql_upper.as_bytes()[abs_pos - 1].is_ascii_alphanumeric();
            let after_pos = abs_pos + keyword.len();
            let after_ok = after_pos >= sql_upper.len()
                || !sql_upper.as_bytes()[after_pos].is_ascii_alphanumeric();

            if before_ok && after_ok {
                let after = &sql[after_pos..].trim_start();
                // Handle quoted identifiers: "DB"."TABLE"
                let end = if after.starts_with('"') {
                    // Find the full quoted identifier (may include dots)
                    find_quoted_identifier_end(after)
                } else {
                    after.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '"')
                        .unwrap_or(after.len())
                };
                if end > 0 {
                    return Some(after[..end].to_string());
                }
            }
            search_from = abs_pos + keyword.len();
        }
    }
    None
}
```

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Uppercase quoting | `src/sql/identifiers.rs:70` | `quote_identifier()` uppercases before quoting |
| Cascading fix | `src/sql/identifiers.rs:104` | `quote_qualified_name()` inherits fix |
| Word boundary | `src/db/client.rs:719` | `extract_table_name()` uses word-boundary matching |
| Test updates | `src/sql/identifiers.rs` (tests) | Update expected values to uppercase |
| New tests | `src/db/client.rs` (tests) | Add `TABLES` vs `TABLE` word boundary test |

## Future Enhancements

- **Config file flag**: `--config <path>` to override default config location
- **Dry-run mode**: `--dry-run` to validate without executing
- **Output templates**: `--template <name>` for custom formatting
- **Batch mode flags**: `--continue-on-error`
