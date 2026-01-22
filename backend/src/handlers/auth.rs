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
    models::user::{AuthResponse, CreateUser, LoginRequest, User, UserResponse},
};
use crate::middleware::auth::generate_jwt;

pub async fn register(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(payload): Json<CreateUser>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if username already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, created_at, updated_at FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    // Check if email already exists
    let existing_email = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, created_at, updated_at FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?;

    if existing_email.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Hash password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    // Insert user into database
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash, full_name)
         VALUES ($1, $2, $3, $4)
         RETURNING id, username, email, password_hash, full_name, created_at, updated_at"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .fetch_one(&pool)
    .await?;

    // Generate JWT tokens
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
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

pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Fetch user by username
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, created_at, updated_at FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::PasswordHash)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Authentication("Invalid credentials".to_string()))?;

    // Generate JWT tokens
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
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

    // Fetch user to ensure they still exist
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

    // Generate new tokens
    let access_claims = Claims::new_access_token(
        user.id,
        user.username.clone(),
        config.jwt_access_token_expiry,
    );
    let refresh_claims = Claims::new_refresh_token(
        user.id,
        user.username.clone(),
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
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}
