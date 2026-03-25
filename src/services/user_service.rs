use crate::error::AppError;
use crate::handlers::user;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::models::user::{UpdateProfileRequest, User, UserResponse};
use crate::services::auth_service::hash_password;
use actix_web::Result;
use sqlx::{PgPool, Postgres};
use tracing_subscriber::fmt::format;
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
        let taken: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)")
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

pub async fn delete_account(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let anon_email = format!("deleted_{}@removed.invalid", user_id);
    sqlx::query_as::<sqlx::Postgres, User>(
        r#"
        UPDATE users SET
            email      = $1,
            full_name  = 'Deleted User'
            password_hash = 'DELETED'
            is_blocked = true,
            is_verified = false,
            updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(anon_email)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    tracing::info!(user_id = %user_id, "Account soft-deleted");
    Ok(())
}

pub async fn change_password(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let row = sqlx::query_as::<sqlx::Postgres, User>("SELECT password FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let valid = crate::services::auth_service::verify_password(current_password, &row.password)?;
    if !valid {
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }
    let new_hash = hash_password(new_password)?;

    sqlx::query_as::<sqlx::Postgres, User>(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(new_hash)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    tracing::info!(user_id = %user_id, "Password changed");
    Ok(())
}
