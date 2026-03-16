use crate::paginations::Pagination;
use crate::services::review_service;
use crate::{
    error::AppError,
    handlers::auth::{claims_from_request, user_id_from_request},
    models::review::{CreateReviewRequest, UpdateReviewRequest},
};
use actix_web::{web, HttpRequest, HttpResponse};
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
