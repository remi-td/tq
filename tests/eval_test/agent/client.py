"""Unified agent client supporting Bedrock, OpenAI, OpenRouter, and Gemini."""

from __future__ import annotations

import asyncio
import json
import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client

from agent.tq_executor import run_tq, load_mcp_tool_schemas

MAX_TOOL_RESULT_CHARS = int(os.environ.get("MAX_TOOL_RESULT_CHARS", "8000"))
REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent


class MockTool:
    def __init__(self, name: str, spec: dict[str, Any]):
        self.name = name
        self.description = spec.get("description", "")
        
        # Convert parameters to json schema
        params = spec.get("parameters", {})
        properties = {}
        required = []
        for p_name, p_spec in params.items():
            t_type = "string"
            if p_spec.get("type_hint") == "int":
                t_type = "integer"
            elif p_spec.get("type_hint") == "bool":
                t_type = "boolean"
            
            properties[p_name] = {
                "type": t_type,
                "description": p_spec.get("description", "")
            }
            if p_spec.get("required", False):
                required.append(p_name)
                
        self.inputSchema = {
            "type": "object",
            "properties": properties
        }
        if required:
            self.inputSchema["required"] = required

    def model_copy(self, update: dict | None = None) -> MockTool:
        tool = MockTool(self.name, {})
        tool.description = self.description
        tool.inputSchema = self.inputSchema
        if update:
            if "description" in update:
                tool.description = update["description"]
        return tool


@dataclass
class TokenUsage:
    input_tokens: int = 0
    output_tokens: int = 0
    total_tokens: int = 0
    cost: float = 0.0


@dataclass
class ToolCallRecord:
    name: str
    input_parameters: dict[str, Any]


@dataclass
class TurnResult:
    tool_calls: list[ToolCallRecord]
    final_response: str
    token_usage: TokenUsage = field(default_factory=TokenUsage)


@dataclass
class AgentResult:
    tool_calls: list[ToolCallRecord]
    final_response: str
    token_usage: TokenUsage = field(default_factory=TokenUsage)


def calculate_cost(model_id: str, input_tokens: int, output_tokens: int) -> float:
    model_id_lower = model_id.lower()
    input_cost = 0.0
    output_cost = 0.0
    
    if "gemini-2.5-flash" in model_id_lower:
        input_cost = 0.075
        output_cost = 0.30
    elif "gpt-4o" in model_id_lower:
        input_cost = 5.00
        output_cost = 15.00
    elif "claude-3-5-sonnet" in model_id_lower or "claude-3.5-sonnet" in model_id_lower:
        input_cost = 3.00
        output_cost = 15.00
    elif "claude-3-sonnet" in model_id_lower:
        input_cost = 3.00
        output_cost = 15.00
        
    return (input_tokens * input_cost + output_tokens * output_cost) / 1_000_000.0


def description_overrides_enabled() -> bool:
    if os.environ.get("DESCRIPTION_OVERRIDES_FILE"):
        return True
    return os.environ.get("USE_DESCRIPTION_OVERRIDES", "").lower() in {"1", "true", "yes"}


def resolve_description_overrides_file() -> Path | None:
    if not description_overrides_enabled():
        return None
    env_path = os.environ.get("DESCRIPTION_OVERRIDES_FILE")
    if env_path:
        return Path(env_path)
    default_path = REPO_ROOT / "tests" / "eval_test" / "description_overrides.json"
    if default_path.exists():
        return default_path
    return None


def get_description_override_status() -> dict[str, str | int | None]:
    if not description_overrides_enabled():
        return {"mode": "live", "file": None, "tool_count": 0}
    overrides_file = resolve_description_overrides_file()
    count = 0
    if overrides_file and overrides_file.exists():
        try:
            count = len(json.loads(overrides_file.read_text()))
        except Exception:
            pass
    return {
        "mode": "overrides",
        "file": str(overrides_file) if overrides_file else None,
        "tool_count": count
    }


def _load_description_overrides() -> dict[str, str]:
    if not description_overrides_enabled():
        return {}
    overrides_file = resolve_description_overrides_file()
    if overrides_file is None or not overrides_file.exists():
        return {}
    try:
        data = json.loads(overrides_file.read_text())
        if isinstance(data, dict):
            return {k: v for k, v in data.items() if isinstance(v, str)}
    except Exception:
        pass
    return {}


def _apply_description_overrides(tools: list, overrides: dict[str, str]) -> list:
    if not overrides:
        return tools
    patched = []
    for tool in tools:
        name = getattr(tool, "name", None)
        if name and name in overrides:
            tool = tool.model_copy(update={"description": overrides[name]})
        patched.append(tool)
    return patched


