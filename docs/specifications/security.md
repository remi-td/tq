# Security Requirements

## Table of Contents

1. [Credential Security](#credential-security)
2. [SQL Injection Prevention](#sql-injection-prevention)
3. [Output Integrity](#output-integrity)
4. [Connection Security](#connection-security)
5. [Data Privacy](#data-privacy)
6. [Supply Chain Security](#supply-chain-security)
7. [CI/CD Security Workflow](#cicd-security-workflow)
8. [Security Hardening](#security-hardening)

---

## Credential Security

### Never Log Credentials

**Implementation**:
- Use `secrecy::Secret<String>` for passwords
- Redact in logs: `user@host:****/db`
- Sanitize error messages
- Clear memory on drop

**Example**:
```rust
// Good
log::debug!("Connecting to {}", sanitize_connection_string(&conn));

// Bad
log::debug!("Connecting with: {}", raw_connection_string);
```

### Prevent Credential Leaks

**Avoid**:
- Passwords in CLI arguments: `tq --password secret123`
- Passwords in environment (minimize): `TQ_PASSWORD=secret`
- World-readable config files

**Prefer**:
- Password files with `0600` permissions
- OS keyring integration
- Interactive prompts
- External credential providers

### File Permissions

#### REQ-SEC-002: Password File Permission Enforcement

**Requirement:** All password files (both the default `~/.tq_passwords` and any profile-level `password_file` entries) MUST have Unix file permissions of `0600` (owner read/write only). If permissions are more permissive, `tq` MUST reject the file with an error and exit — it must not warn and continue.

**Rationale:** Merely warning and continuing undermines the security guarantee. An attacker who can read a group- or world-readable password file can silently steal credentials. Hard rejection forces the user to make a conscious, corrective action.

**Behavior:**

```bash
# Correctly secured password file — tq proceeds normally
$ ls -la ~/.tq_passwords
-rw-------  1 alice  staff  64 Jan 15 10:30 .tq_passwords
$ tq query "SELECT 1"
 1
---
 1
```

```bash
# Insecure password file — tq aborts
$ chmod 0644 ~/.tq_passwords
$ tq query "SELECT 1"
Error: Insecure password file permissions

File: ~/.tq_passwords
Current permissions: 644
Required permissions: 600 (owner read/write only)

Fix:
  chmod 0600 ~/.tq_passwords
```

**Affected paths:**
- Default password file: `~/.tq_passwords`
- Profile `password_file` entries in `~/.tq/config.toml`
- Any file referenced via `--password-file`

**Platform scope:** This requirement applies on Unix platforms (Linux, macOS). On Windows, where POSIX permissions do not exist, `tq` SHOULD apply equivalent ACL checks where feasible, and SHOULD warn rather than error when equivalent enforcement is not available.

**Error message fields:**
| Field | Value |
|-------|-------|
| File path | Exact path that was checked |
| Current permissions | Octal value of actual permissions |
| Required permissions | `600` |
| Fix instruction | `chmod 0600 <path>` |

---

## SQL Injection Prevention

### Input Validation

**Current Scope**: `tq` passes SQL directly to Teradata. User SQL is not validated or rewritten.

**Mitigation**: Parameterized queries should be used for all internally-generated SQL that incorporates user-supplied values (host names, session identifiers, profile names, etc.).

#### REQ-SEC-003: SQL LIKE Pattern Wildcard Escaping

**Requirement:** Any internally-generated SQL statement that uses a `LIKE` clause with user-supplied input MUST escape the LIKE wildcard characters `%` and `_` before interpolation. Single quotes MUST also be escaped. A dedicated escaping utility function MUST be used — ad-hoc escaping at call sites is not permitted.

**Rationale:** Unescaped wildcards in LIKE patterns expand the query match beyond the intended scope. For example, a hostname of `%` would match all sessions in an abort command, causing unintended mass termination.

**Escaping rules:**

| Input character | Escaped output |
|-----------------|----------------|
| `%`             | `\%`           |
| `_`             | `\_`           |
| `'`             | `''`           |
| `\`             | `\\`           |

**Examples:**

```
Input: "db-host-01"   → LIKE '%db-host-01%'       (no change needed)
Input: "host_%"       → LIKE '%host\_\%%'          (wildcards escaped)
Input: "it's"         → LIKE '%it''s%'             (quote escaped)
Input: "100%_done"    → LIKE '%100\%\_done%'       (both escaped)
```

**Usage pattern** (pseudocode):
```rust
// Good — always use the escape utility
let safe_pattern = escape_sql_like(user_input);
let sql = format!("WHERE hostname LIKE '%{}%' ESCAPE '\\'", safe_pattern);

// Bad — ad-hoc or missing escaping
let sql = format!("WHERE hostname LIKE '%{}%'", user_input);
```

**Scope:** This requirement applies to all `tq`-generated SQL, including but not limited to:
- Session search in abort commands
- Metadata queries filtering by database or object name

---

## Output Integrity

### Structured Serialization

#### REQ-SEC-001: JSON Output Must Use Structured Serialization

**Requirement:** All JSON output produced by `tq` — including query results, error responses, and connection configuration payloads — MUST be generated using a dedicated serialization library (e.g., `serde_json`). Manual JSON construction via string formatting (`format!()`, string concatenation, or template strings) is prohibited.

**Rationale:** Manual string formatting cannot reliably escape all special characters (`"`, `\`, control characters such as `\n`, `\t`, `\r`, Unicode escapes). Malformed JSON sent to automated consumers (scripts, agents, pipelines) causes silent data corruption or parse failures. Malformed JSON in connection payloads may produce unpredictable driver behavior.

**Affected surfaces:**
- Query result rows in `--format json` mode
- Error messages serialized to JSON (e.g., `--format json` with an error condition)
- Connection configuration objects passed to the Teradata driver
- Any internal data structure serialized to a file or stdout

**Special characters that must be correctly escaped:**

| Character | Description           | JSON escape |
|-----------|-----------------------|-------------|
| `"`       | Double quote          | `\"`        |
| `\`       | Backslash             | `\\`        |
| `\n`      | Newline               | `\n`        |
| `\r`      | Carriage return       | `\r`        |
| `\t`      | Tab                   | `\t`        |
| U+0000–U+001F | Control characters | `\uXXXX`   |

**Example — error output with special characters:**

```bash
# Error message contains a tab and newline in the DB error text
$ tq --format json query "INVALID SQL"
{
  "error": "SQL syntax error",
  "message": "Unexpected token near\there",
  "code": 3706
}
```

The `\t` in the error text must appear as a JSON string escape, not as a literal tab character that would produce invalid JSON.

**Example — connection config with special characters in password:**

```
Password:  P@ss"word\1
Correct JSON field:  "password": "P@ss\"word\\1"
Incorrect (broken):  "password": "P@ss"word\1"   ← parser error
```

---

## Connection Security

### Connection Timeout

Prevent hanging on unresponsive hosts:

```bash
# Default: 30s timeout
tq query "SELECT 1"

# Custom timeout
tq --timeout 5s ping
```

---

## Data Privacy

### Audit Logging

For compliance environments:

```bash
# Log all queries
export TQ_AUDIT_LOG=/var/log/tq-audit.log
tq query "SELECT * FROM sensitive_table"

# Audit log format (JSON):
{"timestamp":"2024-01-15T10:30:00Z","user":"alice","host":"myhost","database":"prod","query":"SELECT * FROM sensitive_table","rows":42}
```

---

## Supply Chain Security

### REQ-SEC-004: Git Dependencies Must Be Pinned to a Specific Revision

**Requirement:** Any dependency referenced by a Git URL in `Cargo.toml` MUST be pinned to a specific commit revision via the `rev` field. Tracking a branch name or tag alone (which can be silently force-pushed) is not sufficient.

**Rationale:** An unpinned Git dependency fetches the latest commit on the default branch on each `cargo update`. A compromised or accidental upstream commit would be silently incorporated into the next build. Pinning to a specific `rev` makes the exact dependency state auditable and reproducible.

**Correct form:**
```toml
# Good — pinned to a specific immutable commit
[dependencies]
teradatarustapi = { git = "https://github.com/Teradata/teradatarustapi", rev = "abc1234" }

# Bad — tracking HEAD, can change without notice
teradatarustapi = { git = "https://github.com/Teradata/teradatarustapi" }

# Bad — tag alone can be force-pushed
teradatarustapi = { git = "https://github.com/Teradata/teradatarustapi", tag = "v1.0" }
```

**Process:** When upgrading a pinned Git dependency, the new revision MUST be verified (changelog reviewed, build tested) before committing the updated `rev` value.

### Dependency Auditing

```bash
# CI/CD pipeline
cargo audit --deny warnings
cargo outdated --exit-code 1
```

### Binary Verification

```bash
# Provide checksums for releases
$ sha256sum tq-1.0.0-linux-x86_64.tar.gz
a3f2b1c... tq-1.0.0-linux-x86_64.tar.gz

# Sign releases with GPG
$ gpg --verify tq-1.0.0-linux-x86_64.tar.gz.sig
```

### Third-Party Driver License Compliance

`tq` bundles the Teradata SQL driver library, which is proprietary software distributed under a separate Teradata license agreement. The following requirements ensure legal compliance:

**REQ-SEC-LICENSE-001: Installer License Gate**

The install script MUST obtain explicit user acceptance of the Teradata license before installing any files. Interactive and non-interactive modes are both supported. See `docs/specifications/cli-interface.md`, section `REQ-INSTALL-010`, for the complete installer specification.

**REQ-SEC-LICENSE-002: License File Bundling**

Every release archive MUST include `LICENSE.teradata` containing the full Teradata driver license text. This file MUST be sourced from the repository and MUST NOT be fetched remotely at install time.

**REQ-SEC-LICENSE-003: License Text in Repository**

The Teradata driver license text SHALL be stored as a file in the repository (e.g., `LICENSE.teradata` at the project root) so that it is available at build time, auditable via version control, and includable in release archives without network access.

---

## CI/CD Security Workflow

### REQ-SEC-005: Continuous Integration Must Include Security Scanning

**Requirement:** The project MUST maintain a CI workflow that runs automatically on every push to any branch and on every pull request targeting the main branch. This workflow MUST include static analysis, security auditing, and test execution as mandatory quality gates. A build that fails any of these gates MUST NOT be considered releasable.

**Rationale:** Security issues caught in CI cost dramatically less to fix than those discovered in production. Automated enforcement removes the human burden of remembering to run security checks and creates a verifiable audit trail.

#### Trigger Conditions

| Event | Branches | Purpose |
|-------|----------|---------|
| `push` | All branches | Catch regressions immediately |
| `pull_request` | Targeting `master` | Gate merges on quality |

#### Required Jobs

The CI workflow MUST define the following jobs. All jobs MUST succeed for the overall workflow to pass.

---

##### Job: `clippy` — Static Analysis

**Purpose:** Enforce Rust best practices and catch common correctness issues at compile time.

**Toolchain:** Stable Rust

**Command:**
```
cargo clippy -- -D warnings
```

**Gate behavior:** Any clippy warning MUST be treated as an error (`-D warnings`). The job fails if any warning is emitted.

**Rationale:** Clippy warnings frequently indicate latent bugs, unsafe patterns, or code that does not follow Rust idioms. Treating them as errors prevents warning accumulation over time ("warning debt").

---

##### Job: `test` — Test Suite Execution

**Purpose:** Verify that all unit tests pass.

**Toolchain:** Stable Rust

**Command:**
```
cargo test
```

**Gate behavior:** Any test failure fails the job.

**Note:** Integration tests that require a live Teradata database MUST be excluded from the CI job (these require network access to a real database server that CI cannot provide). Only unit tests and offline integration tests are expected to pass in CI.

---

##### Job: `audit` — Dependency Vulnerability Scanning

**Purpose:** Detect known security vulnerabilities in the dependency tree.

**Toolchain:** `cargo-audit` (must be installed as part of the job)

**Install step:**
```
cargo install cargo-audit
```

**Command:**
```
cargo audit
```

**Gate behavior:** Any advisory with severity `warning` or higher fails the job.

**Advisory database:** `cargo audit` fetches the [RustSec Advisory Database](https://rustsec.org/). The job should allow network access for this fetch.

**Handling false positives:** If an advisory is not applicable (e.g., the vulnerable code path is not reachable from `tq`), it MUST be explicitly ignored via an `.cargo/audit.toml` ignore entry with a comment explaining the rationale. Blanket ignores are prohibited.

---

#### Workflow Environment

| Property | Value |
|----------|-------|
| Rust toolchain | `stable` |
| Runner OS | `ubuntu-latest` |
| Caching | Cargo registry and build artifacts should be cached between runs |
| `CARGO_TERM_COLOR` | `always` (readable output in CI logs) |

#### Job Execution Strategy

Jobs SHOULD run in parallel where possible to minimize total wall-clock time:

```
push/PR event
    │
    ├── clippy ──┐
    ├── test  ───┼── all must pass ──► workflow success
    └── audit ──┘
```

Jobs are independent and have no sequencing dependencies between them.

#### Relationship to Release Workflow

The security CI workflow (`ci.yml`) is separate from the release workflow (`release.yml`). The release workflow triggers on version tags (`v*`) and produces release artifacts. The CI workflow triggers on all pushes and PRs and produces no artifacts — it is a quality gate only.

The two workflows are complementary: CI prevents regressions from entering the branch; release creates the distributable binary from a branch already validated by CI.

---

## Security Hardening

### Principle of Least Privilege

**Documentation**:
- Recommend read-only accounts for analysts
- Separate accounts for admin vs. query users
- Use service accounts for automation

### Security Defaults

- Secure by default: warn on insecure configs
- Fail closed: abort on auth errors
- No auto-retry: prevent account lockouts
- Clear secrets: zero memory after use

---
