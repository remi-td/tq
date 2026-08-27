//! Active query inspection command implementation
//!
//! This module provides functionality to inspect the currently executing SQL query
//! and step progress for an active Teradata session using SYSLIB.MonitorSQLText
//! and SYSLIB.MonitorSQLCurrentStep.

use crate::cli::{ActiveQueryArgs, OutputFormat};
use crate::commands::format_helpers::markdown_escape_pipe;
use crate::commands::monitoring_utils::{escape_csv, extract_i64_lenient, extract_trimmed_string};
use crate::db::DatabaseClient;
use crate::error::Result;
use std::io::Write;

/// Session target resolution parameters
#[derive(Debug, Clone)]
pub struct SessionTarget {
    pub session_id: i64,
    pub host_id: i64,
    pub ifp_no: i64,
    pub user_name: String,
}

/// Active query real-time information
#[derive(Debug, Clone)]
pub struct ActiveQueryInfo {
    pub session_id: i64,
    pub user_name: String,
    pub sql_text: String,
    pub current_step: i64,
    pub total_steps: i64,
    pub default_db: String,
}

/// Resolve HostId and IFPNo for a target session from DBC.SessionInfoV
pub fn resolve_session_target(client: &DatabaseClient, session_id: i64) -> Result<Option<SessionTarget>> {
    let sql = format!(
        "SELECT SessionNo, UserName, LogicalHostId, IFPNo \
         FROM DBC.SessionInfoV \
         WHERE SessionNo = {}",
        session_id
    );

    let result = client.execute(&sql)?;
    if let Some(row) = result.rows.first() {
        if row.len() >= 4 {
            let sid = extract_i64_lenient(&row[0]).unwrap_or(session_id);
            let user = extract_trimmed_string(&row[1], "[unknown]");
            let host_id = extract_i64_lenient(&row[2]).unwrap_or(0);
            let ifp_no = extract_i64_lenient(&row[3]).unwrap_or(0);

            return Ok(Some(SessionTarget {
                session_id: sid,
                host_id,
                ifp_no,
                user_name: user,
            }));
        }
    }

    Ok(None)
}

/// Fetch active query info using SYSLIB table functions
pub fn fetch_active_query(client: &DatabaseClient, target: &SessionTarget) -> Result<Option<ActiveQueryInfo>> {
    // 1. Get active SQL text segments
    let sql_text_query = format!(
        "SELECT SQLTxt FROM TABLE (SYSLIB.MonitorSQLText({}, {}, {})) AS t1 ORDER BY SeqNum",
        target.host_id, target.session_id, target.ifp_no
    );

    let sql_result = match client.execute(&sql_text_query) {
        Ok(res) => res,
        Err(e) => {
            let err_str = e.to_string();
            // If session is idle or no query running, MonitorSQLText returns empty or error
            if err_str.contains("3299") || err_str.contains("no request") || err_str.contains("not found") {
                return Ok(None);
            }
            return Err(e);
        }
    };

    if sql_result.rows.is_empty() {
        return Ok(None);
    }

    let mut full_sql = String::new();
    for row in &sql_result.rows {
        if let Some(val) = row.first() {
            let part = extract_trimmed_string(val, "");
            full_sql.push_str(&part);
        }
    }

    if full_sql.trim().is_empty() {
        return Ok(None);
    }

    // 2. Get step progress
    let step_query = format!(
        "SELECT NumOfSteps, CurLvl1StepNo, DefaultDBName \
         FROM TABLE (SYSLIB.MonitorSQLCurrentStep({}, {}, {})) AS t1",
        target.host_id, target.session_id, target.ifp_no
    );

    let (total_steps, current_step, default_db) = match client.execute(&step_query) {
        Ok(res) => {
            if let Some(row) = res.rows.first() {
                let total = extract_i64_lenient(&row[0]).unwrap_or(0);
                let cur = extract_i64_lenient(&row[1]).unwrap_or(0);
                let db = extract_trimmed_string(&row[2], "");
                (total, cur, db)
            } else {
                (0, 0, String::new())
            }
        }
        Err(_) => (0, 0, String::new()),
    };

    Ok(Some(ActiveQueryInfo {
        session_id: target.session_id,
        user_name: target.user_name.clone(),
        sql_text: full_sql.trim().to_string(),
        current_step,
        total_steps,
        default_db,
    }))
}

/// Execute the active-query command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ActiveQueryArgs,
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

    let active_info = match fetch_active_query(client, &target)? {
        Some(info) => info,
        None => {
            writeln!(writer, "Session {} ({}) has no active query running.", target.session_id, target.user_name)?;
            return Ok(());
        }
    };

    match args.format {
        OutputFormat::Table => display_table(&active_info, writer)?,
        OutputFormat::Csv => display_csv(&active_info, writer)?,
        OutputFormat::Json => display_json(&active_info, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(&active_info, writer)?,
    }

    Ok(())
}

