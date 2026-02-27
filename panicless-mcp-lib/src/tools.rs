use serde_json::{json, Value};
use sqlx::PgPool;

use super::protocol::{ContentItem, ToolCallResult, ToolDefinition};
use crate::queries;

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "get_today".to_string(),
            description: "Get today's date, current time (UTC), and day of the week. Call this before creating readings or filtering by date to know the current date. The day_of_week field returns the English name (e.g. Monday, Tuesday).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
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
            name: "advanced_search_books".to_string(),
            description: "Advanced search for books using multiple filter criteria: title, author, ISBN, edition, publication year, language, publisher, and description with pagination support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Search in book title (case-insensitive partial match, optional)"
                    },
                    "author": {
                        "type": "string",
                        "description": "Filter by author name (case-insensitive partial match, optional)"
                    },
                    "isbn": {
                        "type": "string",
                        "description": "Filter by exact ISBN number (optional)"
                    },
                    "edition": {
                        "type": "string",
                        "description": "Filter by edition (case-insensitive partial match, optional)"
                    },
                    "publication_year": {
                        "type": "integer",
                        "description": "Filter by exact publication year (optional)",
                        "minimum": 1000,
                        "maximum": 9999
                    },
                    "language": {
                        "type": "string",
                        "description": "Filter by language (case-insensitive partial match, optional)"
                    },
                    "publisher": {
                        "type": "string",
                        "description": "Filter by publisher (case-insensitive partial match, optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Search in book description (case-insensitive partial match, optional)"
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
            description: "List reading records for a user, optionally filtered by status, year, or date range, with pagination support".to_string(),
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
                    "start_date": {
                        "type": "string",
                        "description": "Filter readings started on or after this date, in YYYY-MM-DD format (optional)"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "Filter readings completed on or before this date, in YYYY-MM-DD format (optional)"
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
                        "description": "Book ID to find similar books for (provide this or book_title)"
                    },
                    "book_title": {
                        "type": "string",
                        "description": "Book title to search for (alternative to book_id). If multiple books match, an error with the list of matches is returned."
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
                "required": []
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
                        "description": "Book ID to create a reading record for (provide this or book_title)"
                    },
                    "book_title": {
                        "type": "string",
                        "description": "Book title to search for (alternative to book_id). If multiple books match, an error with the list of matches is returned."
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
                "required": ["start_date"]
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
        "get_today" => Ok(get_today()),
        "search_books" => search_books(pool, args, user_id).await,
        "advanced_search_books" => advanced_search_books(pool, args, user_id).await,
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

fn get_today() -> ToolCallResult {
    let now = chrono::Utc::now();
    let text = format!(
        "Today's date: {}\nCurrent time (UTC): {}\nDay of the week: {}",
        now.format("%Y-%m-%d"),
        now.format("%H:%M:%S"),
        now.format("%A"),
    );
    ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: None,
    }
}

async fn search_books(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let query = args["query"].as_str();
    let author = args["author"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let books = queries::search_books(pool, user_id, query, author, year, None, None, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if books.is_empty() {
        "No books found matching your criteria.".to_string()
    } else {
        let mut result = format!("Found {} book(s):\n\n", books.len());
        for (i, book) in books.iter().enumerate() {
            result.push_str(&format!(
                "{}. [ID: {}] {} by {}\n   Published: {}, Pages: {}\n   ISBN: {}\n\n",
                i + 1,
                book.id,
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

async fn advanced_search_books(pool: &PgPool, args: Value, user_id: i32) -> Result<ToolCallResult, String> {
    let title = args["title"].as_str();
    let author = args["author"].as_str();
    let isbn = args["isbn"].as_str();
    let edition = args["edition"].as_str();
    let publication_year = args["publication_year"].as_i64().map(|y| y as i32);
    let language = args["language"].as_str();
    let publisher = args["publisher"].as_str();
    let description = args["description"].as_str();
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let books = queries::advanced_search_books(
        pool,
        user_id,
        title,
        author,
        isbn,
        edition,
        publication_year,
        language,
        publisher,
        description,
        None,
        None,
        limit,
        offset,
    )
    .await
    .map_err(|e| e.to_string())?;

    let text = if books.is_empty() {
        "No books found matching your search criteria.".to_string()
    } else {
        let mut result = format!("Found {} book(s):\n\n", books.len());
        for (i, book) in books.iter().enumerate() {
            let mut book_info = format!(
                "{}. [ID: {}] {} by {}\n   Published: {}, Pages: {}\n   ISBN: {}\n",
                i + 1,
                book.id,
                book.title,
                book.author.as_deref().unwrap_or("Unknown"),
                book.publication_year.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string()),
                book.pages.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string()),
                book.isbn.as_deref().unwrap_or("N/A")
            );

            if let Some(edition) = &book.edition {
                book_info.push_str(&format!("   Edition: {}\n", edition));
            }

            if let Some(language) = &book.language {
                book_info.push_str(&format!("   Language: {}\n", language));
            }

            if let Some(publisher) = &book.publisher {
                book_info.push_str(&format!("   Publisher: {}\n", publisher));
            }

            if let Some(description) = &book.description {
                if description.len() > 150 {
                    book_info.push_str(&format!("   Description: {}...\n", &description[..150]));
                } else {
                    book_info.push_str(&format!("   Description: {}\n", description));
                }
            }

            book_info.push('\n');
            result.push_str(&book_info);
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
        .ok_or_else(|| format!("Book with ID {} not found. Use the search_books tool to find the correct book ID.", book_id))?;

    let text = format!(
        "Book Details:\n\nBook ID: {}\nTitle: {}\nAuthor: {}\nEdition: {}\nISBN: {}\nPublished: {} by {}\nPages: {}\nLanguage: {}\n\nDescription:\n{}\n\nReading History ({} time(s) read):\n{}",
        book.id,
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
    use chrono::NaiveDate;

    let status = args["status"].as_str();
    let year = args["year"].as_i64().map(|y| y as i32);
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    let date_from = if let Some(s) = args["start_date"].as_str() {
        Some(NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| "start_date must be in YYYY-MM-DD format".to_string())?)
    } else {
        None
    };

    let date_to = if let Some(s) = args["end_date"].as_str() {
        Some(NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| "end_date must be in YYYY-MM-DD format".to_string())?)
    } else {
        None
    };

    let readings = queries::list_readings(pool, user_id, status, year, date_from, date_to, None, None, None, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if readings.is_empty() {
        "No readings found.".to_string()
    } else {
        let mut result = format!("Found {} reading(s):\n\n", readings.len());
        for (i, reading) in readings.iter().enumerate() {
            result.push_str(&format!(
                "{}. [Reading ID: {}, Book ID: {}] {} by {}\n   Started: {}, Finished: {}\n   Rating: {}\n   Notes: {}\n\n",
                i + 1,
                reading.id,
                reading.book_id,
                reading.book_title,
                reading.book_author.as_deref().unwrap_or("Unknown"),
                reading.start_date.map(|d| d.to_string()).unwrap_or_else(|| "Unknown".to_string()),
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
                .map(|ys| format!("  {}: {} books", ys.year, ys.count))
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
    let book_id = match resolve_book_id(pool, &args, user_id).await {
        Ok(id) => id,
        Err(err_result) => return Ok(err_result),
    };
    let limit = args["limit"].as_i64();
    let offset = args["offset"].as_i64();

    // Validate that the book exists
    let exists = queries::book_exists(pool, user_id, book_id)
        .await
        .map_err(|e| e.to_string())?;
    if !exists {
        return Ok(ToolCallResult {
            content: vec![ContentItem::Text {
                text: format!(
                    "Book with ID {} not found. Use the search_books tool to find the correct book ID.",
                    book_id
                ),
            }],
            is_error: Some(true),
        });
    }

    let similar = queries::find_similar_books(pool, user_id, book_id, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let text = if similar.is_empty() {
        "No similar books found in your library.".to_string()
    } else {
        let mut result = format!("Found {} similar book(s):\n\n", similar.len());
        for (i, book) in similar.iter().enumerate() {
            result.push_str(&format!(
                "{}. [ID: {}] {} by {}\n   Published: {}\n\n",
                i + 1,
                book.id,
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

/// Resolve a book_id from either an explicit book_id or a book_title search.
/// Returns the resolved book_id or a ToolCallResult error.
async fn resolve_book_id(
    pool: &PgPool,
    args: &Value,
    user_id: i32,
) -> Result<i32, ToolCallResult> {
    let book_id = args["book_id"].as_i64().map(|id| id as i32);
    let book_title = args["book_title"].as_str();

    match (book_id, book_title) {
        (Some(id), _) => Ok(id),
        (None, Some(title)) => {
            let matches = queries::find_book_by_title(pool, user_id, title)
                .await
                .map_err(|e| ToolCallResult {
                    content: vec![ContentItem::Text { text: e.to_string() }],
                    is_error: Some(true),
                })?;

            match matches.len() {
                0 => Err(ToolCallResult {
                    content: vec![ContentItem::Text {
                        text: format!(
                            "No book found with title '{}'. Use search_books to find the correct book.",
                            title
                        ),
                    }],
                    is_error: Some(true),
                }),
                1 => Ok(matches[0].id),
                _ => {
                    let list = matches
                        .iter()
                        .map(|b| {
                            format!(
                                "  - [ID: {}] {} by {}",
                                b.id,
                                b.title,
                                b.author.as_deref().unwrap_or("Unknown")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Err(ToolCallResult {
                        content: vec![ContentItem::Text {
                            text: format!(
                                "Multiple books match '{}':\n{}\nPlease specify the book_id directly.",
                                title, list
                            ),
                        }],
                        is_error: Some(true),
                    })
                }
            }
        }
        (None, None) => Err(ToolCallResult {
            content: vec![ContentItem::Text {
                text: "Either book_id or book_title must be provided.".to_string(),
            }],
            is_error: Some(true),
        }),
    }
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
    let book_id = match resolve_book_id(pool, &args, user_id).await {
        Ok(id) => id,
        Err(err_result) => return Ok(err_result),
    };
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
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("foreign key") || msg.contains("violates") {
                format!("Book with ID {} not found. Use search_books to find the correct book ID.", book_id)
            } else {
                msg
            }
        })?;

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
        format!("Reading with ID {} not found. Use list_readings to find the correct reading ID.", reading_id)
    };

    Ok(ToolCallResult {
        content: vec![ContentItem::Text { text }],
        is_error: if updated { None } else { Some(true) },
    })
}
