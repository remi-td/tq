//! Structural agent-safe SQL classification.
//!
//! This module classifies the effective top-level operation of a single SQL
//! statement for agent-safe mode. Unlike a first-keyword heuristic, it:
//!
//! - skips arbitrary interleaved whitespace / line / block comments (via the
//!   shared [`significant_tokens`](crate::sql::parser::significant_tokens)
//!   primitive, so quoting/comment rules cannot diverge from the batch parser);
//! - resolves a `WITH` CTE prologue (paren-aware) to its final operation;
//! - consumes one or more `LOCKING` / `LOCK` request modifiers and classifies
//!   the operation they modify;
//! - fails *closed*: anything not provably classified becomes
//!   [`StatementSafety::Unknown`] rather than being mislabelled as DDL.
//!
//! The classifier never maps "unknown" to `Ddl`. `Unknown` is its own terminal
//! category so the diagnostic surfaced to the user is honest.

use crate::sql::parser::{significant_tokens, SqlToken};

/// Safety classification of a single SQL statement for agent-safe mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementSafety {
    /// `SELECT`/`SEL`, `SHOW`, `HELP`, `EXPLAIN`, and read-only `WITH`/`LOCKING`
    /// forms. Always allowed in agent-safe mode.
    ReadOnly,
    /// `COLLECT STATISTICS` / `COLLECT STATS`. Blocked unless
    /// `--allow-maintenance`.
    Maintenance,
    /// `INSERT`/`INS`, `UPDATE`/`UPD`, `DELETE`/`DEL`, `MERGE`, `UPSERT`.
    /// Blocked unless `--allow-dml`.
    Dml,
    /// `CREATE`, `REPLACE`, `DROP`, `ALTER`, `RENAME`, `GRANT`, `REVOKE`, … —
    /// always blocked.
    Ddl,
    /// Could not be classified; fail closed. `token` is the first significant
    /// token seen (if any); `reason` explains why classification stopped.
    Unknown {
        token: Option<String>,
        reason: String,
    },
}

/// Result of classification: the safety category plus the effective resolved
/// operation keyword (e.g. `UPDATE` for `LOCKING … UPDATE`), used in error
/// messages. For [`StatementSafety::Unknown`] the keyword carried here mirrors
/// the `Unknown.token` (or `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The safety category.
    pub safety: StatementSafety,
    /// The effective resolved operation keyword (uppercased), if one was found.
    pub effective_op: Option<String>,
}

/// Classify the effective top-level operation of a single SQL statement,
/// returning both the safety category and the effective resolved operation
/// keyword for diagnostics.
pub fn classify_statement_detailed(sql: &str) -> Classification {
    let toks = tokens(sql);
    classify_from(&toks, 0)
}

/// Collect the significant words of a statement, uppercased, dropping
/// string-literal markers but preserving punctuation so the CTE/LOCKING scanners
/// can track parentheses and commas.
fn tokens(sql: &str) -> Vec<SqlToken> {
    significant_tokens(sql)
        .map(|t| match t {
            SqlToken::Word(w) => SqlToken::Word(w.to_ascii_uppercase()),
            other => other,
        })
        .collect()
}

/// Classify the effective top-level operation of a single SQL statement.
///
/// See the module documentation for the full rule set.
pub fn classify_statement(sql: &str) -> StatementSafety {
    classify_statement_detailed(sql).safety
}

/// Classify starting at token index `start`. Recursion is bounded by the modifier
/// loop always advancing the index, so this terminates.
fn classify_from(toks: &[SqlToken], start: usize) -> Classification {
    let mut i = start;

    // Skip leading LOCKING / LOCK request modifiers (possibly stacked). Each
    // modifier is consumed up to the operation it precedes.
    loop {
        match toks.get(i) {
            None => {
                return unknown(None, "no statement");
            }
            Some(SqlToken::Word(w)) if w == "LOCKING" || w == "LOCK" => {
                match skip_locking_modifier(toks, i) {
                    Some(next) => {
                        i = next;
                        continue;
                    }
                    None => {
                        return unknown(Some(w.clone()), "malformed LOCKING request modifier");
                    }
                }
            }
            _ => break,
        }
    }

    // Resolve a WITH CTE prologue to its final operation.
    if let Some(SqlToken::Word(w)) = toks.get(i) {
        if w == "WITH" {
            return match resolve_with(toks, i) {
                Some(op_idx) => classify_operation(toks, op_idx),
                None => unknown(
                    Some("WITH".to_string()),
                    "could not resolve WITH prologue to an operation",
                ),
            };
        }
    }

    classify_operation(toks, i)
}

