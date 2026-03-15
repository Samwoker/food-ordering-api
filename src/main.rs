use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, ResponseError};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cloudinary;
mod config;
mod db;
mod error;
mod handlers;
mod middlewares;
mod models;
mod paginations;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let config = config::Config::from_env();
    let bind_addr = format!("{}:{}", config.server_host, config.server_port);
    info!("The app is running on {}", bind_addr);

    let pool = db::create_pool(&config.database_url).await;
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("Failed to create redis client");

    let pool_data = web::Data::new(pool);
    let redis_data = web::Data::new(redis_client);
    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(redis_data.clone())
            .app_data(config_data.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                let response = error::AppError::BadRequest(err.to_string()).error_response();
                actix_web::error::InternalError::from_response(err, response).into()
            }))
            .wrap(Logger::default())
            .configure(routes::configure)
    })
    .bind(bind_addr)?
    .workers(4)
    .run()
    .await
}
