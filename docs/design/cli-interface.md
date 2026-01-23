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

## Future Enhancements

- **Config file flag**: `--config <path>` to override default config location
- **Profile selection**: `--profile <name>` to use named connection profile
- **Dry-run mode**: `--dry-run` to validate without executing
- **Output templates**: `--template <name>` for custom formatting
- **Batch mode flags**: `--continue-on-error`, `--atomic`
