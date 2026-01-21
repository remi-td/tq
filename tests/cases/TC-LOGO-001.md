---
id: TC-LOGO-001
title: Logo Display - Lowercase "tq" with Subtitle
category: Functionality
priority: Critical
sprint: 18
bug: Logo displays uppercase ASCII art instead of lowercase text
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-LOGO-001: Logo Display - Lowercase "tq" with Subtitle

## Purpose

Verify that the tq REPL displays a clean, professional logo with lowercase "tq" text in Teradata orange and includes the subtitle "Teradata Query tool v1.7.0" instead of uppercase ASCII art blocks.

## Scope

**Testing:**
- Logo text is lowercase "tq" (not uppercase ASCII blocks)
- Subtitle "Teradata Query tool v1.7.0" is present
- "tq" is displayed in Teradata orange color (xterm-256 color 202)
- Logo is simple and clean (no fancy ASCII art characters)

**Not Testing:**
- Other REPL functionality
- Logo display in batch mode (logo only appears in REPL)
- Color appearance in terminals without color support

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Terminal with xterm-256 color support (most modern terminals)
- Live Teradata database connection (for REPL to start)
- TQ_LOGON configured in .env or environment variable

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Ensure connection configured
# Check .env file has: TQ_LOGON=user:password@host:port/database
```

### Execution Steps

**Step 1: Start tq REPL**

```bash
./target/release/tq
```

**Expected outcome:**
- REPL starts successfully
- Banner displays at top of screen
- Connection message appears

**Step 2: Observe logo text content**

Look at the very first lines of output (the banner/logo).

**Visual Inspection:**
- [ ] Text "tq" appears in lowercase
- [ ] NO uppercase letters in logo
- [ ] NO ASCII block characters (████, ▄, ▀, etc.)
- [ ] Subtitle "Teradata Query tool v1.7.0" appears on next line

**Step 3: Verify color**

Observe the color of "tq" text.

**Visual Inspection:**
- [ ] "tq" text appears in orange color
- [ ] Color resembles Teradata brand orange (not red, not yellow)
- [ ] Subtitle text color is reasonable (can be white/gray/default)

**Step 4: Verify overall appearance**

Assess the professional appearance of the logo.

**Visual Inspection:**
- [ ] Logo is clean and simple
- [ ] Logo is easy to read
- [ ] Logo appears professional (not cluttered or over-designed)
- [ ] Logo takes minimal vertical space (2-3 lines max)

### Verification

**Text Content Validation:**

The banner should contain:
```
tq
Teradata Query tool v1.7.0
```

NOT contain:
```
████████   ████
   ██      ██  ██
```

**Color Code Validation (Technical):**

For automated testing, verify ANSI escape sequence contains color 202:
- Look for: `\x1b[38;5;202m` (xterm-256 color 202 foreground)
- Or similar ANSI sequence for orange color

**Anti-Pattern Detection:**

The following should NOT appear:
- ✗ "TQ" in uppercase
- ✗ Block characters: ████
- ✗ ASCII art blocks
- ✗ Excessive whitespace or padding
- ✗ Multiple colors (only "tq" should be colored, subtitle normal)

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**CORRECT Logo (Target):**

```
tq  (in orange color - xterm-256 color 202)
Teradata Query tool v1.7.0
```

**Key Characteristics:**
- Lowercase "tq" in orange
- Simple text (no fancy characters)
- Clean subtitle with version
- Professional appearance
- Minimal vertical space

**INCORRECT Logo (Current Bug):**

```
 ████████   ████
    ██      ██  ██
    ██      ██  ██
    ██      ██ ▄██
    ██       ████

 tq  v1.7.0
```

**Problems with current (bug):**
- Uses uppercase ASCII art blocks
- Takes excessive vertical space (6 lines)
- "tq" appears below art (not integrated)
- Missing full subtitle text
- Over-designed and cluttered

## Pass/Fail Criteria

**PASS:**
- ✅ Logo shows lowercase "tq" text (no ASCII art)
- ✅ Subtitle "Teradata Query tool v1.7.0" present
- ✅ "tq" appears in orange color (visual inspection confirms)
- ✅ Logo is clean, simple, professional
- ✅ No block ASCII art characters visible

**FAIL:**
- ❌ Logo shows uppercase ASCII art blocks
- ❌ Missing subtitle or incorrect subtitle text
- ❌ "tq" not in orange color (or no color)
- ❌ Logo contains fancy ASCII art
- ❌ Logo is cluttered or unprofessional

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Terminal]

**Observations:**
[Document what you actually saw]

**Screenshots:**
[If possible, attach screenshot showing actual logo]

**ANSI Color Code Detected:**
[If automated test, paste ANSI sequence found]

**Verdict:**
[PASS/FAIL with explanation]

## Notes

**Bug Context:**
- Sprint 17 attempted to fix logo color but only partially succeeded
- Logo still uses uppercase ASCII art blocks
- This is blocking production use (branding issue)

**Related Files:**
- `src/commands/repl/mod.rs` - print_banner function

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- Logo shows lowercase "tq" text (NOT ASCII block art)
- Logo includes subtitle "Teradata Query tool v1.7.0"
- "tq" is displayed in Teradata orange (xterm-256 color 202)
- Text is simple and clean (no fancy ASCII art)
- Banner matches branding guidelines

**Test Type:** Manual + Interactive (automated test also possible)

**Dependencies:** Terminal with color support, live database connection
