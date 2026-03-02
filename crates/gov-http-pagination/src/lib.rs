//! Shared pagination types for platform API endpoints.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Common pagination query parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed, default 1).
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (default 50, max 100).
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self { page: 1, per_page: 50 }
    }
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Pagination metadata in responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of items across all pages.
    pub total_items: usize,
    /// Total number of pages.
    pub total_pages: u32,
}

/// Generic wrapper for paginated responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The page of data.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub pagination: Pagination,
}

impl Pagination {
    /// Create new pagination metadata from total count and params.
    pub fn new(total_items: usize, page: u32, per_page: u32) -> Self {
        let per_page = per_page.clamp(1, 100);
        let page = page.max(1);
        let total_pages =
            if total_items == 0 { 0 } else { (total_items as u32).div_ceil(per_page) };

        Self { page, per_page, total_items, total_pages }
    }

    /// Get the number of items to skip for the current page.
    pub fn offset(&self) -> usize {
        ((self.page - 1) * self.per_page) as usize
    }

    /// Get the number of items to take for the current page.
    pub fn limit(&self) -> usize {
        self.per_page as usize
    }
}

#[cfg(test)]
mod tests {
    use super::Pagination;

    #[test]
    fn pagination_clamps_page_and_per_page() {
        let pagination = Pagination::new(25, 0, 500);

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 100);
        assert_eq!(pagination.total_pages, 1);
    }

    #[test]
    fn pagination_computes_offset_and_limit() {
        let pagination = Pagination::new(200, 3, 20);

        assert_eq!(pagination.offset(), 40);
        assert_eq!(pagination.limit(), 20);
        assert_eq!(pagination.total_pages, 10);
    }

    #[test]
    fn pagination_handles_empty_result_set() {
        let pagination = Pagination::new(0, 2, 10);

        assert_eq!(pagination.total_pages, 0);
    }
}
