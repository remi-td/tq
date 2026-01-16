---
name: teradata-rust
description: Guides writing Rust code for Teradata database interactions using the teradatarustapi crate's low-level C-style API. Use when implementing Teradata connections, executing queries, or working with the native driver functions.
---

# Teradata Rust Developer

Expert guidance for working with Teradata databases in Rust using the teradatarustapi crate.

## When to Use

- Implementing Teradata database connections in Rust
- Executing SQL queries against Teradata
- Setting up connection parameters and logon mechanisms
- Debugging Teradata connection or query issues
- Building CLI tools or applications that connect to Teradata

## Critical Architecture Information

### API Style: Low-Level C Functions

**The teradatarustapi uses C-style functions at the crate level, NOT an object-oriented API.**

There is no `Connection` struct, no `cursor()` method, and no high-level abstractions. All operations use direct function calls that wrap the Teradata GoSQL Driver.

### Native Library Bundling

The teradatarustapi crate includes pre-compiled native libraries in its git repository:
- macOS: `teradatasql.dylib`
- Linux: `teradatasql.so` (with architecture variants)
- Windows: `teradatasql.dll`

When used as a git dependency, these libraries are in `~/.cargo/git/checkouts/` and must be copied to your build output or project directory.

## Core Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
teradatarustapi = { git = "https://github.com/Teradata/teradatarustapi" }
once_cell = "1.19"  # For singleton driver loading
serde_json = "1.0"  # Connection params are JSON
```

## Automatic Library Bundling

Create `build.rs` to automatically copy the native library:

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(&out_dir)
        .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
        .unwrap().to_path_buf();

    // Platform-specific library name
    let lib_name = if cfg!(target_os = "macos") {
        "teradatasql.dylib"
    } else if cfg!(target_os = "windows") {
        "teradatasql.dll"
    } else {
        "teradatasql.so"
    };

    // Find teradatarustapi checkout in cargo cache
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap();
    let cargo_home = env::var("CARGO_HOME")
        .unwrap_or_else(|_| format!("{}/.cargo", home));
    let git_checkouts = PathBuf::from(cargo_home).join("git/checkouts");

    // Search for the library
    if let Ok(entries) = fs::read_dir(&git_checkouts) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() && path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with("teradatarustapi-"))
                .unwrap_or(false)
            {
                if let Ok(subdirs) = fs::read_dir(&path) {
                    for subdir in subdirs.filter_map(Result::ok) {
                        let lib_source = subdir.path().join(lib_name);
                        if lib_source.exists() {
                            let lib_dest = target_dir.join(lib_name);
                            if fs::copy(&lib_source, &lib_dest).is_ok() {
                                // Embed path at compile time
                                println!("cargo:rustc-env=TERADATA_LIB_DIR={}",
                                    target_dir.display());
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("cargo:warning=Could not find teradatasql library");
}
```

## Core API Functions

### Function: load_driver

```rust
pub fn load_driver(lib_dir: &str) -> Result<(), String>
```

Loads the Teradata native library. Must be called once before any connections.

**Parameters:**
- `lib_dir`: Directory containing the platform-specific library file

**Best Practice:** Use `once_cell` to ensure single initialization:

```rust
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static DRIVER_LOADED: OnceCell<Mutex<String>> = OnceCell::new();

fn ensure_driver_loaded(lib_dir: &str) -> Result<(), String> {
    DRIVER_LOADED.get_or_try_init(|| {
        teradatarustapi::load_driver(lib_dir)?;
        Ok(Mutex::new(lib_dir.to_string()))
    })?;
    Ok(())
}
```

### Function: create_connection

```rust
pub fn create_connection(params_json: &str) -> Result<(u64, u64), String>
```

Creates a database connection. Returns a tuple of `(u_log, conn_handle)`.

**Parameters:**
- `params_json`: JSON string with connection parameters

**Returns:**
- `u_log`: Logging handle (used in all subsequent operations)
- `conn_handle`: Connection handle (used for queries and closing)

**JSON Parameter Format:**
```json
{
  "host": "hostname",
  "user": "username",
  "password": "password",
  "dbs_port": "1025",
  "database": "dbname",
  "logmech": "TD2"
}
```

