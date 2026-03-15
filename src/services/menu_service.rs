use crate::error::AppError;
use crate::models::menu_item::{
    CreateCategoryRequest, CreateMenuItemRequest, MenuCategory, MenuItem, UpdateMenuItemRequest,
};

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
