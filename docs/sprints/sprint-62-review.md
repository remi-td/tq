# Sprint 62 Review: Security Hardening

## Sprint Overview

**Sprint Goal:** Address critical security vulnerabilities identified in comprehensive security audit

**Sprint Theme:** Security Hardening

**Date:** 2026-04-14
**Version:** v1.44.0
**Type:** Feature Sprint

---

## Objectives Completed

### Feature 1: Fix JSON Injection in `to_json_string()` (P0) ✅

Replaced `format!()` manual JSON construction with `serde_json::json!()` in `ConnectionConfig::to_json_string()`.

**Implementation:**
- Passwords and hostnames containing `"`, `\`, newlines, or control characters now produce valid JSON
- `dbs_port` emitted as string for Teradata driver compatibility
- Updated existing test to validate via JSON parsing
- New `test_to_json_string_special_characters` test with injection-prone password

### Feature 2: Fix SQL LIKE Wildcard Injection (P0) ✅

Added `escape_sql_like()` utility and fixed `find_sessions_for_host()` in abort command.

**Implementation:**
- New `escape_sql_like()` in `src/sql/identifiers.rs` escapes `\`, `%`, `_`, and `'`
- `find_sessions_for_host()` now uses `escape_sql_like()` with `ESCAPE '\'` clause
- A `%` hostname can no longer match all sessions
- 2 unit tests covering wildcards and injection prevention

### Feature 3: Consistent Password File Permission Enforcement (P0) ✅

Changed `check_file_permissions()` from warn-only to hard error rejection.

**Implementation:**
- `check_file_permissions()` in `connection.rs` now returns `Err(TqError::InvalidConfig(...))` for insecure permissions
- Error message includes current mode, required mode, and `chmod 0600` fix command
- Consistent with `validate_password_file_permissions()` in `main.rs`
- 2 unit tests: one verifying rejection of 0644, one verifying acceptance of 0600

### Feature 4: Fix Manual JSON in Error Output (P0) ✅

Replaced hand-crafted JSON in `TqError::to_json()` with `serde_json::json!()`.

**Implementation:**
- All special characters (`\t`, `\r`, `"`, `\`, control chars) now properly escaped
- Clean nested structure: `{"ok": false, "error": {...}}`
- New `test_to_json_with_control_characters` test

### Feature 5: Pin Git Dependency (P1) ✅

Pinned `teradatarustapi` to specific commit revision.

**Implementation:**
- Added `rev = "046a8b0faaa6eefa597dbd9fcfc575b066465d74"` to Cargo.toml
- Prevents silent dependency drift on `cargo update`

### Feature 6: Add CI Security Workflow (P1) ✅

New `.github/workflows/ci.yml` with security checks.

**Implementation:**
- Triggers on push to master and pull requests
- `check` job: `cargo clippy -- -D warnings` + `cargo test --lib`
- `audit` job: `cargo install cargo-audit` + `cargo audit`
- Cargo caching for faster CI runs

---

## Metrics

| Metric | Value |
|--------|-------|
| Features completed | 6/6 (100%) |
| P0 features | 4/4 |
| P1 features | 2/2 |
| New unit tests | 6 |
| Total unit tests | 1049 |
| Test pass rate | 100% |
| Clippy warnings | 0 |
| Lines added | ~734 |
| Lines removed | ~45 |
| Version | v1.44.0 |

Token metrics not collected for this sprint - transcript data unavailable.

---

## Agent Reviews

### Technical Review (rust-teradata-architect)

**Verdict: Sound implementation.**

All six changes follow the correct security-hardening pattern: replace hand-crafted string construction with library-backed serialization or dedicated escaping functions. The `serde_json::json!()` replacements in both `to_json_string()` and `to_json()` are textbook fixes. The `escape_sql_like()` function is well-ordered (backslash first, then wildcards, then quotes).

**Remaining concerns:**
- `find_sessions_for_user()` in abort.rs still uses inline `replace('\'', "''")` instead of the shared `escape_sql_string()` utility. Functionally equivalent but inconsistent.
- CI workflow does not trigger on tags. Consider adding `tags: ['v*']` to ensure CI runs before release artifacts.

### Quality Review (quality-validator)

**Verdict: APPROVED.**

100% pass rate across 1049 unit tests and 92 integration tests. All security fixes have corresponding tests that validate via `serde_json::from_str()` parsing (not just string matching). The `test_escape_sql_like_injection_prevention` test explicitly validates the match-all prevention scenario.

Initial gap identified (no unit test for permission enforcement) was addressed with 2 additional tests before sprint closure.

### UX Review (cli-ux-designer)

**Verdict: Acceptable with minor note.**

The breaking behavioral change (password file permission warn -> error) is the correct security posture. The error message includes the file path, current permissions, required mode, and exact `chmod` fix command. The spec (REQ-SEC-002) documents the rationale well.

Minor note: the error message format in code uses inline `\n` separators while the spec shows a multi-line labeled format. The inline format is functional; the labeled format would be more scannable.

---

## Retrospective

### What Went Well

1. **Security audit drove clear scope:** The comprehensive audit produced specific, actionable findings with exact file/line references, making sprint planning trivial.
2. **All low-complexity, high-impact:** Every feature was a focused, surgical fix. No architectural changes needed.
3. **Existing patterns:** The codebase already had `serde_json` as a dependency and `escape_sql_string()` as a pattern to follow, making fixes natural extensions.
4. **Quick coordinator catch:** The ESCAPE clause had a double-backslash bug (`'\\\\'` producing 2 chars instead of 1) that was caught during coordinator review before shipping.

### What Could Be Improved

1. **Permission test gap:** The initial implementation missed unit tests for Feature 3 despite acceptance criteria requiring them. The quality review caught this.
2. **Inconsistent escaping patterns:** `find_sessions_for_user()` still uses inline `.replace()` instead of the shared escape utility. Minor but worth cleaning up.

### Follow-Up Items

- **P3:** Migrate `find_sessions_for_user()` in abort.rs to use `escape_sql_string()` instead of inline `.replace()`
- **P3:** Add `tags: ['v*']` trigger to CI workflow
- **P3:** Align permission error message format with specification's labeled multi-line layout
- **P2:** Add TLS/SSL connection configuration (deferred from security audit)
- **P3:** Add REPL history file permission enforcement

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Sprint review | Sprint Coordinator |
