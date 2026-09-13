"""Codex agent harness adapter.

Invokes `codex exec` in non-interactive JSON mode.
"""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

from .base import AgentHarness, RunResult
from ..telemetry.token_telemetry import TokenUsage
from ..telemetry.database_telemetry import DatabaseMetrics


class CodexHarness(AgentHarness):
    """Executes benchmark tasks using non-interactive Codex CLI."""

    def execute(
        self,
        task_prompt: str,
        table_prefix: str,
        validation_spec: dict[str, Any],
        timeout_seconds: int = 300
    ) -> RunResult:
        run_id = f"codex_{self.model_id}_{self.mode}_{int(time.time())}"
        system_instructions = self.build_system_prompt()
        full_prompt = f"{system_instructions}\n\n{task_prompt}"

        run_tag = f"tqbench_{table_prefix}"
        start_ts = self.db_telemetry.record_start_timestamp()
        t0 = time.time()

        cmd = [
            "codex", "exec",
            "--dangerously-bypass-approvals-and-sandbox",
            "--json",
            "-C", str(self.workspace_dir),
            full_prompt
        ]

        token_usage = TokenUsage()
        commands_executed: list[str] = []
        agent_summary = ""
        error_msg = None

        try:
            sub_env = os.environ.copy()
            sub_env["TQ_QUERY_BAND"] = f"ApplicationName=tq_bench;RunId={run_tag};"
            proc = subprocess.run(
                cmd,
                stdin=subprocess.DEVNULL,
                cwd=str(self.workspace_dir),
                env=sub_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=timeout_seconds
            )
            duration = round(time.time() - t0, 2)

            for line in proc.stdout.splitlines():
                if not line.strip():
                    continue
                try:
                    event = json.loads(line)
                    # Extract tokens from codex json stream if present
                    if event.get("type") == "token.usage":
                        token_usage.input_tokens += event.get("input_tokens", 0)
                        token_usage.output_tokens += event.get("output_tokens", 0)
                    elif event.get("type") == "item.completed":
                        item = event.get("item", {})
                        if item.get("type") == "message":
                            agent_summary = item.get("text", "")
                except Exception:
                    pass

            if proc.returncode != 0 and not error_msg:
                error_msg = f"Codex exited with code {proc.returncode}"

        except subprocess.TimeoutExpired:
            duration = round(time.time() - t0, 2)
            error_msg = f"Execution timed out after {timeout_seconds} seconds"
        except Exception as e:
            duration = round(time.time() - t0, 2)
            error_msg = str(e)

        token_usage.compute_total()

        # Collect database resource consumption from DBQL via QueryBand
        db_metrics = self.db_telemetry.collect_run_metrics(
            table_prefix=table_prefix,
            start_ts=start_ts,
            run_tag=run_tag
        )

        val_res = self.db_telemetry.verify_benchmark_objects(table_prefix, validation_spec)
        costs = self.cost_calculator.calculate_total_effort_cost(
            self.model_id,
            token_usage,
            amp_cpu_sec=db_metrics.delta_cpu_sec,
            amp_io=db_metrics.delta_io,
            req_spool=db_metrics.peak_spool_bytes
        )

        return RunResult(
            run_id=run_id,
            harness="codex",
            model=self.model_id,
            mode=self.mode,
            dataset="tpch_order_fulfillment",
            table_prefix=table_prefix,
            duration_seconds=duration,
            token_usage=token_usage,
            db_metrics=db_metrics,
            costs=costs,
            validation_score=val_res["score"],
            all_assertions_passed=val_res["passed"],
            validation_details=val_res["assertion_results"],
            commands_executed=commands_executed,
            agent_summary=agent_summary,
            error_message=error_msg,
        )
