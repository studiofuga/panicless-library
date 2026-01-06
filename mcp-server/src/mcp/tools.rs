use serde_json::{json, Value};
use sqlx::PgPool;

use super::protocol::{ContentItem, ToolCallResult, ToolDefinition};
use crate::queries;

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "search_books".to_string(),
            description: "Search books in user's library by title, author, or year".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "User ID to search books for"
                    },
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
                    }
                },
                "required": ["user_id"]
            }),
        },
        ToolDefinition {
            name: "get_book_details".to_string(),
            description: "Get detailed information about a specific book including all reading records".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "User ID"
                    },
                    "book_id": {
                        "type": "integer",
                        "description": "Book ID to get details for"
                    }
                },
                "required": ["user_id", "book_id"]
            }),
        },
        ToolDefinition {
            name: "list_readings".to_string(),
            description: "List reading records for a user, optionally filtered by status or year".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "User ID"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["current", "completed", "all"],
                        "description": "Filter by reading status (optional, default: all)"
                    },
                    "year": {
                        "type": "integer",
                        "description": "Filter by year (optional)"
                    }
                },
                "required": ["user_id"]
            }),
        },
        ToolDefinition {
            name: "get_reading_statistics".to_string(),
            description: "Get reading statistics for a user including books read, average rating, and yearly breakdown".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "User ID"
                    },
                    "year": {
                        "type": "integer",
                        "description": "Filter statistics by year (optional)"
                    }
                },
                "required": ["user_id"]
            }),
        },
        ToolDefinition {
            name: "find_similar_books".to_string(),
            description: "Find books by the same author or with similar attributes".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "User ID"
                    },
                    "book_id": {
                        "type": "integer",
                        "description": "Book ID to find similar books for"
                    }
                },
                "required": ["user_id", "book_id"]
            }),
        },
    ]
}

pub async fn execute_tool(
    pool: &PgPool,
    name: &str,
    arguments: Option<Value>,
) -> Result<ToolCallResult, String> {
    let args = arguments.ok_or("Missing arguments")?;

    match name {
        "search_books" => search_books(pool, args).await,
        "get_book_details" => get_book_details(pool, args).await,
        "list_readings" => list_readings(pool, args).await,
        "get_reading_statistics" => get_reading_statistics(pool, args).await,
        "find_similar_books" => find_similar_books(pool, args).await,
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

async fn search_books(pool: &PgPool, args: Value) -> Result<ToolCallResult, String> {
    let user_id = args["user_id"].as_i64().ok_or("user_id is required")? as i32;
    let query = args["query"].as_str();
    let author = args["author"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);

    let books = queries::search_books(pool, user_id, query, author, year)
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

async fn get_book_details(pool: &PgPool, args: Value) -> Result<ToolCallResult, String> {
    let user_id = args["user_id"].as_i64().ok_or("user_id is required")? as i32;
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

async fn list_readings(pool: &PgPool, args: Value) -> Result<ToolCallResult, String> {
    let user_id = args["user_id"].as_i64().ok_or("user_id is required")? as i32;
    let status = args["status"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);

    let readings = queries::list_readings(pool, user_id, status, year)
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

async fn get_reading_statistics(pool: &PgPool, args: Value) -> Result<ToolCallResult, String> {
    let user_id = args["user_id"].as_i64().ok_or("user_id is required")? as i32;

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

async fn find_similar_books(pool: &PgPool, args: Value) -> Result<ToolCallResult, String> {
    let user_id = args["user_id"].as_i64().ok_or("user_id is required")? as i32;
    let book_id = args["book_id"].as_i64().ok_or("book_id is required")? as i32;

    let similar = queries::find_similar_books(pool, user_id, book_id)
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
