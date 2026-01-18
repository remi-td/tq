---
sprint: 7
start_date: 2026-01-19
target_completion: 2026-01-20
status: Planning
---

# Sprint 7 Planning: Advanced Tab Completion & Connection Management

## Sprint Overview

**Sprint Goal:** Enhance the REPL with intelligent tab completion for database objects (tables, columns) and dynamic connection management.

**Sprint Theme:** Interactive Mode Phase 4 - Database-Aware Features

---

## Objectives

1. Enable users to discover and navigate database objects through tab completion
2. Provide dynamic connection switching without restarting the REPL
3. Maintain the high-quality UX established in previous sprints
4. Continue zero technical debt approach

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Tab Completion for Table Names

**Description:** When users type SQL queries in the REPL, pressing Tab after `FROM`, `JOIN`, or `UPDATE` should show available table names from the current database. This dramatically improves discoverability and reduces typos.

**Acceptance Criteria:**
- [ ] Tab completion works after `FROM` keyword (e.g., `SELECT * FROM <Tab>`)
- [ ] Tab completion works after `JOIN` keyword (e.g., `FROM table1 JOIN <Tab>`)
- [ ] Tab completion works after `UPDATE` keyword (e.g., `UPDATE <Tab>`)
- [ ] Completion queries database metadata to get table list
- [ ] Handles case where database connection is slow (timeout/fallback)
- [ ] Prefix matching works (typing `us<Tab>` matches `users`, `user_profiles`)
- [ ] Shows schema.table if multiple schemas exist
- [ ] Performance is acceptable (<500ms for table list retrieval)
- [ ] Errors are handled gracefully (no crashes if metadata query fails)

**Reference:** `detailed-specifications/repl-mode.md#tab-completion`

**Estimated Complexity:** High (requires database metadata queries)

---

### P1 - High Priority (Should Have)

#### Feature 2: Tab Completion for Column Names

**Description:** When users are writing queries, pressing Tab after `SELECT`, `WHERE`, or `ORDER BY` should show available column names for the table in context. This requires analyzing the SQL statement to determine which table's columns to show.

**Acceptance Criteria:**
- [ ] Tab completion works after `SELECT` keyword (e.g., `SELECT <Tab>` shows columns from table in FROM clause)
- [ ] Tab completion works in `WHERE` clause (e.g., `WHERE <Tab>` shows available columns)
- [ ] Tab completion works in `ORDER BY` clause (e.g., `ORDER BY <Tab>`)
- [ ] Completion queries database metadata to get column list for specific table
- [ ] Shows column data type as hint (e.g., `user_id (INTEGER)`)
- [ ] Handles ambiguous context gracefully (multiple tables in query)
- [ ] Performance is acceptable (<300ms for column list retrieval)
- [ ] Errors handled gracefully (no crashes if metadata query fails)

**Reference:** `detailed-specifications/repl-mode.md#tab-completion`

**Estimated Complexity:** High (requires SQL parsing to understand context)

---

#### Feature 3: `/logon` Metacommand

**Description:** Add a `/logon` metacommand to allow users to switch database connections without exiting and restarting the REPL. This is essential for users who work with multiple databases.

**Acceptance Criteria:**
- [ ] `/logon <connection-string>` connects to new database
- [ ] `/logon` with no args shows current connection info
- [ ] Properly disconnects from old database before connecting to new one
- [ ] Preserves REPL history across connection changes
- [ ] Preserves REPL settings (pager, colors, editor mode)
- [ ] Clears cached metadata (table/column completions) after connection change
- [ ] Shows clear success/failure messages
- [ ] Supports all authentication mechanisms (TD2, LDAP, KRB5)
- [ ] Handles connection failures gracefully (reverts to previous connection if possible)

**Reference:** `detailed-specifications/repl-mode.md#metacommands`

**Estimated Complexity:** Medium (builds on existing connection code)

---

### P2 - Medium Priority (Nice to Have)

_No P2 features planned for this sprint. Keeping scope tight to ensure quality._

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- Tab completion for SQL functions (e.g., `SUM`, `AVG`) - deferred to Sprint 8
- Tab completion for database/schema names - deferred to Sprint 8
- Smart SQL query suggestions - too complex, requires ML/heuristics
- Multi-database session management - deferred to configuration feature sprint
- `/disconnect` metacommand - covered by `/logon` functionality

**Rationale:** Keeping sprint focused on database-aware completion and connection management. Additional completion features can be added incrementally in future sprints.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features are implemented, tested, and working as specified
- [ ] All P1 features are implemented and tested
- [ ] 100% test pass rate (unit + integration tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect new features (help text, README)
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per rust-architecture.md)
- [ ] All features validated by quality-validator agent
- [ ] Completion validated by tq-project-manager agent
- [ ] Performance requirements met (completion response time <500ms)

---

## Dependencies

### External Dependencies
- **Teradata teradatarustapi**: Need to verify metadata query support (DBC.Tables, DBC.Columns)
- **reedline crate**: Need to understand dynamic completer updates

### Prerequisite Work
- Sprint 6 completion (tab completion infrastructure exists from keyword completion)
- Environment: Test database with sufficient table/column variety for testing

### Blockers
- **Known blocker:** Database metadata queries may be slow on large databases
  - **Mitigation:** Implement timeout (500ms) and cache metadata for session
- **Potential blocker:** SQL parsing for column context may be complex
  - **Mitigation:** Start with simple cases (`SELECT`, `WHERE`), expand incrementally

---

## Risks & Mitigation

