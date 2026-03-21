# Sprint 41 Test Strategy: GitHub Releases & Binary Distribution

**Created:** 2026-03-20
**Author:** quality-validator
**Sprint:** Sprint 41
**Features:** GitHub Actions Release Workflow (AC-1 to AC-11), Cross-Compilation Build.rs Fix (AC-12 to AC-15), Install Script (AC-16 to AC-22), Sprint 40 Remediation (AC-23 to AC-26)

---

## Overview

Sprint 41 is a DevOps/CI sprint with a fundamentally different test profile compared to feature sprints. The primary deliverables are:

1. **Feature 1: GitHub Actions Release Workflow** - CI/CD YAML configuration that only runs on GitHub infrastructure
2. **Feature 2: Cross-Compilation Build.rs Fix** - Build script changes testable locally via `cargo build` and `cargo test`
3. **Feature 3: Install Script** - POSIX shell script statically analyzable via `shellcheck`
4. **Feature 4: Sprint 40 Remediation** - Code changes to query.rs, repl/mod.rs, params.rs testable via existing test suite

### Critical Distinction: Testable vs Non-Testable Locally

**TESTABLE LOCALLY:**
- `cargo build` succeeds after build.rs changes (AC-14)
- All 855+ existing tests pass after build.rs changes (AC-15, AC-26)
- `install.sh` passes `shellcheck` static analysis (AC-21)
- `release.yml` YAML syntax is valid (manual review)
- Sprint 40 remediation code changes do not break existing tests (AC-26)

**NOT TESTABLE LOCALLY:**
- GitHub Actions workflow actual execution (AC-1 to AC-11) - requires pushing a `v*` tag to GitHub
- Actual cross-compilation on different target architectures (aarch64-linux, arm64-mac) - requires cross-rs toolchain setup
- Binary download and checksum verification from a real GitHub Release (AC-17, AC-18)
- Multi-platform runner matrix (Ubuntu/macOS/Windows runners)
- Release artifact creation and GitHub Release publication

