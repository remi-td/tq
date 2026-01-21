# TC-SECURITY-003: Security Check Ordering - Permission Check Before File Read

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-SECURITY-003 |
| **Title** | Security Check Ordering - Permission Check Before File Read |
| **Category** | Security |
| **Priority** | Critical |
| **Feature** | Sprint 17 - Security Check Ordering Fix (P0) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that Sprint 17 security fix ensures `validate_password_file_permissions` is called **BEFORE** `read_to_string` in `read_password_if_needed` function, eliminating race condition where insecure file content could be read before permission check.

**SECURITY CRITICAL:** This test validates the ordering fix is implemented correctly.

## Scope

This test validates:
- Permission check happens **before** file read attempt
- Insecure file is rejected **without reading content**
- No race condition between check and read
- Behavior matches `config.rs` pattern (correct order)
- Error message indicates permission issue (not read failure)

## Prerequisites

- tq binary built and available (Sprint 17 implementation with ordering fix)
- Ability to create files and set permissions (Linux/macOS)
- Understanding of code change: `src/main.rs` function `read_password_if_needed`

## Test Procedure

### Step 1: Create password file with insecure permissions
```bash
# Create temp password file with intentional permission issue
mkdir -p /tmp/tq-test-security
echo "testpassword123" > /tmp/tq-test-security/insecure-pw

# Set insecure permissions
chmod 0644 /tmp/tq-test-security/insecure-pw

# Verify permissions are insecure
ls -l /tmp/tq-test-security/insecure-pw
# Should show: -rw-r--r--
```

### Step 2: Create unreadable password file (for contrast test)
```bash
# Create file with no read permission (to test different error)
echo "testpassword456" > /tmp/tq-test-security/unreadable-pw
chmod 0000 /tmp/tq-test-security/unreadable-pw

# Verify no read permission
ls -l /tmp/tq-test-security/unreadable-pw
# Should show: ----------
```

### Step 3: Test insecure file - should fail with permission error
```bash
tq -l "testuser@testhost:1025/testdb" \
   --password-file /tmp/tq-test-security/insecure-pw \
   ping 2> /tmp/error-insecure.txt

cat /tmp/error-insecure.txt
```

### Step 4: Test unreadable file - should fail with permission error (different message)
```bash
tq -l "testuser@testhost:1025/testdb" \
   --password-file /tmp/tq-test-security/unreadable-pw \
   ping 2> /tmp/error-unreadable.txt

cat /tmp/error-unreadable.txt
```

### Step 5: Analyze error messages to determine check order
```bash
# Insecure file (0644) should error about PERMISSIONS
grep -i "insecure permissions" /tmp/error-insecure.txt
grep -i "0644" /tmp/error-insecure.txt
echo "Found permission error: $?"

# Unreadable file (0000) should error about PERMISSIONS
# (If permission check happens first, it catches 0000 as wrong)
# (If read happens first, it would show "permission denied" for read)
grep -i "insecure permissions" /tmp/error-unreadable.txt
echo "Found permission error: $?"
```

### Step 6: Verify no file content in errors (content never read)
```bash
# Neither error should contain password from files
grep "testpassword123" /tmp/error-insecure.txt
echo "Found password 123 (FAIL if 0): $?"

grep "testpassword456" /tmp/error-unreadable.txt
echo "Found password 456 (FAIL if 0): $?"
```

### Step 7: Cleanup
```bash
rm -rf /tmp/tq-test-security
rm -f /tmp/error-*.txt
```

## Expected Results

### Step 1 Output:
```
-rw-r--r--  1 user  group  16 Jan 21 10:00 /tmp/tq-test-security/insecure-pw
```

### Step 2 Output:
```
----------  1 user  group  16 Jan 21 10:00 /tmp/tq-test-security/unreadable-pw
```

### Step 3 Output:
Error about **insecure permissions** (not about file read):
```
Error: Password file has insecure permissions: /tmp/tq-test-security/insecure-pw
Current permissions: 0644 (readable by group and others)
Required permissions: 0600 (owner read-write only)

Security risk: Password file is readable by other users

Fix: chmod 0600 /tmp/tq-test-security/insecure-pw
```

**CRITICAL:** Error is about **permissions** (check happened first), not about connection or file reading.

### Step 4 Output:
Error about **insecure permissions** (permission check catches 0000 as invalid):
```
Error: Password file has insecure permissions: /tmp/tq-test-security/unreadable-pw
Current permissions: 0000
Required permissions: 0600 (owner read-write only)
...
```

**Or potentially:** If 0000 passes permission check logic (edge case), then read would fail with different error. This validates order based on which error appears.

### Step 5 Output:
For insecure file (0644):
```
Found permission error: 0
```
(grep found "insecure permissions")

