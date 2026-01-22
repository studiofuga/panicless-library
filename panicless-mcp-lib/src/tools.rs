use serde_json::{json, Value};
use sqlx::PgPool;

use super::protocol::{ContentItem, ToolCallResult, ToolDefinition};
use crate::queries;

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "search_books".to_string(),
            description: "Search books in user's library by title, author, or year with pagination support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query for title or author (optional)"
                    },
                    "author": {
                        "type": "string",
                        "description": "Filter by author (optional)"
                    },
                    "year": {
                        "type": "integer",
                        "description": "Filter by publication year (optional)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 100, max: 500)",
                        "minimum": 1,
                        "maximum": 500
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Number of results to skip for pagination (default: 0)",
                        "minimum": 0
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_book_details".to_string(),
            description: "Get detailed information about a specific book including all reading records".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "book_id": {
                        "type": "integer",
                        "description": "Book ID to get details for"
                    }
                },
                "required": ["book_id"]
            }),
        },
        ToolDefinition {
            name: "list_readings".to_string(),
            description: "List reading records for a user, optionally filtered by status or year, with pagination support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["current", "completed", "all"],
                        "description": "Filter by reading status (optional, default: all)"
                    },
                    "year": {
                        "type": "integer",
                        "description": "Filter by year (optional)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 100, max: 500)",
                        "minimum": 1,
                        "maximum": 500
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Number of results to skip for pagination (default: 0)",
                        "minimum": 0
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_reading_statistics".to_string(),
            description: "Get reading statistics for a user including books read, average rating, and yearly breakdown".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "year": {
                        "type": "integer",
                        "description": "Filter statistics by year (optional)"
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "find_similar_books".to_string(),
            description: "Find books by the same author or with similar attributes, with pagination support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "book_id": {
                        "type": "integer",
                        "description": "Book ID to find similar books for (required)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 50, max: 500)",
                        "minimum": 1,
                        "maximum": 500
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Number of results to skip for pagination (default: 0)",
                        "minimum": 0
                    }
                },
                "required": ["book_id"]
            }),
        },
        ToolDefinition {
            name: "create_book".to_string(),
            description: "Create a new book in the user's library".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Book title (required)"
                    },
                    "author": {
                        "type": "string",
                        "description": "Author name (optional)"
                    },
                    "isbn": {
                        "type": "string",
                        "description": "ISBN (optional)"
                    },
                    "publication_year": {
                        "type": "integer",
                        "description": "Year of publication (optional)"
                    },
                    "publisher": {
                        "type": "string",
                        "description": "Publisher name (optional)"
                    },
                    "pages": {
                        "type": "integer",
                        "description": "Number of pages (optional)"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language of the book (optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Book description (optional)"
                    }
                },
                "required": ["title"]
            }),
        },
        ToolDefinition {
            name: "create_reading".to_string(),
            description: "Create a new reading record for a book".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "book_id": {
                        "type": "integer",
                        "description": "Book ID to create a reading record for (required)"
                    },
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYY-MM-DD format (required)"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYY-MM-DD format (optional)"
                    }
                },
                "required": ["book_id", "start_date"]
            }),
        },
        ToolDefinition {
            name: "update_reading_review".to_string(),
            description: "Add or update a review/comment and rating for a reading record".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reading_id": {
                        "type": "integer",
                        "description": "Reading record ID (required)"
                    },
                    "rating": {
                        "type": "integer",
                        "description": "Rating from 1 to 5 (optional)"
                    },
                    "notes": {
                        "type": "string",
                        "description": "Review/comment notes (optional)"
                    }
                },
                "required": ["reading_id"]
            }),
        },
    ]
}

