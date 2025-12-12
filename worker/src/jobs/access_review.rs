use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct AccessReviewPayload {
    pub campaign_id: uuid::Uuid,
    pub action: AccessReviewAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessReviewAction {
    SendReminders,
    PullUsers,
    GenerateReport,
}

pub async fn execute(_db: &PgPool, payload: &serde_json::Value) -> Result<()> {
    let payload: AccessReviewPayload = serde_json::from_value(payload.clone())?;

    info!(
        campaign_id = %payload.campaign_id,
        action = ?payload.action,
        "Processing access review action"
    );

    match payload.action {
        AccessReviewAction::SendReminders => {
            // TODO: Send reminder notifications to reviewers
        }
        AccessReviewAction::PullUsers => {
            // TODO: Pull user list from integration and create review items
        }
        AccessReviewAction::GenerateReport => {
            // TODO: Generate access review completion report
        }
    }

    Ok(())
}