**Consequence:** For Features 1 and 3 (runtime behavior), the test verdict will be LIMITED/BLOCKED for the workflow execution aspects. Static analysis and structural validation remain executable locally.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: GitHub Actions Release Workflow

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-41-planning.md` lines 40-57 (AC-1 through AC-11)
- Issue #27: GitHub Releases with cross-compiled binaries and install script

**Requirements from Acceptance Criteria:**

1. AC-1: `.github/workflows/release.yml` triggers on `v*` tag push
2. AC-2: Builds for `x86_64-unknown-linux-gnu` on Ubuntu runner
3. AC-3: Builds for `aarch64-unknown-linux-gnu` using cross-compilation
4. AC-4: Builds for `x86_64-apple-darwin` on macOS runner
5. AC-5: Builds for `aarch64-apple-darwin` on macOS runner
6. AC-6: Builds for `x86_64-pc-windows-msvc` on Windows runner
7. AC-7: Each artifact packaged as `tq-<version>-<target>.tar.gz` (Linux/macOS) or `.zip` (Windows)
8. AC-8: Each package includes tq binary + teradatasql native library + LICENSE
9. AC-9: SHA256 `checksums.txt` generated and uploaded to release
10. AC-10: GitHub Release created automatically with all artifacts
11. AC-11: Release body includes version info and download links

**Feature Characteristics:**

**User Interaction Type:**
- Background Process / CI Infrastructure - This is a GitHub Actions YAML configuration that executes exclusively on GitHub-hosted runners when a `v*` tag is pushed. There is no local user interaction.

**Observable Behavior:**
- File system side effects (workflow YAML file existence and syntax)
- Network interactions (GitHub API calls, artifact uploads) - NOT testable locally

**External Dependencies:**
- GitHub Actions runners (ubuntu-latest, macos-latest, windows-latest)
- cross-rs/cross Docker images for aarch64 cross-compilation
- GitHub API for release creation
- Pre-built teradatasql native libraries in teradatarustapi cargo cache

**Validation Challenges:**
- The workflow cannot execute locally - it requires GitHub infrastructure and a real tag push
- Cross-compilation verification requires the cross toolchain to be installed
- Release artifact creation requires GitHub credentials and a repository with Actions enabled
- Multi-architecture build verification requires physical or emulated hardware

**Critical Behaviors to Validate:**
1. YAML syntax is valid (parseable by GitHub Actions) - validatable via `actionlint` or structural review
2. Trigger condition is `on: push: tags: 'v*'` - reviewable in YAML
3. All 5 target platforms are represented in the build matrix - reviewable in YAML
4. Artifact naming convention `tq-<version>-<target>` is correct - reviewable in YAML
5. checksums.txt generation step is present - reviewable in YAML

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "CLI Batch" is NOT checked - this is not a CLI feature
- "Background Process" is checked - requires CI infrastructure
- "File system side effects" is checked - YAML file creation
- "Network interactions" is checked - NOT testable locally
- "Operating system specific" is checked - multi-platform build matrix

**Derived Test Types:**

**Test Type 1: Static YAML Validation (manual review + actionlint if available)**
- **Validates:** YAML syntax correctness, trigger configuration, job matrix structure
- **Approach:** Parse YAML manually; run `actionlint` if installed; check all 5 targets present, trigger on `v*`, artifact naming
- **Rationale:** The only locally executable validation for a CI workflow
- **Gap if missing:** Syntax errors that prevent the workflow from parsing on GitHub
- **Necessity:** REQUIRED

**Test Type 2: Runtime Workflow Execution (tag push)**
- **Validates:** All AC-1 through AC-11 - actual builds, artifacts, checksums, GitHub Release
- **Approach:** Push a test tag (e.g., `v1.22.0-rc1`) to GitHub and observe the Actions run
- **Rationale:** The only way to truly validate workflow correctness
- **Gap if missing:** Cannot verify actual compilation on all platforms, artifact contents, release creation
- **Necessity:** REQUIRED for full validation but NOT EXECUTABLE in automated test suite

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| YAML static analysis | REQUIRED | Only locally executable validation | Syntax errors | MUST IMPLEMENT |
| Runtime workflow execution | REQUIRED | Validates all 11 ACs | Cannot verify builds/artifacts | NOT LOCALLY EXECUTABLE |
| Unit tests | NOT NEEDED | No Rust code in this feature | N/A | SKIP |
| Integration tests | NOT NEEDED | No application logic | N/A | SKIP |

**Summary:**
- REQUIRED but NOT LOCALLY EXECUTABLE: 1 test type (runtime execution)
- REQUIRED and LOCALLY EXECUTABLE: 1 test type (YAML static analysis)
- NOT NEEDED: 2 test types

#### 4. Specification Coverage Map

| Requirement | Test Type | Status |
|-------------|-----------|--------|
| AC-1: triggers on `v*` | YAML review | LOCALLY TESTABLE |
| AC-2 to AC-6: 5 platform targets | YAML review | LOCALLY TESTABLE (structure only) |
| AC-7: artifact naming convention | YAML review | LOCALLY TESTABLE (step names/variables) |
| AC-8: binary + native lib + LICENSE | YAML review | LOCALLY TESTABLE (packaging steps) |
| AC-9: SHA256 checksums.txt | YAML review | LOCALLY TESTABLE (step presence) |
| AC-10: GitHub Release creation | Runtime execution | NOT LOCALLY TESTABLE |
| AC-11: Release body content | Runtime execution | NOT LOCALLY TESTABLE |

#### 5. Gap Analysis

**Runtime Workflow Execution**
- **Reason for omission:** Requires GitHub infrastructure, real runners, and a tag push
- **What won't be validated:** Actual compilation success on 5 platforms, real artifact creation, GitHub Release publication
- **Risk assessment:** MEDIUM - YAML can look correct but fail at runtime due to cross-compilation issues, library path errors, or GitHub API changes
- **Mitigation:** Manual YAML review is thorough; sprint plan identifies cross-compilation risk mitigation (cross-rs Docker approach)
- **Revisit criteria:** Push a release candidate tag after sprint delivery to validate end-to-end

---

### Feature 2: Cross-Compilation Build.rs Fix

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-41-planning.md` lines 62-76 (AC-12 through AC-15)

