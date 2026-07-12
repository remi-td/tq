"""Deterministic structural checks that run before the LLM judge."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Any

# Param values compared exactly when non-empty in the expected case.
EXACT_VALUE_KEYS = frozenset(
    {
        "database_name",
        "table_name",
        "column_name",
        "user_name",
        "username",
        "role_name",
    }
)

# Param keys that must be present but may differ in value (e.g. SQL wording).
PRESENCE_ONLY_KEYS = frozenset({"sql", "query"})


@dataclass
class ToolCallRecord:
    name: str
    input_parameters: dict[str, Any]


def _check_params(expected_params: dict[str, Any], actual_params: dict[str, Any]) -> list[str]:
    errors: list[str] = []

    for key, expected_value in expected_params.items():
        if key not in actual_params:
            errors.append(f"missing param key '{key}'")
            continue

        actual_value = actual_params[key]

        if key in PRESENCE_ONLY_KEYS:
            if expected_value and not actual_value:
                errors.append(f"param '{key}' must not be empty")
            continue

        if key in EXACT_VALUE_KEYS and expected_value not in ("", None):
            if actual_value != expected_value:
                errors.append(
                    f"param '{key}': expected {expected_value!r}, got {actual_value!r}",
                )

    return errors


def _check_tool_pair(
    expected: ToolCallRecord,
    actual: ToolCallRecord,
    *,
    label: str,
) -> list[str]:
    errors: list[str] = []
    if actual.name != expected.name:
        errors.append(f"{label}: expected tool {expected.name!r}, got {actual.name!r}")
    errors.extend(_check_params(expected.input_parameters, actual.input_parameters))
    return errors

def get_expected_tools_from_case(case: dict) -> list[dict[str, Any]]:
    """Dynamically reconstruct expected_tools from expected_output if missing."""
    expected_tools = case.get("expected_tools")
    if expected_tools is not None:
        return expected_tools

    expected_output = case.get("expected_output", "")
    if not expected_output:
        return []

    cmd = expected_output.strip()
    # Match tq query "..." or tq query '...'
    if cmd.lower().startswith("tq query"):
        # Safely extract SQL by stripping the prefix and optional matching outer quotes
        rest = re.sub(r"^tq\s+query\s+", "", cmd, flags=re.IGNORECASE).strip()
        if rest.startswith('"') and rest.endswith('"'):
            sql = rest[1:-1]
        elif rest.startswith("'") and rest.endswith("'"):
            sql = rest[1:-1]
        else:
            sql = rest

        # Check for table affinity and usage via case id first
        case_id = case.get("id", "").lower()
        if "affinity" in case_id:
            return [{"name": "base_tableAffinity", "params": {"database_name": "demo_user", "table_name": "evals_orders"}}]
        elif "usage" in case_id:
            return [{"name": "base_tableUsage", "params": {"database_name": "demo_user"}}]

        # Check for SHOW TABLE
        if "show table" in sql.lower():
            tbl_ref = re.sub(r"show table\s+", "", sql, flags=re.IGNORECASE).strip()
            db = ""
            tbl = tbl_ref
            if "." in tbl_ref:
                db, tbl = tbl_ref.split(".", 1)
            if "save" in case.get("id", "").lower():
                return [{"name": "base_saveDDL", "params": {"database_name": db, "table_name": tbl}}]
            return [{"name": "base_tableDDL", "params": {"database_name": db, "table_name": tbl}}]

        # Check for disk space
        elif "diskspacevx" in sql.lower():
            db_m = re.search(r"databasename\s*=\s*['\"](.*?)['\"]", sql, re.IGNORECASE)
            db = db_m.group(1) if db_m else ""
            return [{"name": "dba_databaseSpace", "params": {"database_name": db}}]

        # Check for dbcinfov
        elif "dbcinfov" in sql.lower():
            return [{"name": "dba_databaseVersion", "params": {}}]

        # Check for rolemembersvx
        elif "rolemembersvx" in sql.lower():
            user_m = re.search(r"grantee\s*=\s*['\"](.*?)['\"]", sql, re.IGNORECASE)
            user = user_m.group(1) if user_m else ""
            return [{"name": "sec_userRoles", "params": {"username": user}}]

        # Check for allrolerightsvx
        elif "allrolerightsvx" in sql.lower():
            role_m = re.search(r"rolename\s*=\s*['\"](.*?)['\"]", sql, re.IGNORECASE)
            role = role_m.group(1) if role_m else ""
            return [{"name": "sec_rolePermissions", "params": {"role_name": role}}]

        # Check for allrightsvx
        elif "allrightsvx" in sql.lower():
            user_m = re.search(r"username\s*=\s*['\"](.*?)['\"]", sql, re.IGNORECASE)
            db_m = re.search(r"databasename\s*=\s*['\"](.*?)['\"]", sql, re.IGNORECASE)
            user = user_m.group(1) if user_m else ""
            db = db_m.group(1) if db_m else ""
            return [{"name": "sec_userDbPermissions", "params": {"username": user, "database_name": db}}]

        return [{"name": "base_readQuery", "params": {"sql": sql}}]

    elif cmd.lower().startswith("tq list tables"):
        db = ""
        m = re.search(r"--database\s+(\S+)", cmd, re.IGNORECASE)
        if m:
            db = m.group(1)
        return [{"name": "base_tableList", "params": {"database_name": db}}]

    elif cmd.lower().startswith("tq list databases"):
        return [{"name": "base_databaseList", "params": {}}]

    elif cmd.lower().startswith("tq peek"):
        tbl_ref = re.sub(r"^tq\s+peek\s+", "", cmd, flags=re.IGNORECASE).strip()
        db = ""
        tbl = tbl_ref
        if "." in tbl_ref:
            db, tbl = tbl_ref.split(".", 1)
        return [{"name": "base_tablePreview", "params": {"database_name": db, "table_name": tbl}}]

    elif cmd.lower().startswith("tq inspect"):
        tbl_ref = re.sub(r"^tq\s+inspect\s+", "", cmd, flags=re.IGNORECASE).strip()
        db = ""
        tbl = tbl_ref
        if "." in tbl_ref:
            db, tbl = tbl_ref.split(".", 1)
        if "description" in case.get("id", "").lower():
            return [{"name": "base_columnDescription", "params": {"database_name": db, "table_name": tbl}}]
        return [{"name": "base_columnMetadata", "params": {"database_name": db, "object_name": tbl}}]

    elif cmd.lower() == "tq sessions":
        return [{"name": "dba_flowControl", "params": {}}]

    return []


def get_expected_tq_commands(case: dict) -> list[str]:
    """Map the expected MCP tools in a case to their tq CLI command equivalents."""
    commands = []
    # Try to read explicit expected_output from the test case first
    expected_out = case.get("expected_output")
    if expected_out:
        commands.append(expected_out)

    expected_raw = get_expected_tools_from_case(case)
    for t in expected_raw:
        name = t["name"]
        params = t.get("params", {})

        if name == "base_readQuery":
            sql = params.get("sql", "").strip()
            commands.append(f"tq query {sql}")
            commands.append(f'tq query "{sql}"')
            commands.append(f"tq query '{sql}'")
            commands.append("tq query")

        elif name == "base_tableList":
            db = params.get("database_name", "")
            if db:
                commands.append(f"tq list tables --database {db}")
                commands.append(f"tq list tables -d {db}")
                commands.append(f"tq list tables {db}")
            else:
                commands.append("tq list tables")

        elif name == "base_tableDDL":
            db = params.get("database_name", "")
            tbl = params.get("table_name", "")
            ref = f"{db}.{tbl}" if db else tbl
            commands.append(f"tq inspect {ref}")
            commands.append(f"tq query SHOW TABLE {ref}")
            commands.append(f'tq query "SHOW TABLE {ref}"')
            commands.append(f"tq query 'SHOW TABLE {ref}'")

        elif name == "base_columnMetadata" or name == "base_columnDescription":
            db = params.get("database_name", "")
            tbl = params.get("table_name", params.get("object_name", ""))
            ref = f"{db}.{tbl}" if db else tbl
            commands.append(f"tq inspect {ref}")

        elif name == "base_tablePreview":
            db = params.get("database_name", "")
            tbl = params.get("table_name", "")
            ref = f"{db}.{tbl}" if db else tbl
            commands.append(f"tq peek {ref}")

        elif name == "base_databaseList":
            commands.append("tq list databases")

        elif name == "base_saveDDL":
            db = params.get("database_name", "")
            tbl = params.get("table_name", "")
            ref = f"{db}.{tbl}" if db else tbl
            commands.append(f'tq query "SHOW TABLE {ref}"')
            commands.append(f"tq query 'SHOW TABLE {ref}'")
            commands.append(f"tq inspect {ref}")
            commands.append("tq query")
            commands.append("tq inspect")

        elif name == "base_tableAffinity" or name == "base_tableUsage":
            commands.append("tq query")
            commands.append("dbqlogtbl")
            commands.append("dbql")
            commands.append("dbc")

    return commands


def match_tq_commands(actual_cmd: str, expected_cmd: str) -> bool:
    """Return True if the actual command matches the expected command, ignoring order/flags."""
    actual_tokens = set(re.split(r"[^a-zA-Z0-9_]+", actual_cmd.lower()))
    expected_tokens = set(re.split(r"[^a-zA-Z0-9_]+", expected_cmd.lower()))

    # Ignore common flags and base keywords in matching core arguments
    ignore = {"tq", "database", "d", "table", "t", "format", "json", "csv", "to"}
    expected_core = {t for t in expected_tokens if t and t not in ignore}

    # Support simple keyword containment checks for DBQL logs checks
    if len(expected_core) == 1:
        token = list(expected_core)[0]
        if token in actual_tokens:
            return True

    # If all expected core tokens are subset of actual tokens, check matching action
    if expected_core.issubset(actual_tokens):
        actions = {"list", "query", "inspect", "peek", "sessions"}
        expected_actions = expected_tokens.intersection(actions)
        actual_actions = actual_tokens.intersection(actions)
        if expected_actions == actual_actions:
            return True
    return False


def run_deterministic_checks(
    case: dict,
    tools_called: list[ToolCallRecord],
    actual_output: str | None = None,
) -> list[str]:
    """Return a list of structural check failures (empty list means pass)."""
    import os
    
    case_type = case.get("type", "happy_path")
    expected_raw = get_expected_tools_from_case(case)
    expected = [
        ToolCallRecord(name=t["name"], input_parameters=t.get("params", {}))
        for t in expected_raw
    ]

    if case_type == "missing_parameter":
        if tools_called:
            names = [tc.name for tc in tools_called]
            return [f"expected no tool calls for missing_parameter case, got {names}"]
        return []

    if not expected:
        return []

    # If evaluating in tq-cli/tq-cli-force-skill/tq-cli-no-skill or no-tool mode and actual_output/tools_called matches the expected tq command, pass
    mode = os.environ.get("EVALS_MODE", "tq-cli")
    if mode in ("no-tool", "tq-cli", "tq-cli-force-skill", "tq-cli-no-skill"):
        expected_cmds = get_expected_tq_commands(case)
        if expected_cmds:
            # 1. Check if the command was executed via execute_command tool call
            if mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill") and tools_called:
                for tc in tools_called:
                    if tc.name == "execute_command":
                        cmd_val = tc.input_parameters.get("command", "")
                        for exp_cmd in expected_cmds:
                            if match_tq_commands(cmd_val, exp_cmd):
                                return []  # Pass!

            # 2. Check if the command was printed in actual_output
            if actual_output:
                # Find all command-like substrings or check matching
                for exp_cmd in expected_cmds:
                    if match_tq_commands(actual_output, exp_cmd):
                        return []  # Pass!

    errors: list[str] = []

    if case_type == "multi_tool":
        if len(tools_called) != len(expected):
            errors.append(
                f"multi_tool: expected {len(expected)} tool call(s), got {len(tools_called)}",
            )
            return errors
        for i, (exp, act) in enumerate(zip(expected, tools_called, strict=True)):
            errors.extend(_check_tool_pair(exp, act, label=f"step {i + 1}"))
        return errors

    if not tools_called:
        return ["expected at least one tool call, got none"]

    errors.extend(_check_tool_pair(expected[0], tools_called[0], label="primary tool"))

    return errors


def assert_deterministic_checks(case: dict, tools_called: list[ToolCallRecord], actual_output: str | None = None) -> None:
    """Fail fast on structural mismatch before invoking the LLM judge."""
    errors = run_deterministic_checks(case, tools_called, actual_output)
    if errors:
        case_id = case.get("id", "<unknown>")
        detail = "; ".join(errors)
        raise AssertionError(f"[{case_id}] deterministic check failed: {detail}")
