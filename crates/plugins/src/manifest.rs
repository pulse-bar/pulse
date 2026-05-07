use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCategory {
    IssueTracking,
    SourceControl,
    Communication,
    Documentation,
    AiProvider,
    Observability,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCapability {
    EnrichTask,
    AttributeTurn,
    IngestTranscript,
    SendNotification,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMethodKind {
    None,
    Pat,
    BasicEmailToken,
    OAuth2Pkce,
    GithubApp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginConnectStyle {
    SingleInstance,
    MultiInstance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub display_name: String,
    pub vendor: String,
    pub description: String,
    pub category: PluginCategory,
    pub capabilities: Vec<PluginCapability>,
    pub auth_methods: Vec<AuthMethodKind>,
    pub preferred_auth: AuthMethodKind,
    pub connect_style: PluginConnectStyle,
    pub icon: String,
    pub docs_url: Option<String>,
}