def _mcp_tool_to_bedrock(tool) -> dict:
    return {
        "toolSpec": {
            "name": tool.name,
            "description": tool.description or "",
            "inputSchema": {"json": tool.inputSchema},
        }
    }


def _mcp_tool_to_openai(tool) -> dict:
    return {
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description or "",
            "parameters": tool.inputSchema,
        }
    }


def _get_skill_system_prompt() -> str:
    skill_path = REPO_ROOT / "agentic" / "skills" / "teradata-query" / "SKILL.md"
    if skill_path.exists():
        try:
            return skill_path.read_text(encoding="utf-8")
        except Exception:
            pass
    return ""


def _parse_cli_command(cmd: str) -> list[ToolCallRecord]:
    """Parse raw tq CLI command calls into virtual ToolCallRecords for checks grading."""
    records = []
    # Match tq query "..."
    query_match = re.search(r"tq\s+query\s+(?:--format\s+\w+\s+)?[\"'](.*?)[\"']", cmd, re.DOTALL | re.IGNORECASE)
    if query_match:
        sql = query_match.group(1).strip()
        records.append(ToolCallRecord(name="base_readQuery", input_parameters={"sql": sql}))
        return records

    # Match tq list tables
    list_tables_match = re.search(r"tq\s+list\s+tables(?:\s+([a-zA-Z0-9_%]+))?(?:\s+--database\s+([a-zA-Z0-9_]+))?", cmd, re.IGNORECASE)
    if list_tables_match:
        pattern = list_tables_match.group(1) or ""
        db = list_tables_match.group(2) or ""
        records.append(ToolCallRecord(name="base_tableList", input_parameters={"database_name": db}))
        return records

    # Match tq inspect
    inspect_match = re.search(r"tq\s+inspect\s+([a-zA-Z0-9_\.]+)", cmd, re.IGNORECASE)
    if inspect_match:
        tbl_ref = inspect_match.group(1)
        db = ""
        tbl = tbl_ref
        if "." in tbl_ref:
            db, tbl = tbl_ref.split(".", 1)
        records.append(ToolCallRecord(name="base_columnMetadata", input_parameters={"database_name": db, "object_name": tbl}))
        return records

    # Match tq peek
    peek_match = re.search(r"tq\s+peek\s+([a-zA-Z0-9_\.]+)", cmd, re.IGNORECASE)
    if peek_match:
        tbl_ref = peek_match.group(1)
        db = ""
        tbl = tbl_ref
        if "." in tbl_ref:
            db, tbl = tbl_ref.split(".", 1)
        records.append(ToolCallRecord(name="base_tablePreview", input_parameters={"database_name": db, "table_name": tbl}))
        return records

    return records


