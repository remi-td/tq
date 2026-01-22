# tq Test Cases

This directory contains comprehensive test case definitions for the tq (Teradata Query) CLI tool.

## IMPORTANT: Sprint 19 Manual Testing Requirements

**Sprint 19 introduces MANDATORY manual testing based on Sprint 18 false positive lessons.**

### Sprint 18 Failure Analysis

Sprint 18 was APPROVED with 100% test pass rate, but user reported bugs STILL present.

**Root Cause:** Automated tests validated CODE behavior, not USER experience.

### Sprint 19 Testing Approach

**MANDATORY for Sprint 19:**
1. ✅ Real Terminal Testing - No PTY automation
2. ✅ Human Visual Validation - Must see what user sees
3. ✅ Screenshot Evidence - Visual proof required
4. ✅ Manual Execution - No code review substitutes
5. ✅ User Scenario Reproduction - Test exact sequences

**Sprint 19 Test Cases (manual visual tests):**
- `TC-LOGO-002.md` - Logo ASCII art verification
- `TC-TAB-COMPLETION-001.md` - Tab completion after FROM
- `TC-TAB-COMPLETION-002.md` - Tab completion qualified names

**Key Lesson:** "100% automated test pass rate means NOTHING if tests don't validate what user sees."

---

## Quick Start

1. **Build tq**:
   ```bash
   cargo build --release
   ```

2. **Set up test environment**:
   ```bash
   # Create .env file with credentials (recommended)
   cp .env.example .env
   # Edit .env to set: TQ_LOGON=testuser:testpass@testhost:1025/testdb

   # Add binary to PATH
   export PATH="$PWD/target/release:$PATH"
   ```

   **Alternative**: Set credentials via environment variable
   ```bash
   export TQ_LOGON="testuser:testpass@testhost:1025/testdb"
   ```

   **Note**: The `.env` file approach is recommended for testing as it keeps credentials secure and is automatically loaded by tq.

3. **Start testing**:
   - Review `INDEX.md` for complete test catalog
   - Execute test cases in recommended order
   - Document results in each test case file

## Test Case Files

- **TCXXX**: Individual test case definitions for each requirement
- **INDEX.md**: Complete test catalog with coverage matrix
- **README.md**: This file (quick start guide)

## Test Structure

Each test case file contains:

- **Metadata**: ID, title, category, priority, feature reference
- **Purpose**: What the test validates
- **Scope**: Specific aspects being tested
- **Prerequisites**: Required setup
- **Test Procedure**: Step-by-step instructions with exact commands
- **Expected Results**: What should happen
- **Actual Results**: Space for documenting execution results
- **Pass/Fail Criteria**: Clear success/failure conditions
- **Notes**: Additional context and considerations

## Categories

- **Functionality**: Core features (ping, query, formats, auth)
- **Error-Handling**: Error detection and user feedback
- **Usability**: Help, verbosity, colors
- **Integration**: Exit codes, environment variables, format compliance
- **Security**: Credential protection

## Priority Levels

- **Critical**: Must pass for MVP release (9 test cases)
- **High**: Important features (11 test cases)
- **Medium**: Quality of life (5 test cases)

## Execution Notes

### Quick Smoke Test
```bash
# Verify basic functionality
tq --version          # Should show version
tq --help            # Should show help
tq ping              # Should connect (if credentials valid)
tq query "SELECT 1"  # Should execute query
```

### Full Test Suite
Follow the recommended order in `INDEX.md`:
1. Smoke tests (TC001, TC003, TC013)
2. Core functionality
3. Error handling
4. Integration
5. Security
6. Quality

### Testing REPL mode

The tq tool is either run in batch mode or in REPL mode, YOU MUST TEST IN BOTH MODES.
This may require the development of a specific tool to test the REPL mode. (update this file with the tool name and usage once created)

### Configuration Methods

Tests support multiple configuration methods (in order of precedence):
1. Command-line `--logon` flag (for explicit override testing)
2. Explicit `TQ_LOGON` environment variable export
3. `.env` file in project directory (recommended for testing)
4. Configuration file (`~/.config/tq/config.toml`)

**Recommended**: Use `.env` file for test execution as it provides a consistent, secure baseline configuration.

### Platform Notes

- **Linux**: All tests supported
- **macOS**: All tests supported
- **Windows**: Most tests supported (some Unix-specific tests may need adjustment)

## Test Results

After running tests, create a summary:

```bash
# Example results summary
cat > test_results_$(date +%Y%m%d).md << 'EOF'
# Test Results - $(date +%Y-%m-%d)

## Pass/Fail Summary
- Critical: 9/9 PASS
- High: 11/11 PASS
- Medium: 5/5 PASS
- **Total: 25/25 PASS**

## Issues Found
None

## Recommendations
Ready for release
EOF
```

## Contributing

When adding new test cases:

1. Use next sequential number (TC026, TC027, etc.)
2. Follow the established template
3. Update INDEX.md
4. Add to appropriate category
5. Assign correct priority

## Support

For questions or issues with test cases:
- Review specifications: `docs/builder/specifications.md`
- Check design guide: `docs/builder/rust-cli-design-general.md`
- See project overview: `CLAUDE.md`

---

**Status**: Test cases defined and ready for execution
**Next Step**: Execute tests and document results
**Goal**: Validate MVP functionality before release
