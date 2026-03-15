use crate::error::AppError;
use crate::handlers::restaurant;
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
                r.id,
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
                r.updated_at
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
    let _ = filter; // keep it for now if needed else remove
    qb.push("LIMIT").push_bind(paging.limit);
    qb.push("OFFSET").push_bind(paging.offset());

    let rows = qb.build_query_as::<Restaurant>().fetch_all(pool).await?;
    let total: i64 = sqlx::query_scalar(
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
    )
    .bind(&filter.search)
    .bind(&filter.category)
    .bind(&filter.rating)
    .fetch_one(pool)
    .await?;
    Ok(RestaurantListResult {
        restaurants: rows.into_iter().map(RestaurantSummary::from).collect(),
        total,
    })
}

pub async fn get_restaurant_by_id(pool: &PgPool, id: Uuid) -> Result<Restaurant, AppError> {
    sqlx::query_as::<sqlx::Postgres, Restaurant>(
        "SELECT * FROM restaurants WHERE id = $1  AND is_approved = true AND is_active = true",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Restaurant not found".to_string()))
}

pub async fn create_restaurant(
    pool: &PgPool,
    owner_id: Uuid,
    body: CreateRestaurantRequest,
) -> Result<Restaurant, AppError> {
    let id = Uuid::new_v4();
    let restaurant = sqlx::query_as::<sqlx::Postgres, Restaurant>(
        r#"
        INSERT INTO restaurants (id, owner_id, name, description, address, category, phone, image_url, lat, lng)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *"#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(body.name.trim())
    .bind(body.description)
    .bind(body.address.trim())
    .bind(body.category.trim())
    .bind(body.phone)
    .bind(body.image_url)
    .bind(body.lat)
    .bind(body.lng)
    .fetch_one(pool)
    .await?;
    tracing::info!(
        restaurant_id = %id,
        owner_id      = %owner_id,
        name          = %restaurant.name,
        "Restaurant created — pending admin approval"
    );

    Ok(restaurant)
}

pub async fn update_restaurant(
    pool: &PgPool,
    restaurant_id: Uuid,
    owner_id: Uuid,
    body: UpdateRestaurantRequest,
) -> Result<Restaurant, AppError> {
    verify_ownership(pool, restaurant_id, owner_id).await?;

    let restaurant = sqlx::query_as::<sqlx::Postgres, Restaurant>(
        r#"
        UPDATE restaurants
        SET
            name        = COALESCE($1, name),
            description = COALESCE($2, description),
            address     = COALESCE($3, address),
            category    = COALESCE($4, category),
            phone       = COALESCE($5, phone),
            image_url   = COALESCE($6, image_url),
            lat         = COALESCE($7, lat),
            lng         = COALESCE($8, lng),
            is_active   = COALESCE($9, is_active),
            updated_at  = NOW()
        WHERE id = $10
        RETURNING *
        "#,
    )
    .bind(body.name.as_ref().map(|s| s.trim()))
    .bind(body.description)
    .bind(body.address.as_ref().map(|s| s.trim()))
    .bind(body.category.as_ref().map(|s| s.trim()))
    .bind(body.phone)
    .bind(body.image_url)
    .bind(body.lat)
    .bind(body.lng)
    .bind(body.is_active)
    .bind(restaurant_id)
    .fetch_one(pool)
    .await?;

    Ok(restaurant)
}

async fn verify_ownership(
    pool: &PgPool,
    restaurant_id: Uuid,
    owner_id: Uuid,
) -> Result<(), AppError> {
    let row =
        sqlx::query_as::<sqlx::Postgres, Restaurant>("SELECT * FROM restaurants WHERE id = $1")
            .bind(restaurant_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Restaurant {} not found", restaurant_id)))?;

    if row.owner_id != owner_id {
        return Err(AppError::Forbidden(
            "You do not own this restaurant".to_string(),
        ));
    }
    Ok(())
}

pub async fn delete_restaurant(
    pool: &PgPool,
    restaurant_id: Uuid,
    owner_id: Uuid,
) -> Result<(), AppError> {
    verify_ownership(pool, restaurant_id, owner_id).await?;
    sqlx::query("UPDATE restaurants SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(restaurant_id)
        .execute(pool)
        .await?;
    Ok(())
}
