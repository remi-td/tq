# Bug Analysis and Fix Guide - Interactive Mode MVP

**Generated**: 2026-01-17
**Commit**: dcc692c8b249f006f7796ba41a4a846f24f744d8

---

## Critical Bug #1: Column Names Display as "col1", "col2"

### Problem Statement

All query results show generic column names ("col1", "col2", "col3", etc.) instead of the actual column names from the SQL query or table schema.

### Impact

- **Severity**: CRITICAL
- **Affects**: 100% of query results in all modes (REPL, batch, all formats)
- **User Impact**:
  - CSV files have wrong headers
  - JSON objects have wrong keys
  - Table output is confusing and unusable
  - Automated scripts parsing output will break

### Root Cause

**File**: `/Users/remi.turpaud/Code/genAI/tq/src/db/client.rs`
**Lines**: 307-331

The `infer_columns()` function generates synthetic column names because it only processes row data, not column metadata:

```rust
/// Infer column metadata from JSON values
///
/// Since teradatarustapi returns JSON, we infer types from values
fn infer_columns(&self, values: &[serde_json::Value]) -> Vec<ColumnMetadata> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let data_type = match v {
                serde_json::Value::Null => TeradataType::Unknown,
                serde_json::Value::Bool(_) => TeradataType::Boolean,
                serde_json::Value::Number(n) => {
                    if n.is_i64() {
                        TeradataType::Integer
                    } else {
                        TeradataType::Decimal
                    }
                }
                serde_json::Value::String(_) => TeradataType::Varchar,
                _ => TeradataType::Unknown,
            };
            ColumnMetadata::new(format!("col{}", i + 1), data_type, true)  // ← BUG IS HERE
            //                           ^^^^^^^^^^^^^^
            //                           Generates "col1", "col2", etc.
        })
        .collect()
}
```

### Why This Happened

The initial implementation assumed that column metadata wasn't available from the `teradatarustapi` crate and attempted to infer it from the row data. However, the API **does provide** column metadata through the `rustgo_result_metadata_wrapper()` function.

### The Solution

The `teradatarustapi` crate provides a function that returns column metadata:

```rust
/// Returns: (activity_count, activity_type, activity_name, column_metadata_json)
pub fn rustgo_result_metadata_wrapper(
    u_log: u64,
    rows_handle: u64,
) -> Result<(u64, u16, String, String), String>
```

The 4th element in the tuple is `column_metadata` - a JSON string containing the actual column names and types.

### Fix Implementation

#### Step 1: Call metadata function after creating rows

**File**: `/Users/remi.turpaud/Code/genAI/tq/src/db/client.rs`

In `execute_and_fetch()` and related functions, after calling `rustgo_create_rows_wrapper()`:

```rust
// Current code (around line 185):
let rows_handle =
    teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, sql, bind_values)
        .map_err(|e| self.map_query_error(&e, sql))?;

// ADD IMMEDIATELY AFTER:
// Get column metadata
let (_, _, _, column_metadata_json) =
    teradatarustapi::rustgo_result_metadata_wrapper(u_log, rows_handle)
        .map_err(|e| TqError::MetadataFetch(e.to_string()))?;

log::debug!("Column metadata: {}", column_metadata_json);
```

#### Step 2: Parse the column metadata JSON

The `column_metadata_json` string contains an array of objects with this structure (example):

```json
[
  {
    "Name": "test_col",
    "Type": "INTEGER",
    "Length": 4,
    "Precision": 10,
    "Scale": 0,
    "Nullable": true
  },
  {
    "Name": "name_col",
    "Type": "VARCHAR",
    "Length": 100,
    "Precision": 0,
    "Scale": 0,
    "Nullable": true
  }
]
```

Add a new function to parse this:

