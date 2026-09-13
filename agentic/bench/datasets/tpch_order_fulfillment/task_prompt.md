# Data Engineering Benchmark Task: Customer Order Fulfillment & Profitability Data Product

You are a Senior Data Engineer tasked with building an enterprise analytical Data Product on Teradata.
All seed data files are located in the `seed_data/` directory:
- `seed_data/stg_customers.csv`
- `seed_data/stg_parts.csv`
- `seed_data/stg_orders.csv`
- `seed_data/stg_lineitem.csv`

## Environment & Object Naming
To maintain workspace isolation, all tables and views created for this run MUST use the prefix: `{table_prefix}`
Target database is the current default user database. Connection details are available in standard environment variables (e.g. DATABASE_URI, TERADATA_HOST).
Note: In Teradata, execute each DDL statement individually.

---

## Your Deliverables

### Layer 1: Ingestion & Staging
Create Teradata staging tables and load the raw seed datasets:
1. `{table_prefix}stg_customers` (from `seed_data/stg_customers.csv`)
2. `{table_prefix}stg_parts` (from `seed_data/stg_parts.csv`)
3. `{table_prefix}stg_orders` (from `seed_data/stg_orders.csv`)
4. `{table_prefix}stg_lineitem` (from `seed_data/stg_lineitem.csv`)

Explore the source data to determine appropriate data types and Primary Index (PI) choices.

### Layer 2: Dimensional & Atomic Fact Modeling
Build the conformed dimension table:
- `{table_prefix}dim_customers`:
  - Contains: `cust_id`, `cust_name`, `mkt_segment`, `nation_name`
  - Choose an optimal Primary Index for customer lookups.

Build the atomic lineitem fact table:
- `{table_prefix}fct_order_lineitem`:
  - Join lineitems with orders and parts.
  - Retain natural keys and dimensional attributes: `order_id`, `line_number`, `cust_id`, `part_id`, `order_date`, `ship_date`, `commit_date`, `order_status`, `order_priority`, `quantity`, `extended_price`, `discount`, `tax`.
  - Calculate business metrics:
    - `net_revenue` = `ROUND(extended_price * (1 - discount), 2)`
    - `supply_cost` = `ROUND(quantity * supply_cost, 2)` (using `supply_cost` from parts)
    - `profit` = `ROUND(net_revenue - supply_cost, 2)`
    - `shipping_delay_days` = `(ship_date - order_date)`
  - Choose an optimal Primary Index for join and query distribution performance.

### Layer 3: Aggregation & Semantic Analytics
Build the periodic summary aggregate table:
- `{table_prefix}agg_monthly_performance`:
  - Roll up performance at the Year, Month, and Market Segment grain.
  - Grouping columns:
    - `order_year` (extracted from `order_date`)
    - `order_month` (extracted from `order_date`)
    - `mkt_segment` (from customer)
  - Metrics:
    - `total_orders` = `COUNT(DISTINCT order_id)`
    - `total_items` = `SUM(quantity)`
    - `total_net_revenue` = `SUM(net_revenue)`
    - `total_profit` = `SUM(profit)`
    - `avg_shipping_delay_days` = `AVG(shipping_delay_days)`

Build the customer profitability dimension table or view:
- `{table_prefix}dim_customer_profitability`:
  - Customer-level summary joining customer details with lifetime metrics:
  - Columns: `cust_id`, `cust_name`, `mkt_segment`, `nation_name`
  - Metrics:
    - `lifetime_orders` = `COUNT(DISTINCT order_id)`
    - `lifetime_items` = `SUM(quantity)`
    - `lifetime_net_revenue` = `SUM(net_revenue)`
    - `lifetime_profit` = `SUM(profit)`
    - `profit_margin_pct` = `ROUND((lifetime_profit / NULLIF(lifetime_net_revenue, 0)) * 100, 2)`

---

### Layer 4: Verification & Audit
Run audit queries to verify the completed data product:
1. Verify row counts across `{table_prefix}stg_customers`, `{table_prefix}stg_parts`, `{table_prefix}stg_orders`, and `{table_prefix}stg_lineitem`.
2. Compute the overall sum of `net_revenue` and `profit` across `{table_prefix}fct_order_lineitem`.
3. Verify reconciliation between `{table_prefix}fct_order_lineitem` and `{table_prefix}agg_monthly_performance` (sum of `total_net_revenue`).
4. Query the Top 5 customers by `lifetime_profit` from `{table_prefix}dim_customer_profitability`.

Conclude with a brief summary of completed objects and verified metrics.
