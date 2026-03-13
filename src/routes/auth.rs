use crate::{handlers::auth, middlewares::auth::AuthMiddleware};

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    configure_auth_routes(cfg);
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1").service(
            web::scope("/auth")
                .route("/register", web::post().to(auth::register))
                .route("/login", web::post().to(auth::login))
                .route("/refresh-token", web::post().to(auth::refresh_token))
                .route("/forgot-password", web::post().to(auth::forgot_password))
                .route("/reset-password", web::post().to(auth::reset_password))
                .route("/verify-email", web::post().to(auth::verify_email))
                .service(
                    web::scope("")
                        .wrap(AuthMiddleware)
                        .route("/logout", web::post().to(auth::logout)),
                ),
        ),
    );
}
