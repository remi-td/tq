"""Unit and regression tests for the tq Agentic Optimization & Evaluation Framework."""

from __future__ import annotations

import unittest
from pathlib import Path

from agentic.bench.telemetry.token_telemetry import TokenUsage, parse_claude_token_usage, parse_gemini_token_usage
from agentic.bench.telemetry.cost_calculator import CostCalculator
from agentic.bench.telemetry.database_telemetry import DatabaseMetrics, TeradataTelemetry
from agentic.bench.analytics.comparison import compare_skill_impact, compare_tool_impact, compute_model_leaderboard
from agentic.bench.recommendations.advisor import OptimizationAdvisor, Recommendation
from agentic.bench.harness.base import RunResult


class TestTokenTelemetry(unittest.TestCase):
    def test_token_usage_total(self):
        u = TokenUsage(input_tokens=1000, output_tokens=200, cache_read_tokens=3000, cache_write_tokens=500)
        self.assertEqual(u.compute_total(), 4700)

    def test_claude_json_parsing(self):
        data = {
            "total_cost_usd": 0.015,
            "usage": {
                "input_tokens": 120,
                "output_tokens": 450,
                "cache_read_input_tokens": 8500,
                "cache_creation_input_tokens": 2000,
                "output_tokens_details": {
                    "thinking_tokens": 250
                }
            }
        }
        t = parse_claude_token_usage(data)
        self.assertEqual(t.input_tokens, 120)
        self.assertEqual(t.output_tokens, 450)
        self.assertEqual(t.cache_read_tokens, 8500)
        self.assertEqual(t.cache_write_tokens, 2000)
        self.assertEqual(t.reasoning_tokens, 250)
        self.assertEqual(t.raw_cost_usd, 0.015)
        self.assertEqual(t.total_tokens, 11070)

    def test_gemini_usage_parsing(self):
        gemini_dict = {
            "prompt_tokens": 5000,
            "completion_tokens": 600,
            "total_tokens": 5600,
            "prompt_tokens_details": {
                "cached_tokens": 4000
            }
        }
        t = parse_gemini_token_usage(gemini_dict)
        self.assertEqual(t.input_tokens, 1000)  # 5000 - 4000
        self.assertEqual(t.output_tokens, 600)
        self.assertEqual(t.cache_read_tokens, 4000)
        self.assertEqual(t.total_tokens, 5600)


class TestCostCalculator(unittest.TestCase):
    def setUp(self):
        self.calc = CostCalculator()

    def test_model_rates_resolution(self):
        flash_rates = self.calc.get_model_rates("gemini-2.5-flash")
        self.assertAlmostEqual(flash_rates["input_per_million"], 0.075)

        sonnet_rates = self.calc.get_model_rates("sonnet")
        self.assertAlmostEqual(sonnet_rates["output_per_million"], 15.00)

        unknown_rates = self.calc.get_model_rates("some-unknown-model")
        self.assertIn("input_per_million", unknown_rates)

    def test_effort_cost_calculation(self):
        usage = TokenUsage(input_tokens=1_000_000, output_tokens=100_000, cache_read_tokens=500_000)
        # For gemini-2.5-flash: 1M in = 0.075, 100k out = 0.03, 500k cache = 0.009375 -> ~0.114375
        costs = self.calc.calculate_total_effort_cost("gemini-2.5-flash", usage, amp_cpu_sec=2.0, amp_io=10_000)

        self.assertGreater(costs["token_cost_usd"], 0.05)
        self.assertEqual(costs["database_cost_usd"], 0.0)
        self.assertEqual(costs["total_effort_cost_usd"], costs["token_cost_usd"])


