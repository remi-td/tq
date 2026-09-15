"""Pi agent harness adapter.

Invokes `@earendil-works/pi-coding-agent` via `pi -p` in non-interactive print mode,
capturing full execution traces, bash tool calls, and detailed token telemetry from session JSONL records.
"""

from __future__ import annotations

import glob
import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

from .base import AgentHarness, RunResult, resolve_database_uri
from ..telemetry.token_telemetry import TokenUsage, parse_pi_session_file
from ..telemetry.database_telemetry import DatabaseMetrics


def resolve_pi_provider_and_model(model_id: str) -> tuple[str, str]:
    """Map model names to (provider, model_id) for Pi CLI."""
    if "/" in model_id:
        provider, model = model_id.split("/", 1)
        return provider, model

    lower = model_id.lower()
    if "gemini" in lower or "gemma" in lower:
        return "google", model_id
    elif "claude" in lower or lower in ("sonnet", "haiku", "opus"):
        model_name = model_id
        if lower == "sonnet":
            model_name = "claude-sonnet"
        elif lower in ("haiku", "claude-haiku-4-5"):
            model_name = "claude-haiku"
        return "anthropic", model_name
    elif "gpt" in lower or "codex" in lower:
        return "openai", model_id
    return "google", model_id


class PiHarness(AgentHarness):
    """Executes benchmark tasks using the Pi coding agent CLI."""

    def execute(
        self,
        task_prompt: str,
        table_prefix: str,
        validation_spec: dict[str, Any],
        timeout_seconds: int = 300
    ) -> RunResult:
        run_id = f"pi_{self.model_id.replace('/', '_')}_{self.mode}_{int(time.time())}"
        provider, resolved_model = resolve_pi_provider_and_model(self.model_id)

        system_instructions = self.build_system_prompt()
        full_prompt = f"{system_instructions}\n\n{task_prompt}"

        run_tag = f"tqbench_{table_prefix}"
        start_ts = self.db_telemetry.record_start_timestamp()
        t0 = time.time()

        session_dir = self.workspace_dir / ".pi_sessions"
        session_dir.mkdir(parents=True, exist_ok=True)

        cmd = [
            "pi",
            "--provider", provider,
            "--model", resolved_model,
            "--approve",
            "--no-context-files",
            "--session-dir", str(session_dir),
            "-p", full_prompt
        ]

        # In tq-with-skill mode, also pass skill flag if available
        if self.mode == "tq-with-skill":
            skill_dir = Path(__file__).resolve().parent.parent.parent / "skills" / "teradata-query"
            if skill_dir.exists():
                cmd.extend(["--skill", str(skill_dir)])

        token_usage = TokenUsage()
        commands_executed: list[str] = []
        agent_summary = ""
        error_msg = None

        try:
            sub_env = os.environ.copy()
            sub_env.pop("TQ_QUERY_BAND", None)

            # Ensure GEMINI_API_KEY is in sub_env if not already exported
            if "GEMINI_API_KEY" not in sub_env:
                bash_profile = Path.home() / ".bash_profile"
                if bash_profile.exists():
                    for line in bash_profile.read_text().splitlines():
                        if "GEMINI_API_KEY" in line and "=" in line:
                            sub_env["GEMINI_API_KEY"] = line.split("=", 1)[1].strip().strip('"').strip("'")
                            break

            db_uri = resolve_database_uri()
            if db_uri:
                sub_env["DATABASE_URI"] = db_uri

            if self.mode in ("baseline-python", "baseline-no-tq"):
                pi_real = shutil.which("pi") or "/opt/homebrew/bin/pi"
                workspace_bin = self.workspace_dir / ".bench_bin"
                workspace_bin.mkdir(exist_ok=True)
                pi_link = workspace_bin / "pi"
                if not pi_link.exists() and os.path.exists(pi_real):
                    try:
                        pi_link.symlink_to(pi_real)
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

            # Find the newest session JSONL file in session_dir
            session_files = sorted(session_dir.glob("*.jsonl"), key=os.path.getmtime, reverse=True)
            if session_files:
                token_usage, commands_executed, session_summary = parse_pi_session_file(str(session_files[0]))
                if session_summary:
                    agent_summary = session_summary
                elif proc.stdout.strip():
                    agent_summary = proc.stdout.strip()
            else:
                agent_summary = proc.stdout.strip()

            if proc.returncode != 0 and not error_msg:
                error_msg = f"Pi process exited with code {proc.returncode}: {proc.stderr[:300]}"

        except subprocess.TimeoutExpired:
            duration = timeout_seconds
            end_ts = self.db_telemetry.record_end_timestamp()
            error_msg = f"Timeout reached after {timeout_seconds}s"
        except Exception as e:
            duration = round(time.time() - t0, 2)
            end_ts = self.db_telemetry.record_end_timestamp()
            error_msg = f"Execution error: {e}"

        # Collect DBQL metrics
        db_metrics = self.db_telemetry.collect_run_metrics(
            start_ts=start_ts,
            end_ts=end_ts,
            table_prefix=table_prefix,
            run_tag=run_tag
        )
        if db_metrics.query_count == 0 and commands_executed:
            db_metrics.query_count = len(commands_executed)

        # Assertion validation
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
            harness="pi",
            model=self.model_id,
            mode=self.mode,
            dataset=validation_spec.get("dataset_name", "tpch_order_fulfillment"),
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
            error_message=error_msg
        )
