# TC-SESS-006: Privilege Error Handling

**Test Case ID:** TC-SESS-006
**Feature:** Sessions Command - Error Handling
**Test Type:** Error Simulation (Unit/Integration)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that `/sessions` command displays helpful error message with GRANT statement example when user lacks SELECT privilege on DBC.MonitorSession.

---

## Prerequisites

- [ ] Rust test framework OR
- [ ] Live Teradata database with ability to revoke privileges (for manual testing)

---

## Test Approach

### Option A: Mock-Based Unit Test (Preferred)

**Implementation:**
```rust
#[test]
fn test_sessions_privilege_error() {
    let mut mock_client = MockDatabaseClient::new();

    // Mock returns permission denied error
    mock_client
        .expect_execute()
        .returning(|_sql| Err(DatabaseError::PermissionDenied(
            "SELECT permission denied on DBC.MonitorSession".into()
        )));

    let mut output = Vec::new();
    let result = execute_sessions(&mock_client, &mut output);

    assert!(result.is_ok()); // Command doesn't panic

    let output_str = String::from_utf8(output).unwrap();

    // Verify error message contains:
    assert!(output_str.contains("Insufficient privileges"));
    assert!(output_str.contains("SELECT permission denied"));
    assert!(output_str.contains("DBC.MonitorSession"));
    assert!(output_str.contains("GRANT"));
    assert!(output_str.contains("Contact your DBA"));
}
```

### Option B: Manual Test with Live Database

**Setup:**
```sql
-- As DBA, revoke access from test user
REVOKE SELECT ON DBC.MonitorSession FROM testuser;
```

**Test Steps:**

#### Step 1: Start REPL as User Without Privilege
**Action:** Connect as user without MonitorSession access
```bash
TQ_LOGON="testuser:password@host:1025/db" tq repl
```

#### Step 2: Execute /sessions Command
**Action:** Type `/sessions` and press Enter
```
tq> /sessions
```

**Expected Result:**
- Error message displays (not crash)
- Message indicates insufficient privileges
- Message mentions DBC.MonitorSession
- Message includes GRANT statement example
- Message suggests contacting DBA

#### Step 3: Verify Error Message Content
**Expected Error Message:**
```
Error: Unable to list sessions
Reason: SELECT permission denied on DBC.MonitorSession

This command requires SELECT access to the MonitorSession table function.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.MonitorSession TO <your_username>;
```

**Pass Criteria:**
- [x] Error message is clear and actionable
- [x] Mentions specific privilege needed (SELECT on DBC.MonitorSession)
- [x] Provides GRANT statement example
- [x] Suggests contacting DBA
- [x] Does not crash or hang

---

## Expected Results

### Success Criteria
- [x] Command handles privilege error gracefully
- [x] Error message is clear and helpful
- [x] Message includes GRANT statement example
- [x] Message mentions DBC.MonitorSession specifically
- [x] Message suggests contacting DBA
- [x] No stack trace or panic

### Expected Error Message
```
Error: Unable to list sessions
Reason: SELECT permission denied on DBC.MonitorSession

This command requires SELECT access to the MonitorSession table function.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.MonitorSession TO <your_username>;
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual Error Message:**
```
[Paste actual error message here]
```

**Observations:**
- [Note clarity of error message]
- [Note if GRANT example is present]
- [Note if message is actionable]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any error handling bugs]
- [List any unclear messages]

---

## Notes

- Mock-based testing is preferred (no database required)
- Manual testing requires ability to revoke privileges
- Error message UX is critical for user experience
- Clear error messages reduce support burden

---

## Related Requirements

- AC-8: Error handling for insufficient privileges (DBC access required)
- REQ-SESS-005.1: Privilege errors SHALL include helpful explanation and GRANT statement example
- repl.md lines 1611-1619: Privilege error message specification
