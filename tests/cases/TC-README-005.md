# TC-README-005: README Documentation Links

**Test Case ID:** TC-README-005
**Feature:** README Links to Roadmap and Documentation (#9)
**Test Type:** Integration (Link Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that README.md contains links to roadmap and documentation, enabling users to find detailed information and project status easily.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists
- [ ] Documentation files exist:
  - docs/roadmap/ directory
  - docs/ directory with specifications

---

## Test Steps

### Step 1: Search for Documentation Links Section
**Action:** Look for documentation or links section
```bash
grep -i '^## Documentation\|^## Links\|^## Resources\|^## Further Reading' README.md
```

**Expected Result:**
- Documentation links section header found
- Section dedicated to additional resources

### Step 2: Search for Roadmap Links
**Action:** Look for roadmap references
```bash
grep -i 'roadmap\|\[roadmap\]' README.md
```

**Expected Result:**
- Roadmap mentioned or linked in README
- Link points to docs/roadmap/ directory

### Step 3: Verify Roadmap Link Syntax
**Action:** Extract roadmap links
```bash
grep -o '\[.*roadmap.*\]([^)]*)' README.md
```

**Expected Result:**
- Markdown link syntax is correct: `[Roadmap](docs/roadmap/)`
- Link is clickable in GitHub rendering

### Step 4: Search for Documentation Links
**Action:** Look for general documentation references
```bash
grep -i 'documentation\|\[docs\]\|\[documentation\]' README.md
```

**Expected Result:**
- Documentation mentioned or linked
- Links point to docs/ directory or specific doc files

### Step 5: Verify Link Paths Exist
**Action:** Check that linked paths actually exist
```bash
# Extract link targets and verify
ls -la docs/roadmap/ 2>/dev/null && echo "Roadmap directory exists"
ls -la docs/ 2>/dev/null && echo "Documentation directory exists"
```

**Expected Result:**
- docs/roadmap/ directory exists
- docs/ directory exists
- Linked paths are valid (no broken links)

### Step 6: Verify Links to Key Documentation Files
**Action:** Check for links to important docs
```bash
grep -i 'specifications\|design\|testing' README.md
```

**Expected Result:**
- May include links to key documentation areas:
  - docs/specifications/
  - docs/design/
  - docs/testing/
- Not required, but enhances navigation

### Step 7: Verify Link Placement
**Action:** Check where documentation links appear
```bash
grep -n -i 'roadmap\|documentation' README.md
```

**Expected Result:**
- Links appear in appropriate location
- Typical placement: After installation/usage, before contributing
- Not hidden at very end

---

## Expected Results

### Success Criteria
- [x] README contains links to roadmap
- [x] README contains links to documentation
- [x] Markdown link syntax is correct
- [x] Linked paths exist (no broken links)
- [x] Links are appropriately placed
- [x] Users can navigate to roadmap and docs easily

### Example Documentation Links Section
```markdown
## Documentation

For detailed information:
- [Roadmap](docs/roadmap/) - Project roadmap and feature status
- [Specifications](docs/specifications/) - Feature specifications
- [Design Documents](docs/design/) - Technical design documentation
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Documentation Links Section:**
```
$ grep -i '^## Documentation' README.md
[Section header]
```

**Roadmap Links:**
```
$ grep -i 'roadmap' README.md
[All roadmap references]
```

**Documentation Links:**
```
$ grep -i 'documentation\|docs/' README.md
[All documentation references]
```

**Link Syntax Validation:**
```
$ grep -o '\[.*\](docs/[^)]*)' README.md
[Extracted markdown links]
```

**Link Path Verification:**
```
$ ls -la docs/roadmap/
[Directory contents - should exist]

$ ls -la docs/
[Directory contents - should exist]
```

**Link Placement:**
```
$ grep -n -i 'roadmap\|documentation' README.md | head -5
[Line numbers - should be in middle section of README]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: Roadmap and documentation links present, valid paths, proper syntax
- FAIL: Links missing, broken paths, or incorrect syntax
- BLOCKED: README.md does not exist

**Broken Links:**
- [If FAIL: List which links are broken or missing]

---

## Notes

- AC-README-005: "Links to roadmap and documentation" (sprint-27-planning.md:104)
- Links help users navigate from README to detailed docs
- Roadmap shows users what features exist and what's planned
- Documentation provides detailed specifications and usage
- Broken links hurt user experience
- Links should use relative paths (docs/roadmap/) not absolute

---

## Related Requirements

- AC-README-005: "Links to roadmap and documentation" (sprint-27-planning.md:104)
- GitHub Issue #9: README should link to project resources
- Roadmap location: docs/roadmap/
- Documentation locations: docs/specifications/, docs/design/, docs/testing/
