# Output Format Specifications

## Format Selection

### Selection Priority

1. **Command-line flag**: `--format table`
2. **Environment variable**: `TQ_FORMAT=json`
3. **Configuration file**: `format = "csv"`
4. **Context-based default**:
   - TTY (interactive): `table`
   - Piped: `csv` or `json` (configurable)

### Format Types

| Format | Use Case | File Extension | MIME Type |
|--------|----------|----------------|-----------|
| `table` | Human-readable, interactive | - | `text/plain` |
| `json` | Scripting, APIs, parsing | `.json` | `application/json` |
| `csv` | Data export, Excel, analysis | `.csv` | `text/csv` |

## Table Format

### Design Philosophy

The table formatting approach focuses on terminal width awareness:

1. **Simplicity** - No complex padding calculations
2. **Terminal Width Awareness** - Detect terminal width and show columns that fit
3. **Clear Truncation** - Explicitly indicate when columns are hidden
4. **Batch Mode Compatibility** - Show all columns in non-TTY contexts

### Terminal Width Detection

**Interactive Mode (TTY):**
- Detect terminal width using terminal size detection
- Default to 80 columns if detection fails
- Recalculate on window resize (best effort)

**Batch Mode (Non-TTY):**
- Show ALL columns regardless of width
- No truncation or column hiding
- Optimized for piping and file redirection

### Column Truncation Strategy

When columns don't fit in terminal width:

1. **Prioritize Leftmost Columns** - Show columns from left to right until width exhausted
2. **Calculate Minimum Width** - Each column gets minimum width based on content
3. **Add Truncation Indicator** - When columns are hidden:
   - Header: Show `| (+n cols) |` in rightmost position
   - Body: Show `| ... |` in rightmost position
4. **No Padding** - Columns are NOT padded, just basic spacing

**Example with 80-column terminal:**

```
┌─────┬──────────┬────────┬──────────┬─────────────┐
│ id  │ username │ active │ dept     │ (+3 cols)   │
├─────┼──────────┼────────┼──────────┼─────────────┤
│ 1   │ alice    │ true   │ eng      │ ...         │
│ 2   │ bob      │ false  │ sales    │ ...         │
│ 3   │ charlie  │ true   │ ops      │ ...         │
└─────┴──────────┴────────┴──────────┴─────────────┘

3 columns hidden: email, created_at, last_login
3 rows in set (0.045s)
```

**Footer Message:**
- When columns are truncated, show: `n columns hidden: col1, col2, col3`
- Suggest: `Use --format csv or --format json to see all columns`

### ASCII Table (Default)

```
┌────┬───────┬────────┐
│ id │ name  │ active │
├────┼───────┼────────┤
│ 1  │ Alice │ true   │
│ 2  │ Bob   │ false  │
└────┴───────┴────────┘

2 rows in set (0.123s)
```

**Features**:
- Box-drawing characters (┌─┬─┐ │ ├─┼─┤ └─┴─┘)
- Minimal spacing (no padding)
- Basic column alignment
- Row count and timing footer
- Terminal width awareness

### Simple Table (--table-style simple)

```
id | name  | active
---+-------+-------
 1 | Alice | true
 2 | Bob   | false

(2 rows)
```

Use Case: Better for copying/pasting, terminal compatibility

### Compact Table (--table-style compact)

```
id name  active
 1 Alice true
 2 Bob   false
```

Use Case: Dense output, logs

### Markdown Table (--table-style markdown)

```
| id | name  | active |
|----|-------|--------|
| 1  | Alice | true   |
| 2  | Bob   | false  |
```

Use Case: Documentation, GitHub issues

### Column Alignment

**Basic alignment only (no padding):**
- **Numbers**: Right-aligned within minimum space
- **Text**: Left-aligned within minimum space
- **Booleans**: Left-aligned within minimum space
- **Dates**: Left-aligned within minimum space

### NULL Representation

