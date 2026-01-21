# tq CLI - Rust Architecture and Implementation Guide

**Version:** 1.5.0
**Status:** Production Ready
**Last Updated:** 2026-01-21

## Recent Changes (Sprint 17)

### Architecture Additions
- **Help Content Management**: Added `src/help.rs` module pattern for extended help topics (Section 15)
- **Security Patterns**: Documented file permission validation order requirements (Section 16)

### Security Improvements
- **Permission Check Order**: Established pattern to validate file permissions BEFORE reading content
- **Permission Enforcement**: Changed from warning to error for insecure password file permissions

---

## Previous Changes (Sprint 8)

### Bug Fixes
- **Table Formatting**: Changed from `ContentArrangement::Dynamic` to `ContentArrangement::DynamicFullWidth` with terminal width detection. Tables now properly expand to use available terminal width.
- **Tab Completion Feedback**: Error messages are now surfaced to users via pseudo-suggestions with `[Error: ...]` format instead of being silently logged. Status messages show `[Status: ...]` for loading/empty states.
- **Result Paging**: The pager module is now properly integrated with the executor. Uses `minus` crate (with `static_output` feature) for interactive scrolling through large result sets.
- **LIMIT Hint**: Changed error message from "Add LIMIT clause" to "Use TOP N or SAMPLE N" to reflect Teradata syntax.

