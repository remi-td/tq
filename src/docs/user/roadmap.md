# tq (Teradata Query) - Product Roadmap

**Version:** 1.5.0  
**Last Updated:** 2026-01-18

---

## Vision

`tq` is a best-in-class command-line client for Teradata databases. Our goal is to create a tool that is:
- **Fast:** Lightning-fast startup and query execution
- **Intuitive:** Easy to learn, powerful to master
- **Reliable:** Rock-solid stability with excellent error handling
- **Modern:** Rich interactive experience with intelligent features

---

## Releases

### v1.5.0 - Database-Aware Intelligence (2026-01-18)

**Sprint 7: Interactive Mode Phase 4**

The REPL is now database-aware, providing intelligent tab completion for tables and columns, plus dynamic connection management.

**Key Features:**
- **Smart Tab Completion:** Press Tab after `FROM`, `JOIN`, or `SELECT` to see available tables and columns from your database
- **Column Type Hints:** See data types while typing queries
- **Dynamic Connections:** Switch databases without exiting with `/logon` command
- **Context-Aware:** Understands your SQL context to show relevant completions

**User Experience:**
```
tq> SELECT * FROM <Tab>
employees  departments  projects  users

tq> SELECT employee_id, first_name, <Tab>
last_name (VARCHAR)  department_id (INTEGER)  hire_date (DATE)

tq> /logon admin@prod-db:1025/analytics
Connected to analytics database!
```

---

### v1.4.0 - Enhanced Productivity (2026-01-18)

**Sprint 6: Interactive Mode Phase 3**

Major usability improvements with tab completion for SQL keywords, result export, and display customization.

**Key Features:**
- **SQL Keyword Completion:** Tab-complete 50+ SQL keywords as you type
- **Export Results:** Save query results to CSV, JSON, or SQL format with `/export` command
- **Display Control:** Toggle paging and syntax highlighting on the fly with `/pager` and `/colors`
- **Table Formatting Fix:** Resolved column alignment issues for perfect tables

---

### v1.3.0 - Visual Polish (2026-01-17)

**Sprint 5: Interactive Mode Phase 2**

Rich visual experience with syntax highlighting and intelligent result paging.

**Key Features:**
- **SQL Syntax Highlighting:** Color-coded SQL keywords, strings, and operators
- **Vertical Paging:** Navigate long result sets with j/k, Page Up/Down keys
- **Horizontal Scrolling:** View wide tables with h/l, arrow keys
- **Query Timing:** See execution time for performance tuning

---

### v1.2.0 - Interactive REPL Foundation (2026-01-16)

**Sprint 4: Interactive Mode Phase 1**

Added interactive REPL mode for exploratory data analysis.

**Key Features:**
- **Interactive Prompt:** Multi-line SQL editing with history (↑/↓ arrows)
- **Session Management:** `/session` command to view connection details
- **Table Inspection:** `/describe` command to explore table schemas
- **Persistent History:** Command history saved across sessions
- **Vi/Emacs Keybindings:** Choose your preferred editor mode

---

### v1.1.0 - Core Connectivity (2026-01-10)

**Sprints 1-3: MVP Features**

Foundation with essential database connectivity and query execution.

**Key Features:**
- **Single Query Execution:** `tq query "SELECT..."` for one-off queries
- **Multiple Output Formats:** Table, JSON, and CSV output with `--format`
- **Connection Testing:** `tq ping` to verify database connectivity
- **Authentication Methods:** Support for TD2, LDAP, Kerberos (KRB5)
- **Flexible Configuration:** Connection strings, environment variables, password files
- **Secure Credentials:** No password leaks in logs or error messages

---

## Next Up

### Sprint 8+ - Batch Mode & Configuration

**Planned Features:**
- **File Input:** Execute SQL from files with `--file` flag
- **Stdin Support:** Pipe SQL into `tq` for scripting
- **Streaming Results:** Handle large result sets efficiently
- **Configuration Files:** Save connection profiles and preferences
- **Multiple Statements:** Execute several queries in sequence

**Focus Areas:**
- Scripting and automation workflows
- Enterprise integration patterns
- Performance at scale

---

## Design Philosophy

### 1. Convention over Configuration
Sensible defaults for 80% of use cases. Zero-config for basic usage.

### 2. Progressive Disclosure
Simple things are easy. Complex things are possible. Features revealed as needed.

### 3. Terminal Context Awareness
Human-friendly output for TTY. Machine-friendly output for pipes and scripts.

### 4. Fail Fast with Clear Guidance
Never leave users guessing. Clear error messages with actionable suggestions.

### 5. Respect UNIX Principles
Play nice with pipes, redirects, and other command-line tools.

---

## Feedback

We're building tq in the open with continuous user feedback. Have ideas or found issues? 

Add feature requests or bug reports to `docs/builder/incoming/` following the templates in the README.

---

**tq** - Fast, intuitive, reliable Teradata CLI for everyone.
