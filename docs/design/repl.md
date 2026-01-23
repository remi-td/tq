# REPL Mode Design

This document explains the interactive Read-Eval-Print Loop implementation in tq.

**Related Specification**: `docs/specifications/repl.md`

## Overview

The REPL provides an interactive SQL environment with command history, syntax highlighting, tab completion, and intelligent result paging.

## Architecture

```
REPL Loop:

Initialize → Connect → Setup Editor → Show Prompt
                                          ↓
                                      Read Input
                                          ↓
                                  ┌───────┴───────┐
                                  ↓               ↓
                            SQL Statement    Metacommand
                                  ↓               ↓
                              Execute         Process
                                  ↓               ↓
                            Format Result    Update State
                                  ↓               ↓
                              Page Output        ↓
                                  ↓               ↓
                              Show Prompt ←───────┘
```

## Module Structure

```
src/commands/repl/
├── mod.rs              # REPL orchestration
├── executor.rs         # Statement execution
├── metadata_completer.rs  # Tab completion
├── highlighter.rs      # Syntax highlighting
├── pager.rs            # Result paging
└── state.rs            # Session state
```

## Core Components

### REPL State

```rust
// src/commands/repl/state.rs

pub struct ReplState {
    pub connection: Connection,
    pub config: FormatOptions,
    pub timing: bool,
    pub pager_enabled: bool,
    pub metadata_cache: MetadataCache,
}

pub struct MetadataCache {
    pub databases: Vec<String>,
    pub tables: HashMap<String, Vec<TableInfo>>,
    pub columns: HashMap<String, Vec<ColumnInfo>>,
    pub last_refresh: Instant,
}
```

### Editor Setup

Uses `reedline` for line editing:

```rust
let mut editor = Reedline::create()
    .with_history(Box::new(
        FileBackedHistory::with_file(100, history_path)?
    ))
    .with_completer(Box::new(
        MetadataCompleter::new(state.metadata_cache.clone())
    ))
    .with_highlighter(Box::new(
        SqlHighlighter::new()
    ))
    .with_validator(Box::new(
        StatementValidator::new()
    ));
```

### Tab Completion

Context-aware suggestions:

```rust
// src/commands/repl/metadata_completer.rs

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let context = parse_completion_context(line, pos);

        match context {
            CompletionContext::AfterFrom | CompletionContext::AfterJoin => {
                // Suggest table names
                self.cache.tables.keys()
                    .map(|t| Suggestion::new(t.clone()))
                    .collect()
            }
            CompletionContext::AfterSelect => {
                // Suggest column names from tables in FROM clause
                let tables = extract_tables_from_query(line);
                tables.iter()
                    .flat_map(|t| self.cache.columns.get(t))
                    .flatten()
                    .map(|c| Suggestion::new(c.name.clone()))
                    .collect()
            }
            CompletionContext::Keyword => {
                // Suggest SQL keywords
                SQL_KEYWORDS.iter()
                    .map(|k| Suggestion::new(k.to_string()))
                    .collect()
            }
            _ => vec![],
        }
    }
}
```

### Syntax Highlighting

```rust
// src/commands/repl/highlighter.rs

impl Highlighter for SqlHighlighter {
    fn highlight(&self, line: &str) -> StyledText {
        let mut styled = StyledText::new();

        for token in tokenize_sql(line) {
            let style = match token.kind {
                TokenKind::Keyword => Style::new().fg(Color::Blue).bold(),
                TokenKind::String => Style::new().fg(Color::Green),
                TokenKind::Number => Style::new().fg(Color::Cyan),
                TokenKind::Comment => Style::new().fg(Color::DarkGray),
                TokenKind::Function => Style::new().fg(Color::Yellow),
                _ => Style::new(),
            };

            styled.push((style, token.text));
        }

        styled
    }
}
```

### Result Paging

```rust
// src/commands/repl/pager.rs

pub fn display_with_pager(output: &str, rows: usize) -> Result<()> {
    if should_page(rows) {
        let mut pager = Pager::new()?;
        pager.set_text(output)?;
        pager.run()?;
    } else {
        println!("{}", output);
    }
    Ok(())
}

fn should_page(rows: usize) -> bool {
    if let Some((_, height)) = terminal::size().ok() {
        rows as u16 > height - 5  // Leave room for prompt
    } else {
        false
    }
}
```

Uses `minus` crate for interactive paging with vi-like keys.

### Statement Execution

