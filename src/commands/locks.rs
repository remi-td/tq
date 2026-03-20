//! Lock information command implementation
//!
//! This module provides functionality to display current lock contention,
//! blocking sessions, and lock details to help DBAs diagnose contention issues.
//!
//! Sprint 38: Initial implementation

use crate::cli::{OutputFormat, LocksArgs};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use super::monitoring_utils::{escape_csv, extract_integer, extract_trimmed_string};
use std::collections::HashMap;
use std::io::Write;

/// SQL query to retrieve lock information from DBC.LockInfoV
///
/// Returns lock details including the locked object, lock type, lock mode,
/// locking session, and any waiting sessions.
const LOCKS_SQL: &str = r#"
SELECT
    TRIM(DatabaseName) || '.' || TRIM(TableName) AS LockedObject,
    CASE LockType
        WHEN 'T' THEN 'Table'
        WHEN 'R' THEN 'Row Hash'
        WHEN 'D' THEN 'Database'
        WHEN 'V' THEN 'View'
        ELSE TRIM(LockType)
    END AS LockTypeName,
    CASE ModeGranted
        WHEN 'A' THEN 'ACCESS'
        WHEN 'R' THEN 'READ'
        WHEN 'W' THEN 'WRITE'
        WHEN 'E' THEN 'EXCLUSIVE'
        ELSE TRIM(ModeGranted)
    END AS LockModeName,
    GrantorSessionId,
    LockerSessionId,
    CASE ModeWanting
        WHEN ' ' THEN NULL
        WHEN '' THEN NULL
        ELSE ModeWanting
    END AS ModeWanting
FROM DBC.LockInfoV
ORDER BY LockerSessionId, LockedObject
"#;

/// Lock information extracted from DBC.LockInfoV
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// Locked object name (database.table)
    pub locked_object: String,
    /// Lock type (Table, Row Hash, Database, View)
    pub lock_type: String,
    /// Lock mode (ACCESS, READ, WRITE, EXCLUSIVE)
    pub lock_mode: String,
    /// Session ID that holds the lock
    pub locking_session: i64,
    /// Session ID of the grantor (the session that granted/holds the lock)
    pub grantor_session: i64,
    /// Whether this row represents a waiting session
    pub is_waiting: bool,
}

impl LockInfo {
    /// Create LockInfo from a query result row
    ///
    /// Returns None if required fields are missing or cannot be parsed.
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 6 {
            return None;
        }

        let locked_object = extract_trimmed_string(&row[0], "[NULL]");
        let lock_type = extract_trimmed_string(&row[1], "[NULL]");
        let lock_mode = extract_trimmed_string(&row[2], "[NULL]");

        let grantor_session = extract_integer(&row[3])?;
        let locking_session = extract_integer(&row[4])?;

        // If ModeWanting is not NULL, this session is waiting for a lock
        let is_waiting = !matches!(&row[5], Value::Null);

        Some(Self {
            locked_object,
            lock_type,
            lock_mode,
            locking_session,
            grantor_session,
            is_waiting,
        })
    }
}

/// A consolidated lock display row for output
#[derive(Debug, Clone)]
pub struct LockDisplayRow {
    /// Locked object name (database.table)
    pub locked_object: String,
    /// Lock type (Table, Row Hash, Database, View)
    pub lock_type: String,
    /// Lock mode (ACCESS, READ, WRITE, EXCLUSIVE)
    pub lock_mode: String,
    /// Session ID that holds the lock
    pub locking_session: i64,
    /// Session IDs waiting for this lock
    pub waiting_sessions: Vec<i64>,
}

/// Blocking chain information
#[derive(Debug, Clone)]
pub struct BlockingChain {
    /// Session ID that is blocking others
    pub blocker_session: i64,
    /// Sessions that are blocked by this session
    pub blocked_sessions: Vec<i64>,
}

