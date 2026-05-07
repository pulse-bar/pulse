use std::sync::Arc;

use async_trait::async_trait;
use pulse_auth::CredentialStore;
use pulse_core::{JiraSite, Settings};
use pulse_enrichment::TaskEnricher;
use pulse_plugins::{
    AuthMethodKind, InstanceState, InstanceStatus, Plugin, PluginCapability, PluginCategory,
    PluginConnectStyle, PluginError, PluginInstanceSummary, PluginManifest, PluginResult,
    PluginStatus,
};

use crate::enricher::{site_credential_key, JiraEnricher};

pub const PLUGIN_ID: &str = "jira";

pub struct JiraPlugin {
    enricher: JiraEnricher,
    manifest: PluginManifest,
}

impl JiraPlugin {
    pub fn new(credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            enricher: JiraEnricher::new(credentials),
            manifest: manifest(),
        }
    }
}

fn manifest() -> PluginManifest {
    PluginManifest {
        id: PLUGIN_ID.into(),
        display_name: "Jira".into(),
        vendor: "Atlassian".into(),
        description:
            "Resolve task IDs to issue titles, status, assignee, and URL. Multi-site, multi-team."
                .into(),
        category: PluginCategory::IssueTracking,
        capabilities: vec![PluginCapability::EnrichTask],
        auth_methods: vec![
            AuthMethodKind::OAuth2Pkce,
            AuthMethodKind::BasicEmailToken,
            AuthMethodKind::Pat,
        ],
        preferred_auth: AuthMethodKind::OAuth2Pkce,
        connect_style: PluginConnectStyle::MultiInstance,
        icon: "jira".into(),
        docs_url: Some("https://github.com/pulse-bar/pulse/blob/main/docs/oauth.md".into()),
    }
}

#[async_trait]
impl Plugin for JiraPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn instances(&self, settings: &Settings) -> Vec<PluginInstanceSummary> {
        settings
            .jira
            .sites
            .iter()
            .map(|s| PluginInstanceSummary {
                instance_id: s.id.clone(),
                label: s.label.clone(),
                subtitle: short_subtitle(s),
                enabled: s.enabled,
            })
            .collect()
    }

    async fn status(
        &self,
        settings: &Settings,
        credentials: Arc<dyn CredentialStore>,
    ) -> PluginStatus {
        let mut instances = Vec::with_capacity(settings.jira.sites.len());
        for site in &settings.jira.sites {
            let state = compute_instance_state(site, &credentials).await;
            instances.push(state);
        }
        PluginStatus::rollup(PLUGIN_ID, instances)
    }

    async fn test_instance(
        &self,
        instance_id: &str,
        settings: &Settings,
        _credentials: Arc<dyn CredentialStore>,
    ) -> PluginResult<()> {
        let mut overlay = settings.clone();
        let mut found = false;
        for s in overlay.jira.sites.iter_mut() {
            if s.id == instance_id {
                s.enabled = true;
                found = true;
            } else {
                s.enabled = false;
            }
        }
        if !found {
            return Err(PluginError::InstanceNotFound(instance_id.into()));
        }
        self.enricher
            .test(&overlay)
            .await
            .map_err(|e| match e {
                pulse_enrichment::EnrichmentError::Auth(s) => PluginError::Auth(s),
                pulse_enrichment::EnrichmentError::Transport(s) => PluginError::Transport(s),
                pulse_enrichment::EnrichmentError::NotConfigured(s) => PluginError::NotConfigured(s),
                other => PluginError::Other(format!("{other}")),
            })
    }
}

fn short_subtitle(s: &JiraSite) -> Option<String> {
    let mut parts = Vec::new();
    if !s.base_url.is_empty() {
        parts.push(s.base_url.replace("https://", "").replace("http://", ""));
    }
    if !s.project_keys.is_empty() {
        parts.push(format!("[{}]", s.project_keys.join(", ")));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

async fn compute_instance_state(
    site: &JiraSite,
    credentials: &Arc<dyn CredentialStore>,
) -> InstanceStatus {
    let needs_creds = !matches!(site.auth_kind, pulse_core::JiraAuthKind::None)
        && !credentials.exists(&site_credential_key(&site.id)).await;

    let state = if !site.enabled {
        InstanceState::Disabled
    } else if site.base_url.is_empty() {
        InstanceState::NeedsCredentials
    } else if needs_creds {
        InstanceState::NeedsCredentials
    } else {
        InstanceState::Connected
    };

    let error = if state == InstanceState::NeedsCredentials {
        if site.base_url.is_empty() {
            Some("Base URL is missing".into())
        } else {
            Some("Credentials not stored — connect or paste a token".into())
        }
    } else {
        None
    };

    InstanceStatus {
        instance_id: site.id.clone(),
        state,
        last_check: Some(chrono::Utc::now()),
        error,
    }
}
