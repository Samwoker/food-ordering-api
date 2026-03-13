use crate::{
    error::AppError,
    handlers::auth::user_id_from_request,
    models::restaurant::{CreateRestaurantRequest, RestaurantFilter, UpdateRestaurantRequest},
    paginations::{PaginatedResponse, Pagination},
    services::restaurant_service,
};

use actix_web::{web, HttpRequest, HttpResponse};
use uuid::Uuid;
use validator::Validate;

pub async fn list_restaurants(
    pool: web::Data<sqlx::PgPool>,
    filter: web::Query<RestaurantFilter>,
    paging: web::Query<Pagination>,
) -> Result<HttpResponse, AppError> {
    let result = restaurant_service::list_restaurants(pool.get_ref(), &filter, &paging).await?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(
        result.restaurants,
        result.total,
        &paging,
    )))
}

pub async fn get_restaurant(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let restaurant =
        restaurant_service::get_restaurant_by_id(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(restaurant))
}
