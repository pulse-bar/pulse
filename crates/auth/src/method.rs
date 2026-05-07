use serde::{Deserialize, Serialize};

use crate::providers::OAuthProviderId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum AuthMethod {
    None,
    Bearer,
    Basic { email: String },
    OAuth2 {
        provider: OAuthProviderId,
        scopes: Vec<String>,
        client_id: Option<String>,
    },
    GitHubApp {
        app_id: String,
        installation_id: String,
    },
}

impl AuthMethod {
    pub fn requires_secret(&self) -> bool {
        !matches!(self, AuthMethod::None)
    }
}

#[derive(Debug, Clone)]
pub struct BasicCredential {
    pub email: String,
    pub token: String,
}
