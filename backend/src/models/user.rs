use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
        }
    }
}

/// SQL column list for the users table, to avoid repetition in queries.
pub const USER_COLUMNS: &str = "id, username, email, password_hash, full_name, role, enabled, invitation_token, invitation_expires_at, created_at, updated_at";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub role: UserRole,
    pub enabled: bool,
    #[serde(skip_serializing)]
    pub invitation_token: Option<String>,
    #[serde(skip_serializing)]
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(max = 100))]
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub role: UserRole,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            enabled: user.enabled,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 100))]
    pub full_name: Option<String>,
}

// --- New DTOs for invitation-based registration ---

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 100))]
    pub full_name: Option<String>,

    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize)]
pub struct AdminCreateUserResponse {
    pub user: UserResponse,
    pub invitation_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CompleteRegistration {
    pub invitation_token: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(max = 100))]
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminUpdateUser {
    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 100))]
    pub full_name: Option<String>,

    pub role: Option<UserRole>,

    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePassword {
    pub current_password: String,

    #[validate(length(min = 8))]
    pub new_password: String,
}
