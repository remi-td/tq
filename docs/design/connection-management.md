# Connection Management Design

This document explains how tq manages connections to Teradata databases.

**Related Specification**: `docs/specifications/configuration.md` (credentials section)

## Overview

tq follows a **simple one-shot connection model**: establish connection → execute command → close connection. The exception is REPL mode, which maintains a persistent session for interactive use.

## Architecture

```
Connection Lifecycle:

┌─────────────────┐
│ Parse CLI Args  │
└────────┬────────┘
         │
┌────────▼────────────┐
│ Build Config        │
│ - Host/port/db      │
│ - Authentication    │
│ - Credentials       │
└────────┬────────────┘
         │
┌────────▼────────────┐
│ Validate Config     │
│ - Required fields   │
│ - Format checks     │
└────────┬────────────┘
         │
┌────────▼────────────┐
│ Resolve Credentials │
│ - Password file     │
│ - Environment       │
│ - Interactive       │
└────────┬────────────┘
         │
┌────────▼────────────┐
│ Establish           │
│ Connection          │
│ (teradatarustapi)   │
└────────┬────────────┘
         │
┌────────▼────────────┐
│ Execute Command     │
└────────┬────────────┘
         │
┌────────▼────────────┐
│ Close Connection    │
└─────────────────────┘
```

## Module Structure

```
src/
├── db/
│   ├── mod.rs
│   ├── connection.rs    # Connection wrapper
│   ├── query.rs         # Query execution
│   └── types.rs         # Data type mapping
├── config.rs            # Configuration, credential resolution
└── utils/
    └── connection_string.rs  # Connection string parsing
```

## Connection Configuration

### ConnectionConfig Type

```rust
// src/config.rs

use secrecy::{Secret, ExposeSecret};

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
```

**Design decisions**:
- `secrecy::Secret` prevents accidental password exposure
- `skip_serializing` ensures passwords never written to logs
- Default values for optional fields
- Strongly-typed authentication mechanism

### Connection String Parsing

Format: `user:password@host:port/database`

```rust
// src/utils/connection_string.rs

impl ConnectionConfig {
    pub fn from_connection_string(s: &str) -> Result<Self> {
        // Parse format: user:password@host:port/database
        let parts = s.split('@').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(Error::InvalidConnectionString(
                "Expected format: user:password@host:port/database".into()
            ));
        }

        // Parse credentials
        let creds = parts[0].split(':').collect::<Vec<_>>();
        let (user, password) = match creds.len() {
            1 => (creds[0].to_string(), None),
            2 => (creds[0].to_string(), Some(Secret::new(creds[1].to_string()))),
            _ => return Err(Error::InvalidConnectionString("Invalid credentials format".into())),
        };

        // Parse host/port/database
        let location = parts[1];
        // ... parsing logic

        Ok(ConnectionConfig {
            user,
            password,
            host,
            port,
            database,
            logmech: LogonMechanism::TD2,
            timeout: Duration::from_secs(30),
        })
    }
}
```

**Validation points**:
- Required fields present
- Port in valid range
- Database name non-empty
- Host resolves (optional, may be slow)

### Credential Resolution

Priority order:
1. Embedded in connection string (user:password@...)
2. Password file (`--password-file` or `~/.tq_passwords`)
3. Environment variable (`TQ_PASSWORD`)
4. Interactive prompt (if TTY)

```rust
impl ConnectionConfig {
    pub fn resolve_password(&mut self, password_file: Option<&Path>) -> Result<()> {
        if self.password.is_some() {
            return Ok(());  // Already provided
        }

        // Try password file
        if let Some(path) = password_file {
            self.password = Some(read_password_from_file(path, self)?);
            return Ok(());
        }

        // Try default password file
        if let Some(home) = dirs::home_dir() {
            let default_path = home.join(".tq_passwords");
            if default_path.exists() {
                if let Ok(pw) = read_password_from_file(&default_path, self) {
                    self.password = Some(pw);
                    return Ok(());
                }
            }
        }

        // Try environment variable
        if let Ok(pw) = env::var("TQ_PASSWORD") {
            self.password = Some(Secret::new(pw));
            return Ok(());
        }

        // Interactive prompt
        if atty::is(atty::Stream::Stdin) {
            self.password = Some(prompt_password(&self.user)?);
            return Ok(());
        }

        Err(Error::MissingPassword(
            "No password provided. Use --password-file or set TQ_PASSWORD".into()
        ))
    }
}
```

