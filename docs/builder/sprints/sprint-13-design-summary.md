# Sprint 13 Design Phase - Summary for Implementation

**Date:** 2026-01-19
**Phase:** Design Complete - Ready for Implementation
**Author:** CLI UX Designer

---

## Overview

This document provides a quick reference for the rust-teradata-architect to implement Sprint 13 features. All detailed specifications are complete and ready for implementation.

---

## Deliverables from Design Phase

### 1. Branding Guidelines (P0 - Critical)

**Document:** `/Users/remi.turpaud/Code/genAI/tq/docs/builder/detailed-specifications/branding-guidelines.md`
**Version:** 2.0.0
**Status:** Ready for Implementation

**Key Specifications:**

#### Logo Design
```
 ████████   ████
    ██     ██  ██
    ██     ██  ██
    ██     ██ ▄██
    ██      ████
```

- Uses Unicode block character `█` (U+2588) exclusively
- 5 lines, perfectly aligned (NO offset in lines 4-5)
- 't' portion (left blocks) in Teradata orange RGB(243, 112, 33)
- 'q' portion (right blocks) in default terminal color

**Implementation code provided in spec:**
```rust
let orange = Color::Rgb(243, 112, 33);

writeln!(writer, " {}   {}",
    orange.paint("████████"),
    "████")?;
writeln!(writer, "    {}     {}",
    orange.paint("██"),
    "██  ██")?;
writeln!(writer, "    {}     {}",
    orange.paint("██"),
    "██  ██")?;
writeln!(writer, "    {}     {}",
    orange.paint("██"),
    "██ ▄██")?;
writeln!(writer, "    {}      {}",
    orange.paint("██"),
    "████")?;
```

#### Prompt Colors

**CRITICAL FIX:** Prompt must be ENTIRELY in Teradata orange (not green).

```rust
// Teradata orange: RGB(243, 112, 33)
let orange_start = "\x1b[38;2;243;112;33m";
let reset = "\x1b[0m";

let normal_prompt = format!("{}tq> {}", orange_start, reset);
let continuation_prompt = format!("{}...> {}", orange_start, reset);
```

#### Tool Name Display

Everywhere `tq` appears:
- 't' in Teradata orange RGB(243, 112, 33)
- 'q' in default terminal color
- Always lowercase

#### Testing Requirements

After implementation, architect MUST:
1. Visually verify logo lines 4-5 are NOT offset
2. Verify 't' stem is perfectly vertical through lines 2-5
3. Verify `tq>` prompt is in Teradata orange (NOT green)
4. Test in at least iTerm2 and one other terminal
5. Run `cargo build` and verify ZERO warnings

---

### 2. Export Syntax Simplification (P1 - High Priority)

**Document:** `/Users/remi.turpaud/Code/genAI/tq/docs/builder/sprints/export-syntax-simplification-design.md`
**Version:** 1.0.0
**Status:** Ready for Implementation

**New Unified Syntax:**
```
/export <format> [destination]
```

**Where:**
- `<format>` is REQUIRED: `table`, `csv`, `json`, `sql`
- `[destination]` is OPTIONAL:
  - File path → Save to file (re-executes query for full dataset)
  - Literal `clipboard` → Copy to system clipboard (uses cached results)
  - Omitted → Display to stdout (terminal)

**Examples:**
```bash
/export csv results.csv      # Save full dataset to file
/export json clipboard       # Copy cached results to clipboard
/export table                # Display table format to terminal
```

**Backward Compatibility:**
- Support deprecated `/export clipboard <format>` with warning
- Warning message: "⚠ Deprecated syntax: Use '/export <format> [destination]' instead"

**Implementation Guidance:**
- Complete parsing pseudocode provided in spec
- Error messages specified
- Test cases listed

---

## Implementation Checklist

### Branding (P0)

- [ ] Update logo in `src/commands/repl/mod.rs` display_logo() function
  - [ ] Use exact spacing from specification
  - [ ] Use `█` block character only (no `|` or `_`)
  - [ ] Apply Teradata orange to 't' portion
  - [ ] Fix all `writeln!()` warnings by using `?` operator

- [ ] Update prompt colors in `src/commands/repl/prompt.rs`
  - [ ] Change ENTIRE `tq>` prompt to Teradata orange
  - [ ] Change ENTIRE `...>` prompt to Teradata orange
  - [ ] Remove any green color references

- [ ] Update tool name display throughout codebase
  - [ ] 't' in orange, 'q' in default wherever `tq` appears
  - [ ] Help text, version output, etc.

- [ ] Visual validation tests (manual)
  - [ ] Logo alignment verified (no offset)
  - [ ] Prompt color verified (orange, not green)
  - [ ] Test in multiple terminals

