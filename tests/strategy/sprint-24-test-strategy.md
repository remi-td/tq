# Sprint 24 Test Strategy: REPL History Enhancement & Process Improvements

**Created:** 2026-01-27
**Author:** quality-validator
**Sprint:** Sprint 24
**Features:** Multi-line Command History (P0), Documentation Accuracy Verification (P0), Fix Sprint 23 Documentation Issues (P1)

---

## Executive Summary

Sprint 24 requires a **hybrid testing approach** combining automated tests with manual validation. Feature 1 (Multi-line Command History) presents **EXTREMELY HIGH false positive risk** due to keyboard interaction behavior, requiring manual validation as PRIMARY test method (Sprint 21 pattern).

**Critical Decision:** Following Sprint 21 precedent, automated tests will validate LOGIC and DATA (history grouping, persistence, recall), but NOT keyboard UX (↑/↓ navigation within multi-line commands). Manual validation is MANDATORY for keyboard behavior.

**Test Strategy:**
- Feature 1: Hybrid (unit + integration + PTY + **MANUAL PRIMARY**)
- Feature 2: Process validation (checklist execution)
- Feature 3: Documentation review (manual verification)

**Risk Assessment:**
- Feature 1: **EXTREMELY HIGH** false positive risk (keyboard UX)
- Feature 2: **LOW** risk (process improvement)
- Feature 3: **LOW** risk (documentation correction)

**Estimated Test Count:**
- Unit tests: 12-15
- Integration tests: 3-4
- PTY tests: 2-3
- Manual validation procedures: 3-4
- **Total: 20-26 tests**

---

## Feature-by-Feature Test Strategy

### Feature 1: Multi-line Command History (#3)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/repl.md` lines 62-143 (Command History)
- Specific requirement: Lines 135-137 "Multi-line SQL stored as single history entry" and "Recalled as complete multi-line block on ↑ arrow"
- Sprint Planning: `docs/sprints/sprint-24-planning.md` lines 41-68

**Requirements:**
1. "Multi-line SQL statements stored as single history entry (grouped until `;` terminator)" - Sprint 24 Planning line 49
2. "↑/↓ arrows recall complete multi-line commands, not individual lines" - Sprint 24 Planning line 50
3. "Cursor navigation works within recalled multi-line commands (↑/↓ moves between lines within command)" - Sprint 24 Planning line 51
4. "History file format unchanged (backward compatible with existing `~/.tq_history`)" - Sprint 24 Planning line 52
5. "All existing history features still work (search, deduplication, exclusions)" - Sprint 24 Planning line 54

**Feature Characteristics:**

**User Interaction Type:** ✅ **Interactive PTY** (REPL terminal UI with keyboard navigation)

**Explanation:** This is a REPL keyboard interaction feature. Users type multi-line SQL, press ↑ arrow to recall, and navigate within the recalled command. This is fundamentally an interactive terminal feature requiring PTY testing.

**Observable Behavior:**
- ✅ Visual output in terminal (multi-line display, cursor position)
- ✅ Structured data output (history file persistence)
- ✅ File system side effects (`~/.tq_history` file modified)
- ✅ State management (history buffer in memory, session persistence)

**External Dependencies:**
- ✅ Database connection (requires live database for REPL)
- ✅ File system access (reads/writes `~/.tq_history`)
- ✅ Terminal/PTY (keyboard input, cursor positioning, multi-line display)

**Validation Challenges:**
1. **Keyboard Navigation Testing** - PTY tests CANNOT reliably distinguish between:
   - ↑ arrow recalling entire multi-line command vs single line
   - ↓ arrow moving within command vs advancing to next history entry
   - Cursor position after recall
   - Visual rendering of multi-line block

2. **Backward Compatibility** - Must work with existing history files without data loss or format corruption

3. **Edge Cases** - Multi-line SQL with comments, strings containing `;`, escaped characters

4. **Integration with reedline** - History behavior controlled by reedline library, not pure logic

**Critical Behaviors to Validate:**
1. Multi-line grouping logic - "Lines are grouped into statements until `;` terminator encountered" (Sprint 24 Planning line 49)
2. History persistence - "Multi-line command written as single entry to `~/.tq_history`"
3. History recall - "↑ arrow loads complete multi-line block into input buffer"
4. Backward compatibility - "Existing `~/.tq_history` files work without modification" (Sprint 24 Planning line 52)
5. Keyboard navigation UX - "User can navigate within recalled multi-line command" (Sprint 24 Planning line 51) **- MANUAL ONLY**

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" checked:
  → Interactive tests (PTY) REQUIRED
  Reason: Unit tests cannot validate terminal output, keyboard behavior, cursor position