**Requirements from Acceptance Criteria:**

1. AC-12: build.rs uses `CARGO_CFG_TARGET_OS` instead of `cfg!(target_os)`
2. AC-13: build.rs uses `CARGO_CFG_TARGET_ARCH` to select correct library variant (e.g., `teradatasql.arm.so` for aarch64)
3. AC-14: Local native builds (cargo build) still work correctly
4. AC-15: All existing tests pass with updated build.rs

**Current State:** The existing `build.rs` uses `cfg!(target_os = "macos")` and `cfg!(target_os = "windows")` which evaluate on the HOST at compile time, not the TARGET. This is correct for native builds but wrong for cross-compilation.

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic (build script - runs at compile time, selects correct native library)

**Observable Behavior:**
- File system side effects (library copied to target directory)
- Build system behavior (`cargo build` succeeds)

**External Dependencies:**
- File system access (reads cargo cache for native library)
- Build system environment variables (CARGO_CFG_TARGET_OS, CARGO_CFG_TARGET_ARCH, OUT_DIR)

**Validation Challenges:**
- Cross-compilation validation requires `--target` flag and cross toolchain installation
- The native library path search is environment-specific (cargo cache location)
- AC-12/AC-13 are code inspection items (correct env var usage) - verifiable by code review, not runtime test

**Critical Behaviors to Validate:**
1. Local `cargo build` still succeeds (AC-14) - verifies no regression in build.rs
2. All 855+ tests pass (AC-15) - verifies build.rs changes don't break compilation
3. Code uses `CARGO_CFG_TARGET_OS`/`CARGO_CFG_TARGET_ARCH` env vars (AC-12, AC-13) - code review

#### 2. Test Strategy Derivation

**Derived Test Types:**

**Test Type 1: Build Verification (cargo build)**
- **Validates:** AC-14 - local native builds still work
- **Approach:** Run `cargo build` and verify exit code 0
- **Rationale:** The most direct proof that build.rs changes don't break the build
- **Gap if missing:** Could have breaking build.rs change that passes test compilation but fails release build
- **Necessity:** REQUIRED

**Test Type 2: Regression Test Suite (cargo test)**
- **Validates:** AC-15 - all existing tests pass
- **Approach:** Run `cargo test` (excluding `#[ignore]` tests) and verify 100% pass rate
- **Rationale:** Build.rs changes affect compilation; any regression in test suite flags a problem
- **Gap if missing:** Could miss subtle runtime regressions from changed build environment
- **Necessity:** REQUIRED

**Test Type 3: Code Inspection (env var usage)**
- **Validates:** AC-12, AC-13 - correct environment variable usage
- **Approach:** Grep for `CARGO_CFG_TARGET_OS` and `CARGO_CFG_TARGET_ARCH` in build.rs; verify `cfg!(target_os)` is removed
- **Rationale:** Cannot test cross-compilation locally; code inspection is the only way to verify correctness
- **Gap if missing:** Code might still use host-platform detection instead of target detection
- **Necessity:** REQUIRED

