# Agentic Data Engineering Benchmark Specification

This specification defines the industry-standard benchmark methodology, dataset requirements, task contracts, and verification criteria for evaluating autonomous coding agents performing end-to-end data platform engineering.

---

## 1. Domain Background & Industry Dataset Research

### 1.1 Limitations of Existing Benchmarks

Traditional AI benchmarks evaluate code generation in isolation (e.g., HumanEval, MBPP) or simple single-shot query answering (e.g., Spider, BIRD, WikiSQL):
- **Spider / BIRD**: Test text-to-SQL translation for read-only `SELECT` queries on pre-loaded SQLite databases. They completely miss the complexity of enterprise data engineering: data ingestion, type casting, schema evolution, primary index distribution, ETL/ELT pipeline design, multi-table reconciliation, and database resource management.
- **SWE-bench**: Evaluates bug fixes on GitHub repositories in isolated Python environments, but lacks data platform interactions, distributed MPP execution, or analytical workloads.

### 1.2 Criteria for Modern Data Engineering Benchmarks

A representative, industry-leading benchmark for autonomous data agents must satisfy six core criteria:

1. **End-to-End Lifecycle**: The agent must start from raw semi-structured or tabular extracts (`seed_data/*.csv`), inspect data characteristics, create staging environments, design dimensional schemas (Fact and Dimension tables), implement ELT transformation logic, run aggregations, and reconcile results.
2. **Distributed MPP Architecture Awareness**: Real-world data warehouses (such as Teradata Vantage) rely on hash partitioning and Massively Parallel Processing (MPP). Agents must make deliberate architectural decisions regarding **Primary Indexes (PI)** to prevent data skew and avoid join redistribution.
3. **Data Volume & Throughput Diversity**: Workloads must span both agile prototyping (small seed data, 50-500 rows) and high-throughput streaming/bulk ingestion (50,000+ rows) where single-row SQL inserts fail and dedicated bulk utilities (e.g., Teradata FastLoad) are required.
4. **Idempotency & Resilience**: Pipeline scripts must handle schema drops, non-existent objects, and multi-statement transactional semantics without crashing.
5. **Multi-Turn Autonomous Execution**: The agent must explore the filesystem, run schema probes, execute pipeline scripts, inspect outputs, debug errors, and produce reconciliation reports across multiple turns.
6. **Unified Effort Cost Metric**: The framework must track total cost to complete the task:
   $$\text{Total Effort Cost} = \text{Token Cost (Input, Output, Cache Reads/Writes)} + \text{Database Resource Utilization (AMP CPU, I/O, Spool)}$$

---

## 2. Standard Benchmark Datasets

The framework standardizes on three core domain datasets spanning distinct enterprise data patterns:

### 2.1 Dataset A: TPC-H Order Fulfillment Analytics (`tpch_order_fulfillment`)

Derived from the Transaction Processing Performance Council (TPC) TPC-H benchmark—the global gold standard for decision support systems.

* **Domain**: Supply chain, order management, and customer logistics.
* **Source Tables**:
  - `stg_customers`: Customer profiles, market segments, credit balances.
  - `stg_orders`: Order headers, order dates, priority classifications, order status.
  - `stg_lineitem`: Order items, quantities, extended prices, discounts, tax, shipping dates, return flags.
  - `stg_parts`: Catalog items, manufacturer details, retail prices.
* **Target Data Products**:
  1. `dim_customers`: Cleaned customer dimension with segment classifications.
  2. `fct_order_lineitem`: Enriched transactional fact table calculating:
     - `net_revenue = extended_price * (1 - discount)`
     - `gross_revenue = extended_price`
     - `profit = net_revenue - (extended_price * 0.70)`
     - `shipping_delay_days = ship_date - order_date`
  3. `agg_monthly_performance`: Monthly business summary grouped by year/month and order priority, aggregating total revenue, profit, item count, and average shipping delay.
  4. `dim_customer_profitability`: Customer profitability ranking and lifetime value.

### 2.2 Dataset B: Multi-Tenant B2B SaaS Metering & Billing (`b2b_saas_billing`)

Reflects modern cloud subscription businesses (e.g., Snowflake, Datadog, Stripe) with usage-based pricing models.

* **Domain**: Product-led growth, API event metering, overage billing, tiered subscription margins.
* **Source Tables**:
  - `stg_accounts`: Organization tier, contract renewal date, monthly base commit.
  - `stg_plans`: Subscription tiers, included unit quotas, tier multipliers.
  - `stg_features`: Feature catalog, unit infrastructure cost, overage unit rate.
  - `stg_usage_events`: High-cardinality usage stream (account ID, feature ID, consumed units, event timestamp).
* **Target Data Products**:
  1. `dim_accounts`: Enriched tenant dimension.
  2. `fct_billed_usage`: Reconciled billable event transactions calculating consumed volume, quota thresholding, and overage charges.
  3. `agg_account_monthly_invoice`: Monthly invoice summary per account.
  4. `dim_account_usage_kpi`: Usage KPI summary tracking gross revenue, infrastructure cost, and net margin per customer.

### 2.3 Dataset C: High-Volume Scale Benchmark (`tpch_50k`)

Stress-tests data volume scaling and batch tool efficiency on 50,000+ transactional lineitems.

* **Domain**: High-throughput transactional data loading.
* **Requirement**: Forces agents to recognize that standard single-row SQL inserts or naive CSV loops are prohibitively slow, validating the agent's ability to choose high-speed block ingestion utilities (`tq fastload`).

