//! Explain plan command implementation
//!
//! This module provides functionality to display Teradata execution plans
//! for SQL statements using the EXPLAIN prefix.
//!
//! Sprint 50: Initial implementation (Issue #24)

use crate::cli::{ExplainArgs, OutputFormat};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use super::monitoring_utils::escape_csv;
use std::io::Write;

/// A single step in an explain plan
#[derive(Debug, Clone)]
pub struct ExplainStep {
    /// Step number (1-based)
    pub step_no: usize,
    /// Explain text for this step
    pub text: String,
}

/// Execute the explain command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ExplainArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let steps = run_explain(client, &args.sql)?;

    match args.format {
        OutputFormat::Table => display_table(&steps, &args.sql, writer)?,
        OutputFormat::Csv => display_csv(&steps, writer)?,
        OutputFormat::Json => display_json(&steps, &args.sql, writer)?,
    }

    Ok(())
}

/// Execute explain in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    if sql.trim().is_empty() {
        writeln!(writer, "Usage: /explain <sql_statement>")?;
        writeln!(writer)?;
        writeln!(writer, "Show the execution plan for a SQL statement.")?;
        writeln!(writer)?;
        writeln!(writer, "Examples:")?;
        writeln!(writer, "  /explain SELECT * FROM employees")?;
        writeln!(writer, "  /explain SELECT COUNT(*) FROM orders WHERE status = 'active'")?;
        writeln!(writer)?;
        return Ok(());
    }

    match run_explain(client, sql) {
        Ok(steps) => {
            if steps.is_empty() {
                writeln!(writer, "Explain plan:")?;
                writeln!(writer, "(no explain output returned)")?;
            } else {
                writeln!(writer, "Explain plan for: {}", truncate_sql(sql, 80))?;
                writeln!(writer, "{}", "─".repeat(60))?;
                for step in &steps {
                    writeln!(writer, "{}", step.text)?;
                }
                writeln!(writer, "{}", "─".repeat(60))?;
                writeln!(writer, "{} step(s)", steps.len())?;
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();
            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
            {
                writeln!(writer, "Error: Insufficient privileges to run EXPLAIN.")?;
                writeln!(writer)?;
                writeln!(writer, "You need SELECT privileges on the referenced objects.")?;
            } else if error_str.contains("syntax") {
                writeln!(writer, "Error: SQL syntax error in statement.")?;
                writeln!(writer)?;
                writeln!(writer, "The EXPLAIN plan could not be generated due to a syntax error:")?;
                writeln!(writer, "  {}", e)?;
            } else {
                writeln!(writer, "Error generating explain plan: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Run EXPLAIN on a SQL statement and parse the results
fn run_explain(client: &DatabaseClient, sql: &str) -> Result<Vec<ExplainStep>> {
    // Strip leading EXPLAIN if user already added it
    let clean_sql = sql.trim();
    let explain_sql = if clean_sql.to_uppercase().starts_with("EXPLAIN") {
        clean_sql.to_string()
    } else {
        format!("EXPLAIN {}", clean_sql)
    };

    let result = client.execute(&explain_sql)?;

    let mut steps = Vec::new();
    for (i, row) in result.rows.iter().enumerate() {
        let text = if let Some(value) = row.first() {
            match value {
                Value::String(s) => s.trim_end().to_string(),
                Value::Null => String::new(),
                other => other.display().trim_end().to_string(),
            }
        } else {
            String::new()
        };

        steps.push(ExplainStep {
            step_no: i + 1,
            text,
        });
    }

    Ok(steps)
}

/// Truncate SQL for display purposes
fn truncate_sql(sql: &str, max_len: usize) -> String {
    let trimmed = sql.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..max_len.saturating_sub(3)])
    }
}

/// Display explain steps in table format
fn display_table<W: Write>(steps: &[ExplainStep], sql: &str, writer: &mut W) -> Result<()> {
    if steps.is_empty() {
        writeln!(writer, "Explain plan:")?;
        writeln!(writer, "(no explain output returned)")?;
        return Ok(());
    }

    writeln!(writer, "Explain plan for: {}", truncate_sql(sql, 80))?;
    writeln!(writer, "{}", "─".repeat(60))?;
    for step in steps {
        writeln!(writer, "{}", step.text)?;
    }
    writeln!(writer, "{}", "─".repeat(60))?;
    writeln!(writer, "{} step(s)", steps.len())?;

    Ok(())
}

/// Display explain steps in CSV format
fn display_csv<W: Write>(steps: &[ExplainStep], writer: &mut W) -> Result<()> {
    writeln!(writer, "StepNo,Text")?;
    for step in steps {
        writeln!(writer, "{},{}", step.step_no, escape_csv(&step.text))?;
    }
    Ok(())
}

/// Display explain steps in JSON format
fn display_json<W: Write>(steps: &[ExplainStep], sql: &str, writer: &mut W) -> Result<()> {
    let json_steps: Vec<serde_json::Value> = steps
        .iter()
        .map(|step| {
            serde_json::json!({
                "StepNo": step.step_no,
                "Text": step.text
            })
        })
        .collect();

    let json = serde_json::json!({
        "SQL": sql,
        "Steps": json_steps,
        "StepCount": steps.len()
    });
    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_sql_short() {
        assert_eq!(truncate_sql("SELECT 1", 80), "SELECT 1");
    }

    #[test]
    fn test_truncate_sql_long() {
        let long_sql = "SELECT very_long_column_name_1, very_long_column_name_2, very_long_column_name_3 FROM very_long_table_name WHERE condition = 'value'";
        let result = truncate_sql(long_sql, 40);
        assert!(result.len() <= 40);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_sql_exact() {
        let sql = "SELECT 1234567890"; // 18 chars
        assert_eq!(truncate_sql(sql, 18), "SELECT 1234567890");
    }

    #[test]
    fn test_truncate_sql_trims() {
        assert_eq!(truncate_sql("  SELECT 1  ", 80), "SELECT 1");
    }

    #[test]
    fn test_explain_step() {
        let step = ExplainStep {
            step_no: 1,
            text: "1) First, we lock a distinct TESTDB.\"pseudo table\" for read on a RowHash to prevent global deadlock for TESTDB.employees.".to_string(),
        };
        assert_eq!(step.step_no, 1);
        assert!(step.text.contains("First"));
    }

    #[test]
    fn test_display_table_empty() {
        let mut output = Vec::new();
        display_table(&[], "SELECT 1", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("no explain output returned"));
    }

    #[test]
    fn test_display_table_with_steps() {
        let steps = vec![
            ExplainStep {
                step_no: 1,
                text: "1) First, we lock a distinct table.".to_string(),
            },
            ExplainStep {
                step_no: 2,
                text: "2) Next, we do an all-AMPs RETRIEVE step.".to_string(),
            },
        ];
        let mut output = Vec::new();
        display_table(&steps, "SELECT * FROM t", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Explain plan for: SELECT * FROM t"));
        assert!(s.contains("First, we lock"));
        assert!(s.contains("all-AMPs RETRIEVE"));
        assert!(s.contains("2 step(s)"));
    }

    #[test]
    fn test_display_csv() {
        let steps = vec![
            ExplainStep {
                step_no: 1,
                text: "Step one text".to_string(),
            },
            ExplainStep {
                step_no: 2,
                text: "Step two, with comma".to_string(),
            },
        ];
        let mut output = Vec::new();
        display_csv(&steps, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("StepNo,Text"));
        assert!(s.contains("1,Step one text"));
        // Comma in text should be escaped
        assert!(s.contains("\"Step two, with comma\""));
    }

    #[test]
    fn test_display_json() {
        let steps = vec![ExplainStep {
            step_no: 1,
            text: "1) First step".to_string(),
        }];
        let mut output = Vec::new();
        display_json(&steps, "SELECT 1", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["SQL"], "SELECT 1");
        assert_eq!(json["StepCount"], 1);
        assert_eq!(json["Steps"][0]["StepNo"], 1);
        assert_eq!(json["Steps"][0]["Text"], "1) First step");
    }

    #[test]
    fn test_display_json_empty() {
        let mut output = Vec::new();
        display_json(&[], "SELECT 1", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["StepCount"], 0);
        assert!(json["Steps"].as_array().unwrap().is_empty());
    }
}
