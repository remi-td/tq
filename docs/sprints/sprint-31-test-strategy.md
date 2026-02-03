# Sprint 31 Test Strategy: Framework Crisis Recovery

**Sprint:** 31 (Maintenance Sprint)
**Quality Validator:** quality-validator agent
**Date:** 2026-02-03
**Status:** PREPARED (Awaiting implementation path selection)

---

## Executive Summary

Sprint 31 addresses a critical framework crisis: two consecutive sprints (29 and 30) achieved 100% automated test pass rates while delivering completely broken features. This test strategy acknowledges fundamental testing limitations and establishes a reality-based validation approach.

**Key Principle:** For Sprint 31, quality-validator verdict is **ADVISORY ONLY**. Final sprint approval requires manual validation by sprint coordinator.

---

## Track 1: Documentation Updates (Passive Monitoring)

### Validation Approach

**No automated tests required.** Documentation changes are validated through:

1. **Content Review**: Verify brutal honesty in Sprint 29/30 assessments
2. **Completeness Check**: Confirm all 4 required files updated per cli-ux-designer specifications
3. **Advisory Verdict**: Provide quality observations, not blocking approval

### Files to Monitor

Track 1 requires updates to:
- `docs/testing/philosophy.md` - Testing limitations section
- `docs/testing/approach.md` - Feature type limitations
- `docs/testing/execution.md` - Manual validation process
- Sprint 29/30 reviews - Honest failure assessment

### Success Criteria

- [ ] All 4 documentation files updated
- [ ] Sprint 29 review updated to reflect FAILED status (not 9.5/10)
- [ ] Sprint 30 review confirms critical failure
- [ ] Testing documentation acknowledges limitations

### Verdict Format

```
TRACK 1 ADVISORY VERDICT: [PASS/CONCERNS]

Documentation Updates:
- philosophy.md: [Updated/Not Updated]
- approach.md: [Updated/Not Updated]
- execution.md: [Updated/Not Updated]
- Sprint reviews: [Honest/Still claiming success]

Observations: [Quality comments]

Recommendation: [Approve/Revise with reasons]
```

---

## Track 2: Pager Resolution - Option A (Fix Pager)

### Test Strategy Overview

**If Option A (Fix Pager) is chosen:**

Sprint 30 demonstrated that automated tests cannot validate visual rendering. Therefore, Option A test strategy combines:

1. **Unit tests** - Validate width calculation logic
2. **Dimensional tests** - Connect Track 3 utilities to render buffer
3. **Manual validation** - MANDATORY terminal testing (not optional)
4. **Evidence capture** - Document proof of functionality

**CRITICAL:** Manual validation is BLOCKING. Automated tests are INSUFFICIENT.

### Phase 1: Unit Tests for render_to_buffer()

**Objective:** Validate that pager can render to buffer for testing

**Test Implementation:**

```rust
// tests/pager_render_buffer_tests.rs

#[cfg(test)]
mod tests {
    use crate::commands::repl::pager::{Pager, PagerConfig};
    use crate::db::QueryResult;

    #[test]
    fn test_render_to_buffer_method_exists() {
        // Compile-time validation that render_to_buffer exists
        let result = create_test_result(5, 3);
        let config = PagerConfig::default();
        let pager = Pager::new(&result, &config);

        // This should compile
        let _buffer: String = pager.render_to_buffer();
    }

    #[test]
    fn test_render_to_buffer_produces_nonempty_output() {
        let result = create_test_result(5, 3);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 80;

        let buffer = pager.render_to_buffer();

        assert!(!buffer.is_empty(), "Rendered output should not be empty");
        assert!(buffer.contains("column"), "Should contain column headers");
    }
}
```

**Files to Create:**
- `tests/pager_render_buffer_tests.rs` (~100 lines)

**Success Criteria:**
- [ ] render_to_buffer() method compiles
- [ ] Method returns non-empty string
- [ ] Output contains table structure (borders, headers)

### Phase 2: Dimensional Tests with Track 3 Integration

**Objective:** Validate rendered output width using visual_validator.rs

**Test Implementation:**

