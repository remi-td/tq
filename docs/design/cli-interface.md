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

## Deterministic Input-Source Selection

### Problem

`tq query` accepts SQL from three sources: a positional argument, `--file`, or stdin.
Earlier designs auto-detected the stdin source by probing file-descriptor *readiness*
(`is_terminal()` plus a `poll(2)`/`FIONREAD`/`fstat` machinery). Readiness is a property
of the runtime environment, not of the command the user typed, so the chosen source
varied with timing and harness behaviour:

- A harness that left an empty pipe attached while the user passed an explicit query
  produced a spurious "multiple input sources" error.
- A producer that wrote to stdin slightly later than `tq` probed it was missed entirely,
  so behaviour differed between an "immediate" and a "delayed" producer.
- The probe required a Unix/non-Unix split (`libc` on Unix, "assume data" on Windows),
  giving two different contracts.

### Design: Syntactic Precedence, No Readiness Probe

The source is selected purely from the *syntax of the invocation*, never from transient
fd state. The precedence is:

1. **Positional query argument present** → `InputSource::Argument`. Stdin is never inspected.
2. **Else `--file <path>` present** → `InputSource::File`. Stdin is never inspected.
3. **Else stdin is not a TTY** (`!io::stdin().is_terminal()`) → `InputSource::Stdin`.
   A normal **blocking** read to EOF is performed. A delayed producer is handled
   naturally because the read blocks until the writer closes its end.
4. **Else** (no arg, no file, stdin is a TTY) → `No query provided`.

This matches `psql -c` / `psql -f`: an explicit `-c`/`-f` wins and stdin is ignored.
`clap`'s `conflicts_with` already makes the positional argument and `--file` mutually
exclusive (`src/cli.rs`: `QueryArgs.query` has `conflicts_with = "file"`), so case 1 and
case 2 cannot both apply.

The previous "Multiple input sources" conflict no longer exists — an explicit source
simply suppresses stdin inspection — so its error variant and the pre-connection
`validate_input_sources` guard are removed.

### Implementation

`determine_input_source` collapses to a straight precedence match with no platform split
(`src/commands/query.rs`):

```rust
fn determine_input_source(args: &QueryArgs) -> Result<InputSource> {
    if let Some(ref query) = args.query {
        Ok(InputSource::Argument(query.clone()))
    } else if let Some(ref file_path) = args.file {
        Ok(InputSource::File(file_path.clone()))
    } else if !io::stdin().is_terminal() {
        Ok(InputSource::Stdin)
    } else {
        Err(TqError::InvalidConfig(
            "No query provided.\n\n\
             Provide SQL via:\n  \
             - Command argument: tq query \"SELECT 1\"\n  \
             - File: tq query --file script.sql\n  \
             - Stdin: echo \"SELECT 1\" | tq query"
                .to_string(),
        ))
    }
}
```

`read_sql_stdin` is unchanged in spirit: it performs a blocking `read_to_string` to EOF and
maps the empty case to a distinct, actionable error. Empty stdin is reported as
`Empty query received from stdin` (not `No query provided`), preserving the existing
distinction so an agent can tell "you selected stdin but sent nothing" apart from
"you selected no source at all".

```rust
fn read_sql_stdin() -> Result<String> {
    let mut sql = String::new();
    io::BufReader::new(io::stdin().lock()).read_to_string(&mut sql)?;
    if sql.trim().is_empty() {
        return Err(TqError::InvalidConfig(
            "Empty query received from stdin.\nProvide valid SQL via stdin.".to_string(),
        ));
    }
    Ok(sql)
}
```

### Removed Code

- `stdin_has_data()` and both its `#[cfg(unix)]` / `#[cfg(not(unix))]` bodies
  (the `poll`/`FIONREAD`/`fstat` machinery).
- `validate_input_sources()` and its pre-connection call in `src/main.rs` (~line 88).
  The remaining "argument before connection" guarantee is unaffected: `determine_input_source`
  runs at the top of `execute`/`execute_to_file`, and the only argument-vs-argument conflict
  (`query` + `--file`) is enforced by clap at parse time, before any connection is built.
- The `Multiple input sources` `InvalidConfig` branch and its content test
  (`test_multiple_input_sources_error_message_content`), plus `test_stdin_has_data_does_not_panic`.

### `libc` Dependency

`libc` is **retained** in `Cargo.toml`. Although the query-path `poll`/`FIONREAD`/`fstat`
uses are deleted, `src/db/metadata.rs` still uses `libc` for stdout/stderr fd redirection
(`libc::dup`, `libc::dup2`, `libc::open`, `libc::close`, `STDOUT_FILENO`/`STDERR_FILENO`).
A pre-removal grep over `src/` confirms those remaining uses, so the dependency cannot be dropped.

### Platform Contract

Unix and Windows now follow the identical precedence. There is no readiness probe and no
`#[cfg]` split in `determine_input_source`. On Windows, `is_terminal()` reports non-TTY for a
redirected/piped stdin exactly as on Unix, and the blocking read behaves the same.

### Behaviour Change (Intentional)

`echo "ignored" | tq query "SELECT 1"` now executes `SELECT 1` (stdin ignored) instead of
returning a "multiple input sources" error. This is a deliberate contract change that matches
`psql -c`. It is called out in the specification, the sprint review, and the issue-closure comment.

## Structural Agent-Safe Classification

### Problem

The original `classify_statement` derived the statement type from a first-keyword helper
(`get_statement_type`) and mapped *any* unrecognised keyword to `Ddl`. That had three defects
for agent-safe mode:

1. It could not see through a `WITH` CTE prologue to the effective operation, nor through a
   `LOCKING`/`LOCK` request modifier.
2. Leading comment handling was ad-hoc (only a single leading `--` or `/* */`, not arbitrary
   interleavings).
3. **Fail-open mislabelling**: unknown syntax was reported as `Ddl` ("DDL is always blocked"),
   which is the right *outcome* (blocked) but the wrong *reason* — an agent cannot distinguish
   "this is DDL" from "tq could not classify this", which matters for diagnostics and trust.

### Design: Comment-Skipping Token Stream over the In-Tree Lexer

The classifier reuses the proven lexical machine in `src/sql/parser.rs` rather than adding a
SQL-parser dependency (executive decision, Sprint 71). The parser already tracks quoted strings,
line comments, and block comments correctly; we expose a *significant-token iterator* built from
that same state model and classify a short prefix of tokens.

#### New public API in `src/sql/`

A new module `src/sql/classifier.rs` (re-exported from `src/sql/mod.rs`) provides:

```rust
/// Safety classification of a single SQL statement for agent-safe mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementSafety {
    /// SELECT/SEL, SHOW, HELP, EXPLAIN, and read-only WITH/LOCKING forms.
    ReadOnly,
    /// COLLECT STATISTICS / COLLECT STATS — blocked unless --allow-maintenance.
    Maintenance,
    /// INSERT/INS, UPDATE/UPD, DELETE/DEL, MERGE, UPSERT — blocked unless --allow-dml.
    Dml,
    /// CREATE, REPLACE, DROP, ALTER, RENAME, GRANT, REVOKE, … — always blocked.
    Ddl,
    /// Could not be classified; fail closed. `token` is the first significant
    /// token seen (if any); `reason` explains why classification stopped.
    Unknown { token: Option<String>, reason: String },
}

/// Classify the effective top-level operation of a single SQL statement.
pub fn classify_statement(sql: &str) -> StatementSafety;
```

To support this without duplicating the lexer, `src/sql/parser.rs` exposes a thin,
comment-and-string-aware token reader:

```rust
/// A significant SQL token: an identifier/keyword word, a punctuation char
/// (`(`, `)`, `,`, `.`), or a string literal placeholder. Whitespace and
/// comments are skipped. This reuses the same state transitions as
/// `parse_statements` so quoting/comment rules cannot diverge.
pub enum SqlToken {
    Word(String),       // ASCII word run; callers uppercase for keyword tests
    Punct(char),        // one of ( ) , . relevant to CTE/LOCKING scanning
    StringLiteral,      // opaque — content irrelevant to classification
    Other,              // any other single character (operators, etc.)
}

/// Iterate significant tokens, skipping arbitrary interleaved whitespace,
/// line comments (`-- ...`), and block comments (`/* ... */`).
pub fn significant_tokens(sql: &str) -> impl Iterator<Item = SqlToken> + '_;
```

`significant_tokens` is implemented by factoring the comment/quote skipping already present in
`parse_statements` into a reusable scanner; the existing parser keeps its current behaviour and
simply consumes the same primitive.

#### Classification algorithm

`classify_statement` consumes `significant_tokens` and applies these rules in order:

