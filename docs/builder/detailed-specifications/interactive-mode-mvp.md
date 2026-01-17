# Interactive Mode MVP Specification

**Version:** 1.0.0
**Status:** Approved for Implementation
**Last Updated:** 2026-01-17
**Sprint:** Interactive Mode Phase 1

---

## 1. Overview

### 1.1 Purpose

This document specifies the Minimum Viable Product (MVP) for tq's interactive mode (`tq repl`). The goal is to deliver a functional, high-quality REPL that users can start using immediately, with a foundation for future enhancements.

### 1.2 Scope

**In Scope (This Sprint):**
- Basic REPL loop with prompt
- Multi-line SQL input with semicolon termination
- In-memory command history (arrow key navigation)
- Basic line editing (backspace, cursor movement)
- Query execution with result display
- Essential metacommands: `/quit`, `/help`, `/session`
- Graceful Ctrl-C handling
- Connection status awareness

**Out of Scope (Future Sprints):**
- Syntax highlighting
- Auto-completion
- Persistent history file
- Tab completion
- Vim/Emacs keybindings
- Result paging
- Theming
- ASCII art logo
- Multiple sessions
- Query saving/loading

### 1.3 Design Principles

1. **Simplicity First**: Start with minimal features that work perfectly
2. **Progressive Enhancement**: Build foundation for future features
3. **No Technical Debt**: Clean implementation ready for extension
4. **Consistent UX**: Match existing tq CLI patterns

---

## 2. User Interface Specification

### 2.1 Startup

When user runs `tq repl`:

```
Connected to myhost.example.com:1025
Database: mydb
User: alice
Logon Mechanism: TD2

Type /help for commands, /quit to exit.

tq>
```

**Startup Behavior:**
1. Establish connection using provided credentials
2. Display connection information (host, database, user, logmech)
3. Display help hint
4. Present prompt, ready for input

**Connection Failure:**
```
Error: Failed to connect to myhost.example.com:1025

Reason: Connection refused

Troubleshooting:
  - Check that the hostname and port are correct
  - Verify the database is running
  - Check firewall settings
```
Exit with code 1.

### 2.2 Prompt Design

**Standard Prompt:**
```
tq>
```

**Multi-line Continuation Prompt:**
```
...>
```

**Prompt Changes:**
- When in multi-line input mode (statement not terminated), show `...>`
- Always show `tq>` for new statements

### 2.3 Input Handling

#### 2.3.1 Single-Line SQL

```
tq> SELECT 1 AS col1;
```

Executes immediately when semicolon is entered.

#### 2.3.2 Multi-Line SQL

```
tq> SELECT
...>     employee_id,
...>     first_name
...> FROM employees
...> WHERE department = 'IT';
```

Statement accumulates until semicolon terminates it.

#### 2.3.3 Statement Termination Rules

- **Semicolon (`;`)**: Terminates and executes statement
- **Empty line after semicolon**: Does nothing, shows new prompt
- **Slash on empty line (`/`)**: Executes accumulated buffer (Oracle-style, optional)

#### 2.3.4 Line Editing

**Supported:**
- Backspace: Delete character before cursor
- Left/Right arrows: Move cursor
- Up/Down arrows: Navigate history
- Ctrl-A: Move to beginning of line
- Ctrl-E: Move to end of line
- Ctrl-C: Cancel current input (or exit if no input)
- Ctrl-D: Exit (EOF)

### 2.4 Command History

**Behavior:**
- Store executed SQL statements in memory
- Up arrow navigates to previous statement
- Down arrow navigates to next statement
- History persists for session duration only (MVP)
- Maximum 1000 entries in memory

**What Gets Stored:**
- Successfully executed SQL statements
- Metacommands are NOT stored in history

### 2.5 Query Execution and Results

After semicolon terminates a statement:

```
tq> SELECT user_id, name FROM users LIMIT 3;

+---------+-------+
| user_id | name  |
+---------+-------+
| 1       | Alice |
| 2       | Bob   |
| 3       | Carol |
+---------+-------+

3 rows (0.045s)

tq>
```