### Architecture Changes
- `src/format/table.rs`: Uses `crossterm::terminal::size()` for dynamic terminal width detection
- `src/commands/repl/pager.rs`: Added `display_with_pager()` and `should_page()` public functions
- `src/commands/repl/executor.rs`: Integrated pager flow with state management
- `src/commands/repl/metadata_completer.rs`: Added error/status suggestion methods for user feedback

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Module Organization](#2-module-organization)
3. [Core Data Models](#3-core-data-models)
4. [Teradata Integration Layer](#4-teradata-integration-layer)
5. [CLI Layer](#5-cli-layer)
6. [Output Formatting](#6-output-formatting)
7. [Configuration Management](#7-configuration-management)
8. [Error Handling Strategy](#8-error-handling-strategy)
9. [Testing Strategy](#9-testing-strategy)
10. [Build Optimization](#10-build-optimization)

---

## 1. Architecture Overview

### 1.1 Design Principles

The `tq` architecture follows these core principles:

1. **Library-First Design**: All business logic in `src/lib.rs`, CLI wrapper in `src/main.rs`
2. **Separation of Concerns**: Clean boundaries between CLI, database, formatting, and configuration
3. **Trait-Based Abstraction**: Use traits for testability and future extensibility
4. **Zero-Cost Abstractions**: Leverage Rust's performance without runtime overhead
5. **Fail Fast**: Validate early, provide clear error messages
6. **Stream-First**: Never buffer large result sets in memory

### 1.2 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│                    (CLI Entry Point)                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                        lib.rs                               │
│                   (Public Library API)                      │
└─────┬───────────────┬───────────────┬─────────────────┬─────┘
      │               │               │                 │
┌─────▼──────┐  ┌────▼─────┐  ┌──────▼──────┐  ┌──────▼──────┐
│    CLI     │  │    DB    │  │   Format    │  │   Config    │
│  (clap)    │  │ (Teradata│  │  (output)   │  │  (figment)  │
└────────────┘  └──────────┘  └─────────────┘  └─────────────┘
                      │
             ┌────────▼────────┐
             │ teradatarustapi │
             │  (C bindings)   │
             └─────────────────┘
```

### 1.3 Dependencies

**Core Dependencies** (keep minimal):
```toml
[dependencies]
# CLI parsing
clap = { version = "4.5", features = ["derive", "env", "wrap_help"] }

# Database connectivity
teradatarustapi = "0.6"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Output formatting
comfy-table = "7.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
csv = "1.3"

# Configuration
figment = { version = "0.10", features = ["toml", "env"] }
toml = "0.8"
directories = "5.0"

# Security
secrecy = { version = "0.8", features = ["serde"] }

# Utilities
once_cell = "1.19"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"
mockall = "0.12"
```

**Rationale for Choices**:
- `clap`: Best-in-class CLI parser with derive macros
- `comfy-table`: Lightweight, reliable table formatting
- `anyhow`/`thiserror`: Industry standard error handling
- `figment`: Flexible configuration merging
- `secrecy`: Zero-on-drop secret handling

---

## 2. Module Organization

### 2.1 File Structure

```
tq/
├── Cargo.toml
├── build.rs                    # Build-time driver setup
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library root (public API)
│   ├── cli.rs                  # CLI argument definitions
│   ├── error.rs                # Error types
│   ├── config.rs               # Configuration management
│   ├── db/
│   │   ├── mod.rs              # Database module root
│   │   ├── connection.rs       # Connection management
│   │   ├── query.rs            # Query execution
│   │   ├── types.rs            # Type conversions
│   │   └── metadata.rs         # Schema inspection
│   ├── format/
│   │   ├── mod.rs              # Format module root
│   │   ├── table.rs            # Table formatting
│   │   ├── json.rs             # JSON formatting
│   │   └── csv.rs              # CSV formatting
│   ├── commands/
│   │   ├── mod.rs              # Command module root
│   │   ├── ping.rs             # Ping command
│   │   └── query.rs            # Query command
│   └── utils/
│       ├── mod.rs              # Utility module root
│       └── connection_string.rs # Connection parsing
├── tests/
│   ├── integration_tests.rs    # End-to-end tests
│   └── fixtures/               # Test data
└── benches/
    └── benchmarks.rs           # Performance benchmarks
```

### 2.2 Module Responsibilities

#### `main.rs` - Binary Entry Point
```rust
//! CLI entry point for tq
//!
//! This module is a thin wrapper that:
//! - Parses CLI arguments with clap
//! - Dispatches to library functions
//! - Handles process exit codes

use clap::Parser;
use tq::{cli::Cli, commands, error::Result};

fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    // Run command and handle errors
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
    }
}
```

#### `lib.rs` - Public Library API
```rust
//! tq - Teradata Query CLI Library
//!
//! This library provides the core functionality for the tq CLI tool.
//! It can be used programmatically or via the CLI binary.

// Public modules
pub mod cli;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod format;

// Re-exports for convenience
pub use error::{Error, Result};
pub use config::Config;
pub use db::{Connection, QueryResult};
pub use format::OutputFormat;
```

---

## 3. Core Data Models

### 3.1 Connection Configuration

```rust
// src/config.rs

use secrecy::{Secret, ExposeSecret};
use serde::{Deserialize, Serialize};

/// Connection configuration with secure credential handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,

    #[serde(skip_serializing)]  // Never serialize passwords
    pub password: Option<Secret<String>>,

    #[serde(default = "default_logmech")]
    pub logmech: LogonMechanism,

    #[serde(default = "default_timeout")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogonMechanism {
    TD2,      // Username/password
    LDAP,     // LDAP authentication
    KRB5,     // Kerberos
    TDNEGO,   // Teradata negotiation
}

impl ConnectionConfig {
    /// Parse from connection string: user:password@host:port/database
    pub fn from_connection_string(s: &str) -> Result<Self> {
        connection_string::parse(s)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(Error::InvalidConfig("host cannot be empty".into()));
        }
        if self.port == 0 {
            return Err(Error::InvalidConfig("port must be non-zero".into()));
        }
        Ok(())
    }

    /// Get password from various sources (file, env, prompt)
    pub fn resolve_password(&mut self, password_file: Option<&Path>) -> Result<()> {
        if self.password.is_none() {
            self.password = Some(self.get_password(password_file)?);
        }
        Ok(())
    }

    fn get_password(&self, password_file: Option<&Path>) -> Result<Secret<String>> {
        // Try password file first
        if let Some(path) = password_file {
            return read_password_from_file(path, &self);
        }

        // Try ~/.tq_passwords
        if let Some(home) = dirs::home_dir() {
            let default_path = home.join(".tq_passwords");
            if default_path.exists() {
                if let Ok(pw) = read_password_from_file(&default_path, &self) {
                    return Ok(pw);
                }
            }
        }

        // Prompt interactively if TTY
        if atty::is(atty::Stream::Stdin) {
            return prompt_password(&self.user);
        }

        Err(Error::MissingPassword(
            "No password provided. Use --password-file or set TQ_PASSWORD".into()
        ))
    }
}

/// Connection string format: user:password@host:port/database
fn default_logmech() -> LogonMechanism {
    LogonMechanism::TD2
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}
```

### 3.2 Query Result Model

```rust
// src/db/types.rs

use std::collections::HashMap;

/// Result of a query execution
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Row>,
    pub row_count: usize,
    pub execution_time: Duration,
}

/// Column metadata
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: TeradataType,
    pub nullable: bool,
}

/// Row is a vector of values
pub type Row = Vec<Value>;

/// Value represents a database value with proper type handling
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Decimal(f64),
    String(String),
    Date(String),       // ISO 8601 format
    Timestamp(String),  // ISO 8601 format
    Bytes(Vec<u8>),
}

impl Value {
    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::Integer(i) => serde_json::Value::Number((*i).into()),
            Value::Decimal(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Date(s) | Value::Timestamp(s) => serde_json::Value::String(s.clone()),
            Value::Bytes(b) => serde_json::Value::String(base64::encode(b)),
        }
    }

    /// Format for display (handles NULL specially)
    pub fn display(&self) -> String {
        match self {
            Value::Null => "[NULL]".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => format!("{:.2}", f),
            Value::String(s) => s.clone(),
            Value::Date(s) | Value::Timestamp(s) => s.clone(),
            Value::Bytes(b) => format!("<{} bytes>", b.len()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TeradataType {
    Integer,
    BigInt,
    SmallInt,
    Decimal,
    Float,
    Char,
    Varchar,
    Date,
    Time,
    Timestamp,
    Boolean,
    Blob,
    Clob,
}

impl TeradataType {
    /// Alignment for table formatting
    pub fn alignment(&self) -> Alignment {
        match self {
            TeradataType::Integer
            | TeradataType::BigInt
            | TeradataType::SmallInt
            | TeradataType::Decimal
            | TeradataType::Float => Alignment::Right,

            TeradataType::Boolean => Alignment::Center,

            _ => Alignment::Left,
        }
    }
}

pub enum Alignment {
    Left,
    Center,
    Right,
}
```

### 3.3 Streaming Result Iterator

```rust
// src/db/query.rs

/// Streaming query result iterator
///
/// This avoids buffering all rows in memory by yielding rows as they arrive
pub struct QueryResultStream {
    stmt: TeraStatement,
    columns: Vec<ColumnMetadata>,
    execution_start: Instant,
}

impl QueryResultStream {
    /// Get column metadata (available immediately)
    pub fn columns(&self) -> &[ColumnMetadata] {
        &self.columns
    }

    /// Get execution time so far
    pub fn elapsed(&self) -> Duration {
        self.execution_start.elapsed()
    }
}

impl Iterator for QueryResultStream {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stmt.fetch_next_row() {
            Ok(Some(row)) => Some(Ok(row)),
            Ok(None) => None,  // End of results
            Err(e) => Some(Err(e.into())),
        }
    }
}

// Usage example:
pub fn execute_streaming(
    conn: &Connection,
    sql: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let stream = conn.execute_stream(sql)?;

    // Write header
    write_csv_header(&stream.columns(), writer)?;

    // Stream rows without buffering
    let mut row_count = 0;
    for row in stream {
        let row = row?;
        write_csv_row(&row, writer)?;
        row_count += 1;
    }

    eprintln!("{} rows exported in {:?}", row_count, stream.elapsed());
    Ok(())
}
```

---

## 4. Teradata Integration Layer

### 4.1 Connection Management

```rust
// src/db/connection.rs

use teradatarustapi::*;

/// High-level Teradata connection wrapper
pub struct Connection {
    inner: TeraConnection,
    config: ConnectionConfig,
}

impl Connection {
    /// Establish connection to Teradata
    pub fn connect(config: ConnectionConfig) -> Result<Self> {
        config.validate()?;

        // Build connection string for Teradata driver
        let conn_str = format_connection_string(&config);

        // Connect with timeout
        let inner = TeraConnection::new(&conn_str)
            .map_err(|e| Error::ConnectionFailed {
                host: config.host.clone(),
                port: config.port,
                source: e.into(),
            })?;

        Ok(Self { inner, config })
    }

    /// Ping database to test connectivity
    pub fn ping(&self) -> Result<Duration> {
        let start = Instant::now();
        self.execute("SELECT 1")?;
        Ok(start.elapsed())
    }

    /// Execute query and return buffered results
    ///
    /// Use this for small result sets where you need all data at once
    pub fn execute(&self, sql: &str) -> Result<QueryResult> {
        let start = Instant::now();

        let stmt = self.inner.prepare(sql)?;
        let result_set = stmt.execute()?;

        let columns = extract_columns(&result_set)?;
        let mut rows = Vec::new();

        while let Some(row) = result_set.fetch_next()? {
            rows.push(convert_row(&row, &columns)?);
        }

        Ok(QueryResult {
            columns,
            row_count: rows.len(),
            rows,
            execution_time: start.elapsed(),
        })
    }

    /// Execute query and return streaming iterator
    ///
    /// Use this for large result sets to avoid memory exhaustion
    pub fn execute_stream(&self, sql: &str) -> Result<QueryResultStream> {
        let start = Instant::now();

        let stmt = self.inner.prepare(sql)?;
        let result_set = stmt.execute()?;
        let columns = extract_columns(&result_set)?;

        Ok(QueryResultStream {
            stmt,
            columns,
            execution_start: start,
        })
    }

    /// Get session information
    pub fn session_info(&self) -> Result<SessionInfo> {
        // Query DBC views for session details
        let sql = "SELECT SESSION, USER, DATABASE, CHARACTER_SET_NAME
                   FROM DBC.SessionInfoV WHERE SessionNo = SESSION";

        let result = self.execute(sql)?;
        parse_session_info(result)
    }
}

/// Format connection string for Teradata driver
fn format_connection_string(config: &ConnectionConfig) -> String {
    let mut params = vec![
        format!("host={}", config.host),
        format!("user={}", config.user),
        format!("dbs_port={}", config.port),
        format!("database={}", config.database),
        format!("logmech={}", format_logmech(config.logmech)),
    ];

    // Add password if present
    if let Some(password) = &config.password {
        params.push(format!("password={}", password.expose_secret()));
    }

    // Add timeout
    params.push(format!("connect_timeout={}", config.timeout.as_secs()));

    // Join with semicolons
    params.join(";")
}

fn format_logmech(logmech: LogonMechanism) -> &'static str {
    match logmech {
        LogonMechanism::TD2 => "TD2",
        LogonMechanism::LDAP => "LDAP",
        LogonMechanism::KRB5 => "KRB5",
        LogonMechanism::TDNEGO => "TDNEGO",
    }
}

/// Extract column metadata from result set
fn extract_columns(result_set: &TeraResultSet) -> Result<Vec<ColumnMetadata>> {
    let mut columns = Vec::new();

    for i in 0..result_set.column_count()? {
        let name = result_set.column_name(i)?;
        let type_code = result_set.column_type(i)?;
        let nullable = result_set.column_nullable(i)?;

        columns.push(ColumnMetadata {
            name,
            data_type: map_teradata_type(type_code),
            nullable,
        });
    }

    Ok(columns)
}

/// Map Teradata type codes to our enum
fn map_teradata_type(type_code: i32) -> TeradataType {
    match type_code {
        // See Teradata documentation for type codes
        -5 => TeradataType::BigInt,
        4 => TeradataType::Integer,
        5 => TeradataType::SmallInt,
        3 => TeradataType::Decimal,
        6 | 8 => TeradataType::Float,
        1 => TeradataType::Char,
        12 => TeradataType::Varchar,
        91 => TeradataType::Date,
        92 => TeradataType::Time,
        93 => TeradataType::Timestamp,
        16 => TeradataType::Boolean,
        2004 => TeradataType::Blob,
        2005 => TeradataType::Clob,
        _ => TeradataType::Varchar,  // Default fallback
    }
}

/// Convert Teradata row to our Row type
fn convert_row(tera_row: &TeraRow, columns: &[ColumnMetadata]) -> Result<Row> {
    let mut row = Vec::with_capacity(columns.len());

    for (i, col) in columns.iter().enumerate() {
        let value = if tera_row.is_null(i)? {
            Value::Null
        } else {
            convert_value(tera_row, i, col.data_type)?
        };
        row.push(value);
    }

    Ok(row)
}

/// Convert individual value based on type
fn convert_value(row: &TeraRow, index: usize, data_type: TeradataType) -> Result<Value> {
    Ok(match data_type {
        TeradataType::Boolean => Value::Boolean(row.get_bool(index)?),
        TeradataType::Integer | TeradataType::SmallInt =>
            Value::Integer(row.get_i64(index)?),
        TeradataType::BigInt => Value::Integer(row.get_i64(index)?),
        TeradataType::Decimal | TeradataType::Float =>
            Value::Decimal(row.get_f64(index)?),
        TeradataType::Char | TeradataType::Varchar =>
            Value::String(row.get_string(index)?),
        TeradataType::Date => Value::Date(row.get_date(index)?.to_string()),
        TeradataType::Timestamp => Value::Timestamp(row.get_timestamp(index)?.to_string()),
        TeradataType::Blob | TeradataType::Clob =>
            Value::Bytes(row.get_bytes(index)?),
        _ => Value::String(row.get_string(index)?),
    })
}
```

### 4.2 Schema Metadata Queries

```rust
// src/db/metadata.rs

impl Connection {
    /// List all databases accessible to the user
    pub fn list_databases(&self) -> Result<Vec<String>> {
        let sql = "SELECT DatabaseName FROM DBC.DatabasesV ORDER BY DatabaseName";
        let result = self.execute(sql)?;

        Ok(result.rows.into_iter()
            .filter_map(|row| row.get(0).and_then(|v| v.as_string().cloned()))
            .collect())
    }

    /// List tables in current database (or pattern)
    pub fn list_tables(&self, pattern: Option<&str>) -> Result<Vec<TableInfo>> {
        let sql = match pattern {
            Some(p) => format!(
                "SELECT TableName, TableKind, CreatorName
                 FROM DBC.TablesV
                 WHERE DatabaseName = DATABASE
                   AND TableName LIKE '{}'
                 ORDER BY TableName",
                escape_sql_like(p)
            ),
            None =>
                "SELECT TableName, TableKind, CreatorName
                 FROM DBC.TablesV
                 WHERE DatabaseName = DATABASE
                 ORDER BY TableName".to_string(),
        };

        let result = self.execute(&sql)?;
        parse_table_info(result)
    }

    /// Describe table structure
    pub fn describe_table(&self, table_name: &str) -> Result<TableDescription> {
        let sql = format!(
            "SELECT ColumnName, ColumnType, Nullable, DefaultValue, ColumnId
             FROM DBC.ColumnsV
             WHERE DatabaseName = DATABASE
               AND TableName = '{}'
             ORDER BY ColumnId",
            escape_sql_string(table_name)
        );

        let columns_result = self.execute(&sql)?;

        let indexes_sql = format!(
            "SELECT IndexName, IndexType, ColumnName
             FROM DBC.IndicesV
             WHERE DatabaseName = DATABASE
               AND TableName = '{}'
             ORDER BY IndexName, IndexNumber",
            escape_sql_string(table_name)
        );

        let indexes_result = self.execute(&indexes_sql)?;

        Ok(TableDescription {
            table_name: table_name.to_string(),
            columns: parse_columns(columns_result)?,
            indexes: parse_indexes(indexes_result)?,
        })
    }
}

#[derive(Debug)]
pub struct TableInfo {
    pub name: String,
    pub kind: TableKind,
    pub owner: String,
}

#[derive(Debug)]
pub enum TableKind {
    Table,
    View,
    Macro,
}

#[derive(Debug)]
pub struct TableDescription {
    pub table_name: String,
    pub columns: Vec<ColumnDescription>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug)]
pub struct ColumnDescription {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct IndexInfo {
    pub name: String,
    pub index_type: String,
    pub columns: Vec<String>,
}

fn escape_sql_string(s: &str) -> String {
    s.replace("'", "''")
}

fn escape_sql_like(s: &str) -> String {
    s.replace("'", "''")
        .replace("_", "\\_")
        .replace("%", "\\%")
}
```

---

## 5. CLI Layer

### 5.1 Argument Definition

```rust
// src/cli.rs

use clap::{Parser, Subcommand, ValueEnum};

/// tq - Teradata Query
///
/// A fast, lightweight command-line client for Teradata databases
#[derive(Parser, Debug)]
#[command(name = "tq")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

/// Global options that apply to all commands
#[derive(Parser, Debug, Clone)]
pub struct GlobalOpts {
    /// Connection string: user:password@host:port/database
    ///
    /// If not provided, reads from TQ_LOGON environment variable
    #[arg(short = 'l', long, env = "TQ_LOGON", global = true)]
    pub logon: Option<String>,

    /// Read password from file (recommended for security)
    ///
    /// File format: one password per line, or host:port:db:user:password
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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Test database connectivity
    Ping(PingArgs),

    /// Execute a SQL query
    Query(QueryArgs),

    /// Start interactive REPL mode (future)
    #[command(hide = true)]  // Hide until implemented
    Repl(ReplArgs),
}

/// Arguments for the ping command
#[derive(Parser, Debug)]
pub struct PingArgs {
    /// Number of ping attempts
    #[arg(short, long, default_value = "1")]
    pub count: u32,

    /// Interval between pings
    #[arg(short, long, default_value = "1s")]
    pub interval: String,
}

/// Arguments for the query command
#[derive(Parser, Debug)]
pub struct QueryArgs {
    /// SQL query to execute
    ///
    /// Mutually exclusive with --file. If neither provided, reads from stdin.
    #[arg(value_name = "QUERY", conflicts_with = "file")]
    pub query: Option<String>,

    /// Read SQL from file
    #[arg(long, value_name = "FILE", conflicts_with = "query")]
    pub file: Option<PathBuf>,

    /// Output format
    #[arg(short, long, env = "TQ_FORMAT", default_value = "table")]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Omit column headers in output
    #[arg(long)]
    pub no_header: bool,

    /// Show query execution time
    #[arg(long)]
    pub timing: bool,

    /// Limit number of rows returned
    #[arg(short = 'n', long, value_name = "N")]
    pub limit: Option<usize>,
}

#[derive(Parser, Debug)]
pub struct ReplArgs {
    /// Disable command history
    #[arg(long)]
    pub no_history: bool,

    /// History file location
    #[arg(long, default_value = "~/.tq_history")]
    pub history_file: PathBuf,

    /// Editor mode (emacs or vi)
    #[arg(long, default_value = "emacs")]
    pub editor_mode: EditorMode,
}

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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EditorMode {
    Emacs,
    Vi,
}

impl GlobalOpts {
    /// Build connection config from CLI options
    pub fn build_connection_config(&self) -> Result<ConnectionConfig> {
        if let Some(ref logon) = self.logon {
            ConnectionConfig::from_connection_string(logon)
        } else {
            // Try loading from config file
            Config::load()?.connection
        }
    }
}
```

### 5.2 Command Implementation

```rust
// src/commands/ping.rs

use crate::{GlobalOpts, PingArgs, db::Connection, error::Result};

pub fn execute(global: GlobalOpts, args: PingArgs) -> Result<()> {
    let mut config = global.build_connection_config()?;
    config.resolve_password(global.password_file.as_deref())?;

    let interval = parse_duration(&args.interval)?;

    for i in 1..=args.count {
        let result = ping_once(&config);

        match result {
            Ok(duration) => {
                if !global.quiet {
                    println!(
                        "Database connection successful ({}ms)",
                        duration.as_millis()
                    );
                }
            }
            Err(e) => {
                eprintln!("Error: Connection failed");
                eprintln!("{}", e);
                return Err(e);
            }
        }

        // Sleep between pings (except after last)
        if i < args.count {
            std::thread::sleep(interval);
        }
    }

    Ok(())
}

fn ping_once(config: &ConnectionConfig) -> Result<Duration> {
    let conn = Connection::connect(config.clone())?;
    conn.ping()
}

// src/commands/query.rs

use crate::{GlobalOpts, QueryArgs, db::Connection, format, error::Result};

pub fn execute(global: GlobalOpts, args: QueryArgs) -> Result<()> {
    // Get SQL from argument, file, or stdin
    let sql = get_sql_input(&args)?;

    // Build connection
    let mut config = global.build_connection_config()?;
    config.resolve_password(global.password_file.as_deref())?;

    let conn = Connection::connect(config)?;

    // Determine output destination
    let mut writer: Box<dyn Write> = match args.output {
        Some(ref path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(stdout().lock()),
    };

    // Execute and format output
    let start = Instant::now();

    match args.format {
        OutputFormat::Table => {
            // Buffer results for table formatting
            let result = conn.execute(&sql)?;
            format::table::write(&result, &mut writer, !args.no_header)?;

            if args.timing {
                eprintln!("\n{} rows in set ({:.3}s)",
                    result.row_count,
                    result.execution_time.as_secs_f64()
                );
            }
        }

        OutputFormat::Json => {
            // Buffer results for JSON array
            let result = conn.execute(&sql)?;
            format::json::write(&result, &mut writer)?;
        }

        OutputFormat::Csv => {
            // Stream results to CSV
            let stream = conn.execute_stream(&sql)?;
            format::csv::write_stream(stream, &mut writer, !args.no_header)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn get_sql_input(args: &QueryArgs) -> Result<String> {
    if let Some(ref query) = args.query {
        Ok(query.clone())
    } else if let Some(ref file) = args.file {
        std::fs::read_to_string(file)
            .map_err(|e| Error::FileReadError(file.clone(), e))
    } else {
        // Read from stdin
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}
```

---

## 6. Output Formatting

### 6.1 Table Formatting

```rust
// src/format/table.rs

use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
use crate::db::{QueryResult, Value};

pub fn write(result: &QueryResult, writer: &mut dyn Write, with_header: bool) -> Result<()> {
    let mut table = Table::new();

    // Configure table style
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_FULL);

    // Add header row
    if with_header {
        let header_cells: Vec<Cell> = result.columns
            .iter()
            .map(|col| {
                Cell::new(&col.name)
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan)
            })
            .collect();

        table.set_header(header_cells);
    }

    // Add data rows
    for row in &result.rows {
        let cells: Vec<Cell> = row.iter()
            .zip(&result.columns)
            .map(|(value, col)| {
                let cell = Cell::new(value.display());

                // Apply alignment based on type
                let cell = match col.data_type.alignment() {
                    Alignment::Left => cell,
                    Alignment::Center => cell.set_alignment(comfy_table::CellAlignment::Center),
                    Alignment::Right => cell.set_alignment(comfy_table::CellAlignment::Right),
                };

                // Style NULL values
                if matches!(value, Value::Null) {
                    cell.fg(Color::DarkGrey).add_attribute(Attribute::Italic)
                } else {
                    cell
                }
            })
            .collect();

        table.add_row(cells);
    }

    // Write table
    write!(writer, "{}", table)?;
    Ok(())
}
```

### 6.2 JSON Formatting

```rust
// src/format/json.rs

use crate::db::QueryResult;
use serde_json::{json, to_writer_pretty};

pub fn write(result: &QueryResult, writer: &mut dyn Write) -> Result<()> {
    let rows: Vec<serde_json::Value> = result.rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (value, col) in row.iter().zip(&result.columns) {
                obj.insert(col.name.clone(), value.to_json());
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    to_writer_pretty(writer, &rows)?;
    writeln!(writer)?;
    Ok(())
}

/// Write streaming JSONL (one object per line)
pub fn write_stream(
    stream: QueryResultStream,
    writer: &mut dyn Write,
) -> Result<()> {
    for row in stream {
        let row = row?;
        let mut obj = serde_json::Map::new();

        for (value, col) in row.iter().zip(stream.columns()) {
            obj.insert(col.name.clone(), value.to_json());
        }

        serde_json::to_writer(&mut *writer, &obj)?;
        writeln!(writer)?;
    }

    Ok(())
}
```

### 6.3 CSV Formatting

```rust
// src/format/csv.rs

use csv::Writer;
use crate::db::{QueryResult, QueryResultStream, Value};

pub fn write(result: &QueryResult, writer: &mut dyn Write, with_header: bool) -> Result<()> {
    let mut csv_writer = Writer::from_writer(writer);

    // Write header
    if with_header {
        let headers: Vec<&str> = result.columns.iter()
            .map(|col| col.name.as_str())
            .collect();
        csv_writer.write_record(&headers)?;
    }

    // Write rows
    for row in &result.rows {
        let fields: Vec<String> = row.iter()
            .map(|v| format_csv_value(v))
            .collect();
        csv_writer.write_record(&fields)?;
    }

    csv_writer.flush()?;
    Ok(())
}

/// Write streaming CSV (memory-efficient for large results)
pub fn write_stream(
    mut stream: QueryResultStream,
    writer: &mut dyn Write,
    with_header: bool,
) -> Result<()> {
    let mut csv_writer = Writer::from_writer(writer);

    // Write header
    if with_header {
        let headers: Vec<&str> = stream.columns().iter()
            .map(|col| col.name.as_str())
            .collect();
        csv_writer.write_record(&headers)?;
    }

    // Stream rows
    for row in stream {
        let row = row?;
        let fields: Vec<String> = row.iter()
            .map(|v| format_csv_value(v))
            .collect();
        csv_writer.write_record(&fields)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn format_csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),  // Empty field for NULL
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Decimal(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Date(s) | Value::Timestamp(s) => s.clone(),
        Value::Bytes(b) => base64::encode(b),
    }
}
```

---

## 7. Configuration Management

### 7.1 Configuration Loading

```rust
// src/config.rs

use figment::{Figment, providers::{Format, Toml, Env, Serialized}};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub connection: ConnectionConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub repl: ReplConfig,

    #[serde(default)]
    pub profiles: HashMap<String, ConnectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_color")]
    pub color: String,

    #[serde(default)]
    pub timing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    #[serde(default = "default_history_file")]
    pub history_file: PathBuf,

    #[serde(default = "default_history_size")]
    pub history_size: usize,

    #[serde(default = "default_editor_mode")]
    pub editor_mode: String,

    #[serde(default = "bool_true")]
    pub syntax_highlight: bool,

    #[serde(default = "bool_true")]
    pub autocomplete: bool,
}

impl Config {
    /// Load configuration from all sources
    pub fn load() -> Result<Self> {
        let config: Config = Figment::new()
            // 1. Built-in defaults
            .merge(Serialized::defaults(Config::default()))

            // 2. System config
            .merge(Toml::file("/etc/tq/config.toml").nested())

            // 3. User config
            .merge(Toml::file(Self::user_config_path()).nested())

            // 4. Project config
            .merge(Toml::file(".tq.toml").nested())

            // 5. Environment variables
            .merge(Env::prefixed("TQ_").split("_"))

            .extract()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Get user config file path
    fn user_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("tq").join("config.toml")
        } else {
            PathBuf::from("~/.config/tq/config.toml")
        }
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        self.connection.validate()?;

        // Check file permissions on config file (warn if too open)
        let config_path = Self::user_config_path();
        if config_path.exists() {
            check_file_permissions(&config_path)?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            output: OutputConfig::default(),
            repl: ReplConfig::default(),
            profiles: HashMap::new(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "table".to_string(),
            color: "auto".to_string(),
            timing: false,
        }
    }
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            history_file: PathBuf::from("~/.tq_history"),
            history_size: 10000,
            editor_mode: "emacs".to_string(),
            syntax_highlight: true,
            autocomplete: true,
        }
    }
}

/// Check file permissions and warn if unsafe
fn check_file_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if file is readable by group or others
        if mode & 0o077 != 0 {
            eprintln!("Warning: Config file {} has unsafe permissions ({:o})",
                path.display(), mode & 0o777);
            eprintln!("Expected: 0600 (owner read/write only)");
            eprintln!("Fix: chmod 0600 {}", path.display());
        }
    }

    Ok(())
}

fn default_format() -> String { "table".to_string() }
fn default_color() -> String { "auto".to_string() }
fn default_history_file() -> PathBuf { PathBuf::from("~/.tq_history") }
fn default_history_size() -> usize { 10000 }
fn default_editor_mode() -> String { "emacs".to_string() }
fn bool_true() -> bool { true }
```

---

## 8. Error Handling Strategy

### 8.1 Error Types

```rust
// src/error.rs

use thiserror::Error;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

/// Application error types
#[derive(Error, Debug)]
pub enum Error {
    /// Connection errors
    #[error("Failed to connect to {host}:{port}")]
    ConnectionFailed {
        host: String,
        port: u16,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Connection timeout after {timeout:?}")]
    ConnectionTimeout {
        timeout: Duration,
    },

    #[error("Authentication failed for user '{user}' using {logmech}")]
    AuthenticationFailed {
        user: String,
        logmech: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Query errors
    #[error("SQL syntax error: {message}")]
    SqlSyntaxError {
        message: String,
        query: Option<String>,
    },

    #[error("Query execution failed: {0}")]
    QueryExecutionError(String),

    #[error("Table '{table}' does not exist")]
    TableNotFound {
        table: String,
    },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Configuration errors
    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Missing password. Use --password-file or interactive prompt")]
    MissingPassword(String),

    /// I/O errors
    #[error("Failed to read file {}: {}", .0.display(), .1)]
    FileReadError(PathBuf, #[source] std::io::Error),

    #[error("Failed to write file {}: {}", .0.display(), .1)]
    FileWriteError(PathBuf, #[source] std::io::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Parsing errors
    #[error("Failed to parse duration: {0}")]
    ParseDurationError(String),

    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(String),

    /// Internal errors (bugs)
    #[error("Internal error: {0}\n\nThis is a bug. Please report it!")]
    InternalError(String),
}

impl Error {
    /// Get exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            // Usage errors
            Error::InvalidConnectionString(_)
            | Error::InvalidConfig(_)
            | Error::ParseDurationError(_)
            | Error::ConfigParseError(_) => 2,

            // Runtime errors
            _ => 1,
        }
    }

    /// Get user-friendly error message with context
    pub fn user_message(&self) -> String {
        match self {
            Error::ConnectionFailed { host, port, .. } => {
                format!(
                    "Error: Failed to connect to {}:{}\n\n\
                     Possible causes:\n\
                     - Database is not running\n\
                     - Hostname or port is incorrect\n\
                     - Firewall is blocking connection\n\
                     - Network is unreachable\n\n\
                     Troubleshooting:\n\
                     1. Verify hostname resolves: ping {}\n\
                     2. Check port is open: nc -zv {} {}\n\
                     3. Confirm credentials are correct\n\
                     4. Check firewall rules",
                    host, port, host, host, port
                )
            }

            Error::AuthenticationFailed { user, logmech, .. } => {
                format!(
                    "Error: Authentication failed\n\n\
                     User: {}\n\
                     Logon mechanism: {}\n\n\
                     Troubleshooting:\n\
                     - Verify username and password are correct\n\
                     - Check if account is locked\n\
                     - Try different logon mechanism: --logmech LDAP",
                    user, logmech
                )
            }

            Error::SqlSyntaxError { message, query } => {
                let mut msg = format!("Error: SQL syntax error\n\n{}", message);
                if let Some(q) = query {
                    msg.push_str(&format!("\n\nQuery:\n{}", q));
                }
                msg
            }

            Error::TableNotFound { table } => {
                format!(
                    "Error: Table '{}' does not exist\n\n\
                     Suggestions:\n\
                     - Check spelling\n\
                     - List tables: tq query \"SELECT * FROM DBC.TablesV WHERE DatabaseName = DATABASE\"\n\
                     - Check current database: tq query \"SELECT DATABASE\"",
                    table
                )
            }

            _ => format!("Error: {}", self),
        }
    }
}
```

### 8.2 Error Display in main.rs

```rust
// In main.rs

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        // Print user-friendly error message
        eprintln!("{}", e.user_message());

        // Exit with appropriate code
        std::process::exit(e.exit_code());
    }
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
// In src/db/connection.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_from_string() {
        let config = ConnectionConfig::from_connection_string(
            "alice:secret@dbhost:1025/mydb"
        ).unwrap();

        assert_eq!(config.user, "alice");
        assert_eq!(config.host, "dbhost");
        assert_eq!(config.port, 1025);
        assert_eq!(config.database, "mydb");
    }

    #[test]
    fn test_connection_config_validation() {
        let mut config = ConnectionConfig::default();
        config.host = String::new();  // Empty host

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_value_to_json() {
        assert_eq!(Value::Integer(42).to_json(), json!(42));
        assert_eq!(Value::String("hello".into()).to_json(), json!("hello"));
        assert_eq!(Value::Null.to_json(), json!(null));
    }
}
```

### 9.2 Integration Tests

```rust
// tests/integration_tests.rs

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Teradata Query"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_query_missing_connection() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("query")
        .arg("SELECT 1")
        .assert()
        .failure()
        .code(2)  // Usage error
        .stderr(predicate::str::contains("connection"));
}

#[test]
fn test_query_from_file() {
    let mut sql_file = NamedTempFile::new().unwrap();
    writeln!(sql_file, "SELECT 1;").unwrap();

    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("--logon")
        .arg("test:test@localhost:1025/test")
        .arg("query")
        .arg("--file")
        .arg(sql_file.path())
        .assert()
        .success();
}

#[test]
fn test_query_json_output() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("--logon")
        .arg("test:test@localhost:1025/test")
        .arg("query")
        .arg("--format")
        .arg("json")
        .arg("SELECT 1 AS value")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"{"value":1}"#));
}
```

### 9.3 Mock Testing for Database

```rust
// src/db/connection.rs

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait DatabaseClient {
    fn connect(&self, config: &ConnectionConfig) -> Result<Connection>;
    fn execute(&self, sql: &str) -> Result<QueryResult>;
    fn execute_stream(&self, sql: &str) -> Result<QueryResultStream>;
}

// Use mock in tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_command_with_mock() {
        let mut mock = MockDatabaseClient::new();

        mock.expect_execute()
            .returning(|_| Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: Duration::from_millis(10),
            }));

        // Test query logic with mock
    }
}
```

---

## 10. Build Optimization

### 10.1 Cargo.toml Configuration

```toml
[package]
name = "tq"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A fast, lightweight command-line client for Teradata databases"
license = "MIT"
repository = "https://github.com/yourusername/tq"
keywords = ["teradata", "database", "cli", "sql"]
categories = ["command-line-utilities", "database"]

