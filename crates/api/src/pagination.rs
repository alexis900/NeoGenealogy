use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationParams {
    pub fn validated(&self) -> Result<(i64, i64), ApiError> {
        let limit = self.limit.unwrap_or(50);
        let offset = self.offset.unwrap_or(0);
        if !(0..=100).contains(&limit) {
            return Err(ApiError::bad_request(
                "INVALID_LIMIT",
                "limit must be between 0 and 100",
            ));
        }
        if offset < 0 {
            return Err(ApiError::bad_request(
                "INVALID_OFFSET",
                "offset must be >= 0",
            ));
        }
        Ok((limit, offset))
    }
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct Paginated<T: Serialize> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}
