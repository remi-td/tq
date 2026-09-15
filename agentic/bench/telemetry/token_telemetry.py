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


def parse_pi_session_file(session_file_path: str) -> tuple[TokenUsage, list[str], str]:
    """Extract TokenUsage, executed bash commands, and final summary from a Pi session JSONL file."""
    import json
    from pathlib import Path

    path = Path(session_file_path)
    if not path.exists():
        return TokenUsage(), [], ""

    total_input = 0
    total_output = 0
    total_cache_read = 0
    total_cache_write = 0
    total_reasoning = 0
    total_cost = 0.0
    commands_executed: list[str] = []
    final_summary = ""

    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except Exception:
            continue

        if entry.get("type") == "message":
            msg = entry.get("message", {})
            role = msg.get("role")
            if role == "assistant":
                usage = msg.get("usage", {})
                if usage:
                    total_input += usage.get("input", 0)
                    total_output += usage.get("output", 0)
                    total_cache_read += usage.get("cacheRead", 0)
                    total_cache_write += usage.get("cacheWrite", 0)
                    total_reasoning += usage.get("reasoning", 0)
                    cost = usage.get("cost", {})
                    total_cost += float(cost.get("total", 0.0))

                content = msg.get("content", [])
                if isinstance(content, list):
                    for item in content:
                        if item.get("type") == "toolCall" and item.get("name") == "bash":
                            args = item.get("arguments", {})
                            cmd = args.get("command", "")
                            if cmd:
                                commands_executed.append(cmd)
                        elif item.get("type") == "text":
                            final_summary = item.get("text", "")
                elif isinstance(content, str):
                    final_summary = content

    token_usage = TokenUsage(
        input_tokens=total_input,
        output_tokens=total_output,
        cache_read_tokens=total_cache_read,
        cache_write_tokens=total_cache_write,
        reasoning_tokens=total_reasoning,
        total_tokens=total_input + total_output + total_cache_read + total_cache_write,
        raw_cost_usd=round(total_cost, 6)
    )
    return token_usage, commands_executed, final_summary

