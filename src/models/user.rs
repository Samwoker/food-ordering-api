
use chrono::{DateTime,Utc};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug,Deserialize,Serialize,Clone,sqlx::Type,PartialEq)]
#[sqlx(type_name="user_role")]

pub enum UserRole{
    Customer,
    RestaurantOwner,
    Admin
}
#[derive(Debug,Serialize,sqlx::FromRow)]
pub struct User{
    pub id:Uuid,
    pub email:String,
    #[serde(skip_serializing)]
    pub password:String,
    pub full_name:String,
    pub role:UserRole,
    pub created_at:DateTime<Utc>,
    pub updated_at:DateTime<Utc>
}

#[derive(Debug,Deserialize,Validate)]
pub struct RegisterRequest{
    #[validate(email(message="Invalid email format"))]
    pub email:String,
    #[validate(length(min = 8 , message = "Password must be at least 8 characters."))]
    pub password:String,
    #[validate(length(min = 2 , message = "Full name is required."))]
    pub full_name:String
}

#[derive(Debug,Deserialize)]
pub struct LoginRequest{
    pub email:String,
    pub password:String
}

#[derive(Debug,Serialize)]
pub struct UserResponse{
    pub id:Uuid,
    pub email:String,
    pub full_name:String,
    pub role:UserRole
}

impl From<User> for UserResponse{
    fn from(user:User) -> Self{
        Self {
            id:user.id,
            email:user.email,
            full_name:user.full_name,
            role:user.role,
        }
    }
}