use crate::{
    config::Config,
    error::AppError,
    handlers::auth::user_id_from_request,
    models::address::{CreateAddressRequest, UpdateAddressRequest},
    models::user::{UpdateProfileRequest, UserResponse},
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