[dependencies]
# (as specified above)

[profile.release]
opt-level = "z"         # Optimize for size
lto = "fat"             # Full link-time optimization
codegen-units = 1       # Single codegen unit for max optimization
strip = "symbols"       # Strip debug symbols
panic = "abort"         # No unwinding (smaller binary)

[profile.dev]
opt-level = 0           # No optimization for faster builds
debug = true            # Include debug symbols

# Optimize dependencies even in dev mode
[profile.dev.package."*"]
opt-level = 3
```

### 10.2 Build Script for Driver Bundling

```rust
// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    // Get Teradata driver library directory
    let driver_dir = env::var("TQ_DRIVER_LIB_DIR")
        .unwrap_or_else(|_| "/opt/teradata/client/lib".to_string());

    // Tell cargo to link the Teradata libraries
    println!("cargo:rustc-link-search=native={}", driver_dir);
    println!("cargo:rustc-link-lib=dylib=teradatapilib");

    // Tell cargo to rerun if the driver location changes
    println!("cargo:rerun-if-env-changed=TQ_DRIVER_LIB_DIR");

    // For static linking on Linux (musl)
    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "musl" {
        println!("cargo:rustc-link-arg=-static");
    }
}
```

### 10.3 Cross-Compilation Support

```bash
# Install cross
cargo install cross

