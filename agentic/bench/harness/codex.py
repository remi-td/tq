"""Codex agent harness adapter.

Invokes `codex exec` in non-interactive JSON mode.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

from .base import AgentHarness, RunResult, resolve_database_uri
from ..telemetry.token_telemetry import TokenUsage
from ..telemetry.database_telemetry import DatabaseMetrics


def map_codex_model(model_id: str) -> str:
    """Map friendly model names to official Codex model slugs."""
    lower = model_id.lower()
    if lower in ("terra", "gpt-5.6-terra"):
        return "gpt-5.6-terra"
    elif lower in ("luna", "gpt-5.6-luna"):
        return "gpt-5.6-luna"
    elif lower in ("sol", "gpt-5.6-sol"):
        return "gpt-5.6-sol"
    return model_id


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

        codex_model = map_codex_model(self.model_id)

        cmd = [
            "codex", "exec",
            "--ignore-user-config",
            "--dangerously-bypass-approvals-and-sandbox",
            "--json",
            "--ephemeral",
            "-m", codex_model,
            "-C", str(self.workspace_dir),
            full_prompt
        ]

        token_usage = TokenUsage()
        commands_executed: list[str] = []
        agent_summary = ""
        error_msg = None

        try:
            sub_env = os.environ.copy()
            # Do not set queryband so comparison is strictly symmetrical
            sub_env.pop("TQ_QUERY_BAND", None)

            db_uri = resolve_database_uri()
            if db_uri:
                sub_env["DATABASE_URI"] = db_uri

            if self.mode in ("baseline-python", "baseline-no-tq"):
                codex_real = shutil.which("codex") or "/Applications/ChatGPT.app/Contents/Resources/codex"
                workspace_bin = self.workspace_dir / ".bench_bin"
                workspace_bin.mkdir(exist_ok=True)
                codex_link = workspace_bin / "codex"
                if not codex_link.exists() and os.path.exists(codex_real):
                    try:
                        codex_link.symlink_to(codex_real)
                    except Exception:
                        pass
                path_parts = sub_env.get("PATH", "").split(":")
                filtered_paths = [str(workspace_bin)] + [
                    p for p in path_parts
                    if not p.endswith(".local/bin") and "target/release" not in p and "target/debug" not in p
                ]
                sub_env["PATH"] = ":".join(filtered_paths)
                sub_env.pop("TQ_LOGON", None)

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
            end_ts = self.db_telemetry.record_end_timestamp()

            for line in proc.stdout.splitlines():
                if not line.strip():
                    continue
                try:
                    event = json.loads(line)
                    event_type = event.get("type")

                    if event_type == "turn.completed":
                        usage = event.get("usage", {})
                        inp = usage.get("input_tokens", 0)
                        cached_inp = usage.get("cached_input_tokens", 0)
                        cache_write = usage.get("cache_write_input_tokens", 0)
                        out = usage.get("output_tokens", 0)
                        reasoning = usage.get("reasoning_output_tokens", 0)
                        pure_inp = max(0, inp - cached_inp)

                        token_usage.input_tokens += pure_inp
                        token_usage.cache_read_tokens += cached_inp
                        token_usage.cache_write_tokens += cache_write
                        token_usage.output_tokens += out
                        token_usage.reasoning_tokens += reasoning

                    elif event_type == "item.completed":
                        item = event.get("item", {})
                        item_type = item.get("type")
                        if item_type == "agent_message":
                            agent_summary = item.get("text", "")
                        elif item_type == "command_execution":
                            cmd_str = item.get("command", "")
                            if cmd_str:
                                commands_executed.append(cmd_str)

                    elif event_type == "token.usage":
                        token_usage.input_tokens += event.get("input_tokens", 0)
                        token_usage.output_tokens += event.get("output_tokens", 0)

                except Exception:
                    pass

            if proc.returncode != 0 and not error_msg:
                error_msg = f"Codex exited with code {proc.returncode}"
                if proc.stderr:
                    error_msg += f": {proc.stderr[:200]}"

        except subprocess.TimeoutExpired:
            duration = round(time.time() - t0, 2)
            end_ts = self.db_telemetry.record_end_timestamp()
            error_msg = f"Execution timed out after {timeout_seconds} seconds"
        except Exception as e:
            duration = round(time.time() - t0, 2)
            end_ts = self.db_telemetry.record_end_timestamp()
            error_msg = str(e)

        token_usage.compute_total()

        # Collect database resource consumption from DBQL across stream execution time span
        db_metrics = self.db_telemetry.collect_run_metrics(
            start_ts=start_ts,
            end_ts=end_ts,
            table_prefix=table_prefix,
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
            dataset=validation_spec.get("dataset", "tpch_order_fulfillment"),
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

