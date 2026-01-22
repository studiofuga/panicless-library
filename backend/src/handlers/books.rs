use axum::{
    extract::{Path, Query, State},
    Json,
};
use validator::Validate;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::book::{Book, BookQuery, CreateBook, UpdateBook},
    models::reading::Reading,
};

pub async fn list_books(
    State(pool): State<DbPool>,
    Query(query): Query<BookQuery>,
    claims: Claims,
) -> AppResult<Json<Vec<Book>>> {

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut sql = String::from(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE user_id = $1"
    );

    let mut param_count = 2;

    if query.search.is_some() {
        sql.push_str(&format!(" AND (title ILIKE ${} OR author ILIKE ${})", param_count, param_count));
        param_count += 1;
    }

    if query.author.is_some() {
        sql.push_str(&format!(" AND author ILIKE ${}", param_count));
        param_count += 1;
    }

    if query.year.is_some() {
        sql.push_str(&format!(" AND publication_year = ${}", param_count));
        param_count += 1;
    }

    sql.push_str(&format!(" ORDER BY title LIMIT ${} OFFSET ${}", param_count, param_count + 1));

    let mut query_builder = sqlx::query_as::<_, Book>(&sql).bind(claims.sub);

    if let Some(search) = query.search {
        let search_pattern = format!("%{}%", search);
        query_builder = query_builder.bind(search_pattern);
    }

    if let Some(author) = query.author {
        let author_pattern = format!("%{}%", author);
        query_builder = query_builder.bind(author_pattern);
    }

    if let Some(year) = query.year {
        query_builder = query_builder.bind(year);
    }

    query_builder = query_builder.bind(limit).bind(offset);

    let books = query_builder.fetch_all(&pool).await?;

    Ok(Json(books))
}

pub async fn get_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<Book>> {

    let book = sqlx::query_as::<_, Book>(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    Ok(Json(book))
}

pub async fn create_book(
    State(pool): State<DbPool>,
    claims: Claims,
    Json(payload): Json<CreateBook>,
) -> AppResult<Json<Book>> {

    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let book = sqlx::query_as::<_, Book>(
        "INSERT INTO books (user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at"
    )
    .bind(claims.sub)
    .bind(&payload.title)
    .bind(&payload.author)
    .bind(&payload.edition)
    .bind(&payload.isbn)
    .bind(&payload.publication_year)
    .bind(&payload.publisher)
    .bind(&payload.pages)
    .bind(&payload.language)
    .bind(&payload.description)
    .bind(&payload.cover_image_url)
    .fetch_one(&pool)
    .await?;

    Ok(Json(book))
}

pub async fn update_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
    Json(payload): Json<UpdateBook>,
) -> AppResult<Json<Book>> {

    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify book belongs to user
    let existing = sqlx::query_as::<_, Book>(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut param_count = 1;

    if payload.title.is_some() {
        updates.push(format!("title = ${}", param_count));
        param_count += 1;
    }
    if payload.author.is_some() {
        updates.push(format!("author = ${}", param_count));
        param_count += 1;
    }
    if payload.edition.is_some() {
        updates.push(format!("edition = ${}", param_count));
        param_count += 1;
    }
    if payload.isbn.is_some() {
        updates.push(format!("isbn = ${}", param_count));
        param_count += 1;
    }
    if payload.publication_year.is_some() {
        updates.push(format!("publication_year = ${}", param_count));
        param_count += 1;
    }
    if payload.publisher.is_some() {
        updates.push(format!("publisher = ${}", param_count));
        param_count += 1;
    }
    if payload.pages.is_some() {
        updates.push(format!("pages = ${}", param_count));
        param_count += 1;
    }
    if payload.language.is_some() {
        updates.push(format!("language = ${}", param_count));
        param_count += 1;
    }
    if payload.description.is_some() {
        updates.push(format!("description = ${}", param_count));
        param_count += 1;
    }
    if payload.cover_image_url.is_some() {
        updates.push(format!("cover_image_url = ${}", param_count));
        param_count += 1;
    }

    if updates.is_empty() {
        return Ok(Json(existing));
    }

    updates.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let sql = format!(
        "UPDATE books SET {} WHERE id = ${} RETURNING id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at",
        updates.join(", "),
        param_count
    );

    let mut query_builder = sqlx::query_as::<_, Book>(&sql);

    if let Some(title) = payload.title {
        query_builder = query_builder.bind(title);
    }
    if let Some(author) = payload.author {
        query_builder = query_builder.bind(author);
    }
    if let Some(edition) = payload.edition {
        query_builder = query_builder.bind(edition);
    }
    if let Some(isbn) = payload.isbn {
        query_builder = query_builder.bind(isbn);
    }
    if let Some(publication_year) = payload.publication_year {
        query_builder = query_builder.bind(publication_year);
    }
    if let Some(publisher) = payload.publisher {
        query_builder = query_builder.bind(publisher);
    }
    if let Some(pages) = payload.pages {
        query_builder = query_builder.bind(pages);
    }
    if let Some(language) = payload.language {
        query_builder = query_builder.bind(language);
    }
    if let Some(description) = payload.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(cover_image_url) = payload.cover_image_url {
        query_builder = query_builder.bind(cover_image_url);
    }

    query_builder = query_builder.bind(book_id);

    let book = query_builder.fetch_one(&pool).await?;

    Ok(Json(book))
}

pub async fn delete_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {

    let result = sqlx::query(
        "DELETE FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id)
    .bind(claims.sub)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Book not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "Book deleted successfully"
    })))
}

pub async fn get_book_readings(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<Vec<Reading>>> {

    // Verify book belongs to user
    let _ = sqlx::query_as::<_, Book>(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    let readings = sqlx::query_as::<_, Reading>(
        "SELECT id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at FROM readings WHERE book_id = $1 AND user_id = $2 ORDER BY start_date DESC"
    )
    .bind(book_id)
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(readings))
}
