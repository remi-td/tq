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

## Future Enhancements

- **Connection pooling**: For multi-threaded use cases
- **SSL/TLS configuration**: Certificate pinning, custom CAs
- **Connection retry logic**: Exponential backoff for transient failures
- **Health check customization**: User-defined validation queries
- **Session variable setting**: Execute SQL on connection
- **Connection logging**: Detailed diagnostics for troubleshooting
