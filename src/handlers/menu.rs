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

pub async fn get_menu_item(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let menu_item = menu_service::get_menu_item(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(menu_item))
}

pub async fn create_category(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let owner_id = user_id_from_request(&req)?;
    let restaurant_id = path.into_inner();
    let category =
        menu_service::create_category(pool.get_ref(), restaurant_id, owner_id, body.into_inner())
            .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message":"Category created successfully",
        "category":category
    })))
}

pub async fn delete_category(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let owner_id = user_id_from_request(&req)?;
    let category_id = path.into_inner();
    menu_service::delete_category(pool.get_ref(), category_id, owner_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message":"Category deleted successfully",
    })))
}
