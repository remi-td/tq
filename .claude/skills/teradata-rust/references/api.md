# Teradata Rust API Reference

## Core Functions

### load_driver
```rust
pub fn load_driver(lib_dir: &str) -> Result<(), String>
```
Loads the native library. Must be called once before any connections.

**Best Practice:** Use `once_cell` for singleton:
```rust
use once_cell::sync::OnceCell;
static DRIVER_LOADED: OnceCell<()> = OnceCell::new();

fn ensure_driver_loaded(lib_dir: &str) -> Result<(), String> {
    DRIVER_LOADED.get_or_try_init(|| {
        teradatarustapi::load_driver(lib_dir)
    }).map(|_| ())
}
```

### create_connection
```rust
pub fn create_connection(params_json: &str) -> Result<(u64, u64), String>
```
Returns `(u_log, conn_handle)`.

**JSON Format:**
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

### rustgo_create_rows_wrapper
```rust
pub fn rustgo_create_rows_wrapper(
    u_log: u64,
    conn_handle: u64,
    request_text: &str,
    bind_values: &str,
) -> Result<u64, String>
```
Executes SQL. Returns `rows_handle`.

**Bind Values:**
- No params: `"null"`
- Single row: `[[123, "John"]]`
- Multiple rows: `[[123, "John"], [456, "Jane"]]`

### rustgo_fetch_row_wrapper
```rust
pub fn rustgo_fetch_row_wrapper(
    u_log: u64,
    rows_handle: u64,
) -> Result<Option<String>, String>
```
Returns `Some(json_row)` or `None` for end.

### go_close_rows_wrapper
```rust
pub fn go_close_rows_wrapper(u_log: u64, rows_handle: u64) -> Result<(), String>
```

### go_close_connection_wrapper
```rust
pub fn go_close_connection_wrapper(u_log: u64, conn_handle: u64) -> Result<(), String>
```

## Logon Mechanisms

| Value | Description |
|-------|-------------|
| `TD2` | Default Teradata auth |
| `LDAP` | LDAP authentication |
| `KRB5` | Kerberos authentication |
| `TDNEGO` | Negotiated authentication |

## Data Type Mapping

| Teradata Type | JSON | Rust Conversion |
|--------------|------|-----------------|
| INTEGER | Number | `value.as_i64()` |
| DECIMAL | String/Number | Parse string |
| VARCHAR | String | `value.as_str()` |
| DATE/TIME | String | Parse with chrono |
| BOOLEAN | Boolean | `value.as_bool()` |
