# User Personas and Use Cases

**Document:** User Personas & Use Cases
**Version:** 1.0.0
**Last Updated:** 2026-01-18
**Owner:** CLI UX Designer

---

## Overview

This document defines the target users of `tq` (Teradata Query), their needs, pain points, and primary use cases. Understanding these personas guides feature prioritization and UX design decisions.

---

## User Personas

### Persona 1: Database Administrator (DBA)

**Profile:** Sarah, Senior DBA

**Background:**
- 10+ years experience managing Teradata environments
- Responsible for database health monitoring and performance tuning
- Works across multiple database instances daily
- Values command-line tools for their speed and scriptability

**Needs:**
- Quick health checks without starting heavy GUI tools
- Connection testing and latency measurement
- Schema inspection and table statistics
- Fast feedback for diagnostic queries
- Scriptable commands for automation

**Pain Points:**
- GUI tools (SQL Assistant, Studio) are slow to start
- Need lightweight diagnostics for quick checks
- Existing CLI tools lack modern UX features
- Password management is cumbersome across many instances

**Usage Pattern:**
- 50+ quick commands per day
- Primarily batch mode for scripts and monitoring
- Occasional REPL for investigation
- Values speed, reliability, and consistent behavior

**Key Features:**
- `tq ping` for connectivity testing
- Fast startup time (< 100ms)
- Secure credential management
- Exit codes for scripting

---

### Persona 2: Data Analyst

**Profile:** Mike, Business Intelligence Analyst

**Background:**
- Writes SQL queries to extract and analyze business data
- Creates reports for stakeholders
- Limited database administration knowledge
- Comfortable with command line but prefers interactive tools

**Needs:**
- Ad-hoc query execution
- Interactive data exploration
- Easy CSV export for Excel analysis
- Table structure inspection
- Query history for recall

**Pain Points:**
- Current tools don't integrate with shell workflows
- CSV export requires manual steps
- Can't easily share queries with colleagues
- Limited command history and recall

**Usage Pattern:**
- Interactive REPL sessions (1-2 hours)
- Frequent CSV exports for reporting
- Query iteration and refinement
- Copy/paste results for sharing

**Key Features:**
- REPL mode with history
- `/describe` for table inspection
- CSV export with headers
- Multi-line query editing
- Syntax highlighting (future)

---

### Persona 3: DevOps Engineer

**Profile:** Alex, Platform Engineer

**Background:**
- Manages CI/CD pipelines and infrastructure automation
- Monitors application and database health
- Writes shell scripts and Python for automation
- Needs reliable, scriptable tools

**Needs:**
- Automated health checks in monitoring systems
- Scripted data extraction for dashboards
- JSON output for processing with jq/Python
- Reliable exit codes for error handling
- Non-interactive execution

**Pain Points:**
- Hard to integrate existing database tools in CI/CD
- GUI tools can't be scripted
- Inconsistent exit codes and error messages
- Password handling in automation is complex

**Usage Pattern:**
- Batch mode scripts (cron jobs, monitoring)
- JSON output piped to processing tools
- Error handling and alerting
- Zero interactive usage

**Key Features:**
- `tq ping` for health checks
- JSON output format
- Reliable exit codes (0=success, 1=error)
- Environment variable configuration
- Password file support

---

### Persona 4: Data Engineer

**Profile:** Jamie, ETL Developer

**Background:**
- Builds and maintains data pipelines
- Works with large datasets (millions of rows)
- Performance and efficiency are critical
- Expert in SQL and data processing

**Needs:**
- Streaming large result sets without memory issues
- Fast query execution
- Pipeline integration (Unix pipes)
- Batch processing support
- Performance monitoring

**Pain Points:**
- Memory issues with large datasets in existing tools
- Slow startup time impacts pipeline performance
- Limited streaming support
- Difficult to monitor query execution time

**Usage Pattern:**
- Large batch exports (millions of rows)
- Pipeline integration with other tools
- Performance-critical workflows
- Minimal interactive use

**Key Features:**
- Streaming result output
- CSV format for pipeline processing
- Query timing display
- Efficient memory usage
- Fast startup time

---

## Primary Use Cases

### UC-1: Quick Connection Test

**Actor:** DBA (Sarah)

**Goal:** Verify database connectivity and measure latency

**Preconditions:**
- Database connection details available (host, port, database)
- User has valid credentials

**Flow:**
1. Set connection details via environment variable:
   ```bash
   export TQ_LOGON="user:pass@host:1025/db"
   ```
2. Execute ping command:
   ```bash
   tq ping
   ```
3. View connection status and latency:
   ```
   Database connection successful (127ms)
   ```

**Success Criteria:**
- Connection succeeds with latency measurement
- Exit code 0 for success, 1 for failure
- Clear error message on failure with troubleshooting tips

**Frequency:** 10-20 times per day

---

### UC-2: One-Shot Query

**Actor:** Data Analyst (Mike)

**Goal:** Execute single SQL query and view results in readable format

**Preconditions:**
- Database connection configured
- User knows SQL query to execute

**Flow:**
1. Execute query with table output:
   ```bash
   tq query "SELECT * FROM employees WHERE dept = 'IT'" --format table
   ```
2. View formatted results in terminal

**Success Criteria:**
- Query executes successfully
- Results displayed in human-readable table format
- Column headers shown
- Row count displayed at bottom

**Frequency:** 20-50 times per day

---

### UC-3: Export to CSV

**Actor:** Data Analyst (Mike)

**Goal:** Export query results to CSV file for analysis in Excel

**Preconditions:**
- Database connection configured
- Query returns tabular data

**Flow:**
1. Execute query with CSV output:
   ```bash
   tq query "SELECT * FROM sales_2024" --format csv > sales.csv
   ```
