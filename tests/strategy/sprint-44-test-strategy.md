# Sprint 44 Test Strategy: Driver Distribution Fix & Profile Polish

**Created:** 2026-03-21
**Author:** quality-validator
**Sprint:** Sprint 44
**Features:**
1. Runtime Driver Library Resolution (Issue #31)
2. Teradata License Acceptance in Installer (Issue #31)
3. Profile Flag Naming Fix (Sprint 43 Deferred)
4. Profile Delete Confirmation (Sprint 43 Deferred)
5. Technical Debt Cleanup (SqlParseError struct variant, display_profiles helper)

---

## Overview

Sprint 44 delivers fixes across two domains:

- **Distribution integrity (Features 1 & 2)**: The binary currently bakes the CI build-time path into its driver lookup, making all released binaries non-functional on user machines. Feature 1 fixes the path resolution logic at the Rust level; Feature 2 adds license gating to the shell installer script. Feature 1 is unit-testable in isolation; Feature 2 requires shell script static analysis because network-based installer testing is out of scope.
- **Profile UX polish (Features 3 & 4)**: Both are CLI batch changes to `tq profile` subcommands. Feature 3 renames flags to align with global args; Feature 4 adds TTY-interactive delete confirmation with a `--force` bypass. Feature 3 is fully unit + CLI-integration testable; Feature 4's TTY path has a partial gap (no automated PTY test for interactive prompts).
- **Technical debt (Feature 5)**: Two pure-logic changes: upgrading `TqError::SqlParseError` to a struct variant preserving line/column, and extracting a `display_profiles()` helper. Both are unit-testable with zero external dependencies.

None of the features in this sprint require a live Teradata database connection.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Runtime Driver Library Resolution (AC-1 through AC-6)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-44-planning.md` - AC-1 through AC-6
- Secondary: `src/db/client.rs` (current implementation), `build.rs` (root cause)

**Requirements (from acceptance criteria):**

1. AC-1: Binary finds teradatasql library in same directory as executable (primary path)
2. AC-2: Fallback chain: exe dir -> `--driver-lib-dir` flag -> `TERADATA_LIB_DIR` env var -> `.` (cwd)
3. AC-3: Error message shows all searched paths when library not found
4. AC-4: `build.rs` no longer sets `TERADATA_LIB_DIR` to absolute build path (or it is only used as last resort)
5. AC-5: Release workflow still packages library alongside binary in tar.gz
6. AC-6: Install script still copies library to install dir alongside binary

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic (internal algorithm, path resolution)

**Explanation:** The driver resolution is an internal startup algorithm in `DatabaseClient::new()`. Users interact with the outcome (driver loads or not) but do not directly invoke the path resolution logic. The observable behavior is either a successful connection or a `DriverLoad` error with searched paths listed.

**Observable Behavior:**
- Structured error messages (AC-3: error shows all searched paths)
- File system access (reads candidate directories to find the library)

**External Dependencies:**
- File system access (reads executable path via `std::env::current_exe()`, probes directories)
- No database connection required
- No network access
- No PTY required

**Validation Challenges:**

1. **Cannot load actual driver in unit tests**: The teradatasql library is not present in the test environment. Tests must validate path resolution logic (which directories are probed, in which order) without actually loading the driver. The implementation must expose a testable path-resolution function separate from the load call.
2. **`current_exe()` returns the test binary path during unit tests**: Tests of the "exe-relative path" logic will use the test binary's location, not a simulated install directory. Tests must account for this or use dependency injection (pass a `PathBuf` to the resolution function instead of calling `current_exe()` inside it).
3. **AC-5 and AC-6**: Release workflow and install script changes cannot be validated by Rust unit tests. AC-5 is validated by code inspection of the GitHub Actions workflow YAML. AC-6 is covered by Feature 2's install script review.

**Critical Behaviors to Validate:**

1. "Fallback chain: exe dir -> `--driver-lib-dir` flag -> `TERADATA_LIB_DIR` env var -> `.` (cwd)" (AC-2): The order of the fallback must be exact. Each position in the chain must be independently testable.
2. "Error message shows all searched paths when library not found" (AC-3): The error must enumerate every path that was tried, giving users actionable debugging information.
3. "`build.rs` no longer sets `TERADATA_LIB_DIR` to absolute build path" (AC-4): The root cause fix must be verifiable by inspecting the compiled binary or confirming `build.rs` no longer emits the problematic `cargo:rustc-env` line.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked: → NOT applicable (this is internal startup logic)
IF "Pure Logic" checked: → Unit tests REQUIRED
  Reason: Path resolution is an algorithm with discrete, predictable outputs per input
IF "File system access" checked: → Unit tests with temp dirs REQUIRED
  Reason: Need to verify path construction is correct without loading the actual library
IF "Operating system specific" NOT checked: → Single platform adequate
  Reason: Path resolution uses std::path which abstracts OS differences
```

**Derived Test Types:**

**Test Type 1: Unit Tests (path resolution logic)**
- **Validates:** AC-1 (exe-relative path is first candidate), AC-2 (fallback chain order), AC-3 (error lists all searched paths)
- **Approach:** Extract a `resolve_driver_paths(exe_path: &Path, flag: Option<&str>, env_var: Option<&str>) -> Vec<PathBuf>` function. Unit tests call this function with controlled inputs and assert the returned path list matches the expected fallback chain.
- **Rationale:** Path resolution is pure logic; unit tests give precise, fast, reproducible coverage without requiring the actual library.
- **Gap if missing:** The fallback chain order could be wrong (e.g., env var consulted before flag), and the error message could list paths in the wrong order or miss paths entirely.
- **Necessity:** REQUIRED

**Test Type 2: Code Inspection (build.rs)**
- **Validates:** AC-4 (`build.rs` no longer sets `TERADATA_LIB_DIR` to absolute path)
- **Approach:** Read `build.rs` in a test and assert it does not emit `cargo:rustc-env=TERADATA_LIB_DIR=/...` with an absolute path. Alternatively, inspect the compiled binary's `option_env!("TERADATA_LIB_DIR")` value in a unit test.
- **Rationale:** The root cause is a build-time side effect, not a runtime behavior. A targeted code inspection test catches this regression immediately.
- **Gap if missing:** A CI regression could re-introduce the absolute-path baking without tests catching it.
- **Necessity:** REQUIRED

**Test Type 3: Integration tests (CLI argument passthrough)**
- **Validates:** That `--driver-lib-dir` flag is correctly wired into the resolution chain (AC-2)
- **Approach:** Test that when `--driver-lib-dir /some/path` is passed, the path resolution function receives that value. This is a wiring test, verifiable at the CLI parsing level without an actual driver.
- **Rationale:** Ensures the clap argument definition and the `DatabaseClient::new()` call are properly connected.
- **Gap if missing:** The flag could be parsed but silently ignored, with the fallback chain never consulting it.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (path resolution) | REQUIRED | AC-1, AC-2, AC-3 - Pure logic with precise, testable outputs | Fallback chain order wrong, error message incomplete | MUST IMPLEMENT |
| Code inspection test (build.rs) | REQUIRED | AC-4 - Build-time root cause verification | Regression reintroduces absolute path baking | MUST IMPLEMENT |
| CLI wiring test | REQUIRED | AC-2 - `--driver-lib-dir` passthrough | Flag parsed but ignored | MUST IMPLEMENT |
| Driver load integration test | NOT NEEDED | Requires actual teradatasql library not in test environment | N/A | SKIP (documented gap) |
| Release workflow test | NOT NEEDED | AC-5 requires CI environment with actual release build | N/A | SKIP (code inspection) |

**Summary:**
- REQUIRED test types: 3 - MUST implement all
- NOT NEEDED test types: 2 - Explicitly omitted with rationale

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-1 | Binary finds teradatasql library in same directory as executable (primary path) | sprint-44-planning.md | Unit test | Exe-relative path must be first in the resolution list | TC-044-001 |
| AC-2 | Fallback chain: exe dir -> `--driver-lib-dir` flag -> `TERADATA_LIB_DIR` env var -> `.` | sprint-44-planning.md | Unit + CLI wiring | Chain order is pure logic; wiring ensures flag reaches the function | TC-044-001, TC-044-002 |
| AC-3 | Error message shows all searched paths when library not found | sprint-44-planning.md | Unit test | Error construction is pure logic; assert searched paths appear in error string | TC-044-003 |
| AC-4 | `build.rs` no longer sets `TERADATA_LIB_DIR` to absolute build path | sprint-44-planning.md | Code inspection | Build-time behavior verified by reading build.rs or checking env! value | TC-044-004 |
| AC-5 | Release workflow still packages library alongside binary | sprint-44-planning.md | Code inspection only | CI-only; verified by reading workflow YAML | TC-044-005 (manual/inspection) |
| AC-6 | Install script copies library to install dir alongside binary | sprint-44-planning.md | Part of Feature 2 tests | Covered by TC-044-007 (install.sh inspection) | TC-044-007 |

**Coverage Gaps:**
- Driver actually loading against real teradatasql is not tested. Accepted risk: this cannot be automated in CI without the proprietary library. Validated manually when a real library is available.

#### 5. Gap Analysis

**Driver Load Integration Tests**
- **Reason for omission:** The teradatasql library is a proprietary Teradata artifact not present in CI. Loading it requires the actual binary, which cannot be distributed in the test repository.
- **What won't be validated:** That the library actually loads from the resolved path at runtime on user machines.
- **Risk assessment:** MEDIUM - Path resolution logic is verified by unit tests, but the final integration step (actual `dlopen`/library load) is not.
- **Mitigation:** Path resolution tests confirm the correct path will be computed. Manual smoke testing with the actual driver covers the load step.
- **Revisit criteria:** If the team gains access to a CI environment with the teradatasql library, add an integration test that actually loads the driver.

#### 6. Test Implementation Plan

**Test Type: Unit Tests (path resolution)**
- **Location:** `src/db/client.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. No flag, no env var: returns `[exe_dir, .]` (2-element chain)
  2. `--driver-lib-dir` flag set: returns `[exe_dir, flag_dir, .]`
  3. `TERADATA_LIB_DIR` env var set: returns `[exe_dir, env_dir, .]`
  4. Both flag and env var set: flag takes precedence after exe_dir, env var after flag
  5. Full chain with all four sources: `[exe_dir, flag_dir, env_dir, .]`
  6. Error message from `DriverLoad` contains all searched paths
- **Mocking strategy:** Inject `exe_path: PathBuf` parameter to avoid calling `current_exe()` inside the resolution function. Use `temp_dir` for file system verification where needed.

**Test Type: Code Inspection (build.rs)**
- **Location:** `src/db/client.rs` or a dedicated `tests/build_sanity.rs`
- **Framework:** `#[test]` using `std::fs::read_to_string`
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Assert `build.rs` does not contain `cargo:rustc-env=TERADATA_LIB_DIR` with an absolute path (i.e., no path starting with `/Users/` or `/home/` baked in)
- **Implementation notes:** Read the file content and assert the pattern is absent. This is a canary test to prevent regression of the root cause.

**Test Type: CLI wiring test**
- **Location:** `src/db/client.rs` or `src/commands/` tests
- **Framework:** `#[test]`
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. When `driver_lib_dir: Some("/custom/path")` is passed to `DatabaseClient::new()`, the resolution function receives `/custom/path` as the flag value (verifiable by inspecting the error when the library is not found, which lists searched paths).
- **Implementation notes:** Construct a `DatabaseClient::new()` with a non-existent driver dir, assert the `DriverLoad` error message contains the specified path.

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: fallback chain order, path construction, error message content
- Code inspection validates: build.rs root cause not reintroduced
- CLI wiring validates: `--driver-lib-dir` flag reaches the resolution function
- Combined coverage: adequate for the logic layer; gap exists at the actual library load layer

**Gaps in combined coverage:**
- Actual driver loading is not tested (requires proprietary library)
- Cross-platform path separator behavior (Linux vs macOS) not explicitly tested, though `std::path::PathBuf` abstracts this

**Acceptance criteria:**
- All specification requirements AC-1 through AC-4 have test coverage
- All test types justified by requirements
- Combined coverage is sufficient to claim "path resolution works as specified"
- Known gaps (driver load, AC-5) are documented and accepted

---

### Feature 2: Teradata License Acceptance in Installer (AC-7 through AC-12)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-44-planning.md` - AC-7 through AC-12
- Secondary: `install.sh` (existing installer shell script)

**Requirements (from acceptance criteria):**

1. AC-7: Install script displays Teradata license summary before download
2. AC-8: Interactive mode: prompts `[y/N]` for acceptance, aborts on decline
3. AC-9: Non-interactive mode: `--accept-license` flag bypasses prompt
4. AC-10: Piped install (`curl | sh`) detects non-TTY and requires `--accept-license`
5. AC-11: License text stored in repository (not fetched remotely)
6. AC-12: README updated with license acceptance instructions

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (shell script, invoked as `sh install.sh [--accept-license]`)
- Interactive TTY (the `[y/N]` prompt requires a terminal when no flag given)

**Explanation:** The installer is a POSIX shell script. It has two modes: non-interactive (flag-based) and interactive (TTY prompt). The interactive path is triggered by a terminal-connected stdin; the non-interactive path by the `--accept-license` flag.

**Observable Behavior:**
- Terminal text output (license summary, prompt text)
- Exit codes (0 on acceptance, non-zero on decline or non-TTY without flag)
- Network interactions (downloads binary - but this is pre-existing, not new behavior)

**External Dependencies:**
- Terminal/PTY (interactive mode requires TTY detection via `[ -t 0 ]` or similar)
- File system access (license text stored in repository, AC-11)
- Network access (existing download logic - not tested here)

**Validation Challenges:**

1. **Shell script testing is not Rust**: Rust's test framework cannot execute `install.sh`. Shell testing requires tools like `shellcheck` (static analysis) or `bats`/`bash` with mocked functions.
2. **Interactive TTY simulation**: Testing the `[y/N]` prompt path requires feeding stdin to the script. This is possible with `echo y | sh install.sh` but requires a test harness.
3. **Network download suppression**: Full integration testing of the installer would attempt to download binaries. Tests must mock or stub the download step to avoid network dependency.
4. **shellcheck availability**: Static analysis with `shellcheck` depends on the tool being installed in CI.

**Critical Behaviors to Validate:**

1. "`--accept-license` flag bypasses prompt" (AC-9): The flag must be parsed correctly and the prompt must be skipped entirely.
2. "Non-TTY mode requires `--accept-license`" (AC-10): When stdin is not a terminal and no flag is given, the script must abort with a meaningful error.
3. "License text stored in repository, not fetched remotely" (AC-11): The license text must exist as a file in the repo and must not be fetched via `curl` or `wget`.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked: → Integration tests REQUIRED
  Reason: Script argument parsing must be validated end-to-end
IF "Interactive PTY" checked: → Partial (--accept-license bypasses TTY; interactive path partially tested)
IF "File system access" checked: → Integration test includes license file existence check
```

**Derived Test Types:**

**Test Type 1: Static Analysis (shellcheck)**
- **Validates:** Script correctness, POSIX compliance, common shell pitfalls
- **Approach:** Run `shellcheck install.sh` and assert it exits with code 0.
- **Rationale:** Shell scripts are error-prone; shellcheck catches quoting issues, missing `${}`, bad conditionals. It validates the structural correctness of the new `--accept-license` argument parsing without executing the script.
- **Gap if missing:** Shell syntax errors, quoting bugs, and POSIX incompatibilities in the new flag parsing would not be caught before deployment.
- **Necessity:** REQUIRED

**Test Type 2: Script execution tests (bash with mocking)**
- **Validates:** AC-7 (license display), AC-9 (flag bypasses prompt), AC-10 (non-TTY abort), AC-8 (decline aborts)
- **Approach:** Source or execute `install.sh` with functions mocked to prevent actual downloads. Use heredoc stdin to simulate interactive input. Assert exit codes and output text.
- **Rationale:** Behavior verification requires actually running the script with controlled inputs.
- **Gap if missing:** The flag parsing could be syntactically valid (passing shellcheck) but logically wrong (e.g., `--accept-license` is parsed but the branch is never reached).
- **Necessity:** REQUIRED

**Test Type 3: File existence check (license text in repo)**
- **Validates:** AC-11 (license text stored in repository, not fetched remotely)
- **Approach:** In both a Rust test and a shell test, assert that a file like `LICENSE-TERADATA-DRIVER` or `docs/teradata-driver-license.txt` exists at a known repo-relative path.
- **Rationale:** AC-11 explicitly requires the license to not be network-fetched. The simplest proof is that the file exists locally.
- **Gap if missing:** A developer could satisfy the "display" requirement by curl-fetching the license at install time, violating AC-11.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| shellcheck static analysis | REQUIRED | Catches shell syntax bugs in new flag parsing without execution | Shell syntax errors, POSIX incompatibilities | MUST IMPLEMENT |
| Script execution tests | REQUIRED | AC-7, AC-8, AC-9, AC-10 - Behavioral verification | Flag logic correct in structure but wrong in behavior | MUST IMPLEMENT |
| License file existence check | REQUIRED | AC-11 - License must be stored in repo | Developer could violate AC-11 without detection | MUST IMPLEMENT |
| Full installer integration test | NOT NEEDED | Would download actual binary; network dependency unacceptable in CI | N/A | SKIP |
| Interactive PTY test (expectrl) | NOT NEEDED | `--accept-license` flag specifically exists to avoid TTY dependency; non-interactive path is primary | N/A | SKIP with note |

**Summary:**
- REQUIRED test types: 3 - MUST implement all
- NOT NEEDED test types: 2 - Explicitly omitted with rationale

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-7 | Install script displays Teradata license summary before download | sprint-44-planning.md | Script execution | Assert license text appears in stdout before download attempt | TC-044-006 |
| AC-8 | Interactive mode: prompts `[y/N]`, aborts on decline | sprint-44-planning.md | Script execution | Feed `n\n` to stdin, assert non-zero exit | TC-044-006 |
| AC-9 | `--accept-license` flag bypasses prompt | sprint-44-planning.md | Script execution | Run with `--accept-license`, assert no prompt, exit 0 before download | TC-044-006 |
| AC-10 | Non-TTY detects no terminal, requires `--accept-license` | sprint-44-planning.md | Script execution | Run with stdin redirected from /dev/null, no flag, assert non-zero exit | TC-044-006 |
| AC-11 | License text stored in repository (not fetched remotely) | sprint-44-planning.md | File existence check | Assert file exists at known path; inspect script does not curl license | TC-044-007 |
| AC-12 | README updated with license acceptance instructions | sprint-44-planning.md | Code inspection | Assert README contains `--accept-license` flag documentation | TC-044-008 (manual/inspection) |

**Coverage Gaps:**
- AC-12 (README update) is validated by code inspection, not an automated test. Acceptable because documentation correctness is a human judgment, not a binary assertion.

#### 5. Gap Analysis

**Full Installer Integration Test**
- **Reason for omission:** The installer downloads binaries from GitHub Releases. Running the full installer in CI would require a released binary to exist and would make network calls, neither of which is appropriate for unit/integration test suites.
- **What won't be validated:** That the actual binary download and installation succeeds after license acceptance.
- **Risk assessment:** LOW - The download logic is pre-existing and unchanged; only the license gate is new.
- **Mitigation:** Shell function mocking tests the new logic in isolation. The download path is covered by existing manual QA.
- **Revisit criteria:** If the installer is significantly refactored, add integration testing with mock HTTP server.

**Interactive PTY test for `[y/N]` prompt**
- **Reason for omission:** The `--accept-license` flag was specifically designed to make the installer scriptable, avoiding the need for a PTY in CI. The TTY interactive path is a UX fallback for manual use only.
- **What won't be validated:** The visual appearance of the `[y/N]` prompt in an interactive terminal.
- **Risk assessment:** LOW - The prompt is simple text output; no visual rendering complexity.
- **Mitigation:** Manual smoke testing of the interactive path before release.
- **Revisit criteria:** If interactive installer UX becomes a quality concern.

#### 6. Test Implementation Plan

**Test Type: shellcheck**
- **Location:** `tests/shell/` directory or Makefile/CI step
- **Framework:** `shellcheck` CLI tool + Bash test wrapper
- **Test count estimate:** 1 test (one invocation of shellcheck)
- **Key scenarios to cover:**
  1. `shellcheck -S error install.sh` exits 0 (no errors at error severity)
- **Implementation notes:** Can be implemented as a Rust test using `std::process::Command` to invoke `shellcheck`, or as a CI step. Rust test approach preferred for single-pass test execution.

**Test Type: Script execution tests**
- **Location:** `tests/shell/test_install_license.sh`
- **Framework:** Bash test script with function mocking
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. `--accept-license` flag: Script proceeds past license gate without prompt (exit 0 from license section)
  2. Non-TTY stdin, no flag: Script exits non-zero with error message about `--accept-license`
  3. Interactive `y` response: Script proceeds past license gate (exit 0)
  4. Interactive `n` response: Script exits non-zero with "Installation aborted" message
- **Implementation notes:** Mock the download function by overriding it with a shell function that exits cleanly. Use `</dev/null` to simulate non-TTY. Use `echo y |` or `echo n |` to simulate interactive input.

**Test Type: License file existence check**
- **Location:** `tests/shell/test_install_license.sh` or a Rust test in `tests/`
- **Framework:** Shell `[ -f <path> ]` assertion or Rust `std::path::Path::exists()`
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. License file exists at expected repo path
  2. `install.sh` does not contain `curl.*license` or `wget.*license` (license not network-fetched)
- **Implementation notes:** The path to the license file must be agreed upon during implementation (e.g., `docs/teradata-driver-license.txt`). Grep install.sh for any `curl` or `wget` call that references a license URL.

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- shellcheck validates: script structural correctness
- Script execution validates: all four behavioral paths of the license gate
- File existence validates: license stored locally, not fetched remotely
- Combined coverage: adequate for all testable ACs; AC-12 and full install path are acceptable gaps

**Acceptance criteria:**
- AC-7 through AC-11 have automated test coverage
- AC-12 has manual inspection coverage
- Known gaps are documented and accepted

---

### Feature 3: Profile Flag Naming Fix (AC-13 through AC-16)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-44-planning.md` - AC-13 through AC-16
- Secondary: `src/cli.rs` (current `ProfileAction` definition with `--auth`/`--pass-file`)

**Current State (from code inspection of `src/cli.rs`):**
- `ProfileAction::Add` uses `long = "auth"` and `long = "pass-file"` with unique IDs to avoid clap conflict
- `ProfileAction::Edit` uses the same workaround
- Global args use `--logmech` and `--password-file`

**Requirements (from acceptance criteria):**

1. AC-13: Profile subcommands use `--logmech` (not `--auth`) for authentication mechanism
2. AC-14: Profile subcommands use `--password-file` (not `--pass-file`) for password file path
3. AC-15: No clap argument conflicts between global and profile-specific args
4. AC-16: User guide updated to use correct flag names

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (command-line flag parsing)

**Explanation:** This is a CLI argument naming change. Users invoke `tq profile add <name> --host <h> --logmech LDAP`. The validation is: does the renamed flag parse correctly and route to the correct field?

**Observable Behavior:**
- CLI argument parsing behavior
- File system side effects (profile written to config with correct logmech/password_file values)
- Error messages when old flags are used (should be "unexpected argument" from clap)

**External Dependencies:**
- File system access (for profile write verification)
- No database required, no network, no PTY

**Validation Challenges:**

1. **Clap global arg conflict**: The root problem is that `--logmech` and `--password-file` are defined as `global = true` in `GlobalOpts`. Defining them again in `ProfileAction` with the same long name creates a clap conflict. The fix must resolve this without breaking either the global args or the profile-specific args. The test must verify both sides work after the fix.
2. **Old flags rejected**: After renaming, `--auth` and `--pass-file` must not be accepted. This is verifiable by running the binary with old flags and asserting a non-zero exit code.

**Critical Behaviors to Validate:**

1. "`tq profile add <name> --host <h> --logmech LDAP` works" (AC-13): The renamed flag must be parseable and the value must reach the handler.
2. "`tq profile add <name> --host <h> --password-file /path` works" (AC-14): Same as above for password file.
3. "No clap conflicts when both global and profile args are present" (AC-15): Running a full command like `tq --logmech TD2 profile add test --host h --logmech LDAP` should not panic or error with a clap conflict.
4. "Old flags `--auth` and `--pass-file` are rejected" (implicit in AC-13/14): Backward incompatibility is intentional; old flags must produce a clear error.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked: → Integration tests REQUIRED
  Reason: Flag parsing must be tested at the CLI level, not just unit level
IF "File system side effects" checked: → Integration tests with temp dir REQUIRED
  Reason: Verify correct values written to config
```

**Derived Test Types:**

**Test Type 1: Unit Tests (profile handler with renamed fields)**
- **Validates:** That `handle_add` and `handle_edit` receive correct values when called with `logmech` and `password_file` parameters (these are already the internal field names; this is a regression test)
- **Approach:** Existing unit tests in `src/commands/profile.rs` already test `handle_add`/`handle_edit` with logmech and password_file. These are NOT changing. Add targeted tests asserting the logmech value is correctly stored.
- **Rationale:** Confirms the internal logic is untouched by the flag renaming.
- **Gap if missing:** A refactor could accidentally rename the struct field rather than just the clap long name.
- **Necessity:** REQUIRED (existing tests cover most; add targeted regression tests)

**Test Type 2: CLI Integration Tests (binary invocation)**
- **Validates:** AC-13 (renamed flag works), AC-14 (renamed flag works), AC-15 (no clap conflict), old flags rejected
- **Approach:** Build the binary and invoke it with the renamed flags. Use `TQ_CONFIG_DIR` env var to redirect config to a temp dir. Assert exit 0 and correct config file content.
- **Rationale:** Flag parsing is a CLI-level concern; only a real binary invocation proves the clap definitions are correct.
- **Gap if missing:** Unit tests test the handler logic; only CLI tests verify the argument name change at the clap level.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (handler logic regression) | REQUIRED | Confirms internal field names unchanged by refactor | Silent data loss if field renamed instead of long name | MUST IMPLEMENT |
| CLI integration tests (binary invocation) | REQUIRED | AC-13, AC-14, AC-15 - Flag naming is CLI-level concern | Handler correct but flags never reach it | MUST IMPLEMENT |
| Interactive tests | NOT NEEDED | Profile commands are non-interactive | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements for flag parsing | N/A | SKIP |

**Summary:**
- REQUIRED test types: 2 - MUST implement all
- NOT NEEDED test types: 2 - Explicitly omitted

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-13 | Profile subcommands use `--logmech` (not `--auth`) | sprint-44-planning.md | CLI integration | Flag name is clap definition; only CLI invocation proves it | TC-044-009 |
| AC-14 | Profile subcommands use `--password-file` (not `--pass-file`) | sprint-44-planning.md | CLI integration | Same as AC-13 | TC-044-009 |
| AC-15 | No clap argument conflicts between global and profile args | sprint-44-planning.md | CLI integration | Conflict manifests as clap panic or error at runtime | TC-044-009 |
| AC-16 | User guide updated to use correct flag names | sprint-44-planning.md | Code inspection | Documentation correctness is human judgment | TC-044-010 (manual) |

**Coverage Gaps:**
- AC-16 is a documentation check, not an automated test. Acceptable because the spec does not require the guide to be machine-readable.

#### 5. Gap Analysis

**No significant gaps.** The combination of unit tests and CLI integration tests covers all behavioral aspects of this feature.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/profile.rs` test module (existing)
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 2 new tests (regression)
- **Key scenarios to cover:**
  1. `handle_add` with logmech="LDAP" stores "LDAP" in config (regression guard)
  2. `handle_add` with password_file set stores path in config (regression guard)

**Test Type: CLI Integration Tests**
- **Location:** `tests/cli_profile_flags.rs` (new integration test file)
- **Framework:** `std::process::Command` to invoke the `tq` binary
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `tq profile add dev --host h --logmech LDAP` exits 0, config contains `logmech = "LDAP"`
  2. `tq profile add dev --host h --password-file /tmp/pw` exits 0, config contains `password_file`
  3. `tq profile add dev --host h --auth LDAP` exits non-zero (old flag rejected)
  4. `tq profile add dev --host h --pass-file /tmp/pw` exits non-zero (old flag rejected)
  5. `tq --logmech TD2 profile add dev --host h --logmech LDAP` exits 0 (no clap conflict, profile logmech takes effect)
- **Setup requirements:** Binary must be built (`cargo build`). Use `TQ_CONFIG_DIR` env var pointing to a temp dir.

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: internal handler logic (logmech/password_file stored correctly)
- CLI integration tests validate: flag names at the clap level, conflict resolution, old flag rejection
- Combined coverage: comprehensive for AC-13 through AC-15

---

### Feature 4: Profile Delete Confirmation (AC-17 through AC-19)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-44-planning.md` - AC-17 through AC-19
- Secondary: `src/commands/profile.rs` (`handle_delete` function - current implementation always requires `--force`)

**Current State (from code inspection):**
- `handle_delete` currently checks `if !force { return Err(...) }` - always requires `--force`
- Sprint 44 changes this: TTY mode shows interactive prompt; non-TTY still requires `--force`

**Requirements (from acceptance criteria):**

1. AC-17: TTY mode: shows `Delete profile 'name'? [y/N]` prompt, proceeds on `y`/`Y`, aborts on any other input
2. AC-18: Non-TTY mode: requires `--force` flag (current behavior preserved for scripting)
3. AC-19: `--force` flag bypasses confirmation in all modes (TTY and non-TTY)

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (non-TTY path, `--force` flag)
- Interactive PTY (TTY path, `[y/N]` prompt)

**Explanation:** The delete command now branches on TTY detection. The `--force` path is CLI batch (non-interactive). The prompt path requires a terminal. Both paths must be tested; the TTY path has limited automated test coverage.

**Observable Behavior:**
- Terminal text output: `Delete profile 'name'? [y/N]` prompt
- File system side effects: profile removed from `~/.tq/config.toml`
- Exit codes: 0 on deletion, non-zero on abort

**External Dependencies:**
- Terminal/PTY (for TTY detection and interactive prompt)
- File system access

**Validation Challenges:**

1. **TTY detection in unit tests**: Rust unit tests run in a non-TTY environment. Any test that invokes `handle_delete` in a unit test will hit the non-TTY branch. Testing the TTY branch requires either PTY simulation (expectrl) or a test that mocks the TTY detection function.
2. **Interactive prompt testing**: To test the `[y/N]` prompt behavior, either use expectrl to simulate a PTY with keypresses, or inject a mock stdin reader into `handle_delete`.
3. **`--force` bypass**: This is the simplest path and is directly testable in existing unit tests (already tested in `test_delete_profile_with_force`).

**Critical Behaviors to Validate:**

1. "`--force` bypasses confirmation in all modes" (AC-19): Existing tests cover this; must not be broken by the refactor.
2. "Non-TTY mode requires `--force`" (AC-18): In a unit test environment (non-TTY), `handle_delete("name", false)` must return an error.
3. "TTY mode shows prompt and proceeds on `y`" (AC-17): Requires PTY simulation or dependency injection.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked: → Interactive tests (expectrl) REQUIRED for TTY path
  Reason: Unit tests cannot simulate a TTY; the `[y/N]` prompt behavior requires PTY
IF "CLI Batch" checked: → Unit tests REQUIRED for --force and non-TTY paths
  Reason: --force and non-TTY behavior is non-interactive and unit-testable
IF "File system side effects" checked: → Tests with temp dir REQUIRED
  Reason: Verify profile actually removed from config after confirmed delete
```

**Derived Test Types:**

**Test Type 1: Unit Tests (--force and non-TTY paths)**
- **Validates:** AC-18 (non-TTY requires `--force`), AC-19 (`--force` bypasses confirmation)
- **Approach:** Existing unit tests `test_delete_profile_with_force` and `test_delete_profile_without_force_fails` already cover these cases. Sprint 44 must ensure these tests still pass after the TTY detection logic is added.
- **Rationale:** The non-TTY path is the simpler branch. Unit tests run in non-TTY; this naturally exercises the "requires `--force` in non-TTY" path.
- **Gap if missing:** The `--force` bypass or non-TTY enforcement could be silently broken by the refactor.
- **Necessity:** REQUIRED (existing tests; verify they still pass)

**Test Type 2: Dependency injection test for TTY path**
- **Validates:** AC-17 (prompt shown and accepted in TTY mode)
- **Approach:** If the implementation accepts a `reader: impl Read` parameter (or an `is_tty: bool` override for testing), tests can inject a simulated "y\n" or "n\n" reader to exercise the TTY branch without a real PTY.
- **Rationale:** This is the preferred approach for testing interactive behavior without PTY overhead. The implementation must be designed for testability.
- **Gap if missing:** The TTY branch (the new prompt logic) is not tested at all in automated tests.
- **Necessity:** REQUIRED if implementation supports injection; RECOMMENDED otherwise

**Test Type 3: Interactive PTY test (expectrl) - fallback**
- **Validates:** AC-17 (prompt appears, `y` proceeds, `n` aborts)
- **Approach:** Use expectrl to spawn the `tq profile delete <name>` command in a PTY, wait for the `[y/N]` prompt, send `y` or `n`, assert exit code and file state.
- **Rationale:** If dependency injection is not possible, expectrl is the only way to test the TTY-interactive path automatically.
- **Gap if missing:** The TTY branch is not tested; a bug in the prompt display or input reading would only be caught manually.
- **Necessity:** REQUIRED as fallback if dependency injection not available; RECOMMENDED if injection is available (for belt-and-suspenders)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (--force, non-TTY) | REQUIRED | AC-18, AC-19 - directly testable in unit test env | --force bypass or non-TTY enforcement broken | MUST IMPLEMENT |
| Dependency injection (TTY path) | REQUIRED | AC-17 - TTY branch otherwise untested | Prompt logic never verified | MUST IMPLEMENT if feasible |
| Interactive PTY test (expectrl) | RECOMMENDED | AC-17 fallback or belt-and-suspenders | Same as above if injection not feasible | IMPLEMENT as fallback |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

**Summary:**
- REQUIRED test types: 2 (unit tests + either injection or expectrl for TTY path)
- RECOMMENDED test types: 1 (expectrl as belt-and-suspenders)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-17 | TTY mode: shows `Delete profile 'name'? [y/N]` prompt | sprint-44-planning.md | Injection or expectrl | TTY path requires terminal or injection | TC-044-011 |
| AC-18 | Non-TTY mode: requires `--force` flag | sprint-44-planning.md | Unit test | Unit tests run in non-TTY; behavior is directly observable | TC-044-012 |
| AC-19 | `--force` flag bypasses confirmation in all modes | sprint-44-planning.md | Unit test | --force path is deterministic, fully unit-testable | TC-044-013 |

**Coverage Gaps:**
- AC-17's TTY path depends on implementation design. If `handle_delete` calls `std::io::stdin()` directly without injection points, automated testing requires expectrl.

#### 5. Gap Analysis

**TTY Interactive Path Without Injection**
- **Reason for omission:** If the implementation does not expose a testable seam (injectable reader), automated testing of the `[y/N]` prompt requires expectrl which adds complexity.
- **What won't be validated:** Visual appearance of the prompt, exact prompt text, multi-character inputs being ignored.
- **Risk assessment:** LOW - The prompt is simple text; the primary risk is the logic (y proceeds, n aborts), which is covered by injection or expectrl tests.
- **Mitigation:** If neither injection nor expectrl is feasible, document a manual test case for the TTY prompt.
- **Revisit criteria:** If TTY-related bugs are reported after release.

#### 6. Test Implementation Plan

**Test Type: Unit Tests (existing, verify still pass)**
- **Location:** `src/commands/profile.rs` existing test module
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 2 existing tests + 2 new
- **Key scenarios:**
  1. `handle_delete("name", true)` with existing profile: succeeds (--force)
  2. `handle_delete("name", false)` without TTY: returns error (non-TTY + no force)
  3. `handle_delete("nonexistent", true)`: returns "does not exist" error (regression)
  4. After `handle_delete("name", true)`: config file no longer contains profile

**Test Type: Dependency injection (preferred TTY path test)**
- **Location:** `src/commands/profile.rs` test module
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 2 tests
- **Key scenarios:**
  1. `handle_delete_with_reader("name", false, is_tty=true, stdin="y\n")`: succeeds
  2. `handle_delete_with_reader("name", false, is_tty=true, stdin="n\n")`: returns error "aborted"
- **Implementation notes:** Requires the implementation to expose `handle_delete` with injectable reader and TTY flag. Request architect to implement with this signature.

**Test Type: expectrl (fallback)**
- **Location:** `tests/interactive_tests.rs` (if expectrl is available)
- **Framework:** expectrl crate
- **Test count estimate:** 2 tests
- **Key scenarios:**
  1. Spawn `tq profile delete <name>`, expect `[y/N]`, send `y`, assert exit 0 and profile gone
  2. Spawn `tq profile delete <name>`, expect `[y/N]`, send `n`, assert non-zero exit and profile preserved
- **Implementation notes:** Requires the binary to be built and a temp config with a profile pre-populated.

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: --force path and non-TTY path (the scriptable behaviors)
- Injection or expectrl validates: TTY interactive path
- Combined coverage: comprehensive if injection is available; adequate if only unit tests + manual for TTY path

---

### Feature 5: Technical Debt Cleanup (AC-20 through AC-21)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-44-planning.md` - AC-20 through AC-21

**Requirements (from acceptance criteria):**

1. AC-20: `TqError::SqlParseError` upgraded to struct variant with `line` and `column` fields
2. AC-21: Shared `display_profiles()` helper extracted from `handle_list`/`handle_profiles`

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic (internal refactoring, no user-facing behavior changes)

**Explanation:** AC-20 is an internal error type enhancement. The observable change is that `TqError::SqlParseError` now carries structured location data (line, column) instead of a plain string. AC-21 is a code deduplication: extracting a shared function used by two call sites. Neither change alters user-facing behavior from the user's perspective.

**Observable Behavior:**
- AC-20: Error messages from parse errors now include line/column information (user-visible improvement)
- AC-21: No change to user-observable output; purely internal

**External Dependencies:**
- None (pure logic, no external dependencies)

**Validation Challenges:**

1. **AC-20 is a type change**: All match arms on `TqError::SqlParseError` must be updated. Tests must confirm the struct variant compiles and carries the right data.
2. **AC-21 is a refactor**: The output of `display_profiles()` must be identical to what `handle_list` previously produced. Tests must assert output equivalence.

**Critical Behaviors to Validate:**

1. "SqlParseError struct variant carries line and column" (AC-20): The variant must be constructible with `line` and `column` fields, and the Display implementation must include them.
2. "SqlParseError integrates with error propagation chain" (AC-20): When a `ParseError` from `sql/parser.rs` is converted to `TqError::SqlParseError`, line and column must be preserved, not discarded.
3. "display_profiles() produces same output as previous handle_list" (AC-21): Extracted helper must be functionally equivalent to the inlined code.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked: → Unit tests REQUIRED
  Reason: All behaviors are deterministic pure functions, no external dependencies
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** AC-20 (struct variant carries line/column, Display includes them), AC-21 (display_profiles output matches expected format)
- **Approach:** Directly construct `TqError::SqlParseError { message: "...", line: 3, column: 7 }` and assert Display output. Test that `ParseError { line: 3, column: 7, message: "..." }` correctly converts to `TqError::SqlParseError` with preserved values.
- **Rationale:** Pure logic; unit tests are the correct and sufficient approach.
- **Gap if missing:** The struct variant could compile but store wrong data (e.g., line and column swapped) or omit them from the Display output.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | AC-20, AC-21 - Pure logic, no external dependencies | Data loss in error propagation, output regression | MUST IMPLEMENT |
| Integration tests | NOT NEEDED | Feature is internal refactoring, no CLI change | N/A | SKIP |
| Interactive tests | NOT NEEDED | No user interaction involved | N/A | SKIP |

**Summary:**
- REQUIRED test types: 1
- NOT NEEDED test types: 2

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-20 | `TqError::SqlParseError` upgraded to struct variant with line/column | sprint-44-planning.md | Unit test | Struct variant construction and Display are pure logic | TC-044-014 |
| AC-21 | Shared `display_profiles()` helper extracted | sprint-44-planning.md | Unit test | Output equivalence is verifiable by unit test | TC-044-015 |

#### 5. Gap Analysis

No significant gaps. Both ACs are pure logic changes fully covered by unit tests.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/error.rs` test module (AC-20), `src/commands/profile.rs` test module (AC-21)
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `TqError::SqlParseError { message: "unterminated string", line: 3, column: 7 }` constructs successfully (AC-20)
  2. Display of above error includes "line 3" and "column 7" or "3:7" (AC-20)
  3. Exit code for `SqlParseError` struct variant is 1 (runtime error) (AC-20 regression)
  4. `ParseError` from `sql/parser.rs` converts to `TqError::SqlParseError` with line/column preserved (AC-20 integration)
  5. `display_profiles()` with a config containing known profiles produces expected string (AC-21)

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: variant construction, Display output, error code, conversion chain, display_profiles output
- Combined coverage: comprehensive; no gaps for pure logic changes

---

## Strategy Summary

**Total Features Analyzed:** 5

**Test Types Required:**

- Unit tests: REQUIRED for all 5 features
- CLI integration tests (binary invocation): REQUIRED for Feature 3 (flag renaming)
- Shell script tests (shellcheck + bash execution): REQUIRED for Feature 2
- Dependency injection / expectrl: REQUIRED for Feature 4 TTY path
- Code inspection: SUPPLEMENTARY for Features 1 (build.rs), 2 (AC-11, AC-12)

**Estimated Test Count:**

| Feature | Unit | CLI Integration | Shell | Total |
|---------|------|----------------|-------|-------|
| F1: Driver Resolution | 8 | 1 | 0 | 9 |
| F2: License Acceptance | 0 | 0 | 7 | 7 |
| F3: Profile Flag Naming | 2 | 5 | 0 | 7 |
| F4: Delete Confirmation | 4 | 0 | 0 | 4 |
| F5: Tech Debt | 5 | 0 | 0 | 5 |
| **Total** | **19** | **6** | **7** | **32** |

**Risk Assessment:**

- HIGH risk gaps: none
- MEDIUM risk gaps:
  - Driver actual load not tested (requires proprietary library)
  - Feature 4 TTY path may fall back to manual-only if injection not feasible
- LOW risk gaps:
  - AC-12 (README update) manual only
  - AC-16 (user guide update) manual only
  - AC-5 (release workflow) code inspection only

**Dependencies Required:**

- Live database: No
- Network access: No
- Specific OS: No (tests use temp dirs, POSIX shell)
- shellcheck: Yes (for Feature 2 static analysis; install via `brew install shellcheck` or `apt install shellcheck`)
- Binary build: Yes (for Feature 3 CLI integration tests; `cargo build` required before running CLI tests)
- expectrl: Optional (for Feature 4 TTY path if dependency injection not available in implementation)

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements (AC-1 through AC-21)
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed per feature
- [x] No hand-waving or vague justifications

---

## Tool Requests

The following tools or capabilities are needed to fully execute this test strategy:

1. **shellcheck**: Required for Feature 2 static analysis of `install.sh`. Must be available in CI and on developer machines. Install: `brew install shellcheck` (macOS) or `apt install shellcheck` (Linux). This is a standard, free, open-source tool with no licensing concerns.

2. **Binary build access**: CLI integration tests for Feature 3 require a compiled `tq` binary. Tests must be structured as integration tests that run after `cargo build` completes. The standard `cargo test` workflow builds the binary in debug mode, making it available at `target/debug/tq`.

3. **Dependency injection in `handle_delete` (implementation request)**: For Feature 4, request that the architect implement `handle_delete` with an injectable reader parameter:
   ```rust
   fn handle_delete(name: &str, force: bool, is_tty: bool, reader: &mut dyn BufRead) -> Result<()>
   ```
   This enables unit testing of the TTY path without PTY simulation. If this is not feasible due to implementation constraints, fall back to expectrl for the interactive path test.

4. **expectrl (optional)**: If dependency injection is not available for Feature 4, `expectrl` (already used in the project for REPL tests) provides PTY simulation. Check Cargo.toml for current version; no new dependency needed if already present.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-21
**Review Status:** DRAFT
**Submitted for Review:** 2026-03-21

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** -
**Review Comments:** -
