# Sprint 29 Test Case Index: Horizontal Paging

**Sprint:** 29
**Feature:** Interactive Horizontal Paging
**Created:** 2026-01-30
**Total Test Cases:** 70 (10 unit + 25 interactive + 10 regression + 12 edge + 13 integration)

---

## Test Case Summary by Type

### Unit Tests (10 tests) - `src/commands/repl/pager.rs`

| Test ID | Description | AC Coverage | Status |
|---------|-------------|-------------|--------|
| TC-HORIZ-001 | Right arrow column offset increment | AC-1 | Created |
| TC-HORIZ-002 | Left arrow column offset decrement | AC-2 | Created |
| TC-HORIZ-003 | Hidden columns right calculation | AC-3 | Created |
| TC-HORIZ-004 | Hidden columns left calculation | AC-4 | Created |
| TC-HORIZ-005 | Status bar column range text | AC-6 | Created |
| TC-HORIZ-006 | Vim h/l key handling | AC-8 | Created |
| TC-HORIZ-007 | H key jump to first column | AC-9 | Created |
| TC-HORIZ-008 | L key jump to last column | AC-10 | Created |
| TC-HORIZ-009 | Column position preserved during vertical scroll | AC-11 | Created |
| TC-HORIZ-010 | Visible column count calculation | Foundation | Created |

**Additional Unit Tests to Implement (15 more in code):**
- Bounds checking for col_offset
- Edge cases: single column, 0 columns, exact fit
- Indicator text generation (left/right)
- Help text content validation
- Integration with existing pager state

### Interactive Tests (25 tests) - `tests/interactive_tests.rs`

#### Core Navigation (AC-1 through AC-5)

| Test ID | Description | AC Coverage | Status |
|---------|-------------|-------------|--------|
| TC-HORIZ-011 | Right arrow scrolls right | AC-1 | Created |
| TC-HORIZ-012 | Left arrow scrolls left | AC-2 | Created |
| TC-HORIZ-013 | Right column indicator display | AC-3 | Created |
| TC-HORIZ-014 | Left column indicator display | AC-4 | Created |
| TC-HORIZ-015 | Pager exit returns to REPL (q/Esc) | AC-5 | Created |
| TC-HORIZ-016 | Status bar column range display | AC-6 | Created |
| TC-HORIZ-017 | Combined horizontal and vertical navigation | AC-7 | To Create |
| TC-HORIZ-018 | Vim h/l keys for horizontal navigation | AC-8 | To Create |
| TC-HORIZ-019 | H key jump to first column | AC-9 | To Create |
| TC-HORIZ-020 | L key jump to last column | AC-10 | To Create |

#### Integration and Advanced (AC-11 through AC-13)

| Test ID | Description | AC Coverage | Status |
|---------|-------------|-------------|--------|
| TC-HORIZ-021 | Column position preserved - vertical scroll | AC-11 | To Create |
| TC-HORIZ-022 | Help text shows horizontal navigation | AC-12 | To Create |
| TC-HORIZ-023 | /pager off disables paging | AC-13 | To Create |
| TC-HORIZ-024 | Right arrow at end - no effect | AC-1 edge | To Create |
| TC-HORIZ-025 | Left arrow at start - no effect | AC-2 edge | To Create |
| TC-HORIZ-026 | Complex keybinding sequence | Integration | To Create |
| TC-HORIZ-027 | Arrow keys and Vim keys interchangeable | AC-8 | To Create |
| TC-HORIZ-028 | Jump keys update indicators correctly | AC-9, AC-10 | To Create |
| TC-HORIZ-029 | Wide table (50+ columns) navigation | Edge | To Create |
| TC-HORIZ-030 | Narrow terminal adaptation | Edge | To Create |
| TC-HORIZ-031 | Status bar integrates row and column | AC-6 + vertical | To Create |
| TC-HORIZ-032 | Help text accessible during horizontal scroll | AC-12 | To Create |
| TC-HORIZ-033 | Multiple pager sessions preserve state | Integration | To Create |
| TC-HORIZ-034 | Rapid key presses (stress test) | Robustness | To Create |
| TC-HORIZ-035 | Single column table - no horizontal scroll | Edge | To Create |

### Regression Tests (10 tests) - `tests/interactive_tests.rs`

