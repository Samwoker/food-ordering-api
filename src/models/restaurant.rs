
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Restaurant {
    pub id:          Uuid,
    pub owner_id:    Uuid,
    pub name:        String,
    pub description: Option<String>,
    pub address:     String,
    pub category:    String,
    pub phone:       Option<String>,
    pub image_url:   Option<String>,
    pub avg_rating:  f64,
    pub is_active:   bool,
    pub is_approved: bool,
    pub lat:         Option<f64>,
    pub lng:         Option<f64>,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRestaurantRequest {
    #[validate(length(min = 2, max = 150, message = "Name must be 2–150 characters"))]
    pub name: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(min = 5, max = 300, message = "Address is required"))]
    pub address: String,

    #[validate(length(min = 1, max = 100, message = "Category is required"))]
    pub category: String,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    pub image_url: Option<String>,
    pub lat:       Option<f64>,
    pub lng:       Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRestaurantRequest {
    #[validate(length(min = 2, max = 150))]
    pub name: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(min = 5, max = 300))]
    pub address: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub category: Option<String>,

    pub phone:     Option<String>,
    pub image_url: Option<String>,
    pub lat:       Option<f64>,
    pub lng:       Option<f64>,
    pub is_active: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct RestaurantFilter {
    pub search:   Option<String>,
    pub category: Option<String>,
    pub rating:   Option<f64>,
    pub lat:    Option<f64>,
    pub lng:    Option<f64>,
    pub radius: Option<f64>,  
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RestaurantSummary {
    pub id:         Uuid,
    pub name:       String,
    pub category:   String,
    pub address:    String,
    pub avg_rating: f64,
    pub image_url:  Option<String>,
}

impl From<Restaurant> for RestaurantSummary {
    fn from(r: Restaurant) -> Self {
        Self {
            id:         r.id,
            name:       r.name,
            category:   r.category,
            address:    r.address,
            avg_rating: r.avg_rating,
            image_url:  r.image_url,
        }
    }
}