# Sprint 17 Test Strategy: Configuration UX Completion

**Created:** 2026-01-21
**Author:** quality-validator
**Sprint:** Sprint 17
**Features:** Help Subcommands, Security Check Ordering, Password Permission Enforcement, Profile Listing, Logmech Parsing Refactoring

---

## Overview

Sprint 17 completes the configuration user experience started in Sprint 16 by adding help subcommands, fixing security issues, and improving profile management. This test strategy derives test requirements from Sprint 17 planning documents and detailed specifications.

**Sprint Context:**
- Builds on Sprint 16's configuration foundation (100% test pass rate, zero tech debt)
- Implements P1 recommendations from Sprint 16 review
- No new database features - focuses on CLI UX and security hardening
- All features are batch mode CLI commands (no REPL changes)

---

## Feature-by-Feature Test Strategy

### Feature 1: Help Subcommands (P0)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 sections 4.4.1
- Secondary: `docs/builder/detailed-specifications/configuration.md` v2.1.0 sections 7.8.1, 7.8.3
- Sprint Planning: `docs/builder/sprints/sprint-17-planning.md` lines 51-68

**Requirements:**
1. "`tq help config` displays comprehensive configuration file documentation" (sprint-17-planning.md line 56)
2. "`tq help credentials` displays password management guide" (sprint-17-planning.md line 57)
3. "Help content includes TOML examples, file locations, and security best practices" (sprint-17-planning.md line 58)
4. "Error handling: Unknown help topics display available topics" (sprint-17-planning.md line 59)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch
**Explanation:** Help subcommands are non-interactive CLI commands that display text and exit. Users invoke them from shell scripts, terminal sessions, or CI/CD pipelines.

**Observable Behavior:**
- ✅ Structured data output (text documentation, formatted help content)
- Exit codes (0 for success, 2 for usage errors)

**External Dependencies:**
- ❌ None (pure logic, no external dependencies)

**Validation Challenges:**
- Content validation: Must verify help text contains required sections (not just that it exists)
- Format validation: Help output should be readable and well-structured
- Completeness: All promised content from specifications must be present

**Critical Behaviors to Validate:**
1. `tq help config` displays configuration file format, profile fields, precedence order, security practices (configuration.md §7.8.1)
2. `tq help credentials` displays password sources, file format, security enforcement, creation steps (configuration.md §7.8.3)
3. `tq help unknown` shows error with list of available topics (cli-interface.md §4.4.1 line 94-96)
4. Help content is comprehensive enough for users to configure tq without external documentation

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: End-to-end CLI execution needs validation with real arguments

IF "Structured data output" checked:
  → Integration tests OR unit tests with content validation REQUIRED
  Reason: Must validate output contains semantically correct content
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Help content generation logic, available topics list
- **Approach:** Test functions that generate help text, verify content completeness
- **Rationale:** Ensures help text contains required sections before integration testing
- **Gap if missing:** Content errors (missing sections) not caught until manual testing
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 2: Integration Tests**
- **Validates:** Full command execution, exit codes, stdout/stderr separation, content presence
- **Approach:** Execute `tq help config`, `tq help credentials`, `tq help unknown` and validate output
- **Rationale:** Validates end-to-end user experience as specified
- **Gap if missing:** CLI argument parsing bugs, output formatting issues, exit code errors
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ⚠️ RECOMMENDED | Validates help content completeness | Content quality issues | IMPLEMENT IF TIME PERMITS |
| Integration tests | ✅ REQUIRED | Validates CLI invocation and output | CLI bugs, user-facing issues | MUST IMPLEMENT |
| Interactive tests | ❌ NOT NEEDED | Help is batch command, not REPL | N/A | SKIP |
| Manual tests | ⚠️ RECOMMENDED | Human validates readability and usefulness | UX issues | DOCUMENT CHECKLIST |

