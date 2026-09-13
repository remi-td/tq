"""Comparative analytics engine for benchmark evaluation results.

Calculates token delta ratios, database efficiency deltas, and cross-model rankings.
"""

from __future__ import annotations

from typing import Any
from ..harness.base import RunResult


def compare_skill_impact(run_with_skill: RunResult, run_no_skill: RunResult) -> dict[str, Any]:
    """Calculate token and effort savings achieved by loading the teradata-query skill."""
    tokens_base = max(1, run_no_skill.token_usage.total_tokens)
    tokens_skill = run_with_skill.token_usage.total_tokens
    token_delta_pct = round(((tokens_base - tokens_skill) / float(tokens_base)) * 100.0, 2)

    cost_base = max(0.000001, run_no_skill.costs.get("total_effort_cost_usd", 0.0))
    cost_skill = run_with_skill.costs.get("total_effort_cost_usd", 0.0)
    cost_delta_pct = round(((cost_base - cost_skill) / cost_base) * 100.0, 2)

    cpu_base = max(0.001, run_no_skill.db_metrics.delta_cpu_sec)
    cpu_skill = run_with_skill.db_metrics.delta_cpu_sec
    cpu_delta_pct = round(((cpu_base - cpu_skill) / cpu_base) * 100.0, 2)

    return {
        "model": run_with_skill.model,
        "token_reduction_pct": token_delta_pct,
        "cost_reduction_pct": cost_delta_pct,
        "database_cpu_savings_pct": cpu_delta_pct,
        "accuracy_with_skill": run_with_skill.validation_score,
        "accuracy_no_skill": run_no_skill.validation_score,
        "skill_advantage": "positive" if cost_delta_pct > 0 else "neutral_or_negative"
    }


def compare_tool_impact(run_tq: RunResult, run_baseline_python: RunResult) -> dict[str, Any]:
    """Calculate the efficiency differential between tq CLI and default Python tools."""
    tokens_py = max(1, run_baseline_python.token_usage.total_tokens)
    tokens_tq = run_tq.token_usage.total_tokens
    token_savings_pct = round(((tokens_py - tokens_tq) / float(tokens_py)) * 100.0, 2)

    cost_py = max(0.000001, run_baseline_python.costs.get("total_effort_cost_usd", 0.0))
    cost_tq = run_tq.costs.get("total_effort_cost_usd", 0.0)
    cost_savings_pct = round(((cost_py - cost_tq) / cost_py) * 100.0, 2)

    duration_py = max(0.1, run_baseline_python.duration_seconds)
    duration_tq = run_tq.duration_seconds
    time_speedup_ratio = round(duration_py / max(0.1, duration_tq), 2)

    return {
        "model": run_tq.model,
        "tq_token_savings_pct": token_savings_pct,
        "tq_cost_savings_pct": cost_savings_pct,
        "speedup_ratio": time_speedup_ratio,
        "tq_score": run_tq.validation_score,
        "baseline_score": run_baseline_python.validation_score,
        "tq_advantage": "superior" if cost_savings_pct > 0 else "comparable"
    }


def compute_model_leaderboard(results: list[RunResult]) -> list[dict[str, Any]]:
    """Rank runs by overall effort cost, accuracy, and efficiency."""
    rows = []
    for r in results:
        rows.append({
            "run_id": r.run_id,
            "harness": r.harness,
            "model": r.model,
            "mode": r.mode,
            "score_pct": r.validation_score,
            "duration_s": r.duration_seconds,
            "total_tokens": r.token_usage.total_tokens,
            "cache_read_tokens": r.token_usage.cache_read_tokens,
            "query_count": r.db_metrics.query_count,
            "error_count": r.db_metrics.error_count,
            "db_cpu_sec": r.db_metrics.delta_cpu_sec,
            "db_io": r.db_metrics.delta_io,
            "token_cost_usd": r.costs.get("token_cost_usd", 0.0),
            "db_cost_usd": r.costs.get("database_cost_usd", 0.0),
            "total_effort_cost_usd": r.costs.get("total_effort_cost_usd", 0.0),
        })

    # Sort by score descending, then total effort cost ascending
    rows.sort(key=lambda x: (-x["score_pct"], x["total_effort_cost_usd"]))
    return rows
