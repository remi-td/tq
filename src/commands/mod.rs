//! Command implementations for tq
//!
//! Each command has a dedicated implementation with proper error handling
//! and output formatting.

pub mod ping;
pub mod query;
pub mod repl;

pub use ping::execute as ping;
pub use query::execute as query;
pub use repl::execute as repl;
