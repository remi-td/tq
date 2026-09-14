# Data Engineering Benchmark Task: B2B SaaS Cloud Metered Billing & Margin Analytics Data Product

You are a Senior Data Engineer tasked with building an enterprise analytical Data Product on Teradata.
All seed data files are located in the `seed_data/` directory:
- `seed_data/stg_accounts.csv`
- `seed_data/stg_plans.csv`
- `seed_data/stg_features.csv`
- `seed_data/stg_usage_events.csv`

## Environment & Object Naming
To maintain workspace isolation, all tables and views created for this run MUST use the prefix: `{table_prefix}`
Target database is the current default user database. Connection details are available in standard environment variables (e.g. DATABASE_URI, TERADATA_HOST).

---

## Your Deliverables

### Layer 1: Ingestion & Staging
Create Teradata staging tables and load the raw seed datasets:
1. `{table_prefix}stg_accounts` (from `seed_data/stg_accounts.csv`)
2. `{table_prefix}stg_plans` (from `seed_data/stg_plans.csv`)
3. `{table_prefix}stg_features` (from `seed_data/stg_features.csv`)
4. `{table_prefix}stg_usage_events` (from `seed_data/stg_usage_events.csv`)

Explore the source data to determine appropriate column data types and Primary Index (PI) choices.

### Layer 2: Dimensional & Atomic Fact Modeling
Build the conformed customer dimension table:
- `{table_prefix}dim_accounts`:
  - Contains: `account_id`, `company_name`, `industry`, `pricing_tier`, `billing_cycle`, `signup_date`
  - Choose an optimal Primary Index for account lookups.

Build the atomic metered usage fact table:
- `{table_prefix}fct_metered_usage`:
  - Join usage events with accounts, plans, and features.
  - Retain natural keys and dimensional attributes: `event_id`, `account_id`, `feature_id`, `event_timestamp`, `region`, `consumed_units`, `pricing_tier`, `service_category`.
  - Calculate business financial metrics:
    - `gross_billed_amount` = `ROUND(consumed_units * overage_unit_rate, 2)`
    - `infrastructure_cost` = `ROUND(consumed_units * unit_infra_cost, 2)`
    - `net_margin` = `ROUND(gross_billed_amount - infrastructure_cost, 2)`
  - Choose an optimal Primary Index for join and query distribution performance.

### Layer 3: Aggregation & Semantic Analytics
Build the periodic summary aggregate table:
- `{table_prefix}agg_monthly_billing`:
  - Roll up metered billing performance at the Year, Month, Pricing Tier, and Industry grain.
  - Grouping columns:
    - `billing_year` (extracted from `event_timestamp`)
    - `billing_month` (extracted from `event_timestamp`)
    - `pricing_tier` (from accounts)
    - `industry` (from accounts)
  - Metrics:
    - `active_accounts` = `COUNT(DISTINCT account_id)`
    - `total_events` = `COUNT(*)`
    - `total_consumed_units` = `SUM(consumed_units)`
    - `total_billed_amount` = `SUM(gross_billed_amount)`
    - `total_infra_cost` = `SUM(infrastructure_cost)`
    - `total_net_margin` = `SUM(net_margin)`
    - `avg_margin_pct` = `ROUND((total_net_margin / NULLIF(total_billed_amount, 0)) * 100, 2)`

Build the customer value dimension table or view:
- `{table_prefix}dim_account_usage_kpi`:
  - Account-level summary joining account details with lifetime consumption and margin metrics:
  - Columns: `account_id`, `company_name`, `industry`, `pricing_tier`
  - Metrics:
    - `lifetime_events` = `COUNT(*)`
    - `lifetime_consumed_units` = `SUM(consumed_units)`
    - `lifetime_billed_amount` = `SUM(gross_billed_amount)`
    - `lifetime_net_margin` = `SUM(net_margin)`
    - `margin_pct` = `ROUND((lifetime_net_margin / NULLIF(lifetime_billed_amount, 0)) * 100, 2)`

---

### Layer 4: Verification & Audit
Run audit queries to verify the completed data product:
1. Verify row counts across `{table_prefix}stg_accounts`, `{table_prefix}stg_plans`, `{table_prefix}stg_features`, and `{table_prefix}stg_usage_events`.
2. Compute the overall sum of `gross_billed_amount` and `net_margin` across `{table_prefix}fct_metered_usage`.
3. Verify reconciliation between `{table_prefix}fct_metered_usage` and `{table_prefix}agg_monthly_billing` (sum of `total_billed_amount`).
4. Query the Top 5 accounts by `lifetime_billed_amount` from `{table_prefix}dim_account_usage_kpi`.

Conclude with a brief summary of completed objects and verified metrics.
