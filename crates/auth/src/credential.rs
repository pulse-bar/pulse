use async_trait::async_trait;
use keyring::Entry;
use serde::{Deserialize, Serialize};

use crate::error::{AuthError, AuthResult};
use crate::token::TokenSet;
use crate::KEYCHAIN_SERVICE;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialKind {
    Bearer,
    Basic,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Credential {
    Bearer { token: String },
    Basic { token: String },
    OAuth2 { tokens: TokenSet },
}

#[async_trait]
pub trait CredentialStore: Send + Sync {
    // `key` is a stable identity for a configured integration (e.g. a
    // Jira-site UUID, a GitHub-org installation ID). Stores must keep
    // the secret out of plaintext settings.
    async fn read(&self, key: &str) -> AuthResult<Credential>;
    async fn store(&self, key: &str, credential: &Credential) -> AuthResult<()>;
    async fn delete(&self, key: &str) -> AuthResult<()>;
    async fn exists(&self, key: &str) -> bool {
        self.read(key).await.is_ok()
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeychainCredentialStore;

impl KeychainCredentialStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(key: &str) -> AuthResult<Entry> {
        Entry::new(KEYCHAIN_SERVICE, key)
            .map_err(|e| AuthError::Keychain(format!("entry: {e}")))
    }
}

#[async_trait]
impl CredentialStore for KeychainCredentialStore {
    async fn read(&self, key: &str) -> AuthResult<Credential> {
        let raw = Self::entry(key)?
            .get_password()
            .map_err(|e| AuthError::Keychain(format!("get: {e}")))?;
        serde_json::from_str(&raw)
            .map_err(|e| AuthError::Other(format!("decode credential: {e}")))
    }

    async fn store(&self, key: &str, credential: &Credential) -> AuthResult<()> {
        let raw = serde_json::to_string(credential)
            .map_err(|e| AuthError::Other(format!("encode credential: {e}")))?;
        Self::entry(key)?
            .set_password(&raw)
            .map_err(|e| AuthError::Keychain(format!("set: {e}")))
    }

    async fn delete(&self, key: &str) -> AuthResult<()> {
        match Self::entry(key)?.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AuthError::Keychain(format!("delete: {e}"))),
        }
    }
}
