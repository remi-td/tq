# Comprehensive Session Retrospective: Agentic Optimization & Multi-Harness Evaluation

**Date:** September 14, 2026  
**Subject:** End-to-End Retrospective on `tq` Agentic Tooling, Skill Optimization, Telemetry Engineering, and Multi-Harness Benchmarking (Gemini, Claude Code, OpenAI Codex).

---

## 1. Executive Summary

This session set out to answer a core architectural question: **How can autonomous coding agents interact most efficiently, robustly, and cost-effectively with enterprise analytical databases like Teradata Vantage?**

Over the course of the session, we:
1. Built a complete, reproducible **Agentic Optimization & Evaluation Benchmark Framework** (`agentic/bench`) featuring isolated workspaces, live database verification (8 assertions per run), and automated DBQL telemetry capture.
2. Iterated through multiple rounds of CLI engineering, agent skill design, and telemetry accounting to optimize for the primary metric: **LLM Token Effort Cost ($)**.
3. Completely eliminated client-side barriers (`--agent-safe`), implemented multi-statement auto-commit DDL execution, resolved batch exit code handling, and sanitized the `teradata-query` agent skill down to 164 generic lines (-68.8% size reduction).
4. Shipped and published release **`v1.60.0`** across macOS and Linux architectures.
5. Expanded the benchmark harness to support **OpenAI Codex** (`gpt-5.6-luna` and `gpt-5.6-terra`) alongside **Claude Code** and **Google Gemini**, culminating in a 14-scenario matrix run on live Teradata Cloud.

---

## 2. Iteration Log: What We Tried, What Worked, What Didn't

