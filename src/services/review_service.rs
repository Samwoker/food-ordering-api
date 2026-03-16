use crate::error::AppError;
use crate::models::review::{
    CreateReviewRequest, RatingBreakdown, Review, ReviewStats, UpdateReviewRequest,
};

use crate::paginations::Pagination;
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
