---
sprint: 35
start_date: 2026-02-13
target_completion: 2026-02-14
status: Planning
---

# Sprint 35 Planning: Configuration Management + REPL Polish

## Reality Check Summary

**Reviewed Sprints:** 7, 8, 9 (last 3 available reviews)
**Patterns Detected:** Healthy velocity - Sprint 7 delivered features, Sprint 8-9 recovered quality, Sprint 34 cleaned technical debt
**Decision:** Feature Sprint
**Rationale:**
- Sprint 34 achieved excellent technical debt cleanup (9.3/10)
- Quality fully restored (649/649 tests passing, 100% pass rate)
- Clean foundation established for new feature development
- No stuck issues, no accumulating debt
- Framework mature and stable

## Sprint Overview

**Sprint Goal:** Deliver project-level configuration management and complete Sprint 34 documentation polish items

**Sprint Theme:** Configuration Management + Quick Wins

This sprint introduces project-level configuration (`.tq.toml`) to enable team-shared connection profiles, complementing the existing user configuration. It also addresses minor documentation gaps from Sprint 34 and adds proper Unicode testing for identifier quoting.

---

## Objectives

1. **Enable Project-Level Configuration** - Implement `.tq.toml` support for team-shared profiles and project-specific settings
2. **Sprint 34 Documentation Polish** - Complete 2 minor documentation gaps (pager emoji, /peek default verification)
3. **Enhanced Unicode Testing** - Add proper Unicode test for SQL identifier quoting

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Project Config File (`.tq.toml`)

**Description:** Support `.tq.toml` configuration file in project directories for team-shared connection profiles and project-specific settings. Complements existing `~/.tq/config.toml` user configuration.

**Acceptance Criteria:**
- [ ] Parse `.tq.toml` from current directory (walks up to find)
- [ ] Load project config before user config (project overrides user)
- [ ] Support same TOML structure as user config (profiles, preferences)
- [ ] `tq profiles` command shows both user and project profiles
- [ ] `--profile` flag works with both user and project profiles
- [ ] Project profiles take precedence over user profiles with same name
- [ ] Comprehensive error handling (invalid TOML, permission errors)
- [ ] Documentation updated (specifications, design docs, user guides)
- [ ] Test coverage: unit tests for config loading, integration tests for profile resolution

