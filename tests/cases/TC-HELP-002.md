# TC-HELP-002: Help Credentials Subcommand - Display Password Management Guide

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HELP-002 |
| **Title** | Help Credentials Subcommand - Display Password Management Guide |
| **Category** | Functionality |
| **Priority** | Critical |
| **Feature** | Sprint 17 - Help Subcommands (P0) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that the `tq help credentials` command displays comprehensive password and credential management documentation including security warnings, password file format, creation steps, and password source priority.

## Scope

This test validates:
- `tq help credentials` command execution
- Help content completeness (all required sections present)
- Security warnings prominent and clear
- Password file format documented
- Creation instructions provided
- Exit code 0 on success
- Output to stdout (not stderr)

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- No external dependencies (no database needed)
- Command line access

## Test Procedure

### Step 1: Execute help credentials command
```bash
tq help credentials
```

### Step 2: Verify exit code
```bash
echo $?
```

### Step 3: Capture full output for content validation
```bash
tq help credentials > /tmp/help-credentials-output.txt 2>&1
cat /tmp/help-credentials-output.txt
```

### Step 4: Verify stdout vs stderr separation
```bash
tq help credentials > /tmp/help-credentials-stdout.txt 2> /tmp/help-credentials-stderr.txt
# Stderr should be empty
cat /tmp/help-credentials-stderr.txt
```

## Expected Results

### Step 1 Output:
Command should display comprehensive help text containing:

**Required Sections:**
1. **PASSWORD SECURITY** - Security warnings and best practices
   - Should warn NEVER use passwords in command-line arguments
   - Should show insecure example with warning
   - Should show secure alternative (password file)
   - Should emphasize visibility risks (ps, history)

2. **PASSWORD FILES** - File format and requirements
   - Should describe format: single line with password
   - Should show example: `echo "mypassword" > ~/.tq/passwords/dev`
   - Should specify required permissions: 0600
   - Should explain enforcement (not just warning)

3. **CREATING A PASSWORD FILE** - Step-by-step instructions
   - Should show directory creation: `mkdir -p ~/.tq/passwords`
   - Should show directory permissions: `chmod 0700 ~/.tq/passwords`
   - Should show file creation: `echo "password" > file`
   - Should show file permissions: `chmod 0600 file`
   - Instructions should be copy-pasteable

4. **PASSWORD SOURCES** - Priority order
   - Should list all 5 sources in priority order:
     1. Connection string (discouraged)
     2. --password-file flag
     3. Profile password_file field
     4. TQ_PASSWORD environment variable (discouraged)
     5. Interactive prompt (secure)
   - Should mark discouraged methods clearly

5. **SECURITY ENFORCEMENT** - Permission enforcement details
   - Should explain that password files MUST have 0600 permissions
   - Should note enforcement (not warning) for password files
   - Should explain config files have different rules (warning only)
   - Should explain rationale (security vs convenience)

**Content Quality Expectations:**
- Security warnings should be prominent (NEVER, ALWAYS, MUST)
- Examples should include both bad (with warnings) and good approaches
- Instructions should be platform-appropriate (POSIX)
- Cross-references to `tq help config` for configuration

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
- ✅ Security warnings are prominent and clear
- ✅ Password file format documented with example
- ✅ Creation steps are complete and copy-pasteable
- ✅ Password source priority order listed
- ✅ Permission enforcement explained (0600 required)
- ✅ Distinguishes password file enforcement from config file warning
- ✅ References `tq help config` for related information

**FAIL if:**
- ❌ Exit code is non-zero
- ❌ Any required section missing
- ❌ Security warnings weak or buried in text
- ❌ No clear guidance on creating password files
- ❌ Permission requirements not explained
- ❌ Does not distinguish enforcement vs warning
- ❌ Output goes to stderr instead of stdout

## Content Validation Checklist

Use this checklist to validate help output contains required information:

### Password Security Section
- [ ] Warns NEVER use passwords in CLI arguments
- [ ] Shows insecure example: `tq -l "user:pass@host"`
- [ ] Shows secure alternative: `tq --password-file ...`
- [ ] Explains visibility risks (ps, history, audit logs)
- [ ] Uses strong language (NEVER, ALWAYS, MUST)

### Password Files Section
- [ ] Describes format: single line with password
- [ ] Shows creation example
- [ ] Specifies 0600 permissions required
- [ ] Explains enforcement (not warning)
- [ ] Notes tq will refuse to read insecure files

### Creating Password File Section
- [ ] Step 1: Create directory (`mkdir -p ~/.tq/passwords`)
- [ ] Step 2: Set directory permissions (`chmod 0700`)
- [ ] Step 3: Create file (`echo "password" > file`)
- [ ] Step 4: Set file permissions (`chmod 0600`)
- [ ] All steps are copy-pasteable shell commands

### Password Sources Section
- [ ] Lists 5 sources in priority order
- [ ] Connection string marked discouraged
- [ ] --password-file flag (recommended)
- [ ] Profile password_file field (recommended)
- [ ] TQ_PASSWORD env var marked discouraged
- [ ] Interactive prompt (secure fallback)

### Security Enforcement Section
- [ ] Password files MUST be 0600
- [ ] tq refuses to read insecure password files
- [ ] Error (not warning) for insecure password files
- [ ] Config files have different rule (warning only)
- [ ] Explains why different (passwords vs metadata)

### Cross-References
- [ ] References `tq help config` for configuration details
- [ ] References profile configuration examples

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Sections Present:**
- [ ] PASSWORD SECURITY
- [ ] PASSWORD FILES
- [ ] CREATING A PASSWORD FILE
- [ ] PASSWORD SOURCES
- [ ] SECURITY ENFORCEMENT

**Content Quality:**

**Security Warnings Prominent:**

**Issues Found:**

## Notes

- Security warnings must be impossible to miss
- Help should enable secure password management without external docs
- Examples should be POSIX-compliant (Linux/macOS focus)
- Windows users may need different commands (document or note)
- Permission enforcement is a key Sprint 17 feature - must be clearly explained

**Related Test Cases:**
- TC-HELP-001: `tq help config` content validation
- TC-HELP-003: Unknown help topic error handling
- TC-SECURITY-001: Validates 0644 password file is rejected (enforcement tested)
- TC-SECURITY-002: Validates config file 0644 only warns (different behavior)

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.1
- `docs/builder/detailed-specifications/configuration.md` v2.1.0 §7.8.3 (lines 686-742)
- `docs/builder/sprints/sprint-17-planning.md` Feature 1 (lines 51-68)
