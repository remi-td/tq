#!/usr/bin/env python3
"""Next-Generation Enterprise Data Platform 360 Benchmark Generator.

Generates a realistic, voluminous multi-table dataset reflective of an end-to-end
data engineering lifecycle:
  1. High-throughput bulk loading (~25,000 records across 5 entities).
  2. Realistic data quality anomalies (dirty casing, whitespace, mixed date formats, nulls, duplicates).
  3. Reusable dimensional modeling (conformed dimensions, atomic facts, Primary Index optimization).
  4. Enterprise semantic layer (reusable views computing OTIF, Gross Profit, Net Margin, RFM).
  5. Analytical querying & executive KPI auditing.
"""

from __future__ import annotations

import csv
import json
import random
from collections import defaultdict
from datetime import date, timedelta
from pathlib import Path

DATASET_DIR = Path(__file__).resolve().parent
SEED_DIR = DATASET_DIR / "seed_data"


def generate_enterprise_data(
    num_customers: int = 1000,
    num_products: int = 250,
    num_orders: int = 4000,
    avg_lines_per_order: int = 5
) -> dict:
    """Generate deterministic seed dataset and gold validation metrics."""
    SEED_DIR.mkdir(parents=True, exist_ok=True)
    rng = random.Random(1337)  # Deterministic seed

    # -------------------------------------------------------------
    # 1. Sales Channels (Lookup Dimension)
    # -------------------------------------------------------------
    channels = [
        {"channel_id": 1, "channel_code": "ONLINE_WEB", "channel_name": "E-Commerce Web Portal", "cost_overhead_pct": 0.05},
        {"channel_id": 2, "channel_code": "MOBILE_APP", "channel_name": "Mobile Application", "cost_overhead_pct": 0.04},
        {"channel_id": 3, "channel_code": "RETAIL_STORE", "channel_name": "Flagship Retail Store", "cost_overhead_pct": 0.12},
        {"channel_id": 4, "channel_code": "B2B_WHOLESALE", "channel_name": "B2B Wholesale Accounts", "cost_overhead_pct": 0.02},
        {"channel_id": 5, "channel_code": "MARKETPLACE", "channel_name": "Third-Party Marketplace", "cost_overhead_pct": 0.15},
    ]

    # -------------------------------------------------------------
    # 2. Products Master (Catalog Dimension)
    # -------------------------------------------------------------
    categories = ["ENTERPRISE_SERVER", "CLOUD_STORAGE", "NETWORKING_SWITCH", "AI_ACCELERATOR", "SECURITY_GATEWAY"]
    manufacturers = ["TITAN_SYSTEMS", "APEX_HARDWARE", "VORTEX_SILICON", "QUANTUM_OPTICS", "NEXUS_CHIPS"]

    products = []
    prod_map = {}
    for p_id in range(1, num_products + 1):
        cat = categories[(p_id - 1) % len(categories)]
        mfgr = manufacturers[(p_id - 1) % len(manufacturers)]
        unit_cost = round(rng.uniform(45.0, 850.0), 2)
        markup = rng.uniform(1.35, 1.85)
        list_price = round(unit_cost * markup, 2)
        prod = {
            "product_id": p_id,
            "sku": f"SKU-{cat[:4]}-{p_id:05d}",
            "product_name": f"{mfgr} {cat.replace('_', ' ').title()} Gen-{p_id % 7 + 1}",
            "category": cat,
            "manufacturer": mfgr,
            "unit_cost": unit_cost,
            "list_price": list_price
        }
        products.append(prod)
        prod_map[p_id] = prod

    # -------------------------------------------------------------
    # 3. Customers Master (with intentional Data Quality anomalies)
    # -------------------------------------------------------------
    regions = ["NORTH_AMERICA", "EMEA", "APAC", "LATAM"]
    tiers_clean = ["STANDARD", "SILVER", "GOLD", "PLATINUM"]
    tiers_dirty_pool = [
        "STANDARD", "standard", " STANDARD ", "Standard",
        "SILVER", "silver", " SILVER", "Silver",
        "GOLD", "gold", "Gold ", " GOLD",
        "PLATINUM", "platinum", "Platinum", " PLATINUM "
    ]

    customers_raw = []
    cust_map = {}
    signup_base = date(2024, 1, 1)

    for c_id in range(1, num_customers + 1):
        region = regions[(c_id - 1) % len(regions)]
        tier_clean = tiers_clean[(c_id - 1) % len(tiers_clean)]
        tier_dirty = rng.choice([t for t in tiers_dirty_pool if t.strip().upper() == tier_clean])
        signup_dt = signup_base + timedelta(days=rng.randint(0, 360))
        
        # Injected DQ: 3% null emails, dirty leading/trailing spaces in names
        email = f"user_{c_id}@enterprise{c_id % 40}.com" if rng.random() > 0.03 else ""
        cust_name = f" Customer {c_id:05d} Inc " if rng.random() < 0.20 else f"Customer {c_id:05d} Inc"

        c = {
            "customer_id": c_id,
            "customer_name": cust_name,
            "customer_tier": tier_dirty,
            "customer_region": region,
            "email_address": email,
            "account_created_date": signup_dt.strftime("%Y-%m-%d")
        }
        customers_raw.append(c)
        cust_map[c_id] = {
            "customer_id": c_id,
            "customer_name": cust_name.strip(),
            "customer_tier": tier_clean,
            "customer_region": region
        }

    # Injected DQ: 15 duplicate customer records representing duplicate sync batches
    duplicate_customers = [dict(c) for c in rng.sample(customers_raw, 15)]
    customers_raw.extend(duplicate_customers)

    # -------------------------------------------------------------
    # 4. Orders & Lineitems (High Volume & Complex Lifecycle)
    # -------------------------------------------------------------
    orders_raw = []
    lineitems_raw = []

    order_base_date = date(2025, 1, 1)
    lineitem_id = 1

    # Ground truth accumulators for clean semantic metrics
    total_valid_lineitems = 0
    total_gross_sales = 0.0
    total_discount_amount = 0.0
    total_net_revenue = 0.0
    total_cogs = 0.0
    total_gross_profit = 0.0
    total_otif_eligible = 0
    total_otif_success = 0
    total_shipping_delay_days = 0

    category_kpis = defaultdict(lambda: {
        "lineitem_count": 0, "net_revenue": 0.0, "gross_profit": 0.0
    })
    channel_kpis = defaultdict(lambda: {
        "order_count": 0, "net_revenue": 0.0, "gross_profit": 0.0, "otif_count": 0
    })
    customer_kpis = defaultdict(lambda: {
        "order_count": 0, "net_revenue": 0.0, "gross_profit": 0.0
    })

    for o_idx in range(1, num_orders + 1):
        order_id = 10000 + o_idx
        cust_id = (o_idx % num_customers) + 1
        chan_id = ((o_idx - 1) % len(channels)) + 1
        order_days = rng.randint(0, 180)
        order_dt = order_base_date + timedelta(days=order_days)
        promised_dt = order_dt + timedelta(days=7)

        # Injected DQ: 10% mixed date formatting in order_date (YYYY/MM/DD instead of YYYY-MM-DD)
        order_date_str = order_dt.strftime("%Y/%m/%d") if rng.random() < 0.10 else order_dt.strftime("%Y-%m-%d")

        # Lines per order (between 3 and 7)
        num_lines = rng.randint(3, 7)
        order_net_rev = 0.0
        order_profit = 0.0

        for line_num in range(1, num_lines + 1):
            p_id = rng.randint(1, num_products)
            prod = prod_map[p_id]
            qty = rng.randint(1, 12)
            unit_price = prod["list_price"]
            unit_cost = prod["unit_cost"]

            # Discount between 0% and 25%
            discount_pct = round(rng.choice([0.0, 0.05, 0.10, 0.15, 0.20, 0.25]), 2)
            
            # Shipping & Delivery logic
            ship_delay = rng.randint(1, 10)  # Ship delay in days
            ship_dt = order_dt + timedelta(days=ship_delay)
            delivery_dt = ship_dt + timedelta(days=rng.randint(1, 4))

            # Delivery status: 92% delivered, 5% returned, 3% cancelled
            status_dice = rng.random()
            if status_dice < 0.05:
                return_flag = "Y"
                status_code = "RETURNED"
            elif status_dice < 0.08:
                return_flag = "N"
                status_code = "CANCELLED"
            else:
                return_flag = "N"
                status_code = "DELIVERED"

            # Injected DQ: cancelled orders have null delivery date
            delivery_str = delivery_dt.strftime("%Y-%m-%d") if status_code != "CANCELLED" else ""

            # Standard financial calculations
            gross = round(qty * unit_price, 2)
            discount_amt = round(gross * discount_pct, 2)
            net_rev = round(gross - discount_amt, 2)
            cogs = round(qty * unit_cost, 2)
            profit = round(net_rev - cogs, 2)

            # On-Time In-Full (OTIF): Delivered on or before promised date AND not returned
            is_otif = 1 if (status_code == "DELIVERED" and delivery_dt <= promised_dt and return_flag == "N") else 0

            # Accumulate ground truth for clean records
            total_valid_lineitems += 1
            total_gross_sales += gross
            total_discount_amount += discount_amt
            total_net_revenue += net_rev
            total_cogs += cogs
            total_gross_profit += profit

            if status_code != "CANCELLED":
                total_otif_eligible += 1
                if is_otif:
                    total_otif_success += 1
                total_shipping_delay_days += ship_delay

            cat = prod["category"]
            category_kpis[cat]["lineitem_count"] += 1
            category_kpis[cat]["net_revenue"] += net_rev
            category_kpis[cat]["gross_profit"] += profit

            channel_kpis[chan_id]["net_revenue"] += net_rev
            channel_kpis[chan_id]["gross_profit"] += profit
            if is_otif:
                channel_kpis[chan_id]["otif_count"] += 1

            order_net_rev += net_rev
            order_profit += profit

            lineitem_row = {
                "lineitem_id": lineitem_id,
                "order_id": order_id,
                "line_number": line_num,
                "product_id": p_id,
                "quantity": qty,
                "unit_price": unit_price,
                "discount_pct": discount_pct,
                "tax_rate": 0.08,
                "ship_date": ship_dt.strftime("%Y-%m-%d"),
                "promised_date": promised_dt.strftime("%Y-%m-%d"),
                "delivery_date": delivery_str,
                "return_flag": return_flag,
                "fulfillment_status": status_code
            }
            lineitems_raw.append(lineitem_row)
            lineitem_id += 1

        order_row = {
            "order_id": order_id,
            "customer_id": cust_id,
            "channel_id": chan_id,
            "order_date": order_date_str,
            "order_priority": rng.choice(["1-URGENT", "2-HIGH", "3-NORMAL", "4-LOW"]),
            "payment_method": rng.choice(["CORPORATE_INVOICE", "CREDIT_CARD", "WIRE_TRANSFER"]),
            "order_total_lines": num_lines,
            "order_net_revenue": round(order_net_rev, 2)
        }
        orders_raw.append(order_row)
        channel_kpis[chan_id]["order_count"] += 1
        customer_kpis[cust_id]["order_count"] += 1
        customer_kpis[cust_id]["net_revenue"] += order_net_rev
        customer_kpis[cust_id]["gross_profit"] += order_profit

    # Injected DQ: 100 duplicate lineitem rows (CDC re-delivery simulation)
    dup_lines = [dict(l) for l in rng.sample(lineitems_raw, 100)]
    lineitems_raw.extend(dup_lines)

    # -------------------------------------------------------------
    # 5. Write CSV Seed Files
    # -------------------------------------------------------------
    def write_csv(filename: str, rows: list[dict], fieldnames: list[str]):
        path = SEED_DIR / filename
        with open(path, "w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(rows)
        print(f"Generated {filename}: {len(rows):,} rows ({path.stat().st_size / 1024:.1f} KB)")

    write_csv("stg_channels.csv", channels, ["channel_id", "channel_code", "channel_name", "cost_overhead_pct"])
    write_csv("stg_products.csv", products, ["product_id", "sku", "product_name", "category", "manufacturer", "unit_cost", "list_price"])
    write_csv("stg_customers.csv", customers_raw, ["customer_id", "customer_name", "customer_tier", "customer_region", "email_address", "account_created_date"])
    write_csv("stg_orders.csv", orders_raw, ["order_id", "customer_id", "channel_id", "order_date", "order_priority", "payment_method", "order_total_lines", "order_net_revenue"])
    write_csv("stg_order_lineitems.csv", lineitems_raw, ["lineitem_id", "order_id", "line_number", "product_id", "quantity", "unit_price", "discount_pct", "tax_rate", "ship_date", "promised_date", "delivery_date", "return_flag", "fulfillment_status"])

    # -------------------------------------------------------------
    # 6. Compute Ground Truth Metrics
    # -------------------------------------------------------------
    overall_net_margin_pct = round((total_gross_profit / max(1.0, total_net_revenue)) * 100.0, 2)
    overall_otif_rate_pct = round((total_otif_success / max(1, total_otif_eligible)) * 100.0, 2)
    avg_shipping_delay_days = round(total_shipping_delay_days / max(1, total_otif_eligible), 2)

    validation_spec = {
        "dataset_name": "enterprise_data_platform_360",
        "description": "Next-Gen Enterprise Omnichannel Retail Data Platform with DQ remediation, MPP modeling, and semantic layer",
        "total_source_lineitems_raw": len(lineitems_raw),
        "total_valid_lineitems_deduped": total_valid_lineitems,
        "total_orders": num_orders,
        "total_customers_unique": num_customers,
        "total_products": num_products,
        "total_channels": len(channels),
        "ground_truth_metrics": {
            "total_gross_sales": round(total_gross_sales, 2),
            "total_discount_amount": round(total_discount_amount, 2),
            "total_net_revenue": round(total_net_revenue, 2),
            "total_cogs": round(total_cogs, 2),
            "total_gross_profit": round(total_gross_profit, 2),
            "net_margin_pct": overall_net_margin_pct,
            "otif_rate_pct": overall_otif_rate_pct,
            "avg_shipping_delay_days": avg_shipping_delay_days
        },
        "target_objects": {
            "staging_tables": [
                "stg_channels", "stg_products", "stg_customers", "stg_orders", "stg_order_lineitems"
            ],
            "dimension_tables": [
                "dim_customers", "dim_products", "dim_channels"
            ],
            "fact_tables": [
                "fct_orders", "fct_order_lineitems"
            ],
            "semantic_views": [
                "v_sem_order_fulfillment", "v_sem_customer_rfm_kpis", "v_sem_channel_performance"
            ]
        },
        "assertions": [
            {
                "id": "staging_tables_loaded",
                "description": "All 5 raw staging tables successfully created and loaded",
                "type": "table_exists",
                "targets": ["stg_channels", "stg_products", "stg_customers", "stg_orders", "stg_order_lineitems"]
            },
            {
                "id": "raw_lineitem_count_matches",
                "description": "Staging lineitems table matches source CSV row count exactly (~20k+ rows)",
                "type": "row_count_exact",
                "target": "stg_order_lineitems",
                "expected": len(lineitems_raw)
            },
            {
                "id": "dim_customers_deduplicated",
                "description": "Customer dimension successfully cleaned of duplicate sync records",
                "type": "row_count_exact",
                "target": "dim_customers",
                "expected": num_customers
            },
            {
                "id": "dim_customers_primary_index",
                "description": "Customer dimension has explicit Primary Index on customer_id",
                "type": "primary_index_column",
                "target": "dim_customers",
                "column": "customer_id"
            },
            {
                "id": "fct_lineitems_deduplicated",
                "description": "Fact table deduplicated CDC lines and matches valid unique lineitems",
                "type": "row_count_exact",
                "target": "fct_order_lineitems",
                "expected": total_valid_lineitems
            },
            {
                "id": "fct_lineitems_primary_index",
                "description": "Lineitems fact table has Primary Index on order_id to prevent join redistribution",
                "type": "primary_index_column",
                "target": "fct_order_lineitems",
                "column": "order_id"
            },
            {
                "id": "semantic_layer_views_created",
                "description": "All 3 reusable semantic views created in database",
                "type": "view_exists",
                "targets": ["v_sem_order_fulfillment", "v_sem_customer_rfm_kpis", "v_sem_channel_performance"]
            },
            {
                "id": "semantic_net_revenue_exact",
                "description": "Semantic view v_sem_order_fulfillment computes exact ground truth Net Revenue",
                "type": "scalar_sum_match",
                "target": "v_sem_order_fulfillment",
                "column": "net_revenue_usd",
                "expected": round(total_net_revenue, 2),
                "tolerance": 1.0
            },
            {
                "id": "semantic_gross_profit_exact",
                "description": "Semantic view v_sem_order_fulfillment computes exact ground truth Gross Profit",
                "type": "scalar_sum_match",
                "target": "v_sem_order_fulfillment",
                "column": "gross_profit_usd",
                "expected": round(total_gross_profit, 2),
                "tolerance": 1.0
            },
            {
                "id": "executive_insights_artifact",
                "description": "Agent produced DATA_PRODUCT_INSIGHTS.md answering business questions via semantic layer",
                "type": "file_exists",
                "filename": "DATA_PRODUCT_INSIGHTS.md"
            }
        ]
    }

    # Write validation_spec.json
    spec_path = DATASET_DIR / "validation_spec.json"
    with open(spec_path, "w", encoding="utf-8") as f:
        json.dump(validation_spec, f, indent=2)
    print(f"Validation specification written to {spec_path}")

    return validation_spec


if __name__ == "__main__":
    generate_enterprise_data()