pub async fn execute_tool(
    pool: &PgPool,
    name: &str,
    arguments: Option<Value>,
    user_id: i32,
) -> Result<ToolCallResult, String> {
    let args = arguments.unwrap_or(serde_json::json!({}));

    match name {
        "search_books" => search_books(pool, args, user_id).await,
        "get_book_details" => get_book_details(pool, args, user_id).await,
        "list_readings" => list_readings(pool, args, user_id).await,
        "get_reading_statistics" => get_reading_statistics(pool, args, user_id).await,
        "find_similar_books" => find_similar_books(pool, args, user_id).await,
        "create_book" => create_book(pool, args, user_id).await,
        "create_reading" => create_reading(pool, args, user_id).await,
        "update_reading_review" => update_reading_review(pool, args, user_id).await,
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

async fn search_books(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let query = args["query"].as_str();
    let author = args["author"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let books = queries::search_books(pool, user_id, query, author, year, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if books.is_empty() {
        "No books found matching your criteria.".to_string()
    } else {
        let mut result = format!("Found {} book(s):\n\n", books.len());
        for (i, book) in books.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} by {}\n   Published: {}, Pages: {}\n   ISBN: {}\n\n",
                i + 1,
                book.title,
                book.author.as_deref().unwrap_or("Unknown"),
                book.publication_year.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string()),
                book.pages.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string()),
                book.isbn.as_deref().unwrap_or("N/A")
            ));
        }
        result
    };

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn get_book_details(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let book_id = args["book_id"].as_i64().ok_or("book_id is required")? as i32;

    let book = queries::get_book_with_readings(pool, user_id, book_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Book not found".to_string())?;

    let text = format!(
        "Book Details:\n\nTitle: {}\nAuthor: {}\nEdition: {}\nISBN: {}\nPublished: {} by {}\nPages: {}\nLanguage: {}\n\nDescription:\n{}\n\nReading History ({} time(s) read):\n{}",
        book.title,
        book.author.as_deref().unwrap_or("Unknown"),
        book.edition.as_deref().unwrap_or("N/A"),
        book.isbn.as_deref().unwrap_or("N/A"),
        book.publication_year.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string()),
        book.publisher.as_deref().unwrap_or("Unknown"),
        book.pages.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string()),
        book.language.as_deref().unwrap_or("N/A"),
        book.description.as_deref().unwrap_or("No description available"),
        book.reading_count,
        book.readings_summary.as_deref().unwrap_or("Never read")
    );

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn list_readings(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let status = args["status"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let readings = queries::list_readings(pool, user_id, status, year, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if readings.is_empty() {
        "No readings found.".to_string()
    } else {
        let mut result = format!("Found {} reading(s):\n\n", readings.len());
        for (i, reading) in readings.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} by {}\n   Started: {}, Finished: {}\n   Rating: {}\n   Notes: {}\n\n",
                i + 1,
                reading.book_title,
                reading.book_author.as_deref().unwrap_or("Unknown"),
                reading.start_date,
                reading.end_date.map(|d| d.to_string()).unwrap_or_else(|| "Still reading".to_string()),
                reading.rating.map(|r| format!("{}/5", r)).unwrap_or_else(|| "Not rated".to_string()),
                reading.notes.as_deref().unwrap_or("No notes")
            ));
        }
        result
    };

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn get_reading_statistics(pool: &PgPool, _args: Value, user_id: i32) -> Result<ToolCallResult, String> {

    let stats = queries::get_reading_stats(pool, user_id)
        .await
        .map_err(|e| e.to_string())?;

    let text = format!(
        "Reading Statistics:\n\nTotal Readings: {}\nCompleted Readings: {}\nCurrently Reading: {}\nUnique Books Read: {}\nAverage Rating: {}\n\nBooks by Year:\n{}",
        stats.total_readings,
        stats.completed_readings,
        stats.current_readings,
        stats.total_books_read,
        stats.average_rating.map(|r| format!("{:.1}/5", r)).unwrap_or_else(|| "No ratings yet".to_string()),
        if stats.books_by_year.is_empty() {
            "No completed readings yet".to_string()
        } else {
            stats.books_by_year.iter()
                .map(|(year, count)| format!("  {}: {} books", year, count))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn find_similar_books(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let book_id = args["book_id"].as_i64().ok_or("book_id is required")? as i32;
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let similar = queries::find_similar_books(pool, user_id, book_id, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if similar.is_empty() {
        "No similar books found in your library.".to_string()
    } else {
        let mut result = format!("Found {} similar book(s):\n\n", similar.len());
        for (i, book) in similar.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} by {}\n   Published: {}\n\n",
                i + 1,
                book.title,
                book.author.as_deref().unwrap_or("Unknown"),
                book.publication_year.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string())
            ));
        }
        result
    };

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn create_book(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let title = args["title"].as_str().ok_or("title is required")?;
    let author = args["author"].as_str();
    let isbn = args["isbn"].as_str();
    let publication_year = args["publication_year"].as_i64().map(|y| y as i32);
    let publisher = args["publisher"].as_str();
    let pages = args["pages"].as_i64().map(|p| p as i32);
    let language = args["language"].as_str();
    let description = args["description"].as_str();

    let book_id = queries::insert_book(
        pool,
        user_id,
        title,
        author,
        isbn,
        publication_year,
        publisher,
        pages,
        language,
        description,
    )
    .await
    .map_err(|e| e.to_string())?;

    let text = format!(
        "Book created successfully!\n\nBook ID: {}\nTitle: {}\nAuthor: {}\nISBN: {}\nPages: {}\n\nYou can now create reading records for this book.",
        book_id,
        title,
        author.unwrap_or("N/A"),
        isbn.unwrap_or("N/A"),
        pages.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string())
    );

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn create_reading(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    use chrono::NaiveDate;
    let book_id = args["book_id"].as_i64().ok_or("book_id is required")? as i32;
    let start_date_str = args["start_date"].as_str().ok_or("start_date is required")?;
    let end_date_str = args["end_date"].as_str();

    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
        .map_err(|_| "start_date must be in YYYY-MM-DD format".to_string())?;

    let end_date = if let Some(date_str) = end_date_str {
        Some(
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| "end_date must be in YYYY-MM-DD format".to_string())?,
        )
    } else {
        None
    };

    let reading_id = queries::insert_reading(pool, user_id, book_id, start_date, end_date)
        .await
        .map_err(|e| e.to_string())?;

    let text = format!(
        "Reading record created successfully!\n\nReading ID: {}\nBook ID: {}\nStart Date: {}\nEnd Date: {}\n\nYou can now add a review and rating to this reading record.",
        reading_id,
        book_id,
        start_date,
        end_date.map(|d| d.to_string()).unwrap_or_else(|| "Not finished".to_string())
    );

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    })
}

async fn update_reading_review(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let reading_id = args["reading_id"].as_i64().ok_or("reading_id is required")? as i32;
    let rating = args["rating"].as_i64().map(|r| r as i32);
    let notes = args["notes"].as_str();

    // Validate rating if provided
    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err("Rating must be between 1 and 5".to_string());
        }
    }

    let updated = queries::update_reading_review(pool, user_id, reading_id, rating, notes)
        .await
        .map_err(|e| e.to_string())?;

    let text = if updated {
        format!(
            "Reading review updated successfully!\n\nReading ID: {}\nRating: {}\nNotes: {}",
            reading_id,
            rating.map(|r| r.to_string()).unwrap_or_else(|| "Not set".to_string()),
            notes.unwrap_or("Not set")
        )
    } else {
        "Reading record not found or you don't have permission to update it.".to_string()
    };

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: if updated { None } else { Some(true) },
    })
}
