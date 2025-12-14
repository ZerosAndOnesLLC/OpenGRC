use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::CacheClient;
use crate::routes::aws::{
    AwsCloudTrailQuery, AwsConfigRulesQuery, AwsFindingsQuery, AwsIamQuery, AwsResourcesQuery,
};
use crate::utils::AppError;

#[derive(Clone)]
pub struct AwsService {
    db: PgPool,
    cache: CacheClient,
}

// ==================== Response Types ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct AwsOverview {
    pub account_id: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub iam_stats: IamStats,
    pub security_stats: SecurityStats,
    pub resource_stats: ResourceStats,
    pub compliance_summary: ComplianceSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IamStats {
    pub total_users: i64,
    pub users_with_mfa: i64,
    pub users_without_mfa: i64,
    pub total_roles: i64,
    pub total_policies: i64,
    pub high_risk_policies: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityStats {
    pub critical_findings: i64,
    pub high_findings: i64,
    pub medium_findings: i64,
    pub low_findings: i64,
    pub total_config_rules: i64,
    pub compliant_rules: i64,
    pub non_compliant_rules: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceStats {
    pub s3_buckets: i64,
    pub ec2_instances: i64,
    pub rds_instances: i64,
    pub security_groups: i64,
    pub public_resources: i64,
    pub unencrypted_resources: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub overall_score: f64,
    pub compliant_checks: i64,
    pub non_compliant_checks: i64,
    pub unknown_checks: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindingsSummary {
    pub by_severity: Vec<SeverityCount>,
    pub by_workflow_status: Vec<WorkflowCount>,
    pub by_compliance_status: Vec<ComplianceCount>,
    pub recent_critical: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeverityCount {
    pub severity: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudTrailStats {
    pub total_events: i64,
    pub root_events: i64,
    pub sensitive_events: i64,
    pub high_risk_events: i64,
    pub events_by_source: Vec<EventSourceCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSourceCount {
    pub source: String,
    pub count: i64,
}

impl AwsService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Overview ====================

    pub async fn get_overview(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
    ) -> Result<AwsOverview, AppError> {
        // Try to get from cache first
        let cache_key = format!("aws:overview:{}:{}", org_id, integration_id);
        if let Ok(Some(cached)) = self.cache.get::<String>(&cache_key).await {
            if let Ok(overview) = serde_json::from_str(&cached) {
                return Ok(overview);
            }
        }

        // Get account ID
        let account: Option<(String,)> = sqlx::query_as(
            "SELECT DISTINCT aws_account_id FROM aws_iam_users WHERE integration_id = $1 LIMIT 1",
        )
        .bind(integration_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let account_id = account.map(|a| a.0).unwrap_or_default();

        // Get last synced time
        let sync_status: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
            "SELECT MAX(last_sync_at) FROM aws_sync_status WHERE integration_id = $1",
        )
        .bind(integration_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let last_synced_at = sync_status.and_then(|s| s.0);

        // Get IAM stats
        let iam_stats = self.get_iam_stats(integration_id).await?;

        // Get security stats
        let security_stats = self.get_security_stats(integration_id).await?;

        // Get resource stats
        let resource_stats = self.get_resource_stats(integration_id).await?;

        // Calculate compliance summary
        let compliance_summary = self
            .calculate_compliance_summary(integration_id)
            .await?;

        let overview = AwsOverview {
            account_id,
            last_synced_at,
            iam_stats,
            security_stats,
            resource_stats,
            compliance_summary,
        };

        // Cache for 5 minutes
        if let Ok(json) = serde_json::to_string(&overview) {
            let _ = self.cache.set(&cache_key, &json, Some(std::time::Duration::from_secs(300))).await;
        }

        Ok(overview)
    }

    async fn get_iam_stats(&self, integration_id: Uuid) -> Result<IamStats, AppError> {
        let user_stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE mfa_enabled = true)
            FROM aws_iam_users
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let role_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_iam_roles WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let policy_stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE risk_score >= 80)
            FROM aws_iam_policies
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(IamStats {
            total_users: user_stats.0,
            users_with_mfa: user_stats.1,
            users_without_mfa: user_stats.0 - user_stats.1,
            total_roles: role_count.0,
            total_policies: policy_stats.0,
            high_risk_policies: policy_stats.1,
        })
    }

    async fn get_security_stats(&self, integration_id: Uuid) -> Result<SecurityStats, AppError> {
        let finding_stats: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE severity_label = 'CRITICAL'),
                COUNT(*) FILTER (WHERE severity_label = 'HIGH'),
                COUNT(*) FILTER (WHERE severity_label = 'MEDIUM'),
                COUNT(*) FILTER (WHERE severity_label = 'LOW')
            FROM aws_security_findings
            WHERE integration_id = $1 AND record_state = 'ACTIVE'
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let config_stats: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE compliance_type = 'COMPLIANT'),
                COUNT(*) FILTER (WHERE compliance_type = 'NON_COMPLIANT')
            FROM aws_config_rules
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(SecurityStats {
            critical_findings: finding_stats.0,
            high_findings: finding_stats.1,
            medium_findings: finding_stats.2,
            low_findings: finding_stats.3,
            total_config_rules: config_stats.0,
            compliant_rules: config_stats.1,
            non_compliant_rules: config_stats.2,
        })
    }

    async fn get_resource_stats(&self, integration_id: Uuid) -> Result<ResourceStats, AppError> {
        let s3_stats: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE is_public = true),
                COUNT(*) FILTER (WHERE encryption_enabled = false)
            FROM aws_s3_buckets
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0, 0));

        let ec2_stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE is_public = true)
            FROM aws_ec2_instances
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0));

        let rds_stats: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE publicly_accessible = true),
                COUNT(*) FILTER (WHERE storage_encrypted = false)
            FROM aws_rds_instances
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0, 0));

        let sg_stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE has_risky_rules = true)
            FROM aws_security_groups
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0));

        Ok(ResourceStats {
            s3_buckets: s3_stats.0,
            ec2_instances: ec2_stats.0,
            rds_instances: rds_stats.0,
            security_groups: sg_stats.0,
            public_resources: s3_stats.1 + ec2_stats.1 + rds_stats.1,
            unencrypted_resources: s3_stats.2 + rds_stats.2,
        })
    }

    async fn calculate_compliance_summary(
        &self,
        integration_id: Uuid,
    ) -> Result<ComplianceSummary, AppError> {
        // Calculate compliance based on config rules and findings
        let config_compliance: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE compliance_type = 'COMPLIANT'),
                COUNT(*) FILTER (WHERE compliance_type IN ('NON_COMPLIANT', 'INSUFFICIENT_DATA'))
            FROM aws_config_rules
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0));

        let finding_compliance: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE compliance_status = 'PASSED'),
                COUNT(*) FILTER (WHERE compliance_status = 'FAILED'),
                COUNT(*) FILTER (WHERE compliance_status IS NULL OR compliance_status NOT IN ('PASSED', 'FAILED'))
            FROM aws_security_findings
            WHERE integration_id = $1 AND record_state = 'ACTIVE'
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or((0, 0, 0));

        let compliant = config_compliance.0 + finding_compliance.0;
        let non_compliant = config_compliance.1 + finding_compliance.1;
        let unknown = finding_compliance.2;
        let total = compliant + non_compliant + unknown;

        let score = if total > 0 {
            (compliant as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(ComplianceSummary {
            overall_score: (score * 100.0).round() / 100.0,
            compliant_checks: compliant,
            non_compliant_checks: non_compliant,
            unknown_checks: unknown,
        })
    }

    // ==================== IAM ====================

    pub async fn list_iam_users(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsIamQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = vec!["integration_id = $1".to_string()];

        if let Some(mfa) = query.mfa_enabled {
            where_clauses.push(format!("mfa_enabled = {}", mfa));
        }

        if query.has_access_keys.unwrap_or(false) {
            where_clauses.push("jsonb_array_length(access_keys) > 0".to_string());
        }

        if let Some(ref search) = query.search {
            where_clauses.push(format!(
                "(user_name ILIKE '%{}%' OR arn ILIKE '%{}%')",
                search.replace('\'', "''"),
                search.replace('\'', "''")
            ));
        }

        let where_sql = where_clauses.join(" AND ");

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM aws_iam_users WHERE {}", where_sql);
        let total: (i64,) = sqlx::query_as(&count_sql)
            .bind(integration_id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let data_sql = format!(
            r#"
            SELECT row_to_json(u.*)
            FROM aws_iam_users u
            WHERE {}
            ORDER BY user_name ASC
            LIMIT $2 OFFSET $3
            "#,
            where_sql
        );

        let rows: Vec<(Value,)> = sqlx::query_as(&data_sql)
            .bind(integration_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let users: Vec<Value> = rows.into_iter().map(|r| r.0).collect();

        Ok((users, total.0))
    }

    pub async fn list_iam_roles(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsIamQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_iam_roles WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(r.*)
            FROM aws_iam_roles r
            WHERE integration_id = $1
            ORDER BY role_name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let roles: Vec<Value> = rows.into_iter().map(|r| r.0).collect();

        Ok((roles, count.0))
    }

    pub async fn list_iam_policies(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsIamQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_iam_policies WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(p.*)
            FROM aws_iam_policies p
            WHERE integration_id = $1
            ORDER BY risk_score DESC, policy_name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let policies: Vec<Value> = rows.into_iter().map(|r| r.0).collect();

        Ok((policies, count.0))
    }

    // ==================== Security Findings ====================

    pub async fn list_findings(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsFindingsQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = vec!["integration_id = $1".to_string()];

        if let Some(ref severity) = query.severity {
            where_clauses.push(format!(
                "severity_label = '{}'",
                severity.replace('\'', "''")
            ));
        }

        if let Some(ref status) = query.workflow_status {
            where_clauses.push(format!(
                "workflow_status = '{}'",
                status.replace('\'', "''")
            ));
        }

        if let Some(ref region) = query.region {
            where_clauses.push(format!("region = '{}'", region.replace('\'', "''")));
        }

        let where_sql = where_clauses.join(" AND ");

        let count: (i64,) = sqlx::query_as(&format!(
            "SELECT COUNT(*) FROM aws_security_findings WHERE {}",
            where_sql
        ))
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(f.*)
            FROM aws_security_findings f
            WHERE integration_id = $1
            ORDER BY severity_normalized DESC, last_observed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let findings: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((findings, count.0))
    }

    pub async fn get_findings_summary(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
    ) -> Result<FindingsSummary, AppError> {
        // By severity
        let severity_counts: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT severity_label, COUNT(*)
            FROM aws_security_findings
            WHERE integration_id = $1 AND record_state = 'ACTIVE'
            GROUP BY severity_label
            ORDER BY
                CASE severity_label
                    WHEN 'CRITICAL' THEN 1
                    WHEN 'HIGH' THEN 2
                    WHEN 'MEDIUM' THEN 3
                    WHEN 'LOW' THEN 4
                    ELSE 5
                END
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // By workflow status
        let workflow_counts: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT workflow_status, COUNT(*)
            FROM aws_security_findings
            WHERE integration_id = $1 AND record_state = 'ACTIVE'
            GROUP BY workflow_status
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // By compliance status
        let compliance_counts: Vec<(Option<String>, i64)> = sqlx::query_as(
            r#"
            SELECT compliance_status, COUNT(*)
            FROM aws_security_findings
            WHERE integration_id = $1 AND record_state = 'ACTIVE'
            GROUP BY compliance_status
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Recent critical findings
        let critical_rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(f.*)
            FROM aws_security_findings f
            WHERE integration_id = $1 AND severity_label = 'CRITICAL' AND record_state = 'ACTIVE'
            ORDER BY last_observed_at DESC
            LIMIT 5
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let critical: Vec<Value> = critical_rows.into_iter().map(|r| r.0).collect();

        Ok(FindingsSummary {
            by_severity: severity_counts
                .into_iter()
                .map(|(s, c)| SeverityCount {
                    severity: s,
                    count: c,
                })
                .collect(),
            by_workflow_status: workflow_counts
                .into_iter()
                .map(|(s, c)| WorkflowCount { status: s, count: c })
                .collect(),
            by_compliance_status: compliance_counts
                .into_iter()
                .map(|(s, c)| ComplianceCount {
                    status: s.unwrap_or_else(|| "UNKNOWN".to_string()),
                    count: c,
                })
                .collect(),
            recent_critical: critical,
        })
    }

    // ==================== Config Rules ====================

    pub async fn list_config_rules(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsConfigRulesQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_config_rules WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(r.*)
            FROM aws_config_rules r
            WHERE integration_id = $1
            ORDER BY
                CASE compliance_type
                    WHEN 'NON_COMPLIANT' THEN 1
                    WHEN 'INSUFFICIENT_DATA' THEN 2
                    ELSE 3
                END,
                config_rule_name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rules: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((rules, count.0))
    }

    // ==================== S3 Buckets ====================

    pub async fn list_s3_buckets(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsResourcesQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_s3_buckets WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(b.*)
            FROM aws_s3_buckets b
            WHERE integration_id = $1
            ORDER BY is_public DESC, encryption_enabled ASC, bucket_name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let buckets: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((buckets, count.0))
    }

    // ==================== EC2 Instances ====================

    pub async fn list_ec2_instances(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsResourcesQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_ec2_instances WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(i.*)
            FROM aws_ec2_instances i
            WHERE integration_id = $1
            ORDER BY is_public DESC, state ASC, instance_id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let instances: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((instances, count.0))
    }

    // ==================== Security Groups ====================

    pub async fn list_security_groups(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsResourcesQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_security_groups WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(sg.*)
            FROM aws_security_groups sg
            WHERE integration_id = $1
            ORDER BY has_risky_rules DESC, group_name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let groups: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((groups, count.0))
    }

    // ==================== RDS Instances ====================

    pub async fn list_rds_instances(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsResourcesQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_rds_instances WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(db.*)
            FROM aws_rds_instances db
            WHERE integration_id = $1
            ORDER BY publicly_accessible DESC, storage_encrypted ASC, db_instance_identifier ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let instances: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((instances, count.0))
    }

    // ==================== CloudTrail ====================

    pub async fn list_cloudtrail_events(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        query: &AwsCloudTrailQuery,
    ) -> Result<(Vec<Value>, i64), AppError> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM aws_cloudtrail_events WHERE integration_id = $1")
                .bind(integration_id)
                .fetch_one(&self.db)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let rows: Vec<(Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(e.*)
            FROM aws_cloudtrail_events e
            WHERE integration_id = $1
            ORDER BY event_time DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(integration_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let events: Vec<Value> = rows.into_iter().map(|r| r.0).collect();
        Ok((events, count.0))
    }

    pub async fn get_cloudtrail_stats(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
    ) -> Result<CloudTrailStats, AppError> {
        let stats: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE is_root_action = true),
                COUNT(*) FILTER (WHERE is_sensitive_action = true),
                COUNT(*) FILTER (WHERE risk_level = 'HIGH' OR risk_level = 'CRITICAL')
            FROM aws_cloudtrail_events
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let by_source: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT event_source, COUNT(*)
            FROM aws_cloudtrail_events
            WHERE integration_id = $1
            GROUP BY event_source
            ORDER BY COUNT(*) DESC
            LIMIT 10
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(CloudTrailStats {
            total_events: stats.0,
            root_events: stats.1,
            sensitive_events: stats.2,
            high_risk_events: stats.3,
            events_by_source: by_source
                .into_iter()
                .map(|(s, c)| EventSourceCount { source: s, count: c })
                .collect(),
        })
    }
}
