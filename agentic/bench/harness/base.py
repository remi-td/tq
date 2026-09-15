"""Base harness abstractions and run result structures.

Defines the AgentHarness lifecycle: workspace isolation, prompt generation,
execution protocol, and telemetry integration.
"""

from __future__ import annotations

import os
import shutil
from abc import ABC, abstractmethod
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any

from ..telemetry.token_telemetry import TokenUsage
from ..telemetry.database_telemetry import DatabaseMetrics, TeradataTelemetry
from ..telemetry.cost_calculator import CostCalculator

BENCH_DIR = Path(__file__).resolve().parent.parent
REPO_ROOT = BENCH_DIR.parent.parent
SKILL_FILE = REPO_ROOT / "agentic" / "skills" / "teradata-query" / "SKILL.md"
PYTHON_SKILL_FILE = REPO_ROOT / "agentic" / "skills" / "teradata-python" / "SKILL.md"


def resolve_database_uri() -> str:
    """Resolve database connection string from env, TQ_LOGON, or ~/.bash_profile."""
    uri = os.environ.get("DATABASE_URI", "")
    if uri:
        return uri
    logon = os.environ.get("TQ_LOGON", "")
    if logon:
        return f"teradata://{logon}" if "://" not in logon else logon
    bash_profile = Path.home() / ".bash_profile"
    if bash_profile.exists():
        for line in bash_profile.read_text().splitlines():
            line = line.strip()
            if line.startswith("export DATABASE_URI="):
                val = line.split("=", 1)[1].strip().strip('"').strip("'")
                if val:
                    return val
            elif line.startswith("export TQ_LOGON="):
                val = line.split("=", 1)[1].strip().strip('"').strip("'")
                if val:
                    return f"teradata://{val}" if "://" not in val else val
    return ""


@dataclass
class RunResult:
    """Consolidated outcome of an evaluation run."""
    run_id: str
    harness: str
    model: str
    mode: str
    dataset: str
    table_prefix: str
    duration_seconds: float = 0.0

    # Token & Database Telemetry
    token_usage: TokenUsage = field(default_factory=TokenUsage)
    db_metrics: DatabaseMetrics = field(default_factory=DatabaseMetrics)
    costs: dict[str, float] = field(default_factory=dict)

    # Verification & Quality
    validation_score: float = 0.0
    all_assertions_passed: bool = False
    validation_details: list[dict[str, Any]] = field(default_factory=list)

    # Execution Trace
    commands_executed: list[str] = field(default_factory=list)
    agent_summary: str = ""
    error_message: str | None = None

    def to_dict(self) -> dict[str, Any]:
        d = asdict(self)
        d["token_usage"] = self.token_usage.to_dict()
        d["db_metrics"] = self.db_metrics.to_dict()
        return d


class AgentHarness(ABC):
    """Abstract interface for agent execution harnesses."""

    def __init__(self, model_id: str, mode: str, workspace_dir: Path):
        self.model_id = model_id
        self.mode = mode
        self.workspace_dir = workspace_dir
        self.cost_calculator = CostCalculator()
        self.db_telemetry = TeradataTelemetry()

    def prepare_workspace(self, dataset_dir: Path) -> None:
        """Copy seed data into the isolated run workspace."""
        self.workspace_dir.mkdir(parents=True, exist_ok=True)
        seed_src = dataset_dir / "seed_data"
        seed_dest = self.workspace_dir / "seed_data"
        if seed_dest.exists():
            shutil.rmtree(seed_dest)
        if seed_src.exists():
            shutil.copytree(seed_src, seed_dest)

    def build_system_prompt(self) -> str:
        """Construct system instructions based on the evaluation mode."""
        base_prompt = (
            "You are an expert Data Engineer working on a Teradata database system. "
            "You write robust, performant SQL and data pipelines."
        )

        if self.mode == "tq-with-skill":
            skill_text = ""
            if SKILL_FILE.exists():
                skill_text = SKILL_FILE.read_text()
            return f"{base_prompt}\n\nAvailable Skill:\n{skill_text}"

        elif self.mode in ("baseline-python", "python-with-skill"):
            skill_text = ""
            if PYTHON_SKILL_FILE.exists():
                skill_text = PYTHON_SKILL_FILE.read_text()
            return f"{base_prompt}\n\nAvailable Skill:\n{skill_text}"

        elif self.mode in ("tq-no-skill", "baseline-no-skill", "baseline-no-tq"):
            return base_prompt

        return base_prompt

    def build_task_prompt(self, template_path: Path, table_prefix: str) -> str:
        """Render task prompt with unique table prefix."""
        text = template_path.read_text()
        text = text.replace("{{TABLE_PREFIX}}", table_prefix)
        return text.replace("{table_prefix}", table_prefix)


    @abstractmethod
    def execute(
        self,
        task_prompt: str,
        table_prefix: str,
        validation_spec: dict[str, Any],
        timeout_seconds: int = 300
    ) -> RunResult:
        """Execute the benchmark run and collect all telemetry."""
        pass