/// Build display rows from raw lock info
///
/// Aggregates raw lock rows into consolidated display rows where
/// each display row represents one lock with all its waiting sessions.
pub fn build_display_rows(locks: &[LockInfo]) -> Vec<LockDisplayRow> {
    // Group locks by (locked_object, locking_session that holds the lock)
    // A lock holder is one where is_waiting is false
    let mut display_map: HashMap<(String, i64), LockDisplayRow> = HashMap::new();

    // First pass: add lock holders
    for lock in locks {
        if !lock.is_waiting {
            let key = (lock.locked_object.clone(), lock.locking_session);
            display_map.entry(key).or_insert_with(|| LockDisplayRow {
                locked_object: lock.locked_object.clone(),
                lock_type: lock.lock_type.clone(),
                lock_mode: lock.lock_mode.clone(),
                locking_session: lock.locking_session,
                waiting_sessions: Vec::new(),
            });
        }
    }

    // Second pass: add waiters to their corresponding lock holders
    for lock in locks {
        if lock.is_waiting {
            // The waiter's grantor_session is the session holding the lock
            let key = (lock.locked_object.clone(), lock.grantor_session);
            if let Some(display_row) = display_map.get_mut(&key) {
                if !display_row
                    .waiting_sessions
                    .contains(&lock.locking_session)
                {
                    display_row.waiting_sessions.push(lock.locking_session);
                }
            }
        }
    }

    let mut rows: Vec<LockDisplayRow> = display_map.into_values().collect();
    rows.sort_by(|a, b| {
        a.locking_session
            .cmp(&b.locking_session)
            .then(a.locked_object.cmp(&b.locked_object))
    });

    // Sort waiting sessions within each row
    for row in &mut rows {
        row.waiting_sessions.sort();
    }

    rows
}

/// Identify blocking chains from display rows
///
/// A blocking chain exists when one or more sessions are waiting
/// for a lock held by another session.
pub fn identify_blocking_chains(display_rows: &[LockDisplayRow]) -> Vec<BlockingChain> {
    // Aggregate all blocked sessions per blocking session
    let mut blocker_map: HashMap<i64, Vec<i64>> = HashMap::new();

    for row in display_rows {
        if !row.waiting_sessions.is_empty() {
            let entry = blocker_map.entry(row.locking_session).or_default();
            for &waiter in &row.waiting_sessions {
                if !entry.contains(&waiter) {
                    entry.push(waiter);
                }
            }
        }
    }

    let mut chains: Vec<BlockingChain> = blocker_map
        .into_iter()
        .map(|(blocker_session, mut blocked_sessions)| {
            blocked_sessions.sort();
            blocked_sessions.dedup();
            BlockingChain {
                blocker_session,
                blocked_sessions,
            }
        })
        .collect();

    chains.sort_by_key(|c| c.blocker_session);
    chains
}

