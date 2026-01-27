# Sprint 26 Planning

**Date:** 2026-01-27
**Type:** Feature Sprint
**Sprint Coordinator:** Main Claude Agent

---

## Reality Check Summary

**Reviewed Sprints:** Sprint 25, Sprint 24, Sprint 23 (28 total sprints completed)

**Patterns Detected:** NONE (Healthy)

**Positive Indicators:**
- ✅ 100% P0 delivery rate across last 3 sprints
- ✅ Zero technical debt maintained consistently
- ✅ Perfect test pass rate (100%) across all recent sprints
- ✅ Documentation quality excellent (Sprint 24 & 25)
- ✅ Cost efficiency improving (Sprint 25: $7.50, 50% lower than typical)
- ✅ Process maturity: Continuous improvement cycle working
- ✅ Framework healthy: Agents performing well, no systemic issues

**Decision:** FEATURE SPRINT

**Rationale:**
- No stuck issues across multiple sprints
- No accumulating technical debt
- No framework issues requiring maintenance
- Healthy velocity with consistent feature delivery
- Recent documentation maintenance (Sprint 25) cleared accumulated doc debt
- Project in excellent health for new feature work

**Quarterly Review:** Not due (last major roadmap update Sprint 22)

---

## Sprint Objectives

### Primary Objective: System Monitoring Features

Implement `/sessions` command to provide DBAs and developers with real-time visibility into Teradata system activity.

### Success Criteria

Sprint 26 succeeds if:
1. ✅ `/sessions` command implemented in both REPL and batch mode
2. ✅ All acceptance criteria met (100%)
3. ✅ 100% automated test pass rate
4. ✅ Zero technical debt introduced
5. ✅ Documentation synchronized with implementation
6. ✅ Ship phase validation passes (including documentation accuracy verification)

---

## Feature Scope

### P0 - Must Have (Sprint 26)

#### Feature 1: `/sessions` Command (GitHub Issue #6)

**Description:**
Add `/sessions` metacommand to REPL mode and `--sessions` flag to batch mode for listing active Teradata sessions with key performance metrics.

**User Story:**
As a DBA or developer, I want to see active sessions on my Teradata system so I can understand current system activity, identify long-running queries, and detect performance issues (CPU/IO skew).

**Target Users:**
- DBAs managing Teradata databases (PRIMARY)
- Developers integrating tq into scripts/pipelines
- DevOps engineers automating database tasks

**Implementation Approach:**
- REPL command: `/sessions` (with aliases `/s`)
- Batch mode flag: `--sessions` (standalone, no SQL needed)
- SQL query: Uses `MonitorSession(-1,'*',0)` table function
- Columns displayed:
  - SessionNo (session identifier)
  - UserName (logged-in user)
  - LogonTime (session start timestamp)
  - PEstate (Parsing Engine state: IDLE/DISPATCHING/ACTIVE)
  - AMPState (AMP state: IDLE/ACTIVE)
  - AMPCPUSec (total AMP CPU seconds consumed)
  - AMPIO (total AMP I/O count)
  - ReqSpool (requested spool space in bytes)
  - Amp CPU Skew % (CPU distribution across AMPs, 0% = perfect balance)
  - Amp IO Skew % (I/O distribution across AMPs, 0% = perfect balance)

