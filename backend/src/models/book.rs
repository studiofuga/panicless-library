use serde::Deserialize;
use validator::Validate;

use super::sort::SortOrder;

pub use panicless_mcp_lib::models::Book;

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
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for BookQuery {
    fn default() -> Self {
        Self {
            search: None,
            author: None,
            year: None,
            page: Some(1),
            limit: Some(20),
            sort_by: None,
            sort_order: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdvancedBookSearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub edition: Option<String>,
    pub publication_year: Option<i32>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for AdvancedBookSearchQuery {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            isbn: None,
            edition: None,
            publication_year: None,
            language: None,
            publisher: None,
            description: None,
            page: Some(1),
            limit: Some(20),
            sort_by: None,
            sort_order: None,
        }
    }
}
