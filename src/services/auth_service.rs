
use crate config::Config;
use crate error::AppError;
use argon2::{
    password_hash::{rand_core::OsRng,PasswordHash,PasswordHasher,PasswordVerifier,SaltString},
    Argon2
};

use chrone::{Duration,Utc};
use jsonwebtoken::{decode,encode,DecodingKey,EncodingKey,Header,Validation};
use serde::{Deserialize,Serialize};
use uuid::Uuid;

#[derive(Debug,Deserialize,Serialize)]

pub struct Claims{
    pub sub :String,
    pub email:String,
    pub role:String,
    pub exp : usize,
    pub iat:usize
}

pub fn password_hash(password:&str) ->Result<String,AppError>{
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
    .hash_password(password.as_bytes(),&salt)
    .map(|hash| hash.to_string())
    .map_err(|_| AppError::InternalServerError)
}

pub fn verify_password(password:&str,hash:&str) ->Result<bool,AppError>{
    let parsed_hash = PasswordHash::new(hash)
                      .map_err(|_| AppError::InternalServerError)?;
    Ok(Argon2::default()
         .verify_password(password.as_bytes(),&parsed_hash)
         .is_ok()
)                  
}

pub fn generate_jwt(
    user_id:Uuid,
    email:String,
    role:String,
    config:&Config
)->Result<String,AppError>{
    let now = Utc::now();
    let expiry = now + Duration::hours(config.jwt_expiry_hours);
    let claims = Claims{
        sub:user_id.to_string(),
        email:email.to_string(),
        role:role.to_string(),
        iat:now.timestamp() as usize,
        exp:expiry.timestamp() as usize
    }

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes())
    ).map_err(|_| AppError::InternalServerError)
}

pub fn decode_jwt(
    token:&str,
    config:&Config
)->Result<Claims,AppError>{
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()).&Validation::default()
    ).map(|data| data.claims)
    .map_err(|_| AppError::InternalServerError)
}