

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Driver {
    pub id: Uuid,
    pub vehicle_type:  Option<String>,   
    pub vehicle_plate: Option<String>,
    pub is_online:     bool,
    pub current_lat: Option<f64>,
    pub current_lng: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DriverWithUser {
    pub id:           Uuid,
    pub full_name:    String,
    pub email:        String,
    pub phone:        Option<String>,
    pub vehicle_type: Option<String>,
    pub is_online:    bool,
    pub current_lat:  Option<f64>,
    pub current_lng:  Option<f64>,
    pub rating:       f64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDriverRequest {
    #[validate(length(max = 20))]
    pub vehicle_type: Option<String>,

    #[validate(length(max = 20))]
    pub vehicle_plate: Option<String>,

    pub is_online: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
pub struct DriverLocation {
    pub driver_id: Uuid,
    pub lat:       f64,
    pub lng:       f64,
    pub updated_at: DateTime<Utc>,
}