# Sprint 10 Planning: Batch Mode Foundation

**Sprint Number:** 10
**Sprint Goal:** Implement foundational batch mode features for scripting and automation
**Status:** PLANNING
**Created:** 2026-01-18

---

## Executive Summary

Sprint 10 marks the transition from interactive REPL enhancements to batch mode capabilities. With Sprint 9's quality recovery complete and all REPL features stable, we're ready to implement the core batch mode features that enable `tq` to be used in scripts, cron jobs, and CI/CD pipelines.

**Strategic Rationale:** Batch mode is essential for production use cases where automation and scripting are required. This sprint focuses on the foundational features (stdin, file input) that unlock the most value.

---

## Sprint Objectives

### Primary Goal
Implement batch mode foundations to enable scripting and automation use cases.

### Success Criteria
1. ✅ Users can pipe SQL to `tq query` from stdin
2. ✅ Users can execute SQL files with `tq query --file`
3. ✅ Multiple SQL statements in a single file execute correctly
4. ✅ Output formats (table/JSON/CSV) work in batch mode
5. ✅ Error handling matches batch-mode specifications
6. ✅ All tests pass (unit + integration + manual validation)
7. ✅ Zero technical debt introduced

---

## Scope

### In Scope (P0 - Must Have)

#### Feature 1: stdin Input Support
**User Story:** As a DevOps engineer, I want to pipe SQL queries to tq from stdin so I can integrate tq into shell scripts and pipelines.

**Requirements:**
- Detect when input is piped (stdin is not a TTY)
- Read SQL from stdin until EOF
- Execute SQL and output results
- Support all output formats (table, JSON, CSV)
- Handle multi-line SQL from stdin
- Exit with appropriate exit code

**Examples:**
```bash
# Pipe from file
cat query.sql | tq query

# Pipe from command
echo "SELECT COUNT(*) FROM users" | tq query --format json

# Heredoc
tq query <<EOF
SELECT employee_id, salary
FROM employees
WHERE salary > 50000
EOF
```

**Technical Notes:**
- Check `atty::is(atty::Stream::Stdin)` to detect piped input
- Read all stdin content before executing (not streaming yet)
- Treat stdin input as if user provided `--sql` flag

---

#### Feature 2: File Input Support
**User Story:** As a DBA, I want to execute SQL scripts from files so I can run saved queries and migration scripts.

**Requirements:**
- Add `--file <path>` flag to `tq query` command
- Read SQL from specified file
- Execute SQL and output results
- Support all output formats
- Handle comments in SQL files (-- and /* */)
- Validate file exists and is readable
- Provide clear error if file not found

**Examples:**
```bash
# Execute SQL from file
tq query --file migrations/create_tables.sql

# With output format
tq query --file reports/monthly.sql --format csv > report.csv

# With connection override
tq query --file script.sql -l admin:pass@prod:1025/db
```

**File Format:**
```sql
-- This is a comment
SELECT * FROM users WHERE active = 1;

/*
 * Multi-line comment
 */
SELECT COUNT(*) FROM orders;
```

**Technical Notes:**
- Use `std::fs::read_to_string()` to read file
- Path can be absolute or relative to current directory
- File extension doesn't matter (.sql, .txt, or no extension all work)

---

#### Feature 3: Multiple Statement Execution
**User Story:** As a developer, I want to execute multiple SQL statements from a single file so I can run setup scripts and data migrations.

**Requirements:**
- Parse SQL file into individual statements (split on `;`)
- Execute statements sequentially
- Display results for each query that returns data
- Continue on success, stop on first error (fail-fast)
- Report which statement failed (line number or statement preview)
- Exit code reflects success/failure

**Examples:**
```sql
-- setup.sql
CREATE TABLE temp_data (id INT, value VARCHAR(100));

INSERT INTO temp_data VALUES (1, 'test');
INSERT INTO temp_data VALUES (2, 'test2');

SELECT * FROM temp_data;

DROP TABLE temp_data;
```

