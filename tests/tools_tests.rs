//! Test Suite for Dimensional Testing Infrastructure
//!
//! Sprint 30: This test file validates the test utilities in `tests/tools/`
//! including visual_validator and terminal_simulator.
//!
//! Run these tests with:
//! ```bash
//! cargo test --test tools_tests
//! ```

mod tools;

use tools::terminal_simulator::{
    create_test_table, TerminalSimulator, TERMINAL_117, TERMINAL_120X40, TERMINAL_80X24,
    TERMINAL_NARROW, TERMINAL_WIDE,
};
use tools::visual_validator::{
    assert_column_widths_within_terminal, assert_no_overflow, assert_truncation_markers_present,
    display_width, line_count, max_line_width,
};

// =============================================================================
// Visual Validator Integration Tests
// =============================================================================

/// Test that assert_no_overflow correctly validates table output
#[test]
fn test_visual_validator_table_output() {
    let table = "╭──────┬──────┬──────╮\n\
                 │ col1 │ col2 │ col3 │\n\
                 ├──────┼──────┼──────┤\n\
                 │ val1 │ val2 │ val3 │\n\
                 ╰──────┴──────┴──────╯";

    // This table is about 22 chars wide, should fit in 80
    assert_no_overflow(table, 80);
}

/// Test that overflow detection works with realistic table data
#[test]
#[should_panic(expected = "OVERFLOW DETECTED")]
fn test_visual_validator_detects_wide_table() {
    // Create a table line that's clearly too wide
    let wide_line = "│ ".to_string() + &"x".repeat(100) + " │";
    assert_no_overflow(&wide_line, 50);
}

/// Test column width validation with formatted table
#[test]
fn test_column_widths_validation() {
    let table = "│ id │ name │ value │\n\
                 │ 1  │ foo  │ bar   │";

    // Table should fit within 80 columns
    assert_column_widths_within_terminal(table, 80);
}

/// Test truncation marker detection
#[test]
fn test_truncation_detection() {
    let table_with_truncation = "│ id │ long_name… │ value │\n\
                                  │ 1  │ truncate…  │ ok    │";

    // Column 1 should have truncation markers
    assert_truncation_markers_present(table_with_truncation, &[1]);
}

/// Test utility functions
#[test]
fn test_utility_functions() {
    // display_width
    assert_eq!(display_width("hello"), 5);
    assert_eq!(display_width(""), 0);

    // max_line_width
    assert_eq!(max_line_width("short\nlonger line\nmed"), 11);
    assert_eq!(max_line_width(""), 0);

    // line_count
    assert_eq!(line_count("one\ntwo\nthree"), 3);
    assert_eq!(line_count("single"), 1);
    assert_eq!(line_count(""), 0);
}

// =============================================================================
// Terminal Simulator Integration Tests
// =============================================================================

/// Test terminal creation and size methods
#[test]
fn test_terminal_simulator_creation() {
    let term = TerminalSimulator::new(80, 24);
    assert_eq!(term.size(), (80, 24));
    assert_eq!(term.width(), 80);
    assert_eq!(term.height(), 24);
}

/// Test terminal simulator with standard sizes
#[test]
fn test_terminal_standard_sizes() {
    let term_80 = TerminalSimulator::from_tuple(TERMINAL_80X24);
    let term_120 = TerminalSimulator::from_tuple(TERMINAL_120X40);
    let term_117 = TerminalSimulator::from_tuple(TERMINAL_117);

    assert_eq!(term_80.size(), (80, 24));
    assert_eq!(term_120.size(), (120, 40));
    assert_eq!(term_117.size(), (117, 40));
}

/// Test validation of output that fits
#[test]
fn test_terminal_validates_fitting_output() {
    let term = TerminalSimulator::new(80, 24);
    let output = "This is a short line\nAnd another\nAnd one more";

    assert!(term.validate_output(output).is_ok());
    assert!(term.width_fits("short line"));
    assert!(term.height_fits(output));
}

/// Test detection of width overflow
#[test]
fn test_terminal_detects_width_overflow() {
    let term = TerminalSimulator::new(40, 24);
    let wide_line = "x".repeat(50);

    assert!(!term.width_fits(&wide_line));
    assert!(term.validate_output(&wide_line).is_err());
    assert_eq!(term.width_overflow(&wide_line), 10);
}

/// Test detection of height overflow
#[test]
fn test_terminal_detects_height_overflow() {
    let term = TerminalSimulator::new(80, 5);
    let many_lines = "line\n".repeat(10);

    assert!(!term.height_fits(&many_lines));
    assert!(term.validate_output(&many_lines).is_err());
    assert_eq!(term.height_overflow(&many_lines), 5);
}

