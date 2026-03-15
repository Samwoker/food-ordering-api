use crate::{
    handlers::{menu, restaurant},
    middlewares::auth::AuthMiddleware,
    middlewares::role_guard::RoleGuard,
};

use actix_web::web;

pub fn configure_restaurant_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1").service(
            web::scope("/restaurants")
                .route("/", web::get().to(restaurant::list_restaurants))
                .route("/{id}", web::get().to(restaurant::get_restaurant))
                .route("/{id}/categories", web::get().to(menu::list_categories))
                .route("/{id}/menu", web::get().to(menu::list_menu))
                .route("/{id}/menu/{menu_id}", web::get().to(menu::get_menu_item))
                .service(
                    web::scope("")
                        .wrap(AuthMiddleware)
                        .wrap(RoleGuard::any(vec!["RestaurantOwner", "Admin"]))
                        .route("/", web::post().to(restaurant::create_restaurant))
                        .route("/update/{id}", web::put().to(restaurant::update_restaurant))
                        .route(
                            "/delete/{id}",
                            web::delete().to(restaurant::delete_restaurant),
                        )
                        .route("/categories", web::post().to(menu::create_category))
                        .route(
                            "/categories/{category_id}",
                            web::delete().to(menu::delete_category),
                        )
                        .route("/menu", web::post().to(menu::create_menu_item)),
                ),
        ),
    );
}