/// Build an `Unknown` classification, mirroring the diagnostic token into
/// `effective_op` for consistent error reporting.
fn unknown(token: Option<String>, reason: &str) -> Classification {
    Classification {
        safety: StatementSafety::Unknown {
            token: token.clone(),
            reason: reason.to_string(),
        },
        effective_op: token,
    }
}

/// Classify the direct operation keyword at index `i`.
fn classify_operation(toks: &[SqlToken], i: usize) -> Classification {
    let word = match toks.get(i) {
        Some(SqlToken::Word(w)) => w.as_str(),
        Some(_) => {
            return unknown(None, "statement does not start with a keyword");
        }
        None => {
            return unknown(None, "no statement");
        }
    };

    let op = word.to_string();
    let safety = match word {
        "SELECT" | "SEL" | "SHOW" | "HELP" | "EXPLAIN" => StatementSafety::ReadOnly,

        "COLLECT" => match toks.get(i + 1) {
            Some(SqlToken::Word(w)) if w == "STATISTICS" || w == "STATS" => {
                StatementSafety::Maintenance
            }
            _ => {
                return unknown(
                    Some("COLLECT".to_string()),
                    "COLLECT not followed by STATISTICS/STATS",
                );
            }
        },

        "INSERT" | "INS" | "UPDATE" | "UPD" | "DELETE" | "DEL" | "MERGE" | "UPSERT" => {
            StatementSafety::Dml
        }

        "CREATE" | "REPLACE" | "DROP" | "ALTER" | "RENAME" | "GRANT" | "REVOKE" | "DATABASE"
        | "USER" | "COMMENT" | "SET" | "BEGIN" | "END" | "GIVE" | "MODIFY" | "FLUSH" | "DUMP"
        | "RESTORE" => StatementSafety::Ddl,

        other => {
            return unknown(Some(other.to_string()), "unrecognised leading operation");
        }
    };

    Classification {
        safety,
        effective_op: Some(op),
    }
}