```rust
/// Parse column metadata JSON from teradatarustapi
fn parse_column_metadata(&self, metadata_json: &str) -> Result<Vec<ColumnMetadata>> {
    #[derive(serde::Deserialize)]
    struct MetadataColumn {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Type")]
        type_name: String,
        #[serde(rename = "Nullable")]
        nullable: bool,
    }

    let metadata: Vec<MetadataColumn> = serde_json::from_str(metadata_json)
        .map_err(|e| TqError::MetadataParsing {
            message: format!("Failed to parse column metadata: {}", e),
        })?;

    Ok(metadata
        .into_iter()
        .map(|col| {
            let data_type = map_type_name_to_teradata_type(&col.type_name);
            ColumnMetadata::new(col.name, data_type, col.nullable)
        })
        .collect())
}

/// Map Teradata type name string to TeradataType enum
fn map_type_name_to_teradata_type(type_name: &str) -> TeradataType {
    match type_name.to_uppercase().as_str() {
        "INTEGER" | "INT" => TeradataType::Integer,
        "BIGINT" => TeradataType::BigInt,
        "SMALLINT" => TeradataType::SmallInt,
        "DECIMAL" | "NUMERIC" => TeradataType::Decimal,
        "FLOAT" | "DOUBLE" | "REAL" => TeradataType::Float,
        "CHAR" | "CHARACTER" => TeradataType::Char,
        "VARCHAR" | "CHARACTER VARYING" => TeradataType::Varchar,
        "DATE" => TeradataType::Date,
        "TIME" => TeradataType::Time,
        "TIMESTAMP" => TeradataType::Timestamp,
        "BOOLEAN" | "BOOL" => TeradataType::Boolean,
        "BLOB" | "BINARY LARGE OBJECT" => TeradataType::Blob,
        "CLOB" | "CHARACTER LARGE OBJECT" => TeradataType::Clob,
        _ => TeradataType::Unknown,
    }
}
```

#### Step 3: Use parsed metadata instead of inferred columns

Modify `fetch_all_rows()` and `fetch_rows_limited()`:

```rust
// OLD CODE:
// Extract column metadata from first row
if columns.is_none() {
    columns = Some(self.infer_columns(&values));  // ← Remove this
}

// NEW CODE:
// Columns already extracted from metadata, just validate count
if columns.is_none() {
    return Err(TqError::InternalError(
        "Column metadata should have been set before fetching rows".into()
    ));
}

// Validate column count matches data
if values.len() != columns.as_ref().unwrap().len() {
    return Err(TqError::ColumnCountMismatch {
        expected: columns.as_ref().unwrap().len(),
        actual: values.len(),
    });
}
```

#### Step 4: Update function signatures

Change `fetch_all_rows()` and `fetch_rows_limited()` to accept columns as a parameter:

```rust
fn fetch_all_rows(
    &self,
    u_log: u64,
    rows_handle: u64,
    columns: Vec<ColumnMetadata>,  // ← Pass columns in
) -> Result<(Vec<ColumnMetadata>, Vec<Row>)> {
    let mut rows = Vec::new();
    let mut row_num = 0;

    while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)
        .map_err(|e| TqError::RowFetch {
            row_num,
            message: e.to_string(),
        })?
    {
        let values: Vec<serde_json::Value> =
            serde_json::from_str(&row_json).map_err(|e| TqError::ResultParsing {
                row_num,
                message: e.to_string(),
            })?;

        let row = self.convert_row(&values, &columns)?;
        rows.push(row);
        row_num += 1;
    }

    Ok((columns, rows))
}
```

#### Step 5: Update all call sites

In `execute_and_fetch()`:

```rust
fn execute_and_fetch(
    &self,
    u_log: u64,
    conn_handle: u64,
    sql: &str,
    start: Instant,
) -> Result<QueryResult> {
    let bind_values = "null";

    // Create result set
    let rows_handle =
        teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, sql, bind_values)
            .map_err(|e| self.map_query_error(&e, sql))?;

    // Get column metadata
    let (_, _, _, column_metadata_json) =
        teradatarustapi::rustgo_result_metadata_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::MetadataFetch(e.to_string()))?;

    let columns = self.parse_column_metadata(&column_metadata_json)?;

    // Fetch all rows with known column metadata
    let (columns, rows) = self.fetch_all_rows(u_log, rows_handle, columns)?;

    // Close result set
    teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
        .map_err(|e| TqError::ResultSetClose(e.to_string()))?;

    log::debug!("Fetched {} rows", rows.len());

    Ok(QueryResult::new(columns, rows, start.elapsed()))
}
```

#### Step 6: Add error type

In `/Users/remi.turpaud/Code/genAI/tq/src/error.rs`, add new error variants:

```rust
#[error("Failed to fetch column metadata: {0}")]
MetadataFetch(String),

#[error("Failed to parse column metadata: {message}")]
MetadataParsing {
    message: String,
},

#[error("Column count mismatch: expected {expected}, got {actual}")]
ColumnCountMismatch {
    expected: usize,
    actual: usize,
},
```

#### Step 7: Remove or deprecate infer_columns()

