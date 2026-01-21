# TC-SECURITY-001: Password File Permission Enforcement - 0644 Rejected

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-SECURITY-001 |
| **Title** | Password File Permission Enforcement - 0644 Rejected |
| **Category** | Security |
| **Priority** | Critical |
| **Feature** | Sprint 17 - Password File Permission Enforcement (P1) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that tq **enforces** (not just warns) password file permission requirements by **rejecting** password files with permissions more permissive than 0600, and provides clear error message with fix command.

**CRITICAL CHANGE:** Sprint 17 changes behavior from WARNING to ERROR for insecure password files.

## Scope

This test validates:
- Password file with 0644 permissions is **rejected** (not just warned)
- Error message clearly explains security risk
- Error message provides fix command (`chmod 0600`)
- tq refuses to read insecure password file content
- Exit code is non-zero (error, not success)
- Behavior change from Sprint 16 (was warning, now error)

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- Ability to create files and set permissions (Linux/macOS)
- **Note:** Windows has different permission model - test on POSIX systems only

## Test Procedure

### Step 1: Create password file with insecure permissions
```bash
# Create temp password file
mkdir -p /tmp/tq-test-passwords
echo "testpassword123" > /tmp/tq-test-passwords/insecure

# Set insecure permissions (world-readable)
chmod 0644 /tmp/tq-test-passwords/insecure

# Verify permissions
ls -l /tmp/tq-test-passwords/insecure
# Should show: -rw-r--r--
```

### Step 2: Attempt to use insecure password file
```bash
# Try to use password file with tq ping
tq -l "testuser@testhost:1025/testdb" \
   --password-file /tmp/tq-test-passwords/insecure \
   ping
```

### Step 3: Verify exit code is non-zero
```bash
tq -l "testuser@testhost:1025/testdb" \
   --password-file /tmp/tq-test-passwords/insecure \
   ping > /dev/null 2>&1
echo $?
```

### Step 4: Capture error message
```bash
tq -l "testuser@testhost:1025/testdb" \
   --password-file /tmp/tq-test-passwords/insecure \
   ping 2> /tmp/password-perm-error.txt

cat /tmp/password-perm-error.txt
```

### Step 5: Verify error message content
```bash
# Check for key phrases
grep -i "insecure permissions" /tmp/password-perm-error.txt
grep "0644" /tmp/password-perm-error.txt  # Current permissions mentioned
grep "0600" /tmp/password-perm-error.txt  # Required permissions mentioned
grep "chmod 0600" /tmp/password-perm-error.txt  # Fix command provided
```

### Step 6: Verify file content was NOT read
```bash
# Check that error is about permissions, not connection
# If file content was read, we'd see connection error instead
grep -i "connection" /tmp/password-perm-error.txt
# Should NOT find connection error (file wasn't read)
```

### Step 7: Cleanup
```bash
rm -rf /tmp/tq-test-passwords
rm -f /tmp/password-perm-error.txt
```

## Expected Results

### Step 1 Output:
```
-rw-r--r--  1 user  group  16 Jan 21 10:00 /tmp/tq-test-passwords/insecure
```
(Permissions are 0644 - world readable)

### Step 2 Output:
Error message on stderr:
```
Error: Password file has insecure permissions: /tmp/tq-test-passwords/insecure
Current permissions: 0644 (readable by group and others)
Required permissions: 0600 (owner read-write only)

Security risk: Password file is readable by other users

Fix: chmod 0600 /tmp/tq-test-passwords/insecure
```

**Message Requirements:**
- Clearly states "insecure permissions"
- Shows file path
- Shows current permissions (0644) with explanation
- Shows required permissions (0600) with explanation
- Explains security risk
- Provides exact fix command that user can copy-paste

### Step 3 Output:
```
1
```
(Non-zero exit code - error occurred)

### Step 4 Output:
Full error message captured (same as Step 2).

### Step 5 Output:
All key phrases found:
- "insecure permissions" present
- "0644" present (current)
- "0600" present (required)
- "chmod 0600" present (fix command)

### Step 6 Output:
No connection error found - proves file content was never read.
**CRITICAL:** If connection error appears, test FAILS (means password was read before permission check).

