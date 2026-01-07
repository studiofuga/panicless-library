use axum::{
    extract::{Path, State},
    Json,
};
use validator::Validate;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::user::{UpdateUser, User, UserResponse},
};

pub async fn get_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<UserResponse>> {
    // Users can only access their own profile
    if claims.sub != user_id {
        return Err(AppError::Authorization("Access denied".to_string()));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

pub async fn update_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
    Json(payload): Json<UpdateUser>,
) -> AppResult<Json<UserResponse>> {
    if claims.sub != user_id {
        return Err(AppError::Authorization("Access denied".to_string()));
    }

    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Build dynamic update query
    let mut query = String::from("UPDATE users SET updated_at = CURRENT_TIMESTAMP");
    let mut param_count = 1;

    if payload.email.is_some() {
        query.push_str(&format!(", email = ${}", param_count));
        param_count += 1;
    }
    if payload.full_name.is_some() {
        query.push_str(&format!(", full_name = ${}", param_count));
        param_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

    let mut query_builder = sqlx::query_as::<_, User>(&query);

    if let Some(ref email) = payload.email {
        query_builder = query_builder.bind(email);
    }
    if let Some(ref full_name) = payload.full_name {
        query_builder = query_builder.bind(full_name);
    }

    query_builder = query_builder.bind(user_id);

    let user = query_builder
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {
    if claims.sub != user_id {
        return Err(AppError::Authorization("Access denied".to_string()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully"
    })))
}