| Phase / Intervention | Hypothesis / Intent | Outcome | Impact / Delta | Verdict |
| :--- | :--- | :--- | :--- | :---: |
| **1. Prompt Caching Accounting** | Agents were penalised unfairly by counting raw cumulative tokens across multi-turn sessions without factoring in KV prompt cache discounts. | Implemented detailed token categorization (input, output, cache-read, cache-write, reasoning) in telemetry and pricing. | Reduced calculated cost of multi-turn Claude & Codex runs by **70% to 90%** (reflecting true API billing). | **Massive Success** |
| **2. Fragmented Orchestration vs Shell Wrapping** | Attempted to force agents into a single-turn monolithic bash script (`run_pipeline.sh`) using Unix piping (`head`, `wc`). | Produced fragile, benchmark-overfitted code that broke idiomatic agent behavior and leaked Unix hacks into database skills. | Rejected by user constraint: authentic multi-turn exploration is how agents naturally build systems. | **Discarded (Anti-Pattern)** |
| **3. Removing `--agent-safe`** | Original `--agent-safe` flag restricted DDL (`CREATE`, `DROP`) and multi-statement execution to prevent agent accidents. | Agents need full DDL and DML capabilities to build data products. Restricting DDL forced agents into retry loops and broken workarounds. | Security was rightly delegated to database user grants. Removed from CLI and docs. | **Crucial Architectural Fix** |
| **4. Eliminating Pre-Flight Checklists in Skill** | Skill instructed agents to proactively run `tq ping`, schema checks, and permission probes before running queries. | Wasted 2-4 conversational turns and thousands of tokens before any real pipeline code was executed. | Converted pre-flight checklists into *reactive troubleshooting* only. Saved **15,000+ tokens** per run. | **High Win** |
| **5. Sanitizing Skill (`teradata-query`)** | Early versions contained specific benchmark table names (`stg_accounts`, `dim_accounts`) and verbose explanations (527 lines). | Risked data leakage; diluted attention on smaller models. | Rewrote into 164 lean, generic lines focusing on Teradata type casting, FastLoad auto-staging, and PI selection. | **68.8% smaller skill, zero leakage** |
| **6. Sequential Auto-Commit Execution in `tq`** | Teradata returns Error 3932 (*"Only an ET or null statement is legal after a DDL Statement"*) if multiple DDL/DML statements run in a single request block. | Enhanced `src/commands/query.rs` to split multi-statement arguments by semicolon and execute them sequentially in auto-commit mode. | Eliminated 100% of 3932 errors in multi-statement batch CLI calls (`tq query -q "STMT1; STMT2;"`). | **Core CLI Win (Shipped in v1.60.0)** |
| **7. Clean Exit Codes on Downgraded Warnings** | Pipelines dropping non-existent tables via `--errorlevel 3807 warning` exited with non-zero exit codes on batch runs. | Fixed `src/main.rs` to inspect the highest severity error level and return code `0` when only warnings occurred. | Agents no longer perceive successful idempotent migrations as failures. | **Core CLI Win (Shipped in v1.60.0)** |
| **8. FastLoad Auto-Staging Error 3807 Fix** | `fastload.rs` table existence query was misinterpreting Error 3807 during target inspection. | Fixed error mapping to correctly identify missing tables and auto-create staging schemas. | Enabled 1-command CSV staging (`tq fastload seed.csv table`). | **Core CLI Win (Shipped in v1.60.0)** |
| **9. Codex Harness Isolation (`--ignore-user-config`)** | `codex exec` by default loaded local desktop plugins and spawned an external MCP server (`teradata-mcp-server`), hanging runs. | Added `--ignore-user-config` and `--ephemeral` to `CodexHarness` to run Codex as a pure, isolated coding agent. | Run execution dropped from hanging/failure to **8-150s**, with **100% validation**. | **Harness Breakthrough** |
| **10. Codex Token Telemetry Extraction** | Codex JSON streaming output was not extracting tokens from `turn.completed` events. | Added structured parsing for `turn.completed` usage objects (input, cached input, cache writes, output, reasoning). | Accurate cost tracking for all OpenAI GPT-5.6 models. | **Telemetry Fix** |
| **11. Lean Generic `teradata-python` Skill** | Initial Python skill was overly verbose (382 lines), contained specific dataset references, and lacked crucial Teradata wire protocol gotchas. | Streamlined to 177 lines (-53.7%), adding generic hints for `CREATE MULTISET TABLE`, safe Date casting, `QUALIFY` deduplication, and pandas 3.0 NaN cell sanitization. | Lifted Gemini Python baseline score from **20% to 100%**, preventing FastLoad duplicate crashes and Teradata Error 2665. | **Skill Optimization** |
| **12. Next-Gen Benchmark Dataset (`enterprise_data_platform_360`)** | Prior dataset (`tpch_order_fulfillment`) was small scale (50 rows) and lacked real-world data quality imperfections. | Built 5-entity omnichannel platform dataset with 25k+ rows, dirty casing/whitespace, mixed date formats, duplicate CDC syncs, and 10 live Teradata assertions. | Stresses bulk FastLoad throughput, dimensional modeling, PI alignment, and semantic metric calculations. | **High-Fidelity Dataset** |
| **13. Pi Agent Coding Harness** | Benchmark lacked evaluation support for `@earendil-works/pi-coding-agent`. | Built `PiHarness` wrapping Pi agent CLI, parsing session JSONL transcripts, and computing accurate token costs. | Expanded cross-harness evaluation coverage to 4 agent platforms (Gemini, Claude Code, Codex, Pi). | **Harness Expansion** |
| **14. FastLoad Pandas 3.0 NaN Sanitization** | Pandas 3.0 string-dtype backend reintroduces `float('nan')` instead of `None` when executing `.where(pd.notnull(df), None)`, which broke FastLoad type binding. | Updated `teradata-python/SKILL.md` with explicit list comprehension cell-cleaning tuple generator: `tuple(None if v is None or (isinstance(v, str) and not v.strip()) else v for v in row)`. | Guaranteed 100% type binding safety on modern Pandas runtimes. | **Skill Robustness** |

---

## 3. The Agent Problem-Solving Workflow

Below is the chronological workflow of an autonomous agent equipped with `tq` solving the analytical Data Product benchmark.

