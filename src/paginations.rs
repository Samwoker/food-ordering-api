use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]

pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Serialize)]

pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, paging: &Pagination) -> Self {
        Self {
            data,
            pagination: PaginationMeta {
                page: paging.page,
                limit: paging.limit,
                total,
                total_pages: (total as f64 / paging.limit as f64).ceil() as i64,
            },
        }
    }
}