### Step 7 Output:
Cleanup successful.

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is non-zero (command fails)
- ✅ Error message states "insecure permissions"
- ✅ Error message shows current permissions (0644)
- ✅ Error message shows required permissions (0600)
- ✅ Error message explains security risk
- ✅ Error message provides fix command (`chmod 0600 <file>`)
- ✅ File content was NOT read (no connection attempt)
- ✅ Error goes to stderr (not stdout)
- ✅ **BEHAVIOR CHANGE: Rejects file (not just warns)**

**FAIL if:**
- ❌ Exit code is 0 (success)
- ❌ Only warning shown (Sprint 16 behavior - should be error now)
- ❌ No error message about permissions
- ❌ Fix command not provided
- ❌ File content was read (connection error occurs)
- ❌ Error message unclear or unhelpful
- ❌ **CRITICAL: Password file accepted with 0644 permissions**

## Security Validation Checklist

### Permission Enforcement
- [ ] 0644 file is **rejected** (error, not warning)
- [ ] tq exits immediately with error
- [ ] File content is never read
- [ ] No password exposure in error message

### Error Message Quality
- [ ] States "insecure permissions" or similar
- [ ] Shows file path
- [ ] Shows current permissions with explanation (0644 = readable by others)
- [ ] Shows required permissions with explanation (0600 = owner only)
- [ ] Explains security risk clearly
- [ ] Provides exact fix command

### Behavior vs Sprint 16
- [ ] Sprint 16: Warned but allowed file
- [ ] Sprint 17: **Rejects file with error**
- [ ] This is a breaking change (documented)

## Test Variations

### Variation 1: Test various insecure permissions
```bash
# Test different insecure permission values
for perms in 0666 0664 0644 0655 0777; do
    chmod $perms /tmp/tq-test-passwords/insecure
    tq --password-file /tmp/tq-test-passwords/insecure ping 2>&1
    echo "Permissions $perms - Exit code: $?"
done
```

**Expected:** All insecure permissions (not 0600) should be rejected.

### Variation 2: Verify 0600 is still accepted (regression test)
**Note:** This should be a separate test case for clarity (positive test).

### Variation 3: Test with profile using password_file
```toml
[profiles.test]
host = "testhost"
password_file = "/tmp/tq-test-passwords/insecure"  # 0644 file
```

```bash
tq --profile test ping
```

**Expected:** Same error - permission check happens whether file specified via flag or profile.

### Variation 4: Test error when used with config file
```bash
# If config file also has 0644 permissions
chmod 0644 ~/.tq/config.toml

# And password file also has 0644
tq --password-file /tmp/tq-test-passwords/insecure ping
```

**Expected:** Password file error (not config file warning) - password files are enforced, config files only warn.

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Error Message:**

**Permission Check:**
- [ ] File rejected (not just warned)
- [ ] File content never read

**Error Message Quality:**
- [ ] Clear and actionable
- [ ] Shows current and required permissions
- [ ] Provides fix command

**Breaking Change Validation:**
- [ ] Behavior changed from Sprint 16 (warning → error)

**Issues Found:**

## Notes

- **BREAKING CHANGE:** Sprint 17 changes from warning to error
- This improves security but may impact existing users with 0644 password files
- Users will see error and must fix permissions to continue
- Error message quality is critical - must be clear and helpful
- Fix command should be copy-pasteable: `chmod 0600 <file>`

**Rationale for Enforcement (from specification):**
- Passwords in world-readable files are critical security vulnerability
- tq prioritizes security over convenience
- Warning was insufficient (users ignored it)
- Error forces proper security configuration

**Platform Considerations:**
- Linux/macOS: Standard POSIX permissions (this test)
- Windows: Different permission model (may need separate test or skip)

**Related Test Cases:**
- TC-SECURITY-002: Config file 0644 only warns (different behavior)
- TC-SECURITY-003: Security check happens before file read (validates order)

**Specification References:**
- `docs/builder/detailed-specifications/configuration.md` v2.1.0 §7.6.3 (lines 401-423)
- `docs/builder/sprints/sprint-17-planning.md` Feature 3 (lines 92-107)