**Summary:**
- ✅ REQUIRED test types: 1 (Integration tests)
- ⚠️ RECOMMENDED test types: 2 (Unit tests, Manual validation)
- ❌ NOT NEEDED test types: 1 (Interactive tests)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| HELP-1 | `tq help config` displays configuration documentation | cli-interface.md §4.4.1 | Integration | Must validate actual command output | TC-HELP-001 |
| HELP-2 | `tq help credentials` displays credentials documentation | cli-interface.md §4.4.1 | Integration | Must validate actual command output | TC-HELP-002 |
| HELP-3 | Unknown help topic shows error | cli-interface.md §4.4.1 | Integration | Must validate error handling and exit code | TC-HELP-003 |
| HELP-4 | Config help includes TOML examples | configuration.md §7.8.1 | Integration | Content validation of help output | TC-HELP-001 |
| HELP-5 | Credentials help includes file format | configuration.md §7.8.3 | Integration | Content validation of help output | TC-HELP-002 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Interactive Tests (expectrl)**
- **Reason for omission:** Help subcommands are batch mode CLI commands, not REPL features
- **What won't be validated:** N/A (no interactive behavior to validate)
- **Risk assessment:** NONE - Feature has no interactive component
- **Mitigation:** N/A
- **Revisit criteria:** Never (wrong test type for this feature)

**Performance/Benchmark Tests**
- **Reason for omission:** Help commands have no performance requirements, execution time not critical
- **What won't be validated:** Help command execution speed
- **Risk:** NONE - Help is documentation display, not performance-critical
- **Mitigation:** N/A
- **Revisit criteria:** If help generation becomes slow (>1s) or users report issues

#### 6. Test Implementation Plan

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or dedicated `tests/help_tests.rs`
- **Framework:** Built-in Rust integration test with std::process::Command
- **Test count estimate:** 3 tests (config, credentials, unknown)
- **Key scenarios to cover:**
  1. `tq help config` - validates exit code 0, stdout contains required sections
  2. `tq help credentials` - validates exit code 0, stdout contains required sections
  3. `tq help unknown` - validates exit code 2, stderr contains error and available topics
- **Content validation approach:** Use `assert!(output.contains("expected section"))` for key sections
- **Sections to validate for config help:**
  - "CONFIGURATION FILE"
  - "FILE FORMAT"
  - "PRECEDENCE ORDER"
  - "PROFILE FIELDS"
  - "SECURITY BEST PRACTICES"
- **Sections to validate for credentials help:**
  - "PASSWORD SECURITY"
  - "PASSWORD FILES"
  - "CREATING A PASSWORD FILE"
  - "PASSWORD SOURCES"
  - "SECURITY ENFORCEMENT"

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Integration tests validate: CLI invocation, exit codes, content presence, error handling
- Manual validation validates: Help text readability, usefulness, formatting quality
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - integration tests cover all observable behavior

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

---

### Feature 2: Security Check Ordering Fix (P0)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-17-planning.md` lines 72-87
- Secondary: `docs/builder/detailed-specifications/configuration.md` v2.1.0 sections 7.6.3
- Code reference: `src/main.rs` function `read_password_if_needed`

**Requirements:**
1. "`validate_password_file_permissions` called BEFORE `read_to_string` in `read_password_if_needed`" (sprint-17-planning.md line 76)
2. "Security check prevents reading insecure files before any file content is accessed" (sprint-17-planning.md line 77)
3. "Behavior matches `config.rs` pattern (correct order)" (sprint-17-planning.md line 78)

**Feature Characteristics:**

**User Interaction Type:** ✅ Pure Logic
**Explanation:** This is an internal code fix ensuring security checks execute before file reads. No direct user-observable change (security behavior was always enforced, but order was wrong).

**Observable Behavior:**
- File system side effects (permission check happens before read attempt)
- Security guarantee (insecure files never have content read)

**External Dependencies:**
- ✅ File system access (checks file permissions)

**Validation Challenges:**
- Race condition testing: Must prove check happens BEFORE read
- Code ordering validation: Not about behavior change, but execution order
- Negative testing: Must verify insecure file content is never accessed

**Critical Behaviors to Validate:**
1. Insecure password file (0644) is rejected without reading content (configuration.md §7.6.3)
2. Permission check error occurs before file read error
3. No race condition between check and read

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked:
  → Unit tests REQUIRED
  Reason: Internal logic validation, no user interaction needed

