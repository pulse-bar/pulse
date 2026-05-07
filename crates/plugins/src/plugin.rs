use std::sync::Arc;

use async_trait::async_trait;
use pulse_auth::CredentialStore;
use pulse_core::Settings;
use serde::{Deserialize, Serialize};

use crate::error::PluginResult;
use crate::manifest::PluginManifest;
use crate::status::PluginStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstanceSummary {
    pub instance_id: String,
    pub label: String,
    pub subtitle: Option<String>,
    pub enabled: bool,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;

    fn instances(&self, settings: &Settings) -> Vec<PluginInstanceSummary>;

    async fn status(
        &self,
        settings: &Settings,
        credentials: Arc<dyn CredentialStore>,
    ) -> PluginStatus;

    async fn test_instance(
        &self,
        instance_id: &str,
        settings: &Settings,
        credentials: Arc<dyn CredentialStore>,
    ) -> PluginResult<()>;
}
