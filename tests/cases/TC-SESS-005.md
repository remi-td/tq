# TC-SESS-005: Help Text Displays Correctly

**Test Case ID:** TC-SESS-005
**Feature:** Sessions Command - Help Integration
**Test Type:** Interactive (PTY)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that the `/help` metacommand includes `/sessions` in the list of available commands with an accurate description.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available (or can test without connection)
- [ ] TQ_LOGON environment variable set

---

## Test Steps

### Step 1: Start REPL
**Action:** Launch tq in REPL mode
```bash
tq repl
```

**Expected Result:**
- REPL starts successfully
- Prompt shows: `tq>`

### Step 2: Execute /help Command
**Action:** Type `/help` and press Enter
```
tq> /help
```

**Expected Result:**
- Help text displays
- Contains section listing available metacommands
- `/sessions` appears in the list

### Step 3: Verify /sessions Entry
**Action:** Inspect help output for `/sessions` line
```
Expected format:
  /sessions, /s    List active Teradata sessions with performance metrics
```

**Expected Result:**
- Entry shows both `/sessions` and `/s` alias
- Description mentions "sessions" and "performance metrics"
- Formatting matches other metacommand entries

### Step 4: Verify /s Alias Mentioned
**Action:** Check if alias is documented
```
Expected text includes:
- Primary command: /sessions
- Alias: /s
```

**Expected Result:**
- Alias `/s` is clearly documented
- User can discover the shorthand form

### Step 5: Exit REPL
**Action:** Type `/quit` and press Enter
```
tq> /quit
```

**Expected Result:**
- REPL exits cleanly

---

## Expected Results

### Success Criteria
- [x] `/help` command executes successfully
- [x] Output includes `/sessions` entry
- [x] Entry shows `/s` alias
- [x] Description is clear and accurate
- [x] Formatting consistent with other commands

### Sample Help Output
```
tq> /help

Available Commands:

  Connection:
    /logon            Connect to database
    /disconnect       Disconnect from database
    /ping             Test database connection

  Schema Inspection:
    /list databases   List all databases
    /list tables      List tables in database
    /describe <table> Show table structure

  Session Monitoring:
    /session          Show current session information
    /sessions, /s     List active Teradata sessions with performance metrics

  Utility:
    /help, /?         Show this help message
    /quit, /q         Exit REPL

Type /help <command> for detailed information.
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual Help Output:**
```
[Paste actual /help output here]
```

**Observations:**
- [Note if `/sessions` appears correctly]
- [Note if alias `/s` is documented]
- [Note description accuracy]
- [Note formatting consistency]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any help text issues]
- [List any missing information]

---

## Notes

- This test validates help text integration
- Can be automated with PTY tests
- Help text is important for discoverability
- Users rely on /help to learn metacommands

---

## Related Requirements

- AC-7: `/help` output includes `/sessions` command description
- REQ-SESS-006.3: Help text SHALL list `/sessions` command with description
- REQ-SESS-006.4: Description: "List active Teradata sessions with performance metrics"
