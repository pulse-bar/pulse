use std::sync::Arc;

use async_trait::async_trait;
use pulse_auth::CredentialStore;
use pulse_core::{JiraSite, Settings, TaskMetadata};
use pulse_enrichment::{EnrichmentError, EnrichmentResult, TaskEnricher};

use crate::client::JiraClient;

pub const ENRICHER_NAME: &str = "jira";

pub fn site_credential_key(site_id: &str) -> String {
    format!("jira:{site_id}")
}

pub struct JiraEnricher {
    client: JiraClient,
    credentials: Arc<dyn CredentialStore>,
}

impl JiraEnricher {
    pub fn new(credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            client: JiraClient::new(),
            credentials,
        }
    }

    pub fn site_for<'a>(task_id: &str, settings: &'a Settings) -> Option<&'a JiraSite> {
        let prefix = task_id.split_once('-').map(|(p, _)| p.to_uppercase())?;
        settings
            .jira
            .sites
            .iter()
            .find(|s| {
                s.enabled
                    && s.project_keys
                        .iter()
                        .any(|k| k.eq_ignore_ascii_case(&prefix))
            })
            .or_else(|| {
                settings
                    .jira
                    .sites
                    .iter()
                    .find(|s| s.enabled && s.project_keys.is_empty())
            })
    }
}

#[async_trait]
impl TaskEnricher for JiraEnricher {
    fn name(&self) -> &'static str {
        ENRICHER_NAME
    }

    fn matches(&self, task_id: &str, settings: &Settings) -> bool {
        Self::site_for(task_id, settings).is_some()
    }

    fn is_configured(&self, settings: &Settings) -> bool {
        settings
            .jira
            .sites
            .iter()
            .any(|s| s.enabled && !s.base_url.is_empty())
    }

    async fn enrich(&self, task_id: &str, settings: &Settings) -> EnrichmentResult<TaskMetadata> {
        let site = Self::site_for(task_id, settings)
            .ok_or_else(|| EnrichmentError::NotConfigured(format!("no Jira site for {task_id}")))?;
        self.client
            .fetch_issue(site, self.credentials.as_ref(), task_id)
            .await
    }

    async fn test(&self, settings: &Settings) -> EnrichmentResult<()> {
        let site = settings
            .jira
            .sites
            .iter()
            .find(|s| s.enabled)
            .ok_or_else(|| EnrichmentError::NotConfigured("no enabled Jira site".into()))?;
        self.client.ping(site, self.credentials.as_ref()).await
    }
}
