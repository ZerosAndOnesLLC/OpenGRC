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
    #[allow(dead_code)]
    id: Uuid,
    #[allow(dead_code)]
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
    pub async fn new(db: PgPool, _cache: CacheClient, config: &Config) -> Self {
        let ses_client = if config.is_production() {
            // In production, initialize SES client
            let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            Some(SesClient::new(&aws_config))
        } else {
            None
        };

        Self {
            db,
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

#[derive(Debug, Clone)]
pub struct TaskReminderData {
    pub task_id: Uuid,
    pub task_title: String,
    pub due_at: DateTime<Utc>,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
}

impl NotificationService {
    /// Send task due date reminder (both email and in-app)
    pub async fn send_task_reminder(
        &self,
        org_id: Uuid,
        data: TaskReminderData,
        reminder_type: &str, // "due_today", "due_soon", "overdue"
    ) -> AppResult<()> {
        // Create in-app notification
        let notification_data = serde_json::json!({
            "task_id": data.task_id,
            "due_at": data.due_at,
        });

        let (title, message) = match reminder_type {
            "overdue" => (
                format!("Overdue Task: {}", data.task_title),
                format!(
                    "The task \"{}\" was due on {} and is overdue.",
                    data.task_title,
                    data.due_at.format("%Y-%m-%d")
                ),
            ),
            "due_today" => (
                format!("Task Due Today: {}", data.task_title),
                format!("The task \"{}\" is due today.", data.task_title),
            ),
            _ => (
                format!("Task Due Soon: {}", data.task_title),
                format!(
                    "The task \"{}\" is due on {}.",
                    data.task_title,
                    data.due_at.format("%Y-%m-%d")
                ),
            ),
        };

        self.create_notification(
            org_id,
            CreateNotification {
                user_id: data.user_id,
                notification_type: format!("task_{}", reminder_type),
                title,
                message,
                data: Some(notification_data),
            },
        )
        .await?;

        Ok(())
    }

    /// Get tasks needing reminders for an organization
    pub async fn get_tasks_needing_reminders(
        &self,
        org_id: Uuid,
    ) -> AppResult<Vec<(TaskReminderData, String)>> {
        // Get overdue tasks (no reminder sent in last 24 hours)
        let overdue: Vec<(Uuid, String, DateTime<Utc>, Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT t.id, t.title, t.due_at, u.id, COALESCE(u.name, u.email), u.email
            FROM tasks t
            JOIN users u ON t.assignee_id = u.id
            WHERE t.organization_id = $1
              AND t.status IN ('open', 'in_progress')
              AND t.due_at < NOW()
              AND t.assignee_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM notifications n
                  WHERE n.user_id = t.assignee_id
                    AND n.notification_type = 'task_overdue'
                    AND (n.data->>'task_id')::uuid = t.id
                    AND n.created_at > NOW() - INTERVAL '24 hours'
              )
            ORDER BY t.due_at ASC
            LIMIT 100
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get tasks due today (no reminder sent today)
        let due_today: Vec<(Uuid, String, DateTime<Utc>, Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT t.id, t.title, t.due_at, u.id, COALESCE(u.name, u.email), u.email
            FROM tasks t
            JOIN users u ON t.assignee_id = u.id
            WHERE t.organization_id = $1
              AND t.status IN ('open', 'in_progress')
              AND t.due_at::date = CURRENT_DATE
              AND t.assignee_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM notifications n
                  WHERE n.user_id = t.assignee_id
                    AND n.notification_type = 'task_due_today'
                    AND (n.data->>'task_id')::uuid = t.id
                    AND n.created_at::date = CURRENT_DATE
              )
            ORDER BY t.due_at ASC
            LIMIT 100
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get tasks due in next 3 days (no reminder sent in last 3 days)
        let due_soon: Vec<(Uuid, String, DateTime<Utc>, Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT t.id, t.title, t.due_at, u.id, COALESCE(u.name, u.email), u.email
            FROM tasks t
            JOIN users u ON t.assignee_id = u.id
            WHERE t.organization_id = $1
              AND t.status IN ('open', 'in_progress')
              AND t.due_at::date > CURRENT_DATE
              AND t.due_at::date <= CURRENT_DATE + INTERVAL '3 days'
              AND t.assignee_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM notifications n
                  WHERE n.user_id = t.assignee_id
                    AND n.notification_type = 'task_due_soon'
                    AND (n.data->>'task_id')::uuid = t.id
                    AND n.created_at > NOW() - INTERVAL '3 days'
              )
            ORDER BY t.due_at ASC
            LIMIT 100
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let mut results = Vec::new();

        for (task_id, task_title, due_at, user_id, user_name, user_email) in overdue {
            results.push((
                TaskReminderData {
                    task_id,
                    task_title,
                    due_at,
                    user_id,
                    user_name,
                    user_email,
                },
                "overdue".to_string(),
            ));
        }

        for (task_id, task_title, due_at, user_id, user_name, user_email) in due_today {
            results.push((
                TaskReminderData {
                    task_id,
                    task_title,
                    due_at,
                    user_id,
                    user_name,
                    user_email,
                },
                "due_today".to_string(),
            ));
        }

        for (task_id, task_title, due_at, user_id, user_name, user_email) in due_soon {
            results.push((
                TaskReminderData {
                    task_id,
                    task_title,
                    due_at,
                    user_id,
                    user_name,
                    user_email,
                },
                "due_soon".to_string(),
            ));
        }

        Ok(results)
    }

    /// Process and send all due task reminders for an organization
    pub async fn process_task_reminders(&self, org_id: Uuid) -> AppResult<i32> {
        let tasks_needing_reminders = self.get_tasks_needing_reminders(org_id).await?;
        let count = tasks_needing_reminders.len() as i32;

        for (data, reminder_type) in tasks_needing_reminders {
            if let Err(e) = self.send_task_reminder(org_id, data, &reminder_type).await {
                tracing::warn!("Failed to send task reminder: {}", e);
            }
        }

        Ok(count)
    }
}

// ==================== Security Alert Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlertData {
    pub alert_type: SecurityAlertType,
    pub integration_id: Uuid,
    pub integration_name: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub details: serde_json::Value,
    pub event_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityAlertType {
    RootAccountActivity,
    SensitiveIamAction,
    FailedAuthentication,
    CriticalSecurityFinding,
    ComplianceViolation,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl NotificationService {
    /// Create security alert notifications for admins/compliance managers
    pub async fn create_security_alert(
        &self,
        org_id: Uuid,
        alert: SecurityAlertData,
    ) -> AppResult<i32> {
        // Get all admin and compliance manager users
        let admin_users: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM users
            WHERE organization_id = $1 AND role IN ('admin', 'compliance_manager')
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        if admin_users.is_empty() {
            tracing::warn!("No admin users found to receive security alert for org {}", org_id);
            return Ok(0);
        }

        let notification_type = format!("security_{}", serde_json::to_string(&alert.alert_type)
            .unwrap_or_default()
            .trim_matches('"'));

        let notification_data = serde_json::json!({
            "alert_type": alert.alert_type,
            "integration_id": alert.integration_id,
            "severity": alert.severity,
            "details": alert.details,
            "event_time": alert.event_time,
        });

        let mut created_count = 0;
        for (user_id,) in admin_users {
            // Check if we already sent this exact alert recently (within 1 hour)
            let (existing_count,): (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM notifications
                WHERE organization_id = $1
                  AND user_id = $2
                  AND notification_type = $3
                  AND title = $4
                  AND created_at > NOW() - INTERVAL '1 hour'
                "#,
            )
            .bind(org_id)
            .bind(user_id)
            .bind(&notification_type)
            .bind(&alert.title)
            .fetch_one(&self.db)
            .await?;

            if existing_count > 0 {
                continue; // Skip duplicate
            }

            self.create_notification(
                org_id,
                CreateNotification {
                    user_id,
                    notification_type: notification_type.clone(),
                    title: alert.title.clone(),
                    message: alert.description.clone(),
                    data: Some(notification_data.clone()),
                },
            )
            .await?;

            created_count += 1;
        }

        if created_count > 0 {
            tracing::info!(
                "Created {} security alert notifications for org {} - {}",
                created_count,
                org_id,
                alert.title
            );
        }

        Ok(created_count)
    }

    /// Create alerts for CloudTrail security events
    pub async fn create_cloudtrail_security_alerts(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
        integration_name: &str,
        root_actions: Vec<serde_json::Value>,
        sensitive_actions: Vec<serde_json::Value>,
        failed_actions: Vec<serde_json::Value>,
    ) -> AppResult<i32> {
        let mut total_alerts = 0;

        // Alert for root account activity (Critical)
        if !root_actions.is_empty() {
            let alert = SecurityAlertData {
                alert_type: SecurityAlertType::RootAccountActivity,
                integration_id,
                integration_name: integration_name.to_string(),
                severity: AlertSeverity::Critical,
                title: format!("Root Account Activity Detected ({} actions)", root_actions.len()),
                description: format!(
                    "AWS root account was used for {} action(s). Root account usage should be minimized and monitored.",
                    root_actions.len()
                ),
                details: serde_json::json!({
                    "actions": root_actions.iter().take(10).collect::<Vec<_>>(),
                    "total_count": root_actions.len(),
                }),
                event_time: root_actions.first()
                    .and_then(|a| a.get("event_time"))
                    .and_then(|t| t.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            };
            total_alerts += self.create_security_alert(org_id, alert).await?;
        }

        // Alert for sensitive IAM actions (High)
        if sensitive_actions.len() >= 5 {
            let alert = SecurityAlertData {
                alert_type: SecurityAlertType::SensitiveIamAction,
                integration_id,
                integration_name: integration_name.to_string(),
                severity: AlertSeverity::High,
                title: format!("Sensitive IAM Actions Detected ({} actions)", sensitive_actions.len()),
                description: format!(
                    "{} sensitive IAM actions detected. Review for unauthorized changes.",
                    sensitive_actions.len()
                ),
                details: serde_json::json!({
                    "actions": sensitive_actions.iter().take(10).collect::<Vec<_>>(),
                    "total_count": sensitive_actions.len(),
                }),
                event_time: sensitive_actions.first()
                    .and_then(|a| a.get("event_time"))
                    .and_then(|t| t.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            };
            total_alerts += self.create_security_alert(org_id, alert).await?;
        }

        // Alert for failed authentication attempts (Medium - only if significant)
        if failed_actions.len() >= 10 {
            let alert = SecurityAlertData {
                alert_type: SecurityAlertType::FailedAuthentication,
                integration_id,
                integration_name: integration_name.to_string(),
                severity: AlertSeverity::Medium,
                title: format!("Multiple Failed API Calls ({} failures)", failed_actions.len()),
                description: format!(
                    "{} failed AWS API calls detected. This may indicate misconfigured permissions or unauthorized access attempts.",
                    failed_actions.len()
                ),
                details: serde_json::json!({
                    "failures": failed_actions.iter().take(10).collect::<Vec<_>>(),
                    "total_count": failed_actions.len(),
                }),
                event_time: failed_actions.first()
                    .and_then(|a| a.get("event_time"))
                    .and_then(|t| t.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            };
            total_alerts += self.create_security_alert(org_id, alert).await?;
        }

        Ok(total_alerts)
    }

    /// Create alerts for Security Hub critical findings
    pub async fn create_securityhub_alerts(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
        integration_name: &str,
        critical_count: i32,
        high_count: i32,
        critical_findings: Vec<serde_json::Value>,
    ) -> AppResult<i32> {
        let mut total_alerts = 0;

        // Alert for critical findings
        if critical_count > 0 {
            let alert = SecurityAlertData {
                alert_type: SecurityAlertType::CriticalSecurityFinding,
                integration_id,
                integration_name: integration_name.to_string(),
                severity: AlertSeverity::Critical,
                title: format!("{} Critical Security Hub Findings", critical_count),
                description: format!(
                    "AWS Security Hub detected {} critical and {} high severity findings that require immediate attention.",
                    critical_count, high_count
                ),
                details: serde_json::json!({
                    "critical_count": critical_count,
                    "high_count": high_count,
                    "critical_findings": critical_findings.iter().take(5).collect::<Vec<_>>(),
                }),
                event_time: Some(Utc::now()),
            };
            total_alerts += self.create_security_alert(org_id, alert).await?;
        } else if high_count >= 10 {
            // Alert for many high findings even if no critical
            let alert = SecurityAlertData {
                alert_type: SecurityAlertType::CriticalSecurityFinding,
                integration_id,
                integration_name: integration_name.to_string(),
                severity: AlertSeverity::High,
                title: format!("{} High Severity Security Hub Findings", high_count),
                description: format!(
                    "AWS Security Hub detected {} high severity findings that should be reviewed.",
                    high_count
                ),
                details: serde_json::json!({
                    "high_count": high_count,
                }),
                event_time: Some(Utc::now()),
            };
            total_alerts += self.create_security_alert(org_id, alert).await?;
        }

        Ok(total_alerts)
    }
}
