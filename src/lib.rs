//! # tq - Teradata Query
//!
//! A lightweight Rust command line client for Teradata databases.
//!
//! This library provides core functionality for connecting to and interacting
//! with Teradata databases. It follows a simple one-shot execution model:
//! connect -> execute -> close.
//!
//! ## Example
//!
//! ```no_run
//! use tq::{ConnectionConfig, DatabaseClient};
//!
//! let config = ConnectionConfig::parse(
//!     "user:password@host:1025/database",
//!     "TD2",
//!     None
//! ).unwrap();
//!
//! let client = DatabaseClient::new(config, None).unwrap();
//! let latency = client.ping().unwrap();
//! println!("Ping: {:.2}ms", latency.as_secs_f64() * 1000.0);
//! ```

pub mod cli;
pub mod connection;
pub mod db;
pub mod error;

// Re-export commonly used types
pub use cli::OutputFormat;
pub use connection::{ConnectionConfig, LogonMechanism};
pub use db::{DatabaseClient, QueryResults, Row};
pub use error::{Result, TqError};
