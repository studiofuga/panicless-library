use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::{
    config::Config,
    crypto::TokenCrypto,
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::auth::Claims,
    models::connector::{Connector, ConnectorResponse, CreateConnectorRequest, validate_provider},
};

/// Create a new connector or update existing one
/// POST /api/connectors
pub async fn create_or_update_connector(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    claims: Claims,
    Json(payload): Json<CreateConnectorRequest>,
) -> AppResult<(StatusCode, Json<ConnectorResponse>)> {
    // Validate provider
    validate_provider(&payload.provider)
        .map_err(|e| AppError::Validation(e))?;

    // Validate token is not empty
    if payload.api_token.trim().is_empty() {
        return Err(AppError::Validation(
            "API token cannot be empty".to_string(),
        ));
    }

    // Encrypt the token
    let crypto = TokenCrypto::new(&config.encryption_key)?;
    let encrypted_token = crypto.encrypt(&payload.api_token)?;

    // Upsert in database
    let connector = sqlx::query_as::<_, Connector>(
        r#"
        INSERT INTO connectors (user_id, provider, encrypted_token, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (user_id, provider)
        DO UPDATE SET
            encrypted_token = $3,
            updated_at = NOW()
        RETURNING id, user_id, provider, encrypted_token, is_active, last_used_at, created_at, updated_at
        "#,
    )
    .bind(claims.sub)
    .bind(&payload.provider)
    .bind(&encrypted_token)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to upsert connector: {}", e);
        AppError::Internal("Failed to create/update connector".to_string())
    })?;

    Ok((StatusCode::CREATED, Json(connector.into())))
}

/// List all connectors for the authenticated user
/// GET /api/connectors
pub async fn list_connectors(
    State(pool): State<DbPool>,
    claims: Claims,
) -> AppResult<Json<Vec<ConnectorResponse>>> {
    let connectors = sqlx::query_as::<_, Connector>(
        "SELECT id, user_id, provider, encrypted_token, is_active, last_used_at, created_at, updated_at FROM connectors WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list connectors: {}", e);
        AppError::Internal("Failed to fetch connectors".to_string())
    })?;

    let responses: Vec<ConnectorResponse> = connectors.into_iter().map(|c| c.into()).collect();
    Ok(Json(responses))
}

/// Get a specific connector by provider
/// GET /api/connectors/:provider
pub async fn get_connector(
    State(pool): State<DbPool>,
    claims: Claims,
    Path(provider): Path<String>,
) -> AppResult<Json<ConnectorResponse>> {
    let connector = sqlx::query_as::<_, Connector>(
        "SELECT id, user_id, provider, encrypted_token, is_active, last_used_at, created_at, updated_at FROM connectors WHERE user_id = $1 AND provider = $2",
    )
    .bind(claims.sub)
    .bind(&provider)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch connector: {}", e);
        AppError::Internal("Failed to fetch connector".to_string())
    })?
    .ok_or_else(|| AppError::NotFound(format!("Connector '{}' not found", provider)))?;

    Ok(Json(connector.into()))
}

/// Delete a connector (soft delete - set is_active to false)
/// DELETE /api/connectors/:provider
pub async fn delete_connector(
    State(pool): State<DbPool>,
    claims: Claims,
    Path(provider): Path<String>,
) -> AppResult<StatusCode> {
    let result = sqlx::query(
        "UPDATE connectors SET is_active = false, updated_at = NOW()
         WHERE user_id = $1 AND provider = $2",
    )
    .bind(claims.sub)
    .bind(&provider)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete connector: {}", e);
        AppError::Internal("Failed to delete connector".to_string())
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Connector '{}' not found", provider)));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Toggle the active status of a connector
/// PATCH /api/connectors/:provider/toggle
pub async fn toggle_connector(
    State(pool): State<DbPool>,
    claims: Claims,
    Path(provider): Path<String>,
) -> AppResult<Json<ConnectorResponse>> {
    let connector = sqlx::query_as::<_, Connector>(
        "SELECT id, user_id, provider, encrypted_token, is_active, last_used_at, created_at, updated_at FROM connectors WHERE user_id = $1 AND provider = $2",
    )
    .bind(claims.sub)
    .bind(&provider)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch connector: {}", e);
        AppError::Internal("Failed to fetch connector".to_string())
    })?
    .ok_or_else(|| AppError::NotFound(format!("Connector '{}' not found", provider)))?;

    let updated_connector = sqlx::query_as::<_, Connector>(
        "UPDATE connectors SET is_active = NOT is_active, updated_at = NOW()
         WHERE id = $1
         RETURNING id, user_id, provider, encrypted_token, is_active, last_used_at, created_at, updated_at",
    )
    .bind(connector.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to toggle connector: {}", e);
        AppError::Internal("Failed to toggle connector".to_string())
    })?;

    Ok(Json(updated_connector.into()))
}