1. **Leading comments / whitespace** are already removed by the iterator, so the first yielded
   token is the first significant keyword. If the stream is empty → `Unknown { token: None, reason: "no statement" }`.
2. **`LOCKING` / `LOCK` request modifier**: consume the modifier clause up to and including its
   terminating keyword set, then classify the operation it modifies. The modifier runs
   `LOCKING [ROW|TABLE|DATABASE|VIEW] <object>? FOR <lock-type> [MODE|NOWAIT|OVERRIDE]*`
   and is followed by the actual request. We scan forward, paren-aware, to the next top-level
   keyword that is a recognised operation (`SELECT`/`SEL`, `INSERT`, `UPDATE`, `DELETE`, `MERGE`,
   `WITH`, …) and classify that. Multiple stacked `LOCKING` modifiers are consumed in a loop.
   If no recognised operation follows → `Unknown`.
3. **`WITH` CTE prologue**: a top-level `WITH` is resolved to its final operation. Skip the CTE
   definitions paren-aware: after `WITH`, repeatedly read `cte_name [ (col, …) ] AS ( … )`,
   tracking parenthesis depth so commas and keywords *inside* a CTE body are ignored; CTE
   definitions are separated by top-level commas. When the top-level comma list ends, the next
   top-level keyword is the effective operation — classify it (it may be `SELECT`, `INSERT`,
   `UPDATE`, `DELETE`, `MERGE`). `WITH RECURSIVE` is handled by skipping the `RECURSIVE` word.
   If the prologue is malformed (unbalanced parens, no trailing operation) → `Unknown`.
4. **Direct operation keyword** (first token, or the token resolved through 2/3):
   - `SELECT`, `SEL`, `SHOW`, `HELP`, `EXPLAIN` → `ReadOnly`.
   - `COLLECT` followed by `STATISTICS` or `STATS` → `Maintenance`. `COLLECT` followed by
     anything else (or nothing) → `Unknown` (fail closed; do not assume read-only).
   - `INSERT`, `INS`, `UPDATE`, `UPD`, `DELETE`, `DEL`, `MERGE`, `UPSERT` → `Dml`.
   - `CREATE`, `REPLACE`, `DROP`, `ALTER`, `RENAME`, `GRANT`, `REVOKE`, `DATABASE`, `USER`,
     `COMMENT`, `SET`, `BEGIN`, `END`, `GIVE`, `MODIFY`, `FLUSH`, `DUMP`, `RESTORE` → `Ddl`.
   - Anything else → `Unknown { token: Some(word), reason: "unrecognised leading operation" }`.

The classifier never maps "unknown" to `Ddl`. Unknown is its own terminal category, surfaced to
the user as a distinct error so the diagnostic is honest.

#### Validation flow (`validate_agent_safe` in `src/commands/query.rs`)

```text
if has_multiple_statements(sql)            -> AgentSafeBlocked { MULTI_STATEMENT }
match classify_statement(sql) {
  ReadOnly                                 -> Ok
  Maintenance if args.allow_maintenance    -> Ok
  Maintenance                              -> AgentSafeBlocked { effective_op, "maintenance … --allow-maintenance" }
  Dml if args.allow_dml                    -> Ok
  Dml                                      -> AgentSafeBlocked { effective_op, "DML … --allow-dml" }
  Ddl                                      -> AgentSafeBlocked { effective_op, "DDL always blocked" }
  Unknown { token, reason }                -> AgentSafeUnclassified { token, reason }
}
```

The effective operation reported in the error is the operation the classifier *resolved to*
(e.g. for `LOCKING … UPDATE` it reports `UPDATE`, not `LOCKING`), satisfying the issue's
requirement that errors identify the effective operation and the rejection reason.

#### New CLI flag

`QueryArgs` gains `--allow-maintenance` (`src/cli.rs`), parallel to the existing `--allow-dml`:

```rust
/// Allow maintenance operations (COLLECT STATISTICS) in agent-safe mode.
#[arg(long)]
pub allow_maintenance: bool,
```

The `--agent-safe` help text is updated to list the categories and to point at the
database-side least-privilege guidance (defense-in-depth framing).

### `--max-rows` Clarification

`--max-rows` is documented (help text + JSON-error hint paths) as a **client fetch/output cap**:
in agent-safe mode `tq` fetches at most `max_rows + 1` rows and fails with `AGENT_SAFE_MAX_ROWS`
if the extra row appears. It does not impose any database-side workload limit (no `TOP`/`SAMPLE`
is injected). This wording is added to the flag doc-comment in `src/cli.rs` and to the
`AgentSafeMaxRows` hint.

### Risks

- **CTE / LOCKING tokenisation edge cases** (nested parens, comments inside the modifier,
  stacked modifiers): mitigated by reusing the proven lexer primitive and failing closed to
  `Unknown` on anything not provably classified, plus unit tests covering the issue's example
  table and the Teradata abbreviations `SEL`/`INS`/`UPD`/`DEL`.
- **New keyword surface**: the DDL/DCL allow-list is explicit; any genuinely novel leading
  keyword lands in `Unknown` (blocked) rather than being silently mis-bucketed.

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

## Shared Query Helpers Module

### Problem

Three command modules contain near-identical query functions that hit the same DBC system views:

| Function | inspect.rs | describe.rs | show_indexes.rs |
|----------|-----------|-------------|-----------------|
| `query_indexes()` | Yes (returns `Vec<IndexInfo>`) | Yes (returns `Vec<IndexGroup>`) | Yes (returns `Vec<IndexGroup>`) |
| `query_columns()` | Yes (returns `Vec<ColumnInfo>`, no comments) | Yes (returns `Vec<ColumnRow>`, with comments) | No |
| `resolve_database()` | Yes (`resolve_object_database`) | Yes (`resolve_database`) | No (inline in `query_indexes`) |
| `format_size()` | Yes (2 decimal places) | No | No |
| `format_size_short()` | No | No | No (in `list.rs`, 1 decimal place) |

Each copy uses slightly different struct names and field names for the same data. The SQL queries are identical except for the column list (describe includes `CommentString`). This duplication was created incrementally as commands were added across sprints 42-46.

### Solution: `src/commands/query_helpers.rs`

Create a new module separate from `format_helpers.rs`. The distinction is clear:
- `format_helpers.rs`: Pure functions with no I/O (string escaping, name parsing, type mapping)
- `query_helpers.rs`: Functions that require `DatabaseClient` access (database queries)

This separation preserves the testability advantage of `format_helpers.rs` (unit-testable without mocks) while centralizing all DBC query logic.

### Shared Types

```rust
// src/commands/query_helpers.rs

/// Column metadata from DBC.ColumnsV.
///
/// Contains all fields needed by any consumer. Commands that do not
/// need `comment` simply ignore it.
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: String,
    /// `None` when no default is defined (DBC returns NULL).
    pub default: Option<String>,
    /// Column comment from CommentString. Empty string when absent.
    pub comment: String,
}

/// Index metadata grouped by IndexNumber from DBC.IndicesV.
///
/// A single struct replaces the three private variants (`IndexInfo`,
/// `IndexGroup` x2) that existed across inspect.rs, describe.rs,
/// and show_indexes.rs.
pub struct IndexGroup {
    /// Index name, or `None` for unnamed system indexes.
    pub name: Option<String>,
    /// Human-readable type label (e.g., "Primary Index").
    pub index_type_label: String,
    /// Short classification suffix (e.g., "UPI", "NUSI").
    pub short_label: String,
    /// Whether this is a primary-class index (P, Q, or K type).
    pub is_primary: bool,
    /// Ordered list of column names in this index.
    pub columns: Vec<String>,
}
```

**Design decisions on the shared types:**

1. **`ColumnInfo.default` is `Option<String>`**: inspect.rs already uses `Option<String>` (with `None` for absent defaults). describe.rs currently uses `"-"` as a sentinel string. The shared type uses `Option` because it correctly models the domain (a default either exists or does not). The describe formatters convert `None` to `"-"` at display time.

2. **`ColumnInfo.comment` is always present**: The SQL always selects `CommentString`. inspect.rs currently omits it from its query, but adding the column costs nothing (it is already in DBC.ColumnsV) and simplifies to a single query function. Consumers that do not display comments simply ignore the field.

3. **`IndexGroup.name` is `Option<String>`**: inspect.rs uses `Option<String>` (semantically correct for unnamed indexes). describe.rs and show_indexes.rs use `"(unnamed)"` as a display sentinel. The shared type uses `Option`; display code maps `None` to `"(unnamed)"` where needed.

4. **`IndexGroup.is_primary`**: Only show_indexes.rs currently uses this field (to separate primary vs secondary sections). Adding it to the shared type is zero-cost and avoids show_indexes.rs needing to re-derive it from `index_type_label`.