**SQL Implementation (from Issue #6):**
```sql
SEL
    SessionNo
    ,UserName
    ,LogonTime
    ,PEState
    ,AMPState
    ,AMPCPUSec
    ,AMPIO
    ,ReqSpool
    ,(100 * (1 - (AvgAmpCPUSec / NULLIFZERO(HotAmp1CPU))))(DECIMAL(4,2))  (TITLE 'Amp CPU Skew %',FORMAT 'ZZ9.9')
    ,(100 * (1 - (AvgAmpIOCnt / NULLIFZERO(HotAmp1IO)))) (DECIMAL(4,2))  (TITLE 'Amp IOSkew %',FORMAT 'ZZ9.9')
FROM TABLE (MonitorSession(-1,'*',0)) AS t1;
```

**Acceptance Criteria:**
1. **AC-1:** `/sessions` command available in REPL with `/s` alias
2. **AC-2:** `tq --sessions` flag works in batch mode (no SQL file required)
3. **AC-3:** Output displays 10 columns in table format: SessionNo, UserName, LogonTime, PEstate, AMPState, AMPCPUSec, AMPIO, ReqSpool, Amp CPU Skew %, Amp IO Skew %
4. **AC-4:** Skew percentages calculated correctly (NULL for inactive sessions)
5. **AC-5:** Logon times formatted as `YYYY/MM/DD HH:MM:SS.ss`
6. **AC-6:** Tab completion suggests `/sessions` command
7. **AC-7:** `/help` output includes `/sessions` command description
8. **AC-8:** Error handling for insufficient privileges (DBC access required)
9. **AC-9:** Handles empty result set (no active sessions besides current)
10. **AC-10:** Works with all output formats (--format csv, json, table)

**Design Considerations:**
- **Privilege Requirements:** Requires SELECT on `DBC.MonitorSession` table function - document in error messages
- **Performance:** MonitorSession(-1) queries all sessions - should be fast (<1s)
- **Filtering:** Initial implementation shows ALL sessions - future enhancement could add filters (user, state, etc.)
- **Refresh:** Not real-time - user must re-run command to refresh
- **Compatibility:** Teradata 14.10+ required for MonitorSession table function

**Documentation Requirements:**
- Specification: Add `/sessions` command to `docs/specifications/repl.md` (REQ-SESS-001 through REQ-SESS-005)
- Design: Add implementation details to `docs/design/repl.md`
- User Guide: Add `/sessions` examples to `docs/user/repl-guide.md`
- Help Text: Update `tq help` and `/help` output
- CLI Interface: Document `--sessions` flag in `docs/specifications/cli-interface.md`

**Test Strategy:**
- Unit tests: Query construction, skew calculation logic, error handling
- Integration tests: Command execution with real database connection
- PTY tests: `/sessions` command in REPL, tab completion, help output
- Manual validation: Visual verification of output format, skew calculation accuracy

**Priority Rationale:** Medium priority from Issue #6, but high value for DBAs (primary user persona). Clear scope, detailed implementation proposal, aligns with system monitoring capabilities.

**Estimated Complexity:** Medium
- New metacommand pattern (similar to `/list`, `/describe`)
- SQL query provided (well-defined)
- Standard table output (existing formatter)
- Skew calculation logic (needs unit tests)

---

### P1 - Should Have (Sprint 26)

No P1 features for Sprint 26. Focusing on delivering solid `/sessions` implementation.

---

## Deferred Features

### Issue #7: Horizontal Paging (Priority: Low)

**Decision:** DEFER to future sprint

**Rationale:**
- Lower priority than monitoring features
- Higher implementation complexity (interactive paging mode)
- Requires new interaction model (arrow keys + exit keys)
- `/sessions` command provides more immediate DBA value
- Horizontal paging is nice-to-have UX improvement vs. functional capability

**Future Consideration:** Sprint 27+ when monitoring features complete

---

## Sprint Scope Summary

**Total Features:** 1 P0 feature
**GitHub Issues:** Issue #6 (selected), Issue #7 (deferred)
**Target Duration:** 1-2 days (similar to Sprint 24 complexity)
**Estimated Cost:** $12-15 (typical feature sprint range)

**Scope Justification:**
- Single focused feature with clear value proposition
- Detailed implementation proposal reduces design risk
- Aligns with primary user persona (DBAs)
- Builds on existing metacommand patterns (low integration risk)
- Solid foundation for future monitoring features

---

## Technical Approach

### Architecture

**Component Additions:**
- New metacommand: `SessionsCommand` struct
- Query builder: Construct MonitorSession SQL with skew calculations
- Batch mode integration: `--sessions` flag handling
- Help text: Update command registry and documentation

**Integration Points:**
- Metacommand registry: Add `/sessions` and `/s` aliases
- Tab completion: Include in metacommand suggestions
- Output formatting: Use existing table formatter
- Error handling: Handle privilege errors gracefully

**Dependencies:**
- No new crate dependencies expected
- Leverages existing database client infrastructure
- Uses standard output formatting pipeline

### Design Patterns

Following established patterns from Sprint 22 (`/list` commands):
- Command struct implements `MetaCommand` trait
- Execute method builds SQL, runs query, formats output
- Aliases registered in command registry
- Help text follows standard format

### Testing Strategy

**Test Distribution:**
- Unit tests: 60% (skew calculations, query construction, error cases)
- Integration tests: 30% (command execution, output validation)
- PTY tests: 10% (interactive behavior, tab completion)

**Critical Test Cases:**
- TC-SESS-001: `/sessions` command execution in REPL
- TC-SESS-002: `tq --sessions` batch mode execution
- TC-SESS-003: Skew calculation accuracy
- TC-SESS-004: Tab completion includes `/sessions`
- TC-SESS-005: Help text displays correctly
- TC-SESS-006: Privilege error handling
- TC-SESS-007: Empty result set handling
- TC-SESS-008: Output format compatibility (CSV, JSON, table)

---

## Risk Assessment

### Low Risk
- ✅ Established metacommand pattern (Sprint 22 `/list` commands)
- ✅ SQL query provided and tested by user
- ✅ Standard output formatting (existing table formatter)
- ✅ No new dependencies required

### Medium Risk
- ⚠️ Skew calculation logic needs validation
  - **Mitigation:** Unit tests with known values, manual verification
- ⚠️ MonitorSession table function permissions
  - **Mitigation:** Clear error messages, documentation of requirements

### No High Risks Identified

**Overall Risk Level:** LOW

---

## Definition of Done

Sprint 26 is complete when:

1. **Feature Implementation:**
   - [x] `/sessions` command implemented in REPL
   - [x] `--sessions` flag implemented in batch mode
   - [x] All 10 acceptance criteria met

2. **Quality Gates:**
   - [x] 100% automated test pass rate (unit + integration + PTY)
   - [x] Zero clippy warnings
   - [x] Zero new technical debt (`TODO`, `FIXME`, `unwrap()` on fallible ops)
   - [x] Code coverage maintained or improved

3. **Documentation:**
   - [x] Specifications updated (`repl.md`, `cli-interface.md`)
   - [x] Design docs updated (`design/repl.md`)
   - [x] User guide updated (`user/repl-guide.md`)
   - [x] Help text implemented
   - [x] Documentation accuracy verified (Phase 4 checklist)

4. **Testing:**
   - [x] Test strategy documented
   - [x] Test cases created (unit, integration, PTY)
   - [x] All tests executed with proof
   - [x] Test results documented

5. **Ship Phase:**
   - [x] Definition of Done validation passes
   - [x] Documentation accuracy verification passes
   - [x] Git commit created with proper message
   - [x] Code pushed to origin/master
   - [x] GitHub Issue #6 closed with implementation details
   - [x] Roadmap updated (status.md)

---

## GitHub Issues Integration

### Issues Selected for Sprint 26

**Issue #6:** `/sessions` command (priority-medium, enhancement)
- Status: sprint-ready → in-progress (commented in Phase 1)
- Will close in Phase 4 after successful implementation

### Issues Deferred

**Issue #7:** Horizontal paging (priority-low, enhancement)
- Status: sprint-ready (remains open for future sprint)
- Rationale documented above

### Issue Updates (Phase 1)

Will comment on Issue #6:
```
Included in Sprint 26. See planning document: docs/sprints/sprint-26-planning.md

**Scope:** `/sessions` metacommand (REPL) and `--sessions` flag (batch mode)
**Timeline:** Sprint 26 (1-2 days)
**Next Steps:** Design phase starting now
```

---

## Success Metrics

### Feature Delivery
- **Target:** 1/1 P0 features (100%)
- **Measure:** All acceptance criteria met

### Quality
- **Target:** 100% automated test pass rate
- **Measure:** cargo test, cargo clippy, PTY tests
- **Target:** Zero technical debt
- **Measure:** No `TODO`, `FIXME`, or risky patterns

### Documentation
- **Target:** 100% documentation accuracy
- **Measure:** Phase 4 verification checklist passes

### Cost Efficiency
- **Target:** $12-15 (typical feature sprint)
- **Measure:** Token usage from session transcript

### Process Maturity
- **Target:** ≤2 iterations (design + implementation)
- **Measure:** Quality validator approval iterations

---

## Related Documents

- **[GitHub Issue #6](https://github.com/anthropics/tq/issues/6)** - Original feature request
- **[Backlog](../roadmap/backlog.md)** - P1 monitoring features
- **[Status Dashboard](../roadmap/status.md)** - Current implementation status
- **[Sprint 25 Review](sprint-25-review.md)** - Previous sprint retrospective
- **[REPL Specification](../specifications/repl.md)** - REPL command requirements
- **[CLI Interface Specification](../specifications/cli-interface.md)** - Batch mode flags

---

## Approval

**Sprint Coordinator Decision:** APPROVED

**Rationale:**
- Clear, focused scope with single P0 feature
- Low risk implementation leveraging established patterns
- High value for primary user persona (DBAs)
- Detailed implementation proposal from user
- Aligns with product roadmap (system monitoring)
- Realistic timeline and cost estimates

**Next Phase:** Proceeding to Phase 2 (Design) immediately.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Sprint 26 initial planning - `/sessions` command | Sprint Coordinator |
