
use chrono::{DateTime,Utc};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug,Serialize,sqlx::FromRow)]

pub struct MenuItem{
    pub id:Uuid,
    pub restaurant_id:Uuid,
    pub name:String,
    pub description:Option<String>,
    pub price:f64,
    pub category:String,
    pub is_available:bool,
    pub created_at:DateTime<Utc>,
    pub updated_at:DateTime<Utc>    
}

#[derive(Debug,Deserialize,Validate)]
pub struct MenuItemRequest{
    #[validate(length(min = 1  , message = "Name must be at least 1 characters."))]
    pub name:String,
    pub description:Option<String>,
    #[validate(length(min = 1 , message = "Category must be at least 1 characters."))]
    pub category:Option<String>,    
    #[validate(range(min = 0.01 , message = "Price must be greater than or equal to 0.01"))]
    pub price:f64,
}