/// Test detailed validation report
#[test]
fn test_terminal_detailed_report() {
    let term = TerminalSimulator::new(80, 24);
    let output = "short\nlines\nhere";

    let report = term.detailed_report(output);
    assert!(report.fits());
    assert!(report.width_fits());
    assert!(report.height_fits());
    assert_eq!(report.output_line_count, 3);
    assert!(report.lines_over_width.is_empty());
}

/// Test create_test_table helper
#[test]
fn test_create_test_table() {
    let table = create_test_table(3, 2, 8);
    let lines: Vec<&str> = table.lines().collect();

    // 1 header + 2 data rows
    assert_eq!(lines.len(), 3);

    // Each line should be a valid table row
    for line in &lines {
        assert!(line.starts_with('│'));
        assert!(line.ends_with('│'));
    }

    // Should contain expected content
    assert!(table.contains("col0"));
    assert!(table.contains("col1"));
    assert!(table.contains("col2"));
    assert!(table.contains("r0c0"));
    assert!(table.contains("r1c2"));
}

// =============================================================================
// Combined Validation Tests - Realistic Scenarios
// =============================================================================

/// Test validating pager output against 117-char terminal (Sprint 29 issue)
#[test]
fn test_sprint_29_scenario_narrow_terminal() {
    let term = TerminalSimulator::from_tuple(TERMINAL_117);

    // Create a table that should fit in 117 chars
    let narrow_table = create_test_table(5, 3, 10);

    // Validate it fits
    let report = term.detailed_report(&narrow_table);

    // If this table doesn't fit, we can see exactly why
    if !report.width_fits() {
        panic!(
            "Table should fit in 117 chars but doesn't:\n\
             Max line width: {}\n\
             Terminal width: {}\n\
             Overflow: {} chars",
            report.max_line_width, report.terminal_width, report.width_overflow
        );
    }
}

/// Test that wide tables are correctly identified as overflowing
#[test]
fn test_wide_table_detection() {
    let term = TerminalSimulator::from_tuple(TERMINAL_80X24);

    // Create a table that's clearly too wide for 80 chars
    let wide_table = create_test_table(10, 2, 15);

    let report = term.detailed_report(&wide_table);
    assert!(
        !report.width_fits(),
        "Wide table (10 cols x 15 chars) should overflow 80-char terminal"
    );
}

/// Test validation across multiple terminal sizes
#[test]
fn test_multiple_terminal_sizes() {
    let sizes = [TERMINAL_NARROW, TERMINAL_80X24, TERMINAL_117, TERMINAL_120X40, TERMINAL_WIDE];

    // A simple table that should fit in all sizes
    let simple_table = "│ id │ name │\n│ 1  │ foo  │";

    for size in sizes {
        let term = TerminalSimulator::from_tuple(size);
        assert!(
            term.validate_output(simple_table).is_ok(),
            "Simple table should fit in {:?} terminal",
            size
        );
    }
}

/// Test that validation catches specific overflowing lines
#[test]
fn test_identifies_overflowing_lines() {
    let term = TerminalSimulator::new(50, 24);

    // Create output where only line 2 overflows
    let output = "short line\n".to_string() + &"x".repeat(60) + "\nshort again";

    let report = term.detailed_report(&output);
    assert_eq!(report.lines_over_width.len(), 1);
    assert_eq!(report.lines_over_width[0].0, 1); // Line index 1 (second line)
    assert_eq!(report.lines_over_width[0].1, 60); // Width 60
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test empty output handling
#[test]
fn test_empty_output() {
    let term = TerminalSimulator::new(80, 24);

    assert_no_overflow("", 80);
    assert_column_widths_within_terminal("", 80);
    assert_truncation_markers_present("", &[]); // No expectations
    assert!(term.validate_output("").is_ok());
}

/// Test single character terminal (extreme edge case)
#[test]
fn test_minimal_terminal() {
    let term = TerminalSimulator::new(1, 1);

    assert!(term.validate_output("x").is_ok());
    assert!(term.validate_output("xx").is_err());
    assert!(term.validate_output("x\ny").is_err());
}

/// Test Unicode handling
#[test]
fn test_unicode_handling() {
    let term = TerminalSimulator::new(20, 24);

    // Ellipsis is 1 display column
    let with_ellipsis = "hello world…";
    assert!(term.width_fits(with_ellipsis));

    // Test with box drawing characters
    let box_chars = "╭──────╮";
    assert!(term.width_fits(box_chars));
}

// =============================================================================
// Documentation Example Tests
// =============================================================================

/// Verify the documentation examples work as described
#[test]
fn test_documentation_examples() {
    // From visual_validator docs
    let table_output = "| col1 | col2 |\n| ---- | ---- |\n| val  | val  |";
    assert_no_overflow(table_output, 80);

    // From terminal_simulator docs
    let term = TerminalSimulator::new(80, 24);
    assert_eq!(term.size(), (80, 24));
    assert!(term.width_fits("short line"));
    assert!(term.height_fits("line1\nline2\nline3"));
}
