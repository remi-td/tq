"""Optimization Advisor and Recommendation Engine.

Analyzes telemetry and execution traces to detect inefficiencies and propose
concrete, actionable optimizations in the `tq` tool directly or companion skills.
"""

from __future__ import annotations

from dataclasses import dataclass, field, asdict
from typing import Any
from ..harness.base import RunResult


@dataclass
class Recommendation:
    """Actionable optimization proposal."""
    category: str  # "cli_feature", "skill_prompt", "database_optimization", "token_compression"
    target: str    # e.g. "tq CLI", "SKILL.md", "tq schema", "DDL templates"
    issue: str
    impact: str
    recommended_action: str

    def to_dict(self) -> dict[str, str]:
        return asdict(self)


class OptimizationAdvisor:
    """Scans benchmark traces and produces targeted recommendations."""

    def analyze_run(self, result: RunResult) -> list[Recommendation]:
        recs: list[Recommendation] = []
        trace = " ".join(result.commands_executed).lower()

        # 1. Token bloat / missing compression flags
        if result.token_usage.total_tokens > 15000 and "--format toon" not in trace and "--agent" not in trace:
            recs.append(Recommendation(
                category="token_compression",
                target="SKILL.md & CLI defaults",
                issue="Agent executed queries without token-compression flags (--agent or --format toon), consuming excess tokens on tabular responses.",
                impact=f"Consumed {result.token_usage.total_tokens} tokens. Using --format toon or --agent reduces token volume by 40-70%.",
                recommended_action="Update SKILL.md to mandate `--agent` or `--format toon` on all tabular data exploration queries, or default `tq` to compact JSON when non-TTY is detected."
            ))

        # 2. Raw DBC system queries instead of tq commands
        if "select * from dbc." in trace or "select tablename from dbc." in trace:
            recs.append(Recommendation(
                category="skill_prompt",
                target="SKILL.md",
                issue="Agent queried DBC catalog views directly with raw SQL instead of using dedicated tq commands (tq list, tq schema, tq inspect).",
                impact="Increases prompt token bloat and query latency; bypasses tq's optimized schema cache.",
                recommended_action="Strengthen SKILL.md rule: 'Always use `tq schema` or `tq inspect <table_name>` for schema discovery instead of querying DBC views directly.'"
            ))

        # 3. High CPU or I/O skew detection
        if result.db_metrics.avg_cpu_skew_pct > 25.0:
            recs.append(Recommendation(
                category="database_optimization",
                target="tq CLI hints",
                issue=f"Created tables exhibited high AMP CPU skew ({result.db_metrics.avg_cpu_skew_pct:.1f}%).",
                impact="Unbalanced AMP workload leads to database hotspotting and wasted CPU units.",
                recommended_action="Add automatic PI skew check to `tq` table creation hints: recommend selecting a high-cardinality primary index column matching downstream join keys."
            ))

        # 4. Incomplete or retried commands
        if result.error_message or result.validation_score < 100.0:
            recs.append(Recommendation(
                category="cli_feature",
                target="tq CLI error handling",
                issue=f"Run encountered errors or partial completion (Score: {result.validation_score}%). Details: {result.error_message or 'Missing assertions'}",
                impact="Causes agent to enter expensive retry loops, multiplying token and runtime costs.",
                recommended_action="Enhance error messaging in `tq query` with explicit remediation hints (e.g. Teradata 3932 DDL/BTET statement separation)."
            ))

        # 5. Token budget awareness
        if result.token_usage.total_tokens > 25000:
            recs.append(Recommendation(
                category="cli_feature",
                target="tq query --budget",
                issue="Query outputs exceeded optimal agent context budget.",
                impact=f"Excessive context consumption ({result.token_usage.total_tokens} tokens) inflates cost and degrades agent reasoning.",
                recommended_action="Recommend agents utilize the newly introduced `tq query --budget <TOKENS>` flag to automatically cap result set size."
            ))

        return recs

    def analyze_matrix(self, results: list[RunResult]) -> list[Recommendation]:
        """Aggregate recommendations across an entire matrix of runs."""
        all_recs: list[Recommendation] = []
        seen = set()
        for r in results:
            for rec in self.analyze_run(r):
                key = (rec.category, rec.target, rec.issue[:30])
                if key not in seen:
                    seen.add(key)
                    all_recs.append(rec)
        return all_recs
