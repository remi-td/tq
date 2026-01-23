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

## Future Enhancements

- Multi-line editing with visual indicators
- Query history search (Ctrl-R)
- Result export from REPL (\export)
- Session transcripts (\spool)
- Variable substitution (\set, \unset)
- Transaction control (\begin, \commit, \rollback)
