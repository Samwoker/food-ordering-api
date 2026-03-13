use crate::error::AppError;
use crate::models::restaurant::{
    CreateRestaurantRequest, Restaurant, RestaurantFilter, RestaurantSummary,
    UpdateRestaurantRequest,
};
use crate::paginations::Pagination;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct RestaurantListResult {
    pub restaurants: Vec<RestaurantSummary>,
    pub total: i64,
}

pub async fn list_restaurants(
    pool: &PgPool,
    filter: &RestaurantFilter,
    paging: &Pagination,
) -> Result<RestaurantListResult, AppError> {
    use sqlx::QueryBuilder;
    let mut qb = QueryBuilder::new(
        r#"
            SELECT
                r.id
                r.name,
                r.description,
                r.address,
                r.category,
                r.phone,
                r.image_url,
                r.avg_rating,
                r.lat,
                r.lng,
                r.is_active,
                r.is_approved,
                r.owner_id,
                r.created_at,
                r.updated_at,
            FROM restaurants r
            WHERE  r.is_approved = true
            AND r.is_active = true

        "#,
    );
    if let Some(ref search) = filter.search {
        qb.push("AND (r.name ILIKE")
            .push_bind(format!("%{}%", search))
            .push("OR r.description ILIKE")
            .push_bind(format!("%{}%", search))
            .push(")");
    }

    if let Some(ref category) = filter.category {
        qb.push("AND r.category = ").push_bind(category.clone());
    }
    if let Some(rating) = filter.rating {
        qb.push("AND r.avg_rating >= ").push_bind(rating);
    }
    if let (Some(lat), Some(lng), Some(radius_km)) = (filter.lat, filter.lng, filter.radius) {
        qb.push(
            r#"
            AND (
                6371 * acos(
                    cos(radians(#lat#)) * cos(radians(r.lat)) *
                    cos(radians(r.lng) - radians(#lng#)) +
                    sin(radians(#lat#)) * sin(radians(r.lat))
                )
            ) <= #radius#
            "#
            .replace("#lat#", &lat.to_string())
            .replace("#lng#", &lng.to_string())
            .replace("#radius#", &radius_km.to_string()),
        );
    }

    match filter.sort.as_deref() {
        Some("rating") => qb.push("ORDER BY r.avg_rating DESC"),
        Some("newest") => qb.push("ORDER BY r.created_at DESC"),
        _ => qb.push("ORDER BY r.created_at DESC"),
    };
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) AS sub",
        qb.sql().replace("SELECT\n            r.id,\n            r.name,\n            r.description,\n            r.address,\n            r.category,\n            r.phone,\n            r.image_url,\n            r.avg_rating,\n            r.lat,\n            r.lng,\n            r.is_active,\n            r.is_approved,\n            r.owner_id,\n            r.created_at,\n            r.updated_at", "SELECT 1")
    );
    qb.push("LIMIT").push_bind(paging.limit);
    qb.push("OFFSET").push_bind(paging.offset());

    let rows = qb.build_query_as::<Restaurant>().fetch_all(pool).await?;
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM restaurants
        WHERE is_approved = true
          AND is_active = true
          AND ($1::TEXT IS NULL OR name    ILIKE '%' || $1 || '%'
                                OR description ILIKE '%' || $1 || '%')
          AND ($2::TEXT IS NULL OR category = $2)
          AND ($3::FLOAT8 IS NULL OR avg_rating >= $3)
        "#,
        filter.search,
        filter.category,
        filter.rating,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    Ok(RestaurantListResult {
        restaurants: rows.into_iter().map(RestaurantSummary::from).collect(),
        total,
    })
}
