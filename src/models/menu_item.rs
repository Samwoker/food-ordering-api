
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
#[derive(Debug, Serialize, FromRow)]
pub struct MenuCategory {
    pub id:            Uuid,
    pub restaurant_id: Uuid,
    pub name:          String,
    pub sort_order:    i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100, message = "Category name is required"))]
    pub name: String,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MenuItem {
    pub id:            Uuid,
    pub restaurant_id: Uuid,
    pub category_id:   Option<Uuid>,
    pub name:         String,
    pub description:  Option<String>,
    pub price:        f64,
    pub image_url:    Option<String>,
    pub is_available: bool,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMenuItemRequest {
    pub restaurant_id: Uuid,
    pub category_id:   Option<Uuid>,

    #[validate(length(min = 1, max = 200, message = "Item name is required"))]
    pub name: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price: f64,

    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMenuItemRequest {
    pub category_id: Option<Uuid>,

    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(range(min = 0.01))]
    pub price: Option<f64>,

    pub image_url:    Option<String>,
    pub is_available: Option<bool>,
}