## Connection Wrapper

### Connection Type

```rust
// src/db/connection.rs

use teradatarustapi::*;

pub struct Connection {
    inner: TeraConnection,
    config: ConnectionConfig,
}

impl Connection {
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

    pub fn ping(&self) -> Result<Duration> {
        let start = Instant::now();
        self.execute("SELECT 1")?;
        Ok(start.elapsed())
    }

    pub fn execute(&self, sql: &str) -> Result<QueryResult> {
        // ... query execution
    }

    pub fn execute_stream(&self, sql: &str) -> Result<QueryResultStream> {
        // ... streaming execution
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Teradata connection closed automatically by TeraConnection::drop
    }
}
```

**Design patterns**:
- Wrapper around `TeraConnection` provides higher-level API
- Config stored for error reporting
- RAII: connection closed when `Connection` dropped
- Separate buffered and streaming execution paths

### Teradata Driver Integration

Connection string format for `teradatarustapi`:

```rust
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
```

## Connection Modes

### One-Shot Mode (Default)

Used by `ping` and `query` commands:

```rust
pub fn execute(global: GlobalOpts, args: QueryArgs) -> Result<()> {
    let mut config = global.build_connection_config()?;
    config.resolve_password(global.password_file.as_deref())?;

    // Connect
    let conn = Connection::connect(config)?;

    // Execute
    let result = conn.execute(&sql)?;

    // Connection dropped here (RAII)
    Ok(())
}
```

**Characteristics**:
- Fresh connection per invocation
- No connection reuse
- Clean error recovery
- Simple resource lifecycle

### Persistent Mode (REPL)

REPL maintains connection across multiple queries:

```rust
pub fn execute(global: GlobalOpts, args: ReplArgs) -> Result<()> {
    let mut config = global.build_connection_config()?;
    config.resolve_password(global.password_file.as_deref())?;

    // Establish persistent connection
    let conn = Connection::connect(config)?;

    // Enter REPL loop
    loop {
        let input = editor.readline("> ")?;

        // Validate connection before each query
        if !is_connection_valid(&conn) {
            eprintln!("Connection lost. Reconnecting...");
            conn = Connection::connect(config.clone())?;
        }

        match execute_statement(&conn, &input) {
            Ok(result) => display_result(result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

**Characteristics**:
- Connection persists across queries
- Connection validation before each query
- Automatic reconnection on failure
- Session state maintained

## Error Handling

### Connection Errors

```rust
#[derive(Error, Debug)]
pub enum Error {
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

    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    #[error("Missing password. Use --password-file or set TQ_PASSWORD")]
    MissingPassword(String),
}
```

**User-friendly error messages**:

```rust
impl Error {
    pub fn user_message(&self) -> String {
        match self {
            Error::ConnectionFailed { host, port, .. } => {
                format!(
                    "Error: Failed to connect to {}:{}\n\n\
                     Troubleshooting:\n\
                     1. Verify hostname resolves: ping {}\n\
                     2. Check port is open: nc -zv {} {}\n\
                     3. Confirm database is running\n\
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
                     - Verify username and password\n\
                     - Check if account is locked\n\
                     - Try different logon mechanism: --logmech LDAP",
                    user, logmech
                )
            }
            _ => format!("Error: {}", self),
        }
    }
}
```

## Security Considerations

### Password Handling

**Never log passwords**:
```rust
// CORRECT: Using secrecy::Secret
let password: Secret<String> = get_password();
println!("Password: {:?}", password);  // Prints: Password: Secret([REDACTED])

// INCORRECT: Plain string
let password: String = get_password();
println!("Password: {:?}", password);  // LEAKS PASSWORD!
```

**Zero memory on drop**:
```rust
{
    let password = Secret::new("sensitive".to_string());
    // Use password
}  // Memory zeroed here
```

**Avoid command-line passwords**:
```bash
# INSECURE: Password visible in ps, shell history
tq query --password "secret123" "SELECT 1"

# SECURE: Password from file
echo "secret123" > /tmp/pw
chmod 0600 /tmp/pw
tq query --password-file /tmp/pw "SELECT 1"