```
┌────┬───────┬────────┐
│ id │ name  │ email  │
├────┼───────┼────────┤
│ 1  │ Alice │ a@e.co │
│ 2  │ Bob   │ [NULL] │
└────┴───────┴────────┘
```

**NULL Values:**
- Display as `[NULL]` in table cells
- Optional styling: gray/italic when supported

## JSON Format

### Array of Objects (Default)

```json
[
  {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com",
    "active": true,
    "created_at": "2024-01-15T10:30:00Z"
  },
  {
    "id": 2,
    "name": "Bob",
    "email": "bob@example.com",
    "active": false,
    "created_at": "2024-01-16T11:45:00Z"
  }
]
```

**Features**:
- Each row is a JSON object
- Column names as keys
- Type preservation (numbers, booleans, null)
- ISO 8601 for dates/timestamps

### Streaming JSONL (--json-format lines)

```jsonl
{"id":1,"name":"Alice","email":"alice@example.com","active":true}
{"id":2,"name":"Bob","email":"bob@example.com","active":false}
```

Use Case: Large datasets, streaming processing

### Metadata Wrapper (--json-format wrapped)

```json
{
  "query": "SELECT id, name FROM users",
  "execution_time_ms": 123,
  "row_count": 2,
  "columns": [
    {"name": "id", "type": "INTEGER"},
    {"name": "name", "type": "VARCHAR"}
  ],
  "rows": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ]
}
```

Use Case: APIs, complete metadata

### Type Mapping

| Teradata Type | JSON Type | Example |
|---------------|-----------|---------|
| INTEGER, BIGINT | number | `42` |
| DECIMAL, FLOAT | number | `3.14` |
| VARCHAR, CHAR | string | `"text"` |
| DATE | string | `"2024-01-15"` |
| TIMESTAMP | string | `"2024-01-15T10:30:00Z"` |
| BOOLEAN | boolean | `true`, `false` |
| NULL | null | `null` |
| BLOB, CLOB | string (base64) | `"YWJjMTIz"` |

## CSV Format

### Standard CSV (RFC 4180)

```csv
id,name,email,active,created_at
1,Alice,alice@example.com,true,2024-01-15T10:30:00Z
2,Bob,bob@example.com,false,2024-01-16T11:45:00Z
```

**Features**:
- Header row (optional with `--no-header`)
- Double-quote escaping for special characters
- Comma separator (configurable with `--delimiter`)
- LF line endings (`\n`)

### Excel-Compatible CSV

```bash
tq query --format csv --excel "SELECT * FROM users" > users.csv
```

**Differences**:
- BOM (Byte Order Mark) for UTF-8
- CRLF line endings (`\r\n`)
- Date format: `YYYY-MM-DD`

### Custom Delimiter (TSV)

```bash
tq query --format csv --delimiter '\t' "SELECT * FROM data" > data.tsv
```

Output:
```tsv
id      name    email
1       Alice   alice@example.com
2       Bob     bob@example.com
```

### Escaping Rules

```csv
id,name,description
1,Alice,"Senior Engineer, Team Lead"
2,Bob,"Quote: ""Hello World"""
3,Charlie,"Line 1
Line 2"
```

**Rules**:
- Fields with commas → quoted
- Fields with quotes → quoted, quotes doubled
- Fields with newlines → quoted

### NULL Representation

```csv
id,name,email
1,Alice,alice@example.com
2,Bob,
```

**Options**:
- Empty field (default)
- `--null-string "NULL"` → explicit marker

## Format Comparison

| Feature | Table | JSON | CSV |
|---------|-------|------|-----|
| Human-readable | ✅ Excellent | ⚠️ Okay | ⚠️ Okay |
| Machine-parseable | ❌ Poor | ✅ Excellent | ✅ Good |
| Type preservation | ❌ No | ✅ Yes | ❌ No (all strings) |
| Streaming friendly | ✅ Yes | ⚠️ JSONL only | ✅ Yes |
| Excel compatible | ❌ No | ❌ No | ✅ Yes |
| File size | N/A | Medium | Small |
| Processing speed | Fast | Medium | Fast |
