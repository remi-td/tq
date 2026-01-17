# Objectives

We want to implement a simple yet best in class command line client for Teradata databases.

# Features

We have two main modes of operation:
- REPL mode
- Batch mode

In REPL mode, the user can interact with the database using a prompt. The prompt should be a simple prompt that allows the user to enter a query.
The prompt should support the following features:
- Syntax highlighting
- Auto-completion
- Nice table formatting (with ability to pane left-right and up/down when resultset is rarger than terminal).
- Ability to export current/last resultset to CSV/JSON
- Familiar navigation with vim/emacs, etc... shortcuts
- Command history
- Special functions for non-sql operations. Eg. `/describe`, `/session`, `/logon`, `/ping` to call special functions. Eg. `/describe` samples data from a structure, `/logon` establishes the connection to a (different) database, `/ping` validates database connectivity and returns response time. 

In batch mode, the user can simply provide a sql query or a SQL file containing a list of queries to execute. The output can be redirected to stdout/err or file using classic UNIX semantics. Similarely SQL tex can be streamed into the utility.

We want to support the following features:
- [ ] Support multiple authentication mechanisms (TD2, LDAP, Kerberos)
- [ ] Support multiple output formats (table, JSON, CSV)
- [ ] Support multiple input formats (SQL, CSV, JSON)

# Implementation
Implement in Rust using:
**Core Stack**:
- `clap` v4 - CLI argument parsing with derive macros
- `teradatarustapi` - Teradata database connectivity
- `tabled` or `comfy-table` - Table formatting
- `rustyline` or `reedline`?
