"""Gemini agent harness adapter.

Uses Google Generative Language API with tool loop execution.
Provides `execute_command` tool to run tq or shell commands in the isolated workspace.
Captures full token usage, turn iterations, and execution traces.
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

from .base import AgentHarness, RunResult
from ..telemetry.token_telemetry import parse_gemini_token_usage, TokenUsage
from ..telemetry.database_telemetry import DatabaseMetrics


class GeminiHarness(AgentHarness):
    """Executes benchmark tasks using Google Gemini models with a tool execution loop."""

    def __init__(self, model_id: str, mode: str, workspace_dir: Path):
        super().__init__(model_id, mode, workspace_dir)
        self.api_key = self._resolve_api_key()

    def _resolve_api_key(self) -> str:
        key = os.environ.get("GEMINI_API_KEY", "")
        if not key:
            # Check ~/.bash_profile
            bash_profile = Path.home() / ".bash_profile"
            if bash_profile.exists():
                for line in bash_profile.read_text().splitlines():
                    if "GEMINI_API_KEY" in line and "=" in line:
                        parts = line.split("=", 1)[1].strip().strip('"').strip("'")
                        if parts:
                            key = parts
                            os.environ["GEMINI_API_KEY"] = key
                            break
        return key

    def execute(
        self,
        task_prompt: str,
        table_prefix: str,
        validation_spec: dict[str, Any],
        timeout_seconds: int = 300,
        max_turns: int = 20
    ) -> RunResult:
        run_id = f"gemini_{self.model_id}_{self.mode}_{int(time.time())}"
        system_instructions = self.build_system_prompt()

        from openai import OpenAI
        client = OpenAI(
            base_url=os.environ.get("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai/"),
            api_key=self.api_key
        )

        tools = [
            {
                "type": "function",
                "function": {
                    "name": "execute_command",
                    "description": "Execute a terminal shell command in the workspace to inspect files, run scripts, or perform operations.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The exact shell command to execute"
                            }
                        },
                        "required": ["command"]
                    }
                }
            }
        ]

        messages = [
            {"role": "system", "content": system_instructions},
            {"role": "user", "content": task_prompt}
        ]

        run_tag = f"tqbench_{table_prefix}"
        start_ts = self.db_telemetry.record_start_timestamp()
        t0 = time.time()

        total_input_tokens = 0
        total_output_tokens = 0
        total_cached_tokens = 0
        commands_executed: list[str] = []
        agent_summary = ""
        error_msg = None

        for turn in range(max_turns):
            if time.time() - t0 > timeout_seconds:
                error_msg = f"Timeout reached after {timeout_seconds}s"
                break

            try:
                resp = client.chat.completions.create(
                    model=self.model_id,
                    messages=messages,
                    tools=tools,
                    tool_choice="auto"
                )
            except Exception as e:
                error_msg = f"Gemini API error: {e}"
                break

            # Accumulate usage
            usage = getattr(resp, "usage", None)
            if usage:
                p_tok = getattr(usage, "prompt_tokens", 0)
                c_tok = getattr(usage, "completion_tokens", 0)
                details = getattr(usage, "prompt_tokens_details", None)
                cached = getattr(details, "cached_tokens", 0) if details else 0

                total_input_tokens += max(0, p_tok - cached)
                total_output_tokens += c_tok
                total_cached_tokens += cached

            choice = resp.choices[0]
            msg = choice.message

            messages.append(msg)

            if not msg.tool_calls:
                agent_summary = msg.content or ""
                break

            # Execute tool calls
            for tc in msg.tool_calls:
                fn_name = tc.function.name
                try:
                    args = json.loads(tc.function.arguments)
                except Exception:
                    args = {}

                if fn_name == "execute_command":
                    cmd_str = args.get("command", "")
                    commands_executed.append(cmd_str)
                    print(f"   [{turn+1}] Executing: {cmd_str[:90]}...", flush=True)
                    proc = None
                    try:
                        sub_env = os.environ.copy()
                        if self.mode in ("baseline-python", "baseline-no-tq"):
                            path_parts = sub_env.get("PATH", "").split(":")
                            sub_env["PATH"] = ":".join(p for p in path_parts if not p.endswith(".local/bin") and "target/release" not in p and "target/debug" not in p)
                            sub_env.pop("TQ_LOGON", None)
                            sub_env.pop("TQ_QUERY_BAND", None)
                            db_uri = sub_env.get("DATABASE_URI", "")
                            if db_uri and "query_band=" not in db_uri:
                                sep = "&" if "?" in db_uri else "?"
                                sub_env["DATABASE_URI"] = f"{db_uri}{sep}query_band=ApplicationName=tq_bench;RunId={run_tag};"
                        else:
                            sub_env["TQ_QUERY_BAND"] = f"ApplicationName=tq_bench;RunId={run_tag};"
                        proc = subprocess.Popen(
                            cmd_str,
                            shell=True,
                            cwd=str(self.workspace_dir),
                            env=sub_env,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                            text=True,
                            preexec_fn=os.setsid
                        )
                        stdout, stderr = proc.communicate(timeout=60)
                        output = stdout
                        if stderr:
                            output += "\nSTDERR:\n" + stderr
                        if not output.strip():
                            output = "(Command executed successfully with no output)"
                    except subprocess.TimeoutExpired:
                        if proc:
                            try:
                                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
                            except Exception:
                                pass
                        output = "Error: Command timed out after 60 seconds."
                    except Exception as e:
                        if proc:
                            try:
                                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
                            except Exception:
                                pass
                        output = f"Execution error: {e}"
                else:
                    output = f"Unknown tool: {fn_name}"

                messages.append({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "name": fn_name,
                    "content": output[:8000]
                })

        duration = round(time.time() - t0, 2)
        token_usage = TokenUsage(
            input_tokens=total_input_tokens,
            output_tokens=total_output_tokens,
            cache_read_tokens=total_cached_tokens,
            total_tokens=total_input_tokens + total_output_tokens + total_cached_tokens
        )

        # Collect database resource consumption from DBQL via QueryBand
        db_metrics = self.db_telemetry.collect_run_metrics(
            table_prefix=table_prefix,
            start_ts=start_ts,
            run_tag=run_tag
        )
        if db_metrics.query_count == 0 and commands_executed:
            db_metrics.query_count = len(commands_executed)

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
            harness="gemini",
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
