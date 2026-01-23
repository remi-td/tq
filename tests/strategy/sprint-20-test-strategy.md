# Sprint 20 Test Strategy

**Created:** 2026-01-23
**Author:** quality-validator
**Sprint:** Sprint 20
**Features:** Logo Bug Fix, Tab Completion Bug Fix

---

## Executive Summary

Sprint 20 is a **Bug Fix Sprint** that addresses two critical bugs that persisted through Sprint 18 and Sprint 19:
1. **Logo Display Bug**: Lowercase ASCII art 'tq' logo not displaying exactly as user specified
2. **Tab Completion Bug**: Pager output appearing during tab completion after "SELECT * FROM "

**Critical Context from Sprint History:**
- **Sprint 18**: Delivered 286/286 tests PASSING but bugs persisted (false positives)
- **Sprint 19**: Attempted fixes, manual validation left pending
- **Root Cause**: Automated tests validated code behavior, NOT user-facing experience

**Test Strategy Philosophy:**
This strategy is designed to **prevent false positives** by combining automated safety nets with mandatory manual validation. We test what users SEE, not just what code DOES.

---

## Feature-by-Feature Test Strategy

### Feature 1: Logo Display - Lowercase ASCII Art

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-20-planning.md` (lines 41-70)
- Secondary: `incoming/open-bugs.md` (lines 6-22)
- Sprint History: `docs/sprints/sprint-19-review.md` (Sprint 18 false positive analysis)

**User's Exact Requirements (from open-bugs.md):**
```
 __