# Build for Linux (musl for static binary)
cross build --release --target x86_64-unknown-linux-musl

# Build for macOS
cargo build --release --target x86_64-apple-darwin

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu
```

### 10.4 Binary Size Analysis

```bash
# Analyze binary size
cargo bloat --release

# Show largest dependencies
cargo tree --edges normal --prefix depth | head -20

# Strip additional symbols
strip target/release/tq

# Use UPX compression (optional, may affect performance)
upx --best --lzma target/release/tq
```

---

## 11. Batch Mode Architecture (Sprint 10)

This section documents the batch mode architecture implemented in Sprint 10.

### 11.1 Overview

Batch mode enables non-interactive use of tq for scripts, cron jobs, CI/CD pipelines, and command-line data processing. Key capabilities:

- Execute SQL from files (`--file`)
- Accept piped input from stdin
- Execute multiple statements sequentially
- Fail-fast error handling with context
- All output formats (table, JSON, CSV)

### 11.2 Module Structure

```
src/
├── sql/
│   ├── mod.rs           # Module exports
│   └── parser.rs        # SQL statement parsing
├── commands/
│   └── query.rs         # Batch execution (refactored)
└── lib.rs               # Add: pub mod sql;
```

### 11.3 Input Source Architecture

#### InputSource Enum

```rust
/// Represents the source of SQL input
#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    /// SQL provided as command-line argument
    Argument(String),
    /// SQL read from a file
    File(PathBuf),
    /// SQL read from stdin (piped input)
    Stdin,
}
```

#### Input Source Resolution

**Precedence order:**
1. Explicit SQL argument: `tq query "SELECT 1"`
2. File flag: `tq query --file script.sql`
3. stdin: `cat script.sql | tq query`

**stdin detection:** Uses `std::io::IsTerminal::is_terminal()` (Rust 1.70+) to detect piped vs interactive input.

### 11.4 Statement Parser

The statement parser (`src/sql/parser.rs`) provides simple semicolon-based statement splitting.

#### Design Decisions

1. **Simple splitting**: Split on `;` without full SQL grammar parsing
2. **Comments preserved**: Pass through to Teradata (handles them correctly)
3. **Line tracking**: Track line numbers for error messages
4. **Empty handling**: Skip whitespace-only statements

#### Known Limitations

- Semicolons inside string literals may cause incorrect splits (rare in practice)
- Documented limitation; full SQL parsing deferred to future if needed

#### Key Types

```rust
/// A parsed SQL statement with metadata
pub struct ParsedStatement {
    pub sql: String,           // Statement text (trimmed)
    pub statement_number: usize, // 1-based index
    pub start_line: usize,     // Line number for errors
}

