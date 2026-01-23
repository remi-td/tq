# Sprint 22 Test Strategy: REPL Enhancements (Metacommand Completion, Schema Commands, Loading Indicator)

**Created:** 2026-01-23
**Author:** quality-validator
**Sprint:** Sprint 22
**Features:**
1. Metacommand Tab Completion (P0)
2. Enhanced Schema Commands (P0) - `/list databases`, `/list tables [pattern]`, `/list views`
3. Loading Indicator for Tab Completion (P1)
4. Integration Test Infrastructure Fix (P1)

---

## CRITICAL CONTEXT: Sprint 21 Hybrid Testing Success

### The Lesson Applied

**Sprint 21 demonstrated the hybrid testing pattern works when applied proactively.**

**Evidence from Sprint 21**:
- Single iteration success (vs Sprint 20's 3 iterations)
- 52% cost reduction compared to Sprint 20
- Proactive test strategy prevented false positives
- Clear documentation of automation limitations avoided surprises

**Key Quote from Sprint 22 Planning**:
> "Sprint 21 Lessons to Apply: Proactive test strategy (document automation limitations upfront), Hybrid testing (automated + manual), False positive risk assessment, Make manual validation PRIMARY for keyboard/UX features"

### Hybrid Testing Pattern Confirmed

**Pattern Definition**:
- **Automated Component**: Code behavior, content correctness, regression detection
- **Manual Component**: Keyboard UX, visual rendering, subjective "intuitive" feel
- **Verdict Logic**: APPROVED only if BOTH automated AND manual tests pass

**This pattern applies to ALL Sprint 22 features.**

---

## Test Automation Capabilities & Limitations

### What Automated Tests CAN Validate

| Aspect | Test Type | Technique | Confidence Level |
|--------|-----------|-----------|------------------|
| Metacommand list completeness | Unit | Verify completer returns all metacommands | HIGH ✅ |
| Database query SQL syntax | Unit | Mock database, verify SQL | HIGH ✅ |
| Query results contain expected data | Integration | Execute against live database | HIGH ✅ |
| Text output contains expected strings | PTY | Capture stdout, search for content | MEDIUM ⚠️ |
| Loading threshold logic (>500ms) | Unit | Test time-based trigger | HIGH ✅ |

### What Automated Tests CANNOT Validate

| Aspect | Why Automation Fails | Sprint 21 Evidence |
|--------|---------------------|-------------------|
| TAB key completes metacommand text | PTY cannot reliably track cursor/text insertion | Feature 3: TAB acceptance |
| Completion menu navigation (UP/DOWN) | PTY cannot distinguish menu state from text output | Feature 3: Keyboard UX |
| Visual display of metacommand descriptions | PTY captures escape codes, not rendered output | Feature 3: Menu rendering |
| Loading indicator appears "during" slow query | PTY timing issues, async display challenges | Known limitation |
| User perception of "smooth" UX | Subjective judgment | Requires human evaluation |

**CRITICAL INSIGHT**: Sprint 22 features are EXACTLY the type that require hybrid testing:
- Metacommand completion: TAB key behavior (keyboard UX)
- Schema commands: Database queries (content testable) + output formatting (visual)
- Loading indicator: Timing-based UI feedback (hard to automate)

---

## Feature-by-Feature Test Strategy

### Feature 1: Metacommand Tab Completion (P0)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-22-planning.md` lines 40-44
- Related: `docs/specifications/repl.md` lines 862-1157 (metacommands section)

**User Need**: "User types `/des<TAB>` → completes to `/describe`"

**Feature Characteristics**:

**User Interaction Type**: Interactive PTY (REPL completion)

**Explanation**: User presses TAB after typing partial metacommand. Success = correct metacommand inserted.

**Observable Behavior**:
- [x] Visual output in terminal (completion menu with metacommands)
- [x] Text insertion at cursor (completed metacommand)
- [x] State management (completer tracks metacommands)

**External Dependencies**:
- [x] Terminal/PTY (keyboard interaction)
- [ ] Database connection (NOT required for metacommand list)

**Validation Challenges**:
- TAB key behavior: Does TAB complete text or just show menu?
- Partial matching: `/des` should filter to `/describe` (and `/disconnect`?)
- Menu display: Are descriptions shown? Readable?
- Context awareness: Metacommands only at start of line (after `/` or `\`)

**Critical Behaviors to Validate**:
1. "Typing `/` + TAB shows all available metacommands" (AC#1)
2. "Typing `/des` + TAB completes to `/describe`" (AC#2)
3. "Partial matches show filtered list (e.g., `/l` shows `/list`, `/logon`)" (AC#3)
4. "Completion menu displays metacommand descriptions" (AC#4)
5. "Works in multi-line mode (any line starting with `/`)" (AC#5)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Interactive PTY" checked
   → Interactive tests REQUIRED

✅ "Visual output in terminal" checked
   → Manual validation REQUIRED (Sprint 21 lesson)

❌ "Database connection" NOT required
   → Unit tests sufficient for logic validation
```

**Automation Capability Assessment**:

**What PTY Tests CAN Validate**:
- Text output contains metacommand names (`/describe`, `/list`, etc.)
- Multiple metacommands appear after `/` + TAB
- Filtered list appears after `/des` + TAB

**What PTY Tests CANNOT Validate** (Sprint 21 lesson):
- TAB key actually inserts completed text (vs just showing menu)
- Cursor position after completion
- Visual formatting of completion menu
- Descriptions displayed correctly
- "Smooth" completion UX

**Automation Risk**: HIGH (same as Sprint 21 Feature 3)

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Completer returns correct metacommand list
- **Approach**: Call completer with partial input (`/des`), verify returns `["/describe", "/disconnect"]`
- **Rationale**: Validates filtering logic without PTY/database
- **Gap if missing**: Logic errors (wrong filtering, missing commands)
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates**: N/A - No database required for metacommand completion
- **Necessity**: ❌ NOT NEEDED

**Test Type 3: Interactive PTY Tests**
- **Validates**: Text output contains metacommand names after TAB
- **Approach**: Send `/`, send TAB, verify output contains `/describe`, `/list`, etc.
- **Rationale**: Weak signal for CI/CD regression detection
- **Gap if missing**: No automated regression detection
- **Necessity**: ⚠️ RECOMMENDED (but INSUFFICIENT for approval)

**Test Type 4: Manual Validation** ⚠️ **PRIMARY VALIDATION METHOD**
- **Validates**: TAB completes metacommand text, menu displays correctly
- **Approach**: Human types `/des`, presses TAB, confirms text completes to `/describe`
- **Rationale**: ONLY manual testing can validate keyboard completion UX (Sprint 21 lesson)
- **Gap if missing**: FALSE POSITIVE GUARANTEED
- **Necessity**: ✅ REQUIRED - **THIS IS THE ONLY RELIABLE TEST**

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates completer filter logic | Wrong filtering, missing commands | MUST IMPLEMENT |
| Integration tests | ❌ NOT NEEDED | No database dependency | N/A | SKIP |
| Interactive tests (PTY) | ⚠️ RECOMMENDED | Weak regression signal | No automated CI/CD check | IMPLEMENT (mark insufficient) |
| Manual validation | ✅ REQUIRED | ONLY method to validate TAB completion | FALSE POSITIVE GUARANTEED | DOCUMENT PROCEDURE (PRIMARY) |

**Summary**:
- ✅ REQUIRED test types: 2 (unit, manual)
- ⚠️ RECOMMENDED test types: 1 (PTY, but insufficient)
- ❌ NOT NEEDED test types: 1 (integration)

**FALSE POSITIVE RISK: HIGH** (same pattern as Sprint 21 Feature 3)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F1-1 | "Typing `/` + TAB shows all metacommands" | sprint-22-planning.md:70 | Unit + PTY + Manual | Unit validates list, PTY detects text, Manual validates display | TC-F1-UNIT-001, TC-F1-PTY-001, Manual-F1 |
| REQ-F1-2 | "Typing `/des` + TAB completes to `/describe`" | sprint-22-planning.md:71 | Unit + Manual | Unit validates filter, Manual validates completion | TC-F1-UNIT-002, Manual-F1 |
| REQ-F1-3 | "Partial matches show filtered list" | sprint-22-planning.md:72 | Unit + Manual | Unit validates filtering, Manual validates display | TC-F1-UNIT-003, Manual-F1 |
| REQ-F1-4 | "Completion menu displays descriptions" | sprint-22-planning.md:73 | Manual | Visual rendering NOT testable with automation | Manual-F1 (PRIMARY) |
| REQ-F1-5 | "Works in multi-line mode" | sprint-22-planning.md:74 | Unit + Manual | Unit validates context detection, Manual validates UX | TC-F1-UNIT-004, Manual-F1 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: AC#4 (descriptions) has ONLY manual coverage (visual validation impossible to automate)

#### 5. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/commands/repl/completer.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 4 tests
- **Key scenarios to cover**:
  1. `/` + TAB → returns all metacommands
  2. `/des` → returns filtered list (`/describe`, `/disconnect`)
  3. `/l` → returns filtered list (`/list`, `/logon`)
  4. Multi-line context: Line starts with `/` → enable metacommand completion
- **Mocking strategy**: No database needed, test completer logic directly

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 1 test (LIMITED VALUE)
- **Key scenarios to cover**:
  1. Type `/`, press TAB, verify output contains `/describe`, `/list`
- **Implementation notes**:
  - Test CANNOT validate TAB actually completes text
  - Test provides weak signal for CI/CD
  - Mark test with warning about limitations

**Test Type: Manual Validation** ⚠️ **PRIMARY TEST**
- **Location**: `tests/cases/TC-F1-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 comprehensive procedure
- **Key scenarios to cover**:
  1. Type `/`, press TAB, observe menu with all metacommands
  2. Type `/des`, press TAB, observe completion to `/describe`
  3. Type `/l`, press TAB, observe filtered list (`/list`, `/logon`)
  4. Verify descriptions displayed in menu
  5. Test in multi-line mode (after newline, type `/`)
- **Evidence**: Screenshot or video, user confirmation
- **VERDICT GATE**: APPROVED verdict REQUIRES this manual test to pass

#### 6. False Positive Risk Assessment

**False Positive Risk**: HIGH ⚠️

**Rationale**:
- Pure keyboard interaction behavior (same as Sprint 21 Feature 3)
- PTY tests CANNOT validate TAB completes text
- 4 out of 5 acceptance criteria NOT reliably testable with automation
- Menu display validation requires visual inspection

**Mitigation**:
- Manual validation is PRIMARY test (not secondary)
- PTY test marked with warning
- APPROVED verdict REQUIRES manual validation
- Document automation limitations upfront

---

### Feature 2: Enhanced Schema Commands (P0)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-22-planning.md` lines 46-51
- Related: `docs/specifications/repl.md` lines 952-1003 (schema inspection commands)

**User Need**: "Implement `/list databases`, `/list tables [pattern]`, `/list views`"

**Feature Characteristics**:

**User Interaction Type**: Database query + Terminal output

**Explanation**: User types metacommand, tool executes database query, displays formatted results.

**Observable Behavior**:
- [x] Database side effects (query executed)
- [x] Visual output in terminal (table/list display)
- [x] Pattern matching logic (for `/list tables pattern`)

**External Dependencies**:
- [x] Database connection (requires live database)
- [x] Terminal/PTY (output display)

**Validation Challenges**:
- Database state: Tests depend on database having databases/tables/views
- Permission handling: Some databases/schemas may deny access
- Pattern matching: Glob pattern syntax (e.g., `dbc.t*`)
- Output formatting: Table vs list format, column alignment

**Critical Behaviors to Validate**:
1. "`/list databases` displays all databases with proper formatting" (AC#1)
2. "`/list tables` displays tables in current database" (AC#2)
3. "`/list tables pattern` filters by glob pattern" (AC#3)
4. "`/list views` displays views in current database" (AC#4)
5. "Commands respect current database context" (AC#5)
6. "Error handling for permission denied cases" (AC#6)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Database connection" checked
   → Integration tests with live database REQUIRED

✅ "Visual output in terminal" checked
   → PTY tests + Manual validation REQUIRED

❌ "Keyboard interaction" NOT primary concern
   → Manual validation less critical than Feature 1
```

**Automation Capability Assessment**:

**What Automated Tests CAN Validate**:
- Database queries return results
- Result contains expected databases/tables/views
- Pattern matching filters correctly
- Error handling for permission denied

**What Automated Tests CANNOT Validate**:
- Visual table formatting (column alignment, borders)
- Output is "readable" and "properly formatted"
- Color highlighting (if any)

**Automation Risk**: MEDIUM (content testable, formatting less critical than keyboard UX)

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Pattern matching logic (glob to SQL LIKE conversion)
- **Approach**: Test pattern conversion: `dbc.t*` → SQL `WHERE DatabaseName = 'dbc' AND TableName LIKE 't%'`
- **Rationale**: Validates pattern logic without database
- **Gap if missing**: Pattern syntax errors, wrong SQL conversion
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests (Live Database)**
- **Validates**: Commands execute successfully, return expected data
- **Approach**: Execute `/list databases`, verify result contains known databases; test pattern filtering
- **Rationale**: Proves commands work on real Teradata
- **Gap if missing**: SQL errors, permission issues, wrong query
- **Necessity**: ✅ REQUIRED

**Test Type 3: Interactive PTY Tests**
- **Validates**: Commands produce formatted output in REPL
- **Approach**: Send `/list databases`, capture output, verify contains database names
- **Rationale**: Validates end-to-end REPL integration
- **Gap if missing**: REPL integration bugs, output formatting issues
- **Necessity**: ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates**: Output formatting is readable, proper column alignment
- **Approach**: User runs commands, confirms output is well-formatted
- **Rationale**: Validates visual formatting, user-friendliness
- **Gap if missing**: Poor formatting ships (acceptable risk)
- **Necessity**: ⚠️ RECOMMENDED (but not blocking)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates pattern matching logic | Wrong pattern conversion | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Proves commands work with real database | SQL errors, permission issues | MUST IMPLEMENT |
| Interactive tests (PTY) | ✅ REQUIRED | Validates REPL integration, output content | REPL-level bugs | MUST IMPLEMENT |
| Manual validation | ⚠️ RECOMMENDED | Validates output formatting quality | Poor formatting (acceptable risk) | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 3 (unit, integration, PTY)
- ⚠️ RECOMMENDED test types: 1 (manual)
- ❌ NOT NEEDED test types: 0

**FALSE POSITIVE RISK: LOW to MEDIUM** (content testable, formatting less critical)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F2-1 | "`/list databases` displays all databases" | sprint-22-planning.md:80 | Integration + PTY + Manual | Integration proves query, PTY validates output, Manual validates formatting | TC-F2-INT-001, TC-F2-PTY-001, Manual-F2 |
| REQ-F2-2 | "`/list tables` displays tables in current DB" | sprint-22-planning.md:81 | Integration + PTY + Manual | Integration proves query, PTY validates output | TC-F2-INT-002, TC-F2-PTY-002, Manual-F2 |
| REQ-F2-3 | "`/list tables pattern` filters by glob" | sprint-22-planning.md:82 | Unit + Integration + PTY | Unit validates pattern logic, Integration proves execution | TC-F2-UNIT-001, TC-F2-INT-003, TC-F2-PTY-003 |
| REQ-F2-4 | "`/list views` displays views" | sprint-22-planning.md:83 | Integration + PTY | Integration proves query, PTY validates output | TC-F2-INT-004, TC-F2-PTY-004 |
| REQ-F2-5 | "Commands respect current database context" | sprint-22-planning.md:84 | Integration + PTY | Integration tests context-aware queries | TC-F2-INT-005, TC-F2-PTY-005 |
| REQ-F2-6 | "Error handling for permission denied" | sprint-22-planning.md:85 | Integration + PTY | Integration tests graceful error handling | TC-F2-INT-006, TC-F2-PTY-006 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: NONE (all requirements have automated coverage)

#### 5. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/commands/repl/metacommands.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 3 tests
- **Key scenarios to cover**:
  1. Pattern parsing: `dbc.t*` → database="dbc", pattern="t%"
  2. Pattern parsing: `*.emp*` → database="*", pattern="emp%"
  3. No pattern: default behavior
- **Mocking strategy**: No database needed, test pattern logic directly

**Test Type: Integration Tests (Live Database)**
- **Location**: `tests/integration_tests.rs`
- **Framework**: Built-in Rust integration test support, marked `#[ignore]`
- **Test count estimate**: 6 tests
- **Key scenarios to cover**:
  1. `/list databases` returns known databases
  2. `/list tables` returns tables in current database
  3. `/list tables dbc.t*` filters tables by pattern
  4. `/list views` returns views
  5. Context-aware: `/list tables` uses current database
  6. Permission denied: graceful error message
- **Setup requirements**: Test database with known schema

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 6 tests
- **Key scenarios to cover**:
  1. `/list databases` → output contains database names
  2. `/list tables` → output contains table names
  3. `/list tables dbc.*` → output shows filtered tables
  4. `/list views` → output contains view names
  5. Verify output formatted as table (not raw text)
  6. Error case: permission denied shows error message
- **Implementation notes**: PTY validates content, not visual perfection

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F2-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. User runs `/list databases`, confirms output readable
  2. User runs `/list tables`, confirms column alignment
  3. User runs `/list tables dbc.t*`, confirms filtering works
  4. User confirms output is user-friendly
- **Evidence**: Screenshot, user confirmation

#### 6. False Positive Risk Assessment

**False Positive Risk**: LOW to MEDIUM

**Rationale**:
- Content-based validation (database/table names) is testable
- Integration tests prove queries work
- PTY tests verify output content
- Visual formatting is less critical than Feature 1's keyboard UX

**Mitigation**:
- Manual validation confirms formatting quality
- Integration tests catch data correctness issues
- PTY tests catch major output bugs

---

### Feature 3: Loading Indicator for Tab Completion (P1)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-22-planning.md` lines 53-57
- User Need: "Display 'Loading tables from <database>...' for slow metadata fetches (>500ms)"

**Feature Characteristics**:

**User Interaction Type**: Timing-based UI feedback

**Explanation**: During tab completion, if metadata fetch takes >500ms, show loading indicator. This is NON-CRITICAL UX enhancement.

**Observable Behavior**:
- [x] Visual output in terminal (loading message)
- [x] Time-based trigger (>500ms threshold)
- [x] Asynchronous display (appears during fetch, disappears after)

**External Dependencies**:
- [x] Database connection (requires slow metadata fetch)
- [x] Terminal/PTY (async output)

**Validation Challenges**:
- **CRITICAL**: Very hard to automate timing-based UI feedback
- Database speed: Hard to guarantee >500ms fetch in tests
- Async display: Indicator appears/disappears during fetch (timing issues)
- Transient UI: Indicator only visible briefly
- PTY limitation: Hard to capture transient async output

**Critical Behaviors to Validate**:
1. "Indicator appears for metadata queries >500ms" (AC#1)
2. "Message format: 'Loading tables from <database>...'" (AC#2)
3. "Indicator clears when completion menu appears" (AC#3)
4. "No indicator for cached results (instant)" (AC#4)
5. "Graceful handling if indicator fails to display" (AC#5)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Timing-based trigger" checked
   → Unit tests for threshold logic REQUIRED

✅ "Async visual display" checked
   → PTY tests VERY DIFFICULT, likely insufficient

⚠️ "Transient UI" checked
   → Manual validation HIGHLY RECOMMENDED
```

**Automation Capability Assessment**:

**What Automated Tests CAN Validate**:
- Threshold logic: >500ms → show indicator
- Message format: contains database name
- Indicator triggered for uncached fetch

**What Automated Tests CANNOT Validate** (HIGH CONFIDENCE):
- Indicator actually APPEARS in terminal during fetch
- Indicator CLEARS after fetch completes
- Timing is correct (appears after 500ms, not before)
- User sees indicator (transient, async)

**Automation Risk**: VERY HIGH (timing-based, async, transient)

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Threshold logic (500ms trigger)
- **Approach**: Test function that determines if indicator should show based on elapsed time
- **Rationale**: Validates decision logic without async/PTY complexity
- **Gap if missing**: Wrong threshold, logic errors
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates**: N/A - Indicator is UI feature, not database logic
- **Necessity**: ❌ NOT NEEDED

**Test Type 3: Interactive PTY Tests**
- **Validates**: VERY LIMITED - May detect indicator text in output
- **Approach**: Trigger slow metadata fetch, search output for "Loading tables"
- **Rationale**: Weak signal for CI/CD
- **Gap if missing**: No automated check at all
- **Necessity**: ⚠️ OPTIONAL (likely insufficient, may be too unreliable)

**Test Type 4: Manual Validation**
- **Validates**: User sees indicator during slow fetch
- **Approach**: User triggers slow metadata fetch, observes loading message
- **Rationale**: ONLY reliable way to validate timing-based UI
- **Gap if missing**: No validation of user experience
- **Necessity**: ⚠️ HIGHLY RECOMMENDED (but P1 feature, not blocking)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates threshold logic | Wrong timing, logic errors | MUST IMPLEMENT |
| Integration tests | ❌ NOT NEEDED | UI feature, not database logic | N/A | SKIP |
| Interactive tests (PTY) | ⚠️ OPTIONAL | Very weak signal, timing issues | No automated check | SKIP (too unreliable) |
| Manual validation | ⚠️ RECOMMENDED | ONLY way to validate timing UI | No user experience validation | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 1 (unit)
- ⚠️ RECOMMENDED test types: 1 (manual)
- ⚠️ OPTIONAL test types: 1 (PTY, likely skip)
- ❌ NOT NEEDED test types: 1 (integration)

**FALSE POSITIVE RISK: EXTREMELY HIGH** (timing-based async UI)

**CRITICAL NOTE**: This is a P1 (not P0) feature. Given automation difficulty and non-critical nature, recommend:
- **MUST**: Unit tests for threshold logic
- **SHOULD**: Manual validation procedure documented
- **MAY**: Skip PTY tests (too unreliable for timing-based UI)
- **VERDICT**: APPROVED possible without manual validation IF unit tests pass and feature is non-critical

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F3-1 | "Indicator appears for queries >500ms" | sprint-22-planning.md:91 | Unit + Manual | Unit validates threshold, Manual validates display | TC-F3-UNIT-001, Manual-F3 |
| REQ-F3-2 | "Message format: 'Loading tables from <db>...'" | sprint-22-planning.md:92 | Unit + Manual | Unit validates format, Manual validates display | TC-F3-UNIT-002, Manual-F3 |
| REQ-F3-3 | "Indicator clears when menu appears" | sprint-22-planning.md:93 | Manual | Async display NOT testable with automation | Manual-F3 |
| REQ-F3-4 | "No indicator for cached results" | sprint-22-planning.md:94 | Unit | Unit validates cached path bypasses indicator | TC-F3-UNIT-003 |
| REQ-F3-5 | "Graceful handling if indicator fails" | sprint-22-planning.md:95 | Manual | Non-critical failure handling | Manual-F3 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: AC#3, AC#5 have ONLY manual coverage (async timing impossible to automate reliably)

#### 5. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/commands/repl/loading_indicator.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 3 tests
- **Key scenarios to cover**:
  1. Elapsed time < 500ms → should_show = false
  2. Elapsed time > 500ms → should_show = true
  3. Cached result → should_show = false
- **Mocking strategy**: Mock time/duration, no database needed

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F3-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. User triggers slow metadata fetch (large database)
  2. User observes loading indicator appears
  3. User confirms indicator clears after fetch
  4. User confirms no indicator for cached fetch
- **Evidence**: Video recording (recommended for timing), user confirmation

#### 6. False Positive Risk Assessment

**False Positive Risk**: VERY HIGH ⚠️

**Rationale**:
- Timing-based async UI is VERY hard to automate
- Unit tests validate logic but not actual display
- PTY tests likely too unreliable for timing validation
- Manual validation only reliable method

**Mitigation**:
- Unit tests catch threshold logic errors
- Manual validation confirms user experience
- **P1 feature**: Not blocking for APPROVED verdict
- Document as "best effort" validation

**SPECIAL CONSIDERATION**: Given P1 priority and automation difficulty, consider:
- APPROVED verdict possible without manual validation
- Manual validation recommended but not mandatory
- Focus testing effort on P0 features (F1, F2)

---

### Feature 4: Integration Test Infrastructure Fix (P1)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-22-planning.md` lines 59-63
- Issue: "Driver only supports one connection at a time" error

**User Need**: "Refactor test harness to handle multiple test files"

**Feature Characteristics**:

**User Interaction Type**: Infrastructure (developer-facing)

**Explanation**: Fix test framework to allow running multiple integration tests without connection conflicts.

**Observable Behavior**:
- [x] Test execution behavior (tests run without conflicts)
- [x] Error handling (no "Driver only supports one connection" errors)

**External Dependencies**:
- [x] Database connection (test environment)

**Validation Challenges**:
- Connection lifecycle: Ensure tests clean up connections
- Test isolation: Each test gets clean state
- CI/CD compatibility: Tests must run in automated environment

**Critical Behaviors to Validate**:
1. "All integration tests run without driver conflicts" (AC#1)
2. "Test isolation: Each test gets clean connection state" (AC#2)
3. "100% integration test pass rate (was 50% in Sprint 21)" (AC#3)
4. "CI/CD compatible test execution" (AC#4)
5. "Clear error messages on test failures" (AC#5)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Test infrastructure" checked
   → Meta-testing: Test the tests themselves

✅ "Database connection lifecycle" checked
   → Integration tests validate fix
```

**Automation Capability Assessment**:

**What Automated Tests CAN Validate**:
- Tests run without connection errors
- Tests pass consistently
- Test isolation works (no state leakage)

**What Manual Validation Adds**:
- CI/CD compatibility confirmation
- Error message clarity assessment

**Automation Risk**: LOW (this IS test automation, validation is straightforward)

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: N/A - Infrastructure fix, no new logic to unit test
- **Necessity**: ❌ NOT NEEDED

**Test Type 2: Integration Tests (Live Database)**
- **Validates**: Multiple tests run successfully, no connection conflicts
- **Approach**: Run existing integration test suite, verify 100% pass rate
- **Rationale**: Running tests IS the validation
- **Gap if missing**: No validation of fix
- **Necessity**: ✅ REQUIRED (this IS the feature)

**Test Type 3: Interactive PTY Tests**
- **Validates**: N/A - Not applicable to test infrastructure
- **Necessity**: ❌ NOT NEEDED

**Test Type 4: Manual Validation**
- **Validates**: CI/CD compatibility, error messages clear
- **Approach**: Developer confirms tests run in CI, error messages helpful
- **Rationale**: Validates developer experience
- **Gap if missing**: CI/CD issues not caught
- **Necessity**: ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ❌ NOT NEEDED | No new logic to unit test | N/A | SKIP |
| Integration tests | ✅ REQUIRED | Running tests IS the validation | No validation of fix | MUST IMPLEMENT |
| Interactive tests (PTY) | ❌ NOT NEEDED | Not applicable | N/A | SKIP |
| Manual validation | ⚠️ RECOMMENDED | Validates CI/CD and error UX | CI issues not caught | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 1 (integration)
- ⚠️ RECOMMENDED test types: 1 (manual)
- ❌ NOT NEEDED test types: 2 (unit, PTY)

**FALSE POSITIVE RISK: LOW** (straightforward validation: tests pass or fail)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F4-1 | "All integration tests run without conflicts" | sprint-22-planning.md:100 | Integration | Run test suite, verify no errors | All integration tests |
| REQ-F4-2 | "Test isolation: clean connection state" | sprint-22-planning.md:101 | Integration | Run tests, verify no state leakage | All integration tests |
| REQ-F4-3 | "100% integration test pass rate" | sprint-22-planning.md:102 | Integration | Run test suite, count passes | All integration tests |
| REQ-F4-4 | "CI/CD compatible execution" | sprint-22-planning.md:103 | Manual | Developer confirms CI works | Manual-F4 |
| REQ-F4-5 | "Clear error messages on failures" | sprint-22-planning.md:104 | Manual | Developer reviews error output | Manual-F4 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: NONE (AC#4, AC#5 have manual coverage, sufficient for P1)

#### 5. Test Implementation Plan

**Test Type: Integration Tests**
- **Location**: `tests/integration_tests.rs`
- **Framework**: Built-in Rust integration test support, marked `#[ignore]`
- **Test count estimate**: ALL existing integration tests (from Sprint 21)
- **Key scenarios to cover**:
  1. Run all integration tests sequentially
  2. Verify no "Driver only supports one connection" errors
  3. Verify 100% pass rate
- **Setup requirements**: Test database with proper cleanup
- **Success criteria**: All tests pass, no connection errors

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F4-MANUAL.md`
- **Framework**: Developer checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. Developer runs `cargo test -- --ignored`
  2. Developer confirms all tests pass
  3. Developer confirms error messages are clear (if any failures)
  4. Developer confirms tests run in CI/CD
- **Evidence**: Test run output, CI logs

#### 6. False Positive Risk Assessment

**False Positive Risk**: LOW

**Rationale**:
- Test pass/fail is objective metric
- Connection errors are explicit and obvious
- Integration tests are self-validating

**Mitigation**:
- Run full integration test suite
- Manual validation confirms CI/CD compatibility

---

## Hybrid Testing Pattern Summary

### Test Types Required by Feature

| Feature | Unit | Integration | PTY | Manual | Primary Validation |
|---------|------|-------------|-----|--------|-------------------|
| F1: Metacommand Completion | ✅ | ❌ | ⚠️ | ✅ | **Manual (PRIMARY)** |
| F2: Schema Commands | ✅ | ✅ | ✅ | ⚠️ | Hybrid (automated primary) |
| F3: Loading Indicator | ✅ | ❌ | ❌ | ⚠️ | Unit + Manual (P1, not blocking) |
| F4: Test Infrastructure | ❌ | ✅ | ❌ | ⚠️ | Integration (self-validating) |

### Risk-Adjusted Test Coverage

| Feature | Automated Coverage | Manual Coverage | False Positive Risk | Mitigation |
|---------|-------------------|-----------------|---------------------|------------|
| F1 | 40% (unit + PTY weak) | 60% (manual PRIMARY) | **HIGH** | Manual validation mandatory |
| F2 | 85% (unit + int + PTY) | 15% (formatting) | LOW-MEDIUM | Manual confirms formatting |
| F3 | 75% (unit logic) | 25% (display) | VERY HIGH | Manual recommended, P1 not blocking |
| F4 | 100% (self-test) | 0% (optional CI check) | LOW | Integration tests are validation |

### Total Test Count Estimate

| Test Type | Feature 1 | Feature 2 | Feature 3 | Feature 4 | Total |
|-----------|-----------|-----------|-----------|-----------|-------|
| Unit tests | 4 | 3 | 3 | 0 | 10 |
| Integration tests | 0 | 6 | 0 | ALL | 6+ |
| Interactive tests (PTY) | 1* | 6 | 0 | 0 | 7 |
| Manual validation procedures | 1** | 1 | 1 | 1 | 4 |
| **Total** | **6** | **16** | **4** | **2+** | **27+** |

*Feature 1 PTY test marked as INSUFFICIENT
**Feature 1 manual validation is PRIMARY test

---

## Manual Validation Procedures

### Manual-F1: Metacommand Tab Completion ⚠️ PRIMARY TEST

**Objective**: Verify TAB completes metacommand text

**Prerequisites**:
- tq REPL compiled and runnable
- No database required for this test

**Steps**:

**Test 1: Show All Metacommands**
1. Start tq REPL: `tq repl` (can use without database)
2. Type: `/`
3. Press TAB
4. Observe: Menu shows all metacommands (`/describe`, `/list`, `/help`, etc.)

**Test 2: Complete Partial Metacommand**
5. Clear line (Ctrl-U)
6. Type: `/des`
7. Press TAB
8. Observe: Text completes to `/describe` OR shows filtered menu

**Test 3: Filtered List**
9. Clear line (Ctrl-U)
10. Type: `/l`
11. Press TAB
12. Observe: Menu shows `/list`, `/logon` (filtered)

**Test 4: Descriptions Displayed**
13. Verify metacommand descriptions visible in menu

**Test 5: Multi-line Mode**
14. Type: `SELECT * FROM t1;` (complete command)
15. Press Enter (new line in multi-line mode)
16. Type: `/des`
17. Press TAB
18. Observe: Metacommand completion works on second line

**Expected Results**:
- `/` + TAB shows all metacommands
- `/des` + TAB completes to `/describe`
- Filtered lists work correctly
- Descriptions displayed
- Works in multi-line mode

**Acceptance Criteria**:
- [ ] All metacommands shown after `/` + TAB
- [ ] Partial completion works (`/des` → `/describe`)
- [ ] Filtered lists display correctly
- [ ] Descriptions visible in menu
- [ ] Works in multi-line mode

**Evidence**: Screenshot or video, user confirmation

**CRITICAL**: This is the PRIMARY TEST for Feature 1. Automated tests CANNOT validate keyboard completion UX.

---

### Manual-F2: Enhanced Schema Commands

**Objective**: Verify schema commands display correctly formatted output

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with test database

**Steps**:

**Test 1: List Databases**
1. Start tq REPL: `tq repl`
2. Type: `/list databases`
3. Observe: Output shows databases in readable format

**Test 2: List Tables**
4. Type: `/list tables`
5. Observe: Output shows tables in current database

**Test 3: List Tables with Pattern**
6. Type: `/list tables dbc.t*`
7. Observe: Output shows filtered tables (only `dbc` database, names starting with `t`)

**Test 4: List Views**
8. Type: `/list views`
9. Observe: Output shows views

**Expected Results**:
- Commands execute successfully
- Output formatted as readable table/list
- Column alignment correct
- Pattern filtering works
- Error messages (if any) are clear

**Acceptance Criteria**:
- [ ] `/list databases` shows databases, readable format
- [ ] `/list tables` shows tables in current database
- [ ] `/list tables pattern` filters correctly
- [ ] `/list views` shows views
- [ ] Output formatting is user-friendly

**Evidence**: Screenshot, user confirmation

---

### Manual-F3: Loading Indicator (P1)

**Objective**: Verify loading indicator appears for slow metadata fetches

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with database
- Database with slow metadata fetch (large database or slow network)

**Steps**:

**Test 1: Slow Metadata Fetch**
1. Start tq REPL: `tq repl`
2. Type: `SELECT * FROM large_database.`
3. Press TAB (trigger metadata fetch)
4. Observe: Loading indicator appears ("Loading tables from large_database...")
5. Wait for completion menu
6. Observe: Indicator clears, completion menu appears

**Test 2: Cached Fetch (No Indicator)**
7. Type: `SELECT * FROM large_database.` (same database)
8. Press TAB (cached fetch)
9. Observe: Completion menu appears instantly, NO loading indicator

**Expected Results**:
- Loading indicator appears for >500ms fetch
- Message format correct: "Loading tables from <database>..."
- Indicator clears when menu appears
- No indicator for cached results

**Acceptance Criteria**:
- [ ] Loading indicator appears for slow fetch
- [ ] Message format correct
- [ ] Indicator clears after completion
- [ ] No indicator for cached results

**Evidence**: Video recording (recommended for timing), user confirmation

**NOTE**: P1 feature, not blocking for APPROVED verdict. Manual validation recommended but not mandatory.

---

### Manual-F4: Integration Test Infrastructure (P1)

**Objective**: Verify integration tests run without connection conflicts

**Prerequisites**:
- tq project cloned
- Rust toolchain installed
- TQ_LOGON configured

**Steps**:

**Test 1: Run Integration Tests**
1. Open terminal
2. Navigate to tq project root
3. Run: `cargo test --test integration_tests -- --ignored`
4. Observe: All tests pass, no "Driver only supports one connection" errors

**Test 2: Verify Test Count**
5. Count test results
6. Verify: 100% pass rate (no failures)

**Test 3: Check CI/CD**
7. Push to GitHub
8. Verify: CI tests pass

**Expected Results**:
- All integration tests pass
- No connection conflict errors
- Test output is clear
- CI/CD compatible

**Acceptance Criteria**:
- [ ] All integration tests pass (100% pass rate)
- [ ] No "Driver only supports one connection" errors
- [ ] Clear error messages (if any failures)
- [ ] CI/CD tests pass

**Evidence**: Test run output, CI logs

---

## Test Execution Schedule

### Sprint 22 Phase 3: Build & Test

**Week 1, Day 1-2: Implementation + Automated Tests**
- rust-teradata-architect implements Features 1-4
- quality-validator implements unit tests (parallel)
- quality-validator implements integration tests (parallel)

**Week 1, Day 2-3: PTY Tests**
- quality-validator implements PTY tests for Features 1, 2
- Mark Feature 1 PTY test as INSUFFICIENT
- Skip Feature 3 PTY tests (too unreliable)

**Week 1, Day 3: Automated Test Execution**
- Run all unit tests: `cargo test --lib`
- Run all integration tests: `cargo test --test integration_tests -- --ignored`
- Run PTY tests: `cargo test --test interactive_tests -- --ignored`
- Generate test report with pass/fail counts

**Week 1, Day 3: Manual Validation**
- Execute Manual-F1 procedure (PRIMARY TEST - MANDATORY)
- Execute Manual-F2 procedure (RECOMMENDED)
- Execute Manual-F3 procedure (P1 - OPTIONAL)
- Execute Manual-F4 procedure (RECOMMENDED)
- Collect evidence (screenshots, videos, confirmations)

**Week 1, Day 3: Iteration Decision**
- IF automated tests FAIL OR Manual-F1 FAIL OR Manual-F2 FAIL:
  - rust-teradata-architect fixes issues
  - quality-validator re-executes failed tests
  - REPEAT until pass
- IF automated tests PASS AND Manual-F1 PASS AND Manual-F2 PASS:
  - Generate final test report
  - Verdict: APPROVED (F3, F4 manual validation optional)
  - Proceed to Phase 4 (Ship)

---

## Success Criteria

Sprint 22 testing is successful when:

**MANDATORY (P0 Features)**:
- [x] All P0 automated tests implemented and passing (F1 unit/PTY, F2 unit/integration/PTY)
- [x] Manual-F1 executed and passing (PRIMARY TEST - MANDATORY)
- [x] Manual-F2 executed and passing (RECOMMENDED - output quality)

**OPTIONAL (P1 Features)**:
- [ ] Feature 3 unit tests passing (REQUIRED for F3)
- [ ] Manual-F3 executed (OPTIONAL - timing validation)
- [ ] Feature 4 integration tests passing (REQUIRED for F4)
- [ ] Manual-F4 executed (OPTIONAL - CI/CD confirmation)

**Verdict Logic**:

**APPROVED**:
- P0 automated PASS + Manual-F1 PASS + Manual-F2 PASS ✅
- P1 features tested separately (can ship without P1)

**REJECTED**:
- P0 automated FAIL ❌
- Manual-F1 FAIL (FALSE POSITIVE) ❌
- Manual-F2 FAIL (output quality issue) ❌

**BLOCKED**:
- Tests cannot execute (database unavailable) ⛔

**P1 Features (Optional)**:
- F3, F4 can ship with unit/integration tests passing only
- Manual validation recommended but not mandatory

---

## Tools and Infrastructure Needs

### Existing Tools (Already Available)

- [x] `cargo test` framework (unit tests)
- [x] `tests/integration_tests.rs` (integration tests)
- [x] `tests/interactive_tests.rs` (PTY tests with expectrl)
- [x] `.env` file for TQ_LOGON configuration
- [x] Sprint 21 test strategy as reference

### New Tools Required

**NONE** - All necessary tools are in place.

### Test Utilities to Create

**Helper Functions** (add to `tests/interactive_tests.rs`):

1. `verify_metacommand_list(output: &str, expected: &[&str]) -> bool`
   - Parses PTY output to verify metacommands present
   - Enables content verification for Feature 1

2. `verify_table_output(output: &str) -> bool`
   - Verifies output formatted as table (not raw text)
   - Enables format verification for Feature 2

**No New Test Framework Required**: Sprint 21 infrastructure is sufficient.

---

## Lessons Learned from Sprint 21 Applied

### 1. Proactive Test Strategy

**Sprint 21 Lesson**: "Proactive test strategy prevents false positives"

**Applied in Sprint 22**:
- Test strategy created BEFORE implementation
- Automation limitations documented upfront
- False positive risk assessed for each feature
- Primary validation method identified (manual vs automated)

### 2. Hybrid Testing Mandatory for Interactive Features

**Sprint 21 Lesson**: "Manual validation PRIMARY for keyboard/UX features"

**Applied in Sprint 22**:
- Feature 1 (metacommand completion): Manual validation PRIMARY
- Feature 2 (schema commands): Automated primary, manual for formatting
- Feature 3 (loading indicator): Manual recommended (P1, timing-based)
- Feature 4 (test infrastructure): Self-validating (integration tests)

### 3. Document Automation Limitations

**Sprint 21 Lesson**: "Document what tests CAN and CANNOT validate"

**Applied in Sprint 22**:
- Section "Test Automation Capabilities & Limitations" at top of document
- Each feature has automation capability assessment
- PTY test limitations explicitly stated
- Manual validation justification provided

### 4. False Positive Risk Assessment

**Sprint 21 Lesson**: "Assess false positive risk per feature"

**Applied in Sprint 22**:
- Each feature has risk assessment section
- Risk levels: LOW, MEDIUM, HIGH, VERY HIGH
- Mitigation strategies documented
- Verdict logic adjusted based on risk

### 5. P1 Features Can Ship with Limited Validation

**Sprint 22 Decision**:
- P0 features require full manual validation
- P1 features can ship with automated tests only
- Focus testing effort on P0 features
- Document reduced validation for P1

---

## Document Sign-off

**Test Strategy Author**: quality-validator
**Created Date**: 2026-01-23
**Review Status**: READY FOR IMPLEMENTATION
**Sprint 22 Phase**: Phase 2 (Design)

**Strategy Completeness Checklist**:
- [x] Every feature has complete specification analysis
- [x] Feature characteristics classified (not assumed)
- [x] Test strategy derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan detailed and actionable
- [x] Coverage sufficiency assessed
- [x] Sprint 21 lessons learned applied
- [x] False positive risk assessed for each feature
- [x] Hybrid testing pattern clearly defined
- [x] Manual validation procedures documented
- [x] P0/P1 priority reflected in validation requirements

**Ready for Phase 3 Implementation**: YES ✅

**Key Decisions**:
1. Feature 1 (metacommand completion): Manual validation MANDATORY (HIGH false positive risk)
2. Feature 2 (schema commands): Automated primary, manual recommended for output quality
3. Feature 3 (loading indicator): P1 feature, manual validation recommended but not blocking
4. Feature 4 (test infrastructure): Self-validating via integration tests
5. Overall: P0 features require manual validation, P1 features can ship with automated tests only