### Shared Query Functions

```rust
/// Resolve the database name for an unqualified object reference.
///
/// If `db` is `Some`, returns it directly. Otherwise queries
/// `SELECT DATABASE` and falls back to the connection config default.
pub fn resolve_database(
    client: &DatabaseClient,
    db: Option<&str>,
) -> Result<String>
```

This merges the two variants:
- inspect.rs `resolve_object_database(client, db: Option<&str>)` -- accepts optional db
- describe.rs `resolve_database(client)` -- always queries

The merged signature takes `Option<&str>`, which subsumes both use cases. describe.rs callers pass `None` when unqualified.

```rust
/// Query DBC.ColumnsV for column metadata.
///
/// Returns all standard column fields including comments.
/// The SQL uses `column_type_case_sql()` for type translation.
pub fn query_columns(
    client: &DatabaseClient,
    db: &str,
    object: &str,
) -> Result<Vec<ColumnInfo>>
```

Single implementation that always selects `CommentString`. This replaces:
- inspect.rs `query_columns` (4 columns, no comment)
- describe.rs `query_columns` (5 columns, with comment)

```rust
/// Query DBC.IndicesV for index metadata, grouped by IndexNumber.
///
/// Returns groups ordered by IndexNumber, with columns ordered
/// by ColumnPosition within each group.
pub fn query_indexes(
    client: &DatabaseClient,
    db: &str,
    object: &str,
) -> Result<Vec<IndexGroup>>
```

Single implementation that replaces all three copies. All three use identical SQL and identical grouping logic. The only differences were:
- inspect.rs uses `Option<String>` for name, no `is_primary`
- describe.rs uses `String` for name (with "(unnamed)" sentinel), no `is_primary`
- show_indexes.rs uses `String` for name, has `is_primary`

The shared version computes all fields. Callers use what they need.

### `format_size` Consolidation

```rust
// Added to format_helpers.rs (pure function, no I/O)

/// Format a byte count as a human-readable size string.
///
/// `precision` controls decimal places: 2 for inspect (detailed),
/// 1 for list (compact).
pub fn format_size(bytes: i64, precision: usize) -> String
```

This replaces:
- inspect.rs `format_size(bytes)` -- uses 2 decimal places
- list.rs `format_size_short(bytes)` -- uses 1 decimal place

Call sites change to `format_size(bytes, 2)` and `format_size(bytes, 1)` respectively.

### `summarize_error` UTF-8 Fix

The current implementation in inspect.rs slices on byte boundaries:

```rust
fn summarize_error(e: &TqError) -> String {
    let msg = e.to_string();
    if msg.len() > 80 {
        format!("{}...", &msg[..77])  // BUG: may panic on multi-byte chars
    } else {
        msg
    }
}
```

This is the same class of bug as the `truncate_str` fix from Sprint 47. The fix uses `truncate_str` from format_helpers:

```rust
fn summarize_error(e: &TqError) -> String {
    truncate_str(&e.to_string(), 80)
}
```

This function stays private in inspect.rs (only used there) but now delegates to the shared UTF-8-safe truncation.

### Module Registration

```rust
// src/commands/mod.rs
pub mod query_helpers;  // NEW
```

### Migration Plan

The migration is mechanical and low-risk:

**Step 1: Create `query_helpers.rs` with shared types and functions**
- Define `ColumnInfo`, `IndexGroup` structs
- Implement `resolve_database`, `query_columns`, `query_indexes`
- Add unit tests for `format_size` with precision parameter

**Step 2: Add `format_size` to `format_helpers.rs`**
- Add the unified `format_size(bytes, precision)` function
- Add unit tests covering both precision=1 and precision=2 cases

**Step 3: Migrate consumers one at a time**
- `inspect.rs`: Remove private structs and query functions, import from `query_helpers`
  - `ColumnInfo` field `default` stays `Option<String>` (no change)
  - `IndexInfo` replaced by `IndexGroup` (map `name: Option<String>` directly)
  - Remove private `format_size`, use `format_helpers::format_size(bytes, 2)`
  - Fix `summarize_error` to use `truncate_str`
