"""tq Executor: executes MCP-equivalent tools by running the local tq CLI binary."""

from __future__ import annotations

import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any

# Resolve path to the sibling repository to parse tool definitions
WORKSPACE_DIR = Path(__file__).resolve().parent.parent.parent.parent.parent
MCP_SERVER_DIR = WORKSPACE_DIR / "teradata-mcp-server"


def get_tq_binary() -> str:
    """Find the tq CLI binary, looking at target builds first, then PATH."""
    # Prefer release build for optimal performance
    release_bin = WORKSPACE_DIR / "tq" / "target" / "release" / "tq"
    if release_bin.exists():
        return str(release_bin)

    debug_bin = WORKSPACE_DIR / "tq" / "target" / "debug" / "tq"
    if debug_bin.exists():
        return str(debug_bin)

    # Check relative to cwd
    if Path("target/release/tq").exists():
        return "target/release/tq"
    if Path("target/debug/tq").exists():
        return "target/debug/tq"

    return "tq"


def run_tq(args: list[str]) -> tuple[int, str, str]:
    """Execute the tq CLI in a subprocess with the appropriate logon environment."""
    binary = get_tq_binary()
    cmd = [binary] + args

    env = os.environ.copy()
    if "TQ_LOGON" not in env and "TERADATA_HOST" in env:
        host = env.get("TERADATA_HOST")
        user = env.get("TERADATA_USER")
        password = env.get("TERADATA_PASSWORD")
        db = env.get("EVALS_DATABASE", "")
        env["TQ_LOGON"] = f"{user}:{password}@{host}:1025/{db}"

    # Always ensure output format is JSON or CSV as needed
    result = subprocess.run(cmd, env=env, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr


def load_mcp_tool_schemas() -> dict[str, dict[str, Any]]:
    """Parse all yml tool definitions from teradata-mcp-server repository."""
    import yaml

    tools = {}
    yml_paths = [
        MCP_SERVER_DIR / "src" / "teradata_mcp_server" / "tools" / "base" / "base_objects.yml",
        MCP_SERVER_DIR / "src" / "teradata_mcp_server" / "tools" / "dba" / "dba_objects.yml",
        MCP_SERVER_DIR / "src" / "teradata_mcp_server" / "tools" / "sec" / "sec_objects.yml",
        MCP_SERVER_DIR / "src" / "teradata_mcp_server" / "tools" / "qlty" / "qlty_objects.yml",
    ]

    for path in yml_paths:
        if not path.exists():
            continue
        try:
            with open(path) as f:
                data = yaml.safe_load(f)
                if isinstance(data, dict):
                    for name, spec in data.items():
                        if spec.get("type") == "tool":
                            tools[name] = spec
        except Exception:
            pass

    # Inject core python-defined tools
    tools["base_readQuery"] = {
        "type": "tool",
        "description": "Execute a user-provided SQL query against Teradata and return the results. Use this tool ONLY when the user supplies an explicit SQL statement or a request that includes filter conditions (WHERE clause, aggregations, JOINs, etc.). Do NOT use for simply browsing or sampling rows from a table — use base_tablePreview for that. The sql parameter is required and must contain the full SQL text.",
        "parameters": {
            "sql": {
                "description": "SQL text, with optional bind-parameter placeholders",
                "type_hint": "str",
                "required": True
            },
            "persist": {
                "description": "Set to True to persist the results as a table and reuse it later. Recommended for large result sets.",
                "type_hint": "bool",
                "default": False
            },
            "row_limit": {
                "description": "Maximum rows to return (default 1000, ceiling 50000). Pass a higher value when you need more rows.",
                "type_hint": "int",
                "default": 1000
            }
        }
    }

    tools["base_columnMetadata"] = {
        "type": "tool",
        "description": "Retrieve detailed technical column metadata for Teradata tables and views, including exact Teradata type codes, character sets (LATIN/UNICODE), decimal precision, scale, nullability, and index classification. Use when the user needs precise Teradata-specific column information, not just basic column names and types. For a simple list of columns and types for a single object, use base_columnDescription instead. Supports bulk retrieval across many objects with payload and time budgets.",
        "parameters": {
            "database_name": {
                "description": "Database name (e.g. 'MKTG_USR')",
                "type_hint": "str",
                "required": True
            },
            "object_name": {
                "description": "Object name (e.g. 'evals_orders')",
                "type_hint": "str",
                "required": False
            },
            "table_kind": {
                "description": "Filter by table kind (e.g., 'T', 'V')",
                "type_hint": "str",
                "required": False
            },
            "fields": {
                "description": "CSV of column metadata fields to include",
                "type_hint": "str",
                "required": False
            }
        }
    }

    tools["base_saveDDL"] = {
        "type": "tool",
        "description": "Extract the DDL for a Teradata table, view, or stored procedure and SAVE it as a .sql file on disk. Use this tool ONLY when the user explicitly wants to export, write, download, or persist DDL to a file. Do NOT use simply to display or view DDL in the conversation — use base_tableDDL to display DDL without saving.",
        "parameters": {
            "database_name": {
                "description": "Database name (e.g., 'MKTG_USR')",
                "type_hint": "str",
                "required": True
            },
            "table_name": {
                "description": "Object name (e.g., 'SP_LOAD_VARIABLES_ARGUMENTARIO_IAG_FICHA_CLIENTE'). Accepts comma-separated values for bulk retrieval.",
                "type_hint": "str",
                "required": True
            },
            "object_type": {
                "description": "Type of object: 'PROCEDURE', 'TABLE', 'VIEW' (default: 'PROCEDURE')",
                "type_hint": "str",
                "default": "PROCEDURE",
                "required": False
            },
            "output_dir": {
                "description": "Directory where to save the DDL file (default: './ddls_extracted')",
                "type_hint": "str",
                "default": "./ddls_extracted",
                "required": False
            }
        }
    }

    return tools
