---
sprint: 16
start_date: 2026-01-21
target_completion: 2026-01-21
status: Planning
---

# Sprint 16 Planning: Interactive Test Validation & Configuration Foundation

## Sprint Overview

**Sprint Goal:** Validate Sprint 13-15 interactive tests with live database, then establish configuration file foundation for connection profiles and user preferences.

**Sprint Theme:** "Validation First, Then Configuration" - Complete test validation work from Sprint 15, then return to feature development with full confidence.

---

## Reality Check Summary

**Reviewed Sprints:** 12, 14, 15 (most recent 3 available)

**Patterns Detected:**
- ✅ Healthy velocity across all 3 sprints
- ✅ Quality infrastructure operational and validated (Sprint 14-15)
- ✅ Zero technical debt across last 3 sprints
- ✅ 100% test pass rate maintained
- ✅ Framework improvements working effectively

**Decision:** Feature Sprint

**Rationale:** No stuck issues, no repeating bugs, quality infrastructure validated. Sprint 15 identified clear next steps: validate interactive tests with live database (P0), then return to feature development. Configuration files are the highest-priority unimplemented features (P1).

---

## Objectives

1. **Validate Interactive Test Suite** - Execute all 20 interactive tests with live Teradata database
2. **Clarify Coverage Metrics** - Document that 40.07% is automated coverage, total coverage ~85%
3. **Configuration File Foundation** - Implement user config file (`~/.tq/config.toml`) with connection profiles
4. **Configuration File Specification** - Complete detailed specification for configuration features

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Interactive Test Execution Validation

**Description:** Execute all 20 interactive tests (15 from Sprint 13, 5 from Sprint 15) against live Teradata database to validate they work correctly in real environment

**Acceptance Criteria:**
- [ ] Set up TQ_LOGON environment variable with test database credentials
- [ ] Execute `cargo test --test interactive_tests -- --ignored` successfully
- [ ] All 20 interactive tests pass with live database
- [ ] Document any failures with root cause analysis
- [ ] Fix any test failures before proceeding to P1 features
- [ ] Generate test execution report in `tests/results/sprint-16/`

