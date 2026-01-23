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

## Sprint 21: Tab Completion Quality Enhancements

This section documents the technical design for Sprint 21 tab completion improvements.

### Feature 1: Complete Database Metadata Fetching (P0)

**Issue**: System databases like `dbc` are missing from tab completion.

**Root Cause Analysis**: The current query in `src/db/metadata.rs` (`load_databases()`) explicitly excludes system databases via a hardcoded exclusion list:

```sql
SELECT TRIM(DatabaseName)
FROM DBC.DatabasesV
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', 'DBC', ...)
```

The exclusion of `DBC` was intentional to reduce noise, but users need `dbc` for system queries.

**Solution Design**:

1. **Remove `DBC` from exclusion list**: The `dbc` database is a legitimate completion target for advanced users querying system views.

2. **Keep other system databases excluded**: Databases like `Console`, `Crashdumps`, `SYSBAR`, etc. provide no value for completion and add noise.

3. **Implementation location**: `src/db/metadata.rs`, function `load_databases()`, lines 461-474.

**Code Change**:
```rust
// Before (Sprint 20):
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', 'DBC', ...)

// After (Sprint 21):
WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', ...)
// Note: 'DBC' REMOVED from exclusion list
```

**Complexity**: Low (single line SQL modification)

---

### Feature 2: Universal Table Metadata Fetching (P0)

**Issue**: Some databases like `demo_user` show "NO RECORDS FOUND" even when tables exist.

**Root Cause Analysis**: The current query in `src/db/metadata.rs` (`load_tables()`) has two filtering issues:

1. **System database exclusion**: Same exclusion list as database names, but for tables. If a user database name happens to match a pattern, it could be filtered.

2. **SAMPLE limit**: The query uses `SAMPLE 10000` which may not capture all tables in large environments, especially if those 10000 samples come from other databases first.

**Investigation Required**: Need to verify if `demo_user` is in the exclusion list or if there's a SAMPLE issue.

**Solution Design**:

1. **Verify database name**: Ensure `demo_user` is not accidentally filtered.

2. **Increase SAMPLE or remove limit**: For environments with many tables, 10000 may be insufficient. Consider removing the SAMPLE limit or increasing it substantially.

3. **Alternative: On-demand loading per database**: Instead of loading ALL tables at startup, load tables for a specific database when user types `database.` + TAB. This is more scalable.

**Recommended Approach - On-Demand Loading**:

```
Current Flow:
REPL Start → Load ALL tables (up to 10000) → Cache

New Flow (Sprint 21):
REPL Start → Load Database Names only
User types "demo_user." + TAB → Load tables for demo_user → Cache
Subsequent "demo_user." + TAB → Use cached data
```

**Implementation**:

1. **Keep startup database loading** (Feature 1 fix)
2. **Make table loading per-database on-demand**:
   - Add `load_tables_for_database(&self, client: &DatabaseClient, database: &str) -> bool`
   - Cache structure: `tables_by_database: HashMap<String, Vec<TableInfo>>`
   - On `SchemaQualifiedTable { schema, prefix }` context, check if `schema` is cached, load if not

**Code Structure**:
```rust
// src/db/metadata.rs
impl MetadataCache {
    /// Per-database table cache
    tables_by_database: HashMap<String, Vec<TableInfo>>,

    /// Load tables for a specific database (on-demand)
    pub fn load_tables_for_database(&mut self, client: &DatabaseClient, database: &str) -> bool {
        if self.tables_by_database.contains_key(&database.to_uppercase()) {
            return true; // Already cached
        }

        let sql = format!(r#"
            SELECT TRIM(TableName), TableKind
            FROM DBC.TablesV
            WHERE UPPER(DatabaseName) = UPPER('{}')
              AND TableKind IN ('T', 'V', 'O')
            ORDER BY TableName
        "#, escape_sql_string(database));

        // Execute and cache...
    }
}
```

**Complexity**: Medium (requires architecture change to per-database caching)

---

### Feature 3: Second TAB Accepts Selection (P1)

**Issue**: Second TAB moves to next item instead of accepting selection (bash/zsh behavior).

**Root Cause Analysis**: The current TAB keybinding uses:

```rust
// src/commands/repl/mod.rs, add_completion_keybinding()
keybindings.add_binding(
    KeyModifiers::NONE,
    KeyCode::Tab,
    ReedlineEvent::UntilFound(vec![
        ReedlineEvent::Menu("completion_menu".to_string()),
        ReedlineEvent::MenuNext,
    ]),
);
```

