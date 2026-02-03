# TC-034-SECURITY-001: SQL Identifier Quoting for Security Hardening

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-034-SECURITY-001 |
| **Title** | SQL Identifier Quoting for Security Hardening |
| **Category** | Unit Test + Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 34 - Security Hardening (AC-6 through AC-10) |
| **Test Type** | Unit + Integration |
| **Created** | 2026-02-03 |

## Purpose

Verify that SQL identifiers are properly quoted in all data sampling commands to prevent SQL injection and handle special characters in table names.

## Acceptance Criteria Coverage

- **AC-6**: SQL identifiers quoted in `/sample` command (`"database"."table"`)
- **AC-7**: SQL identifiers quoted in `/peek` command
- **AC-8**: SQL identifiers quoted in batch mode (`tq sample`, `tq peek`)
- **AC-9**: Unit tests validate quoted identifier generation
- **AC-10**: Regression tests verify functionality with special characters in table names

## Scope

This test validates:
- `quote_identifier()` function correctly quotes single identifiers
- `quote_qualified_name()` function quotes database.table separately
- Double-quote escaping for identifiers containing quotes
- SQL generation in `/sample` command uses quoted identifiers
- SQL generation in `/peek` command uses quoted identifiers
- Batch mode commands properly quote identifiers
- Edge cases: spaces, hyphens, quotes, reserved words

## Prerequisites

- Rust test framework available
- SQL identifier utilities implemented (`src/sql/identifiers.rs`)
- Test database with TQ_LOGON configured (for integration tests)

## Test Procedure

### Test 1: Unit Tests - quote_identifier() Function

**Test Implementation:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_normal_identifier() {
        let result = quote_identifier("customers");
        assert_eq!(result, "\"customers\"");
    }

    #[test]
    fn test_quote_identifier_with_spaces() {
        let result = quote_identifier("My Table");
        assert_eq!(result, "\"My Table\"");
    }

    #[test]
    fn test_quote_identifier_with_hyphens() {
        let result = quote_identifier("table-2024");
        assert_eq!(result, "\"table-2024\"");
    }

    #[test]
    fn test_quote_identifier_with_quotes() {
        // Double quotes must be escaped as ""
        let result = quote_identifier("customer\"data");
        assert_eq!(result, "\"customer\"\"data\"");
    }

    #[test]
    fn test_quote_identifier_with_underscores() {
        let result = quote_identifier("my_table_name");
        assert_eq!(result, "\"my_table_name\"");
    }

    #[test]
    fn test_quote_reserved_word() {
        let result = quote_identifier("select");
        assert_eq!(result, "\"select\"");
    }

    #[test]
    fn test_quote_empty_identifier() {
        let result = quote_identifier("");
        // Should either return "" or error - document behavior
        assert!(result == "\"\"" || result.is_empty());
    }
}
```

### Test 2: Unit Tests - quote_qualified_name() Function

**Test Implementation:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_qualified_normal() {
        let result = quote_qualified_name("mydb.customers");
        assert_eq!(result, "\"mydb\".\"customers\"");
    }

    #[test]
    fn test_quote_qualified_with_spaces() {
        let result = quote_qualified_name("My DB.My Table");
        assert_eq!(result, "\"My DB\".\"My Table\"");
    }

    #[test]
    fn test_quote_qualified_with_special_chars() {
        let result = quote_qualified_name("db-2024.table-name");
        assert_eq!(result, "\"db-2024\".\"table-name\"");
    }

    #[test]
    fn test_quote_single_identifier() {
        // Should handle unqualified names
        let result = quote_qualified_name("customers");
        assert_eq!(result, "\"customers\"");
    }

    #[test]
    fn test_quote_qualified_with_quotes() {
        let result = quote_qualified_name("my\"db.customer\"table");
        assert_eq!(result, "\"my\"\"db\".\"customer\"\"table\"");
    }
}
```

### Test 3: Integration Tests - SQL Generation with Quoting

**Test Implementation:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_sql_normal_table() {
        let sql = generate_sample_sql("customers", 10);

        assert!(sql.contains("\"customers\""), "Table name must be quoted");
        assert!(sql.contains("SAMPLE 10"), "Should include SAMPLE clause");
    }

    #[test]
    fn test_sample_sql_qualified_table() {
        let sql = generate_sample_sql("mydb.customers", 20);

        assert!(sql.contains("\"mydb\".\"customers\""),
                "Qualified name must quote both parts");
        assert!(sql.contains("SAMPLE 20"), "Should include SAMPLE clause");
    }

    #[test]
    fn test_sample_sql_special_chars() {
        let sql = generate_sample_sql("My Table", 5);

        assert!(sql.contains("\"My Table\""),
                "Table with spaces must be quoted");
    }

    #[test]
    fn test_peek_sql_normal_table() {
        let sql = generate_peek_sql("customers", 10);

        assert!(sql.contains("\"customers\""), "Table name must be quoted");
        assert!(sql.contains("TOP 10") || sql.contains("SAMPLE 10"),
                "Should limit rows");
    }

    #[test]
    fn test_peek_metadata_sql_quoted() {
        let sql = generate_peek_metadata_sql("customers");

        // Metadata query should also quote table name
        assert!(sql.contains("\"customers\""),
                "Table name in metadata query must be quoted");
    }
}
```

### Test 4: Integration Tests - Database Execution (Requires Database)

**Test Implementation:**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // Requires database
    fn test_sample_with_special_table_name() {
        // This test requires TQ_LOGON to be set
        // and a test table with special characters to exist

        // Setup: Create table "My Test Table" if it doesn't exist
        // Execute: /sample "My Test Table" 5
        // Verify: No SQL errors, data returned

        // Implementation will be in tests/interactive_tests.rs
        // or as separate integration test file
    }

    #[test]
    #[ignore] // Requires database
    fn test_peek_with_hyphenated_table() {
        // Setup: Create table "table-2024" if it doesn't exist
        // Execute: /peek "table-2024" 10
        // Verify: No SQL errors, data returned
    }
}
```

