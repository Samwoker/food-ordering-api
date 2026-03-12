use crate::{
    config::Config,
    errors::AppError,
    models::user::{
        ForgotPasswordRequest, LoginRequest, RegisterRequest,
        ResetPasswordRequest, UpdateProfileRequest, UserResponse,
    },
    services::{
        auth_service::{
            self, AccessClaims, decode_refresh_token, generate_access_token,
            generate_email_token, generate_refresh_token, generate_reset_token,
            hash_password, revoke_refresh_token, verify_password, consume_email_token,
            consume_reset_token,
        },
    },
};

use actix_web::{web,HttpMessage,HttpRequest,HttpResponse};
use serde::Serialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize)]
pub struct AuthResponse{
    access_token:String,
    refresh_token:String,
    token_type:String,
    user:UserResponse
}

pub async fn register(
    pool: web::Data<sqlx::PgPool>,
    redis:web::Data<redis::Client>,
    config:web::Data<Config>,
    body:web::Json<RegisterRequest>    
) ->Result<HttpResponse,AppError>{
    body.validate().map_err(|e| AppError::BadRequest(e.to_string()));

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        body.email.to_lowercase()
    ).fetch_one(pool.get_ref())
    .await?
    .unwrap_or(false);

    if exists{
        return Err(AppError::Conflict("Email is already registered".to_string()));
    }
    
    let password_hash = hash_password(&body.password)?;
    let user_id = Uuid::new_v4();

    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
            INSERT INTO users(id,email,password_hash,full_name,role,is_verified)
            VALUES($1,$2,$3,$4,'Customer',false)
            RETURNING *
        "#,
        user_id,
        body.email.to_lowercase(),
        password_hash,
        body.full_name.trim(),
    ).fetch_one(pool.get_ref())
    .await?;

    //TODO:send verification email

    let email_token = generate_email_token(user_id,&redis).await?;
    let _ = email_service::send_verification_email(
        &user.email,
        &user.full_name,
        &email_token,
        &config 
    ).await?;

    let access_token = generate_access_token(user_id,&body.email,"Customer",&config)?;
    let refresh_token = generate_refresh_token(user_id,&config,&redis).await?;
    tracing::info!(user_id = %user_id,email = % body.email ,"New user registered.");
    Ok(HttpResponse::Created().json(AuthResponse{
        access_token,
        refresh_token,
        token_type:"Bearer".to_string(),
        user:UserResponse::from(user)
    }))
}

pub async fn login(
    pool:web::Data<sqlx::PgPool>,
    redis:web::Data<redis::Client>,
    config:web::Data<Config>,
    body:web::Json<LoginRequest>
)->Result<HttpResponse,AppError>{
    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE email = $1",
        body.email.to_lowercase()
    ).fetch_optional(pool.get_ref())
    .await?;

    // Constant-time path: run verify even if user not found to prevent timing attacks
    let dummy_hash = "$argon2id$v=19$m=65536,t=2,p=1$aaaaaaaaaaaaaaaaaaaaaaaa$aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let hash_to_check = user.as_ref().map(|u| u.password_hash().as_str()).unwrap_or(dummy_hash);
    let password_valid = verify_password(&body.password,hash_to_check)?;
    let user = user.filter(|_| password_valid)
    .ok_or_else(AppError::Unauthorized("Invalid credentials".to_string()))?;
    
    if user.is_blocked{
        return Err(AppError::Unauthorized(
            "Your account has been suspended. Please contact support.".to_string(),));
    }
    
    let access_token  = generate_access_token(user.id, &user.email, &user.role.to_string(), &config)?;
    let refresh_token = generate_refresh_token(user.id, &config, &redis).await?;

    tracing::info!(user_id = %user.id, "User logged in");
    Ok(HttpResponse::Ok().json(AuthResponse{
        access_token,
        refresh_token,
        token_type:"Bearer".to_string(),
        user:UserResponse::from(user)
    })) 
}

#[derive(serde::Deserialize)]
pub struct LogoutRequest{
    pub refresh_token:String,
}

pub async fn logout(
    req:HttpRequest,
    redis:web::Data<redis::Client>,
    config:web::Data<Config>,
    body:web::Json<LogoutRequest>
)->Result<HttpResponse,AppError>{
    match decode_refresh_token(&body.refresh_token,&config,&redis).await{
        Ok(claims)=>{
            revoke_refresh_token(&claims.jti,&redis).await?;
            tracing::info!(user_id = %claims.sub, "User logged out");
        },
        Err(_)=>{}
    }
    Ok(HttpResponse::ok().json(serde_json::json!({
        "message":"Logged out successfully"
    })))
}

