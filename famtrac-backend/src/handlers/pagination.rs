use serde::{Deserialize, Serialize};

/// Pagination parameters for list requests
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Maximum number of items to return (default: 50, max: 100)
    pub limit: Option<u32>,
    /// Token for fetching the next page of results
    pub next_token: Option<String>,
}

impl PaginationParams {
    /// Get the effective limit, applying defaults and constraints
    pub fn effective_limit(&self) -> u32 {
        const DEFAULT_LIMIT: u32 = 50;
        const MAX_LIMIT: u32 = 100;

        self.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The items in this page
    pub items: Vec<T>,
    /// Token to fetch the next page (None if this is the last page)
    pub next_token: Option<String>,
    /// Total count of items (optional, may be expensive to compute)
    pub total_count: Option<u32>,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response without pagination (all items)
    pub fn unpaginated(items: Vec<T>) -> Self {
        Self {
            items,
            next_token: None,
            total_count: None,
        }
    }

    /// Create a paginated response with a next token
    pub fn with_next_token(items: Vec<T>, next_token: Option<String>) -> Self {
        Self {
            items,
            next_token,
            total_count: None,
        }
    }
}