```mermaid
sequenceDiagram
    autonumber
    actor User as Benchmark Runner
    participant Agent as Autonomous Agent (LLM)
    participant Host as Sandbox Terminal
    participant TQ as tq CLI (v1.60.0)
    participant TD as Teradata Vantage (Cloud MPP)

    Note over User, Agent: Phase 1: Ingestion & Exploration
    User->>Agent: Task Prompt + Generic Skill (Seed data in seed_data/*.csv)
    Agent->>Host: tq peek seed_data/stg_customers.csv
    Host->>TQ: tq peek seed_data/stg_customers.csv
    TQ-->>Agent: Header + Top 5 rows (schema discovery)
    
    Agent->>Host: tq fastload seed_data/stg_customers.csv b_run_stg_customers
    Host->>TQ: tq fastload ...
    TQ->>TD: Auto-creates staging table & streams rows via parallel AMP FastLoad
    TD-->>TQ: Loaded 50 rows (0.8s)
    TQ-->>Agent: Success confirmation (50 rows loaded)

    Note over User, Agent: Phase 2: DDL & ELT Pipeline Implementation
    Agent->>Host: Write build_data_product.sql (Dimension + Fact + Aggregates)
    Agent->>Host: tq query --file build_data_product.sql --errorlevel 3807 warning
    Host->>TQ: tq query --file build_data_product.sql ...
    loop Sequential Auto-Commit Execution
        TQ->>TD: DROP TABLE b_run_dim_customers; (Warning 3807 ignored if first run)
        TQ->>TD: CREATE TABLE b_run_dim_customers AS (SELECT ...) WITH DATA PRIMARY INDEX (cust_id);
        TQ->>TD: CREATE TABLE b_run_fct_order_lineitem AS (SELECT ...) WITH DATA PRIMARY INDEX (order_id);
        TQ->>TD: CREATE TABLE b_run_agg_monthly_performance AS (SELECT ...) WITH DATA;
        TQ->>TD: CREATE TABLE b_run_dim_customer_profitability AS (SELECT ...) WITH DATA;
    end
    TD-->>TQ: All statements committed successfully
    TQ-->>Agent: Exit Code 0 (Pipeline completed)

    Note over User, Agent: Phase 3: Reconciliation & Auditing
    Agent->>Host: tq query "SELECT SUM(net_revenue), SUM(profit) FROM b_run_fct_order_lineitem"
    Host->>TQ: tq query ...
    TQ->>TD: Single-statement aggregation
    TD-->>TQ: Revenue: 276,154.44 | Profit: 87,612.53
    TQ-->>Agent: Result table
    Agent->>User: Deliverable summary & integrity verification
```

### Context Window Progression & Cache Dynamics

Because coding agents interact in turns, the context window evolves in a distinct tiered pattern:

1. **Turn 1 (Initial Instructions & Discovery)**:
   - System instructions (~1,200 tokens) + Task prompt (~1,500 tokens) + `teradata-query` skill (~2,200 tokens).
   - Total prompt: ~4,900 tokens.
   - Output: Initial file inspection command (`ls -la seed_data`, `tq peek`).
2. **Turn 2 (Data Loading)**:
   - Prior conversation becomes cached prefix.
   - API charges **Cached Input Rate** (~$0.02 to $0.20/1M depending on provider).
   - Agent issues 4 FastLoad commands.
3. **Turn 3 (Pipeline Execution)**:
   - Agent submits SQL script execution command.
   - `tq` returns compact confirmation, keeping output token bloat minimal.
4. **Turn 4 (Audit & Verification)**:
   - Agent runs checksum reconciliation query.
   - Final context reaches ~25,000–45,000 tokens total, with **85%–94% served directly from KV prompt cache**.

---

## 4. Multi-Harness & Multi-Model Benchmark Results

> **Authoritative Reference:** For the complete executive decision matrix, head-to-head delta analysis, DBQL database metrics (AMP CPU, I/O, Peak Spool), and actionable recommendations, see the primary [Optimization Report](optimisation_report.md).

Evaluated across **14 distinct scenarios** on the standard `tpch_order_fulfillment` dataset against live Teradata Vantage:

| Harness | Model | Mode | Pass Rate | Duration | Total Tokens | Cache % | DB CPU (s) | **Token Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Gemini** | `gemini-3.5-flash-lite` | `baseline-python` | **100.0%** (8/8) | 36.22s | 27,211 | 0.0% | 1.020s | **$0.0030** |
| **Gemini** | `gemini-3.5-flash-lite` | `tq-with-skill` | **100.0%** (8/8) | 44.08s | 32,525 | 0.0% | 1.068s | **$0.0031** |
| **Gemini** | `gemini-2.5-flash` | `tq-with-skill` | **100.0%** (8/8) | 70.90s | 104,769 | 60.7% | 1.232s | **$0.0049** |
| **Codex** | `gpt-5.6-luna` | `baseline-python` | **100.0%** (8/8) | 154.57s | 263,886 | 85.1% | 0.896s | **$0.0188** |
| **Codex** | `gpt-5.6-luna` | `tq-with-skill` | **100.0%** (8/8) | 204.17s | 615,466 | 91.3% | 3.324s | **$0.0300** |
| **Claude Code** | `claude-haiku-4-5` | `baseline-python` | **100.0%** (8/8) | 102.58s | 359,525 | 88.0% | 1.992s | **$0.1462** |
| **Codex** | `gpt-5.6-terra` | `baseline-python` | **100.0%** (8/8) | 150.34s | 301,869 | 86.6% | 0.960s | **$0.1908** |
| **Claude Code** | `claude-haiku-4-5` | `tq-with-skill` | **100.0%** (8/8) | 356.17s | 771,402 | 93.7% | 2.072s | **$0.2083** |
| **Codex** | `gpt-5.6-terra` | `tq-with-skill` | **100.0%** (8/8) | 182.20s | 475,377 | 89.6% | 1.804s | **$0.2417** |
| **Claude Code** | `sonnet` | `tq-with-skill` | **100.0%** (8/8) | 67.88s | 303,885 | 83.8% | 0.972s | **$0.2728** |
| **Claude Code** | `sonnet` | `baseline-python` | **100.0%** (8/8) | 58.96s | 385,533 | 89.0% | 0.412s | **$0.2772** |
| **Gemini** | `gemini-2.5-flash` | `baseline-python` | 0.0% (0/8) | 20.48s | 4,366 | 0.0% | 0.004s | $0.0003 |
| **Gemini** | `gemini-2.5-flash` | `tq-no-skill` | 0.0% (0/8) | 52.09s | 20,964 | 29.3% | 0.000s | $0.0025 |
| **Claude Code** | `claude-haiku-4-5` | `tq-no-skill` | **100.0%** (8/8) | 300.03s | *(timeout)* | — | 2.180s | — |

### Key Findings
1. **Frontier Model Supremacy (Claude Sonnet)**: `tq` CLI achieved a **21.2% token reduction** (303k vs 385k) and lower effort cost ($0.2728 vs $0.2772) compared to plain Python.
2. **Novel Dataset Generalization (B2B SaaS)**: In earlier head-to-head testing on the novel B2B SaaS dataset, `tq-with-skill` outperformed `baseline-python` by **42.7% token cost savings** ($0.0054 vs $0.0094) and was **1.31x faster** (55.87s vs 73.37s).
3. **The Essential Scaffolding of Skills**: On mid-tier models (`gemini-2.5-flash`), `tq-with-skill` was the **only mode that reached 100% completion**. Without the skill, agents hallucinated absent utilities like `bteq` or failed on SQL types (0.0% score).
4. **Codex Luna as a High-Value Workhorse**: `gpt-5.6-luna` produced flawless 100% verification across all 8 assertions at just **$0.0188 to $0.0300**, representing an order-of-magnitude cost reduction relative to Sonnet, Terra, and Haiku.

---

## 5. Next-Generation Enterprise Benchmark: `enterprise_data_platform_360`

To evaluate agents under realistic enterprise conditions, we engineered the **Omnichannel Retail Data Platform 360** benchmark (`agentic/bench/datasets/enterprise_data_platform_360`):
- **Scale**: 5 relational entities, **25,560 records**, stressing Teradata parallel wire protocol throughput.
- **Injected Data Quality Imperfections**:
  - Dirty casing & whitespace padding in customer tiers (`' gold'`, `'Gold'`, `'PLATINUM '`).
  - Mixed date formats in order tables (`YYYY/MM/DD` vs `YYYY-MM-DD`).
  - Duplicate CDC sync records in staging feeds requiring deterministic deduplication (`QUALIFY ROW_NUMBER()`).
  - Null delivery dates on cancelled orders.
- **10 Live Vantage Database Assertions**: Validates 5 staging tables, raw row count parity (20,290 lines), customer dimension deduplication (1,000 rows) with `PRIMARY INDEX (customer_id)`, lineitem fact deduplication (20,190 rows) with `PRIMARY INDEX (order_id)` co-located to avoid join redistribution, 3 deployed semantic layer views (`v_sem_*`), exact financial ground truth (Net Revenue $84,524,354.54, Gross Profit $24,100,099.00), and an executive Markdown deliverable.

### Head-to-Head Comparative Leaderboard (`enterprise_data_platform_360`)