**Example:**
```rust
let params = format!(
    r#"{{"host":"{}","user":"{}","password":"{}","dbs_port":"{}","database":"{}","logmech":"{}"}}"#,
    host, user, password, port, database, logmech
);

let (u_log, conn_handle) = teradatarustapi::create_connection(&params)
    .map_err(|e| format!("Connection failed: {}", e))?;
```

### Function: rustgo_create_rows_wrapper

```rust
pub fn rustgo_create_rows_wrapper(
    u_log: u64,
    conn_handle: u64,
    request_text: &str,
    bind_values: &str,
) -> Result<u64, String>
```

Executes a SQL query and creates a result set.

**Parameters:**
- `u_log`: Logging handle from create_connection
- `conn_handle`: Connection handle from create_connection
- `request_text`: SQL query string
- `bind_values`: JSON string of bind parameters, or `"null"` for none

**Returns:**
- `rows_handle`: Handle to the result set

**Example:**
```rust
let query = "SELECT * FROM users WHERE id = ?";
let bind_values = r#"[[123]]"#;  // Bind parameters as JSON array

let rows_handle = teradatarustapi::rustgo_create_rows_wrapper(
    u_log,
    conn_handle,
    query,
    bind_values
)?;
```

### Function: rustgo_fetch_row_wrapper

```rust
pub fn rustgo_fetch_row_wrapper(
    u_log: u64,
    rows_handle: u64,
) -> Result<Option<String>, String>
```

Fetches the next row from a result set.

**Parameters:**
- `u_log`: Logging handle
- `rows_handle`: Result set handle from rustgo_create_rows_wrapper

**Returns:**
- `Some(String)`: JSON array of column values, e.g. `["value1", "value2"]`
- `None`: No more rows

**Example:**
```rust
while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)? {
    let values: Vec<serde_json::Value> = serde_json::from_str(&row_json)?;
    // Process row values
}
```

### Function: go_close_rows_wrapper

```rust
pub fn go_close_rows_wrapper(
    u_log: u64,
    rows_handle: u64,
) -> Result<(), String>
```

Closes a result set and releases resources.

### Function: go_close_connection_wrapper

```rust
pub fn go_close_connection_wrapper(
    u_log: u64,
    conn_handle: u64,
) -> Result<(), String>
```

Closes a database connection.

## Connection Management Pattern

### Complete Connection Lifecycle

```rust
use teradatarustapi;

pub struct DatabaseClient {
    config: ConnectionConfig,
    driver_lib_dir: String,
}

impl DatabaseClient {
    pub fn new(config: ConnectionConfig, driver_lib_dir: Option<String>) -> Self {
        let default_dir = option_env!("TERADATA_LIB_DIR").unwrap_or(".");
        Self {
            config,
            driver_lib_dir: driver_lib_dir.unwrap_or_else(|| default_dir.to_string()),
        }
    }

    pub fn execute_query(&self, sql: &str) -> Result<(), String> {
        // 1. Ensure driver is loaded (once per process)
        ensure_driver_loaded(&self.driver_lib_dir)?;

        // 2. Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) = teradatarustapi::create_connection(&connection_string)?;

        // 3. Execute query (with cleanup on error)
        let result = self.execute_query_internal(u_log, conn_handle, sql);

        // 4. Always close connection
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            eprintln!("Warning: Failed to close connection: {}", e);
        }

        result
    }

    fn execute_query_internal(
        &self,
        u_log: u64,
        conn_handle: u64,
        sql: &str,
    ) -> Result<(), String> {
        // Create result set
        let rows_handle = teradatarustapi::rustgo_create_rows_wrapper(
            u_log,
            conn_handle,
            sql,
            "null", // No bind parameters
        )?;

        // Fetch rows
        while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)? {
            println!("Row: {}", row_json);
        }

        // Close result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)?;

        Ok(())
    }
}
```

## Connection String Parsing

Parse `user:password@host:port/database` format:

