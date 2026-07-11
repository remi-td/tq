# Sprint 73 Test Strategy

**Created:** 2026-07-11  
**Author:** quality-validator  
**Sprint:** Sprint 73  
**Features:** FastLoad and FastExport CLI and Client Implementation

---

## Feature-by-Feature Test Strategy

### Feature: fastload (CLI Command & FFI Driver)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/cli-interface.md#fastload---bulk-load-data-fastload`
- Requirements:
  - Parallel execution via multiple connections.
  - Auto-detection of CSV, Parquet, and JSON.
  - Streaming conversion to temporary CSV for Parquet/JSON.
  - Lazy-load table auto-creation if the target table does not exist.
  - Inspecting and displaying warnings/errors from Error Table 1 and 2.

**Feature Characteristics:**
- **User Interaction Type:** CLI Batch
- **Observable Behavior:** Structured data output, file system side effects, database side effects, performance characteristics.
- **External Dependencies:** Database connection, file system access.
- **Validation Challenges:** FastLoad requires an empty table and creates temporary error tables. Testing must clean up these tables. We need to test Parquet and JSON files without ballooning memory.

#### 2. Test Strategy Derivation

**Derived Test Types:**
- **Unit Tests:** Validate CLI parser, auto-detection logic, JSON conversion, and Parquet conversion.
- **Integration Tests:** Validate end-to-end execution against the live Teradata database, checking lazy-table creation, CSV loading, Parquet loading, and JSON loading.

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates formats parsing and file conversions | Conversion bugs could corrupt data before load | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates database interactions & FFI escape calls | FFI logic or SQL driver changes could fail silently | MUST IMPLEMENT |

---

### Feature: fastexport (CLI Command & FFI Driver)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/cli-interface.md#fastexport---bulk-export-data-fastexport`

**Feature Characteristics:**
- **User Interaction Type:** CLI Batch
- **Observable Behavior:** Structured data output, file system side effects.
- **External Dependencies:** Database connection, file system access.

#### 2. Test Strategy Derivation

**Derived Test Types:**
- **Unit Tests:** Validate CLI arguments.
- **Integration Tests:** Run fastexport against a table in the live Teradata database, write to a CSV file, and verify row count and content.

---

## Test Implementation Plan

### Test Type: Unit Tests
- **Location:** `src/cli.rs`, `src/commands/fastload.rs` test module, `src/commands/fastexport.rs` test module.
- **Scenarios to cover:**
  1. CLI arguments parsing for fastload and fastexport.
  2. File extension auto-detection (`csv`, `parquet`, `json`, `ndjson`).
  3. Conversion logic: reading CSV, NDJSON, and Parquet and outputting correct CSV formatted strings.

### Test Type: Integration Tests
- **Location:** `tests/integration_fastload.rs` [NEW]
- **Scenarios to cover:**
  1. `tq fastexport` table to CSV file, verifying row count.
  2. `tq fastload` CSV file into an empty table.
  3. `tq fastload` Parquet file into an empty table.
  4. `tq fastload` JSON file into an empty table.
  5. Lazy-table creation: loading into a table that doesn't exist.
  6. Error handling: loading a malformed file and verifying error table query messages are fetched.

---

## Strategy Summary

- **Total Features:** 2
- **Estimated Test Count:**
  - Unit: 12 tests
  - Integration: 6 tests
  - Total: 18 tests
- **Dependencies Required:** Live database connection (provided by `.env`).
