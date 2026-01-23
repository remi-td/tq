# Export Command Syntax Simplification - Design Document

**Sprint:** 13
**Date:** 2026-01-19
**Author:** CLI UX Designer
**Status:** Design Complete - Pending Implementation

---

## Problem Statement

The current `/export` command has confusing, inconsistent semantics with multiple syntax variations that create cognitive friction for users.

### Current Confusing Syntax

```
/export <format> [file]
/export <format> clipboard
/export clipboard [format]
/export <format> --append [file]
```

**Problems:**
1. **Inconsistent parameter order** - `clipboard` can be 2nd or 1st parameter
2. **Ambiguous semantics** - Is `clipboard` a destination or a special command?
3. **Non-discoverable** - Users must read help text to understand options
4. **Cognitive load** - Multiple patterns to remember
5. **Help text mismatch** - `/help` shows different syntax than `/export` help

**From open-bugs.md:**
```
Make it simply /export <format> [file|clipboard] so semantics are clear!
```

---

## Design Goals

1. **Simplicity** - One clear syntax pattern, easy to remember
2. **Consistency** - Same parameter order for all destinations
3. **Discoverability** - Obvious what each parameter means
4. **Backwards Compatibility** - Don't break existing user scripts
5. **UNIX Philosophy** - Predictable behavior aligned with standard CLI tools

---

## Proposed Solution

### New Unified Syntax

```
/export <format> [destination]
```

**Where:**
- `<format>` is REQUIRED: `table`, `csv`, `json`, `sql`
- `[destination]` is OPTIONAL:
  - File path (e.g., `results.csv`, `/tmp/output.json`)
  - Literal keyword `clipboard` (copies to system clipboard)
  - If omitted: outputs to stdout (REPL displays in terminal)

### Syntax Examples

**Export to file:**
```
/export csv results.csv
/export json /tmp/output.json
/export sql backup.sql
/export table output.txt
```

**Export to clipboard:**
```
/export csv clipboard
/export json clipboard
/export table clipboard
```

**Export to stdout (display in terminal):**
```
/export csv
/export json
```

---

## Behavior Specification

### File Destination

**Format:** `/export <format> <filepath>`

**Behavior:**
1. Re-execute last query without row limit (gets full dataset)
2. Write results to specified file path
3. Create file if doesn't exist
4. **Default: Overwrite** existing file (align with standard tools)
5. Display confirmation message with row count

**Error handling:**
- File path not writable → Error with permission message
- File path is directory → Error "Cannot write to directory"
- Disk full → Error with disk space message

**Example output:**
```
tq> /export csv results.csv
✓ Exported 1,234 rows to results.csv
```

### Clipboard Destination

**Format:** `/export <format> clipboard`

**Behavior:**
1. Use last query result from memory (already fetched, limited to display rows)
2. Format according to `<format>` parameter
3. Copy to system clipboard
4. Display confirmation message

**Note:** Clipboard export uses **cached results** (limited rows), not full dataset re-execution. This is intentional - clipboard is for quick copy-paste workflows, not large dataset exports.

**Error handling:**
- Clipboard unavailable (headless environment) → Error "Clipboard not available"
- Clipboard access denied → Error with permission message

**Example output:**
```
tq> /export csv clipboard
✓ Copied 100 rows to clipboard (CSV format)
```

### Stdout Destination

**Format:** `/export <format>` (no destination specified)

**Behavior:**
1. Use last query result from memory
2. Format according to `<format>` parameter
3. Print to stdout (displays in REPL)
4. No confirmation message (output IS the message)

**Use case:** Preview export format before saving to file

**Example:**
```
tq> /export json
[
  {"id": 1, "name": "Alice"},
  {"id": 2, "name": "Bob"}
]
```

---

## Format Specifications

### Supported Formats

| Format | Description | File Extension | Use Case |
|--------|-------------|----------------|----------|
| `table` | ASCII table (current display format) | `.txt` | Human-readable reports |
| `csv` | Comma-separated values | `.csv` | Spreadsheet import, data analysis |
| `json` | JSON array of objects | `.json` | API integration, modern tools |
| `sql` | SQL INSERT statements | `.sql` | Database migration, backup |

### Format Examples

**CSV:**
```csv
id,name,department
1,Alice,Engineering
2,Bob,Marketing
```

**JSON:**
```json
[
  {"id": 1, "name": "Alice", "department": "Engineering"},
  {"id": 2, "name": "Bob", "department": "Marketing"}
]
```

**SQL:**
```sql
INSERT INTO tablename (id, name, department) VALUES (1, 'Alice', 'Engineering');
INSERT INTO tablename (id, name, department) VALUES (2, 'Bob', 'Marketing');
```

