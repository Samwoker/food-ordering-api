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

pub async fn delete_category(
    pool: &PgPool,
    category_id: Uuid,
    owner_id: Uuid,
) -> Result<(), AppError> {
    let category = sqlx::query_as::<sqlx::Postgres, MenuCategory>(
        "SELECT restaurant_id FROM menu_categories WHERE id = $1 AND owner_id = $2",
    )
    .bind(category_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Category {} not found", category_id)))?;

    verify_restaurant_owner(pool, category.restaurant_id, owner_id).await?;

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE menu_items SET category_id = $1 WHERE category_id = $2")
        .bind(category_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM menu_categories WHERE id = $1")
        .bind(category_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
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

pub async fn create_menu_item(
    pool: &PgPool,
    owner_id: Uuid,
    body: CreateMenuItemRequest,
) -> Result<MenuItem, AppError> {
    verify_restaurant_owner(pool, body.restaurant_id, owner_id).await?;
    let id = Uuid::new_v4();
    if let Some(category_id) = body.category_id {
        let category_belongs = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM menu_categories WHERE id = $1 AND restaurant_id = $2)",
            category_id,
            body.restaurant_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);

        if !category_belongs {
            return Err(AppError::BadRequest(
                "Category does not belong to this restaurant".to_string(),
            ));
        }
    }
    let menu_item = sqlx::query_as::<sqlx::Postgres, MenuItem>(
        r#"
            INSERT INTO menu_items (id,restaurant_id,category_id,name,description,price,image_url)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
        "#,
    )
    .bind(id)
    .bind(body.restaurant_id)
    .bind(body.category_id)
    .bind(body.name.trim())
    .bind(body.description)
    .bind(body.price)
    .bind(body.image_url)
    .fetch_one(pool)
    .await?;
    Ok(menu_item)
}

pub async fn update_menu_item(
    pool: &PgPool,
    owner_id: Uuid,
    item_id: Uuid,
    body: UpdateMenuItemRequest,
) -> Result<MenuItem, AppError> {
    let item = get_menu_item(pool, item_id).await?;
    verify_restaurant_owner(pool, item.restaurant_id, owner_id).await?;
    if let Some(Some(category_id)) = &body.category_id.map(Some) {
        let category_belongs = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM menu_categories WHERE id = $1 ANd restaurant_id = $2)",
            category_id,
            item.restaurant_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);

        if !category_belongs {
            return Err(AppError::BadRequest(
                "Category does not belong to this restaurant".to_string(),
            ));
        }
    }
    let menu_item = sqlx::query_as::<sqlx::Postgres, MenuItem>(
        r#"
        UPDATE menu_items SET
            category_id  = COALESCE($1, category_id),
            name         = COALESCE($2, name),
            description  = COALESCE($3, description),
            price        = COALESCE($4, price),
            image_url    = COALESCE($5, image_url),
            is_available = COALESCE($6, is_available),
            updated_at   = NOW()
        WHERE id = $7
        RETURNING *
        "#,
    )
    .bind(body.category_id)
    .bind(body.name)
    .bind(body.description)
    .bind(body.price)
    .bind(body.image_url)
    .bind(body.is_available)
    .bind(item_id)
    .fetch_one(pool)
    .await?;
    Ok(menu_item)
}
pub async fn delete_menu_item(
    pool: &PgPool,
    owner_id: Uuid,
    item_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    let item = get_menu_item(pool, item_id).await?;
    verify_restaurant_owner(pool, item.restaurant_id, owner_id).await?;
    let in_active_order: bool = sqlx::query_scalar!(
        r#"
            SELECT EXISTS(
                SELECT 1 FROM order_items oi
                JOIN orders o ON o.id = oi.order_id
                WHERE oi.menu_item_id = $1
                AND o.status NOT IN('Delivered','Cancelled')
            )
        "#,
        item_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if in_active_order {
        sqlx::query!(
            "UPDATE menu_items SET is_available = false AND updated_at = NOW() WHERE id = $1",
            item_id
        )
        .execute(pool)
        .await?;
        return Ok(serde_json::json!({
            "deleted": false,
            "message": "Item is in an active order — marked unavailable instead of deleted."
        }));
    }
    sqlx::query!("DELETE FROM menu_items WHERE id=$1", item_id)
        .execute(pool)
        .await?;
    Ok(serde_json::json!({
        "deleted": true,
        "message": "Item deleted successfully."
    }))
}
