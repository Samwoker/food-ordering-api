use crate::{
    handlers::restaurant, middlewares::auth::AuthMiddleware, middlewares::role_guard::RoleGuard,
};

use actix_web::web;
use lettre::transport::smtp::commands::Auth;

pub fn configure_restaurant_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1").service(
            web::scope("/restaurants")
                .route("/", web::get().to(restaurant::list_restaurants))
                .route("/{id}", web::get().to(restaurant::get_restaurant))
                .service(
                    web::scope("")
                        .wrap(AuthMiddleware)
                        .wrap(RoleGuard::any(vec!["RestaurantOwner", "Admin"]))
                        .route("/", web::post().to(restaurant::create_restaurant)),
                ),
        ),
    );
}