**Table:**
```
+----+-------+--------------+
| id | name  | department   |
+----+-------+--------------+
|  1 | Alice | Engineering  |
|  2 | Bob   | Marketing    |
+----+-------+--------------+
```

---

## Backwards Compatibility

### Migration Strategy

**Deprecated syntax (still supported with warnings):**
```
/export clipboard csv          → Warning: Use '/export csv clipboard' instead
/export csv --append file.csv  → Warning: Append mode removed
```

**Deprecation timeline:**
- **v1.7.0 (Sprint 13):** New syntax introduced, old syntax deprecated with warnings
- **v1.8.0 (Sprint 14+):** Old syntax removed (breaking change, major version bump if needed)

**Warning message format:**
```
⚠ Deprecated syntax: Use '/export <format> [destination]' instead
  Example: /export csv clipboard
```

---

## Help Text

### `/export` Metacommand Help

**New help output:**
```
Usage: /export <format> [destination]

Export last query results to file or clipboard

Formats:
  table   ASCII table (human-readable)
  csv     Comma-separated values
  json    JSON array of objects
  sql     SQL INSERT statements

Destinations:
  <file>      File path (exports full dataset)
  clipboard   System clipboard (exports displayed rows)
  (none)      Print to stdout

Examples:
  /export csv results.csv          Export full dataset to CSV file
  /export json clipboard           Copy displayed rows to clipboard
  /export table                    Print table format to terminal
  /export sql backup.sql           Generate SQL INSERT statements

Notes:
  - File exports re-execute query to get full dataset
  - Clipboard exports use cached results (limited rows)
  - Use /help export for more details
```

### `/help` Command Integration

**Updated entry:**
```
/export <format> [dest]   Export results (csv, json, sql, table)
                          Destination: file path or 'clipboard'
```

---

## Error Messages

### Common Errors

**No results to export:**
```
Error: No query results to export
  Run a SELECT query first, then use /export
```

**Invalid format:**
```
Error: Unknown format 'xml'
  Valid formats: table, csv, json, sql
  Example: /export csv results.csv
```

**File not writable:**
```
Error: Cannot write to 'results.csv'
  Permission denied
  Check file permissions and try again
```

**Clipboard unavailable:**
```
Error: Clipboard not available
  This may occur in headless/SSH environments
  Export to file instead: /export csv results.csv
```

**Ambiguous destination:**
```
Error: Ambiguous destination 'clipboard.csv'
  Did you mean:
    /export csv clipboard      (copy to clipboard)
    /export csv clipboard.csv  (save to file named 'clipboard.csv')
```

---

## Implementation Guidelines

### Command Parsing

**Parse order:**
1. Extract format (1st parameter) - REQUIRED
2. Extract destination (2nd parameter) - OPTIONAL
3. Validate format against supported list
4. Determine destination type:
   - If parameter == "clipboard" → Clipboard destination
   - If parameter exists and != "clipboard" → File destination
   - If parameter missing → Stdout destination

**Pseudocode:**
```rust
fn parse_export_command(args: &[String]) -> Result<ExportCommand> {
    let format = args.get(0)
        .ok_or("Missing format")?
        .parse::<ExportFormat>()?;

    let destination = match args.get(1) {
        Some(dest) if dest == "clipboard" => Destination::Clipboard,
        Some(path) => Destination::File(PathBuf::from(path)),
        None => Destination::Stdout,
    };

    Ok(ExportCommand { format, destination })
}
```

### Full Dataset Re-execution Logic

**File destination only:**
```rust
match destination {
    Destination::File(path) => {
        // Re-execute query without limit
        let full_results = re_execute_query_unlimited(last_query)?;
        write_to_file(full_results, format, path)?;
    },
    Destination::Clipboard | Destination::Stdout => {
        // Use cached results (already limited)
        let cached_results = get_last_results()?;
        export_results(cached_results, format, destination)?;
    }
}
```

**Query re-execution:**
- Parse original query to detect user-specified limits (`TOP`, `SAMPLE`)
- If user limit detected → re-execute WITH limit (respect user intent)
- If no user limit → re-execute WITHOUT limit (get full dataset)
- Handle errors gracefully (connection lost, table dropped, etc.)

---

## Testing Strategy

### Unit Tests

**Format parsing:**
```rust
#[test]
fn test_export_parse_file_destination() {
    let cmd = parse_export_command(&["csv", "output.csv"]);
    assert_eq!(cmd.format, ExportFormat::Csv);
    assert_eq!(cmd.destination, Destination::File("output.csv"));
}

#[test]
fn test_export_parse_clipboard_destination() {
    let cmd = parse_export_command(&["json", "clipboard"]);
    assert_eq!(cmd.format, ExportFormat::Json);
    assert_eq!(cmd.destination, Destination::Clipboard);
}

#[test]
fn test_export_parse_stdout_destination() {
    let cmd = parse_export_command(&["table"]);
    assert_eq!(cmd.format, ExportFormat::Table);
    assert_eq!(cmd.destination, Destination::Stdout);
}
```

