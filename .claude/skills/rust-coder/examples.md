# Rust Coding Examples

## Example 1: Struct with Methods (Official Style)

```rust
/// A rectangle defined by its width and height.
///
/// This struct represents a 2D rectangle with positive dimensions.
/// All measurements are in pixels.
///
/// # Examples
///
/// ```
/// let rect = Rectangle::new(10, 20);
/// assert_eq!(rect.area(), 200);
/// ```
pub struct Rectangle {
    /// Width of the rectangle in pixels
    width: u32,
    /// Height of the rectangle in pixels
    height: u32,
}

impl Rectangle {
    /// Creates a new Rectangle with the given dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// let rect = Rectangle::new(10, 20);
    /// ```
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the area of the rectangle in square pixels.
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// Checks if this rectangle can hold another rectangle.
    ///
    /// Returns `true` if both dimensions of this rectangle are strictly
    /// greater than the corresponding dimensions of `other`.
    pub fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```

## Example 2: Trait Implementation

```rust
/// Trait for types that can greet
trait Greet {
    /// Returns a greeting message
    fn greet(&self) -> String;
}

/// A person with a name
struct Person {
    name: String,
}

impl Greet for Person {
    /// Returns a greeting message with the person's name
    fn greet(&self) -> String {
        format!("Hello, my name is {}", self.name)
    }
}
```

## Example 3: Declarative Macro

```rust
/// Macro to automatically derive common traits
macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
            $item
        )+
    };
}

// Usage
auto_derived! {
    /// Server configuration
    struct Server {
        /// Unique server identifier
        id: u64,
        /// Server name
        name: String,
    }
}
```

## Example 4: Result and Error Handling

```rust
use std::fs::File;
use std::io::{self, Read};

/// Reads a file and returns its contents
fn read_file_contents(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Usage
match read_file_contents("config.txt") {
    Ok(contents) => println!("File contents: {}", contents),
    Err(e) => eprintln!("Error reading file: {}", e),
}
```

## Example 5: Builder Pattern

```rust
/// Configuration for database connection.
#[derive(Debug)]
pub struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

impl DatabaseConfig {
    /// Creates a new builder for DatabaseConfig.
    pub fn builder() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder::default()
    }
}

/// Builder for constructing a `DatabaseConfig`.
#[derive(Default)]
pub struct DatabaseConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

impl DatabaseConfigBuilder {
    /// Sets the database host.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the database port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets the username for authentication.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the password for authentication.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the database name.
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Builds the `DatabaseConfig`, returning an error if required fields are missing.
    pub fn build(self) -> Result<DatabaseConfig, &'static str> {
        Ok(DatabaseConfig {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(5432),
            username: self.username.ok_or("username is required")?,
            password: self.password.ok_or("password is required")?,
            database: self.database.ok_or("database is required")?,
        })
    }
}

// Usage
let config = DatabaseConfig::builder()
    .host("localhost")
    .port(5432)
    .username("admin")
    .password("secret")
    .database("mydb")
    .build()?;
```

## Example 6: Expression-Oriented Programming

```rust
// BAD: Statement-based approach with separate assignments
fn get_discount_bad(price: f64, is_member: bool) -> f64 {
    let discount;
    if is_member {
        discount = price * 0.1;
    } else {
        discount = 0.0;
    }
    discount
}

// GOOD: Expression-oriented approach
fn get_discount_good(price: f64, is_member: bool) -> f64 {
    if is_member {
        price * 0.1
    } else {
        0.0
    }
}

// BETTER: Using match for clarity
fn calculate_shipping(distance: u32, express: bool) -> f64 {
    match (distance, express) {
        (0..=10, false) => 5.0,
        (0..=10, true) => 10.0,
        (11..=50, false) => 10.0,
        (11..=50, true) => 20.0,
        (_, false) => 15.0,
        (_, true) => 30.0,
    }
}

// Expression blocks for complex initialization
fn process_data(input: &str) -> Result<ProcessedData, Error> {
    let parsed = {
        let trimmed = input.trim();
        let normalized = trimmed.to_lowercase();
        parse(&normalized)?
    };

    let validated = {
        validate(&parsed)?;
        transform(parsed)
    };

    Ok(validated)
}
```

## Example 7: Enum State Machine (Preferred Over Booleans)

```rust
// BAD: Using boolean flags
struct ConnectionBad {
    is_connected: bool,
    is_authenticated: bool,
    is_encrypted: bool,
    has_error: bool,
}

// GOOD: Using enum for state machine
/// Represents the state of a database connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is not yet established
    Disconnected,
    /// Connection established but not authenticated
    Connected,
    /// Connection authenticated and ready for queries
    Authenticated { session_id: String },
    /// Connection encrypted with TLS
    Encrypted { cipher: String },
    /// Connection encountered an error
    Error { message: String },
}

/// A database connection with type-safe state management.
pub struct Connection {
    host: String,
    state: ConnectionState,
}

impl Connection {
    /// Creates a new disconnected connection.
    pub fn new(host: String) -> Self {
        Self {
            host,
            state: ConnectionState::Disconnected,
        }
    }

    /// Establishes the connection.
    pub fn connect(&mut self) -> Result<(), Error> {
        match self.state {
            ConnectionState::Disconnected => {
                // Connection logic here
                self.state = ConnectionState::Connected;
                Ok(())
            }
            _ => Err(Error::InvalidState),
        }
    }

    /// Authenticates the connection.
    pub fn authenticate(&mut self, credentials: &Credentials) -> Result<(), Error> {
        match &self.state {
            ConnectionState::Connected => {
                let session_id = perform_auth(credentials)?;
                self.state = ConnectionState::Authenticated { session_id };
                Ok(())
            }
            _ => Err(Error::InvalidState),
        }
    }

