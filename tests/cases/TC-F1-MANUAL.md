# TC-F1-MANUAL: Metacommand Tab Completion

**Feature:** Metacommand Tab Completion (Sprint 22, Feature 1, P0)
**Test Type:** Manual Validation (PRIMARY TEST)
**Priority:** CRITICAL
**Author:** quality-validator
**Created:** 2026-01-23
**Sprint:** Sprint 22

---

## ⚠️ CRITICAL NOTE

**This is the PRIMARY TEST for Feature 1 (Metacommand Tab Completion).**

Automated tests (unit tests and PTY tests) CANNOT reliably validate TAB key completion behavior. Only manual validation can confirm that:
- TAB key actually completes metacommand text in the terminal
- Completion menu displays correctly
- Keyboard navigation works intuitively
- User experience is smooth and functional

**VERDICT GATE**: APPROVED verdict for Sprint 22 REQUIRES this manual test to PASS.

---

## Objective

Verify that TAB key completes metacommand text correctly and that completion menu displays metacommands with descriptions.

---

## Prerequisites

- `tq` REPL compiled and runnable
- No database connection required for this test (metacommand completion works offline)

---

## Test Procedure

### Test 1: Show All Metacommands

**Steps:**
1. Start `tq` REPL: `tq repl` (can run without database)
2. Type: `/`
3. Press: `TAB`
4. **Observe:** Menu shows all available metacommands

**Expected Result:**
- Completion menu appears
- Menu contains at least these metacommands:
  - `/describe`
  - `/export`
  - `/help`
  - `/list databases`
  - `/list tables`
  - `/list views`
  - `/logon`
  - `/pager`
  - `/ping`
  - `/quit`
  - `/session`
- Each metacommand shows a description
- First item in menu is highlighted

**Pass Criteria:**
- [ ] Menu appears after typing `/` + TAB
- [ ] All expected metacommands are shown
- [ ] Descriptions are visible next to each command
- [ ] Menu is readable and properly formatted

---

### Test 2: Complete Partial Metacommand (Single Match)

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/des`
3. Press: `TAB`
4. **Observe:** Text completes to `/describe` OR shows filtered menu

**Expected Result:**
- Either:
  - **Auto-complete:** Text immediately completes to `/describe` (if unambiguous)
  - **Filtered menu:** Shows only `/describe` and possibly `/disconnect` (if both match)

**Pass Criteria:**
- [ ] Typing `/des` + TAB either auto-completes or shows filtered menu
- [ ] Result includes `/describe`
- [ ] No unrelated commands shown (e.g., `/help`, `/ping` should NOT appear)

---

### Test 3: Complete Partial Metacommand (Multiple Matches)

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/l`
3. Press: `TAB`
4. **Observe:** Filtered completion menu appears

**Expected Result:**
- Menu shows only metacommands starting with `/l`:
  - `/list databases`
  - `/list tables`
  - `/list views`
  - `/logon`
- Other commands (e.g., `/describe`, `/ping`) NOT shown

**Pass Criteria:**
- [ ] Menu appears with filtered results
- [ ] Only `/l*` commands are shown
- [ ] At minimum: `/list` commands and `/logon` are present
- [ ] Unrelated commands are filtered out

---

### Test 4: Navigate and Accept Completion

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/l`
3. Press: `TAB` (menu appears)
4. Press: `DOWN ARROW` key (move to next item)
5. Press: `DOWN ARROW` again
6. Press: `ENTER` (accept highlighted item)
7. **Observe:** Selected metacommand is inserted into the line

**Expected Result:**
- Arrow keys navigate through menu (highlighting changes)
- ENTER key accepts the currently highlighted item
- Accepted command appears on the line
- Cursor positioned correctly after the command

**Pass Criteria:**
- [ ] DOWN ARROW moves highlight to next menu item
- [ ] UP ARROW moves highlight to previous menu item
- [ ] ENTER accepts the highlighted item
- [ ] Accepted command text appears on the line
- [ ] Cursor is positioned correctly (ready for arguments or execution)

---

### Test 5: Descriptions Displayed

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/`
3. Press: `TAB`
4. **Observe:** Menu displays metacommand descriptions