IF "File system access" checked:
  → Integration tests with temp files REQUIRED
  Reason: Must test actual file permission behavior
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Function call ordering, permission check before read logic
- **Approach:** Mock or test with temp files, verify permission error before read error
- **Rationale:** Ensures security check executes first at code level
- **Gap if missing:** Code ordering bugs, future refactoring could break order
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates:** End-to-end behavior with real files and permissions
- **Approach:** Create password file with 0644 permissions, verify tq rejects it without reading
- **Rationale:** Validates actual security behavior users experience
- **Gap if missing:** Platform-specific permission bugs, real-world file system issues
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates call ordering logic | Execution order bugs | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates real file permission behavior | Platform-specific bugs | MUST IMPLEMENT |
| Interactive tests | ❌ NOT NEEDED | Not a REPL feature | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 2 (Unit, Integration)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 1 (Interactive)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| SEC-1 | Permission check before file read | sprint-17-planning.md line 76 | Unit + Integration | Unit validates order, integration validates behavior | TC-SECURITY-003 |
| SEC-2 | Insecure file rejected before content access | sprint-17-planning.md line 77 | Integration | Must test with real file permissions | TC-SECURITY-001 |
| SEC-3 | Behavior matches config.rs pattern | sprint-17-planning.md line 78 | Unit | Code review validates consistency | TC-SECURITY-003 |

**Coverage Validation:**
- ✅ All requirements covered
- ✅ All test types justified

**Coverage Gaps:** None

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Interactive Tests**
- **Reason:** Not a REPL feature, batch mode only
- **Risk:** NONE
- **Mitigation:** N/A

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/main.rs` test module or dedicated test
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 1 test
- **Key scenario:** Verify permission check error occurs before file read error
- **Approach:** Create test with insecure permission file, assert error message indicates permission issue (not read failure)

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or `tests/security_tests.rs`
- **Framework:** std::process::Command + tempfile crate
- **Test count estimate:** 1 test
- **Key scenario:** Create password file with 0644, invoke tq with --password-file, verify rejection
- **Setup:** Use tempfile to create test password file with 0644 permissions
- **Validation:** Exit code non-zero, error message mentions permissions

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: Code ordering, permission check logic
- Integration tests validate: Real file permission behavior, error messages
- Combined coverage: **Comprehensive**

**Acceptance criteria:**
- ✅ All requirements covered
- ✅ Security guarantee validated
- ✅ Can claim "fix works as specified"

---

### Feature 3: Password File Permission Enforcement (P1)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/configuration.md` v2.1.0 sections 7.6.3 (lines 401-423)
- Secondary: `docs/builder/sprints/sprint-17-planning.md` lines 92-107
- Context: Sprint 16 implementation warned but allowed insecure files

**Requirements:**
1. "Password file with permissions other than 0600 results in error (not warning)" (sprint-17-planning.md line 97)
2. "Error message explains security risk and provides fix command (`chmod 0600 ...`)" (sprint-17-planning.md line 98)
3. "Password files must have 0600 permissions. tq ENFORCES password file permissions." (configuration.md lines 412-415)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch
**Explanation:** Password file permission enforcement happens during connection setup in batch mode commands.

**Observable Behavior:**
- ✅ File system side effects (permission check)
- Error messages with actionable guidance
- Exit codes (error on insecure permissions)

**External Dependencies:**
- ✅ File system access (file permission checking)

**Validation Challenges:**
- Platform differences: File permissions work differently on Windows
- Destructive change: Changes existing behavior from warning to error
- User impact: Existing users with 0644 files will see errors

**Critical Behaviors to Validate:**
1. Password file with 0644 permissions is **rejected** (not warned) (configuration.md line 413)
2. Error message includes current permissions, required permissions, and fix command (configuration.md lines 416-418)
3. Password file with 0600 permissions is **accepted** (no regression)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" + "File system access" checked:
  → Integration tests REQUIRED
  Reason: Must validate actual file permission behavior and error messages
