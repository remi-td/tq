# Security Requirements

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Credential Security](#101-credential-security)
2. [SQL Injection Prevention](#102-sql-injection-prevention)
3. [Connection Security](#103-connection-security)
4. [Data Privacy](#104-data-privacy)
5. [Supply Chain Security](#105-supply-chain-security)
6. [Security Hardening](#106-security-hardening)

---

## 10.1 Credential Security

### 10.1.1 Never Log Credentials

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

### 10.1.2 Prevent Credential Leaks

**Avoid**:
- ❌ Passwords in CLI arguments: `tq --password secret123`
- ❌ Passwords in environment (minimize): `TQ_PASSWORD=secret`
- ❌ World-readable config files

**Prefer**:
- ✅ Password files with `0600` permissions
- ✅ OS keyring integration
- ✅ Interactive prompts
- ✅ External credential providers

### 10.1.3 File Permissions

```bash
# Check config file permissions
$ ls -la ~/.config/tq/config.toml
-rw-------  1 alice  staff  256 Jan 15 10:30 config.toml  # ✅ Good (0600)

# Warn on unsafe permissions
$ chmod 0644 ~/.config/tq/config.toml
$ tq query "SELECT 1"
Warning: Config file ~/.config/tq/config.toml has unsafe permissions (644)
Expected: 0600 (owner read/write only)
Fix: chmod 0600 ~/.config/tq/config.toml

Continuing anyway (use --strict to abort)...
```

## 10.2 SQL Injection Prevention

### 10.2.1 Parameterized Queries (Future)

```bash
# Safe: Use placeholders
tq query "SELECT * FROM users WHERE id = ?" --param 123

# Unsafe: String concatenation
# DON'T ENABLE THIS:
USER_INPUT="1 OR 1=1"
tq query "SELECT * FROM users WHERE id = $USER_INPUT"
```

### 10.2.2 Input Validation

**Current Scope**: `tq` passes SQL directly to Teradata
**Mitigation**: Document security best practices
**Future**: Add `--safe` mode that validates common injection patterns

## 10.3 Connection Security

### 10.3.1 TLS/SSL Encryption (Future)

```bash
# Enforce encrypted connections
tq --ssl-mode require query "SELECT 1"

# Verify server certificate
tq --ssl-mode verify-full --ssl-ca-file ca-cert.pem query "SELECT 1"
```

### 10.3.2 Connection Timeout

Prevent hanging on unresponsive hosts:

```bash
# Default: 30s timeout
tq query "SELECT 1"

# Custom timeout
tq --timeout 5s ping
```

## 10.4 Data Privacy

### 10.4.1 Redact Sensitive Data (Future)

```bash
# Mask sensitive columns in logs/errors
tq query "SELECT ssn, name FROM users" --redact ssn
```

### 10.4.2 Audit Logging

For compliance environments:

```bash
# Log all queries
export TQ_AUDIT_LOG=/var/log/tq-audit.log
tq query "SELECT * FROM sensitive_table"

# Audit log format (JSON):
{"timestamp":"2024-01-15T10:30:00Z","user":"alice","host":"myhost","database":"prod","query":"SELECT * FROM sensitive_table","rows":42}
```

## 10.5 Supply Chain Security

### 10.5.1 Dependency Auditing

```bash
# CI/CD pipeline
cargo audit --deny warnings
cargo outdated --exit-code 1
```

### 10.5.2 Binary Verification

```bash
# Provide checksums for releases
$ sha256sum tq-1.0.0-linux-x86_64.tar.gz
a3f2b1c... tq-1.0.0-linux-x86_64.tar.gz

# Sign releases with GPG
$ gpg --verify tq-1.0.0-linux-x86_64.tar.gz.sig
```

## 10.6 Security Hardening

### 10.6.1 Principle of Least Privilege

**Documentation**:
- Recommend read-only accounts for analysts
- Separate accounts for admin vs. query users
- Use service accounts for automation

### 10.6.2 Security Defaults

- ✅ Secure by default: warn on insecure configs
- ✅ Fail closed: abort on auth errors
- ✅ No auto-retry: prevent account lockouts
- ✅ Clear secrets: zero memory after use

---