/// Execute active-query in REPL mode
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
            write_active_query_error(writer, &e)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    match fetch_active_query(client, &target) {
        Ok(Some(info)) => {
            display_table(&info, writer)?;
        }
        Ok(None) => {
            writeln!(writer, "Session {} ({}) has no active query running.", session_id, target.user_name)?;
        }
        Err(e) => {
            write_active_query_error(writer, &e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

fn write_active_query_error<W: Write>(writer: &mut W, e: &crate::error::TqError) -> Result<()> {
    let error_str = e.to_string().to_lowercase();
    if error_str.contains("privilege")
        || error_str.contains("access")
        || error_str.contains("permission")
        || error_str.contains("3523")
    {
        writeln!(writer, "Error: Insufficient privileges to monitor active queries.")?;
        writeln!(writer, "Required: EXECUTE FUNCTION on SYSLIB.MonitorSQLText")?;
    } else {
        writeln!(writer, "Error fetching active query: {}", e)?;
    }
    Ok(())
}

fn display_table<W: Write>(info: &ActiveQueryInfo, writer: &mut W) -> Result<()> {
    use comfy_table::{presets, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Property", "Value"]);

    table.add_row(vec!["Session ID", &info.session_id.to_string()]);
    table.add_row(vec!["User", &info.user_name]);

    let progress_str = if info.total_steps > 0 {
        format!("Step {} of {}", info.current_step, info.total_steps)
    } else {
        "Parsing / Executing".to_string()
    };
    table.add_row(vec!["Step Progress", &progress_str]);

    if !info.default_db.is_empty() {
        table.add_row(vec!["Default Database", &info.default_db]);
    }

    table.add_row(vec!["Active SQL Text", &info.sql_text]);

    writeln!(writer, "Active Query for Session {}:", info.session_id)?;
    writeln!(writer, "{}", table)?;

    Ok(())
}

fn display_csv<W: Write>(info: &ActiveQueryInfo, writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionID,UserName,CurrentStep,TotalSteps,DefaultDB,SQLText")?;
    writeln!(
        writer,
        "{},{},{},{},{},{}",
        info.session_id,
        escape_csv(&info.user_name),
        info.current_step,
        info.total_steps,
        escape_csv(&info.default_db),
        escape_csv(&info.sql_text)
    )?;
    Ok(())
}

fn display_json<W: Write>(info: &ActiveQueryInfo, writer: &mut W) -> Result<()> {
    let json = serde_json::json!({
        "ok": true,
        "SessionID": info.session_id,
        "UserName": info.user_name,
        "CurrentStep": info.current_step,
        "TotalSteps": info.total_steps,
        "DefaultDB": info.default_db,
        "SQLText": info.sql_text
    });
    writeln!(writer, "{}", serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn display_markdown<W: Write>(info: &ActiveQueryInfo, writer: &mut W) -> Result<()> {
    writeln!(writer, "## Active Query for Session {}", info.session_id)?;
    writeln!(writer)?;
    writeln!(writer, "- **User:** `{}`", markdown_escape_pipe(&info.user_name))?;
    writeln!(writer, "- **Step Progress:** {} / {}", info.current_step, info.total_steps)?;
    if !info.default_db.is_empty() {
        writeln!(writer, "- **Default DB:** `{}`", markdown_escape_pipe(&info.default_db))?;
    }
    writeln!(writer)?;
    writeln!(writer, "```sql\n{}\n```", info.sql_text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_query_info_struct() {
        let info = ActiveQueryInfo {
            session_id: 1234,
            user_name: "DEMO_USER".to_string(),
            sql_text: "SELECT 1".to_string(),
            current_step: 2,
            total_steps: 10,
            default_db: "DEMO_USER".to_string(),
        };
        assert_eq!(info.session_id, 1234);
        assert_eq!(info.user_name, "DEMO_USER");
        assert_eq!(info.sql_text, "SELECT 1");
        assert_eq!(info.current_step, 2);
        assert_eq!(info.total_steps, 10);
    }

    #[test]
    fn test_display_csv() {
        let info = ActiveQueryInfo {
            session_id: 1234,
            user_name: "DEMO_USER".to_string(),
            sql_text: "SELECT * FROM t WHERE a = 'b,c'".to_string(),
            current_step: 1,
            total_steps: 5,
            default_db: "TEST_DB".to_string(),
        };
        let mut out = Vec::new();
        display_csv(&info, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("SessionID,UserName,CurrentStep,TotalSteps,DefaultDB,SQLText"));
        assert!(s.contains("1234,DEMO_USER,1,5,TEST_DB,\"SELECT * FROM t WHERE a = 'b,c'\""));
    }

    #[test]
    fn test_display_json() {
        let info = ActiveQueryInfo {
            session_id: 1234,
            user_name: "DEMO_USER".to_string(),
            sql_text: "SELECT 1".to_string(),
            current_step: 3,
            total_steps: 8,
            default_db: "DEMO_USER".to_string(),
        };
        let mut out = Vec::new();
        display_json(&info, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["SessionID"], 1234);
        assert_eq!(v["CurrentStep"], 3);
        assert_eq!(v["TotalSteps"], 8);
    }
}
