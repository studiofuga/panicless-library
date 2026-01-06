use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBook {
    #[validate(length(min = 1, max = 500))]
    pub title: String,

    #[validate(length(max = 255))]
    pub author: Option<String>,

    #[validate(length(max = 50))]
    pub edition: Option<String>,

    #[validate(length(max = 17))]
    pub isbn: Option<String>,

    #[validate(range(min = 1000, max = 9999))]
    pub publication_year: Option<i32>,

    #[validate(length(max = 255))]
    pub publisher: Option<String>,

    #[validate(range(min = 1))]
    pub pages: Option<i32>,

    #[validate(length(max = 50))]
    pub language: Option<String>,

    pub description: Option<String>,

    #[validate(length(max = 500), url)]
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBook {
    #[validate(length(min = 1, max = 500))]
    pub title: Option<String>,

    #[validate(length(max = 255))]
    pub author: Option<String>,

    #[validate(length(max = 50))]
    pub edition: Option<String>,

    #[validate(length(max = 17))]
    pub isbn: Option<String>,

    #[validate(range(min = 1000, max = 9999))]
    pub publication_year: Option<i32>,

    #[validate(length(max = 255))]
    pub publisher: Option<String>,

    #[validate(range(min = 1))]
    pub pages: Option<i32>,

    #[validate(length(max = 50))]
    pub language: Option<String>,

    pub description: Option<String>,

    #[validate(length(max = 500), url)]
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookQuery {
    pub search: Option<String>,
    pub author: Option<String>,
    pub year: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl Default for BookQuery {
    fn default() -> Self {
        Self {
            search: None,
            author: None,
            year: None,
            page: Some(1),
            limit: Some(20),
        }
    }
}
