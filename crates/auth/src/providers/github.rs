use std::sync::LazyLock;

use crate::providers::{OAuthProviderConfig, OAuthProviderId};

pub static CONFIG: LazyLock<OAuthProviderConfig> = LazyLock::new(|| OAuthProviderConfig {
    id: OAuthProviderId::Github,
    label: "GitHub",
    authorize_url: "https://github.com/login/oauth/authorize".into(),
    token_url: "https://github.com/login/oauth/access_token".into(),
    default_scopes: &["repo", "read:user"],
    audience: None,
    use_pkce: true,
});