**Expected Result:**
- Each metacommand in the menu has a description
- Descriptions are meaningful (e.g., "Show help message", "Exit the REPL")
- Descriptions help user understand command purpose

**Pass Criteria:**
- [ ] All metacommands have visible descriptions
- [ ] Descriptions are meaningful and helpful
- [ ] Description format is consistent

---

### Test 6: Multi-line Mode

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `SELECT * FROM t1;` (complete SQL statement)
3. Press: `ENTER` (creates new line in multi-line mode)
4. Type: `/des`
5. Press: `TAB`
6. **Observe:** Metacommand completion works on second line

**Expected Result:**
- Metacommand completion works even after SQL input
- `/des` + TAB completes to `/describe` (or shows menu)
- Multi-line context doesn't break completion

**Pass Criteria:**
- [ ] Metacommand completion works after SQL input
- [ ] `/des` + TAB provides completions
- [ ] Behavior is same as on first line (Test 2)

---

### Test 7: Case Insensitivity

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/HE` (uppercase)
3. Press: `TAB`
4. **Observe:** Completion matches `/help`

**Expected Result:**
- Uppercase input matches lowercase metacommand
- Completion is case-insensitive

**Pass Criteria:**
- [ ] `/HE` + TAB completes to `/help`
- [ ] Case insensitivity works for all metacommands

---

### Test 8: ESC Dismisses Menu

**Steps:**
1. Clear line (Ctrl-U)
2. Type: `/`
3. Press: `TAB` (menu appears)
4. Press: `ESC`
5. **Observe:** Menu closes without making selection

**Expected Result:**
- ESC key dismisses the completion menu
- Original input (`/`) remains on the line
- Cursor position unchanged

**Pass Criteria:**
- [ ] ESC closes the menu
- [ ] Input line remains unchanged
- [ ] No command is auto-completed

---

## Evidence Collection

**Required Evidence:**
- [ ] Screenshots of completion menu showing metacommands with descriptions
- [ ] Screenshot or video of TAB key completing `/des` → `/describe`
- [ ] Screenshot of filtered menu for `/l` + TAB
- [ ] User confirmation: "I tested all scenarios and they work as expected"

**How to Collect:**
- **Screenshots:** Use OS screenshot tool (macOS: Cmd+Shift+4, Linux: screenshot tool)
- **Video (optional):** Use screen recording to show keyboard interaction
- **Written Confirmation:** Note any issues or unexpected behavior

---

## Acceptance Criteria Summary

✅ **PASS** if ALL of the following are true:
- [ ] All metacommands shown after `/` + TAB
- [ ] Partial completion works (`/des` → `/describe`)
- [ ] Filtered lists display correctly (`/l` → filtered menu)
- [ ] Descriptions visible in menu
- [ ] Keyboard navigation works (UP/DOWN/ENTER/ESC)
- [ ] Works in multi-line mode
- [ ] Case insensitive matching works
- [ ] User confirms "smooth and intuitive UX"

❌ **FAIL** if ANY of the following occur:
- TAB key does not trigger completion
- Menu does not appear or is unreadable
- Descriptions are missing or incorrect
- Keyboard navigation is broken
- Completion doesn't work in multi-line mode
- User experience is "confusing" or "broken"

---

## Notes

**False Positive Risk:** HIGH
Automated PTY tests may report PASS without validating actual keyboard behavior. This manual test is the ONLY reliable validation method.

**Sprint 21 Lesson:**
Manual validation is PRIMARY for keyboard/UX features. Automated tests provide regression detection but cannot validate user experience.

---

## Related Tests

- **Unit Tests:** `test_complete_metacommands_*` in `src/commands/repl/metadata_completer.rs`
- **PTY Test:** `test_metacommand_completion_output` in `tests/interactive_tests.rs` (INSUFFICIENT)
- **Specification:** `docs/specifications/repl.md` lines 283-381

---

## Test Result

**Date Executed:** _____________
**Tester:** _____________
**Verdict:** [ ] PASS  [ ] FAIL
**Notes:**

```
[Record any issues, unexpected behavior, or observations here]
```

**Evidence Files:**
- Screenshot 1: _____________
- Screenshot 2: _____________
- Video (if applicable): _____________

