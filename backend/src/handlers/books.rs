use axum::{
    extract::{Path, Query, State},
    Json,
};
use panicless_mcp_lib::queries;
use validator::Validate;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::book::{AdvancedBookSearchQuery, Book, BookQuery, CreateBook, UpdateBook},
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

    let books = queries::search_books(
        &pool,
        claims.sub,
        query.search.as_deref(),
        query.author.as_deref(),
        query.year,
        query.sort_by.as_deref(),
        query.sort_order.as_ref(),
        Some(limit),
        Some(offset),
    )
    .await?;

    Ok(Json(books))
}

pub async fn get_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<Book>> {
    let book = queries::get_book(&pool, claims.sub, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    Ok(Json(book))
}

pub async fn create_book(
    State(pool): State<DbPool>,
    claims: Claims,
    Json(payload): Json<CreateBook>,
) -> AppResult<Json<Book>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let book = queries::create_book(
        &pool,
        claims.sub,
        &payload.title,
        payload.author.as_deref(),
        payload.edition.as_deref(),
        payload.isbn.as_deref(),
        payload.publication_year,
        payload.publisher.as_deref(),
        payload.pages,
        payload.language.as_deref(),
        payload.description.as_deref(),
        payload.cover_image_url.as_deref(),
    )
    .await?;

    Ok(Json(book))
}

pub async fn update_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
    Json(payload): Json<UpdateBook>,
) -> AppResult<Json<Book>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let book = queries::update_book(
        &pool,
        claims.sub,
        book_id,
        payload.title.as_deref(),
        payload.author.as_deref(),
        payload.edition.as_deref(),
        payload.isbn.as_deref(),
        payload.publication_year,
        payload.publisher.as_deref(),
        payload.pages,
        payload.language.as_deref(),
        payload.description.as_deref(),
        payload.cover_image_url.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    Ok(Json(book))
}

pub async fn delete_book(
    State(pool): State<DbPool>,
    Path(book_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {
    let rows = queries::delete_book(&pool, claims.sub, book_id).await?;

    if rows == 0 {
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
    queries::get_book(&pool, claims.sub, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    let readings = queries::get_book_readings(&pool, claims.sub, book_id).await?;

    Ok(Json(readings))
}

pub async fn delete_all_books(
    State(pool): State<DbPool>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {
    let (readings_deleted, books_deleted) =
        queries::delete_all_user_data(&pool, claims.sub).await?;

    Ok(Json(serde_json::json!({
        "message": "All data deleted successfully",
        "readings_deleted": readings_deleted,
        "books_deleted": books_deleted
    })))
}

pub async fn list_unread_books(
    State(pool): State<DbPool>,
    Query(query): Query<BookQuery>,
    claims: Claims,
) -> AppResult<Json<Vec<Book>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let books = queries::list_unread_books(
        &pool,
        claims.sub,
        query.search.as_deref(),
        query.author.as_deref(),
        query.year,
        query.sort_by.as_deref(),
        query.sort_order.as_ref(),
        Some(limit),
        Some(offset),
    )
    .await?;

    Ok(Json(books))
}

pub async fn advanced_search_books(
    State(pool): State<DbPool>,
    Query(query): Query<AdvancedBookSearchQuery>,
    claims: Claims,
) -> AppResult<Json<Vec<Book>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let books = queries::advanced_search_books(
        &pool,
        claims.sub,
        query.title.as_deref(),
        query.author.as_deref(),
        query.isbn.as_deref(),
        query.edition.as_deref(),
        query.publication_year,
        query.language.as_deref(),
        query.publisher.as_deref(),
        query.description.as_deref(),
        query.sort_by.as_deref(),
        query.sort_order.as_ref(),
        Some(limit),
        Some(offset),
    )
    .await?;

    Ok(Json(books))
}