### 2.4 Dataset D (Flagship): Enterprise Omnichannel Data Platform 360 (`enterprise_data_platform_360`)

The premier, next-generation flagship benchmark reflecting a complete end-to-end data engineering lifecycle:

* **Domain**: Omnichannel enterprise retail (orders, items, customers, products, sales channels).
* **Workload Volume**: 25,000+ records across 5 entities (`stg_channels`, `stg_products`, `stg_customers`, `stg_orders`, `stg_order_lineitems`).
* **Injected Data Quality (DQ) Challenges**:
  - **Whitespace & Dirty Casing**: Customer tiers contain messy casing (`' gold'`, `'Gold'`, `'GOLD '`).
  - **Mixed Date Formats**: Order dates contain both ISO `YYYY-MM-DD` and slash-delimited `YYYY/MM/DD`.
  - **Duplicate Sync Events**: CDC replay duplicates injected into customer and lineitem feeds requiring window deduplication (`QUALIFY ROW_NUMBER() OVER (...) = 1`).
  - **Null Imputation**: Incomplete delivery dates on cancelled orders; missing discount percentages.
* **Teradata MPP Optimization**:
  - Requires deliberate **Primary Index (PI)** selection (`(customer_id)`, `(product_id)`, `(order_id)`) to eliminate cross-AMP join redistribution and prevent CPU hotspot skew.
* **Formal Semantic Layer**:
  - `v_sem_order_fulfillment`: Computes Gross Sales, Net Revenue, COGS, Gross Profit, Gross Margin %, and On-Time In-Full (OTIF) fulfillment rate.
  - `v_sem_customer_rfm_kpis`: Recency, Frequency, Monetary (RFM) analysis and customer lifetime value.
  - `v_sem_channel_performance`: Channel profitability and return rates.
* **Analytical Querying & Deliverable**:
  - Agent queries semantic views to answer executive business questions and delivers `DATA_PRODUCT_INSIGHTS.md`.

---

## 3. Agent Deliverable Contracts

For any benchmark scenario, the autonomous agent is placed in an isolated sandbox workspace containing `seed_data/*.csv` and the scenario task prompt. The agent must autonomously execute and produce the following concrete deliverables:

```
workspace/
├── seed_data/                    # Source CSV extracts provided by harness
│   ├── stg_customers.csv
│   ├── stg_orders.csv
│   └── ...
├── build_data_product.sql         # Idempotent ELT DDL & DML script
├── audit_reconciliation.sql       # Checksum reconciliation script
└── DATA_MART_SUMMARY.md           # Business & technical deliverable documentation
```

### 3.1 Contract Requirements

1. **Table Prefix Isolation**: All created tables and views must be prefixed with the assigned run token (e.g., `b_run42_stg_customers`, `b_run42_fct_order_lineitem`).
2. **Idempotent Migration**: Scripts must be safely re-runnable (dropping existing objects with warning tolerance).
3. **Primary Index Optimization**: Dimension and fact tables must declare explicit Primary Indexes (`PRIMARY INDEX (column)`) aligned with join foreign keys to ensure uniform AMP data distribution.
4. **Precision & Accuracy**: Financial calculations (revenue, profit, tax, overages) must use exact decimal types (`DECIMAL(18, 2)`), preventing floating-point precision loss.

---

## 4. Verification & Validation Protocol

The framework verifies agent completion using an 8-point automated validation assertion suite executed directly against the live database:

| Assertion # | Target Object | Verification Check | Passing Criteria |
| :---: | :--- | :--- | :--- |
| **A1** | Staging Tables | Table existence | All source staging tables created |
| **A2** | Staging Rows | Row count integrity | Staging row count matches source CSVs exactly |
| **A3** | Dimension Table | Table existence & PI | `dim_*` exists with valid Primary Index |
| **A4** | Fact Table | Table existence & PI | `fct_*` exists with valid Primary Index |
| **A5** | Fact Rows | Lineitem count | Fact table matches transactional row count |
| **A6** | Aggregate Object | Table/View existence | Monthly summary / KPI aggregate exists |
| **A7** | Financial KPI (Revenue) | Exact sum check | `SUM(net_revenue)` matches ground truth within 0.01 |
| **A8** | Operational KPI (Metrics)| Metric precision | Calculated delays / overages match ground truth |

The overall validation score is:
$$\text{Validation Score} = \left(\frac{\sum \text{Passed Assertions}}{\text{Total Assertions}}\right) \times 100\%$$

A score of **100.0%** is mandatory for a successful benchmark run.

---

## 5. Comparative Evaluation Modes

To isolate the exact productivity, cost, and quality impact of the `tq` CLI and companion skills, each model must be benchmarked across four execution modes:

1. **`tq-with-skill`**: The agent has access to the `tq` CLI and the companion `teradata-query` agent skill.
2. **`tq-no-skill`**: The agent has access to the `tq` CLI, but no domain skill prompt. Measures raw tool discoverability.
3. **`baseline-python`**: The `tq` binary is removed from the agent's PATH. The agent must use standard Python (`teradatasql`, `pandas`, standard library). Measures the acceleration of `tq` over the industry baseline.
4. **`teradata-mcp`**: The agent interacts with the database exclusively via the Teradata Model Context Protocol (MCP) server. Measures CLI vs MCP ergonomics and token consumption.
