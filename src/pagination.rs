//! Client-side result pagination
//!
//! Provides pagination metadata and row slicing for paginated output.
//! Used by query, search, and list commands when `--page-size` is specified.

use std::io::Write;

use crate::error::Result;

/// Pagination metadata for slicing and rendering paginated results
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationInfo {
    /// Current page number (1-based)
    pub page: usize,
    /// Number of rows per page
    pub page_size: usize,
    /// Total number of rows before pagination
    pub total_rows: usize,
}

impl PaginationInfo {
    /// Create a new PaginationInfo
    pub fn new(page: usize, page_size: usize, total_rows: usize) -> Self {
        Self {
            page,
            page_size,
            total_rows,
        }
    }

    /// Returns (start_index, end_index) for slicing rows.
    ///
    /// If the page exceeds total pages, start >= total_rows and the range is empty.
    pub fn row_range(&self) -> (usize, usize) {
        let start = (self.page - 1) * self.page_size;
        let end = (start + self.page_size).min(self.total_rows);
        (start, end)
    }

    /// Whether there are more pages after the current one
    pub fn has_more(&self) -> bool {
        self.page * self.page_size < self.total_rows
    }

    /// Total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_rows == 0 {
            1
        } else {
            self.total_rows.div_ceil(self.page_size)
        }
    }

    /// Write the pagination footer for non-JSON formats.
    ///
    /// Output: `Page 2 of 5 (47 total rows)`
    pub fn write_footer<W: Write>(&self, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "Page {} of {} ({} total rows)",
            self.page,
            self.total_pages(),
            self.total_rows
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_range_first_page() {
        let pg = PaginationInfo::new(1, 10, 47);
        assert_eq!(pg.row_range(), (0, 10));
    }

    #[test]
    fn test_row_range_middle_page() {
        let pg = PaginationInfo::new(3, 10, 47);
        assert_eq!(pg.row_range(), (20, 30));
    }

    #[test]
    fn test_row_range_last_page() {
        let pg = PaginationInfo::new(5, 10, 47);
        assert_eq!(pg.row_range(), (40, 47));
    }

    #[test]
    fn test_row_range_beyond_last_page() {
        let pg = PaginationInfo::new(6, 10, 47);
        // start = 50, end = min(60, 47) = 47, but start > end effectively means empty
        let (start, end) = pg.row_range();
        assert!(start >= end);
    }

    #[test]
    fn test_has_more() {
        assert!(PaginationInfo::new(1, 10, 47).has_more());
        assert!(PaginationInfo::new(4, 10, 47).has_more());
        assert!(!PaginationInfo::new(5, 10, 47).has_more());
        assert!(!PaginationInfo::new(6, 10, 47).has_more());
    }

    #[test]
    fn test_total_pages() {
        assert_eq!(PaginationInfo::new(1, 10, 47).total_pages(), 5);
        assert_eq!(PaginationInfo::new(1, 10, 50).total_pages(), 5);
        assert_eq!(PaginationInfo::new(1, 10, 1).total_pages(), 1);
        assert_eq!(PaginationInfo::new(1, 10, 0).total_pages(), 1);
    }

    #[test]
    fn test_write_footer() {
        let pg = PaginationInfo::new(2, 10, 47);
        let mut buf = Vec::new();
        pg.write_footer(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "Page 2 of 5 (47 total rows)\n");
    }

    #[test]
    fn test_single_page() {
        let pg = PaginationInfo::new(1, 100, 5);
        assert_eq!(pg.total_pages(), 1);
        assert!(!pg.has_more());
        assert_eq!(pg.row_range(), (0, 5));
    }

    #[test]
    fn test_exact_fit() {
        let pg = PaginationInfo::new(1, 10, 10);
        assert_eq!(pg.total_pages(), 1);
        assert!(!pg.has_more());
        assert_eq!(pg.row_range(), (0, 10));
    }
}
