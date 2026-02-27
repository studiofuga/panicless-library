use chrono::NaiveDate;
use serde::Deserialize;
use validator::Validate;

use super::sort::SortOrder;

pub use panicless_mcp_lib::models::{
    Reading, ReadingStats, ReadingWithBook, YearStats,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReading {
    pub book_id: i32,

    pub start_date: NaiveDate,

    pub end_date: Option<NaiveDate>,

    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i32>,

    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReading {
    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,

    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i32>,

    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CompleteReading {
    pub end_date: NaiveDate,

    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingQuery {
    pub status: Option<String>, // "current", "completed", "all"
    pub book_id: Option<i32>,
    pub year: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for ReadingQuery {
    fn default() -> Self {
        Self {
            status: None,
            book_id: None,
            year: None,
            page: Some(1),
            limit: Some(20),
            sort_by: None,
            sort_order: None,
        }
    }
}