/// Format a list of waiting sessions for display
fn format_waiting_sessions(sessions: &[i64]) -> String {
    if sessions.is_empty() {
        "(none)".to_string()
    } else {
        sessions
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Execute the locks command and write results (batch mode)
///
/// # Arguments
/// * `client` - Database client for executing queries
/// * `args` - Command arguments (format, output file)
/// * `writer` - Output writer
/// * `_use_color` - Whether to use color output
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &LocksArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let result = client.execute(LOCKS_SQL)?;

    let locks: Vec<LockInfo> = result
        .rows
        .iter()
        .filter_map(|row| LockInfo::from_row(row))
        .collect();

    let display_rows = build_display_rows(&locks);
    let chains = identify_blocking_chains(&display_rows);

    match args.format {
        OutputFormat::Table => {
            display_table(&display_rows, &chains, writer)?;
            writeln!(writer)?;
            write_summary_footer(&display_rows, &chains, result.execution_time, writer)?;
        }
        OutputFormat::Csv => display_csv(&display_rows, writer)?,
        OutputFormat::Json => display_json(&display_rows, writer)?,
    }

    Ok(())
}

/// Execute locks query and display for REPL mode
///
/// Displays lock information with blocking chains and error handling.
pub fn execute_for_repl<W: Write>(client: &DatabaseClient, writer: &mut W) -> Result<()> {
    writeln!(writer)?;

    match client.execute(LOCKS_SQL) {
        Ok(result) => {
            let locks: Vec<LockInfo> = result
                .rows
                .iter()
                .filter_map(|row| LockInfo::from_row(row))
                .collect();

            let display_rows = build_display_rows(&locks);
            let chains = identify_blocking_chains(&display_rows);

            if display_rows.is_empty() {
                writeln!(writer, "Lock Information:")?;
                writeln!(writer, "No locks currently held.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "(Query time: {:.3}s)",
                    result.execution_time.as_secs_f64()
                )?;
            } else {
                display_repl_table(&display_rows, writer)?;
                writeln!(writer)?;
                write_summary_footer(&display_rows, &chains, result.execution_time, writer)?;

                // Display blocking chains if any
                if !chains.is_empty() {
                    writeln!(writer)?;
                    writeln!(writer, "Blocking Chain:")?;
                    for chain in &chains {
                        let blocked_str = chain
                            .blocked_sessions
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(
                            writer,
                            "  Session {} blocks sessions: {}",
                            chain.blocker_session, blocked_str
                        )?;
                    }
                }
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();

            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Unable to retrieve lock information.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "This command requires SELECT access to DBC lock views."
                )?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(
                    writer,
                    "  GRANT SELECT ON DBC.LockInfoV TO <your_username>;"
                )?;
            } else if error_str.contains("lockinfov")
                && (error_str.contains("not found") || error_str.contains("does not exist"))
            {
                writeln!(writer, "Error: Lock information view not available.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "DBC.LockInfoV is not accessible on this system."
                )?;
            } else {
                writeln!(writer, "Error retrieving lock information: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Write the summary footer line
fn write_summary_footer<W: Write>(
    display_rows: &[LockDisplayRow],
    chains: &[BlockingChain],
    execution_time: std::time::Duration,
    writer: &mut W,
) -> Result<()> {
    let lock_count = display_rows.len();
    let chain_count = chains.len();

    if chain_count > 0 {
        writeln!(
            writer,
            "{} lock(s) found - {} blocking chain(s) detected (Query time: {:.3}s)",
            lock_count,
            chain_count,
            execution_time.as_secs_f64()
        )?;
    } else {
        writeln!(
            writer,
            "{} lock(s) found (Query time: {:.3}s)",
            lock_count,
            execution_time.as_secs_f64()
        )?;
    }

    Ok(())
}

/// Display locks using a comfy_table for REPL mode
fn display_repl_table<W: Write>(rows: &[LockDisplayRow], writer: &mut W) -> Result<()> {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "Locked Object",
        "Lock Type",
        "Lock Mode",
        "Locking Sess",
        "Waiting Sess",
    ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(&row.locked_object),
            Cell::new(&row.lock_type),
            Cell::new(&row.lock_mode),
            Cell::new(row.locking_session).set_alignment(CellAlignment::Right),
            Cell::new(format_waiting_sessions(&row.waiting_sessions)),
        ]);
    }

    writeln!(writer, "Lock Information:")?;
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Display locks in table format (batch mode)
fn display_table<W: Write>(
    rows: &[LockDisplayRow],
    chains: &[BlockingChain],
    writer: &mut W,
) -> Result<()> {
    if rows.is_empty() {
        writeln!(writer, "Lock Information:")?;
        writeln!(writer, "No locks currently held.")?;
        return Ok(());
    }

    display_repl_table(rows, writer)?;

    // Display blocking chains if any
    if !chains.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Blocking Chain:")?;
        for chain in chains {
            let blocked_str = chain
                .blocked_sessions
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                writer,
                "  Session {} blocks sessions: {}",
                chain.blocker_session, blocked_str
            )?;
        }
    }

    Ok(())
}

/// Format waiting sessions for CSV output
///
/// Returns empty string for no waiters (not "(none)") per CSV conventions.
fn format_waiting_sessions_csv(sessions: &[i64]) -> String {
    if sessions.is_empty() {
        String::new()
    } else {
        sessions
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Display locks in CSV format
fn display_csv<W: Write>(rows: &[LockDisplayRow], writer: &mut W) -> Result<()> {
    writeln!(
        writer,
        "Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess"
    )?;

    for row in rows {
        let waiting = format_waiting_sessions_csv(&row.waiting_sessions);
        writeln!(
            writer,
            "{},{},{},{},{}",
            escape_csv(&row.locked_object),
            escape_csv(&row.lock_type),
            escape_csv(&row.lock_mode),
            row.locking_session,
            escape_csv(&waiting)
        )?;
    }

    Ok(())
}

/// Display locks in JSON format
fn display_json<W: Write>(rows: &[LockDisplayRow], writer: &mut W) -> Result<()> {
    let json_rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "Locked Object": row.locked_object,
                "Lock Type": row.lock_type,
                "Lock Mode": row.lock_mode,
                "Locking Sess": row.locking_session,
                "Waiting Sess": row.waiting_sessions,
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&json_rows)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lock_row(
        object: &str,
        lock_type: &str,
        mode: &str,
        grantor: i64,
        locker: i64,
        mode_wanting: Option<&str>,
    ) -> Vec<Value> {
        vec![
            Value::String(object.to_string()),
            Value::String(lock_type.to_string()),
            Value::String(mode.to_string()),
            Value::Integer(grantor),
            Value::Integer(locker),
            match mode_wanting {
                Some(mw) => Value::String(mw.to_string()),
                None => Value::Null,
            },
        ]
    }

    #[test]
    fn test_lock_info_from_row_holder() {
        let row = make_lock_row("PROD.orders", "Table", "WRITE", 1023, 1023, None);
        let lock = LockInfo::from_row(&row);
        assert!(lock.is_some());

        let lock = lock.unwrap();
        assert_eq!(lock.locked_object, "PROD.orders");
        assert_eq!(lock.lock_type, "Table");
        assert_eq!(lock.lock_mode, "WRITE");
        assert_eq!(lock.locking_session, 1023);
        assert_eq!(lock.grantor_session, 1023);
        assert!(!lock.is_waiting);
    }

    #[test]
    fn test_lock_info_from_row_waiter() {
        let row = make_lock_row("PROD.orders", "Table", "WRITE", 1023, 1045, Some("R"));
        let lock = LockInfo::from_row(&row);
        assert!(lock.is_some());

        let lock = lock.unwrap();
        assert_eq!(lock.locked_object, "PROD.orders");
        assert_eq!(lock.locking_session, 1045);
        assert_eq!(lock.grantor_session, 1023);
        assert!(lock.is_waiting);
    }

    #[test]
    fn test_lock_info_from_row_insufficient_columns() {
        let row = vec![Value::String("PROD.orders".to_string())];
        let lock = LockInfo::from_row(&row);
        assert!(lock.is_none());
    }

    #[test]
    fn test_lock_info_from_row_null_session() {
        let row = vec![
            Value::String("PROD.orders".to_string()),
            Value::String("Table".to_string()),
            Value::String("WRITE".to_string()),
            Value::Null,
            Value::Integer(1023),
            Value::Null,
        ];
        let lock = LockInfo::from_row(&row);
        // grantor_session is NULL -> returns None
        assert!(lock.is_none());
    }

    #[test]
    fn test_build_display_rows_no_locks() {
        let locks: Vec<LockInfo> = Vec::new();
        let rows = build_display_rows(&locks);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_build_display_rows_single_holder_no_waiters() {
        let locks = vec![LockInfo {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "WRITE".to_string(),
            locking_session: 1023,
            grantor_session: 1023,
            is_waiting: false,
        }];

        let rows = build_display_rows(&locks);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].locked_object, "PROD.orders");
        assert_eq!(rows[0].locking_session, 1023);
        assert!(rows[0].waiting_sessions.is_empty());
    }

    #[test]
    fn test_build_display_rows_with_waiters() {
        let locks = vec![
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                grantor_session: 1023,
                is_waiting: false,
            },
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1045,
                grantor_session: 1023,
                is_waiting: true,
            },
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1067,
                grantor_session: 1023,
                is_waiting: true,
            },
        ];

        let rows = build_display_rows(&locks);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].locking_session, 1023);
        assert_eq!(rows[0].waiting_sessions, vec![1045, 1067]);
    }

    #[test]
    fn test_build_display_rows_multiple_locks() {
        let locks = vec![
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                grantor_session: 1023,
                is_waiting: false,
            },
            LockInfo {
                locked_object: "PROD.customers".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "READ".to_string(),
                locking_session: 1078,
                grantor_session: 1078,
                is_waiting: false,
            },
        ];

        let rows = build_display_rows(&locks);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_identify_blocking_chains_no_waiters() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "READ".to_string(),
            locking_session: 1078,
            waiting_sessions: Vec::new(),
        }];

        let chains = identify_blocking_chains(&rows);
        assert!(chains.is_empty());
    }

    #[test]
    fn test_identify_blocking_chains_single_chain() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "WRITE".to_string(),
            locking_session: 1023,
            waiting_sessions: vec![1045, 1067],
        }];

        let chains = identify_blocking_chains(&rows);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].blocker_session, 1023);
        assert_eq!(chains[0].blocked_sessions, vec![1045, 1067]);
    }

    #[test]
    fn test_identify_blocking_chains_multiple_chains() {
        let rows = vec![
            LockDisplayRow {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                waiting_sessions: vec![1045, 1067],
            },
            LockDisplayRow {
                locked_object: "PROD.customers".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "EXCLUSIVE".to_string(),
                locking_session: 1089,
                waiting_sessions: vec![1092],
            },
        ];

        let chains = identify_blocking_chains(&rows);
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].blocker_session, 1023);
        assert_eq!(chains[0].blocked_sessions, vec![1045, 1067]);
        assert_eq!(chains[1].blocker_session, 1089);
        assert_eq!(chains[1].blocked_sessions, vec![1092]);
    }

    #[test]
    fn test_identify_blocking_chains_same_blocker_multiple_locks() {
        // Session 1023 holds locks on two tables, blocking different sessions on each
        let rows = vec![
            LockDisplayRow {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                waiting_sessions: vec![1045],
            },
            LockDisplayRow {
                locked_object: "PROD.customers".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "EXCLUSIVE".to_string(),
                locking_session: 1023,
                waiting_sessions: vec![1051],
            },
        ];

        let chains = identify_blocking_chains(&rows);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].blocker_session, 1023);
        assert_eq!(chains[0].blocked_sessions, vec![1045, 1051]);
    }

    #[test]
    fn test_format_waiting_sessions_empty() {
        assert_eq!(format_waiting_sessions(&[]), "(none)");
    }

    #[test]
    fn test_format_waiting_sessions_single() {
        assert_eq!(format_waiting_sessions(&[1045]), "1045");
    }

    #[test]
    fn test_format_waiting_sessions_multiple() {
        assert_eq!(format_waiting_sessions(&[1045, 1067]), "1045, 1067");
    }

    #[test]
    fn test_display_table_no_locks() {
        let rows: Vec<LockDisplayRow> = Vec::new();
        let chains: Vec<BlockingChain> = Vec::new();

        let mut output = Vec::new();
        display_table(&rows, &chains, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("Lock Information:"));
        assert!(output_str.contains("No locks currently held."));
    }

    #[test]
    fn test_display_table_with_locks() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "WRITE".to_string(),
            locking_session: 1023,
            waiting_sessions: vec![1045, 1067],
        }];
        let chains = vec![BlockingChain {
            blocker_session: 1023,
            blocked_sessions: vec![1045, 1067],
        }];

        let mut output = Vec::new();
        display_table(&rows, &chains, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("Lock Information:"));
        assert!(output_str.contains("PROD.orders"));
        assert!(output_str.contains("WRITE"));
        assert!(output_str.contains("1023"));
        assert!(output_str.contains("1045, 1067"));
        assert!(output_str.contains("Blocking Chain:"));
        assert!(output_str.contains("Session 1023 blocks sessions: 1045, 1067"));
    }

    #[test]
    fn test_display_table_no_blocking_chains_section() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "READ".to_string(),
            locking_session: 1078,
            waiting_sessions: Vec::new(),
        }];
        let chains: Vec<BlockingChain> = Vec::new();

        let mut output = Vec::new();
        display_table(&rows, &chains, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("PROD.orders"));
        assert!(!output_str.contains("Blocking Chain:"));
    }

    #[test]
    fn test_display_csv_output() {
        let rows = vec![
            LockDisplayRow {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                waiting_sessions: vec![1045, 1067],
            },
            LockDisplayRow {
                locked_object: "PROD.employees".to_string(),
                lock_type: "Row Hash".to_string(),
                lock_mode: "READ".to_string(),
                locking_session: 1078,
                waiting_sessions: Vec::new(),
            },
        ];

        let mut output = Vec::new();
        display_csv(&rows, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess"));
        assert!(output_str.contains("PROD.orders,Table,WRITE,1023,\"1045, 1067\""));
        assert!(output_str.contains("PROD.employees,Row Hash,READ,1078,"));
    }

    #[test]
    fn test_display_json_output() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "WRITE".to_string(),
            locking_session: 1023,
            waiting_sessions: vec![1045, 1067],
        }];

        let mut output = Vec::new();
        display_json(&rows, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap();
        assert_eq!(json.len(), 1);
        assert_eq!(json[0]["Locked Object"], "PROD.orders");
        assert_eq!(json[0]["Lock Type"], "Table");
        assert_eq!(json[0]["Lock Mode"], "WRITE");
        assert_eq!(json[0]["Locking Sess"], 1023);

        let waiting = json[0]["Waiting Sess"].as_array().unwrap();
        assert_eq!(waiting.len(), 2);
        assert_eq!(waiting[0], 1045);
        assert_eq!(waiting[1], 1067);
    }

    #[test]
    fn test_display_json_no_waiters() {
        let rows = vec![LockDisplayRow {
            locked_object: "PROD.orders".to_string(),
            lock_type: "Table".to_string(),
            lock_mode: "READ".to_string(),
            locking_session: 1078,
            waiting_sessions: Vec::new(),
        }];

        let mut output = Vec::new();
        display_json(&rows, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap();
        let waiting = json[0]["Waiting Sess"].as_array().unwrap();
        assert!(waiting.is_empty());
    }

    // escape_csv, extract_trimmed_string, and extract_integer tests removed:
    // already tested in monitoring_utils.rs

    #[test]
    fn test_lock_info_from_row_with_decimal_sessions() {
        // Some Teradata versions may return session IDs as Decimal
        let row = vec![
            Value::String("PROD.orders".to_string()),
            Value::String("Table".to_string()),
            Value::String("WRITE".to_string()),
            Value::Decimal(1023.0),
            Value::Decimal(1045.0),
            Value::String("R".to_string()),
        ];

        let lock = LockInfo::from_row(&row);
        assert!(lock.is_some());

        let lock = lock.unwrap();
        assert_eq!(lock.grantor_session, 1023);
        assert_eq!(lock.locking_session, 1045);
        assert!(lock.is_waiting);
    }

    #[test]
    fn test_build_display_rows_deduplicates_waiters() {
        // If the same session appears as a waiter multiple times
        // (e.g., from multiple lock rows), it should only appear once
        let locks = vec![
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1023,
                grantor_session: 1023,
                is_waiting: false,
            },
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1045,
                grantor_session: 1023,
                is_waiting: true,
            },
            // Same waiter appearing again (should be deduplicated)
            LockInfo {
                locked_object: "PROD.orders".to_string(),
                lock_type: "Table".to_string(),
                lock_mode: "WRITE".to_string(),
                locking_session: 1045,
                grantor_session: 1023,
                is_waiting: true,
            },
        ];

        let rows = build_display_rows(&locks);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].waiting_sessions, vec![1045]);
    }

    // =========================================================================
    // Error handling tests (Sprint 39)
    // =========================================================================

    /// Helper: simulate the error classification logic from execute_for_repl
    fn classify_error(error_msg: &str) -> &'static str {
        let error_str = error_msg.to_lowercase();
        if error_str.contains("privilege")
            || error_str.contains("access")
            || error_str.contains("permission")
            || error_str.contains("3523")
        {
            "permission_denied"
        } else if error_str.contains("lockinfov")
            && (error_str.contains("not found") || error_str.contains("does not exist"))
        {
            "view_not_available"
        } else {
            "generic_error"
        }
    }

    #[test]
    fn test_error_classification_privilege() {
        assert_eq!(
            classify_error("[Error 3523] The user does not have SELECT access to DBC.LockInfoV"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_access_denied() {
        assert_eq!(
            classify_error("Access denied to DBC.LockInfoV"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_permission_error() {
        assert_eq!(
            classify_error("Permission denied for lock view query"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_view_not_found() {
        assert_eq!(
            classify_error("Object 'DBC.LockInfoV' not found"),
            "view_not_available"
        );
    }

    #[test]
    fn test_error_classification_view_does_not_exist() {
        assert_eq!(
            classify_error("DBC.LockInfoV does not exist on this system"),
            "view_not_available"
        );
    }

    #[test]
    fn test_error_classification_generic() {
        assert_eq!(
            classify_error("Connection timeout during lock query"),
            "generic_error"
        );
    }

    #[test]
    fn test_lock_info_from_row_malformed_data_all_nulls() {
        // Test with all NULL values - should return None since session IDs are required
        let row = vec![
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ];

        let lock = LockInfo::from_row(&row);
        // grantor_session is NULL -> extract_integer returns None -> from_row returns None
        assert!(lock.is_none());
    }

    #[test]
    fn test_lock_info_from_row_malformed_session_types() {
        // Test with String session IDs (wrong type) - extract_integer returns None for strings
        let row = vec![
            Value::String("PROD.orders".to_string()),
            Value::String("Table".to_string()),
            Value::String("WRITE".to_string()),
            Value::String("not_a_number".to_string()),
            Value::Integer(1023),
            Value::Null,
        ];

        let lock = LockInfo::from_row(&row);
        // String value for grantor_session -> extract_integer returns None -> from_row returns None
        assert!(lock.is_none());
    }

    #[test]
    fn test_build_display_rows_handles_empty_input() {
        let locks: Vec<LockInfo> = Vec::new();
        let rows = build_display_rows(&locks);
        assert!(rows.is_empty());

        let chains = identify_blocking_chains(&rows);
        assert!(chains.is_empty());
    }

    #[test]
    fn test_display_table_empty_locks() {
        let rows: Vec<LockDisplayRow> = Vec::new();
        let chains: Vec<BlockingChain> = Vec::new();

        let mut output = Vec::new();
        display_table(&rows, &chains, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("No locks currently held."));
    }

    #[test]
    fn test_display_csv_empty_locks() {
        let rows: Vec<LockDisplayRow> = Vec::new();

        let mut output = Vec::new();
        display_csv(&rows, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // CSV output should still have the header even with no data
        assert!(output_str.contains("Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess"));
        // But only header line (plus newline)
        let lines: Vec<&str> = output_str.trim().lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_display_json_empty_locks() {
        let rows: Vec<LockDisplayRow> = Vec::new();

        let mut output = Vec::new();
        display_json(&rows, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap();
        assert!(json.is_empty());
    }

    #[test]
    fn test_format_waiting_sessions_csv_empty() {
        assert_eq!(format_waiting_sessions_csv(&[]), "");
    }

    #[test]
    fn test_format_waiting_sessions_csv_single() {
        assert_eq!(format_waiting_sessions_csv(&[1045]), "1045");
    }

    #[test]
    fn test_format_waiting_sessions_csv_multiple() {
        assert_eq!(format_waiting_sessions_csv(&[1045, 1067]), "1045, 1067");
    }
}
