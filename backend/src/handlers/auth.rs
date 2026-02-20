use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    Json,
};
use validator::Validate;

use crate::{
    config::Config,
    db::DbPool,
    errors::{AppError, AppResult},
    middleware::Claims,
    models::user::{
        AuthResponse, ChangePassword, CompleteRegistration, CreateUser, LoginRequest,
        User, UserResponse, USER_COLUMNS,
    },
};
use crate::middleware::auth::generate_jwt;

/// Bootstrap registration: only works when no users exist in the database.
/// Creates the first user as an enabled admin.
pub async fn bootstrap_register(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(payload): Json<CreateUser>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if any users exist
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    if count.0 > 0 {
        return Err(AppError::Authorization(
            "Registration is closed. Use an invitation to register.".to_string(),
        ));
    }

    // Hash password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    // Insert first user as enabled admin
    let query = format!(
        "INSERT INTO users (username, email, password_hash, full_name, role, enabled)
         VALUES ($1, $2, $3, $4, 'admin', true)
         RETURNING {USER_COLUMNS}"
    );
    let user = sqlx::query_as::<_, User>(&query)
        .bind(&payload.username)
        .bind(&payload.email)
        .bind(&password_hash)
        .bind(&payload.full_name)
        .fetch_one(&pool)
        .await?;

    let role = user.role.to_string();

    // Generate JWT tokens
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        role.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
        role,
        config.jwt_refresh_token_expiry,
    );

    let access_token = generate_jwt(&access_claims, &config.jwt_secret)?;
    let refresh_token = generate_jwt(&refresh_claims, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_access_token_expiry,
        user: user.into(),
    }))
}

/// Complete registration using an invitation token.
/// Sets the password and enables the user.
pub async fn complete_registration(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(payload): Json<CompleteRegistration>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Find user by invitation token
    let query = format!(
        "SELECT {USER_COLUMNS} FROM users WHERE invitation_token = $1"
    );
    let user = sqlx::query_as::<_, User>(&query)
        .bind(&payload.invitation_token)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid invitation token".to_string()))?;

    // Check if already enabled (already registered)
    if user.enabled {
        return Err(AppError::Conflict("User already registered".to_string()));
    }

    // Check token expiration
    if let Some(expires_at) = user.invitation_expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Validation("Invitation token has expired".to_string()));
        }
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    // Update user: set password, enable, clear invitation token, optionally update full_name
    let update_query = format!(
        "UPDATE users SET password_hash = $1, enabled = true, invitation_token = NULL, invitation_expires_at = NULL{}
         WHERE id = ${}
         RETURNING {USER_COLUMNS}",
        if payload.full_name.is_some() { ", full_name = $3" } else { "" },
        if payload.full_name.is_some() { "4" } else { "2" },
    );

    let mut query_builder = sqlx::query_as::<_, User>(&update_query)
        .bind(&password_hash);

    if let Some(ref full_name) = payload.full_name {
        query_builder = query_builder.bind(full_name);
    }

    let updated_user = query_builder
        .bind(user.id)
        .fetch_one(&pool)
        .await?;

    let role = updated_user.role.to_string();

    // Generate JWT tokens
    let access_claims = Claims::new_access_token(
        updated_user.id,
        updated_user.username.clone(),
        role.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        updated_user.id,
        updated_user.username.clone(),
        role,
        config.jwt_refresh_token_expiry,
    );

    let access_token = generate_jwt(&access_claims, &config.jwt_secret)?;
    let refresh_token = generate_jwt(&refresh_claims, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_access_token_expiry,
        user: updated_user.into(),
    }))
}

/// Change password for the currently authenticated user.
pub async fn change_password(
    State(pool): State<DbPool>,
    claims: Claims,
    Json(payload): Json<ChangePassword>,
) -> AppResult<Json<serde_json::Value>> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Fetch current user
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(claims.sub)
        .fetch_one(&pool)
        .await?;

    // Verify current password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::PasswordHash)?;

    Argon2::default()
        .verify_password(payload.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Authentication("Invalid current password".to_string()))?;

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_hash = argon2
        .hash_password(payload.new_password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_hash)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Fetch user by username
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE username = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

    // Check if user is enabled
    if !user.enabled {
        return Err(AppError::Authentication("Account is not enabled".to_string()));
    }

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::PasswordHash)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Authentication("Invalid credentials".to_string()))?;

    let role = user.role.to_string();

    // Generate JWT tokens
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        role.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
        role,
        config.jwt_refresh_token_expiry,
    );

    let access_token = generate_jwt(&access_claims, &config.jwt_secret)?;
    let refresh_token = generate_jwt(&refresh_claims, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_access_token_expiry,
        user: user.into(),
    }))
}

pub async fn refresh(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(refresh_token): Json<serde_json::Value>,
) -> AppResult<Json<AuthResponse>> {
    let token = refresh_token
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing refresh_token field".to_string()))?;

    let claims = crate::middleware::auth::verify_jwt(token, &config.jwt_secret)?;

    if claims.token_type != "refresh" {
        return Err(AppError::Authentication("Invalid token type".to_string()));
    }

    // Fetch user to ensure they still exist and are enabled
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

    if !user.enabled {
        return Err(AppError::Authentication("Account is not enabled".to_string()));
    }

    let role = user.role.to_string();

    // Generate new tokens (re-read role from DB)
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        role.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
        role,
        config.jwt_refresh_token_expiry,
    );

    let access_token = generate_jwt(&access_claims, &config.jwt_secret)?;
    let new_refresh_token = generate_jwt(&refresh_claims, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_access_token_expiry,
        user: user.into(),
    }))
}

pub async fn get_current_user(
    State(pool): State<DbPool>,
    claims: Claims,
) -> AppResult<Json<UserResponse>> {
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&query)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}
