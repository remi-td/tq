"""Claude Code headless agent harness adapter.

Invokes `claude -p` with JSON output format and permission bypass,
extracting exact token metrics, costs, and tool execution traces.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

from .base import AgentHarness, RunResult
from ..telemetry.token_telemetry import parse_claude_token_usage, TokenUsage
from ..telemetry.database_telemetry import DatabaseMetrics


class ClaudeCodeHarness(AgentHarness):
    """Executes benchmark tasks using the headless Claude Code CLI."""

    def execute(
        self,
        task_prompt: str,
        table_prefix: str,
        validation_spec: dict[str, Any],
        timeout_seconds: int = 300
    ) -> RunResult:
        run_id = f"claude_{self.model_id}_{self.mode}_{int(time.time())}"
        system_instructions = self.build_system_prompt()

        run_tag = f"tqbench_{table_prefix}"
        start_ts = self.db_telemetry.record_start_timestamp()
        t0 = time.time()

        system_file = self.workspace_dir / "bench_system_prompt.txt"
        system_file.write_text(system_instructions)

        cmd = [
            "claude",
            "-p", task_prompt,
            "--output-format", "json",
            "--model", self.model_id,
            "--dangerously-skip-permissions",
            "--append-system-prompt-file", str(system_file),
        ]

        commands_executed: list[str] = []
        token_usage = TokenUsage()
        agent_summary = ""
        error_msg = None

        try:
            sub_env = os.environ.copy()
            if self.mode in ("baseline-python", "baseline-no-tq"):
                claude_real = shutil.which("claude") or "/Users/remi.turpaud/.local/bin/claude"
                workspace_bin = self.workspace_dir / ".bench_bin"
                workspace_bin.mkdir(exist_ok=True)
                claude_link = workspace_bin / "claude"
                if not claude_link.exists() and os.path.exists(claude_real):
                    try:
                        claude_link.symlink_to(claude_real)
                    except Exception:
                        pass
                path_parts = sub_env.get("PATH", "").split(":")
                filtered_paths = [str(workspace_bin)] + [
                    p for p in path_parts
                    if not p.endswith(".local/bin") and "target/release" not in p and "target/debug" not in p
                ]
                sub_env["PATH"] = ":".join(filtered_paths)
                sub_env.pop("TQ_LOGON", None)
                sub_env.pop("TQ_QUERY_BAND", None)
                db_uri = sub_env.get("DATABASE_URI", "")
                if db_uri and "query_band=" not in db_uri:
                    sep = "&" if "?" in db_uri else "?"
                    sub_env["DATABASE_URI"] = f"{db_uri}{sep}query_band=ApplicationName=tq_bench;RunId={run_tag};"
            else:
                sub_env["TQ_QUERY_BAND"] = f"ApplicationName=tq_bench;RunId={run_tag};"
            proc = subprocess.run(
                cmd,
                cwd=str(self.workspace_dir),
                env=sub_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=timeout_seconds
            )
            duration = round(time.time() - t0, 2)

            if proc.stdout.strip():
                try:
                    data = json.loads(proc.stdout)
                    token_usage = parse_claude_token_usage(data)
                    agent_summary = str(data.get("result", ""))
                except Exception as e:
                    agent_summary = proc.stdout[:1000]
                    error_msg = f"Failed to parse Claude JSON output: {e}"
            else:
                error_msg = proc.stderr[:500] if proc.stderr else "Empty stdout from Claude CLI"

        except subprocess.TimeoutExpired:
            duration = round(time.time() - t0, 2)
            error_msg = f"Execution timed out after {timeout_seconds} seconds"
        except Exception as e:
            duration = round(time.time() - t0, 2)
            error_msg = str(e)

        # Collect database resource consumption from DBQL via QueryBand
        db_metrics = self.db_telemetry.collect_run_metrics(
            table_prefix=table_prefix,
            start_ts=start_ts,
            run_tag=run_tag
        )

        # Validate created objects and gold assertions against live Teradata instance
        val_res = self.db_telemetry.verify_benchmark_objects(table_prefix, validation_spec)

        # Compute cost metrics
        costs = self.cost_calculator.calculate_total_effort_cost(
            self.model_id,
            token_usage,
            amp_cpu_sec=db_metrics.delta_cpu_sec,
            amp_io=db_metrics.delta_io,
            req_spool=db_metrics.peak_spool_bytes
        )

        return RunResult(
            run_id=run_id,
            harness="claude-code",
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
