pub mod auth;
pub mod restaurants;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    auth::configure_auth_routes(cfg);
    restaurants::configure_restaurant_routes(cfg);
}
