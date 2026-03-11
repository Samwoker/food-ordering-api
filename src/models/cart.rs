

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItem {
    pub menu_item_id:  Uuid,
    pub name:          String,
    pub unit_price:    f64,
    pub quantity:      i32,
    pub restaurant_id: Uuid,
    pub image_url:     Option<String>,
}
.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cart {
    pub items: Vec<CartItem>,
    pub restaurant_id: Option<Uuid>,
}

impl Cart {
    pub fn subtotal(&self) -> f64 {
        self.items
            .iter()
            .map(|i| i.unit_price * i.quantity as f64)
            .sum()
    }

    pub fn item_count(&self) -> i32 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn redis_key(user_id: &Uuid) -> String {
        format!("cart:{}", user_id)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddToCartRequest {
    pub menu_item_id: Uuid,

    #[validate(range(min = 1, max = 100, message = "Quantity must be between 1 and 100"))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCartItemRequest {
    #[validate(range(min = 0, max = 100, message = "Quantity must be between 0 and 100"))]
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub items:         Vec<CartItem>,
    pub restaurant_id: Option<Uuid>,
    pub subtotal:      f64,
    pub item_count:    i32,
}

impl From<Cart> for CartResponse {
    fn from(c: Cart) -> Self {
        let subtotal   = c.subtotal();
        let item_count = c.item_count();
        Self {
            restaurant_id: c.restaurant_id,
            items:         c.items,
            subtotal,
            item_count,
        }
    }
}