**Interpretation:**
- If permission error: Check happened **before** read ✅ CORRECT
- If read error: File was read **before** check ❌ INCORRECT (Sprint 16 bug)

### Step 6 Output:
```
Found password 123 (FAIL if 0): 1
Found password 456 (FAIL if 0): 1
```
(grep found nothing - exit code 1)

**CRITICAL:** Passwords should NOT appear in error messages (content never read).

### Step 7 Output:
Cleanup successful.

## Pass/Fail Criteria

**PASS if:**
- ✅ Insecure file (0644) error is about **permissions** (not read)
- ✅ Error message says "insecure permissions" or similar
- ✅ Error message mentions 0644 and 0600
- ✅ File content (passwords) do NOT appear in error messages
- ✅ Permission check clearly happened **before** read attempt
- ✅ Exit codes are non-zero (errors occurred)
- ✅ **Ordering fix validated: check-before-read**

**FAIL if:**
- ❌ Error is about file reading (not permissions)
- ❌ Error says "failed to read file" before mentioning permissions
- ❌ Password content appears in error messages (file was read)
- ❌ Permission check happened **after** read (Sprint 16 bug)
- ❌ Exit codes are 0 (success)

## Validation Logic

This test uses error message analysis to determine execution order:

### Correct Order (Sprint 17 Fix):
```
1. validate_password_file_permissions() → FAILS with "insecure permissions"
2. read_to_string() → NEVER CALLED (early return)
```

**Proof:** Error message is about permissions, no file content in error.

### Incorrect Order (Sprint 16 Bug):
```
1. read_to_string() → Might succeed, reads password
2. validate_password_file_permissions() → Warns but allows
```

**Proof:** Would see connection attempt or different error (file was read).

### Test Strategy:
- Insecure file (0644): Permission check should catch and reject
- If file content appears in output: Read happened (FAIL)
- If permission error occurs: Check happened first (PASS)

## Code Reference

**Sprint 16 (Incorrect):**
```rust
// In src/main.rs read_password_if_needed
let password = fs::read_to_string(path)?;  // Read FIRST (BUG)
validate_password_file_permissions(path)?; // Check AFTER (too late)
```

**Sprint 17 (Fixed):**
```rust
// In src/main.rs read_password_if_needed
validate_password_file_permissions(path)?; // Check FIRST (correct)
let password = fs::read_to_string(path)?;  // Read AFTER (correct)
```

**Reference Implementation (config.rs - Already Correct):**
```rust
// In src/config.rs - correct order already
validate_password_file_permissions(path)?; // Check first
let password = fs::read_to_string(path)?;  // Read after
```

## Security Implications

### Why Order Matters:
1. **Race Condition:** Brief window where insecure file content could be accessed
2. **Defense in Depth:** Check should prevent read, not just warn after
3. **Audit Trail:** Error should show permission check prevented access
4. **Principle of Least Privilege:** Never access content that shouldn't be accessed

### What This Prevents:
- Reading insecure file content (even briefly)
- Exposing passwords in error messages (read before check)
- Race conditions in audit logs
- Time-of-check-time-of-use (TOCTTOU) vulnerabilities

## Actual Results

_To be filled during test execution_

**Insecure File (0644) Error Type:**
- [ ] Permission error (correct)
- [ ] Read error (incorrect - bug)

**Unreadable File (0000) Error Type:**
- [ ] Permission error (check first)
- [ ] Read error (read first)

**Password Content in Errors:**
- [ ] No passwords found (correct)
- [ ] Passwords found (FAIL - file was read)

**Ordering Validation:**
- [ ] Permission check happens before read (PASS)
- [ ] Read happens before check (FAIL - Sprint 16 bug)

**Issues Found:**

## Notes

- This is a **code-level security fix**, not a user-visible feature change
- User experience is similar (file still rejected) but ordering is critical for security
- Race condition eliminated: File content never accessed if permissions wrong
- Ordering matches `config.rs` pattern (consistency across codebase)

**Why This Bug Existed:**
- Sprint 16 implemented permission check but placed it after read
- `config.rs` had correct order, `main.rs` had incorrect order
- Code review in Sprint 16 identified this as P1 issue for Sprint 17

**Test Coverage Note:**
- Unit tests might not catch ordering bugs (need integration test with real files)
- This integration test proves ordering by analyzing actual error messages

**Related Test Cases:**
- TC-SECURITY-001: Validates enforcement (rejection behavior)
- TC-SECURITY-002: Validates config file different policy
- Both tests assume correct ordering (this test proves it)

**Specification References:**
- `docs/builder/sprints/sprint-17-planning.md` Feature 2 (lines 72-87)
- Sprint 16 review: Identified ordering issue (section 6, Issue 2)