✅ "File system side effects" checked:
  → Integration tests REQUIRED
  Reason: Must verify actual file writes to ~/.tq_history

✅ "State management" checked:
  → Unit tests REQUIRED
  Reason: History grouping logic needs validation in isolation

✅ "Keyboard navigation" behavior:
  → Manual validation REQUIRED (PRIMARY)
  Reason: PTY tests have EXTREMELY HIGH false positive risk for keyboard UX
  Reference: Sprint 21 review - "Feature 3 has EXTREMELY HIGH false positive risk"
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Multi-line SQL statement grouping logic (until `;` terminator)
- **Approach:** Mock input streams with multi-line SQL, verify grouping algorithm produces single history entry
- **Rationale:** Core logic must be correct before integration testing. Unit tests catch edge cases (comments, strings with `;`, escaped chars)
- **Gap if missing:** Logic bugs (wrong grouping, off-by-one errors) not caught until expensive PTY testing
- **Necessity:** ✅ **REQUIRED**

**Test Type 2: Integration Tests**
- **Validates:** History persistence to `~/.tq_history` file
- **Approach:** Execute REPL with multi-line commands, exit cleanly, verify file contains single entry per command
- **Rationale:** Actual file I/O must work correctly, backward compatibility verified with real history files
- **Gap if missing:** File format issues, data corruption, compatibility breaks not caught
- **Necessity:** ✅ **REQUIRED**

**Test Type 3: PTY Tests (expectrl)**
- **Validates:** REPL behavior with multi-line input and history recall (data/output validation)
- **Approach:** Send multi-line SQL via expectrl, send ↑ arrow, capture output, verify multi-line content recalled
- **Rationale:** End-to-end validation that feature works in realistic terminal environment
- **Gap if missing:** Integration issues between history logic and REPL not caught
- **Necessity:** ✅ **REQUIRED**
- **CRITICAL LIMITATION:** PTY tests CANNOT validate keyboard navigation UX (see below)

**Test Type 4: Manual Validation (PRIMARY for Keyboard UX)**
- **Validates:** User-perceived keyboard navigation behavior (↑/↓ arrows, cursor position, visual rendering)
- **Approach:** Human tester executes REPL, types multi-line SQL, uses ↑/↓ arrows, verifies smooth UX
- **Rationale:** PTY tests have **EXTREMELY HIGH false positive risk** for keyboard behavior. Sprint 21 lesson: "Feature 3 had EXTREMELY HIGH false positive risk. PTY tests CANNOT validate TAB vs ENTER vs DOWN arrow behavior. Manual validation is PRIMARY test, automated tests are secondary."
- **Gap if missing:** Keyboard UX bugs ship to users (e.g., ↑ recalls one line instead of entire command)
- **Necessity:** ✅ **REQUIRED** (PRIMARY validation for keyboard behavior)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates statement grouping logic | Logic bugs, edge cases not caught | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates file persistence, backward compatibility | File format bugs, data corruption | MUST IMPLEMENT |
| PTY tests (expectrl) | ✅ REQUIRED | Validates REPL integration and data recall | Integration issues not caught | MUST IMPLEMENT |
| **Manual validation** | ✅ **REQUIRED (PRIMARY)** | **Validates keyboard UX that PTY cannot** | **Keyboard navigation bugs ship** | **MUST EXECUTE** |
| Benchmark tests | ❌ NOT NEEDED | No performance requirements in spec | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 4 (unit + integration + PTY + manual)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 1 (benchmark)

**CRITICAL:** Manual validation is **PRIMARY** for keyboard behavior, automated tests are **SECONDARY**.

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| REQ-HIST-1 | "Multi-line SQL statements stored as single history entry" | sprint-24-planning.md:49 | Unit + Integration | Unit validates logic, integration validates file format | TC094, TC095, TC099 |
| REQ-HIST-2 | "↑/↓ arrows recall complete multi-line commands, not individual lines" | sprint-24-planning.md:50 | PTY + **Manual** | PTY validates data recall, **manual validates UX** | TC096, **Manual-1** |
| REQ-HIST-3 | "Cursor navigation works within recalled multi-line commands" | sprint-24-planning.md:51 | **Manual ONLY** | **PTY cannot validate cursor behavior** | **Manual-2** |
| REQ-HIST-4 | "History file format unchanged (backward compatible)" | sprint-24-planning.md:52 | Integration | Test with existing history files | TC100 |
| REQ-HIST-5 | "All existing history features still work (search, deduplication)" | sprint-24-planning.md:54 | Integration + PTY | Verify no regressions | TC101, TC102 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types