**Test Type 4: Cross-Compilation Verification (cargo build --target)**
- **Validates:** AC-12, AC-13 at runtime - build.rs selects correct lib for target platform
- **Approach:** Run `cargo build --target aarch64-unknown-linux-gnu` (requires cross toolchain)
- **Rationale:** The actual correctness test for cross-compilation
- **Gap if missing:** Cannot verify the env var fix actually works for cross-compilation
- **Necessity:** REQUIRED for full validation but NOT EXECUTABLE without cross toolchain

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| cargo build (native) | REQUIRED | Verifies no regression in local builds | Build breakage | MUST IMPLEMENT |
| cargo test (full suite) | REQUIRED | Verifies no regressions across all tests | Test failures | MUST IMPLEMENT |
| Code inspection (env vars) | REQUIRED | Only way to verify cross-compile logic | Wrong lib selection on cross | MUST IMPLEMENT |
| cargo build --target (cross) | REQUIRED (full) | Verifies cross-compile correctness | Cross-compile failure | NOT LOCALLY EXECUTABLE |

#### 4. Specification Coverage Map

| Requirement | Test Type | Status |
|-------------|-----------|--------|
| AC-12: CARGO_CFG_TARGET_OS used | Code inspection | LOCALLY TESTABLE |
| AC-13: CARGO_CFG_TARGET_ARCH used | Code inspection | LOCALLY TESTABLE |
| AC-14: local cargo build works | cargo build | LOCALLY TESTABLE |
| AC-15: all tests pass | cargo test | LOCALLY TESTABLE |

#### 5. Gap Analysis

**Cross-Compilation Runtime Verification**
- **Reason for omission:** Requires installing cross-rs/cross and Docker, plus aarch64 cross toolchain
- **What won't be validated:** Whether `CARGO_CFG_TARGET_OS=linux CARGO_CFG_TARGET_ARCH=aarch64` actually selects `teradatasql.arm.so`
- **Risk assessment:** LOW for native builds (covered by cargo build + cargo test); MEDIUM for actual cross-compilation
- **Mitigation:** Code inspection confirms correct env var usage; the workflow itself will test cross-compilation on GitHub runners
- **Revisit criteria:** If the GitHub Actions release workflow fails on aarch64, revisit build.rs logic

---

