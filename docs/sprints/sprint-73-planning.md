---
sprint: 73
start_date: 2026-07-11
target_completion: 2026-07-11
status: Complete
---

# Sprint 73 Planning: High-Performance FastLoad and FastExport (#50)

## Sprint Overview

**Sprint Goal:** Implement high-performance FastLoad and FastExport capabilities in `tq` CLI, supporting CSV, Parquet, and JSON file formats as source for bulk loading, and CSV format for exporting, with live Teradata database validation.

**Sprint Theme:** High-Performance Data Engineering

**Issue:** [#50](https://github.com/remi-td/tq/issues/50) — `[FEATURE] Implement Fastload`

**Target Version:** v1.54.0

---

## Objectives

1. **Add `tq fastload` CLI Command** — Load CSV, Parquet, and JSON files in parallel using the Teradata native FastLoad protocol. Auto-detect source format and support lazy-load table auto-creation with permissive data types if the target table does not exist.
2. **Add `tq fastexport` CLI Command** — Export Teradata tables or queries directly to CSV files in parallel using the Teradata native FastExport protocol.
3. **Database Client Integration** — Extend `DatabaseClient` to configure and execute FastLoad and FastExport operations via native driver escape functions (e.g. `{fn teradata_read_csv}`, `{fn teradata_write_csv}`, `{fn teradata_require_fastload}`).
4. **Validation and Quality** — Perform automated testing and live validation against the active Teradata trial database, verifying correctness, error handling (reading error tables), and performance improvements.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: FastExport Subcommand
- **Description:** Implement `tq fastexport <source-table> <target-file>` to export a table directly to a CSV file.
- **Implementation:** Prepend `{fn teradata_require_fastexport}{fn teradata_write_csv(target_file)}SELECT * FROM <source-table>` and execute via the driver.
- **Acceptance Criteria:**
  - [x] Exports table data in parallel connections to a CSV file.
  - [x] Supports table name syntax like `dbname.tablename` or just `tablename`.
  - [x] Writes CSV with a header row.
  - [x] Displays execution time and row count (from `activity_count`).

#### Feature 2: FastLoad Subcommand for CSV
- **Description:** Implement `tq fastload <source-file.csv> <target-table>` to bulk load a CSV file into an empty permanent table.
- **Implementation:** Set autocommit to false, execute `{fn teradata_require_fastload}{fn teradata_read_csv(csv_path)}INSERT INTO <target_table> VALUES (?, ?, ...)` with parameter markers matching target table columns, inspect warnings/errors, and commit or rollback.
- **Acceptance Criteria:**
  - [x] Performs parallel FastLoad of CSV files.
  - [x] Empties or expects an empty destination table (as required by FastLoad).
  - [x] Handles data/constraint errors by reading error tables (using `{fn teradata_nativesql}{fn teradata_get_errors}`).
  - [x] Cleans up/deletes error tables if the load completes successfully.

#### Feature 3: Lazy-Load Table Creation
- **Description:** Automatically create the target table with permissive types (e.g. `VARCHAR(1000)`) if it does not exist.
- **Acceptance Criteria:**
  - [x] Checks if the target table exists.
  - [x] If missing, parses the CSV header or Parquet/JSON schema, creates the table with `VARCHAR(1000)` columns (or type-mapped columns), and then initiates the load.

---

### P1 - High Priority (Should Have)

#### Feature 4: Parquet and JSON Source Formats for FastLoad
- **Description:** Support loading `.parquet`, `.json`, and newline-delimited JSON (NDJSON) files via FastLoad.
- **Implementation:** Parse the file in Rust (adding `parquet` and `arrow-csv` dependencies for Parquet, using `serde_json` and `csv` for JSON), serialize it streamingly to a temporary CSV file, and load the CSV natively.
- **Acceptance Criteria:**
  - [x] Auto-detects format from file extension (.parquet, .json, .csv).
  - [x] Supports `--source-format` override (`csv`, `parquet`, `json`).
  - [x] Converts Parquet and JSON files streamingly to temporary CSVs and loads them.
  - [x] Maps Parquet primitive types (BIGINT, DOUBLE, BOOLEAN) to corresponding Teradata types for auto-table creation.

#### Feature 5: FastLoad Custom Suffixes and Connection Scaling
- **Description:** Allow configuring the number of parallel connections and error table parameters.
- **Acceptance Criteria:**
  - [x] Supporting `--sessions <n>` (prepends `{fn teradata_sessions(n)}`).
  - [x] Supporting `--error-table-db <db>`, `--error-table-1-suffix <suffix>`, and `--error-table-2-suffix <suffix>`.

---

### P2 - Medium Priority (Nice to Have)

#### Feature 6: REPL Metacommands `/fastload` and `/fastexport`
- **Description:** Expose the load/export functions directly within the interactive REPL.
- **Acceptance Criteria:**
  - [ ] `/fastload <file> <table>` (Deferred)
  - [ ] `/fastexport <table> <file>` (Deferred)

---

### Explicitly Out of Scope
- Supporting `BLOB` and `CLOB` types (which are natively not supported by FastLoad/FastExport).
- Ingesting multi-gigabyte files entirely in memory (all converters and readers must use streaming I/O).

---

## Success Criteria
- [x] Bumps Cargo.toml version to `1.54.0`.
- [x] All P0 and P1 features implemented and tested.
- [x] 100% test pass rate for unit and integration tests.
- [x] Clean compile with no clippy warnings.
- [x] FastLoad and FastExport validated against the live trial Teradata database.
- [x] Documentation (`docs/specifications/cli-interface.md`, `docs/design/`, and user guides) updated and synchronized.

---

## Dependencies

### External
- **`parquet` crate** (version `53.0.0`) for Parquet parsing.
- **`arrow-csv` crate** (version `53.0.0`) for writing RecordBatches to CSV.
- **Live Teradata database** (`trial-vikzqtnd0db0nglk.env.trial.teradata.com`) for integration testing.

---

## Risks & Mitigation

### Risk 1: Large files causing out-of-memory errors
- **Mitigation:** Implement strict streaming conversion using chunked record batch reading for Parquet and line-by-line reading for NDJSON, writing directly to disk without buffering the entire file in RAM.

### Risk 2: FastLoad error tables locking or not cleaning up
- **Mitigation:** Always run cleanup queries (`DROP TABLE <error_table>`) in an `after_each` / error-handling block if the driver does not auto-drop them.

---

## Files Involved

### CLI & Dispatch
- [cli.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/cli.rs) — Define subcommands and arguments.
- [main.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/main.rs) — Dispatch commands and build config.
- [Cargo.toml](file:///Users/remi.turpaud/Code/genAI/tq/Cargo.toml) — Add `parquet` and `arrow-csv` dependencies, bump version.

### Database Client
- [client.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/db/client.rs) — Implement FFI connection parameters, `fastload`, and `fastexport` low-level drivers.

### Command Execution
- [fastload.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/commands/fastload.rs) [NEW] — High-level logic for format detection, table creation, conversion, execution, and error handling.
- [fastexport.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/commands/fastexport.rs) [NEW] — High-level logic for fastexport execution.
- [mod.rs](file:///Users/remi.turpaud/Code/genAI/tq/src/commands/mod.rs) — Export new commands.

### Documentation
- [cli-interface.md](file:///Users/remi.turpaud/Code/genAI/tq/docs/specifications/cli-interface.md) — Document new commands.
- [connection-management.md](file:///Users/remi.turpaud/Code/genAI/tq/docs/design/connection-management.md) — Document parallel loading design.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-07-11 | 1.0 | Initial Sprint 73 plan for FastLoad & FastExport | Main Agent |
