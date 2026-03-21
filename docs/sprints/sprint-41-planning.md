---
sprint: 41
start_date: 2026-03-20
target_completion: 2026-03-20
status: Planning
---

# Sprint 41 Planning: GitHub Releases & Binary Distribution

## Sprint Overview

**Sprint Goal:** Enable frictionless installation of tq by providing prebuilt binaries via GitHub Releases and a one-liner install script, eliminating the Rust toolchain requirement for end users.

**Sprint Theme:** Binary Distribution & DevOps

---

## Reality Check Summary

- Reviewed sprints: 38, 39, 40
- Patterns detected: Recurring spec/implementation alignment (manageable), integration tests deferred (recurring but minor)
- Decision: Feature Sprint
- Rationale: Healthy velocity (100% test pass rate, growing test suite). User flagged Issue #27 (binary distribution) as top priority. No crisis or stuck issues.

---

## Objectives

1. **GitHub Actions Release Workflow** - Automated CI/CD pipeline that builds, packages, and publishes release binaries for 5 target platforms when a version tag is pushed
2. **Cross-compilation build.rs fix** - Update build.rs to use `CARGO_CFG_TARGET_OS`/`CARGO_CFG_TARGET_ARCH` instead of `cfg!(target_os)` for correct library selection during cross-compilation
3. **Install Script** - POSIX-compatible shell script for one-liner installation on Linux/macOS
4. **Sprint 40 Remediation** - Resolve /p alias, eliminate execute duplication, use LazyLock for regex

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: GitHub Actions Release Workflow (Issue #27)

**Description:** Create `.github/workflows/release.yml` triggered by `v*` tags that builds release binaries for all major platforms, packages them with the correct teradatasql native library, creates a GitHub Release, and uploads all artifacts with SHA256 checksums.

**Acceptance Criteria:**
- [ ] AC-1: `.github/workflows/release.yml` triggers on `v*` tag push
- [ ] AC-2: Builds for `x86_64-unknown-linux-gnu` on Ubuntu runner
- [ ] AC-3: Builds for `aarch64-unknown-linux-gnu` using cross-compilation
- [ ] AC-4: Builds for `x86_64-apple-darwin` on macOS runner
- [ ] AC-5: Builds for `aarch64-apple-darwin` on macOS runner
- [ ] AC-6: Builds for `x86_64-pc-windows-msvc` on Windows runner
- [ ] AC-7: Each artifact packaged as `tq-<version>-<target>.tar.gz` (Linux/macOS) or `.zip` (Windows)
- [ ] AC-8: Each package includes tq binary + teradatasql native library + LICENSE
- [ ] AC-9: SHA256 `checksums.txt` generated and uploaded to release
- [ ] AC-10: GitHub Release created automatically with all artifacts
- [ ] AC-11: Release body includes version info and download links

**Reference:** Issue #27

**Estimated Complexity:** High

---

#### Feature 2: Cross-Compilation Build.rs Fix

**Description:** The current `build.rs` uses `cfg!(target_os)` which checks the HOST platform, not the TARGET. For cross-compilation to work, it must use `CARGO_CFG_TARGET_OS` and `CARGO_CFG_TARGET_ARCH` environment variables provided by Cargo during builds.

**Acceptance Criteria:**
- [ ] AC-12: build.rs uses `CARGO_CFG_TARGET_OS` instead of `cfg!(target_os)`
- [ ] AC-13: build.rs uses `CARGO_CFG_TARGET_ARCH` to select correct library variant (e.g., `teradatasql.arm.so` for aarch64)
- [ ] AC-14: Local native builds (cargo build) still work correctly
- [ ] AC-15: All existing tests pass with updated build.rs

**Reference:** Cross-compilation analysis during Phase 0

**Estimated Complexity:** Medium

---

#### Feature 3: Install Script

**Description:** A POSIX-compatible shell script that detects OS/architecture, downloads the correct binary from the latest GitHub Release, verifies the checksum, and installs to `~/.local/bin`.

**Acceptance Criteria:**
- [ ] AC-16: `install.sh` detects Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- [ ] AC-17: Downloads correct binary from latest GitHub Release using GitHub API
- [ ] AC-18: Verifies SHA256 checksum before installation
- [ ] AC-19: Installs to `~/.local/bin` by default, respects `TQ_INSTALL_DIR` override
- [ ] AC-20: Provides clear error for unsupported platforms (Windows, musl)
- [ ] AC-21: Script is POSIX-compatible (no bashisms, works with sh/dash)
- [ ] AC-22: Usage: `curl -sSL <raw-url>/install.sh | sh`

**Reference:** Issue #27

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### Feature 4: Sprint 40 Remediation

**Description:** Address P0 and select P1 recommendations from Sprint 40 review.

**Acceptance Criteria:**
- [ ] AC-23: Resolve `/p` alias - add to spec and help text (it's useful, keep it)
- [ ] AC-24: Eliminate `execute`/`execute_with_params` duplication in query.rs (accept optional `&ParamStore`)
- [ ] AC-25: Use `LazyLock` for regex compilation in params.rs
- [ ] AC-26: All existing 855 tests pass after changes

**Reference:** Sprint 40 review, recommendations #1-#3, #6

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- `cargo-binstall` metadata in Cargo.toml - future enhancement
- Windows installer (.msi) - binary .zip is sufficient for v1
- musl-based Linux binaries - teradatasql library requires glibc
- Homebrew formula - future distribution channel
- Self-updating mechanism - future feature
- CI for PRs (build/test on push) - separate future workflow
- Sprint 40 P1 items (output format alignment) - deferred to Sprint 42

---

## GitHub Issues

### Selected for Sprint
- #27: GitHub Releases with cross-compiled binaries and install script (priority-high, enhancement) - **P0**

### Deferred
- #24: Query Drill-Down - partially complete, remaining items P2
- #17, #19, #20: PMON features - medium priority
- #21, #22, #23, #25: PMON advanced features - low priority

---

## Dependencies

### External Dependencies
- GitHub Actions runners: ubuntu-latest, macos-latest, windows-latest
- `cross-rs/cross` for aarch64-linux-gnu cross-compilation
- Pre-built teradatasql libraries in teradatarustapi repo (already available for all targets)

### Prerequisite Work
- Sprint 40 complete (done)

### Blockers
- None identified. All teradatasql native libraries are pre-built and available.

---

## Risks & Mitigation

### Risk 1: Cross-compilation library selection
- **Probability:** Medium
- **Impact:** High
- **Mitigation:** build.rs fix (Feature 2) uses CARGO_CFG_TARGET_OS/ARCH. Verify locally with `--target` flag before CI.

### Risk 2: teradatasql library compatibility on GitHub runners
- **Probability:** Low
- **Impact:** High
- **Mitigation:** Libraries are pre-built Go binaries with minimal system dependencies (glibc 2.17+). Ubuntu 20.04+ runners have glibc 2.31+.

### Risk 3: aarch64 cross-compilation complexity
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Use `cross-rs/cross` which provides Docker-based cross-compilation. Alternatively, use GitHub ARM64 runners if available.

### Risk 4: Session budget for DevOps-heavy sprint
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Workflow YAML and install script are declarative/scripting - faster than typical Rust feature implementation. Remediation items are small.

---

## Action Items from Previous Sprint

- [ ] Resolve `/p` alias: add to spec and help (Sprint 40 rec #2)
- [ ] Eliminate `execute`/`execute_with_params` duplication in query.rs (Sprint 40 rec #3)
- [ ] Use `LazyLock` for regex in params.rs (Sprint 40 rec #6)

**Reference:** `docs/sprints/sprint-40-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update README Installation section with new install methods
- Update `/p` alias in specifications and help text
- Review install script UX (error messages, progress output)

**Deliverables:**
- Updated README with Installation section
- Updated specs for /p alias
- Install script UX review

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Fix build.rs for cross-compilation support
- Create `.github/workflows/release.yml`
- Create `install.sh` script
- Sprint 40 remediation: eliminate duplication, LazyLock regex
- Write unit tests for build.rs changes
- Update Cargo.toml version to 1.22.0

**Deliverables:**
- Cross-compilation-ready build.rs
- Complete release workflow
- POSIX install script
- Remediation code changes
- All tests passing

---

### quality-validator (Sonnet)
**Responsibilities:**
- Verify all 855+ tests pass after build.rs and remediation changes
- Validate workflow YAML syntax
- Validate install.sh with shellcheck
- Test build.rs changes with local cargo build

**Deliverables:**
- Test execution report (cargo test output)
- Workflow YAML validation
- Install script validation
- 100% test pass rate

---

## Files Involved

### Feature 1: Release Workflow
**New Files:**
- `.github/workflows/release.yml` - GitHub Actions release workflow

### Feature 2: Build.rs Fix
**Modified Files:**
- `build.rs` - Use CARGO_CFG_TARGET_OS/ARCH for cross-compilation

### Feature 3: Install Script
**New Files:**
- `install.sh` - POSIX install script

### Feature 4: Remediation
**Modified Files:**
- `src/commands/query.rs` - Merge execute/execute_with_params
- `src/commands/repl/mod.rs` - Merge execute/execute_with_params
- `src/params.rs` - LazyLock for regex
- `docs/specifications/repl.md` - Add /p alias documentation
- `src/help/params.txt` or equivalent - Add /p alias to help

### Documentation
- `Readme.md` - Installation section update
- `docs/roadmap/status.md` - Update after sprint
- `docs/roadmap/backlog.md` - Add binary distribution entry

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] Release workflow YAML is valid and complete
- [ ] Install script is POSIX-compliant and handles all target platforms
- [ ] build.rs correctly selects target libraries for cross-compilation
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met
- [ ] Documentation updated (README Installation section)
- [ ] Zero technical debt introduced
- [ ] Sprint 40 remediation items resolved

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-20 | 1.0 | Initial sprint plan | Sprint Coordinator |