**Reference:** [Configuration - Project Config](../specifications/configuration.md#project-config)

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### Feature 2: Sprint 34 Documentation Polish

**Description:** Address 2 minor documentation gaps identified in Sprint 34 review

**Acceptance Criteria:**
- [ ] Add emoji badge (🧪 EXPERIMENTAL) to pager section in specifications
- [ ] Verify `/peek` default count in code and update documentation if needed
- [ ] Update affected files: `docs/specifications/repl.md`, user guides if needed

**Reference:** Sprint 34 Review section 9 (Lessons Learned)

**Estimated Complexity:** Low

**Effort Estimate:** 15 minutes total

---

#### Feature 3: Enhanced Unicode Testing

**Description:** Add proper Unicode test for SQL identifier quoting to validate non-ASCII character handling

**Acceptance Criteria:**
- [ ] Create `test_quote_identifier_unicode_actual()` in `src/sql/identifiers.rs`
- [ ] Test Unicode characters: 中文 (Chinese), العربية (Arabic), emoji, etc.
- [ ] Verify double-quote escaping works with all Unicode
- [ ] All tests pass (649/649 → 650/650)

**Reference:** Sprint 34 Review section 9 (Lessons Learned)

**Estimated Complexity:** Low

**Effort Estimate:** 5 minutes

---

### P2 - Medium Priority (Nice to Have)

#### Feature 4: `.tq.toml` Example File

**Description:** Create example `.tq.toml` file with comments explaining project config usage

**Acceptance Criteria:**
- [ ] Create `.tq.toml.example` in repository root
- [ ] Include commented examples of project profiles
- [ ] Document best practices (team-shared vs user-specific)
- [ ] Reference in user guides and README

**Reference:** [Configuration - Project Config](../specifications/configuration.md#project-config)

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- **Profile editing commands** (`tq profile add/edit/delete`) - Deferred to Sprint 36+
- **Config validation command** (`tq config validate`) - Deferred to Sprint 36+
- **Keyring integration** - P2 backlog feature, not ready
- **Second TAB accepts selection** - Blocked by reedline library limitation
- **Test database setup for CI/CD** - Medium priority, defer to Sprint 36+

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features are implemented, tested, and working as specified
- [ ] All P1 features are implemented and tested
- [ ] 100% test pass rate (unit + integration tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect new features
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per docs/design/*.md)
- [ ] All features validated by quality-validator agent
- [ ] Completion validated by tq-project-manager agent
- [ ] Commit and push to GitHub

---

## Dependencies

### External Dependencies
- No external dependencies required
- Uses existing `toml` crate for parsing

### Prerequisite Work
- Sprint 34 complete (✅ Done - 2026-02-03)
- User config system complete (✅ Done - Sprint 17)
- Configuration specifications written (✅ Done)

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: Config Loading Order Complexity
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Clear precedence rules (project > user), comprehensive tests for resolution order, document behavior clearly

### Risk 2: Path Resolution Edge Cases
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** Walk up directory tree until `.tq.toml` found or filesystem root reached, handle symlinks and permissions gracefully

---

## Follow-Up Items from Sprint 34

Items carried over from Sprint 34 review:

- [ ] Address documentation minor gaps (pager emoji, /peek default) - **Included as P1 Feature 2**
- [ ] Add proper Unicode test for identifier quoting - **Included as P1 Feature 3**
- [ ] Set up test database for CI/CD (optional) - **Deferred to Sprint 36+**

**Reference:** [Sprint 34 Review](sprint-34-review.md#actions-required-before-sprint-35)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design Feature 1: Project config file specifications
- Complete Feature 2: Documentation polish (pager emoji, /peek verification)
- Update specifications: `docs/specifications/configuration.md`
- Update user guides: `docs/user/configuration-guide.md`
- Ensure UX consistency between user and project config

**Deliverables:**
- Updated `docs/specifications/configuration.md` with project config details
- Updated user guides with `.tq.toml` examples
- Documentation polish complete (pager emoji, /peek verification)

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement Feature 1: Project config loading and profile resolution
- Implement Feature 3: Unicode test for identifier quoting
- Write unit tests for config loading logic
- Update `docs/design/configuration.md` if patterns change
- Ensure no regressions in existing config system

**Deliverables:**
- Working implementation of project config (`.tq.toml` support)
- Unicode test added (`test_quote_identifier_unicode_actual()`)
- Unit tests with 100% pass rate (650+ tests)
- Updated `docs/design/configuration.md` if needed
- Zero technical debt

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test cases for project config features
- Execute all test suites (unit + integration)
- Generate test reports in `tests/results/sprint-35/`
- Validate acceptance criteria for all features
- Test config precedence (project > user), path resolution, error handling

**Deliverables:**
- Test cases in `tests/cases/TC-035-*.md`
- Test execution report in `tests/results/sprint-35/REPORT.md`
- 100% test pass rate (650+ tests)
- Validation that all acceptance criteria are met

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Assess technical debt status
- Verify all documentation is synchronized
- Provide go/no-go decision for sprint closure

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment
- Go/no-go recommendation
- Recommendations for Sprint 36

---

## Sprint Timeline

**Estimated Duration:** 1-2 days

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
  - Reviewed sprints 7, 8, 9
  - Decided: Feature Sprint

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - No user approval needed - autonomous execution

- **Phase 2: Design** (Est. 3-4 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Specifications finalized for project config
  - Technical design for config loading

- **Phase 3: Implementation & Testing** (Est. 6-8 hours)
  - Parallel execution: rust-teradata-architect + quality-validator
  - Code + tests delivered
  - Quick wins (documentation, Unicode test) completed first

- **Phase 4: Ship** (Est. 1-2 hours)
  - tq-project-manager validates completion
  - Commit and push to GitHub
  - Update roadmap

- **Phase 5: Retrospective** (Est. 1-2 hours)
  - Use `/sprint-reviewer` skill
  - Create `sprint-35-review.md`
  - Collect metrics

---

## GitHub Issues

### Selected for Sprint
- No GitHub issues selected (issue tracker is clean)

### Deferred
- N/A

**Scope Source:** Sprint based on backlog priorities from `docs/roadmap/backlog.md`

---

## Notes

### Configuration Design Considerations

**Key Decision:** Project config (`.tq.toml`) should complement, not replace, user config (`~/.tq/config.toml`)

**Use Cases:**
- **Project Config:** Team-shared connection profiles (dev, staging, prod), project-specific defaults
- **User Config:** Personal credentials, individual preferences, local overrides

**Precedence:**
1. Command-line flags (highest)
2. Project config (`.tq.toml`)
3. User config (`~/.tq/config.toml`)
4. Built-in defaults (lowest)

**Path Resolution:**
- Start in current directory
- Walk up until `.tq.toml` found or filesystem root reached
- Cache resolved path for session

### Sprint 34 Context

Sprint 34 was an excellent maintenance sprint (9.3/10) that:
- Eliminated code duplication
- Hardened security (SQL identifier quoting)
- Synchronized documentation
- Achieved 649/649 tests passing (100%)
- Left clean foundation for Sprint 35

Sprint 35 builds on this foundation by adding valuable user-facing features while addressing minor polish items.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-13 | 1.0 | Initial sprint plan - Project config + documentation polish | Sprint Coordinator |