2. Open CSV file in Excel for analysis

**Success Criteria:**
- CSV file created with proper headers
- Data properly escaped (quotes, commas, newlines)
- Compatible with Excel, Google Sheets, and CSV parsers
- Large datasets stream without memory issues

**Frequency:** 5-10 times per day

---

### UC-4: Scripted Health Check

**Actor:** DevOps Engineer (Alex)

**Goal:** Automated database monitoring in CI/CD or cron job

**Preconditions:**
- Database connection configured via environment
- Monitoring system can execute shell scripts

**Flow:**
1. Create monitoring script:
   ```bash
   #!/bin/bash
   if tq ping --timeout 5s; then
     echo "Database healthy"
   else
     alert_ops "Database down"
   fi
   ```
2. Schedule script via cron or monitoring system
3. Receive alerts on connection failure

**Success Criteria:**
- Reliable exit codes (0=success, 1=failure)
- Configurable timeout
- Fast execution (< 500ms for healthy connection)
- Clear error messages for logging

**Frequency:** Continuous (every 1-5 minutes)

---

### UC-5: Interactive Exploration

**Actor:** Data Analyst (Mike)

**Goal:** Explore database schema and query data interactively

**Preconditions:**
- Database connection configured
- User wants to explore tables and data

**Flow:**
1. Start REPL:
   ```bash
   tq repl
   ```
2. List databases:
   ```
   tq> \l
   ```
3. List tables:
   ```
   tq> \dt public.*
   ```
4. Describe table structure:
   ```
   tq> \d employees
   ```
5. Query data:
   ```sql
   tq> SELECT * FROM employees LIMIT 10;
   ```
6. Export results:
   ```
   tq> \export csv employees.csv
   ```

**Success Criteria:**
- REPL starts quickly (< 500ms)
- Metacommands work without writing SQL
- Query history recalls previous commands
- Results displayed in readable format
- Can export last result

**Frequency:** 2-5 sessions per day, 30-60 minutes each

---

### UC-6: Pipeline Integration

**Actor:** Data Engineer (Jamie)

**Goal:** Extract data from Teradata for processing in data pipeline

**Preconditions:**
- Database connection configured
- Pipeline tools available (jq, Python, etc.)
- Large dataset to process

**Flow:**
1. Extract data as JSON:
   ```bash
   tq query "SELECT user_id, activity FROM events" --format json
   ```
2. Filter with jq:
   ```bash
   | jq '.[] | select(.activity == "login")'
   ```
3. Process with custom script:
   ```bash
   | transform_script.py
   ```
4. Load to warehouse:
   ```bash
   | load_to_warehouse.sh
   ```

**Success Criteria:**
- JSON output is valid and parseable
- Streaming supports large datasets without memory issues
- Fast execution with minimal overhead
- Proper error handling with exit codes
- stderr used for errors, stdout for data

**Frequency:** Continuous (scheduled pipelines)

---

## Persona-Feature Mapping

| Feature | DBA | Data Analyst | DevOps | Data Engineer |
|---------|-----|--------------|--------|---------------|
| `tq ping` | ✅ High | ⚪ Low | ✅ High | ⚪ Low |
| `tq query` (table) | ⚪ Medium | ✅ High | ⚪ Low | ⚪ Low |
| `tq query` (CSV) | ⚪ Medium | ✅ High | ⚪ Low | ⚪ Medium |
| `tq query` (JSON) | ⚪ Low | ⚪ Medium | ✅ High | ✅ High |
| REPL mode | ⚪ Medium | ✅ High | ⚪ Low | ⚪ Low |
| History | ⚪ Medium | ✅ High | ⚪ Low | ⚪ Low |
| `/describe` | ✅ High | ✅ High | ⚪ Low | ⚪ Medium |
| Streaming | ⚪ Low | ⚪ Medium | ⚪ Medium | ✅ High |
| Exit codes | ✅ High | ⚪ Low | ✅ High | ✅ High |
| Password files | ✅ High | ⚪ Medium | ✅ High | ✅ High |

**Legend:**
- ✅ High priority for persona
- ⚪ Medium/Low priority

---

## Design Implications

### For DBA (Sarah)
- **Performance:** Fast startup and execution are critical
- **Scripting:** Reliable exit codes and error messages
- **Security:** Secure password management across many instances
- **Output:** Concise, parseable output for scripts

### For Data Analyst (Mike)
- **Interactivity:** REPL mode is primary usage
- **Discoverability:** Metacommands for common tasks
- **History:** Command recall across sessions
- **Export:** Easy CSV generation for reporting

### For DevOps Engineer (Alex)
- **Automation:** Non-interactive, scriptable execution
- **Reliability:** Predictable behavior and exit codes
- **Integration:** JSON output for pipeline processing
- **Security:** Environment variables and password files

### For Data Engineer (Jamie)
- **Scalability:** Stream large datasets efficiently
- **Performance:** Minimal overhead and memory usage
- **Composability:** Unix pipe integration
- **Monitoring:** Query timing and progress feedback

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| DBA adoption | 70% of DBAs using tq for daily checks | User survey |
| Query frequency | 100+ queries per user per week | Usage telemetry |
| REPL session length | 30+ minutes average | Usage telemetry |
| Script integration | 50+ automated scripts using tq | GitHub search |
| Performance | < 100ms startup, < 500ms ping | Benchmarks |

---

## Related Documents

- [CLI Interface Design](cli-interface.md) - How commands are structured
- [REPL Mode](repl-mode.md) - Interactive mode specifications
- [Batch Mode](batch-mode.md) - Scripting and automation
- [Configuration](configuration.md) - Credential management
