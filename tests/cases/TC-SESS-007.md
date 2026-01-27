# TC-SESS-007: Empty Result Set Handling

**Test Case ID:** TC-SESS-007
**Feature:** Sessions Command - Edge Case Handling
**Test Type:** Error Simulation (Unit/Integration)
**Priority:** P1
**Created:** 2026-01-27

---

## Objective

Verify that `/sessions` command gracefully handles empty result sets (no active sessions besides current session, or mock scenario with zero rows).

---

## Prerequisites

- [ ] Rust test framework for mock-based testing

---

## Test Approach

### Option A: Mock-Based Unit Test (Preferred)

**Implementation:**
```rust
#[test]
fn test_sessions_empty_result() {
    let mut mock_client = MockDatabaseClient::new();

    // Mock returns empty result set
    mock_client
        .expect_execute()
        .returning(|_sql| Ok(QueryResult {
            columns: vec![
                /* 10 column definitions */
            ],
            rows: vec![], // Empty rows
            row_count: 0,
            execution_time: Duration::from_millis(50),
        }));

    let mut output = Vec::new();
    let result = execute_sessions(&mock_client, &mut output);

    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();

    // Verify output contains:
    assert!(output_str.contains("0 sessions found"));
    assert!(output_str.contains("SessionNo")); // Header still present
    assert!(!output_str.contains("Error")); // Not an error condition
}
```

### Option B: Manual Test (Difficult - Requires Database Isolation)

**Note:** This test is difficult to perform manually because:
- The current tq session itself appears in MonitorSession results
- Cannot reliably create "no sessions" scenario without stopping database
- Mock-based testing is strongly preferred

**If Attempted Manually:**

#### Step 1: Execute Command
**Action:** Run `/sessions` in REPL
```
tq> /sessions
```

**Expected Result:**
- At least one session appears (current session)
- Test cannot easily create zero-session scenario

**Alternative:** Use mock client to simulate empty result

---

## Expected Results

### Success Criteria
- [x] Empty result set does not cause error
- [x] Table header still displays
- [x] Footer shows "0 sessions found"
- [x] No crash or panic
- [x] Clean table structure (even with no data rows)

### Expected Output (Mock Scenario)
```
Active Sessions:
┌───────────┬──────────┬───────────┬─────────┬──────────┬───────────┬───────┬──────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime │ PEstate │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼───────────┼─────────┼──────────┼───────────┼───────┼──────────┼────────────────┼──────────────┤
└───────────┴──────────┴───────────┴─────────┴──────────┴───────────┴───────┴──────────┴────────────────┴──────────────┘

0 sessions found (Query time: 0.050s)
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** cargo test
**Build Version:** [Commit hash]

**Test Output:**
```
[Paste test output here]
```

**Observations:**
- [Note how empty result is displayed]
- [Note if table structure is maintained]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any empty result handling bugs]

---

## Notes

- Mock-based testing is strongly preferred
- Empty result scenario is edge case (current session always present)
- Table structure should be maintained even with 0 rows
- Not an error condition - just empty data

---

## Related Requirements

- AC-9: Handles empty result set (no active sessions besides current)
- REQ-SESS-005.2: Empty result set SHALL still display table with headers
- repl.md lines 1622-1634: Empty result set example
