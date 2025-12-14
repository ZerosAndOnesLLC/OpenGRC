use crate::cache::CacheClient;
use crate::config::Config;
use crate::utils::{AppError, AppResult};
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client as SesClient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationService {
    db: PgPool,
    cache: CacheClient,
    ses_client: Option<SesClient>,
    from_email: String,
    app_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: serde_json::Value,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct EmailTemplate {
    id: Uuid,
    template_type: String,
    subject: String,
    body_html: String,
    body_text: String,
}

#[derive(Debug, Clone)]
pub struct PolicyReminderData {
    pub policy_id: Uuid,
    pub policy_code: String,
    pub policy_title: String,
    pub policy_version: i32,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub organization_name: String,
}

impl NotificationService {
    pub async fn new(db: PgPool, cache: CacheClient, config: &Config) -> Self {
        let ses_client = if config.is_production() {
            // In production, initialize SES client
            let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            Some(SesClient::new(&aws_config))
        } else {
            None
        };

        Self {
            db,
            cache,
            ses_client,
            from_email: std::env::var("SES_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@opengrc.com".to_string()),
            app_url: std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }

    // ==================== In-App Notifications ====================

    /// Create a new in-app notification
    pub async fn create_notification(
        &self,
        org_id: Uuid,
        input: CreateNotification,
    ) -> AppResult<Notification> {
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (organization_id, user_id, notification_type, title, message, data)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, user_id, notification_type, title, message, data, read_at, created_at
            "#,
        )
        .bind(org_id)
        .bind(input.user_id)
        .bind(&input.notification_type)
        .bind(&input.title)
        .bind(&input.message)
        .bind(input.data.unwrap_or_else(|| serde_json::json!({})))
        .fetch_one(&self.db)
        .await?;

        Ok(notification)
    }

    /// Get all notifications for a user
    pub async fn get_user_notifications(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        unread_only: bool,
        limit: i64,
    ) -> AppResult<Vec<Notification>> {
        let notifications = if unread_only {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT id, organization_id, user_id, notification_type, title, message, data, read_at, created_at
                FROM notifications
                WHERE organization_id = $1 AND user_id = $2 AND read_at IS NULL
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(org_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT id, organization_id, user_id, notification_type, title, message, data, read_at, created_at
                FROM notifications
                WHERE organization_id = $1 AND user_id = $2
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(org_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        };

        Ok(notifications)
    }

    /// Get unread notification count
    pub async fn get_unread_count(&self, org_id: Uuid, user_id: Uuid) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE organization_id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, org_id: Uuid, user_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications SET read_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND user_id = $3 AND read_at IS NULL
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(&self, org_id: Uuid, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications SET read_at = NOW()
            WHERE organization_id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ==================== Email Notifications ====================

    /// Send a policy reminder email
    pub async fn send_policy_reminder_email(
        &self,
        org_id: Uuid,
        data: &PolicyReminderData,
        template_type: &str,
    ) -> AppResult<()> {
        // Get email template (org-specific or default)
        let template = self.get_email_template(org_id, template_type).await?;

        // Render template with data
        let acknowledge_url = format!("{}/policies/pending/", self.app_url);
        let subject = self.render_template(&template.subject, data, &acknowledge_url);
        let body_html = self.render_template(&template.body_html, data, &acknowledge_url);
        let body_text = self.render_template(&template.body_text, data, &acknowledge_url);

        // Send via SES
        self.send_email(&data.user_email, &subject, &body_html, &body_text).await?;

        // Record that reminder was sent
        sqlx::query(
            r#"
            INSERT INTO policy_reminders_sent (organization_id, policy_id, user_id, policy_version, reminder_type, channel)
            VALUES ($1, $2, $3, $4, 'scheduled', 'email')
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(org_id)
        .bind(data.policy_id)
        .bind(data.user_id)
        .bind(data.policy_version)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get email template (org-specific or system default)
    async fn get_email_template(&self, org_id: Uuid, template_type: &str) -> AppResult<EmailTemplate> {
        // Try org-specific template first
        let template = sqlx::query_as::<_, EmailTemplate>(
            r#"
            SELECT id, template_type, subject, body_html, body_text
            FROM email_templates
            WHERE (organization_id = $1 OR organization_id IS NULL)
              AND template_type = $2
            ORDER BY organization_id DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(template_type)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Email template {} not found", template_type)))?;

        Ok(template)
    }

    /// Render template with data
    fn render_template(&self, template: &str, data: &PolicyReminderData, acknowledge_url: &str) -> String {
        template
            .replace("{{policy_title}}", &data.policy_title)
            .replace("{{policy_code}}", &data.policy_code)
            .replace("{{policy_version}}", &data.policy_version.to_string())
            .replace("{{user_name}}", &data.user_name)
            .replace("{{organization_name}}", &data.organization_name)
            .replace("{{acknowledge_url}}", acknowledge_url)
    }

    /// Send email via AWS SES
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body_html: &str,
        body_text: &str,
    ) -> AppResult<()> {
        let Some(ref client) = self.ses_client else {
            // Log but don't fail in dev mode
            tracing::info!("SES not configured, would send email to: {} subject: {}", to, subject);
            return Ok(());
        };

        let destination = Destination::builder().to_addresses(to).build();

        let subject_content = Content::builder()
            .data(subject)
            .charset("UTF-8")
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email subject: {}", e)))?;
        let html_content = Content::builder()
            .data(body_html)
            .charset("UTF-8")
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email HTML: {}", e)))?;
        let text_content = Content::builder()
            .data(body_text)
            .charset("UTF-8")
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email text: {}", e)))?;

        let body = Body::builder()
            .html(html_content)
            .text(text_content)
            .build();

        let message = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(message).build();

        client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(destination)
            .content(email_content)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("SES error: {}", e)))?;

        tracing::info!("Sent email to: {} subject: {}", to, subject);

        Ok(())
    }

    // ==================== Policy Reminders ====================

    /// Send policy reminder to a specific user (both email and in-app)
    pub async fn send_policy_reminder(
        &self,
        org_id: Uuid,
        data: PolicyReminderData,
        send_email: bool,
        send_in_app: bool,
    ) -> AppResult<()> {
        // Create in-app notification
        if send_in_app {
            let notification_data = serde_json::json!({
                "policy_id": data.policy_id,
                "policy_code": data.policy_code,
            });

            self.create_notification(
                org_id,
                CreateNotification {
                    user_id: data.user_id,
                    notification_type: "policy_reminder".to_string(),
                    title: format!("Please acknowledge: {}", data.policy_title),
                    message: format!(
                        "The policy \"{}\" ({}) requires your acknowledgment.",
                        data.policy_title, data.policy_code
                    ),
                    data: Some(notification_data),
                },
            )
            .await?;

            // Record reminder sent
            sqlx::query(
                r#"
                INSERT INTO policy_reminders_sent (organization_id, policy_id, user_id, policy_version, reminder_type, channel)
                VALUES ($1, $2, $3, $4, 'manual', 'in_app')
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(org_id)
            .bind(data.policy_id)
            .bind(data.user_id)
            .bind(data.policy_version)
            .execute(&self.db)
            .await?;
        }

        // Send email
        if send_email {
            self.send_policy_reminder_email(org_id, &data, "policy_reminder").await?;
        }

        Ok(())
    }

    /// Get all users who haven't acknowledged a specific policy version
    pub async fn get_users_pending_acknowledgment(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        policy_version: i32,
    ) -> AppResult<Vec<UserInfo>> {
        let users = sqlx::query_as::<_, UserInfo>(
            r#"
            SELECT u.id, u.email, COALESCE(u.first_name || ' ' || u.last_name, u.email) as name
            FROM users u
            WHERE u.organization_id = $1
              AND NOT EXISTS (
                  SELECT 1 FROM policy_acknowledgments pa
                  WHERE pa.policy_id = $2
                    AND pa.policy_version = $3
                    AND pa.user_id = u.id
              )
            "#,
        )
        .bind(org_id)
        .bind(policy_id)
        .bind(policy_version)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}