### Test 5: Regression Tests - Existing Functionality

**Execution:**

```bash
# Run all unit tests
cargo test --lib

# Run integration tests (without database)
cargo test --test '*' --lib

# Run interactive tests with database
cargo test --test interactive_tests -- --ignored --test-threads=1

# Verify no regressions in existing 471 tests
```

**Expected:**
- All 471 existing tests continue to pass
- New tests (14-18) all pass
- Total: 485-489 tests passing
- Zero regressions

## Expected Results

### Test 1: quote_identifier() Unit Tests
- **Status**: PASS
- All 7 tests pass
- Normal identifiers quoted correctly
- Special characters (spaces, hyphens, quotes) handled
- Edge cases (empty, reserved words) handled

### Test 2: quote_qualified_name() Unit Tests
- **Status**: PASS
- All 5 tests pass
- Qualified names split and quoted separately
- Single identifiers handled correctly
- Special characters in both parts handled

### Test 3: SQL Generation Unit Tests
- **Status**: PASS
- All 5 tests pass
- Sample SQL uses quoted identifiers
- Peek SQL uses quoted identifiers
- Metadata queries use quoted identifiers

### Test 4: Database Integration (if executed)
- **Status**: PASS or SKIP (if no database)
- Special character table names work
- No SQL syntax errors
- Data retrieved successfully

### Test 5: Regression Tests
- **Status**: PASS
- All 485-489 tests pass
- Zero regressions
- All existing features work with quoting

## Pass Criteria

- ✅ All unit tests for quote functions pass (17/17)
- ✅ SQL generation tests confirm quoting used (5/5)
- ✅ Integration tests pass or skipped if no database (2/2)
- ✅ Full regression suite passes (100%)
- ✅ AC-6 through AC-10 all satisfied

## Failure Scenarios

| Scenario | Detection | Impact |
|----------|-----------|--------|
| quote_identifier() fails | Unit test failure | Security AC-9 NOT MET |
| SQL not using quoting | Integration test failure | Security AC-6, AC-7, AC-8 NOT MET |
| Special chars cause SQL errors | Database test failure | Security AC-10 NOT MET |
| Regression failures | cargo test output | Security AC-10 NOT MET |
| Quote escaping wrong | Unit test failure | SQL injection risk |

## Security Implications

**Risk Mitigation:**
- Prevents SQL injection through table name manipulation
- Handles edge cases (spaces, quotes, special characters)
- Ensures compatibility with Teradata quoted identifier rules

**Attack Scenarios Prevented:**
- Table name: `customers; DROP TABLE users--`
  - Without quoting: SQL injection
  - With quoting: `"customers; DROP TABLE users--"` (safe, treated as literal name)
- Table name: `My Table` (space)
  - Without quoting: SQL syntax error
  - With quoting: `"My Table"` (valid)

**Teradata Quoting Rules:**
- Double quotes delimit identifiers
- Internal double quotes must be escaped as `""`
- Quoted identifiers are case-sensitive
- Quoted identifiers can contain any character

## Notes

- This is a security improvement with no user-facing behavior changes for normal table names
- Users with special character table names will benefit from improved compatibility
- Integration tests require database setup (may use TQ_LOGON from .env)
- Sprint 33 baseline: 384 unit tests + 87 integration/interactive = 471 total

## Test Database Setup (Optional)

If executing database integration tests:

```sql
-- Create test tables with special character names
CREATE TABLE "My Test Table" (
    id INTEGER,
    name VARCHAR(100),
    created_date DATE
);

CREATE TABLE "table-2024" (
    id INTEGER,
    value DECIMAL(10,2)
);

INSERT INTO "My Test Table" VALUES (1, 'Test', CURRENT_DATE);
INSERT INTO "table-2024" VALUES (1, 100.50);
```

Cleanup:

```sql
DROP TABLE "My Test Table";
DROP TABLE "table-2024";
```

## References

- Sprint 34 Planning: `docs/sprints/sprint-34-planning.md`
- Sprint 34 Test Strategy: `tests/strategy/sprint-34-test-strategy.md` (Track 2)
- Sprint 33 Review: `docs/sprints/sprint-33-review.md` (security gap identification)
- Teradata SQL Reference: Quoted Identifiers section
