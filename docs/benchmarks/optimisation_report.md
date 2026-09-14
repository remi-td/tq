# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by
> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 75.45s | 167,914 | 0.0% | 203 | 2.864 | 12,405 | $0.0139 | $0.0156 | **$0.0294** |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 53.99s | 40,861 | 0.0% | 107 | 6.372 | 30,041 | $0.0050 | $0.0349 | **$0.0398** |

---

## 2. Comparative Impact Analysis

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | -310.94% | 26.07% | 0.72x | 100.0% | 100.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (167914 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 2. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 40861 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789367482`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 75.45s | **Score:** 100.0%
- **Tokens:** 167,914 (Input: 162,196, Output: 5,718, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 203, Errors: 22, AMP CPU: 2.864s, I/O: 12,405, Peak Spool: 4,624,384 bytes
- **Total Effort Cost:** **$0.0294**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_67482_GE_STG_ACCOUNTS', 'B_67482_GE_STG_PLANS', 'B_67482_GE_STG_FEATURES', 'B_67482_GE_STG_USAGE_EVENTS']
  - ✅ PASS: **dim_accounts_created** - Table B_67482_GE_DIM_ACCOUNTS exists: True
  - ✅ PASS: **fct_metered_usage_created** - Table B_67482_GE_FCT_METERED_USAGE exists: True
  - ✅ PASS: **agg_monthly_billing_created** - Table B_67482_GE_AGG_MONTHLY_BILLING exists: True
  - ✅ PASS: **dim_account_usage_kpi_created** - Table/View B_67482_GE_DIM_ACCOUNT_USAGE_KPI exists: True
  - ✅ PASS: **gross_billed_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
  - ✅ PASS: **net_margin_checksum** - Expected 1,135,498.25, got 1,135,498.25 (diff: 0.0000%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789367572`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 53.99s | **Score:** 100.0%
- **Tokens:** 40,861 (Input: 32,440, Output: 8,421, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 107, Errors: 11, AMP CPU: 6.372s, I/O: 30,041, Peak Spool: 7,000,064 bytes
- **Total Effort Cost:** **$0.0398**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_67572_GE_STG_ACCOUNTS', 'B_67572_GE_STG_PLANS', 'B_67572_GE_STG_FEATURES', 'B_67572_GE_STG_USAGE_EVENTS']
  - ✅ PASS: **dim_accounts_created** - Table B_67572_GE_DIM_ACCOUNTS exists: True
  - ✅ PASS: **fct_metered_usage_created** - Table B_67572_GE_FCT_METERED_USAGE exists: True
  - ✅ PASS: **agg_monthly_billing_created** - Table B_67572_GE_AGG_MONTHLY_BILLING exists: True
  - ✅ PASS: **dim_account_usage_kpi_created** - Table/View B_67572_GE_DIM_ACCOUNT_USAGE_KPI exists: True
  - ✅ PASS: **gross_billed_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
  - ✅ PASS: **net_margin_checksum** - Expected 1,135,498.25, got 1,135,498.25 (diff: 0.0000%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
