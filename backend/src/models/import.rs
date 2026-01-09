use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents a single row from the Goodreads CSV export
#[derive(Debug, Deserialize)]
pub struct GoodreadsRecord {
    #[serde(rename = "Book Id")]
    pub book_id: String,

    #[serde(rename = "Title")]
    pub title: String,

    #[serde(rename = "Author")]
    pub author: Option<String>,

    #[serde(rename = "Author l-f")]
    pub author_lf: Option<String>,

    #[serde(rename = "Additional Authors")]
    pub additional_authors: Option<String>,

    #[serde(rename = "ISBN")]
    pub isbn: Option<String>,

    #[serde(rename = "ISBN13")]
    pub isbn13: Option<String>,

    #[serde(rename = "My Rating")]
    pub my_rating: String,

    #[serde(rename = "Average Rating")]
    pub average_rating: Option<String>,

    #[serde(rename = "Publisher")]
    pub publisher: Option<String>,

    #[serde(rename = "Binding")]
    pub binding: Option<String>,

    #[serde(rename = "Number of Pages")]
    pub number_of_pages: Option<String>,

    #[serde(rename = "Year Published")]
    pub year_published: Option<String>,

    #[serde(rename = "Original Publication Year")]
    pub original_publication_year: Option<String>,

    #[serde(rename = "Date Read")]
    pub date_read: Option<String>,

    #[serde(rename = "Date Added")]
    pub date_added: Option<String>,

    #[serde(rename = "Bookshelves")]
    pub bookshelves: Option<String>,

    #[serde(rename = "Bookshelves with positions")]
    pub bookshelves_with_positions: Option<String>,

    #[serde(rename = "Exclusive Shelf")]
    pub exclusive_shelf: String,

    #[serde(rename = "My Review")]
    pub my_review: Option<String>,

    #[serde(rename = "Spoiler")]
    pub spoiler: Option<String>,

    #[serde(rename = "Private Notes")]
    pub private_notes: Option<String>,

    #[serde(rename = "Read Count")]
    pub read_count: Option<String>,

    #[serde(rename = "Owned Copies")]
    pub owned_copies: Option<String>,
}

/// Transformed book data ready for database insertion/update
#[derive(Debug, Clone)]
pub struct TransformedBook {
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub pages: Option<i32>,
    pub shelf: String,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// Response for the import operation
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub summary: ImportSummary,
    pub successes: Vec<ImportSuccess>,
    pub errors: Vec<ImportError>,
}

/// Summary statistics for the import
#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub total_rows: usize,
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub books_created: usize,
    pub books_updated: usize,
    pub readings_created: usize,
}

/// Details of a successful import
#[derive(Debug, Serialize)]
pub struct ImportSuccess {
    pub row_number: usize,
    pub book_id: i32,
    pub book_title: String,
    pub operation: String,
}

/// Details of a failed import
#[derive(Debug, Serialize)]
pub struct ImportError {
    pub row_number: usize,
    pub book_title: Option<String>,
    pub error: String,
}

/// Result of processing a single book
#[derive(Debug)]
pub enum BookImportResult {
    Success {
        book_id: i32,
        book_title: String,
        operation: String,
        reading_created: bool,
    },
    Failure {
        book_title: Option<String>,
        error: String,
    },
}
