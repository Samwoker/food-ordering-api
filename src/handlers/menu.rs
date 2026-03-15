use crate::{
    error::AppError,
    handlers::auth::user_id_from_request,
    models::menu_item::{CreateCategoryRequest, CreateMenuItemRequest, UpdateMenuItemRequest},
    services::menu_service,
};

use actix_web::{web, HttpRequest, HttpResponse};

use uuid::Uuid;
use validator::Validate;

pub async fn list_categories(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let restaurant_id = path.into_inner();
    let categories = menu_service::list_categories(pool.get_ref(), restaurant_id).await?;
    Ok(HttpResponse::Ok().json(categories))
}

pub async fn list_menu(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let restaurant_id = path.into_inner();
    let menus = menu_service::list_menu(pool.get_ref(), restaurant_id).await?;
    Ok(HttpResponse::Ok().json(menus))
}
