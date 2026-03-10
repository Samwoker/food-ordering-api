
use chrono::{DateTime,Utc};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;    

#[derive(Debug,Serialize,Deserialize,Clone,PartialEq,sqlx::Type)]
#[sqlx(type_name = "order_status")]

pub enum OrderStatus{
    Pending,
    Confirmed,
    Preparing,
    OutForDelivery,
    Delivered,
    Cancelled,
}

#[derive(Debug,Serialize,sqlx::FromRow)]
pub struct Order{
    pub id: Uuid,
    pub customer_id: Uuid,
    pub restaurant_id: Uuid,
    pub status: OrderStatus,
    pub total_price: f64,
    pub delivery_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug,Serialize,sqlx::FromRow)]

pub struct OrderItem{
    pub id:Uuid,
    pub order_id: Uuid,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}
#[derive(Debug, Deserialize, Validate)]
pub struct PlaceOrderRequest {
    pub restaurant_id: Uuid,

    #[validate(length(min = 1, message = "At least one item is required"))]
    pub items: Vec<OrderItemRequest>,

    #[validate(length(min = 5, message = "Delivery address is required"))]
    pub delivery_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderItemRequest {
    pub menu_item_id: Uuid,
    pub quantity: i32,
}