/\ \__
\ \ ,_\    __
 \ \ \/  /'__`\
  \ \ \_/\ \L\ \
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
```
"This is a lowercase 't' (left) in Teradata orange and lowercase 'q' (right) in default color, using block characters for clarity."

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL startup display)

**Explanation:** The logo is displayed when the REPL starts. It's visual terminal output with specific ASCII art characters, colors, and layout that users see in their terminal.

**Observable Behavior:**
- ✅ Visual output in terminal (ASCII art, colors, layout)
- ❌ Structured data output (not applicable)
- ❌ File system side effects (not applicable)
- ❌ Database side effects (not applicable)
- ❌ Network interactions (not applicable)
- ❌ Performance characteristics (not performance-critical)
- ❌ State management (stateless display)

**External Dependencies:**
- ❌ Database connection (displayed before connection used)
- ❌ File system access (no file I/O)
- ❌ Network access (no network calls)
- ✅ Terminal/PTY (ANSI color codes, terminal rendering)
- ❌ System clipboard (not applicable)
- ❌ Operating system specific features (ANSI colors are cross-platform)
- ❌ None

**Validation Challenges:**
1. **ASCII art character precision**: Automated tests cannot verify the exact visual shape of ASCII art vs. plain text
2. **Terminal color rendering**: xterm-256 color 202 (orange) may appear differently across terminals
3. **Layout verification**: Distinguishing "info on right" vs. "info below" requires visual inspection
4. **User's subjective assessment**: User must confirm it matches their mental image of "lowercase tq"

**Critical Behaviors to Validate (from specification):**
1. "Logo uses the exact ASCII art provided by user" (sprint-20-planning.md §Feature 1, line 60)
2. "'t' character (left) is colored in Teradata orange (RGB ≈ 255,95,0, color code 202)" (sprint-20-planning.md §Feature 1, line 61)
3. "'q' character (right) is in default terminal color" (sprint-20-planning.md §Feature 1, line 62)
4. "Logo displays correctly on REPL startup" (sprint-20-planning.md §Feature 1, line 63)
5. "Manual testing: User confirms logo looks correct in their terminal" (sprint-20-planning.md §Feature 1, line 65)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Logo appears in REPL terminal output with visual rendering

✅ "Visual output in terminal" checked:
  → Interactive tests OR manual verification REQUIRED
  Reason: ASCII art layout and colors are visual properties

❌ "Database connection" NOT checked:
  → No database dependency for logo display

❌ "Performance characteristics" NOT checked:
  → Logo display is not performance-critical
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Internal logo data structures are correctly defined (character arrays, spacing)
- **Approach:** Test that logo character arrays contain expected patterns (underscores, slashes, backslashes, parentheses)
- **Rationale:** Catches typos or malformed ASCII art in source code constants
- **Gap if missing:** Could ship with misspelled ASCII art characters (e.g., "/" instead of "\\")
- **Necessity:** ⚠️ RECOMMENDED (nice to have but not critical)

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** Logo appears in REPL output, colors are present, text matches expected pattern
- **Approach:** Spawn REPL, capture startup output, verify ANSI color codes (color 202), verify ASCII art characters present
- **Rationale:** Automated safety net that catches major regressions (missing colors, wrong characters, crashed startup)
- **Gap if missing:** No automated regression detection - manual testing every sprint forever
- **Necessity:** ✅ REQUIRED (automated safety net)

**Test Type 3: Manual Visual Inspection**
- **Validates:** User confirms logo LOOKS correct (exact match to specification, subjective assessment)
- **Approach:** Human tester starts REPL, visually compares to user's specification, confirms match
- **Rationale:** Only human can judge if ASCII art "looks like lowercase tq" and if colors are acceptable
- **Gap if missing:** Same failure as Sprint 18 - tests pass but user rejects output
- **Necessity:** ✅ REQUIRED (final arbiter of correctness)

**Test Type 4: Screenshot Evidence**
- **Validates:** Permanent record of what was displayed, proof for future reference
- **Approach:** Human captures screenshot of REPL startup, commits to test results
- **Rationale:** Prevents disputes ("it worked on my machine"), documents exact rendering
- **Gap if missing:** No proof of what was tested, can't compare across sprints
- **Necessity:** ✅ REQUIRED (evidence-based testing)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (logo data) | ⚠️ RECOMMENDED | Validates ASCII art arrays in source | Typos in character arrays not caught until runtime | IMPLEMENT (low effort, high value) |
| Interactive tests (expectrl) | ✅ REQUIRED | Automated regression detection | No automated safety net, manual testing forever | MUST IMPLEMENT |
| Manual visual inspection | ✅ REQUIRED | Human validates subjective correctness | Repeat Sprint 18 failure (passing tests, rejected by user) | MUST IMPLEMENT |
| Screenshot evidence | ✅ REQUIRED | Permanent proof of display | No evidence of what was actually tested | MUST IMPLEMENT |

**Summary:**
- ✅ REQUIRED test types: 3 (interactive automated, manual inspection, screenshot)
- ⚠️ RECOMMENDED test types: 1 (unit tests for logo data)
- ❌ NOT NEEDED test types: 0
- **All required tests MUST pass for APPROVED verdict**

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| LOGO-REQ-1 | "Logo uses the exact ASCII art provided by user" | sprint-20-planning.md line 60 | Unit + Interactive + Manual | Unit validates data, Interactive catches regressions, Manual confirms match | TC-LOGO-003 |
| LOGO-REQ-2 | "'t' character (left) is colored in Teradata orange (RGB ≈ 255,95,0, color code 202)" | sprint-20-planning.md line 61 | Interactive + Manual | Interactive validates ANSI code 202, Manual confirms color looks orange | TC-LOGO-003 |
| LOGO-REQ-3 | "'q' character (right) is in default terminal color" | sprint-20-planning.md line 62 | Interactive + Manual | Interactive validates no color override, Manual confirms default appearance | TC-LOGO-003 |
| LOGO-REQ-4 | "Logo displays correctly on REPL startup" | sprint-20-planning.md line 63 | Interactive + Manual | Interactive validates output appears, Manual confirms correctness | TC-LOGO-003 |
| LOGO-REQ-5 | "Visually verified by reading actual REPL startup output" | sprint-20-planning.md line 64 | Manual + Screenshot | Manual inspection with photographic evidence | TC-LOGO-003 |
| LOGO-REQ-6 | "Manual testing: User confirms logo looks correct in their terminal" | sprint-20-planning.md line 65 | Manual + Screenshot | Only user can confirm it matches their specification | TC-LOGO-003 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types

**Coverage Gaps:**
- **Cross-terminal compatibility**: Not testing on multiple terminal emulators (iTerm2, Terminal.app, Alacritty, etc.)
  - **Risk:** LOW - ANSI color 202 is standard xterm-256, widely supported
  - **Mitigation:** User tests on their actual terminal, which is the target environment

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Logo display is a one-time startup operation with no performance requirements
- **What won't be validated:** Display latency, memory usage, rendering speed
- **Risk assessment:** LOW
- **Mitigation:** Startup time is fast enough that performance is not a concern
- **Revisit criteria:** If users report slow startup or high memory usage

**Integration Tests (Full CLI Invocation)**
- **Reason for omission:** Interactive tests already invoke full REPL process via expectrl
- **What won't be validated:** Nothing - interactive tests cover full integration
- **Risk assessment:** LOW
- **Mitigation:** Interactive tests are full integration tests (spawn real process)
- **Revisit criteria:** N/A - coverage is sufficient

**Automated Screenshot Comparison**
- **Reason for omission:** No image comparison infrastructure, complex to implement, high maintenance burden
- **What won't be validated:** Pixel-perfect visual regression testing
- **Risk assessment:** MEDIUM
- **Mitigation:** Manual screenshot review + human visual inspection
- **Revisit criteria:** If we have frequent visual regressions across sprints

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/mod.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 1-2 tests
- **Key scenarios to cover:**
  1. Verify logo_t array contains expected characters (underscores, slashes, etc.)
  2. Verify logo_q array contains expected characters (parentheses, etc.)
- **Mocking strategy:** No mocking needed (pure data validation)
- **Implementation notes:** Simple string pattern matching on logo constants

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Spawn REPL, capture startup output, verify:
     - Output contains ASCII art characters from user's spec (underscores, slashes, backslashes)
     - ANSI escape sequence for color 202 is present
     - Output contains info lines (version, connection, user)
     - No crashes or errors during startup
- **Implementation notes:**
  - Use expectrl to spawn `tq repl` process
  - Read until first prompt appears
  - Parse output for color codes and ASCII art patterns
  - Assert presence of required elements (not exact visual match)
  - Mark as `#[ignore]` to require explicit database connection

