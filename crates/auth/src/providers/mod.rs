use serde::{Deserialize, Serialize};

pub mod atlassian;
pub mod github;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum OAuthProviderId {
    Atlassian,
    Github,
    Custom,
}

#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub id: OAuthProviderId,
    pub label: &'static str,
    pub authorize_url: String,
    pub token_url: String,
    pub default_scopes: &'static [&'static str],
    pub audience: Option<&'static str>,
    pub use_pkce: bool,
}

impl OAuthProviderConfig {
    pub fn for_id(id: OAuthProviderId) -> Option<&'static OAuthProviderConfig> {
        match id {
            OAuthProviderId::Atlassian => Some(&*atlassian::CONFIG),
            OAuthProviderId::Github => Some(&*github::CONFIG),
            OAuthProviderId::Custom => None,
        }
    }
}

pub fn redirect_uri() -> &'static str {
    "pulse://oauth/callback"
}