# SECURE: Interactive prompt
tq query "SELECT 1"
Password: ****
```

### Connection String Sanitization

Remove passwords before logging:

```rust
pub fn sanitize_connection_string(s: &str) -> String {
    // Replace user:password@host with user:***@host
    let re = Regex::new(r"([^:]+):([^@]+)@").unwrap();
    re.replace(s, "$1:***@").to_string()
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_connection_string_parsing() {
        let config = ConnectionConfig::from_connection_string(
            "alice:secret@dbhost:1025/mydb"
        ).unwrap();

        assert_eq!(config.user, "alice");
        assert_eq!(config.host, "dbhost");
        assert_eq!(config.port, 1025);
        assert_eq!(config.database, "mydb");
    }

    #[test]
    fn test_invalid_connection_string() {
        let result = ConnectionConfig::from_connection_string("invalid");
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_real_connection() -> Result<()> {
    let config = load_test_config()?;
    let conn = Connection::connect(config)?;
    let duration = conn.ping()?;
    assert!(duration < Duration::from_secs(5));
    Ok(())
}
```

### Mock Testing

```rust
#[cfg_attr(test, automock)]
pub trait DatabaseClient {
    fn connect(&self, config: &ConnectionConfig) -> Result<Connection>;
    fn execute(&self, sql: &str) -> Result<QueryResult>;
}
```

## Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Connection config | `src/config.rs` | `ConnectionConfig`, `resolve_password()` |
| Connection wrapper | `src/db/connection.rs` | `Connection::connect()`, `ping()`, `execute()` |
| Connection string | `src/utils/connection_string.rs` | `parse()`, `format_connection_string()` |
| Credential prompt | `src/utils/prompt.rs` | `prompt_password()` |
| Password file | `src/utils/password_file.rs` | `read_password_from_file()` |

## Performance Considerations

### Connection Overhead

- Connection establishment: ~100-500ms depending on network
- SSL/TLS handshake: +50-200ms
- Authentication: +50-100ms

**Optimization**: REPL mode amortizes overhead across multiple queries.

### Connection Validation

Check connection health before query:

```rust
fn is_connection_valid(conn: &Connection) -> bool {
    conn.execute("SELECT 1").is_ok()
}
```

**Trade-off**: Extra round-trip vs catching stale connections early.

## Driver Library Resolution

### Problem

The `build.rs` script sets the `TERADATA_LIB_DIR` compile-time environment variable to the absolute path of the target directory on the build machine (e.g., `/Users/runner/work/tq/tq/target/aarch64-apple-darwin/release`). At runtime, `client.rs` uses `option_env!("TERADATA_LIB_DIR")` to read this baked-in value. On end-user machines, this path does not exist, causing the driver to fail to load.

### Solution: Runtime Fallback Chain

Replace the compile-time `option_env!("TERADATA_LIB_DIR")` with a runtime resolution function that searches multiple locations in priority order:

```
Fallback chain (first match wins):
1. --driver-lib-dir CLI flag (explicit user override)
2. TERADATA_LIB_DIR environment variable (runtime, not compile-time)
3. Executable's directory (std::env::current_exe() parent)
4. Current working directory (".")
```

#### Implementation in `src/db/client.rs`

```rust
/// Resolve the directory containing the Teradata driver library.
///
/// Searches in priority order:
/// 1. Explicit CLI flag (--driver-lib-dir)
/// 2. TERADATA_LIB_DIR environment variable
/// 3. Directory containing the tq executable
/// 4. Current working directory
fn resolve_driver_lib_dir(cli_override: Option<String>) -> String {
    // 1. CLI flag takes highest priority
    if let Some(dir) = cli_override {
        return dir;
    }

    // 2. Runtime environment variable
    if let Ok(dir) = std::env::var("TERADATA_LIB_DIR") {
        if !dir.is_empty() {
            return dir;
        }
    }

    // 3. Executable's directory (primary for installed binaries)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.to_string_lossy().to_string();
        }
    }

    // 4. Current working directory (last resort)
    ".".to_string()
}
```

The `DatabaseClient::new()` method changes from:

```rust
let default_dir = option_env!("TERADATA_LIB_DIR").unwrap_or(".");
let driver_lib_dir = driver_lib_dir.unwrap_or_else(|| default_dir.to_string());
```

To:

```rust
let driver_lib_dir = resolve_driver_lib_dir(driver_lib_dir);
```

#### Error Message Enhancement

When the driver fails to load, the `DriverLoad` error variant's `user_message()` in `src/error.rs` should be updated to show all searched paths:

```rust
TqError::DriverLoad { path, message } => {
    format!(
        "Error: Failed to load Teradata driver\n\n\
         Path: {}\n\
         Cause: {}\n\n\
         Troubleshooting:\n  \
         - Ensure the teradatasql library is in the same directory as the tq binary\n  \
         - Override with: --driver-lib-dir /path/to/lib\n  \
         - Or set: TERADATA_LIB_DIR=/path/to/lib\n  \
         - Verify library file permissions",
        path, message
    )
}
```

Additionally, a new `TqError::DriverNotFound` variant provides richer diagnostics when the library file is not found at all (as opposed to being found but failing to load):

```rust
/// Driver library not found in any searched location
#[error("Teradata driver library not found")]
DriverNotFound { searched_paths: Vec<String> },
```

With a `user_message()` implementation that lists all searched paths.

#### Changes to `build.rs`

The `build.rs` script should **stop** setting `TERADATA_LIB_DIR` via `cargo:rustc-env`. The library copy to the target directory is still useful for local development, but the compile-time env var is harmful for distribution.

The line to remove:

```rust
// REMOVE: This bakes the CI runner's absolute path into the binary
println!("cargo:rustc-env=TERADATA_LIB_DIR={}", target_dir.display());
```

The rest of `build.rs` (finding the library in the cargo cache and copying it to the target directory) remains useful for development builds.

#### Platform Considerations

`std::env::current_exe()` is reliable on all supported platforms (macOS, Linux, Windows). Edge cases:
- **Linux**: May follow `/proc/self/exe` symlink. This is correct behavior.
- **macOS**: Returns the real executable path. Works with both direct execution and symlinks.
- **Deleted executables**: On Linux, if the binary is deleted while running, `current_exe()` may return an error. The fallback chain handles this gracefully.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| Remove compile-time env var | `build.rs:77` | Remove `cargo:rustc-env=TERADATA_LIB_DIR` line |
| Add resolve function | `src/db/client.rs` | New `resolve_driver_lib_dir()` function |
| Update client constructor | `src/db/client.rs:50-53` | Use `resolve_driver_lib_dir()` |
| Enhanced error message | `src/error.rs` | Update `DriverLoad` user message |
| Optional: DriverNotFound | `src/error.rs` | New variant with searched paths list |

## Profile Delete Confirmation

### Design

The `tq profile delete` command currently requires `--force` for all deletions. The improved behavior adds TTY-interactive confirmation:

- **TTY mode**: Prompt `Delete profile 'name'? [y/N]` and wait for input
- **Non-TTY mode**: Require `--force` flag (current behavior preserved)
- **`--force` flag**: Bypass confirmation in all modes

#### TTY Detection

Uses `std::io::IsTerminal` (stable since Rust 1.70, available in our Rust 1.94 toolchain):

```rust
use std::io::IsTerminal;

