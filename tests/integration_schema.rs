//! Integration tests for `tq schema` (Issue #55, Sprint 79)
//!
//! Corresponds to TC111-I01 through TC111-I06 in `tests/strategy/sprint-79-strategy.md`.
//! Every test runs the built `tq` binary against the live Teradata database configured
//! by `TQ_LOGON` (from `.env` via `dotenvy`).
//!
//! Run with:
//! ```bash
//! cargo test --test integration_schema -- --ignored
//! ```

#![allow(deprecated)]

use assert_cmd::Command;
use std::process::Output;

const LIVE_DB: &str = "demo_user";

fn tq_cmd() -> Command {
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.env("TQ_LOGON", logon);
    cmd.env_remove("TQ_PROFILE");
    cmd
}

fn run(args: &[&str]) -> Output {
    tq_cmd().args(args).output().unwrap()
}

fn stdout_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn ensure_test_tables() {
    let _ = run(&["query", "CREATE TABLE demo_user.customers (customer_id INTEGER, name VARCHAR(100)) PRIMARY INDEX (customer_id);"]);
    let _ = run(&["query", "CREATE TABLE demo_user.orders (order_id INTEGER, customer_id INTEGER, total_amount DECIMAL(10,2)) PRIMARY INDEX (order_id);"]);
}

#[test]
#[ignore]
fn tc111_i01_live_schema_current_database() {
    ensure_test_tables();
    let out = run(&["schema"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = stdout_str(&out);
    assert!(stdout.contains("Database Schema:"));
    assert!(stdout.contains("Tables & Indexes"));
}

#[test]
#[ignore]
fn tc111_i02_live_schema_database_and_pattern() {
    ensure_test_tables();
    let out = run(&["schema", LIVE_DB, "cust*"]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    assert!(stdout.contains("Database Schema:"));
}

#[test]
#[ignore]
fn tc111_i03_live_schema_format_json() {
    ensure_test_tables();
    let out = run(&["schema", "--format", "json"]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Must be valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["data"]["database"].is_string());
    assert!(parsed["data"]["tables"].is_array());
    assert!(parsed["data"]["join_paths"].is_array());

    // Also test --json flag shortcut
    let out_json_flag = run(&["schema", "--json"]);
    assert!(out_json_flag.status.success());
    let stdout_flag = stdout_str(&out_json_flag);
    let parsed_flag: serde_json::Value = serde_json::from_str(&stdout_flag).expect("Must be valid JSON");
    assert_eq!(parsed_flag["ok"], true);
}

#[test]
#[ignore]
fn tc111_i04_live_schema_format_markdown() {
    ensure_test_tables();
    let out = run(&["schema", "--format", "markdown"]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    assert!(stdout.contains("# Schema Graph:"));
    assert!(stdout.contains("```mermaid"));
    assert!(stdout.contains("erDiagram"));
}

#[test]
#[ignore]
fn tc111_i05_live_schema_format_compact() {
    ensure_test_tables();
    let out = run(&["schema", "--format", "compact"]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    assert!(stdout.contains("# database:"));
}

#[test]
#[ignore]
fn tc111_i06_live_schema_e2e_relationship_detection() {
    ensure_test_tables();
    let out = run(&["schema", LIVE_DB, "--format", "json"]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON expected");

    let tables = parsed["data"]["tables"].as_array().expect("tables array");
    let table_names: Vec<&str> = tables.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(table_names.contains(&"customers"));
    assert!(table_names.contains(&"orders"));

    let join_paths = parsed["data"]["join_paths"].as_array().expect("join_paths array");
    assert!(!join_paths.is_empty(), "Expected candidate join paths to be detected");

    let has_order_cust_join = join_paths.iter().any(|j| {
        j["from_table"] == "orders"
            && j["to_table"] == "customers"
            && j["from_column"] == "customer_id"
            && j["to_column"] == "customer_id"
    });
    assert!(has_order_cust_join, "Expected join path orders.customer_id -> customers.customer_id");
}