/// Parse SQL text into individual statements
pub fn parse_statements(sql: &str) -> Vec<ParsedStatement>
```

### 11.5 Batch Executor

#### Execution Flow

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│ Input       │───▶│ Resolve      │───▶│ Read SQL        │
│ Arguments   │    │ Source       │    │ Content         │
└─────────────┘    └──────────────┘    └────────┬────────┘
                                                │
                                                ▼
                           ┌────────────────────────────────┐
                           │ parse_statements()             │
                           │ "SELECT 1; SELECT 2;" ──▶      │
                           │ [ParsedStatement, ...]         │
                           └───────────────┬────────────────┘
                                           │
                                           ▼
                   ┌───────────────────────────────────────────┐
                   │           Batch Executor Loop             │
                   │                                           │
                   │  for statement in statements:             │
                   │    1. Progress message → stderr           │
                   │    2. Execute via DatabaseClient          │
                   │    3. Success: format + output            │
                   │       Error: stop + return context        │
                   └───────────────────────────────────────────┘
```

#### Fail-Fast Behavior

- Executes statements sequentially
- Stops immediately on first error
- Reports: statement number, line number, preview, Teradata error
- Returns appropriate exit code (1 for runtime errors, 2 for usage errors)

#### Error Context

```rust
pub struct BatchExecutionError {
    pub statement: ParsedStatement,  // Failed statement
    pub error: TqError,              // Underlying error
    pub successful_count: usize,     // Statements before failure
}
```

