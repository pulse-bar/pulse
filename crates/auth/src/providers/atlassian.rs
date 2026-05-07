use std::sync::LazyLock;

use crate::providers::{OAuthProviderConfig, OAuthProviderId};

pub static CONFIG: LazyLock<OAuthProviderConfig> = LazyLock::new(|| OAuthProviderConfig {
    id: OAuthProviderId::Atlassian,
    label: "Atlassian (Jira / Confluence)",
    authorize_url: "https://auth.atlassian.com/authorize".into(),
    token_url: "https://auth.atlassian.com/oauth/token".into(),
    default_scopes: &["read:jira-user", "read:jira-work", "offline_access"],
    audience: Some("api.atlassian.com"),
    use_pkce: true,
});
