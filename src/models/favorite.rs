
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Favorite {
    pub user_id:       Uuid,
    pub restaurant_id: Uuid,
    pub created_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FavoriteRestaurant {
    pub restaurant_id: Uuid,
    pub name:          String,
    pub category:      String,
    pub address:       String,
    pub avg_rating:    f64,
    pub image_url:     Option<String>,
    pub saved_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FavoriteActionResponse {
    pub restaurant_id: Uuid,
    pub is_favorite:   bool,
}