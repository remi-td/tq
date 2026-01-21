# TC-PROFILES-003: Config Exists But No Profiles - Error Handling

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-PROFILES-003 |
| **Title** | Config Exists But No Profiles - Error Handling |
| **Category** | Error-Handling |
| **Priority** | High |
| **Feature** | Sprint 17 - Profile Listing Command (P1) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that `tq profiles` command handles a configuration file that exists but contains no profiles section gracefully, displaying a helpful message with instructions on adding profiles.

## Scope

This test validates:
- `tq profiles` handling when config file exists but has no profiles
- Error message clarity and helpfulness
- Profile addition instructions provided
- Example profile syntax included
- Exit code 0 (empty profiles is valid state, not an error)
- Reference to `tq help config` for comprehensive documentation

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- Ability to create test configuration file
- No database connection required

## Test Procedure

### Step 1: Create config file with defaults but no profiles
```bash
# Create config directory
mkdir -p ~/.tq-test

# Create config with defaults only (no profiles section)
cat > ~/.tq-test/config.toml <<'EOF'
[defaults]
format = "table"
editor_mode = "emacs"
syntax_highlighting = true
paging = true
timing = false
EOF

# Verify config has no profiles section
grep -c "profiles" ~/.tq-test/config.toml
# Should output 0
```

### Step 2: Execute tq profiles command
```bash
# Point tq to test config (method depends on implementation)
# May need: TQ_CONFIG_DIR=~/.tq-test tq profiles
# Or: tq --config ~/.tq-test/config.toml profiles
tq profiles  # Assuming default location or env var set
```

### Step 3: Verify exit code
```bash
tq profiles > /dev/null 2>&1
echo $?
```

### Step 4: Capture full output
```bash
tq profiles > /tmp/profiles-noprofiles-output.txt 2>&1
cat /tmp/profiles-noprofiles-output.txt
```

### Step 5: Validate output content
```bash
# Check for helpful keywords
grep -i "no profiles" /tmp/profiles-noprofiles-output.txt
grep -i "to add a profile" /tmp/profiles-noprofiles-output.txt
grep -i "help config" /tmp/profiles-noprofiles-output.txt
```

## Expected Results

### Step 1 Output:
Config file created with [defaults] section only.
```
0
```
(No "profiles" string found in config)

### Step 2 Output:
```
No profiles defined in ~/.tq/config.toml

To add a profile, edit ~/.tq/config.toml:
  [profiles.dev]
  host = "dev.company.com"
  port = 1025
  database = "development"
  user = "alice"
  password_file = "~/.tq/passwords/dev"

See 'tq help config' for more information
```

**Message Requirements:**
- States no profiles defined
- Shows config file path (confirms file was found)
- Provides instructions to add profile
- Shows example profile section with common fields
- Example is valid TOML that can be copy-pasted
- References `tq help config` for details
- Tone is instructional, not critical

### Step 3 Output:
```
0
```
(Exit code 0 - Config exists and is valid, just has no profiles. Not an error.)

**Rationale:**
- Config file is valid (has defaults section)
- No profiles is a legitimate state
- User may want defaults without profiles
- Not an error, just means 0 profiles available

### Step 4 Output:
Full message captured, matches expected output from Step 2.

### Step 5 Output:
All key phrases found:
- "no profiles" or "No profiles defined"
- "to add" or "edit"
- "help config"

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 0 (valid config with no profiles is not an error)
- ✅ Message clearly states no profiles defined
- ✅ Message shows config file path (proves file was found and read)
- ✅ Instructions for adding profile provided
- ✅ Example profile section included
- ✅ Example is valid TOML with common fields
- ✅ References `tq help config` for comprehensive docs
- ✅ Tone is helpful and instructional
- ✅ Distinguishes this case from "no config file" (TC-PROFILES-002)