```

**Derived Test Types:**

**Test Type 1: Integration Tests**
- **Validates:** Permission enforcement, error messages, exit codes
- **Approach:** Create password files with various permissions, test tq behavior
- **Rationale:** Only way to test real file permission validation
- **Gap if missing:** Enforcement not validated, user-facing errors not tested
- **Necessity:** ✅ REQUIRED

**Test Type 2: Unit Tests**
- **Validates:** Permission validation logic (0600 check)
- **Approach:** Test permission checking function in isolation
- **Rationale:** Validates core logic independent of file system
- **Gap if missing:** Logic errors in permission check
- **Necessity:** ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Integration tests | ✅ REQUIRED | Validates enforcement behavior | User-facing bugs, error messages | MUST IMPLEMENT |
| Unit tests | ⚠️ RECOMMENDED | Validates permission check logic | Logic bugs | IMPLEMENT IF SIMPLE |
| Interactive tests | ❌ NOT NEEDED | Not REPL feature | N/A | SKIP |

**Summary:**
- ✅ REQUIRED: 1 (Integration)
- ⚠️ RECOMMENDED: 1 (Unit)
- ❌ NOT NEEDED: 1 (Interactive)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| PERM-1 | 0644 permissions rejected with error | sprint-17-planning.md line 97 | Integration | Must test real behavior | TC-SECURITY-001 |
| PERM-2 | Error includes fix command | sprint-17-planning.md line 98 | Integration | Must validate error message content | TC-SECURITY-001 |
| PERM-3 | 0600 permissions accepted | Implicit requirement | Integration | Regression test | Not yet covered - ADD |
| PERM-4 | Config file 0644 only warns | configuration.md §7.3.4 | Integration | Different behavior from password files | TC-SECURITY-002 |

**Coverage Validation:**
- ✅ All requirements covered
- ⚠️ Need to add test for 0600 acceptance (no regression)

**Coverage Gaps:**
- Missing positive test: Must verify 0600 files still work

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Platform-Specific Tests (Windows)**
- **Reason:** File permissions work differently on Windows, behavior may vary
- **What won't be validated:** Windows-specific permission enforcement
- **Risk:** MEDIUM - Windows users may have different experience
- **Mitigation:** Document platform differences, test on Linux/macOS only for Sprint 17
- **Revisit criteria:** If Windows users report issues or request Windows support

#### 6. Test Implementation Plan

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or `tests/security_tests.rs`
- **Framework:** std::process::Command + tempfile + std::fs::set_permissions
- **Test count estimate:** 2 tests
- **Key scenarios:**
  1. Password file with 0644 permissions - verify rejection and error message
  2. Password file with 0600 permissions - verify acceptance (regression test)
- **Setup approach:**
  ```rust
  // Create temp password file
  let temp_file = NamedTempFile::new()?;
  write!(temp_file, "testpassword")?;

  // Set permissions to 0644
  let mut perms = std::fs::metadata(temp_file.path())?.permissions();
  perms.set_mode(0o644);
  std::fs::set_permissions(temp_file.path(), perms)?;

  // Execute tq with --password-file
  let output = Command::new("tq")
      .arg("--password-file")
      .arg(temp_file.path())
      .arg("ping")
      .output()?;

  // Verify rejection
  assert_ne!(output.status.code(), Some(0));
  assert!(String::from_utf8_lossy(&output.stderr).contains("insecure permissions"));
  assert!(String::from_utf8_lossy(&output.stderr).contains("chmod 0600"));
  ```

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Integration tests validate: Permission enforcement, error messages, no regression
- Combined coverage: **Comprehensive** (for Linux/macOS)

**Gaps:**
- Windows platform not tested (documented limitation)

**Acceptance criteria:**
- ✅ All requirements covered for Linux/macOS
- ✅ Error messages validated
- ✅ No regression in 0600 acceptance
- ⚠️ Windows not tested (accepted gap)

---

### Feature 4: Profile Listing Command (P1)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 sections 4.4.5 (lines 272-355)
- Secondary: `docs/builder/detailed-specifications/configuration.md` v2.1.0 sections 7.4.4 (lines 259-293)
- Planning: `docs/builder/sprints/sprint-17-planning.md` lines 111-127

**Requirements:**
1. "`tq profiles` command lists all available profiles from `~/.config/tq/config.toml`" (sprint-17-planning.md line 116)
2. "Output shows profile names and partial connection info (host, database, but NOT passwords)" (sprint-17-planning.md line 117)
3. "Error handling: No config file displays helpful message with setup instructions" (sprint-17-planning.md line 118)
4. "Error handling: Empty profiles section displays 'No profiles defined'" (sprint-17-planning.md line 119)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch
**Explanation:** `tq profiles` is a batch command that reads config and displays output.

**Observable Behavior:**
- ✅ Structured data output (formatted profile list)
- File system side effects (reads config file)
- Error handling (no file, no profiles, parse errors)

**External Dependencies:**
- ✅ File system access (reads ~/.tq/config.toml)

**Validation Challenges:**
- Config file variations: Must test with profiles, without profiles, without file
- Security: Must verify passwords NEVER displayed
- Output format: Must validate readability and structure

**Critical Behaviors to Validate:**
1. Profiles listed with host, database, user, logmech (cli-interface.md lines 290-310)
2. Passwords and password_file paths NEVER shown (cli-interface.md lines 347-350)
3. No config file shows helpful setup message (cli-interface.md lines 315-329)
4. Config exists but no profiles shows guidance (cli-interface.md lines 333-344)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" + "File system access" checked:
  → Integration tests REQUIRED
  Reason: Must validate file reading and output generation
```

