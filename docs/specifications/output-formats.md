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

### Column Width Calculation

#### REQ-TABLE-WIDTH-001: Content-Based Width Calculation

**Requirement:** Column widths shall be calculated from actual cell content, not from database schema type definitions.

**Rationale:** Schema types often define maximum capacity (e.g., VARCHAR(64)) that significantly exceeds typical content length (e.g., 15 characters). Width calculation based on actual content maximizes information density in table displays.

**Calculation Logic:**
```
column_width = max(
  max_content_length_across_all_rows,
  header_length
)
```

**Example:**
- Column: `DatabaseName VARCHAR(64)`
- Actual content: "SystemDB" (8 chars), "TempDB" (6 chars), "UserDB" (6 chars)
- Header: "DatabaseName" (12 chars)
- **Calculated width: 12 characters** (not 64)

#### REQ-TABLE-WIDTH-002: Maximum Width Cap

**Requirement:** Individual column widths shall be capped at a maximum limit to prevent single columns from dominating display space.

**Rationale:** Prevents columns with occasional long values from consuming excessive terminal width, ensuring balanced display across multiple columns.

**Specification:**
- Default maximum: 100 characters per column
- Values exceeding maximum are truncated with ellipsis: `Long value that ex...`
- Width calculation considers truncated length, not original content length

**Example:**
- Content: 150-character string
- Displayed: First 97 chars + "..." (100 chars total)
- Column width: 100 characters

#### REQ-TABLE-WIDTH-003: Minimum Width Enforcement

**Requirement:** Column widths shall never be smaller than the column header length.

**Rationale:** Headers must remain fully visible for column identification. Short content should not compress headers.

**Example:**
- Header: "AccountName" (11 chars)
- Content: "A" (1 char), "B" (1 char)
- **Column width: 11 characters** (header length)

#### REQ-TABLE-WIDTH-004: NULL Value Width Handling

**Requirement:** NULL values shall be represented as `[NULL]` (6 characters) and included in width calculations.

**Rationale:** NULL representation must be visible and factored into column width to prevent layout breaks when NULL values appear.

**Example:**
- Content: "Alice", "Bob", `[NULL]`
- Widths considered: 5, 3, 6
- Max content width: 6 characters (from `[NULL]`)

#### REQ-TABLE-WIDTH-005: Numeric Column Right-Alignment

**Requirement:** Numeric columns shall remain right-aligned within their content-calculated width.

**Rationale:** Right alignment for numbers is essential for readability, especially when comparing values or performing mental arithmetic.

**Example:**
```
┌──────┬────────┐
│ id   │ amount │
├──────┼────────┤
│    1 │    9.5 │
│   42 │  123.0 │
│  103 │ 1500.2 │
└──────┴────────┘
```

#### REQ-TABLE-WIDTH-006: Empty String Width Handling

**Requirement:** Empty strings shall be treated as zero-width content, with column width determined by header or other non-empty values.

**Rationale:** Empty values should not influence column width calculation; headers or actual content should dictate width.

**Example:**
- Header: "Status" (6 chars)
- Content: "active", "", "pending"
- **Column width: 7 characters** (longest content: "pending")

#### REQ-TABLE-WIDTH-007: Width Calculation Timing

**Requirement:** Column widths shall be calculated once per result set, after all rows are fetched, before rendering begins.

**Rationale:** Accurate width calculation requires examining all row content. Single-pass calculation prevents layout inconsistencies during rendering.

**Performance Consideration:** For large result sets (10,000+ rows), width calculation should complete within 100ms to avoid perceived delay.

### Column Truncation Strategy

When columns don't fit in terminal width:

1. **Prioritize Leftmost Columns** - Show columns from left to right until width exhausted
2. **Calculate Content-Based Width** - Each column gets width based on actual content (see REQ-TABLE-WIDTH-001 through REQ-TABLE-WIDTH-007)
3. **Add Truncation Indicator** - When columns are hidden:
   - Header: Show `| (+n cols) |` in rightmost position
   - Body: Show `| ... |` in rightmost position
4. **No Padding** - Columns are NOT padded, just basic spacing

#### REQ-TABLE-WIDTH-008: Multi-Byte Character Support

