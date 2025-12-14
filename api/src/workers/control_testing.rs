use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::interval;
use uuid::Uuid;

/// Configuration for an automated test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    /// Type of automation: "http", "script", "integration"
    pub automation_type: String,
    /// HTTP endpoint to call (for http type)
    pub endpoint: Option<String>,
    /// HTTP method (GET, POST, etc.)
    pub method: Option<String>,
    /// Headers to include
    pub headers: Option<serde_json::Map<String, JsonValue>>,
    /// Request body
    pub body: Option<JsonValue>,
    /// Expected status codes for pass
    pub expected_status_codes: Option<Vec<u16>>,
    /// JSONPath expression to validate response
    pub validation_path: Option<String>,
    /// Expected value at validation path
    pub expected_value: Option<JsonValue>,
    /// Integration ID to use
    pub integration_id: Option<Uuid>,
    /// Integration-specific configuration
    pub integration_config: Option<JsonValue>,
}

/// A control test due for execution
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DueTest {
    pub id: Uuid,
    pub control_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub test_type: String,
    pub automation_config: Option<JsonValue>,
    pub frequency: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
    pub timeout_seconds: Option<i32>,
    pub retry_count: Option<i32>,
}

/// Result of an automated test execution
#[derive(Debug)]
pub struct TestExecutionResult {
    pub status: String,
    pub notes: String,
    pub raw_response: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: i32,
}

/// Alert configuration for a test
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertConfig {
    pub alert_on_failure: bool,
    pub consecutive_failures_threshold: i32,
    pub alert_on_recovery: bool,
    pub alert_recipients: Option<Vec<Uuid>>,
    pub alert_email_enabled: bool,
    pub alert_in_app_enabled: bool,
    pub is_muted: bool,
    pub muted_until: Option<DateTime<Utc>>,
}

/// Worker that runs automated control tests
pub struct ControlTestingWorker {
    db: PgPool,
    http_client: reqwest::Client,
    check_interval_secs: u64,
}