### Integration Tests

**File export:**
- Create temp file, verify contents
- Verify full dataset exported (not limited)
- Verify file overwritten on second export

**Clipboard export:**
- Mock clipboard, verify contents
- Verify limited results used (not full dataset)

**Error cases:**
- No results available
- Invalid format
- Permission denied
- Clipboard unavailable

### Manual Test Cases

**Test Case TC029: Export Syntax Validation**
```
1. Run query: SELECT * FROM table;
2. Test: /export csv results.csv
   Expected: File created with full dataset
3. Test: /export json clipboard
   Expected: Clipboard contains JSON (limited rows)
4. Test: /export table
   Expected: Table printed to terminal
5. Test: /export invalid format
   Expected: Error with valid formats listed
```

---

## User Experience Improvements

### Before (Confusing)

**User mental model:**
- "How do I export to clipboard in JSON?"
- Checks help: `/export clipboard [format]`
- Tries: `/export clipboard json` ✓ Works
- Next day: "How do I export CSV to file?"
- Checks help: `/export <format> [file]`
- Tries: `/export file results.csv` ✗ Error
- Realizes: Format comes first for files but last for clipboard
- **Cognitive friction!**

### After (Clear)

**User mental model:**
- "How do I export?"
- Thinks: Format first, then destination
- `/export json clipboard` ✓ Works
- `/export csv results.csv` ✓ Works
- `/export table` ✓ Works
- **Consistent, predictable!**

---

## Migration Examples

### Old → New Syntax

| Old Syntax | New Syntax | Status |
|------------|------------|--------|
| `/export csv results.csv` | `/export csv results.csv` | No change |
| `/export clipboard csv` | `/export csv clipboard` | Deprecated |
| `/export json clipboard` | `/export json clipboard` | No change |
| `/export csv --append file.csv` | N/A | Removed (use shell: `>> file.csv`) |

---

## Design Rationale

### Why This Design?

**1. Consistent Parameter Order**
- Format always first (describes WHAT)
- Destination always second (describes WHERE)
- Aligns with UNIX tool patterns (e.g., `cp <what> <where>`)

**2. Literal "clipboard" Keyword**
- Unambiguous: `clipboard` is reserved, cannot be confused with filename
- Discoverable: Shows up in help text and autocomplete
- Future-proof: Could add other special destinations (`stdout`, `stderr`)

**3. Optional Destination**
- Stdout as default enables previewing export formats
- Reduces typing for quick format checks
- Aligns with REPL exploration workflow

**4. File = Full Dataset, Clipboard = Cached**
- File exports: User expects complete data (re-execute query)
- Clipboard: User expects quick copy-paste of visible results (use cache)
- Clear mental model based on destination semantics

---

## Open Questions (Resolved)

**Q: Should clipboard also re-execute query for full dataset?**
A: No. Clipboard is for quick copy-paste workflows. Users wanting full dataset should export to file, then read file to clipboard using OS tools.

**Q: Should we support append mode?**
A: No. Use shell redirection for append: `tq query ... >> output.csv`. Keeps tool simple, leverages existing UNIX patterns.

**Q: Should destination default to clipboard or stdout?**
A: Stdout. More useful for REPL exploration, doesn't require clipboard support.

**Q: Should we validate file extensions match format?**
A: No. Allow users to choose any extension. Some users may want `.txt` for CSV, etc.

---

## Success Metrics

### Definition of Success

- [ ] Users can explain export syntax without checking help
- [ ] Zero user questions about export parameter order
- [ ] Help text is self-explanatory
- [ ] All test cases pass
- [ ] Backwards compatibility maintained (with deprecation warnings)
- [ ] Implementation matches specification

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-19 | 1.0.0 | Initial export syntax simplification design | CLI UX Designer |

---

## References

- Sprint 13 Planning: Feature 5 - Export Syntax Simplification
- User Feedback: docs/builder/incoming/open-bugs.md (lines 38-58)
- CLI Design Best Practices: docs/builder/rust-cli-design-general.md
- Current Implementation: src/commands/repl/export.rs

---

## Next Steps

1. **User Review** - Validate design meets user needs
2. **Implementation** - rust-teradata-architect implements new syntax
3. **Testing** - quality-validator executes test cases
4. **Documentation** - Update help text and user guide
5. **Deployment** - Ship in Sprint 13 (v1.7.0)
