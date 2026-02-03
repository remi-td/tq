# Sprint 12 Planning: Critical Bug Verification + Export Enhancements + Branding

**Sprint Number:** 12
**Sprint Type:** Bug Verification + Feature Development
**Sprint Start Date:** 2026-01-19
**Target Duration:** 1-2 days
**Planning Date:** 2026-01-19

---

## Executive Summary

Sprint 12 addresses a critical process failure discovered from user feedback: **Sprint 11 fixes were committed but the binary was never rebuilt**, causing users to still experience the bugs that were supposedly fixed. Additionally, this sprint delivers high-priority export enhancements and adds essential branding elements.

### Critical Discovery

**Root Cause Analysis:**
- Sprint 11 committed fixes at `2026-01-18 20:10:36`
- Binary last built at `2026-01-18 20:05:00` (BEFORE the commit)
- User running `tq 1.6.0` binary WITHOUT Sprint 11 fixes
- Cargo.toml never updated to version `1.6.1`

**Impact:** User experiencing bugs that were already fixed in code but not deployed.

---

## Sprint Goals

### Primary Goal: Restore User Trust Through Proper Deployment
Deploy Sprint 11 fixes properly and verify they work in real usage. Add critical export features and branding to enhance tool usability and professionalism.

### Success Criteria
1. ✅ Binary rebuilt with Sprint 11 fixes and verified working
2. ✅ Tab completion shows database names (not keywords) after `FROM`
3. ✅ Cursor position bug fixed (if exists after rebuild)
4. ✅ Export to clipboard functionality working
5. ✅ Export full dataset to file (without default row limit)
6. ✅ Branding elements added (logo/welcome message)
7. ✅ 100% test pass rate maintained
8. ✅ User validation completed

---

## User Feedback Summary

### Critical Issues (from `docs/builder/incoming/open-bugs.md`)

**1. Tab Completion STILL DOESN'T WORK PROPERLY** 🔴 CRITICAL
```
User Report: "Completion doesn't make sense AGAIN!"
Screenshot: Shows "(SQL keyword)" repeated 25 times when typing "select * from "
Expected: Database names (DBC, SYSUDTLIB, etc.)
Actual: SQL keywords (AS, IN, ON, OR, ALL, AND, etc.)

Additional Issues:
- "selecting a keyword inserts it at the beginning of the current line instead of where my cursor is at"
- "These were right a few sprints ago!"
```

**Root Cause:** User running old binary (v1.6.0) built BEFORE Sprint 11 fixes were committed.

**Resolution:**
- Rebuild binary with Sprint 11 fixes
- Verify database name completion works
- Investigate cursor position insertion bug (may be separate issue)

---

**2. Export Needs Enhancements** 🟡 HIGH PRIORITY

```
User Requirements:
1. "Export should allow to export to clipboard"
2. "Export should allow to export ALL the dataset to a file"
   - Current behavior: Exports only first 100 rows (default limit)
   - Expected: When no user limit specified, export FULL dataset
   - Current correct: When user specifies "TOP 1000", exports 1000 rows
```

**Business Value:** Users need to extract complete datasets for analysis, not just first 100 rows.

---

**3. Branding Missing** 🟡 MEDIUM PRIORITY

```
User Report: "It's very sad but the tool has no logo, no welcome message at all"
Need: "bare minimum of brand identity when we start the tool"
Purpose: "so it can be presented to our clients and users"

Referenced: "This was discussed in your specifications and still there is zero progress"
```

**Business Impact:** Tool lacks professional appearance for client presentations.

---

## Sprint Scope

### Phase 1: Critical Bug Verification (P0)

**Objective:** Properly deploy Sprint 11 fixes and verify they work.

#### Task 1.1: Rebuild and Version Bump ✅ MUST DO
- Update `Cargo.toml` version to `1.6.1`
- Rebuild release binary: `cargo build --release`
- Verify binary reports correct version: `./target/release/tq --version`
- Commit version bump