impl ControlTestingWorker {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
            check_interval_secs: 60, // Check every minute
        }
    }

    /// Start the worker loop
    pub async fn run(self: Arc<Self>) {
        tracing::info!("Starting control testing worker");

        let mut interval = interval(std::time::Duration::from_secs(self.check_interval_secs));

        loop {
            interval.tick().await;

            if let Err(e) = self.process_due_tests().await {
                tracing::error!("Error processing due tests: {}", e);
            }
        }
    }

    /// Process all tests that are due
    async fn process_due_tests(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let due_tests = self.get_due_tests().await?;

        if due_tests.is_empty() {
            tracing::debug!("No tests due for execution");
            return Ok(());
        }

        tracing::info!("Found {} tests due for execution", due_tests.len());

        for test in due_tests {
            if test.test_type != "automated" {
                continue;
            }

            // Create a test run record
            let run_id = self.create_test_run(&test).await?;
            let start_time = Instant::now();

            let result = match self.execute_test(&test).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to execute test {}: {}", test.name, e);
                    TestExecutionResult {
                        status: "error".to_string(),
                        notes: "Test execution failed".to_string(),
                        raw_response: None,
                        error_message: Some(e.to_string()),
                        execution_time_ms: start_time.elapsed().as_millis() as i32,
                    }
                }
            };

            // Record the result
            if let Err(e) = self.record_result(&test, run_id, &result).await {
                tracing::error!("Failed to record result for test {}: {}", test.id, e);
            } else {
                tracing::info!(
                    "Test {} completed with status: {} ({}ms)",
                    test.name,
                    result.status,
                    result.execution_time_ms
                );

                // Check if we need to send alerts
                if result.status == "failed" || result.status == "error" {
                    if let Err(e) = self.check_and_send_alerts(&test, run_id, &result).await {
                        tracing::error!("Failed to send alert for test {}: {}", test.id, e);
                    }
                } else if result.status == "passed" {
                    // Check for recovery alerts
                    if let Err(e) = self.check_recovery_alert(&test, run_id).await {
                        tracing::error!("Failed to check recovery alert for test {}: {}", test.id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all tests that are due for execution
    async fn get_due_tests(&self) -> Result<Vec<DueTest>, sqlx::Error> {
        sqlx::query_as::<_, DueTest>(
            r#"
            SELECT
                ct.id,
                ct.control_id,
                c.organization_id,
                ct.name,
                ct.test_type,
                ct.automation_config,
                ct.frequency,
                ct.next_due_at,
                ct.timeout_seconds,
                ct.retry_count
            FROM control_tests ct
            JOIN controls c ON ct.control_id = c.id
            WHERE ct.test_type = 'automated'
              AND ct.automation_config IS NOT NULL
              AND (ct.is_enabled IS NULL OR ct.is_enabled = true)
              AND (ct.next_due_at IS NULL OR ct.next_due_at <= NOW())
            ORDER BY ct.next_due_at ASC NULLS FIRST
            LIMIT 100
            "#,
        )
        .fetch_all(&self.db)
        .await
    }

    /// Create a test run record
    async fn create_test_run(&self, test: &DueTest) -> Result<Uuid, sqlx::Error> {
        let (id,): (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO control_test_runs (organization_id, control_test_id, control_id, run_type)
            VALUES ($1, $2, $3, 'scheduled')
            RETURNING id
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .bind(test.control_id)
        .fetch_one(&self.db)
        .await?;

        Ok(id)
    }

    /// Execute an automated test
    async fn execute_test(
        &self,
        test: &DueTest,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();

        let config: AutomationConfig = match &test.automation_config {
            Some(json) => serde_json::from_value(json.clone())?,
            None => {
                return Ok(TestExecutionResult {
                    status: "error".to_string(),
                    notes: "No automation config found".to_string(),
                    raw_response: None,
                    error_message: Some("Missing automation config".to_string()),
                    execution_time_ms: 0,
                });
            }
        };

        let mut result = match config.automation_type.as_str() {
            "http" => self.execute_http_test(&config, test.timeout_seconds).await?,
            "integration" => self.execute_integration_test(&config).await?,
            _ => TestExecutionResult {
                status: "error".to_string(),
                notes: format!("Unknown automation type: {}", config.automation_type),
                raw_response: None,
                error_message: Some(format!("Unknown automation type: {}", config.automation_type)),
                execution_time_ms: 0,
            },
        };

        result.execution_time_ms = start_time.elapsed().as_millis() as i32;
        Ok(result)
    }

    /// Execute an HTTP-based test
    async fn execute_http_test(
        &self,
        config: &AutomationConfig,
        timeout_seconds: Option<i32>,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = config
            .endpoint
            .as_ref()
            .ok_or("HTTP test requires endpoint")?;

        let method = config.method.as_deref().unwrap_or("GET");

        // Build client with custom timeout if specified
        let client = if let Some(timeout) = timeout_seconds {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout as u64))
                .build()?
        } else {
            self.http_client.clone()
        };

        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(endpoint),
            "POST" => client.post(endpoint),
            "PUT" => client.put(endpoint),
            "DELETE" => client.delete(endpoint),
            "PATCH" => client.patch(endpoint),
            "HEAD" => client.head(endpoint),
            _ => return Err(format!("Unsupported HTTP method: {}", method).into()),
        };

        // Add headers
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                if let Some(value_str) = value.as_str() {
                    request = request.header(key, value_str);
                }
            }
        }

        // Add body for applicable methods
        if let Some(body) = &config.body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status_code = response.status().as_u16();
        let body = response.text().await?;

        // Check expected status codes
        let expected_codes = config
            .expected_status_codes
            .as_ref()
            .cloned()
            .unwrap_or_else(|| vec![200, 201, 204]);

        let status_ok = expected_codes.contains(&status_code);

        // Check validation path if provided
        let validation_ok = if let (Some(path), Some(expected)) =
            (&config.validation_path, &config.expected_value)
        {
            match serde_json::from_str::<JsonValue>(&body) {
                Ok(json_body) => self.validate_json_path(&json_body, path, expected),
                Err(_) => false,
            }
        } else {
            true
        };

        let (status, notes, error_message) = if status_ok && validation_ok {
            (
                "passed".to_string(),
                format!("HTTP {} returned status {}", method, status_code),
                None,
            )
        } else if !status_ok {
            (
                "failed".to_string(),
                format!(
                    "Unexpected status code: {}. Expected one of: {:?}",
                    status_code, expected_codes
                ),
                Some(format!("HTTP {} returned unexpected status {}", method, status_code)),
            )
        } else {
            (
                "failed".to_string(),
                format!("Response validation failed for path: {:?}", config.validation_path),
                Some("Response validation failed".to_string()),
            )
        };

        Ok(TestExecutionResult {
            status,
            notes,
            raw_response: Some(body.chars().take(5000).collect()),
            error_message,
            execution_time_ms: 0,
        })
    }

    /// Execute an integration-based test
    async fn execute_integration_test(
        &self,
        config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let integration_id = config
            .integration_id
            .ok_or("Integration test requires integration_id")?;

        // Get integration details
        let integration: Option<(String, JsonValue)> = sqlx::query_as(
            "SELECT integration_type, config FROM integrations WHERE id = $1",
        )
        .bind(integration_id)
        .fetch_optional(&self.db)
        .await?;

        let (integration_type, integration_config) = match integration {
            Some((t, c)) => (t, c),
            None => {
                return Ok(TestExecutionResult {
                    status: "error".to_string(),
                    notes: format!("Integration {} not found", integration_id),
                    raw_response: None,
                    error_message: Some(format!("Integration {} not found", integration_id)),
                    execution_time_ms: 0,
                });
            }
        };

        // Execute based on integration type
        match integration_type.as_str() {
            "aws" => self.execute_aws_integration_test(&integration_config, config).await,
            "github" => self.execute_github_integration_test(&integration_config, config).await,
            _ => Ok(TestExecutionResult {
                status: "skipped".to_string(),
                notes: format!(
                    "Integration type '{}' not yet supported for automated testing",
                    integration_type
                ),
                raw_response: None,
                error_message: None,
                execution_time_ms: 0,
            }),
        }
    }

    /// Execute AWS integration test (placeholder)
    async fn execute_aws_integration_test(
        &self,
        _integration_config: &JsonValue,
        _test_config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement AWS-specific compliance checks based on test_config
        Ok(TestExecutionResult {
            status: "skipped".to_string(),
            notes: "AWS integration testing not yet implemented".to_string(),
            raw_response: None,
            error_message: None,
            execution_time_ms: 0,
        })
    }

    /// Execute GitHub integration test (placeholder)
    async fn execute_github_integration_test(
        &self,
        _integration_config: &JsonValue,
        _test_config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement GitHub-specific compliance checks based on test_config
        Ok(TestExecutionResult {
            status: "skipped".to_string(),
            notes: "GitHub integration testing not yet implemented".to_string(),
            raw_response: None,
            error_message: None,
            execution_time_ms: 0,
        })
    }

    /// Validate a JSON path against an expected value
    fn validate_json_path(&self, json: &JsonValue, path: &str, expected: &JsonValue) -> bool {
        // Simple dot-notation path traversal
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return false,
            };
        }

        current == expected
    }

    /// Record the test result and update next_due_at
    async fn record_result(
        &self,
        test: &DueTest,
        run_id: Uuid,
        result: &TestExecutionResult,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        // Update the test run record
        sqlx::query(
            r#"
            UPDATE control_test_runs
            SET completed_at = NOW(),
                status = $2,
                notes = $3,
                raw_output = $4,
                error_message = $5,
                execution_time_ms = $6
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(&result.status)
        .bind(&result.notes)
        .bind(&result.raw_response)
        .bind(&result.error_message)
        .bind(result.execution_time_ms)
        .execute(&mut *tx)
        .await?;

        // Also insert into legacy control_test_results for backwards compatibility
        sqlx::query(
            r#"
            INSERT INTO control_test_results (control_test_id, performed_by, status, notes)
            VALUES ($1, NULL, $2, $3)
            "#,
        )
        .bind(test.id)
        .bind(&result.status)
        .bind(&result.notes)
        .execute(&mut *tx)
        .await?;

        // Update control_tests stats
        let pass_increment = if result.status == "passed" { 1 } else { 0 };
        let fail_increment = if result.status != "passed" && result.status != "skipped" { 1 } else { 0 };

        sqlx::query(
            r#"
            UPDATE control_tests
            SET last_run_at = NOW(),
                last_run_status = $2,
                run_count = COALESCE(run_count, 0) + 1,
                pass_count = COALESCE(pass_count, 0) + $3,
                fail_count = COALESCE(fail_count, 0) + $4,
                next_due_at = $5
            WHERE id = $1
            "#,
        )
        .bind(test.id)
        .bind(&result.status)
        .bind(pass_increment)
        .bind(fail_increment)
        .bind(self.calculate_next_due(test.frequency.as_deref()))
        .execute(&mut *tx)
        .await?;

        // Update monitoring status via DB function
        sqlx::query("SELECT update_control_monitoring_status($1, $2, $3)")
            .bind(test.organization_id)
            .bind(test.control_id)
            .bind(&result.status)
            .execute(&mut *tx)
            .await?;

        // Find matching remediation if failed
        if result.status == "failed" || result.status == "error" {
            let failure_msg = result.error_message.as_deref().unwrap_or(&result.notes);
            let remediation_id: Option<(Uuid,)> = sqlx::query_as(
                "SELECT find_matching_remediation($1, $2)"
            )
            .bind(test.id)
            .bind(failure_msg)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some((rem_id,)) = remediation_id {
                sqlx::query("UPDATE control_test_runs SET remediation_suggested_id = $2 WHERE id = $1")
                    .bind(run_id)
                    .bind(rem_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    /// Check if we need to send alerts and send them
    async fn check_and_send_alerts(
        &self,
        test: &DueTest,
        run_id: Uuid,
        result: &TestExecutionResult,
    ) -> Result<(), sqlx::Error> {
        // Get alert config
        let alert_config: Option<AlertConfig> = sqlx::query_as(
            r#"
            SELECT alert_on_failure, consecutive_failures_threshold, alert_on_recovery,
                   alert_recipients, alert_email_enabled, alert_in_app_enabled,
                   is_muted, muted_until
            FROM control_test_alert_configs
            WHERE organization_id = $1 AND control_test_id = $2
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .fetch_optional(&self.db)
        .await?;

        let config = match alert_config {
            Some(c) => c,
            None => return Ok(()), // No alert config, skip
        };

        // Check if muted
        if config.is_muted {
            if let Some(until) = config.muted_until {
                if until > Utc::now() {
                    return Ok(()); // Still muted
                }
            } else {
                return Ok(()); // Indefinitely muted
            }
        }

        if !config.alert_on_failure {
            return Ok(());
        }

        // Check consecutive failures threshold
        let (consecutive_failures,): (i32,) = sqlx::query_as(
            "SELECT consecutive_failures FROM control_monitoring_status WHERE organization_id = $1 AND control_id = $2"
        )
        .bind(test.organization_id)
        .bind(test.control_id)
        .fetch_one(&self.db)
        .await?;

        if consecutive_failures < config.consecutive_failures_threshold {
            return Ok(()); // Haven't hit threshold yet
        }

        // Get recipients
        let recipients = config.alert_recipients.unwrap_or_default();
        if recipients.is_empty() {
            return Ok(()); // No one to alert
        }

        // Determine severity based on consecutive failures
        let severity = if consecutive_failures >= 5 {
            "critical"
        } else if consecutive_failures >= 3 {
            "high"
        } else {
            "medium"
        };

        // Create alert
        let title = format!("Control test failed: {}", test.name);
        let message = format!(
            "Control test '{}' has failed {} consecutive time(s).\n\nError: {}",
            test.name,
            consecutive_failures,
            result.error_message.as_deref().unwrap_or(&result.notes)
        );

        sqlx::query(
            r#"
            INSERT INTO control_test_alerts (
                organization_id, control_test_id, test_run_id, alert_type, severity,
                title, message, recipients, email_sent, in_app_sent
            )
            VALUES ($1, $2, $3, 'failure', $4, $5, $6, $7, false, false)
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .bind(run_id)
        .bind(severity)
        .bind(&title)
        .bind(&message)
        .bind(&recipients)
        .execute(&self.db)
        .await?;

        // Mark alert sent on run
        sqlx::query("UPDATE control_test_runs SET alert_sent = true, alert_sent_at = NOW() WHERE id = $1")
            .bind(run_id)
            .execute(&self.db)
            .await?;

        // Create in-app notifications for all recipients
        if config.alert_in_app_enabled {
            for recipient_id in &recipients {
                sqlx::query(
                    r#"
                    INSERT INTO notifications (organization_id, user_id, notification_type, title, message, data)
                    VALUES ($1, $2, 'control_test_alert', $3, $4, $5)
                    "#,
                )
                .bind(test.organization_id)
                .bind(recipient_id)
                .bind(&title)
                .bind(&message)
                .bind(serde_json::json!({
                    "control_test_id": test.id,
                    "run_id": run_id,
                    "severity": severity,
                    "consecutive_failures": consecutive_failures
                }))
                .execute(&self.db)
                .await?;
            }
        }

        tracing::info!(
            "Sent {} alert for test {} to {} recipients",
            severity,
            test.name,
            recipients.len()
        );

        Ok(())
    }

    /// Check if we should send a recovery alert
    async fn check_recovery_alert(&self, test: &DueTest, run_id: Uuid) -> Result<(), sqlx::Error> {
        // Get alert config
        let alert_config: Option<AlertConfig> = sqlx::query_as(
            r#"
            SELECT alert_on_failure, consecutive_failures_threshold, alert_on_recovery,
                   alert_recipients, alert_email_enabled, alert_in_app_enabled,
                   is_muted, muted_until
            FROM control_test_alert_configs
            WHERE organization_id = $1 AND control_test_id = $2
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .fetch_optional(&self.db)
        .await?;

        let config = match alert_config {
            Some(c) if c.alert_on_recovery => c,
            _ => return Ok(()), // No config or recovery alerts disabled
        };

        // Check if there was a recent unresolved alert
        let recent_alert: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM control_test_alerts
            WHERE organization_id = $1
              AND control_test_id = $2
              AND alert_type = 'failure'
              AND resolved_at IS NULL
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .fetch_optional(&self.db)
        .await?;

        if recent_alert.is_none() {
            return Ok(()); // No recent unresolved alert
        }

        let recipients = config.alert_recipients.unwrap_or_default();
        if recipients.is_empty() {
            return Ok(());
        }

        // Create recovery alert
        let title = format!("Control test recovered: {}", test.name);
        let message = format!(
            "Control test '{}' has recovered and is now passing.",
            test.name
        );

        sqlx::query(
            r#"
            INSERT INTO control_test_alerts (
                organization_id, control_test_id, test_run_id, alert_type, severity,
                title, message, recipients
            )
            VALUES ($1, $2, $3, 'recovery', 'low', $4, $5, $6)
            "#,
        )
        .bind(test.organization_id)
        .bind(test.id)
        .bind(run_id)
        .bind(&title)
        .bind(&message)
        .bind(&recipients)
        .execute(&self.db)
        .await?;

        // Create in-app notifications
        if config.alert_in_app_enabled {
            for recipient_id in &recipients {
                sqlx::query(
                    r#"
                    INSERT INTO notifications (organization_id, user_id, notification_type, title, message, data)
                    VALUES ($1, $2, 'control_test_recovery', $3, $4, $5)
                    "#,
                )
                .bind(test.organization_id)
                .bind(recipient_id)
                .bind(&title)
                .bind(&message)
                .bind(serde_json::json!({
                    "control_test_id": test.id,
                    "run_id": run_id
                }))
                .execute(&self.db)
                .await?;
            }
        }

        tracing::info!("Sent recovery alert for test {}", test.name);

        Ok(())
    }

    /// Calculate the next due date based on frequency
    fn calculate_next_due(&self, frequency: Option<&str>) -> DateTime<Utc> {
        let now = Utc::now();

        match frequency {
            Some("continuous") => now + Duration::minutes(5),
            Some("hourly") => now + Duration::hours(1),
            Some("daily") => now + Duration::days(1),
            Some("weekly") => now + Duration::weeks(1),
            Some("monthly") => now + Duration::days(30),
            Some("quarterly") => now + Duration::days(90),
            Some("annually") | Some("yearly") => now + Duration::days(365),
            _ => now + Duration::days(1), // Default to daily
        }
    }
}