**Test Type: Manual Visual Inspection**
- **Location:** `tests/cases/TC-LOGO-003.md`
- **Framework:** Manual test case document
- **Test count estimate:** 1 manual test
- **Key scenarios to cover:**
  1. Human starts REPL in real terminal
  2. Human visually compares logo to user's specification
  3. Human confirms:
     - Logo looks like lowercase 'tq'
     - 't' appears orange
     - 'q' appears in default color
     - Layout matches user's intent
     - Overall appearance is acceptable
- **Implementation notes:**
  - Detailed step-by-step instructions
  - Clear pass/fail criteria
  - Checklist format for easy execution
  - Screenshot requirement

**Test Type: Screenshot Evidence**
- **Location:** `tests/results/sprint-20/logo-screenshot.png`
- **Framework:** Screenshot capture (OS-dependent)
- **Test count estimate:** 1 screenshot
- **Key scenarios to cover:**
  1. Capture REPL startup showing logo
  2. Include entire banner area (logo + info lines + first prompt)
  3. High resolution, colors visible
- **Implementation notes:**
  - macOS: Cmd+Shift+4
  - Linux: gnome-screenshot or scrot
  - Windows: Snipping Tool
  - Commit screenshot to git for permanent record

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Logo data structures are correctly defined in source code
- Interactive tests validate: Logo appears in REPL output with correct color codes and ASCII patterns
- Manual inspection validates: Logo LOOKS correct to human eyes (subjective match to specification)
- Screenshot validates: Permanent evidence of exact display for future reference

**Combined coverage:** COMPREHENSIVE