```bash
tq query --file setup.sql
# Output:
# Statement 1: CREATE TABLE - OK
# Statement 2: INSERT - OK (1 row)
# Statement 3: INSERT - OK (1 row)
# Statement 4: SELECT - 2 rows returned
# [table output]
# Statement 5: DROP TABLE - OK
```

**Technical Notes:**
- Simple semicolon splitting (no SQL parser needed yet)
- Skip empty statements (whitespace-only between semicolons)
- Trim whitespace from each statement
- Statement numbering starts at 1 for user-facing messages

---

### In Scope (P1 - Should Have)

#### Feature 4: Enhanced Error Messages for Batch Mode
**Requirements:**
- Show filename and line number for file input errors
- Show statement number for multi-statement failures
- Provide context (failed statement text, up to 80 chars)
- Clear distinction between file I/O errors vs SQL errors

**Example:**
```
Error in migrations/v2.sql at statement 3:
SQL syntax error: [Error 3707] Syntax error near 'FORM'
Statement: SELECT * FORM users WHER...
```

---

#### Feature 5: Batch Mode Output Behavior
**Requirements:**
- Default to table format for TTY, JSON for non-TTY
- No pagination in batch mode (all output to stdout)
- No syntax highlighting in batch mode
- No timing information in batch mode (unless --verbose)
- Quiet mode: suppress headers and formatting (CSV/JSON only)

**Technical Notes:**
- Already detect TTY in existing code
- Disable pager when stdin is piped
- Add `--quiet` flag for minimal output

---

### Out of Scope (Future Sprints)

The following features are documented in batch-mode.md but deferred:

- ❌ **Transaction Control** (`--atomic` flag) - Sprint 11+
- ❌ **Variable Substitution** (`{{var}}` in SQL) - Sprint 11+
- ❌ **Streaming Results** (incremental output) - Sprint 12+
- ❌ **Output to File** (`--output` flag) - Sprint 11+
  - Users can use shell redirection (`> file.csv`) for now
- ❌ **Progress Indicators** - Sprint 11+
- ❌ **Parallel Execution** - Future

**Rationale:** Focus on core functionality first. These advanced features can build on the foundation.

---

## Architecture Considerations

### Key Design Decisions

#### 1. Input Source Precedence
When user provides multiple input sources, precedence is:
1. Explicit SQL argument: `tq query "SELECT 1"`
2. File flag: `tq query --file script.sql`
3. stdin: `cat script.sql | tq query`

Error if multiple sources provided.

#### 2. Statement Parsing Strategy
**Approach:** Simple semicolon splitting for MVP
- Split on `;` character
- No SQL grammar parsing (complex, error-prone)
- Known limitation: Doesn't handle `;` in strings/comments
- Document limitation, address in future sprint if needed

**Why:** 99% of real-world SQL scripts work fine with simple splitting. Full SQL parsing is complex and unnecessary for MVP.

#### 3. Output Modes
**Table Mode (default for TTY):**
- Human-readable tables with borders
- Pagination disabled in batch mode

**JSON Mode:**
- Array of objects
- Suitable for processing with `jq`
- One statement → one JSON array

**CSV Mode:**
- Standard RFC 4180 CSV
- Headers included by default
- `--no-headers` flag for headerless output

#### 4. Error Handling Philosophy
- **Fail Fast:** Stop on first error
- **Clear Context:** Show what failed and where
- **Actionable Messages:** Suggest fixes when possible
- **Exit Codes:** Non-zero on any failure

---

## Technical Implementation Notes

### Required Code Changes

#### 1. CLI Argument Parsing (src/cli.rs)
```rust
// Add to QueryCommand struct
#[arg(short = 'f', long = "file", value_name = "PATH")]
pub file: Option<PathBuf>,

#[arg(long = "no-headers")]
pub no_headers: bool,
```

#### 2. Input Source Detection (src/commands/query.rs)
```rust
enum InputSource {
    Argument(String),    // Explicit SQL argument
    File(PathBuf),       // --file flag
    Stdin,               // Piped input
}

fn determine_input_source(args: &QueryArgs) -> Result<InputSource> {
    // Check for conflicting sources
    // Return appropriate source
}
```

