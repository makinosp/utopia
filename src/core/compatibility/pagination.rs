use serde::{Deserialize, Serialize};

/// Default page number for pagination.
pub const DEFAULT_PAGE: u32 = 1;

/// Default limit for pagination.
pub const DEFAULT_LIMIT: u32 = 50;

/// Maximum limit for pagination.
pub const MAX_LIMIT: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaginationMeta {
    pub total: u64,
    pub count: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Clone)]
pub struct Paginated<T> {
    pub total_records: u64,
    pub records: Vec<T>,
    pub current_page: u64,
    pub per_page: u64,
}

pub fn compute_pagination<T>(paginated: &Paginated<T>) -> PaginationMeta {
    let total_pages = if paginated.per_page == 0 {
        0
    } else {
        paginated.total_records.div_ceil(paginated.per_page)
    };

    PaginationMeta {
        total: paginated.total_records,
        count: paginated.records.len() as u64,
        per_page: paginated.per_page,
        current_page: paginated.current_page,
        total_pages,
    }
}
