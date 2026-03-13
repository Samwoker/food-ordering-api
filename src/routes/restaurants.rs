use crate::{
    handlers::restaurant, middlewares::auth::AuthMiddleware, middlewares::role_guard::RoleGuard,
};

use actix_web::web;

pub fn configure_restaurant_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").service(
        web::scope("/restaurants").route("/", web::get().to(restaurant::list_restaurants)),
    ));
}
