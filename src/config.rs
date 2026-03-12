use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Server
    pub server_host: String,
    pub server_port: u16,

    // Database
    pub database_url: String,

    // Redis — used for cart storage, rate limiting, and session blacklisting
    pub redis_url: String,

    // JWT
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub jwt_refresh_secret: String,
    pub jwt_refresh_expiry_days: i64,

    // Stripe — https://stripe.com/docs/api
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,

    // Email (SMTP or SendGrid)
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,

    // App
    pub frontend_url: String, // Used in password reset emails
    pub rate_limit_per_minute: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            server_host: env_str("SERVER_HOST", "127.0.0.1"),
            server_port: env_num("SERVER_PORT", 8080),
            database_url: env_required("DATABASE_URL"),
            redis_url: env_required("REDIS_URL"),
            jwt_secret: env_required("JWT_SECRET"),
            jwt_expiry_hours: env_num("JWT_EXPIRY_HOURS", 24),
            jwt_refresh_secret: env_required("JWT_REFRESH_SECRET"),
            jwt_refresh_expiry_days: env_num("JWT_REFRESH_EXPIRY_DAYS", 30),
            stripe_secret_key: env_required("STRIPE_SECRET_KEY"),
            stripe_webhook_secret: env_required("STRIPE_WEBHOOK_SECRET"),
            smtp_host: env_str("SMTP_HOST", "smtp.sendgrid.net"),
            smtp_port: env_num("SMTP_PORT", 587),
            smtp_user: env_required("SMTP_USER"),
            smtp_password: env_required("SMTP_PASSWORD"),
            frontend_url: env_str("FRONTEND_URL", "http://localhost:3000"),
            rate_limit_per_minute: env_num("RATE_LIMIT_PER_MINUTE", 100),
        }
    }
}

fn env_required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{} must be set", key))
}

fn env_str(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_num<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