**Coverage Gaps:**
- REQ-HIST-3 (cursor navigation) has NO automated coverage - **acceptable** because PTY tests cannot validate this (Sprint 21 lesson)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests**
- **Reason for omission:** Specification has no performance requirements for history operations
- **What won't be validated:** History recall speed, memory usage for large history files
- **Risk assessment:** LOW - History operations are fast (<10ms), not performance-critical
- **Mitigation:** Monitor in production, add benchmarks if performance issues reported
- **Revisit criteria:** If users report slowness with large history files (>10,000 entries)

**Cross-Platform Tests (Windows/macOS/Linux)**
- **Reason for omission:** History feature uses standard Rust file I/O, reedline handles platform differences
- **What won't be validated:** Platform-specific file path issues, line ending differences
- **Risk assessment:** LOW - reedline is battle-tested across platforms
- **Mitigation:** Test on Linux (primary platform), rely on reedline's cross-platform support
- **Revisit criteria:** If users report platform-specific history bugs

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/history.rs` test module (or similar)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8-10 tests
- **Key scenarios to cover:**
  1. Single-line SQL → single history entry
  2. Multi-line SQL (2-5 lines) → single entry with newlines preserved
  3. Edge case: SQL with comment containing `;` → not treated as terminator
  4. Edge case: SQL with string containing `;` → not treated as terminator
  5. Edge case: Empty lines within multi-line SQL → preserved
  6. Edge case: Multiple statements with `;` → multiple history entries
  7. Backward compatibility: Parse existing history file format correctly
  8. Deduplication: Consecutive identical multi-line commands → single entry
- **Mocking strategy:** Mock file I/O (use in-memory buffer), test pure logic

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or `tests/history_integration.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 3-4 tests
- **Key scenarios to cover:**
  1. Write multi-line command to history file, verify format
  2. Read existing history file with multi-line entries, verify correct parsing
  3. Backward compatibility: Load old history file format without errors
  4. Stress test: 10,000-entry history file, verify no corruption
- **Setup requirements:**
  - Temporary directory for test history files
  - Pre-existing history files with known content (fixtures)
  - Clean up after tests

**Test Type: PTY Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 2-3 tests
- **Key scenarios to cover:**
  1. Type multi-line SQL, exit REPL, restart, press ↑, verify multi-line content appears
  2. Type multiple multi-line commands, press ↑ repeatedly, verify correct recall order
  3. Ctrl-R search finds multi-line commands correctly