Example error output:
```
Error at statement 2 (line 4): SQL syntax error: [Error 3707]...

Statement: SELECT * FORM users...
```

### 11.6 Integration Points

#### Reused Components

| Component | Usage in Batch Mode |
|-----------|-------------------|
| `DatabaseClient` | Execute statements (unchanged) |
| `FormatOptions` | Configure output formatting |
| `write_output_with_timing` | Format and write results |
| `TqError` | All error variants |

#### REPL vs Batch Mode

| Aspect | REPL | Batch |
|--------|------|-------|
| Input | Interactive line editor | File/stdin/argument |
| Execution | Single statement, continue on error | Sequential, fail-fast |
| Output | Pager for large results | Direct to stdout |
| State | Session state maintained | Stateless |
| Progress | Not shown | To stderr |

### 11.7 Testing Strategy

#### Test Levels

1. **Unit tests** (inline in source files)
   - Statement parser: edge cases, whitespace, comments
   - Input source resolution: precedence, validation
   - Error formatting: context, preview

2. **Integration tests** (`tests/integration/`)
   - CLI argument handling
   - File input processing
   - Error propagation

3. **Manual tests** (with real database)
   - stdin piping (echo, heredoc, cat)
   - File execution
   - Multi-statement batches
   - Error handling verification

