pub mod aws;
pub mod github;
pub mod jira;
pub mod oauth;
pub mod provider;

pub use aws::AwsProvider;
pub use github::GitHubProvider;
pub use jira::JiraProvider;
pub use oauth::{
    generate_code_verifier, generate_state, OAuthConfig, OAuthProviderEndpoints, OAuthService,
};
pub use provider::{
    CollectedEvidence, IntegrationCapability, IntegrationProvider, IntegrationRegistry,
    SyncContext, SyncResult, TestConnectionDetails,
};
