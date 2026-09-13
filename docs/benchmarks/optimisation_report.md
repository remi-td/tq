# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by
> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 29.15s | 48,248 | 0.0% | 29 | 0.400 | 5,089 | $0.0048 | $0.0025 | **$0.0073** |
| `gemini` | `gemini-2.5-flash` | `tq-with-skill` | 100.0% | 189.78s | 190,422 | 49.7% | 97 | 0.728 | 6,574 | $0.0105 | $0.0043 | **$0.0148** |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 88.41s | 185,892 | 0.0% | 210 | 1.656 | 15,523 | $0.0146 | $0.0098 | **$0.0244** |
| `claude-code` | `claude-haiku-4-5` | `tq-with-skill` | 100.0% | 182.99s | 1,559,632 | 97.3% | 125 | 1.624 | 11,218 | $0.2648 | $0.0092 | **$0.2741** |
| `claude-code` | `sonnet` | `baseline-python` | 100.0% | 109.18s | 799,428 | 95.3% | 28 | 0.456 | 4,541 | $0.3594 | $0.0027 | **$0.3622** |
| `claude-code` | `sonnet` | `tq-with-skill` | 100.0% | 131.26s | 1,021,577 | 96.3% | 121 | 0.972 | 9,432 | $0.3965 | $0.0058 | **$0.4023** |

---

## 2. Comparative Impact Analysis

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | -285.28% | -235.16% | 0.33x | 100.0% | 100.0% |
| `sonnet` | -27.79% | -11.09% | 0.83x | 100.0% | 100.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (190422 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 2. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 1559632 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-2.5-flash_tq-with-skill_1789329297`
- **Harness / Model:** `gemini` / `gemini-2.5-flash` (tq-with-skill)
- **Duration:** 189.78s | **Score:** 100.0%
- **Tokens:** 190,422 (Input: 88,905, Output: 6,933, Cache Read: 94,584)
- **Database Telemetry (DBQL):** Queries: 97, Errors: 12, AMP CPU: 0.728s, I/O: 6,574, Peak Spool: 65,536 bytes
- **Total Effort Cost:** **$0.0148**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_29297_GE_STG_CUSTOMERS', 'B_29297_GE_STG_PARTS', 'B_29297_GE_STG_ORDERS', 'B_29297_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_29297_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_29297_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_29297_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_29297_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789329506`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 88.41s | **Score:** 100.0%
- **Tokens:** 185,892 (Input: 183,108, Output: 2,784, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 210, Errors: 33, AMP CPU: 1.656s, I/O: 15,523, Peak Spool: 393,216 bytes
- **Total Effort Cost:** **$0.0244**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_29506_GE_STG_CUSTOMERS', 'B_29506_GE_STG_PARTS', 'B_29506_GE_STG_ORDERS', 'B_29506_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_29506_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_29506_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_29506_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_29506_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_claude-haiku-4-5_tq-with-skill_1789329607`
- **Harness / Model:** `claude-code` / `claude-haiku-4-5` (tq-with-skill)
- **Duration:** 182.99s | **Score:** 100.0%
- **Tokens:** 1,559,632 (Input: 307, Output: 8,603, Cache Read: 1,516,956)
- **Database Telemetry (DBQL):** Queries: 125, Errors: 16, AMP CPU: 1.624s, I/O: 11,218, Peak Spool: 262,144 bytes
- **Total Effort Cost:** **$0.2741**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_29607_CL_STG_CUSTOMERS', 'B_29607_CL_STG_PARTS', 'B_29607_CL_STG_ORDERS', 'B_29607_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_29607_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_29607_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_29607_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_29607_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_sonnet_tq-with-skill_1789329820`
- **Harness / Model:** `claude-code` / `sonnet` (tq-with-skill)
- **Duration:** 131.26s | **Score:** 100.0%
- **Tokens:** 1,021,577 (Input: 38, Output: 7,913, Cache Read: 984,071)
- **Database Telemetry (DBQL):** Queries: 121, Errors: 17, AMP CPU: 0.972s, I/O: 9,432, Peak Spool: 262,144 bytes
- **Total Effort Cost:** **$0.4023**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_29820_CL_STG_CUSTOMERS', 'B_29820_CL_STG_PARTS', 'B_29820_CL_STG_ORDERS', 'B_29820_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_29820_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_29820_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_29820_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_29820_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789329964`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 29.15s | **Score:** 100.0%
- **Tokens:** 48,248 (Input: 43,125, Output: 5,123, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 29, Errors: 8, AMP CPU: 0.400s, I/O: 5,089, Peak Spool: 262,144 bytes
- **Total Effort Cost:** **$0.0073**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_29964_GE_STG_CUSTOMERS', 'B_29964_GE_STG_PARTS', 'B_29964_GE_STG_ORDERS', 'B_29964_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_29964_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_29964_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_29964_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_29964_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_sonnet_baseline-python_1789330006`
- **Harness / Model:** `claude-code` / `sonnet` (baseline-python)
- **Duration:** 109.18s | **Score:** 100.0%
- **Tokens:** 799,428 (Input: 36, Output: 9,287, Cache Read: 762,161)
- **Database Telemetry (DBQL):** Queries: 28, Errors: 8, AMP CPU: 0.456s, I/O: 4,541, Peak Spool: 262,144 bytes
- **Total Effort Cost:** **$0.3622**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_30006_CL_STG_CUSTOMERS', 'B_30006_CL_STG_PARTS', 'B_30006_CL_STG_ORDERS', 'B_30006_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_30006_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_30006_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_30006_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_30006_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
