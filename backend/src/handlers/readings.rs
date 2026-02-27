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
    models::reading::{
        CompleteReading, CreateReading, Reading, ReadingQuery, ReadingStats, ReadingWithBook,
        UpdateReading,
    },
};

pub async fn list_readings(
    State(pool): State<DbPool>,
    Query(query): Query<ReadingQuery>,
    claims: Claims,
) -> AppResult<Json<Vec<ReadingWithBook>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let readings = queries::list_readings(
        &pool,
        claims.sub,
        query.status.as_deref(),
        query.year,
        None, // date_from
        None, // date_to
        query.book_id,
        query.sort_by.as_deref(),
        query.sort_order.as_ref(),
        Some(limit),
        Some(offset),
    )
    .await?;

    Ok(Json(readings))
}

pub async fn get_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<Reading>> {
    let reading = queries::get_reading(&pool, claims.sub, reading_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    Ok(Json(reading))
}

pub async fn create_reading(
    State(pool): State<DbPool>,
    claims: Claims,
    Json(payload): Json<CreateReading>,
) -> AppResult<Json<Reading>> {
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

    let reading = queries::create_reading(
        &pool,
        claims.sub,
        payload.book_id,
        payload.start_date,
        payload.end_date,
        payload.rating,
        payload.notes.as_deref(),
    )
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
    claims: Claims,
    Json(payload): Json<UpdateReading>,
) -> AppResult<Json<Reading>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let reading = queries::update_reading(
        &pool,
        claims.sub,
        reading_id,
        payload.start_date,
        payload.end_date,
        payload.rating,
        payload.notes.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    Ok(Json(reading))
}

pub async fn delete_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {
    let rows = queries::delete_reading(&pool, claims.sub, reading_id).await?;

    if rows == 0 {
        return Err(AppError::NotFound("Reading not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "Reading deleted successfully"
    })))
}

pub async fn complete_reading(
    State(pool): State<DbPool>,
    Path(reading_id): Path<i32>,
    claims: Claims,
    Json(payload): Json<CompleteReading>,
) -> AppResult<Json<Reading>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify reading belongs to user and get existing data
    let existing = queries::get_reading(&pool, claims.sub, reading_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    // Validate end_date >= start_date (only if start_date is set)
    if let Some(start) = existing.start_date {
        if payload.end_date < start {
            return Err(AppError::Validation(
                "End date must be after start date".to_string(),
            ));
        }
    }

    let reading = queries::complete_reading(&pool, reading_id, payload.end_date, payload.rating)
        .await?
        .ok_or_else(|| AppError::NotFound("Reading not found".to_string()))?;

    Ok(Json(reading))
}

pub async fn get_reading_stats(
    State(pool): State<DbPool>,
    claims: Claims,
) -> AppResult<Json<ReadingStats>> {
    let stats = queries::get_reading_stats(&pool, claims.sub).await?;
    Ok(Json(stats))
}
