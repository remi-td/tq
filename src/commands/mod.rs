//! Command implementations for tq
//!
//! Each command has a dedicated implementation with proper error handling
//! and output formatting.

pub mod abort;
pub mod active_query;
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
pub mod query_plan;
pub mod repl;
pub mod resources;
pub mod sample;
pub mod search;
pub mod sessions;
pub mod severity;
pub mod show_indexes;
pub mod skew;
pub mod space;
pub mod sysconfig;
pub mod watch;
pub mod fastload;
pub mod fastexport;
pub mod params;
pub mod errorlevel;

pub use abort::execute as abort;
pub use active_query::execute as active_query;
pub use explain::execute as explain;
pub use history::execute as history;
pub use locks::execute as locks;
pub use ping::execute as ping;
pub use query::execute as query;
pub use query_inspect::execute as query_inspect;
pub use query_plan::execute as query_plan;
pub use repl::execute as repl;
pub use resources::execute as resources;
pub use sample::{execute_peek as peek, execute_sample as sample};
pub use sessions::execute as sessions;
pub use skew::execute as skew;
pub use space::execute as space;
pub use space::execute_dbspace as dbspace;
pub use logoff_idle::execute as logoff_idle;
pub use sysconfig::execute as sysconfig;
pub use fastload::execute as fastload;
pub use fastexport::execute as fastexport;
pub use params::execute as params;
pub use errorlevel::execute as errorlevel;

