#!/usr/bin/env python3
"""B2B SaaS Cloud Metered Billing & Usage Benchmark Data Generator.

Generates realistic, deterministic seed datasets for live Teradata benchmarking,
computes gold standard validation metrics, and writes validation_spec.json.
"""

from __future__ import annotations

import argparse
import csv
import json
import random
from collections import defaultdict
from datetime import datetime, timedelta
from pathlib import Path

DATASET_DIR = Path(__file__).resolve().parent
SEED_DIR = DATASET_DIR / "seed_data"


def generate_b2b_billing_data(
    num_accounts: int = 600,
    events_per_account: int = 50,
    seed: int = 42
) -> dict:
    """Generate deterministic B2B SaaS billing seed data and gold validation metrics."""
    SEED_DIR.mkdir(parents=True, exist_ok=True)
    rng = random.Random(seed)

    industries = ["FINTECH", "HEALTHCARE", "ECOMMERCE", "LOGISTICS", "CYBERSECURITY", "EDTECH"]
    tiers = ["FREE", "STARTUP", "SCALE", "ENTERPRISE"]
    billing_cycles = ["ANNUAL", "MONTHLY"]
    regions = ["US-EAST", "US-WEST", "EU-CENTRAL", "AP-SOUTH"]

    company_prefixes = ["Acme", "Nova", "Apex", "Nexus", "Vertex", "Pulse", "Stratum", "Hyper", "Synapse", "Omni"]
    company_suffixes = ["Technologies", "Labs", "Analytics", "Cloud", "Solutions", "Networks", "Systems", "AI"]

    # 1. Subscription Plans (4 tiers)
    plans = [
        {"plan_id": 1, "pricing_tier": "FREE", "monthly_base_fee": 0.00, "included_units": 100},
        {"plan_id": 2, "pricing_tier": "STARTUP", "monthly_base_fee": 99.00, "included_units": 1000},
        {"plan_id": 3, "pricing_tier": "SCALE", "monthly_base_fee": 499.00, "included_units": 5000},
        {"plan_id": 4, "pricing_tier": "ENTERPRISE", "monthly_base_fee": 2499.00, "included_units": 25000},
    ]
    plan_map = {p["pricing_tier"]: p for p in plans}

    # 2. Service Features Catalog
    features = [
        {"feature_id": 101, "feature_code": "COMPUTE_CORE_HR", "service_category": "COMPUTE", "unit_infra_cost": 0.04, "overage_unit_rate": 0.09},
        {"feature_id": 102, "feature_code": "GPU_INFERENCE_HR", "service_category": "AI_ML", "unit_infra_cost": 0.45, "overage_unit_rate": 1.15},
        {"feature_id": 103, "feature_code": "STORAGE_TB_DAY", "service_category": "STORAGE", "unit_infra_cost": 0.02, "overage_unit_rate": 0.06},
        {"feature_id": 104, "feature_code": "EGRESS_GB", "service_category": "NETWORKING", "unit_infra_cost": 0.01, "overage_unit_rate": 0.05},
        {"feature_id": 105, "feature_code": "API_GATEWAY_REQ_K", "service_category": "API", "unit_infra_cost": 0.03, "overage_unit_rate": 0.08},
        {"feature_id": 106, "feature_code": "VECTOR_SEARCH_K", "service_category": "AI_ML", "unit_infra_cost": 0.08, "overage_unit_rate": 0.22},
        {"feature_id": 107, "feature_code": "LOG_INGESTION_GB", "service_category": "STORAGE", "unit_infra_cost": 0.03, "overage_unit_rate": 0.07},
        {"feature_id": 108, "feature_code": "ASYNC_WORKER_HR", "service_category": "COMPUTE", "unit_infra_cost": 0.06, "overage_unit_rate": 0.14},
    ]
    feat_map = {f["feature_id"]: f for f in features}

    # 3. Accounts (Dimension)
    accounts = []
    base_signup = datetime(2025, 1, 1)
    for a_id in range(1001, 1001 + num_accounts):
        c_name = f"{rng.choice(company_prefixes)} {rng.choice(company_suffixes)} {a_id}"
        industry = rng.choice(industries)
        tier = rng.choices(tiers, weights=[20, 40, 25, 15])[0]
        b_cycle = rng.choice(billing_cycles)
        signup = base_signup + timedelta(days=rng.randint(0, 120))
        accounts.append({
            "account_id": a_id,
            "company_name": c_name,
            "industry": industry,
            "pricing_tier": tier,
            "billing_cycle": b_cycle,
            "signup_date": signup.strftime("%Y-%m-%d"),
        })

    # 4. Usage Events (Fact)
    events = []
    event_id = 5000001
    base_date = datetime(2025, 3, 1, 0, 0, 0)

    total_gross_billed = 0.0
    total_infra_cost = 0.0
    total_net_margin = 0.0

    monthly_summary = defaultdict(lambda: {
        "accounts": set(),
        "total_events": 0,
        "total_consumed_units": 0,
        "total_gross_billed": 0.0,
        "total_infra_cost": 0.0,
        "total_net_margin": 0.0,
    })

    account_kpis = defaultdict(lambda: {
        "total_events": 0,
        "total_consumed_units": 0,
        "total_gross_billed": 0.0,
        "total_net_margin": 0.0,
    })

    for acct in accounts:
        a_id = acct["account_id"]
        tier = acct["pricing_tier"]
        ind = acct["industry"]

        # Number of events scaled by tier
        tier_mult = {"FREE": 0.4, "STARTUP": 0.8, "SCALE": 1.2, "ENTERPRISE": 2.0}[tier]
        n_events = int(rng.randint(max(5, int(events_per_account * 0.5)), int(events_per_account * 1.5)) * tier_mult)

        for _ in range(n_events):
            event_id += 1
            feat = rng.choice(features)
            f_id = feat["feature_id"]
            u_cost = feat["unit_infra_cost"]
            u_rate = feat["overage_unit_rate"]

            # Consumed units (between 5 and 500 units per event)
            consumed = rng.randint(5, 500)
            e_time = base_date + timedelta(
                days=rng.randint(0, 90),
                hours=rng.randint(0, 23),
                minutes=rng.randint(0, 59),
                seconds=rng.randint(0, 59)
            )
            region = rng.choice(regions)

            gross_amt = round(consumed * u_rate, 2)
            infra_c = round(consumed * u_cost, 2)
            margin = round(gross_amt - infra_c, 2)

            total_gross_billed += gross_amt
            total_infra_cost += infra_c
            total_net_margin += margin

            m_key = (e_time.year, e_time.month, tier, ind)
            monthly_summary[m_key]["accounts"].add(a_id)
            monthly_summary[m_key]["total_events"] += 1
            monthly_summary[m_key]["total_consumed_units"] += consumed
            monthly_summary[m_key]["total_gross_billed"] += gross_amt
            monthly_summary[m_key]["total_infra_cost"] += infra_c
            monthly_summary[m_key]["total_net_margin"] += margin

            account_kpis[a_id]["total_events"] += 1
            account_kpis[a_id]["total_consumed_units"] += consumed
            account_kpis[a_id]["total_gross_billed"] += gross_amt
            account_kpis[a_id]["total_net_margin"] += margin

            events.append({
                "event_id": event_id,
                "account_id": a_id,
                "feature_id": f_id,
                "event_timestamp": e_time.strftime("%Y-%m-%d %H:%M:%S"),
                "consumed_units": consumed,
                "region": region,
            })

    total_gross_billed = round(total_gross_billed, 2)
    total_infra_cost = round(total_infra_cost, 2)
    total_net_margin = round(total_net_margin, 2)

    # Write Staging CSV files
    with open(SEED_DIR / "stg_accounts.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["account_id", "company_name", "industry", "pricing_tier", "billing_cycle", "signup_date"])
        writer.writeheader()
        writer.writerows(accounts)

    with open(SEED_DIR / "stg_plans.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["plan_id", "pricing_tier", "monthly_base_fee", "included_units"])
        writer.writeheader()
        writer.writerows(plans)

    with open(SEED_DIR / "stg_features.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["feature_id", "feature_code", "service_category", "unit_infra_cost", "overage_unit_rate"])
        writer.writeheader()
        writer.writerows(features)

    with open(SEED_DIR / "stg_usage_events.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["event_id", "account_id", "feature_id", "event_timestamp", "consumed_units", "region"])
        writer.writeheader()
        writer.writerows(events)

    # Build validation specification
    spec = {
        "benchmark_id": "b2b_saas_billing",
        "dataset_version": "1.0",
        "description": "B2B SaaS Cloud Usage, Metered Billing & Margin Analytics Data Product",
        "target_objects": {
            "staging_tables": ["stg_accounts", "stg_plans", "stg_features", "stg_usage_events"],
            "dimension_tables": ["dim_accounts"],
            "fact_tables": ["fct_metered_usage"],
            "aggregate_tables": ["agg_monthly_billing"],
            "views": ["dim_account_usage_kpi"]
        },
        "assertions": [
            {
                "id": "stg_tables_exist",
                "description": "All 4 staging tables exist in target schema",
                "type": "staging_tables_check",
                "expected_count": 4
            },
            {
                "id": "dim_accounts_created",
                "description": "Conformed dimension dim_accounts created",
                "type": "table_exists",
                "target": "dim_accounts"
            },
            {
                "id": "fct_metered_usage_created",
                "description": "Atomic fact table fct_metered_usage created",
                "type": "table_exists",
                "target": "fct_metered_usage"
            },
            {
                "id": "agg_monthly_billing_created",
                "description": "Periodic aggregate table agg_monthly_billing created",
                "type": "table_exists",
                "target": "agg_monthly_billing"
            },
            {
                "id": "dim_account_usage_kpi_created",
                "description": "Customer KPI summary table/view dim_account_usage_kpi created",
                "type": "table_or_view_exists",
                "target": "dim_account_usage_kpi"
            },
            {
                "id": "gross_billed_checksum",
                "description": "Fact table gross_billed_amount checksum matches gold standard within 0.1%",
                "type": "metric_checksum",
                "target_table": "fct_metered_usage",
                "target_column": "gross_billed_amount",
                "expected_value": total_gross_billed,
                "relative_tolerance": 0.001
            },
            {
                "id": "net_margin_checksum",
                "description": "Fact table net_margin checksum matches gold standard within 0.1%",
                "type": "metric_checksum",
                "target_table": "fct_metered_usage",
                "target_column": "net_margin",
                "expected_value": total_net_margin,
                "relative_tolerance": 0.001
            },
            {
                "id": "aggregate_reconciliation_checksum",
                "description": "Aggregate table total_billed_amount reconciles with fact table",
                "type": "metric_checksum",
                "target_table": "agg_monthly_billing",
                "target_column": "total_billed_amount",
                "expected_value": total_gross_billed,
                "relative_tolerance": 0.001
            }
        ]
    }

    with open(DATASET_DIR / "validation_spec.json", "w") as f:
        json.dump(spec, f, indent=2)

    print(f"Generated {len(accounts)} accounts, {len(plans)} plans, {len(features)} features, {len(events):,} usage events.")
    print(f"Total Gross Billed: ${total_gross_billed:,.2f}, Total Net Margin: ${total_net_margin:,.2f}")
    print(f"Validation spec saved to {DATASET_DIR / 'validation_spec.json'}")
    return spec


def main():
    parser = argparse.ArgumentParser(description="Generate B2B SaaS Billing Benchmark Dataset")
    parser.add_argument("--accounts", type=int, default=600, help="Number of customer accounts (default: 600)")
    parser.add_argument("--events-per-account", type=int, default=50, help="Average events per account (default: 50)")
    parser.add_argument("--seed", type=int, default=42, help="Random seed (default: 42)")
    args = parser.parse_args()

    generate_b2b_billing_data(
        num_accounts=args.accounts,
        events_per_account=args.events_per_account,
        seed=args.seed
    )


if __name__ == "__main__":
    main()
