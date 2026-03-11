
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum UserRole {
    Customer,
    RestaurantOwner,
    Driver,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UserRole::Customer        => "Customer",
            UserRole::RestaurantOwner => "RestaurantOwner",
            UserRole::Driver          => "Driver",
            UserRole::Admin           => "Admin",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id:           Uuid,
    pub email:        String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    pub full_name:    String,
    pub role:         UserRole,
    pub is_blocked:   bool,
    pub is_verified:  bool,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 2, max = 100, message = "Full name must be 2–100 characters"))]
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, max = 100))]
    pub full_name: Option<String>,

    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id:          Uuid,
    pub email:       String,
    pub full_name:   String,
    pub role:        UserRole,
    pub is_verified: bool,
    pub created_at:  DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id:          u.id,
            email:       u.email,
            full_name:   u.full_name,
            role:        u.role,
            is_verified: u.is_verified,
            created_at:  u.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token:  String,
    pub refresh_token: String,
    pub user:          UserResponse,
}