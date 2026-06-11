//! SQL parsing, processing, and utilities
//!
//! This module provides SQL-related utilities:
//!
//! - **parser**: Statement splitting for multi-statement batch execution
//! - **types**: Teradata type code formatting (e.g., CV -> VARCHAR)
//! - **identifiers**: SQL identifier quoting and string escaping
//!
//! # Statement Parsing Example
//!
//! ```
//! use tq::sql::parser::{parse_statements, has_multiple_statements};
//!
//! let sql = "SELECT 1; SELECT 2;";
//!
//! if has_multiple_statements(sql) {
//!     let statements = parse_statements(sql).unwrap();
//!     for stmt in statements {
//!         println!("Statement {}: {}", stmt.statement_number, stmt.sql);
//!     }
//! }
//! ```
//!
//! # Type Formatting Example
//!
//! ```
//! use tq::sql::types::format_column_type;
//!
//! let varchar = format_column_type("CV", Some(100), None, None);
//! assert_eq!(varchar, "VARCHAR(100)");
//! ```
//!
//! # Identifier Quoting Example
//!
//! ```
//! use tq::sql::identifiers::{quote_identifier, quote_qualified_name, escape_sql_string};
//!
//! // Identifiers are uppercased to match Teradata's internal storage format
//! let quoted = quote_identifier("my table");
//! assert_eq!(quoted, "\"MY TABLE\"");
//!
//! let qualified = quote_qualified_name("prod", "employees");
//! assert_eq!(qualified, "\"PROD\".\"EMPLOYEES\"");
//!
//! let escaped = escape_sql_string("O'Brien");
//! assert_eq!(escaped, "O''Brien");
//! ```

pub mod classifier;
pub mod identifiers;
pub mod parser;
pub mod types;

// Re-export commonly used types from parser
pub use parser::{
    has_multiple_statements, parse_statements, significant_tokens, ParseError, ParsedStatement,
    SqlToken,
};

// Re-export agent-safe classification
pub use classifier::{
    classify_statement, classify_statement_detailed, Classification, StatementSafety,
};

// Re-export type formatting
pub use types::format_column_type;

// Re-export identifier utilities
pub use identifiers::{escape_sql_like, escape_sql_string, quote_identifier, quote_qualified_name};