**Acceptance Criteria:**
- Binary built after latest commit timestamp
- Version shows `tq 1.6.1`
- Binary includes Sprint 11 code changes

**Estimated Effort:** 10 minutes

---

#### Task 1.2: Manual Tab Completion Verification ✅ MUST DO
Test that Sprint 11 fixes actually work in real usage.

**Test Cases:**
1. **Database Name Completion:**
   ```
   $ ./target/release/tq repl
   tq> select * from [TAB]
   Expected: DBC, SYSUDTLIB, SYSUIF, other database names
   NOT Expected: SQL keywords
   ```

2. **Table Name Completion:**
   ```
   tq> select * from DBC.[TAB]
   Expected: TablesV, ColumnsV, DatabasesV, etc.
   NOT Expected: Keywords
   ```

3. **Column Name Completion:**
   ```
   tq> select * from DBC.TablesV where [TAB]
   Expected: DatabaseName, TableName, TableKind, etc.
   NOT Expected: Keywords
   ```

4. **Cursor Position Bug:**
   ```
   Test if selecting completion inserts at cursor position vs beginning of line
   ```

**Acceptance Criteria:**
- Tab completion shows database/table/column names (not keywords)
- No "(SQL keyword)" spam in inappropriate contexts
- Completions insert at correct cursor position
- Multi-line completion still works (Sprint 9 fix preserved)

**Estimated Effort:** 15 minutes (manual testing)

---

#### Task 1.3: Fix Cursor Position Bug (IF NEEDED)
Only if Task 1.2 reveals cursor position issues after rebuild.

**Acceptance Criteria:**
- Completions insert at cursor position, not beginning of line
- Works across all completion contexts
- Tests added to prevent regression

**Estimated Effort:** 1-2 hours (conditional)

---

### Phase 2: Export Enhancements (P1)

**Objective:** Enable clipboard export and full dataset export to files.

#### Task 2.1: Export to Clipboard Feature
Implement clipboard support for `/export` metacommand.

**Specification:**
```
Syntax:
  /export clipboard              -- Export last result to clipboard (table format)
  /export clipboard json         -- Export as JSON
  /export clipboard csv          -- Export as CSV

Behavior:
- Uses system clipboard (pbcopy on macOS, xclip on Linux, clip on Windows)
- Exports VISIBLE results (respects current row limit)
- Shows confirmation: "Exported N rows to clipboard (format)"
- Graceful error if clipboard unavailable: "Clipboard not available on this system"

Examples:
  tq> select top 10 * from DBC.TablesV;
  [Results shown]
  tq> /export clipboard csv
  Exported 10 rows to clipboard (CSV format)
```

**Dependencies:**
- Consider using `arboard` or `cli-clipboard` crate
- Cross-platform clipboard access
- Graceful degradation if clipboard unavailable

**Acceptance Criteria:**
- `/export clipboard` copies last result to system clipboard
- Supports table, json, csv formats
- Works on macOS (primary), Linux, Windows
- Clear error message if clipboard unavailable
- Help text updated: `/help` and `--help`
- Integration tests with clipboard mocking

**Estimated Effort:** 2-3 hours

---

#### Task 2.2: Export Full Dataset to File
Remove default row limit when exporting to file.

**Current Behavior:**
```
tq> select * from MyTable;
[Shows first 100 rows due to default limit]
tq> /export csv mydata.csv
[Only exports 100 rows - NOT what user wants!]
```

**Expected Behavior:**
```
tq> select * from MyTable;
[Shows first 100 rows in terminal]
tq> /export csv mydata.csv
[Re-executes query WITHOUT limit, exports ALL rows]
Exported 10,000 rows to mydata.csv
```

**Implementation Approach:**
1. **Track Query Source:**
   - User-specified limit: `SELECT TOP 1000 ...` → export 1000 rows
   - Default limit applied: `SELECT * ...` → export ALL rows

