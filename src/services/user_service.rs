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

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    body: UpdateProfileRequest,
) -> Result<User, AppError> {
    if let Some(ref new_email) = body.email {
        let taken: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
        )
        .bind(new_email.to_lowercase())
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if taken {
            return Err(AppError::Conflict(
                "That email is already in use".to_string(),
            ));
        }
    }
    // COALESCE preserves existing value when field is None (not sent by client)
    // When email changes, reset is_verified so user must re-confirm
    let updated = sqlx::query_as::<sqlx::Postgres, User>(
        r#"
        UPDATE users SET
            full_name  = COALESCE($1, full_name),
            email      = COALESCE($2, email),
            -- Reset verification only when email is explicitly changed
            is_verified = CASE
                WHEN $2 IS NOT NULL AND $2 != email THEN false
                ELSE is_verified
            END,
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(body.full_name)
    .bind(body.email.as_deref().map(str::to_lowercase))
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(user_id = %user_id, "Profile updated");

    Ok(updated)
}
