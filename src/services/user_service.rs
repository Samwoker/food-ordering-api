use crate::error::AppError;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::models::user::{UpdateProfileRequest, User, UserResponse};
use crate::services::auth_service::hash_password;
use actix_web::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    sqlx::query_as::<sqlx::Postgres, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
}
