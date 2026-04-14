#![deny(warnings)]
//! # tq - Teradata Query
//!
//! A fast, lightweight command-line client for Teradata databases.
//!
//! This library provides core functionality for connecting to and interacting
//! with Teradata databases. It follows a simple one-shot execution model:
//! connect -> execute -> close.
//!
//! ## Quick Start
//!
//! ```no_run
//! use tq::{ConnectionConfig, DatabaseClient};
//! use tq::cli::LogonMechanism;
//! use std::time::Duration;
//!
//! // Parse connection string
//! let config = ConnectionConfig::from_connection_string(
//!     "user:password@host:1025/database",
//!     LogonMechanism::Td2,
//!     Duration::from_secs(30),
//!     None,
//! ).unwrap();
//!
//! // Create client and test connection
//! let client = DatabaseClient::new(config, None).unwrap();
//! let latency = client.ping().unwrap();
//! println!("Ping: {:.2}ms", latency.as_secs_f64() * 1000.0);
//! ```
//!
//! ## Query Execution
//!
//! ```no_run
//! use tq::{ConnectionConfig, DatabaseClient, format};
//! use tq::cli::{LogonMechanism, OutputFormat};
//! use tq::format::FormatOptions;
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ConnectionConfig::from_connection_string(
//!     "user:pass@host:1025/db",
//!     LogonMechanism::Td2,
//!     Duration::from_secs(30),
//!     None,
//! )?;
//!
//! let client = DatabaseClient::new(config, None)?;
//! let result = client.execute("SELECT 1 AS col")?;
//!
//! // Format as table
//! let options = FormatOptions::default();
//! let output = format::format_to_string(&result, OutputFormat::Table, &options)?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`cli`]: Command-line interface definitions (Clap structs)
//! - [`config`]: Configuration management with Figment
//! - [`db`]: Database connectivity and operations
//! - [`format`]: Output formatters (table, JSON, CSV)
//! - [`commands`]: Command implementations
//! - [`error`]: Error types with user-friendly messages

pub mod cli;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod format;
pub mod help;
pub mod pagination;
pub mod params;
pub mod sql;

// Re-export commonly used types for convenience
pub use cli::{
    AbortArgs, Cli, Command, ExplainArgs, GlobalOpts, HelpArgs, HelpTopic,
    HistoryArgs, InspectArgs, ListArgs, ListObjectType, LogonMechanism, OutputFormat, PeekArgs,
    PingArgs, ProfileAction, QueryArgs, ReplArgs, ResourcesArgs, SampleArgs, ShowIndexesArgs,
    SkewArgs,
};
pub use config::{Config, ConnectionSettings};
pub use db::{
    Alignment, ColumnMetadata, ConnectionConfig, DatabaseClient, QueryResult, Row, TeradataType,
    Value,
};
pub use error::{Result, TqError};
