# TC-HELP-003: Help Unknown Topic - Error Handling

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HELP-003 |
| **Title** | Help Unknown Topic - Error Handling |
| **Category** | Error-Handling |
| **Priority** | High |
| **Feature** | Sprint 17 - Help Subcommands (P0) |
| **Sprint** | 17 |
| **Created** | 2026-01-21 |
| **Updated** | 2026-01-21 |

## Purpose

Verify that `tq help <unknown-topic>` properly handles unknown help topics by displaying a clear error message with a list of available topics and exiting with appropriate error code.

## Scope

This test validates:
- Unknown help topic error handling
- Error message clarity and actionability
- List of available topics displayed
- Exit code 2 (usage error)
- Error output to stderr (not stdout)

## Prerequisites

- tq binary built and available (Sprint 17 implementation)
- No external dependencies (no database needed)
- Command line access

## Test Procedure

### Step 1: Execute help with unknown topic
```bash
tq help unknown
```

### Step 2: Verify exit code
```bash
tq help unknown > /dev/null 2>&1
echo $?
```

### Step 3: Test various unknown topics
```bash
# Test different unknown topic names
tq help invalid 2>&1
tq help foo 2>&1
tq help database 2>&1  # Similar to valid topic but not implemented
```

### Step 4: Verify stderr vs stdout separation
```bash
tq help unknown > /tmp/help-unknown-stdout.txt 2> /tmp/help-unknown-stderr.txt

# Stdout should be empty
cat /tmp/help-unknown-stdout.txt

# Stderr should contain error
cat /tmp/help-unknown-stderr.txt
```

### Step 5: Validate available topics list
```bash
tq help unknown 2>&1 | grep -i "available"
```

## Expected Results

### Step 1 Output:
Error message should be displayed on stderr:
```
Error: Unknown help topic 'unknown'

Available topics:
  config       Configuration file format and usage
  credentials  Password and credential management

Use 'tq help <topic>' for detailed help on a topic
```

**Error Message Requirements:**
- Clearly states topic is unknown
- Shows the invalid topic name provided
- Lists all available topics (config, credentials)
- Provides brief description of each topic
- Suggests correct usage pattern

### Step 2 Output:
```
2
```
(Exit code 2 for usage error)

### Step 3 Output:
All unknown topics should produce similar error messages:
- "Unknown help topic 'invalid'"
- "Unknown help topic 'foo'"
- "Unknown help topic 'database'"

Each should list available topics.

### Step 4 Output:
- **stdout** file should be **empty** (no output on success stream)
- **stderr** file should contain full error message with available topics

### Step 5 Output:
Should find "available" or "Available" in error output, confirming topics are listed.

## Pass/Fail Criteria

**PASS if:**
- ✅ Exit code is 2 (usage error, not 0 or 1)
- ✅ Error output goes to stderr (not stdout)
- ✅ Error message clearly identifies unknown topic
- ✅ Error message shows the topic name provided
- ✅ Available topics are listed (config, credentials)
- ✅ Error message is actionable (suggests correct usage)
- ✅ Consistent behavior for all unknown topic names

**FAIL if:**
- ❌ Exit code is 0 (success) or 1 (generic error)
- ❌ Error goes to stdout instead of stderr
- ❌ Error message is unclear or unhelpful
- ❌ Available topics not listed
- ❌ Error message does not show what topic was invalid
- ❌ Different unknown topics produce inconsistent errors

## Error Message Quality Checklist

### Clarity
- [ ] States "Unknown help topic" or similar clear error
- [ ] Includes the invalid topic name in quotes
- [ ] Uses "Error:" prefix for easy identification

### Actionability
- [ ] Lists all available topics
- [ ] Shows brief description of each topic
- [ ] Suggests correct usage: `tq help <topic>`
- [ ] Does not just say "see --help" (should be self-contained)

### Consistency
- [ ] Same error format for all unknown topics
- [ ] Available topics list always complete
- [ ] Exit code always 2 for usage errors

### User Experience
- [ ] Error is respectful (not condescending)
- [ ] Format is readable (not wall of text)
- [ ] Topics listed in logical order (alphabetical or by importance)

## Test Variations

Test multiple unknown topic variations to ensure robustness:

### Common Typos
```bash
tq help confg      # Missing 'i' in config
tq help credential # Singular instead of plural
tq help creds      # Abbreviation
```

### Case Sensitivity
```bash
tq help Config     # Capitalized
tq help CONFIG     # All caps
```

### Similar but Invalid
```bash
tq help configuration  # Similar to config
tq help password       # Similar to credentials
tq help profiles       # Real command but not help topic
```

**Expected:** All should show consistent error with available topics list.

## Actual Results

_To be filled during test execution_

**Exit Code for Unknown Topic:**

**Error Output:**

**Available Topics Listed:**
- [ ] config
- [ ] credentials

**Error Message Quality:**
- [ ] Clear
- [ ] Actionable
- [ ] Consistent across variations

**Issues Found:**

## Notes

- Exit code 2 is important for shell scripting (usage error vs runtime error)
- Error should go to stderr (UNIX convention)
- Listing available topics is critical for discoverability
- Error should be helpful, not just "topic not found"
- This is a user's first experience when they make a mistake - should be welcoming

**Edge Cases to Consider:**
- Empty topic: `tq help` (should show general help, not error)
- No arguments: Same as `tq help` or `tq --help`
- Special characters in topic: `tq help "config@#$"`
- Very long topic name: `tq help verylongtopicnamethatdoesnotexist`

**Related Test Cases:**
- TC-HELP-001: Valid topic `config` works correctly
- TC-HELP-002: Valid topic `credentials` works correctly

**Specification References:**
- `docs/builder/detailed-specifications/cli-interface.md` v1.2.0 §4.4.1 (lines 94-97)
- `docs/builder/sprints/sprint-17-planning.md` Feature 1 line 59
- Exit code standards: `docs/builder/detailed-specifications/cli-interface.md` §4.5.3