**FAIL if:**
- ❌ Exit code is non-zero (treating no-profiles as error)
- ❌ Message doesn't clarify that config exists but lacks profiles
- ❌ No instructions for adding profiles
- ❌ Example missing or invalid TOML
- ❌ Confusing message (doesn't distinguish from "no config file" case)
- ❌ Tone is critical or unhelpful

## Message Quality Checklist

### Clarity
- [ ] States "No profiles defined in <path>" (not "No profiles found")
- [ ] Shows config file path (proves file exists and was read)
- [ ] Distinguishes from missing config file case

### Actionability
- [ ] Says "To add a profile, edit ~/.tq/config.toml"
- [ ] Shows example profile section
- [ ] Example is complete with all common fields
- [ ] Example is copy-pasteable

### Helpfulness
- [ ] Doesn't imply user did something wrong (valid state)
- [ ] Provides enough info to add profile without external docs
- [ ] References help for comprehensive information
- [ ] Example demonstrates best practices (password_file)

### Technical Correctness
- [ ] Example uses `[profiles.name]` section syntax
- [ ] TOML syntax is valid
- [ ] Field names match actual implementation
- [ ] Example includes password_file (not inline password)

## Edge Cases and Variations

### Variation 1: Config has empty profiles table
```toml
[defaults]
format = "table"

[profiles]
# No profiles defined yet
```

**Expected:** Should detect no profile definitions, show same helpful message.

### Variation 2: Config has malformed profile (syntax error)
**Note:** This is different - parse error should show different message (config invalid, not "no profiles").

### Variation 3: Config has only commented-out profiles
```toml
# [profiles.dev]
# host = "dev.company.com"
```

**Expected:** Comments aren't profiles, should show "no profiles defined".

### Variation 4: Config has defaults and other sections but no profiles
```toml
[defaults]
format = "json"

[other_section]
some_key = "value"
```

**Expected:** Should show "no profiles defined" (other sections are ignored).

## Comparison with Related Cases

| Scenario | TC | Config Exists? | Profiles Exist? | Exit Code | Message |
|----------|----|--------------|--------------------|-----------|---------|
| No config file | TC-PROFILES-002 | ❌ No | N/A | 0 | "No configuration file found at..." |
| Config exists, no profiles | TC-PROFILES-003 | ✅ Yes | ❌ No | 0 | "No profiles defined in..." |
| Config exists, has profiles | TC-PROFILES-001 | ✅ Yes | ✅ Yes | 0 | Lists all profiles |

**Key Distinction:**
- TC-PROFILES-002: File doesn't exist → "No configuration file found"
- TC-PROFILES-003: File exists → "No profiles defined in <file>"

## Actual Results

_To be filled during test execution_

**Exit Code:**

**Message Content:**
- [ ] States no profiles defined
- [ ] Shows config file path
- [ ] Provides edit instructions
- [ ] Includes example profile

**Example Quality:**
- [ ] TOML syntax valid
- [ ] Fields complete and realistic
- [ ] Uses password_file (not inline password)

**Message Clarity:**
- [ ] Distinguishes from "no config file" case
- [ ] Tone is helpful

**Issues Found:**

## Notes

- This is a common state: User creates config with defaults, adds profiles later
- Message should make it clear config exists and is valid (just needs profiles)
- Example should use `[profiles.name]` syntax (not just listing fields)
- User should be able to copy-paste example directly into their config file
- Exit code 0 is correct: Config is valid, just has 0 profiles

**User Journey Context:**
1. User creates config with `tq help config` guidance
2. User sets defaults (format, editor_mode)
3. User runs `tq profiles` to see what they have
4. Gets this message, adds first profile
5. Runs `tq profiles` again, sees TC-PROFILES-001 success output

**Related Test Cases:**
- TC-PROFILES-001: Config with profiles - successful listing
- TC-PROFILES-002: No config file - different message
- TC-HELP-001: `tq help config` explains profile configuration

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.5 (lines 333-344)
- `docs/builder/sprints/sprint-17-planning.md` Feature 4 line 119
