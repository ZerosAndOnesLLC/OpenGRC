pub mod assets;
pub mod audits;
pub mod auth;
pub mod controls;
pub mod evidence;
pub mod frameworks;
pub mod health;
pub mod integrations;
pub mod policies;
pub mod risks;
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
        .route("/api/v1/integrations/:id", get(integrations::get_integration))
        .route("/api/v1/integrations", post(integrations::create_integration))
        .route("/api/v1/integrations/:id", put(integrations::update_integration))
        .route("/api/v1/integrations/:id", delete(integrations::delete_integration))
        .route("/api/v1/frameworks", get(frameworks::list_frameworks))
        .route("/api/v1/frameworks/:id", get(frameworks::get_framework))
        .route("/api/v1/frameworks", post(frameworks::create_framework))
        .route("/api/v1/frameworks/:id", put(frameworks::update_framework))
        .route("/api/v1/frameworks/:id", delete(frameworks::delete_framework))
        .route("/api/v1/frameworks/:framework_id/requirements", get(frameworks::list_requirements))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", get(frameworks::get_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements", post(frameworks::create_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements/batch", post(frameworks::batch_create_requirements))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", put(frameworks::update_requirement))
        .route("/api/v1/frameworks/:framework_id/requirements/:id", delete(frameworks::delete_requirement))
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
