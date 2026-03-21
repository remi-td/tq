# Security Requirements

## Table of Contents

1. [Credential Security](#credential-security)
2. [SQL Injection Prevention](#sql-injection-prevention)
3. [Connection Security](#connection-security)
4. [Data Privacy](#data-privacy)
5. [Supply Chain Security](#supply-chain-security)
6. [Security Hardening](#security-hardening)

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

```bash
# Check config file permissions
$ ls -la ~/.config/tq/config.toml
-rw-------  1 alice  staff  256 Jan 15 10:30 config.toml  # Good (0600)

# Warn on unsafe permissions
$ chmod 0644 ~/.config/tq/config.toml
$ tq query "SELECT 1"
Warning: Config file ~/.config/tq/config.toml has unsafe permissions (644)
Expected: 0600 (owner read/write only)
Fix: chmod 0600 ~/.config/tq/config.toml

Continuing anyway (use --strict to abort)...
```

## SQL Injection Prevention

### Input Validation

**Current Scope**: `tq` passes SQL directly to Teradata
**Mitigation**: Document security best practices

## Connection Security

### Connection Timeout

Prevent hanging on unresponsive hosts:

```bash
# Default: 30s timeout
tq query "SELECT 1"

# Custom timeout
tq --timeout 5s ping
```

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

## Supply Chain Security

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
