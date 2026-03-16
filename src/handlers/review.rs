use crate::paginations::Pagination;
use crate::services::review_service;
use crate::{
    error::AppError,
    handlers::auth::{claims_from_request, user_id_from_request},
    models::review::{CreateReviewRequest, UpdateReviewRequest},
};
use actix_web::{body, web, HttpRequest, HttpResponse};
use uuid::Uuid;
use validator::Validate;

pub async fn list_reviews(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    paging: web::Query<Pagination>,
) -> Result<HttpResponse, AppError> {
    let result = review_service::list_reviews(pool.get_ref(), path.into_inner(), &paging).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data":result.reviews,
        "stats":result.stats,
        "meta":{
            "page":paging.page,
            "limit":paging.limit,
            "total":result.total,
            "total_pages":(result.total as f64 / paging.limit as f64).ceil() as i64
        }
    })))
}

pub async fn create_review(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<CreateReviewRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let claims = claims_from_request(&req)?;
    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let user_name = claims.email.clone();
    let full_name = sqlx::query_scalar!("SELECT full_name FROM users WHERE id = $1", user_id)
        .fetch_optional(pool.get_ref())
        .await?;
    let display_name = full_name.unwrap_or(user_name);

    let review =
        review_service::create_review(pool.get_ref(), user_id, &display_name, body.into_inner())
            .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message":"Review created successfully",
        "review":review
    })))
}
