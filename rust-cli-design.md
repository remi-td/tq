# Comprehensive Rust CLI Architecture for Database Tools

Building a production-quality database CLI in Rust requires mastering five interconnected domains: **project architecture**, **interface design**, **database-specific patterns**, **code quality practices**, and **production deployment**. This guide synthesizes best practices from established tools like ripgrep, bat, psql, and usql to provide authoritative guidance for building a Teradata database CLI.

## Core architectural principles drive success

The most successful Rust CLIs follow a **library-first architecture** where business logic lives in `src/lib.rs` and the binary in `src/main.rs` serves as a thin wrapper. This separation enables unit testing of core logic independently from CLI parsing, allows the library to be reused by other consumers (GUIs, web interfaces), and creates cleaner separation of concerns. Ripgrep exemplifies this pattern with its workspace of specialized crates—`grep` as a facade, `globset` for pattern matching, `ignore` for gitignore handling, and `termcolor` for terminal output.

**Project structure** should scale with complexity. For a database CLI like a Teradata tool, the recommended organization places CLI definitions in `src/cli/`, database operations in `src/db/`, command implementations in `src/commands/`, and shared utilities in `src/utils/`. Each module should expose a clean public API while hiding implementation details. Use traits to define behavior contracts—a `DatabaseClient` trait allows testing with mock implementations and potential future support for different connection methods.

For **dependency management**, clap v4 with derive macros dominates argument parsing with its full POSIX/GNU convention support. Pair it with `anyhow` for application-level error handling (easy context propagation) and `thiserror` for library error types (structured matching). The async runtime choice depends on needs—`tokio` for network I/O, but synchronous code often suffices for simpler database operations. Keep dependencies minimal: audit with `cargo tree` and use feature flags to disable unused functionality.

## Configuration follows a strict layered hierarchy

**Configuration precedence** must follow: CLI arguments → environment variables → project config file → user config file → system config → defaults. The `figment` crate handles this elegantly, merging multiple providers with correct priority:

```rust
let config: Config = Figment::new()
    .merge(Serialized::defaults(Config::default()))
    .merge(Toml::file(config_path))
    .merge(Env::prefixed("TDCLI_"))
    .merge(Serialized::defaults(Config::parse()))
    .extract()?;
```

Store configuration files in platform-appropriate locations using the `directories` crate—`~/.config/appname/` on Linux, `~/Library/Application Support/appname/` on macOS, and `%APPDATA%\appname\` on Windows. TOML works best for Rust tooling given native ecosystem support and human readability.

For **credentials specifically**, never accept passwords as CLI flags (they leak to `ps` output). Instead support `--password-file`, stdin input, or keyring integration via the `keyring` crate. Store connection profiles in a `.tdclipass` file (format: `host:port:database:user:password`) with `chmod 0600` permissions. The `secrecy` crate provides `Secret<String>` types that zero memory on drop and redact in Debug output.

## Error handling balances developer and user needs

**Combine thiserror and anyhow** for the best of both worlds. Define structured error types with thiserror for the library layer where callers need to match variants, then use anyhow at the application layer for easy error propagation with context:

```rust
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection failed to {host}: {source}")]
    Connection { host: String, #[source] source: IoError },
    
    #[error("Query error: {0}")]
    Query(String),
}

// In application code:
let conn = connect(&config)
    .context("Failed to establish database connection")?;
```

For **user-facing errors**, consider `miette` which provides rich diagnostic output with source code snippets, error codes, and help suggestions. Always distinguish between user errors (bad input, missing files) and system errors (network failures, bugs). Exit codes should follow conventions: **0** for success, **1** for general errors, **2** for usage errors. The `exitcode` crate provides BSD-standard constants.

## CLI design follows UNIX conventions strictly

**Clap v4's derive API** provides the cleanest argument definition. Structure the CLI with a top-level struct containing global options and a subcommand enum. Use `#[command(flatten)]` liberally to compose reusable option groups:

```rust
#[derive(Parser)]
#[command(name = "tdcli", version, about)]
pub struct Cli {
    #[command(flatten)]
    global: GlobalOpts,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Connect(ConnectArgs),
    Query { sql: String },
    #[command(subcommand)]
    Catalog(CatalogCommand),
}
```

Mark global options with `global = true` so they can appear anywhere in the command line. Support environment variable fallbacks with `#[arg(env = "TDCLI_HOST")]`. Use `ValueEnum` derive for enumerated options like output format (`--format table|json|csv`).

**UNIX philosophy adherence** means: accepting `-` as stdin/stdout placeholder, respecting `--` to separate options from positional arguments, supporting standard flags (`--help/-h`, `--version/-V`, `--verbose/-v`, `--quiet/-q`), and using kebab-case for long options. Handle signals properly—`ctrlc` crate for simple Ctrl-C handling, `signal-hook` for advanced patterns like double Ctrl-C to force quit.

## Output formatting serves both humans and machines

Detect terminal context with `std::io::IsTerminal` to adjust behavior—show progress bars and colors when interactive, emit clean parseable output when piped. The **`--format` flag pattern** should support at minimum `table` (human-readable, default), `json` (machine-parseable), and `csv` (data exchange).

For **table formatting**, `tabled` offers derive macros for struct-based tables while `comfy-table` provides minimalist reliability with dynamic column wrapping. Color output through `owo-colors` is zero-allocation and respects terminal detection. Always honor `NO_COLOR` environment variable and provide `--color always|auto|never` flag.

**Progress indicators** via `indicatif` should appear for any operation over ~1 second. Use spinners for indeterminate progress, progress bars with ETA for known quantities, and `MultiProgress` for concurrent operations. When not interactive, use `ProgressBar::hidden()` as a no-op.

## Database CLIs require specialized connection handling

