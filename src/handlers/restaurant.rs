use crate::{
    config::Config,
    error::AppError,
    handlers::{auth::user_id_from_request, restaurant},
    models::restaurant::{CreateRestaurantRequest, RestaurantFilter, UpdateRestaurantRequest},
    paginations::{PaginatedResponse, Pagination},
    services::restaurant_service,
};

use crate::cloudinary::upload_image;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpResponse};
use futures_util::StreamExt;
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

pub async fn create_restaurant(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<Config>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let owner_id = user_id_from_request(&req)?;
    let mut image_bytes: Vec<u8> = Vec::new();
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(e.to_string()))?;
        let field_name = field.name().to_string();

        if field_name == "image" {
            while let Some(chunk) = field.next().await {
                image_bytes
                    .extend_from_slice(&chunk.map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
        } else {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
            fields.insert(
                field_name,
                String::from_utf8(data).map_err(|e| AppError::BadRequest(e.to_string()))?,
            );
        }
    }

    let mut create_req = CreateRestaurantRequest {
        name: fields.get("name").cloned().unwrap_or_default(),
        description: fields.get("description").cloned(),
        address: fields.get("address").cloned().unwrap_or_default(),
        category: fields.get("category").cloned().unwrap_or_default(),
        phone: fields.get("phone").cloned(),
        image_url: None,
        lat: fields.get("lat").and_then(|v| v.parse().ok()),
        lng: fields.get("lng").and_then(|v| v.parse().ok()),
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

    let restaurant =
        restaurant_service::create_restaurant(pool.get_ref(), owner_id, create_req).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "restaurant": restaurant,
        "message": "Restaurant created successfully"
    })))
}

pub async fn update_restaurant(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateRestaurantRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let owner_id = user_id_from_request(&req)?;
    let restaurant_id = path.into_inner();
    let updated = restaurant_service::update_restaurant(
        pool.get_ref(),
        restaurant_id,
        owner_id,
        body.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(updated))
}
