use crate::config::Config;
use crate::error::AppError;

pub async fn send_verification_email(
    to_email: &str,
    to_name: &str,
    token: &str,
    config: &Config,
) -> Result<(), AppError> {
    let verify_url = format!("{}/verify-email?token={}", config.frontend_url, token);

    let html_body = format!(
        r#"
        <h2>Hi {name},</h2>
        <p>Please verify your email address by clicking the link below.</p>
        <p>This link expires in <strong>24 hours</strong>.</p>
        <p>
            <a href="{url}" style="
                background:#f97316;color:white;padding:12px 24px;
                border-radius:6px;text-decoration:none;font-weight:bold
            ">Verify Email</a>
        </p>
        <p>Or copy this URL: <code>{url}</code></p>
        "#,
        name = to_name,
        url = verify_url
    );

    send_email(to_email, "Verify your email address", &html_body, config).await
}

pub async fn send_password_reset_email(
    to_email: &str,
    to_name: &str,
    token: &str,
    config: &Config,
) -> Result<(), AppError> {
    let reset_url = format!("{}/reset-password?token={}", config.frontend_url, token);

    let html_body = format!(
        r#"
        <h2>Hi {name},</h2>
        <p>We received a request to reset your password.</p>
        <p>This link expires in <strong>15 minutes</strong>.</p>
        <p>
            <a href="{url}" style="
                background:#ef4444;color:white;padding:12px 24px;
                border-radius:6px;text-decoration:none;font-weight:bold
            ">Reset Password</a>
        </p>
        <p>If you did not request a password reset, you can safely ignore this email.</p>
        "#,
        name = to_name,
        url = reset_url
    );

    send_email(to_email, "Reset your password", &html_body, config).await
}

async fn send_email(to: &str, subject: &str, html: &str, config: &Config) -> Result<(), AppError> {
    use lettre::{
        message::{header::ContentType, Message},
        transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    };

    let email = Message::builder()
        .from(
            format!("Food Ordering <{}>", config.smtp_user)
                .parse()
                .map_err(|_| AppError::InternalServerError)?,
        )
        .to(to
            .parse()
            .map_err(|_| AppError::BadRequest(format!("Invalid email: {}", to)))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html.to_string())
        .map_err(|_| AppError::InternalServerError)?;

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
        .map_err(|_| AppError::InternalServerError)?
        .credentials(creds)
        .port(config.smtp_port)
        .build();

    mailer.send(email).await.map_err(|e| {
        tracing::error!("Failed to send email to {}: {}", to, e);
        AppError::InternalServerError
    })?;

    Ok(())
}
