use axum::{
    extract::{Path, State},
    Json,
};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use validator::Validate;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::user::{
        AdminCreateUser, AdminCreateUserResponse, AdminUpdateUser, User, UserResponse,
        UserRole, USER_COLUMNS,
    },
};

fn generate_invitation_token() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..64).map(|_| rng.gen::<u8>()).collect();
    general_purpose::URL_SAFE_NO_PAD
        .encode(&random_bytes)
        .chars()
        .take(96)
        .collect()
}

/// Create a new user (admin only). The user is created disabled with an invitation token.
pub async fn admin_create_user(
    State(pool): State<DbPool>,
    claims: Claims,
    Json(payload): Json<AdminCreateUser>,
) -> AppResult<Json<AdminCreateUserResponse>> {
    claims.require_admin()?;

    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check username uniqueness
    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(&payload.username)
        .fetch_one(&pool)
        .await?;
    if exists.0 {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    // Check email uniqueness
    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(&payload.email)
        .fetch_one(&pool)
        .await?;
    if exists.0 {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    let invitation_token = generate_invitation_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    let role = payload.role.unwrap_or(UserRole::User);

    // Password placeholder - not a valid Argon2 hash, so login is impossible
    let password_placeholder = "!INVITED!";

    let query = format!(
        "INSERT INTO users (username, email, password_hash, full_name, role, enabled, invitation_token, invitation_expires_at)
         VALUES ($1, $2, $3, $4, $5, false, $6, $7)
         RETURNING {USER_COLUMNS}"
    );
    let user = sqlx::query_as::<_, User>(&query)
        .bind(&payload.username)
        .bind(&payload.email)
        .bind(password_placeholder)
        .bind(&payload.full_name)
        .bind(&role)
        .bind(&invitation_token)
        .bind(expires_at)
        .fetch_one(&pool)
        .await?;

    Ok(Json(AdminCreateUserResponse {
        user: user.into(),
        invitation_token,
    }))
}

/// List all users (admin only).
pub async fn admin_list_users(
    State(pool): State<DbPool>,
    claims: Claims,
) -> AppResult<Json<Vec<UserResponse>>> {
    claims.require_admin()?;

    let query = format!("SELECT {USER_COLUMNS} FROM users ORDER BY id");
    let users = sqlx::query_as::<_, User>(&query)
        .fetch_all(&pool)
        .await?;

    let responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(responses))
}

/// Get a single user by ID (admin only).
pub async fn admin_get_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<UserResponse>> {
    claims.require_admin()?;

    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

/// Update a user (admin only). Can modify email, full_name, role, enabled.
pub async fn admin_update_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
    Json(payload): Json<AdminUpdateUser>,
) -> AppResult<Json<UserResponse>> {
    claims.require_admin()?;

    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Protection: prevent removing the last admin
    if let Some(ref new_role) = payload.role {
        if *new_role == UserRole::User {
            // Check if target user is currently admin
            let target: (String,) = sqlx::query_as("SELECT role::text FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&pool)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

            if target.0 == "admin" {
                let admin_count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM users WHERE role = 'admin'"
                )
                .fetch_one(&pool)
                .await?;

                if admin_count.0 <= 1 {
                    return Err(AppError::Validation(
                        "Cannot demote the last admin".to_string(),
                    ));
                }
            }
        }
    }

    // Build dynamic update query
    let mut set_clauses = vec!["updated_at = CURRENT_TIMESTAMP".to_string()];
    let mut param_index = 1u32;
    let mut binds: Vec<BindValue> = Vec::new();

    if let Some(ref email) = payload.email {
        set_clauses.push(format!("email = ${param_index}"));
        binds.push(BindValue::Str(email.clone()));
        param_index += 1;
    }
    if let Some(ref full_name) = payload.full_name {
        set_clauses.push(format!("full_name = ${param_index}"));
        binds.push(BindValue::Str(full_name.clone()));
        param_index += 1;
    }
    if let Some(ref role) = payload.role {
        set_clauses.push(format!("role = ${param_index}"));
        binds.push(BindValue::Role(role.clone()));
        param_index += 1;
    }
    if let Some(enabled) = payload.enabled {
        set_clauses.push(format!("enabled = ${param_index}"));
        binds.push(BindValue::Bool(enabled));
        param_index += 1;
    }

    let query = format!(
        "UPDATE users SET {} WHERE id = ${param_index} RETURNING {USER_COLUMNS}",
        set_clauses.join(", "),
    );

    let mut q = sqlx::query_as::<_, User>(&query);
    for bind in &binds {
        q = match bind {
            BindValue::Str(s) => q.bind(s),
            BindValue::Bool(b) => q.bind(b),
            BindValue::Role(r) => q.bind(r),
        };
    }
    q = q.bind(user_id);

    let user = q
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

enum BindValue {
    Str(String),
    Bool(bool),
    Role(UserRole),
}

/// Delete a user (admin only).
pub async fn admin_delete_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<serde_json::Value>> {
    claims.require_admin()?;

    // Cannot delete yourself
    if claims.sub == user_id {
        return Err(AppError::Validation("Cannot delete yourself".to_string()));
    }

    // Check if target is the last admin
    let target: (String,) = sqlx::query_as("SELECT role::text FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if target.0 == "admin" {
        let admin_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE role = 'admin'"
        )
        .fetch_one(&pool)
        .await?;

        if admin_count.0 <= 1 {
            return Err(AppError::Validation(
                "Cannot delete the last admin".to_string(),
            ));
        }
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

/// Resend invitation for a user that hasn't completed registration yet (admin only).
pub async fn admin_resend_invitation(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    claims: Claims,
) -> AppResult<Json<AdminCreateUserResponse>> {
    claims.require_admin()?;

    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if user.enabled {
        return Err(AppError::Validation(
            "User has already completed registration".to_string(),
        ));
    }

    let new_token = generate_invitation_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let update_query = format!(
        "UPDATE users SET invitation_token = $1, invitation_expires_at = $2 WHERE id = $3 RETURNING {USER_COLUMNS}"
    );
    let updated_user = sqlx::query_as::<_, User>(&update_query)
        .bind(&new_token)
        .bind(expires_at)
        .bind(user_id)
        .fetch_one(&pool)
        .await?;

    Ok(Json(AdminCreateUserResponse {
        user: updated_user.into(),
        invitation_token: new_token,
    }))
}