**Derived Test Types:**

**Test Type 1: Integration Tests**
- **Validates:** Profile listing, error cases, security (no password display)
- **Approach:** Create test config files, execute `tq profiles`, validate output
- **Rationale:** Only way to test complete user experience with real config files
- **Gap if missing:** Output format issues, security leaks, error message quality
- **Necessity:** ✅ REQUIRED

**Test Type 2: Unit Tests**
- **Validates:** Profile parsing logic, output formatting function
- **Approach:** Test profile loading and formatting in isolation
- **Rationale:** Validates core logic independent of CLI
- **Gap if missing:** Logic bugs in profile parsing
- **Necessity:** ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Integration tests | ✅ REQUIRED | Validates end-to-end user experience | CLI bugs, security issues | MUST IMPLEMENT |
| Unit tests | ⚠️ RECOMMENDED | Validates parsing and formatting logic | Logic bugs | IMPLEMENT IF TIME |
| Interactive tests | ❌ NOT NEEDED | Not REPL feature | N/A | SKIP |
| Manual security review | ✅ REQUIRED | Must verify no password leaks | Security vulnerability | MUST PERFORM |

**Summary:**
- ✅ REQUIRED: 2 (Integration, Security review)
- ⚠️ RECOMMENDED: 1 (Unit)
- ❌ NOT NEEDED: 1 (Interactive)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| PROF-1 | List profiles from config file | cli-interface.md §4.4.5 | Integration | Must test with real config | TC-PROFILES-001 |
| PROF-2 | No config file error | cli-interface.md lines 315-329 | Integration | Must validate error handling | TC-PROFILES-002 |
| PROF-3 | No profiles defined error | cli-interface.md lines 333-344 | Integration | Must validate error handling | TC-PROFILES-003 |
| PROF-4 | Passwords never displayed | cli-interface.md lines 347-350 | Integration + Manual | Security critical | TC-PROFILES-001 |
| PROF-5 | Output shows host, db, user, logmech | cli-interface.md lines 290-310 | Integration | Content validation | TC-PROFILES-001 |

**Coverage Validation:**
- ✅ All requirements covered
- ✅ Security requirement has multiple validation approaches

**Coverage Gaps:** None

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance Tests**
- **Reason:** Profile listing has no performance requirements
- **Risk:** NONE
- **Revisit:** If users have hundreds of profiles and report slowness

