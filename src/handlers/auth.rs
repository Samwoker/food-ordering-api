use crate::{
    config::Config,
    error::AppError,
    models::user::{LoginRequest, RegisterRequest, User, UserResponse},
    services::auth_service::{generate_jwt, hash_password, verify_password},
};
use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

pub async fn register(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<Config>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let existing = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        body.email
    )
    .fetch_optional(pool.get_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let password_hash = hash_password(&body.password)?;
    let user_id = Uuid::new_v4();

    let user = sqlx::query_as!(
        User,
        r#"
         INSERT INTO users (id, email, password, full_name, role)
         VALUES ($1, $2, $3, $4, 'Customer'::user_role)
         RETURNING id, email, password, full_name, role as "role: _", created_at, updated_at
        "#,
        user_id,
        body.email,
        password_hash,
        body.full_name
    )
    .fetch_one(pool.get_ref())
    .await?;

    let token = generate_jwt(user.id, user.email.clone(), user.role.to_string(), &config)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn login(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<Config>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, email, password, full_name, role as "role: _", created_at, updated_at FROM users WHERE email = $1"#,
        body.email
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid Credentials".to_string()))?;

    if !verify_password(&body.password, &user.password)? {
        return Err(AppError::Unauthorized("Invalid Credentials".to_string()));
    }

    let token = generate_jwt(user.id, user.email.clone(), user.role.to_string(), &config)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}