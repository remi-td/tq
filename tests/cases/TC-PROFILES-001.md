# TC-PROFILES-001: List Profiles from Config File

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-PROFILES-001 |
| **Title** | List Profiles from Config File |
| **Category** | Functionality |
| **Priority** | High |
| **Feature** | Sprint 17 - Profile Listing Command (P1) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that `tq profiles` command successfully lists all available connection profiles from the configuration file, displaying profile names and connection metadata while never exposing passwords or password file paths.

## Scope

This test validates:
- `tq profiles` command execution with valid config file
- All profiles listed with names
- Connection metadata displayed (host, port, database, user, logmech)
- **SECURITY CRITICAL**: Passwords and password_file paths NEVER shown
- Exit code 0 on success
- Output formatting and readability

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- Test configuration file created with multiple profiles
- No database connection required (metadata only)

## Test Procedure

### Step 1: Create test configuration file
```bash
# Create test config directory
mkdir -p ~/.tq-test

# Create config file with multiple profiles
cat > ~/.tq-test/config.toml <<'EOF'
[defaults]
format = "table"
timing = true

[profiles.dev]
host = "dev.example.com"
port = 1025
database = "development"
user = "alice"
logmech = "TD2"
password_file = "/secret/passwords/dev"

[profiles.prod]
host = "prod.example.com"
port = 1025
database = "production"
user = "bob"
logmech = "LDAP"
password_file = "/secret/passwords/prod"

[profiles.local]
host = "localhost"
port = 1025
database = "testdb"
user = "dbc"
logmech = "TD2"
# No password_file - will prompt interactively
EOF
```

### Step 2: Execute tq profiles command
```bash
# Point to test config (assuming TQ_CONFIG_DIR env var or use actual location)
tq profiles  # May need to use actual config location
```

### Step 3: Verify exit code
```bash
tq profiles > /dev/null 2>&1
echo $?
```

### Step 4: Validate output content
```bash
tq profiles > /tmp/profiles-output.txt 2>&1
cat /tmp/profiles-output.txt

# Check for each profile
grep "dev" /tmp/profiles-output.txt
grep "prod" /tmp/profiles-output.txt
grep "local" /tmp/profiles-output.txt
```

### Step 5: Security validation - Ensure no password exposure
```bash
tq profiles > /tmp/profiles-output.txt 2>&1

# These should NOT appear in output
grep -i "password" /tmp/profiles-output.txt  # Should not find passwords
grep "/secret/passwords" /tmp/profiles-output.txt  # Should not find password file paths
grep "password_file" /tmp/profiles-output.txt  # Field name should not appear
```

## Expected Results

### Step 1 Output:
Configuration file created successfully with 3 profiles.

### Step 2 Output:
```
Available profiles (from ~/.tq/config.toml):

  dev
    Host:     dev.example.com:1025
    Database: development
    User:     alice
    Logmech:  TD2

  prod
    Host:     prod.example.com:1025
    Database: production
    User:     bob
    Logmech:  LDAP

  local
    Host:     localhost:1025
    Database: testdb
    User:     dbc
    Logmech:  TD2

Use: tq --profile <name> <command>
```

**Output Format Requirements:**
- Profile names clearly displayed (dev, prod, local)
- Connection metadata indented under profile name
- Host and port combined (e.g., "dev.example.com:1025")
- Database, user, logmech shown
- Helpful usage hint at bottom
- **NO password or password_file information**

### Step 3 Output:
```
0
```
(Success exit code)

### Step 4 Output:
All three profiles found in output:
- "dev" appears with dev.example.com
- "prod" appears with prod.example.com
- "local" appears with localhost

### Step 5 Output:
**CRITICAL SECURITY VALIDATION:**
- `grep -i "password"` should return **no matches** (or only in usage hint)
- `grep "/secret/passwords"` should return **no matches**
- `grep "password_file"` should return **no matches**

If ANY of these searches find matches (other than in usage hint), the test **FAILS**.

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 0
- ✅ All profiles listed (dev, prod, local)
- ✅ Profile metadata displayed: host, port, database, user, logmech
- ✅ Output is readable and well-formatted
- ✅ Usage hint provided (`tq --profile <name> <command>`)
- ✅ **SECURITY: No passwords in output**
- ✅ **SECURITY: No password_file paths in output**
- ✅ **SECURITY: No password_file field names in output**

**FAIL if:**
- ❌ Exit code is non-zero
- ❌ Any profile missing from output
- ❌ Metadata missing or incomplete
- ❌ **CRITICAL: Any password information displayed**
- ❌ **CRITICAL: Any password_file path displayed**
- ❌ Output unreadable or poorly formatted
- ❌ No usage guidance provided

## Content Validation Checklist

### Profile Listing
- [ ] All 3 profiles listed: dev, prod, local
- [ ] Profile names are clear and prominent
- [ ] Each profile has metadata section

### Metadata Display (for each profile)
- [ ] Host shown (e.g., dev.example.com)
- [ ] Port shown (e.g., 1025) or combined with host
- [ ] Database shown (e.g., development)
- [ ] User shown (e.g., alice)
- [ ] Logmech shown (e.g., TD2, LDAP)

### Security Validation (CRITICAL)
- [ ] No password values in output
- [ ] No password_file paths in output (e.g., /secret/passwords/dev)
- [ ] No password_file field name in metadata
- [ ] No inline password values (if any profiles had them)
- [ ] Profile "local" without password_file does not show "password_file: (none)" or similar

### Format and Usability
- [ ] Output is readable (not machine format)
- [ ] Indentation or spacing makes structure clear
- [ ] Profile names distinguishable from metadata
- [ ] Usage hint at end: "Use: tq --profile <name> <command>"
- [ ] Config file path shown in header (helpful context)

## Security Test Variations

### Variation 1: Profile with inline password (if supported)
**If tq supports inline passwords in profiles (discouraged but possible):**
```toml
[profiles.insecure]
host = "test.example.com"
user = "testuser"
password = "SECRETPASSWORD123"  # Never do this!
```

**Expected:** `tq profiles` should NOT display "SECRETPASSWORD123" or the password field.

### Variation 2: Profile with complex password file path
```toml
[profiles.test]
password_file = "/very/secret/path/with spaces/password.txt"
```

**Expected:** Path should NOT appear in output.

### Variation 3: Defaults section in config
**Test config with defaults does not affect profile listing:**
```toml
[defaults]
format = "json"
```

**Expected:** Defaults section should not appear in profile listing (profiles only).

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Profiles Listed:**
- [ ] dev
- [ ] prod
- [ ] local

**Metadata Displayed:**
- [ ] Host/Port
- [ ] Database
- [ ] User
- [ ] Logmech

**Security Validation:**
- [ ] No passwords in output
- [ ] No password_file paths in output
- [ ] Security check PASSED

**Output Quality:**

**Issues Found:**

## Notes

- **Security is paramount** - Any password exposure is a critical failure
- This command helps users discover available profiles without editing config file
- Output should be human-readable (not JSON/machine format)
- Profiles without password_file should not show "missing" or "not set" message
- Config file path in header helps users know which config is being read

**Related Test Cases:**
- TC-PROFILES-002: No config file error handling
- TC-PROFILES-003: Config exists but no profiles
- TC-SECURITY-002: Config file permission warnings (different from password files)

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.5 (lines 272-355)
- `docs/builder/detailed-specifications/configuration.md` v2.1.0 §7.4.4 (lines 259-293)
- `docs/builder/sprints/sprint-17-planning.md` Feature 4 (lines 111-127)
