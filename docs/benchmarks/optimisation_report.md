# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It tracks LLM token consumption and cost along with
> database execution performance telemetry (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | **Token Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `claude-code` | `claude-haiku-4-5` | `tq-no-skill` | 100.0% | 300.03s | 0 | 0.0% | 226 | 2.180 | 20,450 | **$0.0000** |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 36.22s | 27,211 | 0.0% | 115 | 1.020 | 9,574 | **$0.0030** |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 44.08s | 32,525 | 0.0% | 137 | 1.068 | 9,050 | **$0.0031** |
| `gemini` | `gemini-2.5-flash` | `tq-with-skill` | 100.0% | 70.9s | 104,769 | 60.7% | 141 | 1.232 | 9,192 | **$0.0049** |
| `codex` | `luna` | `baseline-python` | 100.0% | 154.57s | 263,886 | 85.1% | 114 | 0.896 | 9,582 | **$0.0188** |
| `codex` | `luna` | `tq-with-skill` | 100.0% | 204.17s | 615,466 | 91.3% | 266 | 3.324 | 23,199 | **$0.0300** |
| `claude-code` | `claude-haiku-4-5` | `baseline-python` | 100.0% | 102.58s | 359,525 | 88.0% | 220 | 1.992 | 20,078 | **$0.1462** |
| `codex` | `terra` | `baseline-python` | 100.0% | 150.34s | 301,869 | 86.6% | 122 | 0.960 | 10,690 | **$0.1908** |
| `claude-code` | `claude-haiku-4-5` | `tq-with-skill` | 100.0% | 356.17s | 771,402 | 93.7% | 195 | 2.072 | 17,581 | **$0.2083** |
| `codex` | `terra` | `tq-with-skill` | 100.0% | 182.2s | 475,377 | 89.6% | 201 | 1.804 | 14,881 | **$0.2417** |
| `claude-code` | `sonnet` | `tq-with-skill` | 100.0% | 67.88s | 303,885 | 83.8% | 139 | 0.972 | 9,059 | **$0.2728** |
| `claude-code` | `sonnet` | `baseline-python` | 100.0% | 58.96s | 385,533 | 89.0% | 31 | 0.412 | 5,185 | **$0.2772** |
| `gemini` | `gemini-2.5-flash` | `baseline-python` | 0.0% | 20.48s | 4,366 | 0.0% | 1 | 0.004 | 6 | **$0.0003** |
| `gemini` | `gemini-2.5-flash` | `tq-no-skill` | 0.0% | 52.09s | 20,964 | 29.3% | 1 | 0.000 | 6 | **$0.0025** |

---

## 2. Comparative Impact Analysis

### A. Skill Impact (`tq-with-skill` vs `tq-no-skill`)

| Model | Token Savings % | Token Cost Savings % | DB CPU Savings % | Accuracy Delta | Advantage |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-2.5-flash` | -399.76% | -96.75% | -123100.0% | 100.0% vs 0.0% | **neutral_or_negative** |
| `claude-haiku-4-5` | -77140100.0% | -20830000.0% | 4.95% | 100.0% vs 100.0% | **neutral_or_negative** |

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Token Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | -19.53% | -3.46% | 0.82x | 100.0% | 100.0% |
| `gemini-2.5-flash` | -2299.66% | -1401.22% | 0.29x | 100.0% | 0.0% |
| `claude-haiku-4-5` | -114.56% | -42.48% | 0.29x | 100.0% | 100.0% |
| `sonnet` | 21.18% | 1.62% | 0.87x | 100.0% | 100.0% |
| `luna` | -133.23% | -59.43% | 0.76x | 100.0% | 100.0% |
| `terra` | -57.48% | -26.68% | 0.83x | 100.0% | 100.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (32525 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 2. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 27211 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

### 3. [CLI_FEATURE] tq CLI error handling
- **Identified Issue:** Run encountered errors or partial completion (Score: 0.0%). Details: Missing assertions
- **Observed Impact:** Causes agent to enter expensive retry loops, multiplying token and runtime costs.
- **Recommended Action:** Enhance error messaging in `tq query` with explicit remediation hints (e.g. Teradata 3932 DDL/BTET statement separation).

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789384317`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 44.08s | **Score:** 100.0%
- **Tokens:** 32,525 (Input: 29,531, Output: 2,994, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 137, Errors: 16, AMP CPU: 1.068s, I/O: 9,050, Peak Spool: 393,216 bytes
- **Token Cost:** **$0.0031**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_84317_GE_STG_CUSTOMERS', 'B_84317_GE_STG_PARTS', 'B_84317_GE_STG_ORDERS', 'B_84317_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_84317_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_84317_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_84317_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_84317_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789384376`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 36.22s | **Score:** 100.0%
- **Tokens:** 27,211 (Input: 22,906, Output: 4,305, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 115, Errors: 16, AMP CPU: 1.020s, I/O: 9,574, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.0030**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_84376_GE_STG_CUSTOMERS', 'B_84376_GE_STG_PARTS', 'B_84376_GE_STG_ORDERS', 'B_84376_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_84376_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_84376_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_84376_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_84376_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `gemini_gemini-2.5-flash_tq-with-skill_1789384426`
- **Harness / Model:** `gemini` / `gemini-2.5-flash` (tq-with-skill)
- **Duration:** 70.9s | **Score:** 100.0%
- **Tokens:** 104,769 (Input: 38,383, Output: 2,794, Cache Read: 63,592)
- **Database Telemetry (DBQL):** Queries: 141, Errors: 13, AMP CPU: 1.232s, I/O: 9,192, Peak Spool: 393,216 bytes
- **Token Cost:** **$0.0049**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_84426_GE_STG_CUSTOMERS', 'B_84426_GE_STG_PARTS', 'B_84426_GE_STG_ORDERS', 'B_84426_GE_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_84426_GE_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_84426_GE_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_84426_GE_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_84426_GE_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `gemini_gemini-2.5-flash_baseline-python_1789384510`
- **Harness / Model:** `gemini` / `gemini-2.5-flash` (baseline-python)
- **Duration:** 20.48s | **Score:** 0.0%
- **Tokens:** 4,366 (Input: 4,366, Output: 0, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 1, Errors: 0, AMP CPU: 0.004s, I/O: 6, Peak Spool: 0 bytes
- **Token Cost:** **$0.0003**
- **Assertion Details:**
  - ❌ FAIL: **stg_tables_exist** - Found 0/4 staging tables: []
  - ❌ FAIL: **dim_customers_created** - Table B_84510_GE_DIM_CUSTOMERS exists: False
  - ❌ FAIL: **fct_order_lineitem_created** - Table B_84510_GE_FCT_ORDER_LINEITEM exists: False
  - ❌ FAIL: **agg_monthly_performance_created** - Table B_84510_GE_AGG_MONTHLY_PERFORMANCE exists: False
  - ❌ FAIL: **dim_customer_profitability_created** - Table/View B_84510_GE_DIM_CUSTOMER_PROFITABILITY exists: False
  - ❌ FAIL: **revenue_integrity_checksum** - Failed query on b_84510_ge_fct_order_lineitem: 
  - ❌ FAIL: **profit_integrity_checksum** - Failed query on b_84510_ge_fct_order_lineitem: 
  - ❌ FAIL: **aggregate_reconciliation_checksum** - Failed query on b_84510_ge_agg_monthly_performance: 

### Run: `gemini_gemini-2.5-flash_tq-no-skill_1789384539`
- **Harness / Model:** `gemini` / `gemini-2.5-flash` (tq-no-skill)
- **Duration:** 52.09s | **Score:** 0.0%
- **Tokens:** 20,964 (Input: 9,197, Output: 5,634, Cache Read: 6,133)
- **Database Telemetry (DBQL):** Queries: 1, Errors: 0, AMP CPU: 0.000s, I/O: 6, Peak Spool: 0 bytes
- **Token Cost:** **$0.0025**
- **Assertion Details:**
  - ❌ FAIL: **stg_tables_exist** - Found 0/4 staging tables: []
  - ❌ FAIL: **dim_customers_created** - Table B_84539_GE_DIM_CUSTOMERS exists: False
  - ❌ FAIL: **fct_order_lineitem_created** - Table B_84539_GE_FCT_ORDER_LINEITEM exists: False
  - ❌ FAIL: **agg_monthly_performance_created** - Table B_84539_GE_AGG_MONTHLY_PERFORMANCE exists: False
  - ❌ FAIL: **dim_customer_profitability_created** - Table/View B_84539_GE_DIM_CUSTOMER_PROFITABILITY exists: False
  - ❌ FAIL: **revenue_integrity_checksum** - Failed query on b_84539_ge_fct_order_lineitem: 
  - ❌ FAIL: **profit_integrity_checksum** - Failed query on b_84539_ge_fct_order_lineitem: 
  - ❌ FAIL: **aggregate_reconciliation_checksum** - Failed query on b_84539_ge_agg_monthly_performance: 

### Run: `claude_claude-haiku-4-5_tq-with-skill_1789384599`
- **Harness / Model:** `claude-code` / `claude-haiku-4-5` (tq-with-skill)
- **Duration:** 356.17s | **Score:** 100.0%
- **Tokens:** 771,402 (Input: 141, Output: 9,299, Cache Read: 722,429)
- **Database Telemetry (DBQL):** Queries: 195, Errors: 18, AMP CPU: 2.072s, I/O: 17,581, Peak Spool: 1,015,808 bytes
- **Token Cost:** **$0.2083**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_84599_CL_STG_CUSTOMERS', 'B_84599_CL_STG_PARTS', 'B_84599_CL_STG_ORDERS', 'B_84599_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_84599_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_84599_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_84599_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_84599_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_claude-haiku-4-5_baseline-python_1789384969`
- **Harness / Model:** `claude-code` / `claude-haiku-4-5` (baseline-python)
- **Duration:** 102.58s | **Score:** 100.0%
- **Tokens:** 359,525 (Input: 75, Output: 8,732, Cache Read: 316,445)
- **Database Telemetry (DBQL):** Queries: 220, Errors: 25, AMP CPU: 1.992s, I/O: 20,078, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.1462**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_84969_CL_STG_CUSTOMERS', 'B_84969_CL_STG_PARTS', 'B_84969_CL_STG_ORDERS', 'B_84969_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_84969_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_84969_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_84969_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_84969_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_claude-haiku-4-5_tq-no-skill_1789385085`
- **Harness / Model:** `claude-code` / `claude-haiku-4-5` (tq-no-skill)
- **Duration:** 300.03s | **Score:** 100.0%
- **Tokens:** 0 (Input: 0, Output: 0, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 226, Errors: 25, AMP CPU: 2.180s, I/O: 20,450, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.0000**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85085_CL_STG_CUSTOMERS', 'B_85085_CL_STG_PARTS', 'B_85085_CL_STG_ORDERS', 'B_85085_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85085_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85085_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85085_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85085_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
- **Error:** `Execution timed out after 300 seconds`

### Run: `claude_sonnet_tq-with-skill_1789385401`
- **Harness / Model:** `claude-code` / `sonnet` (tq-with-skill)
- **Duration:** 67.88s | **Score:** 100.0%
- **Tokens:** 303,885 (Input: 14, Output: 3,771, Cache Read: 254,646)
- **Database Telemetry (DBQL):** Queries: 139, Errors: 16, AMP CPU: 0.972s, I/O: 9,059, Peak Spool: 393,216 bytes
- **Token Cost:** **$0.2728**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85401_CL_STG_CUSTOMERS', 'B_85401_CL_STG_PARTS', 'B_85401_CL_STG_ORDERS', 'B_85401_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85401_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85401_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85401_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85401_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `claude_sonnet_baseline-python_1789385482`
- **Harness / Model:** `claude-code` / `sonnet` (baseline-python)
- **Duration:** 58.96s | **Score:** 100.0%
- **Tokens:** 385,533 (Input: 16, Output: 6,047, Cache Read: 342,997)
- **Database Telemetry (DBQL):** Queries: 31, Errors: 8, AMP CPU: 0.412s, I/O: 5,185, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.2772**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85482_CL_STG_CUSTOMERS', 'B_85482_CL_STG_PARTS', 'B_85482_CL_STG_ORDERS', 'B_85482_CL_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85482_CL_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85482_CL_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85482_CL_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85482_CL_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `codex_luna_tq-with-skill_1789385554`
- **Harness / Model:** `codex` / `luna` (tq-with-skill)
- **Duration:** 204.17s | **Score:** 100.0%
- **Tokens:** 615,466 (Input: 45,522, Output: 8,024, Cache Read: 561,920)
- **Database Telemetry (DBQL):** Queries: 266, Errors: 24, AMP CPU: 3.324s, I/O: 23,199, Peak Spool: 1,015,808 bytes
- **Token Cost:** **$0.0300**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85554_CO_STG_CUSTOMERS', 'B_85554_CO_STG_PARTS', 'B_85554_CO_STG_ORDERS', 'B_85554_CO_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85554_CO_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85554_CO_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85554_CO_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85554_CO_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `codex_luna_baseline-python_1789385772`
- **Harness / Model:** `codex` / `luna` (baseline-python)
- **Duration:** 154.57s | **Score:** 100.0%
- **Tokens:** 263,886 (Input: 32,940, Output: 6,434, Cache Read: 224,512)
- **Database Telemetry (DBQL):** Queries: 114, Errors: 16, AMP CPU: 0.896s, I/O: 9,582, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.0188**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85772_CO_STG_CUSTOMERS', 'B_85772_CO_STG_PARTS', 'B_85772_CO_STG_ORDERS', 'B_85772_CO_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85772_CO_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85772_CO_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85772_CO_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85772_CO_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `codex_terra_tq-with-skill_1789385939`
- **Harness / Model:** `codex` / `terra` (tq-with-skill)
- **Duration:** 182.2s | **Score:** 100.0%
- **Tokens:** 475,377 (Input: 43,625, Output: 5,768, Cache Read: 425,984)
- **Database Telemetry (DBQL):** Queries: 201, Errors: 25, AMP CPU: 1.804s, I/O: 14,881, Peak Spool: 1,015,808 bytes
- **Token Cost:** **$0.2417**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_85939_CO_STG_CUSTOMERS', 'B_85939_CO_STG_PARTS', 'B_85939_CO_STG_ORDERS', 'B_85939_CO_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_85939_CO_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_85939_CO_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_85939_CO_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_85939_CO_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)

### Run: `codex_terra_baseline-python_1789386135`
- **Harness / Model:** `codex` / `terra` (baseline-python)
- **Duration:** 150.34s | **Score:** 100.0%
- **Tokens:** 301,869 (Input: 34,742, Output: 5,751, Cache Read: 261,376)
- **Database Telemetry (DBQL):** Queries: 122, Errors: 20, AMP CPU: 0.960s, I/O: 10,690, Peak Spool: 262,144 bytes
- **Token Cost:** **$0.1908**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_86135_CO_STG_CUSTOMERS', 'B_86135_CO_STG_PARTS', 'B_86135_CO_STG_ORDERS', 'B_86135_CO_STG_LINEITEM']
  - ✅ PASS: **dim_customers_created** - Table B_86135_CO_DIM_CUSTOMERS exists: True
  - ✅ PASS: **fct_order_lineitem_created** - Table B_86135_CO_FCT_ORDER_LINEITEM exists: True
  - ✅ PASS: **agg_monthly_performance_created** - Table B_86135_CO_AGG_MONTHLY_PERFORMANCE exists: True
  - ✅ PASS: **dim_customer_profitability_created** - Table/View B_86135_CO_DIM_CUSTOMER_PROFITABILITY exists: True
  - ✅ PASS: **revenue_integrity_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
  - ✅ PASS: **profit_integrity_checksum** - Expected 87,612.47, got 87,612.53 (diff: 0.0001%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 276,154.38, got 276,154.44 (diff: 0.0000%)
