//! Database connectivity and operations
//!
//! This module provides all database-related functionality:
//! - Connection configuration and management
//! - Query execution
//! - Type-safe result handling
//! - Metadata caching for tab completion (Sprint 7)

pub mod client;
pub mod connection;
pub mod metadata;
pub mod types;

// Re-export commonly used types
pub use client::{DatabaseClient, FastloadOptions};
pub use connection::{parse_duration, ConnectionConfig};
pub use metadata::{ColumnInfo, MetadataCache, TableInfo};
pub use types::{Alignment, ColumnMetadata, QueryResult, Row, TeradataType, Value};
