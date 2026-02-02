use axum::{extract::{Multipart, State}, Json};
use chrono::NaiveDate;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::import::{
        BookImportResult, ImportError, ImportResponse, ImportSuccess,
        ImportSummary, TransformedBook,
    },
    services::goodreads_parser::{parse_csv_data, transform_record},
};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

pub async fn import_goodreads_csv(
    State(pool): State<DbPool>,
    claims: Claims,
    mut multipart: Multipart,
) -> AppResult<Json<ImportResponse>> {
    // Extract file from multipart form
    let file_data = extract_file_from_multipart(&mut multipart).await?;

    // Check file size
    if file_data.len() > MAX_FILE_SIZE {
        return Err(AppError::Validation(format!(
            "File too large: {} bytes (max: {} bytes)",
            file_data.len(),
            MAX_FILE_SIZE
        )));
    }

    // Pre-fetch user's existing books for duplicate detection
    let book_maps = build_book_maps(&pool, claims.sub).await?;

    // Parse CSV data
    let csv_results = parse_csv_data(file_data.as_slice())
        .map_err(|e| AppError::Validation(format!("CSV parsing failed: {}", e)))?;

    // Process each record
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    let mut books_created = 0;
    let mut books_updated = 0;
    let mut readings_created = 0;

    for (idx, result) in csv_results.into_iter().enumerate() {
        let row_number = idx + 2; // +2 because CSV has header row and arrays are 0-indexed

        let record = match result {
            Ok(rec) => rec,
            Err(e) => {
                errors.push(ImportError {
                    row_number,
                    book_title: None,
                    error: e,
                });
                continue;
            }
        };

        // Transform record
        let transformed = match transform_record(&record) {
            Ok(t) => t,
            Err(e) => {
                errors.push(ImportError {
                    row_number,
                    book_title: Some(record.title.clone()),
                    error: e,
                });
                continue;
            }
        };

        // Process book import (in individual transaction)
        match process_book_import(&pool, claims.sub, transformed, &book_maps).await {
            Ok(result) => match result {
                BookImportResult::Success {
                    book_id,
                    book_title,
                    operation,
                    reading_created,
                } => {
                    if operation == "created" {
                        books_created += 1;
                    } else {
                        books_updated += 1;
                    }
                    if reading_created {
                        readings_created += 1;
                    }
                    successes.push(ImportSuccess {
                        row_number,
                        book_id,
                        book_title,
                        operation,
                    });
                }
                BookImportResult::Failure { book_title, error } => {
                    errors.push(ImportError {
                        row_number,
                        book_title,
                        error,
                    });
                }
            },
            Err(e) => {
                errors.push(ImportError {
                    row_number,
                    book_title: Some(record.title.clone()),
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    let total_rows = successes.len() + errors.len();
    let successful_imports = successes.len();
    let failed_imports = errors.len();

    Ok(Json(ImportResponse {
        summary: ImportSummary {
            total_rows,
            successful_imports,
            failed_imports,
            books_created,
            books_updated,
            readings_created,
        },
        successes,
        errors,
    }))
}

/// Extract file data from multipart form
async fn extract_file_from_multipart(multipart: &mut Multipart) -> AppResult<Vec<u8>> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("");

        if name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file data: {}", e)))?;

            return Ok(data.to_vec());
        }
    }

    Err(AppError::Validation("No file provided in request".to_string()))
}

/// Build HashMaps for duplicate detection
async fn build_book_maps(
    pool: &DbPool,
    user_id: i32,
) -> AppResult<(HashMap<String, i32>, HashMap<(String, String), i32>)> {
    #[derive(sqlx::FromRow)]
    struct BookInfo {
        id: i32,
        isbn: Option<String>,
        title: String,
        author: Option<String>,
    }

    let books = sqlx::query_as::<_, BookInfo>(
        "SELECT id, isbn, title, author FROM books WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut isbn_map = HashMap::new();
    let mut title_author_map = HashMap::new();

    for book in books {
        // Map by ISBN if present
        if let Some(isbn) = &book.isbn {
            let clean_isbn = isbn.trim().to_lowercase();
            if !clean_isbn.is_empty() {
                isbn_map.insert(clean_isbn, book.id);
            }
        }

        // Map by title + author
        let title_key = book.title.trim().to_lowercase();
        let author_key = book
            .author
            .as_ref()
            .map(|a| a.trim().to_lowercase())
            .unwrap_or_default();
        title_author_map.insert((title_key, author_key), book.id);
    }

    Ok((isbn_map, title_author_map))
}

/// Find duplicate book ID
fn find_duplicate(
    book: &TransformedBook,
    isbn_map: &HashMap<String, i32>,
    title_author_map: &HashMap<(String, String), i32>,
) -> Option<i32> {
    // Try ISBN first
    if let Some(isbn) = &book.isbn {
        let clean_isbn = isbn.trim().to_lowercase();
        if let Some(&book_id) = isbn_map.get(&clean_isbn) {
            return Some(book_id);
        }
    }

    // Try title + author
    let title_key = book.title.trim().to_lowercase();
    let author_key = book
        .author
        .as_ref()
        .map(|a| a.trim().to_lowercase())
        .unwrap_or_default();

    title_author_map.get(&(title_key, author_key)).copied()
}

/// Process a single book import in a transaction
async fn process_book_import(
    pool: &DbPool,
    user_id: i32,
    book: TransformedBook,
    book_maps: &(HashMap<String, i32>, HashMap<(String, String), i32>),
) -> AppResult<BookImportResult> {
    let (isbn_map, title_author_map) = book_maps;

    // Check for duplicate
    let existing_book_id = find_duplicate(&book, isbn_map, title_author_map);

    let mut tx = pool.begin().await?;

    let (book_id, operation) = if let Some(existing_id) = existing_book_id {
        // Update existing book
        let updated_id = sqlx::query_scalar::<_, i32>(
            "UPDATE books
             SET title = $1, author = $2, isbn = $3, publication_year = $4,
                 publisher = $5, pages = $6, updated_at = CURRENT_TIMESTAMP
             WHERE id = $7 AND user_id = $8
             RETURNING id",
        )
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .bind(&book.publication_year)
        .bind(&book.publisher)
        .bind(&book.pages)
        .bind(existing_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        (updated_id, "updated")
    } else {
        // Insert new book
        let new_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO books (user_id, title, author, isbn, publication_year, publisher, pages)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id",
        )
        .bind(user_id)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .bind(&book.publication_year)
        .bind(&book.publisher)
        .bind(&book.pages)
        .fetch_one(&mut *tx)
        .await?;

        (new_id, "created")
    };

    // Import bookshelves for this book
    if !book.bookshelves.is_empty() {
        import_bookshelves(&mut tx, user_id, book_id, &book.bookshelves).await?;
    }

    // Handle reading creation based on shelf
    let mut reading_created = false;
    let shelf_lower = book.shelf.to_lowercase();

    match shelf_lower.as_str() {
        "read" => {
            // Create completed reading (start_date can be None)
            if let Some(end) = book.end_date {
                handle_read_reading_with_update(
                    &mut tx,
                    user_id,
                    book_id,
                    book.start_date,
                    end,
                    &book.rating,
                    &book.notes,
                )
                .await?;
                reading_created = true;
            }
        }
        "currently-reading" => {
            // Create active reading (no end_date)
            sqlx::query(
                "INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes, abandoned)
                 VALUES ($1, $2, $3, NULL, $4, $5, FALSE)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(book_id)
            .bind(book.start_date)
            .bind(&book.rating)
            .bind(&book.notes)
            .execute(&mut *tx)
            .await?;

            reading_created = true;
        }
        "abandoned" => {
            // Create abandoned reading (end_date NULL, abandoned flag TRUE)
            sqlx::query(
                "INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes, abandoned)
                 VALUES ($1, $2, $3, NULL, $4, $5, TRUE)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(book_id)
            .bind(book.start_date)
            .bind(&book.rating)
            .bind(&book.notes)
            .execute(&mut *tx)
            .await?;

            reading_created = true;
        }
        _ => {
            // "to-read" or unknown shelf - no reading record
        }
    }

    tx.commit().await?;

    Ok(BookImportResult::Success {
        book_id,
        book_title: book.title.clone(),
        operation: operation.to_string(),
        reading_created,
    })
}

/// Handle a "read" reading with duplicate detection and conditional update.
/// If a reading with a similar end_date (±7 days) exists, update only NULL rating/notes.
/// Otherwise, insert a new reading.
async fn handle_read_reading_with_update(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i32,
    book_id: i32,
    start_date: Option<NaiveDate>,
    end_date: NaiveDate,
    rating: &Option<i32>,
    notes: &Option<String>,
) -> Result<(), sqlx::Error> {
    // Look for existing reading with similar end_date (±7 days)
    let existing = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM readings
         WHERE user_id = $1 AND book_id = $2
         AND end_date IS NOT NULL
         AND ABS(EXTRACT(DAY FROM (end_date - $3::date))) <= 7
         LIMIT 1",
    )
    .bind(user_id)
    .bind(book_id)
    .bind(end_date)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(reading_id) = existing {
        // Update only NULL fields (don't overwrite existing data)
        sqlx::query(
            "UPDATE readings
             SET rating = COALESCE(rating, $1),
                 notes = COALESCE(notes, $2),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $3",
        )
        .bind(rating)
        .bind(notes)
        .bind(reading_id)
        .execute(&mut **tx)
        .await?;
    } else {
        // Insert new reading (start_date can be NULL)
        sqlx::query(
            "INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes, abandoned)
             VALUES ($1, $2, $3, $4, $5, $6, FALSE)",
        )
        .bind(user_id)
        .bind(book_id)
        .bind(start_date)
        .bind(end_date)
        .bind(rating)
        .bind(notes)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// Import bookshelves for a book (upsert shelves and create many-to-many links)
async fn import_bookshelves(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i32,
    book_id: i32,
    shelves: &[String],
) -> Result<(), sqlx::Error> {
    for shelf_name in shelves {
        // Upsert bookshelf (get or create)
        let bookshelf_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO bookshelves (user_id, name)
             VALUES ($1, $2)
             ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(user_id)
        .bind(shelf_name)
        .fetch_one(&mut **tx)
        .await?;

        // Link book to bookshelf (ignore if already linked)
        sqlx::query(
            "INSERT INTO book_bookshelves (book_id, bookshelf_id)
             VALUES ($1, $2)
             ON CONFLICT (book_id, bookshelf_id) DO NOTHING",
        )
        .bind(book_id)
        .bind(bookshelf_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
