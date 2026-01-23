# Teradata Rust Patterns

## Connection Management Pattern

```rust
pub struct DatabaseClient {
    config: ConnectionConfig,
    driver_lib_dir: String,
}

impl DatabaseClient {
    pub fn execute_query(&self, sql: &str) -> Result<(), String> {
        // 1. Ensure driver loaded (once per process)
        ensure_driver_loaded(&self.driver_lib_dir)?;

        // 2. Create connection
        let (u_log, conn_handle) = teradatarustapi::create_connection(
            &self.config.to_json_string()
        )?;

        // 3. Execute (with cleanup on error)
        let result = self.execute_internal(u_log, conn_handle, sql);

        // 4. ALWAYS close connection
        let _ = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle);

        result
    }
}
```

## Connection String Parsing

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
        // Parse: user:password@host:port/database
        let parts: Vec<&str> = logon.split('@').collect();
        // ... parsing logic
    }

    pub fn to_json_string(&self) -> String {
        format!(
            r#"{{"host":"{}","user":"{}","password":"{}","dbs_port":"{}","database":"{}","logmech":"{}"}}"#,
            self.host, self.user, self.password, self.port, self.database, self.logmech
        )
    }
}
```

## Result Processing

```rust
fn process_results(u_log: u64, rows_handle: u64) -> Result<Vec<Vec<Value>>, String> {
    let mut results = Vec::new();

    while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)? {
        let values: Vec<Value> = serde_json::from_str(&row_json)?;
        results.push(values);
    }

    Ok(results)
}
```

## Build Script for Library Bundling

Create `build.rs` to copy native library:

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let lib_name = if cfg!(target_os = "macos") {
        "teradatasql.dylib"
    } else if cfg!(target_os = "windows") {
        "teradatasql.dll"
    } else {
        "teradatasql.so"
    };

    // Search cargo git checkouts for library
    // Copy to target directory
    // Set TERADATA_LIB_DIR env var
}
```
