# Fix Required: Metadata Parsing Bug

**Bug**: Tool cannot parse column metadata from Teradata API
**File**: `src/db/client.rs`
**Lines**: 248-285 (function `parse_column_metadata`)

---

## Current (Broken) Implementation

```rust
fn parse_column_metadata(&self, metadata_json: &str) -> Result<Vec<ColumnMetadata>> {
    #[derive(serde::Deserialize)]
    struct MetadataColumn {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Type")]
        type_name: String,
        #[serde(rename = "Nullable", default)]
        nullable: bool,
    }

    // This expects an array of objects:
    // [{"Name": "col1", "Type": "INTEGER", "Nullable": true}, ...]
    let metadata: Vec<MetadataColumn> = serde_json::from_str(metadata_json)
        .map_err(|e| TqError::MetadataParsing {
            message: format!("Failed to parse column metadata: {}", e),
        })?;

    Ok(metadata.into_iter()
        .map(|col| {
            let data_type = map_type_name_to_teradata_type(&col.type_name);
            ColumnMetadata::new(col.name, data_type, col.nullable)
        })
        .collect())
}
```

**Error**: `invalid type: map, expected a sequence at line 1 column 0`

---

## Actual API Format

The Teradata API returns column-oriented data (map of arrays):

```json
{
  "ColumnName": ["test", "another_col"],
  "TypeName": ["BYTEINT", "VARCHAR"],
  "Nullable": [false, true],
  "Precision": [3, 100],
  "Scale": [0, 0],
  "MaxByteCount": [1, 100]
}
```

**Key Properties Used**:
- `ColumnName` - array of column names
- `TypeName` - array of type names (e.g., "BYTEINT", "VARCHAR", "DATE")
- `Nullable` - array of boolean values

---

## Fixed Implementation

Replace the `parse_column_metadata` function with:

```rust
fn parse_column_metadata(&self, metadata_json: &str) -> Result<Vec<ColumnMetadata>> {
    // Handle empty metadata (e.g., for DDL statements)
    if metadata_json.is_empty() || metadata_json == "null" || metadata_json == "{}" {
        return Ok(Vec::new());
    }

    // Teradata API returns column-oriented data (map of arrays)
    #[derive(serde::Deserialize)]
    struct MetadataMap {
        #[serde(rename = "ColumnName")]
        column_names: Vec<String>,
        #[serde(rename = "TypeName")]
        type_names: Vec<String>,
        #[serde(rename = "Nullable", default)]
        nullable: Vec<bool>,
    }

    let metadata_map: MetadataMap = serde_json::from_str(metadata_json)
        .map_err(|e| TqError::MetadataParsing {
            message: format!("Failed to parse column metadata: {}", e),
        })?;

    // Verify array lengths match
    let num_columns = metadata_map.column_names.len();
    if metadata_map.type_names.len() != num_columns {
        return Err(TqError::MetadataParsing {
            message: format!(
                "Metadata array length mismatch: {} column names but {} type names",
                num_columns,
                metadata_map.type_names.len()
            ),
        });
    }

    // Transpose from column-oriented to row-oriented format
    let columns: Vec<ColumnMetadata> = metadata_map.column_names
        .into_iter()
        .zip(metadata_map.type_names)
        .enumerate()
        .map(|(i, (name, type_name))| {
            let nullable = metadata_map.nullable.get(i).copied().unwrap_or(true);
            let data_type = map_type_name_to_teradata_type(&type_name);
            ColumnMetadata::new(name, data_type, nullable)
        })
        .collect();

    Ok(columns)
}
```

---

## Test Case to Validate Fix

Add this integration test to ensure the fix works:

```rust
#[test]
#[ignore] // Requires live database connection
fn test_actual_column_names_from_metadata() {
    use std::env;

    // Load from .env file
    dotenv::dotenv().ok();
    let logon = env::var("TQ_LOGON").expect("TQ_LOGON must be set");

    let config = ConnectionConfig::from_connection_string(&logon).unwrap();
    let client = DatabaseClient::new(config, None).unwrap();

    // Execute query with known column names
    let result = client.execute("SELECT 1 AS test_col, 'hello' AS text_col, NULL AS null_col").unwrap();

    // Verify actual column names are used (not generic col1, col2, col3)
    assert_eq!(result.columns.len(), 3);
    assert_eq!(result.columns[0].name, "test_col");
    assert_eq!(result.columns[1].name, "text_col");
    assert_eq!(result.columns[2].name, "null_col");

    // Verify row data
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].len(), 3);
}
```

Run with: `cargo test test_actual_column_names_from_metadata -- --ignored`

---

## Verification Steps

After applying the fix:

1. **Build**: `cargo build --release`
2. **Unit Tests**: `cargo test` (should still pass)
3. **Live Test**: `tq query "SELECT 1 AS test_col"`
   - Should succeed (not error)
   - Should show "test_col" as column name (not "col1")
4. **Multiple Columns**: `tq query "SELECT 1 AS a, 2 AS b, 3 AS c"`
   - Should show actual column names: a, b, c
5. **NULL Handling**: `tq query "SELECT NULL AS null_col"`
   - Should display `[NULL]` in table format
   - Should display `null` in JSON format

---

## Additional Notes

### Type Name Mapping

The `map_type_name_to_teradata_type` function needs to handle these type names:
- "BYTEINT", "SMALLINT", "INTEGER", "BIGINT" → `TeradataType::Integer`
- "DECIMAL", "NUMERIC" → `TeradataType::Decimal`
- "FLOAT", "REAL", "DOUBLE PRECISION" → `TeradataType::Float`
- "CHAR", "VARCHAR" → `TeradataType::Varchar`
- "DATE" → `TeradataType::Date`
- "TIME" → `TeradataType::Time`
- "TIMESTAMP" → `TeradataType::Timestamp`

Check `src/db/types.rs` for the full implementation.

### Edge Cases

1. **Empty Result Sets**: DDL statements may return empty metadata
   - Handle `{}` and `null` JSON responses
2. **Mismatched Array Lengths**: Validate all arrays have same length
   - Return clear error if mismatch detected
3. **Missing Nullable Field**: Default to `true` if not provided
   - Safer to assume nullable than to assume NOT NULL

---

## Testing Recommendations

1. **Add Live Database Test**: Include at least one query execution test in CI
2. **Test Various Data Types**: INT, VARCHAR, DATE, TIMESTAMP, DECIMAL, NULL
3. **Test DDL Statements**: CREATE, DROP, etc. (may have no metadata)
4. **Test Multi-Column Queries**: Verify array transposition works correctly

---

## References

- **API Documentation**: teradatarustapi `rustgo_result_metadata_wrapper` function
- **Specification**: `/Users/remi.turpaud/Code/genAI/tq/docs/builder/specifications.md`
- **Architecture**: `/Users/remi.turpaud/Code/genAI/tq/docs/builder/rust-architecture.md`
- **Previous Working Version**: commit 369af18 (used generic column names)
