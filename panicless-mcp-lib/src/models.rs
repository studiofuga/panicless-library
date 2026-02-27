use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub author: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub pages: Option<i32>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookWithReadings {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub pages: Option<i32>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub reading_count: i64,
    pub readings_summary: Option<String>,
}

/// Raw reading record without joins — used for single-record CRUD operations.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reading {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reading record joined with book title/author — used for list views.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingWithBook {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub book_title: String,
    pub book_author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct YearStats {
    pub year: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStats {
    pub total_readings: i64,
    pub completed_readings: i64,
    pub current_readings: i64,
    pub total_books_read: i64,
    pub average_rating: Option<f64>,
    pub books_by_year: Vec<YearStats>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Validates `sort_by` against a whitelist and returns a safe `(sql_column, sql_direction)` pair.
///
/// - `whitelist` maps parameter names to SQL column expressions (e.g. `("start_date", "r.start_date")`)
/// - Falls back to `default_col` / `default_order` when inputs are `None` or not in whitelist
pub fn resolve_order_by(
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    whitelist: &[(&str, &str)],
    default_col: &str,
    default_order: &str,
) -> (String, String) {
    let col = sort_by
        .and_then(|key| whitelist.iter().find(|(name, _)| *name == key))
        .map(|(_, sql_expr)| sql_expr.to_string())
        .unwrap_or_else(|| default_col.to_string());

    let dir = match sort_order {
        Some(SortOrder::Asc) => "ASC".to_string(),
        Some(SortOrder::Desc) => "DESC".to_string(),
        None => default_order.to_string(),
    };

    (col, dir)
}
