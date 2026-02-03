//! Test Fixtures for Pager Dimensional Validation
//!
//! Sprint 30: Provides helper functions to create QueryResult fixtures
//! for testing the refactored pager that accepts structured data instead
//! of pre-formatted strings.

use tq::db::{ColumnMetadata, QueryResult, TeradataType, Value};
use std::time::Duration;

/// Create a minimal test QueryResult with known structure (3 columns, 5 rows)
///
/// Structure:
/// - Column 1: "id" (integer values 1-5)
/// - Column 2: "name" (string values "row1"-"row5")
/// - Column 3: "value" (integer values 10-50)
///
/// This provides a simple, predictable dataset for basic pager testing.
pub fn create_test_query_result() -> QueryResult {
    let columns = vec![
        ColumnMetadata::new("id", TeradataType::Integer, false),
        ColumnMetadata::new("name", TeradataType::Varchar, false),
        ColumnMetadata::new("value", TeradataType::Integer, false),
    ];

    let rows = vec![
        vec![Value::Integer(1), Value::String("row1".to_string()), Value::Integer(10)],
        vec![Value::Integer(2), Value::String("row2".to_string()), Value::Integer(20)],
        vec![Value::Integer(3), Value::String("row3".to_string()), Value::Integer(30)],
        vec![Value::Integer(4), Value::String("row4".to_string()), Value::Integer(40)],
        vec![Value::Integer(5), Value::String("row5".to_string()), Value::Integer(50)],
    ];

    QueryResult::new(columns, rows, Duration::from_millis(10))
}

/// Create a wide QueryResult with many columns for width testing
///
/// # Arguments
/// * `col_count` - Number of columns to generate (e.g., 30 to simulate DBC.TablesV)
///
/// Each column:
/// - Named "col1", "col2", ..., "colN"
/// - Contains string values "val1_col1", "val2_col1", etc.
/// - Widths vary to simulate real-world scenarios
pub fn create_wide_query_result(col_count: usize) -> QueryResult {
    let columns: Vec<ColumnMetadata> = (0..col_count)
        .map(|i| ColumnMetadata::new(format!("col{}", i + 1), TeradataType::Varchar, false))
        .collect();

    // Create 10 rows of data
    let rows: Vec<Vec<Value>> = (0..10)
        .map(|row_idx| {
            (0..col_count)
                .map(|col_idx| {
                    Value::String(format!("val{}_col{}", row_idx + 1, col_idx + 1))
                })
                .collect()
        })
        .collect();

    QueryResult::new(columns, rows, Duration::from_millis(50))
}

/// Create a QueryResult with a single very wide column
///
/// # Arguments
/// * `value_width` - Width of the value in each cell (e.g., 200 for very long values)
///
/// Tests edge case: What happens when one column has extremely long values?
pub fn create_single_wide_column_result(value_width: usize) -> QueryResult {
    let columns = vec![
        ColumnMetadata::new("wide_column", TeradataType::Varchar, false)
    ];

    // Generate value with specified width (repeated 'x' characters)
    let wide_value = "x".repeat(value_width);

    let rows = vec![
        vec![Value::String(wide_value.clone())],
        vec![Value::String(wide_value.clone())],
        vec![Value::String(wide_value)],
    ];

    QueryResult::new(columns, rows, Duration::from_millis(10))
}

/// Create a QueryResult where all columns have wide values
///
/// # Arguments
/// * `cols` - Number of columns
/// * `width` - Width of values in each column
///
/// Tests scenario: Every column is too wide to fit comfortably, requiring
/// proportional truncation.
pub fn create_all_wide_columns_result(cols: usize, width: usize) -> QueryResult {
    let columns: Vec<ColumnMetadata> = (0..cols)
        .map(|i| ColumnMetadata::new(format!("widecol{}", i + 1), TeradataType::Varchar, false))
        .collect();

    // Each cell has a wide value
    let rows: Vec<Vec<Value>> = (0..5)
        .map(|row_idx| {
            (0..cols)
                .map(|col_idx| {
                    // Generate wide value for this cell
                    let content = format!("row{}_col{}", row_idx + 1, col_idx + 1);
                    let padding = "x".repeat(width.saturating_sub(content.len()));
                    Value::String(format!("{}{}", content, padding))
                })
                .collect()
        })
        .collect();

    QueryResult::new(columns, rows, Duration::from_millis(20))
}

