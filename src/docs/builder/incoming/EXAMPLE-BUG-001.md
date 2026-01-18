# BUG-001: REPL crashes on very wide tables

**Status:** Example (Not a Real Bug)
**Priority:** High
**Reported By:** Example User
**Date:** 2026-01-18

## Description

When querying a table with 100+ columns, the REPL crashes with a panic instead of gracefully handling the wide output.

## Steps to Reproduce

1. Connect to database with very wide table
2. Run: `SELECT * FROM wide_table;` (table with 120 columns)
3. REPL crashes with stack overflow error

## Expected Behavior

Should either:
- Display table with horizontal scrolling enabled automatically
- Warn user about wide table and suggest using JSON/CSV format
- Gracefully truncate columns with "..." indicator

## Actual Behavior

Application panics with:
```
thread 'main' panicked at 'stack overflow', src/format/table.rs:247
```

## Environment

- tq version: v1.5.0
- OS: macOS 14.2
- Teradata version: 17.20
- Terminal: iTerm2 3.4.19
- Terminal width: 180 columns

## Logs/Screenshots

[Stack trace would be included here]

## Impact

High - Crashes REPL session, loses command history and work in progress.

## Suggested Fix

Add validation in table formatter to detect very wide tables and either enable horizontal scrolling automatically or suggest alternative output formats.