The `infer_columns()` function can be removed entirely, or kept as a fallback with a warning:

```rust
/// DEPRECATED: Only use as fallback if metadata fetch fails
fn infer_columns_fallback(&self, values: &[serde_json::Value]) -> Vec<ColumnMetadata> {
    log::warn!("Using column inference fallback - column names will be generic");
    // ... existing implementation ...
}
```

### Testing the Fix

#### Test 1: Simple query with aliases

```bash
./target/release/tq query "SELECT 1 AS test_col, 'hello' AS name_col" --format table
```

**Expected**:
```
╭──────────┬──────────╮
│ test_col ┆ name_col │
╞══════════╪══════════╡
│        1 ┆ hello    │
╰──────────┴──────────╯
```

#### Test 2: Query without aliases

```bash
./target/release/tq query "SELECT * FROM DBC.TablesV WHERE TableName = 'MyTable'" --format table
```

**Expected**: Column names match actual table columns (e.g., "DatabaseName", "TableName", "TableKind")

#### Test 3: JSON format

```bash
./target/release/tq query "SELECT 1 AS id, 'test' AS name" --format json
```

**Expected**:
```json
[
  {
    "id": 1,
    "name": "test"
  }
]
```

#### Test 4: CSV format

```bash
./target/release/tq query "SELECT 1 AS id, 'test' AS name" --format csv
```

**Expected**:
```csv
id,name
1,test
```

### Automated Test

Add to `/Users/remi.turpaud/Code/genAI/tq/tests/integration_tests.rs`:

```rust
#[test]
fn test_query_column_names_with_aliases() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("query")
        .arg("SELECT 1 AS test_column, 'hello' AS message_column")
        .arg("--format")
        .arg("json")
        .env("TQ_LOGON", env_logon())
        .assert()
        .success()
        .stdout(predicate::str::contains("test_column"))
        .stdout(predicate::str::contains("message_column"));
}

#[test]
fn test_query_column_names_without_aliases() {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.arg("query")
        .arg("SELECT DatabaseName, TableName FROM DBC.TablesV SAMPLE 1")
        .arg("--format")
        .arg("json")
        .env("TQ_LOGON", env_logon())
        .assert()
        .success()
        .stdout(predicate::str::contains("DatabaseName"))
        .stdout(predicate::str::contains("TableName"));
}
```

### Estimated Effort

- **Code changes**: 2-3 hours
- **Testing**: 1 hour
- **Total**: 3-4 hours

---

## Critical Bug #2: No Default Row Limit in REPL Mode

### Problem Statement

The REPL mode executes queries without applying a default row limit. Users can accidentally execute queries that return millions of rows, overwhelming the terminal.

### Impact

- **Severity**: MAJOR (user considers critical)
- **Affects**: REPL mode only
- **User Impact**:
  - Large result sets flood terminal
  - Poor user experience
  - Potential memory exhaustion for very large results
  - Terminal becomes unresponsive

### Current Behavior

```bash
tq> SELECT * FROM DBC.TablesV;
-- Returns ALL rows (potentially thousands)
-- Terminal floods with output
```

### Specification Gap

The MVP specification (`docs/builder/detailed-specifications/interactive-mode-mvp.md`) does **NOT** specify a default row limit. This is a reasonable user expectation but wasn't documented.

### Solution Options

#### Option A: Implement 100-row Default (RECOMMENDED)

**Pros**:
- Matches user expectation
- Prevents accidental terminal flooding
- Common in BI/analytics tools
- Easy to override with explicit LIMIT

**Cons**:
- Changes behavior from specification
- May surprise users expecting all rows

**Implementation**:

**File**: `/Users/remi.turpaud/Code/genAI/tq/src/commands/repl/executor.rs`

