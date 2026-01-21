# TC-HELP-001: Help Config Subcommand - Display Configuration Documentation

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HELP-001 |
| **Title** | Help Config Subcommand - Display Configuration Documentation |
| **Category** | Functionality |
| **Priority** | Critical |
| **Feature** | Sprint 17 - Help Subcommands (P0) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that the `tq help config` command displays comprehensive configuration file documentation including file locations, TOML format, profile fields, precedence order, and security best practices.

## Scope

This test validates:
- `tq help config` command execution
- Help content completeness (all required sections present)
- Documentation includes TOML examples
- Security best practices mentioned
- Exit code 0 on success
- Output to stdout (not stderr)

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- No external dependencies (no database needed)
- Command line access

## Test Procedure

### Step 1: Execute help config command
```bash
tq help config
```

### Step 2: Verify exit code
```bash
echo $?
```

### Step 3: Capture full output for content validation
```bash
tq help config > /tmp/help-config-output.txt 2>&1
cat /tmp/help-config-output.txt
```

### Step 4: Verify stdout vs stderr separation
```bash
tq help config > /tmp/help-config-stdout.txt 2> /tmp/help-config-stderr.txt
# Stderr should be empty
cat /tmp/help-config-stderr.txt
```

## Expected Results

### Step 1 Output:
Command should display comprehensive help text containing:

**Required Sections:**
1. **CONFIGURATION FILE** - File location paths
   - Should mention `~/.tq/config.toml` (macOS/Linux)
   - Should mention `%USERPROFILE%\.tq\config.toml` (Windows)
   - Should note file is optional

2. **FILE FORMAT (TOML)** - Configuration structure
   - Should include TOML example with `[defaults]` section
   - Should include TOML example with `[profiles.name]` section
   - Example should show key = "value" syntax

3. **PRECEDENCE ORDER** - Configuration hierarchy
   - Should list: Built-in defaults, User config, Environment variables, Command-line arguments
   - Order should be clear (later overrides earlier)

4. **PROFILE FIELDS** - Available profile settings
   - Should list required fields (host)
   - Should list optional fields (port, database, user, logmech, password_file, timeout)
   - Should show defaults where applicable

5. **SECURITY BEST PRACTICES** - Security guidance
   - Should warn against storing passwords in config
   - Should recommend password_file field
   - Should mention file permission recommendations

**Content Quality Expectations:**
- Text should be readable and well-formatted
- Examples should be syntactically correct TOML
- Guidance should be actionable
- References to related commands (e.g., `tq help credentials`, `tq profiles`)

### Step 2 Output:
```
0
```
(Success exit code)

### Step 3 Output:
Full help text captured successfully, content matches requirements above.

### Step 4 Output:
- stdout file should contain help text
- stderr file should be **empty** (no errors)

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 0
- ✅ Output written to stdout (not stderr)
- ✅ All 5 required sections present in output
- ✅ TOML examples are syntactically correct
- ✅ File locations mentioned
- ✅ Security best practices included
- ✅ Help text is readable and well-formatted
- ✅ References `tq help credentials` and `tq profiles`

**FAIL if:**
- ❌ Exit code is non-zero
- ❌ Any required section missing
- ❌ TOML examples have syntax errors
- ❌ No security guidance provided
- ❌ Output goes to stderr instead of stdout
- ❌ Help text is unreadable or poorly formatted

## Content Validation Checklist

Use this checklist to validate help output contains required information:

### Configuration File Section
- [ ] Mentions `~/.tq/config.toml`
- [ ] Notes file is optional
- [ ] Explains what happens if file missing

### File Format Section
- [ ] Shows `[defaults]` section example
- [ ] Shows `[profiles.name]` section example
- [ ] Examples use correct TOML syntax
- [ ] Common fields demonstrated (format, host, database, user, password_file)

### Precedence Section
- [ ] Lists all 4 levels: defaults, config file, env vars, CLI args
- [ ] Order is clear (numbered or explicit)
- [ ] Example demonstrating override behavior (optional but helpful)

### Profile Fields Section
- [ ] Distinguishes required vs optional fields
- [ ] Lists: host (required)
- [ ] Lists optional: port, database, user, logmech, password_file, timeout
- [ ] Shows default values where applicable

### Security Section
- [ ] Warns against inline passwords in config
- [ ] Recommends password_file field
- [ ] Mentions file permissions (0600 for password files)
- [ ] References `tq help credentials` for more detail

### Cross-References
- [ ] Mentions `tq help credentials` for password management
- [ ] Mentions `tq profiles` command for listing profiles
- [ ] References to documentation or examples

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Sections Present:**
- [ ] CONFIGURATION FILE
- [ ] FILE FORMAT
- [ ] PRECEDENCE ORDER
- [ ] PROFILE FIELDS
- [ ] SECURITY BEST PRACTICES

**Content Quality:**

**Issues Found:**

## Notes

- This test focuses on content presence and completeness
- Readability is subjective but should be assessed
- Help text should enable users to configure tq without external documentation
- TOML examples should be copy-pasteable
- Security guidance is critical - must be clear and prominent

**Related Test Cases:**
- TC-HELP-002: `tq help credentials` content validation
- TC-HELP-003: Unknown help topic error handling
- TC-PROFILES-001: Validates that `tq profiles` command works (referenced in help)

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.1
- `docs/builder/detailed-specifications/configuration.md` v2.1.0 §7.8.1
- `docs/builder/sprints/sprint-17-planning.md` Feature 1 (lines 51-68)