### Risk 1: Metadata Query Performance
- **Probability:** Medium
- **Impact:** High (poor UX if tab completion is slow)
- **Mitigation:**
  - Implement aggressive caching of table/column lists
  - Use async metadata queries with timeout
  - Provide user feedback while querying ("Loading...")

### Risk 2: SQL Parsing Complexity for Column Context
- **Probability:** Medium
- **Impact:** Medium (column completion may not work in all contexts)
- **Mitigation:**
  - Start with simple, common cases
  - Document limitations clearly
  - Use regex/simple parsing instead of full SQL parser
  - Accept that some edge cases won't be supported in v1

### Risk 3: Connection Switching State Management
- **Probability:** Low
- **Impact:** High (crashes or data corruption if not handled correctly)
- **Mitigation:**
  - Follow existing one-shot execution model patterns
  - Properly cleanup old connection before establishing new one
  - Comprehensive testing of connection failure scenarios

---

## Action Items from Previous Sprint

Items carried over from Sprint 6 retrospective:

- [ ] **Improve agent cost efficiency** - Architect agent used significant tokens in Sprint 6
  - Action: Provide more focused prompts, reference existing code patterns
  - Action: Consider using Sonnet for simpler implementation tasks

- [ ] **Streamline test case design** - Test case creation took longer than needed
  - Action: Reuse existing test case patterns
  - Action: Quality validator should focus on new scenarios, not rewrite existing tests

**Reference:** [Sprint 6 Review](sprint-6-review.md)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design tab completion UX for tables and columns
- Design `/logon` metacommand behavior and error messages
- Update `specifications.md` with 🚧 status for Sprint 7 features
- Update `detailed-specifications/repl-mode.md` with comprehensive completion and metacommand specs

**Deliverables:**
- Updated `specifications.md` dashboard
- Detailed spec for table/column completion in `repl-mode.md`
- Detailed spec for `/logon` metacommand in `repl-mode.md`
- UX design validation for all features

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement table name tab completion with metadata queries
- Implement column name tab completion with SQL context awareness
- Implement `/logon` metacommand with proper state management
- Write unit tests for all new code (target: 100% pass rate)
- Update `rust-architecture.md` if new patterns introduced

**Deliverables:**
- Working implementation of all P0/P1 features
- Unit tests with 100% pass rate
- Updated `rust-architecture.md` if needed
- Performance benchmarks showing completion response time <500ms

**Special Focus:**
- Reuse existing completion infrastructure from Sprint 6
- Follow one-shot execution model for connection management
- Minimize token usage by referencing existing patterns

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test cases for:
  - Table completion in various SQL contexts
  - Column completion with multiple tables
  - `/logon` metacommand success and failure scenarios
  - Connection state management edge cases
- Execute all test suites (unit + integration)
- Generate test reports in `tests/results/`
- Validate performance requirements (<500ms completion time)

**Deliverables:**
- Test cases in `tests/cases/TC###.md` (reuse existing patterns where possible)
- Test execution report in `tests/results/YYYYMMDD-HHMMSS/REPORT.md`
- 100% test pass rate
- Performance validation report

**Special Focus:**
- Reuse existing test case templates from previous sprints
- Focus on new scenarios specific to database-aware completion
- Validate error handling thoroughly (slow DB, failed queries, etc.)

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Assess technical debt status (target: zero)
- Verify documentation synchronized with implementation
- Provide go/no-go decision for sprint closure
- Check that performance requirements are met

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment (should be zero)
- Go/no-go recommendation
- Recommendations for Sprint 8

---

## Sprint Timeline

**Estimated Duration:** 2 days

### Phase Breakdown
- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - User approval obtained

- **Phase 2: Design** (Est. 4 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Specifications finalized
  - Technical feasibility validated

- **Phase 3: Implementation** (Est. 8 hours)
  - Parallel execution: rust-teradata-architect + quality-validator
  - Code + tests delivered
  - Unit tests passing

- **Phase 4: Testing** (Est. 2 hours)
  - quality-validator executes all tests
  - 100% pass rate achieved
  - Performance validated

- **Phase 5: Closure** (Est. 2 hours)
  - tq-project-manager validates completion
  - Sprint review created
  - Roadmap updated
  - Version v1.5.0 released

---

## Notes

### Technical Considerations

1. **Metadata Caching Strategy:**
   - Cache table list per session (invalidate on `/logon`)
   - Cache column list per table per session
   - Implement lazy loading (query on first tab press, not on REPL start)

2. **SQL Context Parsing:**
   - Use simple regex-based approach for identifying table context
   - Support common patterns: `FROM table`, `JOIN table`, `UPDATE table`
   - Accept limitations for complex queries (nested subqueries, CTEs)

3. **Connection State Management:**
   - Store connection config in ReplState
   - Implement proper cleanup of old connection
   - Handle case where new connection fails (preserve old connection if possible)

### Performance Targets

- Table list retrieval: <500ms
- Column list retrieval: <300ms
- Tab completion response: <100ms (after metadata cached)
- `/logon` reconnection: <2s

### User Experience Priorities

1. **Discoverability:** Users should naturally discover tab completion through exploration
2. **Speed:** Completion should feel instant (or show "Loading..." feedback)
3. **Forgiveness:** Errors should never crash REPL, always provide clear messages
4. **Consistency:** Completion behavior should match keyword completion from Sprint 6

---

## Approval

**Status:** Pending

**Approved By:** [Awaiting user approval]
**Approval Date:** [YYYY-MM-DD]

**Revisions Requested:**
_[Any changes requested by user]_

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial Sprint 7 plan | Main Agent |