2. **Two Options:**
   - **Option A (Simple):** Re-execute query without limit when exporting to file
   - **Option B (Complex):** Cache full result set, show limited in terminal

3. **Recommendation:** Option A (re-execute)
   - Simpler implementation
   - No memory overhead for large result sets
   - Clear user expectation: file export = full data

**Specification:**
```
File Export Behavior:
- Query with user-specified limit: Export respects user limit
  Example: "SELECT TOP 1000 ..." exports 1000 rows

- Query with default limit: Export removes limit and exports ALL rows
  Example: "SELECT * FROM ..." exports entire table

- Display message with actual row count:
  "Exported 10,000 rows to mydata.csv (full dataset)"

- Warning for very large exports (optional):
  If estimated rows > 100K: "This table has ~500K rows. Export may take time. Continue? [y/N]"
```

**Acceptance Criteria:**
- `/export csv mydata.csv` exports FULL dataset when no user limit specified
- User-specified limits still respected (TOP, SAMPLE)
- Progress indicator for large exports (>10K rows)
- Clear messaging: "Exported N rows (full dataset)" vs "Exported N rows"
- Works in both REPL and batch mode
- Integration tests covering both scenarios
- Documentation updated

**Estimated Effort:** 3-4 hours

---

### Phase 3: Branding (P1)

**Objective:** Add professional branding elements for client presentation.

#### Task 3.1: Welcome Message / ASCII Logo
Add minimal branding when starting REPL.

