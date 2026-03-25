use crate::{
    handlers::user::{self, change_password, delete_me, update_me},
    middlewares::auth::AuthMiddleware,
};

use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1").service(
            web::scope("/users")
                .wrap(AuthMiddleware)
                .route("/me", web::get().to(user::get_me))
                .route("/me", web::put().to(update_me))
                .route("/me", web::delete().to(delete_me))
                .route("/me/change-password", web::post().to(change_password)),
        ),
    );
}
