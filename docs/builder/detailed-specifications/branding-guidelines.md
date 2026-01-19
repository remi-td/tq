# tq Branding Guidelines

**Version:** 1.0.0
**Status:** Active
**Last Updated:** 2026-01-19
**Author:** CLI UX Designer

---

## Purpose

This document defines the complete visual identity and branding standards for `tq` (Teradata Query Tool). All implementations MUST follow these guidelines exactly to maintain consistent, professional branding across all user touchpoints.

---

## Tool Name

### Official Name
**`tq`** (all lowercase, always)

### Usage Rules
- CLI command: `tq`
- Documentation: `tq`
- Help text: `tq`
- Marketing: "tq - Teradata Query Tool"
- User references: `tq`

**NEVER use:**
- ❌ `TQ` (all caps)
- ❌ `Tq` (sentence case)
- ❌ `tQ` (mixed case)

### Visual Identity in Text
When displaying the tool name with color support:
- **First letter 't'** in **Teradata orange** (#F37021)
- **Remaining letters 'q'** in default terminal foreground color

**Example rendering:**
```
[ORANGE]t[DEFAULT]q
```

---

## Color Palette

### Primary Brand Color: Teradata Orange

**Official Teradata Orange:**
- **Hex:** `#F37021`
- **RGB:** `243, 112, 33`
- **Terminal Color Code:** Custom RGB (requires truecolor support)

**Usage:**
- Logo 't' letter
- Interactive prompt (`tq>`)
- Key UI elements
- Section headers
- Emphasis text

**Fallback for 8-color terminals:**
- Use standard bright yellow (`\x1b[33;1m`) when truecolor not supported

### Secondary Colors

**White/Default:**
- Use for body text, normal output
- Terminal default foreground color

**Gray/Dim:**
- Use for timestamps, metadata, secondary information
- Terminal dim/gray color

**Red:**
- Use for errors only
- Terminal red color

**Green:**
- Use for success messages only
- Terminal green color

---

## Logo Design

### Design Specification

**Character Set:** Unicode block character `█` (U+2588 Full Block)

**Dimensions:**
- Width: 15 characters maximum
- Height: 5 lines
- Monospace-friendly: Must render correctly in all monospace fonts

**Design Principles:**
1. **Minimalist** - Simple, clean design
2. **Monospace-friendly** - Perfect alignment in terminal
3. **Professional** - Suitable for enterprise environments
4. **Recognizable** - Instantly identifiable as tq
5. **Terminal-safe** - Renders correctly in all terminals

### Logo ASCII Art

```
████████╗ ██████╗
╚══██╔══╝██╔═══██╗
   ██║   ██║   ██║
   ██║   ██║▄▄ ██║
   ╚═╝   ╚██████╔╝
          ╚═════╝
```

**Alternative Simple Block Design:**
```
 ████████   ████
    ██     ██  ██
    ██     ██  ██
    ██     ██ ▄██
    ██      ████
```

**Color Application:**
- **'t' letter portion** - Teradata orange (#F37021)
- **'q' letter portion** - Default terminal color

**Rendering Requirements:**
- All logo lines must be perfectly aligned (no offset)
- Equal spacing maintained between characters
- Consistent use of block character `█` throughout
- No mixing of `|`, `_`, or other ASCII characters

**Logo Display Context:**
```
[Logo in orange/default colors]
Teradata Query Tool
v[VERSION]
```

**Implementation Notes:**
- Use consistent indentation for all logo lines
- Test rendering in multiple terminal emulators
- Verify alignment before deployment
- Last two lines MUST NOT be offset

---

## Interactive Prompt

### REPL Prompt Design

**Default Prompt:**
```
tq>
```

**Color:**
- **Entire prompt `tq>`** in **Teradata orange** (#F37021)

**Do NOT use:**
- ❌ Green prompt (old default)
- ❌ Default terminal color
- ❌ Any other color

**Multi-line Continuation Prompt:**
```
...>
```

**Color:**
- Same Teradata orange as primary prompt

---

## Terminal Rendering Guidelines

### Color Support Detection

**Priority Order:**
1. **Truecolor (24-bit)** - Use exact Teradata orange RGB(243, 112, 33)
2. **256-color** - Use closest orange approximation
3. **8-color** - Use bright yellow as fallback
4. **No color** - Gracefully disable all colors

**Detection Method:**
```rust
// Check COLORTERM environment variable
if env::var("COLORTERM").ok().as_deref() == Some("truecolor") {
    // Use RGB color
} else {
    // Fall back to ANSI
}
```

### Terminal Compatibility

**Must render correctly in:**
- iTerm2 (macOS)
- Terminal.app (macOS)
- GNOME Terminal (Linux)
- Windows Terminal (Windows)
- Alacritty (cross-platform)
- Kitty (cross-platform)
- Tmux sessions
- SSH sessions

**Test in:**
- Standard 80-column terminals
- Wide terminals (120+ columns)
- Narrow terminals (<80 columns)
- Light and dark terminal themes

---

## Typography

### Font Requirements

**All text MUST be monospace-friendly:**
- Designed for monospace fonts (Courier, Monaco, Consolas, etc.)
- No assumptions about proportional fonts
- Consistent character widths

### Text Styles

**Headers:**
```
Header Text (Bold if supported, otherwise default)
```

**Body Text:**
```
Regular terminal default foreground
```

**Emphasis:**
```
[ORANGE]Emphasized text[DEFAULT]
```

**Metadata/Secondary:**
```
[GRAY]Secondary information[DEFAULT]
```

---

## Welcome Banner

### REPL Startup Banner

**Full banner format:**
```
[Logo in color]
Teradata Query Tool
v[VERSION]

Connected to [HOST]
Database: [DATABASE]
Session: [SESSION_ID]
Type /help for available commands
```

**Color application:**
- Logo: Per logo specification above
- "Teradata Query Tool": Default color
- Version: Gray/dim
- Connection info: Default color
- Database/Session values: Teradata orange (for emphasis)
- Help hint: Gray/dim

**Spacing:**
- One blank line after logo
- One blank line after version
- One blank line after session info
- Prompt starts immediately after

---

## Error Messages and Status

### Error Formatting

**Error prefix:**
```
[RED]Error:[DEFAULT] [error message]
```

**Do NOT use:**
- ❌ Stack traces (unless `--debug` flag)
- ❌ Go-style traces
- ❌ Internal implementation details

### Success Messages

**Success prefix:**
```
[GREEN]✓[DEFAULT] [success message]
```

**Example:**
```
✓ Connected to tdprod.example.com
```

### Info Messages

**Info format:**
```
[ORANGE]ℹ[DEFAULT] [info message]
```

**Example:**
```
ℹ Exporting 1,234 rows to results.csv
```

---

## Help Text Branding

### Help Command Header

```
tq [VERSION]
Teradata Query Tool

USAGE:
    tq [OPTIONS] <COMMAND>
```

**Color:**
- `tq`: Tool name color (t in orange, q in default)
- Version: Gray/dim
- "Teradata Query Tool": Default
- Usage section: Default

### Metacommand Help (`/help`)

**Header:**
```
tq - Teradata Query Tool
Available Commands:
```

**Command list:**
```
  /help                  Show this help message
  /session               Display session information
  /quit                  Exit the REPL
  ...
```

**Color:**
- Command names (`/help`): Teradata orange
- Descriptions: Default color

---

## Accessibility

### Color Blindness Considerations

**Guidelines:**
- Never use color as the ONLY indicator of meaning
- Use text labels in addition to colors
- Errors have "Error:" prefix (not just red color)
- Success has "✓" symbol (not just green color)

### Screen Reader Support

**Text alternatives:**
- Logo includes "Teradata Query Tool" text
- All visual elements have text equivalents
- Status messages are plain text

---

## File Export Headers

### CSV Export Header Comments

```
# Generated by tq v[VERSION]
# Teradata Query Tool
# Generated at: [TIMESTAMP]
# Query: [QUERY]
[data...]
```

### JSON Export Metadata

```json
{
  "tq_version": "[VERSION]",
  "generated_at": "[TIMESTAMP]",
  "query": "[QUERY]",
  "rows": [...]
}
```

---

## Version Display

### Version Command Output

**Format:**
```
tq [VERSION]
Teradata Query Tool
```

**Example:**
```
tq 1.6.1
Teradata Query Tool
```

**Color:**
- `tq`: Tool name color (t in orange)
- Version number: Default
- Subtitle: Gray/dim

---

## Consistency Checklist

Before any release, verify:

- [ ] Logo renders correctly (no offset in last two lines)
- [ ] Logo uses only `█` block character (not `|` or `_`)
- [ ] Tool name always lowercase `tq`
- [ ] 't' letter in Teradata orange (#F37021) everywhere
- [ ] Interactive prompt `tq>` in Teradata orange (not green)
- [ ] Welcome banner displays correctly
- [ ] Error messages use red color + "Error:" prefix
- [ ] Success messages use green color + "✓" symbol
- [ ] All colors have appropriate fallbacks
- [ ] Text readable in light AND dark terminal themes
- [ ] Monospace alignment perfect in all contexts

---

## Implementation Notes

### Color Application Code Pattern

**Rust example:**
```rust
use colored::*;

// Tool name
println!("{}{}",
    "t".truecolor(243, 112, 33),  // Teradata orange
    "q"                             // Default color
);

// Prompt
print!("{}",
    "tq> ".truecolor(243, 112, 33)
);

// Logo line (example)
println!("{}",
    "████████╗ ██████╗ ".truecolor(243, 112, 33)
);
```

### Terminal Color Compatibility

**Always check terminal support:**
```rust
// Check if colors should be enabled
let use_color = atty::is(Stream::Stdout)
    && env::var("NO_COLOR").is_err();
```

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-19 | 1.0.0 | Initial branding guidelines created for Sprint 13 | CLI UX Designer |

---

## References

- Teradata Corporate Brand Guidelines (orange color reference)
- Sprint 13 Planning: Feature 3 - Logo Branding Issues
- User Feedback: docs/builder/incoming/open-bugs.md (lines 6-22)
- Tab Completion Failure Analysis: Branding section

---

## Approval

**Status:** DRAFT - Pending User Review

**Next Steps:**
1. User reviews and approves logo design
2. User validates color specifications
3. Implementation follows approved guidelines
4. Testing validates rendering across terminals

Once approved, this becomes the authoritative branding reference for all tq development.
