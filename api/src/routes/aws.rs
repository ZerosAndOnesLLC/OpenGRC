use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

// ==================== AWS Overview ====================

/// Get AWS integration overview with compliance stats
pub async fn get_aws_overview(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let overview = services.aws.get_overview(org_id, id).await?;
    Ok(Json(json!({ "data": overview })))
}

// ==================== IAM Routes ====================

#[derive(Debug, Deserialize, Default)]
pub struct AwsIamQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub mfa_enabled: Option<bool>,
    pub has_access_keys: Option<bool>,
    pub search: Option<String>,
}

/// List IAM users with MFA/access key status
pub async fn list_iam_users(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsIamQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (users, total) = services.aws.list_iam_users(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": users,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// List IAM roles
pub async fn list_iam_roles(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsIamQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (roles, total) = services.aws.list_iam_roles(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": roles,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// List IAM policies with risk analysis
pub async fn list_iam_policies(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsIamQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (policies, total) = services.aws.list_iam_policies(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": policies,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

// ==================== Security Hub Findings ====================

#[derive(Debug, Deserialize, Default)]
pub struct AwsFindingsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub severity: Option<String>,
    pub workflow_status: Option<String>,
    pub compliance_status: Option<String>,
    pub region: Option<String>,
    pub search: Option<String>,
}

/// List Security Hub findings
pub async fn list_findings(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsFindingsQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (findings, total) = services.aws.list_findings(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": findings,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// Get findings summary by severity
pub async fn get_findings_summary(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let summary = services.aws.get_findings_summary(org_id, id).await?;
    Ok(Json(json!({ "data": summary })))
}

// ==================== AWS Config Rules ====================

#[derive(Debug, Deserialize, Default)]
pub struct AwsConfigRulesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub compliance_type: Option<String>,
    pub region: Option<String>,
    pub search: Option<String>,
}

/// List AWS Config rules and compliance
pub async fn list_config_rules(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsConfigRulesQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (rules, total) = services.aws.list_config_rules(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": rules,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

// ==================== Resources (S3/EC2/RDS) ====================

#[derive(Debug, Deserialize, Default)]
pub struct AwsResourcesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub resource_type: Option<String>,
    pub region: Option<String>,
    pub search: Option<String>,
}

/// List S3 buckets
pub async fn list_s3_buckets(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsResourcesQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (buckets, total) = services.aws.list_s3_buckets(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": buckets,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// List EC2 instances
pub async fn list_ec2_instances(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsResourcesQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (instances, total) = services.aws.list_ec2_instances(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": instances,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// List EC2 security groups
pub async fn list_security_groups(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsResourcesQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (groups, total) = services.aws.list_security_groups(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": groups,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// List RDS instances
pub async fn list_rds_instances(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsResourcesQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (instances, total) = services.aws.list_rds_instances(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": instances,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

// ==================== CloudTrail Events ====================

#[derive(Debug, Deserialize, Default)]
pub struct AwsCloudTrailQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub event_name: Option<String>,
    pub event_source: Option<String>,
    pub user_name: Option<String>,
    pub is_root: Option<bool>,
    pub is_sensitive: Option<bool>,
    pub risk_level: Option<String>,
    pub region: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// List CloudTrail events
pub async fn list_cloudtrail_events(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<AwsCloudTrailQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let (events, total) = services.aws.list_cloudtrail_events(org_id, id, &query).await?;
    Ok(Json(json!({
        "data": events,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// Get CloudTrail event statistics
pub async fn get_cloudtrail_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    verify_aws_integration(&services, org_id, id).await?;

    let stats = services.aws.get_cloudtrail_stats(org_id, id).await?;
    Ok(Json(json!({ "data": stats })))
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

async fn verify_aws_integration(
    services: &AppServices,
    org_id: Uuid,
    integration_id: Uuid,
) -> AppResult<()> {
    let integration = services
        .integration
        .get_integration(org_id, integration_id)
        .await?;

    if integration.integration.integration_type != "aws" {
        return Err(AppError::BadRequest(
            "This endpoint is only available for AWS integrations".to_string(),
        ));
    }

    Ok(())
}