```rust
// tests/pager_width_validation_tests.rs

#[cfg(test)]
mod tests {
    use crate::commands::repl::pager::{Pager, PagerConfig};

    // Import Track 3 utilities
    mod tools {
        include!("tools/visual_validator.rs");
    }

    #[test]
    fn test_pager_no_overflow_at_80_chars() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 80;

        let rendered = pager.render_to_buffer();

        // Use Track 3 validator
        tools::assert_no_overflow(&rendered, 80);
    }

    #[test]
    fn test_pager_no_overflow_at_117_chars() {
        // User-reported problematic width
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 117;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 117);
    }

    #[test]
    fn test_pager_no_overflow_at_120_chars() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 120;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 120);
    }

    #[test]
    fn test_pager_no_overflow_at_160_chars() {
        let result = create_wide_result(20, 10);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 160;

        let rendered = pager.render_to_buffer();

        tools::assert_no_overflow(&rendered, 160);
    }

    #[test]
    fn test_pager_indicators_visible_when_columns_hidden() {
        let result = create_wide_result(30, 5);
        let config = PagerConfig::default();
        let mut pager = Pager::new(&result, &config);
        pager.term_width = 80; // Force hiding columns

        let rendered = pager.render_to_buffer();

        // Should show right indicator (columns hidden to the right)
        assert!(rendered.contains("-->"), "Should show right scroll indicator");
    }
}
```

**Files to Create:**
- `tests/pager_width_validation_tests.rs` (~200 lines)

**Success Criteria:**
- [ ] All dimensional tests pass (80, 117, 120, 160 widths)
- [ ] No overflow detected by visual_validator
- [ ] Indicators appear when columns hidden
- [ ] Track 3 utilities successfully validate rendered output

### Phase 3: Manual Terminal Validation (MANDATORY)

**Objective:** Prove pager works in real terminal environments

**CRITICAL:** This phase is BLOCKING. If manual validation fails, Option A has failed regardless of automated test results.

#### Manual Test Script

Create `tests/manual/pager_validation.sh`:

```bash
#!/bin/bash
# Manual pager validation script
# Sprint 31: Option A - Fix Pager

set -e

# Check database connection available
if [ -z "$TQ_LOGON" ]; then
    echo "ERROR: TQ_LOGON not set in .env"
    exit 1
fi

echo "=== Sprint 31: Manual Pager Validation ==="
echo ""

# Test widths: 80, 117, 120, 160
WIDTHS=(80 117 120 160)

for WIDTH in "${WIDTHS[@]}"; do
    echo "=== Testing at terminal width $WIDTH ==="

    OUTPUT_FILE="/tmp/pager_test_${WIDTH}.txt"

    # Resize terminal (macOS iTerm2/Terminal.app)
    printf '\e[8;40;'$WIDTH't'
    sleep 0.5

    # Run test query with pager enabled
    # Use SAMPLE to get wide result set
    echo "SELECT * FROM dbc.databases SAMPLE 50;" | \
        script -q "$OUTPUT_FILE" \
        cargo run --release -- repl --logon "$TQ_LOGON"

    # Analyze output
    MAX_LINE=$(awk '{ if (length > max) max = length } END { print max }' "$OUTPUT_FILE")
    echo "  Max line width: $MAX_LINE"
    echo "  Terminal width: $WIDTH"

    if [ "$MAX_LINE" -gt "$WIDTH" ]; then
        echo "  ❌ FAIL: Overflow detected ($MAX_LINE > $WIDTH)"
        exit 1
    else
        echo "  ✅ PASS: No overflow"
    fi

    echo ""
done

echo "=== All terminal widths passed ==="
echo ""
echo "Please manually verify in terminal:"
echo "  1. Arrow keys navigate columns"
echo "  2. Indicators show hidden columns"
echo "  3. Output is readable (not garbled)"
echo "  4. 'q' returns to REPL"
echo ""
echo "If all checks pass, Option A is successful."
```

#### Manual Validation Checklist

Sprint coordinator MUST complete this checklist:

| Terminal Width | Functionality | Status | Evidence File |
|---------------|---------------|--------|--------------|
| 80 chars | Table renders, indicators work | ☐ Pass / ☐ Fail | `/tmp/pager_test_80.txt` |
| 117 chars | Table renders (user-reported width) | ☐ Pass / ☐ Fail | `/tmp/pager_test_117.txt` |
| 120 chars | Table renders | ☐ Pass / ☐ Fail | `/tmp/pager_test_120.txt` |
| 160 chars | More columns visible | ☐ Pass / ☐ Fail | `/tmp/pager_test_160.txt` |

