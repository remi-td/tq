# Mission: Build Enterprise Omnichannel Retail Data Platform 360

You are an expert Data Platform Engineer building an end-to-end analytical Data Product on **Teradata Vantage** for an omnichannel enterprise retailer.

You are provided with raw seed CSV data extracts in the local `seed_data/` directory.

All database tables and views you create **MUST** be prefixed with: `{{TABLE_PREFIX}}`
(e.g., `{{TABLE_PREFIX}}stg_customers`, `{{TABLE_PREFIX}}fct_order_lineitems`, `{{TABLE_PREFIX}}v_sem_order_fulfillment`).

---

## The End-to-End Workflow

You must execute a complete 5-phase data engineering lifecycle:

### Phase 1: High-Volume Ingestion & Staging
Load the 5 raw CSV extracts in `seed_data/` into Teradata staging tables:
1. `seed_data/stg_channels.csv` -> `{{TABLE_PREFIX}}stg_channels`
2. `seed_data/stg_products.csv` -> `{{TABLE_PREFIX}}stg_products`
3. `seed_data/stg_customers.csv` -> `{{TABLE_PREFIX}}stg_customers`
4. `seed_data/stg_orders.csv` -> `{{TABLE_PREFIX}}stg_orders`
5. `seed_data/stg_order_lineitems.csv` -> `{{TABLE_PREFIX}}stg_order_lineitems` (~20,000+ records)

*Hint*: Use high-throughput streaming bulk loader `tq fastload` for instantaneous loading.

### Phase 2: Data Quality & Profiling
Explore and profile the ingested data. Note that enterprise source data contains real-world imperfections:
- **Customer Tiers**: Staged with dirty casing and leading/trailing whitespace (e.g. `' gold'`, `'Gold'`, `'GOLD '`). Cleanse using `UPPER(TRIM(customer_tier))`.
- **Order Dates**: Contain mixed date formats (`YYYY-MM-DD` and `YYYY/MM/DD`). Standardize into clean Teradata `DATE`.
- **Duplicate Records**: Ingestion feeds contain duplicate sync events. Deduplicate customer records on `customer_id` and lineitem records on `lineitem_id` (e.g., via `QUALIFY ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...) = 1`).
- **Null Handling**: Incomplete delivery dates on cancelled orders; handle nulls gracefully.

*Hint*: Use `tq peek`, `tq inspect`, or queries with `--budget` to profile without flooding the LLM context.

### Phase 3: Reusable Dimensional Modeling & Teradata MPP Optimization
Design and deploy clean, conformed dimensional and atomic fact tables:
1. `{{TABLE_PREFIX}}dim_customers`:
   - Unique, deduplicated customer records (exactly 1,000 rows).
   - Standardized tier (`STANDARD`, `SILVER`, `GOLD`, `PLATINUM`).
   - Declared with: `PRIMARY INDEX (customer_id)` to distribute evenly across AMPs.
2. `{{TABLE_PREFIX}}dim_products`:
   - Product catalog with SKU, category, manufacturer, unit cost, and list price.
   - Declared with: `PRIMARY INDEX (product_id)`.
3. `{{TABLE_PREFIX}}dim_channels`:
   - Lookup dimension for sales channels.
4. `{{TABLE_PREFIX}}fct_orders`:
   - Grain: One row per order header.
   - Declared with: `PRIMARY INDEX (order_id)`.
5. `{{TABLE_PREFIX}}fct_order_lineitems`:
   - Grain: One row per unique lineitem (deduplicated).
   - Calculated fields:
     - `gross_sales_usd = quantity * unit_price`
     - `discount_amount_usd = quantity * unit_price * discount_pct`
     - `net_revenue_usd = (quantity * unit_price) - (quantity * unit_price * discount_pct)`
     - `cogs_usd = quantity * unit_cost`
     - `gross_profit_usd = net_revenue_usd - cogs_usd`
   - Declared with: `PRIMARY INDEX (order_id)` to match the join key of the orders fact and eliminate AMP redistributions.

### Phase 4: Enterprise Semantic Layer Generation
Create reusable Teradata database views implementing standard business metric definitions:
1. `{{TABLE_PREFIX}}v_sem_order_fulfillment`:
   - Joins `fct_order_lineitems`, `fct_orders`, `dim_products`, `dim_customers`, and `dim_channels`.
   - Metrics:
     - `gross_sales_usd`, `discount_amount_usd`, `net_revenue_usd`, `cogs_usd`, `gross_profit_usd`
     - `gross_margin_pct = (gross_profit_usd / NULLIFZERO(net_revenue_usd)) * 100`
     - `is_otif = CASE WHEN fulfillment_status = 'DELIVERED' AND delivery_date <= promised_date AND return_flag = 'N' THEN 1 ELSE 0 END`
     - `shipping_delay_days = (ship_date - order_date)`
2. `{{TABLE_PREFIX}}v_sem_customer_rfm_kpis`:
   - Customer-level summary aggregating total orders, total net revenue, total gross profit, and average order value.
3. `{{TABLE_PREFIX}}v_sem_channel_performance`:
   - Channel-level summary comparing total net revenue, total gross profit, OTIF delivery rate %, and return rate %.

### Phase 5: Analytical Querying & Executive Insights
Execute analytical queries against your semantic views to answer key business questions:
1. What is the enterprise-wide On-Time In-Full (OTIF) rate % and overall Gross Profit Margin %?
2. Which sales channel generates the highest total net revenue, and what is its OTIF rate?
3. Which product category generates the highest total gross profit?

Document your findings and reconciliation summary in an executive report file:
`DATA_PRODUCT_INSIGHTS.md`

---

## Deliverables Checklist
1. All staging tables created and loaded.
2. All dimensional and fact tables created with optimal Teradata Primary Indexes.
3. All 3 semantic layer views (`v_sem_*`) deployed and active.
4. `DATA_PRODUCT_INSIGHTS.md` created with verified answers to executive business questions.
