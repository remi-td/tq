---
name: teradata-python
description: Connect to Teradata and build performant data pipelines in Python using teradatasql. Covers connection setup, high-performance bulk loading (FastLoad), schema exploration, DDL, and ELT transformations.
user-invocable: true
argument-hint: [script or pipeline]
---

# Teradata Python Development with `teradatasql`

This skill provides expert patterns for connecting to Teradata, bulk loading datasets, executing DDL, and running high-performance ELT transformations using Python and the official `teradatasql` driver.

---

## 1. Prerequisites & Installation

The official Teradata Python driver is `teradatasql` (compliant with Python DB-API 2.0):

```bash
pip install teradatasql pandas
```

Verify installation:
```bash
python3 -c "import teradatasql; print('teradatasql ready:', teradatasql.__version__)"
```

---

## 2. Connecting to Teradata

Teradata connection parameters are typically passed via environment variables (`DATABASE_URI`, `TQ_LOGON`, or individual `TERADATA_*` variables).

### Universal Connection Helper

Use this helper function to parse credentials automatically from the environment:

```python
import os
import re
from urllib.parse import urlparse
import teradatasql


def get_teradata_connection():
    """Establish connection using DATABASE_URI, TQ_LOGON, or individual env vars."""
    uri_str = os.environ.get("DATABASE_URI") or os.environ.get("TQ_LOGON", "")
    
    if uri_str:
        # Standardize scheme for urlparse: e.g. user:pass@host:1025/dbname
        if not uri_str.startswith("teradata://") and "://" not in uri_str:
            uri_str = "teradata://" + uri_str
        
        # Strip trailing query params if present
        base_uri = uri_str.split("?")[0]
        u = urlparse(base_uri)
        
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

    conn = teradatasql.connect(
        host=host,
        dbs_port=port,
        user=user,
        password=password,
        database=database,
        logmech=os.environ.get("TERADATA_LOGMECH", "TD2")
    )
    return conn
```

---

## 3. High-Performance Bulk Loading & FastLoad

Teradata is an MPP database with multiple AMPs (Access Module Processors). Inserting rows one-by-one (`INSERT INTO ... VALUES (...)`) over network sockets is extremely slow. Always use bulk batch insertion.

### Method A: Teradata FastLoad via Driver (`{fn teradata_try_fastload}`)

The `teradatasql` driver includes built-in support for the Teradata FastLoad protocol over prepared statements. FastLoad streams data directly into AMPs in parallel.

**FastLoad Requirements in Teradata:**
1. The target staging table **MUST be empty**.
2. Staging tables should NOT have secondary indexes or referential constraints.
3. Target table must have an explicit `PRIMARY INDEX`.

```python
import csv
import pandas as pd


def fastload_csv(conn, table_name: str, csv_path: str, date_cols: list[str] = None):
    """Bulk load CSV into an empty Teradata staging table using FastLoad protocol."""
    df = pd.read_csv(csv_path)
    
    # Format dates to standard Teradata 'YYYY-MM-DD' strings
    if date_cols:
        for col in date_cols:
            if col in df.columns:
                df[col] = pd.to_datetime(df[col]).dt.strftime('%Y-%m-%d')

    # Replace pandas NaN with None so driver binds SQL NULL
    df = df.where(pd.notnull(df), None)
    
    cols = list(df.columns)
    placeholders = ", ".join(["?" for _ in cols])
    col_names = ", ".join(cols)

    # Use {fn teradata_try_fastload} prefix to enable parallel wire FastLoad protocol
    sql = f"{{fn teradata_try_fastload}}INSERT INTO {table_name} ({col_names}) VALUES ({placeholders})"
    
    data = [tuple(x) for x in df.to_numpy()]
    
    cursor = conn.cursor()
    # Execute batch insertion
    cursor.executemany(sql, data)
    conn.commit()
    print(f"Successfully fastloaded {len(data):,} rows into {table_name}")
```

### Method B: Chunked Batch `executemany` (for Tables with Existing Rows)

If the table is not empty or FastLoad is not applicable:

```python
def batch_insert(conn, table_name: str, cols: list[str], rows: list[tuple], batch_size: int = 10000):
    """Insert rows in 10k chunks using parameterized executemany."""
    placeholders = ", ".join(["?" for _ in cols])
    sql = f"INSERT INTO {table_name} ({', '.join(cols)}) VALUES ({placeholders})"
    
    cursor = conn.cursor()
    for i in range(0, len(rows), batch_size):
        chunk = rows[i:i + batch_size]
        cursor.executemany(sql, chunk)
    conn.commit()
```

---

## 4. DDL & Primary Index Best Practices

### A. Idempotent DDL (Drop & Recreate)

When dropping tables in deployment scripts, handle error `3807` ("Object does not exist") gracefully:

```python
def safe_drop(cursor, table_name: str, object_type: str = "TABLE"):
    """Drop table or view if it exists without crashing on first run."""
    try:
        cursor.execute(f"DROP {object_type} {table_name}")
    except teradatasql.DatabaseError as e:
        # Error 3807 indicates object does not exist; safe to ignore
        if "3807" not in str(e):
            raise
```

### B. Choosing Primary Indexes (PI)

