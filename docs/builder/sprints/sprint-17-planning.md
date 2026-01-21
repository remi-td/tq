---
sprint: 17
start_date: 2026-01-21
target_completion: 2026-01-21
status: Planning
---

# Sprint 17 Planning: Configuration UX Completion

## Sprint Overview

**Sprint Goal:** Complete the configuration user experience by implementing help subcommands, fixing security issues, and adding profile management commands.

**Sprint Theme:** Configuration UX Polish - Building on Sprint 16's configuration foundation to deliver a complete, secure, and user-friendly configuration experience.

---

## Reality Check Summary

**Reviewed Sprints:** 14, 15, 16

**Patterns Detected:** None - Healthy velocity

**Decision:** Feature Sprint

**Rationale:**
- All three previous sprints achieved 100% objective completion
- Zero technical debt maintained across all sprints
- Sprint 16 delivered configuration foundation with clear P1 follow-up work identified
- No stuck issues, no framework problems, no repeating bugs
- Natural progression: Sprint 16 laid foundation, Sprint 17 completes the user experience

---

## Objectives

High-level objectives for this sprint:

1. **Complete Help System**: Implement promised help subcommands for configuration and credentials
2. **Security Hardening**: Fix security check ordering and enforce password file permissions
3. **Profile Management**: Add profile listing capability for better discoverability
4. **Code Quality**: Address minor code duplication and inconsistencies
5. **Documentation**: Synchronize all documentation with new features

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Help Subcommands (`tq help config` and `tq help credentials`)

**Description:** Implement the help subcommands promised in Sprint 16 configuration help text. Currently help text references these commands but they return "unrecognized subcommand" errors.

**Acceptance Criteria:**
- [ ] `tq help config` displays comprehensive configuration file documentation
- [ ] `tq help credentials` displays password management guide
- [ ] Help content includes TOML examples, file locations, and security best practices
- [ ] Error handling: Unknown help topics display available topics
- [ ] Tests: Unit tests for help command routing and content generation
- [ ] Documentation: `detailed-specifications/cli-interface.md` updated with help subcommands

**Reference:** `sprint-16-review.md` section 7 (P1 recommendations), `detailed-specifications/configuration.md` v2.0.0 lines 628-701

**Estimated Complexity:** Medium (3-4 hours)

**Sprint 16 Context:** Help text in `--help` output promises these subcommands but they were deferred. Users see error: "unrecognized subcommand 'config'". This affects user trust and onboarding experience.

---

#### Feature 2: Security Check Ordering Fix

**Description:** Fix critical security issue in `src/main.rs` where password file is read before validating file permissions, creating a race condition.

**Acceptance Criteria:**
- [ ] `validate_password_file_permissions` called BEFORE `read_to_string` in `read_password_if_needed`
- [ ] Security check prevents reading insecure files before any file content is accessed
- [ ] Behavior matches `config.rs` pattern (correct order)
- [ ] Tests: Unit test verifying permission check happens first
- [ ] No regressions in password file functionality

**Reference:** `sprint-16-review.md` section 6 (Code Quality, Issue 2), `src/main.rs` function `read_password_if_needed`

**Estimated Complexity:** Low (1 hour)

**Sprint 16 Context:** Code review identified that main.rs reads password file content before checking permissions. config.rs has correct order. This is a Medium priority security concern.

---

### P1 - High Priority (Should Have)

#### Feature 3: Password File Permission Enforcement

**Description:** Align implementation with specification by enforcing (not just warning) password file permission requirements (0600).

**Acceptance Criteria:**
- [ ] Password file with permissions other than 0600 results in error (not warning)
- [ ] Error message explains security risk and provides fix command (`chmod 0600 ...`)
- [ ] Consistent behavior between `main.rs` and `config.rs`
- [ ] Tests: Integration tests for insecure permission scenarios
- [ ] Documentation: `detailed-specifications/configuration.md` clarifies enforcement vs warning

**Reference:** `sprint-16-review.md` section 5 (UX Review, Password Security), `detailed-specifications/configuration.md` v2.0.0 security section

**Estimated Complexity:** Low (1 hour)

**Sprint 16 Context:** Specification requires 0600 permissions. Current implementation warns but allows insecure files. UX review rated this 10/10 but noted inconsistency between warning and enforcement.

---

#### Feature 4: Profile Listing Command

**Description:** Add `tq profiles` command to list available connection profiles from config file, improving profile discoverability.

**Acceptance Criteria:**
- [ ] `tq profiles` command lists all available profiles from `~/.config/tq/config.toml`
- [ ] Output shows profile names and partial connection info (host, database, but NOT passwords)
- [ ] Error handling: No config file displays helpful message with setup instructions
- [ ] Error handling: Empty profiles section displays "No profiles defined"
- [ ] Tests: Unit tests for profile listing logic, integration test with sample config
- [ ] Documentation: `detailed-specifications/cli-interface.md` updated with new command

**Reference:** `sprint-16-review.md` section 5 (UX Review, Enhancement Opportunity), section 7 (P2 recommendations)

