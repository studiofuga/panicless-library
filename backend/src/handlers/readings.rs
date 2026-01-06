use axum::{
    extract::{Path, Query, Request, State},
    Json,
};
use validator::Validate;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::reading::{
        CompleteReading, CreateReading, Reading, ReadingQuery, ReadingStats,
        ReadingWithBook, UpdateReading, YearStats,
    },
};

pub async fn list_readings(
    State(pool): State<DbPool>,
    Query(query): Query<ReadingQuery>,
    request: Request,
) -> AppResult<Json<Vec<ReadingWithBook>>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut sql = String::from(
        "SELECT r.*, b.title as book_title, b.author as book_author
         FROM readings r
         JOIN books b ON r.book_id = b.id
         WHERE r.user_id = $1"
    );

    let mut param_count = 2;

    if let Some(ref status) = query.status {
        match status.as_str() {
            "current" => sql.push_str(" AND r.end_date IS NULL"),
            "completed" => sql.push_str(" AND r.end_date IS NOT NULL"),
            _ => {} // "all" or invalid - no filter
        }
    }

    if query.book_id.is_some() {
        sql.push_str(&format!(" AND r.book_id = ${}", param_count));
        param_count += 1;
    }

    if query.year.is_some() {
        sql.push_str(&format!(
            " AND (EXTRACT(YEAR FROM r.start_date) = ${} OR EXTRACT(YEAR FROM r.end_date) = ${})",
            param_count, param_count
        ));
        param_count += 1;
    }

    sql.push_str(&format!(
        " ORDER BY r.start_date DESC LIMIT ${} OFFSET ${}",
        param_count,
        param_count + 1
    ));

    let mut query_builder = sqlx::query_as::<_, ReadingWithBook>(&sql).bind(claims.sub);

    if let Some(book_id) = query.book_id {
        query_builder = query_builder.bind(book_id);
    }

    if let Some(year) = query.year {
        query_builder = query_builder.bind(year);
    }

    query_builder = query_builder.bind(limit).bind(offset);

    let readings = query_builder.fetch_all(&pool).await?;

    Ok(Json(readings))
}

pub async fn get_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    request: Request,
) -> AppResult<Json<Reading>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    let reading = sqlx::query_as::<_, Reading>(
        "SELECT * FROM readings WHERE id = $1 AND user_id = $2"
    )
    .bind(reading_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    Ok(Json(reading))
}

pub async fn create_reading(
    State(pool): State<DbPool>,
    request: Request,
    Json(payload): Json<CreateReading>,
) -> AppResult<Json<Reading>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Validate end_date >= start_date if both are set
    if let Some(end_date) = payload.end_date {
        if end_date < payload.start_date {
            return Err(AppError::Validation(
                "End date must be after start date".to_string(),
            ));
        }
    }

    let reading = sqlx::query_as::<_, Reading>(
        "INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
    .bind(claims.sub)
    .bind(&payload.book_id)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .bind(&payload.rating)
    .bind(&payload.notes)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Check if this is a constraint violation (book ownership check)
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.message().contains("does not belong to user") {
                return AppError::Authorization(
                    "Cannot create reading for book that doesn't belong to you".to_string(),
                );
            }
            if db_err.constraint() == Some("idx_readings_no_overlap") {
                return AppError::Conflict(
                    "You already have an ongoing reading for this book".to_string(),
                );
            }
        }
        AppError::Database(e)
    })?;

    Ok(Json(reading))
}

pub async fn update_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    request: Request,
    Json(payload): Json<UpdateReading>,
) -> AppResult<Json<Reading>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify reading belongs to user
    let existing = sqlx::query_as::<_, Reading>(
        "SELECT * FROM readings WHERE id = $1 AND user_id = $2"
    )
    .bind(reading_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut param_count = 1;

    if payload.start_date.is_some() {
        updates.push(format!("start_date = ${}", param_count));
        param_count += 1;
    }
    if payload.end_date.is_some() {
        updates.push(format!("end_date = ${}", param_count));
        param_count += 1;
    }
    if payload.rating.is_some() {
        updates.push(format!("rating = ${}", param_count));
        param_count += 1;
    }
    if payload.notes.is_some() {
        updates.push(format!("notes = ${}", param_count));
        param_count += 1;
    }

    if updates.is_empty() {
        return Ok(Json(existing));
    }

    updates.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let sql = format!(
        "UPDATE readings SET {} WHERE id = ${} RETURNING *",
        updates.join(", "),
        param_count
    );

    let mut query_builder = sqlx::query_as::<_, Reading>(&sql);

    if let Some(start_date) = payload.start_date {
        query_builder = query_builder.bind(start_date);
    }
    if let Some(end_date) = payload.end_date {
        query_builder = query_builder.bind(end_date);
    }
    if let Some(rating) = payload.rating {
        query_builder = query_builder.bind(rating);
    }
    if let Some(notes) = payload.notes {
        query_builder = query_builder.bind(notes);
    }

    query_builder = query_builder.bind(reading_id);

    let reading = query_builder.fetch_one(&pool).await?;

    Ok(Json(reading))
}

pub async fn delete_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    request: Request,
) -> AppResult<Json<serde_json::Value>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    let result = sqlx::query("DELETE FROM readings WHERE id = $1 AND user_id = $2")
        .bind(reading_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Reading not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "Reading deleted successfully"
    })))
}

pub async fn complete_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    request: Request,
    Json(payload): Json<CompleteReading>,
) -> AppResult<Json<Reading>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify reading belongs to user and get existing data
    let existing = sqlx::query_as::<_, Reading>(
        "SELECT * FROM readings WHERE id = $1 AND user_id = $2"
    )
    .bind(reading_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    // Validate end_date >= start_date
    if payload.end_date < existing.start_date {
        return Err(AppError::Validation(
            "End date must be after start date".to_string(),
        ));
    }

    let reading = sqlx::query_as::<_, Reading>(
        "UPDATE readings SET end_date = $1, rating = $2, updated_at = CURRENT_TIMESTAMP
         WHERE id = $3
         RETURNING *"
    )
    .bind(&payload.end_date)
    .bind(&payload.rating)
    .bind(reading_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(reading))
}

pub async fn get_reading_stats(
    State(pool): State<DbPool>,
    request: Request,
) -> AppResult<Json<ReadingStats>> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No claims found".to_string()))?;

    // Total readings
    let total_readings: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM readings WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    // Completed readings
    let completed_readings: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND end_date IS NOT NULL"
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    // Current readings
    let current_readings: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND end_date IS NULL"
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    // Total books read (distinct)
    let total_books_read: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT book_id) FROM readings WHERE user_id = $1 AND end_date IS NOT NULL"
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    // Average rating
    let avg_rating: (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(rating) FROM readings WHERE user_id = $1 AND rating IS NOT NULL"
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    // Books by year (from end_date)
    let books_by_year: Vec<YearStats> = sqlx::query_as(
        "SELECT EXTRACT(YEAR FROM end_date)::INTEGER as year, COUNT(*)::BIGINT as count
         FROM readings
         WHERE user_id = $1 AND end_date IS NOT NULL
         GROUP BY EXTRACT(YEAR FROM end_date)
         ORDER BY year DESC"
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(ReadingStats {
        total_readings: total_readings.0,
        completed_readings: completed_readings.0,
        current_readings: current_readings.0,
        total_books_read: total_books_read.0,
        average_rating: avg_rating.0,
        books_by_year,
    }))
}
