# TC-README-004: README Installation Instructions

**Test Case ID:** TC-README-004
**Feature:** README Clear Installation Instructions (#9)
**Test Type:** Integration (Content Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that README.md contains clear, concise, and complete installation instructions that enable users to get started with tq quickly.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists
- [ ] Understanding of tq installation methods

---

## Test Steps

### Step 1: Verify Installation Section Exists
**Action:** Look for Installation or Quick Start section
```bash
grep -i '^## Installation\|^## Quick Start\|^## Getting Started' README.md
```

**Expected Result:**
- Installation section header found
- Section is clearly marked with ##
- Common patterns: "## Installation", "## Quick Start"

### Step 2: Verify Installation Section Content
**Action:** Read installation section
```bash
grep -A 20 -i '^## Installation' README.md
```

**Expected Result:**
- Section contains installation instructions
- Instructions are present (not empty placeholder)
- Multiple lines of content (not just header)

### Step 3: Verify Cargo Install Method (Primary)
**Action:** Check for Rust/Cargo installation instructions
```bash
grep -i 'cargo install\|cargo build' README.md
```

**Expected Result:**
- Cargo installation method documented
- Instructions clear: `cargo install tq` or `cargo build --release`
- Rust-based installation is primary method

### Step 4: Verify Prerequisites Documented
**Action:** Check for prerequisite documentation
```bash
grep -i 'prerequisite\|requirement\|rust toolchain\|cargo' README.md
```

**Expected Result:**
- Prerequisites mentioned (Rust toolchain, Cargo)
- Users know what they need before installing
- Links to rustup or Rust installation if needed

### Step 5: Verify Installation Steps are Clear
**Action:** Read installation section for clarity

**Review Criteria:**
- [ ] Steps are numbered or bulleted (not wall of text)
- [ ] Commands are in code blocks for easy copy-paste
- [ ] Each step is actionable (clear what to do)
- [ ] No ambiguous instructions

**Expected Result:**
- Installation instructions are easy to follow
- New users can complete installation successfully
- Professional presentation

### Step 6: Verify Post-Installation Verification
**Action:** Check if README includes verification step
```bash
grep -i 'verify\|tq --version\|tq --help' README.md
```

**Expected Result:**
- README suggests how to verify installation worked
- Common patterns: "Run `tq --version`", "Test with `tq --help`"
- Helps users confirm successful install

### Step 7: Verify Installation Section Placement
**Action:** Check where installation section appears
```bash
grep -n -i '^## Installation\|^## Quick Start' README.md
```

**Expected Result:**
- Installation section is early in README (< 100 lines)
- Appears after What/Visual, before detailed usage
- User-focused ordering

---

## Expected Results

### Success Criteria
- [x] Installation section exists and is clearly marked
- [x] Cargo installation method documented
- [x] Prerequisites mentioned (Rust toolchain)
- [x] Installation steps are clear and actionable
- [x] Commands in code blocks for easy copy-paste
- [x] Post-installation verification suggested
- [x] Section is appropriately placed (early in README)

### Example Installation Section (Reference)
```markdown
## Installation

### Prerequisites
- Rust toolchain (install via [rustup](https://rustup.rs/))

### Install from crates.io
```bash
cargo install tq
```

### Build from source
```bash
git clone https://github.com/user/tq.git
cd tq
cargo build --release
```

### Verify installation
```bash
tq --version
```
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Installation Section Header:**
```
$ grep -i '^## Installation' README.md
[Section header]
```

**Installation Section Content:**
```
$ grep -A 20 -i '^## Installation' README.md
[Full section content]
```

**Cargo Install Method:**
```
$ grep -i 'cargo install\|cargo build' README.md
[Cargo commands]
```

**Prerequisites Documentation:**
```
$ grep -i 'prerequisite\|requirement\|rust' README.md
[Prerequisites mentioned]
```

**Post-Installation Verification:**
```
$ grep -i 'tq --version\|verify' README.md
[Verification steps]
```

**Section Placement:**
```
$ grep -n -i '^## Installation' README.md
[Line number - should be < 100]
```

**Clarity Assessment:**
```
Steps numbered/bulleted: [YES/NO]
Commands in code blocks: [YES/NO]
Each step actionable: [YES/NO]
No ambiguous instructions: [YES/NO]
Overall clarity: [CLEAR / UNCLEAR]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: Installation section clear, complete, cargo install documented
- FAIL: Section missing, unclear, or incomplete
- BLOCKED: README.md does not exist

**Clarity Issues:**
- [If FAIL: Describe what is unclear or missing]

---

## Notes

- AC-README-004: "Installation instructions clear and concise" (sprint-27-planning.md:103)
- Clear installation is critical for user onboarding
- tq is Rust project, so cargo install is primary method
- Prerequisites (Rust toolchain) should be mentioned
- Commands in code blocks help users copy-paste accurately
- Verification step builds user confidence

---

## Related Requirements

- AC-README-004: "Installation instructions clear and concise" (sprint-27-planning.md:103)
- AC-README-001: "TLDR introduction section (What/Visual/Quick Start)" - Quick Start includes install
- GitHub Issue #9: README should be user-focused (good installation UX)
- User Journey: New user needs to install tq quickly and successfully