**Estimated Complexity:** Medium (2-3 hours)

**Sprint 16 Context:** Users currently need `cat ~/.config/tq/config.toml | grep '^\[profiles\.'` to discover profiles. P2 priority but high value for usability.

---

### P2 - Medium Priority (Nice to Have)

#### Feature 5: Logmech Parsing Refactoring

**Description:** Eliminate code duplication between `config.rs` and `main.rs` by making `config::parse_logmech` public and reusing it.

**Acceptance Criteria:**
- [ ] Make `config::parse_logmech` public (or create shared function)
- [ ] Replace inline parsing in `main.rs` with function call
- [ ] All existing tests pass (no behavior changes)
- [ ] No new dependencies or complexity introduced
- [ ] Code review confirms DRY principle applied correctly

**Reference:** `sprint-16-review.md` section 6 (Code Quality, Issue 3)

**Estimated Complexity:** Low (1 hour)

**Sprint 16 Context:** Minor code duplication identified. Low priority but easy win for code maintainability.

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- **Profile editing/creation commands** - `tq config add-profile`, `tq config edit-profile` (deferred to Sprint 18+)
  - Rationale: Sprint 17 focuses on completing Sprint 16 foundation, not extending it

- **Configuration file validation command** - `tq config validate` (deferred to Sprint 18+)
  - Rationale: Not in Sprint 16 recommendations, adds complexity

- **Profile import/export functionality** - (deferred to Sprint 18+)
  - Rationale: Advanced feature, Sprint 17 is about polish not new capabilities

- **Interactive config wizard** - `tq config init` (deferred to Sprint 19+)
  - Rationale: Nice-to-have, but manual config file creation is documented and sufficient

