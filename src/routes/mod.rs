pub mod auth;
pub mod restaurants;
pub mod user;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    auth::configure_auth_routes(cfg);
    restaurants::configure_restaurant_routes(cfg);
    user::configure_user_routes(cfg);
}