| Test ID | Description | What Not to Break | Status |
|---------|-------------|-------------------|--------|
| TC-REGR-001 | Vertical scrolling still works (j/k) | Existing vertical paging | To Create |
| TC-REGR-002 | Page up/down still works (Space/b) | Page navigation | To Create |
| TC-REGR-003 | Jump to top/bottom still works (g/G) | Jump navigation | To Create |
| TC-REGR-004 | Status bar shows correct row position | Row display | To Create |
| TC-REGR-005 | /pager off works for tall tables | Vertical paging disable | To Create |
| TC-REGR-006 | Pager exit (q) still safe | Safety requirement | To Create |
| TC-REGR-007 | Existing unit tests still pass | Unit test suite | To Create |
| TC-REGR-008 | Cell truncation still works | Truncation layer | To Create |
| TC-REGR-009 | Table formatting consistent | Border rendering | To Create |
| TC-REGR-010 | REPL commands work after paging | REPL integration | To Create |

### Edge Case Tests (12 tests) - Mixed unit + interactive

| Test ID | Description | Scenario | Status |
|---------|-------------|----------|--------|
| TC-EDGE-001 | Single column table - unit | col_offset stays 0 | To Create |
| TC-EDGE-002 | Exact terminal fit - unit | visible = total | To Create |
| TC-EDGE-003 | 50+ columns - unit | Large count handling | To Create |
| TC-EDGE-004 | Narrow terminal (40 cols) - unit | Min 1 column | To Create |
| TC-EDGE-005 | Wide terminal (300 cols) - unit | Many columns fit | To Create |
| TC-EDGE-006 | Single column table - interactive | No scroll visible | To Create |
| TC-EDGE-007 | Exact fit table - interactive | No indicators | To Create |
| TC-EDGE-008 | 50+ columns - interactive | Full navigation | To Create |
| TC-EDGE-009 | Narrow terminal - interactive | Adapts gracefully | To Create |
| TC-EDGE-010 | Wide terminal - interactive | Uses space well | To Create |
| TC-EDGE-011 | Empty result set | No crash | To Create |
| TC-EDGE-012 | Very wide columns (200+ chars) | Single column fits | To Create |

### Integration Tests (13 tests) - Keybinding combinations

| Test ID | Description | Key Sequence | Status |
|---------|-------------|--------------|--------|
| TC-INTEG-001 | Right scroll + down scroll | → → ↓ ↓ | To Create |
| TC-INTEG-002 | Jump end + up + jump start | L ↑ ↑ H | To Create |
| TC-INTEG-003 | Arrows + Vim keys mixed | → l → h | To Create |
| TC-INTEG-004 | Page down + horizontal scroll | Space h l | To Create |
| TC-INTEG-005 | Help during horizontal scroll | → → ? q | To Create |
| TC-INTEG-006 | All navigation modes combined | → ↓ l k H G Space | To Create |
| TC-INTEG-007 | Rapid alternating scroll | → ↓ ← ↑ → ↓ | To Create |
| TC-INTEG-008 | Jump + scroll + jump | H → → L ← H | To Create |
| TC-INTEG-009 | Vertical page + horizontal scroll | Space → b ← | To Create |
| TC-INTEG-010 | Multi-scroll then jump | → → → → → H | To Create |
| TC-INTEG-011 | Horizontal scroll at various row positions | See description | To Create |
| TC-INTEG-012 | Exit and re-enter pager | Query → q, Query → | To Create |
| TC-INTEG-013 | Column position across queries | Query1 → →, Query2 | To Create |

---

## Acceptance Criteria Coverage Map

