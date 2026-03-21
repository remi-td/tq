# Sprint 44 Planning: Driver Distribution Fix & Profile Polish

**Date:** 2026-03-21
**Type:** Feature Sprint (Bug-Heavy)
**Status:** Planning

## Reality Check Summary
- Reviewed sprints: 41, 42, 43
- Patterns detected: Critical bug #31 (driver missing from binary), Sprint 43 deferred UX bugs
- Decision: Feature Sprint with P0 bugs driving scope
- Rationale: Sprints 41-43 show healthy velocity (100% test pass, single-session). However, Issue #31 is a critical blocker making ALL released binaries non-functional. Sprint 43 deferred 3 must-fix items. Combined, these form a compelling bug-fix-heavy sprint.

## Sprint Goal

Fix the critical driver distribution bug that makes released binaries non-functional, add Teradata license acceptance to the installer, and resolve Sprint 43's deferred profile management UX issues.

## Sprint Theme

Release Pipeline Integrity & Profile UX Polish

---

## Objectives

1. **Fix driver library resolution** (Issue #31) - Binary must find teradatasql at runtime relative to executable, not hardcoded build path
2. **Add Teradata license acceptance** (Issue #31) - Install script must display license and require acceptance
3. **Fix profile flag naming** (Sprint 43 deferred) - Resolve `--auth`/`--pass-file` vs `--logmech`/`--password-file` inconsistency
4. **Implement profile delete confirmation** (Sprint 43 deferred) - TTY-interactive `[y/N]` prompt or spec update

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Runtime Driver Library Resolution (Issue #31)

**Description:** The binary currently uses `option_env!("TERADATA_LIB_DIR")` which bakes the CI build-time path (e.g., `/Users/runner/work/tq/tq/target/aarch64-apple-darwin/release`) into the binary. At runtime on user machines, this path doesn't exist. Fix: resolve library directory relative to the executable's location at runtime, with fallback chain.

**Root Cause:** `build.rs:77` sets `TERADATA_LIB_DIR` to `target_dir.display()` which is the CI runner's absolute path. `client.rs:52` uses this at runtime via `option_env!`.

**Acceptance Criteria:**
- [ ] AC-1: Binary finds teradatasql library in same directory as executable (primary path)
- [ ] AC-2: Fallback chain: exe dir → `--driver-lib-dir` flag → `TERADATA_LIB_DIR` env var → `.` (cwd)
- [ ] AC-3: Error message shows all searched paths when library not found
- [ ] AC-4: `build.rs` no longer sets `TERADATA_LIB_DIR` to absolute build path (or it's only used as last resort)
- [ ] AC-5: Release workflow still packages library alongside binary in tar.gz
- [ ] AC-6: Install script still copies library to install dir alongside binary

**Reference:** Issue #31, `src/db/client.rs:50-53`, `build.rs:77`
**Estimated Complexity:** Medium

#### Feature 2: Teradata License Acceptance in Installer (Issue #31)

**Description:** The install script must display the Teradata driver license agreement and require user acceptance before installing. Support both interactive (TTY prompt) and non-interactive (`--accept-license` flag) modes.

**Acceptance Criteria:**
- [ ] AC-7: Install script displays Teradata license summary before download
- [ ] AC-8: Interactive mode: prompts `[y/N]` for acceptance, aborts on decline
- [ ] AC-9: Non-interactive mode: `--accept-license` flag bypasses prompt
- [ ] AC-10: Piped install (`curl | sh`) detects non-TTY and requires `--accept-license`
- [ ] AC-11: License text stored in repository (not fetched remotely)
- [ ] AC-12: README updated with license acceptance instructions

**Reference:** Issue #31 user request
**Estimated Complexity:** Medium

#### Feature 3: Profile Flag Naming Fix (Sprint 43 Deferred)

**Description:** Profile subcommands use `--auth`/`--pass-file` while global args use `--logmech`/`--password-file`. This inconsistency confuses users. Fix by resolving the clap global arg conflict properly.

**Acceptance Criteria:**
- [ ] AC-13: Profile subcommands use `--logmech` (not `--auth`) for authentication mechanism
- [ ] AC-14: Profile subcommands use `--password-file` (not `--pass-file`) for password file path
- [ ] AC-15: No clap argument conflicts between global and profile-specific args
- [ ] AC-16: User guide updated to use correct flag names

**Reference:** Sprint 43 Review Section 5, `src/commands/profile.rs`, `src/cli.rs`
**Estimated Complexity:** Medium (clap global arg refactoring)

### P1 - High Priority (Should Have)

#### Feature 4: Profile Delete Confirmation (Sprint 43 Deferred)

**Description:** `tq profile delete` currently always requires `--force` flag. Implement TTY-interactive confirmation prompt as specified.

**Acceptance Criteria:**
- [ ] AC-17: TTY mode: shows `Delete profile 'name'? [y/N]` prompt
- [ ] AC-18: Non-TTY mode: requires `--force` flag (current behavior preserved)
- [ ] AC-19: `--force` flag bypasses confirmation in all modes

**Reference:** Sprint 43 Review Section 5
**Estimated Complexity:** Low

#### Feature 5: Sprint 43 Technical Debt Cleanup

**Description:** Address remaining Sprint 43 should-fix items.

**Acceptance Criteria:**
- [ ] AC-20: `TqError::SqlParseError` upgraded to struct variant with line/column fields
- [ ] AC-21: Shared `display_profiles()` helper extracted from `handle_list`/`handle_profiles`

**Reference:** Sprint 43 Review Section 7
**Estimated Complexity:** Low

### Explicitly Out of Scope

- PMON features (Issues #17-25) - Focus on fixing critical distribution bug first
- Pager improvements - Stable, low priority
- Windows build - Blocked by reedline/crossterm issue
- Keyring integration - P2 backlog item

---

## GitHub Issues

### Selected for Sprint
- #31: Teradata driver missing from binary (priority-high, bug) — P0

### Deferred
- #17-25: PMON features — Next sprint after distribution is fixed
- #24: Query Drill-Down — Partially complete, remaining work deferred

---

## Dependencies

### External Dependencies
- Teradata driver license text must be available (check teradatarustapi repo)
- `std::env::current_exe()` for runtime executable path resolution

### Prerequisite Work
- Sprint 43 complete (profile management exists, parser error handling exists)

---

## Risks & Mitigation

### Risk 1: Clap global arg refactoring breaks existing commands
- **Probability:** Medium
- **Impact:** High
- **Mitigation:** Run full test suite after refactoring. The 896 existing tests will catch regressions.

### Risk 2: `current_exe()` unreliable on some platforms
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Fallback chain ensures graceful degradation. Document known platform limitations.

---

## Action Items from Sprint 43

- [ ] Fix `--auth`/`--pass-file` vs `--logmech`/`--password-file` inconsistency (Sprint 43 #1)
- [ ] Implement TTY-interactive delete confirmation (Sprint 43 #2)
- [ ] Update user guide to match actual flag names (Sprint 43 #3)
- [ ] Extract shared `display_profiles()` helper (Sprint 43 #4)
- [ ] Upgrade `TqError::SqlParseError` to struct variant (Sprint 43 #5)

**Reference:** `docs/sprints/sprint-43-review.md` Section 7

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update `docs/specifications/cli-interface.md` for driver resolution behavior
- Update `docs/specifications/security.md` for license acceptance
- Validate profile flag naming in specifications

**Deliverables:**
- Updated specifications reflecting driver resolution and license acceptance
- Profile flag naming consistency validation

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement runtime driver resolution in `src/db/client.rs`
- Update `build.rs` to not hardcode absolute paths
- Fix clap global arg conflicts for profile commands
- Implement delete confirmation prompt
- Clean up technical debt (SqlParseError, display_profiles)
- Update install script with license acceptance

**Deliverables:**
- Working driver resolution with fallback chain
- Fixed profile flag names
- Updated install script
- All unit tests passing

---

## Files Involved

### Objective 1: Runtime Driver Resolution
**Source Files:**
- `src/db/client.rs` - Driver loading logic, fallback chain
- `build.rs` - Remove/change TERADATA_LIB_DIR absolute path
- `src/error.rs` - Enhanced error message with searched paths

**Documentation:**
- `docs/design/connection-management.md` - Driver resolution design

### Objective 2: License Acceptance
**Source Files:**
- `install.sh` - License display, TTY detection, --accept-license flag
- `Readme.md` - Updated install instructions

### Objective 3: Profile Flag Fix
**Source Files:**
- `src/cli.rs` - Clap arg definitions, global arg refactoring
- `src/commands/profile.rs` - Flag references
- `docs/user/configuration-guide.md` - User guide updates

### Objective 4: Delete Confirmation
**Source Files:**
- `src/commands/profile.rs` - TTY detection, confirmation prompt

### Objective 5: Tech Debt
**Source Files:**
- `src/error.rs` - SqlParseError struct variant
- `src/commands/profile.rs` - display_profiles helper
- `src/main.rs` - Updated handle_profiles call

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-21 | 1.0 | Initial sprint plan | Sprint Coordinator |
