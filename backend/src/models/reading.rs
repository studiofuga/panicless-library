use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reading {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
}

impl Default for ReadingQuery {
    fn default() -> Self {
        Self {
            status: None,
            book_id: None,
            year: None,
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ReadingWithBook {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub book_title: String,
    pub book_author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingStats {
    pub total_readings: i64,
    pub completed_readings: i64,
    pub current_readings: i64,
    pub total_books_read: i64,
    pub average_rating: Option<f64>,
    pub books_by_year: Vec<YearStats>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct YearStats {
    pub year: i32,
    pub count: i64,
}
