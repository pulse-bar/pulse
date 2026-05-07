use std::sync::Arc;

use chrono::{Duration, Utc};
use pulse_auth::{Credential, OAuthProviderConfig, OAuthProviderId};
use tauri_plugin_opener::OpenerExt;
use tauri::Emitter;
use pulse_core::{
    ActiveTask, DashboardSummary, DateRange, EnrichmentStatus, JiraSite, OnboardingStatus,
    Settings, TaskMetadata,
};
use pulse_ext_jira as jira_ext;
use pulse_plugins::{PluginInstanceSummary, PluginManifest, PluginStatus};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::state::ShellState;

type Arg<'a> = State<'a, Arc<ShellState>>;

#[tauri::command]
pub async fn get_active_task(state: Arg<'_>) -> Result<ActiveTask, String> {
    state.state.snapshot().map_err(err)
}

#[tauri::command]
pub async fn get_dashboard(state: Arg<'_>, days: Option<u32>) -> Result<DashboardSummary, String> {
    let to = Utc::now();
    let from = to - Duration::days(days.unwrap_or(7).max(1) as i64);
    let db = state.state.db();
    let totals = db.usage_totals(from, to).map_err(err)?;
    let mut tasks = db.usage_by_task(from, to).map_err(err)?;
    let unattributed = tasks
        .iter()
        .find(|t| t.task_id.is_none())
        .map(|t| t.usage.clone())
        .unwrap_or_default();
    tasks.retain(|t| t.task_id.is_some());
    let daily = db.daily_series(from, to).map_err(err)?;
    let model_share = db.model_share(from, to).map_err(err)?;
    Ok(DashboardSummary {
        range: DateRange { from, to },
        totals,
        tasks,
        unattributed,
        daily,
        model_share,
    })
}

#[tauri::command]
pub async fn get_settings(state: Arg<'_>) -> Result<Settings, String> {
    Ok(state.state.settings())
}

#[tauri::command]
pub async fn save_settings(state: Arg<'_>, settings: Settings) -> Result<(), String> {
    state.state.save_settings(settings).map_err(err)
}

#[tauri::command]
pub async fn get_onboarding_status(state: Arg<'_>) -> Result<OnboardingStatus, String> {
    let root = state.primary_watch_root();
    let found = root.as_ref().map(|p| p.exists()).unwrap_or(false);
    Ok(OnboardingStatus {
        claude_dir_found: found && state.claude_code_present(),
        claude_dir_path: root.map(|p| p.to_string_lossy().to_string()),
        sessions_discovered: state.discovered_sessions(),
        ingest_complete: !state.should_show_onboarding(),
    })
}