    /// Returns true if the connection can execute queries.
    pub fn is_ready(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Authenticated { .. } | ConnectionState::Encrypted { .. }
        )
    }
}
```

## Example 8: Proper Module Organization

```rust
// src/lib.rs
//! Database client library for connecting to various database systems.
//!
//! This library provides a unified interface for database operations
//! with support for connection pooling, transactions, and migrations.

// Imports first, version-sorted, self/super first
use std::collections::HashMap;
use std::io::{self, Read, Write};

// Module declarations after imports
pub mod connection;
pub mod error;
pub mod query;

// Re-exports for convenience
pub use connection::Connection;
pub use error::DbError;
pub use query::Query;

// Constants
const DEFAULT_PORT: u16 = 5432;
const MAX_CONNECTIONS: usize = 100;

// Type aliases
pub type Result<T> = std::result::Result<T, DbError>;

// Public structs
/// Configuration for database client.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl Config {
    /// Creates a new configuration with default port.
    pub fn new(host: String, database: String) -> Self {
        Self {
            host,
            port: DEFAULT_PORT,
            database,
        }
    }
}

// Module-private helper functions
fn validate_config(config: &Config) -> Result<()> {
    if config.host.is_empty() {
        return Err(DbError::InvalidConfig("host cannot be empty".into()));
    }
    Ok(())
}
```

## Example 9: Comprehensive Error Handling with thiserror

```rust
use std::io;
use thiserror::Error;

/// Errors that can occur during database operations.
#[derive(Error, Debug)]
pub enum DbError {
    /// Connection to database failed
    #[error("connection failed to {host}:{port}")]
    ConnectionFailed {
        host: String,
        port: u16,
        #[source]
        source: io::Error,
    },

    /// Query execution failed
    #[error("query execution failed: {message}")]
    QueryFailed {
        message: String,
        query: String,
    },

    /// Authentication error
    #[error("authentication failed for user {username}")]
    AuthenticationFailed {
        username: String,
    },

    /// Invalid configuration
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Timeout occurred
    #[error("operation timed out after {seconds}s")]
    Timeout {
        seconds: u64,
    },

    /// IO error wrapper
    #[error("IO error")]
    Io(#[from] io::Error),
}

/// Result type alias for database operations.
pub type Result<T> = std::result::Result<T, DbError>;

// Usage example
fn connect_to_database(host: &str, port: u16) -> Result<Connection> {
    std::net::TcpStream::connect((host, port))
        .map(Connection::new)
        .map_err(|e| DbError::ConnectionFailed {
            host: host.to_string(),
            port,
            source: e,
        })
}
```

## Example 10: Proper Use of Cow for Flexible Ownership

```rust
use std::borrow::Cow;

/// Processes text that might be owned or borrowed.
///
/// This function demonstrates efficient use of `Cow` to avoid unnecessary
/// allocations when the input doesn't need modification.
pub fn normalize_text<'a>(input: Cow<'a, str>) -> Cow<'a, str> {
    if input.chars().all(|c| c.is_ascii_lowercase()) {
        // No modification needed, return as-is
        input
    } else {
        // Modification needed, convert to owned String
        Cow::Owned(input.to_lowercase())
    }
}

// Usage examples
fn example_usage() {
    // Borrowed input, no allocation needed
    let result1 = normalize_text(Cow::Borrowed("hello"));
    assert_eq!(result1, "hello");

    // Borrowed input, requires allocation for transformation
    let result2 = normalize_text(Cow::Borrowed("Hello"));
    assert_eq!(result2, "hello");

    // Owned input
    let owned = String::from("WORLD");
    let result3 = normalize_text(Cow::Owned(owned));
    assert_eq!(result3, "world");
}

/// Configuration value that can be owned or borrowed.
pub enum ConfigValue<'a> {
    Str(Cow<'a, str>),
    Int(i64),
    Bool(bool),
}

impl<'a> ConfigValue<'a> {
    /// Converts to owned version for storage.
    pub fn into_owned(self) -> ConfigValue<'static> {
        match self {
            ConfigValue::Str(s) => ConfigValue::Str(Cow::Owned(s.into_owned())),
            ConfigValue::Int(i) => ConfigValue::Int(i),
            ConfigValue::Bool(b) => ConfigValue::Bool(b),
        }
    }
}
```

## Example 11: Method Chaining with Proper Formatting

```rust
use std::collections::HashMap;

/// Processes a collection of user data with method chaining.
pub fn process_users(users: Vec<User>) -> HashMap<String, ProcessedUser> {
    users
        .into_iter()
        .filter(|u| u.is_active)
        .filter(|u| u.age >= 18)
        .map(|u| {
            let processed = ProcessedUser {
                id: u.id,
                name: u.name.to_uppercase(),
                email: u.email.to_lowercase(),
            };
            (u.id.clone(), processed)
        })
        .collect()
}

/// Builder pattern with fluent method chaining.
pub struct QueryBuilder {
    table: Option<String>,
    columns: Vec<String>,
    conditions: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            table: None,
            columns: Vec::new(),
            conditions: Vec::new(),
            limit: None,
        }
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn select(mut self, column: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self
    }

    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn build(self) -> Result<Query, String> {
        let table = self.table.ok_or("table is required")?;
        Ok(Query {
            table,
            columns: self.columns,
            conditions: self.conditions,
            limit: self.limit,
        })
    }
}

// Usage with proper multi-line formatting
let query = QueryBuilder::new()
    .table("users")
    .select("id")
    .select("name")
    .select("email")
    .where_clause("age >= 18")
    .where_clause("is_active = true")
    .limit(100)
    .build()?;
```
