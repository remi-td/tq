# Output Format Specifications

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Format Selection](#81-format-selection)
2. [Table Format](#82-table-format)
3. [JSON Format](#83-json-format)
4. [CSV Format](#84-csv-format)
5. [Format Comparison](#85-format-comparison)

---

## 8.1 Format Selection

### 8.1.1 Selection Priority

1. **Command-line flag**: `--format table`
2. **Environment variable**: `TQ_FORMAT=json`
3. **Configuration file**: `format = "csv"`
4. **Context-based default**:
   - TTY (interactive): `table`
   - Piped: `csv` or `json` (configurable)

### 8.1.2 Format Types

| Format | Use Case | File Extension | MIME Type |
|--------|----------|----------------|-----------|
| `table` | Human-readable, interactive | - | `text/plain` |
| `json` | Scripting, APIs, parsing | `.json` | `application/json` |
| `csv` | Data export, Excel, analysis | `.csv` | `text/csv` |

## 8.2 Table Format

### 8.2.1 ASCII Table (Default)

```
┌──────────┬──────────┬─────────┐
│ id       │ name     │ active  │
├──────────┼──────────┼─────────┤
│ 1        │ Alice    │ true    │
│ 2        │ Bob      │ false   │
└──────────┴──────────┴─────────┘

2 rows in set (0.123s)
```

**Features**:
- Box-drawing characters
- Auto-sizing columns
- Truncation with ellipsis for wide content
- Row count and timing footer

### 8.2.2 Simple Table (--table-style simple)

```
 id  | name  | active
-----+-------+--------
 1   | Alice | true
 2   | Bob   | false

(2 rows)
```

**Use Case**: Better for copying/pasting, terminal compatibility

### 8.2.3 Compact Table (--table-style compact)

```
id name  active
 1 Alice true
 2 Bob   false
```

**Use Case**: Dense output, logs

### 8.2.4 Markdown Table (--table-style markdown)

```
| id | name  | active |
|----|-------|--------|
| 1  | Alice | true   |
| 2  | Bob   | false  |
```

**Use Case**: Documentation, GitHub issues

### 8.2.5 Column Alignment

- **Numbers**: Right-aligned
- **Text**: Left-aligned
- **Booleans**: Centered
- **Dates**: Left-aligned

### 8.2.6 Wide Content Handling

```
┌──────────┬──────────┬──────────────────────┐
│ id       │ name     │ description          │
├──────────┼──────────┼──────────────────────┤
│ 1        │ Alice    │ Senior Software E... │  ← truncated
│ 2        │ Bob      │ Product Manager      │
└──────────┴──────────┴──────────────────────┘

Use --no-truncate to see full content
```

### 8.2.7 NULL Representation

```
┌──────────┬──────────┬─────────┐
│ id       │ name     │ email   │
├──────────┼──────────┼─────────┤
│ 1        │ Alice    │ a@ex.co │
│ 2        │ Bob      │ [NULL]  │  ← grayed, italic
└──────────┴──────────┴─────────┘
```

## 8.3 JSON Format

### 8.3.1 Array of Objects (Default)

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

### 8.3.2 Streaming JSONL (--json-format lines)

```jsonl
{"id":1,"name":"Alice","email":"alice@example.com","active":true}
{"id":2,"name":"Bob","email":"bob@example.com","active":false}
```

**Use Case**: Large datasets, streaming processing

### 8.3.3 Metadata Wrapper (--json-format wrapped)

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

**Use Case**: APIs, complete metadata

### 8.3.4 Type Mapping

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

## 8.4 CSV Format

### 8.4.1 Standard CSV (RFC 4180)

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

### 8.4.2 Excel-Compatible CSV

```bash
tq query --format csv --excel "SELECT * FROM users" > users.csv
```

**Differences**:
- BOM (Byte Order Mark) for UTF-8
- CRLF line endings (`\r\n`)
- Date format: `YYYY-MM-DD`

### 8.4.3 Custom Delimiter (TSV)

```bash
tq query --format csv --delimiter '\t' "SELECT * FROM data" > data.tsv
```

Output:
```tsv
id      name    email
1       Alice   alice@example.com
2       Bob     bob@example.com
```

### 8.4.4 Escaping Rules

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

### 8.4.5 NULL Representation

```csv
id,name,email
1,Alice,alice@example.com
2,Bob,
```

**Options**:
- Empty field (default)
- `--null-string "NULL"` → explicit marker

## 8.5 Format Comparison

| Feature | Table | JSON | CSV |
|---------|-------|------|-----|
| Human-readable | ✅ Excellent | ⚠️ Okay | ⚠️ Okay |
| Machine-parseable | ❌ Poor | ✅ Excellent | ✅ Good |
| Type preservation | ❌ No | ✅ Yes | ❌ No (all strings) |
| Streaming friendly | ✅ Yes | ⚠️ JSONL only | ✅ Yes |
| Excel compatible | ❌ No | ❌ No | ✅ Yes |
| File size | N/A | Medium | Small |
| Processing speed | Fast | Medium | Fast |

---