#### 3. Statement Parser (new: src/sql/parser.rs)
```rust
pub fn parse_statements(sql: &str) -> Vec<String> {
    sql.split(';')
       .map(|s| s.trim())
       .filter(|s| !s.is_empty())
       .map(|s| s.to_string())
       .collect()
}
```

#### 4. Batch Executor (src/commands/query.rs)
```rust
fn execute_batch(
    client: &mut DatabaseClient,
    statements: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    for (idx, stmt) in statements.iter().enumerate() {
        execute_statement(client, stmt, idx + 1, format)?;
    }
    Ok(())
}
```

### Testing Strategy

#### Unit Tests
- Input source detection logic
- Statement parsing (single, multiple, with comments)
- Error handling for file not found
- Stdin vs file vs argument precedence

#### Integration Tests
- Execute single statement from file
- Execute multiple statements from file
- stdin input with various formats
- Error handling (bad SQL, file not found)
- Output format verification

#### Manual Testing
Real Teradata database tests:
```bash
# Test 1: stdin with echo
echo "SELECT 1 as test_column" | ./target/release/tq query

# Test 2: stdin with heredoc
./target/release/tq query <<EOF
SELECT current_date;
SELECT current_time;
EOF

# Test 3: File input
cat > test.sql <<EOF
SELECT 1;
SELECT 2;
SELECT 3;
EOF
./target/release/tq query --file test.sql

# Test 4: Error handling
echo "INVALID SQL" | ./target/release/tq query
echo $?  # Should be non-zero

# Test 5: Multiple formats
echo "SELECT * FROM DBC.TablesV SAMPLE 5" | ./target/release/tq query --format json
echo "SELECT * FROM DBC.TablesV SAMPLE 5" | ./target/release/tq query --format csv
```

---

## Dependencies

### External Dependencies
No new crates needed. Existing dependencies sufficient:
- `clap` - CLI argument parsing (already used)
- `std::fs` - File I/O (standard library)
- `atty` - TTY detection (already used)

### Internal Dependencies
- Existing `DatabaseClient` for query execution
- Existing output formatters (table, JSON, CSV)
- Existing error handling infrastructure

---

## Risks and Mitigations

### Risk 1: SQL Statement Splitting Edge Cases
**Risk:** Simple semicolon splitting fails with `;` in strings or comments

**Example:**
```sql
INSERT INTO messages VALUES ('Hello; World');  -- Splits incorrectly
```

**Mitigation:**
- Document limitation in release notes
- Implement simple heuristic: ignore `;` in single-quoted strings
- Full SQL parser can be added later if users hit this issue
- Most real-world scripts don't have this problem

### Risk 2: Large File Performance
**Risk:** Loading large SQL files into memory could exhaust memory

**Mitigation:**
- Phase 1 (this sprint): Read entire file into memory (simple, works for files up to 100MB)
- Phase 2 (future): Implement streaming file reader if needed
- Document file size limitations in help text

### Risk 3: Transaction Semantics
**Risk:** Multiple statements execute without transaction control

**Example:** Statement 3 fails, but statements 1-2 already committed

**Mitigation:**
- Clearly document "fail-fast" behavior (not atomic)
- Add `--atomic` flag in future sprint for transaction control
- Users can wrap in BEGIN/COMMIT if needed

### Risk 4: Comment Handling
**Risk:** SQL comments might interfere with parsing

**Mitigation:**
- Phase 1: Don't strip comments, let Teradata handle them
- Teradata SQL parser already handles comments correctly
- If issues arise, add comment stripping in Phase 2

---

## Success Metrics

### Functionality Metrics
- ✅ stdin input works with echo, cat, heredoc
- ✅ File input works with absolute and relative paths
- ✅ Multiple statements execute sequentially
- ✅ All output formats work (table, JSON, CSV)
- ✅ Error messages include context (file, line, statement)

