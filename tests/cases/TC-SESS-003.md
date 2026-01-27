# TC-SESS-003: Skew Calculation Accuracy (Unit Tests)

**Test Case ID:** TC-SESS-003
**Feature:** Sessions Command - Skew Calculation
**Test Type:** Unit Test
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that the skew calculation algorithm correctly computes CPU and IO skew percentages, handles NULL cases (IDLE sessions), and formats output with two decimal places.

---

## Prerequisites

- [ ] Rust test framework available
- [ ] Unit test module in place

---

## Test Cases

### Unit Test 1: Active Session with Skew
**Test Name:** `test_calculate_skew_active_session`

**Input:**
```rust
let avg_amp_cpu = 50.0;
let hot_amp1_cpu = 60.0;
let expected_skew = 16.67; // 100 * (1 - (50/60)) = 16.67%
```

**Expected Result:**
```rust
assert_eq!(calculate_skew(avg_amp_cpu, hot_amp1_cpu), Some(16.67));
```

**Pass Criteria:** Function returns Some(16.67)

---

### Unit Test 2: IDLE Session (NULL Skew)
**Test Name:** `test_calculate_skew_idle_session`

**Input:**
```rust
let avg_amp_cpu = 0.0;
let hot_amp1_cpu = 0.0;
```

**Expected Result:**
```rust
assert_eq!(calculate_skew(avg_amp_cpu, hot_amp1_cpu), None);
```

**Pass Criteria:** Function returns None (not Some(0.0) or error)

---

### Unit Test 3: Perfect Balance (0% Skew)
**Test Name:** `test_calculate_skew_perfect_balance`

**Input:**
```rust
let avg_amp_cpu = 100.0;
let hot_amp1_cpu = 100.0;
let expected_skew = 0.0; // 100 * (1 - (100/100)) = 0%
```

**Expected Result:**
```rust
assert_eq!(calculate_skew(avg_amp_cpu, hot_amp1_cpu), Some(0.0));
```

**Pass Criteria:** Function returns Some(0.0)

---

### Unit Test 4: Extreme Skew (Near 100%)
**Test Name:** `test_calculate_skew_extreme_skew`

**Input:**
```rust
let avg_amp_cpu = 1.0;
let hot_amp1_cpu = 100.0;
let expected_skew = 99.0; // 100 * (1 - (1/100)) = 99%
```

**Expected Result:**
```rust
assert_eq!(calculate_skew(avg_amp_cpu, hot_amp1_cpu), Some(99.0));
```

**Pass Criteria:** Function returns Some(99.0)

---

### Unit Test 5: LogonTime Formatting
**Test Name:** `test_format_logon_time`

**Input:**
```rust
let teradata_timestamp = "2026-01-27 15:33:26.00";
```

**Expected Result:**
```rust
assert_eq!(format_logon_time(teradata_timestamp), "2026/01/27 15:33:26.00");
```

**Pass Criteria:** Hyphens replaced with slashes, time portion unchanged

---

### Unit Test 6: SessionInfo from Complete Row
**Test Name:** `test_session_info_from_row_complete`

**Input:**
```rust
let row = vec![
    Value::Integer(1076),
    Value::String("DBC".into()),
    Value::Timestamp("2026-01-27 15:33:26.00".into()),
    Value::String("ACTIVE".into()),
    Value::String("ACTIVE".into()),
    Value::Decimal(366.736),
    Value::Integer(75335),
    Value::Integer(26753187840),
    Value::Decimal(350.0), // avg_amp_cpu
    Value::Decimal(360.0), // hot_amp1_cpu
    Value::Decimal(72000.0), // avg_amp_io
    Value::Decimal(75000.0), // hot_amp1_io
];
```

