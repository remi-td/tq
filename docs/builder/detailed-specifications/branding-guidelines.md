# tq Branding Guidelines

**Version:** 3.0.0
**Status:** Active - Sprint 19 Implementation
**Last Updated:** 2026-01-22
**Owner:** CLI UX Designer

---

## Purpose

This document defines the complete visual identity and branding standards for `tq` (Teradata Query Tool). All implementations MUST follow these guidelines exactly to maintain consistent, professional branding across all user touchpoints.

**CRITICAL:** This is the authoritative specification. The architect must implement EXACTLY as specified with zero interpretation or deviation.

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
- **ANSI 24-bit truecolor escape:** `\x1b[38;2;243;112;33m`
- **Reset escape:** `\x1b[0m`

**Usage:**
- Logo 't' letter
- Tool name 't' letter
- Interactive prompt (`tq>` - ENTIRE prompt)
- Multi-line continuation prompt (`...>` - ENTIRE prompt)
- Key UI elements
- Section headers
- Emphasis text

**Implementation in Rust (using `ansi_term` or `colored` crate):**
```rust
use ansi_term::Color;

// Teradata orange
let orange = Color::Rgb(243, 112, 33);

// Apply to text
orange.paint("text")
```

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

### CURRENT APPROVED DESIGN (Sprint 19+)

**Character Set:** ASCII characters `_`, `|`, `{`, `}`, `\`, `` ` ``

**Design Philosophy:** Lowercase ASCII art rendering of "tq" letters using standard ASCII characters. Information messages are displayed to the RIGHT of the logo on the same lines.

**Dimensions:**
- Width: Logo portion approximately 12 characters, total line width varies with info messages
- Height: Exactly 5 lines
- Monospace-friendly: Must render correctly in all monospace fonts

### Logo ASCII Art - EXACT SPECIFICATION

```
 _
| |_   __ _
|  _| / _` |
 \__|  \__, |
          |_|
```

**Visual Structure:**
- Lines 1-5: Lowercase "t" (left) + lowercase "q" (right)
- The "t": Vertical stem with horizontal top bar
- The "q": Circular body with descending tail

### Implementation with Info Messages

**Sprint 19+ Layout:** Info messages appear on the SAME lines as the logo, to the right:

```
 _            Teradata Query Tool v1.7.0
| |_   __ _   Connected to host:1025
|  _| / _` |  Database: demo_user
 \__|  \__, |  User: demo_user
          |_|  Default row limit: 100
```

**Critical Implementation Details:**

The logo is implemented as two separate character arrays that are combined line-by-line:

**'t' portion (colored in Teradata orange):**
```
 _
| |_
|  _|
 \__|
```

**'q' portion (default terminal color):**
```

 __ _
/ _` |
\__, |
   |_|
```

**Combined output:** Each line combines `[orange_t][default_q]   [info_message]`

### Visual Structure

```
[Line 1]   _            <- 't' top bar
[Line 2]  | |_   __ _   <- 't' stem + horizontal + 'q' top
[Line 3]  |  _| / _` |  <- 't' stem + base + 'q' body
[Line 4]   \__|  \__, | <- 't' bottom + 'q' body with tail start
[Line 5]          |_|   <- 'q' descending tail
```

The 't' is a lowercase letter with vertical stem and horizontal connector. The 'q' is a lowercase letter with circular body and descending tail.

### Color Application

**Color Splitting:**
- **'t' portion:** Teradata orange (#F37021 / xterm-256 color 202)
  - All characters in the 't' shape: `_`, `|`, `{`, `\`

- **'q' portion:** Default terminal color
  - All characters in the 'q' shape: `_`, `` ` ``, `/`, `|`, `{`, etc.

**Implementation Pattern (Sprint 19+):**
```rust
use ansi_term::Color;

let orange = Color::Fixed(202);  // Teradata orange

let logo_t = [
    " _    ",
    "| |_  ",
    "|  _| ",
    " \\__| ",
    "      ",
];

let logo_q = [
    "      ",
    " __ _ ",
    "/ _` |",
    "\\__, |",
    "   |_|",
];

// Build info lines
let info_lines = vec![
    format!("Teradata Query Tool v{}", env!("CARGO_PKG_VERSION")),
    format!("Connected to {}:{}", config.host, config.port),
    format!("Database: {}", config.database),
    format!("User: {}", config.user),
    format!("Default row limit: {}", args.default_limit),
];

