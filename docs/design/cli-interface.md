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

## Future Enhancements

- **Config file flag**: `--config <path>` to override default config location
- **Dry-run mode**: `--dry-run` to validate without executing
- **Output templates**: `--template <name>` for custom formatting
- **Batch mode flags**: `--continue-on-error`
