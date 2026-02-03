# TC-034-CODE-QUALITY-001: Extract format_column_type() to Shared Module

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-034-CODE-QUALITY-001 |
| **Title** | Extract format_column_type() to Shared Module - Code Quality |
| **Category** | Unit Test + Code Review |
| **Priority** | Critical |
| **Feature** | Sprint 34 - Code Quality Cleanup (AC-1 through AC-5) |
| **Test Type** | Unit + Code Review |
| **Created** | 2026-02-03 |

## Purpose

Verify that duplicate `format_column_type()` function has been extracted to a shared module and that both consumers use the shared implementation.

## Acceptance Criteria Coverage

- **AC-1**: `format_column_type()` extracted to shared module (`src/sql/types.rs`)
- **AC-2**: Both `sample.rs` and `metacommands.rs` use shared implementation
- **AC-3**: Unit tests pass for shared utility module
- **AC-4**: No code duplication detected in technical review
- **AC-5**: Zero regressions (all 471 tests continue to pass)

## Scope

This test validates:
- Shared module exists at `src/sql/types.rs`
- Function `format_column_type()` is implemented with proper signature
- All Teradata type codes correctly formatted (VARCHAR, INTEGER, DECIMAL, etc.)
- Both consumers import and use the shared function
- No duplicate implementations remain in codebase
- All existing tests continue to pass

## Prerequisites

- Rust test framework available
- Shared types module implemented

## Test Procedure

### Test 1: Unit Tests for format_column_type()

**Test Implementation:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_varchar() {
        // CV with length
        let result = format_column_type("CV", Some(100), None);
        assert_eq!(result, "VARCHAR(100)");
    }

    #[test]
    fn test_format_char() {
        // CF with length
        let result = format_column_type("CF", Some(50), None);
        assert_eq!(result, "CHAR(50)");
    }

    #[test]
    fn test_format_integer() {
        // I (INTEGER)
        let result = format_column_type("I", None, None);
        assert_eq!(result, "INTEGER");
    }

    #[test]
    fn test_format_decimal() {
        // D with precision and scale
        let result = format_column_type("D", Some(10), Some(2));
        assert_eq!(result, "DECIMAL(10,2)");
    }

    #[test]
    fn test_format_date() {
        // DA (DATE)
        let result = format_column_type("DA", None, None);
        assert_eq!(result, "DATE");
    }

    #[test]
    fn test_format_timestamp() {
        // TS (TIMESTAMP)
        let result = format_column_type("TS", Some(6), None);
        assert_eq!(result, "TIMESTAMP(6)");
    }

    #[test]
    fn test_format_time_with_timezone() {
        // TZ (TIME WITH TIME ZONE)
        let result = format_column_type("TZ", Some(6), None);
        assert_eq!(result, "TIME(6) WITH TIME ZONE");
    }

    #[test]
    fn test_format_json() {
        // JN (JSON)
        let result = format_column_type("JN", None, None);
        assert_eq!(result, "JSON");
    }

    #[test]
    fn test_format_unknown_type() {
        // Unknown type code
        let result = format_column_type("XX", Some(10), Some(2));
        assert!(result.contains("XX"));
        assert!(result.contains("10"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_format_binary_varying() {
        // BV (VARBYTE)
        let result = format_column_type("BV", Some(200), None);
        assert_eq!(result, "VARBYTE(200)");
    }

    #[test]
    fn test_format_clob() {
        // CO (CLOB)
        let result = format_column_type("CO", None, None);
        assert_eq!(result, "CLOB");
    }

    #[test]
    fn test_format_blob() {
        // BO (BLOB)
        let result = format_column_type("BO", None, None);
        assert_eq!(result, "BLOB");
    }
}
```

### Test 2: Code Review - Module Exists and Exports

**Verification Commands:**

```bash
# Check that shared module exists
test -f src/sql/types.rs
echo "Module exists: $?"

# Check that module exports format_column_type
grep -q "pub fn format_column_type" src/sql/types.rs
echo "Function exported: $?"

# Check sql/mod.rs re-exports types module
grep -q "pub mod types" src/sql/mod.rs
echo "Module declared: $?"
```

**Expected:**
- All three commands return 0 (success)
- Module exists at correct path
- Function is public
- Module is declared in mod.rs

### Test 3: Code Review - Consumers Use Shared Implementation

**Verification Commands:**

```bash
# Check sample.rs imports shared function
grep -q "use.*sql.*types.*format_column_type" src/commands/sample.rs
echo "sample.rs imports: $?"

# Check metacommands.rs imports shared function (if needed)
grep -q "use.*sql.*types.*format_column_type" src/commands/repl/metacommands.rs
echo "metacommands.rs imports: $?"

# Verify NO local implementations remain
# Should find only ONE definition (in types.rs)
count=$(grep -r "fn format_column_type" src/ | wc -l | tr -d ' ')
echo "Definition count: $count"
test "$count" -eq 1
echo "Single definition only: $?"
```

**Expected:**
- sample.rs imports the shared function
- metacommands.rs imports if it uses type formatting
- Only one definition exists (in types.rs)
- No duplicate implementations

### Test 4: Regression Test - Full Suite

**Execution:**

```bash
# Run all unit tests
cargo test --lib

# Verify pass count matches or exceeds Sprint 33 baseline
# Sprint 33: 384 unit tests
# Sprint 34: 384 + new tests (8-12) = 392-396 tests
```

**Expected:**
- All tests pass (100% pass rate)
- Test count increased by 8-12 (new unit tests for format_column_type)
- Zero test failures
- Zero regressions

## Expected Results

### Test 1: Unit Tests
- **Status**: PASS
- All 12 unit tests for format_column_type() pass
- Type formatting is correct for all Teradata type codes
- Edge cases (unknown types) handled gracefully

### Test 2: Module Structure
- **Status**: PASS
- Module src/sql/types.rs exists
- Function format_column_type() is public
- Module properly declared in sql/mod.rs

### Test 3: Shared Usage
- **Status**: PASS
- sample.rs imports and uses shared function
- metacommands.rs imports if needed
- Only one implementation exists (no duplicates)

### Test 4: Regression
- **Status**: PASS
- All 392-396 tests pass
- Zero regressions
- New tests added successfully

## Pass Criteria

- ✅ All unit tests for format_column_type() pass (12/12)
- ✅ Code review confirms module structure correct
- ✅ Code review confirms no duplicate implementations
- ✅ Full regression suite passes (100%)
- ✅ AC-1 through AC-5 all satisfied

## Failure Scenarios

| Scenario | Detection | Impact |
|----------|-----------|--------|
| Unit tests fail | cargo test output | Code Quality AC-3 NOT MET |
| Duplicate implementations found | grep verification | Code Quality AC-4 NOT MET |
| Regression failures | cargo test output | Code Quality AC-5 NOT MET |
| Module not exported | Import errors | Code Quality AC-1 NOT MET |
| Consumers don't use shared code | grep verification | Code Quality AC-2 NOT MET |

## Notes

- This is a pure refactoring test - no user-facing behavior changes
- Focus is on code organization and preventing code drift
- All type formatting must produce identical output before and after extraction
- Sprint 33 had 384 unit tests as baseline

## References

- Sprint 34 Planning: `docs/sprints/sprint-34-planning.md`
- Sprint 34 Test Strategy: `tests/strategy/sprint-34-test-strategy.md` (Track 1)
- Sprint 33 Review: `docs/sprints/sprint-33-review.md` (technical debt identification)
