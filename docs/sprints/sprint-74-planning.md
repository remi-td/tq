---
sprint: 74
start_date: 2026-07-16
target_completion: 2026-07-16
status: Planning
---

# Sprint 74 Planning: FastLoad Delimiter Validation and Release

## Sprint Overview

**Sprint Goal:** Review, validate, and polish the `delimiter` argument for `fastload` added in the current branch `fastload-update`, resolve compilation issues in the integration tests, eliminate tech debt related to delimiter-format mismatches, merge the branch into `master`, bump the version, and build.

**Sprint Theme:** Code Quality & Release Validation

---

## Reality Check Summary
- **Reviewed sprints:** Sprint 73, Sprint 72, Sprint 69.
- **Patterns detected:**
  - **Integration Test Compilation Failure:** Adding `delimiter` to `FastloadArgs` caused compilation errors in `tests/integration_fastload.rs` because the field was not initialized.
  - **Delimiter-Format Mismatch (Tech Debt):** When converting Parquet/JSON streamingly to temporary CSV, we write with comma delimiter but still pass the custom/auto-detected delimiter options to the Teradata loader. This can cause loader failures.
- **Decision:** Maintenance Sprint
- **Rationale:** Focus on code hygiene, fixing the test suite compilation, validating UX boundaries for the new option, and executing a clean merge/release.

---

## Objectives

1. **Fix Integration Test Compilation:** Resolve the missing field errors in `tests/integration_fastload.rs`.
2. **Prevent Delimiter-Format Mismatches:**
   - Enforce that `--delimiter` cannot be combined with Parquet or JSON formats (since they do not use field delimiters).
   - Ensure the temporary CSV loader options always default to comma when format is Parquet or JSON.
3. **Execute Full Test Suite:** Verify 100% pass rate for unit and integration tests.
4. **Release Prep:** Bump package version in `Cargo.toml`, merge branch to `master`, and build the binary.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Fix Integration Test Compilation
- **Description:** Populate `delimiter: None` in all `FastloadArgs` initializers inside `tests/integration_fastload.rs`.
- **Acceptance Criteria:**
  - [ ] `tests/integration_fastload.rs` compiles successfully.

#### Feature 2: Format and Delimiter UX Validation
- **Description:** Ensure that `--delimiter` is only used with CSV/TSV format. Return a descriptive error if configured with Parquet or JSON.
- **Acceptance Criteria:**
  - [ ] Running `tq fastload` with Parquet/JSON and `--delimiter` returns a clear validation error.
  - [ ] Loading Parquet/JSON streamingly uses the default comma delimiter for the loader backend (ignoring auto-detected extensions).

---

### P1 - High Priority (Should Have)

#### Feature 3: Full Test Verification
- **Description:** Run all unit and integration tests to ensure no regressions.
- **Acceptance Criteria:**
  - [ ] `cargo test` executes and passes (100% pass rate).

---

### P2 - Medium Priority (Nice to Have)

#### Feature 4: Version Bump and Merge
- **Description:** Bump version to `1.54.1` in `Cargo.toml` and merge the changes into `master`.
- **Acceptance Criteria:**
  - [ ] `Cargo.toml` version is bumped to `1.54.1`.
  - [ ] Changes are cleanly merged into the `master` branch.
  - [ ] The release build (`cargo build --release`) compiles cleanly.

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working as specified.
- [ ] 100% test pass rate.
- [ ] Documentation updated to reflect `--delimiter` CLI flags.
- [ ] Cargo version bumped to `1.54.1`.
- [ ] Code successfully merged to `master`.

---

## Agent Assignments

- **Sprint Coordinator (Main Agent):** Orchestrates the workflow and executes tasks.
- **rust-teradata-architect:** Reviews architecture, ensures correct delimiter passing in FFI, and fixes tests.
- **quality-validator:** Runs test execution loops and verifies success.

---

## Timeline

- **Phase 1: Planning & Setup** (Complete)
- **Phase 2: Design & Verification**
- **Phase 3: Implementation & Fixes**
- **Phase 4: Ship & Close**

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-07-16 | 1.0 | Initial Sprint 74 plan | Sprint Coordinator |
