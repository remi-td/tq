# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tq** (Teradata Query) is a lightweight Rust command line client for Teradata databases. It follows a simple one-shot execution model: one tool call -> one connection -> close session when done.

## Claude Skills for this project
Use the following skills when working with code in this repository:
- teradata-rust: Guides writing idiomatic Rust code for Teradata database interactions using the teradatarustapi 
- rust-coder: for writing idiomatic Rust code
- rust-debugger: for debugging Rust code

## Installation & Usage

The tool is distributed via Cargo (Rust's package manager):

```bash
# From crates.io (once published)
cargo install tq

# From source
cargo install --path .
```

Command line interface:
```bash
tq [OPTIONS] <QUERY>
```

Required argument:
- `<QUERY>` - The SQL query to execute

Options:
- `-l, --logon` - Database logon string format: "user:password@databaseserver:port/schema"
- `--logmech` - Logon mechanism (default: "TD2"). Supported: TD2, LDAP, KRB5, TDNEGO
- `--format` - Output format: table (default), json, csv
- `--password-file` - Read password from file instead of connection string
- `-h, --help` - Print help
- `-V, --version` - Print version

## Architecture Principles

**Library-First Design**: Business logic lives in `src/lib.rs`, with `src/main.rs` serving as a thin CLI wrapper. This enables:
- Unit testing of core logic independent of CLI parsing
- Potential reuse by other consumers (GUI, web interface)
- Clean separation of concerns

**Project Structure**:
```
src/
  lib.rs          # Public library API
  main.rs         # CLI entry point
  cli/            # CLI argument definitions (clap)
  db/             # Database operations and connection handling
  commands/       # Command implementations (query, ping, etc.)
  utils/          # Shared utilities (formatting, error handling)
```

**Module Organization**:
- Each module exposes a clean public API
- Implementation details remain private
- Use traits to define behavior contracts (e.g., `DatabaseClient` trait)
- Traits enable testing with mock implementations

## Key Dependencies

**Core Stack**:
- `clap` v4 - CLI argument parsing with derive macros
- `anyhow` - Application-level error handling (easy context propagation)
- `thiserror` - Library-level error types (structured matching)
- `teradatarustapi` - Teradata database connectivity

**Optional Enhancements**:
- `figment` - Layered configuration (CLI → env → config file → defaults)
- `directories` - Platform-appropriate config file locations
- `secrecy` - Secure credential handling (zero memory on drop, redacted debug)
- `keyring` - System keyring integration
- `tabled` or `comfy-table` - Table formatting
- `owo-colors` - Zero-allocation terminal colors
- `exitcode` - Standard exit code constants

## Development Commands

**Build:**
```bash
cargo build
cargo build --release
```

**Run:**
```bash
cargo run -- [OPTIONS] <QUERY>

# Examples:
cargo run -- -l "user:pass@host:1025/db" "SELECT * FROM table"
cargo run -- --format json "SELECT 1 AS col"
```

**Test:**
```bash
cargo test
cargo test <test_name>  # Run specific test
cargo test -- --nocapture  # Show output
```

**Lint & Format:**
```bash
cargo clippy -- -D warnings  # Lint with warnings as errors
cargo fmt  # Format code
cargo fmt --check  # Check formatting without modifying
```

**Audit:**
```bash
cargo audit  # Check for security vulnerabilities
cargo tree   # Review dependency tree
```

## Configuration Strategy

**Precedence Hierarchy** (highest to lowest):
1. CLI arguments
2. Environment variables (prefixed with `TQ_`)
3. Project config file (`.tq.toml` in current directory)
4. User config file (`~/.config/tq/config.toml`)
5. System config (`/etc/tq/config.toml`)
6. Built-in defaults

**Environment Variables**:
- `TQ_HOST` - Database host
- `TQ_PORT` - Database port (default: 1025)
- `TQ_USER` - Database username
- `TQ_PASSWORD` - Database password (use with caution)
- `TQ_DATABASE` - Default database/schema
- `TQ_LOGMECH` - Logon mechanism (default: TD2)
- `TQ_FORMAT` - Default output format

**Connection String Formats**:
```bash
# Standard format (current)
user:password@host:port/database

# URL-style DSN (future consideration)
td://user:password@host:port/database
teradata://user:password@host:port/database?logmech=TD2
```

## Security Best Practices

**Credentials**:
- NEVER accept passwords as CLI flags (they leak to `ps` output)
- Support `--password-file` for file-based passwords
- Support reading from stdin when appropriate
- Consider keyring integration via `keyring` crate
- Use `secrecy::Secret<String>` to zero memory on drop
- Never log sensitive values (redact in Debug output)

**SQL Injection Prevention**:
- ALWAYS use parameterized queries with bind parameters
- NEVER use string interpolation for user input
- Validate and sanitize all input at boundaries

**File Permissions**:
- Credential files should be `chmod 0600`
- Validate paths against traversal attacks
- Store config in platform-appropriate secure locations

## Error Handling

**Strategy**:
- Use `thiserror` for library error types (structured, matchable)
- Use `anyhow` for application error handling (context propagation)
- Distinguish user errors (bad input) from system errors (network failures)

**Exit Codes**:
- **0** - Success
- **1** - General errors (query failed, connection error)
- **2** - Usage errors (invalid arguments, bad syntax)

**Example Pattern**:
```rust
// Library layer (src/db/mod.rs)
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection failed to {host}: {source}")]
    Connection { host: String, #[source] source: IoError },

    #[error("Query error: {0}")]
    Query(String),
}

// Application layer (src/main.rs)
let conn = connect(&config)
    .context("Failed to establish database connection")?;
```

## Output Formatting

**Format Options**:
- `table` - Human-readable table (default for TTY)
- `json` - Machine-parseable JSON
- `csv` - Data exchange format

**Terminal Detection**:
- Detect TTY with `std::io::IsTerminal`
- Show colors and formatting when interactive
- Emit clean parseable output when piped
- Honor `NO_COLOR` environment variable
- Provide `--color always|auto|never` flag

## Testing Strategy

**Unit Tests**:
- Place in source files with `#[cfg(test)] mod tests`
- Test CLI parsing logic independently
- Test core functions with mock implementations
- Make functions testable by accepting `impl std::io::Write`

**Integration Tests**:
- Use `assert_cmd` for end-to-end CLI testing
- Use `assert_fs` for temporary file fixtures
- Use `predicates` for output assertions
- Mock database connections with trait implementations

**Example**:
```rust
#[test]
fn test_query_execution() -> Result<()> {
    let mut cmd = Command::cargo_bin("tq")?;
    cmd.args(["query", "SELECT 1"])
       .env("TQ_CONNECTION", "test://...")
       .assert()
       .success()
       .stdout(predicate::str::contains("1"));
    Ok(())
}
```

## Build & Release Optimization

**Release Profile** (`Cargo.toml`):
```toml
[profile.release]
opt-level = "z"       # Size optimization
lto = "fat"           # Link-time optimization
codegen-units = 1     # Maximum optimization
panic = "abort"       # Remove unwinding code
strip = "symbols"     # Remove debug symbols
```

**Cross-Platform Builds**:
- Linux: Target `x86_64-unknown-linux-musl` for static binaries
- macOS: Standard `x86_64-apple-darwin` and `aarch64-apple-darwin`
- Windows: Standard `x86_64-pc-windows-msvc`
- Use `cross-rs` for cross-compilation

**Distribution**:
- GitHub releases with platform-specific archives
- `cargo install` support from crates.io
- Include shell completions (bash, zsh, fish, PowerShell)
- Include man pages and documentation

## Database API

We use the Teradata Rust API based on the Co drivers: https://github.com/Teradata/teradatarustapi

**Connection Management**:
- Simple one-shot model: connect -> execute -> close
- No connection pooling (not needed for CLI)
- Proper cleanup on Ctrl-C and errors

**Query Execution**:
- Support single queries from command line
- Support script files (`-f script.sql` - future consideration)
- Stream large result sets efficiently
- Handle query cancellation on Ctrl-C

**Supported Features**:
- Multiple logon mechanisms (TD2, LDAP, Kerberos)
- Parameterized queries for safety
- Transaction control (future consideration)
- Multiple output formats

## UNIX Philosophy Adherence

- Accept `-` as stdin/stdout placeholder
- Respect `--` to separate options from positional arguments
- Support standard flags: `-h/--help`, `-V/--version`, `-v/--verbose`, `-q/--quiet`
- Use kebab-case for long options
- Handle signals properly (Ctrl-C for graceful shutdown)
- Exit with appropriate status codes

## License

MIT License
