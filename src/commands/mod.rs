//! Command implementations for tq
//!
//! Each command has a dedicated implementation with proper error handling
//! and output formatting.

pub mod abort;
pub mod explain;
pub mod format_helpers;
pub mod history;
pub mod inspect;
pub mod list;
pub mod logoff_idle;
pub mod query_helpers;
pub mod locks;
pub mod monitoring_utils;
pub mod ping;
pub mod profile;
pub mod query;
pub mod query_inspect;
pub mod repl;
pub mod resources;
pub mod sample;
pub mod search;
pub mod sessions;
pub mod show_indexes;
pub mod skew;
pub mod sysconfig;
pub mod watch;

pub use abort::execute as abort;
pub use explain::execute as explain;
pub use history::execute as history;
pub use locks::execute as locks;
pub use ping::execute as ping;
pub use query::execute as query;
pub use query_inspect::execute as query_inspect;
pub use repl::execute as repl;
pub use resources::execute as resources;
pub use sample::{execute_peek as peek, execute_sample as sample};
pub use sessions::execute as sessions;
pub use skew::execute as skew;
pub use logoff_idle::execute as logoff_idle;
pub use sysconfig::execute as sysconfig;