#### Coverage Targets

| Component | Target |
|-----------|--------|
| Statement parser | 95%+ |
| Input resolution | 90%+ |
| Batch executor | 85%+ |
| Error propagation | 90%+ |

---

## 12. Implementation Roadmap

### Phase 1: MVP (Current)
- ✅ Basic CLI structure with clap
- ✅ Teradata connection management
- ✅ `ping` command
- ✅ `query` command with table/JSON/CSV output
- ✅ Connection string parsing
- ✅ Secure credential handling
- ✅ Error handling with user-friendly messages

### Phase 2: Configuration & Profiles
- [ ] Configuration file loading (figment)
- [ ] Connection profiles
- [ ] Password file support
- [ ] Environment variable precedence

### Phase 3: REPL Mode (COMPLETED)
- [x] Interactive REPL with reedline
- [x] Command history (persistent to ~/.tq_history)
- [x] Multi-line SQL input
- [x] Syntax highlighting (nu-ansi-term)
- [x] Tab completion (database-aware)
- [x] Result paging (vertical and horizontal)

### Phase 4: Batch Mode (Sprint 10 - In Progress)
- [ ] stdin input support (piped SQL)
- [ ] File input support (--file flag)
- [ ] Multi-statement execution
- [ ] Enhanced batch error messages
- [ ] Batch output behavior (no pager, TTY detection)