**Reference:** Sprint 15 recommendation P0 - [Sprint 15 Review](sprint-15-review.md#recommendations-for-sprint-16)

**Estimated Complexity:** Low (1 hour execution + potential fixes)

**Rationale:** Sprint 15 tests validated via code review but not executed against live database. Real execution provides confidence before feature work.

---

#### Feature 2: Coverage Metrics Documentation

**Description:** Clarify that 40.07% baseline is "automated coverage" and document total coverage including interactive tests

**Acceptance Criteria:**
- [ ] Update testing-guidelines.md with coverage definitions
- [ ] Document "automated coverage" (40.07% measured by cargo-tarpaulin)
- [ ] Document "total coverage" (~85% estimated including interactive tests)
- [ ] Explain that REPL modules require interactive tests, not just unit tests
- [ ] Document coverage expectations by module type
- [ ] Update specifications.md if needed

**Reference:** Sprint 15 recommendation P1 - [Sprint 15 Review](sprint-15-review.md#recommendations-for-sprint-16)

**Estimated Complexity:** Low (30 minutes)

**Rationale:** Prevent confusion about "low" 40.07% coverage. This is expected and appropriate given REPL architecture.

---

### P1 - High Priority (Should Have)

#### Feature 3: User Configuration File (`~/.tq/config.toml`)

**Description:** Implement user-level configuration file for default preferences and connection profiles

**Acceptance Criteria:**
- [ ] Parse `~/.tq/config.toml` on startup (if exists)
- [ ] Support `[defaults]` section for preferences (format, editor_mode, syntax_highlighting, paging, timing)
- [ ] Support `[profiles.<name>]` sections for named connections
- [ ] Profile fields: host, port, database, user, logmech, password_file (optional)
- [ ] CLI flags override config file settings
- [ ] Environment variables override config file settings
- [ ] Clear error messages for invalid TOML syntax
- [ ] Config file is optional - tool works without it
- [ ] Help text documents config file location and structure
- [ ] Unit tests for config parsing logic

**Reference:** [detailed-specifications/configuration.md](../detailed-specifications/configuration.md)

**Estimated Complexity:** Medium (4-6 hours)

**Rationale:** Most requested feature. Enables named connection profiles and persistent preferences.

---

#### Feature 4: Configuration Specification Completion

**Description:** Complete detailed specification for configuration management features

**Acceptance Criteria:**
- [ ] Document config file format and structure in detail
- [ ] Document precedence order (CLI > env vars > config file > defaults)
- [ ] Specify connection profile format and fields
- [ ] Document default preferences available
- [ ] Provide complete examples for common use cases
- [ ] Specify error handling for invalid config
- [ ] Document security considerations (password_file vs inline passwords)
- [ ] Update specifications.md with configuration status

**Reference:** [detailed-specifications/configuration.md](../detailed-specifications/configuration.md)

**Estimated Complexity:** Low (2-3 hours)

**Rationale:** Specification must be complete before implementation. Guides architect and validates design.

---

### P2 - Medium Priority (Nice to Have)

#### Feature 5: Profile Selection CLI Flag

**Description:** Add `--profile <name>` flag to select connection profile from config file

**Acceptance Criteria:**
- [ ] `--profile <name>` selects named profile from config
- [ ] Error if profile doesn't exist
- [ ] Profile settings can be overridden by other CLI flags
- [ ] Help text documents --profile flag
- [ ] Unit tests for profile selection logic

**Reference:** [detailed-specifications/configuration.md](../detailed-specifications/configuration.md)

**Estimated Complexity:** Low (1-2 hours)

**Rationale:** Natural extension of Feature 3. Makes profiles immediately useful.

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- **Project-level config file (`.tq.toml`)** - Deferred to Sprint 17+
- **Keyring integration** - Requires external dependencies, deferred
- **Config file validation command** - Nice-to-have, deferred
- **Config file migration tools** - No legacy config to migrate
- **Advanced transaction control** - Separate sprint focus
- **Variable substitution** - Separate sprint focus
- **Streaming large results** - Requires different architecture

**Rationale:** Sprint 16 establishes configuration foundation. Advanced features build on this in future sprints.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features are implemented, tested, and working as specified
- [ ] All 20 interactive tests pass with live database (P0)
- [ ] Coverage metrics documented in testing-guidelines.md (P0)
- [ ] User config file implemented and tested (P1)
- [ ] Configuration specification complete (P1)
- [ ] 100% test pass rate (unit + integration + interactive tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect new features
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per rust-architecture.md)
- [ ] All features validated by quality-validator agent
- [ ] Completion validated by tq-project-manager agent

---

## Dependencies

### External Dependencies
- `toml` crate for TOML parsing (add to Cargo.toml)
- `serde` and `serde_derive` for config deserialization (already in project)
- Test database with TQ_LOGON credentials configured

### Prerequisite Work
- ✅ Sprint 15 complete (interactive tests written, coverage baseline established)
- ✅ Sprint 14 quality infrastructure operational (DoD, testing-checklist)
- ✅ Configuration specification exists (needs completion)

### Blockers
- **Test Database Access:** Interactive test validation requires live Teradata database
  - **Mitigation:** User will provide TQ_LOGON credentials via .env file (already documented)

---

## Risks & Mitigation

### Risk 1: Interactive Tests May Fail with Live Database
- **Probability:** Low
- **Impact:** Medium (blocks P0, delays feature work)
- **Mitigation:** Tests designed following existing patterns, validated via code review. If failures occur, architect will fix before proceeding to P1.

### Risk 2: TOML Parsing Edge Cases
- **Probability:** Low
- **Impact:** Low (clear error messages handle invalid config)
- **Mitigation:** Use well-tested `toml` crate. Comprehensive unit tests for parsing logic. Clear error messages guide users.

### Risk 3: Config File Precedence Complexity
- **Probability:** Low
- **Impact:** Low (specification clarifies precedence)
- **Mitigation:** Document precedence clearly (CLI > env > config > defaults). Unit tests validate precedence logic.

---

## Action Items from Previous Sprint

Items carried over from Sprint 15 retrospective:

- [x] **Run interactive tests with live database** - P0 objective in Sprint 16 (Feature 1)
- [x] **Clarify coverage metrics in docs** - P0 objective in Sprint 16 (Feature 2)
- [x] **Plan next feature sprint** - Sprint 16 planning complete (this document)
- [ ] **Evaluate CI test database setup** - Deferred to Sprint 17+ (requires infrastructure work)
- [ ] **Update Rust toolchain for llvm-cov** - Low priority, optional (cargo-tarpaulin works fine)

**Reference:** [Sprint 15 Review - Action Items](sprint-15-review.md#action-items)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Complete configuration.md specification with full detail
- Update specifications.md with Sprint 16 status and configuration features
- Design user-facing help text and error messages for config file
- Validate configuration UX and ergonomics
- Review config file examples for clarity

**Deliverables:**
- Updated `detailed-specifications/configuration.md` with complete specification
- Updated `specifications.md` with 🚧 status for in-progress features
- UX validation report for configuration features
- Config file examples and documentation

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Execute all 20 interactive tests with live database (P0)
- Implement config file parsing and profile management (P1)
- Add --profile flag for profile selection (P2 if time permits)
- Write unit tests for all config parsing logic
- Update rust-architecture.md if config loading changes architecture
- Fix any interactive test failures

**Deliverables:**
- Test execution report showing all 20 tests passing
- Working implementation of config file features
- Unit tests for config parsing with 100% pass rate
- Updated rust-architecture.md if needed
- Zero technical debt

---

### quality-validator (Sonnet)
**Responsibilities:**
- Validate interactive test execution results (P0)
- Design test cases for config file features (P1)
- Execute all test suites (unit + integration + interactive)
- Generate comprehensive quality report in `tests/results/sprint-16/`
- Validate acceptance criteria for all features
- Update testing-guidelines.md with coverage clarifications (P0)

**Deliverables:**
- Test execution report in `tests/results/sprint-16/REPORT.md`
- Test case designs for config features in `tests/cases/TC###.md`
- Updated testing-guidelines.md with coverage documentation
- 100% test pass rate validation
- Quality approval or blocking concerns

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Assess technical debt status
- Verify all documentation synchronized
- Provide go/no-go decision for sprint closure
- Create sprint-16-review.md

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment
- Go/no-go recommendation
- Sprint 16 review document
- Recommendations for Sprint 17

---

## Sprint Timeline

**Estimated Duration:** 1 day

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
  - Reviewed last 3 sprints
  - Decision: Feature Sprint

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - Objectives and scope defined

- **Phase 2: Design** (Est. 2-3 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Configuration specification completed
  - Feasibility assessment for config implementation

- **Phase 3: Implementation** (Est. 4-6 hours)
  - Parallel execution: rust-teradata-architect + quality-validator
  - Interactive tests executed (P0)
  - Config file implementation (P1)
  - Tests designed and executed

- **Phase 4: Validation & Ship** (Est. 1 hour)
  - quality-validator final validation
  - tq-project-manager completion assessment
  - Git commit and push

- **Phase 5: Retrospective** (Est. 30 minutes)
  - Invoke /sprint-reviewer skill
  - Create sprint-16-review.md
  - Document lessons learned

---

## Notes

- **Test Database Required:** Sprint 16 P0 requires live Teradata database access. User will provide credentials via TQ_LOGON environment variable.

- **Configuration Priority:** Config files are highest-priority unimplemented feature (P1 status). Multiple sprints have deferred this work. Sprint 16 is the right time to implement foundation.

- **Incremental Approach:** Sprint 16 implements user config file only. Project-level config (`.tq.toml`) deferred to Sprint 17 to keep scope manageable.

- **Validation First:** P0 objectives (test execution, coverage docs) must complete before P1 feature work begins. This ensures quality infrastructure is validated before new features.

- **Quality Confidence:** Sprints 14-15 established and validated quality infrastructure. Sprint 16 can proceed with full confidence in test coverage and quality gates.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0 | Initial sprint plan - Interactive Test Validation & Configuration Foundation | Sprint Coordinator |
