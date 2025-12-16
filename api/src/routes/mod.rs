pub mod access_reviews;
pub mod ai;
pub mod analytics;
pub mod assets;
pub mod audits;
pub mod auth;
pub mod aws;
pub mod controls;
pub mod control_test_automation;
pub mod enterprise;
pub mod evidence;
pub mod evidence_automation;
pub mod frameworks;
pub mod health;
pub mod integrations;
pub mod notifications;
pub mod policies;
pub mod policy_templates;
pub mod questionnaires;
pub mod reports;
pub mod risks;
pub mod search;
pub mod soc2;
pub mod sso;
pub mod tasks;
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
        // Vendor portal routes (public, token-based access)
        .route("/api/v1/vendor-portal", get(questionnaires::get_portal_access))
        .route("/api/v1/vendor-portal/response", post(questionnaires::save_portal_response))
        .route("/api/v1/vendor-portal/submit", post(questionnaires::submit_portal_questionnaire))
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
        .route("/api/v1/vendors/:id/documents", get(vendors::list_documents))
        .route("/api/v1/vendors/:id/documents", post(vendors::create_document))
        .route("/api/v1/vendors/:vendor_id/documents/:document_id", get(vendors::get_document))
        .route("/api/v1/vendors/:vendor_id/documents/:document_id", put(vendors::update_document))
        .route("/api/v1/vendors/:vendor_id/documents/:document_id", delete(vendors::delete_document))
        .route("/api/v1/vendors/documents/expiring", get(vendors::get_expiring_documents))
        // Questionnaire routes
        .route("/api/v1/questionnaires/templates", get(questionnaires::list_templates))
        .route("/api/v1/questionnaires/templates", post(questionnaires::create_template))
        .route("/api/v1/questionnaires/templates/:id", get(questionnaires::get_template))
        .route("/api/v1/questionnaires/templates/:id", put(questionnaires::update_template))
        .route("/api/v1/questionnaires/templates/:id", delete(questionnaires::delete_template))
        .route("/api/v1/questionnaires/templates/:id/publish", post(questionnaires::publish_template))
        .route("/api/v1/questionnaires/templates/:template_id/sections", post(questionnaires::create_section))
        .route("/api/v1/questionnaires/templates/:template_id/sections/:section_id", put(questionnaires::update_section))
        .route("/api/v1/questionnaires/templates/:template_id/sections/:section_id", delete(questionnaires::delete_section))
        .route("/api/v1/questionnaires/templates/:template_id/questions", post(questionnaires::create_question))
        .route("/api/v1/questionnaires/templates/:template_id/questions/:question_id", put(questionnaires::update_question))
        .route("/api/v1/questionnaires/templates/:template_id/questions/:question_id", delete(questionnaires::delete_question))
        .route("/api/v1/questionnaires/assignments", get(questionnaires::list_assignments))
        .route("/api/v1/questionnaires/assignments", post(questionnaires::create_assignment))
        .route("/api/v1/questionnaires/assignments/:id", get(questionnaires::get_assignment))
        .route("/api/v1/questionnaires/assignments/:id/review", post(questionnaires::review_assignment))
        .route("/api/v1/questionnaires/assignments/:id", delete(questionnaires::delete_assignment))
        .route("/api/v1/questionnaires/stats", get(questionnaires::get_stats))
        .route("/api/v1/assets", get(assets::list_assets))
        .route("/api/v1/assets/stats", get(assets::get_asset_stats))
        .route("/api/v1/assets/:id", get(assets::get_asset))
        .route("/api/v1/assets", post(assets::create_asset))
        .route("/api/v1/assets/:id", put(assets::update_asset))
        .route("/api/v1/assets/:id", delete(assets::delete_asset))
        .route("/api/v1/assets/:id/controls", post(assets::link_controls))
        .route("/api/v1/assets/:id/controls", delete(assets::unlink_controls))
        .route("/api/v1/assets/discover/:integration_id", post(assets::discover_assets))
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
        .route("/api/v1/audits/:audit_id/findings/:finding_id/remediation-task", post(audits::create_remediation_task))
        .route("/api/v1/audits/:id/evidence-package", get(audits::get_evidence_package))
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
        .route("/api/v1/notifications/process-task-reminders", post(notifications::process_task_reminders))
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
        // SOC 2 Report Parser routes
        .route("/api/v1/soc2/reports", get(soc2::get_report_summaries))
        .route("/api/v1/soc2/reports/:id", get(soc2::get_parsed_report))
        .route("/api/v1/soc2/reports/:id", delete(soc2::delete_parsed_report))
        .route("/api/v1/vendors/:vendor_id/soc2/reports", get(soc2::list_vendor_reports))
        .route("/api/v1/vendors/:vendor_id/documents/:document_id/parse", post(soc2::parse_document))
        .route("/api/v1/vendors/:vendor_id/documents/:document_id/soc2", get(soc2::get_document_report))
        // Access Review routes
        .route("/api/v1/access-reviews/campaigns", get(access_reviews::list_campaigns))
        .route("/api/v1/access-reviews/campaigns", post(access_reviews::create_campaign))
        .route("/api/v1/access-reviews/campaigns/:id", get(access_reviews::get_campaign))
        .route("/api/v1/access-reviews/campaigns/:id", put(access_reviews::update_campaign))
        .route("/api/v1/access-reviews/campaigns/:id", delete(access_reviews::delete_campaign))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/items", get(access_reviews::list_items))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/items", post(access_reviews::add_items))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/items/:item_id", get(access_reviews::get_item))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/items/:item_id/review", post(access_reviews::review_item))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/items/:item_id/removal", post(access_reviews::request_removal))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/bulk-review", post(access_reviews::bulk_review))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/sync", post(access_reviews::sync_from_integration))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/removal-logs", get(access_reviews::get_removal_logs))
        .route("/api/v1/access-reviews/removal-logs/:log_id/complete", post(access_reviews::complete_removal))
        .route("/api/v1/access-reviews/stats", get(access_reviews::get_stats))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/certification", get(access_reviews::get_certification_report))
        .route("/api/v1/access-reviews/campaigns/:campaign_id/certification/csv", get(access_reviews::download_certification_csv))
        // Task routes
        .route("/api/v1/tasks", get(tasks::list_tasks))
        .route("/api/v1/tasks/stats", get(tasks::get_task_stats))
        .route("/api/v1/tasks/my", get(tasks::get_my_tasks))
        .route("/api/v1/tasks/overdue", get(tasks::get_overdue_tasks))
        .route("/api/v1/tasks/recurring", get(tasks::list_recurring_tasks))
        .route("/api/v1/tasks/recurring/process", post(tasks::process_recurring_tasks))
        .route("/api/v1/tasks/:id", get(tasks::get_task))
        .route("/api/v1/tasks", post(tasks::create_task))
        .route("/api/v1/tasks/:id", put(tasks::update_task))
        .route("/api/v1/tasks/:id", delete(tasks::delete_task))
        .route("/api/v1/tasks/:id/complete", post(tasks::complete_task))
        .route("/api/v1/tasks/:id/occurrences", get(tasks::get_task_occurrences))
        .route("/api/v1/tasks/:id/recurrence-history", get(tasks::get_recurrence_history))
        .route("/api/v1/tasks/:id/skip", post(tasks::skip_next_occurrence))
        .route("/api/v1/tasks/:id/pause", post(tasks::pause_recurring_task))
        .route("/api/v1/tasks/:id/resume", post(tasks::resume_recurring_task))
        .route("/api/v1/tasks/:task_id/comments", get(tasks::list_comments))
        .route("/api/v1/tasks/:task_id/comments", post(tasks::add_comment))
        // AI Assistant routes
        .route("/api/v1/ai/config", get(ai::get_configuration))
        .route("/api/v1/ai/config", post(ai::save_configuration))
        .route("/api/v1/ai/toggle", post(ai::toggle_ai))
        .route("/api/v1/ai/stats", get(ai::get_stats))
        // AI Policy Drafting
        .route("/api/v1/ai/policy/draft", post(ai::draft_policy))
        .route("/api/v1/ai/policy/drafts", get(ai::list_policy_drafts))
        .route("/api/v1/ai/policy/drafts/:draft_id/accept", post(ai::accept_policy_draft))
        // AI Evidence Summarization
        .route("/api/v1/ai/evidence/summarize", post(ai::summarize_evidence))
        .route("/api/v1/ai/evidence/:evidence_id/summary", get(ai::get_evidence_summary))
        // AI Gap Analysis
        .route("/api/v1/ai/gap-analysis", post(ai::get_gap_recommendations))
        .route("/api/v1/ai/gap-analysis/:framework_id", get(ai::list_gap_recommendations))
        // AI Risk Scoring
        .route("/api/v1/ai/risk/score", post(ai::suggest_risk_scoring))
        .route("/api/v1/ai/risk/:risk_id/assessment", get(ai::get_risk_assessment))
        .route("/api/v1/ai/risk/:risk_id/assessment/:assessment_id/accept", post(ai::accept_risk_assessment))
        // AI Natural Language Search
        .route("/api/v1/ai/search", post(ai::natural_language_search))
        // AI Audit Preparation
        .route("/api/v1/ai/audit/prepare", post(ai::prepare_audit))
        .route("/api/v1/ai/audit/:audit_id/preparation", get(ai::get_audit_preparation))
        // Analytics - Compliance Trends
        .route("/api/v1/analytics/snapshots", post(analytics::capture_snapshot))
        .route("/api/v1/analytics/snapshots/current", get(analytics::get_current_snapshot))
        .route("/api/v1/analytics/trends", get(analytics::get_compliance_trends))
        // Analytics - Risk Predictions
        .route("/api/v1/analytics/predictions", get(analytics::get_risk_predictions))
        .route("/api/v1/analytics/predictions/summary", get(analytics::get_risk_prediction_summary))
        .route("/api/v1/analytics/predictions/factors", get(analytics::get_prediction_factors))
        .route("/api/v1/analytics/predictions/:risk_id", get(analytics::get_risk_prediction))
        .route("/api/v1/analytics/predictions/:risk_id/recompute", post(analytics::recompute_risk_prediction))
        // Analytics - Benchmarks
        .route("/api/v1/analytics/benchmarks", get(analytics::get_available_benchmarks))
        .route("/api/v1/analytics/benchmarks/comparison", get(analytics::get_latest_benchmark_comparison))
        .route("/api/v1/analytics/benchmarks/:benchmark_id/compare", post(analytics::compare_to_benchmark))
        // Analytics - Custom Reports
        .route("/api/v1/analytics/reports", get(analytics::list_saved_reports))
        .route("/api/v1/analytics/reports", post(analytics::create_saved_report))
        .route("/api/v1/analytics/reports/templates", get(analytics::get_report_templates))
        .route("/api/v1/analytics/reports/:report_id", get(analytics::get_saved_report))
        .route("/api/v1/analytics/reports/:report_id", put(analytics::update_saved_report))
        .route("/api/v1/analytics/reports/:report_id", delete(analytics::delete_saved_report))
        // Analytics - Executive Dashboard
        .route("/api/v1/analytics/executive", get(analytics::get_executive_dashboard))
        .route("/api/v1/analytics/executive/metrics", get(analytics::get_executive_metrics))
        .route("/api/v1/analytics/executive/widgets", get(analytics::get_dashboard_widgets))
        .route("/api/v1/analytics/executive/widgets", post(analytics::create_dashboard_widget))
        .route("/api/v1/analytics/executive/widgets/:widget_id", put(analytics::update_dashboard_widget))
        .route("/api/v1/analytics/executive/widgets/:widget_id", delete(analytics::delete_dashboard_widget))
        // Enterprise Features - Permissions
        .route("/api/v1/permissions", get(enterprise::list_permissions))
        .route("/api/v1/permissions/grouped", get(enterprise::list_permissions_grouped))
        // Enterprise Features - Roles (RBAC)
        .route("/api/v1/roles", get(enterprise::list_roles))
        .route("/api/v1/roles", post(enterprise::create_role))
        .route("/api/v1/roles/:id", get(enterprise::get_role))
        .route("/api/v1/roles/:id", put(enterprise::update_role))
        .route("/api/v1/roles/:id", delete(enterprise::delete_role))
        // Enterprise Features - User Roles
        .route("/api/v1/users/:user_id/roles", get(enterprise::get_user_roles))
        .route("/api/v1/users/:user_id/roles", put(enterprise::assign_user_roles))
        .route("/api/v1/auth/permissions", get(enterprise::get_my_permissions))
        // Enterprise Features - SSO/SAML
        .route("/api/v1/sso/config", get(enterprise::get_sso_configuration))
        .route("/api/v1/sso/config", post(enterprise::create_sso_configuration))
        .route("/api/v1/sso/config", put(enterprise::update_sso_configuration))
        .route("/api/v1/sso/config", delete(enterprise::delete_sso_configuration))
        .route("/api/v1/sso/domains", post(enterprise::add_sso_domain))
        .route("/api/v1/sso/domains/:domain_id/verify", post(enterprise::verify_sso_domain))
        .route("/api/v1/sso/saml/metadata", get(enterprise::get_saml_metadata))
        // Enterprise Features - SCIM
        .route("/api/v1/scim/config", get(enterprise::get_scim_configuration))
        .route("/api/v1/scim/config", post(enterprise::create_scim_configuration))
        .route("/api/v1/scim/config", put(enterprise::update_scim_configuration))
        .route("/api/v1/scim/token", post(enterprise::generate_scim_token))
        .route("/api/v1/scim/token", delete(enterprise::revoke_scim_token))
        // Enterprise Features - Audit Logs
        .route("/api/v1/audit-logs", get(enterprise::list_activity_logs))
        // Enterprise Features - Audit Exports (SIEM)
        .route("/api/v1/audit-exports", get(enterprise::list_audit_export_configurations))
        .route("/api/v1/audit-exports", post(enterprise::create_audit_export_configuration))
        .route("/api/v1/audit-exports/:id", delete(enterprise::delete_audit_export_configuration))
        // Enterprise Features - Branding (White-labeling)
        .route("/api/v1/branding", get(enterprise::get_branding))
        .route("/api/v1/branding", put(enterprise::update_branding))
        .route("/api/v1/branding/domain", post(enterprise::set_custom_domain))
        // Enterprise Features - API Keys
        .route("/api/v1/api-keys", get(enterprise::list_api_keys))
        .route("/api/v1/api-keys", post(enterprise::create_api_key))
        .route("/api/v1/api-keys/:id/revoke", post(enterprise::revoke_api_key))
        // Enterprise Features - Usage & Rate Limiting
        .route("/api/v1/usage/rate-limit", get(enterprise::get_rate_limit_status))
        .route("/api/v1/usage/stats", get(enterprise::get_usage_stats))
        // Enterprise Features - Stats
        .route("/api/v1/enterprise/stats", get(enterprise::get_enterprise_stats))
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
