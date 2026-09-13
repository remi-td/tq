#!/usr/bin/env python3
"""Master execution runner for the tq Agentic Optimization & Evaluation Framework.

Usage:
  python3 runner.py --matrix configs/default_matrix.yaml --live-db
  python3 runner.py --harness claude-code --model claude-haiku-4-5 --mode tq-with-skill --live-db
  python3 runner.py --harness gemini --model gemini-2.5-flash --mode tq-with-skill --live-db
"""

from __future__ import annotations

import argparse
import os
import sys
import time
import yaml
from pathlib import Path
from typing import Any

# Ensure module path resolution
BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR.parent.parent) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR.parent.parent))

from agentic.bench.harness.base import AgentHarness, RunResult
from agentic.bench.harness.claude_code import ClaudeCodeHarness
from agentic.bench.harness.gemini import GeminiHarness
from agentic.bench.harness.codex import CodexHarness
from agentic.bench.analytics.report_generator import ReportGenerator
from agentic.bench.telemetry.database_telemetry import TeradataTelemetry

DATASETS_DIR = BENCH_DIR / "datasets"
WORKSPACE_BASE = BENCH_DIR / "workspaces"


def get_harness(harness_type: str, model_id: str, mode: str, workspace_dir: Path) -> AgentHarness:
    """Factory creating the designated AgentHarness."""
    if harness_type == "claude-code":
        return ClaudeCodeHarness(model_id, mode, workspace_dir)
    elif harness_type == "gemini":
        return GeminiHarness(model_id, mode, workspace_dir)
    elif harness_type == "codex":
        return CodexHarness(model_id, mode, workspace_dir)
    else:
        raise ValueError(f"Unsupported harness type: {harness_type}")


def execute_single_run(
    harness_type: str,
    model_id: str,
    mode: str,
    dataset_name: str = "tpch_order_fulfillment",
    live_db: bool = True,
    cleanup: bool = True,
    timeout_seconds: int = 300
) -> RunResult:
    """Run an individual benchmark scenario."""
    dataset_dir = DATASETS_DIR / dataset_name
    template_path = dataset_dir / "task_prompt.md"
    spec_path = dataset_dir / "validation_spec.json"

    with open(spec_path, "r") as f:
        spec = yaml.safe_load(f) if spec_path.suffix in (".yaml", ".yml") else __import__("json").load(f)

    # Unique table prefix for isolation: e.g. b_gh45_
    prefix_token = f"b_{int(time.time()) % 100000}_{harness_type[:2]}_"
    table_prefix = prefix_token.lower()

    run_dir = WORKSPACE_BASE / f"run_{prefix_token}_{model_id.replace('-', '_').replace('.', '_')}"
    harness = get_harness(harness_type, model_id, mode, run_dir)
    harness.prepare_workspace(dataset_dir)

    task_prompt = harness.build_task_prompt(template_path, table_prefix)

    print(f"\n=======================================================")
    print(f"🚀 Launching Run: [{harness_type}] {model_id} ({mode})")
    print(f"📁 Workspace: {run_dir}")
    print(f"🏷️ Table Prefix: {table_prefix}")
    print(f"=======================================================")

    result = harness.execute(
        task_prompt=task_prompt,
        table_prefix=table_prefix,
        validation_spec=spec,
        timeout_seconds=timeout_seconds
    )

    print(f"✅ Completed: Duration={result.duration_seconds}s, Score={result.validation_score}%, Tokens={result.token_usage.total_tokens:,}, Cost=${result.costs.get('total_effort_cost_usd', 0.0):.4f}")

    if cleanup and live_db:
        cleaned = TeradataTelemetry().cleanup_benchmark_objects(table_prefix)
        if cleaned:
            print(f"🧹 Cleaned up {len(cleaned)} benchmark table(s): {cleaned}")

    return result


def main() -> None:
    parser = argparse.ArgumentParser(description="tq Agentic Optimization & Evaluation Benchmark Runner")
    parser.add_argument("--matrix", help="Path to matrix YAML configuration file")
    parser.add_argument("--harness", choices=["claude-code", "gemini", "codex"], help="Harness to run")
    parser.add_argument("--model", help="Model ID")
    parser.add_argument("--mode", choices=["tq-with-skill", "tq-no-skill", "baseline-python"], default="tq-with-skill")
    parser.add_argument("--dataset", default="tpch_order_fulfillment", help="Dataset name")
    parser.add_argument("--live-db", action="store_true", default=True, help="Enable live Teradata database validation")
    parser.add_argument("--no-cleanup", action="store_true", help="Do not drop benchmark tables after run")
    parser.add_argument("--output-dir", default=str(BENCH_DIR.parent.parent / "docs" / "benchmarks"), help="Output directory for reports")
    parser.add_argument("--timeout", type=int, default=300, help="Per-run timeout in seconds")

    args = parser.parse_args()

    results: list[RunResult] = []

    if args.matrix:
        matrix_file = Path(args.matrix)
        if not matrix_file.exists():
            print(f"Matrix file not found: {matrix_file}", file=sys.stderr)
            sys.exit(1)

        with open(matrix_file, "r") as f:
            cfg = yaml.safe_load(f)

        runs = cfg.get("runs", [])
        dataset = cfg.get("dataset", args.dataset)
        print(f"Loaded matrix '{cfg.get('name')}' with {len(runs)} run scenario(s).")

        for r_cfg in runs:
            res = execute_single_run(
                harness_type=r_cfg["harness"],
                model_id=r_cfg["model"],
                mode=r_cfg.get("mode", "tq-with-skill"),
                dataset_name=dataset,
                live_db=args.live_db,
                cleanup=not args.no_cleanup,
                timeout_seconds=args.timeout
            )
            results.append(res)
    else:
        if not args.harness or not args.model:
            print("Error: Must specify either --matrix or both --harness and --model.", file=sys.stderr)
            sys.exit(1)

        res = execute_single_run(
            harness_type=args.harness,
            model_id=args.model,
            mode=args.mode,
            dataset_name=args.dataset,
            live_db=args.live_db,
            cleanup=not args.no_cleanup,
            timeout_seconds=args.timeout
        )
        results.append(res)

    # Generate and save report
    generator = ReportGenerator()
    md_path, json_path = generator.save_report(results, args.output_dir)

    print("\n=======================================================")
    print(f"📊 Benchmark Report Generated:")
    print(f"   Markdown: {md_path}")
    print(f"   JSON:     {json_path}")
    print("=======================================================\n")


if __name__ == "__main__":
    main()
