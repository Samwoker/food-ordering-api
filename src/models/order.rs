
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Preparing,
    Ready,
    PickedUp,
    Delivered,
    Cancelled,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Order {
    pub id:                  Uuid,
    pub customer_id:         Uuid,
    pub restaurant_id:       Uuid,

    pub driver_id:           Option<Uuid>,

    pub status:              OrderStatus,
    pub total_price:         f64,

    pub delivery_address:    String,
    pub delivery_lat:        Option<f64>,
    pub delivery_lng:        Option<f64>,
    pub cancellation_reason: Option<String>,

    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct OrderItem {
    pub id:           Uuid,
    pub order_id:     Uuid,
    pub menu_item_id: Uuid,

    pub name:         String,

    pub unit_price:   f64,

    pub quantity:     i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PlaceOrderRequest {
    pub address_id: Uuid,
    #[validate(length(min = 1, message = "Payment method is required"))]
    pub payment_method: String,
    #[validate(length(max = 500))]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CancelOrderRequest {
    #[validate(length(max = 500))]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub order: Order,
    pub items: Vec<OrderItem>,
}