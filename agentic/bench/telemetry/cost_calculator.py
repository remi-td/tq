"""Cost calculator for LLM tokens and Teradata database consumption.

Calculates total monetary effort cost based on token categories and database metrics.
"""

from __future__ import annotations

import yaml
from pathlib import Path
from typing import Any
from .token_telemetry import TokenUsage

CONFIG_DIR = Path(__file__).resolve().parent.parent / "configs"
DEFAULT_PRICING_FILE = CONFIG_DIR / "pricing_rates.yaml"


class CostCalculator:
    """Calculates unified effort cost."""

    def __init__(self, pricing_file: Path | str | None = None):
        self.pricing_file = Path(pricing_file) if pricing_file else DEFAULT_PRICING_FILE
        self.pricing = self._load_pricing()

    def _load_pricing(self) -> dict[str, Any]:
        if not self.pricing_file.exists():
            return {}
        with open(self.pricing_file, "r") as f:
            return yaml.safe_load(f) or {}

    def get_model_rates(self, model_id: str) -> dict[str, float]:
        models = self.pricing.get("models", {})
        # Exact match or prefix match
        if model_id in models:
            return models[model_id]

        for k, v in models.items():
            if k in model_id or model_id.startswith(k):
                return v

        return models.get("default", {
            "input_per_million": 2.0,
            "output_per_million": 8.0,
            "cache_read_per_million": 0.2,
            "cache_write_per_million": 2.5,
        })

    def calculate_token_cost(self, model_id: str, usage: TokenUsage) -> float:
        """Calculate LLM cost based on categorized tokens."""
        # If the harness already provided a verified total_cost_usd, use it as baseline
        if usage.raw_cost_usd > 0.0:
            return round(usage.raw_cost_usd, 6)

        rates = self.get_model_rates(model_id)
        input_cost = (usage.input_tokens / 1_000_000.0) * rates.get("input_per_million", 2.0)
        output_cost = (usage.output_tokens / 1_000_000.0) * rates.get("output_per_million", 8.0)
        cache_read_cost = (usage.cache_read_tokens / 1_000_000.0) * rates.get("cache_read_per_million", 0.2)
        cache_write_cost = (usage.cache_write_tokens / 1_000_000.0) * rates.get("cache_write_per_million", 2.5)

        total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost
        return round(total_cost, 6)

    def calculate_database_cost(self, amp_cpu_sec: float, amp_io: int, req_spool: int) -> float:
        """Calculate Teradata cloud consumption cost."""
        db_rates = self.pricing.get("database", {})
        cpu_rate = db_rates.get("cost_per_amp_cpu_second", 0.005)
        io_rate = db_rates.get("cost_per_thousand_io", 0.0001)

        cpu_cost = amp_cpu_sec * cpu_rate
        io_cost = (amp_io / 1000.0) * io_rate
        return round(cpu_cost + io_cost, 6)

    def calculate_total_effort_cost(
        self,
        model_id: str,
        token_usage: TokenUsage,
        amp_cpu_sec: float = 0.0,
        amp_io: int = 0,
        req_spool: int = 0
    ) -> dict[str, float]:
        """Combine LLM token cost and database consumption cost into total effort cost."""
        token_cost = self.calculate_token_cost(model_id, token_usage)
        db_cost = self.calculate_database_cost(amp_cpu_sec, amp_io, req_spool)
        total_effort_cost = round(token_cost + db_cost, 6)

        return {
            "token_cost_usd": token_cost,
            "database_cost_usd": db_cost,
            "total_effort_cost_usd": total_effort_cost,
        }
