
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Address {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub label:      String,
    pub address:    String,
    pub lat:        Option<f64>,
    pub lng:        Option<f64>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAddressRequest {
    #[validate(length(min = 1, max = 50, message = "Label must be 1–50 characters"))]
    pub label: String,
    #[validate(length(min = 5, max = 300, message = "Address must be 5–300 characters"))]
    pub address: String,
    pub lat:        Option<f64>,
    pub lng:        Option<f64>,
    pub is_default: Option<bool>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAddressRequest {
    #[validate(length(min = 1, max = 50))]
    pub label: Option<String>,

    #[validate(length(min = 5, max = 300))]
    pub address: Option<String>,

    pub lat:        Option<f64>,
    pub lng:        Option<f64>,
    pub is_default: Option<bool>,
}