#[derive(serde::Deserialize)]
pub struct RefreshTokenRequest{
    pub refresh_token:String
}

pub async fn refresh_token(
    pool:web::Data<sqlx::PgPool>,
    redis:web::Data<redis:Client>,
    config:web::Data<Config>,
    body:web::Json<RefreshTokenRequest>
)->Result<HttpResponse,AppError>{
    let claims = decode_refresh_token(&body.refresh_token,&config,&redis).await?;
    let user_id:Uuid = claims.sub.parse().map_err(|_| AppError::InternalError)?;
    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE id=$1",
        user_id
    ).fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|_| AppError::Unauthorized("User no longer exists".to_string()))?;

    if user.is_blocked{
        revoke_refresh_token(&claims.jti,&redis).await?;
        return Err(AppError::Unauthorized("Account has been suspended".to_string()));
    }
    revoke_refresh_token(&claims.jti,&redis).await?;
    let new_access = generate_access_token(user_id,&user.email,&user.role.to_string(),&config)?;
    let new_refresh = generate_refresh_token(user_id,&config,&redis).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "access_token":new_access,
        "refresh_toke":new_refresh,
        "token_type":"Bearer"
    })))
}

pub async fn forgot_password(
    pool:web::Data<sqlx::PgPool>,
    redis:web::Data<redis::Client>,
    config:Web::Data<Config>,
    body:web::Json<ForgotPasswordRequest>
)->Result<HttpResponse,AppError>{
    let generic_response = HttpResponse::Ok().json(serde_json::json!({
        "message":"If that email is registered , a reset link has been sent"
    }));

    let user = sqlx::query!(
        "SELECT id, full_name FROM users WHERE email=$1 AND is_blocked=false ",
        body/app-unavailable-in-region.email.to_lowercase()
    ).fetch_optional(pool.get_ref())
    .await?;

    let Some(user) = user else{
        return Ok(generic_response);
    }
    let user_id:Uuid = user.id;
    let reset_token = generate_reset_token(user_id,&redis).await?;

    let _ = email_service::send_password_reset_email(
        &user.email,
        &user.full_name,
        &reset_token,
        &config
    ).await;
    tracing::info!(user_id = %user_id, "Password reset email sent");
    Ok(generic_response)
}

pub async fn reset_password(
    pool:web::Data<sqlx::PgPool>,
    redis:web::Data<redis::Client>,
    body:web::Json<ResetPasswordRequest>
)->Result<HttpResponse,AppError>{
    body.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let user_id = consume_reset_token(&body.token,&redis).await?;
    let new_hash = password_hash(&body.new_password)?;
    let rows = sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 ",
        new_hash,
        user_id
    ).execute(pool.get_ref())
    .await?;
    .rows_affected();

    if row == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }
    tracing::info!(user_id = %user_id,"Password reset successfully");
    Ok(HttpResponse::Ok().json(serde_json::json!(
        "message":"Password updated successfully. You can now login."
    )))
}
#[derive(serde::Deserialize)]
pub struct VerifyEmailRequest{
    pub token:String
}

pub async fn verify_email(
    pool:web::Data<sqlx::PgPool>,
    redis:web::Data<redis::Client>,
    body:web::Json<VerifyEmailRequest>
)->Result<HttpResponse,AppError>{
    let user_id = consume_email_token(&body.token,&redis).await?;
    let rows = sqlx::query!(
        "UPDATE users SET is_verified = true , updated_at = NOW() WHERE id = $1",
        user_id
    ).execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    tracing::info!(user_id = %user_id, "Email verified");

    Ok(HttpResponse::Ok().json(serde_json::json!(
        "message":"Email verified successfully. You can now login."
    )))
}

pub fn claims_from_request(
    req:&HttpRequest
)->Result<AccessClaims,AppError>{
    req
    .extensions()
    .get::<AccessClaims>()
    .cloned()
    .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))
}

pub fn user_id_from_request(req:&HttpRequest)->Result<Uuid,AppError>{
    claims_from_request(req)?
    .sub
    .parse()
    .map_err(|_| AppError::InternalError)   
}