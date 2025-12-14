pub mod assets;
pub mod audits;
pub mod auth;
pub mod aws;
pub mod controls;
pub mod control_test_automation;
pub mod evidence;
pub mod evidence_automation;
pub mod frameworks;
pub mod health;
pub mod integrations;
pub mod notifications;
pub mod policies;
pub mod policy_templates;
pub mod reports;
pub mod risks;
pub mod search;
pub mod sso;
pub mod vendors;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::middleware::{auth_middleware, logging_middleware, AuthState};
use crate::services::AppServices;

pub fn create_router(services: Arc<AppServices>, auth_state: Arc<AuthState>, cors_origins: Vec<String>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            cors_origins
                .iter()
                .map(|origin| origin.parse().expect("Invalid CORS origin"))
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/v1/policy-templates", get(policy_templates::list_policy_templates))
        .route("/api/v1/policy-templates/search", get(policy_templates::search_policy_templates))
        .route("/api/v1/policy-templates/categories", get(policy_templates::get_template_categories))
        .route("/api/v1/policy-templates/frameworks", get(policy_templates::get_template_frameworks))
        .route("/api/v1/policy-templates/:id", get(policy_templates::get_policy_template))
        // OAuth callback - must be public since it's called by OAuth providers
        .route("/api/v1/integrations/oauth/callback", get(integrations::oauth_callback))
        .with_state(services.clone());

    // SSO routes - no auth middleware (used to establish authentication)
    let sso_routes = Router::new()
        .route("/api/sso/exchange", post(sso::exchange_code))
        .route("/api/sso/userinfo", post(sso::get_userinfo))
        .route("/api/sso/validate", get(sso::validate_sso))
        .route("/api/sso/logout", post(sso::logout_sso))
        .with_state(auth_state.clone());

    let protected_routes = Router::new()
        .route("/api/v1/auth/me", get(auth::me))
        .route("/api/v1/controls", get(controls::list_controls))
        .route("/api/v1/controls/stats", get(controls::get_control_stats))
        .route("/api/v1/controls/:id", get(controls::get_control))
        .route("/api/v1/controls", post(controls::create_control))
        .route("/api/v1/controls/:id", put(controls::update_control))
        .route("/api/v1/controls/:id", delete(controls::delete_control))
        .route("/api/v1/controls/:id/requirements", post(controls::map_requirements))
        .route("/api/v1/controls/:id/requirements", delete(controls::unmap_requirements))
        .route("/api/v1/controls/:id/tests", get(controls::list_control_tests))
        .route("/api/v1/controls/:id/tests", post(controls::create_control_test))
        .route("/api/v1/controls/:control_id/tests/:test_id/results", post(controls::record_test_result))
        .route("/api/v1/evidence", get(evidence::list_evidence))
        .route("/api/v1/evidence/stats", get(evidence::get_evidence_stats))
        .route("/api/v1/evidence/:id", get(evidence::get_evidence))
        .route("/api/v1/evidence", post(evidence::create_evidence))
        .route("/api/v1/evidence/:id", put(evidence::update_evidence))
        .route("/api/v1/evidence/:id", delete(evidence::delete_evidence))
        .route("/api/v1/evidence/:id/controls", post(evidence::link_controls))
        .route("/api/v1/evidence/:id/controls", delete(evidence::unlink_controls))
        .route("/api/v1/evidence/:id/upload-url", post(evidence::get_upload_url))
        .route("/api/v1/evidence/:id/confirm-upload", post(evidence::confirm_upload))
        .route("/api/v1/evidence/:id/download-url", get(evidence::get_download_url))
        .route("/api/v1/evidence/:id/upload", post(evidence::upload_file))
        .route("/api/v1/policies", get(policies::list_policies))
        .route("/api/v1/policies/pending", get(policies::get_pending_policies))
        .route("/api/v1/policies/stats", get(policies::get_policy_stats))
        .route("/api/v1/policies/:id", get(policies::get_policy))
        .route("/api/v1/policies", post(policies::create_policy))
        .route("/api/v1/policies/:id", put(policies::update_policy))
        .route("/api/v1/policies/:id", delete(policies::delete_policy))
        .route("/api/v1/policies/:id/versions", get(policies::get_policy_versions))
        .route("/api/v1/policies/:id/acknowledge", post(policies::acknowledge_policy))
        .route("/api/v1/policies/:id/acknowledgments", get(policies::get_policy_acknowledgments))
        .route("/api/v1/risks", get(risks::list_risks))
        .route("/api/v1/risks/stats", get(risks::get_risk_stats))
        .route("/api/v1/risks/:id", get(risks::get_risk))
        .route("/api/v1/risks", post(risks::create_risk))
        .route("/api/v1/risks/:id", put(risks::update_risk))
        .route("/api/v1/risks/:id", delete(risks::delete_risk))
        .route("/api/v1/risks/:id/controls", post(risks::link_controls))
        .route("/api/v1/risks/:id/controls", delete(risks::unlink_controls))
        .route("/api/v1/risks/heatmap", get(risks::get_risk_heatmap))
        .route("/api/v1/vendors", get(vendors::list_vendors))
        .route("/api/v1/vendors/stats", get(vendors::get_vendor_stats))
        .route("/api/v1/vendors/:id", get(vendors::get_vendor))
        .route("/api/v1/vendors", post(vendors::create_vendor))
        .route("/api/v1/vendors/:id", put(vendors::update_vendor))
        .route("/api/v1/vendors/:id", delete(vendors::delete_vendor))
        .route("/api/v1/vendors/:id/assessments", get(vendors::get_assessments))
        .route("/api/v1/vendors/:id/assessments", post(vendors::create_assessment))
        .route("/api/v1/assets", get(assets::list_assets))
        .route("/api/v1/assets/stats", get(assets::get_asset_stats))
        .route("/api/v1/assets/:id", get(assets::get_asset))
        .route("/api/v1/assets", post(assets::create_asset))
        .route("/api/v1/assets/:id", put(assets::update_asset))
        .route("/api/v1/assets/:id", delete(assets::delete_asset))
        .route("/api/v1/assets/:id/controls", post(assets::link_controls))
        .route("/api/v1/assets/:id/controls", delete(assets::unlink_controls))
        .route("/api/v1/audits", get(audits::list_audits))
        .route("/api/v1/audits/stats", get(audits::get_audit_stats))
        .route("/api/v1/audits/:id", get(audits::get_audit))
        .route("/api/v1/audits", post(audits::create_audit))
        .route("/api/v1/audits/:id", put(audits::update_audit))
        .route("/api/v1/audits/:id", delete(audits::delete_audit))
        .route("/api/v1/audits/:audit_id/requests", get(audits::list_requests))
        .route("/api/v1/audits/:audit_id/requests", post(audits::create_request))
        .route("/api/v1/audits/:audit_id/requests/:request_id/responses", post(audits::add_response))
        .route("/api/v1/audits/:audit_id/findings", get(audits::list_findings))
        .route("/api/v1/audits/:audit_id/findings", post(audits::create_finding))
        .route("/api/v1/audits/:audit_id/findings/:finding_id", put(audits::update_finding))
        .route("/api/v1/integrations", get(integrations::list_integrations))
        .route("/api/v1/integrations/available", get(integrations::list_available_integrations))
        .route("/api/v1/integrations/stats", get(integrations::get_integration_stats))
        .route("/api/v1/integrations/test", post(integrations::test_connection_preview))
        .route("/api/v1/integrations/:id", get(integrations::get_integration))
        .route("/api/v1/integrations", post(integrations::create_integration))
        .route("/api/v1/integrations/:id", put(integrations::update_integration))
        .route("/api/v1/integrations/:id", delete(integrations::delete_integration))
        .route("/api/v1/integrations/:id/test", post(integrations::test_connection))
        .route("/api/v1/integrations/:id/sync", post(integrations::trigger_sync))
        .route("/api/v1/integrations/:id/collect-evidence", post(integrations::collect_evidence))
        .route("/api/v1/integrations/:id/logs", get(integrations::get_sync_logs))
        // OAuth routes
        .route("/api/v1/integrations/oauth/:type/authorize", post(integrations::oauth_authorize))
        .route("/api/v1/integrations/oauth/:type/check", get(integrations::oauth_check))
        .route("/api/v1/integrations/:id/oauth/refresh", post(integrations::oauth_refresh))
        // Integration Health Monitoring
        .route("/api/v1/integrations/health", get(integrations::get_all_health))
        .route("/api/v1/integrations/health/stats", get(integrations::get_health_stats))
        .route("/api/v1/integrations/health/failures", get(integrations::get_recent_failures))
        .route("/api/v1/integrations/health/trend", get(integrations::get_health_trend))
        .route("/api/v1/integrations/:id/health", get(integrations::get_integration_health))
        .route("/api/v1/frameworks", get(frameworks::list_frameworks))
        .route("/api/v1/frameworks/:id", get(frameworks::get_framework))
        .route("/api/v1/frameworks", post(frameworks::create_framework))
        .route("/api/v1/frameworks/:id", put(frameworks::update_framework))
        .route("/api/v1/frameworks/:id", delete(frameworks::delete_framework))
        .route("/api/v1/frameworks/:framework_id/requirements", get(frameworks::list_requirements))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", get(frameworks::get_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements", post(frameworks::create_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements/batch", post(frameworks::batch_create_requirements))
        .route("/api/v1/frameworks/:framework_id/requirements/import", post(frameworks::import_requirements))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", put(frameworks::update_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", delete(frameworks::delete_requirement))
        .route("/api/v1/frameworks/:framework_id/gap-analysis", get(frameworks::get_gap_analysis))
        .route("/api/v1/reports/types", get(reports::list_report_types))
        .route("/api/v1/reports/:report_type/csv", get(reports::generate_csv_report))
        .route("/api/v1/reports/:report_type/pdf", get(reports::generate_pdf_report))
        .route("/api/v1/notifications", get(notifications::list_notifications))
        .route("/api/v1/notifications/count", get(notifications::get_unread_count))
        .route("/api/v1/notifications/read-all", put(notifications::mark_all_as_read))
        .route("/api/v1/notifications/:id/read", put(notifications::mark_as_read))
        .route("/api/v1/search", get(search::search))
        .route("/api/v1/search/status", get(search::search_status))
        .route("/api/v1/search/reindex", post(search::reindex_all))
        // Evidence Automation routes
        .route("/api/v1/evidence/freshness/summary", get(evidence_automation::get_freshness_summary))
        .route("/api/v1/evidence/freshness/stale", get(evidence_automation::get_stale_by_source))
        .route("/api/v1/evidence/freshness/update", post(evidence_automation::update_freshness_scores))
        .route("/api/v1/evidence/freshness/slas", get(evidence_automation::list_freshness_slas))
        .route("/api/v1/evidence/:id/freshness", get(evidence_automation::get_evidence_freshness))
        .route("/api/v1/evidence/collection-tasks", get(evidence_automation::list_collection_tasks))
        .route("/api/v1/evidence/collection-tasks", post(evidence_automation::create_collection_task))
        .route("/api/v1/evidence/collection-tasks/:id", put(evidence_automation::update_collection_task))
        .route("/api/v1/evidence/collection-tasks/:id", delete(evidence_automation::delete_collection_task))
        .route("/api/v1/evidence/changes", get(evidence_automation::list_evidence_changes))
        .route("/api/v1/evidence/changes/count", get(evidence_automation::get_pending_change_count))
        .route("/api/v1/evidence/changes/:id/acknowledge", put(evidence_automation::acknowledge_change))
        .route("/api/v1/evidence/mapping-rules", get(evidence_automation::list_mapping_rules))
        .route("/api/v1/evidence/mapping-rules", post(evidence_automation::create_mapping_rule))
        .route("/api/v1/evidence/mapping-rules/:id", put(evidence_automation::update_mapping_rule))
        .route("/api/v1/evidence/mapping-rules/:id", delete(evidence_automation::delete_mapping_rule))
        // Control Test Automation routes
        .route("/api/v1/control-testing/templates", get(control_test_automation::list_templates))
        .route("/api/v1/control-testing/templates/categories", get(control_test_automation::get_template_categories))
        .route("/api/v1/control-testing/templates/:id", get(control_test_automation::get_template))
        .route("/api/v1/control-testing/tests/:test_id/alert-config", get(control_test_automation::get_alert_config))
        .route("/api/v1/control-testing/tests/:test_id/alert-config", put(control_test_automation::update_alert_config))
        .route("/api/v1/control-testing/tests/:test_id/remediations", get(control_test_automation::list_test_remediations))
        .route("/api/v1/control-testing/tests/:test_id/trigger", post(control_test_automation::trigger_test))
        .route("/api/v1/control-testing/remediations", get(control_test_automation::list_global_remediations))
        .route("/api/v1/control-testing/remediations", post(control_test_automation::create_remediation))
        .route("/api/v1/control-testing/remediations/:id", get(control_test_automation::get_remediation))
        .route("/api/v1/control-testing/monitoring/summary", get(control_test_automation::get_monitoring_summary))
        .route("/api/v1/control-testing/monitoring/controls", get(control_test_automation::list_monitored_controls))
        .route("/api/v1/control-testing/monitoring/controls/:control_id", get(control_test_automation::get_control_monitoring_status))
        .route("/api/v1/control-testing/monitoring/controls/:control_id/acknowledge", put(control_test_automation::acknowledge_control_alert))
        .route("/api/v1/control-testing/runs", get(control_test_automation::list_test_runs))
        .route("/api/v1/control-testing/runs/:run_id", get(control_test_automation::get_test_run))
        .route("/api/v1/control-testing/alerts", get(control_test_automation::list_alerts))
        .route("/api/v1/control-testing/alerts/count", get(control_test_automation::get_alert_count))
        .route("/api/v1/control-testing/alerts/:alert_id/acknowledge", put(control_test_automation::acknowledge_alert))
        .route("/api/v1/control-testing/alerts/:alert_id/resolve", put(control_test_automation::resolve_alert))
        // AWS Integration specific routes
        .route("/api/v1/integrations/:id/aws/overview", get(aws::get_aws_overview))
        .route("/api/v1/integrations/:id/aws/iam/users", get(aws::list_iam_users))
        .route("/api/v1/integrations/:id/aws/iam/roles", get(aws::list_iam_roles))
        .route("/api/v1/integrations/:id/aws/iam/policies", get(aws::list_iam_policies))
        .route("/api/v1/integrations/:id/aws/findings", get(aws::list_findings))
        .route("/api/v1/integrations/:id/aws/findings/summary", get(aws::get_findings_summary))
        .route("/api/v1/integrations/:id/aws/config-rules", get(aws::list_config_rules))
        .route("/api/v1/integrations/:id/aws/s3/buckets", get(aws::list_s3_buckets))
        .route("/api/v1/integrations/:id/aws/ec2/instances", get(aws::list_ec2_instances))
        .route("/api/v1/integrations/:id/aws/ec2/security-groups", get(aws::list_security_groups))
        .route("/api/v1/integrations/:id/aws/rds/instances", get(aws::list_rds_instances))
        .route("/api/v1/integrations/:id/aws/cloudtrail", get(aws::list_cloudtrail_events))
        .route("/api/v1/integrations/:id/aws/cloudtrail/stats", get(aws::get_cloudtrail_stats))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(services.clone());

    Router::new()
        .merge(public_routes)
        .merge(sso_routes)
        .merge(protected_routes)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(logging_middleware))
        .layer(cors)
}
