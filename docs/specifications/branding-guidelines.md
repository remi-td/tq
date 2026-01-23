# Branding Guidelines

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

## Logo Design

### Current Design

**Character Set:** ASCII characters `_`, `|`, `{`, `}`, `\`, `` ` ``

**Design Philosophy:** Lowercase ASCII art rendering of "tq" letters using standard ASCII characters. Information messages are displayed to the RIGHT of the logo on the same lines.

**Dimensions:**
- Width: Logo portion approximately 12 characters, total line width varies with info messages
- Height: Exactly 5 lines
- Monospace-friendly: Must render correctly in all monospace fonts

### Logo ASCII Art

**User's Exact Specification:**

```
 __
/\ \__
\ \ ,_\    __
 \ \ \/  /'__`\
  \ \ \_/\ \L\ \
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
```

**Visual Structure:**
- Lines 1-9: Lowercase "t" (left) + lowercase "q" (right) using block characters
- The "t": Left portion forming lowercase letter 't' shape
- The "q": Right portion forming lowercase letter 'q' shape with descending tail
- Character set: `_`, `/`, `\`, `|`, `` ` ``, `{`, `'`, space
- This is the definitive ASCII art design for clarity

### Implementation with Info Messages

Info messages appear on the SAME lines as the logo, to the right:

```
 __                Teradata Query Tool v1.7.0
/\ \__             Connected to host:1025
\ \ ,_\    __      Database: demo_user
 \ \ \/  /'__`\    User: demo_user
  \ \ \_/\ \L\ \   Default row limit: 100
   \ \__\ \___, \  [additional info as needed]
    \/__/\/___/\ \
              \ \_\
               \/_/
```

**Color Application:**

The logo is implemented as two separate character arrays that are combined line-by-line:

**'t' portion (colored in Teradata orange - left side):**
```
 __
/\ \__
\ \ ,_\
 \ \ \/
  \ \ \_
   \ \__
    \/__
```

**'q' portion (default terminal color - right side):**
```

    __
  /'__`\
/\ \L\ \
 \___, \
\/___/\ \
      \ \_\
       \/_/
```

**Combined output:** Each line combines `[orange_t][default_q]   [info_message]`

### Layout Rules

1. Each line combines: `[colored_t_part][q_part]   [info_message]`
2. Spaces separate logo from info messages (adjust for readability)
3. Info messages vertically aligned (all start at same column)
4. Logo is 9 lines tall
5. If fewer than 9 info lines, remaining logo lines print without info

### Logo Display Context

```
[Blank line]
 __                Teradata Query Tool v1.7.0
/\ \__             Connected to mcp-host:1025
\ \ ,_\    __      Database: demo_user
 \ \ \/  /'__`\    User: demo_user
  \ \ \_/\ \L\ \   Default row limit: 100
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
[Blank line]
Type /help for commands, /quit to exit.
[Blank line]
```

## Interactive Prompt

### REPL Prompt Design

**Default Prompt:**
```
tq>
```

**CRITICAL REQUIREMENT:** The ENTIRE prompt `tq>` must be in Teradata orange (#F37021).

**Multi-line Continuation Prompt:**
```
...>
```

**Color:**
- Same Teradata orange (#F37021) as primary prompt
- ENTIRE prompt `...>` in orange

## Terminal Rendering Guidelines

### Color Support Detection

**Priority Order:**
1. **Truecolor (24-bit)** - Use exact Teradata orange RGB(243, 112, 33)
2. **256-color** - Use closest orange approximation
3. **8-color** - Use bright yellow as fallback
4. **No color** - Gracefully disable all colors

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

## Welcome Banner

### REPL Startup Banner

**Full banner format:**
```
[Blank line]
[Logo in Teradata orange/default colors - 5 lines]
[Blank line]
Type /help for commands, /quit to exit.
[Blank line]
```

**Color application:**
- Logo: Per logo specification (orange/default split)
- Tool name "tq": 't' in orange, 'q' in default
- Version: Gray/dim (optional, can be default)
- Connection info labels: Default color
- Connection info values: Default color
- Help hint: Default or gray/dim

**Spacing:**
- One blank line before logo
- One blank line after logo
- One blank line after help hint
- Prompt starts immediately after

## Error Messages and Status

### Error Formatting

**Error prefix:**
```
[RED]Error:[DEFAULT] [error message]
```

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

## Accessibility

### Color Blindness Considerations

**Guidelines:**
- Never use color as the ONLY indicator of meaning
- Use text labels in addition to colors
- Errors have "Error:" prefix (not just red color)
- Success has "✓" symbol (not just green color)

### Screen Reader Support

**Text alternatives:**
- Logo includes version text immediately after
- All visual elements have text equivalents
- Status messages are plain text

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