**Expected Result:**
```rust
let session = SessionInfo::from_row(&row).unwrap();
assert_eq!(session.session_no, 1076);
assert_eq!(session.user_name, "DBC");
assert_eq!(session.logon_time, "2026/01/27 15:33:26.00");
assert_eq!(session.pe_state, "ACTIVE");
assert_eq!(session.amp_state, "ACTIVE");
assert_eq!(session.amp_cpu_sec, 366.736);
assert_eq!(session.amp_io, 75335);
assert_eq!(session.req_spool, 26753187840);
// Skew: 100 * (1 - (350/360)) = 2.78
assert_eq!(session.cpu_skew, Some(2.78));
// Skew: 100 * (1 - (72000/75000)) = 4.0
assert_eq!(session.io_skew, Some(4.0));
```

**Pass Criteria:** All fields parsed correctly, skew calculated

---

### Unit Test 7: SessionInfo from IDLE Row (NULL Skew)
**Test Name:** `test_session_info_from_row_idle`

**Input:**
```rust
let row = vec![
    Value::Integer(1077),
    Value::String("DBC".into()),
    Value::Timestamp("2026-01-27 15:33:27.00".into()),
    Value::String("IDLE".into()),
    Value::String("IDLE".into()),
    Value::Decimal(0.0),
    Value::Integer(6),
    Value::Integer(0),
    Value::Decimal(0.0), // avg_amp_cpu = 0
    Value::Decimal(0.0), // hot_amp1_cpu = 0
    Value::Decimal(0.0), // avg_amp_io = 0
    Value::Decimal(0.0), // hot_amp1_io = 0
];
```

**Expected Result:**
```rust
let session = SessionInfo::from_row(&row).unwrap();
assert_eq!(session.session_no, 1077);
assert_eq!(session.pe_state, "IDLE");
assert_eq!(session.cpu_skew, None); // NULL for IDLE
assert_eq!(session.io_skew, None); // NULL for IDLE
```

**Pass Criteria:** Skew is None (not Some(0.0))

---

### Unit Test 8: SessionInfo with NULL Fields
**Test Name:** `test_session_info_from_row_nulls`

**Input:**
```rust
let row = vec![
    Value::Integer(1078),
    Value::String("alice".into()),
    Value::Timestamp("2026-01-27 16:00:00.00".into()),
    Value::String("ACTIVE".into()),
    Value::String("ACTIVE".into()),
    Value::Null, // AMPCPUSec is NULL
    Value::Null, // AMPIO is NULL
    Value::Null, // ReqSpool is NULL
    Value::Decimal(100.0),
    Value::Decimal(110.0),
    Value::Decimal(5000.0),
    Value::Decimal(5200.0),
];
```

**Expected Result:**
```rust
let session = SessionInfo::from_row(&row).unwrap();
assert_eq!(session.amp_cpu_sec, 0.0); // NULL defaults to 0
assert_eq!(session.amp_io, 0); // NULL defaults to 0
assert_eq!(session.req_spool, 0); // NULL defaults to 0
// Skew still calculated from avg/hot values
assert!(session.cpu_skew.is_some());
assert!(session.io_skew.is_some());
```

**Pass Criteria:** NULL fields default to 0, skew still calculated

---

## Expected Results

### Success Criteria
- [x] All 8 unit tests pass
- [x] Skew calculation formula correct: `100 * (1 - (avg / hot))`
- [x] NULL skew when hot = 0
- [x] LogonTime format conversion works
- [x] SessionInfo parsing handles complete rows
- [x] SessionInfo parsing handles IDLE rows
- [x] SessionInfo parsing handles NULL fields

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** cargo test
**Build Version:** [Commit hash]

**Test Output:**
```
[Paste cargo test output here]
```

**Pass/Fail Count:**
- Passed: [count]/8
- Failed: [count]/8

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any bugs in skew calculation logic]
- [List any NULL handling issues]

---

## Notes

- These unit tests validate pure logic without database dependency
- Run with: `cargo test --lib calculate_skew`
- Run with: `cargo test --lib session_info_from_row`

---

## Related Requirements

- AC-4: Skew percentages calculated correctly (NULL for inactive sessions)
- AC-5: Logon times formatted as YYYY/MM/DD HH:MM:SS.ss
- REQ-SESS-004.1: NULL skew displayed as [--]
- REQ-SESS-004.2: Skew format X.XX (two decimal places)
- design/repl.md: Skew calculation formula