#### 6. Test Implementation Plan

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or `tests/profiles_tests.rs`
- **Framework:** std::process::Command + tempfile for config files
- **Test count estimate:** 3 tests
- **Key scenarios:**
  1. Config file with profiles - verify all profiles listed, no passwords
  2. No config file - verify helpful error message
  3. Config file with no profiles section - verify "No profiles defined" message
- **Setup approach:**
  ```rust
  // Create temp config with profiles
  let config_content = r#"
  [profiles.dev]
  host = "dev.example.com"
  port = 1025
  database = "devdb"
  user = "alice"
  logmech = "TD2"
  password_file = "/secret/path"

  [profiles.prod]
  host = "prod.example.com"
  database = "proddb"
  user = "bob"
  "#;

  // Execute tq profiles with custom config location
  let output = Command::new("tq").arg("profiles").env("TQ_CONFIG", temp_path).output()?;

  // Validate output
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("dev"));
  assert!(stdout.contains("dev.example.com"));
  assert!(stdout.contains("alice"));
  assert!(!stdout.contains("/secret/path")); // SECURITY: No password file path
  assert!(!stdout.contains("password")); // SECURITY: No password field
  ```

**Manual Security Review Checklist:**
- [ ] Review output format implementation to ensure no password fields included
- [ ] Test with profile containing inline password (if supported) - must not display
- [ ] Test with password_file field - path must not display
- [ ] Review code for debug/logging that might leak passwords

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Integration tests validate: Profile listing, error handling, basic security
- Manual security review validates: Comprehensive password leak prevention
- Combined coverage: **Comprehensive**

**Acceptance criteria:**
- ✅ All requirements covered
- ✅ Security validated by tests and manual review
- ✅ Error cases covered
- ✅ Can claim "works as specified"

---

### Feature 5: Logmech Parsing Refactoring (P2)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-17-planning.md` lines 133-149
- Code reference: `src/config.rs` and `src/main.rs` logmech parsing

**Requirements:**
1. "Make `config::parse_logmech` public (or create shared function)" (sprint-17-planning.md line 138)
2. "Replace inline parsing in `main.rs` with function call" (sprint-17-planning.md line 139)
3. "All existing tests pass (no behavior changes)" (sprint-17-planning.md line 140)

**Feature Characteristics:**

**User Interaction Type:** ✅ Pure Logic (internal refactoring)
**Explanation:** Code quality improvement with no user-observable changes. Eliminates code duplication.

**Observable Behavior:**
- ❌ None (internal refactoring, no behavior change)

**External Dependencies:**
- ❌ None

**Validation Challenges:**
- No observable behavior to test (refactoring only)
- Must ensure no regressions introduced
- Must validate DRY principle applied

**Critical Behaviors to Validate:**
1. All existing tests continue to pass (sprint-17-planning.md line 140)
2. Logmech parsing behavior unchanged
3. No new dependencies or complexity (sprint-17-planning.md line 141)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" + "No behavior change" checked:
  → Regression tests REQUIRED (run existing test suite)
  → New tests NOT NEEDED (no new behavior)