**Result Display:**
- Use existing table formatter
- Show row count and execution time
- Return to prompt after results

**Error Handling:**
```
tq> SELECT * FROM nonexistent;

Error: [3807] Table 'nonexistent' does not exist

tq>
```

Errors display on stderr, then return to prompt (don't exit).

### 2.6 Metacommands

Metacommands start with `/` and execute immediately (no semicolon needed).

#### `/help` - Show Help

```
tq> /help

tq REPL Commands:
  /help           Show this help message
  /quit           Exit the REPL
  /session        Show current session information

SQL Execution:
  Enter SQL statements ending with semicolon (;)
  Multi-line statements are supported

Keyboard Shortcuts:
  Up/Down         Navigate command history
  Ctrl-C          Cancel current input
  Ctrl-D          Exit REPL

tq>
```

#### `/quit` - Exit REPL

```
tq> /quit
Goodbye!
```

Exit with code 0.

Also triggered by:
- `/exit`
- `/q`
- Ctrl-D on empty line

#### `/session` - Show Session Info

```
tq> /session

Session Information:
  Host:           myhost.example.com:1025
  Database:       mydb
  User:           alice
  Logon Mechanism: TD2
  Session Start:  2026-01-17 10:30:45
  Queries Run:    5

tq>
```

### 2.7 Graceful Exit

**Ctrl-C Handling:**
- If input buffer is empty: Exit with message "Use /quit or Ctrl-D to exit"
- If input buffer has content: Clear buffer, show new prompt
- During query execution: Cancel query, show message, return to prompt

**Ctrl-D Handling:**
- If input buffer is empty: Exit cleanly
- If input buffer has content: Do nothing (or beep)

**Exit Message:**
```
Goodbye!
```

---

## 3. Technical Specification

### 3.1 Dependencies

**Required Crate:**
- `reedline` (v0.27+): Modern readline implementation in pure Rust
  - Provides line editing, history, prompt handling
  - Cross-platform (Windows, macOS, Linux)
  - Actively maintained, used by Nushell

**Why reedline:**
- Pure Rust (no C dependencies)
- Built-in history support
- Extensible for future features (completion, syntax highlighting)
- Good Windows support
- Active community

### 3.2 Module Structure

```
src/
  commands/
    mod.rs
    ping.rs
    query.rs
    repl/                 # New module
      mod.rs              # REPL command entry point
      prompt.rs           # Prompt handling
      executor.rs         # SQL execution logic
      commands.rs         # Metacommand handling
```

### 3.3 REPL State Machine

```
                 +-------------+
                 |   STARTUP   |
                 +------+------+
                        |
                        v
                 +------+------+
          +----->|  PROMPTING  |<-----+
          |      +------+------+      |
          |             |             |
          |    +--------+--------+    |
          |    |                 |    |
          v    v                 v    |
    +-----------+         +-----------+
    | EXECUTING |         | MULTILINE |
    +-----------+         +-----------+
          |                     |
          +----------+----------+
                     |
                     v
               +-----+-----+
               |  RESULT   |
               +-----+-----+
                     |
                     +---------> back to PROMPTING

Exit States: /quit, Ctrl-D, connection lost
```

### 3.4 Connection Handling

**One Persistent Connection:**
- Establish connection at REPL start
- Reuse connection for all queries
- Reconnect if connection lost (with user notification)
- Clean disconnect on exit

**Connection Loss Detection:**
```
tq> SELECT 1;

Error: Connection lost. Attempting to reconnect...
Reconnected successfully.

tq> SELECT 1;
[Results displayed]
```

### 3.5 Error Handling

| Error Type | User Experience |
|------------|-----------------|
| Connection failure at start | Error message, exit code 1 |
| SQL syntax error | Error message, return to prompt |
| Table not found | Error message, return to prompt |
| Permission denied | Error message, return to prompt |
| Connection lost mid-session | Attempt reconnect, notify user |
| Internal error | Error message, optionally continue |

**Never exit on recoverable errors.** The REPL should be resilient.

### 3.6 Signal Handling

| Signal | Behavior |
|--------|----------|
| SIGINT (Ctrl-C) | Cancel current operation or clear input |
| SIGTERM | Clean exit |
| SIGHUP | Clean exit |

---

## 4. Implementation Notes

### 4.1 Entry Point

```rust
// src/commands/repl/mod.rs

pub fn execute(
    global: &GlobalOpts,
    args: &ReplArgs,
    client: &DatabaseClient,
) -> Result<()> {
    // Initialize reedline
    // Show startup banner
    // Enter REPL loop
    // Clean exit
}
```

### 4.2 REPL Loop Skeleton

```rust
fn repl_loop(
    editor: &mut Reedline,
    client: &DatabaseClient,
    state: &mut ReplState,
) -> Result<()> {
    loop {
        let prompt = if state.is_multiline() {
            "...> "
        } else {
            "tq> "
        };

        match editor.read_line(prompt)? {
            Signal::Success(line) => {
                if line.starts_with('/') {
                    handle_metacommand(&line, state)?;
                } else {
                    state.append_input(&line);
                    if line.trim_end().ends_with(';') {
                        execute_statement(client, state)?;
                    }
                }
            }
            Signal::CtrlC => {
                if state.has_input() {
                    state.clear_input();
                    println!("^C");
                }
            }
            Signal::CtrlD => {
                if !state.has_input() {
                    println!("Goodbye!");
                    break;
                }
            }
        }
    }
    Ok(())
}
```

### 4.3 State Management

```rust
struct ReplState {
    input_buffer: String,
    history_count: usize,
    session_start: Instant,
    queries_executed: usize,
    connection_info: ConnectionInfo,
}
```

---

## 5. Acceptance Criteria

### 5.1 Functional Requirements

- [ ] `tq repl` starts and shows connection info
- [ ] Single-line SQL executes on semicolon
- [ ] Multi-line SQL accumulates and executes on semicolon
- [ ] Results display in table format
- [ ] `/help` shows help text
- [ ] `/quit` exits cleanly
- [ ] `/session` shows session info
- [ ] Up/Down arrows navigate history
- [ ] Ctrl-C cancels input
- [ ] Ctrl-D exits on empty input
- [ ] SQL errors show message and return to prompt
- [ ] Connection errors show message and exit

### 5.2 Non-Functional Requirements

- [ ] REPL starts in < 2 seconds (including connection)
- [ ] Input response is immediate (< 50ms)
- [ ] Memory usage stays constant during session
- [ ] Clean exit with no resource leaks

### 5.3 Test Cases

See testing-guidelines.md for test methodology.

**Core Test Cases:**
1. TC-REPL-001: Start REPL and verify banner
2. TC-REPL-002: Execute single-line SELECT
3. TC-REPL-003: Execute multi-line SELECT
4. TC-REPL-004: Test /help command
5. TC-REPL-005: Test /quit command
6. TC-REPL-006: Test /session command
7. TC-REPL-007: Test history navigation
8. TC-REPL-008: Test Ctrl-C clears input
9. TC-REPL-009: Test Ctrl-D exits
10. TC-REPL-010: Test SQL error handling
11. TC-REPL-011: Test connection failure at start

---

## 6. Future Enhancement Hooks

The MVP implementation should be designed to easily support:

1. **Syntax Highlighting**: reedline supports highlighters
2. **Auto-completion**: reedline supports completers
3. **Persistent History**: reedline has FileBackedHistory
4. **Keybinding Modes**: reedline supports vi/emacs modes
5. **Theming**: Color configuration can be added to prompts

---

## 7. Approval

- [ ] CLI UX Designer: Approved
- [ ] Rust Architect: Technical feasibility confirmed
- [ ] Quality Validator: Test cases defined

---

**Document End**
