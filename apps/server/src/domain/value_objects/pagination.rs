use crate::domain::errors::{DomainError, DomainResult};

/// Pagination parameters for list queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationParams {
    page: u32,
    page_size: u32,
}

impl PaginationParams {
    /// Default page size when not specified
    pub const DEFAULT_PAGE_SIZE: u32 = 20;

    /// Maximum allowed page size to prevent abuse
    pub const MAX_PAGE_SIZE: u32 = 100;

    /// Minimum page number
    pub const MIN_PAGE: u32 = 1;

    /// Create new pagination parameters with validation
    pub fn new(page: u32, page_size: u32) -> DomainResult<Self> {
        if page < Self::MIN_PAGE {
            return Err(DomainError::validation(format!(
                "Page must be at least {}",
                Self::MIN_PAGE
            )));
        }

        if page_size == 0 {
            return Err(DomainError::validation(
                "Page size must be greater than 0".to_string(),
            ));
        }

        if page_size > Self::MAX_PAGE_SIZE {
            return Err(DomainError::validation(format!(
                "Page size cannot exceed {}",
                Self::MAX_PAGE_SIZE
            )));
        }

        Ok(Self { page, page_size })
    }

    /// Create default pagination parameters (page 1, default page size)
    pub fn default() -> Self {
        Self {
            page: Self::MIN_PAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
        }
    }

    /// Get the current page number (1-indexed)
    pub fn page(&self) -> u32 {
        self.page
    }

    /// Get the page size
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Calculate the offset for database queries (0-indexed)
    pub fn offset(&self) -> u64 {
        ((self.page - 1) as u64) * (self.page_size as u64)
    }

    /// Get limit for database queries
    pub fn limit(&self) -> u64 {
        self.page_size as u64
    }
}

/// Pagination metadata to be included in paginated responses
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationMetadata {
    total_count: u64,
    page: u32,
    page_size: u32,
    total_pages: u32,
    has_next: bool,
    has_previous: bool,
}

impl PaginationMetadata {
    /// Create pagination metadata from parameters and total count
    pub fn new(params: &PaginationParams, total_count: u64) -> Self {
        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64) / (params.page_size as f64)).ceil() as u32
        };

        let has_next = params.page < total_pages;
        let has_previous = params.page > PaginationParams::MIN_PAGE;

        Self {
            total_count,
            page: params.page,
            page_size: params.page_size,
            total_pages,
            has_next,
            has_previous,
        }
    }

    /// Get total count of items across all pages
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Get current page number (1-indexed)
    pub fn page(&self) -> u32 {
        self.page
    }

    /// Get page size
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Get total number of pages
    pub fn total_pages(&self) -> u32 {
        self.total_pages
    }

    /// Check if there is a next page
    pub fn has_next(&self) -> bool {
        self.has_next
    }

    /// Check if there is a previous page
    pub fn has_previous(&self) -> bool {
        self.has_previous
    }
}

/// Paginated result containing items and pagination metadata
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    items: Vec<T>,
    metadata: PaginationMetadata,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(items: Vec<T>, params: &PaginationParams, total_count: u64) -> Self {
        let metadata = PaginationMetadata::new(params, total_count);
        Self { items, metadata }
    }

    /// Get the items in this page
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    /// Get pagination metadata
    pub fn metadata(&self) -> &PaginationMetadata {
        &self.metadata
    }

    /// Consume self and return items and metadata separately
    pub fn into_parts(self) -> (Vec<T>, PaginationMetadata) {
        (self.items, self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_valid() {
        let params = PaginationParams::new(1, 20).unwrap();
        assert_eq!(params.page(), 1);
        assert_eq!(params.page_size(), 20);
        assert_eq!(params.offset(), 0);
        assert_eq!(params.limit(), 20);
    }

    #[test]
    fn test_pagination_params_offset_calculation() {
        let params = PaginationParams::new(3, 20).unwrap();
        assert_eq!(params.offset(), 40); // (3-1) * 20 = 40
    }

    #[test]
    fn test_pagination_params_invalid_page() {
        let result = PaginationParams::new(0, 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_params_invalid_page_size_zero() {
        let result = PaginationParams::new(1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_params_invalid_page_size_too_large() {
        let result = PaginationParams::new(1, 101);
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_metadata() {
        let params = PaginationParams::new(2, 20).unwrap();
        let metadata = PaginationMetadata::new(&params, 100);

        assert_eq!(metadata.total_count(), 100);
        assert_eq!(metadata.page(), 2);
        assert_eq!(metadata.page_size(), 20);
        assert_eq!(metadata.total_pages(), 5);
        assert!(metadata.has_next());
        assert!(metadata.has_previous());
    }

    #[test]
    fn test_pagination_metadata_first_page() {
        let params = PaginationParams::new(1, 20).unwrap();
        let metadata = PaginationMetadata::new(&params, 100);

        assert!(metadata.has_next());
        assert!(!metadata.has_previous());
    }

    #[test]
    fn test_pagination_metadata_last_page() {
        let params = PaginationParams::new(5, 20).unwrap();
        let metadata = PaginationMetadata::new(&params, 100);

        assert!(!metadata.has_next());
        assert!(metadata.has_previous());
    }

    #[test]
    fn test_pagination_metadata_empty_results() {
        let params = PaginationParams::new(1, 20).unwrap();
        let metadata = PaginationMetadata::new(&params, 0);

        assert_eq!(metadata.total_pages(), 0);
        assert!(!metadata.has_next());
        assert!(!metadata.has_previous());
    }

    #[test]
    fn test_paginated_result() {
        let params = PaginationParams::new(1, 20).unwrap();
        let items = vec![1, 2, 3];
        let result = PaginatedResult::new(items.clone(), &params, 100);

        assert_eq!(result.items(), &items);
        assert_eq!(result.metadata().total_count(), 100);
    }
}