```

**Derived Test Types:**

**Test Type 1: Regression Tests**
- **Validates:** No existing functionality broken
- **Approach:** Run full existing test suite (unit + integration)
- **Rationale:** Refactoring should not change behavior
- **Gap if missing:** Regression bugs introduced
- **Necessity:** ✅ REQUIRED

**Test Type 2: Code Review**
- **Validates:** DRY principle applied, no duplication remains
- **Approach:** Manual code review of config.rs and main.rs
- **Rationale:** Ensures refactoring goal achieved
- **Gap if missing:** Incomplete refactoring
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Regression tests | ✅ REQUIRED | Must ensure no behavior change | Regressions introduced | RUN EXISTING SUITE |
| Code review | ✅ REQUIRED | Must verify refactoring complete | Incomplete refactoring | MANUAL REVIEW |
| New unit tests | ❌ NOT NEEDED | No new behavior to test | N/A | SKIP |
| New integration tests | ❌ NOT NEEDED | No new behavior to test | N/A | SKIP |

**Summary:**
- ✅ REQUIRED: 2 (Regression, Code review)
- ❌ NOT NEEDED: 2 (New unit/integration tests)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| REFACTOR-1 | Make parse_logmech public/shared | sprint-17-planning.md line 138 | Code Review | Code structure validation | Manual review |
| REFACTOR-2 | Replace inline parsing in main.rs | sprint-17-planning.md line 139 | Code Review | Code structure validation | Manual review |
| REFACTOR-3 | All existing tests pass | sprint-17-planning.md line 140 | Regression | Behavior preservation | Existing test suite |

**Coverage Validation:**
- ✅ All requirements covered
- ✅ Appropriate validation method for each requirement

**Coverage Gaps:** None

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**New Test Cases**
- **Reason:** Refactoring adds no new behavior, existing tests already cover logmech parsing
- **What won't be validated:** N/A (existing tests validate behavior)
- **Risk:** NONE
- **Mitigation:** Run full regression suite
- **Revisit criteria:** Never (tests already exist)

#### 6. Test Implementation Plan

**Validation Approach: Regression Testing**
- **Location:** Run existing test suite: `cargo test`
- **Framework:** Existing test framework
- **Test count:** ~280+ existing tests (from Sprint 16 baseline)
- **Execution:**
  ```bash
  # Run all unit and integration tests
  cargo test --lib
  cargo test --test integration_tests

  # Verify all pass
  echo "Expected: 280+ tests passing (Sprint 16 baseline)"
  ```
- **Success criteria:** 100% test pass rate, no new failures

**Validation Approach: Code Review**
- **Checklist:**
  - [ ] `config::parse_logmech` is public or shared function exists
  - [ ] `main.rs` no longer has inline logmech parsing
  - [ ] Both locations use same parsing function
  - [ ] No new dependencies added
  - [ ] Code is simpler and more maintainable
  - [ ] No duplicate parsing logic remains

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Regression tests validate: No behavior change, all features still work
- Code review validates: Refactoring goal achieved, DRY principle applied
- Combined coverage: **Sufficient for refactoring**

**Acceptance criteria:**
- ✅ All existing tests pass
- ✅ Code review confirms refactoring complete
- ✅ Can claim "refactoring successful, no regressions"

---

## Strategy Summary

**Total Features Analyzed:** 5

**Test Types Required:**
- Unit tests: ⚠️ [Security ordering, Permission validation, Help content] (recommended)
- Integration tests: ✅ [All features except refactoring] (required)
- Interactive tests: ❌ [none] (not applicable)
- Regression tests: ✅ [Refactoring only] (required)
- Code review: ✅ [Refactoring, Security] (required)
- Manual testing: ⚠️ [Help readability, Security review] (recommended)

**Estimated Test Count:**
- Integration: 9 tests
  - Help: 3 (config, credentials, unknown)
  - Security: 3 (ordering, 0644 rejection, 0600 acceptance)
  - Profiles: 3 (with profiles, no config, no profiles)
- Unit: ~5 tests (if implemented)
  - Help content generation: 2
  - Permission validation: 2
  - Security ordering: 1
- Regression: ~280 tests (existing suite)
- **Total new tests: 9-14** (integration required, unit recommended)

**Test Cases to Document:**
- TC-HELP-001: `tq help config` displays configuration help
- TC-HELP-002: `tq help credentials` displays credentials help
- TC-HELP-003: Unknown help topic shows error
- TC-PROFILES-001: List profiles from config file
- TC-PROFILES-002: No config file error handling
- TC-PROFILES-003: Config exists but no profiles error handling
- TC-SECURITY-001: Password file with 0644 permissions rejected
- TC-SECURITY-002: Config file with 0644 permissions warns (not rejects)
- TC-SECURITY-003: Security check happens before file read

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:**
  - Windows platform not tested for file permissions (documented limitation)
- **LOW risk gaps:**
  - Unit tests for help content (integration tests sufficient)
  - Unit tests for profile parsing (integration tests sufficient)

**Dependencies Required:**
- Live database: ❌ NO (Sprint 17 features are CLI-only, no database interaction)
- Network access: ❌ NO
- Specific OS: ⚠️ YES (File permission tests for Linux/macOS only)
- Config files: ✅ YES (Temp files created in tests)

**Sprint 17 Specific Notes:**
- **No database required** - All features are configuration/CLI UX, can test without Teradata
- **No REPL changes** - All features are batch mode, no interactive tests needed
- **Security focus** - Multiple tests validate security enforcement and password protection
- **Regression critical** - Must verify Sprint 16 config features still work

---

## Test Execution Strategy

### Phase 1: Unit Tests (Optional, Time Permitting)
Run first to catch logic issues early:
```bash
cargo test --lib test_security_ordering
cargo test --lib test_permission_validation
cargo test --lib test_help_content
```

### Phase 2: Integration Tests (Required)
Execute in priority order:

**Critical Path (Must Pass):**
1. TC-HELP-001, TC-HELP-002, TC-HELP-003 - Help subcommands
2. TC-SECURITY-003 - Security ordering fix
3. TC-SECURITY-001 - Permission enforcement

**High Priority:**
4. TC-PROFILES-001, TC-PROFILES-002, TC-PROFILES-003 - Profile listing
5. TC-SECURITY-002 - Config file permissions (warning vs error)

**Execution:**
```bash
# Run new integration tests
cargo test --test integration_tests test_help
cargo test --test integration_tests test_security
cargo test --test integration_tests test_profiles

