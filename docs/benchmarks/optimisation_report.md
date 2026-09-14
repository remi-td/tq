# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by
> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 55.3s | 101,515 | 0.0% | 147 | 3.760 | 10,232 | $0.0085 | $0.0198 | **$0.0284** |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 0.0% | 27.57s | 47,070 | 0.0% | 3 | 0.024 | 164 | $0.0050 | $0.0001 | **$0.0052** |

---

## 2. Comparative Impact Analysis

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | -115.67% | -448.83% | 0.5x | 100.0% | 0.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 101515 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

### 2. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (101515 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 3. [CLI_FEATURE] tq CLI error handling
- **Identified Issue:** Run encountered errors or partial completion (Score: 0.0%). Details: Missing assertions
- **Observed Impact:** Causes agent to enter expensive retry loops, multiplying token and runtime costs.
- **Recommended Action:** Enhance error messaging in `tq query` with explicit remediation hints (e.g. Teradata 3932 DDL/BTET statement separation).

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789364085`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 55.3s | **Score:** 100.0%
- **Tokens:** 101,515 (Input: 97,419, Output: 4,096, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 147, Errors: 16, AMP CPU: 3.760s, I/O: 10,232, Peak Spool: 10,096,640 bytes
- **Total Effort Cost:** **$0.0284**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_64085_GE_STG_CUSTOMERS', 'B_64085_GE_STG_PARTS', 'B_64085_GE_STG_ORDERS', 'B_64085_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_64085_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_64085_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_64085_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_64085_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 17,631,394.11, got 17,631,408.46 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789364160`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 27.57s | **Score:** 0.0%
- **Tokens:** 47,070 (Input: 40,401, Output: 6,669, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 3, Errors: 0, AMP CPU: 0.024s, I/O: 164, Peak Spool: 0 bytes
- **Total Effort Cost:** **$0.0052**
- **Assertion Details:**
  - ❌ FAIL: **stg_tables_exist** - Found 0/4 staging tables: []
  - ❌ FAIL: **dim_customers_created** - Table B_64160_GE_DIM_CUSTOMERS exists: False
  - ❌ FAIL: **fct_order_lineitem_created** - Table B_64160_GE_FCT_ORDER_LINEITEM exists: False
  - ❌ FAIL: **agg_monthly_performance_created** - Table B_64160_GE_AGG_MONTHLY_PERFORMANCE exists: False
  - ❌ FAIL: **dim_customer_profitability_created** - Table/View B_64160_GE_DIM_CUSTOMER_PROFITABILITY exists: False
  - ❌ FAIL: **revenue_integrity_checksum** - Failed query on b_64160_ge_fct_order_lineitem: 
  - ❌ FAIL: **profit_integrity_checksum** - Failed query on b_64160_ge_fct_order_lineitem: 
  - ❌ FAIL: **aggregate_reconciliation_checksum** - Failed query on b_64160_ge_agg_monthly_performance: 
