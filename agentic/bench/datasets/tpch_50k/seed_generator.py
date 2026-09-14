#!/usr/bin/env python3
"""TPC-H Order Fulfillment & Profitability Benchmark Data Generator.

Generates calibrated, deterministic seed datasets for live Teradata benchmarking,
computes gold standard validation metrics, and writes validation_spec.json.
"""

from __future__ import annotations

import csv
import json
from pathlib import Path
from datetime import date, timedelta
from collections import defaultdict
import random

DATASET_DIR = Path(__file__).resolve().parent
SEED_DIR = DATASET_DIR / "seed_data"


def generate_tpch_data(
    num_customers: int = 3700,
    num_parts: int = 200,
    orders_per_cust: int = 8,
    lines_per_order: int = 5
) -> dict:
    """Generate deterministic seed dataset and gold validation metrics."""
    SEED_DIR.mkdir(parents=True, exist_ok=True)
    rng = random.Random(42)

    nations = ["UNITED STATES", "UNITED KINGDOM", "GERMANY", "FRANCE", "JAPAN", "BRAZIL", "AUSTRALIA", "CANADA"]
    segments = ["BUILDING", "AUTOMOBILE", "MACHINERY", "HOUSEHOLD", "FURNITURE"]
    priorities = ["1-URGENT", "2-HIGH", "3-MEDIUM", "4-LOW"]
    categories = ["HARDWARE", "FASTENERS", "ELECTRONICS", "HYDRAULICS", "STRUCTURAL"]
    manufacturers = ["MFGR#1", "MFGR#2", "MFGR#3", "MFGR#4", "MFGR#5"]

    # 1. Customers
    customers = []
    cust_map = {}
    for c_id in range(1, num_customers + 1):
        mkt = rng.choice(segments)
        nation = rng.choice(nations)
        c = {
            "cust_id": c_id,
            "cust_name": f"Customer#{c_id:06d}",
            "mkt_segment": mkt,
            "nation_name": nation,
        }
        customers.append(c)
        cust_map[c_id] = c

    # 2. Parts Catalog
    parts = []
    part_map = {}
    for p_id in range(100, 100 + num_parts):
        cost = round(rng.uniform(15.0, 120.0), 2)
        p = {
            "part_id": p_id,
            "part_name": f"Part#{p_id:05d}",
            "mfgr": rng.choice(manufacturers),
            "category": rng.choice(categories),
            "supply_cost": cost,
        }
        parts.append(p)
        part_map[p_id] = p

    # 3. Orders and Lineitems
    orders = []
    lineitems = []
    order_id = 1000
    base_date = date(2025, 1, 1)

    total_net_revenue = 0.0
    total_supply_cost = 0.0
    total_profit = 0.0
    completed_orders_count = 0
    total_shipping_days = 0
    shipped_lines_count = 0

    monthly_perf = defaultdict(lambda: {
        "order_ids": set(),
        "lineitem_count": 0,
        "total_quantity": 0,
        "total_net_revenue": 0.0,
        "total_profit": 0.0,
        "shipping_delay_days_sum": 0,
    })

    cust_profitability = defaultdict(lambda: {
        "order_ids": set(),
        "total_quantity": 0,
        "total_net_revenue": 0.0,
        "total_profit": 0.0,
    })

    for cust in customers:
        n_orders = rng.randint(1, orders_per_cust)
        for _ in range(n_orders):
            order_id += 1
            o_date = base_date + timedelta(days=rng.randint(0, 180))
            priority = rng.choice(priorities)
            status = rng.choice(["F", "O", "P"])  # Fulfilled, Open, Pending
            if status == "F":
                completed_orders_count += 1

            n_lines = rng.randint(1, lines_per_order)
            order_total = 0.0

            for line_no in range(1, n_lines + 1):
                part = rng.choice(parts)
                part_id = part["part_id"]
                part_cost = part["supply_cost"]

                qty = rng.randint(1, 20)
                unit_price = round(part_cost * rng.uniform(1.25, 1.85), 2)
                ext_price = round(qty * unit_price, 2)
                discount = round(rng.choice([0.00, 0.02, 0.05, 0.10]), 2)
                tax = 0.08

                net_amount = round(ext_price * (1.0 - discount), 2)
                item_cost = round(qty * part_cost, 2)
                item_profit = round(net_amount - item_cost, 2)

                order_total += net_amount
                total_net_revenue += net_amount
                total_supply_cost += item_cost
                total_profit += item_profit

                ship_delay = rng.randint(1, 15)
                ship_date = o_date + timedelta(days=ship_delay)
                commit_date = o_date + timedelta(days=10)
                ret_flag = "R" if (status == "F" and rng.random() < 0.05) else "N"
                line_status = "F" if status == "F" else "O"

                if line_status == "F":
                    total_shipping_days += ship_delay
                    shipped_lines_count += 1

                # Monthly performance aggregation tracking
                m_key = (o_date.year, o_date.month, cust["mkt_segment"])
                monthly_perf[m_key]["order_ids"].add(order_id)
                monthly_perf[m_key]["lineitem_count"] += 1
                monthly_perf[m_key]["total_quantity"] += qty
                monthly_perf[m_key]["total_net_revenue"] += net_amount
                monthly_perf[m_key]["total_profit"] += item_profit
                monthly_perf[m_key]["shipping_delay_days_sum"] += ship_delay

                # Customer profitability tracking
                c_id = cust["cust_id"]
                cust_profitability[c_id]["order_ids"].add(order_id)
                cust_profitability[c_id]["total_quantity"] += qty
                cust_profitability[c_id]["total_net_revenue"] += net_amount
                cust_profitability[c_id]["total_profit"] += item_profit

                lineitems.append({
                    "order_id": order_id,
                    "line_number": line_no,
                    "part_id": part_id,
                    "quantity": qty,
                    "extended_price": ext_price,
                    "discount": discount,
                    "tax": tax,
                    "return_flag": ret_flag,
                    "line_status": line_status,
                    "ship_date": ship_date.isoformat(),
                    "commit_date": commit_date.isoformat(),
                })

            orders.append({
                "order_id": order_id,
                "cust_id": cust["cust_id"],
                "order_status": status,
                "total_price": round(order_total, 2),
                "order_date": o_date.isoformat(),
                "order_priority": priority,
            })

    # Write CSV files
    with open(SEED_DIR / "stg_customers.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["cust_id", "cust_name", "mkt_segment", "nation_name"])
        writer.writeheader()
        writer.writerows(customers)

    with open(SEED_DIR / "stg_parts.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["part_id", "part_name", "mfgr", "category", "supply_cost"])
        writer.writeheader()
        writer.writerows(parts)

    with open(SEED_DIR / "stg_orders.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["order_id", "cust_id", "order_status", "total_price", "order_date", "order_priority"])
        writer.writeheader()
        writer.writerows(orders)

    with open(SEED_DIR / "stg_lineitem.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["order_id", "line_number", "part_id", "quantity", "extended_price", "discount", "tax", "return_flag", "line_status", "ship_date", "commit_date"])
        writer.writeheader()
        writer.writerows(lineitems)

    total_net_revenue = round(total_net_revenue, 2)
    total_supply_cost = round(total_supply_cost, 2)
    total_profit = round(total_profit, 2)
    avg_shipping_delay = round(total_shipping_days / max(1, shipped_lines_count), 2)

    spec = {
        "dataset_name": "tpch_order_fulfillment",
        "counts": {
            "customers": len(customers),
            "parts": len(parts),
            "orders": len(orders),
            "lineitems": len(lineitems),
            "fulfilled_orders": completed_orders_count,
            "agg_monthly_rows": len(monthly_perf)
        },
        "gold_metrics": {
            "total_net_revenue": total_net_revenue,
            "total_supply_cost": total_supply_cost,
            "total_profit": total_profit,
            "avg_shipping_delay_days": avg_shipping_delay,
            "total_shipped_lines": shipped_lines_count
        },
        "target_objects": {
            "staging_tables": [
                "stg_customers",
                "stg_parts",
                "stg_orders",
                "stg_lineitem"
            ],
            "dimension_tables": [
                "dim_customers"
            ],
            "fact_tables": [
                "fct_order_lineitem"
            ],
            "aggregate_tables": [
                "agg_monthly_performance"
            ],
            "analytics_views_or_tables": [
                "dim_customer_profitability"
            ]
        },
        "assertions": [
            {
                "id": "stg_tables_exist",
                "description": "All 4 staging tables (stg_customers, stg_parts, stg_orders, stg_lineitem) created and populated",
                "type": "staging_tables_check"
            },
            {
                "id": "dim_customers_created",
                "description": "Conformed dimension table dim_customers created with cust_id Primary Index",
                "type": "table_exists",
                "target": "dim_customers"
            },
            {
                "id": "fct_order_lineitem_created",
                "description": "Atomic fact table fct_order_lineitem created with lineitem grain",
                "type": "table_exists",
                "target": "fct_order_lineitem"
            },
            {
                "id": "agg_monthly_performance_created",
                "description": "Aggregate summary table agg_monthly_performance created at Year-Month-Segment grain",
                "type": "table_exists",
                "target": "agg_monthly_performance"
            },
            {
                "id": "dim_customer_profitability_created",
                "description": "Customer profitability dimension or semantic view created",
                "type": "table_or_view_exists",
                "target": "dim_customer_profitability"
            },
            {
                "id": "revenue_integrity_checksum",
                "description": "Fact table net revenue checksum matches gold standard within 0.1%",
                "type": "metric_checksum",
                "target_table": "fct_order_lineitem",
                "target_column": "net_revenue",
                "expected_value": total_net_revenue,
                "relative_tolerance": 0.001
            },
            {
                "id": "profit_integrity_checksum",
                "description": "Fact table gross profit checksum matches gold standard within 0.1%",
                "type": "metric_checksum",
                "target_table": "fct_order_lineitem",
                "target_column": "profit",
                "expected_value": total_profit,
                "relative_tolerance": 0.001
            },
            {
                "id": "aggregate_reconciliation_checksum",
                "description": "Aggregate table total_net_revenue reconciles with fact net revenue",
                "type": "metric_checksum",
                "target_table": "agg_monthly_performance",
                "target_column": "total_net_revenue",
                "expected_value": total_net_revenue,
                "relative_tolerance": 0.001
            }
        ]
    }

    with open(DATASET_DIR / "validation_spec.json", "w") as f:
        json.dump(spec, f, indent=2)

    print(f"Generated {len(customers)} customers, {len(parts)} parts, {len(orders)} orders, {len(lineitems)} lineitems.")
    print(f"Total Net Revenue: ${total_net_revenue:,.2f}, Total Profit: ${total_profit:,.2f}")
    print(f"Validation spec saved to {DATASET_DIR / 'validation_spec.json'}")
    return spec


if __name__ == "__main__":
    generate_tpch_data()
