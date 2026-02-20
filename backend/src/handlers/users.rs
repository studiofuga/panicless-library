use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::user::{User, UserResponse, USER_COLUMNS},
};

pub async fn get_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<UserResponse>> {
    // Admin can view any user, regular users can only view themselves
    claims.require_admin_or_self(user_id)?;

    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}
