use crate::config::Config;
use crate::error::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    pub sub: String,
    pub email: String,
    pub role: String,

    pub kind: String,

    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: String,
    pub jti: String,
    pub kind: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Uses Argon2id variant by default

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::InternalServerError)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::InternalServerError)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    config: &Config,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiry = now + Duration::hours(config.jwt_expiry_hours);

    let claims = AccessClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        kind: "access".to_string(),
        iat: now.timestamp() as usize,
        exp: expiry.timestamp() as usize,
    };

    encode(
        &Header::default(), // HS256
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError)
}

pub fn decode_access_token(token: &str, config: &Config) -> Result<AccessClaims, AppError> {
    let claims = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized("Invalid or expired access token".to_string()))?;

    if claims.kind != "access" {
        return Err(AppError::Unauthorized("Wrong token type".to_string()));
    }

    Ok(claims)
}
fn refresh_token_key(jti: &str) -> String {
    format!("refresh_token:{}", jti)
}

pub async fn generate_refresh_token(
    user_id: Uuid,
    config: &Config,
    redis: &redis::Client,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiry = now + Duration::days(config.jwt_refresh_expiry_days);
    let jti = Uuid::new_v4().to_string(); // Unique token ID

    let claims = RefreshClaims {
        sub: user_id.to_string(),
        jti: jti.clone(),
        kind: "refresh".to_string(),
        iat: now.timestamp() as usize,
        exp: expiry.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_refresh_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError)?;

    let mut conn = redis.get_async_connection().await?;
    let ttl_secs = (config.jwt_refresh_expiry_days * 86_400) as usize;
    conn.set_ex::<_, _, ()>(
        refresh_token_key(&jti),
        user_id.to_string(),
        ttl_secs as u64,
    )
    .await?;

    Ok(token)
}
pub async fn decode_refresh_token(
    token: &str,
    config: &Config,
    redis: &redis::Client,
) -> Result<RefreshClaims, AppError> {
    let claims = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_refresh_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    if claims.kind != "refresh" {
        return Err(AppError::Unauthorized("Wrong token type".to_string()));
    }
    let mut conn = redis.get_async_connection().await?;
    let exists: bool = conn.exists(refresh_token_key(&claims.jti)).await?;

    if !exists {
        return Err(AppError::Unauthorized(
            "Refresh token has been revoked. Please log in again.".to_string(),
        ));
    }

    Ok(claims)
}

pub async fn revoke_refresh_token(jti: &str, redis: &redis::Client) -> Result<(), AppError> {
    let mut conn = redis.get_async_connection().await?;
    conn.del::<_, ()>(refresh_token_key(jti)).await?;
    Ok(())
}

const EMAIL_TOKEN_TTL: usize = 60 * 60 * 24; // 24 hours in seconds

fn email_verify_key(token: &str) -> String {
    format!("email_verify:{}", token)
}

pub async fn generate_email_token(
    user_id: Uuid,
    redis: &redis::Client,
) -> Result<String, AppError> {
    let token: String = (0..32)
        .map(|_| format!("{:02x}", rand::thread_rng().gen::<u8>()))
        .collect();

    let mut conn = redis.get_async_connection().await?;
    conn.set_ex::<_, _, ()>(
        email_verify_key(&token),
        user_id.to_string(),
        EMAIL_TOKEN_TTL as u64,
    )
    .await?;

    Ok(token)
}
pub async fn consume_email_token(token: &str, redis: &redis::Client) -> Result<Uuid, AppError> {
    let mut conn = redis.get_async_connection().await?;
    let user_id_str: Option<String> = conn.get_del(email_verify_key(token)).await?;

    let user_id_str = user_id_str
        .ok_or_else(|| AppError::BadRequest("Invalid or expired verification token".to_string()))?;

    Uuid::parse_str(&user_id_str).map_err(|_| AppError::InternalServerError)
}

const RESET_TOKEN_TTL: usize = 60 * 15; // 15 minutes — short window for security

fn password_reset_key(token: &str) -> String {
    format!("pwd_reset:{}", token)
}

/// Generate a short-lived password reset token (15 min TTL).
pub async fn generate_reset_token(
    user_id: Uuid,
    redis: &redis::Client,
) -> Result<String, AppError> {
    let token: String = (0..32)
        .map(|_| format!("{:02x}", rand::thread_rng().gen::<u8>()))
        .collect();

    let mut conn = redis.get_async_connection().await?;
    conn.set_ex::<_, _, ()>(
        password_reset_key(&token),
        user_id.to_string(),
        RESET_TOKEN_TTL as u64,
    )
    .await?;

    Ok(token)
}

/// Validate a reset token and return the associated user_id.
/// Consumes the token — single use only.
pub async fn consume_reset_token(token: &str, redis: &redis::Client) -> Result<Uuid, AppError> {
    let mut conn = redis.get_async_connection().await?;

    let user_id_str: Option<String> = conn.get_del(password_reset_key(token)).await?;

    let user_id_str = user_id_str
        .ok_or_else(|| AppError::BadRequest("Invalid or expired reset token".to_string()))?;

    Uuid::parse_str(&user_id_str).map_err(|_| AppError::InternalServerError)
}
