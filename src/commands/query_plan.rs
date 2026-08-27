//! Real-time query execution plan command implementation
//!
//! This module provides functionality to inspect the step-by-step execution plan
//! and real-time step metrics (estimated vs actual row counts, skew, elapsed time)
//! for a currently running Teradata query using SYSLIB.MonitorSQLSteps.

use crate::cli::{OutputFormat, QueryPlanArgs};
use crate::commands::active_query::resolve_session_target;
use crate::commands::format_helpers::markdown_escape_pipe;
use crate::commands::monitoring_utils::{escape_csv, extract_f64_lenient, extract_i64_lenient, extract_trimmed_string};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use std::io::Write;

/// A single step in a real-time query execution plan
#[derive(Debug, Clone)]
pub struct PlanStepInfo {
    pub step_num: i64,
    pub confidence: i64,
    pub est_row_count: f64,
    pub act_row_count: f64,
    pub est_row_count_skew: f64,
    pub act_row_count_skew: f64,
    pub est_elapsed_time: f64,
    pub act_elapsed_time: f64,
    pub step_text: String,
}

impl PlanStepInfo {
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 9 {
            return None;
        }

        let step_num = extract_i64_lenient(&row[0])?;
        let confidence = extract_i64_lenient(&row[1]).unwrap_or(0);
        let est_row_count = extract_f64_lenient(&row[2]).unwrap_or(0.0);
        let act_row_count = extract_f64_lenient(&row[3]).unwrap_or(-1.0);
        let est_row_count_skew = extract_f64_lenient(&row[4]).unwrap_or(0.0);
        let act_row_count_skew = extract_f64_lenient(&row[5]).unwrap_or(-1.0);
        let est_elapsed_time = extract_f64_lenient(&row[6]).unwrap_or(0.0);
        let act_elapsed_time = extract_f64_lenient(&row[7]).unwrap_or(-1.0);
        let step_text = extract_trimmed_string(&row[8], "");

        Some(Self {
            step_num,
            confidence,
            est_row_count,
            act_row_count,
            est_row_count_skew,
            act_row_count_skew,
            est_elapsed_time,
            act_elapsed_time,
            step_text,
        })
    }

    pub fn is_completed(&self) -> bool {
        self.act_row_count >= 0.0 || self.act_elapsed_time >= 0.0
    }
}

/// Fetch real-time query plan steps from SYSLIB.MonitorSQLSteps
pub fn fetch_query_plan_steps(client: &DatabaseClient, host_id: i64, session_id: i64, ifp_no: i64) -> Result<Vec<PlanStepInfo>> {
    let sql = format!(
        "SELECT StepNum, Confidence, EstRowCount, ActRowCount, EstRowCountSkew, ActRowCountSkew, EstElapsedTime, ActElapsedTime, SQLStep \
         FROM TABLE (SYSLIB.MonitorSQLSteps({}, {}, {})) AS t1 \
         ORDER BY StepNum",
        host_id, session_id, ifp_no
    );

    let result = match client.execute(&sql) {
        Ok(res) => res,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("3299") || err_str.contains("no request") || err_str.contains("not found") {
                return Ok(Vec::new());
            }
            return Err(e);
        }
    };

    let steps = result
        .rows
        .iter()
        .filter_map(|row| PlanStepInfo::from_row(row))
        .collect();

    Ok(steps)
}

/// Execute query-plan command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryPlanArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let target = match resolve_session_target(client, args.session_id)? {
        Some(t) => t,
        None => {
            writeln!(writer, "Session {} not found on system.", args.session_id)?;
            return Ok(());
        }
    };

    let steps = fetch_query_plan_steps(client, target.host_id, target.session_id, target.ifp_no)?;

    if steps.is_empty() {
        writeln!(writer, "Session {} ({}) has no active query plan steps.", target.session_id, target.user_name)?;
        return Ok(());
    }

    match args.format {
        OutputFormat::Table => display_table(&steps, args.session_id, writer)?,
        OutputFormat::Csv => display_csv(&steps, writer)?,
        OutputFormat::Json => display_json(&steps, args.session_id, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(&steps, args.session_id, writer)?,
    }

    Ok(())
}

