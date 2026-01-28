# TC-LICENSE-004: README Licensing Section

**Test Case ID:** TC-LICENSE-004
**Feature:** README Licensing Section (#8)
**Test Type:** Integration (Content Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that README.md contains a licensing section that informs users about tq's license and links to the LICENSE file.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists
- [ ] LICENSE file exists and validated

---

## Test Steps

### Step 1: Verify README.md Exists
**Action:** Check that README.md file exists
```bash
ls -la README.md
```

**Expected Result:**
- README.md exists at project root
- File is readable

### Step 2: Search for Licensing Section
**Action:** Look for licensing-related section headers
```bash
grep -i '^## License\|^## Licensing\|^# License' README.md
```

**Expected Result:**
- Licensing section header found
- Section is marked with markdown header (##)
- Common patterns: "## License", "## Licensing"

### Step 3: Verify Licensing Section Content
**Action:** Read the licensing section
```bash
# Extract License section and a few lines after
grep -A 10 -i '^## License' README.md
```

**Expected Result:**
- Section mentions MIT license (tq's base license)
- Section mentions third-party dependencies
- Section provides information to users about licensing

### Step 4: Verify Link to LICENSE File
**Action:** Check for link to LICENSE file
```bash
grep -i '\[LICENSE\]\|LICENSE file\|see LICENSE' README.md
```

**Expected Result:**
- README links to or mentions LICENSE file
- Users are directed to LICENSE for full terms
- Link is clear and functional

### Step 5: Verify Attribution Mention
**Action:** Check if README mentions third-party attributions
```bash
grep -i 'third.party\|attribution\|teradatarustapi' README.md
```

**Expected Result:**
- README mentions third-party dependencies are attributed
- May mention teradatarustapi specifically
- Users are informed about multi-license nature

### Step 6: Verify Licensing Information Placement
**Action:** Check that licensing section is in appropriate location (not at very top)
```bash
# Get line number of License section
grep -n -i '^## License' README.md
```

**Expected Result:**
- License section is NOT in first 50 lines (user-facing content comes first)
- License section is near bottom (standard README practice)
- Typical placement: After installation, usage, contributing sections

---

## Expected Results

### Success Criteria
- [x] README.md contains licensing section
- [x] Section is clearly marked with header
- [x] Section mentions MIT license (tq base license)
- [x] Section links to or references LICENSE file
- [x] Section mentions third-party dependencies/attributions
- [x] Section is placed appropriately (near bottom of README)

### Example Licensing Section (Reference)
```markdown
## License

tq is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

tq includes third-party dependencies (teradatarustapi and Go language libraries)
which are licensed under their respective licenses. All third-party attributions
are included in the LICENSE file.
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Licensing Section Header:**
```
$ grep -i '^## License' README.md
[Output - header line]
```

**Licensing Section Content:**
```
$ grep -A 10 -i '^## License' README.md
[Full section content]
```

**LICENSE File Link:**
```
$ grep -i '\[LICENSE\]\|LICENSE file' README.md
[Links or references to LICENSE file]
```

**Third-Party Mention:**
```
$ grep -i 'third.party\|attribution' README.md
[Mentions of third-party attributions]
```

**Section Placement:**
```
$ grep -n -i '^## License' README.md
[Line number - should be near bottom]

Total README lines: [X]
License section at line: [Y]
Percentage: [Y/X * 100]% through README (should be > 70%)
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: Licensing section present, mentions MIT and third-party, links to LICENSE
- FAIL: Section missing, incomplete, or doesn't link to LICENSE
- BLOCKED: README.md does not exist

**Defects Found:**
- [If FAIL: List what is missing or incorrect]

---

## Notes

- README licensing section is user-facing (different from LICENSE file itself)
- Section should be informative but brief (full details in LICENSE file)
- Standard practice: Licensing section near end of README
- Link to LICENSE file is important for users who want full terms
- Mentioning third-party attributions shows transparency

---

## Related Requirements

- AC-LICENSE-005: "README licensing section added" (sprint-27-planning.md:96)
- GitHub Issue #8: LICENSE - Transparency for users
- AC-README-XXX: README should be professional and complete (see issue #9)