### Quality Metrics
- ✅ 100% unit test pass rate
- ✅ 100% integration test pass rate
- ✅ Manual tests pass against real Teradata
- ✅ Zero new warnings
- ✅ Zero technical debt

### Performance Metrics (Nice to Have)
- File input <50ms overhead vs direct execution
- Parse 100-statement file in <10ms

---

## Acceptance Criteria

Sprint 10 is complete when:

1. **Feature Completeness**
   - [ ] stdin input implemented and tested
   - [ ] File input implemented and tested
   - [ ] Multiple statement execution implemented
   - [ ] Error handling meets specifications
   - [ ] Help text updated for new flags

2. **Quality Gates**
   - [ ] All unit tests pass
   - [ ] All integration tests pass
   - [ ] Manual testing checklist complete
   - [ ] Zero build warnings
   - [ ] Code reviewed by rust-teradata-architect

3. **Documentation**
   - [ ] README updated with batch mode examples
   - [ ] specifications.md updated (batch mode features marked ✅)
   - [ ] batch-mode.md updated with implementation notes
   - [ ] Help text (`tq query --help`) shows new flags

4. **Sprint Artifacts**
   - [ ] Sprint 10 review document created
   - [ ] Version bumped to v1.6.0
   - [ ] Git commit with descriptive message
   - [ ] Sprint 10 metrics collected

---

## Timeline Estimate

**Complexity Assessment:** Medium
- New feature area (batch mode) but builds on existing infrastructure
- Input source detection is straightforward
- Statement parsing is simple for MVP
- Testing requires manual validation

**Estimated Effort:**
- Planning & Design: 2-3 hours (parallel agents)
- Implementation: 3-4 hours (rust-teradata-architect)
- Testing: 2 hours (quality-validator + manual)
- Review & Documentation: 1 hour

**Total:** 8-10 hours (1-2 days of focused work)

---

## Open Questions for Design Phase

These questions will be answered by the cli-ux-designer and rust-teradata-architect agents during parallel design:

### For cli-ux-designer:
1. What should the UX be when user provides conflicting input sources?
2. Should there be a `--quiet` flag, or is format selection sufficient?
3. What level of detail in error messages? (filename, line, statement preview?)
4. Should we have `--continue-on-error` flag or always fail-fast?

### For rust-teradata-architect:
1. Best approach for stdin detection and reading?
2. Where should statement parsing logic live? (new module or existing?)
3. How to refactor existing query command to support multiple execution modes?
4. Should we add a `StatementExecutor` trait for flexibility?

---

## Next Steps

1. **Approve This Plan**
   - Review sprint scope and objectives
   - Confirm priorities and out-of-scope items
   - Approve proceeding to design phase

2. **Launch Parallel Design (Main Agent Coordinates)**
   - Launch cli-ux-designer agent for detailed UX specifications
   - Launch rust-teradata-architect agent for technical feasibility and architecture
   - Main agent synthesizes outputs

3. **Verify Database Connectivity**
   - Check `.env` file exists
   - Run `./target/release/tq ping`
   - Confirm database available before implementation

4. **Implementation Phase**
   - rust-teradata-architect implements features
   - quality-validator designs test cases in parallel
   - Main agent reviews outputs

5. **Test Execution & Fix Loop**
   - quality-validator executes tests
   - If failures: rust-teradata-architect fixes, quality-validator re-tests
   - Loop until 100% pass rate

6. **Sprint Closure**
   - tq-project-manager validates completion
   - Main agent creates sprint review
   - Main agent updates specifications.md
   - Main agent collects metrics with `/collect-metrics 10`

---

## Notes

- This sprint focuses on **foundational batch mode only**
- Advanced features (transactions, variables, streaming) deferred to future sprints
- Quality is paramount: 100% test pass rate required before sprint closure
- Follow lessons learned from Sprint 9: autonomous execution, sequential fixes, comprehensive testing

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial Sprint 10 planning | Main Claude Agent |