### Feature 3: Install Script

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-41-planning.md` lines 79-96 (AC-16 through AC-22)

**Requirements from Acceptance Criteria:**

1. AC-16: Detects Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
2. AC-17: Downloads correct binary from latest GitHub Release using GitHub API
3. AC-18: Verifies SHA256 checksum before installation
4. AC-19: Installs to `~/.local/bin` by default, respects `TQ_INSTALL_DIR` override
5. AC-20: Provides clear error for unsupported platforms (Windows, musl)
6. AC-21: Script is POSIX-compatible (no bashisms, works with sh/dash)
7. AC-22: Usage: `curl -sSL <raw-url>/install.sh | sh`

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (shell script executed non-interactively)
- Background Process (piped from curl)

**Observable Behavior:**
- Structured data output (error messages to stderr)
- File system side effects (binary installed to ~/.local/bin or TQ_INSTALL_DIR)
- Network interactions (GitHub API calls, binary download) - NOT testable without live GitHub Release

**External Dependencies:**
- Network access (GitHub API, binary download) - requires live release
- Operating system specific features (uname -s, uname -m, sha256sum/shasum)
- File system access (write to install directory)

**Validation Challenges:**
- Network-dependent behavior (GitHub API, download) requires a published release
- Platform detection tests require running on actual Linux/macOS/Windows
- POSIX compliance testing requires sh/dash (not just bash)
- Checksum verification requires a real binary with a real checksum

**Critical Behaviors to Validate:**
1. POSIX compatibility - no bashisms (AC-21) - testable with `shellcheck --shell=sh`
2. Platform detection logic is correct (AC-16) - reviewable in script
3. Installation directory logic: default `~/.local/bin` and `TQ_INSTALL_DIR` override (AC-19) - reviewable
4. Unsupported platform error message (AC-20) - could be tested with mocked `uname` output
5. Network-dependent behaviors (AC-17, AC-18) - NOT testable without live release

#### 2. Test Strategy Derivation

**Derived Test Types:**

**Test Type 1: Shellcheck Static Analysis**
- **Validates:** AC-21 - POSIX compatibility, no bashisms, common shell pitfalls
- **Approach:** Run `shellcheck --shell=sh install.sh`
- **Rationale:** The definitive POSIX compliance checker; catches syntax errors, quoting issues, bashisms
- **Gap if missing:** Could ship a script with bash-specific syntax that fails with sh/dash
- **Necessity:** REQUIRED
- **Blocker:** shellcheck must be installed; if not available, this test is BLOCKED

**Test Type 2: Script Structure Code Review**
- **Validates:** AC-16, AC-19, AC-20 - platform detection logic, directory handling, error messages
- **Approach:** Manual review of platform detection (uname -s/uname -m branches), TQ_INSTALL_DIR handling, unsupported platform messages
- **Rationale:** Script logic is inspectable and does not require network for structural validation
- **Gap if missing:** Logic errors in platform detection that don't show up until runtime
- **Necessity:** REQUIRED

**Test Type 3: Network-Independent Execution Test**
- **Validates:** AC-20 partially, AC-19 partially - script runs without crashing, error paths work
- **Approach:** Run `sh install.sh` in a controlled environment with `TQ_INSTALL_DIR=/tmp/test-install`, expect network failure (no real release yet) but verify graceful error handling
- **Rationale:** Verifies script syntax is executable and error paths are sane
- **Gap if missing:** Script might have runtime syntax errors that shellcheck misses
- **Necessity:** RECOMMENDED

**Test Type 4: End-to-End Install Test (with live release)**
- **Validates:** AC-16 through AC-22 - full install flow
- **Approach:** After publishing a release, run `curl -sSL <url>/install.sh | sh` on each platform
- **Rationale:** Only true validation of the install experience
- **Gap if missing:** Cannot verify download, checksum, and actual binary installation work
- **Necessity:** REQUIRED for full validation but NOT EXECUTABLE without live release

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| shellcheck static analysis | REQUIRED | POSIX compliance, bashism detection | Script fails on sh/dash | MUST IMPLEMENT (BLOCKED if no shellcheck) |
| Script structure review | REQUIRED | Platform detection, error messages | Logic errors in platform selection | MUST IMPLEMENT |
| Network-independent run | RECOMMENDED | Runtime syntax verification | Silent runtime failures | SHOULD IMPLEMENT |
| End-to-end install test | REQUIRED (full) | Full AC coverage | Cannot verify downloads/install | NOT LOCALLY EXECUTABLE |

#### 4. Specification Coverage Map

| Requirement | Test Type | Status |
|-------------|-----------|--------|
| AC-16: platform detection | shellcheck + review | LOCALLY TESTABLE |
| AC-17: download from GitHub API | End-to-end | NOT LOCALLY TESTABLE |
| AC-18: SHA256 checksum verify | End-to-end | NOT LOCALLY TESTABLE |
| AC-19: install dir + TQ_INSTALL_DIR | Code review | LOCALLY TESTABLE |
| AC-20: unsupported platform error | Code review + partial run | LOCALLY TESTABLE |
| AC-21: POSIX compatible | shellcheck --shell=sh | LOCALLY TESTABLE |
| AC-22: usage pattern | Code review | LOCALLY TESTABLE |

#### 5. Gap Analysis

**End-to-End Install Test (Network + Live Release)**
- **Reason for omission:** Requires a published GitHub Release with binaries and checksums
- **What won't be validated:** AC-17 (download), AC-18 (checksum verification), actual binary placement
- **Risk assessment:** MEDIUM - static analysis can miss runtime issues like wrong URL construction, failed checksum comparison
- **Mitigation:** shellcheck catches most shell script bugs; code review catches logical errors; network error path tested with dry run
- **Revisit criteria:** After first release tag is pushed and release is published

**Shellcheck Availability**
- shellcheck is NOT installed in the current development environment
- **Impact:** If shellcheck is not available at test execution time, the POSIX compliance test is BLOCKED
- **Mitigation request:** Request shellcheck installation (`brew install shellcheck` on macOS or `apt-get install shellcheck` on Ubuntu)

---

### Feature 4: Sprint 40 Remediation

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-41-planning.md` lines 100-113 (AC-23 through AC-26)
- Reference: `docs/sprints/sprint-40-review.md` recommendations #1-#3, #6

