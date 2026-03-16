use std::f32::consts::E;

use crate::error::AppError;
use crate::models::review::{
    CreateReviewRequest, RatingBreakdown, Review, ReviewStats, UpdateReviewRequest,
};

use crate::paginations::Pagination;
use redis::streams::StreamMaxlen;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReviewListResult {
    pub reviews: Vec<Review>,
    pub stats: ReviewStats,
    pub total: i64,
}

#[derive(sqlx::FromRow)]
struct ReviewStatsRow {
    pub total: Option<i64>,
    pub avg_rating: Option<f64>,
    pub five: Option<i64>,
    pub four: Option<i64>,
    pub three: Option<i64>,
    pub two: Option<i64>,
    pub one: Option<i64>,
}

pub async fn list_reviews(
    pool: &PgPool,
    restaurant_id: Uuid,
    paging: &Pagination,
) -> Result<ReviewListResult, AppError> {
    let reviews = sqlx::query_as::<sqlx::Postgres,Review>(
        "SELECT * FROM reviews WHERE restaurant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ).bind(restaurant_id)
    .bind(paging.limit)
    .bind(paging.offset())
    .fetch_all(pool)
    .await?;

    let stats_row = sqlx::query_as::<sqlx::Postgres, ReviewStatsRow>(
        r#"
        SELECT
            COUNT(*) as total,
            COALESCE(AVG(rating::FLOAT8),0.0) AS avg_rating,
            COUNT(*) FILTER (WHERE rating = 5) AS five,
            COUNT(*) FILTER (WHERE rating = 4) AS four,
            COUNT(*) FILTER (WHERE rating = 3) AS three,
            COUNT(*) FILTER (WHERE rating = 2) AS two,
            COUNT(*) FILTER (WHERE rating = 1) AS one
        FROM reviews
        WHERE restaurant_id = $1    
        "#,
    )
    .bind(restaurant_id)
    .fetch_one(pool)
    .await?;

    let total = stats_row.total.unwrap_or(0);
    let stats = ReviewStats {
        total_reviews: total,
        avg_rating: stats_row.avg_rating.unwrap_or(0.0),
        breakdown: RatingBreakdown {
            five: stats_row.five.unwrap_or(0),
            four: stats_row.four.unwrap_or(0),
            three: stats_row.three.unwrap_or(0),
            two: stats_row.two.unwrap_or(0),
            one: stats_row.one.unwrap_or(0),
        },
    };
    Ok(ReviewListResult {
        reviews,
        stats,
        total,
    })
}

pub async fn create_review(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    body: CreateReviewRequest,
) -> Result<Review, AppError> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM restaurants WHERE id = $1 AND is_approved = true)",
        body.restaurant_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound("Restaurant not found".to_string()));
    }

    if let Some(order_id) = body.order_id {
        let valid_order = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM orders
                WHERE id = $1
                AND customer_id = $2
                AND restaurant_id = $3
                AND status = 'Delivered'
        ) 
            "#,
            order_id,
            user_id,
            body.restaurant_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);

        if !valid_order {
            return Err(AppError::BadRequest(
                "Order not found or not yet delivered".to_string(),
            ));
        }
    }
    let review = sqlx::query_as::<sqlx::Postgres, Review>(
        "INSERT INTO reviews (id , restaurant_id,user_id,user_name,rating,comment,order_id)
         VALUES($1,$2,$3,$4,$5,$6,$7)
         RETURNING *
        ",
    )
    .bind(Uuid::new_v4())
    .bind(body.restaurant_id)
    .bind(user_id)
    .bind(name)
    .bind(body.rating)
    .bind(body.comment)
    .bind(body.order_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Conflict("You have already reviewed this restaurant".to_string()))?;

    Ok(review)
}
