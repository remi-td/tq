use thiserror::Error;

/// Library-level errors using thiserror for structured error handling
#[derive(Error, Debug)]
pub enum TqError {
    #[error("Invalid connection string format: {0}")]
    InvalidConnectionString(String),

    #[error("Connection failed to {host}: {message}")]
    Connection { host: String, message: String },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, TqError>;