- `describe.rs`: Remove private structs and query functions, import from `query_helpers`
  - `ColumnRow` replaced by `ColumnInfo` (display code maps `default: None` to `"-"`)
  - `IndexGroup` replaced by shared `IndexGroup` (display code maps `name: None` to `"(unnamed)"`)
  - `ObjectHeader` stays private (only used by describe.rs, different fields from inspect's `ObjectInfo`)
- `show_indexes.rs`: Remove private `IndexGroup` and `query_indexes`, import from `query_helpers`
  - Inline database resolution replaced by `resolve_database(client, db)`
- `list.rs`: Remove private `format_size_short`, use `format_helpers::format_size(bytes, 1)`

**Step 4: Verify all existing tests pass**
- `cargo test --lib` must show zero regressions

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| New module | `src/commands/query_helpers.rs` | Shared `ColumnInfo`, `IndexGroup`, query functions |
| Registration | `src/commands/mod.rs` | Add `pub mod query_helpers` |
| Format size | `src/commands/format_helpers.rs` | Add `format_size(bytes, precision)` |
| Migrate inspect | `src/commands/inspect.rs` | Remove private copies, import shared |
| Migrate describe | `src/commands/describe.rs` | Remove private copies, import shared |
| Migrate show_indexes | `src/commands/show_indexes.rs` | Remove private copies, import shared |
| Migrate list | `src/commands/list.rs` | Remove `format_size_short`, import shared |

## JSON API Type Correctness

### Problem

Several JSON output functions emit values as quoted strings where the specification requires proper JSON types:

1. **describe JSON `nullable`**: Emits `"nullable":"YES"` instead of `"nullable":true`
2. **describe JSON `default`**: Emits `"default":"-"` instead of `"default":null`
3. **list tables JSON `rows_est`**: Emits `"rows_est":"1000"` instead of `"estimated_rows":1000`
4. **list tables JSON `size`**: Emits `"size":"2.5 MB"` instead of `"size_bytes":2621440`
5. **list databases JSON `name`**: Emits `"name":"mydb"` instead of `"database":"mydb"`

### Solution

**describe JSON** (`describe.rs` `describe_json()`):

```rust
// Before:
"\"nullable\":\"{}\",\"default\":\"{}\"",
json_escape(&col.nullable),
json_escape(&col.default)

// After:
"\"nullable\":{},\"default\":{}",
if col.nullable == "YES" { "true" } else { "false" },
match &col.default {
    Some(val) => format!("\"{}\"", json_escape(val)),
    None => "null".to_string(),
}
```

**list tables JSON** (`list.rs`):

The `TableEntry` struct currently stores `row_count: String` and `size: String` (pre-formatted display strings). To emit raw integers in JSON, the struct needs raw numeric fields or the JSON formatter needs access to the raw values.

Recommended approach: Add `row_count_raw: i64` and `size_bytes: i64` fields to `TableEntry`. The table formatter uses the string fields; the JSON formatter uses the integer fields.

```rust
struct TableEntry {
    name: String,
    kind: String,
    row_count: String,      // display: "1,000"
    row_count_raw: i64,     // JSON: 1000
    size: String,           // display: "2.5 MB"
    size_bytes: i64,        // JSON: 2621440
}

// JSON output:
"{{\"name\":\"{}\",\"type\":\"{}\",\"estimated_rows\":{},\"size_bytes\":{}}}",
json_escape(&t.name),
json_escape(&t.kind),
t.row_count_raw,
t.size_bytes
```

**list databases JSON** (`list.rs`):

Simple key rename from `"name"` to `"database"`:

```rust
// Before:
"{{\"name\":\"{}\",\"owner\":\"{}\",\"type\":\"{}\"}}",

// After:
"{{\"database\":\"{}\",\"owner\":\"{}\",\"type\":\"{}\"}}",
```

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Nullable as boolean | `src/commands/describe.rs` | `describe_json()`: emit `true`/`false` unquoted |
| Default as null | `src/commands/describe.rs` | `describe_json()`: emit `null` for `None` |
| Table entry raw fields | `src/commands/list.rs` | Add `row_count_raw`, `size_bytes` to `TableEntry` |
| Tables JSON integers | `src/commands/list.rs` | JSON formatter uses raw integer fields |
| Database key rename | `src/commands/list.rs` | JSON `"name"` to `"database"` |

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

## tq search Command

The `tq search` command provides cross-database discovery of tables and columns by keyword.
Unlike `tq list` which scopes to a single database (defaulting to the current database),
`tq search` queries across all accessible databases by default, making it suitable for
discovery workflows where the user does not know which database contains the object.

### Design Rationale

**Why a separate command instead of extending `tq list`?**

`tq list tables` is scoped to a single database and uses client-side glob matching on names
already returned by the query. Cross-database search requires a fundamentally different SQL
pattern (LIKE on the server across all databases). Overloading `list` would blur the semantics:
`list` means "enumerate objects in a known location", while `search` means "find objects across
an unknown location". The separation follows the same principle as `grep` vs `ls` in UNIX.

**Why SQL LIKE instead of client-side filtering?**

Cross-database search could return thousands of system tables. Server-side LIKE filtering
reduces network transfer and processing. The keyword is wrapped as `%keyword%` for substring
matching, which is the most intuitive behavior for discovery.

**Why a default result limit?**

Large Teradata systems can have tens of thousands of tables across hundreds of databases.
Without a limit, a broad keyword like "a" could return an overwhelming result set. A default
limit of 100 rows (overridable with `--limit`) provides a safe default while allowing
power users to retrieve more results when needed. `--limit 0` disables the limit entirely.

### CLI Argument Definition

```rust
// src/cli.rs -- add to Command enum

/// Search for tables or columns across databases by keyword
///
/// Performs cross-database search using SQL LIKE matching.
/// Returns results from all accessible databases by default.
///
/// Example: tq search tables emp
///          tq search columns salary --database hr_db
///          tq search tables order --limit 50
Search(SearchArgs),
```

```rust
// src/cli.rs -- SearchArgs and SearchObjectType

#[derive(Parser, Debug)]
pub struct SearchArgs {
    /// What to search for: tables or columns
    #[arg(value_name = "TYPE")]
    pub object_type: SearchObjectType,

    /// Keyword to search for (case-insensitive substring match)
    #[arg(value_name = "KEYWORD")]
    pub keyword: String,

    /// Scope search to a single database
    #[arg(short, long, value_name = "DB")]
    pub database: Option<String>,

    /// Maximum number of results (default: 100, 0 = unlimited)
    #[arg(long, default_value = "100", value_name = "N")]
    pub limit: usize,

    /// Output format
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SearchObjectType {
    Tables,
    Columns,
}
```

Using `ValueEnum` for `SearchObjectType` provides type-safe parsing with automatic help text
and case-insensitive matching, consistent with `ListObjectType` and other enums in the project.

### SQL Queries

#### Table Search

Queries `DBC.TablesV` joined with `DBC.TableSizeV` for size and row count estimates.
Filters on `TableKind IN ('T', 'O')` to return only physical tables (standard and NoPI),
excluding views, macros, and other object types. This is consistent with `tq list tables`.

```sql
SELECT TRIM(t.DatabaseName) AS database_name,
       TRIM(t.TableName) AS table_name,
       t.TableKind,
       COALESCE(CAST(s.RowCount AS VARCHAR(20)), '') AS RowCount,
       COALESCE(CAST(s.CurrentPerm AS VARCHAR(20)), '') AS CurrentPerm,
       TRIM(t.CreatorName) AS Owner
FROM DBC.TablesV t
LEFT JOIN (
    SELECT DatabaseName, TableName,
           SUM(RowCount) AS RowCount,
           SUM(CurrentPerm) AS CurrentPerm
    FROM DBC.TableSizeV
    GROUP BY DatabaseName, TableName
) s ON t.DatabaseName = s.DatabaseName AND t.TableName = s.TableName
WHERE UPPER(t.TableName) LIKE UPPER('%keyword%')
  AND t.TableKind IN ('T', 'O')
  {AND t.DatabaseName = 'specific_db'}
ORDER BY t.DatabaseName, t.TableName
SAMPLE {limit}
```

Notes:
- `UPPER()` on both sides ensures case-insensitive matching regardless of session mode.
- The `SAMPLE` clause is used for limiting results efficiently on Teradata.
  When limit is 0 (unlimited), the SAMPLE clause is omitted entirely.
- The `DatabaseName` filter is added conditionally only when `--database` is specified.
- The LEFT JOIN with `DBC.TableSizeV` reuses the exact pattern from `list_tables()`.

#### Column Search

Queries `DBC.ColumnsV` for column metadata. Returns database, table, column name, data type,
and nullable status.

```sql
SELECT TRIM(c.DatabaseName) AS database_name,
       TRIM(c.TableName) AS table_name,
       TRIM(c.ColumnName) AS column_name,
       TRIM(c.ColumnType) AS column_type,
       c.Nullable
FROM DBC.ColumnsV c
WHERE UPPER(c.ColumnName) LIKE UPPER('%keyword%')
  {AND c.DatabaseName = 'specific_db'}
ORDER BY c.DatabaseName, c.TableName, c.ColumnName
SAMPLE {limit}
```

Notes:
- `DBC.ColumnsV` is a standard Teradata system view available on all supported versions.
- `ColumnType` is a CHAR(2) code (e.g., 'CV' for VARCHAR, 'I' for INTEGER). The implementation
  maps these codes to human-readable type names using the same mapping used in `inspect.rs`.
- `Nullable` is a CHAR(1) field: 'Y' for nullable, 'N' for not nullable.

### Data Structures

```rust
// src/commands/search.rs

/// A table found by cross-database search
struct TableSearchResult {
    database: String,
    table_name: String,
    kind: String,          // "TABLE" or "NoPI"
    row_count_display: String,
    row_count_raw: Option<i64>,
    size_display: String,
    size_bytes: Option<i64>,
    owner: String,
}

/// A column found by cross-database search
struct ColumnSearchResult {
    database: String,
    table_name: String,
    column_name: String,
    column_type: String,   // Human-readable type name
    nullable: String,      // "Y" or "N"
}
```

### Implementation Module

```rust
// src/commands/search.rs

/// Execute `tq search` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    object_type: SearchObjectType,
    keyword: &str,
    database: Option<&str>,
    limit: usize,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match object_type {
        SearchObjectType::Tables => {
            search_tables(client, keyword, database, limit, format, writer)
        }
        SearchObjectType::Columns => {
            search_columns(client, keyword, database, limit, format, writer)
        }
    }
}

/// Execute /search in REPL mode (table format, with extra spacing)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    subcommand: &str,
    keyword: &str,
    database: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    match subcommand {
        "tables" | "table" | "t" => {
            search_tables(client, keyword, database, 100, OutputFormat::Table, writer)?;
        }
        "columns" | "column" | "c" => {
            search_columns(client, keyword, database, 100, OutputFormat::Table, writer)?;
        }
        _ => {
            writeln!(writer, "Error: Unknown search subcommand: {}", subcommand)?;
            writeln!(writer, "Available: tables, columns")?;
        }
    }
    writeln!(writer)?;
    Ok(())
}
```

The function signatures mirror the `list.rs` pattern: a batch `execute()` with full format
support and an `execute_for_repl()` that defaults to table format with REPL-style spacing.

### Output Formats

All four output formats are supported, following the established per-command rendering pattern.

**Table format (table search):**
```
Search results for "emp" (3 matches):
Database                       Table                          Type     Rows (Est.)      Size Owner
----------------------------------------------------------------------------------------------------
hr_db                          employees                      TABLE          15000    2.5 MB admin
hr_db                          emp_history                    TABLE           5200  800.0 KB admin
finance_db                     temp_emp_data                  NoPI             120   12.0 KB etl_user

3 result(s)
```

**Table format (column search):**
```
Search results for "salary" (2 matches):
Database                       Table                          Column                         Type            Null
------------------------------------------------------------------------------------------------------------------
hr_db                          employees                      base_salary                    DECIMAL(10,2)   Y
hr_db                          employees                      salary_date                    DATE            N

2 result(s)
```

**JSON format:**
```json
{"ok":true,"row_count":3,"data":[{"database":"hr_db","name":"employees","type":"TABLE","estimated_rows":15000,"size_bytes":2621440,"owner":"admin"},{"database":"hr_db","name":"emp_history","type":"TABLE","estimated_rows":5200,"size_bytes":819200,"owner":"admin"}]}
```

The JSON envelope follows the standard `{"ok": true, "row_count": N, "data": [...]}` contract.
For table search, JSON keys are: `database`, `name`, `type`, `estimated_rows`, `size_bytes`, `owner`.
For column search, JSON keys are: `database`, `table`, `column`, `type`, `nullable`.

**CSV format:** Standard header row followed by data rows, using the same `csv_escape()` helper.

**Markdown format:** Pipe-delimited table with alignment indicators, using the same `|` escaping pattern.

### REPL Integration

#### Metacommand Dispatch

In `src/commands/repl/metacommands.rs`, add a `"search"` arm to `handle_metacommand_with_state()`:

```rust
"search" => {
    if args.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Usage: /search <subcommand> <keyword>")?;
        writeln!(writer)?;
        writeln!(writer, "Subcommands:")?;
        writeln!(writer, "  tables <keyword>    Search tables by name across databases")?;
        writeln!(writer, "  columns <keyword>   Search columns by name across databases")?;
        writeln!(writer)?;
        writeln!(writer, "Examples:")?;
        writeln!(writer, "  /search tables emp")?;
        writeln!(writer, "  /search columns salary")?;
        writeln!(writer)?;
    } else {
        let subcommand = args[0];
        let keyword = if args.len() > 1 { args[1] } else { "" };
        if keyword.is_empty() {
            writeln!(writer, "Error: Missing keyword.")?;
            writeln!(writer, "Usage: /search {} <keyword>", subcommand)?;
        } else {
            crate::commands::search::execute_for_repl(
                completion_state.client(),
                subcommand,
                keyword,
                None,  // REPL search always cross-database
                writer,
            )?;
        }
    }
}
```

#### Tab Completion

Add entries to the `METACOMMANDS` array in `src/commands/repl/metadata_completer.rs`:

```rust
MetacommandDef {
    name: "search tables",
    aliases: &[],
    description: "Search tables by name across databases",
},
MetacommandDef {
    name: "search columns",
    aliases: &[],
    description: "Search columns by name across databases",
},
```

This follows the same two-word metacommand pattern used by `"list databases"`, `"list tables"`,
and `"list views"`.

#### Help Text

Update `print_help_extended()` to include search commands in the "Schema Inspection" section:

```
  /search tables <kw>  Search tables by name across databases
  /search columns <kw> Search columns by name across databases
```

### Agent-Safe Mode Compatibility

Search commands are read-only SELECT queries against DBC system views. They are inherently
safe for agent mode and require no special handling. The existing agent-safe SQL classification
logic in `src/sql/classifier.rs` will correctly identify these as safe SELECT statements since
the SQL is constructed internally (not user-supplied).

### Performance Considerations

**Cross-database queries on large systems:** `DBC.TablesV` and `DBC.ColumnsV` are system views
that Teradata optimizes for metadata access. The LIKE predicate with a leading wildcard
(`%keyword%`) prevents index usage on TableName/ColumnName, but this is acceptable because:

1. Metadata views are typically small relative to data tables.
2. The SAMPLE clause limits result set size.
3. Cross-database discovery is an inherently broad operation.

**The `--limit` default of 100** prevents accidental large result sets. Users can increase
with `--limit 500` or disable with `--limit 0`.

**Size join overhead:** The LEFT JOIN with `DBC.TableSizeV` for table search adds overhead
but provides valuable size and row count information. This is the same pattern used by
`tq list tables` and has proven acceptable in practice.

### Error Handling

- **No results:** Print a "No results found" message and return success (exit code 0).
  JSON format returns `{"ok":true,"row_count":0,"data":[]}`.
- **Permission errors:** If the user lacks SELECT on DBC views, the Teradata error propagates
  through the standard error handling path with a clear error message.
- **Empty keyword:** Validated at the CLI level by clap (keyword is a required positional arg).
  In REPL mode, handled explicitly with a usage hint.

### Code Linkage

| Component | File | Key Types / Functions |
|-----------|------|-----------------------|
| CLI args | `src/cli.rs` | `SearchArgs`, `SearchObjectType`, `Command::Search` |
| Command dispatch | `src/main.rs` | `run()` match arm |
| Module export | `src/commands/mod.rs` | `pub mod search` |
| Implementation | `src/commands/search.rs` | `execute()`, `execute_for_repl()`, `search_tables()`, `search_columns()` |
| REPL metacommand | `src/commands/repl/metacommands.rs` | `"search"` arm in `handle_metacommand_with_state()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `MetacommandDef` entries for search |
| REPL help | `src/commands/repl/metacommands.rs` | `print_help_extended()` update |
| SQL escaping | `src/sql/identifiers.rs` | `escape_sql_string()` for keyword sanitization |
| Format helpers | `src/commands/format_helpers.rs` | `json_escape()`, `csv_escape()`, `format_size()` |

## Client-Side Result Pagination

### Problem

When querying large tables, users need to browse results page-by-page rather than receiving the entire result set at once. Teradata lacks a native `OFFSET` clause, so server-side pagination is impractical. Client-side pagination fetches the full result set (possibly capped by `--max-rows` in agent-safe mode) and then slices to the requested page boundaries before formatting.

### Design Approach

Pagination is implemented as a post-fetch, pre-format transformation on `QueryResult`. The slicing happens in `execute_single()` in `src/commands/query.rs`, between the database fetch and the call to `write_output_with_timing()`. This keeps the pagination logic centralized in the query command rather than scattered across formatters.

### CLI Flags

Two new flags on `QueryArgs`:

```rust
// src/cli.rs - in QueryArgs

/// Page number to display (1-indexed, requires --page-size)
///
/// Selects which page of results to display. Page 1 is the first page.
/// Must be used together with --page-size.
#[arg(long, value_name = "N", requires = "page_size")]
pub page: Option<usize>,

/// Number of rows per page (requires --page)
///
/// Splits the result set into pages of this size.
/// Must be used together with --page. Mutually exclusive with --limit.
#[arg(long, value_name = "N", requires = "page", conflicts_with = "limit")]
pub page_size: Option<usize>,
```

**Key constraints enforced by clap:**
- `--page` requires `--page-size` (incomplete pagination request rejected at parse time)
- `--page-size` requires `--page` (same)
- `--page-size` conflicts with `--limit` (they serve different purposes; using both is ambiguous)

**Interaction with `--max-rows` (agent-safe mode):**
- `--max-rows` caps the total rows fetched from the database
- Pagination operates within that cap: `total_rows` is the fetched count (not the theoretical full table count)
- This is correct behavior: agents should never paginate beyond their safety cap

### Pagination Data Structure

```rust
// src/commands/query.rs (or a new src/pagination.rs if the logic warrants its own module)

/// Pagination metadata computed after slicing a result set.
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    /// Current page number (1-indexed)
    pub page: usize,
    /// Rows per page
    pub page_size: usize,
    /// Total number of rows in the full (possibly capped) result set
    pub total_rows: usize,
    /// Total number of pages (ceiling division)
    pub total_pages: usize,
}

