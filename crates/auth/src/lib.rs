pub mod credential;
pub mod error;
pub mod loopback;
pub mod method;
pub mod oauth;
pub mod providers;
pub mod token;

pub use credential::{Credential, CredentialKind, CredentialStore, KeychainCredentialStore};
pub use error::{AuthError, AuthResult};
pub use loopback::{LoopbackServer, LOOPBACK_PORTS};
pub use method::{AuthMethod, BasicCredential};
pub use oauth::{OAuthClient, PkceFlow};
pub use providers::{OAuthProviderConfig, OAuthProviderId};
pub use token::TokenSet;

pub const KEYCHAIN_SERVICE: &str = "dev.pulse.app";