### Export Syntax (P1)

- [ ] Update export command parsing in `src/commands/repl/metacommands.rs` (or wherever export is implemented)
  - [ ] Parse format (required, first argument)
  - [ ] Parse destination (optional, second argument)
  - [ ] Detect `clipboard` as special destination
  - [ ] Treat other strings as file paths
  - [ ] Default to stdout if omitted

- [ ] Add backward compatibility support
  - [ ] Detect `/export clipboard <format>` pattern
  - [ ] Show deprecation warning
  - [ ] Still execute correctly

- [ ] Update help text
  - [ ] `/export` command help (inline)
  - [ ] `/help` output entry
  - [ ] Update examples

- [ ] Unit tests
  - [ ] Test parsing all destination types
  - [ ] Test deprecated syntax detection
  - [ ] Test error cases

### Build Quality (P1)

- [ ] Fix build warnings from Sprint 12
  - [ ] Use `let _ = writeln!(...);` or `writeln!(...)?;` pattern
  - [ ] Zero warnings after build

---

## User Requirements Addressed

From `docs/builder/incoming/open-bugs.md`:

### Logo and Branding (Lines 6-22)
✅ Tool name `tq` in lowercase - SPECIFIED
✅ Letter 't' in Teradata orange (#F37021) - SPECIFIED with RGB values
✅ Use block character █ - SPECIFIED with exact Unicode
✅ Logo renders correctly (last two lines not offset) - SPECIFIED with exact spacing
✅ Interactive prompt `tq>` in Teradata orange (not green) - SPECIFIED with implementation code

### Export Command (Lines 24-44)
✅ Simplify to `/export <format> [file|clipboard]` - SPECIFIED as `/export <format> [destination]`
✅ Clear semantics - Format first, destination second
✅ Backward compatibility - Deprecated syntax still works with warning

---

## Files Modified

### Primary Implementation Files
- `src/commands/repl/mod.rs` - Logo display function
- `src/commands/repl/prompt.rs` - Prompt color configuration
- `src/commands/repl/metacommands.rs` - Export command parsing (likely location)
- Help text strings throughout REPL code

### Documentation Files (Already Updated)
- `docs/builder/detailed-specifications/branding-guidelines.md` (v2.0.0)
- `docs/builder/sprints/export-syntax-simplification-design.md` (v1.0.0)
- `docs/builder/specifications.md` (updated with 🚧 status)

---

## Critical Success Factors

1. **EXACT Implementation Required**
   - Logo must match specification EXACTLY (spacing, characters, colors)
   - No interpretation or "improvements"
   - Follow the code samples provided

2. **Visual Validation Mandatory**
   - Architect must visually verify logo and prompt colors
   - Take screenshots if needed for validation
   - Test in real terminals, not just unit tests

3. **Zero Build Warnings**
   - Fix all `writeln!()` Result handling warnings
   - Clean build required before completion

4. **User Validation Required**
   - After implementation, user must validate visual appearance
   - Cannot mark complete without user sign-off

---

## Testing Strategy

### Unit Tests
- Export command parsing (all cases)
- Format validation
- Deprecated syntax detection

### Visual Tests (Manual)
- Logo alignment check
- Prompt color check (NOT green)
- Terminal compatibility (iTerm2 minimum)

### Integration Tests
- Export to file works
- Export to clipboard works
- Export to stdout works
- Deprecated syntax works with warning

---

## Open Questions / Blockers

**NONE** - Design is complete and unambiguous.

If the architect encounters any ambiguity, refer to:
1. Branding Guidelines v2.0.0 (comprehensive)
2. Export Syntax Design v1.0.0 (comprehensive)
3. This summary document

All specifications include implementation code samples and exact requirements.

---

## Next Phase

**Phase 3: Implementation**

Architect should:
1. Read branding-guidelines.md in full
2. Read export-syntax-simplification-design.md in full
3. Implement logo per exact specification
4. Implement prompt colors per exact specification
5. Implement export syntax simplification
6. Fix build warnings
7. Run visual validation tests
8. Commit changes with proper documentation

**Estimated Time:** 2-3 hours for careful, correct implementation

---

## References

- [Branding Guidelines v2.0.0](../detailed-specifications/branding-guidelines.md)
- [Export Syntax Design v1.0.0](export-syntax-simplification-design.md)
- [Sprint 13 Planning](sprint-13-planning.md)
- [User Feedback: open-bugs.md](../incoming/open-bugs.md)

---

**Design Phase Complete - Ready for Implementation**

All specifications are comprehensive, unambiguous, and include implementation guidance. The architect can implement without interpretation or guesswork.
