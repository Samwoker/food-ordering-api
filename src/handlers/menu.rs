use crate::cloudinary::upload_image;
use crate::{
    config::Config,
    error::AppError,
    handlers::auth::user_id_from_request,
    models::menu_item::{CreateCategoryRequest, CreateMenuItemRequest, UpdateMenuItemRequest},
    services::menu_service,
};
use futures_util::StreamExt;

use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpResponse};

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

pub async fn create_menu_item(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    mut payload: Multipart,
    config: web::Data<Config>,
) -> Result<HttpResponse, AppError> {
    let owner_id = user_id_from_request(&req)?;
    let mut image_bytes: Vec<u8> = Vec::new();
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    while let Some(item) = payload.next().await {
        let mut field =
            item.map_err(|e: actix_multipart::MultipartError| AppError::BadRequest(e.to_string()))?;
        let field_name = field.name().to_string();
        if field_name == "image" {
            while let Some(chunk) = field.next().await {
                image_bytes.extend_from_slice(&chunk.map_err(
                    |e: actix_multipart::MultipartError| AppError::BadRequest(e.to_string()),
                )?)
            }
        } else {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(|e: actix_multipart::MultipartError| {
                    AppError::BadRequest(e.to_string())
                })?);
            }
            fields.insert(
                field_name,
                String::from_utf8(data).map_err(|e| AppError::BadRequest(e.to_string()))?,
            );
        }
    }
    let mut create_req = CreateMenuItemRequest {
        name: fields.get("name").cloned().unwrap_or_default(),
        category_id: fields
            .get("category_id")
            .and_then(|v| Uuid::parse_str(v).ok()),
        description: fields.get("description").cloned(),
        price: fields
            .get("price")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0),
        restaurant_id: path.into_inner(),
        image_url: None,
    };
    create_req
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    if !image_bytes.is_empty() {
        let image_url = upload_image(&config.cloudinary_url, image_bytes)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        create_req.image_url = Some(image_url);
    }
    let menu_item = menu_service::create_menu_item(pool.get_ref(), owner_id, create_req).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message":"Menu item created successfully",
        "menu_item":menu_item
    })))
}

pub async fn update_menu_item(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateMenuItemRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let owner_id = user_id_from_request(&req)?;
    let menu_item_id = path.into_inner();
    let update_req = body.into_inner();
    let menu_item =
        menu_service::update_menu_item(pool.get_ref(), owner_id, menu_item_id, update_req).await?;
    Ok(HttpResponse::Ok().json(menu_item))
}

pub async fn delete_menu_item(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let owner_id = user_id_from_request(&req)?;
    let item_id = path.into_inner();
    let result = menu_service::delete_menu_item(pool.get_ref(), owner_id, item_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
