use crate::{
    config::{self, Config},
    error::AppError,
    handlers::auth::{self, user_id_from_request},
    models::{
        address::{CreateAddressRequest, UpdateAddressRequest},
        user::{UpdateProfileRequest, UserResponse},
    },
    paginations::Pagination,
    services::{auth_service, email_service, user_service},
};

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

pub async fn get_me(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = user_id_from_request(&req)?;
    let user = user_service::get_profile(pool.get_ref(), user_id).await?;

    let unread_notifications: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
        user_id
    )
    .fetch_one(pool.get_ref())
    .await?
    .unwrap_or(0);

    Ok(HttpResponse::Ok().json(serde_json::json!({
      "user":UserResponse::from(user),
      "unread_notification":unread_notifications
    })))
}

pub async fn update_me(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    redis: web::Data<redis::Client>,
    config: web::Data<Config>,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let user_id = user_id_from_request(&req)?;
    let email_changed = body.email.is_some();
    let user = user_service::update_profile(pool.get_ref(), user_id, body.into_inner()).await?;

    if email_changed {
        let token = auth_service::generate_email_token(user_id, &redis).await?;
        let _ =
            email_service::send_verification_email(&user.email, &user.full_name, &token, &config)
                .await;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user":    UserResponse::from(user),
        "message": if email_changed {
            "Profile updated. Please check your new email to re-verify."
        } else {
            "Profile updated."
        }
    })))
}

#[derive(Deserialize)]

pub struct DeleteAccountRequest {
    pub confirm: String,
}

pub async fn delete_me(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<DeleteAccountRequest>,
) -> Result<HttpResponse, AppError> {
    if body.confirm != "DELETE" {
        return Err(AppError::BadRequest(
            r#"Send {"confirm":"DELETE"} to confirm account deletion"#.to_string(),
        ));
    }

    let user_id = user_id_from_request(&req)?;
    user_service::delete_account(pool.get_ref(), user_id).await?;
    tracing::warn!(user_id = %user_id, "Account deleted by user request");
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Your account has been deleted. We're sorry to see you go."
    })))
}
