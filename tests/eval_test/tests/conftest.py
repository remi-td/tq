"""Shared fixtures and hooks for the Teradata eval suite."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest
from dotenv import load_dotenv

load_dotenv()

CASES_DIR = Path(__file__).parent.parent / "cases"
MODULES = ["base", "dba", "sec", "qlty", "chat", "plot", "tmpl"]


def _substitute(obj, evals_db: str):
    """Recursively replace {EVALS_DATABASE} in strings within dicts/lists."""
    if isinstance(obj, str):
        return obj.replace("{EVALS_DATABASE}", evals_db)
    if isinstance(obj, dict):
        return {k: _substitute(v, evals_db) for k, v in obj.items()}
    if isinstance(obj, list):
        return [_substitute(i, evals_db) for i in obj]
    return obj


def load_cases(module: str) -> list[dict]:
    path = CASES_DIR / f"{module}.json"
    if not path.exists():
        return []
    data = json.loads(path.read_text())
    return [c for c in data.get("cases", []) if "id" in c]


def assert_eval_case(case: dict, bedrock_client, agent_model_id: str, judge_llm) -> None:
    """Run and score any eval case (single- or multi-turn)."""
    from tests.case_runner import assert_eval_case as _assert_eval_case

    evals_db = os.environ.get("EVALS_DATABASE", "").strip()
    resolved = _substitute(case, evals_db)
    _assert_eval_case(resolved, bedrock_client, agent_model_id, judge_llm)


@pytest.fixture(scope="session")
def bedrock_client():
    # Return bedrock client if provider is bedrock, otherwise None
    provider = os.environ.get("EVALS_PROVIDER", "bedrock").lower()
    if provider == "bedrock":
        import boto3
        region = os.environ.get("AWS_REGION", "us-east-1")
        return boto3.client("bedrock-runtime", region_name=region)
    return None


@pytest.fixture(scope="session")
def agent_model_id() -> str:
    provider = os.environ.get("EVALS_PROVIDER", "bedrock").lower()
    if provider == "bedrock":
        return os.environ.get("BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20241022-v2:0")
    elif provider == "openai":
        return os.environ.get("OPENAI_MODEL_ID", "gpt-4o")
    elif provider == "openrouter":
        return os.environ.get("OPENROUTER_MODEL_ID", "anthropic/claude-3.5-sonnet")
    elif provider == "gemini":
        return os.environ.get("GEMINI_MODEL_ID", "gemini-2.5-flash")
    return "unknown"


@pytest.fixture(scope="session")
def judge_llm(bedrock_client):
    from judge.universal_llm import UniversalLLM
    return UniversalLLM()


def pytest_sessionstart(session) -> None:
    """Initialize eval result collection for live eval runs."""
    from agent.client import get_description_override_status
    from judge.report import begin_eval_run

    provider = os.environ.get("EVALS_PROVIDER", "bedrock").lower()
    agent_model = os.environ.get("EVALS_AGENT_MODEL")
    if not agent_model:
        if provider == "bedrock":
            agent_model = os.environ.get("BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20241022-v2:0")
        elif provider == "openai":
            agent_model = os.environ.get("OPENAI_MODEL_ID", "gpt-4o")
        elif provider == "openrouter":
            agent_model = os.environ.get("OPENROUTER_MODEL_ID", "anthropic/claude-3.5-sonnet")
        elif provider == "gemini":
            agent_model = os.environ.get("GEMINI_MODEL_ID", "gemini-2.5-flash")

    judge_model = os.environ.get("EVALS_JUDGE_MODEL", agent_model)
    override_status = get_description_override_status()
    
    # Set mode as combination of EVALS_MODE and description override status
    evals_mode = os.environ.get("EVALS_MODE", "tq-cli")
    desc_mode = f"{evals_mode}_{override_status['mode']}"

    begin_eval_run(
        agent_model_id=agent_model or "unknown",
        judge_model_id=judge_model or "unknown",
        evals_database=os.environ.get("EVALS_DATABASE", "").strip(),
        description_mode=desc_mode,
        description_overrides_file=override_status.get("file"),
        description_override_count=int(override_status.get("tool_count") or 0),
    )


def pytest_sessionfinish(session, exitstatus) -> None:
    """Write a markdown/json summary when live eval cases were executed."""
    from judge.report import get_current_report, write_eval_summary

    report = get_current_report()
    if report is None or not report.results:
        return

    artifacts = write_eval_summary(report)
    terminal = session.config.pluginmanager.get_plugin("terminalreporter")
    if terminal is not None:
        total_tokens = sum(r.total_tokens for r in report.results)
        total_cost = sum(r.cost for r in report.results)
        terminal.write_line("")
        terminal.write_line(f"Eval run: {artifacts.run_id}")
        terminal.write_line(f"Run directory: results/{artifacts.run_dir.name}")
        terminal.write_line(f"Total agent tokens consumed: {total_tokens:,} | Cost: ${total_cost:.5f}")
        terminal.write_line("Summary: results/latest_summary.md (copy of this run)")
        terminal.write_line("Index: results/index.json")
