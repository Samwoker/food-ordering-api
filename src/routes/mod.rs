use actix_web::web;
use crate::handlers::auth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
    );
}