fn handle_delete(name: &str, force: bool) -> Result<()> {
    if !force {
        if std::io::stdin().is_terminal() {
            // Interactive mode: prompt for confirmation
            eprint!("Delete profile '{}'? [y/N] ", name);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .map_err(|e| TqError::IoError(e))?;
            if !input.trim().eq_ignore_ascii_case("y") {
                eprintln!("Aborted.");
                return Ok(());
            }
        } else {
            // Non-TTY: require --force
            return Err(TqError::InvalidConfig(format!(
                "Deleting profile '{}' requires --force (or run interactively)",
                name
            )));
        }
    }
    // ... proceed with deletion
}
```

#### Testing Consideration

Unit tests for `handle_delete` run in non-TTY mode (piped stdin), so the existing test behavior is preserved. The `--force` path remains the primary mechanism for programmatic and test use.

### Code Linkage

| Change | File | Description |
|--------|------|-------------|
| TTY detection + prompt | `src/commands/profile.rs` | Update `handle_delete()` |
| No new dependencies | - | `std::io::IsTerminal` is in std |

## Future Enhancements

- **Connection pooling**: For multi-threaded use cases
- **SSL/TLS configuration**: Certificate pinning, custom CAs
- **Connection retry logic**: Exponential backoff for transient failures
- **Health check customization**: User-defined validation queries
- **Session variable setting**: Execute SQL on connection
- **Connection logging**: Detailed diagnostics for troubleshooting