- **Implementation notes:**
  - Use temporary history file (avoid polluting user's `~/.tq_history`)
  - Timing considerations: expect() calls may need longer timeouts for multi-line input
  - Output validation: Check that recalled text contains newlines, not just data content
- **CRITICAL LIMITATION:** PTY tests validate DATA (multi-line content recalled) but NOT UX (keyboard navigation feels correct)

**Test Type: Manual Validation Procedures**
- **Location:** `tests/cases/TC-MANUAL-HIST-*.md`
- **Framework:** Human tester with real REPL
- **Test count estimate:** 3-4 manual procedures
- **Key scenarios to cover:**
  1. **Manual-1:** Type 3-line SELECT statement, press ↑, verify ALL 3 lines appear (not just last line)
  2. **Manual-2:** Recall 3-line command, press ↓ within command, verify cursor moves within command (not to next history entry)
  3. **Manual-3:** Recall 3-line command, edit middle line, verify changes apply correctly
  4. **Manual-4:** Verify visual rendering looks correct (no formatting glitches, cursor visible)
- **Evidence requirements:**
  - Screenshots of REPL with multi-line commands visible
  - Description of keyboard actions and observed behavior
  - Pass/Fail verdict for each scenario
- **Time estimate:** 15-20 minutes total for all manual procedures

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Statement grouping logic, edge cases (comments, strings), deduplication
- Integration tests validate: File persistence, backward compatibility, existing history file loading
- PTY tests validate: REPL integration, multi-line content recall, search functionality
- Manual validation validates: Keyboard navigation UX (↑/↓ arrows, cursor position, visual rendering)

**Combined coverage: COMPREHENSIVE**

**Gaps in combined coverage:**
- Cross-platform compatibility not tested (acceptable - rely on reedline)
- Performance/stress testing not included (acceptable - no requirements)
- Long-term stability not tested (acceptable - no soak test requirements)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Cross-platform gap acceptable because reedline handles platform differences, history uses standard Rust I/O
- Performance gap acceptable because no performance requirements in specification, operations are fast
- Manual validation gap (PTY limitations) acceptable because manual procedures defined and will be executed

**VERDICT:** If automated tests pass (100%) AND manual validation passes (4/4 procedures), we can claim feature "works as specified."

---

### Feature 2: Documentation Accuracy Verification (P0)

#### 1. Specification Analysis

**Specification References:**
- Sprint Planning: `docs/sprints/sprint-24-planning.md` lines 72-91
- Sprint 22 Review: Section 5 (UX Review) - Pattern syntax documentation mismatch
- Sprint 23 Review: Section 7 (Lessons Learned) - `--force` flag documented but not implemented

**Requirements:**
1. "Phase 4 process document updated with documentation verification checklist" - Sprint 24 Planning line 83
2. "Verification covers user guides, specifications, and examples" - Sprint 24 Planning line 84
3. "Sprint coordinator executes verification before final commit" - Sprint 24 Planning line 85
4. "Process prevents shipping with doc/implementation mismatches" - Sprint 24 Planning line 86

**Feature Characteristics:**

**User Interaction Type:** ⏸️ **None** - This is a process improvement, not user-facing feature

**Explanation:** This is a documentation quality gate added to Phase 4 (Ship). No code changes, no user interaction. Validation is process execution verification.

**Observable Behavior:**
- Phase 4 checklist includes documentation verification step
- Documentation matches actual delivered features

**Validation Challenges:**
- Cannot automate "documentation accuracy" - requires human judgment
- Must verify across multiple document types (specifications, user guides, examples)

**Critical Behaviors to Validate:**
1. Phase 4 process document updated - "Checklist includes doc verification"
2. Verification executed - "Sprint coordinator performs verification before commit"
3. Mismatches caught - "No documented-but-unimplemented features ship"

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
❌ "Interactive PTY" NOT checked: Process improvement, no user interaction
❌ "Database connection" NOT checked: Documentation validation only
✅ "Manual verification" REQUIRED: Human judgment needed for documentation accuracy
```

**Derived Test Types:**

**Test Type 1: Process Validation (Manual)**
- **Validates:** Phase 4 process document updated with verification checklist
- **Approach:** Review `.claude/skills/sprint-coordinator/process/phase4-ship.md`, verify documentation verification step exists
- **Rationale:** Process improvement must be documented and visible to sprint coordinator
- **Gap if missing:** No enforcement mechanism, mismatches continue in future sprints
- **Necessity:** ✅ **REQUIRED**

**Test Type 2: Execution Verification (Manual)**
- **Validates:** Sprint coordinator executes documentation verification in Sprint 24 Phase 4
- **Approach:** Observe Sprint 24 Ship phase, verify coordinator checks docs vs delivered features
- **Rationale:** Process improvement only works if executed
- **Gap if missing:** Process document exists but not followed
- **Necessity:** ✅ **REQUIRED**

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Manual process validation | ✅ REQUIRED | Verify process document updated | Process improvement not documented | MUST VERIFY |
| Manual execution verification | ✅ REQUIRED | Verify coordinator executes checklist | Process not followed | MUST OBSERVE |
| Automated tests | ❌ NOT NEEDED | No code changes, process only | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| REQ-DOC-1 | "Phase 4 process document updated" | sprint-24-planning.md:83 | Manual verification | Process document review | Manual-DOC-1 |
| REQ-DOC-2 | "Verification covers user guides, specifications, examples" | sprint-24-planning.md:84 | Manual execution | Observe Sprint 24 Ship | Manual-DOC-2 |
| REQ-DOC-3 | "Sprint coordinator executes verification before final commit" | sprint-24-planning.md:85 | Manual observation | Verify in Sprint 24 | Manual-DOC-3 |

#### 5. Gap Analysis

**No test types omitted** - Process validation is inherently manual.

#### 6. Test Implementation Plan

**Manual Procedure 1: Verify Process Document Updated**
- **Location:** `tests/cases/TC-MANUAL-DOC-PROCESS.md`
- **Steps:**
  1. Read `.claude/skills/sprint-coordinator/process/phase4-ship.md`
  2. Verify "Documentation Accuracy Verification" section exists
  3. Verify checklist includes: specifications, user guides, examples
  4. Verdict: PASS if section exists with comprehensive checklist, FAIL otherwise

**Manual Procedure 2: Observe Sprint 24 Ship Phase**
- **Location:** Sprint 24 Phase 4 execution
- **Steps:**
  1. Observe sprint coordinator during Phase 4
  2. Verify coordinator executes documentation verification
  3. Verify coordinator checks each delivered feature against documentation
  4. Verify any mismatches are caught and corrected before commit
  5. Verdict: PASS if verification executed, FAIL if skipped

---

### Feature 3: Fix Sprint 23 Documentation Issues (P1)

#### 1. Specification Analysis

**Specification References:**
- Sprint Planning: `docs/sprints/sprint-24-planning.md` lines 95-115
- Sprint 23 Review: Section 5 (UX Review), Section 7 (Recommendations)

**Issues:**
1. `--force` flag documented in `docs/specifications/batch-mode.md` and `docs/user/batch-mode-guide.md` but not implemented
2. Teradata session type compatibility needs better documentation

**Requirements:**
1. "Remove `--force` flag documentation from specifications and user guide" - Sprint 24 Planning line 106
2. "Add Teradata session type compatibility section to user guide" - Sprint 24 Planning line 107
3. "Update error messages for transaction control to explain session limitations" - Sprint 24 Planning line 108

**Feature Characteristics:**

**User Interaction Type:** ⏸️ **None** - Documentation corrections only

**Explanation:** This is a documentation accuracy fix. No code changes required (except minor error message improvement). Validation is documentation review.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
❌ "Code changes" NOT significant: Only error message improvement
✅ "Documentation verification" REQUIRED: Human review of documentation changes
```

**Derived Test Types:**

**Test Type 1: Documentation Review (Manual)**
- **Validates:** `--force` flag removed from all documentation
- **Approach:** Search `docs/specifications/batch-mode.md` and `docs/user/batch-mode-guide.md` for `--force`, verify removed
- **Rationale:** Documentation accuracy is the entire feature
- **Gap if missing:** Documented feature still misleads users
- **Necessity:** ✅ **REQUIRED**

**Test Type 2: Documentation Completeness (Manual)**
- **Validates:** Teradata session compatibility section added to user guide
- **Approach:** Read user guide, verify section explains DBC/SQL vs BTEQ vs TeraSQL, provides workarounds
- **Rationale:** Users need guidance on transaction control limitations
- **Gap if missing:** Users get cryptic Error 3706 without explanation
- **Necessity:** ✅ **REQUIRED**

**Test Type 3: Error Message Validation (Unit Test)**
- **Validates:** Transaction control error messages mention session limitations
- **Approach:** Unit test triggering Error 3706, verify error message includes troubleshooting guidance
- **Rationale:** Improved error messages help users diagnose issues
- **Gap if missing:** Error message remains cryptic
- **Necessity:** ⚠️ **RECOMMENDED** (nice to have, not blocking)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Manual documentation review | ✅ REQUIRED | Verify `--force` removed | Misleading documentation persists | MUST VERIFY |
| Manual completeness check | ✅ REQUIRED | Verify session compatibility section added | Users lack guidance | MUST VERIFY |
| Error message unit test | ⚠️ RECOMMENDED | Verify helpful error text | Error remains cryptic (acceptable) | SHOULD IMPLEMENT |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| REQ-FIX-1 | "Remove `--force` flag from documentation" | sprint-24-planning.md:106 | Manual review | Documentation accuracy check | Manual-FIX-1 |
| REQ-FIX-2 | "Add Teradata session compatibility section" | sprint-24-planning.md:107 | Manual review | Completeness check | Manual-FIX-2 |
| REQ-FIX-3 | "Update error messages for transaction control" | sprint-24-planning.md:108 | Unit test (optional) | Error message validation | TC103 (optional) |

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Automated Documentation Testing**
- **Reason for omission:** Documentation accuracy requires human judgment (e.g., "is this explanation clear?")
- **What won't be validated:** Prose quality, example correctness, completeness
- **Risk assessment:** MEDIUM - Manual review may miss subtle issues
- **Mitigation:** Use cli-ux-designer agent for documentation review (human-level judgment)
- **Revisit criteria:** If documentation bugs continue in Sprint 25+

#### 6. Test Implementation Plan

**Manual Procedure 1: Verify `--force` Removed**
- **Location:** `tests/cases/TC-MANUAL-FIX-FORCE.md`
- **Steps:**
  1. Search `docs/specifications/batch-mode.md` for `--force` or `-f`
  2. Search `docs/user/batch-mode-guide.md` for `--force` or `-f`
  3. Verify zero occurrences (or only in "deferred features" section)
  4. Verdict: PASS if removed, FAIL if still present

**Manual Procedure 2: Verify Session Compatibility Section**
- **Location:** `tests/cases/TC-MANUAL-FIX-SESSION.md`
- **Steps:**
  1. Read `docs/user/batch-mode-guide.md` "Transaction Control" section
  2. Verify explains DBC/SQL, BTEQ, TeraSQL session types
  3. Verify explains Error 3706 limitation
  4. Verify provides workarounds (remove --atomic, use BTEQ mode, manual transactions)
  5. Verdict: PASS if comprehensive, FAIL if missing

**Optional Unit Test: Error Message**
- **Location:** `src/error.rs` test module
- **Test:** Trigger transaction error, verify message includes "Teradata session type" and "Error 3706"
- **Priority:** LOW (error already works, this is usability improvement)

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: ✅ Feature 1 (8-10 tests), Feature 3 optional (1 test)
- Integration tests: ✅ Feature 1 (3-4 tests)
- PTY tests: ✅ Feature 1 (2-3 tests)
- Manual validation: ✅ **ALL FEATURES** (Feature 1: 4 procedures, Feature 2: 2 procedures, Feature 3: 2 procedures)

**Estimated Test Count:**
- Unit: 8-11 tests
- Integration: 3-4 tests
- PTY: 2-3 tests
- Manual: 8 procedures
- **Total: 21-26 tests/procedures**

**Risk Assessment:**
- **HIGH risk gaps:** Feature 1 keyboard UX (mitigated by manual validation)
- **MEDIUM risk gaps:** None
- **LOW risk gaps:** Cross-platform compatibility (acceptable - rely on reedline)

**Dependencies Required:**
- Live database: Yes (for PTY and integration tests)
- File system access: Yes (for history file tests)
- Terminal/PTY: Yes (for interactive tests)
- Specific OS: No (test on Linux, reedline handles cross-platform)

---

## Automation Limitations (Sprint 21 Pattern)

### What PTY Tests CAN Validate

✅ **Data Validation:**
- Multi-line content recalled correctly
- History file persistence (entries exist)
- Search functionality (Ctrl-R finds multi-line commands)
- Order of history entries

✅ **Output Content:**
- Text appears in output buffer
- Expected strings present
- Error messages displayed

### What PTY Tests CANNOT Validate

❌ **Keyboard Behavior:**
- ↑ arrow recalls entire command vs single line (user perception)
- ↓ arrow moves within command vs advances to next entry
- Cursor position after recall (visual)
- Line-by-line navigation within multi-line command

❌ **Visual Rendering:**
- Multi-line display formatting
- Cursor visibility and position
- Terminal color/highlighting
- Line wrapping behavior

❌ **User Experience:**
- "Smooth" navigation (subjective)
- "Intuitive" behavior (subjective)
- Timing/responsiveness perception

### Why Manual Validation is PRIMARY

**Sprint 21 Lesson (Feature 3 - Second TAB Accepts):**
> "Feature 3 has EXTREMELY HIGH false positive risk. PTY tests CANNOT validate TAB vs ENTER vs DOWN arrow behavior. Manual validation is PRIMARY test, automated tests are secondary."

**Applied to Sprint 24 Feature 1:**
- PTY tests can verify multi-line content is recalled (data check)
- PTY tests CANNOT verify keyboard navigation feels correct (UX check)
- **Manual validation is PRIMARY** for keyboard behavior
- Automated tests are SECONDARY (support evidence)

**Verdict Logic:**
- APPROVED: Automated PASS (100%) **AND** Manual PASS (all procedures) ✅
- REJECTED: Manual validation FAILED (keyboard UX broken) ❌
- BLOCKED: Manual validation NOT PERFORMED ⏸️

---

## Test Implementation Checklist

**Before submitting to rust-teradata-architect:**

Feature 1: Multi-line Command History
- [ ] Unit tests implemented (8-10 tests for grouping logic)
- [ ] Integration tests implemented (3-4 tests for file persistence)
- [ ] PTY tests implemented (2-3 tests for REPL integration)
- [ ] Manual validation procedures documented (4 procedures)
- [ ] All test types from strategy present
- [ ] No test gaps

Feature 2: Documentation Accuracy Verification
- [ ] Manual procedure for process document review
- [ ] Manual procedure for Sprint 24 Ship observation
- [ ] Process validation criteria defined

Feature 3: Fix Sprint 23 Documentation Issues
- [ ] Manual procedure for `--force` removal verification
- [ ] Manual procedure for session compatibility section review
- [ ] Optional error message unit test (if time permits)

**Critical Verification:**
- [ ] Manual validation procedures are detailed (not placeholders)
- [ ] Evidence requirements specified (screenshots, descriptions)
- [ ] Pass/Fail criteria clear and unambiguous
- [ ] Time estimates provided for manual procedures

---

## Verdict Criteria

### APPROVED Criteria

**Feature 1 (Multi-line Command History):**
- ✅ Unit tests: 100% pass rate (8-10/8-10)
- ✅ Integration tests: 100% pass rate (3-4/3-4)
- ✅ PTY tests: 100% pass rate (2-3/2-3)
- ✅ **Manual validation: 100% pass rate (4/4 procedures)** - **MANDATORY**
- ✅ Zero regressions (existing history features still work)

**Feature 2 (Documentation Accuracy Verification):**
- ✅ Process document updated (verified)
- ✅ Sprint coordinator executes verification in Phase 4 (observed)
- ✅ No doc/implementation mismatches in Sprint 24 deliverables

**Feature 3 (Fix Sprint 23 Documentation Issues):**
- ✅ `--force` removed from all documentation (verified)
- ✅ Session compatibility section added (complete and accurate)
- ⚠️ Error message improvement (optional, not blocking)

### REJECTED Criteria

**Feature 1:**
- ❌ Manual validation: Any procedure FAILS
  - ↑ arrow recalls single line instead of entire command
  - ↓ arrow navigation confusing/broken
  - Cursor position incorrect after recall
  - Visual rendering glitches
- ❌ Automated tests: <100% pass rate (logic bugs)
- ❌ Backward compatibility broken (existing history files corrupted)

**Feature 2:**
- ❌ Process document not updated
- ❌ Sprint coordinator skips verification in Phase 4

**Feature 3:**
- ❌ `--force` still present in documentation
- ❌ Session compatibility section missing or incomplete

### BLOCKED Criteria

**Feature 1:**
- ⏸️ Manual validation NOT PERFORMED (no human tester available)
- ⏸️ Live database unavailable (cannot run integration/PTY tests)

**Feature 2:**
- ⏸️ Phase 4 not yet reached (cannot observe execution)

**Feature 3:**
- ⏸️ cli-ux-designer agent unavailable (cannot review documentation)

---

## Strategy Validation Checklist

**Before submitting strategy to coordinator:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (not guessed)
- ✅ Every test type has clear rationale
- ✅ Gap analysis is complete and honest
- ✅ Specification coverage map includes all requirements
- ✅ Every requirement maps to at least one test type
- ✅ Test implementation plan is detailed and actionable
- ✅ Coverage sufficiency is assessed
- ✅ No hand-waving or vague justifications
- ✅ Sprint 21/23 lessons applied (hybrid testing, automation limitations documented)

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-27
**Review Status:** READY FOR REVIEW

**Submitted for Review:** 2026-01-27

**Key Points for Reviewer:**
1. Feature 1 has **EXTREMELY HIGH** false positive risk (keyboard UX)
2. Manual validation is **PRIMARY** for Feature 1 (Sprint 21 pattern)
3. Features 2 and 3 are process/documentation (manual verification only)
4. Total test count: 21-26 tests/procedures (13-18 automated, 8 manual)
5. Automation limitations explicitly documented

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Initial Sprint 24 test strategy | quality-validator |