**Requirements from Acceptance Criteria:**

1. AC-23: Resolve `/p` alias - add to spec and help text
2. AC-24: Eliminate `execute`/`execute_with_params` duplication in query.rs (accept optional `&ParamStore`)
3. AC-25: Use `LazyLock` for regex compilation in params.rs
4. AC-26: All existing 855 tests pass after changes

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic (internal refactoring - code quality improvements with no new user-facing behavior)
- CLI Batch + Interactive PTY (the `/p` alias documentation update affects user-facing behavior in REPL)

**Observable Behavior:**
- No new observable behavior - these are refactoring and documentation changes
- The `/p` alias already works; AC-23 adds it to spec and help text only

**External Dependencies:**
- None for the refactoring (unit tests + existing test suite)

**Validation Challenges:**
- The key challenge is proving NO regressions - the entire 855-test suite must pass
- `LazyLock` usage is a compile-time guarantee (compiles with stable Rust 1.80+)
- Code duplication elimination must not change behavior

**Critical Behaviors to Validate:**
1. All 855 existing tests pass after changes (AC-26) - the definitive regression check
2. `/p` alias documented in help output (`tq repl --help` or `/help` in REPL)
3. Code compiles cleanly without warnings (LazyLock, merged execute function)

#### 2. Test Strategy Derivation

**Derived Test Types:**

**Test Type 1: Full Regression Suite (cargo test)**
- **Validates:** AC-26 - all 855 tests pass after changes
- **Approach:** Run `cargo test` (excluding `#[ignore]` live-DB tests), verify count matches or exceeds 855
- **Rationale:** The single most important proof that refactoring introduced no regressions
- **Gap if missing:** Cannot claim "zero regressions" without running all tests
- **Necessity:** REQUIRED

**Test Type 2: Help Text Verification**
- **Validates:** AC-23 - `/p` alias appears in help text
- **Approach:** Run `tq repl --help` or check help file content for `/p` alias mention
- **Rationale:** AC-23 is a documentation/UX change verifiable without a database
- **Gap if missing:** `/p` alias might work but remain undocumented, violating AC-23
- **Necessity:** REQUIRED

**Test Type 3: Code Quality Verification (clippy + compilation)**
- **Validates:** AC-24, AC-25 - no duplication, LazyLock usage
- **Approach:** Run `cargo clippy -- -D warnings` to catch any issues; verify compilation succeeds
- **Rationale:** Refactoring must compile cleanly; clippy validates code quality
- **Gap if missing:** Could ship code with warnings or subtle quality issues
- **Necessity:** RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Full regression suite (cargo test) | REQUIRED | Proves no regressions from refactoring | Regressions go undetected | MUST IMPLEMENT |
| Help text verification | REQUIRED | Validates AC-23 (documentation change) | /p alias undocumented | MUST IMPLEMENT |
| Code quality (clippy) | RECOMMENDED | Validates refactoring quality | Latent code warnings | SHOULD IMPLEMENT |
| Interactive REPL test | NOT NEEDED | No new behavior - alias already works | N/A | SKIP |
| Database integration tests | NOT NEEDED | Refactoring only, no query logic change | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement | Test Type | Status |
|-------------|-----------|--------|
| AC-23: /p alias in spec + help | Help text check | LOCALLY TESTABLE |
| AC-24: execute deduplication | Code inspection + cargo test | LOCALLY TESTABLE |
| AC-25: LazyLock for regex | Code inspection + compilation | LOCALLY TESTABLE |
| AC-26: 855 tests pass | cargo test full suite | LOCALLY TESTABLE |