/// Create a QueryResult with varying column widths for realistic testing
///
/// Simulates a real-world table with mixed column widths:
/// - Short ID columns (10 chars)
/// - Medium name columns (30 chars)
/// - Long description columns (100+ chars)
pub fn create_mixed_width_query_result() -> QueryResult {
    let columns = vec![
        ColumnMetadata::new("id", TeradataType::Integer, false),
        ColumnMetadata::new("short_name", TeradataType::Varchar, false),
        ColumnMetadata::new("long_description", TeradataType::Varchar, false),
        ColumnMetadata::new("medium_field", TeradataType::Varchar, false),
    ];

    let rows = vec![
        vec![
            Value::Integer(1),
            Value::String("item1".to_string()),
            Value::String("This is a very long description that will definitely require truncation in narrow terminals because it contains so much detail about the item".to_string()),
            Value::String("Some medium-length content here".to_string()),
        ],
        vec![
            Value::Integer(2),
            Value::String("item2".to_string()),
            Value::String("Another lengthy description with lots of words and information that exceeds reasonable display width in most terminal configurations".to_string()),
            Value::String("More medium content".to_string()),
        ],
    ];

    QueryResult::new(columns, rows, Duration::from_millis(15))
}

/// Count visible columns in rendered pager output
///
/// Parses the pager's rendered output to determine how many columns
/// are currently visible. This is useful for validating that the pager
/// adjusts column selection based on terminal width.
///
/// # Example
/// ```ignore
/// let rendered = pager.render_page(0);
/// let visible = count_visible_columns(&rendered);
/// assert!(visible >= 3, "Should show at least 3 columns");
/// ```
pub fn count_visible_columns(rendered: &str) -> usize {
    // Find the header row (first row with │ separators)
    for line in rendered.lines() {
        if line.contains('│') && !is_border_line(line) {
            // Count the number of │ separators (columns = separators - 1)
            let separator_count = line.chars().filter(|&c| c == '│').count();
            if separator_count >= 2 {
                return separator_count - 1;
            }
        }
    }
    0
}

/// Check if a line is a border line (not a data row)
fn is_border_line(line: &str) -> bool {
    line.chars().all(|c| {
        matches!(c, '─' | '│' | '┌' | '┐' | '├' | '┤' | '└' | '┘' |
                     '╭' | '╮' | '╰' | '╯' | '┼' | '┬' | '┴' | ' ')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_query_result() {
        let result = create_test_query_result();
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.row_count, 5);
        assert_eq!(result.columns[0].name, "id");
    }

    #[test]
    fn test_create_wide_query_result() {
        let result = create_wide_query_result(30);
        assert_eq!(result.columns.len(), 30);
        assert_eq!(result.row_count, 10);
    }

    #[test]
    fn test_create_single_wide_column_result() {
        let result = create_single_wide_column_result(200);
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.row_count, 3);
    }

    #[test]
    fn test_create_all_wide_columns_result() {
        let result = create_all_wide_columns_result(10, 50);
        assert_eq!(result.columns.len(), 10);
        assert_eq!(result.row_count, 5);
    }

    #[test]
    fn test_create_mixed_width_query_result() {
        let result = create_mixed_width_query_result();
        assert_eq!(result.columns.len(), 4);
        assert_eq!(result.row_count, 2);
    }

    #[test]
    fn test_count_visible_columns() {
        // Test with a sample rendered output
        let rendered = "│ col1 │ col2 │ col3 │\n│ val1 │ val2 │ val3 │";
        assert_eq!(count_visible_columns(rendered), 3);
    }

    #[test]
    fn test_count_visible_columns_with_borders() {
        let rendered = "╭──────┬──────┬──────╮\n│ col1 │ col2 │ col3 │\n├──────┼──────┼──────┤";
        assert_eq!(count_visible_columns(rendered), 3);
    }

    #[test]
    fn test_is_border_line() {
        assert!(is_border_line("├──────┼──────┼──────┤"));
        assert!(is_border_line("╭──────┬──────┬──────╮"));
        assert!(!is_border_line("│ col1 │ col2 │ col3 │"));
    }
}
