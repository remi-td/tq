"""Report Generator for the Optimization Framework.

Formats benchmark results into executive Markdown reports and JSON data files.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any
from ..harness.base import RunResult
from .comparison import compute_model_leaderboard, compare_skill_impact, compare_tool_impact
from ..recommendations.advisor import OptimizationAdvisor, Recommendation


class ReportGenerator:
    """Generates Markdown reports and structured exports."""

    def __init__(self, advisor: OptimizationAdvisor | None = None):
        self.advisor = advisor or OptimizationAdvisor()

    def generate_markdown_report(
        self,
        results: list[RunResult],
        title: str = "tq Agentic Optimization & Evaluation Benchmark Report"
    ) -> str:
        leaderboard = compute_model_leaderboard(results)
        recommendations = self.advisor.analyze_matrix(results)

        lines = [
            f"# {title}",
            "",
            "> **Executive Summary:** This report benchmarks autonomous coding agents executing an end-to-end",
            "> Data Product pipeline on Teradata. It accounts for all expense items: LLM token consumption by",
            "> category (input, output, cache-read, reasoning) and induced database resource consumption (AMP CPU, I/O, Spool).",
            "",
            "## 1. Executive Leaderboard",
            "",
            "| Harness | Model | Mode | Score | Duration | Total Tokens | Cache Read % | Queries | DB CPU (s) | DB I/O | Token Cost ($) | DB Cost ($) | **Total Effort Cost ($)** |",
            "| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |"
        ]

        for r in leaderboard:
            cache_pct = 0.0
            if r["total_tokens"] > 0:
                cache_pct = round((r["cache_read_tokens"] / float(r["total_tokens"])) * 100.0, 1)

            lines.append(
                f"| `{r['harness']}` | `{r['model']}` | `{r['mode']}` | {r['score_pct']}% | {r['duration_s']}s | {r['total_tokens']:,} | {cache_pct}% | {r.get('query_count', 0)} | {r['db_cpu_sec']:.3f} | {r.get('db_io', 0):,} | ${r['token_cost_usd']:.4f} | ${r['db_cost_usd']:.4f} | **${r['total_effort_cost_usd']:.4f}** |"
            )

        lines.extend([
            "",
            "---",
            "",
            "## 2. Comparative Impact Analysis",
            ""
        ])

        # Skill Impact comparisons
        skill_pairs: list[tuple[RunResult, RunResult]] = []
        for r_skill in results:
            if r_skill.mode == "tq-with-skill":
                for r_no in results:
                    if r_no.mode == "tq-no-skill" and r_no.model == r_skill.model and r_no.harness == r_skill.harness:
                        skill_pairs.append((r_skill, r_no))

        if skill_pairs:
            lines.append("### A. Skill Impact (`tq-with-skill` vs `tq-no-skill`)")
            lines.append("")
            lines.append("| Model | Token Savings % | Total Effort Cost Savings % | DB CPU Savings % | Accuracy Delta | Advantage |")
            lines.append("| :--- | :---: | :---: | :---: | :---: | :---: |")
            for r_s, r_n in skill_pairs:
                cmp = compare_skill_impact(r_s, r_n)
                lines.append(
                    f"| `{cmp['model']}` | {cmp['token_reduction_pct']}% | {cmp['cost_reduction_pct']}% | {cmp['database_cpu_savings_pct']}% | {cmp['accuracy_with_skill']}% vs {cmp['accuracy_no_skill']}% | **{cmp['skill_advantage']}** |"
                )
            lines.append("")

        # Tool Impact comparisons (tq vs baseline python)
        tool_pairs: list[tuple[RunResult, RunResult]] = []
        for r_tq in results:
            if r_tq.mode == "tq-with-skill":
                for r_py in results:
                    if r_py.mode == "baseline-python" and r_py.model == r_tq.model:
                        tool_pairs.append((r_tq, r_py))

        if tool_pairs:
            lines.append("### B. Tool Acceleration (`tq` vs `baseline-python`)")
            lines.append("")
            lines.append("| Model | Token Savings % | Cost Savings % | Speedup Ratio | `tq` Score | Baseline Score |")
            lines.append("| :--- | :---: | :---: | :---: | :---: | :---: |")
            for r_t, r_p in tool_pairs:
                cmp = compare_tool_impact(r_t, r_p)
                lines.append(
                    f"| `{cmp['model']}` | {cmp['tq_token_savings_pct']}% | {cmp['tq_cost_savings_pct']}% | {cmp['speedup_ratio']}x | {cmp['tq_score']}% | {cmp['baseline_score']}% |"
                )
            lines.append("")

        lines.extend([
            "---",
            "",
            "## 3. Actionable Optimization Recommendations",
            ""
        ])

        if recommendations:
            for idx, rec in enumerate(recommendations, 1):
                lines.extend([
                    f"### {idx}. [{rec.category.upper()}] {rec.target}",
                    f"- **Identified Issue:** {rec.issue}",
                    f"- **Observed Impact:** {rec.impact}",
                    f"- **Recommended Action:** {rec.recommended_action}",
                    ""
                ])
        else:
            lines.append("No critical inefficiencies detected across the benchmark matrix.")

        lines.extend([
            "---",
            "",
            "## 4. Run Details & Validation Breakdown",
            ""
        ])

        for r in results:
            lines.append(f"### Run: `{r.run_id}`")
            lines.append(f"- **Harness / Model:** `{r.harness}` / `{r.model}` ({r.mode})")
            lines.append(f"- **Duration:** {r.duration_seconds}s | **Score:** {r.validation_score}%")
            lines.append(f"- **Tokens:** {r.token_usage.total_tokens:,} (Input: {r.token_usage.input_tokens:,}, Output: {r.token_usage.output_tokens:,}, Cache Read: {r.token_usage.cache_read_tokens:,})")
            lines.append(f"- **Database Telemetry (DBQL):** Queries: {r.db_metrics.query_count}, Errors: {r.db_metrics.error_count}, AMP CPU: {r.db_metrics.delta_cpu_sec:.3f}s, I/O: {r.db_metrics.delta_io:,}, Peak Spool: {r.db_metrics.peak_spool_bytes:,} bytes")
            lines.append(f"- **Total Effort Cost:** **${r.costs.get('total_effort_cost_usd', 0.0):.4f}**")
            lines.append("- **Assertion Details:**")
            for a in r.validation_details:
                status = "✅ PASS" if a.get("passed") else "❌ FAIL"
                lines.append(f"  - {status}: **{a.get('assertion_id')}** - {a.get('details')}")
            if r.error_message:
                lines.append(f"- **Error:** `{r.error_message}`")
            lines.append("")

        return "\n".join(lines)

    def save_report(self, results: list[RunResult], output_dir: Path | str) -> tuple[Path, Path]:
        """Write both Markdown report and JSON dataset to disk."""
        out = Path(output_dir)
        out.mkdir(parents=True, exist_ok=True)

        md_content = self.generate_markdown_report(results)
        md_path = out / "optimisation_report.md"
        md_path.write_text(md_content)

        json_data = {
            "runs": [r.to_dict() for r in results],
            "leaderboard": compute_model_leaderboard(results),
            "recommendations": [rec.to_dict() for rec in self.advisor.analyze_matrix(results)]
        }
        json_path = out / "optimisation_report.json"
        json_path.write_text(json.dumps(json_data, indent=2))

        return md_path, json_path
