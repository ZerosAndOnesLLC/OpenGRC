use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;
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
}

/// Result of an automated test execution
#[derive(Debug)]
pub struct TestExecutionResult {
    pub status: String,
    pub notes: String,
    pub raw_response: Option<String>,
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
                .timeout(std::time::Duration::from_secs(30))
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

            match self.execute_test(&test).await {
                Ok(result) => {
                    if let Err(e) = self.record_result(&test, &result).await {
                        tracing::error!("Failed to record result for test {}: {}", test.id, e);
                    } else {
                        tracing::info!(
                            "Test {} completed with status: {}",
                            test.name,
                            result.status
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute test {}: {}", test.name, e);
                    // Record failure
                    let failure_result = TestExecutionResult {
                        status: "failed".to_string(),
                        notes: format!("Automation error: {}", e),
                        raw_response: None,
                    };
                    let _ = self.record_result(&test, &failure_result).await;
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
                ct.next_due_at
            FROM control_tests ct
            JOIN controls c ON ct.control_id = c.id
            WHERE ct.test_type = 'automated'
              AND ct.automation_config IS NOT NULL
              AND (ct.next_due_at IS NULL OR ct.next_due_at <= NOW())
            ORDER BY ct.next_due_at ASC NULLS FIRST
            LIMIT 100
            "#,
        )
        .fetch_all(&self.db)
        .await
    }

    /// Execute an automated test
    async fn execute_test(
        &self,
        test: &DueTest,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let config: AutomationConfig = match &test.automation_config {
            Some(json) => serde_json::from_value(json.clone())?,
            None => {
                return Err("No automation config found".into());
            }
        };

        match config.automation_type.as_str() {
            "http" => self.execute_http_test(&config).await,
            "integration" => self.execute_integration_test(&config).await,
            _ => Err(format!("Unknown automation type: {}", config.automation_type).into()),
        }
    }

    /// Execute an HTTP-based test
    async fn execute_http_test(
        &self,
        config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = config
            .endpoint
            .as_ref()
            .ok_or("HTTP test requires endpoint")?;

        let method = config.method.as_deref().unwrap_or("GET");

        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(endpoint),
            "POST" => self.http_client.post(endpoint),
            "PUT" => self.http_client.put(endpoint),
            "DELETE" => self.http_client.delete(endpoint),
            "PATCH" => self.http_client.patch(endpoint),
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
            .map(|c| c.clone())
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

        let (status, notes) = if status_ok && validation_ok {
            (
                "passed".to_string(),
                format!("HTTP {} returned status {}", method, status_code),
            )
        } else if !status_ok {
            (
                "failed".to_string(),
                format!(
                    "Unexpected status code: {}. Expected one of: {:?}",
                    status_code, expected_codes
                ),
            )
        } else {
            (
                "failed".to_string(),
                format!("Response validation failed for path: {:?}", config.validation_path),
            )
        };

        Ok(TestExecutionResult {
            status,
            notes,
            raw_response: Some(body.chars().take(1000).collect()),
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
                    status: "failed".to_string(),
                    notes: format!("Integration {} not found", integration_id),
                    raw_response: None,
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
            }),
        }
    }

    /// Execute AWS integration test (placeholder)
    async fn execute_aws_integration_test(
        &self,
        _integration_config: &JsonValue,
        _test_config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // This would check AWS resources, security groups, IAM policies, etc.
        Ok(TestExecutionResult {
            status: "skipped".to_string(),
            notes: "AWS integration testing not yet implemented".to_string(),
            raw_response: None,
        })
    }

    /// Execute GitHub integration test (placeholder)
    async fn execute_github_integration_test(
        &self,
        _integration_config: &JsonValue,
        _test_config: &AutomationConfig,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // This would check repo settings, branch protections, etc.
        Ok(TestExecutionResult {
            status: "skipped".to_string(),
            notes: "GitHub integration testing not yet implemented".to_string(),
            raw_response: None,
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
        result: &TestExecutionResult,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        // Insert test result
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

        // Update next_due_at based on frequency
        let next_due = self.calculate_next_due(test.frequency.as_deref());

        sqlx::query(
            r#"
            UPDATE control_tests
            SET next_due_at = $2
            WHERE id = $1
            "#,
        )
        .bind(test.id)
        .bind(next_due)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Calculate the next due date based on frequency
    fn calculate_next_due(&self, frequency: Option<&str>) -> DateTime<Utc> {
        let now = Utc::now();

        match frequency {
            Some("daily") => now + Duration::days(1),
            Some("weekly") => now + Duration::weeks(1),
            Some("monthly") => now + Duration::days(30),
            Some("quarterly") => now + Duration::days(90),
            Some("annually") | Some("yearly") => now + Duration::days(365),
            _ => now + Duration::days(7), // Default to weekly
        }
    }
}