```rust
pub fn execute_sql<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    writer: &mut W,
    use_color: bool,
) -> Result<usize> {
    let trimmed = sql.trim();

    if trimmed.is_empty() || trimmed == ";" {
        return Ok(0);
    }

    let sql_to_execute = trimmed.trim_end_matches(';').trim();

    if sql_to_execute.is_empty() {
        return Ok(0);
    }

    log::debug!("Executing SQL: {}", truncate_for_log(sql_to_execute));

    // NEW: Check if query needs default limit
    let needs_limit = is_select_without_limit(sql_to_execute);
    let row_limit = if needs_limit { Some(100) } else { None };

    let start = Instant::now();

    // Execute with or without limit
    let result = if let Some(limit) = row_limit {
        log::debug!("Applying default REPL limit: {} rows", limit);
        client.execute_with_limit(sql_to_execute, limit)?
    } else {
        client.execute(sql_to_execute)?
    };

    let execution_time = start.elapsed();
    let row_count = result.row_count;

    // Show results
    let format_options = FormatOptions::default()
        .with_header(true)
        .with_color(use_color);

    write_output_with_timing(
        &result,
        writer,
        OutputFormat::Table,
        &format_options,
        true,
    )?;

    // NEW: Show limit message if applied
    if needs_limit {
        writeln!(writer)?;
        writeln!(
            writer,
            "Showing first {} rows. Add LIMIT clause for different results.",
            row_count
        )?;
    }

    Ok(row_count)
}

/// Check if SQL is a SELECT without explicit LIMIT
fn is_select_without_limit(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();

    // Must be a SELECT
    if !sql_upper.trim_start().starts_with("SELECT") {
        return false;
    }

    // Check if already has LIMIT
    if sql_upper.contains("LIMIT") || sql_upper.contains("TOP") {
        return false;
    }

    true
}
```

Add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_select_without_limit_basic() {
        assert!(is_select_without_limit("SELECT * FROM table"));
        assert!(is_select_without_limit("  SELECT col FROM t"));
    }

    #[test]
    fn test_is_select_with_limit() {
        assert!(!is_select_without_limit("SELECT * FROM table LIMIT 10"));
        assert!(!is_select_without_limit("SELECT TOP 100 * FROM table"));
    }

    #[test]
    fn test_non_select_statements() {
        assert!(!is_select_without_limit("INSERT INTO table VALUES (1)"));
        assert!(!is_select_without_limit("UPDATE table SET col = 1"));
        assert!(!is_select_without_limit("DELETE FROM table"));
    }
}
```

**Configuration**:

Make limit configurable in `/Users/remi.turpaud/Code/genAI/tq/src/cli.rs`:

```rust
#[derive(Parser, Debug)]
pub struct ReplArgs {
    // ... existing fields ...

    /// Default row limit for SELECT queries (0 = unlimited)
    #[arg(long, default_value = "100", env = "TQ_REPL_LIMIT")]
    pub default_limit: usize,
}
```

Usage:
```bash
# Use default 100-row limit
tq repl

# Custom limit
tq repl --default-limit 500

# Unlimited
tq repl --default-limit 0

# Environment variable
export TQ_REPL_LIMIT=200
tq repl
```

#### Option B: Warn Before Large Results

Show interactive warning before executing potentially large queries.

**Pros**:
- Gives user control
- Educates users about query impact

**Cons**:
- Interrupts workflow
- Hard to estimate row count without executing
- May not work in non-interactive contexts

**NOT RECOMMENDED** for MVP - adds complexity.

#### Option C: Document Current Behavior

Add to `/help` output:

```
SQL Execution:
  Enter SQL statements ending with semicolon (;)
  Multi-line statements are supported

  TIP: Use LIMIT clause to restrict result size
       Example: SELECT * FROM table LIMIT 100
```

**Pros**:
- No code changes
- Matches most SQL REPL tools

**Cons**:
- Doesn't prevent user frustration
- Terminal flooding still possible

### Recommendation

**Implement Option A** (100-row default limit) with configurable override:

1. Default to 100 rows for SELECT queries without LIMIT
2. Make configurable via `--default-limit` flag
3. Allow `--default-limit 0` to disable
4. Show message when limit is applied
5. Update `/help` to document this behavior

### Testing

```bash
# Test default limit
./target/release/tq repl
tq> SELECT * FROM DBC.TablesV;
-- Should show max 100 rows with message

# Test explicit LIMIT override
tq> SELECT * FROM DBC.TablesV LIMIT 50;
-- Should show 50 rows, no limit message

# Test non-SELECT
tq> SHOW TABLES;
-- Should show all, no limit applied
```

### Estimated Effort

- **Code changes**: 2-3 hours
- **Testing**: 1 hour
- **Total**: 3-4 hours

---

## Summary

Both bugs have clear solutions:

1. **Column names bug**: Use `rustgo_result_metadata_wrapper()` to get actual column metadata (3-4 hours)
2. **Row limit issue**: Implement configurable default limit for SELECT queries (3-4 hours)

**Total estimated effort**: 6-8 hours to fix both issues

**Testing effort**: 2-3 hours for comprehensive validation

**Total time to production**: 1-2 days