---

## Consolidated Test Implementation Plan

### Test Suite 1: Regression & Build Verification (All Features)

**Location:** Executed via command line (not a Rust test file)
**Framework:** cargo, shell
**Tests:**

| Test ID | Command | Validates | Expected Result |
|---------|---------|-----------|-----------------|
| TS41-001 | `cargo build` | AC-14 (build.rs works for native) | Exit code 0 |
| TS41-002 | `cargo test` (no `--ignored`) | AC-15, AC-26 (855+ tests pass) | 100% pass rate, 855+ tests |
| TS41-003 | `cargo clippy -- -D warnings` | AC-24, AC-25 (code quality) | Zero warnings |

### Test Suite 2: Build.rs Code Inspection

**Location:** Manual grep/read of build.rs
**Tests:**

| Test ID | Check | Validates | Expected Result |
|---------|-------|-----------|-----------------|
| TS41-004 | grep `CARGO_CFG_TARGET_OS` in build.rs | AC-12 | Present (replaces cfg!(target_os)) |
| TS41-005 | grep `CARGO_CFG_TARGET_ARCH` in build.rs | AC-13 | Present (for aarch64 lib selection) |
| TS41-006 | grep `cfg!(target_os)` in build.rs | AC-12 | Absent (removed) |

### Test Suite 3: Release Workflow YAML Review

**Location:** Manual review of `.github/workflows/release.yml`
**Tests:**

| Test ID | Check | Validates | Expected Result |
|---------|-------|-----------|-----------------|
| TS41-007 | Trigger: `push: tags: 'v*'` present | AC-1 | Present |
| TS41-008 | 5 targets in build matrix | AC-2 to AC-6 | All 5 targets listed |
| TS41-009 | Artifact naming `tq-<version>-<target>` | AC-7 | Naming pattern present |
| TS41-010 | Packaging step includes native lib + LICENSE | AC-8 | Steps present in workflow |
| TS41-011 | `sha256sum` checksums.txt step present | AC-9 | Checksum step present |
| TS41-012 | GitHub Release creation step present | AC-10 | Release creation step present |
| TS41-013 | actionlint (if available) | Syntax validation | Zero errors |

### Test Suite 4: Install Script Validation

**Location:** Command line execution against `install.sh`
**Tests:**

| Test ID | Command | Validates | Expected Result |
|---------|---------|-----------|-----------------|
| TS41-014 | `shellcheck --shell=sh install.sh` | AC-21 (POSIX) | Zero warnings/errors |
| TS41-015 | grep for `uname -s` and `uname -m` | AC-16 (platform detect) | Both present |
| TS41-016 | grep for `TQ_INSTALL_DIR` | AC-19 (install dir override) | Present with default fallback |
| TS41-017 | grep for unsupported platform error | AC-20 (error message) | Error message present |
| TS41-018 | `sh install.sh` dry run (no live release) | Script executable, fails gracefully | Non-zero exit with informative error |

### Test Suite 5: Help Text Verification

**Location:** Binary execution
**Tests:**

| Test ID | Command | Validates | Expected Result |
|---------|---------|-----------|-----------------|
| TS41-019 | `tq repl --help` or help text file review | AC-23 (/p alias) | `/p` alias visible in help |

---

## Tool Requirements

### Required Tools (must be available at test execution)

| Tool | Purpose | Availability | Blocker? |
|------|---------|--------------|----------|
| `cargo` | Build and test Rust code | Available | CRITICAL |
| `cargo clippy` | Code quality validation | Available | REQUIRED |
| `sh` | Run install.sh | Available (macOS/Linux) | REQUIRED |
| `grep` | Code inspection checks | Available | REQUIRED |

### Desired Tools (test will be BLOCKED if unavailable)