```rust
// src/commands/repl/executor.rs

pub fn execute_statement(
    state: &mut ReplState,
    sql: &str,
) -> Result<ExecutionResult> {
    let start = Instant::now();

    // Execute query
    let result = state.connection.execute(sql)?;

    // Format output
    let output = format_result(&result, &state.config)?;

    let execution_result = ExecutionResult {
        output,
        row_count: result.row_count,
        execution_time: start.elapsed(),
    };

    // Display with paging if needed
    display_with_pager(&execution_result.output, result.row_count)?;

    // Show timing if enabled
    if state.timing {
        eprintln!(
            "\n{} rows in set ({:.3}s)",
            result.row_count,
            execution_result.execution_time.as_secs_f64()
        );
    }

    Ok(execution_result)
}
```

## Metacommands

Special commands prefixed with backslash:

```rust
pub enum Metacommand {
    ListDatabases,          // \l
    ListTables(Option<String>), // \dt [pattern]
    Describe(String),       // \d table_name
    ToggleTiming,          // \timing
    ToggleExpanded,        // \x
    Help(Option<String>),  // \h [topic]
    Quit,                  // \q
}

pub fn parse_metacommand(input: &str) -> Option<Metacommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('\\') {
        return None;
    }

    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    match parts.first()? {
        &"l" => Some(Metacommand::ListDatabases),
        &"dt" => Some(Metacommand::ListTables(parts.get(1).map(|s| s.to_string()))),
        &"d" => Some(Metacommand::Describe(parts.get(1)?.to_string())),
        &"timing" => Some(Metacommand::ToggleTiming),
        &"x" => Some(Metacommand::ToggleExpanded),
        &"h" => Some(Metacommand::Help(parts.get(1).map(|s| s.to_string()))),
        &"q" => Some(Metacommand::Quit),
        _ => None,
    }
}
```

## Metadata Caching

Fetch metadata at startup for tab completion:

```rust
impl MetadataCache {
    pub fn refresh(&mut self, conn: &Connection) -> Result<()> {
        // List databases
        self.databases = conn.list_databases()?;

        // List tables in current database
        self.tables.clear();
        let tables = conn.list_tables(None)?;
        for table in tables {
            self.tables.insert(table.name.clone(), table);

            // Get columns for each table
            let columns = conn.describe_table(&table.name)?;
            self.columns.insert(table.name.clone(), columns);
        }

        self.last_refresh = Instant::now();
        Ok(())
    }

    pub fn is_stale(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(300)  // 5 minutes
    }
}
```

## Error Handling

REPL continues on errors (unlike batch mode):

```rust
loop {
    match editor.readline(&prompt) {
        Ok(Signal::Success(input)) => {
            match execute_statement(&mut state, &input) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error: {}", e);
                    // Continue REPL loop
                }
            }
        }
        Ok(Signal::CtrlC) => {
            // Cancel current input, show new prompt
            continue;
        }
        Ok(Signal::CtrlD) | Err(_) => {
            // Exit REPL
            break;
        }
    }
}
```

## Code Linkage

| Component | File Path | Key Types |
|-----------|-----------|-----------|
| REPL orchestration | `src/commands/repl/mod.rs` | `execute()`, main loop |
| Statement executor | `src/commands/repl/executor.rs` | `execute_statement()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `MetadataCompleter` |
| Syntax highlighting | `src/commands/repl/highlighter.rs` | `SqlHighlighter` |
| Result pager | `src/commands/repl/pager.rs` | `display_with_pager()` |
| REPL state | `src/commands/repl/state.rs` | `ReplState`, `MetadataCache` |

## Design Trade-offs

### Persistent Connection
**Chosen**: Single connection for entire session
**Alternative**: Reconnect per query
**Rationale**: Performance, session state preservation

### Metadata Caching
**Chosen**: Cache at startup, manual refresh
**Alternative**: Query on-demand
**Rationale**: Fast completion, reduced network calls

### Paging Strategy
**Chosen**: Automatic paging based on terminal height
**Alternative**: Always page or never page
**Rationale**: Optimal UX for different result sizes

## Tab Completion Caching Architecture

### Overview

Tab completion provides context-aware suggestions for database names, table names, and column names. The completion system uses a multi-tiered caching strategy to balance responsiveness with data freshness.

**Sprint 20 Critical Fix**: Database names are now loaded at REPL startup, BEFORE the reedline editor is initialized. This prevents TTY conflicts that caused "Page 1: records 0 - 0" output during tab completion.

### Architecture

```
REPL Startup Sequence (Sprint 20):

Connect to Database
        ↓
Create CompletionState
        ↓
Load Database Names (CRITICAL: Before Editor Init)
        ↓
Show Banner
        ↓
Initialize reedline Editor
        ↓
Start REPL Loop

Tab Completion Flow:

User presses TAB
        ↓
Analyze SQL Context
        ↓
