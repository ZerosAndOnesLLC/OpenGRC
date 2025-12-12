use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct NotificationPayload {
    pub notification_id: uuid::Uuid,
    pub channel: NotificationChannel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Slack,
    InApp,
}

pub async fn execute(_db: &PgPool, payload: &serde_json::Value) -> Result<()> {
    let payload: NotificationPayload = serde_json::from_value(payload.clone())?;

    info!(
        notification_id = %payload.notification_id,
        channel = ?payload.channel,
        "Sending notification"
    );

    // TODO: Implement notification sending
    // 1. Load notification details from DB
    // 2. Based on channel:
    //    - Email: Send via SMTP/SES
    //    - Slack: Send via Slack webhook
    //    - InApp: Already created, just mark as sent
    // 3. Update notification status

    Ok(())
}
