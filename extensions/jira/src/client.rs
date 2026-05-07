use std::time::Duration;

use chrono::Utc;
use pulse_auth::{AuthError, Credential, CredentialStore};
use pulse_core::{JiraAuthKind, JiraSite, TaskMetadata};
use pulse_enrichment::{EnrichmentError, EnrichmentResult};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::enricher::site_credential_key;

pub struct JiraClient {
    http: Client,
}

impl JiraClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("pulse-bar/1.0 (+https://github.com/pulse-bar/pulse)")
            .build()
            .expect("reqwest client");
        Self { http }
    }

    pub async fn fetch_issue(
        &self,
        site: &JiraSite,
        store: &dyn CredentialStore,
        key: &str,
    ) -> EnrichmentResult<TaskMetadata> {
        let base = site.base_url.trim_end_matches('/');
        let url = format!(
            "{base}/rest/api/3/issue/{key}?fields=summary,status,assignee,issuetype,priority,project"
        );
        let headers = build_headers(site, store).await?;
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| EnrichmentError::Transport(format!("{e}")))?;

        match resp.status() {
            StatusCode::OK => {
                let body: Value = resp
                    .json()
                    .await
                    .map_err(|e| EnrichmentError::Parse(format!("{e}")))?;
                Ok(parse_issue(site, key, &body))
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(EnrichmentError::Auth(format!("{}", resp.status())))
            }
            StatusCode::NOT_FOUND => Err(EnrichmentError::Other(format!("{key} not found"))),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(30);
                Err(EnrichmentError::RateLimited {
                    retry_after_secs: retry_after,
                })
            }
            other => Err(EnrichmentError::Transport(format!("status {other}"))),
        }
    }

    pub async fn ping(
        &self,
        site: &JiraSite,
        store: &dyn CredentialStore,
    ) -> EnrichmentResult<()> {
        let base = site.base_url.trim_end_matches('/');
        let url = format!("{base}/rest/api/3/myself");
        let headers = build_headers(site, store).await?;
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| EnrichmentError::Transport(format!("{e}")))?;
        match resp.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(EnrichmentError::Auth(format!("{}", resp.status())))
            }
            other => Err(EnrichmentError::Transport(format!("status {other}"))),
        }
    }
}

async fn build_headers(
    site: &JiraSite,
    store: &dyn CredentialStore,
) -> EnrichmentResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("pulse-bar/1.0"));

    if matches!(site.auth_kind, JiraAuthKind::None) {
        return Ok(headers);
    }

    let key = site_credential_key(&site.id);
    let cred = store.read(&key).await.map_err(map_auth_err)?;

    let value = match (site.auth_kind, &cred) {
        (JiraAuthKind::Bearer, Credential::Bearer { token }) => format!("Bearer {token}"),
        (JiraAuthKind::Basic, Credential::Basic { token }) => {
            let email = site.email.as_deref().ok_or_else(|| {
                EnrichmentError::Auth("Basic auth needs an account email".into())
            })?;
            format!("Basic {}", base64_encode(&format!("{email}:{token}")))
        }
        (JiraAuthKind::OAuth2, Credential::OAuth2 { tokens }) => tokens.authorization_header(),
        _ => {
            return Err(EnrichmentError::Auth(
                "stored credential type doesn't match site auth_kind".into(),
            ));
        }
    };

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&value)
            .map_err(|e| EnrichmentError::Auth(format!("invalid header: {e}")))?,
    );
    Ok(headers)
}

fn map_auth_err(err: AuthError) -> EnrichmentError {
    match err {
        AuthError::NotConfigured(s) => EnrichmentError::NotConfigured(s),
        AuthError::Keychain(s) => EnrichmentError::Auth(format!("keychain: {s}")),
        AuthError::Transport(s) => EnrichmentError::Transport(s),
        AuthError::OAuth(s) => EnrichmentError::Auth(format!("oauth: {s}")),
        AuthError::TokenExpired => EnrichmentError::Auth("token expired".into()),
        AuthError::UnknownFlow(s) => EnrichmentError::Other(format!("unknown flow: {s}")),
        AuthError::Other(s) => EnrichmentError::Other(s),
    }
}

fn parse_issue(site: &JiraSite, key: &str, body: &Value) -> TaskMetadata {
    let fields = body.get("fields");
    let title = fields
        .and_then(|f| f.get("summary"))
        .and_then(Value::as_str)
        .map(String::from);
    let status = fields
        .and_then(|f| f.get("status"))
        .and_then(|s| s.get("name"))
        .and_then(Value::as_str)
        .map(String::from);
    let assignee = fields
        .and_then(|f| f.get("assignee"))
        .and_then(|a| a.get("displayName"))
        .and_then(Value::as_str)
        .map(String::from);
    let issue_type = fields
        .and_then(|f| f.get("issuetype"))
        .and_then(|t| t.get("name"))
        .and_then(Value::as_str)
        .map(String::from);
    let priority = fields
        .and_then(|f| f.get("priority"))
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .map(String::from);
    let project_key = fields
        .and_then(|f| f.get("project"))
        .and_then(|p| p.get("key"))
        .and_then(Value::as_str)
        .map(String::from);

    let url = Some(format!(
        "{}/browse/{key}",
        site.base_url.trim_end_matches('/')
    ));

    TaskMetadata {
        task_id: key.to_string(),
        enricher: "jira".into(),
        title,
        status,
        assignee,
        url,
        project_key,
        issue_type,
        priority,
        fetched_at: Utc::now(),
    }
}

fn base64_encode(s: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = (bytes[i] as u32) << 16 | (bytes[i + 1] as u32) << 8 | (bytes[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        out.push(TABLE[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = (bytes[i] as u32) << 16;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = (bytes[i] as u32) << 16 | (bytes[i + 1] as u32) << 8;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}
