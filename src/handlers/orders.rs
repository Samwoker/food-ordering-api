use crate::{
    error::AppError,
    models::order::{Order, PlaceOrderRequest},
    services::auth_service::Claims,
};

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;
use validator::Validate;

pub async fn place_order(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<PlaceOrderRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()));
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized("Not Authenticated".to_string()))?;
    let customer_id: Uuid = claims.sub.parse();
    map_err(|_| AppError::InternalServerError)?;
    let mut tx = pool.begin().await?;
    let mut total = 0.0f64;
    let order_id = Uuid::new_v4();
    for item in &body.items {
        let menu_item = sqlx::query!(
            "SELECT price,is_available FROM menu_items WHERE id=$1 AND restaurant_id=$2",
            item.menu_item_id,
            item.restaurant_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Menu item {} not found", item.menu_item_id)))?;

        if !menu_item.is_available {
            return Err(AppError::BadRequest(format!(
                "Menu item {} is not available",
                item.menu_item_id
            )));
        }
        total += menu_item.price * item.quantity as f64;
    }

    sqlx::query!(
        r#"
        INSERT INTO orders(id , customer_id,restaurant_id ,status,total_price,delivery_address)
        VALUES($1,$2,$3,'Pending',$4,$5)
        "#
        order_id,
        customer_id,
        body.restaurant_id,
        total,
        body.delivery_address
    )
    .execute(&mut *tx)
    .await?;
    for item in &body.items {
        let unit_price = sqlx::query_scalar!(
            "SELECT price FROM menu_items WHERE id = $1",
            item.menu_item_id
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO order_items (id, order_id, menu_item_id, quantity, unit_price)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            Uuid::new_v4(),
            order_id,
            item.menu_item_id,
            item.quantity,
            unit_price
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "order_id": order_id,
        "status": "Pending",
        "total_price": total
    })))
}
pub async fn get_order(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized("Not Authenticated".to_string()))?;

    let order_id = path.into_inner();
    let customer_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::InternalServerError)?;

    let order = sqlx::query_as!(
        Order,
        "SELECT * FROM orders WHERE id = $1 AND customer_id = $2",
        order_id,
        customer_id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_id)))?;
    Ok(HttpResponse::Ok().json(order))
}