This means:
- **First TAB**: `Menu("completion_menu")` activates menu
- **Second TAB**: Menu already active, so `UntilFound` tries `Menu` (inapplicable), then executes `MenuNext` (move to next item)

**Reedline Investigation Results**:

After investigating reedline v0.38.0 source code and GitHub issues:

1. **No `MenuAccept` event exists**: reedline does not have a dedicated event for accepting menu selection ([GitHub Issue #624](https://github.com/nushell/reedline/issues/624) - OPEN as of 2024).

2. **`Enter` when menu is active**: This calls `replace_in_buffer()` then `MenuEvent::Deactivate`, which IS the accept behavior (reedline engine.rs lines 1096-1107).

3. **Problem**: We cannot distinguish "second TAB while menu open" from "first TAB" at the keybinding level.

**Feasibility Assessment**: **AT RISK / NOT FEASIBLE with current reedline**

The requested behavior (second TAB accepts selection) requires one of:
- A new `MenuAccept` event in reedline (upstream change)
- Custom menu implementation that tracks TAB press count
- Fork of reedline with custom event handling

**Alternative Approaches Investigated**:

| Approach | Feasibility | Complexity | Notes |
|----------|-------------|------------|-------|
| Upstream PR to add `MenuAccept` | Medium | High | Would require reedline maintainer buy-in |
| Fork reedline | High | Very High | Maintenance burden, version drift |
| Custom `EditMode` | Low | Very High | Would need to reimplement all keybindings |
| Track state in completer | Not Possible | N/A | Completer doesn't control keybindings |
| Bind TAB to `Enter` when menu open | Not Possible | N/A | No conditional binding mechanism |

**Recommendation**: **DEFER TO FUTURE SPRINT**

This feature requires upstream reedline changes. The recommended path is:
1. Document limitation clearly for user
2. Submit feature request to reedline (reference existing issue #624)
3. Consider contributing a PR to reedline if prioritized
4. Interim workaround: Users can press Enter to accept selection

**User Communication**:
```
Current behavior: TAB cycles through completions, Enter accepts
Bash/zsh behavior: Second TAB accepts
Status: Requires reedline library enhancement (tracked upstream)
```

---

### Feature 4: Smart Database-Dot-TAB Completion (P1)

**Issue**: User wants `dem` + TAB to complete to `demo_user.` and immediately show tables.

**Current Behavior**:
1. `dem` + TAB → Shows `demo_user` in menu
2. User navigates, presses Enter → `demo_user` inserted
3. User types `.` + TAB → Shows tables in `demo_user`

**Desired Behavior**:
1. `dem` + TAB → If only one match (`demo_user`), auto-complete to `demo_user.` and immediately show tables

**Feasibility Assessment**: **FEASIBLE**

This can be achieved by modifying the completion logic.

**Solution Design**:

1. **Detect unique database match**: In `complete_tables()`, if:
   - Context is `TableName` (after FROM/JOIN)
   - Exactly ONE database matches the prefix
   - No tables match the prefix in current database

   Then: Return the database name WITH trailing dot, and trigger table completion.

2. **Challenge**: reedline completer returns suggestions, it doesn't control follow-up actions.

3. **Alternative Implementation**: Use `append_whitespace: false` for database suggestions (already done), and enhance `SchemaQualifiedTable` handling to work seamlessly when user types the dot.

**Detailed Design**:

The key insight is that Feature 4 is partially solved if:
- Database completions already have `append_whitespace: false` (they do)
- After accepting `demo_user`, user types `.`
- On `.` + TAB, we enter `SchemaQualifiedTable { schema: "demo_user", prefix: "" }` context
- This triggers table loading for that database (Feature 2 on-demand loading)

**What's Missing**: The "auto-add dot and show tables" part. This requires the completer to:
1. Recognize single-match scenario
2. Append `.` to the completion value
3. Somehow trigger immediate re-completion

**Approach**: Modify suggestion value to include the dot when appropriate.

```rust
// In complete_tables():
if databases.len() == 1 && prefix_matches_database_exactly {
    // Single database match - append dot to enable quick table access
    suggestions.push(Suggestion {
        value: format!("{}.", db_name), // Include dot
        description: Some("(database - press TAB for tables)".to_string()),
        append_whitespace: false,
        ...
    });
}
```

**After selection**: User sees `demo_user.` and can immediately TAB again for tables.

**Complexity**: Medium (requires careful edge case handling)

**Edge Cases**:
- Multiple database matches: Don't add dot (user needs to disambiguate first)
- Database prefix also matches table name: Show both options
- Empty prefix: Show all databases without dots

---

### Feature 5: Automated Regression Testing (P2)

**Design Guidance for quality-validator**:

**Test Categories**:

1. **Unit Tests** (in `src/db/metadata.rs`, `src/commands/repl/metadata_completer.rs`):
   - `test_load_databases_includes_dbc` - Verify DBC not filtered
   - `test_load_tables_for_database` - Verify per-database loading
   - `test_completion_context_analysis` - Context detection
   - `test_database_suggestion_format` - Verify dot handling

2. **Integration Tests** (with mock or test database):
   - `test_tab_completion_shows_dbc`
   - `test_tab_completion_for_user_database`
   - `test_schema_qualified_completion`

3. **Manual Validation Required** (due to reedline TTY interaction):
   - Visual verification of menu display
   - TAB key behavior (navigation vs acceptance)
   - No pager output during completion

**Test File Locations**:
- `src/db/metadata.rs` - Unit tests in `mod tests`
- `tests/cases/TC-TAB-*.md` - Test case documentation
- `tests/results/sprint-21/` - Execution evidence

---

## Sprint 21 Implementation Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Feature 1: Include `dbc` | IMPLEMENTED | Removed 'DBC' from exclusion list in `load_databases()` and `load_tables()` |
| Feature 2: Universal tables | IMPLEMENTED | On-demand per-database loading via `load_tables_for_database()` |
| Feature 3: Second TAB accepts | DEFERRED | Blocked by reedline Issue #624 - no `MenuAccept` event |
| Feature 4: Smart database.TAB | IMPLEMENTED | Appends '.' to single-match database suggestions |
| Feature 5: Testability | IMPLEMENTED | Added unit tests for all new functionality |

**Implementation Details**:

1. **Feature 1** (lines 464, 392 in `src/db/metadata.rs`):
   - Removed 'DBC' from the exclusion lists in both `load_databases()` and `load_tables()` queries
   - `dbc` database now appears in tab completion

2. **Feature 2** (new methods in `src/db/metadata.rs`):
   - Added `tables_by_database: HashMap<String, Vec<TableInfo>>` to `MetadataCache`
   - Added `load_tables_for_database()` for on-demand loading
   - Added `has_tables_for_database()`, `get_tables_for_database()`, `find_tables_in_database_by_prefix()`
   - Updated `complete_schema_tables()` in `metadata_completer.rs` to trigger on-demand loading

3. **Feature 4** (in `complete_tables()` in `metadata_completer.rs`):
   - When exactly one database matches prefix and no tables match
   - Appends '.' to suggestion value: `format!("{}.", db_name)`
   - Description shows "(database - TAB for tables)"

4. **Feature 5** (unit tests added):
   - `test_has_tables_for_database`
   - `test_get_tables_for_database`
   - `test_find_tables_in_database_by_prefix`
   - `test_metadata_cache_clear_clears_per_database_tables`
   - `test_dbc_not_in_exclusion_list`

---

## Metacommand Tab Completion

This section documents the technical design for metacommand tab completion, enabling users to type `/des<TAB>` and see `/describe` in the completion menu.

### Architecture

Metacommand completion integrates with the existing `MetadataCompleter` by detecting when the input starts with `/` or `\` and providing metacommand suggestions instead of SQL completions.

```
Tab Completion Decision Flow:

User presses TAB
        |
        v
Check if line starts with '/' or '\'
        |
    +---+---+
    |       |
    v       v
  YES       NO
    |       |
    v       v
Metacommand   SQL Context
Completion    Completion
    |       (existing)
    v
Filter metacommands by prefix
    |
    v
Return suggestions with descriptions
```

### Implementation Location

**Primary file**: `src/commands/repl/metadata_completer.rs`

**Extension points**:
1. Add metacommand detection in `complete()` method (around line 489)
2. Add `complete_metacommands()` helper method
3. Define metacommand registry with names, aliases, and descriptions

### Metacommand Registry

```rust
// src/commands/repl/metadata_completer.rs

/// Metacommand definition for completion
struct MetacommandDef {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
}

/// Registry of all available metacommands
const METACOMMANDS: &[MetacommandDef] = &[
    MetacommandDef { name: "help", aliases: &["?"], description: "Show help message" },
    MetacommandDef { name: "quit", aliases: &["q", "exit"], description: "Exit the REPL" },
    MetacommandDef { name: "session", aliases: &[], description: "Show session information" },
    MetacommandDef { name: "ping", aliases: &[], description: "Test database connection" },
    MetacommandDef { name: "describe", aliases: &["d"], description: "Describe table structure" },
    MetacommandDef { name: "export", aliases: &[], description: "Export query results" },
    MetacommandDef { name: "pager", aliases: &[], description: "Toggle result paging" },
    MetacommandDef { name: "colors", aliases: &[], description: "Toggle syntax highlighting" },
    MetacommandDef { name: "logon", aliases: &[], description: "Switch database connection" },
    // Sprint 22 additions:
    MetacommandDef { name: "list databases", aliases: &["l"], description: "List all databases" },
    MetacommandDef { name: "list tables", aliases: &["dt"], description: "List tables in database" },
    MetacommandDef { name: "list views", aliases: &["dv"], description: "List views in database" },
];
```

### Completion Logic

```rust
/// Complete metacommands
fn complete_metacommands(&self, prefix: &str) -> Vec<Suggestion> {
    let prefix_lower = prefix.to_lowercase();

    METACOMMANDS
        .iter()
        .filter(|cmd| {
            cmd.name.starts_with(&prefix_lower) ||
            cmd.aliases.iter().any(|a| a.starts_with(&prefix_lower))
        })
        .map(|cmd| Suggestion {
            value: format!("/{}", cmd.name),
            description: Some(cmd.description.to_string()),
            style: None,
            extra: None,
            span: reedline::Span { start: 0, end: 0 }, // Set by caller
            append_whitespace: cmd.name.contains(' '), // Space for commands with args
        })
        .collect()
}
```

### Multi-word Metacommand Handling

Commands like `/list tables` require special handling:

1. First TAB after `/list` shows subcommands: `databases`, `tables`, `views`
2. Subcommand completion uses same registry pattern
3. Space-separated parts treated as single command

```rust
/// Check if completing a multi-word metacommand
fn complete_metacommand_subcommand(&self, prefix: &str) -> Vec<Suggestion> {
    let parts: Vec<&str> = prefix.split_whitespace().collect();

    match parts.as_slice() {
        ["list"] | ["list", ""] => {
            // Show subcommands: databases, tables, views
            vec![
                Suggestion { value: "/list databases".into(), description: Some("List all databases".into()), .. },
                Suggestion { value: "/list tables".into(), description: Some("List tables".into()), .. },
                Suggestion { value: "/list views".into(), description: Some("List views".into()), .. },
            ]
        }
        ["list", partial] => {
            // Filter subcommands by partial match
            let subcommands = ["databases", "tables", "views"];
            subcommands.iter()
                .filter(|s| s.starts_with(&partial.to_lowercase()))
                .map(|s| Suggestion { value: format!("/list {}", s), .. })
                .collect()
        }
        _ => Vec::new()
    }
}
```

---

## Schema Inspection Commands

This section documents the technical design for `/list databases`, `/list tables [pattern]`, and `/list views` commands.

### Architecture

Schema commands query Teradata system catalog views and format results for display. They integrate with the existing metacommand handler in `src/commands/repl/metacommands.rs`.

```
Schema Command Flow:

User types "/list tables emp%"
        |
        v
Parse metacommand (handle_metacommand_with_state)
        |
        v
Match "list" command with args ["tables", "emp%"]
        |
        v
Call execute_list_tables(client, pattern, writer)
        |
        v
Build SQL query for DBC.TablesV
        |
        v
Execute query via DatabaseClient
        |
        v
Format results as columnar output
        |
        v
Display to user
```

### Implementation Location

**Primary file**: `src/commands/repl/metacommands.rs`

**New functions**:
- `execute_list_databases()`
- `execute_list_tables(pattern: Option<&str>)`
- `execute_list_views()`

### `/list databases` Implementation

```rust
/// Execute /list databases
///
/// Queries DBC.DatabasesV and displays all accessible databases.
fn execute_list_databases<W: Write>(
    client: &DatabaseClient,
    writer: &mut W,
) -> Result<()> {
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name,
               OwnerName,
               CommentString
        FROM DBC.DatabasesV
        WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps', ...)
        ORDER BY DatabaseName
    "#;

    match client.execute(sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Databases ({} total):", result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                let owner = row.get(1).map(|v| v.display()).unwrap_or_default();
                writeln!(writer, "  {:<30} (owner: {})", name, owner)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing databases: {}", e)?;
        }
    }
    Ok(())
}
```

### `/list tables [pattern]` Implementation

```rust
/// Execute /list tables [pattern]
///
/// Lists tables in current database, with optional glob pattern filtering.
/// Pattern supports:
/// - `*` matches any characters
/// - `?` matches single character
/// - `dbc.*` matches database prefix
fn execute_list_tables<W: Write>(
    client: &DatabaseClient,
    pattern: Option<&str>,
    current_database: &str,
    writer: &mut W,
) -> Result<()> {
    // Determine database context and table filter
    let (database, table_pattern) = parse_table_pattern(pattern, current_database);

    // Convert glob to SQL LIKE pattern
    let like_pattern = glob_to_sql_like(&table_pattern);

    let sql = format!(r#"
        SELECT TRIM(TableName) AS table_name,
               TableKind,
               CommentString
        FROM DBC.TablesV
        WHERE UPPER(DatabaseName) = UPPER('{}')
          AND TableKind IN ('T', 'V', 'O')
          AND TableName LIKE '{}'
        ORDER BY TableName
    "#, escape_sql_string(&database), like_pattern);

    match client.execute(&sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Tables in '{}' ({} found):", database, result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                let kind = row.get(1).map(|v| format_table_kind(&v.display())).unwrap_or_default();
                writeln!(writer, "  {:<40} ({})", name, kind)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing tables: {}", e)?;
        }
    }
    Ok(())
}

/// Parse table pattern to extract database and table filter
fn parse_table_pattern(pattern: Option<&str>, current_db: &str) -> (String, String) {
    match pattern {
        Some(p) if p.contains('.') => {
            let parts: Vec<&str> = p.splitn(2, '.').collect();
            (parts[0].to_string(), parts.get(1).unwrap_or(&"*").to_string())
        }
        Some(p) => (current_db.to_string(), p.to_string()),
        None => (current_db.to_string(), "*".to_string()),
    }
}

/// Convert glob pattern to SQL LIKE pattern
fn glob_to_sql_like(pattern: &str) -> String {
    pattern
        .replace('*', "%")
        .replace('?', "_")
}
```

### `/list views` Implementation

```rust
/// Execute /list views
///
/// Lists views in current database (TableKind = 'V').
fn execute_list_views<W: Write>(
    client: &DatabaseClient,
    current_database: &str,
    writer: &mut W,
) -> Result<()> {
    let sql = format!(r#"
        SELECT TRIM(TableName) AS view_name,
               CommentString
        FROM DBC.TablesV
        WHERE UPPER(DatabaseName) = UPPER('{}')
          AND TableKind = 'V'
        ORDER BY TableName
    "#, escape_sql_string(current_database));

    match client.execute(&sql) {
        Ok(result) => {
            writeln!(writer)?;
            writeln!(writer, "Views in '{}' ({} found):", current_database, result.row_count)?;
            writeln!(writer)?;

            for row in &result.rows {
                let name = row.first().map(|v| v.display()).unwrap_or_default();
                writeln!(writer, "  {}", name)?;
            }
            writeln!(writer)?;
        }
        Err(e) => {
            writeln!(writer, "Error listing views: {}", e)?;
        }
    }
    Ok(())
}
```

### Metacommand Handler Integration

Update `handle_metacommand_with_state()` to handle the new commands:

```rust
// In handle_metacommand_with_state()
match command.as_str() {
    // ... existing commands ...

    "list" => {
        if args.is_empty() {
            writeln!(writer, "Usage: /list databases | tables [pattern] | views")?;
        } else {
            match args[0].to_lowercase().as_str() {
                "databases" | "database" => {
                    execute_list_databases(completion_state.client(), writer)?;
                }
                "tables" | "table" => {
                    let pattern = args.get(1).map(|s| *s);
                    let current_db = &state.connection_info().database;
                    execute_list_tables(completion_state.client(), pattern, current_db, writer)?;
                }
                "views" | "view" => {
                    let current_db = &state.connection_info().database;
                    execute_list_views(completion_state.client(), current_db, writer)?;
                }
                _ => {
                    writeln!(writer, "Unknown list target: {}", args[0])?;
                    writeln!(writer, "Usage: /list databases | tables [pattern] | views")?;
                }
            }
        }
    }

    // Aliases
    "l" => { /* delegate to list databases */ }
    "dt" => { /* delegate to list tables */ }
    "dv" => { /* delegate to list views */ }
}
```

---

## Loading Indicator for Slow Metadata Fetches

This section documents the technical design for displaying a loading indicator when metadata queries take longer than 500ms.

### Architecture

The loading indicator uses a background thread to display progress while the main thread executes the metadata query. This provides user feedback during slow network operations.

```
Loading Indicator Flow:

User types "database." + TAB
        |
        v
Check if database tables cached
        |
    +---+---+
    |       |
    v       v
  YES       NO
    |       |
    v       v
Return     Start loading indicator thread
cached     |
           v
           Display "Loading tables from <database>..."
           |
           v
           Execute metadata query
           |
           v
           Stop indicator, clear line
           |
           v
           Show completions
```

### Implementation Location

**Primary file**: `src/db/metadata.rs` (loading logic)
**Secondary file**: `src/commands/repl/metadata_completer.rs` (UI feedback)

### Design Approach

Two approaches were considered:

1. **Background thread with channel** (Chosen)
   - Spawn thread to display spinner
   - Main thread executes query
   - Signal thread when complete
   - Pros: Non-blocking, responsive
   - Cons: Thread overhead, complexity

2. **Timeout-based display**
   - Start timer before query
   - If timer exceeds threshold, print message
   - Pros: Simple
   - Cons: Message may appear mid-query, flicker

### Implementation

```rust
// src/db/metadata.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Threshold for showing loading indicator
const LOADING_INDICATOR_THRESHOLD: Duration = Duration::from_millis(500);

/// Execute a query with loading indicator for slow operations
fn execute_with_loading_indicator<T, F>(
    message: &str,
    threshold: Duration,
    operation: F,
) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let indicator_shown = Arc::new(AtomicBool::new(false));
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Clone for thread
    let indicator_shown_clone = Arc::clone(&indicator_shown);
    let stop_signal_clone = Arc::clone(&stop_signal);
    let message = message.to_string();

    // Spawn indicator thread
    let handle = thread::spawn(move || {
        thread::sleep(threshold);

        if !stop_signal_clone.load(Ordering::Relaxed) {
            eprint!("\r{}", message);
            let _ = std::io::Write::flush(&mut std::io::stderr());
            indicator_shown_clone.store(true, Ordering::Relaxed);
        }
    });

    // Execute operation
    let result = operation();

    // Stop indicator
    stop_signal.store(true, Ordering::Relaxed);
    let _ = handle.join();

    // Clear indicator line if shown
    if indicator_shown.load(Ordering::Relaxed) {
        eprint!("\r{}\r", " ".repeat(message.len()));
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    result
}
```

### Integration with On-Demand Loading

```rust
// src/db/metadata.rs

impl MetadataCache {
    pub fn load_tables_for_database(&mut self, client: &DatabaseClient, database: &str) -> bool {
        let db_upper = database.to_uppercase();

        if self.tables_by_database.contains_key(&db_upper) {
            return true;
        }

        let message = format!("Loading tables from {}...", database);

        execute_with_loading_indicator(
            &message,
            LOADING_INDICATOR_THRESHOLD,
            || {
                // Existing query logic...
                let sql = format!(r#"
                    SELECT TRIM(TableName), TableKind
                    FROM DBC.TablesV
                    WHERE UPPER(DatabaseName) = UPPER('{}')
                    AND TableKind IN ('T', 'V', 'O')
                    ORDER BY TableName
                "#, escape_sql_string(database));

                // Execute and cache...
            }
        )
    }
}
```

### Considerations

1. **Thread safety**: Uses atomics for cross-thread communication
2. **Terminal handling**: Writes to stderr to avoid interfering with completion output
3. **Cleanup**: Clears indicator line on completion
4. **reedline compatibility**: Must not interfere with terminal raw mode

---

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
- Second TAB accepts selection (requires reedline enhancement)