// Print each line: [orange_t][default_q]   [info]
for (i, (t_part, q_part)) in logo_t.iter().zip(logo_q.iter()).enumerate() {
    let t_colored = orange.bold().paint(*t_part);
    let info = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");
    writeln!(writer, "{}{}   {}", t_colored, q_part, info)?;
}
```

**CRITICAL LAYOUT RULES:**
1. Each line combines: `[colored_t_part][q_part]   [info_message]`
2. Three spaces separate logo from info messages
3. Info messages vertically aligned (all start at same column)
4. Logo portions are fixed width (t=6 chars, q=6 chars, total 12 chars for logo)
5. If fewer than 5 info lines, remaining logo lines print without info

**Testing Requirements:**
- Render the logo and visually verify proper ASCII art shape
- All info lines must be vertically aligned on the right
- The 't' and 'q' shapes must be recognizable as lowercase letters
- Test in multiple terminals (iTerm2, Terminal.app, Alacritty, etc.)
- Verify 't' portion displays in Teradata orange
- Verify 'q' portion displays in default terminal color

### Logo Display Context

```
[Blank line]
 _            Teradata Query Tool v1.7.0
| |_   __ _   Connected to mcp-host:1025
|  _| / _` |  Database: demo_user
 \__|  \__, |  User: demo_user
          |_|  Default row limit: 100
[Blank line]
Type /help for commands, /quit to exit.
[Blank line]
```

### Design Evolution History

**Sprint 1-12:** No logo, minimal branding

**Sprint 13-17:** Uppercase block art logo using █ (U+2588) characters
- 5 lines, 17 characters wide
- Uppercase "T" and "Q" shapes
- Color split: 't' in orange, 'q' in default
- Status: **DEPRECATED**

**Sprint 18:** Plain text lowercase "tq" (INCORRECT)
- Simple text: `tq\nTeradata Query tool v1.7.0`
- Info displayed BELOW logo
- Status: **REVERTED in Sprint 19** (did not meet user requirements)

**Sprint 19+:** Lowercase ASCII art "tq" with info on right (CURRENT)
- ASCII characters `_|{}\` ` forming lowercase letter shapes
- Info messages on same lines to the RIGHT of logo
- Color split: 't' in orange, 'q' in default
- Status: **ACTIVE** (current approved design)

---

## Interactive Prompt

### REPL Prompt Design

**Default Prompt:**
```
tq>
```