#[tauri::command]
pub async fn open_dashboard(handle: AppHandle) -> Result<(), String> {
    if let Some(win) = handle.get_webview_window("dashboard") {
        win.show().map_err(err)?;
        win.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_settings(handle: AppHandle) -> Result<(), String> {
    if let Some(win) = handle.get_webview_window("settings") {
        win.show().map_err(err)?;
        win.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reset_database(state: Arg<'_>) -> Result<(), String> {
    state.state.db().reset().map_err(err)
}

#[tauri::command]
pub async fn trigger_full_rescan(state: Arg<'_>) -> Result<u64, String> {
    Ok(state.watcher.full_rescan().await)
}

#[tauri::command]
pub async fn get_task_metadata(
    state: Arg<'_>,
    task_id: String,
) -> Result<Option<TaskMetadata>, String> {
    state.state.db().task_metadata(&task_id).map_err(err)
}

#[tauri::command]
pub async fn get_enrichment_status(state: Arg<'_>) -> Result<EnrichmentStatus, String> {
    Ok(state.enrichment.current_status())
}

#[tauri::command]
pub async fn run_enrichment_now(state: Arg<'_>) -> Result<u64, String> {
    Ok(state.enrichment.run_once().await)
}

#[tauri::command]
pub async fn save_jira_sites(state: Arg<'_>, sites: Vec<JiraSite>) -> Result<(), String> {
    state
        .state
        .update_settings(|s| s.jira.sites = sites)
        .map_err(err)
}

#[tauri::command]
pub async fn upsert_jira_site(state: Arg<'_>, site: JiraSite) -> Result<(), String> {
    state
        .state
        .update_settings(|s| {
            if let Some(idx) = s.jira.sites.iter().position(|x| x.id == site.id) {
                s.jira.sites[idx] = site;
            } else {
                s.jira.sites.push(site);
            }
        })
        .map_err(err)
}

#[tauri::command]
pub async fn delete_jira_site(state: Arg<'_>, site_id: String) -> Result<(), String> {
    state
        .state
        .update_settings(|s| s.jira.sites.retain(|x| x.id != site_id))
        .map_err(err)?;
    let key = jira_ext::site_credential_key(&site_id);
    let _ = state.credentials.delete(&key).await;
    Ok(())
}

#[tauri::command]
pub async fn store_jira_token(
    state: Arg<'_>,
    site_id: String,
    auth_kind: String,
    token: String,
) -> Result<(), String> {
    let key = jira_ext::site_credential_key(&site_id);
    if token.is_empty() {
        return state.credentials.delete(&key).await.map_err(err);
    }
    let cred = match auth_kind.as_str() {
        "bearer" => Credential::Bearer { token },
        "basic" => Credential::Basic { token },
        other => return Err(format!("unsupported auth kind {other} for token storage")),
    };
    state.credentials.store(&key, &cred).await.map_err(err)
}

#[tauri::command]
pub async fn jira_token_present(state: Arg<'_>, site_id: String) -> Result<bool, String> {
    let key = jira_ext::site_credential_key(&site_id);
    Ok(state.credentials.exists(&key).await)
}

#[tauri::command]
pub async fn delete_jira_token(state: Arg<'_>, site_id: String) -> Result<(), String> {
    let key = jira_ext::site_credential_key(&site_id);
    state.credentials.delete(&key).await.map_err(err)
}

#[tauri::command]
pub async fn test_jira_site(state: Arg<'_>, site_id: String) -> Result<(), String> {
    use pulse_enrichment::TaskEnricher;
    let mut settings = state.state.settings();
    settings
        .jira
        .sites
        .iter_mut()
        .for_each(|s| s.enabled = s.id == site_id);
    let enricher = jira_ext::JiraEnricher::new(state.credentials.clone());
    enricher.test(&settings).await.map_err(|e| format!("{e}"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthBeginInput {
    pub provider: OAuthProviderId,
    pub site_id: String,
    pub client_id: String,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthBeginOutput {
    pub authorize_url: String,
    pub state: String,
}

#[tauri::command]
pub async fn oauth_begin(
    state: Arg<'_>,
    input: OAuthBeginInput,
) -> Result<OAuthBeginOutput, String> {
    let provider = OAuthProviderConfig::for_id(input.provider)
        .ok_or_else(|| "unknown OAuth provider".to_string())?;
    let (authorize_url, flow) = state
        .oauth
        .begin(provider, &input.client_id, &input.site_id, input.scopes)
        .map_err(err)?;
    Ok(OAuthBeginOutput {
        authorize_url,
        state: flow.state,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCompleteInput {
    pub provider: OAuthProviderId,
    pub state: String,
    pub code: String,
}

#[tauri::command]
pub async fn list_plugins(state: Arg<'_>) -> Result<Vec<PluginManifest>, String> {
    Ok(state.plugins.manifests())
}

#[tauri::command]
pub async fn list_plugin_statuses(state: Arg<'_>) -> Result<Vec<PluginStatus>, String> {
    let settings = state.state.settings();
    Ok(state
        .plugins
        .statuses(&settings, state.credentials.clone())
        .await)
}

#[tauri::command]
pub async fn list_plugin_instances(
    state: Arg<'_>,
    plugin_id: String,
) -> Result<Vec<PluginInstanceSummary>, String> {
    let settings = state.state.settings();
    state
        .plugins
        .instances(&plugin_id, &settings)
        .map_err(err)
}

#[tauri::command]
pub async fn test_plugin_instance(
    state: Arg<'_>,
    plugin_id: String,
    instance_id: String,
) -> Result<(), String> {
    let settings = state.state.settings();
    let plugin = state
        .plugins
        .get(&plugin_id)
        .ok_or_else(|| format!("unknown plugin {plugin_id}"))?;
    plugin
        .test_instance(&instance_id, &settings, state.credentials.clone())
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn oauth_complete(state: Arg<'_>, input: OAuthCompleteInput) -> Result<(), String> {
    let provider = OAuthProviderConfig::for_id(input.provider)
        .ok_or_else(|| "unknown OAuth provider".to_string())?;
    let (flow, tokens) = state
        .oauth
        .complete(provider, &input.state, &input.code)
        .await
        .map_err(err)?;
    let key = jira_ext::site_credential_key(&flow.site_id);
    state
        .credentials
        .store(&key, &Credential::OAuth2 { tokens })
        .await
        .map_err(err)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectJiraInput {
    pub site_id: String,
    pub client_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectJiraOutput {
    pub authorize_url: String,
    pub redirect_uri: String,
    pub port: u16,
}

// Drives the full OAuth flow: persist the latest config atomically,
// spin up a loopback HTTP server, open the system browser, await the
// callback in a background task, and store the token in the keychain.
#[tauri::command]
pub async fn connect_jira_oauth(
    handle: AppHandle,
    state: Arg<'_>,
    input: ConnectJiraInput,
) -> Result<ConnectJiraOutput, String> {
    if input.client_id.trim().is_empty() {
        return Err("Atlassian OAuth client ID is required. See docs/jira-setup.md.".into());
    }
    let provider = OAuthProviderConfig::for_id(OAuthProviderId::Atlassian)
        .ok_or_else(|| "Atlassian provider not configured".to_string())?;

    // Persist auth_kind + client_id atomically before starting the flow.
    // Returns false if the site_id has never been saved — caller should
    // hit Save first.
    let site_exists = state
        .state
        .update_settings(|s| {
            if let Some(idx) = s.jira.sites.iter().position(|x| x.id == input.site_id) {
                s.jira.sites[idx].auth_kind = pulse_core::JiraAuthKind::OAuth2;
                s.jira.sites[idx].oauth_client_id = Some(input.client_id.clone());
                true
            } else {
                false
            }
        })
        .map_err(err)?;
    if !site_exists {
        return Err(format!(
            "Jira site {} hasn't been saved yet — fill in the form and try again.",
            input.site_id
        ));
    }

    let (auth_url, flow, server) = state
        .oauth
        .begin_loopback(provider, &input.client_id, &input.site_id, None)
        .await
        .map_err(err)?;
    let redirect_uri = server.redirect_uri();
    let port = server.port();

    handle
        .opener()
        .open_url(&auth_url, None::<String>)
        .map_err(|e| format!("could not open browser: {e}"))?;

    let oauth = state.oauth.clone();
    let credentials = state.credentials.clone();
    let provider_clone = provider.clone();
    let flow_clone = flow.clone();
    let site_id = input.site_id.clone();
    let redirect_clone = redirect_uri.clone();
    let app_handle_for_emit = handle.clone();
    tauri::async_runtime::spawn(async move {
        let outcome = async {
            let code = server
                .await_callback(&flow_clone.state)
                .await
                .map_err(|e| format!("{e}"))?;
            let tokens = oauth
                .complete_loopback(&provider_clone, &flow_clone, &code, &redirect_clone)
                .await
                .map_err(|e| format!("{e}"))?;
            let key = pulse_ext_jira::site_credential_key(&site_id);
            credentials
                .store(&key, &Credential::OAuth2 { tokens })
                .await
                .map_err(|e| format!("{e}"))
        }
        .await;
        let payload = serde_json::json!({
            "siteId": site_id,
            "ok": outcome.is_ok(),
            "error": outcome.err(),
        });
        let _ = app_handle_for_emit.emit("pulse://oauth-result", payload);
    });

    Ok(ConnectJiraOutput {
        authorize_url: auth_url,
        redirect_uri,
        port,
    })
}

fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e}")
}
