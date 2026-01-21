# TC-PROFILES-002: No Config File - Error Handling

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-PROFILES-002 |
| **Title** | No Config File - Error Handling |
| **Category** | Error-Handling |
| **Priority** | High |
| **Feature** | Sprint 17 - Profile Listing Command (P1) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that `tq profiles` command handles the absence of a configuration file gracefully by displaying a helpful error message with setup instructions, rather than failing silently or showing a cryptic error.

## Scope

This test validates:
- `tq profiles` error handling when config file doesn't exist
- Error message clarity and actionability
- Setup instructions provided
- Example configuration included
- Exit code 0 (no config is not an error, just empty state)
- Reference to `tq help config` for more information

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- **No config file present** (or use separate test directory)
- No database connection required

## Test Procedure

### Step 1: Ensure no config file exists
```bash
# Move existing config aside if present
if [ -f ~/.tq/config.toml ]; then
    mv ~/.tq/config.toml ~/.tq/config.toml.backup
fi

# Verify no config file
ls ~/.tq/config.toml 2>&1
# Should show "No such file or directory"
```

### Step 2: Execute tq profiles command
```bash
tq profiles
```

### Step 3: Verify exit code
```bash
tq profiles > /dev/null 2>&1
echo $?
```

### Step 4: Capture full output
```bash
tq profiles > /tmp/profiles-noconfig-output.txt 2>&1
cat /tmp/profiles-noconfig-output.txt
```

### Step 5: Validate output content
```bash
# Check for helpful keywords
grep -i "no configuration file" /tmp/profiles-noconfig-output.txt
grep -i "create" /tmp/profiles-noconfig-output.txt
grep -i "help config" /tmp/profiles-noconfig-output.txt
```

### Step 6: Cleanup - Restore original config if needed
```bash
if [ -f ~/.tq/config.toml.backup ]; then
    mv ~/.tq/config.toml.backup ~/.tq/config.toml
fi
```

## Expected Results

### Step 1 Output:
```
ls: ~/.tq/config.toml: No such file or directory
```
(Confirms no config file present)

### Step 2 Output:
```
No configuration file found at ~/.tq/config.toml

To create a configuration file with profiles:
  mkdir -p ~/.tq
  cat > ~/.tq/config.toml <<EOF
  [profiles.dev]
  host = "dev.company.com"
  port = 1025
  database = "development"
  user = "alice"
  password_file = "~/.tq/passwords/dev"
  EOF

See 'tq help config' for more information
```

**Message Requirements:**
- States config file not found
- Shows expected file path (~/.tq/config.toml)
- Provides step-by-step creation instructions
- Includes example profile with all common fields
- Example is copy-pasteable
- References `tq help config` for comprehensive documentation
- Tone is helpful, not critical

### Step 3 Output:
```
0
```
(Exit code 0 - No config file is not an error, just means no profiles defined)

**Rationale for exit code 0:**
- Config file is optional in tq design
- User may be exploring before setup
- Not an error condition, just empty state
- Consistent with "profiles listed successfully: 0 profiles"

### Step 4 Output:
Full message captured, matches expected output from Step 2.

### Step 5 Output:
All key phrases found:
- "no configuration file" or "not found"
- "create" or "To create"
- "help config" or "tq help config"

### Step 6 Output:
Original config restored if it existed.

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 0 (no config is not an error)
- ✅ Message clearly states config file not found
- ✅ Message shows expected file path
- ✅ Creation instructions provided (mkdir, cat, EOF pattern)
- ✅ Example profile included with common fields
- ✅ Example is syntactically valid TOML
- ✅ References `tq help config` for more details
- ✅ Tone is helpful and welcoming
- ✅ Message is actionable (user can fix immediately)

**FAIL if:**
- ❌ Exit code is non-zero (treating no-config as error)
- ❌ Cryptic error message ("file not found" with no context)
- ❌ No setup instructions provided
- ❌ Example profile missing or invalid TOML
- ❌ No reference to help system
- ❌ Tone is critical or unhelpful ("ERROR: Config missing!")

## Message Quality Checklist

### Clarity
- [ ] States "No configuration file found" or similar
- [ ] Shows exact expected path (~/.tq/config.toml)
- [ ] Explains this means no profiles available

### Actionability
- [ ] Provides creation steps (mkdir, create file)
- [ ] Steps are copy-pasteable shell commands
- [ ] Example profile is complete and valid
- [ ] Example uses realistic field values

### Helpfulness
- [ ] Tone is welcoming to new users
- [ ] Doesn't make user feel like they did something wrong
- [ ] References comprehensive help (`tq help config`)
- [ ] Example can be modified for user's needs

### Technical Correctness
- [ ] File path is correct for platform (POSIX shown, Windows noted if different)
- [ ] TOML syntax in example is valid
- [ ] Example includes password_file (secure practice)
- [ ] Example does NOT include inline password

## Edge Cases to Test

### Variation 1: Config directory doesn't exist
```bash
# Remove entire config directory
rm -rf ~/.tq

# Run tq profiles
tq profiles
```

**Expected:** Same helpful message, mentions creating directory first.

### Variation 2: Config directory exists but file doesn't
```bash
# Create directory, no file
mkdir -p ~/.tq
rm -f ~/.tq/config.toml

# Run tq profiles
tq profiles
```

**Expected:** Same helpful message (directory existing is fine).

### Variation 3: Config file exists but is empty
**Note:** This might be TC-PROFILES-003 scenario (no profiles defined).

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Message Content:**
- [ ] States config file not found
- [ ] Shows file path
- [ ] Provides creation instructions
- [ ] Includes example profile
- [ ] References tq help config

**Example Quality:**
- [ ] TOML syntax valid
- [ ] Fields realistic and complete
- [ ] Uses password_file (not inline password)

**Tone and Helpfulness:**

**Issues Found:**

## Notes

- No config file is a valid state for tq (config is optional)
- First-time users will encounter this - message quality is critical for onboarding
- Example should demonstrate best practices (password_file, not inline password)
- Message should empower user to self-serve (don't require external documentation)
- Platform differences: Windows path would be `%USERPROFILE%\.tq\config.toml`

**Alternative Design Consideration:**
Some tools use exit code 1 for "nothing to show" - but tq's design treats missing config as expected state, so 0 is appropriate. This is consistent with "profiles listed successfully: 0 profiles found".

**Related Test Cases:**
- TC-PROFILES-001: Successfully lists profiles when config exists
- TC-PROFILES-003: Config exists but no profiles section
- TC-HELP-001: `tq help config` provides comprehensive setup documentation

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.5 (lines 315-329)
- `docs/builder/sprints/sprint-17-planning.md` Feature 4 line 118
