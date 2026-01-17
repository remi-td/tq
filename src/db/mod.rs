//! Database connectivity and operations
//!
//! This module provides all database-related functionality:
//! - Connection configuration and management
//! - Query execution
//! - Type-safe result handling

pub mod client;
pub mod connection;
pub mod types;

// Re-export commonly used types
pub use client::DatabaseClient;
pub use connection::{parse_duration, ConnectionConfig};
pub use types::{Alignment, ColumnMetadata, QueryResult, Row, TeradataType, Value};