**Requirement:** Width calculations shall account for multi-byte Unicode characters (e.g., emoji, CJK characters) using display width, not byte count.

**Rationale:** Display width differs from byte count for Unicode. A 3-byte emoji may display as 2 columns, while a 1-byte ASCII displays as 1 column.

**Example:**
- Content: "Hello 👋" (byte count: 10, display width: 8)
- **Column width uses: 8** (display width)

**Implementation Note:** Use Unicode display width calculation (UAX #11 East Asian Width).

#### REQ-TABLE-WIDTH-009: Consistency Across Result Sets

**Requirement:** Multiple queries in the same session may produce different column widths based on their respective result set content.

**Rationale:** Each query result should optimize its own display density. Fixed widths across queries would sacrifice information density.

**Example:**
```sql
-- Query 1: Short names
SELECT DatabaseName FROM DBC.Databases WHERE DatabaseName LIKE 'Sys%';
-- Column width: ~12 chars (header + short content)

-- Query 2: Long names
SELECT DatabaseName FROM DBC.Databases WHERE DatabaseName LIKE 'ProductionBackup%';
-- Column width: ~25 chars (header + longer content)
```

#### REQ-TABLE-WIDTH-010: Batch Mode Behavior

**Requirement:** In batch mode (non-TTY contexts), content-based width calculation still applies, but no terminal width limit is enforced.

**Rationale:** Batch mode should show all columns. Content-based widths still improve readability by avoiding excessive whitespace from schema-defined types.

**Example (piped output):**
```bash
tq query "SELECT * FROM DBC.Databases" | less
# All columns visible, widths calculated from content, no truncation
```

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

### Content-Based Width: Before and After

#### Example: `SELECT * FROM DBC.Databases` in 117-character terminal

**Before (Schema-Based Width):**
```
┌──────────────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────────┬──────────────┐
│ DatabaseName                                                     │ CreatorName                                                      │ (+14 cols)   │
├──────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────┼──────────────┤
│ SystemDB                                                         │ DBC                                                              │ ...          │
│ TempDB                                                           │ DBC                                                              │ ...          │
│ UserDB                                                           │ DBC                                                              │ ...          │
└──────────────────────────────────────────────────────────────────┴──────────────────────────────────────────────────────────────────┴──────────────┘

14 columns hidden: OwnerName, AccountName, ProtectionType, JournalFlag, PermSpace, SpoolSpace, TempSpace, ...
3 rows in set (0.045s)
```
- Schema: `DatabaseName VARCHAR(64)`, `CreatorName VARCHAR(64)`
- Result: 2 columns visible, 64 chars each = 128+ chars consumed
- Problem: Actual content only ~10 chars, massive wasted space

**After (Content-Based Width):**
```
┌──────────────┬─────────────┬───────────┬─────────────┬────────────────┬─────────────┬───────────┬───────────┬───────────┬─────────────┐
│ DatabaseName │ CreatorName │ OwnerName │ AccountName │ ProtectionType │ JournalFlag │ PermSpace │ SpoolSpace│ TempSpace │ (+7 cols)   │
├──────────────┼─────────────┼───────────┼─────────────┼────────────────┼─────────────┼───────────┼───────────┼───────────┼─────────────┤
│ SystemDB     │ DBC         │ DBC       │ $SYSTEM     │ None           │ None        │  1048576  │    524288 │    262144 │ ...         │
│ TempDB       │ DBC         │ DBC       │ $SYSTEM     │ None           │ None        │        0  │         0 │  1048576  │ ...         │
│ UserDB       │ DBC         │ UserAdmin │ $USER       │ Read           │ Dual        │  5242880  │  1048576  │    524288 │ ...         │
└──────────────┴─────────────┴───────────┴─────────────┴────────────────┴─────────────┴───────────┴───────────┴───────────┴─────────────┘

7 columns hidden: CreateTimeStamp, LastAlterName, LastAlterTimeStamp, ...
3 rows in set (0.045s)
```
- Width: Based on max(content, header) per column
- Result: 9 columns visible (was 2), 7 hidden (was 14)
- Benefit: 4.5x more columns visible, significantly improved information density

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
