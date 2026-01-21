# TC-SECURITY-002: Config File Permission Warning - 0644 Allowed

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-SECURITY-002 |
| **Title** | Config File Permission Warning - 0644 Allowed |
| **Category** | Security |
| **Priority** | High |
| **Feature** | Sprint 17 - Security Validation (Context) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that tq treats configuration file permissions differently from password file permissions: config files with 0644 permissions issue a **warning** but are still **allowed** (not rejected), because config files should not contain passwords.

**PURPOSE:** Contrast with TC-SECURITY-001 (password files enforced) to validate different security policies.

## Scope

This test validates:
- Config file with 0644 permissions issues **warning** (not error)
- Config file is still **processed** (profiles loaded)
- Warning explains recommendation but doesn't block usage
- Exit code is 0 (success with warning)
- Different behavior from password files (which are rejected)

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- Ability to create config files and set permissions (Linux/macOS)
- Config file contains profiles (to verify file is actually processed)

## Test Procedure

### Step 1: Create config file with insecure permissions
```bash
# Create config directory
mkdir -p ~/.tq-test

# Create config with profile
cat > ~/.tq-test/config.toml <<'EOF'
[defaults]
format = "table"

[profiles.dev]
host = "dev.example.com"
port = 1025
database = "development"
user = "alice"
password_file = "~/.tq/passwords/dev"
EOF

# Set permissions to 0644 (world-readable)
chmod 0644 ~/.tq-test/config.toml

# Verify permissions
ls -l ~/.tq-test/config.toml
# Should show: -rw-r--r--
```

### Step 2: Use config file with tq profiles command
```bash
# Point to test config (method depends on implementation)
# Assuming TQ_CONFIG_DIR or similar
tq profiles 2>&1
```

### Step 3: Verify exit code is 0 (success)
```bash
tq profiles > /dev/null 2>&1
echo $?
```

### Step 4: Capture warning and output
```bash
# Capture both stderr (warning) and stdout (profile list)
tq profiles > /tmp/config-perm-stdout.txt 2> /tmp/config-perm-stderr.txt

echo "=== STDERR (Warning) ==="
cat /tmp/config-perm-stderr.txt

echo "=== STDOUT (Profile List) ==="
cat /tmp/config-perm-stdout.txt
```

### Step 5: Verify warning content
```bash
# Check for warning keywords
grep -i "warning" /tmp/config-perm-stderr.txt
grep "0644" /tmp/config-perm-stderr.txt
grep "0600" /tmp/config-perm-stderr.txt
grep -i "recommendation" /tmp/config-perm-stderr.txt
```

### Step 6: Verify config was still processed
```bash
# Profile should be listed (proves config was loaded)
grep "dev" /tmp/config-perm-stdout.txt
grep "dev.example.com" /tmp/config-perm-stdout.txt
```

### Step 7: Cleanup
```bash
rm -rf ~/.tq-test
rm -f /tmp/config-perm-*.txt
```

## Expected Results

### Step 1 Output:
```
-rw-r--r--  1 user  group  200 Jan 21 10:00 ~/.tq-test/config.toml
```
(Permissions are 0644)

### Step 2 Output:
**stderr:** Warning message
```
Warning: Configuration file ~/.tq-test/config.toml has permissive permissions (0644)
Recommendation: chmod 0600 ~/.tq-test/config.toml
```

**stdout:** Profile listing (config was processed)
```
Available profiles (from ~/.tq-test/config.toml):

  dev
    Host:     dev.example.com:1025
    Database: development
    User:     alice
    Logmech:  TD2

Use: tq --profile <name> <command>
```

**Key Differences from Password File Error:**
- Says "Warning" not "Error"
- Says "Recommendation" not "Required"
- Config is still processed (profile listed)
- Exit code is 0 (success)