**CRITICAL REQUIREMENT:** The ENTIRE prompt `tq>` must be in Teradata orange (#F37021), NOT green or any other color.

**User's explicit requirement:** "I would also use the same teradata orange as default for the interactive prompt color: `tq>` (currently olive green)."

**Implementation:**
```rust
// Teradata orange ANSI escape: \x1b[38;2;243;112;33m
// Reset: \x1b[0m
let orange_start = "\x1b[38;2;243;112;33m";
let reset = "\x1b[0m";

let normal_prompt = format!("{}tq> {}", orange_start, reset);
```

**Do NOT use:**
- ❌ Green prompt (old default) - THIS WAS THE BUG
- ❌ Default terminal color
- ❌ Any other color
- ❌ Splitting the prompt into parts with different colors

**Multi-line Continuation Prompt:**
```
...>
```

**Color:**
- Same Teradata orange (#F37021) as primary prompt
- ENTIRE prompt `...>` in orange

**Implementation:**
```rust
let continuation_prompt = format!("{}...> {}", orange_start, reset);
```

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
use ansi_term::Color;
use std::env;

// Check COLORTERM environment variable
let use_truecolor = env::var("COLORTERM")
    .ok()
    .map(|v| v == "truecolor" || v == "24bit")
    .unwrap_or(false);

// Always use RGB color (ansi_term handles fallbacks automatically)
let orange = Color::Rgb(243, 112, 33);
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
[Blank line]
[Logo in Teradata orange/default colors - 5 lines]
[Blank line]
 tq  v[VERSION]
[Blank line]
Connected to [HOST]:[PORT]
Database: [DATABASE]
User: [USER]
Logon Mechanism: [LOGMECH]
[Additional session settings...]
[Blank line]
Type /help for commands, /quit to exit.
[Blank line]
```

**Color application:**
- Logo: Per logo specification (orange/default split)
- Tool name "tq": 't' in orange, 'q' in default
- Version: Gray/dim (optional, can be default)
- Connection info labels: Default color
- Connection info values: Default color (NOT orange - keep it simple)
- Help hint: Default or gray/dim

**Spacing:**
- One blank line before logo
- One blank line after logo
- One blank line after version line
- One blank line after session info
- One blank line after help hint
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
- Version: Default
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
- Command names (`/help`): Default or orange (optional for emphasis)
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
- Logo includes "tq v[VERSION]" text immediately after
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
tq 1.7.0
Teradata Query Tool
```

**Color:**
- `tq`: Tool name color (t in orange, q in default)
- Version number: Default
- Subtitle: Default or gray/dim

---

## Consistency Checklist

Before any release, verify:

- [ ] Logo renders correctly (no offset in last two lines)
- [ ] Logo uses only `█` block character (U+2588) and `▄` (U+2584) for descender
- [ ] Logo has NO `|` or `_` characters
- [ ] Tool name always lowercase `tq`
- [ ] 't' letter in Teradata orange (#F37021) everywhere
- [ ] 'q' letter in default terminal color everywhere
- [ ] Interactive prompt `tq>` ENTIRELY in Teradata orange (not green, not default)
- [ ] Multi-line prompt `...>` ENTIRELY in Teradata orange
- [ ] Welcome banner displays correctly with proper spacing
- [ ] Error messages use red color + "Error:" prefix
- [ ] Success messages use green color + "✓" symbol
- [ ] All colors have appropriate fallbacks
- [ ] Text readable in light AND dark terminal themes
- [ ] Monospace alignment perfect in all contexts
- [ ] Test in at least 3 different terminal emulators

---

## Implementation Validation

### Manual Testing Required

After implementation, the architect MUST perform these visual checks:

1. **Logo Alignment Test:**
   - Start `tq repl`
   - Visually verify lines 4 and 5 of the logo are NOT offset to the right
   - Verify the 't' stem is perfectly vertical through lines 2-5
   - Take screenshot if needed for validation

2. **Prompt Color Test:**
   - Verify `tq>` prompt is in Teradata orange (NOT green)
   - Type a multi-line SQL statement and verify `...>` is also orange
   - Compare orange color to logo orange (should match exactly)

3. **Terminal Compatibility Test:**
   - Test in iTerm2 (macOS primary)
   - Test in at least one other terminal (Terminal.app, Alacritty, etc.)
   - Verify colors render correctly in both light and dark themes

4. **Build Warnings Test:**
   - Run `cargo build`
   - Verify ZERO warnings related to `writeln!()` or Result handling
   - All `writeln!()` calls must properly handle the Result with `?` operator

---

## Implementation Code Reference

### Complete Logo Implementation (Rust)

```rust
use ansi_term::Color;
use std::io::Write;

fn display_logo<W: Write>(writer: &mut W) -> Result<(), std::io::Error> {
    let orange = Color::Rgb(243, 112, 33);

    writeln!(writer)?;

    // Logo: 't' in orange, 'q' in default terminal color
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

    writeln!(writer)?;

    // Tool name: 't' in orange, 'q' in default
    writeln!(writer, " {}{}  v{}",
        orange.paint("t"),
        "q",
        env!("CARGO_PKG_VERSION"))?;

    writeln!(writer)?;

    Ok(())
}
```

### Prompt Implementation (Rust)

```rust
pub struct TqPrompt {
    normal_prompt: String,
    continuation_prompt: String,
}

impl TqPrompt {
    pub fn new() -> Self {
        // Teradata orange: RGB(243, 112, 33)
        let orange_start = "\x1b[38;2;243;112;33m";
        let reset = "\x1b[0m";

        Self {
            normal_prompt: format!("{}tq> {}", orange_start, reset),
            continuation_prompt: format!("{}...> {}", orange_start, reset),
        }
    }
}
```

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-22 | 3.0.0 | Updated logo specification to reflect Sprint 19 lowercase ASCII art design with info on right. Added design evolution history. Deprecated uppercase block art from v2.0.0. | CLI UX Designer |
| 2026-01-19 | 2.0.0 | Complete rewrite with unambiguous specifications, exact logo design, prompt color fix, implementation validation requirements | CLI UX Designer |
| 2026-01-19 | 1.0.0 | Initial branding guidelines created for Sprint 13 | CLI UX Designer |

---

## References

- Teradata Corporate Brand Guidelines (orange color: #F37021)
- Sprint 13 Planning: Feature 3 - Logo Branding Issues
- User Feedback: docs/builder/incoming/open-bugs.md (lines 6-22)
- User's explicit requirements:
  - Tool name `tq` in lowercase
  - Letter 't' in Teradata orange
  - Use block character █ (simpler than | and _)
  - Logo renders correctly (last two lines not offset)
  - Interactive prompt `tq>` in Teradata orange (not green)

---

## Approval

**Status:** Ready for Implementation

**User Requirements Addressed:**
1. ✅ Tool name `tq` in lowercase - SPECIFIED
2. ✅ Letter 't' in Teradata orange (#F37021) - SPECIFIED with RGB values
3. ✅ Use block character █ - SPECIFIED with exact Unicode codepoint
4. ✅ Logo renders correctly (last two lines not offset) - SPECIFIED with exact spacing
5. ✅ Interactive prompt `tq>` in Teradata orange (not green) - SPECIFIED with implementation code

**Next Steps:**
1. Architect implements logo per exact specification
2. Architect implements prompt colors per specification
3. Architect runs manual validation tests
4. User validates visual appearance
5. Document approved and becomes authoritative reference

This specification is complete, unambiguous, and ready for implementation without guesswork.
