# Sprint 62 Planning: Security Hardening

## Sprint Overview

**Sprint Goal:** Address critical security vulnerabilities identified in comprehensive security audit

**Sprint Theme:** Security Hardening

**Date:** 2026-04-14
**Type:** Feature Sprint

---

## Reality Check Summary
- Reviewed sprints: 59, 60, 61
- Patterns detected: None - healthy velocity, 100% feature delivery across all 3 sprints
- Decision: Feature Sprint (security hardening)
- Rationale: External security audit identified 7 actionable findings. No crisis, but security gaps should be addressed proactively.

---

## Objectives

1. Eliminate injection vulnerabilities (JSON injection in connection strings, SQL LIKE wildcard injection)
2. Make security enforcement consistent (password file permissions)
3. Fix manual JSON construction to prevent malformed output
4. Harden supply chain (pin git dependency, add CI security checks)

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Fix JSON Injection in `to_json_string()`

**Description:** `ConnectionConfig::to_json_string()` constructs JSON via `format!()` without escaping. Passwords or hostnames containing `"` or `\` produce malformed JSON sent to the Teradata driver.

**Acceptance Criteria:**
- [ ] `to_json_string()` uses `serde_json` for safe JSON construction
- [ ] Passwords containing `"`, `\`, newlines produce valid JSON
- [ ] Unit test verifying special characters in password/host/user are correctly escaped
- [ ] Existing `test_to_json_string` test updated and passing

**Files:** `src/db/connection.rs`
**Estimated Complexity:** Low

---

#### Feature 2: Fix SQL LIKE Wildcard Injection in Abort Commands

**Description:** `find_sessions_for_host()` uses `LIKE '%{}%'` with user input, only escaping single quotes but not LIKE wildcards. A `%` hostname would match all sessions.

**Acceptance Criteria:**
- [ ] New `escape_sql_like()` utility in `src/sql/identifiers.rs` that escapes `%`, `_`, and `'`
- [ ] `find_sessions_for_host()` uses the new escape function
- [ ] Unit tests for `escape_sql_like()` including wildcard characters
- [ ] Unit test verifying `%` as hostname doesn't produce a match-all pattern

**Files:** `src/sql/identifiers.rs`, `src/commands/abort.rs`
**Estimated Complexity:** Low

---

#### Feature 3: Consistent Password File Permission Enforcement

**Description:** `check_file_permissions()` in `connection.rs` only warns about insecure permissions (used for default `~/.tq_passwords` and profile password files), while `validate_password_file_permissions()` in `main.rs` enforces and rejects. All password file reads should reject insecure permissions.

**Acceptance Criteria:**
- [ ] `check_file_permissions()` in `connection.rs` returns error (not just warning) for insecure permissions
- [ ] Profile password files loaded via `read_password_from_file()` in `config.rs` also enforce 0600
- [ ] Consistent error message format across all password file paths
- [ ] Unit tests verifying rejection of insecure password files

**Files:** `src/db/connection.rs`, `src/config.rs`
**Estimated Complexity:** Low

---

#### Feature 4: Fix Manual JSON Construction in Error Output

**Description:** `TqError::to_json()` manually constructs JSON with incomplete character escaping (misses `\t`, `\r`, control chars). Could produce malformed JSON consumed by automated agents.

**Acceptance Criteria:**
- [ ] `to_json()` uses `serde_json` for JSON construction
- [ ] Error messages with tabs, carriage returns, and control characters produce valid JSON
- [ ] All existing error JSON tests pass
- [ ] New test with control characters in error messages

**Files:** `src/error.rs`
**Estimated Complexity:** Low

---

### P1 - High Priority (Should Have)

#### Feature 5: Pin Git Dependency

**Description:** `teradatarustapi` is an unpinned git dependency tracking HEAD. A compromised commit would be silently incorporated on `cargo update`.

**Acceptance Criteria:**
- [ ] `Cargo.toml` pins `teradatarustapi` to a specific `rev`
- [ ] Current revision captured from `Cargo.lock`
- [ ] Build succeeds with pinned revision

**Files:** `Cargo.toml`, `Cargo.lock`
**Estimated Complexity:** Low

---

#### Feature 6: Add CI Security Workflow

**Description:** No CI pipeline runs security checks. Need `cargo audit`, `cargo clippy`, and test execution on PRs.

**Acceptance Criteria:**
- [ ] New `.github/workflows/ci.yml` that runs on push and PR
- [ ] Workflow includes: `cargo clippy -- -D warnings`
- [ ] Workflow includes: `cargo test`
- [ ] Workflow includes: `cargo audit` (or install + run)
- [ ] Workflow targets stable Rust

**Files:** `.github/workflows/ci.yml`
**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- TLS/SSL configuration for database connections (requires Teradata driver support investigation)
- OS keychain integration for secrets management (large scope, separate sprint)
- Binary signing / SBOM generation (separate DevSecOps sprint)
- History file permission enforcement (minor risk, can be done later)
- Output file path validation (OS permissions already prevent most issues)

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] All P1 features implemented and tested
- [ ] 100% test pass rate (unit + integration)
- [ ] Zero clippy warnings
- [ ] No new technical debt introduced
- [ ] All acceptance criteria met

---

## Dependencies

### External Dependencies
- None

### Prerequisite Work
- None (all changes are independent of each other)

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: serde_json changes JSON field ordering
- **Probability:** Low
- **Impact:** Low (field order doesn't matter for JSON consumers)
- **Mitigation:** Update tests to be order-independent if needed

### Risk 2: Pinning teradatarustapi to wrong revision
- **Probability:** Low
- **Impact:** High (build would break)
- **Mitigation:** Extract exact revision from current Cargo.lock

---

## Agent Assignments

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement Features 1-4 (code security fixes)
- Implement Feature 5 (pin dependency)
- Write unit tests for all changes

### quality-validator (Sonnet)
**Responsibilities:**
- Execute all test suites
- Verify JSON output validity
- Validate permission enforcement behavior

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Feature 6: Design CI workflow
- Review error message changes for UX consistency

---

## Files Involved

### Feature 1: JSON Injection Fix
- `src/db/connection.rs` - Replace format!() with serde_json

### Feature 2: LIKE Wildcard Injection Fix
- `src/sql/identifiers.rs` - New escape_sql_like() function
- `src/commands/abort.rs` - Use escape_sql_like in find_sessions_for_host

### Feature 3: Permission Enforcement
- `src/db/connection.rs` - Make check_file_permissions return error
- `src/config.rs` - Ensure consistent enforcement

### Feature 4: Error JSON Fix
- `src/error.rs` - Replace manual JSON with serde_json

### Feature 5: Pin Dependency
- `Cargo.toml` - Add rev= to teradatarustapi

### Feature 6: CI Workflow
- `.github/workflows/ci.yml` - New file

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Initial sprint plan | Sprint Coordinator |
