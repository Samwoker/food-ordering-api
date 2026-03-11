use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Review {
    pub id:            Uuid,
    pub restaurant_id: Uuid,
    pub user_id:       Uuid,
    pub user_name:     String,
    pub rating:        i32,
    pub comment:       Option<String>,
    pub order_id:      Option<Uuid>,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReviewRequest {
    pub restaurant_id: Uuid,

    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5"))]
    pub rating: i32,

    #[validate(length(max = 1000, message = "Comment must be under 1000 characters"))]
    pub comment: Option<String>,

    pub order_id: Option<Uuid>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReviewRequest {
    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i32>,

    #[validate(length(max = 1000))]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewStats {
    pub total_reviews: i64,
    pub avg_rating:    f64,
    pub breakdown: RatingBreakdown,
}
#[derive(Debug, Serialize)]
pub struct RatingBreakdown {
    pub five:  i64,
    pub four:  i64,
    pub three: i64,
    pub two:   i64,
    pub one:   i64,
}