# Current bug and urgent issues

Updated on 2026-01-22 at 23:45

## BUG 1: Logo - Need Block Characters

**Status:** ✅ FIXED (pending manual verification)

The logo now uses block characters for clearer display.

**New Design:**
```
 ▀▀█▀▀     █▀▀█
   █      █   █
   █      █▄▄█
```

Where 't' (left) is in Teradata orange and 'q' (right) is in default color.

## BUG 2: Tab Completion Shows Pager Output

**Status:** ✅ FIXED (pending manual verification - enhanced to suppress stderr)

**Original Problem:**
Tab completion showed pager-style output instead of completion menu.

**User Report:**
```
tq> ? select * from dbc.[TAB]
Page 10: records 0 - 0  total: 0  [FULL]
```

**Expected:**
- After `select * from [TAB]`: Show dropdown of database names
- After `select * from dbc.[TAB]`: Show dropdown of tables in DBC database

**Root Cause Analysis:**
The "Page 10: records..." output was coming from teradatarustapi Go library. Sprint 19's StdoutSuppressor approach FAILED because it only redirected stdout, but the library writes to stderr.

**Fix Applied (Sprint 20):**
Enhanced OutputSuppressor now redirects BOTH stdout (fd 1) AND stderr (fd 2) to /dev/null during metadata queries. This should eliminate all output from teradatarustapi during tab completion.

**Architecture Confirmed:**
- Database names: ✅ Cached (extracted from table cache)
- Tables: ✅ Cached (loaded once on first TAB press)
- Columns: ✅ Cached per table as needed
- Output suppression: ✅ NOW SUPPRESSES BOTH STDOUT AND STDERR

**Manual Verification Needed:**
Please test the following:
1. Build: `cargo build`
2. Launch: `./target/debug/tq repl`
3. Type: `select * from ` and press TAB
   - Expected: Dropdown menu of database names (NO "Page X: records..." output)
4. Type: `select * from dbc.` and press TAB
   - Expected: Dropdown menu of tables in DBC (NO pager output)
5. Report: Does it work correctly now?

If the output still appears, it means teradatarustapi is writing directly to TTY, which would require a different approach (pre-loading metadata before REPL starts).
