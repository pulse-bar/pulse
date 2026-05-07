use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{AuthError, AuthResult};
use crate::loopback::LoopbackServer;
use crate::providers::{redirect_uri, OAuthProviderConfig};
use crate::token::TokenSet;

#[derive(Debug, Clone)]
pub struct PkceFlow {
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub provider_id: crate::providers::OAuthProviderId,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub site_id: String,
}

impl PkceFlow {
    pub fn new(
        provider_id: crate::providers::OAuthProviderId,
        client_id: String,
        scopes: Vec<String>,
        site_id: String,
    ) -> Self {
        let code_verifier = random_url_safe(64);
        let code_challenge = challenge(&code_verifier);
        let state = random_url_safe(32);
        Self {
            state,
            code_verifier,
            code_challenge,
            provider_id,
            client_id,
            scopes,
            site_id,
        }
    }
}

#[derive(Clone)]
pub struct OAuthClient {
    http: Client,
    flows: Arc<RwLock<HashMap<String, PkceFlow>>>,
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .user_agent("pulse-bar/1.0")
                .build()
                .expect("reqwest client"),
            flows: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl OAuthClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(
        &self,
        provider: &OAuthProviderConfig,
        client_id: &str,
        site_id: &str,
        scopes_override: Option<Vec<String>>,
    ) -> AuthResult<(String, PkceFlow)> {
        self.build_url(provider, client_id, site_id, scopes_override, redirect_uri())
    }

    fn build_url(
        &self,
        provider: &OAuthProviderConfig,
        client_id: &str,
        site_id: &str,
        scopes_override: Option<Vec<String>>,
        redirect: &str,
    ) -> AuthResult<(String, PkceFlow)> {
        let scopes = scopes_override.unwrap_or_else(|| {
            provider
                .default_scopes
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        });
        let flow = PkceFlow::new(
            provider.id,
            client_id.to_string(),
            scopes.clone(),
            site_id.to_string(),
        );

        let mut url = Url::parse(&provider.authorize_url)
            .map_err(|e| AuthError::OAuth(format!("authorize_url: {e}")))?;
        {
            let mut q = url.query_pairs_mut();
            if let Some(audience) = provider.audience {
                q.append_pair("audience", audience);
            }
            q.append_pair("client_id", client_id);
            q.append_pair("scope", &scopes.join(" "));
            q.append_pair("redirect_uri", redirect);
            q.append_pair("response_type", "code");
            q.append_pair("state", &flow.state);
            if provider.use_pkce {
                q.append_pair("code_challenge", &flow.code_challenge);
                q.append_pair("code_challenge_method", "S256");
            }
            q.append_pair("prompt", "consent");
        }

        self.flows.write().insert(flow.state.clone(), flow.clone());
        Ok((url.into(), flow))
    }

    // The full enterprise UX: spin up a loopback HTTP server, build the
    // authorize URL with that as the redirect, return both. Caller opens
    // the browser to `auth_url` and awaits `await_callback` to resolve.
    pub async fn begin_loopback(
        &self,
        provider: &OAuthProviderConfig,
        client_id: &str,
        site_id: &str,
        scopes_override: Option<Vec<String>>,
    ) -> AuthResult<(String, PkceFlow, LoopbackServer)> {
        let server = LoopbackServer::bind().await?;
        let redirect = server.redirect_uri();
        let (auth_url, flow) =
            self.build_url(provider, client_id, site_id, scopes_override, &redirect)?;
        Ok((auth_url, flow, server))
    }

    pub async fn complete_loopback(
        &self,
        provider: &OAuthProviderConfig,
        flow: &PkceFlow,
        code: &str,
        redirect: &str,
    ) -> AuthResult<TokenSet> {
        // Drop the in-memory flow record before exchanging — guards against
        // accidental replay.
        self.flows.write().remove(&flow.state);
        self.exchange_code(provider, flow, code, redirect).await
    }

    async fn exchange_code(
        &self,
        provider: &OAuthProviderConfig,
        flow: &PkceFlow,
        code: &str,
        redirect: &str,
    ) -> AuthResult<TokenSet> {
        let mut form: Vec<(&str, String)> = vec![
            ("grant_type", "authorization_code".into()),
            ("client_id", flow.client_id.clone()),
            ("code", code.into()),
            ("redirect_uri", redirect.into()),
        ];
        if provider.use_pkce {
            form.push(("code_verifier", flow.code_verifier.clone()));
        }

        let resp = self
            .http
            .post(&provider.token_url)
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|e| AuthError::Transport(format!("{e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::OAuth(format!("status {status}: {body}")));
        }
        let raw: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::OAuth(format!("decode: {e}")))?;
        Ok(TokenSet::from_response(
            raw.access_token,
            raw.refresh_token,
            raw.token_type.unwrap_or_else(|| "Bearer".into()),
            raw.scope,
            raw.expires_in,
        ))
    }

    pub async fn complete(
        &self,
        provider: &OAuthProviderConfig,
        state: &str,
        code: &str,
    ) -> AuthResult<(PkceFlow, TokenSet)> {
        let flow = self
            .flows
            .write()
            .remove(state)
            .ok_or_else(|| AuthError::UnknownFlow(state.into()))?;
        let tokens = self
            .exchange_code(provider, &flow, code, redirect_uri())
            .await?;
        Ok((flow, tokens))
    }

    pub async fn refresh(
        &self,
        provider: &OAuthProviderConfig,
        client_id: &str,
        tokens: &TokenSet,
    ) -> AuthResult<TokenSet> {
        let refresh_token = tokens
            .refresh_token
            .as_deref()
            .ok_or(AuthError::TokenExpired)?;
        let form = [
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ];
        let resp = self
            .http
            .post(&provider.token_url)
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|e| AuthError::Transport(format!("{e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::OAuth(format!("refresh status {status}: {body}")));
        }
        let raw: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::OAuth(format!("decode: {e}")))?;
        Ok(TokenSet::from_response(
            raw.access_token,
            raw.refresh_token.or_else(|| tokens.refresh_token.clone()),
            raw.token_type.unwrap_or_else(|| "Bearer".into()),
            raw.scope.or_else(|| tokens.scope.clone()),
            raw.expires_in,
        ))
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

fn random_url_safe(bytes: usize) -> String {
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::thread_rng();
    (0..bytes)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect()
}

fn challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64_url(&digest)
}

fn base64_url(data: &[u8]) -> String {
    const TABLE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8 | (data[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        out.push(TABLE[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
    } else if rem == 2 {
        let n = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
    }
    out
}
