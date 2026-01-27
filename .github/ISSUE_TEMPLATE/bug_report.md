---
name: Bug Report
about: Report a bug or unexpected behavior in tq
title: "[BUG] "
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of what the bug is.

## Steps to Reproduce

1. Run command: `tq [your command here]`
2. [Additional steps...]
3. See error

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened. Include any error messages or unexpected output.

```
[Paste error messages or output here]
```

## Environment

**tq Version:**
```bash
tq --version
```

**Operating System:**
- [ ] macOS (version: )
- [ ] Linux (distribution: )
- [ ] Windows (version: )

**Shell/Terminal:**
- Shell: (e.g., bash, zsh, fish)
- Terminal: (e.g., Terminal.app, iTerm2, Windows Terminal)

**Teradata Environment:**
- Teradata version: (if known)
- Connection method:
  - [ ] Command-line flags (`-h`, `-u`, etc.)
  - [ ] Environment variable (`TQ_LOGON`)
  - [ ] Configuration profile

## Execution Mode

Which mode were you using when the bug occurred?

- [ ] One-shot query (`tq -q "SELECT ..."`)
- [ ] REPL/Interactive mode (`tq` then commands)
- [ ] Batch mode (reading from file)
- [ ] Piped input (`echo "SELECT ..." | tq`)

## Additional Context

**Configuration:**
Do you have a `.tq/config.toml` or profile configuration? (Share relevant parts, **remove any credentials**)

**Recent Changes:**
- [ ] This is a fresh installation
- [ ] This worked before and recently broke
- [ ] I just upgraded tq from version: ___

**Output Format:**
If the bug relates to output formatting, which format were you using?
- [ ] Table (default)
- [ ] JSON
- [ ] CSV
- [ ] Other: ___

**Sample Query:**
If possible, provide a minimal query that reproduces the issue (anonymize sensitive data):

```sql
-- Your sample query here
```

**Screenshots:**
If applicable, add screenshots to help explain the problem (especially for display/formatting issues).

**Workaround:**
If you found a temporary workaround, please share it to help others.

## Additional Information

Any other context about the problem that might be helpful.

---

**Note:** Please ensure you've removed any sensitive information (passwords, database names, sensitive data) before submitting.
