use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl TokenSet {
    pub fn from_response(
        access_token: String,
        refresh_token: Option<String>,
        token_type: String,
        scope: Option<String>,
        expires_in_secs: Option<u64>,
    ) -> Self {
        let issued_at = Utc::now();
        let expires_at = expires_in_secs
            .map(|s| issued_at + Duration::seconds(s as i64 - 30));
        Self {
            access_token,
            refresh_token,
            token_type,
            scope,
            issued_at,
            expires_at,
        }
    }

    // Tokens are considered expired 30s before their actual expiry to
    // avoid a window where the daemon issues a request right as the
    // server flips the token invalid.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp,
            None => false,
        }
    }

    pub fn authorization_header(&self) -> String {
        let scheme = if self.token_type.is_empty() {
            "Bearer"
        } else {
            self.token_type.as_str()
        };
        format!("{scheme} {}", self.access_token)
    }
}
