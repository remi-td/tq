# tq Agentic Optimization & Evaluation Benchmark Report

> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end
> Data Product pipeline on Teradata. It tracks LLM token consumption and cost along with
> database execution performance telemetry (AMP CPU, I/O, Spool).

## 1. Executive Leaderboard

| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | **Token Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `gemini` | `gemini-3.5-flash-lite` | `tq-with-skill` | 100.0% | 55.87s | 57,393 | 0.0% | 153 | 4.252 | 10,568 | **$0.0054** |
| `gemini` | `gemini-3.5-flash-lite` | `baseline-python` | 100.0% | 73.37s | 99,580 | 0.0% | 218 | 3.132 | 17,237 | **$0.0094** |

---

## 2. Comparative Impact Analysis

### B. Tool Acceleration (`tq` vs `baseline-python`)

| Model | Token Savings % | Token Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `gemini-3.5-flash-lite` | 42.36% | 42.7% | 1.31x | 100.0% | 100.0% |

---

## 3. Actionable Optimization Recommendations

### 1. [CLI_FEATURE] tq query --budget
- **Identified Issue:** Query outputs exceeded optimal agent context budget.
- **Observed Impact:** Excessive context consumption (57393 tokens) inflates cost and degrades agent reasoning.
- **Recommended Action:** Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size.

### 2. [TOKEN_COMPRESSION] SKILL.md & CLI defaults
- **Identified Issue:** Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.
- **Observed Impact:** Consumed 99580 tokens. Using --format toon or --agent reduces token volume by 40-70%.
- **Recommended Action:** Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected.

---

## 4. Run Details & Validation Breakdown

### Run: `gemini_gemini-3.5-flash-lite_tq-with-skill_1789378464`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (tq-with-skill)
- **Duration:** 55.87s | **Score:** 100.0%
- **Tokens:** 57,393 (Input: 52,489, Output: 4,904, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 153, Errors: 19, AMP CPU: 4.252s, I/O: 10,568, Peak Spool: 4,870,144 bytes
- **Token Cost:** **$0.0054**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_78464_GE_STG_ACCOUNTS', 'B_78464_GE_STG_PLANS', 'B_78464_GE_STG_FEATURES', 'B_78464_GE_STG_USAGE_EVENTS']
  - ✅ PASS: **dim_accounts_created** - Table B_78464_GE_DIM_ACCOUNTS exists: True
  - ✅ PASS: **fct_metered_usage_created** - Table B_78464_GE_FCT_METERED_USAGE exists: True
  - ✅ PASS: **agg_monthly_billing_created** - Table B_78464_GE_AGG_MONTHLY_BILLING exists: True
  - ✅ PASS: **dim_account_usage_kpi_created** - Table/View B_78464_GE_DIM_ACCOUNT_USAGE_KPI exists: True
  - ✅ PASS: **gross_billed_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
  - ✅ PASS: **net_margin_checksum** - Expected 1,135,498.25, got 1,135,498.25 (diff: 0.0000%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)

### Run: `gemini_gemini-3.5-flash-lite_baseline-python_1789378535`
- **Harness / Model:** `gemini` / `gemini-3.5-flash-lite` (baseline-python)
- **Duration:** 73.37s | **Score:** 100.0%
- **Tokens:** 99,580 (Input: 90,827, Output: 8,753, Cache Read: 0)
- **Database Telemetry (DBQL):** Queries: 218, Errors: 27, AMP CPU: 3.132s, I/O: 17,237, Peak Spool: 4,374,528 bytes
- **Token Cost:** **$0.0094**
- **Assertion Details:**
  - ✅ PASS: **stg_tables_exist** - Found 4/4 staging tables: ['B_78535_GE_STG_ACCOUNTS', 'B_78535_GE_STG_PLANS', 'B_78535_GE_STG_FEATURES', 'B_78535_GE_STG_USAGE_EVENTS']
  - ✅ PASS: **dim_accounts_created** - Table B_78535_GE_DIM_ACCOUNTS exists: True
  - ✅ PASS: **fct_metered_usage_created** - Table B_78535_GE_FCT_METERED_USAGE exists: True
  - ✅ PASS: **agg_monthly_billing_created** - Table B_78535_GE_AGG_MONTHLY_BILLING exists: True
  - ✅ PASS: **dim_account_usage_kpi_created** - Table/View B_78535_GE_DIM_ACCOUNT_USAGE_KPI exists: True
  - ✅ PASS: **gross_billed_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
  - ✅ PASS: **net_margin_checksum** - Expected 1,135,498.25, got 1,135,498.25 (diff: 0.0000%)
  - ✅ PASS: **aggregate_reconciliation_checksum** - Expected 1,852,842.02, got 1,852,842.02 (diff: 0.0000%)