/// Execute query-plan in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    session_id: i64,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let target = match resolve_session_target(client, session_id) {
        Ok(Some(t)) => t,
        Ok(None) => {
            writeln!(writer, "Session {} not found on system.", session_id)?;
            writeln!(writer)?;
            return Ok(());
        }
        Err(e) => {
            write_plan_error(writer, &e)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    match fetch_query_plan_steps(client, target.host_id, target.session_id, target.ifp_no) {
        Ok(steps) => {
            if steps.is_empty() {
                writeln!(writer, "Session {} ({}) has no active query plan steps.", session_id, target.user_name)?;
            } else {
                display_table(&steps, session_id, writer)?;
            }
        }
        Err(e) => {
            write_plan_error(writer, &e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

fn write_plan_error<W: Write>(writer: &mut W, e: &crate::error::TqError) -> Result<()> {
    let error_str = e.to_string().to_lowercase();
    if error_str.contains("privilege")
        || error_str.contains("access")
        || error_str.contains("permission")
        || error_str.contains("3523")
    {
        writeln!(writer, "Error: Insufficient privileges to monitor query plan steps.")?;
        writeln!(writer, "Required: EXECUTE FUNCTION on SYSLIB.MonitorSQLSteps")?;
    } else {
        writeln!(writer, "Error fetching query plan steps: {}", e)?;
    }
    Ok(())
}

fn format_count(val: f64) -> String {
    if val < 0.0 {
        "[pending]".to_string()
    } else {
        format!("{:.0}", val)
    }
}

fn format_time(val: f64) -> String {
    if val < 0.0 {
        "[pending]".to_string()
    } else {
        format!("{:.3}s", val)
    }
}

fn display_table<W: Write>(steps: &[PlanStepInfo], session_id: i64, writer: &mut W) -> Result<()> {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    writeln!(writer, "Real-Time Query Plan Steps for Session {}:", session_id)?;

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "Step",
        "Conf",
        "Est Rows",
        "Act Rows",
        "Est Time",
        "Act Time",
        "Step Description",
    ]);

    for step in steps {
        table.add_row(vec![
            Cell::new(step.step_num).set_alignment(CellAlignment::Right),
            Cell::new(step.confidence).set_alignment(CellAlignment::Right),
            Cell::new(format_count(step.est_row_count)).set_alignment(CellAlignment::Right),
            Cell::new(format_count(step.act_row_count)).set_alignment(CellAlignment::Right),
            Cell::new(format_time(step.est_elapsed_time)).set_alignment(CellAlignment::Right),
            Cell::new(format_time(step.act_elapsed_time)).set_alignment(CellAlignment::Right),
            Cell::new(&step.step_text),
        ]);
    }

    writeln!(writer, "{}", table)?;
    let completed_cnt = steps.iter().filter(|s| s.is_completed()).count();
    writeln!(writer, "{} total step(s) ({} completed)", steps.len(), completed_cnt)?;

    Ok(())
}

fn display_csv<W: Write>(steps: &[PlanStepInfo], writer: &mut W) -> Result<()> {
    writeln!(
        writer,
        "StepNum,Confidence,EstRowCount,ActRowCount,EstRowCountSkew,ActRowCountSkew,EstElapsedTime,ActElapsedTime,SQLStep"
    )?;

    for step in steps {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            step.step_num,
            step.confidence,
            step.est_row_count,
            step.act_row_count,
            step.est_row_count_skew,
            step.act_row_count_skew,
            step.est_elapsed_time,
            step.act_elapsed_time,
            escape_csv(&step.step_text)
        )?;
    }

    Ok(())
}

fn display_json<W: Write>(steps: &[PlanStepInfo], session_id: i64, writer: &mut W) -> Result<()> {
    let json_steps: Vec<serde_json::Value> = steps
        .iter()
        .map(|s| {
            serde_json::json!({
                "StepNum": s.step_num,
                "Confidence": s.confidence,
                "EstRowCount": s.est_row_count,
                "ActRowCount": if s.act_row_count < 0.0 { serde_json::Value::Null } else { serde_json::json!(s.act_row_count) },
                "EstRowCountSkew": s.est_row_count_skew,
                "ActRowCountSkew": if s.act_row_count_skew < 0.0 { serde_json::Value::Null } else { serde_json::json!(s.act_row_count_skew) },
                "EstElapsedTime": s.est_elapsed_time,
                "ActElapsedTime": if s.act_elapsed_time < 0.0 { serde_json::Value::Null } else { serde_json::json!(s.act_elapsed_time) },
                "Completed": s.is_completed(),
                "SQLStep": s.step_text
            })
        })
        .collect();

    let json = serde_json::json!({
        "ok": true,
        "SessionID": session_id,
        "TotalSteps": steps.len(),
        "CompletedSteps": steps.iter().filter(|s| s.is_completed()).count(),
        "Steps": json_steps
    });

    writeln!(writer, "{}", serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn display_markdown<W: Write>(steps: &[PlanStepInfo], session_id: i64, writer: &mut W) -> Result<()> {
    writeln!(writer, "## Query Plan Steps for Session {}", session_id)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "| Step | Conf | Est Rows | Act Rows | Est Time | Act Time | Step Description |"
    )?;
    writeln!(writer, "| ---: | ---: | ---: | ---: | ---: | ---: | :--- |")?;

    for step in steps {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} | {} | {} |",
            step.step_num,
            step.confidence,
            format_count(step.est_row_count),
            format_count(step.act_row_count),
            format_time(step.est_elapsed_time),
            format_time(step.act_elapsed_time),
            markdown_escape_pipe(&step.step_text)
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_step_from_row() {
        let row = vec![
            Value::Integer(1),
            Value::Integer(3),
            Value::Decimal(100.0),
            Value::Decimal(95.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.05),
            Value::Decimal(0.04),
            Value::String("First step description".to_string()),
        ];

        let step = PlanStepInfo::from_row(&row);
        assert!(step.is_some());
        let step = step.unwrap();
        assert_eq!(step.step_num, 1);
        assert_eq!(step.confidence, 3);
        assert_eq!(step.est_row_count, 100.0);
        assert_eq!(step.act_row_count, 95.0);
        assert!(step.is_completed());
    }

    #[test]
    fn test_plan_step_pending() {
        let row = vec![
            Value::Integer(2),
            Value::Integer(0),
            Value::Decimal(5000.0),
            Value::Decimal(-1.0),
            Value::Decimal(0.0),
            Value::Decimal(-1.0),
            Value::Decimal(10.0),
            Value::Decimal(-1.0),
            Value::String("Pending step description".to_string()),
        ];

        let step = PlanStepInfo::from_row(&row).unwrap();
        assert_eq!(step.step_num, 2);
        assert_eq!(step.act_row_count, -1.0);
        assert!(!step.is_completed());
    }

    #[test]
    fn test_display_csv() {
        let steps = vec![PlanStepInfo {
            step_num: 1,
            confidence: 3,
            est_row_count: 100.0,
            act_row_count: 95.0,
            est_row_count_skew: 0.0,
            act_row_count_skew: 0.0,
            est_elapsed_time: 0.1,
            act_elapsed_time: 0.08,
            step_text: "RETRIEVE step".to_string(),
        }];

        let mut out = Vec::new();
        display_csv(&steps, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("StepNum,Confidence,EstRowCount,ActRowCount"));
        assert!(s.contains("1,3,100,95,0,0,0.1,0.08,RETRIEVE step"));
    }
}