# Or run all integration tests
cargo test --test integration_tests
```

### Phase 3: Regression Tests (Required for Refactoring)
Verify no Sprint 16 features broken:
```bash
# Run full test suite
cargo test --lib
cargo test --test integration_tests

# Expected: 280+ tests passing (Sprint 16 baseline)
# Any failures indicate regression
```

### Phase 4: Manual Validation (Recommended)
Execute manual checks:

**Help Readability:**
- [ ] Run `tq help config` - Is output readable and comprehensive?
- [ ] Run `tq help credentials` - Is output helpful for users?
- [ ] Verify examples in help text are accurate

**Security Review:**
- [ ] Run `tq profiles` with profile containing password_file - No path displayed?
- [ ] Check code for debug logging that might leak passwords
- [ ] Verify error messages don't expose sensitive info

**Code Review (for refactoring):**
- [ ] Verify `config::parse_logmech` is public/shared
- [ ] Verify `main.rs` uses shared function (no duplication)
- [ ] Check git diff for refactoring completeness

### Phase 5: Test Report Generation
Document results in `tests/results/sprint-17/REPORT.md`

---

## Success Criteria

Sprint 17 test strategy is successful if:

1. **Test Coverage Complete:**
   - ✅ All 5 features have test strategy defined
   - ✅ All P0 and P1 requirements mapped to tests
   - ✅ Test types derived from feature characteristics (not assumed)

2. **Test Implementation Achievable:**
   - ✅ 9 integration tests (clear scope, well-defined)
   - ✅ No database dependency (can test without Teradata)
   - ✅ Test setup straightforward (temp files, command execution)

3. **Quality Assurance Robust:**
   - ✅ Security features have multiple validation approaches
   - ✅ Regression testing prevents Sprint 16 breakage
   - ✅ Manual validation covers subjective quality (readability, usefulness)

4. **Gaps Identified and Accepted:**
   - ✅ Windows platform limitation documented
   - ✅ Optional unit tests identified (not blocking)
   - ✅ All gaps have risk assessment

5. **Execution Plan Clear:**
   - ✅ Test phases defined with dependencies
   - ✅ Priority order established
   - ✅ Success criteria for each phase

---

## Strategy Validation Checklist

**Before submitting for review:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (not guessed)
- ✅ Every test type has clear rationale
- ✅ Gap analysis is complete and honest
- ✅ Specification coverage map includes all requirements
- ✅ Every requirement maps to at least one test type
- ✅ Test implementation plan is detailed and actionable
- ✅ Coverage sufficiency is assessed
- ✅ No hand-waving or vague justifications

**Strategy Status:** READY FOR IMPLEMENTATION

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-21
**Review Status:** DRAFT
**Sprint:** 17 - Configuration UX Completion