| Tool | Purpose | Availability | Install Command |
|------|---------|--------------|-----------------|
| `shellcheck` | POSIX compliance validation of install.sh | NOT INSTALLED | `brew install shellcheck` (macOS) |
| `actionlint` | GitHub Actions YAML validation | NOT INSTALLED | `brew install actionlint` (macOS) |

**IMPORTANT:** The quality-validator has confirmed that `shellcheck` and `actionlint` are NOT available in the current environment. Test TS41-014 (shellcheck) and TS41-013 (actionlint) will be BLOCKED unless these tools are installed.

**Tool Request:** The coordinator should arrange for `shellcheck` installation before test execution. `actionlint` is recommended but not strictly required - manual YAML review can substitute.

---

## Strategy Summary

**Total Features Analyzed:** 4

**Test Types Required:**
- Build verification (`cargo build`): Feature 2 (AC-14)
- Regression test suite (`cargo test`): Features 2, 4 (AC-15, AC-26)
- Code inspection (grep/review): Features 1, 2, 3 (AC-1 to AC-13, AC-16, AC-19 to AC-22)
- Static analysis (`shellcheck`): Feature 3 (AC-21) - BLOCKED without shellcheck
- Help text verification: Feature 4 (AC-23)

**Estimated Test Count:**
- Automated (cargo): 2 commands (build + test = 855+ individual tests)
- Code inspection: 13 checks (TS41-004 to TS41-019)
- Static analysis: 1 command (shellcheck)
- Total test items: 19 check items + 855+ existing Rust tests

**Risk Assessment:**
- HIGH risk gaps: GitHub Actions runtime execution (AC-10, AC-11), cross-compilation (AC-12/13 runtime), end-to-end install (AC-17, AC-18)
- MEDIUM risk gaps: YAML correctness without actionlint; script runtime behavior without shellcheck
- LOW risk gaps: cargo build and test regressions (covered locally)

**Dependencies Required:**
- Live database: NO (all locally-executable tests are no-DB)
- Network access: NO (locally-executable tests only)
- Specific OS: macOS (current environment)
- Tools needed: `shellcheck` (not installed), `actionlint` (not installed, optional)
- GitHub infrastructure: Required for HIGH risk items but out of scope for local testing

**Locally Executable ACs:**
- AC-12, AC-13, AC-14, AC-15 (build.rs) - FULLY TESTABLE
- AC-1, AC-7 to AC-9 (workflow structure) - STRUCTURALLY TESTABLE via YAML review
- AC-16, AC-19 to AC-22 (install script) - TESTABLE via shellcheck + review
- AC-23, AC-24, AC-25, AC-26 (remediation) - FULLY TESTABLE

**NOT Locally Executable ACs:**
- AC-2 to AC-6 (actual multi-platform builds) - require GitHub runners
- AC-10, AC-11 (GitHub Release creation) - require GitHub API and real tag
- AC-17, AC-18 (download + checksum verification) - require live GitHub Release

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] Testable vs non-testable items explicitly distinguished
- [x] Tool availability gaps identified with install instructions

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-20
**Review Status:** DRAFT

**Known Blockers:**
1. `shellcheck` not installed - install with `brew install shellcheck` before TS41-014 can execute
2. `actionlint` not installed - install with `brew install actionlint` for TS41-013 (optional)
3. GitHub Actions runtime tests (AC-2 to AC-11) - require tag push to GitHub, out of scope for local test suite
4. End-to-end install test (AC-17, AC-18) - require live GitHub Release, out of scope for local test suite

**Verdict on local test coverage:**
- Features 2 and 4 (build.rs fix, remediation): FULLY TESTABLE locally
- Feature 1 (workflow YAML): PARTIALLY TESTABLE locally (structure review only)
- Feature 3 (install script): PARTIALLY TESTABLE locally (shellcheck + structure review), BLOCKED for POSIX test without shellcheck
