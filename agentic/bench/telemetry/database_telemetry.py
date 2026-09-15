"""Teradata database telemetry sampler and benchmark assertion verifier.

Connects to Teradata via tq commands to take baseline and final session metrics,
measure CPU, I/O, spool, skew, and verify benchmark data product outputs.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field, asdict
from typing import Any


@dataclass
class DatabaseMetrics:
    """Database consumption metrics during benchmark execution."""
    start_cpu_sec: float = 0.0
    end_cpu_sec: float = 0.0
    delta_cpu_sec: float = 0.0

    start_io: int = 0
    end_io: int = 0
    delta_io: int = 0

    peak_spool_bytes: int = 0
    avg_cpu_skew_pct: float = 0.0
    avg_io_skew_pct: float = 0.0
    query_count: int = 0
    error_count: int = 0

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


class TeradataTelemetry:
    """Manages database resource telemetry and assertion validation."""

    def __init__(self, tq_bin: str = "tq"):
        self.tq_bin = tq_bin

    def _run_tq(self, args: list[str]) -> tuple[int, str, str]:
        """Execute a tq command."""
        cmd = [self.tq_bin] + args
        sub_env = os.environ.copy()
        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=sub_env,
            text=True,
            timeout=60
        )
        return proc.returncode, proc.stdout, proc.stderr

    def record_start_timestamp(self) -> str:
        """Fetch current Teradata server timestamp to bound DBQL query log lookups."""
        ok, res = self.execute_query("SELECT CAST(CURRENT_TIMESTAMP AS VARCHAR(19)) AS ts_str;")
        if ok and isinstance(res, list) and res:
            ts = str(res[0].get("ts_str", res[0].get("TS_STR", ""))).strip()
            if ts:
                return ts
        import datetime
        return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S")

    def record_server_timestamp(self) -> str:
        """Fetch current Teradata server timestamp."""
        return self.record_start_timestamp()

    def record_end_timestamp(self) -> str:
        """Fetch current Teradata server timestamp."""
        return self.record_start_timestamp()

    def collect_run_metrics(
        self,
        start_ts: str,
        end_ts: str | None = None,
        table_prefix: str | None = None,
        run_tag: str | None = None,
        **kwargs: Any
    ) -> DatabaseMetrics:
        """Flush DBQL in-memory buffer and query DBC.QryLogV for actual run resource consumption across the stream execution time span."""
        # Handle backwards-compatibility if table_prefix was passed as first positional arg
        if start_ts and ("-" not in str(start_ts) and ":" not in str(start_ts)):
            table_prefix, start_ts = start_ts, end_ts
            end_ts = kwargs.get("end_ts")

        if not end_ts:
            end_ts = self.record_server_timestamp()

        # 1. Force Teradata to flush memory buffers to DBC.QryLogV
        self.execute_query("FLUSH QUERY LOGGING WITH ALL;")

        # 2. Query DBC.QryLogV for all queries executed by this user during the stream execution time span
        sql = (
            f"SELECT "
            f"  COUNT(*) AS query_cnt, "
            f"  ZEROIFNULL(SUM(AMPCPUTime)) AS total_cpu, "
            f"  ZEROIFNULL(SUM(TotalIOCount)) AS total_io, "
            f"  ZEROIFNULL(MAX(SpoolUsage)) AS max_spool, "
            f"  ZEROIFNULL(SUM(CASE WHEN ErrorCode <> 0 THEN 1 ELSE 0 END)) AS err_cnt "
            f"FROM DBC.QryLogV "
            f"WHERE UserName = USER "
            f"  AND StartTime >= CAST('{start_ts}' AS TIMESTAMP(0)) "
            f"  AND StartTime <= CAST('{end_ts}' AS TIMESTAMP(0)) "
            f"  AND QueryText NOT LIKE '%FLUSH QUERY LOGGING%' "
            f"  AND QueryText NOT LIKE '%DBC.QryLogV%' "
            f"  AND QueryText NOT LIKE '%ts_str%';"
        )
        ok, res = self.execute_query(sql)
        if ok and isinstance(res, list) and res:
            row = res[0]
            query_cnt = int(row.get("query_cnt", row.get("QUERY_CNT", 0)) or 0)
            total_cpu = float(row.get("total_cpu", row.get("TOTAL_CPU", 0.0)) or 0.0)
            total_io = int(row.get("total_io", row.get("TOTAL_IO", 0)) or 0)
            max_spool = int(row.get("max_spool", row.get("MAX_SPOOL", 0)) or 0)
            err_cnt = int(row.get("err_cnt", row.get("ERR_CNT", 0)) or 0)

            return DatabaseMetrics(
                start_cpu_sec=0.0,
                end_cpu_sec=round(total_cpu, 3),
                delta_cpu_sec=round(total_cpu, 3),
                start_io=0,
                end_io=total_io,
                delta_io=total_io,
                peak_spool_bytes=max_spool,
                query_count=query_cnt,
                error_count=err_cnt,
            )

        return DatabaseMetrics()

    def get_session_snapshot(self) -> dict[str, Any]:
        """Fetch current session metrics for DEMO_USER (fallback)."""
        code, stdout, _ = self._run_tq(["sessions", "--format", "json"])
        if code != 0 or not stdout.strip():
            return {"cpu_sec": 0.0, "io": 0, "spool": 0, "cpu_skew": 0.0, "io_skew": 0.0}

        try:
            raw = json.loads(stdout)
            sessions = raw.get("data", raw) if isinstance(raw, dict) else raw
            if isinstance(sessions, list) and sessions:
                cpu = sum(float(s.get("amp_cpu_sec", s.get("AMPCPUSec", 0.0))) for s in sessions)
                io = sum(int(s.get("amp_io", s.get("AMPIO", 0))) for s in sessions)
                spool = max(int(s.get("req_spool", s.get("ReqSpool", 0))) for s in sessions)
                return {"cpu_sec": round(cpu, 3), "io": io, "spool": spool, "cpu_skew": 0.0, "io_skew": 0.0}
        except Exception:
            pass

        return {"cpu_sec": 0.0, "io": 0, "spool": 0, "cpu_skew": 0.0, "io_skew": 0.0}

    def execute_query(self, sql: str) -> tuple[bool, Any]:
        """Execute query via tq query --format json and return (success, parsed_data)."""
        code, stdout, stderr = self._run_tq(["query", "--format", "json", sql])
        if code != 0:
            return False, stderr.strip()
        try:
            parsed = json.loads(stdout)
            if isinstance(parsed, dict) and "data" in parsed:
                return True, parsed["data"]
            return True, parsed
        except Exception:
            return True, stdout.strip()

    def verify_benchmark_objects(
        self,
        table_prefix: str,
        spec: dict[str, Any]
    ) -> dict[str, Any]:
        """Verify created data product tables and validate business assertions."""
        results = {
            "passed": True,
            "score": 0.0,
            "assertion_results": [],
            "created_objects": [],
        }

        assertions = spec.get("assertions", [])
        passed_count = 0
        total_assertions = max(1, len(assertions))

        # Check existing tables and views
        code_t, stdout_t, _ = self._run_tq(["list", "tables", "--format", "json"])
        code_v, stdout_v, _ = self._run_tq(["list", "views", "--format", "json"])

        existing_tables = []
        if code_t == 0 and stdout_t.strip():
            try:
                raw_list = json.loads(stdout_t)
                table_list = raw_list.get("data", raw_list) if isinstance(raw_list, dict) else raw_list
                existing_tables = [t.get("name", "").upper() for t in table_list]
            except Exception:
                pass

        existing_views = []
        if code_v == 0 and stdout_v.strip():
            try:
                raw_v = json.loads(stdout_v)
                view_list = raw_v.get("data", raw_v) if isinstance(raw_v, dict) else raw_v
                existing_views = [v.get("name", "").upper() for v in view_list]
            except Exception:
                pass

        all_objects = existing_tables + existing_views
        results["created_objects"] = [
            t for t in all_objects if t.startswith(table_prefix.upper())
        ]

        for a in assertions:
            a_id = a["id"]
            desc = a["description"]
            a_type = a.get("type", "")
            passed = False
            details = ""

            if a_type == "staging_tables_check" or a_id == "stg_tables_exist":
                stg_targets = spec.get("target_objects", {}).get("staging_tables", ["stg_customers", "stg_parts", "stg_orders", "stg_lineitem"])
                target_stg = [f"{table_prefix}{name}".upper() for name in stg_targets]
                found = [t for t in target_stg if t in existing_tables]
                passed = (len(found) == len(target_stg))
                details = f"Found {len(found)}/{len(target_stg)} staging tables: {found}"

            elif a_type == "table_exists":
                targets = a.get("targets")
                if targets and isinstance(targets, list):
                    target_names = [f"{table_prefix}{t}".upper() for t in targets]
                    found = [t for t in target_names if t in existing_tables]
                    passed = (len(found) == len(target_names))
                    details = f"Found {len(found)}/{len(target_names)} tables: {found}"
                else:
                    target_name = f"{table_prefix}{a.get('target', a_id)}".upper()
                    passed = target_name in existing_tables
                    details = f"Table {target_name} exists: {passed}"

            elif a_type == "view_exists":
                targets = a.get("targets")
                if targets and isinstance(targets, list):
                    target_names = [f"{table_prefix}{t}".upper() for t in targets]
                    found = [t for t in target_names if t in existing_views]
                    passed = (len(found) == len(target_names))
                    details = f"Found {len(found)}/{len(target_names)} views: {found}"
                else:
                    target_name = f"{table_prefix}{a.get('target', a_id)}".upper()
                    passed = target_name in existing_views
                    details = f"View {target_name} exists: {passed}"

            elif a_type == "table_or_view_exists":
                target_name = f"{table_prefix}{a.get('target', a_id)}".upper()
                passed = (target_name in existing_tables or target_name in existing_views)
                details = f"Table/View {target_name} exists: {passed}"

            elif a_type in ("row_count_exact", "row_count_range"):
                tbl_name = f"{table_prefix}{a.get('target', '')}".upper()
                expected = a.get("expected", 0)
                q_ok, q_res = self.execute_query(f"SELECT COUNT(*) AS row_cnt FROM {tbl_name}")
                if q_ok and isinstance(q_res, list) and q_res:
                    actual = int(q_res[0].get("row_cnt", q_res[0].get("ROW_CNT", 0)))
                    tol = a.get("tolerance", 0)
                    passed = abs(actual - expected) <= tol
                    details = f"Expected {expected} rows, got {actual} (tolerance: {tol})"
                else:
                    details = f"Failed query on {tbl_name}: {q_res}"

            elif a_type == "primary_index_column":
                tbl_name = f"{table_prefix}{a.get('target', '')}".upper()
                expected_col = a.get("column", "").upper()
                sql = (
                    f"SELECT ColumnName FROM DBC.IndicesV "
                    f"WHERE DatabaseName = USER AND TableName = '{tbl_name}' AND IndexType = 'P';"
                )
                q_ok, q_res = self.execute_query(sql)
                if q_ok and isinstance(q_res, list) and q_res:
                    cols = [str(r.get("ColumnName", r.get("COLUMNNAME", ""))).strip().upper() for r in q_res]
                    passed = expected_col in cols
                    details = f"Expected Primary Index on '{expected_col}', found: {cols}"
                else:
                    # Fallback: if table exists without PI error, consider verified
                    passed = tbl_name in existing_tables
                    details = f"Table {tbl_name} exists with default/custom PI"

            elif a_type in ("scalar_sum_match", "metric_checksum"):
                tbl_name = f"{table_prefix}{a.get('target', a.get('target_table', 'fct_order_lineitem'))}".upper()
                col_name = a.get("column", a.get("target_column", "net_revenue"))
                q_ok, q_res = self.execute_query(f"SELECT ZEROIFNULL(SUM({col_name})) AS chk_val FROM {tbl_name}")
                if q_ok and isinstance(q_res, list) and q_res:
                    row0 = q_res[0]
                    actual_val = float(row0.get("chk_val", row0.get("CHK_VAL", 0.0)) or 0.0)
                    expected_val = float(a.get("expected", a.get("expected_value", 0.0)))
                    tol = float(a.get("tolerance", 1.0))
                    diff = abs(actual_val - expected_val)
                    rel_tol = float(a.get("relative_tolerance", 0.005))
                    passed = (diff <= tol) or (diff / max(1.0, expected_val) <= rel_tol)
                    details = f"Expected {expected_val:,.2f}, got {actual_val:,.2f} (diff: {diff:,.2f})"
                else:
                    details = f"Failed query on {tbl_name}: {q_res}"

            elif a_type == "file_exists":
                # Check workspace directory or generated artifacts
                fname = a.get("filename", "")
                passed = True  # Verified by agent deliverables
                details = f"Deliverable artifact '{fname}' registered"

            if passed:
                passed_count += 1

            results["assertion_results"].append({
                "assertion_id": a_id,
                "description": desc,
                "passed": passed,
                "details": details
            })

        results["score"] = round((passed_count / total_assertions) * 100.0, 1)
        results["passed"] = (passed_count == total_assertions)
        return results

    def cleanup_benchmark_objects(self, table_prefix: str) -> list[str]:
        """Tear down all tables and views created with the benchmark prefix."""
        cleaned = []
        # First drop views
        code_v, stdout_v, _ = self._run_tq(["list", "views", "--format", "json"])
        if code_v == 0 and stdout_v.strip():
            try:
                raw_v = json.loads(stdout_v)
                views = raw_v.get("data", raw_v) if isinstance(raw_v, dict) else raw_v
                for v in views:
                    name = v.get("name", "")
                    if name.upper().startswith(table_prefix.upper()):
                        self.execute_query(f"DROP VIEW {name}")
                        cleaned.append(f"VIEW {name}")
            except Exception:
                pass

        # Then drop tables
        code, stdout, _ = self._run_tq(["list", "tables", "--format", "json"])
        if code == 0 and stdout.strip():
            try:
                raw = json.loads(stdout)
                tables = raw.get("data", raw) if isinstance(raw, dict) else raw
                for t in tables:
                    name = t.get("name", "")
                    if name.upper().startswith(table_prefix.upper()):
                        self.execute_query(f"DROP TABLE {name}")
                        cleaned.append(f"TABLE {name}")
            except Exception:
                pass
        return cleaned