**Specification:**
```
$ tq repl

 _____
|_   _|__ _
  | |/ _` |
  | | (_| |
  |_|\__, |    Teradata Query Tool
        |_|    v1.6.1

Type /help for commands, or enter SQL to execute.
Connected to: my_teradata_host

tq>
```

**Requirements:**
- Minimal ASCII art logo (simple, works in any terminal)
- Version number displayed
- Connection info shown
- Clean, professional appearance
- Not shown in batch mode or when stdin is piped
- Can be disabled: `--no-banner` flag

**Alternative (even simpler):**
```
$ tq repl
tq v1.6.1 - Teradata Query Tool
Type /help for commands or enter SQL to execute.
Connected to: my_teradata_host

tq>
```

**Acceptance Criteria:**
- Welcome message shown on REPL start (TTY only)
- Shows version, connection status
- Optional ASCII logo (user preference)
- Not shown in batch mode
- `--no-banner` flag to disable
- Professional appearance suitable for client demos

**Estimated Effort:** 1-2 hours

---

#### Task 3.2: Update Help Text and Metadata
Ensure all help text reflects professional branding.

**Updates:**
- `tq --help`: Add tagline "Teradata Query Tool - Fast, lightweight CLI client"
- `tq repl --help`: Mention branding
- README.md: Add logo/branding section
- Documentation: Consistent terminology

**Acceptance Criteria:**
- All help text includes tool tagline
- Consistent branding across documentation
- Professional tone throughout

**Estimated Effort:** 30 minutes

---

## Priorities and Scope

### Must Have (P0) - Sprint Fails Without These
- ✅ Task 1.1: Rebuild and version bump
- ✅ Task 1.2: Manual verification of Sprint 11 fixes
- ⚠️ Task 1.3: Fix cursor position bug (if needed)

### Should Have (P1) - High User Value
- ✅ Task 2.1: Export to clipboard
- ✅ Task 2.2: Export full dataset to file
- ✅ Task 3.1: Welcome message/branding

### Nice to Have (P2) - Can Defer
- Task 3.2: Help text updates (include if time permits)

---

## Risks and Mitigations

### Risk 1: Cursor Position Bug Exists After Rebuild
**Likelihood:** Medium
**Impact:** High (user frustration)

**Mitigation:**
- Test thoroughly in Task 1.2
- If bug exists, investigate and fix before continuing
- May require reedline library investigation

---

### Risk 2: Full Dataset Export Performance
**Likelihood:** Low
**Impact:** Medium (slow exports, user frustration)

**Mitigation:**
- Implement progress indicator for large exports
- Test with large tables (100K+ rows)
- Add optional confirmation for very large tables

---

### Risk 3: Clipboard Cross-Platform Support
**Likelihood:** Medium
**Impact:** Low (graceful degradation possible)

**Mitigation:**
- Use well-maintained clipboard library (arboard)
- Graceful error messages if clipboard unavailable
- Prioritize macOS (user's primary platform)
- Document platform support clearly

---

## Testing Strategy

### Unit Tests
- Clipboard export functionality (mocked clipboard)
- Full dataset export logic (query rewriting)
- Welcome message rendering
- Version display

### Integration Tests
- Export to clipboard (各 formats)
- Export full dataset vs limited dataset
- Branding display in various modes
- Regression tests for Sprint 11 fixes

### Manual Tests (Required)
- ✅ Tab completion verification (Task 1.2)
- ✅ Clipboard export on macOS
- ✅ Full dataset export with large tables
- ✅ Welcome message appearance
- ✅ Cross-terminal compatibility

### Test Pass Criteria
- 100% unit test pass rate
- 100% integration test pass rate
- All manual test cases pass
- Zero regressions from previous sprints

---

## Definition of Done

A task is considered DONE when:
1. ✅ Code implemented and committed
2. ✅ Unit tests written and passing
3. ✅ Integration tests written and passing
4. ✅ Manual testing completed (where applicable)
5. ✅ Documentation updated (help text, README, specs)
6. ✅ No technical debt introduced
7. ✅ User validation completed
8. ✅ Changes committed to git

Sprint is considered DONE when:
1. ✅ All P0 tasks completed
2. ✅ All P1 tasks completed (or explicitly deferred with justification)
3. ✅ 100% test pass rate
4. ✅ User validation: User confirms issues resolved
5. ✅ Binary rebuilt and version bumped
6. ✅ Sprint review document created
7. ✅ Specifications updated
8. ✅ Changes committed and pushed to GitHub

---

## Dependencies

### External Dependencies
- Clipboard library: `arboard` or `cli-clipboard`
- Teradata database: Live connection for manual testing

### Internal Dependencies
- Sprint 11 fixes must be in binary (Task 1.1 blocks everything)
- User availability for final validation

---

## Process Improvements for This Sprint

### Lesson from Sprint 11 Failure
Sprint 11 was marked "CODE COMPLETE" but:
- Version never bumped
- Binary never rebuilt
- User never asked to test
- Fixes never actually deployed

### New Process for Sprint 12
1. **Build Verification:** Every sprint MUST rebuild binary before claiming completion
2. **Version Discipline:** Version bump MUST be part of completion criteria
3. **User Validation:** For UI features, user MUST test before sprint closure
4. **Definition of Done:** "Code complete" ≠ "Sprint complete"

### Updated Sprint Closure Checklist
- [ ] All code committed
- [ ] Version bumped in Cargo.toml
- [ ] **Binary rebuilt with `cargo build --release`**
- [ ] **Binary version verified: `./target/release/tq --version`**
- [ ] All tests passing
- [ ] **User validation completed** (for UI features)
- [ ] Sprint review created
- [ ] Specifications updated
- [ ] Changes pushed to GitHub

---

## Communication Plan

### User Updates
1. **Sprint Start:** Present this plan for approval
2. **Task 1.1 Complete:** Notify user binary rebuilt, ready for testing
3. **Task 1.2 Complete:** Share manual test results
4. **Task 2.1 Complete:** Demo clipboard export
5. **Task 2.2 Complete:** Demo full dataset export
6. **Task 3.1 Complete:** Show welcome message
7. **Sprint End:** Request final user validation

### Expected User Involvement
- **Sprint Planning:** Approve scope (5 minutes)
- **Mid-Sprint:** Test rebuilt binary (15 minutes)
- **Sprint Closure:** Final validation (15 minutes)

**Total User Time:** ~35 minutes

---

## Success Metrics

### Quantitative
- 100% test pass rate
- 0 technical debt items
- Version bumped: 1.6.0 → 1.6.1
- Binary rebuilt: Yes
- Features delivered: 3 (clipboard export, full dataset export, branding)
- Bugs verified fixed: 2 (tab completion, table display)

### Qualitative
- User confirms tab completion works correctly
- User confirms exports meet requirements
- User satisfied with branding appearance
- User trust restored through proper deployment
- Tool ready for client presentation

---

## Timeline Estimate

### Day 1
- Phase 1: Critical Bug Verification (1-2 hours)
- Phase 2: Export Enhancements (4-5 hours)

### Day 2
- Phase 3: Branding (2 hours)
- Testing and validation (2 hours)
- Sprint closure (1 hour)

**Total Estimated Effort:** 10-12 hours over 1-2 days

---

## References

- **Sprint 11 Review:** `docs/builder/sprints/sprint-11-review.md`
- **User Feedback:** `docs/builder/incoming/open-bugs.md`
- **Current Specifications:** `docs/builder/specifications.md`
- **REPL Specifications:** `docs/builder/detailed-specifications/repl-mode.md`
- **Output Formats:** `docs/builder/detailed-specifications/output-formats.md`

---

## Action Items from Previous Sprint

From Sprint 11 Review:
1. ✅ Create interactive testing framework (expectrl) - Defer to Sprint 13
2. ✅ Update testing-guidelines.md - Defer to Sprint 13
3. ✅ **Fix deployment process** - Addressed in this sprint (P0)
4. ✅ User validation of Sprint 11 fixes - Included in Task 1.2

---

## Notes

### Why This Sprint is Critical

Sprint 11 claimed to fix tab completion and table display, but user is STILL experiencing bugs because:
1. Binary was never rebuilt with the fixes
2. User running old version (1.6.0) without Sprint 11 code
3. Process failure caused user frustration: "doesn't work AGAIN!"

**Sprint 12 Priority:** Restore trust by properly deploying fixes AND delivering high-value features.

### Why Export Enhancements are Urgent

User explicitly prioritized: "Two key enhancements that I need you to prioritize"
- Export to clipboard: Common workflow, high convenience
- Full dataset export: Current behavior is broken (exports only 100 rows)

### Why Branding Matters

User context: "so it can be presented to our clients and users"
- Tool is customer-facing
- Professional appearance matters
- Currently has zero branding ("very sad")
- Quick win to improve perception

---

## Sign-Off

**Sprint Coordinator:** Claude (Main Agent)
**Planning Date:** 2026-01-19
**Approval Required:** User (user)

**Ready to Proceed:** Pending user approval of this plan

---

## Appendix: Sprint 11 Root Cause Analysis

### Timeline of Events
```
2026-01-18 20:05:00  - Binary built (tq 1.6.0)
2026-01-18 20:10:36  - Sprint 11 fixes committed (166ae30)
2026-01-18 20:xx:xx  - Sprint 11 marked "CODE COMPLETE"
2026-01-19 08:xx:xx  - User reports: "STILL DOESN'T WORK"
2026-01-19 09:xx:xx  - Root cause identified: Binary never rebuilt
```

### What Went Wrong
1. Code changes committed ✅
2. Tests passing ✅
3. Review document created ✅
4. **Binary never rebuilt ❌**
5. **Version never bumped ❌**
6. **User never asked to test ❌**

### What We're Fixing
✅ Make binary rebuild mandatory in Definition of Done
✅ Make version bump mandatory before sprint closure
✅ Make user validation mandatory for UI features
✅ Update sprint coordinator checklist

### Lesson Learned
**"CODE COMPLETE" is not the same as "DEPLOYED AND VERIFIED"**

For future sprints:
- Build binary BEFORE sprint closure
- Verify binary version
- Get user validation for UI changes
- Don't mark sprint complete until binary is tested
