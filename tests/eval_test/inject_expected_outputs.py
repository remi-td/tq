import json
import os
from pathlib import Path

CASES_DIR = Path(__file__).resolve().parent / "cases"

def map_tool_to_tq_command(tool_name: str, params: dict) -> str:
    if tool_name == "base_readQuery":
        sql = params.get("sql", "")
        return f"tq query \"{sql}\""
    elif tool_name == "base_tableList":
        db = params.get("database_name", "")
        if db:
            return f"tq list tables --database {db}"
        return "tq list tables"
    elif tool_name == "base_tableDDL":
        db = params.get("database_name", "")
        tbl = params.get("table_name", "")
        ref = f"{db}.{tbl}" if db else tbl
        return f"tq query \"SHOW TABLE {ref}\""
    elif tool_name == "base_columnMetadata" or tool_name == "base_columnDescription":
        db = params.get("database_name", "")
        tbl = params.get("table_name", params.get("object_name", ""))
        ref = f"{db}.{tbl}" if db else tbl
        return f"tq inspect {ref}"
    elif tool_name == "base_tablePreview":
        db = params.get("database_name", "")
        tbl = params.get("table_name", "")
        ref = f"{db}.{tbl}" if db else tbl
        return f"tq peek {ref}"
    elif tool_name == "base_databaseList":
        return "tq list databases"
    elif tool_name == "base_saveDDL":
        db = params.get("database_name", "")
        tbl = params.get("table_name", "")
        ref = f"{db}.{tbl}" if db else tbl
        return f"tq query \"SHOW TABLE {ref}\""
    
    # DBA tools
    elif tool_name == "dba_tableSpace":
        db = params.get("database_name", "")
        tbl = params.get("table_name", "")
        ref = f"{db}.{tbl}" if db else tbl
        return f"tq inspect {ref}"
    elif tool_name == "dba_databaseSpace":
        db = params.get("database_name", "")
        return f"tq query \"SELECT DatabaseName, SUM(CurrentPerm) FROM DBC.DiskSpaceVX WHERE DatabaseName = '{db}' GROUP BY 1\""
    elif tool_name == "dba_databaseVersion":
        return "tq query \"SELECT * FROM DBC.DBCInfoV;\""
    elif tool_name == "dba_flowControl":
        return "tq sessions"
    
    # Sec tools
    elif tool_name == "sec_userRoles":
        user = params.get("user_name", params.get("username", ""))
        return f"tq query \"SELECT Grantee, Role FROM DBC.RoleMembersVX WHERE Grantee = '{user}'\""
    elif tool_name == "sec_rolePermissions":
        role = params.get("role_name", "")
        return f"tq query \"SELECT RoleName, DatabaseName, TableName, AccessRight FROM DBC.AllRoleRightsVX WHERE RoleName = '{role}'\""
    elif tool_name == "sec_userDbPermissions":
        user = params.get("user_name", params.get("username", ""))
        db = params.get("database_name", "")
        return f"tq query \"SELECT DatabaseName, TableName, AccessRight FROM DBC.AllRightsVX WHERE UserName = '{user}' AND DatabaseName = '{db}'\""

    return "tq query"

def process_file(path: Path):
    with open(path) as f:
        data = json.load(f)
    
    modified = False
    for case in data.get("cases", []):
        expected_tools = case.get("expected_tools")
        if expected_tools is not None:
            if "expected_output" not in case and expected_tools:
                primary = expected_tools[0]
                cmd = map_tool_to_tq_command(primary["name"], primary.get("params", {}))
                case["expected_output"] = cmd
            del case["expected_tools"]
            modified = True
            
        # Support turns inside multi-turn cases
        for turn in case.get("turns", []):
            expected_tools = turn.get("expected_tools")
            if expected_tools is not None:
                if "expected_output" not in turn and expected_tools:
                    primary = expected_tools[0]
                    cmd = map_tool_to_tq_command(primary["name"], primary.get("params", {}))
                    turn["expected_output"] = cmd
                del turn["expected_tools"]
                modified = True
                
    if modified:
        with open(path, "w") as f:
            json.dump(data, f, indent=2)
        print(f"Cleaned and updated {path.name}")

def main():
    for p in CASES_DIR.glob("*.json"):
        process_file(p)

if __name__ == "__main__":
    main()