/// Consume a single `LOCKING [ROW|TABLE|DATABASE|VIEW] <object>? FOR <lock-type>
/// [MODE|NOWAIT|OVERRIDE]*` modifier starting at `start` (which points at the
/// `LOCKING`/`LOCK` keyword). Returns the index of the first token *after* the
/// modifier (the start of the modified request), or `None` if no recognised
/// operation keyword follows.
///
/// The Teradata modifier grammar is open-ended (object names, optional `FOR`
/// clause variants), so rather than parse it exhaustively we scan forward,
/// parenthesis-aware, to the next top-level operation keyword. This is robust to
/// stacked modifiers and arbitrary object qualification.
fn skip_locking_modifier(toks: &[SqlToken], start: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut i = start + 1; // skip LOCKING/LOCK itself
    while i < toks.len() {
        match &toks[i] {
            SqlToken::Punct('(') => depth += 1,
            SqlToken::Punct(')') => depth -= 1,
            SqlToken::Word(w) if depth == 0 => {
                // Another stacked modifier: let the caller's loop consume it.
                if w == "LOCKING" || w == "LOCK" {
                    return Some(i);
                }
                // The first top-level operation keyword ends the modifier.
                if is_operation_keyword(w) {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Resolve a `WITH [RECURSIVE] cte AS ( … ) [, cte2 AS ( … )]* <operation>`
/// prologue to the index of the trailing top-level operation keyword.
///
/// `start` points at the `WITH` keyword. We skip the CTE definition list by
/// tracking parenthesis depth (so commas / keywords *inside* a CTE body are
/// ignored) and stop at the first top-level operation keyword that is not part
/// of a CTE definition. Returns `None` on a malformed / unbalanced prologue.
fn resolve_with(toks: &[SqlToken], start: usize) -> Option<usize> {
    let mut i = start + 1; // skip WITH

    // Optional RECURSIVE
    if let Some(SqlToken::Word(w)) = toks.get(i) {
        if w == "RECURSIVE" {
            i += 1;
        }
    }

    let mut depth: i32 = 0;
    while i < toks.len() {
        match &toks[i] {
            SqlToken::Punct('(') => depth += 1,
            SqlToken::Punct(')') => {
                depth -= 1;
                if depth < 0 {
                    return None; // unbalanced
                }
            }
            SqlToken::Word(w) if depth == 0 => {
                // At top level, a CTE definition looks like `name [ (cols) ] AS
                // ( body )`. The trailing operation is the first top-level
                // operation keyword we encounter that is not the CTE structural
                // keyword `AS`.
                if is_operation_keyword(w) {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Whether `word` (already uppercased) is a recognised top-level operation
/// keyword that terminates a LOCKING modifier or a WITH prologue.
fn is_operation_keyword(word: &str) -> bool {
    matches!(
        word,
        "SELECT"
            | "SEL"
            | "INSERT"
            | "INS"
            | "UPDATE"
            | "UPD"
            | "DELETE"
            | "DEL"
            | "MERGE"
            | "UPSERT"
            | "WITH"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Read-only ----

    #[test]
    fn test_select_readonly() {
        assert_eq!(classify_statement("SELECT * FROM t"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("  select 1"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("SEL * FROM t"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("SHOW VIEW db.v"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("EXPLAIN SELECT 1"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("HELP TABLE t"), StatementSafety::ReadOnly);
    }

    // ---- Comments ----

    #[test]
    fn test_leading_comments_skipped() {
        assert_eq!(classify_statement("-- c\nSELECT 1"), StatementSafety::ReadOnly);
        assert_eq!(
            classify_statement("/* a */ /* b */ SELECT 1"),
            StatementSafety::ReadOnly
        );
        assert_eq!(
            classify_statement("/* a */ -- b\n /* c */ SELECT 1"),
            StatementSafety::ReadOnly
        );
        assert_eq!(
            classify_statement("/* block */ INSERT INTO t VALUES (1)"),
            StatementSafety::Dml
        );
    }

    // ---- WITH CTE ----

    #[test]
    fn test_with_cte_readonly() {
        assert_eq!(
            classify_statement("WITH x AS (SELECT 1) SELECT * FROM x"),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_with_cte_with_column_list() {
        assert_eq!(
            classify_statement("WITH x (a, b) AS (SELECT 1, 2) SELECT a FROM x"),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_with_recursive_readonly() {
        assert_eq!(
            classify_statement(
                "WITH RECURSIVE r AS (SELECT 1 AS n) SELECT n FROM r"
            ),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_with_multiple_ctes() {
        assert_eq!(
            classify_statement(
                "WITH a AS (SELECT 1), b AS (SELECT 2) SELECT * FROM a, b"
            ),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_with_resolves_to_dml() {
        assert_eq!(
            classify_statement("WITH x AS (SELECT 1) INSERT INTO t SELECT * FROM x"),
            StatementSafety::Dml
        );
        assert_eq!(
            classify_statement("WITH x AS (SELECT 1) DELETE FROM t WHERE id IN (SELECT 1 FROM x)"),
            StatementSafety::Dml
        );
    }

    #[test]
    fn test_with_nested_parens_ignored() {
        // Commas and keywords inside CTE bodies (and nested parens) must not be
        // mistaken for the trailing operation.
        assert_eq!(
            classify_statement(
                "WITH x AS (SELECT a, b FROM (SELECT 1 AS a, 2 AS b) s) SELECT * FROM x"
            ),
            StatementSafety::ReadOnly
        );
    }

    // ---- LOCKING modifiers ----

    #[test]
    fn test_locking_select_readonly() {
        assert_eq!(
            classify_statement("LOCKING t FOR ACCESS SELECT * FROM t"),
            StatementSafety::ReadOnly
        );
        assert_eq!(
            classify_statement("LOCKING ROW FOR ACCESS SELECT 1"),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_locking_write_is_dml() {
        assert_eq!(
            classify_statement("LOCKING ROW FOR WRITE UPDATE t SET x = 1"),
            StatementSafety::Dml
        );
        assert_eq!(
            classify_statement("LOCKING TABLE t FOR WRITE DELETE FROM t"),
            StatementSafety::Dml
        );
        assert_eq!(
            classify_statement("LOCKING ROW FOR WRITE INSERT INTO t VALUES (1)"),
            StatementSafety::Dml
        );
        assert_eq!(
            classify_statement("LOCKING ROW FOR WRITE MERGE INTO t USING s ON t.a=s.a"),
            StatementSafety::Dml
        );
    }

    #[test]
    fn test_stacked_locking_modifiers() {
        assert_eq!(
            classify_statement(
                "LOCKING TABLE a FOR ACCESS LOCKING TABLE b FOR ACCESS SELECT * FROM a, b"
            ),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_locking_then_with() {
        assert_eq!(
            classify_statement(
                "LOCKING ROW FOR ACCESS WITH x AS (SELECT 1) SELECT * FROM x"
            ),
            StatementSafety::ReadOnly
        );
    }

    #[test]
    fn test_locking_no_operation_is_unknown() {
        assert!(matches!(
            classify_statement("LOCKING TABLE t FOR ACCESS"),
            StatementSafety::Unknown { .. }
        ));
    }

    // ---- Maintenance ----

    #[test]
    fn test_collect_statistics_maintenance() {
        assert_eq!(
            classify_statement("COLLECT STATISTICS ON t COLUMN c"),
            StatementSafety::Maintenance
        );
        assert_eq!(
            classify_statement("COLLECT STATS ON t"),
            StatementSafety::Maintenance
        );
        assert_eq!(
            classify_statement("collect statistics on t"),
            StatementSafety::Maintenance
        );
    }

    #[test]
    fn test_collect_other_is_unknown() {
        // COLLECT not followed by STATISTICS/STATS must NOT be assumed read-only.
        assert!(matches!(
            classify_statement("COLLECT DEMOGRAPHICS FOR t"),
            StatementSafety::Unknown { .. }
        ));
        assert!(matches!(
            classify_statement("COLLECT"),
            StatementSafety::Unknown { .. }
        ));
    }

    // ---- DML (incl. Teradata abbreviations) ----

    #[test]
    fn test_dml() {
        assert_eq!(classify_statement("INSERT INTO t VALUES (1)"), StatementSafety::Dml);
        assert_eq!(classify_statement("UPDATE t SET x=1"), StatementSafety::Dml);
        assert_eq!(classify_statement("DELETE FROM t"), StatementSafety::Dml);
        assert_eq!(classify_statement("MERGE INTO t USING s"), StatementSafety::Dml);
        assert_eq!(classify_statement("UPSERT INTO t VALUES (1)"), StatementSafety::Dml);
    }

    #[test]
    fn test_dml_abbreviations() {
        assert_eq!(classify_statement("INS INTO t VALUES (1)"), StatementSafety::Dml);
        assert_eq!(classify_statement("UPD t SET x=1"), StatementSafety::Dml);
        assert_eq!(classify_statement("DEL FROM t"), StatementSafety::Dml);
    }

    // ---- DDL ----

    #[test]
    fn test_ddl() {
        assert_eq!(classify_statement("CREATE TABLE t (id INT)"), StatementSafety::Ddl);
        assert_eq!(classify_statement("DROP TABLE t"), StatementSafety::Ddl);
        assert_eq!(classify_statement("ALTER TABLE t ADD x INT"), StatementSafety::Ddl);
        assert_eq!(classify_statement("RENAME TABLE t TO t2"), StatementSafety::Ddl);
        assert_eq!(classify_statement("REPLACE VIEW v AS SELECT 1"), StatementSafety::Ddl);
        assert_eq!(classify_statement("GRANT SELECT ON t TO u"), StatementSafety::Ddl);
        assert_eq!(classify_statement("REVOKE SELECT ON t FROM u"), StatementSafety::Ddl);
    }

    // ---- Unknown (fail closed) ----

    #[test]
    fn test_unknown_fails_closed() {
        match classify_statement("FROBNICATE THE WIDGETS") {
            StatementSafety::Unknown { token, .. } => {
                assert_eq!(token.as_deref(), Some("FROBNICATE"));
            }
            other => panic!("expected Unknown, got {:?}", other),
        }
    }

    #[test]
    fn test_unknown_not_mislabelled_ddl() {
        // The defining property: a genuinely unrecognised leading keyword is
        // Unknown, NOT Ddl.
        assert!(matches!(
            classify_statement("WIBBLE foo"),
            StatementSafety::Unknown { .. }
        ));
    }

    #[test]
    fn test_empty_is_unknown() {
        assert!(matches!(
            classify_statement(""),
            StatementSafety::Unknown { token: None, .. }
        ));
        assert!(matches!(
            classify_statement("   -- only a comment\n"),
            StatementSafety::Unknown { token: None, .. }
        ));
    }
}