┌───────┴───────────────────┐
↓                           ↓
After FROM/JOIN?     After SELECT/WHERE?
↓                           ↓
Need Database/Table    Need Columns
↓                           ↓
Check Database Cache   Check Column Cache
(pre-loaded at startup)
↓                           ↓
┌───────┴───────┐    ┌───────┴───────┐
↓               ↓    ↓               ↓
Always Hit   Need Tables  Cache Hit  Cache Miss
(pre-loaded)  (lazy load)     ↓          ↓
    ↓              ↓      Return    Load from
Return         Load from  Suggestions  Database
Databases      Database      (suppressed output)
    ↓         (suppressed)
    +              ↓
Return        Cache Tables
Tables in         ↓
Current DB   Return Tables
```

### Cache Structure

```rust
// src/db/metadata.rs

pub struct MetadataCache {
    /// Database names - LOADED AT STARTUP (Sprint 20)
    /// Pre-loaded before editor init to avoid TTY conflicts
    databases: Option<Vec<String>>,

    /// Tables list - loaded lazily on first completion
    tables: Option<Vec<TableInfo>>,

    /// Columns per table: HashMap<"database.table", Vec<ColumnInfo>>
    /// Loaded lazily when user needs column completion
    columns: HashMap<String, Vec<ColumnInfo>>,

    /// Timestamp of database list load
    databases_loaded_at: Option<Instant>,

    /// Timestamp of table list load
    tables_loaded_at: Option<Instant>,

    /// Current database context (for unqualified table names)
    current_database: String,
}
```

### Loading Strategy

#### Database Names (Sprint 20 - Pre-loaded at Startup)
- **When**: At REPL startup, BEFORE editor initialization
- **Why**: Prevents TTY conflicts - the teradatarustapi may output pager messages during queries, which interferes with reedline's terminal handling
- **Query**: `SELECT TRIM(DatabaseName) FROM DBC.DatabasesV WHERE DatabaseName NOT IN (...system databases...)`
- **Cache Duration**: Session lifetime (cleared on /logon)
- **Rationale**: Database list rarely changes, pre-loading ensures no queries during completion

#### Table Names
- **When**: User presses TAB after FROM/JOIN keyword (lazy load)
- **Scope**: Load all user tables (excluding system databases)
- **Query**: `SELECT TRIM(DatabaseName), TRIM(TableName), TableKind FROM DBC.TablesV WHERE ...`
- **Cache Duration**: Session lifetime
- **Rationale**: Tables may be large, lazy load balances startup time vs. completion latency
- **Output Suppression**: Uses `OutputSuppressor` to redirect stdout/stderr during query

#### Table Names for Specific Database
- **When**: User types `database.` and presses TAB
- **Scope**: Filter from cached table list
- **No Additional Query**: Uses already-cached table data
- **Rationale**: Avoids additional network roundtrip

#### Column Names
- **When**: User presses TAB in column context (SELECT, WHERE, etc.)
- **Scope**: Load columns for specific table only when needed
- **Query**: `SELECT TRIM(ColumnName), ColumnType FROM DBC.ColumnsV WHERE DatabaseName = ? AND TableName = ?`
- **Cache Duration**: Session lifetime per table
- **Rationale**: Column lists can be large, lazy load minimizes memory

### Output Suppression and TTY Conflict Resolution

The teradatarustapi library (Go-based FFI) may print debug output during query execution:
```
Page 1: records 0 - 0  total: 0
```

**Sprint 20 Solution (Iteration 2)**: The fix is to pre-load ALL metadata at REPL startup, BEFORE the reedline editor is initialized, AND ensure that tab completion NEVER triggers database queries.

**Key Design Principles:**
1. **All metadata pre-loaded at startup**: Both database names AND table metadata are loaded before reedline initializes
2. **Zero queries during completion**: The completion code uses ONLY cached data - if data isn't cached, it returns empty rather than querying
3. **Startup loading is safe**: Any driver output during startup is harmless (before terminal is in raw mode)

**Implementation in `src/commands/repl/mod.rs`:**
```rust
pub fn execute(...) -> Result<()> {
    // Create completion state
    let completion_state = Arc::new(Mutex::new(CompletionState::new(client, database)));

    // Pre-load ALL metadata BEFORE editor initialization
    {
        let mut cs = completion_state.lock().unwrap();
        cs.ensure_databases_loaded();  // Load database names
        cs.ensure_tables_loaded();     // Load table metadata
    }

    // NOW initialize reedline (after all queries complete)
    let mut editor = create_editor(args, writer, Arc::clone(&completion_state))?;
    // ...
}
```

**Implementation in `src/commands/repl/metadata_completer.rs`:**
```rust
fn complete_tables(&self, prefix: &str) -> Vec<Suggestion> {
    // Use ONLY cached data - NO queries
    if state.cache().has_databases() {
        // Return cached database completions
    }
    if state.cache().has_tables() {
        // Return cached table completions
    }
    // If not cached, return empty (don't query!)
}
```

**Why File Descriptor Redirection Failed**: The `OutputSuppressor` approach (redirecting stdout/stderr to `/dev/null`) did not reliably suppress the pager output. This may be because:
- The Go library buffers output before the redirect takes effect
- There are timing issues with the CGO bridge
- The library may write directly to the controlling TTY in some circumstances

**The Robust Solution**: By ensuring tab completion NEVER triggers queries, we eliminate the problem at its source. The trade-off is slightly longer startup time, but this provides a much better user experience during interactive use.

### Completion Context Analysis

The `sql_context` module analyzes the SQL being typed to determine what completions are relevant:

```rust
// src/commands/repl/sql_context.rs