Every permanent Teradata table **must specify an explicit `PRIMARY INDEX (column)`**:
- Choose high-cardinality unique or natural keys (e.g. `order_id`, `cust_id`, `(order_id, line_number)`).
- **Never omit the Primary Index** (omitting PI creates a NoPI table, which degrades query performance).
- **Avoid low-cardinality columns** (e.g. `status`, `gender`, `mkt_segment`) as Primary Index, which causes severe AMP CPU/Perm space skew.

*Example DDL:*
```sql
CREATE TABLE stg_orders (
    order_id INTEGER NOT NULL,
    cust_id INTEGER,
    order_status CHAR(1),
    total_price DECIMAL(18,2),
    order_date DATE FORMAT 'YYYY-MM-DD',
    order_priority VARCHAR(20)
) PRIMARY INDEX (order_id);

CREATE TABLE stg_lineitem (
    order_id INTEGER NOT NULL,
    line_number INTEGER NOT NULL,
    part_id INTEGER,
    quantity INTEGER,
    extended_price DECIMAL(18,2),
    discount DECIMAL(5,2),
    tax DECIMAL(5,2),
    return_flag CHAR(1),
    line_status CHAR(1),
    ship_date DATE FORMAT 'YYYY-MM-DD',
    commit_date DATE FORMAT 'YYYY-MM-DD'
) PRIMARY INDEX (order_id, line_number);
```

---

## 5. In-Database ELT Transformations

Teradata is an MPP engine. **Do not download data into Python pandas to perform joins, rollups, or aggregations.** Execute the transformations in-database using `CREATE TABLE ... AS (SELECT ...) WITH DATA`:

```python
def execute_elt(cursor, prefix: str):
    # 1. Conformed Dimension
    cursor.execute(f"""
    CREATE TABLE {prefix}dim_customers AS (
        SELECT DISTINCT
            cust_id,
            cust_name,
            mkt_segment,
            nation_name
        FROM {prefix}stg_customers
    ) WITH DATA
    PRIMARY INDEX (cust_id);
    """)

    # 2. Atomic Fact Table
    cursor.execute(f"""
    CREATE TABLE {prefix}fct_order_lineitem AS (
        SELECT
            l.order_id,
            l.line_number,
            o.cust_id,
            l.part_id,
            o.order_date,
            l.ship_date,
            l.commit_date,
            o.order_status,
            o.order_priority,
            l.quantity,
            l.extended_price,
            l.discount,
            l.tax,
            ROUND(l.extended_price * (1 - l.discount), 2) AS net_revenue,
            ROUND(l.quantity * p.supply_cost, 2) AS supply_cost,
            ROUND(ROUND(l.extended_price * (1 - l.discount), 2) - ROUND(l.quantity * p.supply_cost, 2), 2) AS profit,
            (l.ship_date - o.order_date) AS shipping_delay_days
        FROM {prefix}stg_lineitem l
        JOIN {prefix}stg_orders o ON l.order_id = o.order_id
        JOIN {prefix}stg_parts p ON l.part_id = p.part_id
    ) WITH DATA
    PRIMARY INDEX (order_id, line_number);
    """)

    # 3. Monthly Aggregate Table
    cursor.execute(f"""
    CREATE TABLE {prefix}agg_monthly_performance AS (
        SELECT
            EXTRACT(YEAR FROM f.order_date) AS order_year,
            EXTRACT(MONTH FROM f.order_date) AS order_month,
            c.mkt_segment,
            COUNT(DISTINCT f.order_id) AS total_orders,
            SUM(f.quantity) AS total_items,
            SUM(f.net_revenue) AS total_net_revenue,
            SUM(f.profit) AS total_profit,
            AVG(CAST(f.shipping_delay_days AS FLOAT)) AS avg_shipping_delay_days
        FROM {prefix}fct_order_lineitem f
        JOIN {prefix}dim_customers c ON f.cust_id = c.cust_id
        GROUP BY 1, 2, 3
    ) WITH DATA
    PRIMARY INDEX (order_year, order_month, mkt_segment);
    """)
```

---

## 6. Verification and Reconciliation

Always run validation queries directly in Teradata to audit counts and metric integrity:

```python
def verify_pipeline(cursor, prefix: str):
    # Check row counts
    for tbl in ["stg_customers", "stg_orders", "fct_orders"]:
        cursor.execute(f"SELECT COUNT(*) FROM {prefix}{tbl}")
        cnt = cursor.fetchone()[0]
        print(f"Table {prefix}{tbl}: {cnt:,} rows")

    # Check metric totals
    cursor.execute(f"SELECT SUM(amount) FROM {prefix}fct_orders")
    row = cursor.fetchone()
    print(f"Total Amount: ${row[0]:,.2f}")
```

---

## 7. Complete End-to-End Pipeline Pattern

When designing an autonomous Python pipeline script (e.g. `run_pipeline.py`):
1. Import `teradatasql`, `pandas`, `os`, `urllib.parse`.
2. Establish connection using `get_teradata_connection()`.
3. Drop previous objects with `safe_drop()`.
4. Create staging tables with explicit `PRIMARY INDEX`.
5. Bulk load CSVs using `fastload_csv()` or batch `executemany()`.
6. Run in-database ELT SQL statements to build dimensions, facts, and aggregate tables.
7. Print audit metrics to stdout.
8. Close cursor and connection cleanly (`cursor.close(); conn.close()`).