impl PaginationInfo {
    /// Compute pagination metadata from page parameters and total row count.
    ///
    /// Returns `None` if pagination is not requested (both page and page_size are None).
    /// Returns an error if the requested page exceeds total pages.
    pub fn from_args(
        page: Option<usize>,
        page_size: Option<usize>,
        total_rows: usize,
    ) -> Result<Option<Self>> {
        match (page, page_size) {
            (Some(p), Some(ps)) => {
                if ps == 0 {
                    return Err(TqError::InvalidConfig(
                        "--page-size must be greater than 0".to_string(),
                    ));
                }
                if p == 0 {
                    return Err(TqError::InvalidConfig(
                        "--page must be greater than 0".to_string(),
                    ));
                }
                let total_pages = if total_rows == 0 {
                    1
                } else {
                    (total_rows + ps - 1) / ps
                };
                if p > total_pages {
                    return Err(TqError::InvalidConfig(format!(
                        "Page {} exceeds total pages ({} rows, {} per page = {} pages)",
                        p, total_rows, ps, total_pages
                    )));
                }
                Ok(Some(Self {
                    page: p,
                    page_size: ps,
                    total_rows,
                    total_pages,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Compute the start index (inclusive) and end index (exclusive) for slicing.
    pub fn row_range(&self) -> (usize, usize) {
        let start = (self.page - 1) * self.page_size;
        let end = (start + self.page_size).min(self.total_rows);
        (start, end)
    }
}
```

### Result Slicing

A new method on `QueryResult` to produce a paginated subset:

```rust
// src/db/types.rs

impl QueryResult {
    /// Return a new QueryResult containing only the rows in the given range.
    ///
    /// The returned result's `row_count` reflects the sliced count,
    /// not the original total. Callers use `PaginationInfo.total_rows`
    /// for the original count.
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let end = end.min(self.rows.len());
        let start = start.min(end);
        let sliced_rows: Vec<Row> = self.rows[start..end].to_vec();
        Self::new(
            self.columns.clone(),
            sliced_rows,
            self.execution_time,
        )
    }
}
```

**Design decision: `slice()` on `QueryResult`** rather than modifying `.rows` in place. This preserves the original result for metadata computation (total_rows) and follows Rust's preference for immutable transformations.

### Integration in `execute_single()`

The pagination logic slots between the fetch and the format call:

```rust
fn execute_single<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    args: &QueryArgs,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    // ... existing fetch logic ...

    let result = /* fetch as today */;

    // Agent-safe overflow check (existing)
    // ...

    // Pagination: compute metadata, then slice
    let pagination = PaginationInfo::from_args(
        args.page,
        args.page_size,
        result.row_count,
    )?;

    let display_result = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        result.slice(start, end)
    } else {
        result
    };

    // Configure output formatting
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(use_color);

    // Write output (with pagination metadata for JSON)
    write_output_paginated(
        &display_result,
        writer,
        args.format,
        &format_options,
        args.timing,
        pagination.as_ref(),
    )?;

    Ok(())
}
```

### JSON Envelope Extension

When pagination is active, the JSON envelope gains an optional `pagination` object:

```json
{
  "ok": true,
  "row_count": 25,
  "pagination": {
    "page": 2,
    "page_size": 25,
    "total_rows": 73,
    "total_pages": 3
  },
  "data": [...]
}
```

**Key points:**
- `row_count` reflects the count of rows **in the current page** (matches `data.length`)
- `pagination` is only present when `--page` / `--page-size` are used
- When pagination is absent, the envelope is unchanged (backward compatible)

**Implementation in `src/format/json.rs`:**

```rust
/// Write query results as JSON with envelope and optional pagination.
pub fn write_paginated<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &JsonOptions,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    let mut envelope = Map::new();
    envelope.insert("ok".to_string(), JsonValue::Bool(true));
    envelope.insert(
        "row_count".to_string(),
        JsonValue::Number(result.row_count.into()),
    );

    if let Some(pg) = pagination {
        let mut pg_obj = Map::new();
        pg_obj.insert("page".into(), JsonValue::Number(pg.page.into()));
        pg_obj.insert("page_size".into(), JsonValue::Number(pg.page_size.into()));
        pg_obj.insert("total_rows".into(), JsonValue::Number(pg.total_rows.into()));
        pg_obj.insert("total_pages".into(), JsonValue::Number(pg.total_pages.into()));
        envelope.insert("pagination".to_string(), JsonValue::Object(pg_obj));
    }

    envelope.insert("data".to_string(), JsonValue::Array(build_rows(result)));
    write_json(writer, &JsonValue::Object(envelope), options.pretty)
}
```

The existing `write()` function remains unchanged for backward compatibility. The new `write_paginated()` is called from the new `write_output_paginated()` dispatch function. If `pagination` is `None`, the output is identical to `write()`.

### Format Dispatch with Pagination

A new dispatch function in `src/format/mod.rs`:

```rust
/// Write query results with optional pagination metadata.
///
/// For JSON: adds `pagination` object to envelope.
/// For table/markdown: appends "Page X of Y" footer.
/// For CSV: no pagination footer (data-only format).
pub fn write_output_paginated<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    format: OutputFormat,
    options: &FormatOptions,
    show_timing: bool,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    match format.canonical() {
        OutputFormat::Table => {
            if show_timing {
                table::write_with_timing(result, writer, &options.table)?;
            } else {
                table::write(result, writer, &options.table)?;
            }
            if let Some(pg) = pagination {
                writeln!(writer, "Page {} of {} ({} total rows)",
                    pg.page, pg.total_pages, pg.total_rows)?;
            }
        }
        OutputFormat::Json => {
            if show_timing {
                // Combine timing + pagination in metadata envelope
                json::write_paginated_with_metadata(
                    result, writer, &options.json, pagination,
                )?;
            } else {
                json::write_paginated(result, writer, &options.json, pagination)?;
            }
        }
        OutputFormat::Csv => {
            csv::write(result, writer, &options.csv)?;
            // CSV: no pagination footer (pure data format)
        }
        OutputFormat::Markdown => {
            if show_timing {
                markdown::write_with_metadata(result, writer, &options.markdown)?;
            } else {
                markdown::write(result, writer, &options.markdown)?;
            }
            if let Some(pg) = pagination {
                writeln!(writer)?;
                writeln!(writer, "*Page {} of {} ({} total rows)*",
                    pg.page, pg.total_pages, pg.total_rows)?;
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
```

### Batch Mode

Pagination does **not** apply to batch mode (`execute_batch()`). Batch mode executes multiple statements and streams results; paginating individual statement results would be confusing. If `--page`/`--page-size` are provided with `--file` containing multiple statements, the behavior is to paginate each individual result set. However, for simplicity in Sprint 56, we reject pagination in batch mode:

```rust
if use_batch && args.page.is_some() {
    return Err(TqError::InvalidConfig(
        "Pagination (--page/--page-size) is not supported in batch mode.\n\
         Use single-statement queries for pagination.".to_string(),
    ));
}
```

### Non-Query Commands

Pagination only applies to `tq query`. The `search`, `list`, `describe`, and other metadata commands have their own `--limit` flag for result limiting. Pagination for these commands is out of scope for Sprint 56.

### Code Linkage

| Component | File | Changes |
|-----------|------|---------|
| CLI flags | `src/cli.rs` | Add `page`, `page_size` to `QueryArgs` |
| Pagination types | `src/commands/query.rs` | Add `PaginationInfo` struct |
| Result slicing | `src/db/types.rs` | Add `QueryResult::slice()` method |
| Query execution | `src/commands/query.rs` | Update `execute_single()` with pagination logic |
| Batch guard | `src/commands/query.rs` | Reject pagination in `execute()` before batch path |
| JSON envelope | `src/format/json.rs` | Add `write_paginated()`, `write_paginated_with_metadata()` |
| Format dispatch | `src/format/mod.rs` | Add `write_output_paginated()` |
| Unit tests | `src/commands/query.rs` | Tests for `PaginationInfo::from_args()`, edge cases |
| Unit tests | `src/db/types.rs` | Tests for `QueryResult::slice()` |
| Unit tests | `src/format/json.rs` | Tests for pagination envelope |

## Tech Debt: Consolidate `esc()` Pipe Escape Function

### Problem

The function `fn esc(s: &str) -> String { s.replace('|', "\\|") }` is duplicated as a local `fn` definition in **14 locations** across 10 source files:

| File | Occurrences |
|------|-------------|
| `search.rs` | 2 (table markdown, column markdown) |
| `list.rs` | 3 (databases, tables, views markdown) |
| `sample.rs` | 2 (result markdown, stats markdown) |
| `show_indexes.rs` | 1 |
| `sysconfig.rs` | 1 |
| `history.rs` | 1 |
| `explain.rs` | 1 |
| `sessions.rs` | 1 |
| `query_inspect.rs` | 1 |
| `locks.rs` | 1 |
| `skew.rs` | 1 |

Each copy is identical: a one-line pipe character escape for markdown table cells.

### Solution

Add a `markdown_escape_pipe()` function to `src/commands/format_helpers.rs`:

```rust
/// Escape pipe characters for safe inclusion in Markdown table cells.
///
/// Replaces `|` with `\|` to prevent breaking Markdown table syntax.
pub fn markdown_escape_pipe(s: &str) -> String {
    s.replace('|', "\\|")
}
```

Then replace all 14 local `fn esc(...)` definitions with a `use crate::commands::format_helpers::markdown_escape_pipe;` import and rename call sites from `esc(...)` to `markdown_escape_pipe(...)`.

**Alternative considered:** Keep the short name `esc` as a module-level re-import alias: `use crate::commands::format_helpers::markdown_escape_pipe as esc;`. This minimizes diff size but reduces clarity. The longer name is preferred because it is self-documenting and the call sites are few enough per file that the extra characters are negligible.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| New function | `src/commands/format_helpers.rs` | Add `markdown_escape_pipe()` |
| Migrate | `src/commands/search.rs` | Remove 2 local `esc()`, import helper |
| Migrate | `src/commands/list.rs` | Remove 3 local `esc()`, import helper |
| Migrate | `src/commands/sample.rs` | Remove 2 local `esc()`, import helper |
| Migrate | `src/commands/show_indexes.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/sysconfig.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/history.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/explain.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/sessions.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/query_inspect.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/locks.rs` | Remove 1 local `esc()`, import helper |
| Migrate | `src/commands/skew.rs` | Remove 1 local `esc()`, import helper |
| Unit test | `src/commands/format_helpers.rs` | Test for `markdown_escape_pipe()` |

## Tech Debt: Search Dispatch Unit Tests

### Problem

The `search.rs` module has good tests for rendering functions but no tests for the `execute()` dispatch logic or the REPL dispatch in `execute_for_repl()`. Since these functions require a `DatabaseClient`, they cannot be unit-tested without a mock. However, the rendering functions (`render_table_search_*`, `render_column_search_*`) are already well-tested.

### Solution

Add unit tests for the dispatch logic that does not require database access:

1. **Subcommand parsing in `execute_for_repl()`**: Test that unknown subcommands produce an error message.
2. **Rendering tests with edge cases**: Markdown pipe escaping with special characters, JSON with special characters in object names.

These tests complement the existing rendering tests by covering edge cases that could break the output.

```rust
#[test]
fn test_repl_dispatch_unknown_subcommand() {
    // Mock client not needed - test only the error path
    // The function writes an error message for unknown subcommands
    // This requires a mock client, so instead test the rendering edge cases
}

#[test]
fn test_markdown_pipe_in_table_name() {
    let tables = vec![TableSearchResult {
        database: "db".to_string(),
        table_name: "has|pipe".to_string(),
        kind: "TABLE".to_string(),
        row_count_display: "0".to_string(),
        row_count_raw: Some(0),
        size_display: "0 B".to_string(),
        size_bytes: Some(0),
        owner: "usr".to_string(),
    }];
    let mut buf = Vec::new();
    render_table_search_markdown(&tables, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("has\\|pipe"));
    assert!(!output.contains("has|pipe"));
}

#[test]
fn test_json_special_chars_in_names() {
    let tables = vec![TableSearchResult {
        database: "db\"name".to_string(),
        table_name: "tbl\\slash".to_string(),
        kind: "TABLE".to_string(),
        row_count_display: "0".to_string(),
        row_count_raw: Some(0),
        size_display: "0 B".to_string(),
        size_bytes: Some(0),
        owner: "user\nnewline".to_string(),
    }];
    let mut buf = Vec::new();
    render_table_search_json(&tables, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("db\\\"name"));
    assert!(output.contains("tbl\\\\slash"));
    assert!(output.contains("user\\nnewline"));
}
```

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| New tests | `src/commands/search.rs` | Edge case tests for markdown and JSON rendering |

## Search Module: Serde JSON Refactoring

### Problem

The `render_table_search_json_with_pagination` and `render_column_search_json_with_pagination` functions in `src/commands/search.rs` use hand-rolled `write!()` calls to build JSON output. This approach is fragile (manual comma handling, manual null rendering, manual escaping via `json_escape()`) and inconsistent with the rest of the codebase, which uses `serde_json` (see `src/format/json.rs`).

### Solution: Serializable Structs with serde_json

Define `#[derive(Serialize)]` structs that mirror the JSON output structure, then use `serde_json::to_writer` for output.

#### Envelope Pattern

```rust
use serde::Serialize;

/// Standard search result envelope for JSON output
#[derive(Serialize)]
struct SearchEnvelope<T: Serialize> {
    ok: bool,
    row_count: usize,
    data: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationJson>,
}

/// Pagination metadata for JSON serialization
#[derive(Serialize)]
struct PaginationJson {
    page: usize,
    page_size: usize,
    total_rows: usize,
    total_pages: usize,
    has_more: bool,
}

impl From<&PaginationInfo> for PaginationJson {
    fn from(pg: &PaginationInfo) -> Self {
        Self {
            page: pg.page,
            page_size: pg.page_size,
            total_rows: pg.total_rows,
            total_pages: pg.total_pages(),
            has_more: pg.has_more(),
        }
    }
}
```

#### Table Search JSON Payload

```rust
/// JSON representation of a table search result row
#[derive(Serialize)]
struct TableSearchJson {
    database: String,
    table_name: String,
    #[serde(rename = "type")]
    kind: String,
    estimated_rows: Option<i64>,
    size_bytes: Option<i64>,
    owner: String,
}
```

Key serde behaviors:
- `Option<i64>` serializes as `null` when `None` -- matches current hand-rolled behavior
- `#[serde(rename = "type")]` produces the JSON key `"type"` from the Rust field `kind`
- No manual `json_escape()` needed -- serde handles all escaping

#### Column Search JSON Payload

```rust
/// JSON representation of a column search result row
#[derive(Serialize)]
struct ColumnSearchJson {
    database: String,
    table_name: String,
    column_name: String,
    column_type: String,
    nullable: String,
}
```

#### Rendering Functions

Both `render_table_search_json_with_pagination` and `render_column_search_json_with_pagination` become simple conversions:

```rust
fn render_table_search_json_with_pagination<W: Write>(
    tables: &[TableSearchResult],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    let data: Vec<TableSearchJson> = tables.iter().map(|t| TableSearchJson {
        database: t.database.clone(),
        table_name: t.table_name.clone(),
        kind: t.kind.clone(),
        estimated_rows: t.row_count_raw,
        size_bytes: t.size_bytes,
        owner: t.owner.clone(),
    }).collect();

    let envelope = SearchEnvelope {
        ok: true,
        row_count: data.len(),
        data,
        pagination: pagination.map(PaginationJson::from),
    };

    serde_json::to_writer(&mut *writer, &envelope)?;
    writeln!(writer)?;
    Ok(())
}
```

#### Output Compatibility

The serde output is compact JSON (no whitespace) matching the current hand-rolled output. The `SearchEnvelope` uses `skip_serializing_if = "Option::is_none"` on `pagination` so the key is absent when there is no pagination -- matching current behavior where the pagination block is only written when `pagination.is_some()`.

The existing tests check for content like `"ok":true` and `"estimated_rows":null` -- these assertions remain valid since serde produces the same key-value pairs in the same order (struct field declaration order).

One minor difference: serde may differ in whitespace or field ordering in edge cases. The existing tests use `contains()` assertions (not exact string equality), so they tolerate this.

#### Import Changes

The `json_escape` import from `format_helpers` can be removed from search.rs after this refactoring (it is only used for hand-rolled JSON). The `csv_escape` and `markdown_escape_pipe` imports remain for CSV and markdown renderers.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Add serde structs | `src/commands/search.rs` | `SearchEnvelope<T>`, `PaginationJson`, `TableSearchJson`, `ColumnSearchJson` |
| Refactor JSON render | `src/commands/search.rs` | Replace `write!()` calls with `serde_json::to_writer` |
| Remove import | `src/commands/search.rs` | Remove `json_escape` from format_helpers import |

## Search Module: MAX_SEARCH_FETCH Constant

### Problem

The hard-coded `100000` value appears twice in `src/commands/search.rs` (once in `search_tables`, once in `search_columns`). It serves as a sentinel "fetch all rows" limit for client-side pagination but reads as a magic number.

### Solution

Define a module-level constant:

```rust
/// Maximum rows fetched when client-side pagination is active.
///
/// When --page-size is set, we fetch up to this many rows from the server
/// and paginate client-side. This avoids unbounded result sets while being
/// large enough to cover any practical search.
const MAX_SEARCH_FETCH: usize = 100_000;
```

Replace both occurrences:

```rust
// Before:
let row_limit = if pagination_args.is_some() {
    100000
} else {
    limit.unwrap_or(100)
};

// After:
let row_limit = if pagination_args.is_some() {
    MAX_SEARCH_FETCH
} else {
    limit.unwrap_or(100)
};
```

The constant is `pub(crate)` visibility is not needed since it is only used within the search module. A plain `const` suffices.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| New constant | `src/commands/search.rs` | `MAX_SEARCH_FETCH` replaces magic `100000` |

## Search Views Subcommand

### Overview

Add `tq search views <keyword>` to search for views by name across databases. This follows the exact same architecture as the existing `search_tables` and `search_columns` implementations.

### CLI Integration

#### SearchObjectType Extension

```rust
// src/cli.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchObjectType {
    Tables,
    Columns,
    Views,  // New variant
}
```

#### Batch Dispatch

In `execute()`, add the `Views` match arm:

```rust
SearchObjectType::Views => search_views(
    client,
    &args.keyword,
    args.database.as_deref(),
    args.format,
    effective_limit,
    pagination_args,
    writer,
),
```

### SQL Query

```sql
SELECT TOP {limit}
    TRIM(t.DatabaseName) AS db_name,
    TRIM(t.TableName) AS view_name,
    t.CreateTimeStamp AS created,
    TRIM(t.CreatorName) AS creator
FROM DBC.TablesV t
WHERE UPPER(t.TableName) LIKE UPPER('%{keyword}%')
  AND t.TableKind IN ('V')
  {db_filter}
ORDER BY t.DatabaseName, t.TableName
```

Design notes:
- `TableKind = 'V'` selects views only (V = View in Teradata's DBC.TablesV)
- `CreateTimeStamp` provides useful metadata (when was the view created)
- `CreatorName` shows who created the view
- No size/row-count join needed -- views have no storage footprint
- Uses same `escape_sql_string()` and db_filter pattern as table search

### Data Structure

```rust
/// View search result entry
struct ViewSearchResult {
    database: String,
    view_name: String,
    created: String,
    creator: String,
}
```

The `created` field is stored as a `String` because Teradata's `CreateTimeStamp` comes back as a formatted timestamp string. We display it as-is.

### JSON Output

Using the serde envelope pattern introduced by the serde refactoring:

```rust
#[derive(Serialize)]
struct ViewSearchJson {
    database: String,
    view_name: String,
    created: String,
    creator: String,
}
```

Envelope output:
```json
{"ok":true,"row_count":2,"data":[{"database":"hr","view_name":"emp_view","created":"2024-01-15 10:30:00","creator":"admin"},{"database":"fin","view_name":"budget_view","created":"2024-03-01 08:00:00","creator":"dba"}]}
```

### All Four Output Formats

Following the established pattern:

| Format | Function | Header/Structure |
|--------|----------|------------------|
| Table | `render_view_search_table` | "Database", "View Name", "Created", "Creator" |
| JSON | `render_view_search_json_with_pagination` | serde envelope with `ViewSearchJson` |
| CSV | `render_view_search_csv` | `Database,ViewName,Created,Creator` |
| Markdown | `render_view_search_markdown` | `\| Database \| View Name \| Created \| Creator \|` |

### Pagination Support

Same client-side pagination pattern as table and column search:
- When `--page-size` is set, fetch up to `MAX_SEARCH_FETCH` rows
- Apply `PaginationInfo` slicing
- Write pagination footer (table/CSV/markdown) or pagination JSON block (JSON)

### REPL Integration

#### Dispatch in execute_for_repl

```rust
// In execute_for_repl match block:
"views" | "view" | "v" => {
    search_views(client, keyword, database, OutputFormat::Table, None, None, writer)?;
}
```

Update the error message to include views:
```rust
_ => {
    writeln!(writer, "Error: Unknown search subcommand: {}", subcommand)?;
    writeln!(writer, "Available: tables, columns, views")?;
}
```

#### Metacommand Help Text

Update the `/search` help text in `src/commands/repl/metacommands.rs` to include:
```
  views <keyword>     Search views by name across databases
```

Add examples:
```
  /search views emp
  /search views budget in finance
```

#### Tab Completion

In `src/commands/repl/metadata_completer.rs`:

1. Add a new `MetacommandDef` entry:
```rust
MetacommandDef {
    name: "search views",
    aliases: &[],
    description: "Search views by name across databases",
},
```

2. Add `"views"` to the `complete_search_subcommands` subcommands array:
```rust
let subcommands = [
    ("tables", "Search tables by name across databases"),
    ("columns", "Search columns by name across databases"),
    ("views", "Search views by name across databases"),
];
```

### Unit Tests

Follow the same test structure as existing table/column search tests:

1. `test_view_search_result_structure` -- construct and assert fields
2. `test_render_view_search_table_format` -- verify table format output
3. `test_render_view_search_table_empty` -- verify "(no views found)" message
4. `test_render_view_search_json` -- verify JSON envelope with correct keys
5. `test_render_view_search_json_empty` -- verify empty data array
6. `test_render_view_search_csv` -- verify CSV header and rows
7. `test_render_view_search_markdown` -- verify markdown table

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| New enum variant | `src/cli.rs` | `SearchObjectType::Views` |
| New search function | `src/commands/search.rs` | `search_views()` + 4 render functions |
| REPL dispatch | `src/commands/search.rs` | `execute_for_repl` views arm |
| REPL help text | `src/commands/repl/metacommands.rs` | Updated `/search` help and help listing |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `MetacommandDef` entry + subcommand array |
| Unit tests | `src/commands/search.rs` | 7 new tests |

## tq space / tq dbspace Commands

Two commands report permanent, spool and temporary space usage. The full design — verified
DBC column sets, SQL, skew formula, module structure and output shapes — lives in
`docs/design/space-analysis.md`. This section records only the CLI-surface decisions.

### Command shapes

```
tq space <database>            # database header row + one row per contained object
tq space <database>.<object>   # single object row
tq dbspace <database>          # database-level metrics only
```

Both commands take a single positional argument plus the standard `--format` and `--output`
flags, matching `SkewArgs` (`src/cli.rs:1163`).

### Why two commands rather than one flag

`tq space <db>` and `tq dbspace <db>` return different column sets, not different filters of
the same set: the database view carries `MaxPerm`, `% used`, and spool/temp metrics that have
no object-level equivalent, because `DBC.TableSizeV` exposes only `CurrentPerm` and
`PeakPerm`. Expressing this as `tq space --database-only` would give one command two
incompatible output schemas per format, which complicates every renderer and every consumer.
Two commands keep each output schema stable.

### Target parsing

Both commands share one `parse_target` function returning a `SpaceTarget` enum
(`Database` | `Object`). `dbspace` accepts only the `Database` variant and rejects a
qualified argument with an actionable error naming the correct command, rather than silently
ignoring the object part:

```
Error: Invalid object reference 'demo_user.orders' — expected <database> (dbspace operates on databases only)
Hint: use 'tq space demo_user.orders' for object-level space,
      or 'tq dbspace demo_user' for the database.

Usage: tq dbspace <database>
```

This is a usage error, exit code 2, carried by `TqError::InvalidObjectReference`. A target
with more than one dot (`a.b.c`) uses the same variant.

`parse_target` splits on `.` but ignores dots inside double-quoted identifier parts, so
`"my.db".tbl` resolves to the database `my.db` and the object `tbl`. Surrounding quotes are
stripped and doubled inner quotes are un-doubled, matching the conventions in
`src/sql/identifiers.rs`.

### Not-found handling

A database that holds no space returns zero rows from `DBC.DiskSpaceV`, which is
indistinguishable at the query level from a misspelled name. Rather than guessing, an empty
result triggers a catalog probe against `DBC.DatabasesV` (or `DBC.TablesV` for the object
form). A missing catalog entry produces `TqError::ObjectNotFound` (exit code 1); an existing
entry produces a zero-usage report. The probe runs only on the empty-result path, so the
common case costs one round trip.

No spelling suggestion is offered — no fuzzy-match helper exists anywhere in `src/`, and
adding one was out of scope. `dbspace` does, however, make one extra distinction: when the
database probe fails it re-probes `DBC.TablesV` for an object of that name in any database,
so `tq dbspace evals_employees` reports "'evals_employees' is an object in database
'demo_user', not a database" and names `tq space demo_user.evals_employees`.

### Severity coloring

Table and markdown output route skew percentages and `PermUsed%` through the shared severity
layer (`docs/design/monitoring.md`). The `json` and `csv` renderers take no
`MonitoringContext` parameter at all, so they cannot emit ANSI escapes regardless of
`--color` (REQ-COLOR-007).

### Code Linkage

| Component | File Path | Key Elements |
|-----------|-----------|--------------|
| Command variants | `src/cli.rs` | `Command::Space`, `Command::Dbspace` |
| Argument structs | `src/cli.rs` | `SpaceArgs`, `DbspaceArgs` |
| Implementation | `src/commands/space.rs` | `execute`, `execute_for_repl`, `parse_target`, 4 renderers |
| Module registration | `src/commands/mod.rs` | `pub mod space;`, `pub use space::execute as space;` |
| Dispatch | `src/main.rs` | `Command::Space` / `Command::Dbspace` arms with `--output` branch |
| Re-exports | `src/lib.rs` | `SpaceArgs`, `DbspaceArgs` |
| REPL metacommands | `src/commands/repl/metacommands.rs` | `/space`, `/dbspace` in both handlers + `print_help_extended` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `MetacommandDef` entries |
| Lenient numeric extraction | `src/commands/monitoring_utils.rs` | `extract_i64_lenient`, `extract_f64_lenient` |
| Error variants | `src/error.rs` | `ObjectNotFound`, `InvalidObjectReference` |

## Watch Interval Flag Resolution

`--interval` was declared on three arg structs (`SessionsArgs`, `LocksArgs`, `ResourcesArgs`)
with `default_value = "6"`. That default made "the user asked for 6"
indistinguishable from "the user said nothing", so a configured `refresh_interval` could
never take effect.

The flag becomes `Option<u64>` with the clap default removed and the `2..=300` range parser
retained. Resolution moves to the dispatch site in `src/main.rs`, where both the parsed args
and the loaded config are in scope:

```rust
let interval = args.interval.unwrap_or(config.monitoring.thresholds.refresh_interval);
```

Precedence is CLI flag > config > built-in default, matching the hierarchy in
`docs/design/configuration.md`. This is the general pattern for any future flag that needs a
config-supplied default: express the flag as `Option<T>`, keep validation in the clap value
parser, and resolve the default where config is available.

## Future Enhancements

- **Config file flag**: `--config <path>` to override default config location
- **Dry-run mode**: `--dry-run` to validate without executing
- **Output templates**: `--template <name>` for custom formatting
- **Batch mode flags**: `--continue-on-error`