class UnifiedAgentRunner:
    def __init__(self, provider: str, model_id: str, mode: str):
        self.provider = provider
        self.model_id = model_id
        self.mode = mode

        # Init LLM client
        if self.provider == "bedrock":
            import boto3
            region = os.environ.get("AWS_REGION", "us-east-1")
            self.llm_client = boto3.client("bedrock-runtime", region_name=region)
        else:
            from openai import OpenAI
            if self.provider == "openai":
                self.llm_client = OpenAI(api_key=os.environ.get("OPENAI_API_KEY"))
            elif self.provider == "openrouter":
                self.llm_client = OpenAI(
                    base_url="https://openrouter.ai/api/v1",
                    api_key=os.environ.get("OPENROUTER_API_KEY")
                )
            elif self.provider == "gemini":
                self.llm_client = OpenAI(
                    base_url=os.environ.get("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai/"),
                    api_key=os.environ.get("GEMINI_API_KEY")
                )

    async def get_tools(self, mcp_session: ClientSession | None = None) -> list:
        if self.mode == "no-tool":
            return []

        if self.mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill"):
            # Expose generic command execution tool
            cmd_tool = {
                "name": "execute_command",
                "description": "Execute a terminal/shell command on the local system to interact with Teradata (e.g. tq query, tq list, etc.) and return its stdout/stderr.",
                "inputSchema": {
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
            # Wrap as MockTool
            return [MockTool("execute_command", {"description": cmd_tool["description"], "parameters": cmd_tool["inputSchema"]["properties"]})]

        # For mcp: load either live MCP tools or fallback YAML tool definitions
        raw_tools = []
        if self.mode == "mcp" and mcp_session:
            resp = await mcp_session.list_tools()
            raw_tools = resp.tools
        else:
            schemas = load_mcp_tool_schemas()
            raw_tools = [MockTool(name, spec) for name, spec in schemas.items()]

        overrides = _load_description_overrides()
        return _apply_description_overrides(raw_tools, overrides)

    async def run_chat_loop(self, prompts: list[str], max_steps: int, mcp_session: ClientSession | None = None) -> list[TurnResult]:
        tools = await self.get_tools(mcp_session)
        
        # Prepare system prompt
        system_prompt = "You are an AI data assistant connecting to a Teradata database."
        if self.mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill"):
            if self.mode == "tq-cli-force-skill":
                system_prompt += "\n" + _get_skill_system_prompt()
            elif self.mode == "tq-cli":
                system_prompt += (
                    "\nAvailable Skills:\n"
                    "- Name: teradata-query\n"
                    "  Description: Install, configure, and use the tq CLI tool to run Teradata queries, explore schemas, monitor sessions, and manage database objects.\n"
                    "  Instructions File: agentic/skills/teradata-query/SKILL.md\n"
                    "  Note: If you need detailed instructions or examples for using the 'tq' CLI, you may read this instructions file using standard shell commands (e.g. cat, grep) via the 'execute_command' tool.\n"
                )
            
            system_prompt += "\nUse the execute_command tool to run tq commands to interact with the database."
            system_prompt += "\nIMPORTANT: The 'tq' CLI is already installed and fully configured. Connection is ready and verified. DO NOT ask the user for confirmation, configuration, profiles, or verification commands (like tq --version or tq ping). Directly run the correct 'tq' CLI commands using the 'execute_command' tool to fulfill the user's request."

        messages: list[dict] = []
        turn_results: list[TurnResult] = []

        # Tool definitions formatted for provider
        if self.provider == "bedrock":
            llm_tools = [_mcp_tool_to_bedrock(t) for t in tools]
        else:
            llm_tools = [_mcp_tool_to_openai(t) for t in tools]

        for prompt in prompts:
            # Append user message
            messages.append({"role": "user", "content": prompt if self.provider != "bedrock" else [{"text": prompt}]})
            turn_tool_calls: list[ToolCallRecord] = []
            final_response = ""
            input_tokens = 0
            output_tokens = 0

            for step in range(max_steps):
                # Call LLM
                if self.provider == "bedrock":
                    resp = self.llm_client.converse(
                        modelId=self.model_id,
                        messages=messages,
                        toolConfig={"tools": llm_tools, "toolChoice": {"auto": {}}} if llm_tools else None,
                        additionalModelRequestFields={"system": system_prompt} if system_prompt else None
                    )
                    # Extract usage
                    usage = resp.get("usage", {})
                    input_tokens += usage.get("inputTokens", 0)
                    output_tokens += usage.get("outputTokens", 0)

                    stop_reason = resp["stopReason"]
                    output_message = resp["output"]["message"]
                    messages.append(output_message)

                    if stop_reason == "tool_use":
                        tool_results = []
                        for block in output_message.get("content", []):
                            if "toolUse" in block:
                                tu = block["toolUse"]
                                name = tu["name"]
                                params = tu.get("input", {})
                                tool_id = tu["toolUseId"]

                                # Execute tool call
                                text_res, records = await self._execute_and_record(name, params, mcp_session)
                                turn_tool_calls.extend(records)
                                tool_results.append({
                                    "toolResult": {
                                        "toolUseId": tool_id,
                                        "content": [{"text": text_res}]
                                    }
                                })
                        
                        messages.append({"role": "user", "content": tool_results})
                    else:
                        # Extract final text
                        text_parts = [b["text"] for b in output_message.get("content", []) if "text" in b]
                        final_response = "".join(text_parts)
                        break
                else:
                    # openai, openrouter, gemini
                    # System prompt goes to system role in OpenAI API
                    api_messages = [{"role": "system", "content": system_prompt}] + messages
                    resp = self.llm_client.chat.completions.create(
                        model=self.model_id,
                        messages=api_messages,
                        tools=llm_tools if llm_tools else None,
                        tool_choice="auto" if llm_tools else None
                    )
                    # Extract usage
                    usage = getattr(resp, "usage", None)
                    if usage:
                        input_tokens += getattr(usage, "prompt_tokens", 0)
                        output_tokens += getattr(usage, "completion_tokens", 0)

                    choice = resp.choices[0]
                    message = choice.message
                    
                    # Convert OpenAI assistant message for history
                    assistant_msg = {"role": "assistant", "content": message.content or ""}
                    if message.tool_calls:
                        assistant_msg["tool_calls"] = [
                            {
                                "id": tc.id,
                                "type": "function",
                                "function": {"name": tc.function.name, "arguments": tc.function.arguments}
                            } for tc in message.tool_calls
                        ]
                    messages.append(assistant_msg)

                    if message.tool_calls:
                        for tc in message.tool_calls:
                            name = tc.function.name
                            try:
                                params = json.loads(tc.function.arguments)
                            except Exception:
                                params = {}
                            
                            text_res, records = await self._execute_and_record(name, params, mcp_session)
                            turn_tool_calls.extend(records)
                            
                            messages.append({
                                "role": "tool",
                                "tool_call_id": tc.id,
                                "name": name,
                                "content": text_res
                            })
                    else:
                        final_response = message.content or ""
                        break

            total_tokens = input_tokens + output_tokens
            cost = calculate_cost(self.model_id, input_tokens, output_tokens)
            token_usage = TokenUsage(
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                total_tokens=total_tokens,
                cost=cost
            )
            turn_results.append(TurnResult(
                tool_calls=turn_tool_calls,
                final_response=final_response,
                token_usage=token_usage
            ))

        return turn_results

    async def _execute_and_record(self, name: str, params: dict, mcp_session: ClientSession | None = None) -> tuple[str, list[ToolCallRecord]]:
        """Run tool (live MCP vs tq executor vs tq CLI command tool) and return (result_text, list of virtual records)."""
        if self.mode == "mcp" and mcp_session:
            try:
                res = await mcp_session.call_tool(name, params)
                res_text = json.dumps([c.model_dump() for c in res.content], default=str)
                return res_text, [ToolCallRecord(name=name, input_parameters=params)]
            except Exception as e:
                return f"Error executing live MCP tool: {e}", [ToolCallRecord(name=name, input_parameters=params)]



        if self.mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill") and name == "execute_command":
            cmd = params.get("command", "")
            code, stdout, stderr = run_tq(cmd.replace("tq ", "").split())
            txt = stdout if code == 0 else f"Error: {stderr}"
            # Translate raw shell command to virtual tool call records
            virtual_records = _parse_cli_command(cmd)
            # Include the raw execute_command tool call as the primary record for CLI checks
            raw_record = ToolCallRecord(name="execute_command", input_parameters={"command": cmd})
            return txt, [raw_record] + virtual_records

        return "No action taken", []


def _run_async(coro):
    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        return asyncio.run(coro)

    import concurrent.futures
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
        future = executor.submit(asyncio.run, coro)
        return future.result()


def run_agent(prompt: str, provider: str | None = None, model_id: str | None = None, mode: str | None = None, max_steps: int | None = None) -> AgentResult:
    provider = provider or os.environ.get("EVALS_PROVIDER", "bedrock")
    model_id = model_id or os.environ.get("EVALS_AGENT_MODEL") or os.environ.get("BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20241022-v2:0")
    mode = mode or os.environ.get("EVALS_MODE", "tq-cli")
    max_steps = max_steps or int(os.environ.get("AGENT_MAX_STEPS", "5"))
    mcp_url = os.environ.get("MCP_SERVER_URL", "http://127.0.0.1:8001/mcp")

    runner = UnifiedAgentRunner(provider, model_id, mode)

    async def _run():
        if mode == "mcp":
            async with streamablehttp_client(mcp_url) as (read, write, _):
                async with ClientSession(read, write) as session:
                    await session.initialize()
                    res = await runner.run_chat_loop([prompt], max_steps, mcp_session=session)
        else:
            res = await runner.run_chat_loop([prompt], max_steps)
            
        all_calls = []
        for tr in res:
            all_calls.extend(tr.tool_calls)
        in_t = sum(tr.token_usage.input_tokens for tr in res)
        out_t = sum(tr.token_usage.output_tokens for tr in res)
        tot_t = sum(tr.token_usage.total_tokens for tr in res)
        cost_val = sum(tr.token_usage.cost for tr in res)
        token_usage = TokenUsage(input_tokens=in_t, output_tokens=out_t, total_tokens=tot_t, cost=cost_val)

        return AgentResult(
            tool_calls=all_calls,
            final_response=res[0].final_response if res else "",
            token_usage=token_usage
        )

    return _run_async(_run())


def run_agent_turns(prompts: list[str], provider: str | None = None, model_id: str | None = None, mode: str | None = None, max_steps_per_turn: int | None = None) -> list[TurnResult]:
    provider = provider or os.environ.get("EVALS_PROVIDER", "bedrock")
    model_id = model_id or os.environ.get("EVALS_AGENT_MODEL") or os.environ.get("BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20241022-v2:0")
    mode = mode or os.environ.get("EVALS_MODE", "tq-cli")
    max_steps = max_steps_per_turn or int(os.environ.get("AGENT_MAX_STEPS_PER_TURN", "3"))
    mcp_url = os.environ.get("MCP_SERVER_URL", "http://127.0.0.1:8001/mcp")

    runner = UnifiedAgentRunner(provider, model_id, mode)

    async def _run():
        if mode == "mcp":
            async with streamablehttp_client(mcp_url) as (read, write, _):
                async with ClientSession(read, write) as session:
                    await session.initialize()
                    return await runner.run_chat_loop(prompts, max_steps, mcp_session=session)
        else:
            return await runner.run_chat_loop(prompts, max_steps)

    return _run_async(_run())
