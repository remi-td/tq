---
name: teradata-python
description: Connect to Teradata and build performant data pipelines in Python using teradatasql. Covers connection setup, FastLoad bulk loading, DDL best practices, and in-database ELT.
user-invocable: true
argument-hint: [script or pipeline]
---

# Teradata Python Development with `teradatasql`

This skill provides generic, production-ready patterns for connecting to Teradata, bulk loading data via FastLoad, managing schemas, and executing in-database ELT transformations using Python and the official `teradatasql` driver.

---

## 1. Connecting to Teradata

Teradata connection parameters are provided via `DATABASE_URI`, `TQ_LOGON`, or individual `TERADATA_*` environment variables.

```python
import os
from urllib.parse import urlparse
import teradatasql


def get_teradata_connection():
    """Establish connection from DATABASE_URI, TQ_LOGON, or environment variables."""
    uri_str = os.environ.get("DATABASE_URI") or os.environ.get("TQ_LOGON", "")
    if uri_str:
        if not uri_str.startswith("teradata://") and "://" not in uri_str:
            uri_str = "teradata://" + uri_str
        u = urlparse(uri_str.split("?")[0])
        host = u.hostname or "localhost"
        port = u.port or 1025
        user = u.username or os.environ.get("USER", "")
        password = u.password or ""
        database = u.path.lstrip("/") or user
    else:
        host = os.environ.get("TERADATA_HOST", "localhost")
        port = int(os.environ.get("TERADATA_PORT", 1025))
        user = os.environ.get("TERADATA_USER", "demo_user")
        password = os.environ.get("TERADATA_PASSWORD", "")
        database = os.environ.get("TERADATA_DATABASE", user)

    return teradatasql.connect(
        host=host, dbs_port=port, user=user, password=password, database=database,
        logmech=os.environ.get("TERADATA_LOGMECH", "TD2")
    )
```

---

## 2. High-Performance Bulk Loading (`FastLoad`)

Inserting rows one-by-one is prohibitively slow on MPP architectures. The `teradatasql` driver supports Teradata's parallel wire FastLoad protocol via the `{fn teradata_try_fastload}` prefix.

### Key FastLoad Requirements
1. **Always Use `CREATE MULTISET TABLE`**: Teradata defaults to `SET` tables which reject duplicate rows. Staging tables must be created as `CREATE MULTISET TABLE` with an explicit `PRIMARY INDEX`.
2. **Table Must Be Empty**: FastLoad streams into an empty table. Drop or truncate before loading.
3. **Clean Empty Strings**: Empty CSV strings `""` must be converted to `None` in Python so the driver binds true database `NULL`s instead of empty strings.

```python
import pandas as pd


def fastload_csv(conn, table_name: str, csv_path: str):
    """Stream CSV rows directly into an empty Teradata MULTISET table using FastLoad."""
    df = pd.read_csv(csv_path, dtype=object, keep_default_na=False)
    
    # Clean empty/whitespace strings to None (immune to pandas 3.0 NaN dtype casting)
    rows = [
        tuple(None if v is None or (isinstance(v, str) and not v.strip()) else v for v in row)
        for row in df.values.tolist()
    ]

    cols = list(df.columns)
    placeholders = ", ".join(["?" for _ in cols])
    sql = f"{{fn teradata_try_fastload}}INSERT INTO {table_name} ({', '.join(cols)}) VALUES ({placeholders})"
    
    cursor = conn.cursor()
    cursor.executemany(sql, rows)
    conn.commit()
    cursor.close()
    print(f"Fastloaded {len(rows):,} rows into {table_name}")
```

---

## 3. DDL & Primary Index Best Practices

### A. Idempotent DDL (Handling Error 3807)
Dropping non-existent objects raises Teradata Error `3807` ("Object does not exist"). Catch and ignore it:

```python
def safe_drop(cursor, object_name: str, object_type: str = "TABLE"):
    """Drop table or view safely without failing on first deployment."""
    try:
        cursor.execute(f"DROP {object_type} {object_name}")
    except teradatasql.DatabaseError as e:
        if "3807" not in str(e):
            raise
```

### B. Table Definition & Primary Index (PI) Selection
- **Table Type**: Always use `CREATE MULTISET TABLE` for staging and transactional data.
- **Primary Index**: Always declare an explicit `PRIMARY INDEX (high_cardinality_col)` matching natural join keys (e.g. `entity_id`) to distribute data uniformly across AMPs and eliminate CPU skew.

```sql
CREATE MULTISET TABLE stg_events (
    event_id INTEGER NOT NULL,
    entity_id INTEGER,
    event_type VARCHAR(50),
    amount_usd DECIMAL(18,2),
    event_date VARCHAR(50)
) PRIMARY INDEX (event_id);
```

### C. Safe Date Casting (Preventing Error 2665)
In Teradata SQL, casting an empty string `''` to DATE throws `[Error 2665] Invalid date`. Always sanitize nullable date columns with `NULLIF(TRIM(...), '')`:

```sql
-- Safe date cast handling empty strings and standardizing slash separators:
CAST(
    CASE 
        WHEN date_col LIKE '%/%' THEN OREPLACE(TRIM(date_col), '/', '-')
        ELSE NULLIF(TRIM(date_col), '')
    END AS DATE FORMAT 'YYYY-MM-DD'
) AS event_date
```

---

## 4. In-Database ELT & Semantic Views

Do not download datasets into Python to perform joins or rollups. Perform transformations inside Teradata:

### A. Deduplication with `QUALIFY`
Deduplicate raw staging feeds using native window ranking:
```sql
CREATE MULTISET TABLE dim_entities AS (
    SELECT
        entity_id,
        TRIM(entity_name) AS entity_name,
        UPPER(TRIM(status_code)) AS status_code
    FROM stg_entities
    QUALIFY ROW_NUMBER() OVER (PARTITION BY entity_id ORDER BY load_timestamp DESC) = 1
) WITH DATA
PRIMARY INDEX (entity_id);
```

### B. Semantic Views
Expose reusable business metrics (e.g. margin, delivery cycle, success rate) via views:
```sql
CREATE VIEW v_sem_performance AS
SELECT
    f.event_id,
    f.entity_id,
    f.gross_amount,
    f.net_revenue,
    (f.net_revenue - f.cost) AS gross_profit,
    ((f.net_revenue - f.cost) / NULLIFZERO(f.net_revenue)) * 100 AS margin_pct
FROM fct_events f;
```

---

## 5. Context Hygiene & Token Management

When exploring data or debugging in Python:
- **Never print entire DataFrames to stdout** (e.g. `print(df)` or row-iteration loops). Dumping thousands of rows floods the context window and dilutes attention.
- **Inspect compactly**:
  ```python
  print(df.shape)
  print(df.head(3))
  print(df['status_col'].value_counts(dropna=False).head(5))
  ```
- If a SQL error occurs, inspect the specific query and error message rather than printing raw table rows.