### Step 3 Output:
```
0
```
(Success exit code - warning doesn't prevent execution)

### Step 4 Output:
- stderr contains warning message
- stdout contains profile listing
- Both outputs present (proves config was processed despite warning)

### Step 5 Output:
All warning keywords found:
- "Warning" or "warning"
- "0644" (current permissions)
- "0600" (recommended permissions)
- "Recommendation" or "recommend"

### Step 6 Output:
Profile "dev" found in output with host "dev.example.com".
**CRITICAL:** This proves config file was loaded and processed, not rejected.

### Step 7 Output:
Cleanup successful.

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 0 (success with warning)
- ✅ Warning message issued (to stderr)
- ✅ Warning uses "Warning" not "Error"
- ✅ Warning says "Recommendation" not "Required"
- ✅ Config file is **processed** (profiles loaded)
- ✅ Profile listing works normally
- ✅ Warning mentions current and recommended permissions
- ✅ **Different behavior from password files** (allowed vs rejected)

**FAIL if:**
- ❌ Exit code is non-zero (error)
- ❌ Config file is rejected (should only warn)
- ❌ No warning issued (permissive permissions should trigger warning)
- ❌ Error instead of warning (too strict for config files)
- ❌ Config not processed (profiles not loaded)
- ❌ Warning unclear or missing permission recommendations

## Security Policy Comparison

| File Type | Security Policy | Insecure Permissions | Behavior | Exit Code |
|-----------|----------------|----------------------|----------|-----------|
| **Password File** | ENFORCED | 0644, 0666, etc. | **Rejected** (ERROR) | Non-zero |
| **Config File** | RECOMMENDED | 0644 | **Allowed** (WARNING) | 0 |

**Rationale for Different Policies:**
- **Password files** contain credentials → **enforce** 0600 (security critical)
- **Config files** should NOT contain passwords → **warn** about 0644 (best practice, not critical)
- Config files in shared environments may legitimately be team-readable
- tq specification prohibits inline passwords in config files

### Validation Checklist

**Config File Policy (This Test):**
- [ ] 0644 config file issues warning
- [ ] Warning goes to stderr
- [ ] Warning uses "Warning" not "Error"
- [ ] Warning uses "Recommendation" not "Required"
- [ ] Config is still processed
- [ ] Exit code is 0 (success)

**Contrast with Password File Policy (TC-SECURITY-001):**
- [ ] 0644 password file issues **error** (not warning)
- [ ] Error goes to stderr
- [ ] Error uses "Error" not "Warning"
- [ ] Error uses "Required" not "Recommendation"
- [ ] Password file is **rejected**
- [ ] Exit code is non-zero (failure)

**Policy Consistency:**
- [ ] Both tests mention 0600 as correct permissions
- [ ] Both provide `chmod 0600` fix command
- [ ] Different enforcement reflects different security risks

## Edge Cases

### Variation 1: Config with 0600 permissions (secure)
```bash
chmod 0600 ~/.tq-test/config.toml
tq profiles 2>&1
```

**Expected:** No warning, profiles listed normally.

### Variation 2: Config with 0777 permissions (very insecure)
```bash
chmod 0777 ~/.tq-test/config.toml
tq profiles 2>&1
```

**Expected:** Warning issued (same as 0644), config still processed.

### Variation 3: Config contains inline password (violation)
```toml
[profiles.bad]
host = "test.com"
password = "INLINE_PASSWORD"  # VIOLATION of specification
```

**Note:** Specification prohibits inline passwords in config files. If implementation supports it (for migration), inline passwords should trigger additional security warning beyond file permissions.

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Warning Message:**

**Config Processing:**
- [ ] Config was loaded
- [ ] Profiles listed correctly

**Warning Quality:**
- [ ] Uses "Warning" not "Error"
- [ ] Uses "Recommendation" not "Required"
- [ ] Provides chmod fix command

**Policy Validation:**
- [ ] Different from password file enforcement
- [ ] Rationale clear (config ≠ passwords)

**Issues Found:**

## Notes

- Config files should never contain passwords (use password_file field instead)
- Warning is appropriate: encourages security best practice without blocking
- Some environments legitimately need team-readable configs
- Enforcement for password files, recommendation for config files

**Why Not Enforce Config Permissions?**
1. Config files contain metadata (hosts, usernames) not secrets
2. Shared teams may need readable configs
3. Specification prohibits inline passwords in configs
4. Over-enforcement hurts usability for low-risk files

**User Guidance:**
- Config files with profiles: Use `password_file` field (enforced at 0600)
- Config files themselves: 0600 recommended but 0644 acceptable
- Never put passwords in config files (even with 0600)

**Related Test Cases:**
- TC-SECURITY-001: Password file 0644 rejected (contrasts with this test)
- TC-SECURITY-003: Security check ordering (validates check-before-read)
- TC-PROFILES-001: Profile listing (validates config loading works)

**Specification References:**
- `docs/builder/detailed-specifications/configuration.md` v2.1.0 §7.3.4 (lines 175-195)
- `docs/builder/sprints/sprint-17-planning.md` Context for security changes