| AC | Description | Unit Tests | Interactive Tests | Total Coverage |
|----|-------------|------------|-------------------|----------------|
| AC-1 | Right arrow scrolls right | TC-HORIZ-001 | TC-HORIZ-011, TC-HORIZ-024 | 3 tests |
| AC-2 | Left arrow scrolls left | TC-HORIZ-002 | TC-HORIZ-012, TC-HORIZ-025 | 3 tests |
| AC-3 | Right indicator `(+N cols)` | TC-HORIZ-003 | TC-HORIZ-013 | 2 tests |
| AC-4 | Left indicator `(+N cols)` | TC-HORIZ-004 | TC-HORIZ-014 | 2 tests |
| AC-5 | q/Esc exits to REPL | - | TC-HORIZ-015, TC-REGR-006 | 2 tests |
| AC-6 | Status bar column range | TC-HORIZ-005 | TC-HORIZ-016, TC-HORIZ-031 | 3 tests |
| AC-7 | Horizontal + vertical paging | - | TC-HORIZ-017, TC-HORIZ-026, TC-INTEG-001-011 | 13 tests |
| AC-8 | Vim h/l keys | TC-HORIZ-006 | TC-HORIZ-018, TC-HORIZ-027 | 3 tests |
| AC-9 | H jumps to first column | TC-HORIZ-007 | TC-HORIZ-019, TC-HORIZ-028 | 3 tests |
| AC-10 | L jumps to last column | TC-HORIZ-008 | TC-HORIZ-020, TC-HORIZ-028 | 3 tests |
| AC-11 | Column position preserved | TC-HORIZ-009 | TC-HORIZ-021, TC-INTEG-011 | 3 tests |
| AC-12 | Help shows horizontal controls | Unit in code | TC-HORIZ-022, TC-HORIZ-032 | 3 tests |
| AC-13 | /pager off disables paging | - | TC-HORIZ-023, TC-REGR-005 | 2 tests |

**Total AC Coverage:** All 13 acceptance criteria have multiple test coverage (2-13 tests each)

---

## Test Execution Order

1. **Unit Tests** (`cargo test --lib pager`) - Run first, fast feedback
2. **Integration Tests** (`cargo test --test integration_tests`) - Non-database tests
3. **Interactive Tests** (`cargo test --test interactive_tests -- --ignored`) - Requires database
4. **Manual UX Validation** - Human validates subjective quality

---

## Test Dependencies

### Required Test Data

| Table | Columns | Rows | Purpose |
|-------|---------|------|---------|
| test_wide_table_20 | 20 | 10 | Small wide table |
| test_wide_table_30 | 30 | 10 | Medium wide table |
| test_wide_table_32 | 32 | 10 | AC-6 example (32 cols) |
| test_wide_table_40 | 40 | 10 | Large wide table |
| test_wide_table_50 | 50 | 10 | Extreme width |
| test_wide_tall_table | 30 | 100 | Combined wide + tall |
| test_single_column | 1 | 10 | Edge case |

### Helper Functions to Implement

**In `tests/interactive_tests.rs`:**
- `setup_wide_test_table(n)` - Creates table with n columns
- `send_key(p, key)` - Sends KeyCode to PTY
- `extract_leftmost_column(output)` - Parses first column name
- `extract_column_range(output)` - Parses "Columns X-Y of Z"
- `extract_right_indicator_count(output)` - Parses "(+N cols)"
- `extract_left_indicator_count(output)` - Parses left indicator
- `read_available_output(p)` - Reads PTY output
- `create_test_table_wide_tall(cols, rows)` - Combined test data

**In `src/commands/repl/pager.rs` test module:**
- `create_test_table(n)` - Mock table with n columns
- `create_test_table_wide_tall(cols, rows)` - Mock wide + tall table

---

## Test Metrics Estimate

| Metric | Unit Tests | Interactive Tests | Total |
|--------|-----------|-------------------|-------|
| Test cases created | 25 | 35 | 70 |
| Expected assertions | ~100 | ~150 | ~250 |
| Execution time | <1 sec | 2-5 min | 2-5 min |
| Database required | No | Yes | - |
| PTY required | No | Yes | - |

---

## Coverage Gaps Accepted

1. **Cross-platform testing** - Only tested on development platform and CI (Linux)
   - Risk: LOW - crossterm provides cross-platform abstraction
2. **Extreme stress (1000+ columns)** - Not tested
   - Risk: LOW - Practical use cases <50 columns
3. **Subjective UX quality** - Not automated
   - Risk: LOW - Manual testing validates

---

## Notes

- Test case files TC-HORIZ-017 through TC-INTEG-013 need to be created (detailed specs below)
- All test files follow standard template from TC001.md
- Each test case includes: Metadata, Purpose, AC Coverage, Scope, Prerequisites, Procedure, Expected Results, Pass/Fail Criteria, Notes
- Interactive tests marked with `#[ignore]` - require live database
- Unit tests run without database or PTY
- Test strategy derived from `tests/strategy/sprint-29-test-strategy.md`