**Gaps in combined coverage:**
- Cross-terminal compatibility (only tested on user's terminal)
- Color blindness accessibility (orange may not be distinguishable for some users)
- Non-xterm-256 terminals (may not support color 202)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- **Cross-terminal gap is acceptable because:** User tests on their actual terminal, which is the target environment. Supporting all terminals is out of scope.
- **Color blindness gap is acceptable because:** Teradata brand color is a design requirement, not a usability requirement. Alternative accessibility is not in Sprint 20 scope.
- **Non-xterm-256 gap is acceptable because:** Modern terminals universally support xterm-256. Legacy terminal support is not required.

---

### Feature 2: Tab Completion - Suppress Pager Output

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-20-planning.md` (lines 72-106)
- Secondary: `incoming/open-bugs.md` (lines 24-40)
- Sprint History: `docs/sprints/sprint-19-review.md` (Sprint 19 implementation analysis)

**User's Exact Requirements (from open-bugs.md):**
> "If I press tab after `select * from ` I get:
> ```
> tq> ? select * from
> Page 1: records 0 - 0  total: 0
> ```
> You story about teradatarustapi is writing directly to TTY doesn't make any sense to me since the query functionality works well otherwise and uses the same drivers..."

**User's Recommended Solution:**
- Cache all database names at startup (`sel databasename from dbc.databases;`)
- Cache all database object names incrementally as databases are used
- Implement proper menu-based completion with filtering and navigation
- Research how this is best implemented in other Rust tools
- Design a robust solution with test mechanism

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL tab completion during typing)

**Explanation:** Tab completion is an interactive feature triggered by pressing TAB key in REPL. The observable behavior is visual output (completion menu) that should NOT include pager text.

**Observable Behavior:**
- ✅ Visual output in terminal (completion menu, must NOT include pager output)
- ❌ Structured data output (not applicable)
- ❌ File system side effects (cache is in-memory only)
- ✅ Database side effects (metadata queries load database/table names)
- ❌ Network interactions (database queries, but through existing connection)
- ❌ Performance characteristics (caching is for performance, but not measured)
- ✅ State management (cache persists during REPL session, cleared on reconnect)

**External Dependencies:**
- ✅ Database connection (CRITICAL - requires live database to load metadata)
- ❌ File system access (no file I/O)
- ❌ Network access (uses existing database connection)
- ✅ Terminal/PTY (TAB key input, completion menu display)
- ❌ System clipboard (not applicable)
- ❌ Operating system specific features (OutputSuppressor uses Unix fd manipulation)
- ❌ None

**Validation Challenges:**
1. **Keyboard input simulation**: Automated tests must simulate TAB key press in PTY
2. **Pager output detection**: Must verify pager text does NOT appear (negative assertion)
3. **Database availability**: Tests are BLOCKED if database not accessible
4. **Completion menu rendering**: Visual completion menu may differ across readline implementations
5. **Timing issues**: Metadata loading is asynchronous, may cause race conditions in tests

**Critical Behaviors to Validate (from specification):**
1. "Tab completion after 'select * from ' does NOT show pager output" (sprint-20-planning.md §Feature 2, line 93)
2. "Database names are cached at REPL startup or first completion request" (sprint-20-planning.md §Feature 2, line 94)
3. "Table names are cached incrementally as needed" (sprint-20-planning.md §Feature 2, line 95)
4. "Completion menu shows databases, filters as user types" (sprint-20-planning.md §Feature 2, line 96)
5. "After selecting database with '.', completion shows tables in that database" (sprint-20-planning.md §Feature 2, line 97)
6. "Manual testing: User confirms tab completion works without pager output" (sprint-20-planning.md §Feature 2, line 101)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Tab completion is triggered by keyboard input in REPL

✅ "Visual output in terminal" checked:
  → Interactive tests OR manual verification REQUIRED
  Reason: Must verify pager output does NOT appear (negative assertion)

✅ "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Metadata queries need real database to test

✅ "State management" checked:
  → Tests must validate cache behavior across session
  Reason: Caching is core feature requirement
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** OutputSuppressor correctly redirects stdout/stderr to /dev/null
- **Approach:** Test OutputSuppressor struct in isolation, verify file descriptor manipulation logic
- **Rationale:** Core mechanism that prevents pager output must be unit-tested
- **Gap if missing:** OutputSuppressor could fail silently without unit tests
- **Necessity:** ✅ REQUIRED (critical infrastructure component)

**Test Type 2: Interactive Tests (expectrl) - Automated Component**
- **Validates:** Tab completion does NOT produce pager output text in captured output
- **Approach:** Spawn REPL with database, send "select * from ", send TAB key, capture output, assert "Page" NOT in output
- **Rationale:** Automated safety net that catches regression if pager output reappears
- **Gap if missing:** No automated regression detection - manual testing every sprint
- **Necessity:** ✅ REQUIRED (automated safety net)

**Test Type 3: Interactive Tests (expectrl) - Completion Content**
- **Validates:** Tab completion shows database names, not keywords
- **Approach:** Spawn REPL, send "select * from ", send TAB key, verify database names appear in output
- **Rationale:** Completion must work correctly, not just avoid pager output
- **Gap if missing:** Could suppress pager but break completion entirely
- **Necessity:** ✅ REQUIRED (functional correctness)

**Test Type 4: Manual Visual Inspection**
- **Validates:** User confirms NO pager output appears in actual terminal during tab completion
- **Approach:** Human presses TAB in real REPL, visually confirms no "Page X: records..." text
- **Rationale:** Only human can confirm visual absence of pager output in real terminal
- **Gap if missing:** Same failure as Sprint 18/19 - tests pass but user sees pager output
- **Necessity:** ✅ REQUIRED (final arbiter of bug fix)

**Test Type 5: Screenshot Evidence**
- **Validates:** Permanent record of tab completion showing NO pager output
- **Approach:** Human captures screenshot of REPL after pressing TAB, commits to test results
- **Rationale:** Proof that pager output is absent, prevents disputes
- **Gap if missing:** No evidence of what was tested
- **Necessity:** ✅ REQUIRED (evidence-based testing)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (OutputSuppressor) | ✅ REQUIRED | Core mechanism must be tested in isolation | Silent failures in fd manipulation | MUST IMPLEMENT |
| Interactive automated (negative pager) | ✅ REQUIRED | Automated regression detection | No automated safety net | MUST IMPLEMENT |
| Interactive automated (completion content) | ✅ REQUIRED | Verify completion still works | Could break completion while fixing pager | MUST IMPLEMENT |
| Manual visual inspection | ✅ REQUIRED | Human validates absence of pager output | Repeat Sprint 18/19 failure | MUST IMPLEMENT |
| Screenshot evidence | ✅ REQUIRED | Permanent proof of display | No evidence of testing | MUST IMPLEMENT |

**Summary:**
- ✅ REQUIRED test types: 5 (unit, 2× interactive automated, manual inspection, screenshot)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 0
- **All required tests MUST pass for APPROVED verdict**

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| TAB-REQ-1 | "Tab completion after 'select * from ' does NOT show pager output" | sprint-20-planning.md line 93 | Interactive automated + Manual | Automated catches regression, Manual confirms visual absence | TC-TAB-COMPLETION-003 |
| TAB-REQ-2 | "Database names are cached at REPL startup or first completion request" | sprint-20-planning.md line 94 | Unit + Interactive | Unit tests cache logic, Interactive validates caching works | TC-TAB-COMPLETION-003 |
| TAB-REQ-3 | "Table names are cached incrementally as needed" | sprint-20-planning.md line 95 | Unit + Interactive | Unit tests cache logic, Interactive validates incremental loading | TC-TAB-COMPLETION-003 |
| TAB-REQ-4 | "Completion menu shows databases, filters as user types" | sprint-20-planning.md line 96 | Interactive automated + Manual | Automated validates data, Manual confirms display | TC-TAB-COMPLETION-003 |
| TAB-REQ-5 | "After selecting database with '.', completion shows tables in that database" | sprint-20-planning.md line 97 | Interactive automated + Manual | Automated validates qualified name completion, Manual confirms UX | TC-TAB-COMPLETION-003 |
| TAB-REQ-6 | "Manual testing: User confirms tab completion works without pager output" | sprint-20-planning.md line 101 | Manual + Screenshot | Only user can confirm bug is fixed | TC-TAB-COMPLETION-003 |
| TAB-IMPL-1 | "OutputSuppressor redirects stdout/stderr to /dev/null" | src/db/metadata.rs lines 17-30 | Unit | Core mechanism must work correctly | Unit tests in metadata.rs |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types

**Coverage Gaps:**
- **Windows platform**: OutputSuppressor is Unix-only (uses libc file descriptors)
  - **Risk:** MEDIUM - Windows users may still see pager output
  - **Mitigation:** Sprint 20 focuses on Unix (macOS/Linux), Windows is future work
  - **Documented in:** src/db/metadata.rs (cfg(not(unix)) stub implementation)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Caching is for user experience (responsiveness), not measured performance SLA
- **What won't be validated:** Metadata query execution time, cache lookup speed, memory usage
- **Risk assessment:** LOW
- **Mitigation:** If caching is noticeably slow, users will report it
- **Revisit criteria:** If users report slow tab completion or high memory usage

**Cross-Platform Tests (Windows)**
- **Reason for omission:** OutputSuppressor is Unix-only, Windows stub implementation exists
- **What won't be validated:** Tab completion behavior on Windows
- **Risk assessment:** MEDIUM
- **Mitigation:** Sprint 20 is bug fix for user's environment (Unix), Windows is future enhancement
- **Revisit criteria:** If Windows users report pager output during tab completion

**Load Testing (Many Databases/Tables)**
- **Reason for omission:** No scalability requirements in specification
- **What won't be validated:** Performance with 1000+ databases or 10000+ tables
- **Risk assessment:** LOW
- **Mitigation:** Test database has reasonable number of objects (<100)
- **Revisit criteria:** If users report slowness with large catalogs

#### 6. Test Implementation Plan

**Test Type: Unit Tests (OutputSuppressor)**
- **Location:** `src/db/metadata.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 2-3 tests
- **Key scenarios to cover:**
  1. OutputSuppressor::new() successfully opens /dev/null and duplicates fds
  2. OutputSuppressor::drop() restores original stdout/stderr
  3. OutputSuppressor gracefully degrades if fd operations fail
- **Mocking strategy:** No mocking (tests real Unix file descriptors)
- **Implementation notes:**
  - Unix-only tests (cfg(unix))
  - Test by writing to stdout during suppression, verify not visible
  - Test restoration by writing after drop, verify visible

**Test Type: Interactive Automated (Negative Pager Assertion)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Spawn REPL with database connection
  2. Send "select * from "
  3. Send TAB key
  4. Capture output
  5. Assert "Page" NOT in output
  6. Assert "records" NOT in output
  7. Assert "total" NOT in output
- **Implementation notes:**
  - Mark as `#[ignore]` (requires database)
  - Use expectrl::spawn to create PTY
  - Send TAB via write_all(b"\t")
  - Read until prompt reappears
  - Negative assertions (assert!(!output.contains("Page")))

**Test Type: Interactive Automated (Completion Content)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Spawn REPL with database connection
  2. Send "select * from "
  3. Send TAB key
  4. Verify database names appear in output
  5. Verify SQL keywords do NOT appear
- **Implementation notes:**
  - Mark as `#[ignore]` (requires database)
  - Parse output for database names from test environment
  - Assert positive (database names present)
  - Assert negative (keywords like "SELECT" absent)

**Test Type: Manual Visual Inspection**
- **Location:** `tests/cases/TC-TAB-COMPLETION-003.md`
- **Framework:** Manual test case document
- **Test count estimate:** 1 manual test
- **Key scenarios to cover:**
  1. Human starts REPL in real terminal
  2. Human types "select * from " (no Enter)
  3. Human presses TAB key
  4. Human visually observes output:
     - Completion menu appears (databases/tables)
     - NO pager output ("Page X: records...")
  5. Human confirms completion works correctly
- **Implementation notes:**
  - Detailed step-by-step instructions
  - Clear pass/fail criteria
  - Checklist for observations
  - Screenshot requirement
  - Compare with user's bug report (verify bug is fixed)

**Test Type: Screenshot Evidence**
- **Location:** `tests/results/sprint-20/tab-completion-screenshot.png`
- **Framework:** Screenshot capture (OS-dependent)
- **Test count estimate:** 1 screenshot
- **Key scenarios to cover:**
  1. Capture REPL after pressing TAB
  2. Show "select * from " line
  3. Show completion output below
  4. Prove no "Page X: records..." appears
- **Implementation notes:**
  - macOS: Cmd+Shift+4
  - Linux: gnome-screenshot or scrot
  - Screenshot must be clear enough to read text
  - Commit to git for permanent record

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: OutputSuppressor correctly redirects stdout/stderr to /dev/null
- Interactive automated validates: Pager output does NOT appear in captured PTY output
- Interactive automated validates: Completion menu shows correct content (databases, not keywords)
- Manual inspection validates: User confirms NO pager output in real terminal
- Screenshot validates: Permanent evidence of absence of pager output

**Combined coverage:** COMPREHENSIVE

**Gaps in combined coverage:**
- Windows platform (OutputSuppressor not implemented)
- Large-scale testing (1000+ databases/tables)
- Network instability (slow metadata queries)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- **Windows gap is acceptable because:** Sprint 20 is fixing bug for user's Unix environment. Windows support is future work.
- **Large-scale gap is acceptable because:** No scalability requirements in specification. Test database has typical object counts.
- **Network instability gap is acceptable because:** Metadata loading uses existing connection timeout mechanisms. No special handling required.

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ [Feature 1 (logo data), Feature 2 (OutputSuppressor)]
- Interactive tests (automated): ✅ [Feature 1 (logo rendering), Feature 2 (pager absence + completion content)]
- Manual tests: ✅ [Feature 1 (visual logo inspection), Feature 2 (visual pager absence)]
- Screenshot evidence: ✅ [Feature 1 (logo), Feature 2 (tab completion)]

**Estimated Test Count:**
- Unit: 3-5 tests
- Interactive automated: 3 tests (1 logo, 2 tab completion)
- Manual: 2 tests (1 logo, 1 tab completion)
- Screenshots: 2 files
- **Total: 8-10 tests + 2 screenshots**

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:** Windows platform (OutputSuppressor not implemented)
- **LOW risk gaps:** Cross-terminal compatibility, color blindness, large-scale testing

**Dependencies Required:**
- Live database: **YES** - Tab completion tests require database connection
- Network access: **NO** (uses existing database connection)
- Specific OS: **YES** - OutputSuppressor requires Unix (macOS/Linux)
- Other: Terminal with xterm-256 color support (standard)

---

## Critical Success Factors (Preventing Sprint 18/19 Failures)

### Why Sprint 18 Failed

**Root Cause:** Automated tests validated CODE BEHAVIOR, not USER EXPERIENCE.

**Sprint 18 Problems:**
1. **Logo Test**: Checked for text "tq" and color 202, but didn't verify ASCII art vs. plain text
2. **Tab Completion Test**: PTY automation captured data structures, but missed pager output in real terminal
3. **False Confidence**: 286/286 tests PASSED, but user rejected both features

**Quote from Sprint 19 Review:**
> "Sprint 18 delivered 100% test pass rate but bugs persisted because automated tests validated code behavior, not user-visible experience."

### Why Sprint 19 Left Validation Pending

**Root Cause:** Manual-only tests BLOCKED AI agent execution.

**Sprint 19 Problems:**
1. **Manual-only tests**: 2/3 tests required physical keyboard interaction
2. **No automated safety net**: No regression detection if code changes
3. **Execution blocker**: AI agent cannot press TAB key or capture screenshots
4. **Verdict BLOCKED**: Sprint couldn't be approved without user validation

**Quote from Sprint 19 Review:**
> "Deliberately Designed as Manual Tests: TC-TAB-COMPLETION-001 explicitly states: 'CRITICAL: This MUST be done in an ACTUAL terminal, NOT automated test'"

### Sprint 20 Hybrid Testing Strategy

**Solution:** Combine automated safety nets with mandatory manual validation.

**Automated Component (AI Agent Can Execute):**
1. **Negative assertions**: Assert "Page" NOT in output (automated regression detection)
2. **Positive assertions**: Assert database names present (functional correctness)
3. **Unit tests**: Validate OutputSuppressor mechanism (infrastructure testing)
4. **Result**: PASS or FAIL (automated safety net for CI/CD)

**Manual Component (Human Must Execute):**
1. **Visual inspection**: Human confirms no pager output in real terminal
2. **Subjective assessment**: Human confirms logo looks correct
3. **Screenshot evidence**: Human captures proof of display
4. **Result**: CONFIRMS or REJECTS automated result

**Verdict Logic:**
- **APPROVED**: If and only if BOTH automated AND manual tests PASS
- **REJECTED**: If automated tests FAIL (implementation broken)
- **REJECTED**: If automated tests PASS but manual tests FAIL (UX broken, repeat of Sprint 18)
- **BLOCKED**: If automated tests cannot execute (database unavailable)

**This Strategy Prevents:**
- ❌ Sprint 18 failure (false positives): Manual validation catches UX issues
- ❌ Sprint 19 failure (execution blocked): Automated tests provide immediate feedback
- ✅ Best of both worlds: Automated safety net + human validation

---

## Test Execution Dependencies

### Database Requirements

**CRITICAL:** Tab completion tests are BLOCKED if database is not available.

**Required Configuration:**
- `.env` file with `TQ_LOGON` environment variable
- Accessible Teradata database (test or development instance)
- Database with at least 2-3 databases visible to test user
- Database with at least 5-10 tables in one database

**If Database Not Available:**
- Tab completion automated tests: **BLOCKED** (cannot execute)
- Tab completion manual tests: **BLOCKED** (cannot execute)
- Logo tests: **NOT BLOCKED** (logo displays before connection used)
- **Sprint Verdict**: BLOCKED if tab completion cannot be tested

**Mitigation:**
- Logo tests can be executed independently
- Tab completion tests require database setup before execution
- Quality-validator must report BLOCKED verdict with clear requirements

### Terminal Requirements

**Manual Tests Require:**
- Real terminal emulator (iTerm2, Terminal.app, gnome-terminal, etc.)
- xterm-256 color support (standard on modern terminals)
- Keyboard input capability (to press TAB key)
- Screenshot capture tool (OS-dependent)

**Automated Tests Require:**
- expectrl crate (PTY simulation)
- Unix operating system (macOS or Linux)
- Rust test framework

---

## Test Execution Sequence

**Phase 1: Unit Tests (No Database Required)**
1. Run unit tests for logo data structures: `cargo test logo --lib`
2. Run unit tests for OutputSuppressor: `cargo test output_suppressor --lib`
3. **Expected**: All unit tests PASS
4. **If FAIL**: Fix code before proceeding to integration

**Phase 2: Interactive Automated Tests (Database Required)**
1. Verify database connection: Check `.env` file, test connection
2. Run logo interactive test: `cargo test interactive_logo -- --ignored --test-threads=1`
3. Run tab completion negative test: `cargo test interactive_tab_no_pager -- --ignored --test-threads=1`
4. Run tab completion content test: `cargo test interactive_tab_content -- --ignored --test-threads=1`
5. **Expected**: All interactive tests PASS
6. **If BLOCKED**: Report database unavailable, provide setup instructions
7. **If FAIL**: Fix code, repeat Phase 1 and Phase 2

**Phase 3: Manual Visual Inspection (User Required)**
1. Execute TC-LOGO-003: User starts REPL, visually inspects logo, captures screenshot
2. Execute TC-TAB-COMPLETION-003: User presses TAB, confirms no pager output, captures screenshot
3. **Expected**: Both manual tests PASS with screenshot evidence
4. **If FAIL**: Report which test failed and why (user feedback verbatim)

**Phase 4: Final Verdict**
- **APPROVED**: If ALL of the following are true:
  - All unit tests PASS (Phase 1)
  - All interactive automated tests PASS (Phase 2)
  - All manual tests PASS (Phase 3)
  - All screenshots captured and committed
- **REJECTED**: If any test FAILS in any phase
- **BLOCKED**: If database unavailable (Phase 2 cannot execute)

---

## Tool Requirements

### Existing Tools (Already Available)

**No new tools required.** All testing infrastructure exists:

1. **Unit testing**: Built-in Rust test framework (`#[test]`)
2. **Interactive testing**: expectrl crate (already in Cargo.toml)
3. **PTY simulation**: expectrl::spawn (already used in tests/interactive_tests.rs)
4. **Manual testing**: Test case documents (template exists in tests/README.md)
5. **Screenshot capture**: OS-provided tools (Cmd+Shift+4 on macOS)

### Tools NOT Required

**No custom tools need to be developed:**

- ❌ Custom PTY wrapper (expectrl is sufficient)
- ❌ Image comparison tool (manual visual inspection is more reliable)
- ❌ Database mocking (tests use real database)
- ❌ Output parsing library (simple string matching is sufficient)

### Test Infrastructure Assessment

**Current State:** READY

- `tests/interactive_tests.rs` already contains interactive test infrastructure
- `tests/cases/` already contains test case templates
- `tests/results/` already has result storage structure
- `.env` configuration already supports database credentials

**Gaps:** NONE

All required testing infrastructure is in place. No new tools need to be developed.

---

## Test Case Definitions

### TC-LOGO-003: Logo Display Verification (Sprint 20 Retry)

**Purpose:** Verify logo displays exactly as user specified in open-bugs.md

**Type:** Hybrid (Interactive Automated + Manual)

**Prerequisites:**
- tq binary built
- Terminal with xterm-256 color support

**Automated Component:**
1. Spawn REPL with expectrl
2. Capture startup output
3. Assert output contains ASCII art characters (_, /, \, `, etc.)
4. Assert ANSI escape sequence for color 202 present
5. Assert info lines present (version, connection, user)

**Manual Component:**
1. Human starts `./target/release/tq repl`
2. Human visually compares logo to specification in open-bugs.md
3. Human confirms:
   - Logo looks like lowercase 'tq' (not uppercase)
   - 't' appears orange (Teradata brand color)
   - 'q' appears in default color (white/black)
   - ASCII art matches user's specification line-by-line
4. Human captures screenshot: `tests/results/sprint-20/logo-screenshot.png`

**Pass Criteria:**
- ✅ Automated test PASSES (color codes and characters present)
- ✅ Manual inspection PASSES (user confirms logo looks correct)
- ✅ Screenshot captured and committed

**Fail Criteria:**
- ❌ Automated test FAILS (missing characters or colors)
- ❌ Manual inspection FAILS (logo doesn't match specification)
- ❌ Screenshot missing or unclear

### TC-TAB-COMPLETION-003: Tab Completion Without Pager Output (Sprint 20 Retry)

**Purpose:** Verify tab completion shows databases without pager output

**Type:** Hybrid (Interactive Automated + Manual)

**Prerequisites:**
- tq binary built
- Database connection configured in `.env`
- Database with accessible databases and tables

**Automated Component:**
1. Spawn REPL with expectrl and database connection
2. Send "select * from "
3. Send TAB key (b"\t")
4. Capture output until prompt returns
5. Assert "Page" NOT in output (negative assertion)
6. Assert "records" NOT in output (negative assertion)
7. Assert database names present in output (positive assertion)
8. Assert SQL keywords (SELECT, FROM) NOT in output (negative assertion)

**Manual Component:**
1. Human starts `./target/release/tq repl`
2. Human types "select * from " (without Enter)
3. Human presses TAB key
4. Human observes output:
   - Completion menu appears (databases/tables)
   - NO pager text ("Page 1: records 0 - 0 total: 0")
   - NO pager indicators ([FULL], Page X:, etc.)
5. Human confirms completion works (can select database)
6. Human captures screenshot: `tests/results/sprint-20/tab-completion-screenshot.png`

**Pass Criteria:**
- ✅ Automated test PASSES (no pager text in output)
- ✅ Automated test PASSES (database names present)
- ✅ Manual inspection PASSES (user confirms no pager output visible)
- ✅ Screenshot captured showing clean completion menu

**Fail Criteria:**
- ❌ Automated test FAILS (pager text detected)
- ❌ Automated test FAILS (database names missing)
- ❌ Manual inspection FAILS (user sees pager output)
- ❌ Screenshot shows pager output

**Blocked Criteria:**
- ⛔ Database not available (automated test cannot run)
- ⛔ Database connection fails (both automated and manual blocked)

---

## Strategy Validation Checklist

**Before submitting to sprint coordinator for review:**

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
- ✅ Hybrid testing strategy prevents Sprint 18/19 failures
- ✅ Tool requirements assessed (none needed)
- ✅ Test execution sequence defined
- ✅ Database dependencies clearly documented

**All checkboxes checked:** ✅ Strategy is complete and ready for review.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-23
**Review Status:** DRAFT
**Submitted for Review:** [Pending]

**Critical Insight:**
This test strategy learns from Sprint 18 and Sprint 19 failures by combining automated safety nets (regression detection) with mandatory manual validation (user experience verification). Tests validate what users SEE, not just what code DOES.

**Key Innovation:**
Hybrid testing prevents both false positives (Sprint 18) and execution blockers (Sprint 19) by requiring BOTH automated and manual tests to pass for APPROVED verdict.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test strategy for Sprint 20 | quality-validator |
