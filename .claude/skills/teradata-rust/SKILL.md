---
name: teradata-rust
description: Guides writing Rust code for Teradata using teradatarustapi. Use when implementing Teradata connections, executing queries, or debugging database issues.
---

# Teradata Rust Developer

Expert guidance for Teradata databases in Rust using teradatarustapi.

## Critical Architecture

**This crate uses C-style functions, NOT object-oriented API.**

No `Connection` struct, no `cursor()` method. All operations use direct function calls.

## Quick Start

### Dependencies
```toml
[dependencies]
teradatarustapi = { git = "https://github.com/Teradata/teradatarustapi" }
once_cell = "1.19"
serde_json = "1.0"
```

### Basic Usage
```rust
// 1. Load driver (once per process)
teradatarustapi::load_driver("./lib")?;

// 2. Connect
let params = r#"{"host":"...", "user":"...", "password":"...", "dbs_port":"1025", "database":"...", "logmech":"TD2"}"#;
let (u_log, conn_handle) = teradatarustapi::create_connection(params)?;

// 3. Execute query
let rows_handle = teradatarustapi::rustgo_create_rows_wrapper(
    u_log, conn_handle, "SELECT * FROM users", "null"
)?;

// 4. Fetch rows
while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)? {
    // row_json is JSON array: ["value1", "value2"]
}

// 5. Cleanup (ALWAYS)
teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)?;
teradatarustapi::go_close_connection_wrapper(u_log, conn_handle)?;
```

## Key Points

| Topic | Rule |
|-------|------|
| Driver | Load once per process with `once_cell` |
| Connections | Always close, even on errors |
| Parameters | JSON format: `{"host":"...", ...}` |
| Bind values | JSON arrays: `[[1, "a"]]` or `"null"` |
| Results | JSON arrays per row |

## Logon Mechanisms

| Value | Description |
|-------|-------------|
| `TD2` | Default Teradata auth |
| `LDAP` | LDAP authentication |
| `KRB5` | Kerberos |

## Detailed References

- **[API Reference](references/api.md)**: All functions and data types
- **[Patterns](references/patterns.md)**: Connection management, parsing, build.rs
- **[Troubleshooting](references/troubleshooting.md)**: Errors, debugging, security

## Guidelines

- Always load driver before connections
- Use `once_cell` for single driver initialization
- Always close connections, even on errors
- Never log passwords or connection strings
- Use parameterized queries to prevent SQL injection

## Teradata System Object Verification

**CRITICAL: Never invent or assume Teradata system tables, views, or table functions exist.** Teradata's internal API surface is not predictable by analogy. Always verify against the live database before implementing.

**Confirmed working table functions:**
- `MonitorSession(-1, '*', 0)` — Active sessions
- `MonitorAbortSession(session_id)` — Abort a session
- `MonitorCancelRequest(session_id)` — Cancel running query

**Confirmed NOT existing:**
- `MonitorSetResource` — Does NOT exist (priority is TASM/TDWM)

**Dependency analysis (from Issue #33):**
- Use `DBC.TVM.CreateText` + `DBC.TextTbl` (TextType='C') for dependency analysis
- Search for fully-qualified `"DB"."Object"` patterns in CreateText
- Do NOT use `DBC.ViewTextV` for dependency analysis — unreliable for qualified references

**DDL retrieval:**
- `SHOW VIEW "db"."name"` returns DDL as fixed-width VARCHAR chunks
- Chunks are split at arbitrary character boundaries — concatenate directly WITHOUT newlines
- `DBC.TableSizeV` may not exist on all systems — always use with graceful fallback

**Testing rule:** Before committing any command that uses Teradata system SQL, test the raw query:
```bash
echo "<sql>" | cargo run -- query --format csv
```
