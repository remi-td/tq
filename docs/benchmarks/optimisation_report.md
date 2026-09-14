# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by
> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 39.8s | 40,595 | 0.0% | 113 | 0.820 | 9,486 | $0.0042 | $0.0050 | **$0.0093** |

---

## 2. Comparative Impact Analysis

---

## 3. Actionable Optimization Recommendations

### 1. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 40595 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

### 2. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (40595 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789366944`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 39.8s | **Score:** 100.0%
- **Tokens:** 40,595 (Input: 35,436, Output: 5,159, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 113, Errors: 16, AMP CPU: 0.820s, I/O: 9,486, Peak Spool: 262,144 bytes
- **Total Effort Cost:** **$0.0093**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_66944_GE_STG_CUSTOMERS', 'B_66944_GE_STG_PARTS', 'B_66944_GE_STG_ORDERS', 'B_66944_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_66944_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_66944_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_66944_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_66944_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