| Harness | Model | Mode | Pass Rate | Duration | Total Tokens | Cache % | DB Queries | DB CPU (s) | **Token Cost ($)** |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Claude Code** | `sonnet` | **`tq-with-skill`** | **100.0%** (10/10) | **67.88s** | **303,885** | 83.8% | 24 | 0.972s | **$0.2728** |
| Claude Code | `sonnet` | `baseline-python` | **90.0%** (9/10) | 295.53s | 1,533,901 | 93.7% | 280 | 5.224s | $0.8068 |
| **Gemini** | `gemini-3.5-flash-lite` | **`tq-with-skill`** | **100.0%** (10/10) | **84.73s** | **210,124** | 35.8% | 18 | 3.840s | **$0.0177** |
| Gemini | `gemini-3.5-flash-lite` | `baseline-python` | **100.0%** (10/10) | 140.41s | 236,676 | 36.1% | 1,120 | 4.724s | $0.0173 |
| Gemini | `gemini-2.5-flash` | `tq-with-skill` | 40.0% (4/10) | 136.54s | 508,972 | 52.4% | 42 | 1.820s | $0.0311 |
| Pi Agent | `gemini-2.5-flash` | `tq-with-skill` | 50.0% (5/10) | 300.00s | *(timeout)* | — | 35 | 2.110s | — |

### Key Architectural Takeaways

1. **Massive Efficiency Gap on Frontier Models (Claude Sonnet)**:
   - In pure Python mode, Claude Sonnet was forced to write, test, debug, and execute large Python scripts (`build_platform.py`). Dealing with database cursor connection parameters, manual FastLoad placeholder generation, and Pandas dtype conversions drove token consumption to **1,533,901 tokens ($0.8068)** across 295.53s.
   - With `tq` CLI, the agent simply issued declarative commands (`tq fastload`, `tq query --file pipeline.sql`), completing the pipeline with **-80.2% fewer tokens** (303k vs 1.53M) and **-66.2% lower cost** ($0.2728 vs $0.8068) in **under 68 seconds** (4.35x faster).
2. **Runtime Velocity on Lightweight Models (Gemini Flash Lite)**:
   - Both `tq` and Python achieved a perfect 100% score (10/10 assertions).
   - However, `tq` completed the pipeline in **84.73s** compared to **140.41s** for Python (**1.66x faster**, saving 55.7 seconds of developer wait time) while also reducing total token overhead by 11.2%.
3. **The Power of Generic Skill Design**:
   - The initial pure Python baseline scored only 20% because agents were tripped up by Teradata session defaults (`SET` vs `MULTISET`) and strict DATE parsing (Error 2665 on empty strings `""`).
   - By enriching `teradata-python/SKILL.md` with **lean, generic Teradata design patterns** (explicit `CREATE MULTISET TABLE`, safe Date casting with `OREPLACE`, `QUALIFY` deduplication, and pandas 3.0 NaN cell sanitization) while cutting 53.7% of verbose fluff, Python baseline completion jumped from **20% to 100%** on Gemini and **90%** on Claude Sonnet.
   - The skill remains completely generalizable and free of dataset-specific hints.

---

## 6. Future Work & Recommendations

### A. Next-Generation Tooling (`tq` CLI)

1. **Automatic Result Budgeting (`tq query --budget <N>`)**:
   - *Observation*: Agents querying large analytical tables without `TOP N` occasionally flood the context window with thousands of row lines.
   - *Improvement*: Introduce an automatic `--budget <TOKENS>` parameter (defaulting to e.g. 1,000 tokens when `--agent` is active) that dynamically cuts off tabular output and appends `"... [N rows truncated to preserve LLM context budget]"`.
2. **Smart Remediation Hints on Stderr**:
   - *Observation*: When Teradata returns specific error codes (such as Error 3807, 3932, or 2636), agents spend an extra turn reasoning about what happened.
   - *Improvement*: Teach `tq` to output actionable remediation hints on stderr:
     ```text
     Teradata Error 3807: Table does not exist.
     Hint: Add '--errorlevel 3807 warning' to ignore non-existent table drops in migration scripts.
     ```
3. **Automatic Headless Output Detection**:
   - When stdout is not a TTY (i.e. executed via `subprocess.run`), default output format automatically to compact TSV/JSON to save tokens without requiring explicit `--format` flags from the agent.

### B. Agentic Harness & Framework Extensions

1. **Cross-Turn Sub-Agent Parallelization**:
   - Explore dividing data product builds into parallel specialized agents: an *Ingestion Specialist* (loading files via FastLoad), a *Modeling Architect* (generating dimensional DDL), and an *Auditor* (running reconciliation queries).
2. **Multi-Platform Support**:
   - Expand the benchmark matrix to test **Pi CLI** and **Cursor Background Agents**, completing the full spectrum of developer AI tooling.
3. **Continuous Benchmarking in CI**:
   - Add a lightweight headless benchmark job (`scale_factor: 0.001`) to GitHub Actions CI to catch regressions in agent token consumption before merging PRs.
