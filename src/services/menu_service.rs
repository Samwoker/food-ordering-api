use crate::error::AppError;
use crate::models::menu_item::{
    CreateCategoryRequest, CreateMenuItemRequest, MenuCategory, MenuItem, UpdateMenuItemRequest,
};
use crate::models::restaurant::Restaurant;

use futures_util::future::poll_fn;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub async fn list_categories(
    pool: &PgPool,
    restaurant_id: Uuid,
) -> Result<Vec<MenuCategory>, AppError> {
    let categories = sqlx::query_as::<sqlx::Postgres, MenuCategory>(
        "SELECT * FROM menu_categories WHERE restaurant_id = $1 ORDER BY sort_order ASC, name ASC",
    )
    .bind(restaurant_id)
    .fetch_all(pool)
    .await?;
    Ok(categories)
}

pub async fn list_menu(pool: &PgPool, restaurant_id: Uuid) -> Result<Vec<MenuItem>, AppError> {
    let menus = sqlx::query_as::<sqlx::Postgres, MenuItem>(
        "SELECT * FROM menu_items WHERE restaurant_id = $1 ORDER BY sort_order ASC, name ASC",
    )
    .bind(restaurant_id)
    .fetch_all(pool)
    .await?;
    Ok(menus)
}

pub async fn get_menu_item(pool: &PgPool, id: Uuid) -> Result<MenuItem, AppError> {
    let menu_item =
        sqlx::query_as::<sqlx::Postgres, MenuItem>("SELECT * FROM menu_items WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Menu item not found".to_string()))?;
    Ok(menu_item)
}

pub async fn create_category(
    pool: &PgPool,
    restaurant_id: Uuid,
    owner_id: Uuid,
    body: CreateCategoryRequest,
) -> Result<MenuCategory, AppError> {
    verify_restaurant_owner(pool, restaurant_id, owner_id).await?;
    let sort_order = match body.sort_order {
        Some(order) => order,
        None => {
            let max: Option<i32> = sqlx::query_scalar::<sqlx::Postgres, Option<i32>>(
                "SELECT MAX(sort_order) FROM menu_categories WHERE restaurant_id = $1",
            )
            .bind(restaurant_id)
            .fetch_one(pool)
            .await?;
            max.unwrap_or(0) + 10
        }
    };
    let category = sqlx::query_as::<sqlx::Postgres, MenuCategory>(
        "INSERT INTO menu_categories (id,restaurant_id , name,sort_order)
         VALUES($1,$2,$3,$4)
         RETURNING *
        ",
    )
    .bind(Uuid::new_v4())
    .bind(restaurant_id)
    .bind(body.name.trim())
    .bind(sort_order)
    .fetch_one(pool)
    .await?;
    Ok(category)
}

async fn verify_restaurant_owner(
    pool: &PgPool,
    restaurant_id: Uuid,
    owner_id: Uuid,
) -> Result<(), AppError> {
    let row =
        sqlx::query_as::<sqlx::Postgres, Restaurant>("SELECT * FROM restaurants WHERE id = $1")
            .bind(restaurant_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Restaurant not found".to_string()))?;
    if row.owner_id != owner_id {
        return Err(AppError::Forbidden(
            "You are not the owner of this restaurant".to_string(),
        ));
    }
    Ok(())
}