```rust
pub struct ConnectionConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub logmech: String,
}

impl ConnectionConfig {
    pub fn parse(logon: &str, logmech: &str) -> Result<Self, String> {
        let parts: Vec<&str> = logon.split('@').collect();
        if parts.len() != 2 {
            return Err("Expected format: user:password@host:port/database".into());
        }

        let credentials = parts[0];
        let host_info = parts[1];

        let cred_parts: Vec<&str> = credentials.split(':').collect();
        if cred_parts.len() != 2 {
            return Err("Credentials must be user:password".into());
        }

        let host_db: Vec<&str> = host_info.split('/').collect();
        if host_db.len() != 2 {
            return Err("Host must include database: host:port/database".into());
        }

        let host_port: Vec<&str> = host_db[0].split(':').collect();
        if host_port.len() != 2 {
            return Err("Host must include port: host:port".into());
        }

        Ok(ConnectionConfig {
            user: cred_parts[0].to_string(),
            password: cred_parts[1].to_string(),
            host: host_port[0].to_string(),
            port: host_port[1].parse().map_err(|_| "Invalid port")?,
            database: host_db[1].to_string(),
            logmech: logmech.to_string(),
        })
    }

    pub fn to_json_string(&self) -> String {
        format!(
            r#"{{"host":"{}","user":"{}","password":"{}","dbs_port":"{}","database":"{}","logmech":"{}"}}"#,
            self.host, self.user, self.password, self.port, self.database, self.logmech
        )
    }
}
```

## Logon Mechanisms

Supported values for `logmech` parameter:
- **TD2**: Default Teradata authentication (username/password)
- **LDAP**: LDAP authentication
- **KRB5**: Kerberos authentication
- **TDNEGO**: Negotiated authentication

Specify in JSON connection string: `"logmech": "TD2"`

## Error Handling Best Practices

### Connection Error Messages

Parse and enhance error messages:

```rust
fn parse_connection_error(error: &str) -> String {
    if error.contains("Connection refused") {
        format!("Connection refused. Ensure database is running. Error: {}", error)
    } else if error.contains("timeout") {
        format!("Connection timeout. Check network connectivity. Error: {}", error)
    } else if error.contains("Invalid credentials") || error.contains("Logon failed") {
        format!("Authentication failed. Verify credentials. Error: {}", error)
    } else {
        format!("Connection failed: {}", error)
    }
}
```

### Resource Cleanup

Always close connections, even on error:

```rust
pub fn ping(&self) -> Result<(), String> {
    let (u_log, conn_handle) = teradatarustapi::create_connection(&params)?;

    // Execute with error handling
    let result = execute_ping_query(u_log, conn_handle);

    // ALWAYS close connection
    if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
        eprintln!("Warning: Failed to close connection: {}", e);
    }

    result
}
```

## Parameterized Queries

Bind parameters are passed as JSON arrays:

```rust
// Single row of parameters
let bind_values = r#"[[123, "John"]]"#;

// Multiple rows (for batch operations)
let bind_values = r#"[[123, "John"], [456, "Jane"]]"#;

// No parameters
let bind_values = "null";

let rows_handle = teradatarustapi::rustgo_create_rows_wrapper(
    u_log,
    conn_handle,
    "INSERT INTO users (id, name) VALUES (?, ?)",
    bind_values,
)?;
```

## Result Processing

Parse JSON row results:

```rust
use serde_json::Value;

fn process_results(u_log: u64, rows_handle: u64) -> Result<Vec<Vec<Value>>, String> {
    let mut results = Vec::new();

    while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)? {
        let values: Vec<Value> = serde_json::from_str(&row_json)
            .map_err(|e| format!("Failed to parse row: {}", e))?;
        results.push(values);
    }

    Ok(results)
}
```