**Additional Manual Checks:**

- [ ] Arrow keys navigate columns (left/right work)
- [ ] Indicators (`<--` and `-->`) appear when columns hidden
- [ ] Output is readable (no garbled characters)
- [ ] `q` returns to REPL (doesn't exit program)
- [ ] No visual artifacts or misalignment
- [ ] Column headers align with data

**Evidence Capture:**

All evidence files MUST be included in sprint review:
- `/tmp/pager_test_80.txt`
- `/tmp/pager_test_117.txt`
- `/tmp/pager_test_120.txt`
- `/tmp/pager_test_160.txt`

### Phase 4: Build Verification

**Standard checks:**

```bash
# Clean build
cargo clean
cargo build --release

# Clippy
cargo clippy -- -D warnings

# All tests
cargo test --lib
cargo test --test '*'
```

**Success Criteria:**
- [ ] Clean build (no warnings)
- [ ] Clippy passes (no warnings)
- [ ] All unit tests pass
- [ ] All dimensional tests pass

### Option A Success Criteria Summary

Option A is successful ONLY if ALL criteria are met:

1. ✅ Unit tests pass (render_to_buffer functionality)
2. ✅ Dimensional tests pass (80, 117, 120, 160 widths)
3. ✅ Build verification passes (clippy, no warnings)
4. ✅ **Manual terminal validation PASSES** (all 4 widths + functionality checks)
5. ✅ Evidence files capture proof of functionality
6. ✅ Pager enabled by default (`pager_enabled: true`)

**If ANY criterion fails, especially manual validation, Option A has FAILED.**

---

## Track 2: Pager Resolution - Option B (Remove Pager)

### Test Strategy Overview

**If Option B (Remove Pager) is chosen:**

Validation focuses on:

1. **Regression tests** - Ensure existing features still work
2. **Build verification** - Confirm clean compilation after removal
3. **Reference verification** - All pager references removed/stubbed
4. **Documentation verification** - Docs updated to reflect removal

**NO MANUAL VALIDATION REQUIRED** for Option B.

### Phase 1: Regression Tests

**Objective:** Verify existing features unaffected by pager removal

**Test Execution:**

```bash
# Run full test suite
cargo test --lib
cargo test --test '*'

# Run interactive tests
cargo test --test interactive_tests -- --ignored

# Check for test failures
cargo test 2>&1 | grep -i "test result"
```

**Expected Results:**
- All non-pager tests continue to pass
- No new test failures introduced
- Test count decreases (pager tests removed)

**Success Criteria:**
- [ ] All unit tests pass (excluding removed pager tests)
- [ ] All integration tests pass
- [ ] All interactive tests pass (non-pager features)
- [ ] No regressions detected

### Phase 2: Build Verification

**Objective:** Confirm code compiles cleanly after removal

**Build Checks:**

```bash
# Clean build
cargo clean
cargo build

# Check for warnings
cargo build 2>&1 | grep -i warning

# Clippy check
cargo clippy -- -D warnings

# Check for unused code
cargo clippy -- -W dead_code
```

**Success Criteria:**
- [ ] `cargo build` succeeds with zero warnings
- [ ] `cargo clippy` passes with zero warnings
- [ ] No dead code warnings (pager code fully removed)
- [ ] No unused imports related to pager

### Phase 3: Reference Verification

**Objective:** Ensure all pager references removed or properly stubbed

**Manual Grep Audit:**

```bash
# Search for pager references in source
grep -r "pager" src/ --exclude-dir=target

# Expected results:
# - src/commands/repl/pager.rs: Stub module with documentation
# - src/commands/repl/executor.rs: Simplified (no pager integration)
# - src/commands/repl/state.rs: No pager_enabled field
# - src/commands/repl/metacommands.rs: /pager command prints deprecation

# Search for should_page, display_with_pager
grep -r "should_page\|display_with_pager" src/

# Expected: Only in pager.rs stub

# Search for PagerConfig
grep -r "PagerConfig" src/

# Expected: Only in pager.rs stub
```

**Files to Verify:**

| File | Expected State | Verification |
|------|---------------|-------------|
| `src/commands/repl/pager.rs` | Stub (~50 lines) with docs | ☐ Verified |
| `src/commands/repl/executor.rs` | Pager integration removed | ☐ Verified |
| `src/commands/repl/state.rs` | `pager_enabled` field removed | ☐ Verified |
| `src/commands/repl/metacommands.rs` | `/pager` prints deprecation | ☐ Verified |

**Success Criteria:**
- [ ] All pager code removed or stubbed
- [ ] No orphaned imports
- [ ] No dead code paths
- [ ] Stub module documents removal rationale

### Phase 4: Documentation Verification

**Objective:** Confirm documentation reflects pager removal

**Files to Verify:**

| File | Required Update | Verification |
|------|----------------|-------------|
| `docs/design/repl.md` | Pager section removed/updated | ☐ Verified |
| `docs/specifications/repl.md` | Note pager not supported | ☐ Verified |
| `docs/roadmap/status.md` | Pager feature status updated | ☐ Verified |

**Content Verification:**

`docs/specifications/repl.md` should include:

```markdown
### Result Display

Results are displayed in formatted tables with:
- Colored headers (when colors enabled)
- Aligned columns
- Row counts and timing information

**Note:** Built-in result paging is not currently supported. For large results:
- Use `SAMPLE N` or `TOP N` in queries to limit rows
- Use `/export` to save results to file
- Pipe output to external pager: `tq repl | less -S`
```

**Success Criteria:**
- [ ] Design docs updated
- [ ] Specifications updated
- [ ] Status dashboard updated
- [ ] Alternative approaches documented

### Phase 5: Track 3 Utilities Decision

**Objective:** Verify Track 3 utilities documented or removed

**Option 1: Keep Utilities (RECOMMENDED)**

- Add documentation in `tests/tools/README.md`:
  ```markdown
  ## Terminal Validation Utilities

  Sprint 30 developed dimensional validation utilities retained for future use:
  - `visual_validator.rs` - Terminal width assertions
  - `terminal_simulator.rs` - Terminal simulation

  These utilities can validate any output requiring terminal dimension constraints.
  ```

**Option 2: Remove Utilities**

- Remove `tests/tools/visual_validator.rs`
- Remove `tests/tools/terminal_simulator.rs`
- Update references in test files

**Recommended:** Keep utilities (no runtime cost, potential future value)

**Success Criteria:**
- [ ] Decision documented (keep or remove)
- [ ] If keep: Documentation added to `tests/tools/`
- [ ] If remove: Files deleted, references updated

### Option B Success Criteria Summary

Option B is successful if ALL criteria are met:

1. ✅ All regression tests pass
2. ✅ Build verification passes (zero warnings)
3. ✅ All pager references removed/stubbed
4. ✅ Documentation updated
5. ✅ Track 3 utilities decision documented
6. ✅ `/pager` command provides helpful message

**NO MANUAL VALIDATION REQUIRED** for Option B.

---

## Quality Validator Verdict Framework

### Verdict Categories

For Sprint 31, quality-validator provides **ADVISORY VERDICT** only:

**ADVISORY PASS**: Tests executed successfully, quality standards met
**ADVISORY CONCERNS**: Tests passed but quality observations noted
**ADVISORY FAIL**: Tests failed or significant quality issues detected

**CRITICAL:** Sprint coordinator makes final approval decision, especially for Option A which requires manual validation.

### Option A Verdict Format

```
ADVISORY VERDICT: [PASS/CONCERNS/FAIL]

=== Automated Test Results ===
Unit Tests (render_to_buffer): X/X pass
Dimensional Tests (width validation): X/X pass
Build Verification: [PASS/FAIL]
Clippy: [PASS/FAIL]

=== Manual Validation Status ===
Terminal Testing: [REQUIRED - Coordinator must complete]
Evidence Files: [Present/Missing]
  - /tmp/pager_test_80.txt: [Yes/No]
  - /tmp/pager_test_117.txt: [Yes/No]
  - /tmp/pager_test_120.txt: [Yes/No]
  - /tmp/pager_test_160.txt: [Yes/No]

Manual Checklist: [Completed by coordinator/Pending]

=== Quality Observations ===
[Detailed observations about test quality, coverage, etc.]

=== Recommendation ===
[APPROVE/BLOCK with clear reasoning]

CRITICAL NOTE: This is an advisory verdict. Final approval requires
coordinator's manual validation of pager functionality in real terminal.
Automated tests are INSUFFICIENT to approve Option A.
```

### Option B Verdict Format

```
ADVISORY VERDICT: [PASS/CONCERNS/FAIL]

=== Test Results ===
Regression Tests: X/X pass
Build Verification: [PASS/FAIL]
Clippy: [PASS/FAIL]
Reference Audit: [CLEAN/ISSUES FOUND]

=== Documentation Verification ===
Design docs updated: [YES/NO]
Specifications updated: [YES/NO]
Status updated: [YES/NO]

=== Code Quality ===
Dead code removed: [YES/NO]
Stub properly documented: [YES/NO]
/pager command helpful: [YES/NO]

=== Track 3 Decision ===
Utilities: [KEPT/REMOVED]
Documentation: [ADDED/N/A]

=== Quality Observations ===
[Detailed observations]

=== Recommendation ===
[APPROVE/BLOCK with clear reasoning]

NOTE: Option B requires no manual validation. If automated tests pass
and documentation is complete, recommend APPROVE.
```

---

## Testing Limitations Acknowledged

This test strategy explicitly acknowledges limitations learned from Sprint 29 and Sprint 30:

### What Automated Tests CAN Validate

✅ Code compiles
✅ Unit logic correctness
✅ API contracts
✅ String width calculations
✅ Configuration handling
✅ Regression detection

### What Automated Tests CANNOT Validate

❌ Visual rendering in real terminals
❌ Interactive navigation usability
❌ Actual user experience
❌ Terminal-specific rendering quirks
❌ Readability of output

### Critical Lesson from Sprint 29/30

**100% automated test pass rate DOES NOT guarantee feature functionality.**

Sprint 29: 386/386 tests passed → feature completely broken
Sprint 30: 449/449 tests passed → feature still broken

**For Sprint 31 Option A:** Manual validation is NOT optional. It is MANDATORY and BLOCKING.

---

## Timeline

### Option A Timeline

- Phase 1 (Unit tests): 1 hour
- Phase 2 (Dimensional tests): 1.5 hours
- Phase 3 (Manual validation): 2 hours (CRITICAL)
- Phase 4 (Build verification): 0.5 hours
- **Total: 5 hours**

### Option B Timeline

- Phase 1 (Regression tests): 0.5 hours
- Phase 2 (Build verification): 0.5 hours
- Phase 3 (Reference verification): 0.5 hours
- Phase 4 (Documentation verification): 0.5 hours
- Phase 5 (Track 3 decision): 0.5 hours
- **Total: 2.5 hours**

---

## Success Metrics

### Option A Success (Fix Pager)

**REQUIRED for APPROVE verdict:**

1. ✅ All automated tests pass (100%)
2. ✅ Build clean (zero warnings)
3. ✅ **All manual validation checks PASS**
4. ✅ Evidence files captured
5. ✅ Pager enabled by default
6. ✅ Coordinator personally verifies feature works

**If ANY criterion fails: REJECT**

### Option B Success (Remove Pager)

**REQUIRED for APPROVE verdict:**

1. ✅ All regression tests pass
2. ✅ Build clean (zero warnings)
3. ✅ All references removed/stubbed
4. ✅ Documentation complete
5. ✅ Track 3 decision documented

**If ANY criterion fails: REJECT**

---

## Appendix: Test File Locations

### Option A Test Files

```
tests/
├── pager_render_buffer_tests.rs       # NEW: render_to_buffer unit tests
├── pager_width_validation_tests.rs    # NEW: dimensional tests with Track 3
└── manual/
    └── pager_validation.sh            # NEW: manual validation script

Evidence files (git-ignored):
/tmp/pager_test_80.txt
/tmp/pager_test_117.txt
/tmp/pager_test_120.txt
/tmp/pager_test_160.txt
```

### Option B Test Files

No new test files required. Verification via existing test suite.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial test strategy for Sprint 31 Options A and B | quality-validator |
