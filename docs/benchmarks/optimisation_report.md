# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by
> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 38.44s | 33,226 | 0.0% | 113 | 3.256 | 9,536 | $0.0035 | $0.0172 | **$0.0208** |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 51.49s | 110,749 | 0.0% | 141 | 3.996 | 9,972 | $0.0092 | $0.0210 | **$0.0301** |

---

## 2. Comparative Impact Analysis

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | -233.32% | -45.09% | 0.75x | 100.0% | 100.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (110749 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 2. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 33226 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789366794`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 51.49s | **Score:** 100.0%
- **Tokens:** 110,749 (Input: 106,924, Output: 3,825, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 141, Errors: 16, AMP CPU: 3.996s, I/O: 9,972, Peak Spool: 10,108,928 bytes
- **Total Effort Cost:** **$0.0301**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_66794_GE_STG_CUSTOMERS', 'B_66794_GE_STG_PARTS', 'B_66794_GE_STG_ORDERS', 'B_66794_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_66794_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_66794_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_66794_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_66794_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 17,631,394.11, got 17,631,408.46 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789366860`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 38.44s | **Score:** 100.0%
- **Tokens:** 33,226 (Input: 28,560, Output: 4,666, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 113, Errors: 16, AMP CPU: 3.256s, I/O: 9,536, Peak Spool: 13,111,296 bytes
- **Total Effort Cost:** **$0.0208**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_66860_GE_STG_CUSTOMERS', 'B_66860_GE_STG_PARTS', 'B_66860_GE_STG_ORDERS', 'B_66860_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_66860_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_66860_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_66860_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_66860_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 17,631,394.11, got 17,631,408.46 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 54,249,318.93, got 54,249,333.28 (diff: 0.0000%)