## CLI Application Pattern

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, required = true)]
    logon: String,

    #[arg(long, default_value = "TD2")]
    logmech: String,

    #[arg(long)]
    ping: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = ConnectionConfig::parse(&cli.logon, &cli.logmech)?;
    let client = DatabaseClient::new(config, None);

    if cli.ping {
        client.ping()?;
        println!("Success! Database is reachable.");
    }

    Ok(())
}
```

## Testing Strategy

### Unit Tests

Test parsing and configuration logic without database:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_parse() {
        let config = ConnectionConfig::parse(
            "user:pass@host:1025/db",
            "TD2"
        ).unwrap();

        assert_eq!(config.user, "user");
        assert_eq!(config.host, "host");
        assert_eq!(config.port, 1025);
    }

    #[test]
    fn test_json_string_format() {
        let config = ConnectionConfig {
            user: "test".into(),
            password: "pass".into(),
            host: "localhost".into(),
            port: 1025,
            database: "db".into(),
            logmech: "TD2".into(),
        };

        let json = config.to_json_string();
        assert!(json.contains(r#""host":"localhost""#));
        assert!(json.contains(r#""user":"test""#));
    }
}
```

### Integration Tests

Test actual database connections (requires real database):

```rust
#[cfg(test)]
mod integration_tests {
    #[test]
    #[ignore]  // Run with: cargo test -- --ignored
    fn test_real_connection() {
        let logon = std::env::var("TEST_TD_LOGON")
            .expect("Set TEST_TD_LOGON for integration tests");

        let config = ConnectionConfig::parse(&logon, "TD2").unwrap();
        let client = DatabaseClient::new(config, None);

        assert!(client.ping().is_ok());
    }
}
```

## Common Teradata Data Types

Results are returned as JSON, so type conversion is manual:

| Teradata Type | JSON Representation | Rust Conversion |
|--------------|---------------------|-----------------|
| INTEGER, SMALLINT | Number | `value.as_i64()` |
| BIGINT | Number | `value.as_i64()` |
| DECIMAL, NUMERIC | String or Number | `value.as_f64()` or parse string |
| FLOAT, DOUBLE | Number | `value.as_f64()` |
| CHAR, VARCHAR | String | `value.as_str()` |
| DATE, TIME | String | Parse with chrono |
| BOOLEAN | Boolean | `value.as_bool()` |

## Security Best Practices

1. **Never log connection strings** - they contain passwords
2. **Use environment variables** for credentials:
   ```rust
   let logon = env::var("TD_LOGON").expect("TD_LOGON not set");
   ```
3. **Use parameterized queries** to prevent SQL injection
4. **Close connections** to avoid resource leaks
5. **Don't commit credentials** to version control

## Debugging Tips

### Enable verbose output

```rust
eprintln!("Attempting connection to: {}", host);
let (u_log, conn_handle) = create_connection(&params)?;
eprintln!("Connected successfully, handles: {}, {}", u_log, conn_handle);
```

### Verify library loading

```bash
# Check if library is present
ls -la target/debug/teradatasql.*
ls -la target/release/teradatasql.*

# Run with library search debugging (macOS)
DYLD_PRINT_LIBRARIES=1 ./target/debug/your-app

# Linux
LD_DEBUG=libs ./target/debug/your-app
```

## Common Pitfalls

1. **Forgetting to load driver** - Call `load_driver()` before connections
2. **Not closing connections** - Always close to prevent leaks
3. **Incorrect JSON format** - Connection params must be valid JSON
4. **Missing native library** - Ensure build script copies the library
5. **Using wrong parameter format** - Bind values must be JSON arrays
6. **Not handling connection errors** - Network issues are common

## Performance Considerations

For CLI tools with one-shot operations:
- Load driver once per process (use `once_cell`)
- Create connection, execute, close (no pooling needed)
- Close connections immediately after use

For long-running applications:
- Implement connection pooling manually if needed
- Reuse connections for multiple queries
- Monitor connection health

## Additional Resources

- teradatarustapi GitHub: https://github.com/Teradata/teradatarustapi
- Teradata SQL Reference: For SQL syntax and features
- libloading crate: Used internally for dynamic library loading

## Guidelines

- Always load the driver before creating connections
- Use `once_cell` pattern for single driver initialization
- Always close connections, even on errors
- Use JSON format for all connection parameters and bind values
- Never log passwords or connection strings
- Test connection string parsing independently
- Handle connection failures with clear error messages
- Copy native library automatically with build script