- **Architectural refactoring** - Config loading redesign
  - Rationale: Sprint 16 code quality is excellent (9.4/10), no refactoring needed

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features (help subcommands, security fix) are implemented, tested, and working
- [ ] All P1 features (permission enforcement, profiles command) are implemented and tested
- [ ] 100% test pass rate (unit + integration + interactive tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation synchronized: specifications.md, cli-interface.md, configuration.md
- [ ] Zero technical debt introduced
- [ ] Zero build/clippy warnings (maintained from Sprint 16)
- [ ] Code quality meets rust-architecture.md standards
- [ ] All features validated by quality-validator agent
- [ ] Completion validated by tq-project-manager agent
- [ ] Sprint 16 P1 action items completed (help subcommands, security fix, permission enforcement)

---

## Dependencies

### External Dependencies
- None - All work uses existing dependencies (figment, clap, secrecy)

### Prerequisite Work
- Sprint 16 configuration foundation must be complete (✅ DONE)
- All Sprint 16 tests passing (✅ 272/272 tests passing)

### Blockers
- None identified

**Low Risk Sprint:** No external dependencies, no database changes, builds on proven Sprint 16 foundation.

---

## Risks & Mitigation

### Risk 1: Help Subcommand Implementation Complexity
- **Probability:** Low
- **Impact:** Medium (P0 feature could take longer than estimated)
- **Mitigation:** Use existing clap subcommand patterns from `tq repl` and `tq query`. Help content is well-documented in configuration.md v2.0.0. If complexity exceeds estimate, reduce help content detail (links to docs) rather than defer feature.

### Risk 2: Password Permission Enforcement Breaks Existing Workflows
- **Probability:** Low
- **Impact:** Medium (users with insecure files will see errors instead of warnings)
- **Mitigation:** Error messages provide clear fix command (`chmod 0600 ...`). This is the specified behavior. Document breaking change in sprint review. Consider adding `--allow-insecure-permissions` flag if user feedback indicates need (post-sprint).

### Risk 3: Test Execution Environment Issues
- **Probability:** Low
- **Impact:** Low (interactive tests might have PTY issues like Sprint 16)
- **Mitigation:** Sprint 16 established PTY handling patterns. Reuse graceful degradation approach with clear warnings. Document any new PTY limitations discovered.

---

## Action Items from Previous Sprint

Items carried over from Sprint 16 review that need to be addressed:

- [x] Reality Check: Review last 3 sprints for patterns (Phase 0 complete)
- [ ] Implement `tq help config` subcommand (P1 from sprint-16-review.md section 7)
- [ ] Implement `tq help credentials` subcommand (P1 from sprint-16-review.md section 7)
- [ ] Fix security check ordering in main.rs (P1 from sprint-16-review.md section 6)
- [ ] Add password file permission validation (P1 from sprint-16-review.md section 5)
- [ ] Add `tq profiles` command (P2 from sprint-16-review.md section 7)
- [ ] Refactor logmech parsing to eliminate duplication (P2 from sprint-16-review.md section 6)

**Reference:** `docs/builder/sprints/sprint-16-review.md` sections 5, 6, 7

**Note:** All action items are in sprint scope except documentation enhancements (deferred to Sprint 18).

---

## Agent Assignments

Clear assignment of responsibilities to specialized agents:

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design help subcommand output format and content
- Design profile listing command output
- Update specifications: `specifications.md`, `detailed-specifications/cli-interface.md`, `detailed-specifications/configuration.md`
- Ensure UX consistency with existing help output
- Validate error messages are actionable and user-friendly

**Deliverables:**
- Updated `specifications.md` with 🚧 status for in-progress features
- Updated `cli-interface.md` with help subcommands and profiles command
- UX design validation for all features
- Help content structured for both terminal display and reference

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement help subcommands using clap subcommand pattern
- Fix security check ordering in `src/main.rs`
- Implement password permission enforcement (align with specification)
- Implement `tq profiles` command with profile loading logic
- Refactor logmech parsing (P2, time permitting)
- Write unit tests for all new code
- Ensure zero regressions in existing functionality

**Deliverables:**
- Working implementation of all P0 and P1 features
- Unit tests with 100% pass rate
- Security fix verified and tested
- Code quality maintained (zero warnings)
- Technical debt report (expected: zero new debt)

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for help subcommands (TC### format)
- Design test cases for profiles command
- Design test cases for security check ordering
- Execute all test suites (unit + integration + interactive)
- Generate test reports in `tests/results/sprint-17/`
- Validate all acceptance criteria met
- Verify no regressions in Sprint 16 configuration features

**Deliverables:**
- Test cases in `tests/cases/TC###.md` for new features
- Test execution report in `tests/results/sprint-17/REPORT.md`
- 100% test pass rate (expected: 280+ tests)
- Validation that all P0/P1 acceptance criteria are met
- Quality verdict: APPROVED/BLOCKED

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion against Definition of Done
- Assess technical debt status (expected: zero)
- Verify documentation synchronized with implementation
- Provide go/no-go decision for sprint closure
- Create git commit and push to GitHub after validation passes

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment
- Go/no-go recommendation
- Git commit with comprehensive message
- Recommendations for Sprint 18

---

## Sprint Timeline

**Estimated Duration:** 1 day (8-12 hours agent work)

### Phase Breakdown

- **Phase 0: Reality Check** (✅ Complete)
  - Reviewed Sprints 14, 15, 16
  - Pattern detection: Healthy velocity, no warning signs
  - Decision: Feature Sprint

- **Phase 1: Planning** (✅ Complete)
  - Sprint planning document created
  - Scope defined: 5 features (2 P0, 2 P1, 1 P2)
  - Agent assignments clear

- **Phase 2: Design** (Est. 2-3 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - cli-ux-designer: Design help output, profiles output, update specifications
  - rust-teradata-architect: Assess implementation feasibility, identify patterns
  - Deliverable: Design document + feasibility assessment

- **Phase 3: Build & Test** (Est. 5-7 hours)
  - Parallel execution: rust-teradata-architect (implement) + quality-validator (design tests)
  - rust-teradata-architect: Implement all features, write unit tests
  - quality-validator: Design test cases, execute tests, iterate on failures
  - Deliverable: Working code + 100% test pass rate

- **Phase 4: Ship** (Est. 1-2 hours)
  - tq-project-manager: Validate against Definition of Done
  - Create git commit with comprehensive message
  - Push to GitHub
  - Deliverable: Sprint shipped, v1.7.0 tagged (minor version bump for new commands)

- **Phase 5: Retrospective** (Est. 2-3 hours)
  - Use `/sprint-reviewer` skill to launch 3 agents in parallel
  - Generate sprint-17-review.md with metrics, lessons, recommendations
  - Collect token/cost metrics for framework optimization
  - Deliverable: Sprint review document

**Critical Path:** Phase 3 (Build & Test) is longest duration. Help subcommand implementation is P0 and most complex (3-4 hours).

---

## Notes

### Context from Sprint 16

Sprint 16 delivered excellent configuration foundation:
- Configuration file system with profiles (P1 complete)
- Password security with `password_file` field (P2 complete)
- Global `--profile` flag (P2 complete)
- 100% test pass rate (272 tests)
- Outstanding UX (9/10 usability score)

**Key Quote from Sprint 16 Review:**
> "Help text promises `tq help config` but subcommand not yet implemented (deferred to Sprint 17)"

Sprint 17 completes what Sprint 16 promised. Users currently see error when running promised commands.

### Version Bump Consideration

Sprint 17 adds new user-facing commands (`tq help config`, `tq profiles`). This justifies minor version bump to **v1.7.0**.

Breaking change: Password permission enforcement changes from warning to error. Document in sprint review and CHANGELOG.

### Testing Strategy

- Unit tests: Help content generation, profile listing logic, permission checks
- Integration tests: Full command execution with sample config files
- Interactive tests: No new interactive tests needed (REPL not changed)
- Security tests: Permission validation before file read, insecure file handling

### Success Indicators

Sprint 17 is successful if:
1. Users can run `tq help config` and `tq help credentials` (no more "unrecognized subcommand")
2. Users can run `tq profiles` to discover available profiles
3. Password security is enforced (0600 permissions required)
4. Security race condition is eliminated
5. All tests pass, zero warnings, zero technical debt

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0 | Initial Sprint 17 plan - Configuration UX Completion | Sprint Coordinator |