### Phase 5: Advanced Features
- [ ] Schema metadata commands
- [ ] Transaction support (--atomic flag)
- [ ] Variable substitution (--var flag)
- [ ] Query templates
- [ ] SSL/TLS support
- [ ] Keyring integration

### Phase 6: Distribution
- [ ] Shell completions (bash/zsh/fish)
- [ ] Man pages
- [ ] Homebrew formula
- [ ] Binary releases
- [ ] Docker image

---

## 12. Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Startup time | < 100ms | Fast CLI feel |
| Memory (idle) | < 10 MB | Lightweight tool |
| Memory (query) | < 50 MB | Constant regardless of result size |
| Binary size | < 5 MB | Easy distribution |
| Query latency | < 1ms overhead | Minimal CLI overhead |
| Large export (10M rows) | Streaming | No memory growth |

**Measurement**:
```bash
# Startup time
time tq --version

# Memory usage
/usr/bin/time -v tq query "SELECT * FROM large_table" > /dev/null

# Binary size
ls -lh target/release/tq
```

---

## 13. Security Checklist

- [x] Never log passwords (use secrecy::Secret)
- [x] Sanitize connection strings in error messages
- [x] Check file permissions on config files
- [x] Use password files with 0600 permissions
- [x] Clear secrets from memory on drop
- [ ] Support keyring integration (future)
- [ ] Support SSL/TLS connections (future)
- [ ] Audit logging for compliance (future)

---

## 14. Code Quality Standards

### Linting
```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Documentation
```bash
# Generate and view documentation
cargo doc --open

# Check documentation coverage
cargo doc --document-private-items
```

### Testing
```bash
# Run all tests
cargo test --all-features

# Run tests with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

---

## 15. Help Content Management (Sprint 17)

### 15.1 Help Content Architecture

Extended help content for topics like configuration and credential management is managed through a dedicated help module:

```
src/
├── help.rs                 # Help content functions
└── help/
    ├── config.txt          # Configuration help text
    └── credentials.txt     # Credential management help text
```

### 15.2 Help Module Pattern

```rust
// src/help.rs

/// Get help content for configuration
pub fn config_help() -> &'static str {
    include_str!("help/config.txt")
}

/// Get help content for credentials
pub fn credentials_help() -> &'static str {
    include_str!("help/credentials.txt")
}

/// Get general help (when no topic specified)
pub fn general_help() -> &'static str {
    "Available help topics:\n\n\
     tq help config       Configuration file format and usage\n\
     tq help credentials  Password and credential management\n"
}
```

**Design Rationale:**
- `include_str!()` embeds content at compile time (no runtime file I/O)
- Separate `.txt` files keep help content maintainable
- Content sourced from specification documents ensures consistency

### 15.3 Help Command Integration

The `Help` subcommand uses clap's `ValueEnum` for topic validation:

```rust
#[derive(Subcommand, Debug)]
pub enum Command {
    // ... existing commands
    Help(HelpArgs),
}

#[derive(Parser, Debug)]
pub struct HelpArgs {
    #[arg(value_name = "TOPIC")]
    pub topic: Option<HelpTopic>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HelpTopic {
    Config,
    Credentials,
}
```

---

## 16. Security Patterns (Sprint 17)

### 16.1 File Permission Validation Order

**CRITICAL:** When reading sensitive files (passwords, credentials), ALWAYS validate file permissions BEFORE reading file content:

```rust
// CORRECT: Validate permissions FIRST
fn read_sensitive_file(path: &Path) -> Result<String> {
    // 1. Check permissions (fail fast if insecure)
    validate_file_permissions(path)?;

    // 2. Read content only after validation passes
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

// INCORRECT: Reading before validation
fn read_sensitive_file_wrong(path: &Path) -> Result<String> {
    // BUG: Content loaded before permission check
    let content = std::fs::read_to_string(path)?;
    validate_file_permissions(path)?;  // Too late!
    Ok(content)
}
```

**Rationale:** Reading insecure files before validation exposes sensitive data in memory even when the operation ultimately fails.

### 16.2 Permission Enforcement vs Warning

For password files, the tool MUST enforce permissions rather than just warn:

```rust
// CORRECT: Return error for insecure permissions
if mode & 0o077 != 0 {
    return Err(TqError::InvalidConfig(format!(
        "Password file '{}' has insecure permissions {:04o}. Required: 0600.\n\
         Fix: chmod 0600 {}",
        path.display(), mode, path.display()
    )));
}

// INCORRECT: Warn but continue
if mode & 0o077 != 0 {
    log::warn!("Insecure permissions");  // User may not see this!
}
```

**Rationale:** Warnings can be ignored or missed. Security requirements must be enforced with hard failures.

---

## Conclusion

This architecture provides a solid foundation for implementing the `tq` CLI tool. Key design decisions:

1. **Library-first design** enables reusability and testability
2. **Trait-based abstractions** allow mocking and future extensibility
3. **Streaming by default** prevents memory issues with large datasets
4. **Secure credential handling** with secrecy and proper file permissions
5. **Clear error messages** with actionable troubleshooting steps
6. **Minimal dependencies** keep the binary small and builds fast
7. **Type-safe configuration** with serde and figment
8. **Zero-copy where possible** for optimal performance

The implementation follows Rust best practices and idioms throughout, ensuring maintainable, safe, and performant code.
