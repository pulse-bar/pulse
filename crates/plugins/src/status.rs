use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginState {
    NotConnected,
    Connecting,
    Connected,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceState {
    NeedsCredentials,
    Connected,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatus {
    pub instance_id: String,
    pub state: InstanceState,
    pub last_check: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStatus {
    pub plugin_id: String,
    pub state: PluginState,
    pub instances: Vec<InstanceStatus>,
    pub last_check: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl PluginStatus {
    pub fn rollup(plugin_id: &str, instances: Vec<InstanceStatus>) -> Self {
        let any_connected = instances.iter().any(|i| i.state == InstanceState::Connected);
        let any_error = instances.iter().any(|i| i.state == InstanceState::Error);
        let any_needs = instances
            .iter()
            .any(|i| i.state == InstanceState::NeedsCredentials);

        let state = if instances.is_empty() {
            PluginState::NotConnected
        } else if any_connected && !any_error {
            PluginState::Connected
        } else if any_error {
            PluginState::Error
        } else if any_needs {
            PluginState::NotConnected
        } else {
            PluginState::Disabled
        };

        let error = instances
            .iter()
            .find(|i| i.state == InstanceState::Error)
            .and_then(|i| i.error.clone());

        Self {
            plugin_id: plugin_id.to_string(),
            state,
            instances,
            last_check: Some(Utc::now()),
            error,
        }
    }
}
