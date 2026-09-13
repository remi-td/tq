"""Token telemetry extractor and data models.

Parses token metrics from Claude Code, Gemini, and Codex output formats.
Categorizes input, output, cache creation, cache read, and reasoning tokens.
"""

from __future__ import annotations

from dataclasses import dataclass, field, asdict
from typing import Any


@dataclass
class TokenUsage:
    """Detailed token usage categorization."""
    input_tokens: int = 0
    output_tokens: int = 0
    cache_read_tokens: int = 0
    cache_write_tokens: int = 0
    reasoning_tokens: int = 0
    total_tokens: int = 0
    raw_cost_usd: float = 0.0

    def compute_total(self) -> int:
        if self.total_tokens == 0:
            self.total_tokens = (
                self.input_tokens
                + self.output_tokens
                + self.cache_read_tokens
                + self.cache_write_tokens
            )
        return self.total_tokens

    def to_dict(self) -> dict[str, Any]:
        self.compute_total()
        return asdict(self)


def parse_claude_token_usage(json_data: dict[str, Any]) -> TokenUsage:
    """Extract TokenUsage from Claude Code headless JSON output."""
    usage = json_data.get("usage", {})
    details = usage.get("output_tokens_details", {})
    cache_creation = usage.get("cache_creation", {})

    input_tokens = usage.get("input_tokens", 0)
    output_tokens = usage.get("output_tokens", 0)
    cache_read = usage.get("cache_read_input_tokens", 0)
    cache_write = (
        usage.get("cache_creation_input_tokens", 0)
        or cache_creation.get("ephemeral_1h_input_tokens", 0)
        + cache_creation.get("ephemeral_5m_input_tokens", 0)
    )
    reasoning = details.get("thinking_tokens", 0)
    raw_cost = float(json_data.get("total_cost_usd", 0.0))

    t = TokenUsage(
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        cache_read_tokens=cache_read,
        cache_write_tokens=cache_write,
        reasoning_tokens=reasoning,
        raw_cost_usd=raw_cost,
    )
    t.compute_total()
    return t


def parse_gemini_token_usage(resp_usage: Any) -> TokenUsage:
    """Extract TokenUsage from Gemini API or OpenAI-compatible usage object."""
    if resp_usage is None:
        return TokenUsage()

    if isinstance(resp_usage, dict):
        prompt_tokens = resp_usage.get("prompt_tokens", 0)
        completion_tokens = resp_usage.get("completion_tokens", 0)
        details = resp_usage.get("prompt_tokens_details", {})
        cached_tokens = details.get("cached_tokens", 0)
        reasoning = resp_usage.get("completion_tokens_details", {}).get("reasoning_tokens", 0)
        total = resp_usage.get("total_tokens", 0)
    else:
        prompt_tokens = getattr(resp_usage, "prompt_tokens", 0)
        completion_tokens = getattr(resp_usage, "completion_tokens", 0)
        details = getattr(resp_usage, "prompt_tokens_details", None)
        cached_tokens = getattr(details, "cached_tokens", 0) if details else 0
        comp_details = getattr(resp_usage, "completion_tokens_details", None)
        reasoning = getattr(comp_details, "reasoning_tokens", 0) if comp_details else 0
        total = getattr(resp_usage, "total_tokens", 0)

    # Prompt tokens in OpenAI format often includes cached_tokens
    pure_input = max(0, prompt_tokens - cached_tokens)

    t = TokenUsage(
        input_tokens=pure_input,
        output_tokens=completion_tokens,
        cache_read_tokens=cached_tokens,
        cache_write_tokens=0,
        reasoning_tokens=reasoning,
        total_tokens=total,
    )
    t.compute_total()
    return t
