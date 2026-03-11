
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum NotificationType {
    OrderPlaced,
    OrderAccepted,
    OrderPreparing,
    OrderReady,
    OrderPickedUp,
    OrderDelivered,
    OrderCancelled,
    DriverAssigned,
    PaymentSucceeded,
    PaymentFailed,
    PromotionAlert,
    AccountVerified,
    PasswordChanged,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Notification {
    pub id:       Uuid,
    pub user_id:  Uuid,
    pub kind:     NotificationType,
    pub title:    String,
    pub body:     String,
    #[sqlx(json)]
    pub data:     Option<JsonValue>,

    pub is_read:  bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateNotificationPayload {
    pub user_id: Uuid,
    pub kind:    NotificationType,
    pub title:   String,
    pub body:    String,
    pub data:    Option<JsonValue>,
}
#[derive(Debug, Serialize)]
pub struct NotificationSummary {
    pub unread_count: i64,
}