class TestComparativeAnalytics(unittest.TestCase):
    def test_skill_comparison(self):
        r_skill = RunResult(
            run_id="run_s",
            harness="claude-code",
            model="claude-haiku-4-5",
            mode="tq-with-skill",
            dataset="tpch",
            table_prefix="b1_",
            token_usage=TokenUsage(input_tokens=2000, output_tokens=500, total_tokens=2500),
            db_metrics=DatabaseMetrics(delta_cpu_sec=1.2),
            costs={"total_effort_cost_usd": 0.005},
            validation_score=100.0
        )
        r_no_skill = RunResult(
            run_id="run_ns",
            harness="claude-code",
            model="claude-haiku-4-5",
            mode="tq-no-skill",
            dataset="tpch",
            table_prefix="b2_",
            token_usage=TokenUsage(input_tokens=8000, output_tokens=2000, total_tokens=10000),
            db_metrics=DatabaseMetrics(delta_cpu_sec=3.6),
            costs={"total_effort_cost_usd": 0.020},
            validation_score=80.0
        )

        cmp = compare_skill_impact(r_skill, r_no_skill)
        self.assertEqual(cmp["token_reduction_pct"], 75.0)
        self.assertEqual(cmp["cost_reduction_pct"], 75.0)
        self.assertAlmostEqual(cmp["database_cpu_savings_pct"], 66.67, places=1)
        self.assertEqual(cmp["skill_advantage"], "positive")

    def test_tool_comparison(self):
        r_tq = RunResult(
            run_id="run_tq",
            harness="claude-code",
            model="sonnet",
            mode="tq-with-skill",
            dataset="tpch",
            table_prefix="b1_",
            duration_seconds=15.0,
            token_usage=TokenUsage(total_tokens=4000),
            costs={"total_effort_cost_usd": 0.025},
            validation_score=100.0
        )
        r_py = RunResult(
            run_id="run_py",
            harness="claude-code",
            model="sonnet",
            mode="baseline-python",
            dataset="tpch",
            table_prefix="b2_",
            duration_seconds=60.0,
            token_usage=TokenUsage(total_tokens=16000),
            costs={"total_effort_cost_usd": 0.100},
            validation_score=75.0
        )

        cmp = compare_tool_impact(r_tq, r_py)
        self.assertEqual(cmp["tq_token_savings_pct"], 75.0)
        self.assertEqual(cmp["tq_cost_savings_pct"], 75.0)
        self.assertEqual(cmp["speedup_ratio"], 4.0)
        self.assertEqual(cmp["tq_advantage"], "superior")


class TestOptimizationAdvisor(unittest.TestCase):
    def test_advisor_rule_firing(self):
        advisor = OptimizationAdvisor()

        # Inefficient run with high skew and raw DBC queries
        r = RunResult(
            run_id="bad_run",
            harness="gemini",
            model="gemini-2.5-flash",
            mode="tq-no-skill",
            dataset="tpch",
            table_prefix="b1_",
            token_usage=TokenUsage(total_tokens=28000),
            db_metrics=DatabaseMetrics(avg_cpu_skew_pct=42.0),
            commands_executed=[
                "tq query 'SELECT * FROM DBC.TablesV'",
                "tq query 'SELECT * FROM large_table'"
            ],
            validation_score=75.0,
            error_message="Syntax error near END"
        )

        recs = advisor.analyze_run(r)
        categories = {rec.category for rec in recs}

        self.assertIn("token_compression", categories)
        self.assertIn("skill_prompt", categories)
        self.assertIn("database_optimization", categories)
class TestDatabaseTelemetry(unittest.TestCase):
    def test_collect_run_metrics_time_span_sql(self):
        telem = TeradataTelemetry()
        executed_queries = []

        def mock_execute_query(sql):
            executed_queries.append(sql)
            if "QryLogV" in sql:
                return True, [{
                    "total_records": 12,
                    "query_cnt": 10,
                    "total_cpu": 1.45,
                    "total_io": 8500,
                    "max_spool": 1200000,
                    "err_cnt": 0
                }]
            return True, [{"ts_str": "2026-09-14 07:15:00"}]

        telem.execute_query = mock_execute_query

        metrics = telem.collect_run_metrics(
            start_ts="2026-09-14 07:10:00",
            end_ts="2026-09-14 07:14:30",
            table_prefix="b_test_"
        )

        self.assertEqual(metrics.query_count, 10)
        self.assertEqual(metrics.delta_cpu_sec, 1.45)
        self.assertEqual(metrics.delta_io, 8500)
        self.assertEqual(metrics.peak_spool_bytes, 1200000)

        # Check that flush was executed
        self.assertIn("FLUSH QUERY LOGGING WITH ALL;", executed_queries[0])

        # Check that the DBQL query uses time span without QueryBand and filters probes in WHERE
        dbql_sql = executed_queries[1]
        self.assertIn("UserName = USER", dbql_sql)
        self.assertIn("StartTime >= CAST('2026-09-14 07:10:00' AS TIMESTAMP(0))", dbql_sql)
        self.assertIn("StartTime <= CAST('2026-09-14 07:14:30' AS TIMESTAMP(0))", dbql_sql)
        self.assertIn("QueryText NOT LIKE '%FLUSH QUERY LOGGING%'", dbql_sql)
        self.assertNotIn("GetQueryBandValue", dbql_sql)
        self.assertNotIn("QueryText LIKE '%b_test_%'", dbql_sql)


if __name__ == "__main__":
    unittest.main()