**Connection strings** should follow URL-style DSN format: `driver://user:pass@host:port/database?param=value`. Parse with validation, support driver aliases (e.g., `td` for Teradata), and auto-detect connection type from format. For Teradata specifically, support its authentication methods (LDAP, Kerberos, TD2) and SSL configuration modes. Becasue this is a teradata client the `driver://` element should be optional and `teradata://` or `td://` should be accepted (everything else fails with a "database not supported" message).

**Connection pooling** matters less for CLI tools than servers, but still valuable for REPL sessions. Use small pool sizes (**2-4 connections**), configure appropriate timeouts (30s wait, 30s create, 5s recycle), and implement health checks on retrieval. The `deadpool` crate provides async pooling with configurable recycling methods. For synchronous needs, `r2d2` remains reliable.

**Query execution patterns** must handle: single queries (`-c "SELECT ..."`) and script files (`-f script.sql`), single-transaction mode (`--single-transaction`) for atomicity, parameter binding for SQL injection prevention, streaming large result sets with async streams, and query cancellation on Ctrl-C. Use `sqlx` with compile-time query validation when possible—it catches type mismatches before runtime.

## REPL implementation powers interactive use

The **Read-Eval-Print Loop** forms the heart of interactive database CLIs. `reedline` (powering Nushell) or `rustyline` (readline clone) provide the foundation with command history, line editing, and completion support. Essential REPL features include:

- **Multi-line input handling**: Detect incomplete statements (unclosed quotes, missing semicolons) and continue prompting
- **Command history**: Persist to `~/.tdcli_history`, support interactive search (Ctrl-R)
- **SQL syntax highlighting**: `syntect` with Sublime Text syntax definitions or `tree-sitter-highlight`
- **Tab completion**: Context-aware suggestions—table names after FROM, column names after SELECT
- **Metacommands**: psql-style backslash commands processed before SQL interpretation

```
\l          - list databases
\dt         - list tables  
\d TABLE    - describe table structure
\timing     - toggle query timing
\x          - toggle expanded display
\q          - quit
```

**Catalog browsing** queries Teradata's DBC views: `DBC.TablesV` for tables, `DBC.ColumnsV` for columns, `DBC.IndicesV` for indexes. Cache schema metadata at connection time for completion, provide `\rehash` to refresh. Support pattern filtering with SQL wildcards.

## Testing strategies ensure reliability

**Unit tests** belong in source files (`#[cfg(test)] mod tests`), testing CLI parsing logic and core functions independently. Make functions testable by accepting `impl std::io::Write` instead of hardcoding stdout—tests can pass `Vec<u8>` to capture output.

**Integration tests** with `assert_cmd` and `predicates` verify end-to-end behavior:

```rust
#[test]
fn test_query_execution() -> Result<()> {
    let mut cmd = Command::cargo_bin("tdcli")?;
    cmd.args(["query", "SELECT 1"])
       .env("TDCLI_CONNECTION", "test://...")
       .assert()
       .success()
       .stdout(predicate::str::contains("1"));
    Ok(())
}
```

Use `assert_fs` for temporary file fixtures, `mockall` for trait mocking (especially `DatabaseClient`), and `insta` for snapshot testing complex outputs like formatted tables. For async code, `#[tokio::test]` with `tokio::time::pause()` enables deterministic time-dependent tests.

## Security requires constant vigilance

**SQL injection prevention** is non-negotiable—always use parameterized queries with bind parameters, never string interpolation. SQLx's compile-time validation catches many issues, but runtime vigilance remains essential for dynamic queries.

**Credential security** encompasses: never logging sensitive values (use `secrecy::Secret<T>`), storing credentials in keyring or permission-restricted files, avoiding command-line password flags, and supporting secure credential providers. Validate TLS certificates properly with configurable verification levels.

**Input validation** should happen at boundaries—validate all user input before processing, use newtypes for validated data, limit input sizes to prevent DoS, and sanitize paths against traversal attacks.

## Production builds optimize for distribution

**Release profile optimization** in `Cargo.toml` dramatically reduces binary size:

```toml
[profile.release]
opt-level = "z"       # Size optimization
lto = "fat"           # Link-time optimization
codegen-units = 1     # Maximum optimization
panic = "abort"       # Remove unwinding
strip = "symbols"     # Remove debug info
```

For Linux portability, target `x86_64-unknown-linux-musl` for fully static binaries. Note musl's slower allocator—replace with `jemalloc` or `mimalloc` for multi-threaded workloads. Cross-compile with `cross-rs` for other platforms.

**Distribution channels** should include: GitHub releases with platform-specific archives, `cargo install` support, `cargo-binstall` metadata for pre-built binary installation, Homebrew formula for macOS, and consideration of apt/chocolatey for respective platforms. Each archive should contain the binary, shell completions (bash, zsh, fish, PowerShell via `clap_complete`), man pages, and documentation.

## CI/CD ensures consistent quality

**GitHub Actions workflow** should run on every push/PR: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and `cargo audit`. Use `Swatinem/rust-cache@v2` for fast builds, matrix strategy across `ubuntu-latest`, `macos-latest`, and `windows-latest`.

**Release automation** with `release-plz` or `cargo-release` handles version bumping, changelog generation (via `git-cliff`), git tagging, and crates.io publishing. Embed version information with `vergen` to include git SHA, build timestamp, and target triple in `--version` output.

For **self-update capability**, the `self_update` crate enables `tdcli update` commands that fetch latest releases from GitHub, verify integrity, and replace the binary in place.

This architecture positions a Teradata CLI for production success—maintainable code structure, excellent user experience, robust error handling, comprehensive testing, and straightforward distribution. The patterns scale from simple query execution to full-featured interactive sessions with catalog browsing, syntax highlighting, and intelligent completion.