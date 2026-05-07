use std::sync::Arc;

use parking_lot::RwLock;
use pulse_auth::CredentialStore;
use pulse_core::Settings;

use crate::error::{PluginError, PluginResult};
use crate::manifest::PluginManifest;
use crate::plugin::{Plugin, PluginInstanceSummary};
use crate::status::PluginStatus;

#[derive(Clone, Default)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<Vec<Arc<dyn Plugin>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, plugin: Arc<dyn Plugin>) {
        self.plugins.write().push(plugin);
    }

    pub fn manifests(&self) -> Vec<PluginManifest> {
        self.plugins
            .read()
            .iter()
            .map(|p| p.manifest().clone())
            .collect()
    }

    pub fn get(&self, plugin_id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins
            .read()
            .iter()
            .find(|p| p.manifest().id == plugin_id)
            .cloned()
    }

    pub async fn statuses(
        &self,
        settings: &Settings,
        credentials: Arc<dyn CredentialStore>,
    ) -> Vec<PluginStatus> {
        let plugins = self.plugins.read().clone();
        let mut out = Vec::with_capacity(plugins.len());
        for plugin in plugins {
            out.push(plugin.status(settings, credentials.clone()).await);
        }
        out
    }

    pub fn instances(
        &self,
        plugin_id: &str,
        settings: &Settings,
    ) -> PluginResult<Vec<PluginInstanceSummary>> {
        let plugin = self
            .get(plugin_id)
            .ok_or_else(|| PluginError::Other(format!("unknown plugin {plugin_id}")))?;
        Ok(plugin.instances(settings))
    }
}
