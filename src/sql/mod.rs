//! SQL parsing and processing utilities
//!
//! This module provides SQL-related utilities for batch mode execution:
//!
//! - **parser**: Statement splitting for multi-statement batch execution
//!
//! # Example
//!
//! ```
//! use tq::sql::parser::{parse_statements, has_multiple_statements};
//!
//! let sql = "SELECT 1; SELECT 2;";
//!
//! if has_multiple_statements(sql) {
//!     let statements = parse_statements(sql);
//!     for stmt in statements {
//!         println!("Statement {}: {}", stmt.statement_number, stmt.sql);
//!     }
//! }
//! ```

pub mod parser;

// Re-export commonly used types
pub use parser::{has_multiple_statements, parse_statements, ParsedStatement};