pub enum CompletionContext {
    /// After FROM, JOIN keywords - suggest databases + tables
    TableName { prefix: String },

    /// After "database." - suggest tables in that database
    SchemaQualifiedTable { schema: String, prefix: String },

    /// After SELECT, WHERE, ORDER BY - suggest columns
    ColumnName {
        tables: Vec<TableReference>,
        prefix: String,
        table_qualifier: Option<String>,
    },

    /// Generic keyword completion
    Keyword,
}

pub fn analyze_context(sql: &str, cursor_pos: usize) -> CompletionContext {
    // Tokenize SQL
    // Find cursor position relative to keywords
    // Determine appropriate context
}
```

### Integration with reedline

```rust
// src/commands/repl/metadata_completer.rs

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let context = analyze_context(line, pos);

        match context {
            CompletionContext::TableName { prefix } => {
                // Ensure databases loaded
                // Return databases + tables in current db
                self.complete_tables(&prefix)
            }
            CompletionContext::SchemaQualifiedTable { schema, prefix } => {
                // Ensure tables loaded for this database
                // Return tables in specified database
                self.complete_schema_tables(&schema, &prefix)
            }
            CompletionContext::ColumnName { tables, prefix, .. } => {
                // Ensure columns loaded for referenced tables
                // Return matching columns
                self.complete_columns(&tables, &prefix)
            }
            CompletionContext::Keyword => {
                // Return empty - keyword completion disabled
                vec![]
            }
        }
    }
}
```

### Design Trade-offs

#### Lazy Loading vs Eager Loading
**Chosen**: Lazy loading for tables and columns
**Alternative**: Load all metadata at startup
**Rationale**:
- Large databases may have thousands of tables
- Users typically work with subset of databases
- Reduces startup time and memory usage
- Trade-off: First completion in new context slightly slower

#### Per-Database vs Global Table Cache
**Chosen**: Per-database caching
**Alternative**: Single global table list
**Rationale**:
- Teradata uses `database.table` naming model
- Reduces memory for multi-database environments
- Enables targeted cache invalidation

#### Output Suppression Strategy
**Chosen**: File descriptor redirection (dup2 to /dev/null)
**Alternative**: Environment variable, connection parameter
**Rationale**:
- Works without driver modifications
- RAII pattern ensures cleanup even on panic
- Platform-specific (Unix only), Windows no-op

### Error Handling

Metadata loading failures should NOT block tab completion:

```rust
pub fn ensure_tables_loaded(&mut self) -> bool {
    if self.cache.has_tables() {
        return true;
    }

    match self.cache.load_tables(&self.client) {
        true => {
            log::debug!("Tables loaded successfully");
            true
        }
        false => {
            // Log error but don't crash
            log::warn!("Failed to load tables: {}",
                self.cache.last_error().unwrap_or("unknown"));
            false
        }
    }
}
```

### Cache Invalidation

Cache is cleared on:
1. `/logon` - New connection established
2. Session change to different database
3. (Future) After DDL statements (CREATE, DROP, ALTER)

### Performance Targets

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Cache hit completion | < 50ms | Instant feel |
| Database list load | < 500ms | Acceptable first-time delay |
| Table list load | < 500ms | Per-database, lazy |
| Column list load | < 300ms | Per-table, lazy |

## Future Enhancements

- Multi-line editing with visual indicators
- Query history search (Ctrl-R)
- Result export from REPL (\export)
- Session transcripts (\spool)
- Variable substitution (\set, \unset)
- Transaction control (\begin, \commit, \rollback)
- Async metadata loading (background thread)
- DDL-triggered cache invalidation
- Fuzzy matching for completion (like pgcli)
